//! The read rule's rows: which rows of an input or of a def's givens are candidates for a read, and which one it takes.
//! Spec: L §4.6 steps 1–4 (candidates, replacement, precedence, ties), §4.5 (the cut sees
//! what is known by it), §4.3 (the law date; a read with none).
//! Twin: engine/tools/refcore/evaluate.py (`time_point`, `need_law`, `row_matches`,
//! `key_ord`, `unstated_ok`, `row_visible`, `candidates`, `choose`), loader.py
//! (`pattern_norm`, `_pos_rel`, `narrower`, `key_order`), model.py (`Row.span`, `Row.known`,
//! `row_timeless`, `InputInfo.lawless`).
//! Invariants: a plain scan of the target's rows in the order versions state them; the
//! candidates keep that order; two rows still tied are refused, never broken by order.
//! Scenarios, joint models, `wording` pins and known windows stop before load ends
//! (`NotBuilt`), so every row here is the base's, known on one day, and no pin narrows it.

use crate::base::{Code, Load, Refusal};
use crate::model::{
    BindInfo, Domain, InputInfo, KeyVal, Pat, Reality, Row, RowInstant, Span, TimePos,
};
use crate::syntax::Stop;
use crate::time::Day;

/// A read's clocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clocks {
    /// The law date d; none for a read with no law date (L §4.3).
    pub law: Option<Day>,
    /// The cut K: the read sees what is known by it (L §4.5).
    pub cut: Day,
    /// The ask's cut, past which a row read for a later day is `held` (L §8.2).
    pub ask_cut: Day,
}

/// Which rows a read takes as candidates by their time (L §4.6 step 1, Readings).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Test {
    /// A row whose span holds this day; none: a read with no law date, which only a row
    /// that governs `always` answers.
    At(Option<Day>),
    /// A reading at this instant.
    Instant(Day),
}

/// The day a read's spans are tested at, and its time key's period (L §4.6 step 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Point {
    /// The period's first day, or the law date.
    pub day: Option<Day>,
    /// The time key, for an input keyed by one period index.
    pub period: Option<KeyVal>,
}

/// A broken promise of load's: `ENGINE-INVARIANT` (E §11).
pub(super) fn invariant(what: &str) -> Stop {
    Stop::Refused(Refusal::new(Code::EngineInvariant, what.to_owned()))
}

/// A dated read with no law date: load refuses it (`LOAD-LAWDATE`, L §4.3), a check S
/// doesn't build.
pub(super) fn no_law_date() -> Stop {
    Stop::NotBuilt("a dated read with no law date (LOAD-LAWDATE)")
}

/// A version load didn't build: `ENGINE-INVARIANT` (E §11).
fn no_version() -> Stop {
    invariant("a row names a version load didn't build")
}

/// Version `id`'s `known` (L §4.5).
pub(super) fn version_known(m: &Reality, id: &str) -> Result<Day, Stop> {
    m.versions.get(id).map(|v| v.known).ok_or_else(no_version)
}

/// Version `id`'s governs span in force under `cut` (L §4.6 step 1, §5.3).
pub(super) fn version_span(m: &Reality, id: &str, cut: Day) -> Result<Option<Span>, Stop> {
    m.versions
        .get(id)
        .map(|v| v.span_at(cut).cloned())
        .ok_or_else(no_version)
}

/// Whether version `id` governs `always` (L §4.3 rule 1).
fn version_always(m: &Reality, id: &str) -> Result<bool, Stop> {
    m.versions
        .get(id)
        .map(|v| v.governs.as_ref().is_some_and(Span::is_always))
        .ok_or_else(no_version)
}

/// The file version `id` is in (L §8.3).
fn version_file(m: &Reality, id: &str) -> Result<String, Stop> {
    m.versions
        .get(id)
        .map(|v| v.file.clone())
        .ok_or_else(no_version)
}

