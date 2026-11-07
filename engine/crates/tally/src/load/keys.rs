//! Load's keys: a key as written, resolved against the index, kind or list at its position.
//! Spec: L §2.5 (keys by position, their source text), §4.1 (open indexes take the keys rows
//! bring, from the earliest known of the versions that bring them), §5.6 (key lists), §5.7
//! (things), §5.10 (keyed places).
//! Twin: engine/tools/refcore/loader.py (`key_source_text`, `resolve_key_literal`,
//! `range_literal`, `pattern_key`, `key_order`, `resolve_text`).
//! Invariants: a resolved key is a key its domain holds (an open index's once a row brings
//! it); a keyed place inside a key is resolved one level deep, never recursively.

use super::steps::{Loader, refuse};
use super::types::text;
use crate::base::Load;
use crate::eval::Engine;
use crate::model::{BindInfo, Domain, IndexForm, IndexInfo, KeyVal, NameKind, Reality};
use crate::syntax::{Domain as Written, Expr, Key, Lit, Postfix, RowKey, Stop};
use crate::time::{Day, Period, PeriodKind};

/// A key as written at a key position: a literal, a keyed place `konto[ana]`, or an
/// expression, which names no key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum KeyLit {
    /// A literal or a name.
    Lit(Lit),
    /// A keyed place with its keys.
    Place(String, Vec<Lit>),
    /// Any other expression, as written (`if true then ana else ben`, `(ana)`, `2026-02 + 1`):
    /// `LOAD-KEY` at every position (L §8.3; tally 2.1.6's #100, pick P1).
    Written(String),
}

impl KeyLit {
    /// A row's key, unless it is `*` or a range.
    pub(super) fn of_row_key(key: &RowKey) -> Option<KeyLit> {
        match key {
            RowKey::Lit(lit) => Some(KeyLit::Lit(lit.clone())),
            RowKey::Place(name, keys) => Some(KeyLit::Place(name.clone(), keys.clone())),
            RowKey::Star | RowKey::Range(..) => None,
        }
    }

    /// A key written as an expression: a literal, a name, or a keyed place of literals.
    pub(super) fn of_expr(expr: &Expr) -> Option<KeyLit> {
        if let Some(lit) = lit_of_expr(expr) {
            return Some(KeyLit::Lit(lit));
        }
        let Expr::Post(base, posts) = expr else {
            return None;
        };
        let Expr::Name(name) = base.as_ref() else {
            return None;
        };
        let [Postfix::Keys(keys)] = posts.as_slice() else {
            return None;
        };
        let mut lits = Vec::new();
        for key in keys {
            let Key::Expr(inner) = key else {
                return None;
            };
            lits.push(lit_of_expr(inner)?);
        }
        Some(KeyLit::Place(name.clone(), lits))
    }

    /// A key written as an expression: a literal, a name, a keyed place of literals, or any
    /// other expression as written (refcore's `_txt`). L §2.5, §8.3 `LOAD-KEY`.
    pub(super) fn written(expr: &Expr) -> KeyLit {
        KeyLit::of_expr(expr).unwrap_or_else(|| KeyLit::Written(text(expr)))
    }

    /// The key as written, as every message names it: a place with its keys, `konto[ana]`
    /// (L §2.5, tally 2.1.6; pick P1).
    pub(super) fn text(&self) -> String {
        match self {
            KeyLit::Lit(lit) => lit_text(lit),
            KeyLit::Place(name, keys) => {
                let written: Vec<String> = keys.iter().map(lit_text).collect();
                format!("{name}[{}]", written.join(", "))
            }
            KeyLit::Written(written) => written.clone(),
        }
    }
}

/// A literal or a name written as an expression.
pub(super) fn lit_of_expr(expr: &Expr) -> Option<Lit> {
    if let Expr::Lit(lit) = expr {
        return Some(lit.clone());
    }
    if let Expr::Name(name) = expr {
        return Some(Lit::Name(name.clone()));
    }
    None
}

/// A literal's text, as a message names it (refcore's `_lit_text`, `resolve_text`).
pub(super) fn lit_text(lit: &Lit) -> String {
    lit.text()
}

