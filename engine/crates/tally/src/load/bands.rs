//! Load's band tables: each row's ends in the argument's order, each version's table whole, and tables of one key replacing only with `replace`.
//! Spec: L §5.9 (one table per version: ascending, no overlap, meeting closed against open,
//! `int` arguments over the integers; rows replaced, a patch or a whole table; LOAD-BAND,
//! LOAD-REPLACE), §4.6 steps 2 and 4 (LOAD-OVERLAP), §8.3.
//! Twin: engine/tools/refcore/loader.py (`band_end`, `order_key`, `check_bands`,
//! `check_table`, `band_defect`, `_uncovered`, `_span_ov`, `_fmt_pt`).
//! Invariants: an end is a value of the band's argument type, ordered as refcore's
//! `order_key` orders it; the first defect of a table refuses the reality at its row; tables
//! are checked band by band in declaration order, each band's tables in the order their
//! versions were read.

use super::check::invariant;
use super::steps::{Loader, refuse};
use crate::base::{Code, Load, Place as At, Refusal};
use crate::eval::{ConvertFor, Converted, convert};
use crate::model::{
    BandEnd, BandInfo, BandRow, BandTable, KeyVal, Reality, Span, Ty, Value, money_num,
};
use crate::num::Num;
use crate::syntax::{Lit, Stop, Value as Written};
use crate::time::Day;
use core::cmp::Ordering;
use std::collections::BTreeSet;

impl Loader {
    /// A band row's end: the literal as a value of the argument's type, then its place in
    /// that type's order (refcore's `band_end`, `order_key`). L §5.9.
    pub(super) fn band_end(
        &self,
        lit: &Lit,
        id: &str,
        arg_type: &Ty,
        file: &str,
        line: u32,
    ) -> Result<BandEnd, Stop> {
        let target = format!("band {id}");
        let written = Written::Lit(lit.clone());
        let converted = convert(&self.m, &written, arg_type, &target, &ConvertFor::plain())
            .map_err(|stop| placed(stop, file, line))?;
        let Converted::Value(value) = converted else {
            return Err(invariant(format!("a band end of {id} that is a hole")));
        };
        order_key(&self.m, &value)
    }
}

/// A value's place in a band's order: a number's value, money's in its currency's units, a
/// date, a period, a key's position in its index (refcore's `order_key`). L §5.9.
fn order_key(m: &Reality, value: &Value) -> Result<BandEnd, Stop> {
    match value {
        Value::Number(n) => Ok(BandEnd::Num(n.q.clone())),
        Value::Money(money) => money_num(m, money)
            .map(BandEnd::Num)
            .map_err(super::steps::num_stop),
        Value::Day(day) => Ok(BandEnd::Day(*day)),
        Value::Period(period) => Ok(BandEnd::Period(*period)),
        Value::Key(key) => key_end(m, key),
        Value::Window(_) => Err(Stop::NotBuilt("a band's end that is a window")),
        Value::Bool(_)
        | Value::MonthDay(..)
        | Value::Text(_)
        | Value::Code(_)
        | Value::Record(_)
        | Value::List(_)
        | Value::Calendar(_)
        | Value::None
        | Value::Never
        | Value::Hole(_)
        | Value::Word(_)
        | Value::OutKey { .. }
        | Value::Mv(_) => Err(unplaced(
            Load::Unit,
            "a band over values that are not numbers or dates".to_owned(),
        )),
    }
}

/// A key end: a key of a list or range index at its position (refcore's `order_key`, its
/// `Key` and things branches). L §5.9.
fn key_end(m: &Reality, key: &KeyVal) -> Result<BandEnd, Stop> {
    match key {
        KeyVal::Key { index, name } => {
            let place = m.indexes.get(index).and_then(|ix| ix.pos(key));
            place
                .map(|p| BandEnd::Key(p, key.clone()))
                .ok_or_else(|| unplaced(Load::Key, format!("{name} is not a key of {index}")))
        }
        KeyVal::Int(n) => Ok(BandEnd::Num(Num::int(*n))),
        KeyVal::Day(day) => Ok(BandEnd::Day(*day)),
        KeyVal::Period(period) => Ok(BandEnd::Period(*period)),
        KeyVal::Thing(name) => Err(unplaced(Load::Unit, format!("a band over things ({name})"))),
        KeyVal::Mv(mv) => Err(unplaced(
            Load::Unit,
            format!("a band over things ({})", mv.id),
        )),
    }
}

