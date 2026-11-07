//! What load decides from the item graph: cycles, guarded recurrences, unread items, running reads, the decisions' reach, walk-bound defs, left-out moves, the strata.
//! Spec: E §3.1 steps 6–8; L §5.11 (a same-key cycle; a recurrence read guarded by a
//! condition on its key; reading itself both earlier and later), §5.14.1 (running reads
//! and `reads`, decisions, walk-bound defs, the movements a projection leaves out),
//! §5.14.2 (the order within a day: the decision graph, written `before` and `after`,
//! cycles), §4.6 (a reading `after` a plan or an observation), §8.3 (`LOAD-CYCLE`,
//! `-GUARD`, `-READS`, `-ORDER`), §5.5 (`LOAD-FORKS`), §8.5 (`WARN-UNREAD`).
//! Twin: engine/tools/refcore/check.py (`Checker.run` from `check_graph` on, `check_graph`,
//! `_cycles`); check3.py (`_tname`, `_covers`, `m_check_moves_graph`, `m_strata`, `_meets`,
//! `m_check_after_readings`).
//! Invariants: every search of the graph is a loop over an explicit stack (R-16); cycles are
//! searched from sorted nodes over sorted reads, as refcore searches them, so a refusal names
//! the cycle refcore names; the moves' running reads are checked by id and their marks in
//! the order of refcore's `repr` of them, an explicit text key; the strata are the layers of
//! an acyclic graph once each cycle no written order breaks is one component.

use super::across::{
    check_literal_ranges, check_period_keys, check_proj_seen, check_scenario_moves,
    check_schedules, check_seen_day_limits, check_seen_days,
};
use super::check::{Files, ShiftRead, walk};
use super::cone::{check_path_lines, finish_random};
use super::lawdates::{check_lawdates, check_move_lawdates};
use super::parts::{arrival_reads, parts_walk_bound};
use super::pins::check_forks;
use super::srows::check_srow_reads;
use super::steps::{Loader, refuse};
use super::types::text;
use crate::base::{Code, Load, Refusal, Warn};
use crate::model::{
    GapRole, Graph, Mark, MoveInfo, Node, NodeKind, Reality, Rel, RowInstant, Shift, Target, Ty,
    pattern_text,
};
use crate::syntax::{Expr, Field, Lit, Order, Postfix, Section, Stop, Value as Written};
use std::collections::{BTreeMap, BTreeSet};

/// An invariant of the graph's own searches broken: a cycle or a layer that must exist.
fn broken(what: &str) -> Stop {
    Stop::Refused(Refusal::new(Code::EngineInvariant, what.to_owned()))
}

/// A node's state in a depth-first search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Visit {
    /// On the current path.
    Open,
    /// Done with.
    Done,
}

impl Loader {
    /// The item graph and what load decides from it, in refcore's order (`Checker.run` from
    /// `check_graph` on): the walk; same-key cycles, recurrence reads and their guards, law
    /// dates, the asks' axis combinations, unread items; the running reads, the decisions' reach, walk-bound defs (an
    /// arrival rule's among them), left-out moves, the strata; an arrival rule's reads of its
    /// own day; readings `after` a plan; a decision's balances of days load sees; periods
    /// across keys; moves' law dates; what a scenario's expression rows read; schedules that
    /// give no date (issue #72). A scenario's reality keeps only the asks its world evaluates.
    /// E §3.1 steps 5–8; L §4.2, §4.3, §5.11, §5.14.1, §5.14.2, §6.4.
    pub(super) fn build_graph(&mut self) -> Result<(), Stop> {
        let files = Files::of(&self.sections);
        let mut walked = walk(&self.m, &self.sections)?;
        self.m.asks.retain(|ask| !walked.dropped.contains(&ask.id));
        self.m.asks_alone = core::mem::take(&mut walked.alone);
        for fact in core::mem::take(&mut walked.facts) {
            if let Some(mv) = self.m.moves.get_mut(&fact.id) {
                mv.read_targets = fact.read_targets;
                mv.changes = fact.changes;
            }
        }
        same_key_cycles(&self.m, &walked.graph)?;
        recurrences(&walked.graph, &walked.shifts)?;
        let dated = check_lawdates(&self.m, &walked, &files)?;
        walked.graph.ask_axes = check_forks(&self.m, &walked.graph, &files)?;
        let unread_items = unread(&self.m, &walked.graph);
        self.m.warnings.extend(unread_items);
        let decided = from_decisions(&self.m, &walked.graph);
        running_reads(&self.m, &walked.graph, &decided, &files)?;
        let mut graph = core::mem::take(&mut walked.graph);
        graph.from_decisions = decided;
        let mut bound = walk_bound(&self.m, &graph);
        bound.extend(parts_walk_bound(&self.m, &graph));
        graph.walk_bound = bound;
        graph.left_out = left_out(&graph);
        self.m.graph = graph;
        self.strata()?;
        arrival_reads(&self.m, &self.m.graph, &files)?;
        after_readings(&self.m, &files)?;
        check_seen_days(&self.m, &walked.day_reads, &files)?;
        check_seen_day_limits(&self.m, &files)?;
        check_literal_ranges(&self.m, &files)?;
        check_period_keys(&self.m, &walked.periods)?;
        check_move_lawdates(&self.m, &walked, &dated, &files)?;
        let random = finish_random(
            &self.m,
            &walked.queries,
            &walked.orient_items,
            &walked.draws,
            &files,
        )?;
        check_proj_seen(&self.m, &walked.projs, &random)?;
        check_srow_reads(&self.m, &walked.srows, &walked.read_order)?;
        check_schedules(&self.m, &files)?;
        check_scenario_moves(&self.m, &files)?;
        check_path_lines(&self.m, &walked.queries)
    }

