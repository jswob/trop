//! A register: the entries, versions, scenario facts and rows a read met, each entry kept once per run, every register shared, never copied.
//! Spec: L §8.2 (the register: its entries, their words and their order; One read set: what
//! decides whether a movement counts where it isn't counted), §6.4 (the scenario facts a read
//! met), E §1 (the run stores nothing: the table of entries lives for one run, handed to each
//! ask's engine), §3.2 (each cell's result keeps what its evaluation met), §5 step 4 (the
//! versions an adjustment names), §9 item 3 (the read set a movement is listed by), §10 (the
//! rows `inputs-of` lists); P §3.5 (the memo's memory: a register shared, not copied; the perf
//! plan's G6b, issue #252).
//! Twin: engine/tools/refcore/evaluate.py (`Coll`: its `reg`, `rows`, `missing`).
//! Invariants: an entry's text is kept once, in the run's table, under an id given in the
//! order entries are first noted, which no line is ever printed in: the entries print in their
//! text order (the word, the text, the figure, by code point; R-47). A register's ids are a
//! sorted list shared by every register equal to it, a clone a count: a merge keeps its own
//! list where it holds the other's, takes the other's where that holds its own, and makes one
//! new list otherwise. Its deciding entries (pins, `pin met`, assumptions) are a second list
//! kept beside it, so reading them never scans the whole. A merge adds every entry, version,
//! fact and row of the other, never its `missing`.

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use super::state::invariant;
use crate::syntax::Stop;

/// A register entry a read meets (L §8.2): its word (`assumed`, `held`, `given`, `missing`),
/// its text and the figure it was met at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The register word.
    pub word: &'static str,
    /// Its text.
    pub text: String,
    /// The figure, or the empty text.
    pub figure: String,
}

impl Ord for Entry {
    /// The register's order: the word, the text, the figure, by code point.
    fn cmp(&self, other: &Entry) -> core::cmp::Ordering {
        self.word
            .cmp(other.word)
            .then_with(|| self.text.cmp(&other.text))
            .then_with(|| self.figure.cmp(&other.figure))
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Entry) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// The words of the entries that decide whether a movement counts where it isn't counted:
/// its pins, `solve`'s, and its assumptions (L §8.2 One read set; P35 on #241).
const DECIDING: [&str; 3] = ["pinned", "pin met", "assumed"];

/// The bit of an id whose entry's word is a deciding one; the other 31 are its place in the
/// run's table.
const DECIDES: u32 = 1 << 31;

/// An entry's id in the run's table (L §8.2): its place, given in the order entries are first
/// noted, and whether its word decides. No order of lines is ever an order of ids (R-47), so
/// it has none; a register sorts by the number inside this file alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntryId(u32);

impl EntryId {
    /// Whether its entry's word is a deciding one (L §8.2 One read set).
    fn decides(self) -> bool {
        self.0 & DECIDES != 0
    }

    /// Its place in the run's table.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` where the place doesn't fit a position, which no platform the engine
    /// builds for has.
    fn slot(self) -> Result<usize, Stop> {
        usize::try_from(self.0 & !DECIDES).map_err(|_| invariant("an entry's id is a position"))
    }
}

/// The run's register entries, each once, by id and by entry (E §1: the run's, handed to
/// each ask's engine and back; never global).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Entries {
    /// The entries, by their ids' places.
    all: Vec<Rc<Entry>>,
    /// Each entry's id.
    ids: BTreeMap<Rc<Entry>, EntryId>,
}

impl Entries {
    /// The id of `entry`, a new one when the run hasn't met it.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` past 2^31 entries in one run.
    pub fn intern(&mut self, entry: Entry) -> Result<EntryId, Stop> {
        if let Some(id) = self.ids.get(&entry) {
            return Ok(*id);
        }
        let place = u32::try_from(self.all.len())
            .ok()
            .filter(|n| n & DECIDES == 0)
            .ok_or_else(|| invariant("a run notes at most 2^31 register entries"))?;
        let id = if DECIDING.contains(&entry.word) {
            EntryId(place | DECIDES)
        } else {
            EntryId(place)
        };
        let entry = Rc::new(entry);
        self.all.push(Rc::clone(&entry));
        self.ids.insert(entry, id);
        Ok(id)
    }

    /// The id of `entry`, where the run has met it.
    #[must_use]
    pub fn find(&self, entry: &Entry) -> Option<EntryId> {
        self.ids.get(entry).copied()
    }

