//! Explanation traces: the derivation `explain` prints of a value, recorded while its cone is evaluated again with tracing on.
//! Spec: E §10 (`explain` re-evaluates the cell's cone with tracing on; a def cell: every rule
//! with its label and state, the winner with its `v` version and source, under the winner its
//! causes depth first in the order written, every row read with its version, source,
//! locator, governs span or instant, and known date; a `min`, `max`, `argmin` or `argmax` by
//! its first winning term; an `as if`: the pins with their values and the derivation inside
//! them beside the one outside); L §5.18 Explanation order (a rule's states: won; beaten, by
//! whom; not applicable; unknown).
//! Twin: engine/tools/refcore/evaluate.py (`TraceNode`, `def_cell` and `eval_rules` with
//! `tracing`, `trace_row`, `band_value`'s node, `asif_traced`, `pin_lines`, `trace_snap`,
//! `trace_winner`); ask.py (`evaluate`'s explain arm swaps the memo out, `explain_pins`).
//! Invariants: a node is added under the innermost open one, and none while a shared
//! structure (a walk, a plan, adjustments, links) evaluates, whose reads are its own (E §3.2);
//! a read that reaches one is noted, for the walk's forms, but a plain balance read, which is
//! a node of its own, printed in the balance form (ruling P63); a def cell traced once is a
//! reference to its node after; a cell computed from the work stack is traced apart, then
//! referenced by the read that replays; a read that replays starts its children again; a def
//! cell's rules show once its candidates are known, as refcore records them; the traced
//! evaluation keeps its cells in a memo of its own, dropped after it, so tracing changes no
//! value the ask's other reads see; a helper keeps the first stop it meets for the trace's
//! end, so the evaluator's recursive frames hold no more than a node's place (R-16).

use std::collections::{BTreeMap, BTreeSet};

mod parts;
pub(crate) use parts::{Keep, Part};

use super::memo::{CellId, CellKey, Cells, Open};
use super::stack::Item;
use super::state::{Ctx, Engine, Env, invariant};
use crate::base::Code;
use crate::model::{KeyVal, Reality, Row, Value, World};
use crate::read::{origin, row_known, row_span};
use crate::syntax::{Edge, Expr, Instant, Stop};
use crate::time::Day;

/// What a node of a trace is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Kind {
    /// The explained expression: its children are what it read.
    Root,
    /// A def cell, with the law date it ran at.
    Def(Option<Day>),
    /// A row read.
    Row,
    /// A band row read.
    BandRow,
    /// A group of causes: a rule's `when`, the winner's value, an `as if`'s pins.
    Group,
    /// An `as if`'s derivation inside its pins or outside them, its value printed after its
    /// label once known.
    Side,
    /// A plain balance read, printed in E §10's balance form from the walk it read (ruling
    /// P63, #132).
    Balance(Balance),
}

/// Whether a trace had noted a read of the walk, kept across a balance read (E §10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Walked(bool);

/// A traced balance read, what makes two reads one: the place and unit, the day and whether
/// at its start, whether `with uncertain`, the cut and the timeline it was read in (E §10,
/// §3.3). Its balance form is rendered on its node as it is read, from the walk it read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Balance {
    /// The place.
    pub place: String,
    /// The unit.
    pub unit: String,
    /// The day.
    pub day: Day,
    /// Whether the balance is at the day's start.
    pub start: bool,
    /// Whether the read counts uncertain movements (`with uncertain`).
    pub uncertain: bool,
    /// The cut.
    pub cut: Day,
    /// The timeline: its world and pins.
    pub timeline: super::timelines::TimelineKey,
}

/// One rule of a traced def cell, as `explain` prints it: its label, its state, and the
/// groups its `when` and, for the winner, its value read (refcore's `rule_states`,
/// `when_children`, `value_children`). L §5.18.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuleLine {
    /// Its label.
    pub label: String,
    /// Its state; none while it has none, and then it doesn't print.
    pub state: Option<RuleState>,
    /// The group its `when` read, if it has one and it was read.
    pub when: Option<usize>,
    /// The group the winner's value read.
    pub value: Option<usize>,
}

impl RuleLine {
    /// The rule with its state, where it has one: a rule that prints (L §5.18).
    pub(crate) fn stated(&self) -> Option<(&RuleLine, &RuleState)> {
        Some((self, self.state.as_ref()?))
    }
}

/// The state of a rule of a traced def cell (L §5.18 Explanation order: won; beaten, by
/// whom; not applicable; unknown, when its `when` is a hole; refcore's `rule_states`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RuleState {
    /// It won: `won`, with the version that states it and its source where it cites one,
    /// `won (version v, source s)`.
    Won(Option<(String, String)>),
    /// It applies and rules that beat it do, by their labels in their text's order:
    /// `beaten by a, b`.
    Beaten(Vec<String>),
    /// Its `when` is false: `not applicable`.
    NotApplicable,
    /// Its `when` is a hole: `unknown`.
    Unknown,
    /// It was the first candidate when a hole decided the value: `candidate`.
    Candidate,
}