/// The point an input's read is tested at: the first day of its time key's period when it
/// is keyed by one period index, else the law date (L §4.6 step 1, §4.3).
///
/// # Errors
///
/// `ENGINE-INVARIANT` when the time key isn't a day or a period.
pub fn time_point(input: &InputInfo, keys: &[KeyVal], law: Option<Day>) -> Result<Point, Stop> {
    let TimePos::At(at) = input.time_pos else {
        return Ok(Point {
            day: law,
            period: None,
        });
    };
    let key = keys
        .get(at)
        .ok_or_else(|| invariant("a read has fewer keys than its input's binds"))?;
    let day = key
        .start()
        .ok_or_else(|| invariant("a time key that is neither a day nor a period"))?;
    Ok(Point {
        day: Some(day),
        period: Some(key.clone()),
    })
}

/// A row's span under cut `cut`: its own, else its version's in force (L §4.6 step 1).
///
/// # Errors
///
/// `ENGINE-INVARIANT` when its version is missing.
pub fn row_span(m: &Reality, row: &Row, cut: Day) -> Result<Option<Span>, Stop> {
    if let Some(own) = &row.own_span {
        return Ok(Some(own.clone()));
    }
    version_span(m, &row.version, cut)
}

/// A row's `known`: its own, else its version's (L §4.5).
///
/// # Errors
///
/// `ENGINE-INVARIANT` when its version is missing.
pub fn row_known(m: &Reality, row: &Row) -> Result<Day, Stop> {
    if let Some(own) = row.own_known {
        return Ok(own);
    }
    version_known(m, &row.version)
}

/// A row as a refusal names it, `a.tally:41` (L §8.3).
///
/// # Errors
///
/// `ENGINE-INVARIANT` when its version is missing.
pub fn row_place(m: &Reality, row: &Row) -> Result<String, Stop> {
    Ok(format!("{}:{}", version_file(m, &row.version)?, row.line))
}

/// Whether a row governs `always` under every cut: its own span is `always`, or it has none
/// and its version governs `always` (L §4.3 rule 1; a `moves` line isn't built).
fn row_timeless(m: &Reality, row: &Row) -> Result<bool, Stop> {
    if row.instant.is_some() {
        return Ok(false);
    }
    if let Some(own) = &row.own_span {
        return Ok(own.is_always());
    }
    version_always(m, &row.version)
}

