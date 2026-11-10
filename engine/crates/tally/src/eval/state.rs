//! The evaluator's state: a read's context, the variables in scope, the register entries a cell collects, and the engine that holds the memo.
//! Spec: E §3.2 (every value is a cell: an item, its keys and a context; the memo maps cells to
//! results, a value or a refusal, with the register entries the cell met and its read set);
//! L §4.3 (the law date), §4.5 (the cut), §5.18 (the ask line's `with uncertain`, in its own
//! reads' context only), §8.2 (the register); §7.6 items 1 and 4 (a day past the calendar and
//! the end of time read as a day, `past_days` and `a_day`; a unit's power past 64); §4.4 (a
//! read's world, `Ctx::world`: two reads of one item at the same keys in it are one value);
//! L §6.4 (the scenario world a read is in: the layers of the stack it sees).
//! Twin: engine/tools/refcore/evaluate.py (`Ctx`, `Coll`, `Engine` and its collector stack,
//! `past_days`).
//! Invariants: a cell is computed once per context and its result, refusal included, is kept;
//! a cell read while it is being computed is `RUN-CYCLE`; recursion through def cells is
//! counted, and past its limit the cell demanded waits on the work stack (R-16).

use std::collections::{BTreeMap, BTreeSet};

use crate::Days;
use crate::base::{Code, Refusal, Run, Warn};
use crate::book::{Adjustment, Coverage, Linked, Obs};
use crate::model::{Domain, KeyVal, Node, NodeKind, Reality, Thing, UnitError, Value, World};
use crate::num::NumError;
use crate::syntax::{Ask, AskBody, Expr, Stop};
use crate::time::Day;
use crate::walk::{Inst, ProjKey, Walk, WalkAt};

use super::bindings::reads_name;
use super::context::{Layer, Layers};
use super::memo::{CellId, Cells, Frame, Memo, Open, Reads};
use super::timelines::Timelines;

/// The deepest chain of def cells computed by plain calls; past it the cell demanded waits on
/// the work stack (R-16, BUILDER §4). U-15, measured at S: a debug build uses about 19 KB of
/// stack per def cell, so 32 cells take about 0.6 MB of the 2 MiB thread T's runner gives a
/// run, leaving the rest for expressions between cells and the walk's frames.
pub(super) const DEPTH_LIMIT: u32 = 32;

/// A read's context (refcore's `Ctx`, the parts S builds).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ctx {
    /// The law date d; none for a read with no law date (L §4.3).
    pub law: Option<Day>,
    /// The cut K: what the read sees (L §4.5).
    pub cut: Day,
    /// The walk point of a running read, inside a decision (L §5.14.1); none at the cut.
    pub at: Option<WalkAt>,
    /// The ask line's `with uncertain`, which every projection and binding the line's own
    /// reads reach takes (L §5.18). A timeline's and a decision's contexts are made anew
    /// without it, so no decision sees it (L §5.14: "the clause is the read's").
    pub unc: bool,
    /// The pins and wording pins in force, as one interned layer (E §3.2; `context.rs`). A
    /// timeline's and a decision's contexts take the timeline's: the pins the walk reaches.
    pub layer: Layer,
    /// The world read in: the reality's whole stack of scenarios, or the layers under one of
    /// them (L §6.4; refcore's `Ctx.world`). A timeline's contexts take the timeline's.
    pub world: World,
}

/// The walks up to a day of one timeline (refcore's `prefix_walks`, `plan_list`'s cache and
/// `_walk_blocked`). E §3.2, §4.1, §4.2 step 5.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Prefixes {
    /// Each walk up to a day, by (cut, its last day), by its place in the walks.
    pub(crate) walks: BTreeMap<(Day, Day), usize>,
    /// Each plan move's instances dated on or before a day, by (move, cut, day).
    pub(crate) plans: BTreeMap<(String, Day, Day), std::rc::Rc<Vec<Inst>>>,
    /// The cells in progress when each cut's main walk was last given up: while one of them
    /// still is, a leg's read takes the walk up to its day without trying the main walk.
    pub(crate) blocked: BTreeMap<Day, BTreeSet<CellId>>,
}