    /// The decision graph and its layers: who acts after whom, the written orders, the
    /// cycles no order breaks, each decision's stratum (refcore's `m_strata`). E §3.1 step 8;
    /// L §5.14.2.
    fn strata(&mut self) -> Result<(), Stop> {
        let m = &self.m;
        let mut decisions: Vec<String> = m
            .moves
            .entries()
            .filter(|(_, mv)| mv.decision)
            .map(|(id, _)| id.clone())
            .collect();
        decisions.sort();
        let mut after = BTreeMap::new();
        for a in &decisions {
            let acts_after = decisions
                .iter()
                .filter(|b| *b != a && acts_after(m, a, b))
                .cloned()
                .collect();
            after.insert(a.clone(), acts_after);
        }
        let mut written = BTreeSet::new();
        for a in &decisions {
            let Some(mv) = m.moves.get(a) else {
                continue;
            };
            for (order, other) in &mv.decl.order {
                self.written_order(a, *order, other, &mut after, &mut written)?;
            }
        }
        let groups = match order_cycles(&self.m.graph, &after, &decisions) {
            Ok(groups) => merge(groups),
            Err(cycle) => return Err(self.unordered(&cycle)),
        };
        for group in &groups {
            join(group, &mut after);
        }
        let levels = levels(&self.m, &after)?;
        let mut classes = Vec::new();
        for (id, mv) in self.m.moves.entries() {
            let stratum = if mv.decision {
                *levels
                    .get(id)
                    .ok_or_else(|| broken("a decision has no stratum"))?
            } else {
                0
            };
            let reads_own = mv
                .read_targets
                .iter()
                .any(|read| !matches!(read, Target::Move(_)) && meets(&self.m, read, &mv.changes));
            classes.push((id.clone(), stratum, mv.decision && reads_own));
        }
        for (id, stratum, reads_own) in classes {
            if let Some(mv) = self.m.moves.get_mut(&id) {
                mv.stratum = stratum;
                mv.self_reads = reads_own;
            }
        }
        self.m.graph.after = after;
        self.m.graph.order_groups = groups;
        self.m.graph.written_order = written;
        Ok(())
    }

    /// `a before b` or `a after b`, a decision's written order against another decision
    /// (refcore's `m_strata`, its `before` and `after` lines). L §5.14.2.
    fn written_order(
        &self,
        a: &str,
        order: Order,
        b: &str,
        after: &mut BTreeMap<String, BTreeSet<String>>,
        written: &mut BTreeSet<(String, String)>,
    ) -> Result<(), Stop> {
        let word = match order {
            Order::Before => "before",
            Order::After => "after",
        };
        if !self.m.moves.contains(b) {
            return Err(self.at_move(a, Load::Name, format!("unknown move {b} ({word} {b})")));
        }
        if !after.contains_key(b) {
            if order == Order::Before {
                let message = format!("{a} before {b}: {b} is a plan or an observation");
                return Err(self.at_move(a, Load::Order, message));
            }
            return Ok(());
        }
        let reads_legs = self
            .m
            .moves
            .get(a)
            .is_some_and(|mv| mv.read_targets.contains(&Target::Move(b.to_owned())));
        let (later, earlier) = match order {
            Order::Before if reads_legs => {
                let message = format!("{a} before {b} contradicts a read of {b}'s legs");
                return Err(self.at_move(a, Load::Order, message));
            }
            Order::Before => (b, a),
            Order::After => (a, b),
        };
        if let Some(set) = after.get_mut(later) {
            set.insert(earlier.to_owned());
        }
        if let Some(set) = after.get_mut(earlier) {
            set.remove(later);
        }
        written.insert((later.to_owned(), earlier.to_owned()));
        Ok(())
    }

    /// `LOAD-ORDER` for decisions that read each other on days load knows, with no
    /// written order. L §5.14.2.
    fn unordered(&self, cycle: &[String]) -> Stop {
        let Some(first) = cycle.first() else {
            return broken("an empty cycle of decisions");
        };
        let message = format!(
            "decisions {} read each other with no before or after",
            cycle.join(" and ")
        );
        self.at_move(first, Load::Order, message)
    }

