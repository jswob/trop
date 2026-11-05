//! Load's scenario checks: no scenario row of a base item governs a day before the scenario's `from`, and a scenario changes who is alive only from it.
//! Spec: L §6.4 (`from D`: a scenario's rows of the base's items govern D or later; a
//! lifespan it states into a base kind agrees with the base's on every day before D), §8.3
//! (`LOAD-SCENARIO`'s forms `a row of <s> for <x>, an item of the base, governs a day before
//! <from>` and `<s> changes who is alive in <K> on <d>, before <from>`).
//! Twin: engine/tools/refcore/loader.py (`check_scenarios`, `check_scenario_lifespans`).
//! Invariants: the items are walked in refcore's order (defs' givens and rules, bands' tables,
//! inputs' rows and observations, then things' entries); the first barred row refuses.

use super::check::invariant;
use super::steps::refuse;
use crate::base::{Code, Load, Refusal};
use crate::model::{Entry, Reality, RowInstant, RuleVer, Span};
use crate::syntax::Stop;
use crate::time::Day;

/// Whether a span governs a day before `from`: always, open at its start, or starting
/// before it (refcore's `early`).
fn early(span: Option<&Span>, from: Day) -> bool {
    match span {
        None => true,
        Some(span) => span.is_always() || span.start().is_none_or(|start| start < from),
    }
}

/// The scenario that states version `version`, with its `from`, if a scenario does.
fn scenario_of(m: &Reality, version: &str) -> Option<(String, Day)> {
    let scenario = m.scenarios.of(version)?;
    let from = m.scenarios.from.get(scenario)?;
    Some((scenario.to_owned(), *from))
}

/// The file and line of a version load has built.
fn version_place(m: &Reality, version: &str) -> Result<(String, u32), Stop> {
    let info = m
        .versions
        .get(version)
        .ok_or_else(|| invariant(format!("version {version} isn't built")))?;
    Ok((info.file.clone(), info.line))
}

/// `LOAD-SCENARIO`: a scenario's row of a base item governs a day before its `from`, at
/// the row's line in its version's file.
fn barred(m: &Reality, version: &str, scenario: &str, from: Day, what: &str, line: u32) -> Stop {
    let message = format!(
        "a row of {scenario} for {what}, an item of the base, governs a day before {}",
        from.text()
    );
    match version_place(m, version) {
        Ok((file, _)) => refuse(Load::Scenario, message, &file, line),
        Err(stop) => stop,
    }
}

/// The day a version is known from, for the latest-known entry (refcore's `known_first`).
fn known_of(m: &Reality, version: Option<&String>) -> Option<Day> {
    let info = m.versions.get(version?)?;
    Some(info.known)
}

/// The span a version governs.
fn governs(m: &Reality, version: &str) -> Option<Span> {
    m.versions.get(version).and_then(|v| v.governs.clone())
}

/// `LOAD-SCENARIO` for a rule a scenario's version brings into a base def, governing a day
/// before the scenario's `from` (refcore's `check_scenarios`, its rules). L §6.4.
fn rule_barred(m: &Reality, id: &str, label: &str, rule: &RuleVer) -> Result<(), Stop> {
    let Some(version) = &rule.version else {
        return Ok(());
    };
    let Some((scenario, from)) = scenario_of(m, version) else {
        return Ok(());
    };
    let span = rule.span.clone().or_else(|| governs(m, version));
    if early(span.as_ref(), from) {
        let what = format!("{id}.{label}");
        return Err(barred(m, version, &scenario, from, &what, rule.rule.line));
    }
    Ok(())
}

/// No scenario row of an item the base declares governs a day before the scenario's
/// `from`: a def's givens and rules, a band's tables, an input's rows (a reading by its
/// instant) and observations (refcore's `check_scenarios`). L §6.4.
///
/// # Errors
///
/// `LOAD-SCENARIO` at the first such row.
pub(super) fn check_scenarios(m: &Reality) -> Result<(), Stop> {
    for (id, def) in m.defs.entries() {
        if m.scenarios.of(id).is_some() {
            continue;
        }
        for row in &def.givens {
            if let Some((scenario, from)) = scenario_of(m, &row.version) {
                let span = row.own_span.clone().or_else(|| governs(m, &row.version));
                if early(span.as_ref(), from) {
                    return Err(barred(m, &row.version, &scenario, from, id, row.line));
                }
            }
        }
        for (label, versions) in &def.rules {
            for rule in versions {
                rule_barred(m, id, label, rule)?;
            }
        }
    }
    for (id, band) in m.bands.entries() {
        if m.scenarios.of(id).is_some() {
            continue;
        }
        for table in &band.tables {
            if let Some((scenario, from)) = scenario_of(m, &table.version)
                && early(governs(m, &table.version).as_ref(), from)
            {
                // A band table keeps no line of its own: its version's line places it.
                let (_, line) = version_place(m, &table.version)?;
                return Err(barred(m, &table.version, &scenario, from, id, line));
            }
        }
    }
    for (id, input) in m.inputs.entries() {
        if m.scenarios.of(id).is_some() {
            continue;
        }
        for row in &input.rows {
            let Some((scenario, from)) = scenario_of(m, &row.version) else {
                continue;
            };
            if let Some(RowInstant::At(day, _)) = &row.instant {
                if *day < from {
                    return Err(barred(m, &row.version, &scenario, from, id, row.line));
                }
                continue;
            }
            let span = row.own_span.clone().or_else(|| governs(m, &row.version));
            if early(span.as_ref(), from) {
                return Err(barred(m, &row.version, &scenario, from, id, row.line));
            }
        }
        for obs in &input.observations {
            if let Some((scenario, from)) = scenario_of(m, &obs.version)
                && early(governs(m, &obs.version).as_ref(), from)
            {
                return Err(barred(m, &obs.version, &scenario, from, id, obs.line));
            }
        }
    }
    check_scenario_lifespans(m)
}