/// A key's source text: the text that names it in its index (refcore's `key_source_text`).
/// L §2.5.
pub(super) fn key_source_text(lit: &Lit, file: &str, line: u32) -> Result<String, Stop> {
    match lit {
        Lit::Num(n) if n.whole && n.unit.is_none() => Ok(n.text.clone()),
        Lit::Name(_)
        | Lit::Key(_)
        | Lit::Cur(_)
        | Lit::Day(_)
        | Lit::Month(..)
        | Lit::Quarter(..)
        | Lit::MonthDay(..) => Ok(lit_text(lit)),
        Lit::Num(_) | Lit::Bool(_) | Lit::Inf { .. } | Lit::Nominal(_) => Err(refuse(
            Load::Key,
            format!("{} is not a key", lit_text(lit)),
            file,
            line,
        )),
    }
}

/// The text a key of an index other than an integer or period one is found by.
fn index_text(lit: &Lit) -> Option<String> {
    match lit {
        Lit::Name(_)
        | Lit::Key(_)
        | Lit::Num(_)
        | Lit::MonthDay(..)
        | Lit::Cur(_)
        | Lit::Day(_)
        | Lit::Month(..)
        | Lit::Quarter(..) => Some(lit_text(lit)),
        Lit::Bool(_) | Lit::Inf { .. } | Lit::Nominal(_) => None,
    }
}

/// A whole number with no unit, as an integer key.
pub(super) fn int_of(lit: &Lit) -> Option<i64> {
    let Lit::Num(n) = lit else {
        return None;
    };
    if n.whole && n.unit.is_none() {
        n.value.to_i64()
    } else {
        None
    }
}

/// The key a literal names at a position whose domain is `domain` (refcore's
/// `resolve_key_literal`). L §2.5.
pub(super) fn resolve_key(
    m: &Reality,
    key: &KeyLit,
    domain: &Domain,
    file: &str,
    line: u32,
) -> Result<KeyVal, Stop> {
    match domain {
        Domain::Index(id) => resolve_index_key(m, key, id, file, line),
        Domain::Kind(kind) => resolve_thing(m, key, kind, file, line),
        Domain::Axis(axis) => resolve_option(m, key, axis, file, line),
        Domain::List(keys) => {
            let KeyLit::Lit(lit) = key else {
                return Err(refuse(
                    Load::Key,
                    format!("{} is not a key of the list", key.text()),
                    file,
                    line,
                ));
            };
            let text = lit_text(lit);
            if !keys.iter().any(|k| lit_text(k) == text) {
                return Err(refuse(
                    Load::Key,
                    format!("{text} is not a key of the list"),
                    file,
                    line,
                ));
            }
            Ok(int_of(lit).map_or_else(
                || KeyVal::Key {
                    index: "[list]".to_owned(),
                    name: text.clone(),
                },
                KeyVal::Int,
            ))
        }
        Domain::Range => range_key_of(key, file, line),
    }
}

/// A key a row writes at a stepped range's position whose ends and step are literals, kept for
/// the check once load is done (refcore's `register_range_lit`). L §2.5.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RangeKey {
    /// The range's ends and step, as written.
    range: Box<(Expr, Expr, Option<Expr>)>,
    /// The key, as written.
    key: Lit,
    /// The row's file and line.
    at: (String, u32),
}

impl RangeKey {
    /// The key `key` written at the position of binding `bind`, at `file:line`; none where the
    /// binding isn't a stepped range whose ends and step are literals, which load can't see
    /// (refcore's `range_is_literal`; its key is checked where it is read). L §2.5.
    pub(super) fn of(bind: &BindInfo, key: &Lit, file: &str, line: u32) -> Option<RangeKey> {
        let Written::Range(range) = &bind.node.domain else {
            return None;
        };
        let literal = literal_end(&range.0)
            && literal_end(&range.1)
            && range.2.as_ref().is_none_or(literal_end);
        literal.then(|| RangeKey {
            range: range.clone(),
            key: key.clone(),
            at: (file.to_owned(), line),
        })
    }
}

/// Whether a range's end or step is a literal: a number (a duration included), a day, a
/// month or a quarter, negated or in parentheses (refcore's `range_is_literal`). L §2.5.
pub(super) fn literal_end(e: &Expr) -> bool {
    let mut node = e;
    while let Expr::Neg(inner) | Expr::Paren(inner) = node {
        node = inner;
    }
    matches!(
        node,
        Expr::Lit(Lit::Num(_) | Lit::Day(_) | Lit::Month(..) | Lit::Quarter(..))
    )
}