    /// A refusal at the move `id`'s section.
    fn at_move(&self, id: &str, code: Load, message: String) -> Stop {
        let found = self.sections.iter().find_map(|(file, section)| {
            if let Section::Move(mv) = section
                && mv.id == id
            {
                Some((file.as_str(), mv.line))
            } else {
                None
            }
        });
        match found {
            Some((file, line)) => refuse(code, message, file, line),
            None => Stop::Refused(Refusal::new(Code::Load(code), message)),
        }
    }
}

/// Same-key cycles among defs and bands: a read at the reader's own key, or a band's read
/// of an unkeyed def (refcore's `check_graph`, `_cycles`). L §5.11; E §3.1 step 6.
fn same_key_cycles(m: &Reality, graph: &Graph) -> Result<(), Stop> {
    let mut same: BTreeMap<Node, BTreeSet<Node>> = BTreeMap::new();
    for (reader, reads) in &graph.edges {
        if !is_def_or_band(reader) {
            continue;
        }
        for (target, rels) in reads {
            let unkeyed_def = reader.kind == NodeKind::Band
                && target.kind == NodeKind::Def
                && m.defs
                    .get(&target.id)
                    .is_some_and(|def| def.binds.is_empty());
            if is_def_or_band(target) && (rels.contains(&Rel::Same) || unkeyed_def) {
                same.entry(reader.clone())
                    .or_insert_with(BTreeSet::new)
                    .insert(target.clone());
            }
        }
    }
    let Some(cycle) = first_cycle(&same) else {
        return Ok(());
    };
    let Some(again) = cycle.last() else {
        return Err(broken("an empty same-key cycle"));
    };
    let ids: Vec<&str> = cycle.iter().map(|node| node.id.as_str()).collect();
    Err(Stop::Refused(Refusal::new(
        Code::Load(Load::Cycle),
        format!(
            "{} depends on itself at the same key: {}",
            again.id,
            ids.join(" -> ")
        ),
    )))
}

/// The recurrence reads: a def's read at a shifted key of a def that reads it back is
/// `LOAD-GUARD` unless a condition on the key variable guards it, and `LOAD-CYCLE` when it
/// reads itself both earlier and later on one variable (refcore's `check_graph`, its
/// `shift_reads` loop and `dirs`). L §5.11.
fn recurrences(graph: &Graph, shifts: &[ShiftRead]) -> Result<(), Stop> {
    // (reader, variable) → whether it reads earlier, later, in the order first met
    let mut ways: Vec<(&Node, &str, bool, bool)> = Vec::new();
    for read in shifts {
        if read.target != read.reader && !graph.reaches(&read.target, &read.reader) {
            continue;
        }
        if !read.guarded {
            let var = &read.var;
            let sign = match read.shift {
                Shift::Earlier => "-",
                Shift::Later => "+",
            };
            let message = format!(
                "{}[{var} {sign} n] is a recurrence read with no condition on {var} or pos({var}) guarding it",
                read.target.id
            );
            return Err(refuse(Load::Guard, message, &read.at.0, read.at.1));
        }
        let earlier = read.shift == Shift::Earlier;
        match ways
            .iter_mut()
            .find(|(reader, var, ..)| *reader == &read.reader && *var == read.var)
        {
            Some(way) => {
                way.2 = way.2 || earlier;
                way.3 = way.3 || !earlier;
            }
            None => ways.push((&read.reader, &read.var, earlier, !earlier)),
        }
    }
    let Some((reader, var, ..)) = ways
        .iter()
        .find(|(_, _, earlier, later)| *earlier && *later)
    else {
        return Ok(());
    };
    Err(Stop::Refused(Refusal::new(
        Code::Load(Load::Cycle),
        format!("{} reads itself both earlier and later on {var}", reader.id),
    )))
}

/// The running reads and projections an item reaches through defs and its moves' days,
/// each once, ordered as refcore sorts them (by `repr` of its `(mark, target)` tuple;
/// refcore's `reached`). E §3.1 step 7.
fn reached_marks(graph: &Graph, item: &Node) -> Vec<(Mark, Target)> {
    let mut found: Vec<(String, Mark, Target)> = Vec::new();
    let mut seen = BTreeSet::new();
    let mut stack = vec![item.clone()];
    while let Some(next) = stack.pop() {
        if !seen.insert(next.clone()) {
            continue;
        }
        for (mark, target) in graph.marks.get(&next).into_iter().flatten() {
            let text = format!("('{}', {})", mark.word(), target_repr(target));
            if !found.iter().any(|(old, ..)| *old == text) {
                found.push((text, *mark, target.clone()));
            }
        }
        for target in graph.reads(&next).into_iter().flat_map(BTreeMap::keys) {
            if carries_marks(target) {
                stack.push(target.clone());
            }
        }
    }
    found.sort_by(|a, b| a.0.cmp(&b.0));
    found
        .into_iter()
        .map(|(_, mark, target)| (mark, target))
        .collect()
}

