//! Load's keys: a key as written, resolved against the index, kind or list at its position.
//! Spec: L §2.5 (keys by position, their source text), §4.1 (open indexes take the keys rows
//! bring, from the earliest known of the versions that bring them), §5.6 (key lists), §5.7
//! (things), §5.10 (keyed places).
//! Twin: engine/tools/refcore/loader.py (`key_source_text`, `resolve_key_literal`,
//! `range_literal`, `pattern_key`, `key_order`, `resolve_text`).
//! Invariants: a resolved key is a key its domain holds (an open index's once a row brings
//! it); a keyed place inside a key is resolved one level deep, never recursively.

use super::steps::{Loader, refuse};
use crate::base::Load;
use crate::model::{BindInfo, Domain, IndexForm, KeyVal, NameKind, Reality};
use crate::syntax::{Expr, Key, Lit, Postfix, RowKey, Stop};
use crate::time::{Day, Period, PeriodKind};

/// A key as written at a key position: a literal, or a keyed place `konto[ana]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum KeyLit {
    /// A literal or a name.
    Lit(Lit),
    /// A keyed place with its keys.
    Place(String, Vec<Lit>),
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

    /// The key as its message names it.
    pub(super) fn text(&self) -> String {
        match self {
            KeyLit::Lit(lit) => lit_text(lit),
            KeyLit::Place(name, _) => name.clone(),
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
    match lit {
        Lit::Num(n) => n.text.clone(),
        Lit::Day(day) => day.text(),
        Lit::Month(year, month) => format!("{year:04}-{month:02}"),
        Lit::Quarter(year, quarter) => format!("{year:04}-Q{quarter}"),
        Lit::MonthDay(month, day) => format!("{month:02}-{day:02}"),
        Lit::Key(text) | Lit::Cur(text) | Lit::Name(text) => text.clone(),
        Lit::Bool(value) => value.to_string(),
        Lit::Inf { neg } => if *neg { "-inf" } else { "inf" }.to_owned(),
        Lit::Nominal(_) => "nominal".to_owned(),
    }
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
        Domain::Axis(_) => Err(Stop::NotBuilt("axes")),
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
        Domain::Range => range_key_of(key),
    }
}

/// A key literal at a stepped range's position: the value as written (refcore's
/// `range_literal`); its membership is checked at run. L §2.5.
pub(super) fn range_key_of(key: &KeyLit) -> Result<KeyVal, Stop> {
    let not_built = Stop::NotBuilt("a stepped range's key of that kind");
    let KeyLit::Lit(lit) = key else {
        return Err(not_built);
    };
    let value = match lit {
        Lit::Day(day) => Some(KeyVal::Day(*day)),
        Lit::Month(year, month) => Period::month(*year, *month).map(KeyVal::Period),
        Lit::Quarter(year, quarter) => Period::quarter(*year, *quarter).map(KeyVal::Period),
        Lit::Num(_)
        | Lit::MonthDay(..)
        | Lit::Key(_)
        | Lit::Cur(_)
        | Lit::Bool(_)
        | Lit::Inf { .. }
        | Lit::Nominal(_)
        | Lit::Name(_) => int_of(lit).map(KeyVal::Int),
    };
    value.ok_or(not_built)
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
        KeyLit::Place(..) => None,
    };
    let text = lit.map_or_else(|| "placekey".to_owned(), lit_text);
    let not_a_key = || {
        refuse(
            Load::Key,
            format!("{text} is not a key of {id}"),
            file,
            line,
        )
    };
    if index.is_int {
        let value = lit.and_then(int_of).map(KeyVal::Int);
        return value
            .filter(|v| index.pos(v).is_some())
            .ok_or_else(not_a_key);
    }
    if index.period.is_some() {
        let found = lit.and_then(|l| period_key(&index.keys, l));
        return found.ok_or_else(not_a_key);
    }
    if let Some(found) = lit.and_then(index_text).and_then(|t| index.by_text(&t)) {
        return Ok(found.clone());
    }
    if index.form == IndexForm::Open {
        return Err(refuse(
            Load::Key,
            format!("{text} is not a key of {id}: no row brings it"),
            file,
            line,
        ));
    }
    Err(not_a_key())
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
            KeyVal::Key { .. } | KeyVal::Thing(_) | KeyVal::Int(_) => false,
        })
        .cloned()
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
            let found = keyed_place_thing(m, name, keys, file, line)?;
            match found {
                Some(thing) if m.thing_in(&thing, kind) => Ok(KeyVal::Thing(thing)),
                Some(_) | None => Err(refuse(
                    Load::Key,
                    format!("{name}[...] is not a key of {kind}"),
                    file,
                    line,
                )),
            }
        }
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
    keys: &[Lit],
    file: &str,
    line: u32,
) -> Result<Option<String>, Stop> {
    if m.names.get(name) != Some(&NameKind::Place) {
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
    for (lit, bind) in keys.iter().zip(&place.binds) {
        values.push(resolve_key(
            m,
            &KeyLit::Lit(lit.clone()),
            &bind.domain,
            file,
            line,
        )?);
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
            | KeyLit::Place(..) => {
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