impl Ctx {
    /// The context of law date `law` and cut `cut`, at no walk point, without `with
    /// uncertain`, unpinned, in the reality's whole world (L §5.14).
    #[must_use]
    pub fn new(law: Option<Day>, cut: Day) -> Ctx {
        Ctx {
            law,
            cut,
            at: None,
            unc: false,
            layer: Layer::BASE,
            world: World::Whole,
        }
    }

    /// The context in world `world` (L §6.4).
    #[must_use]
    pub fn in_world(self, world: World) -> Ctx {
        Ctx { world, ..self }
    }

    /// The context under the ask line's `with uncertain` (L §5.18).
    #[must_use]
    pub fn with_uncertain(self) -> Ctx {
        Ctx { unc: true, ..self }
    }

    /// The context with law date `law`.
    #[must_use]
    pub fn at_law(self, law: Option<Day>) -> Ctx {
        Ctx { law, ..self }
    }

    /// The context as a read's text in the counted build's distinct reads (P §1.1): a running
    /// read's cells are kept per walk point (E §3.2).
    #[cfg(feature = "counters")]
    pub(super) fn key(self) -> String {
        self.world()
    }

    /// The world a read is made in, as text: its law date, cut, walk point, `with
    /// uncertain`, pins and scenario world; two reads of one item at the same keys in one
    /// world are one value (L §4.4, §6.4).
    pub(super) fn world(self) -> String {
        let law = self.law.map_or_else(|| "-".to_owned(), Day::text);
        let unc = if self.unc { "|uncertain" } else { "" };
        let pins = if self.layer.is_base() {
            String::new()
        } else {
            format!("|pins {}", self.layer.number())
        };
        let pins = match self.world {
            World::Whole => pins,
            World::Lowest(depth) => format!("{pins}|world {depth}"),
        };
        match self.at {
            None => format!("{law}|{}{unc}{pins}", self.cut.text()),
            Some(at) => format!(
                "{law}|{}|{}@{}{unc}{pins}",
                self.cut.text(),
                at.walk,
                at.point.text()
            ),
        }
    }
}

/// A bound variable: its name, its value, and the domain it ranges over when it is a key
/// (refcore's static `key ix` type of the variable).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Var {
    /// The name.
    pub name: String,
    /// The value.
    pub value: Value,
    /// The index or kind it is a key of.
    pub domain: Option<Domain>,
}

/// What a bare literal is where it stands: in a def, a rule or an ask only `0`, `1` and `-1`
/// are bare figures; in a band's row every bare literal is (L §3.2 Inline figures, §5.9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Figures {
    /// A def, a rule, a move or an ask.
    Rule,
    /// A band's row.
    Band,
}

/// The variables in scope: each name with its value, the latest bound last.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Env {
    /// The bindings, the innermost last.
    vars: Vec<Var>,
    /// How bare literals read here.
    pub figures: Figures,
}

impl Env {
    /// No variables, in a def's scope.
    #[must_use]
    pub fn empty() -> Env {
        Env {
            vars: Vec::new(),
            figures: Figures::Rule,
        }
    }

    /// A band row's scope: its argument bound, every bare literal a figure (L §5.9).
    #[must_use]
    pub fn band(arg: &str, x: Value) -> Env {
        Env {
            vars: vec![Var {
                name: arg.to_owned(),
                value: x,
                domain: None,
            }],
            figures: Figures::Band,
        }
    }

    /// The variable `name`, the innermost binding first.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Var> {
        self.vars.iter().rev().find(|v| v.name == name)
    }

    /// This scope with `name` bound to `value` too.
    #[must_use]
    pub fn with(&self, name: &str, value: Value, domain: Option<Domain>) -> Env {
        let mut out = self.clone();
        out.bind(name, value, domain);
        out
    }

    /// Whether expression `e` may read a variable of this scope (refcore's `_pertuple`'s
    /// test: a binding in scope whose variable the expression names).
    #[must_use]
    pub(crate) fn read_by(&self, e: &Expr) -> bool {
        self.vars.iter().any(|v| reads_name(e, &v.name))
    }

    /// Binds `name` to `value` in this scope.
    pub fn bind(&mut self, name: &str, value: Value, domain: Option<Domain>) {
        self.vars.push(Var {
            name: name.to_owned(),
            value,
            domain,
        });
    }
}

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