    /// The entry of `id`.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for an id this table didn't give.
    pub fn entry(&self, id: EntryId) -> Result<&Entry, Stop> {
        self.all
            .get(id.slot()?)
            .map(Rc::as_ref)
            .ok_or_else(|| invariant("a register entry is in its run's table"))
    }

    /// The word of `id`'s entry.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for an id this table didn't give.
    pub fn word(&self, id: EntryId) -> Result<&'static str, Stop> {
        Ok(self.entry(id)?.word)
    }

    /// Whether one of `ids` is an entry of word `word`.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for an id this table didn't give.
    pub fn any_word(&self, ids: &[EntryId], word: &str) -> Result<bool, Stop> {
        for id in ids {
            if self.word(*id)? == word {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// The register entries `met` holds, in their text order: the word, the text, the
    /// figure, by code point (L §8.2; R-47).
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for an entry this table lacks.
    pub fn sorted(&self, met: &Collected) -> Result<Vec<&Entry>, Stop> {
        let mut out = Vec::with_capacity(met.ids().len());
        for id in met.ids() {
            out.push(self.entry(*id)?);
        }
        out.sort();
        Ok(out)
    }
}

/// Entry ids sorted by number, none or a shared non-empty list: a clone is a count, never a
/// copy.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Ids(Option<Rc<Vec<EntryId>>>);

impl Ids {
    /// The ids, by number.
    fn as_slice(&self) -> &[EntryId] {
        self.0.as_deref().map_or(&[], Vec::as_slice)
    }

    /// Whether it holds `id`.
    fn holds(&self, id: EntryId) -> bool {
        self.as_slice().binary_search_by_key(&id.0, |e| e.0).is_ok()
    }

    /// Adds `id`: at the end where it is the newest (an entry met for the first time is),
    /// else in a new list.
    fn insert(&mut self, id: EntryId) {
        let Some(list) = &mut self.0 else {
            self.0 = Some(Rc::new(vec![id]));
            return;
        };
        let Err(at) = list.binary_search_by_key(&id.0, |e| e.0) else {
            return;
        };
        if at == list.len() {
            Rc::make_mut(list).push(id);
            return;
        }
        *list = Rc::new(union(list, &[id]));
    }

    /// Adds every id of `other`: unchanged where this holds them all, `other`'s list where
    /// it holds every one of this, one new list otherwise (a few ids into many are each
    /// found by bisection).
    fn merge(&mut self, other: &Ids) {
        let Some(theirs) = &other.0 else {
            return;
        };
        let Some(ours) = &self.0 else {
            self.0 = Some(Rc::clone(theirs));
            return;
        };
        if Rc::ptr_eq(ours, theirs) {
            return;
        }
        let (a, b) = (ours.as_slice(), theirs.as_slice());
        if few_of(b, a) {
            let new: Vec<EntryId> = b.iter().copied().filter(|id| !self.holds(*id)).collect();
            if !new.is_empty() {
                self.0 = Some(Rc::new(union(a, &new)));
            }
            return;
        }
        if few_of(a, b) && a.iter().all(|id| other.holds(*id)) {
            self.0 = Some(Rc::clone(theirs));
            return;
        }
        match lacks(a, b) {
            (_, false) => {}
            (false, true) => self.0 = Some(Rc::clone(theirs)),
            (true, true) => self.0 = Some(Rc::new(union(a, b))),
        }
    }

    /// The ids but those of `gone`, none where none is left.
    fn without(&self, gone: &[EntryId]) -> Ids {
        let mut kept = Vec::new();
        for id in self.as_slice() {
            if !gone.contains(id) {
                kept.push(*id);
            }
        }
        Ids((!kept.is_empty()).then(|| Rc::new(kept)))
    }
}

/// Whether `few` holds at most a sixteenth as many ids as `many`: then each is found in
/// `many` by bisection rather than both read through.
fn few_of(few: &[EntryId], many: &[EntryId]) -> bool {
    few.len().checked_mul(16).is_some_and(|n| n <= many.len())
}

/// Whether `a` holds an id `b` lacks, and whether `b` holds one `a` lacks: both read through
/// once, together, until both are known.
fn lacks(a: &[EntryId], b: &[EntryId]) -> (bool, bool) {
    let (mut ours, mut theirs) = (a.iter().peekable(), b.iter().peekable());
    let (mut a_more, mut b_more) = (false, false);
    while !(a_more && b_more) {
        match (ours.peek(), theirs.peek()) {
            (Some(left), Some(right)) => match left.0.cmp(&right.0) {
                core::cmp::Ordering::Less => a_more = ours.next().is_some(),
                core::cmp::Ordering::Greater => b_more = theirs.next().is_some(),
                core::cmp::Ordering::Equal => {
                    ours.next();
                    theirs.next();
                }
            },
            (Some(_), None) => return (true, b_more),
            (None, Some(_)) => return (a_more, true),
            (None, None) => break,
        }
    }
    (a_more, b_more)
}

/// The ids of `a` and of `b`, each once, by number.
fn union(a: &[EntryId], b: &[EntryId]) -> Vec<EntryId> {
    let mut out = Vec::with_capacity(a.len().max(b.len()));
    let (mut ours, mut theirs) = (a.iter().peekable(), b.iter().peekable());
    loop {
        let next = match (ours.peek(), theirs.peek()) {
            (Some(left), Some(right)) => match left.0.cmp(&right.0) {
                core::cmp::Ordering::Less => ours.next(),
                core::cmp::Ordering::Greater => theirs.next(),
                core::cmp::Ordering::Equal => {
                    theirs.next();
                    ours.next()
                }
            },
            (Some(_), None) => ours.next(),
            (None, Some(_)) => theirs.next(),
            (None, None) => break,
        };
        out.extend(next.copied());
    }
    out
}

/// A shared set of versions, none or one non-empty set: a clone is a count, never a copy.
type Versions = Option<Rc<BTreeSet<String>>>;

/// A shared set of scenario facts, each (its section, its item), as [`Versions`].
type Facts = Option<Rc<BTreeSet<(String, String)>>>;

/// What a cell's evaluation met: its register entries (refcore's `Coll.reg`), and whether a
/// `missing(…)` read directly in it held (refcore's `Coll.missing`, never merged outward).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Collected {
    /// The register entries, by id.
    entries: Ids,
    /// Of those, the deciding ones (L §8.2 One read set): pins, `pin met`, assumptions.
    deciding: Ids,
    /// Whether a `missing(r)` evaluated in this collector, not in a cell below it, was true.
    missing: bool,
    /// The versions of the rows read (refcore's `Coll.rows`, their versions): which versions
    /// a later step's adjustment names (E §5 step 4).
    versions: Versions,
    /// The scenarios' facts read, each (its section, its item): a row of a scenario's
    /// version, a thing's entry or place a scenario states, a scenario's move a binding
    /// ranges over: the read set E §9 item 3 lists a movement by (L §6.4, ruling P45).
    scenario: Facts,
    /// Of those, each (its section, its item) a movement booked before the scenario's `from`
    /// lists: a row of a scenario's version, and a thing's entry a `lifespan` read met
    /// (refcore's `Coll.rows` and `lifeof` entries; L §8.2 `before from:`, tally 2.1.7 R7).
    listed: Facts,
    /// The rows read, by stable id, each with its version and locator: an input's, a given,
    /// a band row, a reading and an observed row, the walk's counted movements' included
    /// (refcore's `Coll.rows`). Kept only by an engine answering `inputs-of` (E §3.2, §10).
    rows: Option<Rc<BTreeMap<String, RowSeen>>>,
}

/// A row read, as `inputs-of` lists it: its version and its locator (E §10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowSeen {
    /// The version that states it.
    pub version: String,
    /// Its locator, where it states one.
    pub locator: Option<String>,
}

