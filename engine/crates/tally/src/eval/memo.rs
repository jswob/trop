//! The memo: each def cell's result and read set under a typed key, shared by every law date of the cell's class.
//! Spec: E §3.2 (cells and the memo; the clauses are in the context; the law date is a class:
//! the interval between consecutive boundaries of the governs spans of the rows and rules the
//! item reaches and of the lifespans of the things at the keys it reads, every law date inside
//! it giving the same value; a result keeps its read set, "the rows and cells it read, band
//! rows included"); P §3.1 (the memo key: key values, the context trimmed to what the item can
//! depend on), §3.2 (the law date class layer), §3.3 (the test a trimming must pass: two
//! contexts share a cell only where the cell's result, register included, is the same in
//! both), §4 (read sets on every memo cell, built now).
//! Twin: engine/tools/refcore/evaluate.py (`def_cell`, whose memo keys a cell by its exact law
//! date and cut).
//! Invariants: a cell kept for law date d holds for every law date of its class, and every
//! read that tests the law date narrows the class of the cell computing it: a row, a rule or
//! a band table chosen by its span to the side of each span boundary d is on, a row that may
//! be `held` to d's side of the ask's cut, and a text that names d (a hole's question, a
//! `held` entry, `RUN-CONFLICT`, `RUN-NO-RULE`), a reading, a calendar's days and a priced
//! figure to d alone; a cell read at the reader's own law date narrows the reader to the read
//! cell's class; a cycle met narrows every open cell to its own day; a walk-bound cell is kept
//! at its exact law date; under `Memo::Exact` every class is one day; a cell read under the
//! ask line's `with uncertain` is a cell of its own. A kept cell's read set holds the inputs it
//! read at their keys, the rows, bands and band rows it chose and the cells it read, never what
//! those cells read, nor what a walk, a plan, adjustments or links read while it first needed
//! them: those are shared by every reader.
//! PERF: P8-14 (def misses: the loan schedule's cells were each computed once per law date of
//! the instance that read them, 480 law dates for one class); plain twin: refcore, whose memo
//! keys each cell by its exact law date as `Memo::Exact` does here; the equivalence test runs
//! the kernel corpus under both (crates/tally-cli/tests/walk_oracle.rs).

use core::cmp::Ordering;
use std::collections::BTreeMap;

use super::state::{Cell, Ctx, Engine, invariant};
use crate::model::{BandInfo, DefInfo, KeyVal, Reality, Row, RuleVer, Span};
use crate::read::row_span;
use crate::syntax::Stop;
use crate::time::Day;
use crate::walk::WalkAt;

/// How the memo keeps a cell across law dates (E §3.2; the switch of P §3.3's on/off test).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Memo {
    /// Every law date of a cell's class shares its result.
    Classes,
    /// Each cell is kept at its exact law date, as refcore keeps it: the plain twin.
    Exact,
}

/// Law dates from a first day to the day before `until`, either end open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LawClass {
    /// The first day, or none: from the start of time.
    from: Option<Day>,
    /// The first day after it, or none: open.
    until: Option<Day>,
}

impl LawClass {
    /// Every law date.
    pub(crate) fn all() -> LawClass {
        LawClass {
            from: None,
            until: None,
        }
    }

    /// Law date `day` alone; for the calendar's last day, `day..`, which holds it alone.
    pub(crate) fn day(day: Day) -> LawClass {
        LawClass {
            from: Some(day),
            until: day.next(),
        }
    }

    /// Whether it holds law date `day`.
    fn holds(self, day: Day) -> bool {
        self.from.is_none_or(|f| f <= day) && self.until.is_none_or(|u| day < u)
    }