/// What a cell's evaluation met: its register entries (refcore's `Coll.reg`), and whether a
/// `missing(…)` read directly in it held (refcore's `Coll.missing`, never merged outward).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Collected {
    /// The register entries.
    pub entries: BTreeSet<Entry>,
    /// Whether a `missing(r)` evaluated in this collector, not in a cell below it, was true.
    pub missing: bool,
    /// The versions of the rows read (refcore's `Coll.rows`, their versions): which versions
    /// a later step's adjustment names (E §5 step 4).
    pub versions: BTreeSet<String>,
    /// The scenarios' facts read, each (its section, its item): a row of a scenario's
    /// version, a thing's entry or place a scenario states, a scenario's move a binding
    /// ranges over: the read set E §9 item 3 lists a movement by (L §6.4, ruling P45).
    pub scenario: BTreeSet<(String, String)>,
    /// The rows read, by stable id, each with its version and locator: an input's, a given,
    /// a band row, a reading and an observed row, the walk's counted movements' included
    /// (refcore's `Coll.rows`). Kept only by an engine answering `inputs-of` (E §3.2, §10).
    pub rows: BTreeMap<String, RowSeen>,
}

/// A row read, as `inputs-of` lists it: its version and its locator (E §10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowSeen {
    /// The version that states it.
    pub version: String,
    /// Its locator, where it states one.
    pub locator: Option<String>,
}

/// Whether the collectors keep the rows read: an engine answering `inputs-of` keeps them,
/// any other doesn't (E §10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rows {
    /// The rows read are kept with what each read met.
    Kept,
    /// They aren't.
    Unkept,
}

impl Collected {
    /// Adds the register entries, versions and rows `other` met (its `missing` stays its own).
    pub fn merge(&mut self, other: &Collected) {
        self.entries.extend(other.entries.iter().cloned());
        self.versions.extend(other.versions.iter().cloned());
        self.scenario.extend(other.scenario.iter().cloned());
        for (sid, seen) in &other.rows {
            self.rows.insert(sid.clone(), seen.clone());
        }
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
}

/// A memo cell: the result of a def at its keys in one context, with what it met and what it
/// read (E §3.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Cell {
    /// The value, or the refusal.
    pub result: Result<Value, Stop>,
    /// What its evaluation met.
    pub met: Collected,
    /// Its read set: the inputs, rows, band rows and cells it read.
    pub reads: Reads,
}

/// A def cell waiting on the work stack: demanded past the counted depth, it is computed from
/// the outermost read, with the cells that were being computed above it still in progress.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Task {
    /// The def.
    pub def: String,
    /// Its keys.
    pub keys: Vec<KeyVal>,
    /// The cell's context, its law date set by the def's `at`.
    pub ctx: Ctx,
    /// The cells in progress that unwound so it could be computed from depth 0: they stay in
    /// progress until it is, then the read waiting on it replays them.
    pub unwound: Vec<CellId>,
}