impl RuleState {
    /// The state as `explain` prints it after the rule's label. L §5.18.
    pub(crate) fn text(&self) -> String {
        match self {
            RuleState::Won(None) => "won".to_owned(),
            RuleState::Won(Some((version, cite))) => {
                format!("won (version {version}, source {cite})")
            }
            RuleState::Beaten(by) => format!("beaten by {}", by.join(", ")),
            RuleState::NotApplicable => "not applicable".to_owned(),
            RuleState::Unknown => "unknown".to_owned(),
            RuleState::Candidate => "candidate".to_owned(),
        }
    }

    /// Whether the rule won, so its value's causes print under it. L §5.18.
    pub(crate) fn won(&self) -> bool {
        matches!(self, RuleState::Won(_))
    }
}

/// A node of a trace (refcore's `TraceNode`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Node {
    /// What it is.
    pub kind: Kind,
    /// Its label: a def cell's figure, a row's stable id, a group's words.
    pub label: String,
    /// A def cell's or a row's value, a refusal included, once known.
    pub value: Option<Item>,
    /// Lines about the node itself: a row's provenance, a def's `given`.
    pub lines: Vec<String>,
    /// What it read, in the order read, by node.
    pub children: Vec<usize>,
    /// A def cell's rules, by label: printed in the order of their labels' text.
    pub rules: BTreeMap<String, RuleLine>,
    /// Whether a def cell's rules show: once its candidates are known (refcore's
    /// `eval_rules`, where it sets `rule_states`).
    pub shown: bool,
    /// A def cell's def, by id, and its keys as its label writes them (refcore's `def_id`,
    /// `keys`): what a long derivation folds by (E §10; tally 2.1.7 S4).
    pub cell: Option<(String, String)>,
}

impl Node {
    /// A node of `kind` labelled `label`, nothing read yet.
    fn new(kind: Kind, label: String) -> Node {
        Node {
            kind,
            label,
            value: None,
            lines: Vec::new(),
            children: Vec::new(),
            rules: BTreeMap::new(),
            shown: false,
            cell: None,
        }
    }
}

/// The node of the explained expression.
const ROOT: usize = 0;

/// The node a cell computed from the work stack hangs under until the read that replays
/// references it; never printed.
const APART: usize = 1;

/// A trace being recorded: its nodes, the root first, the ones open now, each traced def
/// cell's node, the ask's memo, set aside while the cone is evaluated again, and the first
/// stop a helper met.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Trace {
    /// The nodes, the root first.
    nodes: Vec<Node>,
    /// The nodes open now, the innermost last.
    open: Vec<usize>,
    /// Each def cell traced, by its node.
    cells: BTreeMap<CellId, usize>,
    /// Each def cell traced by a term an extreme's explanation dropped, by its node, kept so
    /// a later read takes the node back rather than trace the cell's cone again: each cell
    /// is traced once per explain (E §3.2, §10; P7).
    dropped: BTreeMap<CellId, usize>,
    /// The ask's memo, back once the trace ends.
    saved: Cells,
    /// Whether a traced read reached the walk or a structure it shares: its causes are a
    /// balance's, a projection's or a movement's, the walk's forms (E §10).
    pub(crate) walked: bool,
    /// The first stop a helper met, the trace's end's.
    stop: Option<Stop>,
    /// Each decision move whose instances' days a read traced, by its walk: once per
    /// explain, as the plan pass lists them once (E §10).
    days: BTreeSet<(usize, String)>,
    /// Each decision run a read counted and traced, by its walk and instance: once per
    /// explain, as the walk runs it once (E §10).
    runs: BTreeSet<(usize, String)>,
    /// The marks of the plan pass traced now, where one is (P20).
    marks: Option<parts::Marks>,
    /// The marks of the last plan pass traced, by its move and cut, for the read that asked.
    fresh: Option<(String, Day, parts::Marks)>,
    /// The parts of plan passes a read set aside, for a read that counts their instances.
    stash: parts::Stash,
}

/// What a trace knows of a def cell a read reaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Seen {
    /// No read is traced now: the memo gives the cell.
    Untraced,
    /// A read is traced and the trace hasn't met the cell: it is evaluated again.
    Unmet,
    /// The trace met the cell, at this node: the memo gives it, referenced by its node.
    Met(usize),
    /// The trace met the cell at this node under a term an extreme's explanation dropped:
    /// the memo gives it, and its node is taken back where it is read now, printed in full
    /// where it is first reached, as the cell traced again would be (E §10).
    Dropped(usize),
}

/// Where a trace stood before a `min`, `max`, `argmin` or `argmax` read its terms: the node
/// open, its children then, and the nodes made then (refcore's `trace_snap`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Snap {
    /// The node open.
    parent: usize,
    /// How many children it had.
    children: usize,
    /// How many nodes there were.
    nodes: usize,
}

impl Trace {
    /// The node at `at`.
    pub(crate) fn node(&self, at: usize) -> Result<&Node, Stop> {
        self.nodes
            .get(at)
            .ok_or_else(|| invariant("a trace's node past its nodes"))
    }