    /// The law dates both hold.
    fn and(self, other: LawClass) -> LawClass {
        let from = match (self.from, other.from) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (a, b) => a.or(b),
        };
        let until = match (self.until, other.until) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        LawClass { from, until }
    }

    /// The law dates on `day`'s side of boundary `at`, the first day of the later side.
    fn side(self, day: Day, at: Day) -> LawClass {
        let half = if day < at {
            LawClass {
                from: None,
                until: Some(at),
            }
        } else {
            LawClass {
                from: Some(at),
                until: None,
            }
        };
        self.and(half)
    }

    /// The law dates on `day`'s side of each end of `span`, where `span` holds a law date
    /// exactly when it holds `day` (L §4.6 step 1).
    fn by_span(self, day: Day, span: &Span) -> LawClass {
        let mut out = self;
        if let Some(start) = span.start() {
            out = out.side(day, start);
        }
        if let Some(end) = span.end() {
            out = out.side(day, end);
        }
        out
    }

    /// The law dates on `day`'s side of the ask's cut `ask_cut`: a row read for a later day
    /// is `held` (L §8.2).
    pub(super) fn by_ask_cut(self, day: Day, ask_cut: Day) -> LawClass {
        match ask_cut.next() {
            Some(after) => self.side(day, after),
            None => self,
        }
    }
}

/// A key value's place in the memo's order: by kind, then by what the key holds, a
/// movement by its stable id (E §3.2: keys are values). Only the memo looks it up; no output
/// is ordered by it (R-47).
fn key_order(a: &KeyVal, b: &KeyVal) -> Ordering {
    match (a, b) {
        (
            KeyVal::Key {
                index: ia,
                name: na,
            },
            KeyVal::Key {
                index: ib,
                name: nb,
            },
        ) => ia.cmp(ib).then_with(|| na.cmp(nb)),
        (KeyVal::Thing(x), KeyVal::Thing(y)) => x.cmp(y),
        (KeyVal::Day(x), KeyVal::Day(y)) => x.cmp(y),
        (KeyVal::Period(x), KeyVal::Period(y)) => x
            .start()
            .cmp(&y.start())
            .then_with(|| x.end().cmp(&y.end())),
        (KeyVal::Int(x), KeyVal::Int(y)) => x.cmp(y),
        (KeyVal::Mv(x), KeyVal::Mv(y)) => x.id.cmp(&y.id),
        _ => kind_rank(a).cmp(&kind_rank(b)),
    }
}

/// The rank of a key value's kind in the memo's order.
fn kind_rank(k: &KeyVal) -> u8 {
    match k {
        KeyVal::Key { .. } => 0,
        KeyVal::Thing(_) => 1,
        KeyVal::Day(_) => 2,
        KeyVal::Period(_) => 3,
        KeyVal::Int(_) => 4,
        KeyVal::Mv(_) => 5,
    }
}

/// Two key lists in the memo's order: position by position, then the shorter first.
fn keys_order(a: &[KeyVal], b: &[KeyVal]) -> Ordering {
    for (x, y) in a.iter().zip(b) {
        let o = key_order(x, y);
        if o != Ordering::Equal {
            return o;
        }
    }
    a.len().cmp(&b.len())
}

/// Two walk points in the memo's order: none first, then the walk, then the point.
fn walk_at_order(a: Option<WalkAt>, b: Option<WalkAt>) -> Ordering {
    match (a, b) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (Some(x), Some(y)) => x.walk.cmp(&y.walk).then_with(|| x.point.cmp(&y.point)),
    }
}

/// A cell's key in the memo, its law date apart: the def, its keys, the cut, for a
/// walk-bound def the walk point, and the ask line's `with uncertain` (E §3.2: the clauses
/// are in the context).
#[derive(Debug, Clone)]
pub(crate) struct CellKey {
    /// The def.
    def: String,
    /// Its keys.
    keys: Vec<KeyVal>,
    /// The cut.
    cut: Day,
    /// The walk point of a walk-bound def's cell.
    at: Option<WalkAt>,
    /// Whether the cell is read under the ask line's `with uncertain` (L §5.18).
    unc: bool,
}

impl CellKey {
    /// The key of def `def` at `keys` in context `ctx`, its law date apart.
    pub(super) fn new(def: &str, keys: &[KeyVal], ctx: Ctx) -> CellKey {
        CellKey {
            def: def.to_owned(),
            keys: keys.to_vec(),
            cut: ctx.cut,
            at: ctx.at,
            unc: ctx.unc,
        }
    }
}

impl Ord for CellKey {
    /// The memo's order: the def, the keys, then the context.
    fn cmp(&self, other: &CellKey) -> Ordering {
        self.def
            .cmp(&other.def)
            .then_with(|| keys_order(&self.keys, &other.keys))
            .then_with(|| self.context_order(other))
    }
}

