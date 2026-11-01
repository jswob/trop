//! The gap rule and the read of an input: the row a read takes, `none` outside a lifespan, or the question of the hole a read with no row is.
//! Spec: L §4.6 step 5 (no candidate: `HOLE-UNGOVERNED`'s questions, outside a lifespan),
//! Readings (a plain read of a reading, `HOLE-NOREADING`), §4.2 (a calendar's question),
//! §5.7 (lifespans and memberships), §8.1.
//! Twin: engine/tools/refcore/evaluate.py (`origin`, `no_candidate`, `read_input`,
//! `read_reading`), model.py (`member_span`).
//! Invariants: a question is text and the item it is asked of; the hole itself is built in
//! the hole module (R-116). A thing stated `same` belongs to a joint model, which S doesn't
//! build.

use std::collections::BTreeSet;

use super::rows::{Clocks, Point, Test, candidates, choose, invariant, need_law, time_point};
use crate::model::{Domain, InputInfo, KeyVal, Reality, Span, Thing};
use crate::syntax::Stop;
use crate::time::Day;

/// A hole's question and the figure it is asked of, `pay[ana, 2026-01]` (L §8.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    /// The question.
    pub text: String,
    /// The figure: the item at its keys.
    pub origin: String,
}

/// What a read of an input finds (L §4.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Found {
    /// The row it takes, by its position among the input's rows.
    Row(usize),
    /// `none`: no row, outside a lifespan (L §4.6 step 5).
    None,
    /// A hole: no row, with its question (`HOLE-UNGOVERNED`, `HOLE-NOREADING`).
    Hole(Question),
}

/// How an input is read: plainly, or as the added days of a calendar (L §4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Asked {
    /// A plain read.
    Plain,
    /// The days a `calendar(… added x)` reads.
    Calendar,
}

/// The figure a read is of: the item, with its keys in brackets (L §8.1, §2.4).
#[must_use]
pub fn origin(id: &str, keys: &[KeyVal]) -> String {
    if keys.is_empty() {
        return id.to_owned();
    }
    format!("{id}[{}]", keys_text(keys))
}

/// Keys as a question names them, `ana, 2026-01` (L §4.6 step 5).
fn keys_text(keys: &[KeyVal]) -> String {
    keys.iter().map(KeyVal::text).collect::<Vec<_>>().join(", ")
}

/// The span over which `thing` is of `kind` or a subkind under cut `cut`: its one
/// membership there, or the hull of several (L §5.7).
#[must_use]
pub fn member_span(m: &Reality, thing: &Thing, kind: &str, cut: Day) -> Option<Span> {
    let subtree = m.subtree(kind);
    let kinds: BTreeSet<&str> = thing
        .entries
        .iter()
        .filter(|e| subtree.contains(&e.kind))
        .map(|e| e.kind.as_str())
        .collect();
    let spans: Vec<Span> = kinds
        .iter()
        .filter_map(|k| m.span_in(thing, k, cut))
        .collect();
    if let [one] = spans.as_slice() {
        return Some(one.clone());
    }
    if spans.is_empty() {
        return None;
    }
    // the hull: open where any one is open
    let starts: Option<Vec<Day>> = spans.iter().map(Span::start).collect();
    let ends: Option<Vec<Day>> = spans.iter().map(Span::end).collect();
    let start = starts.and_then(|days| days.into_iter().min());
    let end = ends.and_then(|days| days.into_iter().max());
    Some(Span::between(start, end))
}

/// Whether a key names a thing outside its lifespan on `day`, so a read with no row is
/// `none` (L §4.6 step 5, Outside a lifespan).
fn outside_lifespan(m: &Reality, input: &InputInfo, keys: &[KeyVal], day: Day, cut: Day) -> bool {
    for (bind, key) in input.binds.iter().zip(keys) {
        let (Domain::Kind(kind), KeyVal::Thing(name)) = (&bind.domain, key) else {
            continue;
        };
        let Some(thing) = m.things.get(name) else {
            continue;
        };
        if !member_span(m, thing, kind, cut).is_some_and(|span| span.contains(day)) {
            return true;
        }
    }
    false
}