    /// What the explained expression read, by node.
    pub(crate) fn top(&self) -> Result<&[usize], Stop> {
        Ok(self.node(ROOT)?.children.as_slice())
    }

    /// The node at `at`, to change.
    fn node_mut(&mut self, at: usize) -> Result<&mut Node, Stop> {
        self.nodes
            .get_mut(at)
            .ok_or_else(|| invariant("a trace's node past its nodes"))
    }

    /// Keeps the stop `done` met, the first only, for the trace's end.
    fn keep(&mut self, done: Result<(), Stop>) {
        if let Err(stop) = done
            && self.stop.is_none()
        {
            self.stop = Some(stop);
        }
    }

    /// Adds `node` under the innermost open node; returns its place.
    fn add(&mut self, node: Node) -> Result<usize, Stop> {
        let at = self.nodes.len();
        self.nodes.push(node);
        self.reference(at)?;
        Ok(at)
    }

    /// Adds `node`, kept or its stop kept; returns its place.
    fn added(&mut self, node: Node) -> Option<usize> {
        match self.add(node) {
            Ok(at) => Some(at),
            Err(stop) => {
                self.keep(Err(stop));
                None
            }
        }
    }

    /// Lists the node at `at` under the innermost open node.
    fn reference(&mut self, at: usize) -> Result<(), Stop> {
        let parent = *self
            .open
            .last()
            .ok_or_else(|| invariant("a trace with no node open"))?;
        self.node_mut(parent)?.children.push(at);
        Ok(())
    }

    /// Adds `node` apart from every other, as a rule's group is; returns its place.
    fn add_apart(&mut self, node: Node) -> usize {
        let at = self.nodes.len();
        self.nodes.push(node);
        at
    }

    /// The rule labelled `label` of the def cell's node at `at`, added with no state if it
    /// has none yet.
    fn rule_mut(&mut self, at: usize, label: &str) -> Result<&mut RuleLine, Stop> {
        let rules = &mut self.node_mut(at)?.rules;
        Ok(rules.entry(label.to_owned()).or_insert_with(|| RuleLine {
            label: label.to_owned(),
            state: None,
            when: None,
            value: None,
        }))
    }

    /// The trace of an ask's line explained under its `with` pins, this trace read inside
    /// them: under the root, the pins with their values, then this derivation with the value
    /// inside them, then each of `outside`'s, read outside them, with its options' tag and its
    /// value (refcore's `explain_pins`, `outside_of`). E §10, L §5.5.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a trace past the nodes it can number.
    pub(crate) fn beside_pins(
        mut self,
        outside: Vec<(String, Trace, Item)>,
        pins: Vec<String>,
        inside_value: Item,
    ) -> Result<Trace, Stop> {
        let mut outer = Vec::with_capacity(outside.len());
        for (tag, trace, value) in outside {
            let top = self.taken_in(trace)?;
            let mut side = Node::new(Kind::Side, format!("outside the pins{tag}"));
            side.value = Some(value);
            side.children = top;
            outer.push(side);
        }
        let inside_top = core::mem::take(&mut self.node_mut(ROOT)?.children);
        let mut group = Node::new(Kind::Group, "the pins, evaluated outside them".to_owned());
        for line in pins {
            let pin = Node::new(Kind::Group, format!("pin {line}"));
            group.children.push(self.add_apart(pin));
        }
        let mut inside = Node::new(Kind::Side, "inside the pins".to_owned());
        inside.value = Some(inside_value);
        inside.children = inside_top;
        let mut top = vec![self.add_apart(group), self.add_apart(inside)];
        for side in outer {
            top.push(self.add_apart(side));
        }
        self.node_mut(ROOT)?.children = top;
        Ok(self)
    }

    /// Trace `other`'s nodes taken into this one, each renumbered past this one's: the
    /// places of the causes its root read.
    fn taken_in(&mut self, other: Trace) -> Result<Vec<usize>, Stop> {
        let offset = self.nodes.len();
        let top: Vec<usize> = other
            .node(ROOT)?
            .children
            .iter()
            .map(|c| shifted(*c, offset))
            .collect::<Result<_, Stop>>()?;
        for mut node in other.nodes {
            node.children = node
                .children
                .iter()
                .map(|c| shifted(*c, offset))
                .collect::<Result<_, Stop>>()?;
            for rule in node.rules.values_mut() {
                rule.when = rule.when.map(|w| shifted(w, offset)).transpose()?;
                rule.value = rule.value.map(|v| shifted(v, offset)).transpose()?;
            }
            self.nodes.push(node);
        }
        Ok(top)
    }
}

impl Engine {
    /// Whether the ask is traced or lists its inputs, an `explain` or an `inputs-of`, whatever
    /// is evaluating now (E §10).
    pub(crate) fn explaining(&self) -> bool {
        self.trace.is_some() || self.keep_rows == super::Rows::Kept
    }