impl CellKey {
    /// The contexts' order: the cut, the walk point, the clause.
    fn context_order(&self, other: &CellKey) -> Ordering {
        self.cut
            .cmp(&other.cut)
            .then_with(|| walk_at_order(self.at, other.at))
            .then_with(|| self.unc.cmp(&other.unc))
    }
}

impl PartialOrd for CellKey {
    fn partial_cmp(&self, other: &CellKey) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for CellKey {
    fn eq(&self, other: &CellKey) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for CellKey {}

/// One cell exactly, its law date included: what is in progress, or waits on the work stack
/// (E §3.2: a cell read while it is computed is `RUN-CYCLE`).
#[derive(Debug, Clone)]
pub(crate) struct CellId {
    /// The cell's key.
    pub(super) key: CellKey,
    /// Its law date.
    law: Option<Day>,
}

impl CellId {
    /// The cell of `key` at law date `law`.
    pub(super) fn new(key: CellKey, law: Option<Day>) -> CellId {
        CellId { key, law }
    }
}

impl Ord for CellId {
    /// The key's order, then the law date, none first.
    fn cmp(&self, other: &CellId) -> Ordering {
        self.key
            .cmp(&other.key)
            .then_with(|| self.law.cmp(&other.law))
    }
}

impl PartialOrd for CellId {
    fn partial_cmp(&self, other: &CellId) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for CellId {
    fn eq(&self, other: &CellId) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for CellId {}

/// One read a def cell made (E §3.2: "the rows and cells it read, band rows included").
#[derive(Debug, Clone)]
pub(crate) enum Read {
    /// An input read at its keys: a new candidate row or key there is what can change the
    /// cell (P §4.1 step 2).
    Input {
        /// The input.
        id: String,
        /// The keys read.
        keys: Vec<KeyVal>,
    },
    /// A def's `given` rows read at its keys, whether or not one was chosen: a new given row
    /// there is what can change the cell (L §5.11 step 1, P §4.1 step 2).
    Givens {
        /// The def.
        def: String,
        /// The keys read.
        keys: Vec<KeyVal>,
    },
    /// A row the read rule chose, an input's or a def's given: its version, its stable id
    /// (L §2.5).
    Row {
        /// The version that states it.
        version: String,
        /// Its stable id, `v1:rent[2026-03]`.
        sid: String,
    },
    /// A band read at its key: a new table or row there is what can change the cell.
    Band {
        /// The band.
        id: String,
        /// Its key, for a keyed band.
        key: Option<KeyVal>,
    },
    /// A band row chosen: the band, its table's place among the band's, the row's in it.
    BandRow {
        /// The band.
        band: String,
        /// The table's place.
        table: usize,
        /// The row's place.
        row: usize,
    },
    /// A def cell read, at the reader's law date.
    Cell(CellId),
}

impl Read {
    /// The rank of a read's kind in the set's order.
    fn rank(&self) -> u8 {
        match self {
            Read::Input { .. } => 0,
            Read::Givens { .. } => 1,
            Read::Row { .. } => 2,
            Read::Band { .. } => 3,
            Read::BandRow { .. } => 4,
            Read::Cell(_) => 5,
        }
    }
}

/// Two keys of a band read: none first, then by the key (the memo's order).
fn band_key_order(a: Option<&KeyVal>, b: Option<&KeyVal>) -> Ordering {
    match (a, b) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (Some(x), Some(y)) => key_order(x, y),
    }
}

impl Ord for Read {
    /// The set's order: by kind, then by what the read names. Only the set looks it up; no
    /// output is ordered by it (R-47).
    fn cmp(&self, other: &Read) -> Ordering {
        match (self, other) {
            (Read::Input { id: a, keys: x }, Read::Input { id: b, keys: y })
            | (Read::Givens { def: a, keys: x }, Read::Givens { def: b, keys: y }) => {
                a.cmp(b).then_with(|| keys_order(x, y))
            }
            (
                Read::Row {
                    version: va,
                    sid: a,
                },
                Read::Row {
                    version: vb,
                    sid: b,
                },
            ) => va.cmp(vb).then_with(|| a.cmp(b)),
            (Read::Band { id: a, key: x }, Read::Band { id: b, key: y }) => a
                .cmp(b)
                .then_with(|| band_key_order(x.as_ref(), y.as_ref())),
            (
                Read::BandRow {
                    band: a,
                    table: ta,
                    row: ra,
                },
                Read::BandRow {
                    band: b,
                    table: tb,
                    row: rb,
                },
            ) => a.cmp(b).then_with(|| (ta, ra).cmp(&(tb, rb))),
            (Read::Cell(a), Read::Cell(b)) => a.cmp(b),
            _ => self.rank().cmp(&other.rank()),
        }
    }
}

impl PartialOrd for Read {
    fn partial_cmp(&self, other: &Read) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Read {
    fn eq(&self, other: &Read) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Read {}

/// A def cell's read set: what its own evaluation read, the cells it read included but not
/// what they read (E §3.2, P §4). A walk, a plan, adjustments and links are shared by every
/// cell that reads them, so what they read is theirs, not the cell's that first needed them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Reads {
    /// The reads: in the order made while the cell is open; once kept, sorted, each once,
    /// with no spare room (a cell keeps a handful, P §3.5).
    list: Vec<Read>,
}

impl Reads {
    /// The set as a cell keeps it: sorted, each read once, no spare room.
    fn closed(mut self) -> Reads {
        self.list.sort();
        self.list.dedup();
        self.list.shrink_to_fit();
        self
    }
}

/// A read set open now: a def cell's, or a shared structure's (a walk, a plan, adjustments,
/// links), which keeps none, since what it reads is no reading cell's (E §3.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Open {
    /// A def cell's, gathering.
    Cell(Reads),
    /// A shared structure's: its reads aren't kept.
    Apart,
}

/// A kept cell: the law date it was computed at, the class it holds for, its result.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Kept {
    /// The law date it was computed at; none for a cell with no law date.
    law: Option<Day>,
    /// The law dates it holds for.
    class: LawClass,
    /// The result and what it met.
    cell: Cell,
}

/// The memo: each key's kept cells, one per law date class reached (E §3.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Cells {
    /// How cells are shared across law dates.
    memo: Memo,
    /// The kept cells by key.
    kept: BTreeMap<CellKey, Vec<Kept>>,
}

impl Cells {
    /// An empty memo sharing cells as `memo` says.
    pub(super) fn new(memo: Memo) -> Cells {
        Cells {
            memo,
            kept: BTreeMap::new(),
        }
    }

