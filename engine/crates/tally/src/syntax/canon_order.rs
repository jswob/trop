//! The canonical format's sort keys: what the reality declares (an index's keys, a version's
//! `known`, an item's bindings) and the components rules and version items sort by.
//! Spec: L §2.4 (rules "sorted by label, then by the known date of their v, a rule without v
//! first, then by their own governs"; version items "entries … first, then by target id, then
//! keys in index order …, with a keyless row first, then by the start of their own governs span
//! or their instant"; "the wider key comes first"; GD-28's last key, the item's own canonical
//! line compared as ASCII); L §8.3 (`LOAD-KEY`, `LOAD-NAME`).
//! Twin: engine/tools/refcore/canon.py (`Table`, `key_text`, `key_value`, `first_day`, `Open`,
//! `Tie`, `Keys`, `Rev`, `cmp_comp`, `cmp_keys`, `key_comp`, `key_at`, `span_key`).
//! Invariants: a component compares only with one of its own form (a mismatch is
//! `ENGINE-INVARIANT`, where refcore would crash); a member's key is compared by text only when
//! two are equal, otherwise its order waits for joint models (G9).

use super::parse::Stop;
use super::tree::{Bind, Domain, File, Governs, IndexForm, Instant, Known, Lit, RowKey, Section};
use crate::base::{Code, Load, Place, Refusal};
use crate::num::Num;
use crate::time::Day;
use core::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

/// One sort component: a Python value of refcore's keys, or one of its special components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Val {
    /// A whole number (a rank, a position).
    Int(i64),
    /// A text, compared by code point.
    Text(String),
    /// A day.
    Day(Day),
    /// An exact number.
    Num(Num),
    /// No day: a span's start that is no date.
    Nothing,
    /// A tuple, compared position by position, the shorter first.
    Tup(Vec<Val>),
    /// Sorted in reverse: a range's last key (the wider range first).
    Rev(Box<Val>),
    /// An item's keys as one component (L §2.4, tally 2.1.3), compared position by position.
    Keys(Vec<Val>),
    /// A span's rank and value; what it leaves tied, the next component breaks (GD-28).
    Tie(i64, Vec<Val>),
    /// A key of a member's index or kind, whose order is the member's: its text.
    Member(String),
}

/// How keys at a position sort (refcore's `Table.key_order`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum KeyOrder {
    /// By their position in a list, by text.
    Pos(BTreeMap<String, i64>, String),
    /// By value.
    Value,
    /// By text.
    Text,
    /// By the order of a member's index or kind (G9).
    Member,
}

/// A key as an item writes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum KeyRef {
    /// `*`.
    Star,
    /// One key.
    One(Lit),
    /// A range of keys.
    Range(Lit, Lit),
    /// A place keyed by things, `konto[ana]`: its text.
    Place(String),
}

/// What the reality declares, read by the reprint (refcore's `Table`): the last declaration of
/// an id wins, as a later file's does there.
#[derive(Debug, Clone, Default)]
pub(super) struct Decls {
    /// Each index's form, by id.
    indexes: BTreeMap<String, IndexForm>,
    /// The kinds' ids.
    kinds: BTreeSet<String>,
    /// The bindings of each declaration that has them, by id.
    binds: BTreeMap<String, Vec<Bind>>,
    /// Each version's `known`, by id.
    versions: BTreeMap<String, Known>,
}

impl Decls {
    /// The declarations of every file.
    pub(super) fn of(files: &[File]) -> Decls {
        let mut decls = Decls::default();
        for section in files.iter().flat_map(|file| &file.sections) {
            decls.add(section);
        }
        decls
    }

    /// One section's declaration.
    fn add(&mut self, section: &Section) {
        let (id, binds) = match section {
            Section::Index(ix) => {
                self.indexes.insert(ix.id.clone(), ix.form.clone());
                return;
            }
            Section::Kind(kind) => {
                self.kinds.insert(kind.id.clone());
                return;
            }
            Section::Version(v) => {
                self.versions.insert(v.id.clone(), v.known);
                return;
            }
            Section::Input(x) => (&x.id, &x.binds),
            Section::Def(x) => (&x.id, &x.binds),
            Section::Band(x) => (&x.id, &x.binds),
            Section::Place(x) => (&x.id, &x.binds),
            Section::Fill(x) => (&x.id, &x.binds),
            Section::Move(x) => (&x.id, &x.binds),
            Section::Source(_) | Section::Unit(_) | Section::Ask(_) | Section::Axis(_) => return,
        };
        self.binds.insert(id.clone(), binds.clone());
    }

    /// A version's `known`, by id.
    pub(super) fn known(&self, version: &str) -> Option<Known> {
        self.versions.get(version).copied()
    }