    /// Whether a read now is traced: an `explain` is being traced and no shared structure is
    /// evaluating (E §3.2, §10).
    pub(crate) fn tracing(&self) -> bool {
        self.trace.is_some() && !self.reads.iter().any(|o| matches!(o, Open::Apart))
    }

    /// The trace, where a read now is traced.
    fn traced(&mut self) -> Option<&mut Trace> {
        if !self.tracing() {
            return None;
        }
        self.trace.as_mut()
    }

    /// The trace and the place of its node `node`, where both are.
    fn traced_node(&mut self, node: Option<usize>) -> Option<(&mut Trace, usize)> {
        let (Some(at), Some(trace)) = (node, self.trace.as_mut()) else {
            return None;
        };
        Some((trace, at))
    }

    /// Starts tracing an `explain`: the ask's memo set aside for one of the trace's own, so
    /// the cone is evaluated again (refcore's `evaluate`, its explain arm). E §10.
    pub(crate) fn begin_trace(&mut self) {
        let fresh = self.cells.emptied();
        let saved = core::mem::replace(&mut self.cells, fresh);
        self.trace = Some(Trace {
            nodes: vec![
                Node::new(Kind::Root, String::new()),
                Node::new(Kind::Group, String::new()),
            ],
            open: vec![ROOT],
            cells: BTreeMap::new(),
            dropped: BTreeMap::new(),
            saved,
            walked: false,
            stop: None,
            days: BTreeSet::new(),
            runs: BTreeSet::new(),
            marks: None,
            fresh: None,
            stash: BTreeMap::new(),
        });
    }

    /// Ends the trace: the ask's memo back, the trace returned.
    ///
    /// # Errors
    ///
    /// The first stop a helper met while tracing; `ENGINE-INVARIANT` with no trace begun.
    pub(crate) fn end_trace(&mut self) -> Result<Trace, Stop> {
        let mut trace = self
            .trace
            .take()
            .ok_or_else(|| invariant("a trace ended with none begun"))?;
        core::mem::swap(&mut self.cells, &mut trace.saved);
        match trace.stop.take() {
            Some(stop) => Err(stop),
            None => Ok(trace),
        }
    }

    /// Keeps `stop`, the first only, for the trace's end.
    pub(super) fn trace_kept(&mut self, stop: Stop) {
        if let Some(trace) = self.trace.as_mut() {
            trace.keep(Err(stop));
        }
    }

    /// Notes that a traced read reaches the walk, or a structure the walk shares (a plan,
    /// adjustments, links, observations), whose causes `explain` prints in the walk's forms
    /// (E §10).
    pub(crate) fn trace_walk(&mut self) {
        if let Some(trace) = self.traced() {
            trace.walked = true;
        }
    }

    /// Gives the trace's note of a walk read back as it stood at `back`: the reads since were
    /// a structure the explanation prints in its own form, a move's instances traced or an
    /// observation pass that met no member (E §10).
    pub(crate) fn trace_walked_back(&mut self, back: Walked) {
        if let Some(trace) = self.trace.as_mut() {
            trace.walked = back.0;
        }
    }

    /// Opens a shared structure computed while a read is traced, a move's instances: a read
    /// set of its own, as [`Engine::reads_apart`] opens, so its reads are never the reading
    /// cell's, but traced, so the rows and cells it reads print as causes of the read (E §3.2,
    /// §10).
    pub(crate) fn trace_plan_open(&mut self) {
        self.open_reads();
    }

    /// Closes [`Engine::trace_plan_open`]; returns the trace's note of a walk read then.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a read set closed out of order.
    pub(crate) fn trace_plan_close(&mut self) -> Result<Walked, Stop> {
        self.close_reads()?;
        Ok(self.trace_walked())
    }

    /// Whether a read now is traced and hasn't yet traced the days of decision move
    /// `move_id`'s instances in walk `walk`; noted as traced from now on (E §10).
    pub(crate) fn trace_first_days(&mut self, walk: usize, move_id: &str) -> bool {
        self.traced()
            .is_some_and(|trace| trace.days.insert((walk, move_id.to_owned())))
    }

    /// Whether a read now is traced and hasn't yet traced decision run `inst` of walk
    /// `walk`; noted as traced from now on, so a run's reads print once per explain, where
    /// a read first counts it (E §10).
    pub(crate) fn trace_first_run(&mut self, walk: usize, inst: &str) -> bool {
        self.traced()
            .is_some_and(|trace| trace.runs.insert((walk, inst.to_owned())))
    }

    /// Whether the trace has noted a read of the walk, false with none: what a balance read
    /// gives back once its own read of the walk is its node (E §10).
    pub(crate) fn trace_walked(&self) -> Walked {
        Walked(self.trace.as_ref().is_some_and(|t| t.walked))
    }