/// A target as Python's `repr` prints refcore's tuple for it, which orders refcore's sort.
fn target_repr(target: &Target) -> String {
    match target {
        Target::Thing(name) => format!("('thing', '{name}')"),
        Target::PlaceSec(id) => format!("('placesec', '{id}')"),
        Target::Move(id) => format!("('move', '{id}')"),
        Target::Kind(id) => format!("('kind', '{id}')"),
        Target::Unknown => "('unknown',)".to_owned(),
    }
}

/// What a target names in a refusal (refcore's `_tname`).
fn target_name(target: &Target) -> &str {
    match target {
        Target::Thing(id) | Target::PlaceSec(id) | Target::Move(id) | Target::Kind(id) => id,
        Target::Unknown => "a place",
    }
}

/// Whether a declared read covers a running read's target: the same, a keyed place and one
/// of its instances, an event kind and its moves or subkinds, or anything load can't see
/// (refcore's `_covers`). L §5.14.1.
fn covers(m: &Reality, declared: &[Target], target: &Target) -> bool {
    *target == Target::Unknown || declared.iter().any(|d| covers_one(m, d, target))
}

/// Whether one declared read covers a running read's target.
fn covers_one(m: &Reality, declared: &Target, target: &Target) -> bool {
    if *declared == Target::Unknown || declared == target {
        return true;
    }
    if let (Target::PlaceSec(section), Target::Thing(thing)) = (declared, target) {
        return section_holds(m, section, thing);
    }
    let Target::Kind(kind) = declared else {
        return false;
    };
    if !m.kinds.contains(kind) {
        return false;
    }
    let sub = m.subtree(kind);
    match target {
        Target::Kind(other) => sub.contains(other),
        Target::Move(id) => m.moves.get(id).is_some_and(|mv| sub.contains(&mv.kind)),
        Target::Thing(_) | Target::PlaceSec(_) | Target::Unknown => false,
    }
}

/// `LOAD-READS`: a plan that reaches a running read or a projection; a decision whose day
/// reads running state, that reaches one of a place it doesn't declare, or that declares a
/// read it never makes; then `bal(x)` with no day outside a decision and the defs it reaches
/// (refcore's `m_check_moves_graph`, its `LOAD-READS` and `balnow_items`). L §5.14.1; E §3.1
/// step 7.
fn running_reads(
    m: &Reality,
    graph: &Graph,
    decided: &BTreeSet<Node>,
    files: &Files,
) -> Result<(), Stop> {
    let mut ids: Vec<&String> = m.moves.entries().map(|(id, _)| id).collect();
    ids.sort();
    for id in ids {
        let Some(mv) = m.moves.get(id) else {
            continue;
        };
        if let Some(message) = move_reads(m, graph, id, mv) {
            return Err(refuse(
                Load::Reads,
                message,
                files.get("move", id)?,
                mv.decl.line,
            ));
        }
    }
    let Some(item) = graph.balnow.iter().find(|item| !decided.contains(*item)) else {
        return Ok(());
    };
    Err(Stop::Refused(Refusal::new(
        Code::Load(Load::Reads),
        format!(
            "{} holds bal(x) with no day: a running read outside a decision and its defs",
            item.id
        ),
    )))
}

/// The `LOAD-READS` message for one move, where it needs one (refcore's
/// `m_check_moves_graph`, its loop body). L §5.14.1.
fn move_reads(m: &Reality, graph: &Graph, id: &str, mv: &MoveInfo) -> Option<String> {
    let reached = reached_marks(graph, &Node::new(NodeKind::Move, id));
    let without = |target: &Target| {
        let name = target_name(target);
        format!("{id} reaches a running read or projection of {name} without reads {name}")
    };
    if !mv.decision {
        return reached.first().map(|(_, target)| without(target));
    }
    if let Some((_, target)) = reached_marks(graph, &Node::new(NodeKind::MoveOn, id)).first() {
        return Some(format!(
            "{id}: its day reads running state ({}); a decision's day reads no running state",
            target_name(target)
        ));
    }
    if let Some((_, target)) = reached
        .iter()
        .find(|(_, target)| !covers(m, &mv.read_targets, target))
    {
        return Some(without(target));
    }
    if reached.iter().any(|(_, target)| *target == Target::Unknown) {
        return None;
    }
    for declared in &mv.read_targets {
        let read = reached
            .iter()
            .any(|(_, target)| covers_one(m, declared, target));
        if !read {
            let name = target_name(declared);
            return Some(format!("{id} declares reads {name} and never reads it"));
        }
    }
    None
}