/// `LOAD-KEY <k> is not a value of <a> .. <b>[ step <s>]` at its row for the first key rows
/// write at a stepped range's position that the range doesn't hold, once load is done, the
/// keys in the order load met them (refcore's `check_range_lits`, at the first ask; L §2.5
/// makes it a load refusal, so the engine refuses whatever the asks). L §2.5, §8.3 `LOAD-KEY`
/// (tally 2.1.6's #100, pick P2).
///
/// # Errors
///
/// That `LOAD-KEY`; `NotBuilt` for a range of numbers not integers; an invariant.
pub(super) fn check_range_keys(m: &Reality, keys: &[RangeKey]) -> Result<(), Stop> {
    let mut engine = Engine::new(m.on);
    for written in keys {
        if let Some(message) = engine.range_key_missing(m, &written.range, &written.key)? {
            let (file, line) = &written.at;
            return Err(refuse(Load::Key, message, file, *line));
        }
    }
    Ok(())
}

/// The index name a stepped range's key that is a number other than an integer is kept
/// under, by its text, as a key list's keys are under `[list]`: no read finds it, and load
/// checks it against the range's values (refcore's `range_literal`, a `Num`).
pub(super) const RANGE_KEYS: &str = "[range]";

/// A key literal at a stepped range's position: a value as written, a day, a period or a
/// number (refcore's `range_literal`); whether the range holds it is checked once load is done
/// (`check_range_keys`). L §2.5, §5.6.
///
/// # Errors
///
/// `LOAD-KEY <key> is not a value of a stepped range` for a key that is no value, as written
/// (tally 2.1.6's #100, pick P2).
pub(super) fn range_key_of(key: &KeyLit, file: &str, line: u32) -> Result<KeyVal, Stop> {
    let KeyLit::Lit(lit) = key else {
        return Err(not_a_range_value(key, file, line));
    };
    let value = match lit {
        Lit::Day(day) => Some(KeyVal::Day(*day)),
        Lit::Month(year, month) => Period::month(*year, *month).map(KeyVal::Period),
        Lit::Quarter(year, quarter) => Period::quarter(*year, *quarter).map(KeyVal::Period),
        Lit::Num(_) => Some(int_of(lit).map_or_else(
            || KeyVal::Key {
                index: RANGE_KEYS.to_owned(),
                name: lit_text(lit),
            },
            KeyVal::Int,
        )),
        Lit::MonthDay(..)
        | Lit::Key(_)
        | Lit::Cur(_)
        | Lit::Bool(_)
        | Lit::Inf { .. }
        | Lit::Nominal(_)
        | Lit::Name(_) => None,
    };
    value.ok_or_else(|| not_a_range_value(key, file, line))
}

/// `LOAD-KEY <key> is not a value of a stepped range`, the key as written (refcore's
/// `range_literal`; tally 2.1.6's #100, pick P2). L §2.5, §8.3 `LOAD-KEY`.
pub(super) fn not_a_range_value(key: &KeyLit, file: &str, line: u32) -> Stop {
    refuse(
        Load::Key,
        format!("{} is not a value of a stepped range", key.text()),
        file,
        line,
    )
}

/// A key of an index: an integer, a period by its label, or a key by its text.
fn resolve_index_key(
    m: &Reality,
    key: &KeyLit,
    id: &str,
    file: &str,
    line: u32,
) -> Result<KeyVal, Stop> {
    let Some(index) = m.indexes.get(id) else {
        return Err(Stop::NotBuilt("an index that isn't built"));
    };
    let lit = match key {
        KeyLit::Lit(lit) => Some(lit),
        KeyLit::Place(..) | KeyLit::Written(_) => None,
    };
    let text = key.text();
    let not_a_key = || {
        refuse(
            Load::Key,
            format!("{text} is not a key of {id}"),
            file,
            line,
        )
    };
    if let Some(found) = lit.and_then(|l| index_key_of(index, l)) {
        return Ok(found);
    }
    if !index.is_int && index.period.is_none() && index.form == IndexForm::Open {
        return Err(refuse(
            Load::Key,
            format!("{text} is not a key of {id}: no row brings it"),
            file,
            line,
        ));
    }
    Err(not_a_key())
}

/// The key literal `lit` names in `index`: an integer of an integer index, a period of a
/// period index by its label, else a key by its text; none when the index lacks it (refcore's
/// `resolve_key_literal`, an index's position). L §2.5.
#[must_use]
pub fn index_key_of(index: &IndexInfo, lit: &Lit) -> Option<KeyVal> {
    if index.is_int {
        return int_of(lit)
            .map(KeyVal::Int)
            .filter(|v| index.pos(v).is_some());
    }
    if index.period.is_some() {
        return period_key(&index.keys, lit);
    }
    index_text(lit).and_then(|t| index.by_text(&t)).cloned()
}