    /// The cell kept for `key` that holds law date `law`, with its class.
    pub(super) fn get(&self, key: &CellKey, law: Option<Day>) -> Option<(&Cell, LawClass)> {
        let kept = self.kept.get(key)?;
        kept.iter()
            .find(|k| match (k.law, law) {
                (None, None) => true,
                (Some(_), Some(day)) => k.class.holds(day),
                (None, Some(_)) | (Some(_), None) => false,
            })
            .map(|k| (&k.cell, k.class))
    }

    /// Keeps `cell`, computed at law date `law`, for the law dates of `class`; under
    /// `Memo::Exact` for `law` alone. Returns the class kept.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a class that doesn't hold the law date it was computed at.
    pub(super) fn keep(
        &mut self,
        key: CellKey,
        law: Option<Day>,
        class: LawClass,
        cell: Cell,
    ) -> Result<LawClass, Stop> {
        let class = match (self.memo, law) {
            (Memo::Exact, Some(day)) => LawClass::day(day),
            (Memo::Exact | Memo::Classes, _) => class,
        };
        if law.is_some_and(|day| !class.holds(day)) {
            return Err(invariant(
                "a cell's law date class doesn't hold its own day",
            ));
        }
        let kept = Kept { law, class, cell };
        match self.kept.get_mut(&key) {
            Some(cells) => cells.push(kept),
            None => {
                self.kept.insert(key, vec![kept]);
            }
        }
        Ok(class)
    }
}

/// An open def cell's class: the law date it is computed at and the law dates its reads so
/// far hold for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Frame {
    /// The law date.
    law: Option<Day>,
    /// The class so far.
    class: LawClass,
}

impl Engine {
    /// Opens a def cell's read set (E §3.2).
    pub(super) fn open_reads(&mut self) {
        self.reads.push(Open::Cell(Reads::default()));
    }

