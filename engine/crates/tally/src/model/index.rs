//! Indexes and versions: the keys an index holds in their order, and what a version states about its rows.
//! Spec: L §4.1 (indexes: lists, ranges, open, the built-in period indexes), §5.6 (keys and
//! their order), §4.5 (`known`), §4.6 (governs spans), §5.3 (versions, `enters`, `covers`,
//! gap roles, `moves`).
//! Twin: engine/tools/refcore/model.py (`IndexInfo`, `VersionInfo`, `span_at`); loader.py
//! (`build_indexes`, `build_range`, `build_versions`).
//! Invariants: an index's keys are in its order (an open index's by text); a key's position
//! is its place in that order, found by halving a slice of the positions sorted by key, kept
//! beside the keys by every method that sets or adds them (P §3.4: a frozen table is a sorted
//! slice searched by bisection); a version's known date, its window's last day too, is on or
//! before the model's `on`; a version's span under a cut is that of the latest-known `moves`
//! line the cut sees, else its own.

use super::key::{KeyVal, Span};
use crate::time::{Day, PeriodKind};
use core::cmp::Ordering;

/// How an index's keys come (refcore's `IndexInfo.form`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexForm {
    /// `= [a, b]`, and the keys versions add.
    List,
    /// `= period lo .. hi [by n] [from f]`, or an integer range.
    Range,
    /// `: open`: the keys rows bring.
    Open,
    /// `days`, `months`, `years`, `bank_account`.
    Builtin,
}

/// An index (refcore's `IndexInfo`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexInfo {
    /// Its id.
    pub id: String,
    /// How its keys come.
    pub form: IndexForm,
    /// Whether its keys are integers.
    pub is_int: bool,
    /// For a period index, its period kind.
    pub period: Option<PeriodKind>,
    /// Its keys, in its order.
    keys: Vec<KeyVal>,
    /// The keys' positions, sorted by `lookup_order` of their keys, equal keys by position:
    /// what `pos` halves.
    by_key: Vec<usize>,
    /// The keys a version added, with that version.
    pub added: Vec<(KeyVal, String)>,
    /// `assumed "…"`.
    pub assumed: Option<String>,
    /// `v V`.
    pub v: Option<String>,
    /// Each key's source text, with the key.
    pub texts: Vec<(String, KeyVal)>,
    /// The line of its section.
    pub line: Option<u32>,
    /// Whether its range states `from`: its keys are anchored periods, of which `day_of` and
    /// `schedule` have no meaning (refcore's `decl.frm`). L §4.1, §4.2.
    pub anchored: bool,
}

impl IndexInfo {
    /// An index with no key yet.
    #[must_use]
    pub fn new(id: &str, form: IndexForm) -> IndexInfo {
        IndexInfo {
            id: id.to_owned(),
            form,
            is_int: false,
            period: None,
            keys: Vec::new(),
            by_key: Vec::new(),
            added: Vec::new(),
            assumed: None,
            v: None,
            texts: Vec::new(),
            line: None,
            anchored: false,
        }
    }

    /// Its keys, in its order. L §4.1, §5.6.
    #[must_use]
    pub fn keys(&self) -> &[KeyVal] {
        &self.keys
    }

    /// Sets its keys, given in its order, an open index's then put in the text order of their
    /// names (refcore's `IndexInfo.finish`), and sorts their positions for `pos`. L §2.5,
    /// §4.1, §5.6.
    pub fn set_keys(&mut self, keys: Vec<KeyVal>) {
        self.keys = keys;
        if self.form == IndexForm::Open {
            // a stable sort, each key's text made once
            self.keys.sort_by_cached_key(KeyVal::text);
        }
        // a stable sort: equal keys keep their positions' order
        let mut sorted: Vec<(usize, &KeyVal)> = self.keys.iter().enumerate().collect();
        sorted.sort_by(|(_, x), (_, y)| lookup_order(x, y));
        self.by_key = sorted.into_iter().map(|(at, _)| at).collect();
    }

