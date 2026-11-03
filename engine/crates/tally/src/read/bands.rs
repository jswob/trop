//! The read rule for bands: the table a read takes, replaced rows patched in, and the questions of a read with no table or row.
//! Spec: L §5.9 (a version's table as one row of the read rule; rows replaced), §4.6 steps
//! 2, 4 and 5, §4.3 (a band read with no law date), §8.1 (`HOLE-UNGOVERNED`), §8.2
//! (`held`).
//! Twin: engine/tools/refcore/evaluate.py (`band_table`, `_table_of`, `band_origin`,
//! `band_rows`, `band_no_row`, `band_value`'s held mark), model.py (`Table.replace`,
//! `BandRow.same_interval`).
//! Invariants: refcore's recursion through earlier tables is a loop: the chain of tables,
//! each the latest known before the one above it, while that one says `replace`. A table
//! that replaces another wholly is told from load's band checks, which are M0's, so it
//! stops with `NotBuilt`.

use super::gap::Question;
use super::rows::{Clocks, invariant, no_law_date, version_known, version_span};
use crate::base::{Code, Load, Refusal};
use crate::model::{BandInfo, BandRow, BandTable as Table, KeyVal, Reality};
use crate::syntax::Stop;
use crate::time::Day;

/// A band row a read can take: its table's position and its own (L §5.9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BandAt {
    /// The table.
    pub table: usize,
    /// The row in it.
    pub row: usize,
}

/// Whether a table is a patch of `replace` rows: it has rows, all `replace` (L §5.9).
fn replaces(table: &Table) -> bool {
    !table.rows.is_empty() && table.rows.iter().all(|r| r.replace)
}

/// Whether two band rows state the same interval (L §5.9 Rows replaced).
fn same_interval(a: &BandRow, b: &BandRow) -> bool {
    a.lo == b.lo && a.hi == b.hi && a.lo_closed == b.lo_closed && a.hi_closed == b.hi_closed
}

/// A refusal of a band read: its code and message (L §8.3).
fn refuse(code: Load, message: String) -> Stop {
    Stop::Refused(Refusal::new(Code::Load(code), message))
}

/// The table at position `at` (L §5.9).
fn table_at(band: &BandInfo, at: usize) -> Result<&Table, Stop> {
    band.tables
        .get(at)
        .ok_or_else(|| invariant("a band table past its tables"))
}

/// The tables a read of band `band` at `key` may take: its version is known by the cut and
/// governs the law date, or governs `always` for a read with no law date (L §5.9, §4.6
/// step 1, §4.3).
fn table_candidates(
    m: &Reality,
    band: &BandInfo,
    key: Option<&KeyVal>,
    clocks: Clocks,
) -> Result<Vec<usize>, Stop> {
    let mut out = Vec::new();
    for (at, table) in band.tables.iter().enumerate() {
        if table.key.as_ref() != key || version_known(m, &table.version)? > clocks.cut {
            continue;
        }
        let Some(span) = version_span(m, &table.version, clocks.cut)? else {
            continue;
        };
        let governs = match clocks.law {
            Some(day) => span.contains(day),
            None if span.is_always() => true,
            None => return Err(no_law_date()),
        };
        if governs {
            out.push(at);
        }
    }
    Ok(out)
}

/// The latest-known table of `pool` (L §4.6 step 4, §5.9).
fn latest(m: &Reality, band: &BandInfo, pool: &[usize]) -> Result<Option<usize>, Stop> {
    let mut best: Option<(usize, Day)> = None;
    let mut tied = false;
    for &at in pool {
        let known = version_known(m, &table_at(band, at)?.version)?;
        match best {
            Some((_, held)) if held == known => tied = true,
            Some((_, held)) if held > known => {}
            Some(_) | None => {
                best = Some((at, known));
                tied = false;
            }
        }
    }
    if tied {
        let message = format!("band {}: two tables known the same day", band.id);
        return Err(refuse(Load::Overlap, message));
    }
    Ok(best.map(|(at, _)| at))
}

/// The chain of tables a read takes: the latest known, then while it says `replace`, the
/// latest known before it (L §5.9 Rows replaced, §4.6 steps 2 and 4).
fn chain(m: &Reality, band: &BandInfo, cands: &[usize]) -> Result<Vec<usize>, Stop> {
    let mut out = Vec::new();
    let mut pool = cands.to_vec();
    while let Some(top) = latest(m, band, &pool)? {
        out.push(top);
        let table = table_at(band, top)?;
        if !replaces(table) {
            if let [_, _, ..] = pool.as_slice() {
                let message = format!("band {}: tables overlap without replace", band.id);
                return Err(refuse(Load::Overlap, message));
            }
            break;
        }
        let known = version_known(m, &table.version)?;
        let mut earlier = Vec::new();
        for &at in &pool {
            if version_known(m, &table_at(band, at)?.version)? < known {
                earlier.push(at);
            }
        }
        pool = earlier;
    }
    Ok(out)
}