/// The key of a period index a written date or period names (refcore's label match and
/// `_pk_matches`).
fn period_key(keys: &[KeyVal], lit: &Lit) -> Option<KeyVal> {
    let (label, kind) = match lit {
        Lit::Num(n) if n.whole && n.unit.is_none() && n.four => (n.text.clone(), PeriodKind::Year),
        Lit::Month(..) => (lit_text(lit), PeriodKind::Month),
        Lit::Quarter(..) => (lit_text(lit), PeriodKind::Quarter),
        Lit::Day(_) => (lit_text(lit), PeriodKind::Day),
        Lit::Num(_)
        | Lit::MonthDay(..)
        | Lit::Key(_)
        | Lit::Cur(_)
        | Lit::Bool(_)
        | Lit::Inf { .. }
        | Lit::Nominal(_)
        | Lit::Name(_) => return None,
    };
    keys.iter()
        .find(|k| match k {
            KeyVal::Day(day) => kind == PeriodKind::Day && day.text() == label,
            KeyVal::Period(p) => p.kind() == kind && p.label() == label,
            KeyVal::Key { .. } | KeyVal::Thing(_) | KeyVal::Int(_) | KeyVal::Mv(_) => false,
        })
        .cloned()
}

/// Whether `axis` is a declared axis with the option `text` (refcore's `k in m.axes[ax]`).
/// L §5.5.
pub(super) fn is_option(m: &Reality, axis: &str, text: &str) -> bool {
    let Some(declared) = m.axes.get(axis) else {
        return false;
    };
    declared
        .options
        .iter()
        .any(|(option, _)| option.as_str() == text)
}

/// An axis's option named by a literal (refcore's `resolve_key_literal`, its axis arm).
/// L §5.5.
fn resolve_option(
    m: &Reality,
    key: &KeyLit,
    axis: &str,
    file: &str,
    line: u32,
) -> Result<KeyVal, Stop> {
    let text = match key {
        KeyLit::Lit(lit @ (Lit::Name(_) | Lit::Key(_) | Lit::Num(_))) => lit_text(lit),
        KeyLit::Lit(_) | KeyLit::Place(..) | KeyLit::Written(_) => key.text(),
    };
    let found = is_option(m, axis, &text);
    let named = matches!(key, KeyLit::Lit(Lit::Name(_) | Lit::Key(_) | Lit::Num(_)));
    if found && named {
        return Ok(KeyVal::Key {
            index: axis.to_owned(),
            name: text,
        });
    }
    Err(refuse(
        Load::Key,
        format!("{text} is not a key of {axis}"),
        file,
        line,
    ))
}

/// A thing of a kind: by its name, a place's id, or a keyed place at its keys.
fn resolve_thing(
    m: &Reality,
    key: &KeyLit,
    kind: &str,
    file: &str,
    line: u32,
) -> Result<KeyVal, Stop> {
    match key {
        KeyLit::Place(name, keys) => {
            let keys: Vec<KeyLit> = keys.iter().cloned().map(KeyLit::Lit).collect();
            match keyed_place_thing(m, name, &keys, file, line)? {
                Some(thing) if m.thing_in(&thing, kind) => Ok(KeyVal::Thing(thing)),
                Some(_) | None => Err(refuse(
                    Load::Key,
                    format!("{} is not a key of {kind}", key.text()),
                    file,
                    line,
                )),
            }
        }
        KeyLit::Written(written) => Err(refuse(
            Load::Key,
            format!("{written} is not a key of {kind}"),
            file,
            line,
        )),
        KeyLit::Lit(lit) => {
            let text = match lit {
                Lit::Name(_) | Lit::Key(_) | Lit::Num(_) | Lit::Cur(_) => Some(lit_text(lit)),
                Lit::Day(_)
                | Lit::Month(..)
                | Lit::Quarter(..)
                | Lit::MonthDay(..)
                | Lit::Bool(_)
                | Lit::Inf { .. }
                | Lit::Nominal(_) => None,
            };
            if let Some(name) = text.as_ref()
                && m.thing_in(name, kind)
            {
                return Ok(KeyVal::Thing(name.clone()));
            }
            Err(refuse(
                Load::Key,
                format!(
                    "{} is not a key of {kind}",
                    text.unwrap_or_else(|| lit_text(lit))
                ),
                file,
                line,
            ))
        }
    }
}