    /// Closes the innermost read set, a def cell's, and returns it as the cell keeps it.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` with no def cell's open innermost.
    pub(super) fn close_reads(&mut self) -> Result<Reads, Stop> {
        match self.reads.pop() {
            Some(Open::Cell(reads)) => Ok(reads.closed()),
            Some(Open::Apart) | None => Err(invariant("a cell's read set closed with none open")),
        }
    }

    /// Opens a shared structure's read set (a walk, a plan, adjustments, links), which keeps
    /// nothing, so its reads aren't the reading cell's; returns the mark
    /// [`Engine::reads_back`] takes (E §3.2, P §4).
    pub(crate) fn reads_apart(&mut self) -> usize {
        let mark = self.reads.len();
        self.reads.push(Open::Apart);
        mark
    }

    /// Closes the read sets opened since `mark`, a shared structure's and any a refusal left
    /// open inside it.
    pub(crate) fn reads_back(&mut self, mark: usize) {
        self.reads.truncate(mark);
    }

    /// Adds `read` to the innermost read set, when it is a def cell's (E §3.2).
    pub(super) fn read(&mut self, read: Read) {
        if let Some(Open::Cell(top)) = self.reads.last_mut() {
            top.list.push(read);
        }
    }

    /// Opens the class of a def cell computed at law date `law` (E §3.2).
    pub(super) fn open_class(&mut self, law: Option<Day>) {
        self.frames.push(Frame {
            law,
            class: LawClass::all(),
        });
    }

    /// Closes the innermost def cell's class and returns it.
    pub(super) fn close_class(&mut self) -> Result<LawClass, Stop> {
        self.frames
            .pop()
            .map(|f| f.class)
            .ok_or_else(|| invariant("a cell's law date class closed with none open"))
    }

    /// Narrows the class of the innermost def cell to `class`, when it is computed at law
    /// date `law`: a read at another law date tells nothing of its own (E §3.2).
    pub(super) fn narrow(&mut self, law: Option<Day>, class: LawClass) {
        if law.is_none() {
            return;
        }
        if let Some(top) = self.frames.last_mut()
            && top.law == law
        {
            top.class = top.class.and(class);
        }
    }

    /// Narrows the class of the innermost def cell to law date `law` alone: what it read
    /// names the day, or turns on it beyond a span (E §3.2).
    pub(super) fn narrow_to_day(&mut self, law: Option<Day>) {
        if let Some(day) = law {
            self.narrow(law, LawClass::day(day));
        }
    }

    /// A cycle met, of cells, a plan, links or adjustments: every open def cell is kept at its
    /// own law date alone, since what it gives turns on what is in progress (E §3.2, L §5.11
    /// `RUN-CYCLE`).
    pub(crate) fn narrow_open_to_days(&mut self) {
        for frame in &mut self.frames {
            if let Some(day) = frame.law {
                frame.class = LawClass::day(day);
            }
        }
    }
}

/// The law dates on `day`'s side of every span of `rows` under cut `cut`: those where each
/// row's span holds the read's day exactly when it holds `day` (L §4.6 step 1, E §3.2).
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a row whose version load didn't build.
pub(super) fn rows_class(m: &Reality, rows: &[Row], cut: Day, day: Day) -> Result<LawClass, Stop> {
    let mut class = LawClass::all();
    for row in rows {
        if row.instant.is_some() {
            continue;
        }
        if let Some(span) = row_span(m, row, cut)? {
            class = class.by_span(day, &span);
        }
    }
    Ok(class)
}

/// The governs span of version `id` under cut `cut`.
fn version_span(m: &Reality, id: &str, cut: Day) -> Result<Option<Span>, Stop> {
    m.versions
        .get(id)
        .map(|v| v.span_at(cut).cloned())
        .ok_or_else(|| invariant("a version load didn't build"))
}

/// The law dates on `day`'s side of every span a version of a rule of `def` is tested by
/// under cut `cut`: its version's, its `v` version's and its own (L §5.11 step 2, E §3.2).
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a rule whose version load didn't build.
pub(super) fn rules_class(
    m: &Reality,
    def: &DefInfo,
    cut: Day,
    day: Day,
) -> Result<LawClass, Stop> {
    let mut class = LawClass::all();
    for (_, versions) in &def.rules {
        for rule in versions {
            class = rule_class(m, rule, cut, day, class)?;
        }
    }
    Ok(class)
}