impl Collected {
    /// Records entry `id` (L §8.2).
    pub fn note(&mut self, id: EntryId) {
        self.entries.insert(id);
        if id.decides() {
            self.deciding.insert(id);
        }
    }

    /// Records each of `ids` (L §8.2).
    pub fn note_all(&mut self, ids: &[EntryId]) {
        for id in ids {
            self.note(*id);
        }
    }

    /// Adds the register entries, versions and rows `other` met (its `missing` stays its own).
    pub fn merge(&mut self, other: &Collected) {
        self.entries.merge(&other.entries);
        self.deciding.merge(&other.deciding);
        merge_versions(&mut self.versions, &other.versions);
        merge_facts(&mut self.scenario, &other.scenario);
        merge_facts(&mut self.listed, &other.listed);
        self.merge_rows(other);
    }

    /// Adds the rows `other` read, nothing else: what a refusal of the order tested gives a
    /// read of a refused run, its read set, not the register lines of the reads that
    /// tested it (L §8.2 One read set; E §4.2 step 3; P20).
    pub(crate) fn merge_rows(&mut self, other: &Collected) {
        let Some(theirs) = &other.rows else {
            return;
        };
        let Some(ours) = &mut self.rows else {
            self.rows = Some(Rc::clone(theirs));
            return;
        };
        let held = |(sid, seen): (&String, &RowSeen)| ours.get(sid) == Some(seen);
        if Rc::ptr_eq(ours, theirs) || theirs.iter().all(held) {
            return;
        }
        let into = Rc::make_mut(ours);
        for (sid, seen) in theirs.iter() {
            into.insert(sid.clone(), seen.clone());
        }
    }