/// Every band's tables (refcore's `check_bands`): each table whole, then the tables of one
/// key: two known the same day on overlapping spans tie; a later one overlaps an earlier one
/// only with `replace`, all its rows `replace`, patching the earlier table's intervals or
/// forming a whole table, and a patch only where a table is in force. L §5.9.
///
/// # Errors
///
/// `LOAD-BAND`, `LOAD-OVERLAP` or `LOAD-REPLACE`, at the table's first row.
pub(super) fn check_bands(m: &Reality) -> Result<(), Stop> {
    for (id, band) in m.bands.entries() {
        for table in &band.tables {
            check_table(m, band, table, Whole::Unless)?;
        }
        let mut keys: Vec<Option<&KeyVal>> = Vec::new();
        for table in &band.tables {
            if !keys.contains(&table.key.as_ref()) {
                keys.push(table.key.as_ref());
            }
        }
        for key in keys {
            let tables: Vec<&BandTable> = band
                .tables
                .iter()
                .filter(|t| t.key.as_ref() == key)
                .collect();
            for at in 0..tables.len() {
                tables_of_one_key(m, id, band, at, &tables)?;
            }
        }
    }
    Ok(())
}

/// One table against the other tables of its key (refcore's `check_bands`, its loop over a
/// key's tables). L §5.9.
fn tables_of_one_key(
    m: &Reality,
    id: &str,
    band: &BandInfo,
    at: usize,
    tables: &[&BandTable],
) -> Result<(), Stop> {
    let table = *tables
        .get(at)
        .ok_or_else(|| invariant("a band table past its key's".to_owned()))?;
    let known = version_known(m, &table.version)?;
    let mut earlier = Vec::new();
    let mut same = Vec::new();
    for (j, other) in tables.iter().enumerate() {
        if j == at || !spans_overlap(m, &other.version, &table.version)? {
            continue;
        }
        match version_known(m, &other.version)?.cmp(&known) {
            Ordering::Less => earlier.push(*other),
            Ordering::Equal => same.push(*other),
            Ordering::Greater => {}
        }
    }
    let (file, line) = table_place(m, table)?;
    let vid = &table.version;
    if let Some(first) = same.first() {
        let message = format!(
            "band {id}: tables of {vid} and {} overlap, known the same day",
            first.version
        );
        return Err(refuse(Load::Overlap, message, &file, line));
    }
    if !table.rows.iter().any(|r| r.replace) {
        if let Some(first) = earlier.first() {
            let message = format!(
                "band {id}: the table of {vid} overlaps the table of {} without replace",
                first.version
            );
            return Err(refuse(Load::Overlap, message, &file, line));
        }
        return Ok(());
    }
    if !table.rows.iter().all(|r| r.replace) {
        let message = format!("band {id} in {vid} mixes replace rows and plain rows");
        return Err(refuse(Load::Band, message, &file, line));
    }
    if earlier.is_empty() {
        // L §8.3 LOAD-REPLACE's band form where its version has a span on the model's `on`
        // (refcore's `check_bands`)
        let message = match version_span(m, vid)? {
            Some(span) => format!(
                "replace rows of {id} in {vid} patch no table on {}",
                span.text()
            ),
            None => format!("replace {id}: no earlier row to replace"),
        };
        return Err(refuse(Load::Replace, message, &file, line));
    }
    let whole = match check_table(m, band, table, Whole::Asked) {
        Ok(()) => true,
        // A table that isn't whole is a `LOAD-BAND` refusal of its own; any other stop (a
        // number past the cap, an invariant) is passed on.
        Err(Stop::Refused(r)) if r.code() == Code::Load(Load::Band) => false,
        Err(other) => return Err(other),
    };
    let matches = table.rows.iter().all(|row| in_earlier(row, &earlier));
    if !matches && !whole {
        let message = format!(
            "band {id} in {vid}: the replace rows neither match the earlier table's intervals nor form a whole table"
        );
        return Err(refuse(Load::Band, message, &file, line));
    }
    if whole {
        return Ok(());
    }
    let Some(span) = version_span(m, vid)? else {
        return Ok(());
    };
    let mut others = Vec::new();
    for e in &earlier {
        if let Some(s) = version_span(m, &e.version)? {
            others.push(s);
        }
    }
    if let Some(gap) = uncovered(&span, &others) {
        let message = format!(
            "replace rows of {id} in {vid} patch no table on {}",
            gap.text()
        );
        return Err(refuse(Load::Replace, message, &file, line));
    }
    Ok(())
}

