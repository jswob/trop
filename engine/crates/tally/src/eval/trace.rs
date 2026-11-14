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

use std::collections::BTreeMap;

use super::memo::{CellId, CellKey, Cells, Open};
use super::stack::Item;
use super::state::{Ctx, Engine, Env, invariant};
use crate::base::Code;
use crate::model::{KeyVal, Reality, Row, RowInstant, Value};
use crate::read::{origin, row_known, row_span};
use crate::syntax::{Edge, Expr, Stop};
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
    /// Its state: `won (version v, source s)`, `beaten by a, b`, `not applicable`,
    /// `unknown`, `candidate`; empty while it has none, and then it doesn't print.
    pub state: String,
    /// The group its `when` read, if it has one and it was read.
    pub when: Option<usize>,
    /// The group the winner's value read.
    pub value: Option<usize>,
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
    /// The ask's memo, back once the trace ends.
    saved: Cells,
    /// Whether a traced read reached the walk or a structure it shares: its causes are a
    /// balance's, a projection's or a movement's, the walk's forms (E §10).
    pub(crate) walked: bool,
    /// The first stop a helper met, the trace's end's.
    stop: Option<Stop>,
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
            state: String::new(),
            when: None,
            value: None,
        }))
    }

    /// The trace of an ask's line explained under its `with` pins, this trace read inside
    /// them: under the root, the pins with their values, then this derivation with the value
    /// inside them, then `outside`'s, read outside them, with its value (refcore's
    /// `explain_pins`). E §10.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a trace past the nodes it can number.
    pub(crate) fn beside_pins(
        mut self,
        outside: Trace,
        pins: Vec<String>,
        (inside_value, outside_value): (Item, Item),
    ) -> Result<Trace, Stop> {
        let offset = self.nodes.len();
        let read_outside = &outside.node(ROOT)?.children;
        let outside_top: Vec<usize> = read_outside
            .iter()
            .map(|c| shifted(*c, offset))
            .collect::<Result<_, Stop>>()?;
        for mut node in outside.nodes {
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
        let inside_top = core::mem::take(&mut self.node_mut(ROOT)?.children);
        let mut group = Node::new(Kind::Group, "the pins, evaluated outside them".to_owned());
        for line in pins {
            let pin = Node::new(Kind::Group, format!("pin {line}"));
            group.children.push(self.add_apart(pin));
        }
        let mut inside = Node::new(Kind::Side, "inside the pins".to_owned());
        inside.value = Some(inside_value);
        inside.children = inside_top;
        let mut outer = Node::new(Kind::Side, "outside the pins".to_owned());
        outer.value = Some(outside_value);
        outer.children = outside_top;
        let top = vec![
            self.add_apart(group),
            self.add_apart(inside),
            self.add_apart(outer),
        ];
        self.node_mut(ROOT)?.children = top;
        Ok(self)
    }
}

impl Engine {
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
            saved,
            walked: false,
            stop: None,
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

    /// Whether a read now is traced and the trace has met def cell `key` at law date `law`,
    /// with its node.
    pub(super) fn trace_seen(&self, key: &CellKey, law: Option<Day>) -> Seen {
        let Some(trace) = self.trace.as_ref().filter(|_| self.tracing()) else {
            return Seen::Untraced;
        };
        match trace.cells.get(&CellId::new(key.clone(), law)) {
            Some(at) => Seen::Met(*at),
            None => Seen::Unmet,
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
    pub(super) fn trace_state(&mut self, node: Option<usize>, label: &str, state: String) {
        let Some((trace, at)) = self.traced_node(node) else {
            return;
        };
        match trace.rule_mut(at, label) {
            Ok(rule) => rule.state = state,
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
        let ask_cut = self.ask_cut;
        let trace = self.traced()?;
        match provenance(m, row, ask_cut) {
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

    /// Where the trace stands before a `min`, `max`, `argmin` or `argmax` reads its terms.
    pub(super) fn trace_snap(&self) -> Option<Snap> {
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
    /// again alone; the cells they traced are traced anew where met again (refcore's
    /// `trace_winner`, its drop). L §5.18, E §10.
    fn trace_drop(&mut self, snap: Snap) {
        let Some(trace) = self.trace.as_mut() else {
            return;
        };
        match trace.node_mut(snap.parent) {
            Ok(node) => node.children.truncate(snap.children),
            Err(stop) => trace.keep(Err(stop)),
        }
        trace.cells.retain(|_, at| *at < snap.nodes);
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
fn provenance(m: &Reality, row: &Row, ask_cut: Day) -> Result<String, Stop> {
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
    match &row.instant {
        Some(RowInstant::At(day, edge)) => {
            let edge = match edge {
                Some(Edge::Start) => " start",
                Some(Edge::End) => " end",
                None => "",
            };
            bits.push(format!("at {}{edge}", day.text()));
        }
        Some(RowInstant::After(_)) => {
            return Err(Stop::NotBuilt("explain of a reading after a movement"));
        }
        None => {
            let span = row_span(m, row, ask_cut)?.map_or_else(|| "?".to_owned(), |s| s.text());
            bits.push(format!("governs {span}"));
        }
    }
    bits.push(format!("known {}", row_known(m, row)?.text()));
    Ok(bits.join(", "))
}