    /// The domain of the binding at `position` of the declaration `target` (unqualified).
    pub(super) fn domain(
        &self,
        qual: Option<&str>,
        target: &str,
        position: usize,
    ) -> Option<&Domain> {
        if qual.is_some() {
            return None;
        }
        self.binds
            .get(target)
            .and_then(|binds| binds.get(position))
            .map(|bind| &bind.domain)
    }

    /// How keys over `domain` sort (L §2.4 "keys in index order (things and an open index's
    /// keys in the text order of their stable ids, §2.5)").
    pub(super) fn key_order(&self, domain: Option<&Domain>) -> KeyOrder {
        match domain {
            None => KeyOrder::Text,
            Some(Domain::List(keys)) => positions(keys, "the list"),
            Some(Domain::Range(_)) => KeyOrder::Value,
            Some(Domain::Name { id, qual: None }) => match self.indexes.get(id) {
                Some(form) => form_order(form, id),
                None if ["days", "months", "years"].contains(&id.as_str()) => KeyOrder::Value,
                None => KeyOrder::Text,
            },
            Some(Domain::Name { id, qual: Some(q) }) => {
                let name = format!("{q}.{id}");
                match self.indexes.get(&name) {
                    Some(form) => form_order(form, &name),
                    None if !self.kinds.contains(&name) => KeyOrder::Member,
                    None => KeyOrder::Text,
                }
            }
        }
    }
}

/// How keys of an index of `form` sort.
fn form_order(form: &IndexForm, id: &str) -> KeyOrder {
    match form {
        IndexForm::List(keys) => positions(keys, id),
        IndexForm::Range { .. } => KeyOrder::Value,
        IndexForm::Open => KeyOrder::Text,
    }
}

/// Keys by their position in `keys`, a later one of the same text winning.
fn positions(keys: &[Lit], what: &str) -> KeyOrder {
    let mut at = BTreeMap::new();
    for (n, key) in (0_i64..).zip(keys) {
        at.insert(key.text(), n);
    }
    KeyOrder::Pos(at, what.to_owned())
}

/// A key's text (refcore's `key_text`).
pub(super) fn key_text(key: &KeyRef) -> String {
    match key {
        KeyRef::Star => "*".to_owned(),
        KeyRef::One(lit) => lit.text(),
        KeyRef::Range(a, b) => format!("{}..{}", a.text(), b.text()),
        KeyRef::Place(text) => text.clone(),
    }
}

/// The first day of a span's end or a version's `known`: a day, a month, a quarter, a year.
pub(super) fn first_day(lit: &Lit) -> Option<Day> {
    match lit {
        Lit::Day(day) => Some(*day),
        Lit::Month(year, month) => Day::from_ymd(*year, *month, 1),
        Lit::Quarter(year, quarter) => {
            let month = quarter.checked_mul(3)?.checked_sub(2)?;
            Day::from_ymd(*year, month, 1)
        }
        Lit::Num(n) if n.whole && n.unit.is_none() => {
            let year = u16::try_from(n.value.to_i64()?).ok()?;
            Day::from_ymd(year, 1, 1)
        }
        Lit::Num(_)
        | Lit::MonthDay(..)
        | Lit::Key(_)
        | Lit::Cur(_)
        | Lit::Bool(_)
        | Lit::Inf { .. }
        | Lit::Nominal(_)
        | Lit::Name(_) => None,
    }
}

/// A version's `known` as its first day: a window by its first.
pub(super) fn known_day(known: Known) -> Day {
    match known {
        Known::Day(day) | Known::Window(day, _) => day,
    }
}

/// A day as a component, or nothing.
fn day_val(day: Option<Day>) -> Val {
    day.map_or(Val::Nothing, Val::Day)
}

/// The span component (L §2.4, round 19, GD-24): a row with no span of its own first, then a
/// span with no start (a row governing always among them), then by start; a row read `after m`
/// after the dated ones; an instant by its day, a day's start before its end. A rule's own
/// span comes as `Governs::Span`.
pub(super) fn span_key(gov: Option<&Governs>, instant: Option<&Instant>) -> Val {
    match (gov, instant) {
        (Some(Governs::Always), _) => Val::Tie(1, Vec::new()),
        (Some(Governs::Span(span)), _) => match &span.from {
            None => Val::Tie(1, Vec::new()),
            Some(from) => Val::Tie(2, vec![day_val(first_day(from))]),
        },
        (None, Some(Instant::After(_))) => Val::Tie(3, Vec::new()),
        (None, Some(Instant::At(day, edge))) => {
            let at = match edge {
                Some(super::tree::Edge::Start) => 0,
                None => 1,
                Some(super::tree::Edge::End) => 2,
            };
            Val::Tie(2, vec![Val::Day(*day), Val::Int(at)])
        }
        (None, None) => Val::Tie(0, Vec::new()),
    }
}