/// The evaluator: the memo, the collector stack and the work stack (refcore's `Engine`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Engine {
    /// The memo: each cell's result, kept for the law dates of its class (E §3.2).
    pub(super) cells: Cells,
    /// The classes of the def cells open now, the innermost last (E §3.2).
    pub(super) frames: Vec<Frame>,
    /// The read sets open now, the innermost last: each open def cell's, and a shared
    /// structure's whose reads are its own (E §3.2).
    pub(super) reads: Vec<Open>,
    /// The cells being computed.
    pub(crate) active: BTreeSet<CellId>,
    /// The collectors, the innermost last; never empty.
    pub(super) stack: Vec<Collected>,
    /// The def cells open now.
    pub(super) depth: u32,
    /// The cells demanded past the counted depth, the latest last (R-16).
    pub(super) work: Vec<Task>,
    /// The ask's cut, past which a row read for a later day is `held` (L §8.2).
    pub(crate) ask_cut: Day,
    /// The walks, by their place in the list (E §3.3).
    pub(crate) walks: Vec<Walk>,
    /// The main walk of each cut's timeline, by its place in `walks`.
    pub(crate) main_walks: BTreeMap<Day, usize>,
    /// The walks being walked now: a read of one of them is a read of a walk under
    /// construction.
    pub(crate) walking: BTreeSet<usize>,
    /// The day source: the event walk, or the oracle's every day (P §2.2).
    pub(crate) days: Days,
    /// Each move's instances per cut (E §4.1).
    pub(crate) plans: BTreeMap<(String, Day), std::rc::Rc<Vec<Inst>>>,
    /// The moves whose instances are being computed (RUN-CYCLE).
    pub(crate) plan_active: BTreeSet<(String, Day)>,
    /// Each plan move's adjustments per cut (E §5).
    pub(crate) adjustments: BTreeMap<(String, Day), Vec<Adjustment>>,
    /// The moves whose adjustments are being computed (RUN-CYCLE).
    pub(crate) adjust_active: BTreeSet<(String, Day)>,
    /// The cells in progress when a leg's read started its walk: a walk that needs one of them
    /// is given up for the walk up to the leg's day (refcore's `guarded_walk`).
    pub(crate) guarded: Option<BTreeSet<CellId>>,
    /// The walks up to a day in the timeline in force, what their plan passes list, and the
    /// cells that last made a cut's main walk be given up (refcore's `prefix_walks`,
    /// `plan_list`, `_walk_blocked`; E §3.2, §4.1).
    pub(crate) prefixes: Prefixes,
    /// While a walk up to a day plans: that day, past which its plan pass lists no plan's
    /// instance (refcore's `_body_upto`). E §4.1.
    pub(crate) body_upto: Option<Day>,
    /// The warnings of the walks done, in the order met (L §8.5).
    pub(crate) walk_warnings: Vec<(Warn, String)>,
    /// The projections' walks, by what makes each (E §4.3).
    pub(crate) proj_walks: BTreeMap<ProjKey, usize>,
    /// How a written projection's common case is read: fence F5's tree, or its plain twin,
    /// the projection's own walk (E §4.3, P §2.3).
    pub(crate) lows: crate::walk::Lows,
    /// The fault planted in the run, while it waits for its day: none once planted, and
    /// always none without it (INTERFACE §4, pick P8).
    #[cfg(feature = "fault-injection")]
    pub(crate) plant: Option<crate::walk::Fault>,
    /// Fence F5's trees in the timeline in force (E §4.3, P §2.3; `walk/lows.rs`).
    pub(crate) trees: crate::walk::Trees,
    /// The item whose expression is evaluated now: a projection's holder, whose dependents
    /// it leaves out (L §5.14.1).
    pub(crate) holder: Option<Node>,
    /// The observations each cut sees (L §5.19).
    pub(crate) observed: BTreeMap<Day, Vec<Obs>>,
    /// The versions each cut sees that cover each place (L §5.19 Coverage).
    pub(crate) covered: BTreeMap<Day, Coverage>,
    /// Each instance's links, judged per cut (L §5.19 Links).
    pub(crate) links: BTreeMap<(Day, String), Linked>,
    /// The links being judged (`RUN-CYCLE`).
    pub(crate) link_active: BTreeSet<(Day, String)>,
    /// The refusals of observations' records, per cut, by observation (L §5.19 Links).
    pub(crate) obs_errs: BTreeMap<(Day, String), Refusal>,
    /// The adjustments a settlement move's binding over adjustments is held to while its
    /// candidates are made (refcore's `_adj_restrict`, E §5 step 5).
    pub(crate) adj_restrict: Option<Vec<Adjustment>>,
    /// Each plan move's adjustments per cut with their settlements linked (E §5 step 5).
    pub(crate) settled: BTreeMap<(String, Day), Vec<Adjustment>>,
    /// Every adjustment per cut, unsettled: what a binding over adjustments ranges over (E §5).
    pub(crate) unsettled: BTreeMap<Day, Vec<Adjustment>>,
    /// The cuts whose adjustments are being measured (RUN-CYCLE).
    pub(crate) adjust_all_active: BTreeSet<Day>,
    /// Each booked plan instance's amount A₀ by (instance, booking cut, a walk's limit), kept
    /// across a run's asks (E §9 item 5).
    pub(crate) booked: crate::walk::Bookings,
    /// Each main walk's signature once it is done, by its place in `walks` (E §9 item 4).
    pub(crate) signatures: BTreeMap<usize, crate::invariant::Signature>,
    /// The legs each combination's main walk of a cut refused, where the main walk read an
    /// axis the timeline sets no option of (E §3.3, §4.2 step 5).
    pub(crate) each_refused: BTreeMap<Day, Vec<super::timelines::LegRefusals>>,
    /// The pass a business-day function runs now over the days its calendar's added windows
    /// can take (L §4.2, §4.4).
    pub(super) calendar_pass: Option<super::business::CalendarPass>,
    /// The splits `prorata`'s `rest largest` made, once per total, step and weights (L §5.13
    /// Cost).
    pub(super) splits: super::prorata::Splits,
    /// The context's interned layers: the pin sets and wording pins met, the open scopes
    /// (E §3.2, P §3.1, §3.5).
    pub(super) layers: Layers,
    /// The timeline in force, and the others parked: one per world and pins the walk reaches
    /// (E §3.3; `timelines.rs`). The walk fields above hold the one in force.
    pub(super) timelines: Timelines,
    /// The base a scenario's expression row being evaluated names by its prefix, and the
    /// world it is read in (L §6.4; `read.rs` `row_expr`).
    pub(super) row_base: Option<(String, World)>,
    /// The expression rows being evaluated, by row, the read's figure and its law date: one met
    /// again reads itself (refcore's `_sexpr_active`, `RUN-CYCLE`).
    pub(super) rows_active: BTreeSet<(String, String, Option<Day>)>,
    /// Whether the collectors keep the rows read: an engine answering `inputs-of` (E §10).
    pub(crate) keep_rows: Rows,
    /// The explanation being traced: an engine answering `explain` while its cone is
    /// evaluated again (E §10; `trace.rs`).
    pub(super) trace: Option<super::trace::Trace>,
    /// The ask line's context outside its `with` pins, where it has any: an `explain`'s
    /// derivation outside them (E §10; refcore's `_outside_ctx`).
    pub(crate) unpinned: Option<Ctx>,
    /// What this engine did, in P §1.1's units: the `counters` build only (BUILDER §13.2).
    #[cfg(feature = "counters")]
    pub(crate) counts: crate::Counters,
}

