//! Indexes and versions: the keys an index holds in their order, and what a version states about its rows.
//! Spec: L §4.1 (indexes: lists, ranges, open, the built-in period indexes), §5.6 (keys and
//! their order), §4.5 (`known`), §4.6 (governs spans), §5.3 (versions, `enters`, `covers`,
//! gap roles, `moves`).
//! Twin: engine/tools/refcore/model.py (`IndexInfo`, `VersionInfo`, `span_at`); loader.py
//! (`build_indexes`, `build_range`, `build_versions`).
//! Invariants: an index's keys are in its order (an open index's by text); a key's position
//! is its place in that order; a version's known date, its window's last day too, is on or
//! before the model's `on`; a version's span under a cut is that of the latest-known `moves`
//! line the cut sees, else its own.

use super::key::{KeyVal, Span};
use crate::time::{Day, PeriodKind};

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
    pub keys: Vec<KeyVal>,
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
            added: Vec::new(),
            assumed: None,
            v: None,
            texts: Vec::new(),
            line: None,
            anchored: false,
        }
    }

    /// Puts an open index's keys in the text order of their names (refcore's
    /// `IndexInfo.finish`). L §2.5, §5.6.
    pub fn finish(&mut self) {
        if self.form == IndexForm::Open {
            self.keys.sort_by_key(KeyVal::text);
        }
    }

    /// The position of `key`, from 0 (refcore's `IndexInfo.pos`). L §4.2 `pos`.
    #[must_use]
    pub fn pos(&self, key: &KeyVal) -> Option<usize> {
        self.keys.iter().position(|k| k == key)
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
            KeyVal::Key { .. } | KeyVal::Thing(_) | KeyVal::Int(_) | KeyVal::Mv(_) => false,
        })
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