    /// Adds a plain balance read and what it gave, refused or not, as a node printed in the
    /// balance form, where a read now is traced; its read of the walk is that node, so the
    /// note of a walk read goes back to `before`, as it stood before the read (ruling P63,
    /// #132). E §10.
    pub(crate) fn trace_balance(
        &mut self,
        m: &Reality,
        before: Walked,
        read: Balance,
        (got, walk): (Item, Option<usize>),
    ) {
        // the balance form now, from the walk the read used, in the timeline it is in force
        // in: the derivation prints after the line's reads, when another may be
        let lines = match walk {
            Some(id) => crate::ask::plain_lines(
                m,
                self,
                id,
                (&read.place, &read.unit),
                (read.day, read.start, read.uncertain),
            ),
            None => Ok(Vec::new()),
        };
        let lines = match lines {
            Ok(lines) => lines,
            Err(stop) => {
                self.trace_kept(stop);
                return;
            }
        };
        let Some(trace) = self.traced() else {
            return;
        };
        trace.walked = before.0;
        let start = if read.start { " start" } else { "" };
        let with = if read.uncertain {
            " with uncertain"
        } else {
            ""
        };
        let label = format!("bal({}, {}{start}){with}", read.place, read.day.text());
        let mut node = Node::new(Kind::Balance(read), label);
        node.value = Some(got);
        node.lines = lines;
        trace.added(node);
    }

    /// Opens the node of a written projection `read` from day `start` the walk refused, `r`,
    /// under the open node, for what the refused balance rests on to print under it; returns
    /// it, for [`Engine::trace_closed`] (L §8.2; E §10; P31).
    pub(crate) fn trace_refused_balance(
        &mut self,
        read: Balance,
        start: &str,
        r: &crate::base::Refusal,
    ) -> Option<usize> {
        let trace = self.traced()?;
        let label = format!("bal({}, {}) from {start}", read.place, read.day.text());
        let mut node = Node::new(Kind::Balance(read), label);
        node.value = Some(Item::Refused(r.clone()));
        let at = trace.added(node)?;
        trace.open.push(at);
        Some(at)
    }

    /// Closes the node at `at`, opened last by a read outside this module.
    pub(crate) fn trace_closed(&mut self, at: Option<usize>) {
        self.trace_close(at);
    }

    /// Whether a read now is traced and the trace has met def cell `key` at law date `law`,
    /// with its node, or met it under a term an extreme's explanation dropped.
    pub(super) fn trace_seen(&self, key: &CellKey, law: Option<Day>) -> Seen {
        let Some(trace) = self.trace.as_ref().filter(|_| self.tracing()) else {
            return Seen::Untraced;
        };
        let id = CellId::new(key.clone(), law);
        match (trace.cells.get(&id), trace.dropped.get(&id)) {
            (Some(at), _) => Seen::Met(*at),
            (None, Some(at)) => Seen::Dropped(*at),
            (None, None) => Seen::Unmet,
        }
    }

    /// References the node at `at` from the innermost open one: a cell met again (L §5.18
    /// "a cause met again is referenced by its line").
    pub(super) fn trace_again(&mut self, at: usize) {
        if let Some(trace) = self.trace.as_mut() {
            let done = trace.reference(at);
            trace.keep(done);
        }
    }

    /// Takes back the node at `at` of def cell `id`, met under a term an extreme's
    /// explanation dropped: met again from now on, and referenced from the innermost open
    /// node, where it prints in full if nothing printed it before, as the cell traced again
    /// would (E §10, §3.2: one evaluation per cell; P7).
    pub(super) fn trace_taken_back(&mut self, id: CellId, at: usize) {
        if let Some(trace) = self.trace.as_mut() {
            trace.dropped.remove(&id);
            trace.cells.insert(id, at);
            let done = trace.reference(at);
            trace.keep(done);
        }
    }

    /// Opens the node of def cell `id` of def `def` at keys `keys`, run at law date `law`;
    /// returns it.
    pub(super) fn trace_def(
        &mut self,
        id: &CellId,
        (def, keys): (&str, &[KeyVal]),
        law: Option<Day>,
    ) -> Option<usize> {
        let trace = self.traced()?;
        let label = origin(def, keys);
        let written = label
            .strip_prefix(def)
            .and_then(|rest| rest.strip_prefix('['))
            .and_then(|rest| rest.strip_suffix(']'))
            .map_or_else(String::new, str::to_owned);
        let mut node = Node::new(Kind::Def(law), label);
        node.cell = Some((def.to_owned(), written));
        let at = trace.added(node)?;
        trace.cells.insert(id.clone(), at);
        trace.open.push(at);
        Some(at)
    }

    /// Opens a group apart from the open node, for a rule's `when` or the winner's value;
    /// returns it.
    pub(super) fn trace_group(&mut self) -> Option<usize> {
        let trace = self.traced()?;
        let at = trace.add_apart(Node::new(Kind::Group, String::new()));
        trace.open.push(at);
        Some(at)
    }

    /// Opens a node of `kind` labelled `label` under the open node, a group or an `as if`'s
    /// side; returns it.
    pub(super) fn trace_open(&mut self, kind: Kind, label: &str) -> Option<usize> {
        let trace = self.traced()?;
        let at = trace.added(Node::new(kind, label.to_owned()))?;
        trace.open.push(at);
        Some(at)
    }