/// Whether a table of `replace` rows only is checked: when asked whether it is whole, never
/// as a table of its own (refcore's `check_table`'s `whole`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Whole {
    /// Asked whether it forms a whole table.
    Asked,
    /// Checked as its version's table, unless it is of `replace` rows only.
    Unless,
}

/// One version's table of a band, whole: each row's ends in order, an `int` band's ends
/// whole, and each row starting where the row before it ends (refcore's `check_table`); a
/// table of `replace` rows only is checked when asked whether it is whole. L §5.9.
fn check_table(m: &Reality, band: &BandInfo, table: &BandTable, whole: Whole) -> Result<(), Stop> {
    let replace = !table.rows.is_empty() && table.rows.iter().all(|r| r.replace);
    if replace && whole == Whole::Unless {
        return Ok(());
    }
    let id = &band.id;
    let vid = &table.version;
    let is_int = band.arg_type == Ty::Int;
    let file = version_file(m, vid)?;
    for row in &table.rows {
        let (Some(lo), Some(hi)) = (&row.lo, &row.hi) else {
            continue;
        };
        let order = end_order(lo, hi)?;
        let empty = order == Ordering::Equal && !(row.lo_closed && row.hi_closed);
        if order == Ordering::Greater || empty {
            let message = format!(
                "band {id} in {vid}: rows unordered, overlapping or with a gap at {}",
                end_text(lo)?
            );
            return Err(refuse(Load::Band, message, &file, row.line));
        }
    }
    if is_int {
        for (before, next) in table.rows.iter().zip(table.rows.iter().skip(1)) {
            let (Some(BandEnd::Num(hi)), Some(BandEnd::Num(lo))) = (&before.hi, &next.lo) else {
                continue;
            };
            let (last, first) = int_ends(before, hi, next, lo)?;
            if !last.is_integer() || !first.is_integer() {
                let message = format!("band {id} in {vid}: an int band row with a fractional end");
                return Err(refuse(Load::Band, message, &file, next.line));
            }
        }
    }
    if let Some((at, line)) = band_defect(band, &table.rows)? {
        let message =
            format!("band {id} in {vid}: rows unordered, overlapping or with a gap at {at}");
        return Err(refuse(Load::Band, message, &file, line));
    }
    Ok(())
}

/// The first row, in the table's order, that doesn't start where the row before it ends (an
/// overlap, a gap or a row out of order), as its lower end as written, `-inf` for a row
/// from -inf, and its line; an `int` band's rows meet over the integers (refcore's
/// `band_defect`). L §5.9, §8.3.
fn band_defect(band: &BandInfo, rows: &[BandRow]) -> Result<Option<(String, u32)>, Stop> {
    let is_int = band.arg_type == Ty::Int;
    for (before, next) in rows.iter().zip(rows.iter().skip(1)) {
        let (Some(hi), Some(lo)) = (&before.hi, &next.lo) else {
            let at = next.lo.as_ref().map_or(Ok("-inf".to_owned()), end_text)?;
            return Ok(Some((at, next.line)));
        };
        let meets = match (is_int, hi, lo) {
            (true, BandEnd::Num(h), BandEnd::Num(l)) => {
                let (last, first) = int_ends(before, h, next, l)?;
                let after = last
                    .checked_add(&Num::int(1))
                    .map_err(super::steps::num_stop)?;
                first == after
            }
            _ => end_order(lo, hi)? == Ordering::Equal && before.hi_closed != next.lo_closed,
        };
        if !meets {
            return Ok(Some((end_text(lo)?, next.line)));
        }
    }
    Ok(None)
}