impl Engine {
    /// An engine with an empty memo, for an ask whose cut is `ask_cut`, walking the event
    /// walk's days.
    #[must_use]
    pub fn new(ask_cut: Day) -> Engine {
        Engine {
            cells: Cells::new(Memo::Classes),
            frames: Vec::new(),
            reads: Vec::new(),
            active: BTreeSet::new(),
            stack: vec![Collected::default()],
            depth: 0,
            work: Vec::new(),
            ask_cut,
            walks: Vec::new(),
            main_walks: BTreeMap::new(),
            walking: BTreeSet::new(),
            days: Days::Events,
            plans: BTreeMap::new(),
            plan_active: BTreeSet::new(),
            adjustments: BTreeMap::new(),
            adjust_active: BTreeSet::new(),
            guarded: None,
            prefixes: Prefixes::default(),
            body_upto: None,
            walk_warnings: Vec::new(),
            proj_walks: BTreeMap::new(),
            lows: crate::walk::Lows::Tree,
            #[cfg(feature = "fault-injection")]
            plant: None,
            trees: crate::walk::Trees::default(),
            holder: None,
            observed: BTreeMap::new(),
            covered: BTreeMap::new(),
            links: BTreeMap::new(),
            link_active: BTreeSet::new(),
            obs_errs: BTreeMap::new(),
            adj_restrict: None,
            settled: BTreeMap::new(),
            unsettled: BTreeMap::new(),
            adjust_all_active: BTreeSet::new(),
            booked: BTreeMap::new(),
            signatures: BTreeMap::new(),
            each_refused: BTreeMap::new(),
            calendar_pass: None,
            splits: super::prorata::Splits::default(),
            layers: Layers::default(),
            timelines: Timelines::default(),
            row_base: None,
            rows_active: BTreeSet::new(),
            keep_rows: Rows::Unkept,
            trace: None,
            unpinned: None,
            #[cfg(feature = "counters")]
            counts: crate::Counters::default(),
        }
    }

    /// The engine holding the booked plan amounts a run's earlier asks met, so a later ask's
    /// cut is checked against them (E §9 item 5).
    #[must_use]
    pub fn with_booked(self, booked: crate::walk::Bookings) -> Engine {
        Engine { booked, ..self }
    }

    /// What this engine did, in P §1.1's units: the `counters` build only (BUILDER §13.2).
    #[cfg(feature = "counters")]
    #[must_use]
    pub fn counters(&self) -> &crate::Counters {
        &self.counts
    }