    /// Adds keys after its others, the keys versions bring, and sorts the positions again; an
    /// open index's keys that are all its own names go in at their places in text order with
    /// no sort (`add_name`). L §4.1, §4.5.
    pub fn add_keys(&mut self, more: Vec<KeyVal>) {
        if self.form == IndexForm::Open && self.keys.iter().chain(&more).all(|k| self.names(k)) {
            for key in more {
                self.add_name(key);
            }
            return;
        }
        let mut keys = core::mem::take(&mut self.keys);
        keys.extend(more);
        self.set_keys(keys);
    }

    /// Whether `key` is one of this index's own names, as every key of an open index is
    /// (refcore's `IndexInfo.keys` of an open index). L §4.1.
    fn names(&self, key: &KeyVal) -> bool {
        matches!(key, KeyVal::Key { index, .. } if *index == self.id)
    }

    /// An open index's own name, put in at its place in the text order of its names, found
    /// by halving, after any equal one (refcore's `IndexInfo.finish`, a stable sort). While
    /// every key is one of its own names, `lookup_order` is that same text order, so the
    /// positions sorted for `pos` are the positions in order, and stay so with one more at
    /// the end. L §2.5, §4.1, §5.6.
    fn add_name(&mut self, key: KeyVal) {
        let text = key.text();
        let at = self.keys.partition_point(|k| k.text() <= text);
        let mut old = core::mem::take(&mut self.keys).into_iter();
        self.keys = old.by_ref().take(at).collect();
        self.keys.push(key);
        self.keys.extend(old);
        self.by_key.push(self.by_key.len());
    }

    /// The position of `key`, from 0, the first if two keys are equal (refcore's
    /// `IndexInfo.pos`): sought by halving the positions sorted by key, so a list of up to
    /// 1 000 000 keys (L §7.6 item 2) finds one in about 20 steps, not by a walk through its
    /// keys (P §3.4). L §4.2 `pos`, §5.6.
    #[must_use]
    pub fn pos(&self, key: &KeyVal) -> Option<usize> {
        let from = self
            .by_key
            .partition_point(|at| self.order_at(*at, key) == Some(Ordering::Less));
        // the keys the order calls equal to `key`, by position: the first that is `key`
        for at in self.by_key.get(from..)? {
            if self.order_at(*at, key) != Some(Ordering::Equal) {
                return None;
            }
            if self.keys.get(*at) == Some(key) {
                return Some(*at);
            }
        }
        None
    }

    /// The lookup's order of the key at position `at` against `key`. L §5.6.
    fn order_at(&self, at: usize, key: &KeyVal) -> Option<Ordering> {
        self.keys.get(at).map(|k| lookup_order(k, key))
    }

    /// The key with source text `text`.
    #[must_use]
    pub fn by_text(&self, text: &str) -> Option<&KeyVal> {
        self.texts.iter().find(|(t, _)| t == text).map(|(_, k)| k)
    }

    /// The version that added `key`, if a version did.
    #[must_use]
    pub fn added_by(&self, key: &KeyVal) -> Option<&str> {
        self.added
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// The key whose day or period holds `day` (refcore's `IndexInfo.holding`). L §4.6.
    #[must_use]
    pub fn holding(&self, day: Day) -> Option<&KeyVal> {
        self.keys.iter().find(|k| match k {
            KeyVal::Day(d) => *d == day,
            KeyVal::Period(p) => p.contains(day),
            KeyVal::Key { .. }
            | KeyVal::Thing(_)
            | KeyVal::Int(_)
            | KeyVal::Mv(_)
            | KeyVal::Value(_) => false,
        })
    }
}

/// The order `pos` sorts and halves keys by: the kind of key, then its text, day, period or
/// number; keys it calls equal are told apart by `==`. Only the lookup reads it: no output
/// and no rule is ordered by it (R-47). L §4.1, §5.6.
fn lookup_order(one: &KeyVal, other: &KeyVal) -> Ordering {
    match (one, other) {
        (
            KeyVal::Key { index, name },
            KeyVal::Key {
                index: other_index,
                name: other_name,
            },
        ) => (index, name).cmp(&(other_index, other_name)),
        (KeyVal::Thing(name), KeyVal::Thing(other_name)) => name.cmp(other_name),
        (KeyVal::Day(day), KeyVal::Day(other_day)) => day.cmp(other_day),
        (KeyVal::Period(period), KeyVal::Period(other_period)) => {
            let ends = (period.start(), period.end());
            ends.cmp(&(other_period.start(), other_period.end()))
        }
        (KeyVal::Int(n), KeyVal::Int(other_n)) => n.cmp(other_n),
        (KeyVal::Mv(mv), KeyVal::Mv(other_mv)) => mv.id.cmp(&other_mv.id),
        (KeyVal::Value(v), KeyVal::Value(other_v)) => v.text.cmp(&other_v.text),
        _ => kind_rank(one).cmp(&kind_rank(other)),
    }
}

/// A key's kind, as `lookup_order` ranks two keys of different kinds.
fn kind_rank(k: &KeyVal) -> u8 {
    match k {
        KeyVal::Key { .. } => 0,
        KeyVal::Thing(_) => 1,
        KeyVal::Day(_) => 2,
        KeyVal::Period(_) => 3,
        KeyVal::Int(_) => 4,
        KeyVal::Mv(_) => 5,
        KeyVal::Value(_) => 6,
    }
}

/// A version's gap role for a place (L §5.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GapRole {
    /// `pad x from y`: the gap is padded from `y`.
    Pad(String),
    /// `anchor x`.
    Anchor,
}