    /// What of these reads decides whether a movement counts where it isn't counted (a
    /// refused one, a day outside a read's span, left out, covered, a false `when`): its pins,
    /// `solve`'s and its assumptions; never a `held:` value of its day (L §8.2 One read set,
    /// one statement for every form and outcome: P35 on #241).
    #[must_use]
    pub(crate) fn deciding(&self) -> Collected {
        Collected {
            entries: self.deciding.clone(),
            deciding: self.deciding.clone(),
            ..Collected::default()
        }
    }

    /// Whether these reads met a deciding entry ([`Collected::deciding`]).
    #[must_use]
    pub(crate) fn decides(&self) -> bool {
        self.deciding.0.is_some()
    }

    /// What a movement's day that puts it outside a read's span gives that read: what decides
    /// it ([`Collected::deciding`]) and the rows the day read, which decided that it doesn't
    /// count, never its `when` or amount (L §8.2 One read set; P35 on #241; P20).
    #[must_use]
    pub(crate) fn outside(&self) -> Collected {
        let mut out = self.deciding();
        out.rows.clone_from(&self.rows);
        out
    }

    /// These reads without the entries `gone`: an ask's pin no read of its line took isn't
    /// listed under it (L §8.2 One read set; tally 2.1.8 §4, #161).
    #[must_use]
    pub(crate) fn without(&self, gone: &[EntryId]) -> Collected {
        Collected {
            entries: self.entries.without(gone),
            deciding: self.deciding.without(gone),
            ..self.clone()
        }
    }

    /// Whether these reads met any register entry (L §8.2).
    #[must_use]
    pub fn has_entries(&self) -> bool {
        self.entries.0.is_some()
    }

    /// Whether these reads met entry `id` (L §8.2).
    #[must_use]
    pub fn holds(&self, id: EntryId) -> bool {
        self.entries.holds(id)
    }

    /// The register's entries' ids, in no order a line may print (R-47): what a read of
    /// their words or texts in `entries` takes.
    #[must_use]
    pub fn ids(&self) -> &[EntryId] {
        self.entries.as_slice()
    }

    /// Records that a `missing(…)` read in this collector held (L §8.2 `missing`).
    pub(crate) fn note_missing(&mut self) {
        self.missing = true;
    }

    /// Whether a `missing(…)` read in this collector, not in a cell below it, held (L §8.2
    /// `missing`).
    #[must_use]
    pub(crate) fn missing_held(&self) -> bool {
        self.missing
    }

    /// Records that a row of version `version` was read (E §5 step 4).
    pub(crate) fn note_version(&mut self, version: &str) {
        if self.versions.as_ref().is_some_and(|v| v.contains(version)) {
            return;
        }
        let set = self.versions.get_or_insert_with(Rc::default);
        Rc::make_mut(set).insert(version.to_owned());
    }

    /// The versions of the rows read, by code point (E §5 step 4); none where it read none.
    #[must_use]
    pub fn versions(&self) -> Option<&BTreeSet<String>> {
        self.versions.as_deref()
    }

    /// Records that scenario fact `item` of section `section` was read (L §6.4; E §9 item 3).
    pub(crate) fn note_scenario(&mut self, (section, item): (&str, &str)) {
        note_fact(&mut self.scenario, (section, item));
    }

    /// Records that scenario fact `item` of section `section` was read, one a movement booked
    /// before the scenario's `from` lists (L §6.4, §8.2 `before from:`; E §9 item 3).
    pub(crate) fn note_listed(&mut self, (section, item): (&str, &str)) {
        note_fact(&mut self.scenario, (section, item));
        note_fact(&mut self.listed, (section, item));
    }

    /// The scenarios' facts read, each (its section, its item), in that order (E §9 item 3);
    /// none where it read none.
    #[must_use]
    pub fn scenario(&self) -> Option<&BTreeSet<(String, String)>> {
        self.scenario.as_deref()
    }

    /// The scenarios' facts read that a movement booked before the scenario's `from` lists
    /// (L §8.2 `before from:`); none where it read none.
    #[must_use]
    pub fn listed(&self) -> Option<&BTreeSet<(String, String)>> {
        self.listed.as_deref()
    }