    /// Adds a group labelled `label` under the open node, with nothing under it: an `as if`'s
    /// pin with its value.
    pub(super) fn trace_leaf(&mut self, label: String) {
        if let Some(trace) = self.traced() {
            trace.added(Node::new(Kind::Group, label));
        }
    }

    /// Adds a running read of `place`, `value`, under the open node: a walk-bound def's cell
    /// read again at its walk point prints "its running reads as balances up to that point"
    /// of `day`, a hole with its question, as a line prints it (E §10, L §5.18 Output, §8.1;
    /// refcore's `e_bal`, its `now` form, `fmt_value`).
    ///
    /// # Errors
    ///
    /// A number error printing the value.
    pub(crate) fn trace_running(
        &mut self,
        m: &Reality,
        place: &str,
        value: &Value,
        day: Day,
    ) -> Result<(), Stop> {
        if self.traced().is_none() {
            return Ok(());
        }
        let label = format!(
            "running read bal({place}) = {}, the balance up to this point of {}",
            value.text(m).map_err(super::state::num_stop)?,
            day.text()
        );
        self.trace_leaf(label);
        Ok(())
    }

    /// Closes the node at `at`, opened last.
    pub(super) fn trace_close(&mut self, at: Option<usize>) {
        let Some((trace, at)) = self.traced_node(at) else {
            return;
        };
        if trace.open.pop() != Some(at) {
            trace.keep(Err(invariant("a trace's node closed out of order")));
        }
    }

    /// Sets the value of the node at `at`: a value, or the refusal its evaluation met; a stop
    /// that is no refusal sets none, since the read unwinds past it.
    pub(super) fn trace_value(&mut self, at: Option<usize>, result: &Result<Value, Stop>) {
        let Some((trace, at)) = self.traced_node(at) else {
            return;
        };
        let value = match result {
            Ok(v) => Some(Item::Value(v.clone())),
            Err(Stop::Refused(r)) => Some(Item::Refused(r.clone())),
            Err(_) => None,
        };
        match trace.node_mut(at) {
            Ok(node) => node.value = value,
            Err(stop) => trace.keep(Err(stop)),
        }
    }

    /// Sets the value of def cell `id`'s node at `at`, as [`Engine::trace_value`]; a stop that
    /// is no refusal also forgets the cell, so a later read evaluates it again rather than
    /// reference a node that holds no value. The memo may hold the cell's value from before
    /// the trace dropped it (`trace_drop`): the read past the counted depth unwinds, and its
    /// replay reads the cell again (E §3.2, §10).
    pub(super) fn trace_cell(
        &mut self,
        id: &CellId,
        at: Option<usize>,
        result: &Result<Value, Stop>,
    ) {
        self.trace_value(at, result);
        if matches!(result, Ok(_) | Err(Stop::Refused(_))) {
            return;
        }
        if let Some(trace) = self.trace.as_mut()
            && trace.cells.get(id).copied() == at
        {
            trace.cells.remove(id);
        }
    }

    /// Adds a line about the node at `at`: a def's `given`.
    pub(super) fn trace_line(&mut self, at: Option<usize>, line: String) {
        let Some((trace, at)) = self.traced_node(at) else {
            return;
        };
        match trace.node_mut(at) {
            Ok(node) => node.lines.push(line),
            Err(stop) => trace.keep(Err(stop)),
        }
    }

    /// Sets the state of the rule labelled `label` of the def cell's node `node`.
    pub(super) fn trace_state(&mut self, node: Option<usize>, label: &str, state: RuleState) {
        let Some((trace, at)) = self.traced_node(node) else {
            return;
        };
        match trace.rule_mut(at, label) {
            Ok(rule) => rule.state = Some(state),
            Err(stop) => trace.keep(Err(stop)),
        }
    }

    /// Sets the groups the rule labelled `label` of the def cell's node `node` read: its
    /// `when`'s, or the winner's value's.
    pub(super) fn trace_groups(
        &mut self,
        node: Option<usize>,
        label: &str,
        (when, value): (Option<usize>, Option<usize>),
    ) {
        let Some((trace, at)) = self.traced_node(node) else {
            return;
        };
        match trace.rule_mut(at, label) {
            Ok(rule) => {
                rule.when = when.or(rule.when);
                rule.value = value.or(rule.value);
            }
            Err(stop) => trace.keep(Err(stop)),
        }
    }

    /// Shows the rules of the def cell's node `node`: its candidates are known.
    pub(super) fn trace_shown(&mut self, node: Option<usize>) {
        let Some((trace, at)) = self.traced_node(node) else {
            return;
        };
        match trace.node_mut(at) {
            Ok(cell) => cell.shown = true,
            Err(stop) => trace.keep(Err(stop)),
        }
    }