/// `class` on `day`'s side of the spans one rule version is tested by (L §5.11 step 2).
fn rule_class(
    m: &Reality,
    rule: &RuleVer,
    cut: Day,
    day: Day,
    class: LawClass,
) -> Result<LawClass, Stop> {
    let mut out = class;
    for id in [&rule.version, &rule.v].into_iter().flatten() {
        if let Some(span) = version_span(m, id, cut)? {
            out = out.by_span(day, &span);
        }
    }
    if let Some(own) = &rule.span {
        out = out.by_span(day, own);
    }
    Ok(out)
}

/// The law dates on `day`'s side of every band table's governs span under cut `cut` (L
/// §5.9, §4.6 step 1, E §3.2).
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a table whose version load didn't build.
pub(super) fn tables_class(
    m: &Reality,
    band: &BandInfo,
    cut: Day,
    day: Day,
) -> Result<LawClass, Stop> {
    let mut class = LawClass::all();
    for table in &band.tables {
        if let Some(span) = version_span(m, &table.version, cut)? {
            class = class.by_span(day, &span);
        }
    }
    Ok(class)
}

/// Whether a state with span `span` may be `held`: it is dated, with a start and no end, so
/// a read for a day past the ask's cut is held (L §8.2).
pub(super) fn may_hold(span: &Span) -> bool {
    !span.is_always() && span.start().is_some() && span.end().is_none()
}

#[cfg(test)]
mod tests {
    use super::{CellId, CellKey, Cells, LawClass, Memo, Read, Reads, keys_order};
    use crate::eval::state::{Cell, Collected, Ctx};
    use crate::model::{KeyVal, Span, Value};
    use crate::time::Day;
    use core::cmp::Ordering;

    fn day(y: u16, m: u8, d: u8) -> Day {
        Day::from_ymd(y, m, d).expect("a day")
    }

    fn cell(n: i64) -> Cell {
        Cell {
            result: Ok(Value::int(n)),
            met: Collected::default(),
            reads: Reads::default(),
        }
    }

    #[test]
    fn a_span_splits_the_law_dates_at_its_ends() {
        let span = Span::between(Some(day(2026, 1, 1)), Some(day(2027, 1, 1)));
        let inside = LawClass::all().by_span(day(2026, 6, 1), &span);
        assert_eq!(
            inside,
            LawClass {
                from: Some(day(2026, 1, 1)),
                until: Some(day(2027, 1, 1)),
            },
            "inside the span"
        );
        let before = LawClass::all().by_span(day(2025, 6, 1), &span);
        assert_eq!(
            before,
            LawClass {
                from: None,
                until: Some(day(2026, 1, 1)),
            },
            "before it"
        );
        let after = LawClass::all().by_span(day(2027, 1, 1), &span);
        assert_eq!(
            after,
            LawClass {
                from: Some(day(2027, 1, 1)),
                until: None,
            },
            "from its end on"
        );
        let always = LawClass::all().by_span(day(2025, 6, 1), &Span::always());
        assert_eq!(always, LawClass::all(), "always splits nothing");
    }

    #[test]
    fn a_class_holds_what_every_span_keeps_alike() {
        let a = Span::between(Some(day(2026, 1, 1)), None);
        let b = Span::between(None, Some(day(2026, 7, 1)));
        let class = LawClass::all()
            .by_span(day(2026, 3, 1), &a)
            .by_span(day(2026, 3, 1), &b);
        for (d, inside) in [
            (day(2025, 12, 31), false),
            (day(2026, 1, 1), true),
            (day(2026, 6, 30), true),
            (day(2026, 7, 1), false),
        ] {
            assert_eq!(class.holds(d), inside, "{}", d.text());
            let same = a.contains(d) == a.contains(day(2026, 3, 1))
                && b.contains(d) == b.contains(day(2026, 3, 1));
            assert_eq!(class.holds(d), same, "the twin's test at {}", d.text());
        }
        assert!(
            LawClass::day(day(2026, 3, 1)).holds(day(2026, 3, 1)),
            "a day"
        );
        assert!(
            !LawClass::day(day(2026, 3, 1)).holds(day(2026, 3, 2)),
            "alone"
        );
    }