/// A version (refcore's `VersionInfo`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionInfo {
    /// Its id.
    pub id: String,
    /// The source it cites.
    pub cite: String,
    /// `governs`: none, or its span.
    pub governs: Option<Span>,
    /// `known`: the day, or a window's first day.
    pub known: Day,
    /// `known window(a, b)`: the window's last day, b (L §4.5); none for a day.
    pub window: Option<Day>,
    /// The `moves` lines of later versions that move this one, in version order (L §5.3).
    pub moved_by: Vec<Moved>,
    /// `enters D`.
    pub enters: Option<Day>,
    /// `assumed "…"`.
    pub assumed: Option<String>,
    /// `covers x [span]`: the place and its span (none: the version's governs).
    pub covers: Vec<(String, Option<Span>)>,
    /// Its gap roles, by place.
    pub gap_roles: Vec<(String, GapRole)>,
    /// The file it is in.
    pub file: String,
    /// Its line.
    pub line: u32,
}

/// A `moves V to SPAN` line, as the version it moves holds it (refcore's `moved_by`). L §5.3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Moved {
    /// The version whose line it is.
    pub by: String,
    /// That version's `known`, or its window's first day.
    pub known: Day,
    /// That version's known window's last day, if it has one.
    pub window: Option<Day>,
    /// The span it moves V to.
    pub span: Span,
}

impl Moved {
    /// Whether cut `cut` sees the line: its version is known by `cut`, its whole window
    /// included (L §4.5; a cut inside the window is the run's `windows each`, not built).
    #[must_use]
    pub fn seen_at(&self, cut: Day) -> bool {
        self.window.unwrap_or(self.known) <= cut
    }
}

impl VersionInfo {
    /// The governs span in force under cut `cut`: the span of the latest-known `moves` line
    /// the cut sees, else the version's own (refcore's `VersionInfo.span_at`). L §4.6 step 1,
    /// §5.3.
    #[must_use]
    pub fn span_at(&self, cut: Day) -> Option<&Span> {
        let mut best: Option<&Moved> = None;
        for line in self.moved_by.iter().filter(|line| line.seen_at(cut)) {
            if best.is_none_or(|b| line.known > b.known) {
                best = Some(line);
            }
        }
        best.map_or(self.governs.as_ref(), |line| Some(&line.span))
    }

    /// Its entry into force for a row with time `own` (refcore's `VersionInfo.enters_at`):
    /// `enters` if stated, else the later of `known` and the row's own start, or of the
    /// governs span's start. L §5.3.
    #[must_use]
    pub fn enters_at(&self, cut: Day, own: OwnTime) -> Day {
        if let Some(day) = self.enters {
            return day;
        }
        let start = match own {
            OwnTime::Stated(own_start) => own_start,
            OwnTime::Version => self.span_at(cut).and_then(Span::start),
        };
        start.map_or(self.known, |s| self.known.max(s))
    }
}

/// A row's own time, for its entry into force.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnTime {
    /// The row has none: its version's governs span is its time.
    Version,
    /// Its own span, lifespan or instant, starting on this day (none: no start).
    Stated(Option<Day>),
}