    /// Adds the node of row `row` under the open node, its value to come: its stable id, then
    /// its version, source, locator, governs span or instant, and known date (refcore's
    /// `trace_row`). E §10.
    pub(super) fn trace_row(&mut self, m: &Reality, row: &Row) -> Option<usize> {
        let seen = (self.ask_cut, self.timeline_world());
        let trace = self.traced()?;
        match provenance(m, row, seen) {
            Ok(line) => {
                let mut node = Node::new(Kind::Row, row.sid.clone());
                node.lines.push(line);
                trace.added(node)
            }
            Err(stop) => {
                trace.keep(Err(stop));
                None
            }
        }
    }

    /// Adds the node of observation `id` of observed input `input` under the open node, a
    /// read of its leg giving `leg`: `row <version>:<id>` with its version, source, locator,
    /// day and known date, as a row prints (E §10: every row read, "the traced value equals
    /// the stored one"; refcore omits it, #169).
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for an observation or a version load didn't build.
    pub(crate) fn trace_observation(
        &mut self,
        m: &Reality,
        (input, id): (&str, &str),
        leg: &Value,
    ) -> Result<(), Stop> {
        if self.traced().is_none() {
            return Ok(());
        }
        let built = m
            .inputs
            .get(input)
            .ok_or_else(|| invariant("an observation's input load didn't build"))?;
        let o = built
            .observations
            .iter()
            .find(|o| o.sid() == id)
            .ok_or_else(|| invariant("an observation load didn't build"))?;
        let version = m
            .versions
            .get(&o.version)
            .ok_or_else(|| invariant("an observation of a version load didn't build"))?;
        let mut bits = vec![
            format!("version {}", o.version),
            format!("source {}", version.cite),
        ];
        if let Some(source) = m.sources.get(&version.cite) {
            bits.push(format!("\"{}\"", source.title));
        }
        if let Some(locator) = &o.locator {
            bits.push(format!("locator \"{locator}\""));
        }
        if let Some(day) = o.day {
            bits.push(format!("on {}", day.text()));
        }
        bits.push(format!(
            "known {}",
            o.own_known.unwrap_or(version.known).text()
        ));
        let mut node = Node::new(Kind::Row, format!("{}:{id}", o.version));
        node.lines.push(bits.join(", "));
        node.value = Some(Item::Value(leg.clone()));
        if let Some(trace) = self.traced() {
            trace.added(node);
        }
        Ok(())
    }

    /// Adds the node of a band row under the open node: `<figure>: row <interval> (version
    /// v[, "locator"])` (refcore's `band_value`, its node). E §10.
    pub(super) fn trace_band_row(
        &mut self,
        figure: &str,
        interval: &str,
        (version, locator): (&str, Option<&String>),
    ) {
        let Some(trace) = self.traced() else {
            return;
        };
        let located = locator.map_or_else(String::new, |l| format!(", \"{l}\""));
        let label = format!("{figure}: row {interval} (version {version}{located})");
        trace.added(Node::new(Kind::BandRow, label));
    }

    /// Moves what the trace listed under the open node since `since` to the place `mark`
    /// stood at among that node's children, after what earlier moves `done` (each mark with
    /// how many it took there) put before it: the reads of a hole booking's answer, listed
    /// where the instance's own reads end, as refcore answers each booking where it makes the
    /// instance (its `answer_booking`). None moves where `mark` stood under another node.
    /// E §10, §5 step 1.
    pub(crate) fn trace_move_to(&mut self, since: Snap, mark: Snap, done: &mut Vec<(Snap, usize)>) {
        let Some(trace) = self.trace.as_mut() else {
            return;
        };
        if mark.parent != since.parent || mark.children > since.children {
            return;
        }
        let node = match trace.node_mut(since.parent) {
            Ok(node) => node,
            Err(stop) => {
                trace.keep(Err(stop));
                return;
            }
        };
        if since.children > node.children.len() {
            return;
        }
        let mut to = mark.children;
        for (d, count) in done.iter() {
            if d.parent != mark.parent || d.children > mark.children {
                continue;
            }
            let Some(after) = to.checked_add(*count) else {
                return;
            };
            to = after;
        }
        if to > since.children {
            return;
        }
        // the children before `to`, those listed since `since`, then the rest before `since`
        let old = core::mem::take(&mut node.children);
        let count = old.iter().skip(since.children).count();
        let mut moved = Vec::with_capacity(old.len());
        moved.extend(old.iter().take(to));
        moved.extend(old.iter().skip(since.children));
        moved.extend(old.iter().take(since.children).skip(to));
        node.children = moved;
        done.push((mark, count));
    }

    /// Where the trace stands before a `min`, `max`, `argmin` or `argmax` reads its terms, or
    /// before a hole booking's answer is traced.
    pub(crate) fn trace_snap(&self) -> Option<Snap> {
        if !self.tracing() {
            return None;
        }
        let trace = self.trace.as_ref()?;
        let parent = *trace.open.last()?;
        let children = trace.nodes.get(parent)?.children.len();
        Some(Snap {
            parent,
            children,
            nodes: trace.nodes.len(),
        })
    }