/// A canonical line as the last component, "compared as ASCII" (GD-28).
pub(super) fn ascii_key(text: String) -> Val {
    Val::Tup(vec![Val::Int(9), Val::Text(text)])
}

/// A key's component at one position (L §2.4: "`*` before every other key, and a range before
/// the keys and the ranges it holds"; a place keyed by things is one key, sorted as things);
/// a key its list doesn't hold is `LOAD-KEY` at `place`.
pub(super) fn key_comp(key: &KeyRef, order: &KeyOrder, place: &Place) -> Result<Val, Stop> {
    if *key == KeyRef::Star {
        return Ok(Val::Tup(vec![Val::Tup(vec![Val::Int(-1)])]));
    }
    if *order == KeyOrder::Member {
        return Ok(Val::Member(key_text(key)));
    }
    match key {
        KeyRef::Range(a, b) => Ok(Val::Tup(vec![
            key_at(a, order, place)?,
            Val::Int(0),
            Val::Rev(Box::new(key_at(b, order, place)?)),
        ])),
        KeyRef::Place(text) => Ok(Val::Tup(vec![
            Val::Tup(vec![Val::Int(2), Val::Text(text.clone())]),
            Val::Int(1),
        ])),
        KeyRef::One(lit) => Ok(Val::Tup(vec![key_at(lit, order, place)?, Val::Int(1)])),
        KeyRef::Star => Ok(Val::Tup(vec![Val::Tup(vec![Val::Int(-1)])])),
    }
}

/// One key's place in its order.
fn key_at(key: &Lit, order: &KeyOrder, place: &Place) -> Result<Val, Stop> {
    let text = key.text();
    match order {
        KeyOrder::Pos(at, what) => match at.get(&text) {
            Some(n) => Ok(Val::Tup(vec![Val::Int(0), Val::Int(*n)])),
            None => Err(Stop::Refused(Refusal::at(
                Code::Load(Load::Key),
                format!("{text} is not a key of {what}"),
                place.clone(),
            ))),
        },
        KeyOrder::Value => Ok(key_value(key)),
        KeyOrder::Text | KeyOrder::Member => Ok(Val::Tup(vec![Val::Int(2), Val::Text(text)])),
    }
}

/// A key by value: numbers, then dates, then text.
fn key_value(key: &Lit) -> Val {
    let first = |day: Option<Day>| Val::Tup(vec![Val::Int(1), day_val(day)]);
    match key {
        Lit::Num(n) => Val::Tup(vec![Val::Int(0), Val::Num(n.value.clone())]),
        Lit::Day(day) => first(Some(*day)),
        Lit::Month(..) | Lit::Quarter(..) => first(first_day(key)),
        Lit::MonthDay(..)
        | Lit::Key(_)
        | Lit::Cur(_)
        | Lit::Bool(_)
        | Lit::Inf { .. }
        | Lit::Nominal(_)
        | Lit::Name(_) => Val::Tup(vec![Val::Int(2), Val::Text(key.text())]),
    }
}