/// Whether an input is read with no law date: not a reading, not observed, and every row
/// governs `always` (L §4.3 rule 1).
///
/// # Errors
///
/// `ENGINE-INVARIANT` when a row's version is missing.
pub fn lawless(m: &Reality, input: &InputInfo) -> Result<bool, Stop> {
    if input.reading || input.observed {
        return Ok(false);
    }
    for row in &input.rows {
        if !row_timeless(m, row)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// A read of a law-dated input needs a law date (L §4.3 rule 2).
///
/// # Errors
///
/// `NotBuilt` for a dated read with no law date, which load refuses (`LOAD-LAWDATE`).
pub fn need_law(m: &Reality, input: &InputInfo, day: Option<Day>) -> Result<(), Stop> {
    if day.is_none() && !lawless(m, input)? {
        return Err(no_law_date());
    }
    Ok(())
}

/// A key's place in its domain's order: an index's position, a key list's position, a
/// thing's stable id `kind:key`, or a range key's value (L §4.6 step 1, §2.5).
enum Rank {
    /// A position.
    Pos(usize),
    /// A stable id's text, compared by code point.
    Text(String),
    /// An integer.
    Int(i64),
    /// A day, or a period's first day.
    Day(Day),
}

/// Whether `a` comes no later than `b`; none when they don't compare (L §4.6 step 1).
fn rank_le(a: &Rank, b: &Rank) -> Option<bool> {
    match (a, b) {
        (Rank::Pos(x), Rank::Pos(y)) => Some(x <= y),
        (Rank::Text(x), Rank::Text(y)) => Some(x <= y),
        (Rank::Int(x), Rank::Int(y)) => Some(x <= y),
        (Rank::Day(x), Rank::Day(y)) => Some(x <= y),
        _ => None,
    }
}

/// `key`'s rank in `domain`, none where it has none (L §4.6 step 1, §2.5; an axis isn't
/// built).
fn rank(m: &Reality, domain: &Domain, key: &KeyVal) -> Option<Rank> {
    match domain {
        Domain::Index(id) => m.indexes.get(id)?.pos(key).map(Rank::Pos),
        Domain::List(keys) => {
            let text = key.text();
            keys.iter().position(|k| k.text() == text).map(Rank::Pos)
        }
        Domain::Kind(_) => {
            let KeyVal::Thing(name) = key else {
                return None;
            };
            let base = m.things.get(name)?.base.as_deref()?;
            Some(Rank::Text(format!("{base}:{name}")))
        }
        Domain::Range => match key {
            KeyVal::Int(n) => Some(Rank::Int(*n)),
            KeyVal::Day(day) => Some(Rank::Day(*day)),
            KeyVal::Period(period) => Some(Rank::Day(period.start())),
            KeyVal::Key { .. } | KeyVal::Thing(_) => None,
        },
        Domain::Axis(_) => None,
    }
}

/// Whether `key` lies in the range `lo..hi` of `domain`, in its order (L §4.6 step 1).
fn within(m: &Reality, domain: &Domain, lo: &KeyVal, hi: &KeyVal, key: &KeyVal) -> bool {
    let (Some(low), Some(high), Some(at)) = (
        rank(m, domain, lo),
        rank(m, domain, hi),
        rank(m, domain, key),
    ) else {
        return false;
    };
    rank_le(&low, &at) == Some(true) && rank_le(&at, &high) == Some(true)
}

/// Whether every key a row states matches the read's key: `*` every key, a range every key
/// from one end to the other (L §4.6 step 1).
fn row_matches(m: &Reality, row: &Row, binds: &[BindInfo], keys: &[KeyVal]) -> bool {
    for ((pat, key), bind) in row.pattern.iter().zip(keys).zip(binds) {
        let fits = match pat {
            Pat::Star => true,
            Pat::Key(stated) => stated == key,
            Pat::Range(lo, hi) => within(m, &bind.domain, lo, hi, key),
        };
        if !fits {
            return false;
        }
    }
    true
}

/// Whether index `id` had `key` on day `known`: a key a version added counts from that
/// version's `known` (L §4.6 step 1, §6.2).
fn index_had(m: &Reality, id: &str, key: &KeyVal, known: Day) -> Result<bool, Stop> {
    let index = m
        .indexes
        .get(id)
        .ok_or_else(|| invariant("a bind names an index load didn't build"))?;
    let Some(by) = index.added_by(key) else {
        return Ok(true);
    };
    Ok(version_known(m, by)? <= known)
}

/// Whether `kind` (or a subkind) had thing `key` on day `known`: one of its entries there,
/// other than a `replace`, is known by then (L §4.6 step 1, §6.2).
fn kind_had(m: &Reality, kind: &str, key: &KeyVal, known: Day) -> Result<bool, Stop> {
    let KeyVal::Thing(name) = key else {
        return Ok(false);
    };
    let Some(thing) = m.things.get(name) else {
        return Ok(false);
    };
    let kinds = m.subtree(kind);
    for entry in &thing.entries {
        if entry.replace || !kinds.contains(&entry.kind) {
            continue;
        }
        let Some(version) = &entry.version else {
            return Ok(true);
        };
        if version_known(m, version)? <= known {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Whether each key a row leaves unstated, or covers with `*`, is one its index or kind
/// had on the row's `known` date: a row for every key, stated before a key was added,
/// doesn't cover it (L §4.6 step 1, §6.2).
fn unstated_ok(m: &Reality, row: &Row, binds: &[BindInfo], keys: &[KeyVal]) -> Result<bool, Stop> {
    let known = row_known(m, row)?;
    for ((pat, key), bind) in row.pattern.iter().zip(keys).zip(binds) {
        if *pat != Pat::Star {
            continue;
        }
        let had = match &bind.domain {
            Domain::Index(id) => index_had(m, id, key, known)?,
            Domain::Kind(kind) => kind_had(m, kind, key, known)?,
            Domain::Axis(_) | Domain::List(_) | Domain::Range => true,
        };
        if !had {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The day of a reading's instant `at D`; none for `after m` (L §4.6 Readings).
fn instant_day(instant: &RowInstant) -> Option<Day> {
    match instant {
        RowInstant::At(day, _) => Some(*day),
        RowInstant::After(_) => None,
    }
}

/// Whether a row holds the read's time: a reading at its instant; a state when its span
/// holds the day, or governs `always` for a read with no law date (L §4.6 step 1,
/// Readings).
fn in_time(m: &Reality, row: &Row, test: Test, cut: Day) -> Result<bool, Stop> {
    if let Some(instant) = &row.instant {
        return Ok(match test {
            Test::At(_) => true,
            Test::Instant(at) => instant_day(instant) == Some(at),
        });
    }
    let Some(span) = row_span(m, row, cut)? else {
        return Ok(false);
    };
    match test {
        Test::At(None) if span.is_always() => Ok(true),
        Test::At(None) => Err(no_law_date()),
        Test::At(Some(day)) | Test::Instant(day) => Ok(span.contains(day)),
    }
}

/// The candidates of a read among `rows`, by position: every key it states matches, the
/// keys it leaves unstated were there when it was known, it is known by the cut, and its
/// span holds the read's time (L §4.6 step 1).
///
/// # Errors
///
/// `NotBuilt` for a dated row read with no law date (`LOAD-LAWDATE`); `ENGINE-INVARIANT`
/// for a row whose version is missing.
pub fn candidates(
    m: &Reality,
    rows: &[Row],
    binds: &[BindInfo],
    keys: &[KeyVal],
    test: Test,
    cut: Day,
) -> Result<Vec<usize>, Stop> {
    let mut out = Vec::new();
    for (at, row) in rows.iter().enumerate() {
        if row_matches(m, row, binds, keys)
            && unstated_ok(m, row, binds, keys)?
            && row_known(m, row)? <= cut
            && in_time(m, row, test, cut)?
        {
            out.push(at);
        }
    }
    Ok(out)
}

/// How one key position of a row relates to another's (L §4.6 steps 2 and 3.2).
enum Rel {
    /// They state the key alike: the same literal, the same range or `*`.
    Alike,
    /// The first lies strictly inside the second.
    Inside,
    /// Neither.
    Apart,
}

/// The relation at one key position: alike, a literal inside a range that holds it, a
/// range inside a wider one, anything inside `*` (L §4.6 step 3.2).
fn relation(m: &Reality, domain: &Domain, a: &Pat, b: &Pat) -> Rel {
    if a == b {
        return Rel::Alike;
    }
    let (Pat::Key(lo_a) | Pat::Range(lo_a, _), Pat::Range(lo_b, hi_b)) = (a, b) else {
        return if *b == Pat::Star {
            Rel::Inside
        } else {
            Rel::Apart
        };
    };
    let hi_a = match a {
        Pat::Range(_, hi) => hi,
        Pat::Key(_) | Pat::Star => lo_a,
    };
    if within(m, domain, lo_b, hi_b, lo_a) && within(m, domain, lo_b, hi_b, hi_a) {
        Rel::Inside
    } else {
        Rel::Apart
    }
}

/// Whether pattern `a` is narrower than `b`: at every position its key is the other's or
/// lies inside it, and at one strictly inside (L §4.6 step 3.2).
fn narrower(m: &Reality, binds: &[BindInfo], a: &[Pat], b: &[Pat]) -> bool {
    let mut strict = false;
    for ((pa, pb), bind) in a.iter().zip(b).zip(binds) {
        match relation(m, &bind.domain, pa, pb) {
            Rel::Alike => {}
            Rel::Inside => strict = true,
            Rel::Apart => return false,
        }
    }
    strict
}

/// The candidates no `replace` row beats: a later-known row at the same keys that says
/// `replace` wins over the rows known before it (L §4.6 step 2).
fn unreplaced(m: &Reality, rows: &[Row], cands: &[usize]) -> Result<Vec<usize>, Stop> {
    let mut keep = Vec::new();
    for &at in cands {
        let row = rows
            .get(at)
            .ok_or_else(|| invariant("a candidate past its rows"))?;
        let known = row_known(m, row)?;
        let mut beaten = false;
        for &other in cands {
            let Some(o) = rows.get(other) else {
                return Err(invariant("a candidate past its rows"));
            };
            if other != at && o.replace && o.pattern == row.pattern && row_known(m, o)? > known {
                beaten = true;
                break;
            }
        }
        if !beaten {
            keep.push(at);
        }
    }
    Ok(keep)
}

/// The candidates no other remaining candidate is narrower than (L §4.6 step 3.2).
fn narrowest(m: &Reality, rows: &[Row], binds: &[BindInfo], keep: &[usize]) -> Vec<usize> {
    let mut out = Vec::new();
    for &at in keep {
        let Some(row) = rows.get(at) else {
            continue;
        };
        if !narrower_than(m, rows, binds, keep, at, &row.pattern) {
            out.push(at);
        }
    }
    out
}

/// Whether a candidate of `keep` other than the one at `at` is narrower than `pattern` (L
/// §4.6 step 3.2).
fn narrower_than(
    m: &Reality,
    rows: &[Row],
    binds: &[BindInfo],
    keep: &[usize],
    at: usize,
    pattern: &[Pat],
) -> bool {
    for &other in keep {
        if other == at {
            continue;
        }
        if let Some(o) = rows.get(other)
            && narrower(m, binds, &o.pattern, pattern)
        {
            return true;
        }
    }
    false
}

/// The candidates with a span (or an instant), where any has one (L §4.6 step 3.3).
fn spanned(m: &Reality, rows: &[Row], keep: Vec<usize>, cut: Day) -> Result<Vec<usize>, Stop> {
    let mut out = Vec::new();
    for &at in &keep {
        let row = rows
            .get(at)
            .ok_or_else(|| invariant("a candidate past its rows"))?;
        let dated =
            row.instant.is_some() || row_span(m, row, cut)?.is_some_and(|span| !span.is_always());
        if dated {
            out.push(at);
        }
    }
    Ok(if out.is_empty() { keep } else { out })
}

/// The row a read takes among its candidates: replacement, then precedence (the narrower
/// row, then a row with a span over one with `always`); none when there is no candidate
/// (L §4.6 steps 2–4).
///
/// # Errors
///
/// `LOAD-OVERLAP` for two candidates still tied (L §4.6 step 4); `ENGINE-INVARIANT` for a
/// row whose version is missing.
pub fn choose(
    m: &Reality,
    rows: &[Row],
    binds: &[BindInfo],
    cands: &[usize],
    cut: Day,
    id: &str,
) -> Result<Option<usize>, Stop> {
    if cands.is_empty() {
        return Ok(None);
    }
    let mut keep = unreplaced(m, rows, cands)?;
    if keep.len() > 1 {
        keep = narrowest(m, rows, binds, &keep);
    }
    if keep.len() > 1 {
        keep = spanned(m, rows, keep, cut)?;
    }
    if let [first, second, ..] = keep.as_slice() {
        let (Some(a), Some(b)) = (rows.get(*first), rows.get(*second)) else {
            return Err(invariant("a candidate past its rows"));
        };
        let message = format!(
            "rows {} and {} of {id} tie",
            row_place(m, a)?,
            row_place(m, b)?
        );
        return Err(Stop::Refused(Refusal::new(
            Code::Load(Load::Overlap),
            message,
        )));
    }
    Ok(keep.first().copied())
}

/// Whether a row read for day `day` is `held`: a state row whose span has a start and no
/// end, read for a day past the ask's cut (L §8.2, §4.6 step 5).
///
/// # Errors
///
/// `ENGINE-INVARIANT` when its version is missing.
pub fn held(m: &Reality, row: &Row, day: Option<Day>, clocks: Clocks) -> Result<bool, Stop> {
    let Some(read_for) = day else {
        return Ok(false);
    };
    if row.instant.is_some() {
        return Ok(false);
    }
    let Some(span) = row_span(m, row, clocks.cut)? else {
        return Ok(false);
    };
    Ok(!span.is_always()
        && span.start().is_some()
        && span.end().is_none()
        && read_for > clocks.ask_cut)
}