/// The rows of a band read at `key` under `clocks`, each with its table: the table the read
/// rule takes, with the rows a later `replace` table restates patched in; none where no
/// table governs (L §5.9, §4.6).
///
/// # Errors
///
/// `LOAD-OVERLAP` for tables known the same day or overlapping without `replace`;
/// `NotBuilt` for a `replace` table that stands as a whole table, or for a dated band read
/// with no law date (`LOAD-LAWDATE`).
pub fn band_rows(
    m: &Reality,
    band: &BandInfo,
    key: Option<&KeyVal>,
    clocks: Clocks,
) -> Result<Option<Vec<BandAt>>, Stop> {
    let cands = table_candidates(m, band, key, clocks)?;
    let tables = chain(m, band, &cands)?;
    let mut rows: Option<Vec<BandAt>> = None;
    for &at in tables.iter().rev() {
        let table = table_at(band, at)?;
        rows = Some(match rows {
            None if replaces(table) => {
                return Err(Stop::NotBuilt(
                    "a band's replace table with no table under it",
                ));
            }
            None => (0..table.rows.len())
                .map(|row| BandAt { table: at, row })
                .collect(),
            Some(base) => patch(band, at, &base)?,
        });
    }
    Ok(rows)
}

/// The earlier table's rows with those a `replace` table restates swapped for its rows,
/// when each of its rows has an interval of the earlier table's (L §5.9 Rows replaced).
fn patch(band: &BandInfo, at: usize, base: &[BandAt]) -> Result<Vec<BandAt>, Stop> {
    let table = table_at(band, at)?;
    let mut base_rows = Vec::new();
    for held in base {
        let row = table_at(band, held.table)?
            .rows
            .get(held.row)
            .ok_or_else(|| invariant("a band row past its table"))?;
        base_rows.push(row);
    }
    let mut matches = true;
    for row in &table.rows {
        matches = matches && base_rows.iter().any(|b| same_interval(row, b));
    }
    if !matches {
        return Err(Stop::NotBuilt(
            "a band's replace rows that form a whole table",
        ));
    }
    let mut out = Vec::new();
    for (held, earlier) in base.iter().zip(base_rows) {
        let restated = table.rows.iter().position(|r| same_interval(r, earlier));
        out.push(restated.map_or(*held, |row| BandAt { table: at, row }));
    }
    Ok(out)
}

/// Whether a band row read from table `table` is `held`: its version's span has a start and
/// no end, and the law date is past the ask's cut (L §8.2).
///
/// # Errors
///
/// `ENGINE-INVARIANT` when the table's version is missing.
pub fn table_held(m: &Reality, table: &Table, clocks: Clocks) -> Result<bool, Stop> {
    let (Some(span), Some(law)) = (version_span(m, &table.version, clocks.cut)?, clocks.law) else {
        return Ok(false);
    };
    Ok(!span.is_always() && span.start().is_some() && span.end().is_none() && law > clocks.ask_cut)
}

/// The figure a band read is of, `b` or `b[k]` (L §8.1).
#[must_use]
pub fn band_origin(id: &str, key: Option<&KeyVal>) -> String {
    key.map_or_else(|| id.to_owned(), |k| format!("{id}[{}]", k.text()))
}

/// `HOLE-UNGOVERNED` for a band no table governs: `b: no value at <d>`, `b at k: no value at
/// <d>`, or with no law date `… no value` (L §4.6 step 5).
#[must_use]
pub fn band_no_table(id: &str, key: Option<&KeyVal>, law: Option<Day>) -> Question {
    let at = law.map_or_else(String::new, |day| format!(" at {}", day.text()));
    let text = key.map_or_else(
        || format!("{id}: no value{at}"),
        |k| format!("{id} at {}: no value{at}", k.text()),
    );
    Question {
        text,
        origin: band_origin(id, key),
    }
}

/// `HOLE-UNGOVERNED` for a band whose table holds no row for its argument, printed `x`:
/// `b: no row holds x at <d>`, or with no law date `b: no row holds x` (L §5.9, §8.1).
#[must_use]
pub fn band_no_row(id: &str, key: Option<&KeyVal>, x: &str, law: Option<Day>) -> Question {
    let figure = band_origin(id, key);
    let at = law.map_or_else(String::new, |day| format!(" at {}", day.text()));
    Question {
        text: format!("{figure}: no row holds {x}{at}"),
        origin: figure,
    }
}