/// `LOAD-ORDER` for a reading `after m` of `stated` or `opening`, with m a plan or an
/// observation: load sees m's class, and the batch has no order (refcore's
/// `m_check_after_readings`). L §4.6 Readings, §5.14.2, §8.3.
fn after_readings(m: &Reality, files: &Files) -> Result<(), Stop> {
    for id in ["stated", "opening"] {
        let Some(input) = m.inputs.get(id) else {
            continue;
        };
        for row in input.rows.iter().chain(&input.part_rows) {
            let Some(RowInstant::After(node)) = &row.instant else {
                continue;
            };
            let mut base = node;
            if let Expr::Post(inner, posts) = node
                && let [Postfix::Keys(_)] = posts.as_slice()
            {
                base = inner;
            }
            let Expr::Name(name) = base else {
                continue;
            };
            let plan = m.moves.get(name).is_some_and(|mv| !mv.decision);
            let observed = m.inputs.get(name).is_some_and(|named| named.observed);
            if plan || observed {
                let shown = text(node);
                let message = format!(
                    "{id}{} after {shown}: {shown} is a plan or an observation",
                    pattern_text(&row.pattern)
                );
                return Err(refuse(
                    Load::Order,
                    message,
                    files.of_version(&row.version)?,
                    row.line,
                ));
            }
        }
    }
    Ok(())
}

/// Whether a node is a def or a band.
fn is_def_or_band(node: &Node) -> bool {
    matches!(node.kind, NodeKind::Def | NodeKind::Band)
}

/// The first cycle a depth-first search meets, from each node in order over its sorted
/// reads, closed by its first node (refcore's `_cycles`).
fn first_cycle(graph: &BTreeMap<Node, BTreeSet<Node>>) -> Option<Vec<Node>> {
    let mut state: BTreeMap<Node, Visit> = BTreeMap::new();
    for start in graph.keys() {
        if state.contains_key(start) {
            continue;
        }
        state.insert(start.clone(), Visit::Open);
        let mut stack = vec![(start.clone(), pending(graph.get(start)))];
        while let Some((node, reads)) = stack.last_mut() {
            let Some(next) = reads.pop() else {
                state.insert(node.clone(), Visit::Done);
                stack.pop();
                continue;
            };
            match state.get(&next) {
                Some(Visit::Open) => return Some(closed_cycle(&stack, next)),
                Some(Visit::Done) => {}
                None => {
                    state.insert(next.clone(), Visit::Open);
                    let next_reads = pending(graph.get(&next));
                    stack.push((next, next_reads));
                }
            }
        }
    }
    None
}

/// A node's reads still to visit, the first last, so `pop` takes them in order.
fn pending(reads: Option<&BTreeSet<Node>>) -> Vec<Node> {
    reads.map_or_else(Vec::new, |reads| reads.iter().rev().cloned().collect())
}

/// The cycle from `again`'s place on the path back to it, `again` written at both ends.
fn closed_cycle(stack: &[(Node, Vec<Node>)], again: Node) -> Vec<Node> {
    let path = stack.iter().map(|(node, _)| node);
    let mut cycle: Vec<Node> = path.skip_while(|node| **node != again).cloned().collect();
    cycle.push(again);
    cycle
}

/// `input x is read by nothing` for an input that isn't built in or observed, `def d is read
/// by nothing`, and `place p is read by nothing` for a `place` section no line names, for
/// each nothing reads (refcore's `check_graph`, its end). L §8.5; E §3.1 step 6.
fn unread(m: &Reality, graph: &Graph) -> Vec<(Warn, String)> {
    // A def an arrival rule or a takes clause reads is read (tally 2.1, refcore's
    // `parts_reads`).
    let read: BTreeSet<&Node> = graph
        .edges
        .values()
        .flat_map(BTreeMap::keys)
        .chain(&graph.parts_reads)
        .collect();
    let mut out = Vec::new();
    for (id, input) in m.inputs.entries() {
        let unread = !read.contains(&Node::new(NodeKind::Input, id));
        if unread && !input.builtin && !input.observed {
            out.push((Warn::Unread, format!("input {id} is read by nothing")));
        }
    }
    for (id, _) in m.defs.entries() {
        if !read.contains(&Node::new(NodeKind::Def, id)) {
            out.push((Warn::Unread, format!("def {id} is read by nothing")));
        }
    }
    let named = named_places(m, &read);
    for (id, _) in m.places.entries() {
        if !named.contains(id) {
            out.push((Warn::Unread, format!("place {id} is read by nothing")));
        }
    }
    out
}

/// The `place` sections some line names (tally 2.1.2): a read, a leg or a binding over its
/// kind (a `Place` node among the graph's targets), a row whose value is one of its things,
/// an end of an observed row's leg, either place of a version's `pad x from y`. A row keyed
/// by the place (`opening`, a reading), an `anchor` and a `covers` line name none (refcore's
/// `check_graph`, `values_naming`). L §8.5; E §3.1 step 6.
fn named_places(m: &Reality, read: &BTreeSet<&Node>) -> BTreeSet<String> {
    let mut owner: BTreeMap<&str, &str> = BTreeMap::new();
    for (id, place) in m.places.entries() {
        for (_, name) in &place.things {
            owner.insert(name.as_str(), id.as_str());
        }
    }
    let mut things: Vec<String> = Vec::new();
    for (_, input) in m.inputs.entries() {
        if let Some(ty) = &input.ty {
            for row in &input.rows {
                row_things(m, &row.value, ty, &mut things);
            }
        }
        for leg in input.observations.iter().flat_map(|o| &o.legs) {
            things.push(leg.from.clone());
            things.push(leg.to.clone());
        }
    }
    for (_, def) in m.defs.entries() {
        for row in &def.givens {
            row_things(m, &row.value, &def.ty, &mut things);
        }
    }
    for (_, version) in m.versions.entries() {
        for (place, role) in &version.gap_roles {
            if let GapRole::Pad(from) = role {
                things.push(place.clone());
                things.push(from.clone());
            }
        }
    }
    let mut named: BTreeSet<String> = read
        .iter()
        .filter(|node| node.kind == NodeKind::Place)
        .map(|node| node.id.clone())
        .collect();
    named.extend(
        things
            .iter()
            .filter_map(|thing| owner.get(thing.as_str()))
            .map(|id| (*id).to_owned()),
    );
    named
}