/// The days of a span before `from`, as its first day (none: from the start of time) and
/// the first day after them; `None` when there are none (refcore's `before`).
fn before(span: Option<&Span>, from: Day) -> Option<(Option<Day>, Day)> {
    let span = span?;
    let lo = if span.is_always() { None } else { span.start() };
    let hi = match span.end() {
        Some(end) if !span.is_always() && end <= from => end,
        Some(_) | None => from,
    };
    if lo.is_some_and(|lo| lo >= hi) {
        return None;
    }
    Some((lo, hi))
}

/// The day before `day`, an invariant past the calendar's first.
fn previous(day: Day) -> Result<Day, Stop> {
    day.previous()
        .ok_or_else(|| invariant("a day before the calendar's first".to_owned()))
}

/// A scenario's entry into a kind the base declares, or a replace of a base thing's entry,
/// changes who is alive only from the scenario's `from`: its lifespan agrees with the one it
/// replaces on every day before (refcore's `check_scenario_lifespans`). L §6.4.
fn check_scenario_lifespans(m: &Reality) -> Result<(), Stop> {
    for (_, thing) in m.things.entries() {
        for (at, entry) in thing.entries.iter().enumerate() {
            let Some(version) = &entry.version else {
                continue;
            };
            let Some((scenario, from)) = scenario_of(m, version) else {
                continue;
            };
            if m.scenarios.of(&entry.kind).is_some() {
                continue;
            }
            let mut bases: Vec<Option<&str>> = vec![None];
            let mut over = m.scenarios.over.get(&scenario).and_then(Option::as_deref);
            while let Some(world) = over {
                if bases.contains(&Some(world)) {
                    break;
                }
                bases.push(Some(world));
                over = m.scenarios.over.get(world).and_then(Option::as_deref);
            }
            let mut old: Option<&Entry> = None;
            for (other_at, other) in thing.entries.iter().enumerate() {
                let in_bases = match &other.version {
                    None => true,
                    Some(v) => bases.contains(&m.scenarios.of(v)),
                };
                if other_at == at || other.kind != entry.kind || !in_bases {
                    continue;
                }
                let newer = match old {
                    None => true,
                    Some(o) => {
                        other.replace
                            && (o.version.is_none()
                                || (other.version.is_some()
                                    && known_of(m, other.version.as_ref())
                                        >= known_of(m, o.version.as_ref())))
                    }
                };
                if newer {
                    old = Some(other);
                }
            }
            let a = old.and_then(|o| before(Some(&o.span), from));
            let b = before(Some(&entry.span), from);
            if a == b {
                continue;
            }
            let day = match (a, b) {
                (Some((alo, ahi)), Some((blo, bhi))) => {
                    if alo == blo {
                        ahi.min(bhi)
                    } else if let (Some(alo), Some(blo)) = (alo, blo) {
                        alo.min(blo)
                    } else if let Some(lo) = alo.or(blo) {
                        previous(lo)?
                    } else {
                        return Err(invariant("two lifespans that differ and agree".to_owned()));
                    }
                }
                (Some((lo, hi)), None) | (None, Some((lo, hi))) => match lo {
                    Some(lo) => lo,
                    None => previous(hi)?,
                },
                (None, None) => continue,
            };
            let message = format!(
                "{scenario} changes who is alive in {} on {}, before {}",
                entry.kind,
                day.text(),
                from.text()
            );
            let Some(line) = entry.line else {
                return Err(Stop::Refused(Refusal::new(
                    Code::Load(Load::Scenario),
                    message,
                )));
            };
            let (file, _) = version_place(m, version)?;
            return Err(refuse(Load::Scenario, message, &file, line));
        }
    }
    Ok(())
}