/// Compares two components (refcore's `cmp_comp`).
///
/// # Errors
///
/// Two members' keys that differ (their order is G9's); two components of different forms,
/// or numbers past the cap (`ENGINE-INVARIANT`).
pub(super) fn compare(left: &Val, right: &Val) -> Result<Ordering, Stop> {
    if let Val::Member(text) = right {
        return member(left, text);
    }
    let order = match left {
        Val::Member(text) => return member(right, text),
        Val::Int(one) => match right {
            Val::Int(other) => Some(one.cmp(other)),
            Val::Text(_)
            | Val::Day(_)
            | Val::Num(_)
            | Val::Nothing
            | Val::Tup(_)
            | Val::Rev(_)
            | Val::Keys(_)
            | Val::Tie(..)
            | Val::Member(_) => None,
        },
        Val::Text(one) => match right {
            Val::Text(other) => Some(one.cmp(other)),
            Val::Int(_)
            | Val::Day(_)
            | Val::Num(_)
            | Val::Nothing
            | Val::Tup(_)
            | Val::Rev(_)
            | Val::Keys(_)
            | Val::Tie(..)
            | Val::Member(_) => None,
        },
        Val::Day(one) => match right {
            Val::Day(other) => Some(one.number().cmp(&other.number())),
            Val::Int(_)
            | Val::Text(_)
            | Val::Num(_)
            | Val::Nothing
            | Val::Tup(_)
            | Val::Rev(_)
            | Val::Keys(_)
            | Val::Tie(..)
            | Val::Member(_) => None,
        },
        Val::Nothing => match right {
            Val::Nothing => Some(Ordering::Equal),
            Val::Int(_)
            | Val::Text(_)
            | Val::Day(_)
            | Val::Num(_)
            | Val::Tup(_)
            | Val::Rev(_)
            | Val::Keys(_)
            | Val::Tie(..)
            | Val::Member(_) => None,
        },
        Val::Num(one) => match right {
            Val::Num(other) => return one.compare(other).map_err(|_| mismatch()),
            Val::Int(_)
            | Val::Text(_)
            | Val::Day(_)
            | Val::Nothing
            | Val::Tup(_)
            | Val::Rev(_)
            | Val::Keys(_)
            | Val::Tie(..)
            | Val::Member(_) => None,
        },
        Val::Tup(one) | Val::Keys(one) => match right {
            Val::Tup(other) | Val::Keys(other) => return compare_all(one, other),
            Val::Int(_)
            | Val::Text(_)
            | Val::Day(_)
            | Val::Num(_)
            | Val::Nothing
            | Val::Rev(_)
            | Val::Tie(..)
            | Val::Member(_) => None,
        },
        Val::Rev(one) => match right {
            Val::Rev(other) => return compare(other, one),
            Val::Int(_)
            | Val::Text(_)
            | Val::Day(_)
            | Val::Num(_)
            | Val::Nothing
            | Val::Tup(_)
            | Val::Keys(_)
            | Val::Tie(..)
            | Val::Member(_) => None,
        },
        Val::Tie(rank, one) => match right {
            Val::Tie(other_rank, other) => match rank.cmp(other_rank) {
                Ordering::Equal => return compare_all(one, other),
                Ordering::Less => Some(Ordering::Less),
                Ordering::Greater => Some(Ordering::Greater),
            },
            Val::Int(_)
            | Val::Text(_)
            | Val::Day(_)
            | Val::Num(_)
            | Val::Nothing
            | Val::Tup(_)
            | Val::Rev(_)
            | Val::Keys(_)
            | Val::Member(_) => None,
        },
    };
    order.ok_or_else(mismatch)
}

/// A member's key against another component: equal to the same key, otherwise its order is
/// the member's, which waits for joint models (G9).
fn member(other: &Val, text: &str) -> Result<Ordering, Stop> {
    if let Val::Member(same) = other
        && same == text
    {
        return Ok(Ordering::Equal);
    }
    Err(Stop::NotBuilt(
        "the canonical order of keys of a member's index or kind (G9)",
    ))
}

/// Compares two lists of components position by position, the shorter first.
pub(super) fn compare_all(a: &[Val], b: &[Val]) -> Result<Ordering, Stop> {
    for (x, y) in a.iter().zip(b) {
        let order = compare(x, y)?;
        if order != Ordering::Equal {
            return Ok(order);
        }
    }
    Ok(a.len().cmp(&b.len()))
}

/// Two sort components of different forms met.
fn mismatch() -> Stop {
    Stop::Refused(Refusal::new(
        Code::EngineInvariant,
        "two canonical sort keys of different forms".to_owned(),
    ))
}

/// The positions of `keys` sorted by them, equal ones keeping their order (refcore's
/// `stable_sort`).
///
/// # Errors
///
/// The first comparison that fails.
pub(super) fn sorted_by(keys: Vec<Vec<Val>>) -> Result<Vec<usize>, Stop> {
    let mut items: Vec<(Vec<Val>, usize)> = keys.into_iter().zip(0..).collect();
    let mut failed: Option<Stop> = None;
    items.sort_by(|(a, _), (b, _)| match compare_all(a, b) {
        Ok(order) => order,
        Err(stop) => {
            failed.get_or_insert(stop);
            Ordering::Equal
        }
    });
    match failed {
        Some(stop) => Err(stop),
        None => Ok(items.into_iter().map(|(_, at)| at).collect()),
    }
}

/// The key of a row key written in an item (refcore's row key nodes).
pub(super) fn row_key(key: &RowKey) -> KeyRef {
    match key {
        RowKey::Star => KeyRef::Star,
        RowKey::Lit(lit) => KeyRef::One(lit.clone()),
        RowKey::Range(a, b) => KeyRef::Range(a.clone(), b.clone()),
        RowKey::Place(id, keys) => {
            let texts: Vec<String> = keys.iter().map(Lit::text).collect();
            KeyRef::Place(format!("{id}[{}]", texts.join(", ")))
        }
    }
}
