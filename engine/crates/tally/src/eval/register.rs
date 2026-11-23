//! A register: the entries a read met, the versions, scenario facts and rows it read; one API, so how a register is kept stays in this file.
//! Spec: L §8.2 (the register: its entries, their words and their order; One read set: what
//! decides whether a movement counts where it isn't counted), §6.4 (the scenario facts a read
//! met), E §3.2 (each cell's result keeps what its evaluation met), §5 step 4 (the versions an
//! adjustment names), §9 item 3 (the read set a movement is listed by), §10 (the rows
//! `inputs-of` lists).
//! Twin: engine/tools/refcore/evaluate.py (`Coll`: its `reg`, `rows`, `missing`).
//! Invariants: the entries are a set, read only in their text order (the word, the text, the
//! figure, by code point; R-47); a merge adds every entry, version, fact and row of the other,
//! never its `missing`.

use std::collections::{BTreeMap, BTreeSet, btree_set};

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
    /// The set's order: the word, the text, the figure, by code point.
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

/// What a cell's evaluation met: its register entries (refcore's `Coll.reg`), and whether a
/// `missing(…)` read directly in it held (refcore's `Coll.missing`, never merged outward).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Collected {
    /// The register entries.
    entries: BTreeSet<Entry>,
    /// Whether a `missing(r)` evaluated in this collector, not in a cell below it, was true.
    missing: bool,
    /// The versions of the rows read (refcore's `Coll.rows`, their versions): which versions
    /// a later step's adjustment names (E §5 step 4).
    versions: BTreeSet<String>,
    /// The scenarios' facts read, each (its section, its item): a row of a scenario's
    /// version, a thing's entry or place a scenario states, a scenario's move a binding
    /// ranges over: the read set E §9 item 3 lists a movement by (L §6.4, ruling P45).
    scenario: BTreeSet<(String, String)>,
    /// Of those, each (its section, its item) a movement booked before the scenario's `from`
    /// lists: a row of a scenario's version, and a thing's entry a `lifespan` read met
    /// (refcore's `Coll.rows` and `lifeof` entries; L §8.2 `before from:`, tally 2.1.7 R7).
    listed: BTreeSet<(String, String)>,
    /// The rows read, by stable id, each with its version and locator: an input's, a given,
    /// a band row, a reading and an observed row, the walk's counted movements' included
    /// (refcore's `Coll.rows`). Kept only by an engine answering `inputs-of` (E §3.2, §10).
    rows: BTreeMap<String, RowSeen>,
}

/// A row read, as `inputs-of` lists it: its version and its locator (E §10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowSeen {
    /// The version that states it.
    pub version: String,
    /// Its locator, where it states one.
    pub locator: Option<String>,
}

/// A register's entries in their text order: the word, the text, the figure (L §8.2).
#[derive(Debug, Clone)]
pub struct EntriesSorted<'a> {
    /// The entries left.
    inner: btree_set::Iter<'a, Entry>,
}

impl<'a> Iterator for EntriesSorted<'a> {
    type Item = &'a Entry;

    fn next(&mut self) -> Option<&'a Entry> {
        self.inner.next()
    }
}

impl Collected {
    /// Records entry `entry` (L §8.2).
    pub fn note(&mut self, entry: Entry) {
        self.entries.insert(entry);
    }

    /// Records each of `entries` (L §8.2).
    pub fn note_all(&mut self, entries: &[Entry]) {
        self.entries.extend(entries.iter().cloned());
    }

    /// Adds the register entries, versions and rows `other` met (its `missing` stays its own).
    pub fn merge(&mut self, other: &Collected) {
        self.entries.extend(other.entries.iter().cloned());
        self.versions.extend(other.versions.iter().cloned());
        self.scenario.extend(other.scenario.iter().cloned());
        self.listed.extend(other.listed.iter().cloned());
        self.merge_rows(other);
    }

    /// Adds the rows `other` read, nothing else: what a refusal of the order tested gives a
    /// read of a refused run, its read set, not the register lines of the reads that
    /// tested it (L §8.2 One read set; E §4.2 step 3; P20).
    pub(crate) fn merge_rows(&mut self, other: &Collected) {
        for (sid, seen) in &other.rows {
            self.rows.insert(sid.clone(), seen.clone());
        }
    }

    /// What of these reads decides whether a movement counts where it isn't counted (a
    /// refused one, a day outside a read's span, left out, covered, a false `when`): its pins,
    /// `solve`'s and its assumptions; never a `held:` value of its day (L §8.2 One read set,
    /// one statement for every form and outcome: P35 on #241).
    #[must_use]
    pub(crate) fn deciding(&self) -> Collected {
        let mut out = Collected::default();
        for e in &self.entries {
            if DECIDING.contains(&e.word) {
                out.entries.insert(e.clone());
            }
        }
        out
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
    pub(crate) fn without(&self, gone: &[Entry]) -> Collected {
        let mut out = self.clone();
        for e in gone {
            out.entries.remove(e);
        }
        out
    }

    /// Whether these reads met any register entry (L §8.2).
    #[must_use]
    pub fn has_entries(&self) -> bool {
        !self.entries.is_empty()
    }

    /// Whether these reads met entry `entry` (L §8.2).
    #[must_use]
    pub fn holds(&self, entry: &Entry) -> bool {
        self.entries.contains(entry)
    }

    /// The register entries in their text order: the word, the text, the figure, by code
    /// point (L §8.2; R-47).
    #[must_use]
    pub fn entries_sorted(&self) -> EntriesSorted<'_> {
        EntriesSorted {
            inner: self.entries.iter(),
        }
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
        if !self.versions.contains(version) {
            self.versions.insert(version.to_owned());
        }
    }

    /// The versions of the rows read, by code point (E §5 step 4).
    #[must_use]
    pub fn versions(&self) -> &BTreeSet<String> {
        &self.versions
    }

    /// Records that scenario fact `item` of section `section` was read (L §6.4; E §9 item 3).
    pub(crate) fn note_scenario(&mut self, (section, item): (&str, &str)) {
        self.scenario.insert((section.to_owned(), item.to_owned()));
    }

    /// Records that scenario fact `item` of section `section` was read, one a movement booked
    /// before the scenario's `from` lists (L §6.4, §8.2 `before from:`; E §9 item 3).
    pub(crate) fn note_listed(&mut self, (section, item): (&str, &str)) {
        self.note_scenario((section, item));
        self.listed.insert((section.to_owned(), item.to_owned()));
    }

    /// The scenarios' facts read, each (its section, its item), in that order (E §9 item 3).
    #[must_use]
    pub fn scenario(&self) -> &BTreeSet<(String, String)> {
        &self.scenario
    }

    /// The scenarios' facts read that a movement booked before the scenario's `from` lists
    /// (L §8.2 `before from:`).
    #[must_use]
    pub fn listed(&self) -> &BTreeSet<(String, String)> {
        &self.listed
    }

    /// Records row `sid` of `version` with its `locator` as read (E §10 `inputs-of`).
    pub(crate) fn note_row(&mut self, sid: &str, version: &str, locator: Option<&String>) {
        self.rows.insert(
            sid.to_owned(),
            RowSeen {
                version: version.to_owned(),
                locator: locator.cloned(),
            },
        );
    }

    /// The rows read, by stable id (E §10 `inputs-of`).
    #[must_use]
    pub fn rows(&self) -> &BTreeMap<String, RowSeen> {
        &self.rows
    }
}