#[cfg(test)]
mod tests {
    use super::{IndexForm, IndexInfo};
    use crate::model::KeyVal;
    use crate::time::Day;

    fn key(index: &str, name: &str) -> KeyVal {
        KeyVal::Key {
            index: index.to_owned(),
            name: name.to_owned(),
        }
    }

    #[test]
    fn a_list_finds_each_key_at_its_written_place() {
        let mut list = IndexInfo::new("big", IndexForm::List);
        let names: Vec<String> = (1..=1000).map(|n| format!("k{n}")).collect();
        list.set_keys(names.iter().map(|n| key("big", n)).collect());
        for (at, name) in names.iter().enumerate() {
            assert_eq!(list.pos(&key("big", name)), Some(at), "{name}");
        }
        assert_eq!(list.pos(&key("big", "k0")), None, "not a key");
        assert_eq!(list.pos(&key("other", "k1")), None, "another index's key");
        assert_eq!(list.pos(&KeyVal::Int(1)), None, "another kind of key");
    }

    #[test]
    fn keys_added_later_and_open_keys_keep_their_places() {
        let mut list = IndexInfo::new("qs", IndexForm::List);
        list.set_keys(vec![key("qs", "q2"), key("qs", "q1")]);
        list.add_keys(vec![key("qs", "q0")]);
        let found: Vec<Option<usize>> = ["q2", "q1", "q0"]
            .iter()
            .map(|n| list.pos(&key("qs", n)))
            .collect();
        assert_eq!(found, vec![Some(0), Some(1), Some(2)], "written order");
        let mut open = IndexInfo::new("acc", IndexForm::Open);
        open.add_keys(vec![key("acc", "b")]);
        open.add_keys(vec![key("acc", "a")]);
        assert_eq!(
            open.keys(),
            [key("acc", "a"), key("acc", "b")],
            "text order"
        );
        assert_eq!(open.pos(&key("acc", "b")), Some(1), "b after a");
    }

    #[test]
    fn open_names_added_one_at_a_time_match_one_sort() {
        // 500 names added from the last: their text order (n0, n1, n10, n100, …) is neither
        let names: Vec<String> = (0..500_u32).rev().map(|n| format!("n{n}")).collect();
        let mut one_by_one = IndexInfo::new("acc", IndexForm::Open);
        for name in &names {
            one_by_one.add_keys(vec![key("acc", name)]);
        }
        let mut sorted = IndexInfo::new("acc", IndexForm::Open);
        sorted.set_keys(names.iter().map(|n| key("acc", n)).collect());
        assert_eq!(one_by_one.keys(), sorted.keys(), "text order");
        assert_eq!(one_by_one.by_key, sorted.by_key, "the positions pos halves");
        for (at, k) in sorted.keys().iter().enumerate() {
            assert_eq!(one_by_one.pos(k), Some(at), "{k:?}");
        }
        // another index's key among them: all sorted again, `m` first in text order
        one_by_one.add_keys(vec![key("other", "m")]);
        assert_eq!(
            one_by_one.pos(&key("other", "m")),
            Some(0),
            "the foreign key"
        );
        assert_eq!(one_by_one.pos(&key("acc", "n0")), Some(1), "still found");
    }

    #[test]
    fn days_and_integers_are_found_by_halving() {
        let mut days = IndexInfo::new("days", IndexForm::Builtin);
        let first = Day::from_ymd(2026, 1, 1).expect("day");
        let mut list = Vec::new();
        let mut d = Some(first);
        for _ in 0..400 {
            let Some(day) = d else { break };
            list.push(KeyVal::Day(day));
            d = day.next();
        }
        days.set_keys(list.clone());
        for (at, k) in list.iter().enumerate() {
            assert_eq!(days.pos(k), Some(at), "day {at}");
        }
        let mut ints = IndexInfo::new("n", IndexForm::Range);
        ints.set_keys((-5..=5).map(KeyVal::Int).collect());
        assert_eq!(ints.pos(&KeyVal::Int(-5)), Some(0), "first");
        assert_eq!(ints.pos(&KeyVal::Int(5)), Some(10), "last");
        assert_eq!(ints.pos(&KeyVal::Int(6)), None, "past the end");
    }
}