    #[test]
    fn the_ask_cut_splits_the_held_days_off() {
        let cut = day(2026, 9, 30);
        let class = LawClass::all().by_ask_cut(day(2026, 6, 5), cut);
        assert!(class.holds(cut), "the cut's own day is not held");
        assert!(!class.holds(day(2026, 10, 1)), "the day after is");
    }

    #[test]
    fn a_kept_cell_answers_every_law_date_of_its_class_and_no_other() {
        let mut memo = Cells::new(Memo::Classes);
        let key = CellKey::new("debt", &[KeyVal::Int(3)], Ctx::new(None, day(2026, 9, 30)));
        let class = LawClass {
            from: Some(day(2026, 1, 1)),
            until: Some(day(2027, 1, 1)),
        };
        memo.keep(key.clone(), Some(day(2026, 6, 5)), class, cell(7))
            .expect("kept");
        let at = |d: Day| memo.get(&key, Some(d)).map(|(c, _)| c.result.clone());
        assert_eq!(
            at(day(2026, 12, 5)),
            Some(Ok(Value::int(7))),
            "in the class"
        );
        assert_eq!(at(day(2027, 1, 5)), None, "outside it");
        assert!(
            memo.get(&key, None).is_none(),
            "no law date is another cell"
        );
        let mut exact = Cells::new(Memo::Exact);
        exact
            .keep(key.clone(), Some(day(2026, 6, 5)), class, cell(7))
            .expect("kept");
        assert!(
            exact.get(&key, Some(day(2026, 6, 5))).is_some(),
            "its own day"
        );
        assert!(
            exact.get(&key, Some(day(2026, 12, 5))).is_none(),
            "the twin keeps the day"
        );
        let elsewhere = LawClass {
            from: Some(day(2027, 1, 1)),
            until: None,
        };
        assert!(
            memo.keep(key, Some(day(2026, 6, 5)), elsewhere, cell(1))
                .is_err(),
            "a class that doesn't hold its own day is an invariant break"
        );
    }

    #[test]
    fn keys_compare_by_kind_then_value() {
        let a = [KeyVal::Thing("ana".to_owned()), KeyVal::Int(2)];
        let b = [KeyVal::Thing("ana".to_owned()), KeyVal::Int(10)];
        assert_eq!(keys_order(&a, &b), Ordering::Less, "by value");
        assert_eq!(keys_order(&a, &a), Ordering::Equal, "equal");
        assert_eq!(
            keys_order(&[KeyVal::Int(9)], &[KeyVal::Day(day(2026, 1, 1))]),
            Ordering::Greater,
            "by kind"
        );
        assert_eq!(keys_order(&a[..1], &a), Ordering::Less, "the shorter first");
    }

    /// A def reading an input, a def reading that def and a band.
    const READS: &str = r#"model rs
  tally 2
  on 2026-06-30
  start 2026-01-01
  end 2026-12-31

source card "a card"

unit PLN: currency minor 0.01

input fee: money PLN

band tax(v: money PLN): money PLN

def base: money PLN at reader
  rule base = fee

def total: money PLN at reader
  rule sum = base + tax(base)

version v26
  cite card
  governs 2026
  known 2026-01-01
  fee = 10.00
  tax: [0, inf) = round(v * 0.1, 0.01, half-up)

ask t: total law 2026-03-01
"#;