/// The things a row's written value names where its type takes a thing of a kind: the value
/// itself or a record's fields, each literal read as a read converts it, never refused here
/// (refcore's `values_naming` over `convert_value`). A list row is a `dates` value, its
/// items days, so it names none. L §8.5, §4.2, §3.1.
fn row_things(m: &Reality, value: &Written, ty: &Ty, out: &mut Vec<String>) {
    match value {
        Written::Lit(lit) => out.extend(thing_of(m, lit, ty)),
        Written::Record(fields) => {
            let Ty::Record(types) = ty else {
                return;
            };
            for (name, field_ty) in types {
                let lit = fields.iter().find_map(|f| match f {
                    Field::Named(n, v) if n == name => crate::eval::field_literal(v),
                    Field::Named(..) | Field::Leg(_) => None,
                });
                out.extend(lit.and_then(|lit| thing_of(m, &lit, field_ty)));
            }
        }
        Written::List(_)
        | Written::Hole(_)
        | Written::Never
        | Written::None
        | Written::Str(_)
        | Written::Calendar(_)
        | Written::Days(_)
        | Written::Expr(_)
        | Written::Dist(_) => {}
    }
}

/// The thing a key literal is under `key k` of a kind: an index's key names none (refcore's
/// `key_value`, its index before its kind). L §4.2.
fn thing_of(m: &Reality, lit: &Lit, ty: &Ty) -> Option<String> {
    let Ty::Key(kind) = ty else {
        return None;
    };
    if m.indexes.get(kind).is_some() {
        return None;
    }
    let text = lit.text();
    m.thing_in(&text, kind).then_some(text)
}

/// Whether a mark travels along a read of `target`: through defs and a move's day, never a
/// band, a move or a place (refcore's `reached`).
fn carries_marks(target: &Node) -> bool {
    matches!(target.kind, NodeKind::Def | NodeKind::MoveOn)
}

/// The decisions, and the defs and days they reach (refcore's `from_decisions`). E §3.1
/// step 7.
fn from_decisions(m: &Reality, graph: &Graph) -> BTreeSet<Node> {
    let mut out = BTreeSet::new();
    for (id, mv) in m.moves.entries() {
        if !mv.decision {
            continue;
        }
        let mut stack = vec![Node::new(NodeKind::Move, id)];
        while let Some(next) = stack.pop() {
            if let Some(reads) = graph.reads(&next)
                && !out.contains(&next)
            {
                stack.extend(reads.keys().filter(|t| carries_marks(t)).cloned());
            }
            out.insert(next);
        }
    }
    out
}

/// Whether `item` reaches a running read or a projection through defs and days (refcore's
/// `reached`, non-empty).
pub(super) fn reaches_mark(graph: &Graph, item: &Node) -> bool {
    let mut seen = BTreeSet::new();
    let mut stack = vec![item.clone()];
    while let Some(next) = stack.pop() {
        if graph
            .marks
            .get(&next)
            .is_some_and(|marks| !marks.is_empty())
        {
            return true;
        }
        if let Some(reads) = graph.reads(&next)
            && seen.insert(next)
        {
            stack.extend(reads.keys().filter(|t| carries_marks(t)).cloned());
        }
    }
    false
}

/// The walk-bound defs: each reaches a mark and a decision reaches it. E §3.1 step 7.
fn walk_bound(m: &Reality, graph: &Graph) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for (id, _) in m.defs.entries() {
        let def = Node::new(NodeKind::Def, id);
        if graph.from_decisions.contains(&def) && reaches_mark(graph, &def) {
            out.insert(def.id);
        }
    }
    out
}

/// For each holder of a projection, the moves that depend on it, which its projection
/// leaves out; an ask's leaves out none (refcore's `m_check_moves_graph`, `lo_memo`).
/// L §5.14.1.
fn left_out(graph: &Graph) -> BTreeMap<Node, BTreeSet<String>> {
    let mut readers: BTreeMap<&Node, Vec<&Node>> = BTreeMap::new();
    for (reader, reads) in &graph.edges {
        for target in reads.keys() {
            readers.entry(target).or_insert_with(Vec::new).push(reader);
        }
    }
    let mut out = BTreeMap::new();
    for holder in &graph.holders {
        let start = match holder.kind {
            NodeKind::Ask => {
                out.insert(holder.clone(), BTreeSet::new());
                continue;
            }
            NodeKind::MoveOn => Node::new(NodeKind::Move, &holder.id),
            NodeKind::Input
            | NodeKind::Def
            | NodeKind::Band
            | NodeKind::Move
            | NodeKind::Place
            | NodeKind::Price
            | NodeKind::Balances
            | NodeKind::Parts
            | NodeKind::Axis
            | NodeKind::Fill
            | NodeKind::Srow => holder.clone(),
        };
        out.insert(holder.clone(), dependent_moves(&readers, start));
    }
    out
}

