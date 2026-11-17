//! The gap rule and the read of an input: the row a read takes, `none` outside a lifespan, or the question of the hole a read with no row is.
//! Spec: L §4.6 step 5 (no candidate: `HOLE-UNGOVERNED`'s questions, outside a lifespan; the
//! candidates a `wording` pin keeps, L §4.3),
//! Readings (a plain read of a reading, `HOLE-NOREADING`), Past the last row (`last(x, d)`),
//! §4.2 (a calendar's question), §5.7 (lifespans and memberships), §8.1, §6.4 (the rows,
//! lifespans and memberships a read's world sees).
//! Twin: engine/tools/refcore/evaluate.py (`origin`, `no_candidate`, `read_input`,
//! `read_reading`, `e_lastx`), model.py (`member_span`).
//! Invariants: a question is text and the item it is asked of; the hole itself is built in
//! the hole module (R-116). A thing stated `same` belongs to a joint model, which S doesn't
//! build.

use std::collections::BTreeSet;

use super::rows::{
    Clocks, Point, Test, candidates, choose, invariant, need_law, row_start, standing_rows,
    time_point, worded,
};
use crate::model::{Domain, InputInfo, KeyVal, Reality, Span, Thing, World};
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

/// The span over which `thing` is of `kind` or a subkind under the cut and in the world of
/// `seen`: its one membership there, or the hull of several (L §5.7, §6.4).
#[must_use]
pub fn member_span(m: &Reality, thing: &Thing, kind: &str, seen: (Day, World)) -> Option<Span> {
    let subtree = m.subtree(kind);
    let kinds: BTreeSet<&str> = thing
        .entries
        .iter()
        .filter(|e| subtree.contains(&e.kind))
        .map(|e| e.kind.as_str())
        .collect();
    let spans: Vec<Span> = kinds
        .iter()
        .filter_map(|k| m.span_in(thing, k, seen))
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

/// Whether a key names a thing outside its lifespan on `day` in the world of `seen`, so a
/// read with no row is `none` (L §4.6 step 5, Outside a lifespan; §6.4).
fn outside_lifespan(
    m: &Reality,
    (input, keys): (&InputInfo, &[KeyVal]),
    day: Day,
    seen: (Day, World),
) -> bool {
    for (bind, key) in input.binds.iter().zip(keys) {
        let (Domain::Kind(kind), KeyVal::Thing(name)) = (&bind.domain, key) else {
            continue;
        };
        let Some(thing) = m.things.get(name) else {
            continue;
        };
        if !member_span(m, thing, kind, seen).is_some_and(|span| span.contains(day)) {
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
    seen: (Day, World),
    asked: Asked,
) -> Result<Found, Stop> {
    if let Some(day) = point.day
        && outside_lifespan(m, (input, keys), day, seen)
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
    let cands = candidates(m, &input.rows, (&input.binds, keys), test, clocks)?;
    let cands = worded(m, &input.rows, cands, clocks)?;
    match choose(
        m,
        &input.rows,
        &input.binds,
        &cands,
        (clocks.cut, clocks.world),
        &input.id,
    )? {
        Some(at) => Ok(Found::Row(at)),
        None => no_candidate(m, input, keys, &point, (clocks.cut, clocks.world), asked),
    }
}

/// `last(x[k], d)`'s read: among the rows of `input` at `keys` whose span began on or before
/// `day` (a span with no start before every day; a reading's instants on or before it),
/// those that began latest, and of them the row the read rule takes; with none,
/// `HOLE-NOREADING` for a reading, else the gap rule's question at `day` (L §4.6 step 5,
/// Readings, Past the last row; §5.3; refcore's `e_lastx`).
///
/// # Errors
///
/// `LOAD-OVERLAP` for rows still tied; `RUN-DOMAIN` where two units' readings stand at the
/// instant (L §5.10); `ENGINE-INVARIANT` where load's promises break.
pub fn read_last(
    m: &Reality,
    input: &InputInfo,
    keys: &[KeyVal],
    day: Day,
    clocks: Clocks,
) -> Result<Found, Stop> {
    let kept = match last_began(m, input, keys, day, clocks)? {
        Ok(kept) => kept,
        Err(found) => return Ok(found),
    };
    one_unit(m, input, &kept, "last")?;
    match choose(
        m,
        &input.rows,
        &input.binds,
        &kept,
        (clocks.cut, clocks.world),
        &input.id,
    )? {
        Some(at) => Ok(Found::Row(at)),
        None => Err(invariant("last's candidates chose no row")),
    }
}

/// The rows of `input` at `keys` whose span began on or before `day` (a span with no start
/// before every day; a reading's instants on or before it) that began latest; with none,
/// what `last(x[k], d)` reads instead: `HOLE-NOREADING` for a reading, else the gap rule's
/// question at `day`. `since(x[k], d)` reads only the day they began, which they share, so
/// it takes none over another: at a place of unit `any`, one reading per unit (L §4.6 step
/// 5, Readings, Past the last row; refcore's `e_lastx`, `e_call_since`).
///
/// # Errors
///
/// `ENGINE-INVARIANT` where load's promises break.
pub fn last_began(
    m: &Reality,
    input: &InputInfo,
    keys: &[KeyVal],
    day: Day,
    clocks: Clocks,
) -> Result<Result<Vec<usize>, Found>, Stop> {
    let test = Test::Last(day);
    let cands = candidates(m, &input.rows, (&input.binds, keys), test, clocks)?;
    let cands = worded(m, &input.rows, cands, clocks)?;
    let mut starts = Vec::with_capacity(cands.len());
    for &at in &cands {
        let row = input
            .rows
            .get(at)
            .ok_or_else(|| invariant("a candidate past its rows"))?;
        starts.push((at, row_start(m, row, (clocks.cut, clocks.world))?));
    }
    let began: BTreeSet<Option<Day>> = starts.iter().map(|(_, start)| *start).collect();
    // the latest instant whose reading was removed, held for the note when no reading stands
    let mut gone = None;
    for latest in began.into_iter().rev() {
        let kept: Vec<usize> = starts
            .iter()
            .filter(|(_, start)| *start == latest)
            .map(|(at, _)| *at)
            .collect();
        // a reading `replace … = none` removed leaves its instant with no reading, so the
        // reading before it is the last (L §5.19, L §4.6 Readings; pick P24, SQ-1)
        let stand = standing_rows(m, &input.rows, &kept)?;
        if !(input.reading && !stand.is_empty() && stand.iter().all(|at| removed(input, *at))) {
            return Ok(Ok(kept));
        }
        if gone.is_none() {
            gone = Some(kept);
        }
    }
    if let Some(kept) = gone {
        return Ok(Ok(kept));
    }
    if input.reading {
        let figure = origin(&input.id, keys);
        return Ok(Err(Found::Hole(Question {
            text: format!("{figure} has no reading on or before {}", day.text()),
            origin: figure,
        })));
    }
    let point = Point {
        day: Some(day),
        period: None,
    };
    no_candidate(
        m,
        input,
        keys,
        &point,
        (clocks.cut, clocks.world),
        Asked::Plain,
    )
    .map(Err)
}

/// Whether the row at position `at` of `input` states `none`: a reading it removes (L §5.19).
fn removed(input: &InputInfo, at: usize) -> bool {
    matches!(input.rows.get(at), Some(row) if row.value == crate::syntax::Value::None)
}

/// A read of reading `input` that names no unit, among the rows `cands` at one instant:
/// refused where the rows that stand there state two units (L §5.10, tally 2.1.8 R14;
/// refcore's `choose`, its unit test). `read` names the read: `last`, or the input.
///
/// # Errors
///
/// `RUN-DOMAIN` for two units' readings; `ENGINE-INVARIANT` where load's promises break.
fn one_unit(m: &Reality, input: &InputInfo, cands: &[usize], read: &str) -> Result<(), Stop> {
    if !input.reading || cands.len() < 2 {
        return Ok(());
    }
    let (mut units, mut instants, mut place) = (BTreeSet::new(), BTreeSet::new(), None);
    for at in standing_rows(m, &input.rows, cands)? {
        let row = input
            .rows
            .get(at)
            .ok_or_else(|| invariant("a candidate past its rows"))?;
        place = place.or_else(|| crate::book::reading_place(row));
        let unit = crate::book::reading_place(row)
            .map(|p| m.reading_ty(p, &row.value))
            .transpose()
            .map_err(crate::eval::num_stop)?
            .flatten();
        if let Some((unit, _)) = unit {
            units.insert(unit);
        }
        instants.insert(instant_of(row));
    }
    match (place, instants.first()) {
        (Some(place), Some(instant)) if units.len() > 1 && instants.len() == 1 => {
            let units: Vec<String> = units.into_iter().collect();
            Err(two_units(read, place, instant, &units))
        }
        _ => Ok(()),
    }
}

/// A reading row's instant as text: `2026-02-15 end`, `after save[2026-03]`, or its day
/// (refcore's `instant_text`). L §4.6 Readings.
fn instant_of(row: &crate::model::Row) -> String {
    match &row.instant {
        Some(crate::model::RowInstant::At(day, Some(edge))) => {
            crate::book::instant_text(*day, *edge)
        }
        Some(crate::model::RowInstant::At(day, None)) => day.text(),
        Some(crate::model::RowInstant::After(node)) => {
            format!("after {}", crate::load::expr_text(node))
        }
        None => String::new(),
    }
}

/// `RUN-DOMAIN` for a read that names no unit where readings in two units stand at its
/// instant: `<read> of <x>: its readings at <instant> are in EUR and PLN` (L §5.10, §8.4;
/// tally 2.1.8 R14; refcore's `units_at_instant`).
#[must_use]
pub fn two_units(read: &str, place: &str, instant: &str, units: &[String]) -> Stop {
    let listed = match units.split_last() {
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
        None => String::new(),
    };
    Stop::Refused(crate::base::Refusal::new(
        crate::base::Code::Run(crate::base::Run::Domain),
        format!("{read} of {place}: its readings at {instant} are in {listed}"),
    ))
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
    let cands = candidates(m, &input.rows, (&input.binds, keys), test, clocks)?;
    let cands = worded(m, &input.rows, cands, clocks)?;
    one_unit(m, input, &cands, &input.id)?;
    // a reading `replace … = none` removed leaves its instant with no reading (L §5.19)
    if let Some(at) = choose(
        m,
        &input.rows,
        &input.binds,
        &cands,
        (clocks.cut, clocks.world),
        &input.id,
    )? && !removed(input, at)
    {
        return Ok(Found::Row(at));
    }
    let figure = origin(&input.id, keys);
    Ok(Found::Hole(Question {
        text: format!("{figure} has no reading at {}", day.text()),
        origin: figure,
    }))
}