/// An `int` band's last integer of a row and first integer of the next: an open end is the
/// integer inside it.
fn int_ends(before: &BandRow, hi: &Num, next: &BandRow, lo: &Num) -> Result<(Num, Num), Stop> {
    let one = Num::int(1);
    let last = if before.hi_closed {
        hi.clone()
    } else {
        hi.checked_sub(&one).map_err(super::steps::num_stop)?
    };
    let first = if next.lo_closed {
        lo.clone()
    } else {
        lo.checked_add(&one).map_err(super::steps::num_stop)?
    };
    Ok((last, first))
}

/// How two ends of one band compare (refcore compares `order_key`s). L §5.9.
fn end_order(a: &BandEnd, b: &BandEnd) -> Result<Ordering, Stop> {
    match (a, b) {
        (BandEnd::Num(x), BandEnd::Num(y)) => x.compare(y).map_err(super::steps::num_stop),
        (BandEnd::Day(x), BandEnd::Day(y)) => Ok(x.cmp(y)),
        (BandEnd::Period(x), BandEnd::Period(y)) => {
            Ok((x.start(), x.end()).cmp(&(y.start(), y.end())))
        }
        (BandEnd::Key(x, _), BandEnd::Key(y, _)) => Ok(x.cmp(y)),
        _ => Err(invariant(
            "two ends of one band of different types".to_owned(),
        )),
    }
}

/// Whether an earlier table holds a row of `row`'s interval (refcore's `check_bands`, its
/// `matches`). L §5.9.
fn in_earlier(row: &BandRow, earlier: &[&BandTable]) -> bool {
    for table in earlier {
        if table.rows.iter().any(|o| same_interval(row, o)) {
            return true;
        }
    }
    false
}

/// Whether two rows state one interval (refcore's `BandRow.same_interval`). L §5.9.
fn same_interval(a: &BandRow, b: &BandRow) -> bool {
    a.lo == b.lo && a.hi == b.hi && a.lo_closed == b.lo_closed && a.hi_closed == b.hi_closed
}

/// An end as load's message prints it: a whole number plainly, another as `n/d`, a date, a
/// period by its label, a key by its name (refcore's `_fmt_pt`).
fn end_text(end: &BandEnd) -> Result<String, Stop> {
    match end {
        BandEnd::Num(q) => fraction_text(q),
        BandEnd::Day(day) => Ok(day.text()),
        BandEnd::Period(period) => Ok(period.label()),
        BandEnd::Key(_, key) => Ok(key.text()),
    }
}

/// A number as Python prints a `Fraction`: `5501`, `11/2` (refcore's `_fmt_pt`).
fn fraction_text(q: &Num) -> Result<String, Stop> {
    let plain = |n: &Num| -> Result<String, Stop> {
        Ok(n.to_fixed(0)
            .map_err(super::steps::num_stop)?
            .replace(' ', ""))
    };
    if q.is_integer() {
        return plain(q);
    }
    let Some(places) = q.decimal_places().map_err(super::steps::num_stop)? else {
        let text = q.to_text().map_err(super::steps::num_stop)?;
        let fraction = text.split_once(" (").map_or(text.as_str(), |(f, _)| f);
        return Ok(fraction.to_owned());
    };
    for denominator in decimal_denominators(places) {
        let d = Num::int(denominator);
        let n = q.checked_mul(&d).map_err(super::steps::num_stop)?;
        if n.is_integer() {
            return Ok(format!("{}/{denominator}", plain(&n)?));
        }
    }
    q.to_text().map_err(super::steps::num_stop)
}