/// The moves that read `start`, directly or through other items.
fn dependent_moves(readers: &BTreeMap<&Node, Vec<&Node>>, start: Node) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut moves = BTreeSet::new();
    let mut stack = vec![start];
    while let Some(next) = stack.pop() {
        if !seen.insert(next.clone()) {
            continue;
        }
        if next.kind == NodeKind::Move {
            moves.insert(next.id.clone());
        }
        if let Some(found) = readers.get(&next) {
            stack.extend(found.iter().map(|node| (*node).clone()));
        }
    }
    moves
}

/// Whether decision `a` acts after decision `b`: it reads a place `b` changes, or `b`'s
/// legs, and `b` doesn't depend on it (refcore's `m_strata`). E §3.1 step 8.
fn acts_after(m: &Reality, a: &str, b: &str) -> bool {
    let (Some(reader), Some(changer)) = (m.moves.get(a), m.moves.get(b)) else {
        return false;
    };
    let reads_legs = reader.read_targets.contains(&Target::Move(b.to_owned()));
    let reads_place = reader
        .read_targets
        .iter()
        .any(|read| !matches!(read, Target::Move(_)) && meets(m, read, &changer.changes));
    let depends = m
        .graph
        .reaches(&Node::new(NodeKind::Move, b), &Node::new(NodeKind::Move, a));
    (reads_legs || reads_place) && !depends
}

/// Whether a read meets one of a move's changes: the same place, a keyed place and one of
/// its instances, a kind and a place of it, or a place load can't see (refcore's `_meets`).
/// L §5.14.2.
fn meets(m: &Reality, read: &Target, changes: &[Target]) -> bool {
    changes.iter().any(|change| meets_one(m, read, change))
}

/// Whether a read meets one change.
fn meets_one(m: &Reality, read: &Target, change: &Target) -> bool {
    if change == read || *change == Target::Unknown || *read == Target::Unknown {
        return true;
    }
    if let (Target::PlaceSec(section), Target::Thing(thing))
    | (Target::Thing(thing), Target::PlaceSec(section)) = (change, read)
    {
        return section_holds(m, section, thing);
    }
    if let Target::Kind(kind) = change {
        let sub = m.subtree(kind);
        if let Target::Thing(thing) = read {
            let base = m.things.get(thing).and_then(|t| t.base.as_ref());
            return base.is_some_and(|base| sub.contains(base));
        }
        if let Target::PlaceSec(section) = read {
            return m
                .places
                .get(section)
                .is_some_and(|place| sub.contains(&place.kind));
        }
    }
    false
}

/// Whether `thing` is an instance of the keyed place `section`.
fn section_holds(m: &Reality, section: &str, thing: &str) -> bool {
    m.things
        .get(thing)
        .and_then(|t| t.place.as_ref())
        .is_some_and(|keys| keys.place == section)
}

/// The cycles of the decision graph, each recorded where the walk orders it (a day load
/// can't see), or the first on days load knows, which no written order breaks (refcore's
/// `m_strata`, its `visit`). L §5.14.2.
fn order_cycles(
    graph: &Graph,
    after: &BTreeMap<String, BTreeSet<String>>,
    decisions: &[String],
) -> Result<Vec<BTreeSet<String>>, Vec<String>> {
    let mut state: BTreeMap<String, Visit> = BTreeMap::new();
    let mut groups = Vec::new();
    for start in decisions {
        if state.contains_key(start) {
            continue;
        }
        state.insert(start.clone(), Visit::Open);
        let mut stack = vec![(start.clone(), pending_ids(after.get(start)))];
        while let Some((node, reads)) = stack.last_mut() {
            let Some(next) = reads.pop() else {
                state.insert(node.clone(), Visit::Done);
                stack.pop();
                continue;
            };
            match state.get(&next) {
                Some(Visit::Open) => groups.push(open_cycle(graph, &stack, &next)?),
                Some(Visit::Done) => {}
                None => {
                    state.insert(next.clone(), Visit::Open);
                    let next_reads = pending_ids(after.get(&next));
                    stack.push((next, next_reads));
                }
            }
        }
    }
    Ok(groups)
}

/// A decision's decisions still to visit, the first last.
fn pending_ids(after: Option<&BTreeSet<String>>) -> Vec<String> {
    after.map_or_else(Vec::new, |set| set.iter().rev().cloned().collect())
}