/// The thing a keyed place names at its written keys, if the place is keyed. L §5.10.
pub(super) fn keyed_place_thing(
    m: &Reality,
    name: &str,
    keys: &[KeyLit],
    file: &str,
    line: u32,
) -> Result<Option<String>, Stop> {
    if m.names.kind(name) != Some(NameKind::Place) {
        return Ok(None);
    }
    let Some(place) = m.places.get(name) else {
        return Ok(None);
    };
    if place.binds.is_empty() {
        return Ok(None);
    }
    if keys.len() != place.binds.len() {
        return Err(refuse(
            Load::Key,
            format!("wrong number of keys for place {name}"),
            file,
            line,
        ));
    }
    let mut values = Vec::new();
    for (key, bind) in keys.iter().zip(&place.binds) {
        values.push(resolve_key(m, key, &bind.domain, file, line)?);
    }
    Ok(place
        .things
        .iter()
        .find(|(k, _)| *k == values)
        .map(|(_, thing)| thing.clone()))
}

/// A key's position in its domain's order, for a key range's ends (refcore's `key_order`).
/// L §4.6 step 1.
pub(super) fn key_order(m: &Reality, key: &KeyVal, domain: &Domain) -> Option<usize> {
    match domain {
        Domain::Index(id) => m.indexes.get(id).and_then(|ix| ix.pos(key)),
        Domain::List(keys) => {
            let text = key.text();
            keys.iter().position(|k| lit_text(k) == text)
        }
        Domain::Kind(_) | Domain::Axis(_) | Domain::Range => None,
    }
}

impl Loader {
    /// A key at a row's position; a key of an open index a version's row brings is added,
    /// from the earliest known of the versions that bring it (refcore's `pattern_key`). L §4.1.
    pub(super) fn pattern_key(
        &mut self,
        key: &KeyLit,
        bind: &BindInfo,
        version: Option<&str>,
        file: &str,
        line: u32,
    ) -> Result<KeyVal, Stop> {
        if let (Domain::Index(id), Some(by)) = (&bind.domain, version)
            && self
                .m
                .indexes
                .get(id)
                .is_some_and(|ix| ix.form == IndexForm::Open)
        {
            return self.open_key(id, key, by, file, line);
        }
        resolve_key(&self.m, key, &bind.domain, file, line)
    }

    /// The key of an open index a row brings.
    fn open_key(
        &mut self,
        id: &str,
        key: &KeyLit,
        version: &str,
        file: &str,
        line: u32,
    ) -> Result<KeyVal, Stop> {
        let text = match key {
            KeyLit::Lit(lit @ (Lit::Name(_) | Lit::Key(_) | Lit::Num(_) | Lit::Day(_))) => {
                lit_text(lit)
            }
            KeyLit::Lit(
                Lit::Month(..)
                | Lit::Quarter(..)
                | Lit::MonthDay(..)
                | Lit::Cur(_)
                | Lit::Bool(_)
                | Lit::Inf { .. }
                | Lit::Nominal(_),
            )
            | KeyLit::Place(..)
            | KeyLit::Written(_) => {
                return Err(refuse(
                    Load::Key,
                    format!("a key of the open index {id} is a key literal"),
                    file,
                    line,
                ));
            }
        };
        let known = self.m.versions.get(version).map(|v| v.known);
        let Some(index) = self.m.indexes.get(id) else {
            return Err(Stop::NotBuilt("an index that isn't built"));
        };
        if let Some(found) = index.by_text(&text).cloned() {
            let old_known: Option<Day> = index
                .added_by(&found)
                .and_then(|other| self.m.versions.get(other))
                .map(|v| v.known);
            if let (Some(new), Some(old)) = (known, old_known)
                && new < old
                && let Some(held) = self.m.indexes.get_mut(id)
                && let Some(slot) = held.added.iter_mut().find(|(k, _)| *k == found)
            {
                version.clone_into(&mut slot.1);
            }
            return Ok(found);
        }
        let Some(open) = self.m.indexes.get_mut(id) else {
            return Err(Stop::NotBuilt("an index that isn't built"));
        };
        let value = KeyVal::Key {
            index: id.to_owned(),
            name: text.clone(),
        };
        open.texts.push((text, value.clone()));
        open.keys.push(value.clone());
        open.added.push((value.clone(), version.to_owned()));
        open.finish();
        Ok(value)
    }
}
