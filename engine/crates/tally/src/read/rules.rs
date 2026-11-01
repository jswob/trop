//! The read rule for dated rules: which version of a def's rule is in force at the law date.
//! Spec: L §4.6 (one rule for rows, givens, band rows and dated rules: replacement, ties),
//! §5.11 step 2 (the versions of one rule that exist at d and K), §4.3 (a rule read with no
//! law date).
//! Twin: engine/tools/refcore/evaluate.py (`rule_exists`, `rule_in_force`), model.py
//! (`RuleVer.known`).
//! Invariants: a label's versions are scanned in the order versions state them; two
//! `replace` versions known the same day both in force are refused, never broken by order.

use super::rows::{no_law_date, version_known, version_span};
use crate::base::{Code, Load, Refusal};
use crate::model::{Reality, RuleVer, Span};
use crate::syntax::Stop;
use crate::time::Day;

/// Whether a span holds the law date; a read with no law date only where it is `always`
/// (L §4.3 rules 1–2, §5.11 step 2).
fn holds(span: &Span, law: Option<Day>) -> Result<bool, Stop> {
    match law {
        Some(day) => Ok(span.contains(day)),
        None if span.is_always() => Ok(true),
        None => Err(no_law_date()),
    }
}

/// Whether a version of a rule exists at law date `law` under cut `cut`: its version (the
/// `replace rule` line's, else `v V`'s) is known by the cut and governs the law date, and
/// so does its own span (L §5.11 step 2, §4.6 step 1).
fn rule_exists(m: &Reality, rule: &RuleVer, law: Option<Day>, cut: Day) -> Result<bool, Stop> {
    if let Some(version) = &rule.version {
        if version_known(m, version)? > cut {
            return Ok(false);
        }
        let span = match &rule.span {
            Some(own) => Some(own.clone()),
            None => version_span(m, version, cut)?,
        };
        let Some(span) = span else {
            return Ok(false);
        };
        return holds(&span, law);
    }
    let mut exists = true;
    if let Some(v) = &rule.v {
        if version_known(m, v)? > cut {
            return Ok(false);
        }
        let Some(span) = version_span(m, v, cut)? else {
            return Ok(false);
        };
        exists = holds(&span, law)?;
    }
    if exists && let Some(own) = &rule.span {
        let day = law.ok_or_else(no_law_date)?;
        exists = own.contains(day);
    }
    Ok(exists)
}

/// A rule version's `known`: its `replace rule` version's, else its `v` version's (L §4.5).
fn rule_known(m: &Reality, rule: &RuleVer) -> Result<Option<Day>, Stop> {
    match (&rule.version, &rule.v) {
        (Some(version), _) | (None, Some(version)) => version_known(m, version).map(Some),
        (None, None) => Ok(None),
    }
}

/// The version of rule `label` of def `def` in force at law date `law` under cut `cut`, by
/// its position among `versions`: the latest-known `replace` among those that exist, else
/// the one that exists; none where none does (L §5.11 step 2, §4.6 steps 2 and 4).
///
/// # Errors
///
/// `LOAD-OVERLAP` for two `replace` versions known the same day, `LOAD-LABEL` for two
/// versions of the label in force with no `replace`; `NotBuilt` for a dated rule read with
/// no law date (`LOAD-LAWDATE`).
pub fn rule_in_force(
    m: &Reality,
    def: &str,
    label: &str,
    versions: &[RuleVer],
    law: Option<Day>,
    cut: Day,
) -> Result<Option<usize>, Stop> {
    let mut live = Vec::new();
    let mut replacing = Vec::new();
    for (at, rule) in versions.iter().enumerate() {
        if !rule_exists(m, rule, law, cut)? {
            continue;
        }
        live.push(at);
        if rule.replace {
            replacing.push((at, rule_known(m, rule)?));
        }
    }
    let latest = replacing.iter().map(|(_, known)| *known).max();
    if let Some(known) = latest {
        let same: Vec<usize> = replacing
            .iter()
            .filter(|(_, k)| *k == known)
            .map(|(at, _)| *at)
            .collect();
        if let [_, _, ..] = same.as_slice() {
            return Err(overlap(def, label, versions, &same));
        }
        return Ok(same.first().copied());
    }
    if let [_, _, ..] = live.as_slice() {
        let message = format!("rule {label} already exists in {def}");
        return Err(Stop::Refused(Refusal::new(
            Code::Load(Load::Label),
            message,
        )));
    }
    Ok(live.first().copied())
}

/// `LOAD-OVERLAP` for `replace` versions of one rule known the same day, their versions
/// named in text order (L §4.6 steps 2 and 4).
fn overlap(def: &str, label: &str, versions: &[RuleVer], same: &[usize]) -> Stop {
    let mut ids: Vec<String> = Vec::new();
    for &at in same {
        let id = versions
            .get(at)
            .and_then(|rule| rule.version.clone())
            .unwrap_or_else(|| "?".to_owned());
        ids.push(id);
    }
    ids.sort();
    let message = format!(
        "rules {def}.{label} of {} overlap, neither replacing the other",
        ids.join(" and ")
    );
    Stop::Refused(Refusal::new(Code::Load(Load::Overlap), message))
}