    /// Records row `sid` of `version` with its `locator` as read (E §10 `inputs-of`).
    pub(crate) fn note_row(&mut self, sid: &str, version: &str, locator: Option<&String>) {
        let rows = self.rows.get_or_insert_with(Rc::default);
        Rc::make_mut(rows).insert(
            sid.to_owned(),
            RowSeen {
                version: version.to_owned(),
                locator: locator.cloned(),
            },
        );
    }

    /// The rows read, by stable id (E §10 `inputs-of`); none where it read none.
    #[must_use]
    pub fn rows(&self) -> Option<&BTreeMap<String, RowSeen>> {
        self.rows.as_deref()
    }
}

/// Adds `theirs` to `ours`: unchanged where it holds them, `theirs` where that holds it, a
/// copy of `ours` with them added otherwise.
fn merge_versions(ours: &mut Versions, theirs: &Versions) {
    let Some(b) = theirs else {
        return;
    };
    let Some(a) = ours else {
        *ours = Some(Rc::clone(b));
        return;
    };
    if Rc::ptr_eq(a, b) || b.is_subset(a) {
        return;
    }
    if a.is_subset(b) {
        *a = Rc::clone(b);
        return;
    }
    Rc::make_mut(a).extend(b.iter().cloned());
}

/// [`merge_versions`] of two sets of scenario facts.
fn merge_facts(ours: &mut Facts, theirs: &Facts) {
    let Some(b) = theirs else {
        return;
    };
    let Some(a) = ours else {
        *ours = Some(Rc::clone(b));
        return;
    };
    if Rc::ptr_eq(a, b) || b.is_subset(a) {
        return;
    }
    if a.is_subset(b) {
        *a = Rc::clone(b);
        return;
    }
    Rc::make_mut(a).extend(b.iter().cloned());
}

/// Adds fact `item` of section `section` to `facts`, copied first where it is shared.
fn note_fact(facts: &mut Facts, (section, item): (&str, &str)) {
    let fact = (section.to_owned(), item.to_owned());
    if facts.as_ref().is_some_and(|set| set.contains(&fact)) {
        return;
    }
    let set = facts.get_or_insert_with(Rc::default);
    Rc::make_mut(set).insert(fact);
}

#[cfg(test)]
mod tests {
    use super::{Collected, Entries, Entry};

    fn entry(word: &'static str, text: &str) -> Entry {
        Entry {
            word,
            text: text.to_owned(),
            figure: String::new(),
        }
    }

    #[test]
    fn a_register_prints_in_text_order_not_first_met() {
        let mut table = Entries::default();
        let mut met = Collected::default();
        for (word, text) in [("held", "b"), ("assumed", "z"), ("held", "a")] {
            met.note(table.intern(entry(word, text)).expect("an id"));
        }
        let sorted = table.sorted(&met).expect("entries");
        let texts: Vec<&str> = sorted.iter().map(|e| e.text.as_str()).collect();
        assert_eq!(texts, ["z", "a", "b"], "assumed before held, then by text");
    }

    #[test]
    fn a_merge_shares_where_one_holds_the_other() {
        let mut table = Entries::default();
        let ids: Vec<_> = ["a", "b", "c"]
            .iter()
            .map(|t| table.intern(entry("held", t)).expect("an id"))
            .collect();
        let mut small = Collected::default();
        small.note_all(ids.get(..2).expect("two"));
        let mut big = small.clone();
        big.note_all(&ids);
        let mut into = small.clone();
        into.merge(&big);
        assert_eq!(into, big, "the smaller takes the bigger's");
        let mut back = big.clone();
        back.merge(&small);
        assert_eq!(back, big, "the bigger holds the smaller");
        let mut other = Collected::default();
        other.note(table.intern(entry("held", "d")).expect("an id"));
        back.merge(&other);
        assert_eq!(back.ids().len(), 4, "a union of both");
        assert!(back.holds(*ids.first().expect("a")), "a held");
    }

    #[test]
    fn deciding_entries_are_kept_apart() {
        let mut table = Entries::default();
        let mut met = Collected::default();
        met.note(table.intern(entry("held", "x")).expect("an id"));
        assert!(!met.decides(), "held decides nothing");
        let pin = table.intern(entry("pinned", "p = 1")).expect("an id");
        met.note(pin);
        let deciding = met.deciding();
        assert_eq!(deciding.ids(), [pin], "the pin alone");
        assert!(!met.without(&[pin]).decides(), "taken away with it");
        assert_eq!(
            table.intern(entry("pinned", "p = 1")),
            Ok(pin),
            "met again, the same id"
        );
    }
}