    #[test]
    fn a_cells_read_set_holds_what_it_read_and_not_what_its_cells_read() {
        let files = crate::syntax::parse_reality(&[crate::api::Source {
            path: "rs.tally".to_owned(),
            text: READS.to_owned(),
        }])
        .expect("parses");
        let m = crate::load::load(files).expect("loads");
        let ask = m.asks.first().expect("an ask");
        let crate::syntax::AskBody::Exprs(exprs) = &ask.body else {
            panic!("an ask of values");
        };
        let e = exprs.first().expect("an expression");
        let law = day(2026, 3, 1);
        let ctx = Ctx::new(Some(law), m.on);
        let mut engine = crate::eval::Engine::new(m.on);
        let v = engine
            .value(&m, e, &crate::eval::Env::empty(), ctx)
            .expect("a value");
        assert_eq!(v.text(&m).expect("prints"), "11.00 PLN", "fee and its tax");
        let reads_of = |def: &str| {
            let key = CellKey::new(def, &[], ctx);
            let (cell, _) = engine.cells.get(&key, Some(law)).expect("a kept cell");
            cell.reads.list.clone()
        };
        let input = m.inputs.get("fee").expect("the input");
        let row = input.rows.first().expect("its row");
        let fee = Read::Input {
            id: "fee".to_owned(),
            keys: Vec::new(),
        };
        let fee_row = Read::Row {
            version: "v26".to_owned(),
            sid: row.sid.clone(),
        };
        let base = Read::Cell(CellId::new(CellKey::new("base", &[], ctx), Some(law)));
        let tax = Read::Band {
            id: "tax".to_owned(),
            key: None,
        };
        let tax_row = Read::BandRow {
            band: "tax".to_owned(),
            table: 0,
            row: 0,
        };
        let of_base = reads_of("base");
        assert!(of_base.contains(&fee), "base read fee");
        assert!(of_base.contains(&fee_row), "and its row of v26");
        assert_eq!(of_base.len(), 2, "and nothing else");
        let of_total = reads_of("total");
        assert!(of_total.contains(&base), "total read base's cell");
        assert!(of_total.contains(&tax), "the band tax");
        assert!(of_total.contains(&tax_row), "and its row");
        assert_eq!(of_total.len(), 3, "base read once, at one law date");
        assert!(!of_total.contains(&fee), "what base read is base's");
    }

    /// A def keyed by tenant with a given row for one tenant only.
    const GIVENS: &str = r#"model gv
  tally 2
  on 2026-06-30
  start 2026-01-01
  end 2026-12-31

source card "a card"

unit PLN: currency minor 0.01

index tenant = [a, b]

input fee: money PLN

def cap[t in tenant]: money PLN at reader
  rule base = fee

version v26
  cite card
  governs 2026
  known 2026-01-01
  fee = 10.00
  given cap[a] = 3.00

ask a: cap[a] law 2026-03-01

ask b: cap[b] law 2026-03-01
"#;

    #[test]
    fn a_defs_givens_are_in_its_read_set_whether_or_not_one_is_chosen() {
        let files = crate::syntax::parse_reality(&[crate::api::Source {
            path: "gv.tally".to_owned(),
            text: GIVENS.to_owned(),
        }])
        .expect("parses");
        let m = crate::load::load(files).expect("loads");
        let law = day(2026, 3, 1);
        let ctx = Ctx::new(Some(law), m.on);
        let mut engine = crate::eval::Engine::new(m.on);
        let mut values = Vec::new();
        for ask in &m.asks {
            let crate::syntax::AskBody::Exprs(exprs) = &ask.body else {
                panic!("an ask of values");
            };
            let e = exprs.first().expect("an expression");
            let v = engine
                .value(&m, e, &crate::eval::Env::empty(), ctx)
                .expect("a value");
            values.push(v.text(&m).expect("prints"));
        }
        assert_eq!(values, ["3.00 PLN", "10.00 PLN"], "a's given, b's rule");
        let tenant = |name: &str| KeyVal::Key {
            index: "tenant".to_owned(),
            name: name.to_owned(),
        };
        let reads_of = |key: &KeyVal| {
            let key = CellKey::new("cap", core::slice::from_ref(key), ctx);
            let (cell, _) = engine.cells.get(&key, Some(law)).expect("a kept cell");
            cell.reads.list.clone()
        };
        let givens = |key: KeyVal| Read::Givens {
            def: "cap".to_owned(),
            keys: vec![key],
        };
        let fee = Read::Input {
            id: "fee".to_owned(),
            keys: Vec::new(),
        };
        let given_row = m
            .defs
            .get("cap")
            .and_then(|d| d.givens.first())
            .map(|row| Read::Row {
                version: "v26".to_owned(),
                sid: row.sid.clone(),
            })
            .expect("cap's given row");
        let of_a = reads_of(&tenant("a"));
        assert!(of_a.contains(&givens(tenant("a"))), "a's givens");
        assert!(of_a.contains(&given_row), "and the given row chosen");
        assert!(!of_a.contains(&fee), "the rule isn't read past a given");
        let of_b = reads_of(&tenant("b"));
        assert!(
            of_b.contains(&givens(tenant("b"))),
            "b's givens, though none was chosen"
        );
        assert!(!of_b.contains(&given_row), "a's row isn't b's");
        assert!(of_b.contains(&fee), "then the rule read fee");
    }
}