/// The divisors of 10^places, `2^i * 5^j`, in ascending order, those that fit an i64.
fn decimal_denominators(places: u32) -> BTreeSet<i64> {
    let mut out = BTreeSet::new();
    for i in 0..=places {
        for j in 0..=places {
            let value = 2_i64
                .checked_pow(i)
                .zip(5_i64.checked_pow(j))
                .and_then(|(a, b)| a.checked_mul(b));
            if let Some(v) = value {
                out.insert(v);
            }
        }
    }
    out
}

/// The first stretch of `span` that no span of `others` covers, or none (refcore's
/// `_uncovered`). L §5.9.
fn uncovered(span: &Span, others: &[Span]) -> Option<Span> {
    let (lo, hi) = edges(span);
    let mut ivs: Vec<(Edge, Edge)> = others.iter().map(edges).collect();
    ivs.sort();
    let mut cur = lo;
    for (a, b) in ivs {
        if cur >= hi {
            return None;
        }
        if a > cur {
            return Some(edge_span(cur, a.min(hi)));
        }
        cur = cur.max(b);
    }
    (cur < hi).then(|| edge_span(cur, hi))
}

/// A span's first day and the first day after it, open ends at the edges of time.
fn edges(span: &Span) -> (Edge, Edge) {
    if span.is_always() {
        return (FIRST, LAST);
    }
    (
        span.start().map_or(FIRST, |d| (1, Some(d))),
        span.end().map_or(LAST, |d| (1, Some(d))),
    )
}

/// The span from `start` to `end`, an edge of time an open end.
fn edge_span(start: Edge, end: Edge) -> Span {
    Span::between(start.1, end.1)
}

/// A day, `(1, Some(day))`, or an edge of time, before every day `(0, None)` and after every
/// day `(2, None)` (refcore's `date.min` and `date.max`), ordered as written.
type Edge = (u8, Option<Day>);

/// Before every day.
const FIRST: Edge = (0, None);

/// After every day.
const LAST: Edge = (2, None);

/// Whether two versions' spans in force at `on` overlap (refcore's `_span_ov`). L §4.6.
fn spans_overlap(m: &Reality, one: &str, other: &str) -> Result<bool, Stop> {
    match (version_span(m, one)?, version_span(m, other)?) {
        (Some(first), Some(second)) => Ok(first.overlaps(&second)),
        _ => Ok(false),
    }
}

/// A version's span in force at the model's `on`. L §4.6, §5.3.
fn version_span(m: &Reality, id: &str) -> Result<Option<Span>, Stop> {
    let version = m
        .versions
        .get(id)
        .ok_or_else(|| invariant(format!("version {id} isn't built")))?;
    Ok(version.span_at(m.on).cloned())
}

/// A version's `known`, or its window's first day. L §4.5.
fn version_known(m: &Reality, id: &str) -> Result<Day, Stop> {
    m.versions
        .get(id)
        .map(|v| v.known)
        .ok_or_else(|| invariant(format!("version {id} isn't built")))
}

/// The file a version stands in.
fn version_file(m: &Reality, id: &str) -> Result<String, Stop> {
    m.versions
        .get(id)
        .map(|v| v.file.clone())
        .ok_or_else(|| invariant(format!("version {id} isn't built")))
}

/// Where a table is written: its version's file, its first row's line (refcore's
/// `Table.loc`).
fn table_place(m: &Reality, table: &BandTable) -> Result<(String, u32), Stop> {
    let line = table
        .rows
        .first()
        .map(|r| r.line)
        .ok_or_else(|| invariant("a band table with no row".to_owned()))?;
    Ok((version_file(m, &table.version)?, line))
}

/// A refusal of `code` with no place, as refcore's `order_key` raises it.
fn unplaced(code: Load, message: String) -> Stop {
    Stop::Refused(Refusal::new(Code::Load(code), message))
}

/// A refusal met converting an end, put at its row's line where it names no place.
fn placed(stop: Stop, file: &str, line: u32) -> Stop {
    match stop {
        Stop::Refused(refusal) => Stop::Refused(refusal.placed(At::Line(file.to_owned(), line))),
        other @ (Stop::NotBuilt(_) | Stop::Deep | Stop::Guarded) => other,
    }
}
