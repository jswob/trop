//! The memo: each def cell's result under a typed key, shared by every law date of the cell's class.
//! Spec: E §3.2 (cells and the memo; the law date is a class: the interval between consecutive
//! boundaries of the governs spans of the rows and rules the item reaches and of the
//! lifespans of the things at the keys it reads, every law date inside it giving the same
//! value); P §3.1 (the memo key: key values, the context trimmed to what the item can depend
//! on), §3.2 (the law date class layer), §3.3 (the test a trimming must pass: two contexts
//! share a cell only where the cell's result, register included, is the same in both).
//! Twin: engine/tools/refcore/evaluate.py (`def_cell`, whose memo keys a cell by its exact law
//! date and cut).
//! Invariants: a cell kept for law date d holds for every law date of its class, and every
//! read that tests the law date narrows the class of the cell computing it: a row, a rule or
//! a band table chosen by its span to the side of each span boundary d is on, a row that may
//! be `held` to d's side of the ask's cut, and a text that names d (a hole's question, a
//! `held` entry, `RUN-CONFLICT`, `RUN-NO-RULE`), a reading, a calendar's days and a priced
//! figure to d alone; a cell read at the reader's own law date narrows the reader to the read
//! cell's class; a cycle met narrows every open cell to its own day; a walk-bound cell is kept
//! at its exact law date; under `Memo::Exact` every class is one day.
//! PERF: P8-14 (def misses: the loan schedule's cells were each computed once per law date of
//! the instance that read them, 480 law dates for one class); plain twin: refcore, whose memo
//! keys each cell by its exact law date as `Memo::Exact` does here; the equivalence test runs
//! the kernel corpus under both (crates/tally-cli/tests/walk_oracle.rs).

use core::cmp::Ordering;
use std::collections::BTreeMap;

use super::state::{Cell, Engine, invariant};
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

/// A cell's key in the memo, its law date apart: the def, its keys, the cut, and for a
/// walk-bound def the walk point (E §3.2).
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
}

impl CellKey {
    /// The key of def `def` at `keys` under cut `cut`, at walk point `at`.
    pub(super) fn new(def: &str, keys: &[KeyVal], cut: Day, at: Option<WalkAt>) -> CellKey {
        CellKey {
            def: def.to_owned(),
            keys: keys.to_vec(),
            cut,
            at,
        }
    }
}

impl Ord for CellKey {
    /// The memo's order: the def, the keys, the cut, the walk point.
    fn cmp(&self, other: &CellKey) -> Ordering {
        self.def
            .cmp(&other.def)
            .then_with(|| keys_order(&self.keys, &other.keys))
            .then_with(|| self.cut.cmp(&other.cut))
            .then_with(|| walk_at_order(self.at, other.at))
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
    use super::{CellKey, Cells, LawClass, Memo, keys_order};
    use crate::eval::state::{Cell, Collected};
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
        let key = CellKey::new("debt", &[KeyVal::Int(3)], day(2026, 9, 30), None);
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
}