    /// The engine walking the days of `days`.
    #[must_use]
    pub fn with_days(self, days: Days) -> Engine {
        Engine { days, ..self }
    }

    /// The engine computing as its fences say: keeping its cells as `memo` says, shared by a
    /// law date class or at each exact law date, the plain twin (E §3.2, P §3.3); and reading
    /// a written projection's common case as `lows` says, from fence F5's tree or through the
    /// projection's own walk, the plain twin (E §4.3, P §2.3).
    #[must_use]
    pub fn with_how(self, (memo, lows): (Memo, crate::walk::Lows)) -> Engine {
        Engine {
            cells: Cells::new(memo),
            lows,
            ..self
        }
    }

    /// The engine answering `ask`: the holder of the projections its lines write (L §5.14.1).
    /// Its `with uncertain` is in the line's context, not here (L §5.18, [`Ctx::with_uncertain`]).
    #[must_use]
    pub fn for_ask(self, ask: &Ask) -> Engine {
        Engine {
            holder: Some(Node::new(NodeKind::Ask, &ask.id)),
            // `explain` prints the row a balance's reading came from (E §10)
            keep_rows: match ask.body {
                AskBody::InputsOf(_) | AskBody::Explain(_) => Rows::Kept,
                AskBody::Exprs(_)
                | AskBody::Residuals(_)
                | AskBody::ReadersOf(_)
                | AskBody::Check(..)
                | AskBody::Solve(_)
                | AskBody::Diff(_) => Rows::Unkept,
            },
            ..self
        }
    }

    /// Evaluates with `item` as the holder of the projections met, the outer holder back
    /// after (L §5.14.1: a projection leaves out what depends on the item holding it).
    pub(crate) fn holding(&mut self, item: Node) -> Option<Node> {
        self.holder.replace(item)
    }

    /// The warnings the walks met, in the order met (L §8.5).
    #[must_use]
    pub fn walk_warnings(&self) -> &[(Warn, String)] {
        &self.walk_warnings
    }

    /// Walk `id`.
    pub(crate) fn walk(&self, id: usize) -> Result<&Walk, Stop> {
        self.walks
            .get(id)
            .ok_or_else(|| invariant("a walk the engine lacks"))
    }

    /// Walk `id`, to change.
    pub(crate) fn walk_mut(&mut self, id: usize) -> Result<&mut Walk, Stop> {
        self.walks
            .get_mut(id)
            .ok_or_else(|| invariant("a walk the engine lacks"))
    }

    /// Records a register entry in the innermost collector (L §8.2).
    pub(crate) fn note(&mut self, word: &'static str, text: String, figure: String) {
        if let Some(top) = self.stack.last_mut() {
            top.entries.insert(Entry { word, text, figure });
        }
    }

    /// Records that a scenario's fact was read, its section and its item, in the innermost
    /// collector: the read set E §9 item 3 lists a movement by (L §6.4, ruling P45).
    pub(crate) fn note_scenario(&mut self, section: &str, item: &str) {
        if let Some(top) = self.stack.last_mut() {
            top.scenario.insert((section.to_owned(), item.to_owned()));
        }
    }

    /// Records the scenarios' facts of thing `thing` a read in world `world` met: the
    /// sections of its entries, its place and its key things a scenario states (L §5.7: a
    /// lifespan is a fact; §6.4). E §9 item 3.
    pub(crate) fn note_thing(&mut self, m: &Reality, thing: &Thing, world: World) {
        for section in m.thing_scenario_sections(thing, world) {
            self.note_scenario(&section, &thing.name);
        }
    }

    /// Records that a row of version `version` was read, in the innermost collector (E §5
    /// step 4: the versions an adjustment names).
    pub(super) fn note_version(&mut self, version: &str) {
        if let Some(top) = self.stack.last_mut() {
            top.versions.insert(version.to_owned());
        }
    }

    /// Records that row `sid` of `version`, with its `locator`, was read, in the innermost
    /// collector of an engine answering `inputs-of` (E §10).
    pub(super) fn note_row(&mut self, sid: &str, version: &str, locator: Option<&String>) {
        if self.keep_rows == Rows::Unkept {
            return;
        }
        if let Some(top) = self.stack.last_mut() {
            top.note_row(sid, version, locator);
        }
    }