/// The cycle from `again` on the path: a group the walk orders, or, where every member's
/// day reads nothing (load knows the days), the cycle refused.
fn open_cycle(
    graph: &Graph,
    stack: &[(String, Vec<String>)],
    again: &str,
) -> Result<BTreeSet<String>, Vec<String>> {
    let path = stack.iter().map(|(id, _)| id);
    let cycle: Vec<String> = path.skip_while(|id| *id != again).cloned().collect();
    let known = cycle.iter().all(|id| {
        graph
            .reads(&Node::new(NodeKind::MoveOn, id))
            .is_none_or(BTreeMap::is_empty)
    });
    if known {
        return Err(cycle);
    }
    Ok(cycle.into_iter().collect())
}

/// The cycles that share a decision, merged into one component (refcore's `merged`).
fn merge(groups: Vec<BTreeSet<String>>) -> Vec<BTreeSet<String>> {
    let mut merged: Vec<BTreeSet<String>> = Vec::new();
    for group in groups {
        let mut joined = group;
        let (meeting, apart): (Vec<_>, Vec<_>) = merged
            .into_iter()
            .partition(|held| !held.is_disjoint(&joined));
        for held in meeting {
            joined.extend(held);
        }
        merged = apart;
        merged.push(joined);
    }
    merged
}

/// A component as one stratum: its members act after what any of them acts after, outside
/// it (refcore's `m_strata`). E §3.1 step 8.
fn join(group: &BTreeSet<String>, after: &mut BTreeMap<String, BTreeSet<String>>) {
    let mut outside = BTreeSet::new();
    for member in group {
        if let Some(set) = after.get(member) {
            outside.extend(set.iter().filter(|id| !group.contains(*id)).cloned());
        }
    }
    for member in group {
        after.insert(member.clone(), outside.clone());
    }
}

/// Each decision's layer: one more than the highest it acts after, 1 for none; a decision
/// met again on the path counts 1 (refcore's `lev`), in the order of the moves.
fn levels(
    m: &Reality,
    after: &BTreeMap<String, BTreeSet<String>>,
) -> Result<BTreeMap<String, u32>, Stop> {
    let mut level: BTreeMap<String, u32> = BTreeMap::new();
    for (id, mv) in m.moves.entries() {
        if mv.decision && !level.contains_key(id) {
            level_of(id, after, &mut level)?;
        }
    }
    Ok(level)
}

/// `start`'s layer, with those of the decisions under it, into `level`.
fn level_of(
    start: &str,
    after: &BTreeMap<String, BTreeSet<String>>,
    level: &mut BTreeMap<String, u32>,
) -> Result<(), Stop> {
    let mut path: BTreeSet<String> = BTreeSet::from([start.to_owned()]);
    let mut stack = vec![(start.to_owned(), pending_ids(after.get(start)), 0_u32)];
    while let Some((node, below, highest)) = stack.last_mut() {
        let Some(next) = below.pop() else {
            let own = highest
                .checked_add(1)
                .ok_or_else(|| broken("a stratum past the count"))?;
            level.insert(node.clone(), own);
            path.remove(node.as_str());
            stack.pop();
            if let Some((_, _, parent)) = stack.last_mut() {
                *parent = (*parent).max(own);
            }
            continue;
        };
        if let Some(known) = level.get(&next) {
            *highest = (*highest).max(*known);
        } else if path.contains(&next) {
            *highest = (*highest).max(1);
        } else {
            path.insert(next.clone());
            let next_below = pending_ids(after.get(&next));
            stack.push((next, next_below, 0));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{first_cycle, merge};
    use crate::model::{Node, NodeKind};
    use std::collections::{BTreeMap, BTreeSet};

    fn def(id: &str) -> Node {
        Node::new(NodeKind::Def, id)
    }

    fn ids(words: &[&str]) -> BTreeSet<String> {
        words.iter().map(|w| (*w).to_owned()).collect()
    }

    #[test]
    fn the_first_cycle_is_closed_by_the_node_met_again() {
        let mut graph: BTreeMap<Node, BTreeSet<Node>> = BTreeMap::new();
        graph.insert(def("a"), BTreeSet::from([def("b")]));
        graph.insert(def("b"), BTreeSet::from([def("c")]));
        graph.insert(def("c"), BTreeSet::from([def("b")]));
        let cycle: Vec<String> = first_cycle(&graph)
            .expect("b and c read each other")
            .iter()
            .map(|n| n.id.clone())
            .collect();
        assert_eq!(cycle, ["b", "c", "b"], "a is on the path, not in the cycle");
        graph.remove(&def("c"));
        assert!(first_cycle(&graph).is_none(), "no cycle left");
    }

    #[test]
    fn cycles_sharing_a_decision_merge_in_place() {
        let merged = merge(vec![ids(&["a", "b"]), ids(&["x", "y"]), ids(&["b", "c"])]);
        assert_eq!(
            merged,
            [ids(&["x", "y"]), ids(&["a", "b", "c"])],
            "the joined group goes last, the others keep their order"
        );
    }
}