    /// Drops what the terms traced since `snap`, so the first winning term can be traced
    /// again alone; the cells they traced are set aside, each node taken back where its cell
    /// is met again, never traced a second time (refcore's `trace_winner`, its drop, which
    /// traces them anew). L §5.18, E §10, §3.2; P7.
    fn trace_drop(&mut self, snap: Snap) {
        let Some(trace) = self.trace.as_mut() else {
            return;
        };
        match trace.node_mut(snap.parent) {
            Ok(node) => node.children.truncate(snap.children),
            Err(stop) => trace.keep(Err(stop)),
        }
        let (kept, dropped) = core::mem::take(&mut trace.cells)
            .into_iter()
            .partition(|(_, at)| *at < snap.nodes);
        trace.cells = kept;
        trace.dropped.extend::<BTreeMap<CellId, usize>>(dropped);
    }

    /// Explains a `min`, `max`, `argmin` or `argmax` whose result is `result` by its first
    /// winning term in binding order: what the terms traced since `snap` is dropped, and the
    /// first of `terms` (its expression, its scope, what it gives the result: its value, or an
    /// `argmin`'s or `argmax`'s key) that gives `result` is read again, alone; a hole or a
    /// window, as result or term, wins nothing (refcore's `trace_winner`). L §5.18, E §10.
    ///
    /// # Errors
    ///
    /// The stops the term's read meets that are no refusal, and `ENGINE-INVARIANT`.
    pub(super) fn trace_winner(
        &mut self,
        m: &Reality,
        (snap, result): (Snap, &Value),
        terms: &[(&Expr, &Env, Value)],
        ctx: Ctx,
    ) -> Result<(), Stop> {
        let open = |v: &Value| matches!(v, Value::Hole(_) | Value::Window(_));
        if open(result) {
            return Ok(());
        }
        let Some((term, env, _)) = terms.iter().find(|(_, _, v)| !open(v) && v == result) else {
            return Ok(());
        };
        self.trace_drop(snap);
        match self.ev(m, term, env, ctx) {
            Ok(_) => Ok(()),
            Err(Stop::Refused(r)) if r.code() != Code::EngineInvariant => Ok(()),
            Err(stop) => Err(stop),
        }
    }

    /// Where a read from outside the evaluator starts: the open node and its children then,
    /// which a replay of the read starts from again.
    pub(super) fn trace_mark(&self) -> Option<(usize, usize)> {
        let trace = self.trace.as_ref()?;
        let parent = *trace.open.last()?;
        Some((parent, trace.nodes.get(parent)?.children.len()))
    }

    /// Starts a replayed read's children again from `mark`.
    pub(super) fn trace_restart(&mut self, mark: Option<(usize, usize)>) {
        let (Some((parent, children)), Some(trace)) = (mark, self.trace.as_mut()) else {
            return;
        };
        match trace.node_mut(parent) {
            Ok(node) => node.children.truncate(children),
            Err(stop) => trace.keep(Err(stop)),
        }
    }

    /// Hangs what the work stack computes apart from the open node, until `trace_join`.
    pub(super) fn trace_apart(&mut self) {
        if let Some(trace) = self.trace.as_mut() {
            trace.open.push(APART);
        }
    }

    /// Ends `trace_apart`.
    pub(super) fn trace_join(&mut self) {
        let Some(trace) = self.trace.as_mut() else {
            return;
        };
        if trace.open.pop() != Some(APART) {
            trace.keep(Err(invariant("a trace's work stack joined out of order")));
        }
    }
}

/// The place of node `at` of a trace set after `offset` nodes of another.
fn shifted(at: usize, offset: usize) -> Result<usize, Stop> {
    at.checked_add(offset)
        .ok_or_else(|| invariant("a trace's node past its nodes"))
}

/// A row's provenance as `explain` prints it: `version v, source s, "title", locator "…",
/// governs <span>` or `at <instant>`, `known <day>` (refcore's `trace_row`). L §3.7, E §10.
fn provenance(m: &Reality, row: &Row, seen: (Day, World)) -> Result<String, Stop> {
    let version = m
        .versions
        .get(&row.version)
        .ok_or_else(|| invariant("a row of a version load didn't build"))?;
    let mut bits = vec![
        format!("version {}", row.version),
        format!("source {}", version.cite),
    ];
    if let Some(source) = m.sources.get(&version.cite) {
        bits.push(format!("\"{}\"", source.title));
    }
    if let Some(locator) = &row.locator {
        bits.push(format!("locator \"{locator}\""));
    }
    if let Some(Instant { day, edge }) = &row.instant {
        let edge = match edge {
            Some(Edge::Start) => " start",
            Some(Edge::End) => " end",
            None => "",
        };
        bits.push(format!("at {}{edge}", day.text()));
    } else {
        let span = row_span(m, row, seen)?.map_or_else(|| "?".to_owned(), |s| s.text());
        bits.push(format!("governs {span}"));
    }
    bits.push(format!("known {}", row_known(m, row)?.text()));
    Ok(bits.join(", "))
}