    /// Marks that a `missing(…)` read in the innermost collector held (L §8.2 `missing`).
    pub(super) fn note_missing(&mut self) {
        if let Some(top) = self.stack.last_mut() {
            top.missing = true;
        }
    }

    /// Opens a collector.
    pub(crate) fn push(&mut self) {
        self.stack.push(Collected::default());
    }

    /// Closes the innermost collector and returns what it met, without adding it to the one
    /// outside.
    pub(crate) fn pop_alone(&mut self) -> Collected {
        if self.stack.len() > 1
            && let Some(top) = self.stack.pop()
        {
            return top;
        }
        Collected::default()
    }

    /// Adds what a closed collector met to the innermost one.
    pub(crate) fn absorb(&mut self, met: &Collected) {
        if let Some(top) = self.stack.last_mut() {
            top.merge(met);
        }
    }

    /// What the outermost collector has met so far: the register of the reads made.
    #[must_use]
    pub fn collected(&self) -> Collected {
        match self.stack.first() {
            Some(outer) => outer.clone(),
            None => Collected::default(),
        }
    }
}

/// A refusal with its code and message (L §8.4).
pub(super) fn refuse(code: Run, message: String) -> Stop {
    Stop::Refused(Refusal::new(Code::Run(code), message))
}

/// A load refusal met at run, as refcore raises it there (L §8.3).
pub(super) fn refuse_load(code: crate::base::Load, message: String) -> Stop {
    Stop::Refused(Refusal::new(Code::Load(code), message))
}

/// A value of a type its place doesn't take, at a place whose every run-time value load
/// types: a test (a bool, a hole, `none`; a window's comparison is a hole), a date
/// `date_or_window` checked, the number `round` takes, `pos`'s key, a schedule's roll code.
/// Load refuses it first (`LOAD-UNIT`, L §8.4). A place that `never`, a window or an
/// interval can reach answers them itself (L §3.1, §4.4), or stops "not implemented".
pub(super) fn not_typed() -> Stop {
    invariant("a value of a type its place doesn't take: load's types refuse it first (LOAD-UNIT)")
}

/// A value of a type its place doesn't take: `LOAD-UNIT` with `what` and the value as it
/// prints (refcore raises it where load's checker would have).
pub(super) fn wrong_type(m: &Reality, what: &str, v: &Value) -> Stop {
    match v.text(m) {
        Ok(text) => refuse_load(crate::base::Load::Unit, format!("{what} {text}")),
        Err(error) => num_stop(error),
    }
}

/// `RUN-LIMIT` for a day computed by a step of sign `toward` past 9999-12-31, or before
/// 0001-01-01 (refcore's `past_days`). L §7.6 item 1, §8.4.
pub(crate) fn past_days(toward: i64) -> Stop {
    let message = if toward > 0 {
        "a day passes 9999-12-31"
    } else {
        "a day comes before 0001-01-01"
    };
    refuse(Run::Limit, message.to_owned())
}

/// A day read as a day: the end of time is none, `RUN-LIMIT`. L §7.6 item 1.
pub(crate) fn a_day(day: Day) -> Result<Day, Stop> {
    if day.is_end() {
        return Err(past_days(1));
    }
    Ok(day)
}

/// A broken promise of load's or the evaluator's: `ENGINE-INVARIANT` (E §11).
pub(super) fn invariant(what: &str) -> Stop {
    Stop::Refused(Refusal::new(Code::EngineInvariant, what.to_owned()))
}

/// A number error as the refusal refcore raises (L §7.1, §8.4).
pub(crate) fn num_stop(error: NumError) -> Stop {
    match error {
        NumError::Div => refuse(Run::Div, "division by zero".to_owned()),
        NumError::Digits => refuse(
            Run::Digits,
            "a value passes 4 096 digits; round where the law rounds".to_owned(),
        ),
        NumError::Domain => refuse(Run::Domain, "outside its operation's domain".to_owned()),
        NumError::Invariant => invariant("the number kernel's invariant"),
    }
}

/// A unit operation's error met at run (`UnitError::refusal`): a power past 64 in size is the
/// load refusal refcore's `power_limit` raises where it meets it (a unit a key gives,
/// L §3.2); a number error is `num_stop`'s. L §7.6 item 4.
pub(crate) fn unit_stop(error: UnitError) -> Stop {
    error.refusal(num_stop, None)
}