/// What a read with no candidate gives: `none` where a key names a thing outside its
/// lifespan; else a hole whose question is the input's `question`, or names the period of
/// its time key, the law date, or neither for a read with no law date (L §4.6 step 5, §4.2
/// for a calendar's days).
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a calendar's days read with no law date.
pub fn no_candidate(
    m: &Reality,
    input: &InputInfo,
    keys: &[KeyVal],
    point: &Point,
    cut: Day,
    asked: Asked,
) -> Result<Found, Stop> {
    if let Some(day) = point.day
        && outside_lifespan(m, input, keys, day, cut)
    {
        return Ok(Found::None);
    }
    let id = input.id.as_str();
    let text = if asked == Asked::Calendar {
        let day = point
            .day
            .ok_or_else(|| invariant("a calendar read with no law date"))?;
        format!("calendar {id} doesn't cover {}", day.year())
    } else if let Some(question) = &input.question {
        question.clone()
    } else {
        no_value(id, keys, point)
    };
    Ok(Found::Hole(Question {
        text,
        origin: origin(id, keys),
    }))
}

/// `HOLE-UNGOVERNED`'s question: `x at k: no value for <period>`, `x at k: no value at <d>`,
/// `x: no value at <d>`, and with no law date `x at k: no value`, `x: no value` (L §4.6
/// step 5).
fn no_value(id: &str, keys: &[KeyVal], point: &Point) -> String {
    let at = if keys.is_empty() {
        String::new()
    } else {
        format!(" at {}", keys_text(keys))
    };
    match (&point.period, point.day) {
        (Some(period), _) => format!("{id}{at}: no value for {}", period.text()),
        (None, Some(day)) => format!("{id}{at}: no value at {}", day.text()),
        (None, None) => format!("{id}{at}: no value"),
    }
}

/// A read of input `input` at `keys`: the row the read rule takes, `none`, or a hole (L
/// §4.6; a reading's plain read takes the reading at the law date, Readings).
///
/// # Errors
///
/// `LOAD-OVERLAP` for rows still tied; `NotBuilt` for a dated read with no law date
/// (`LOAD-LAWDATE`); `ENGINE-INVARIANT` where load's promises break.
pub fn read_input(
    m: &Reality,
    input: &InputInfo,
    keys: &[KeyVal],
    clocks: Clocks,
    asked: Asked,
) -> Result<Found, Stop> {
    if input.reading {
        return read_reading(m, input, keys, clocks);
    }
    let point = time_point(input, keys, clocks.law)?;
    need_law(m, input, point.day)?;
    let test = Test::At(point.day);
    let cands = candidates(m, &input.rows, &input.binds, keys, test, clocks.cut)?;
    match choose(m, &input.rows, &input.binds, &cands, clocks.cut, &input.id)? {
        Some(at) => Ok(Found::Row(at)),
        None => no_candidate(m, input, keys, &point, clocks.cut, asked),
    }
}

/// A plain read of a reading at law date d: the reading at that instant, else a hole naming
/// the reading and the day (`HOLE-NOREADING`, L §4.6 Readings).
fn read_reading(
    m: &Reality,
    input: &InputInfo,
    keys: &[KeyVal],
    clocks: Clocks,
) -> Result<Found, Stop> {
    need_law(m, input, clocks.law)?;
    let day = clocks
        .law
        .ok_or_else(|| invariant("a reading read with no law date"))?;
    let test = Test::Instant(day);
    let cands = candidates(m, &input.rows, &input.binds, keys, test, clocks.cut)?;
    if let Some(at) = choose(m, &input.rows, &input.binds, &cands, clocks.cut, &input.id)? {
        return Ok(Found::Row(at));
    }
    let figure = origin(&input.id, keys);
    Ok(Found::Hole(Question {
        text: format!("{figure} has no reading at {}", day.text()),
        origin: figure,
    }))
}
