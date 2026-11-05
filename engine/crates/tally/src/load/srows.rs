//! A scenario's expression rows, typed and searched for what they read, and the asks a scenario's reality checks.
//! Spec: L §6.4 (a scenario row may be an expression over the base's facts, the base's items
//! named with the base's id as a prefix and the scenario's own bare; it reads inputs, bands
//! and defs, never a place or a movement, directly or through a def; an ask is evaluated in
//! the worlds its line names), §2.3 Calls (`household.b(x)` in a scenario's expression row),
//! §8.3 (`LOAD-SCENARIO`, `LOAD-NAME`, `LOAD-UNIT`); E §3.1 step 5 (the checks across items:
//! a scenario's expression rows, what a scenario's items reach).
//! Twin: engine/tools/refcore/check2.py (`m_check_scen_rows`, `m_check_srow_reads`);
//! check.py (`Checker.top_scenario`, `ask_worlds`, `t_name`'s base-world arm, the asks' filter
//! in `Checker.run`); check.py `prefixed_call`'s base arm.
//! Invariants: a row's reads hang off its own `srow` node, which the row's input or def reads,
//! so the graph's searches see them as refcore's do; the rows are typed in refcore's order
//! (inputs, then defs' givens, each in the order built); a row's reads are searched in the
//! order each reader first made them, from the last def reached, as refcore's stack does; every
//! search is a loop over an explicit stack (R-16).

use super::check::{Checker, Files, Scope, Stands, invariant};
use super::cone::all_children;
use super::types::{name_word, refused};
use crate::base::{Code, Load, Place as At, Refusal};
use crate::model::{Graph, NameKind, Node, NodeKind, Reality, Rel, Row, Ty};
use crate::syntax::{
    Ask, AskBody, Bind, Def, Domain, Expr, LawDate, Lit, PinForm, RuleValue, Stop, Value,
};
use std::collections::{BTreeMap, BTreeSet};

/// The world a scenario row's prefixed names are read in (refcore's `_world` mark): the
/// base's id, and the scenarios the base stacks on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BaseWorld {
    /// The base's id, the prefix: the model's, or the scenario the row's is over.
    pub(super) id: String,
    /// The scenarios the base is, or stacks on.
    pub(super) chain: BTreeSet<String>,
}

/// A scenario's expression row as the checker typed it (refcore's `srow` item and its
/// `srow_loc`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SrowAt {
    /// Its node, which the row's input or def reads.
    pub(super) node: Node,
    /// The input or def it is a row of.
    pub(super) target: String,
    /// The scenario whose version states it.
    pub(super) scenario: String,
    /// The file and line it is written on.
    pub(super) at: (String, u32),
}

/// The scenario whose reality this is, the top of its stack: the first by id that no other
/// scenario is over; none for a base (refcore's `top_scenario`). L §6.4.
pub(super) fn top_scenario(m: &Reality) -> Option<&str> {
    let below: BTreeSet<&str> = m
        .scenarios
        .over
        .values()
        .flatten()
        .map(String::as_str)
        .collect();
    m.scenarios
        .over
        .keys()
        .map(String::as_str)
        .find(|id| !below.contains(id))
}

/// The worlds an ask's line is evaluated in: its file's scenario, its `in`, a diff's two
/// worlds, its `{scenario s}` pins, and `as if {scenario s}` in its line or in a def it reaches,
/// the defs its line names and then the graph's edges (refcore's `ask_worlds`). L §6.4, §5.18.
pub(super) fn ask_worlds(m: &Reality, graph: &Graph, ask: &Ask) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    if let Some(own) = m.scenarios.of(&ask.id) {
        out.insert(own.to_owned());
    }
    if let Some(world) = &ask.world {
        out.insert(world.clone());
    }
    if let AskBody::Diff(diff) = &ask.body {
        out.insert(diff.world.clone());
        out.insert(diff.against.clone());
    }
    for pin in &ask.pins {
        if let PinForm::Scenario(id) = &pin.form {
            out.insert(id.clone());
        }
    }
    let named = scan(&ask_roots(ask), &mut out);
    // Before the ask is checked its own reads aren't in the graph: the defs its line names,
    // then theirs.
    let mut stack = vec![Node::new(NodeKind::Ask, &ask.id)];
    stack.extend(
        named
            .iter()
            .filter(|id| m.defs.get(id).is_some())
            .map(|id| Node::new(NodeKind::Def, id)),
    );
    let mut seen = BTreeSet::new();
    while let Some(x) = stack.pop() {
        if !seen.insert(x.clone()) {
            continue;
        }
        if x.kind == NodeKind::Def
            && let Some(def) = m.defs.get(&x.id)
        {
            scan(&def_roots(&def.decl), &mut out);
        }
        if let Some(reads) = graph.reads(&x) {
            stack.extend(reads.keys().cloned());
        }
    }
    out
}

/// The `{scenario s}` pins of every `as if` under `roots`, into `out`; the names met, in the
/// order met (refcore's `scan`, and the walk for the defs an ask's line names).
fn scan(roots: &[&Expr], out: &mut BTreeSet<String>) -> Vec<String> {
    let mut named = Vec::new();
    let mut stack: Vec<&Expr> = roots.iter().rev().copied().collect();
    while let Some(x) = stack.pop() {
        if let Expr::AsIf(_, pins) = x {
            for pin in pins {
                if let PinForm::Scenario(id) = &pin.form {
                    out.insert(id.clone());
                }
                stack.extend(pin_exprs(&pin.form));
            }
        }
        if let Expr::Name(id) | Expr::Lit(Lit::Name(id)) = x {
            named.push(id.clone());
        }
        if let Expr::Agg(agg) = x {
            for bind in &agg.binds {
                stack.extend(bind_exprs(bind));
            }
            stack.extend(agg.ties.iter().flatten());
        }
        stack.extend(all_children(x));
    }
    named
}

/// The expressions a pin holds.
fn pin_exprs(form: &PinForm) -> Vec<&Expr> {
    match form {
        PinForm::Law(e) | PinForm::Known(e) | PinForm::Wording(e, _) => vec![e],
        PinForm::Set { value, .. } => vec![value],
        PinForm::Scenario(_) => Vec::new(),
    }
}

/// The expressions a binding holds: its range's ends and step, its `on`, its `where`.
fn bind_exprs(bind: &Bind) -> Vec<&Expr> {
    let mut out = Vec::new();
    if let Domain::Range(range) = &bind.domain {
        out.extend([&range.0, &range.1]);
        out.extend(range.2.as_ref());
    }
    out.extend(bind.on.as_ref());
    out.extend(bind.filter.as_ref());
    out
}

/// The expressions an ask's line holds (refcore's walk of the ask's node).
fn ask_roots(ask: &Ask) -> Vec<&Expr> {
    let mut out: Vec<&Expr> = Vec::new();
    match &ask.body {
        AskBody::Exprs(exprs) => out.extend(exprs.iter()),
        AskBody::Residuals(residuals) => out.push(&residuals.place),
        AskBody::Explain(e) | AskBody::InputsOf(e) | AskBody::ReadersOf(e) => out.push(e),
        AskBody::Check(a, b) => out.extend([a, b]),
        AskBody::Solve(solve) => out.extend([&solve.left, &solve.right, &solve.target]),
        AskBody::Diff(diff) => {
            out.push(&diff.expr);
            for bind in &diff.binds {
                out.extend(bind_exprs(bind));
            }
        }
    }
    for bind in &ask.binds {
        out.extend(bind_exprs(bind));
    }
    for pin in &ask.pins {
        out.extend(pin_exprs(&pin.form));
    }
    out.extend(ask.law.as_ref());
    out.extend(ask.known.as_ref());
    out
}

/// The expressions a def's section holds: its binds, its `at`, its rules' values and `when`s
/// (refcore's walk of the def's `decl`).
fn def_roots(def: &Def) -> Vec<&Expr> {
    let mut out = Vec::new();
    for bind in &def.binds {
        out.extend(bind_exprs(bind));
    }
    if let Some(LawDate::Expr(at)) = &def.at {
        out.push(at);
    }
    for rule in &def.rules {
        match &rule.value {
            RuleValue::Expr(e) => out.push(e),
            RuleValue::Solve(solve) => out.extend([&solve.left, &solve.right, &solve.target]),
            RuleValue::Hole(_) => {}
        }
        out.extend(rule.when.as_ref());
    }
    out
}

/// What a scenario row reads that it may not: a place, the balances, a move, or an observed
/// input's movements (refcore's `m_check_srow_reads`, its `what`).
fn barred(m: &Reality, target: &Node) -> Option<String> {
    match target.kind {
        NodeKind::Place => Some(format!("the place {}", target.id)),
        NodeKind::Balances => Some("a balance".to_owned()),
        NodeKind::Move | NodeKind::MoveOn => Some(format!("the move {}", target.id)),
        NodeKind::Input => m
            .inputs
            .get(&target.id)
            .is_some_and(|input| input.observed)
            .then(|| format!("the movements of {}", target.id)),
        NodeKind::Def
        | NodeKind::Band
        | NodeKind::Ask
        | NodeKind::Price
        | NodeKind::Parts
        | NodeKind::Axis
        | NodeKind::Fill
        | NodeKind::Srow => None,
    }
}

/// What `reader` reads, in the order it first read each (refcore's edges, a dict in
/// insertion order); a read the order doesn't hold follows in the graph's order.
fn reads_in_order(graph: &Graph, order: &BTreeMap<Node, Vec<Node>>, reader: &Node) -> Vec<Node> {
    let Some(reads) = graph.reads(reader) else {
        return Vec::new();
    };
    let mut out: Vec<Node> = Vec::new();
    if let Some(first) = order.get(reader) {
        for target in first.iter().filter(|target| reads.contains_key(target)) {
            out.push(target.clone());
        }
    }
    for target in reads.keys() {
        if !out.contains(target) {
            out.push(target.clone());
        }
    }
    out
}

/// `LOAD-SCENARIO`: a scenario's expression row reads a place, a balance, a move or an
/// observed input's movements, directly or through defs, fills and bands, at the row's line
/// (refcore's `m_check_srow_reads`). L §6.4, §8.3.
///
/// # Errors
///
/// The first such read's refusal, the defs it is read through named in the order reached.
pub(super) fn check_srow_reads(
    m: &Reality,
    srows: &[SrowAt],
    order: &BTreeMap<Node, Vec<Node>>,
) -> Result<(), Stop> {
    for srow in srows {
        let mut seen = BTreeSet::new();
        let mut stack: Vec<(Node, Vec<String>)> = vec![(srow.node.clone(), Vec::new())];
        while let Some((reader, path)) = stack.pop() {
            for target in reads_in_order(&m.graph, order, &reader) {
                if let Some(what) = barred(m, &target) {
                    return Err(read_barred(srow, &what, &path));
                }
                let through =
                    matches!(target.kind, NodeKind::Def | NodeKind::Fill | NodeKind::Band);
                if through && seen.insert(target.clone()) {
                    let mut next = path.clone();
                    next.push(target.id.clone());
                    stack.push((target, next));
                }
            }
        }
    }
    Ok(())
}

/// The refusal of a scenario row that reads `what`, through the defs of `path`.
fn read_barred(srow: &SrowAt, what: &str, path: &[String]) -> Stop {
    let via = if path.is_empty() {
        String::new()
    } else {
        format!(" through {}", path.join(", "))
    };
    let message = format!(
        "the expression row of {} in {} reads {what}{via}: it reads inputs, bands and defs, never a place or a movement",
        srow.target, srow.scenario
    );
    let (file, line) = srow.at.clone();
    Stop::Refused(Refusal::at(
        Code::Load(Load::Scenario),
        message,
        At::Line(file, line),
    ))
}

/// The world a row of `scenario` reads its prefixed names in: the scenario it is over, or the
/// model, with the scenarios below (refcore's `m_check_scen_rows`, its `base_w`, `base_id`,
/// `chain`).
fn base_world(m: &Reality, scenario: &str) -> BaseWorld {
    let over = m.scenarios.over.get(scenario).cloned().flatten();
    let id = over.clone().unwrap_or_else(|| m.header.id.clone());
    let mut chain = BTreeSet::new();
    let mut next = over;
    while let Some(world) = next {
        if !chain.insert(world.clone()) {
            break;
        }
        next = m.scenarios.over.get(&world).cloned().flatten();
    }
    BaseWorld { id, chain }
}

/// Where a section a scenario's file declares counts for a base read: the sorts refcore's
/// `item_scen` looks up (defs, inputs, moves, bands, places, kinds, indexes, fills, asks).
fn scenario_item(kind: NameKind) -> bool {
    match kind {
        NameKind::Def
        | NameKind::Input
        | NameKind::Move
        | NameKind::Band
        | NameKind::Place
        | NameKind::Kind
        | NameKind::Index
        | NameKind::Fill
        | NameKind::Ask => true,
        NameKind::Unit | NameKind::Axis | NameKind::Version | NameKind::Source | NameKind::Cut => {
            false
        }
    }
}

/// A name a scenario row reads with the base's id as its prefix: an item the base declares,
/// an input, a band or a def (refcore's `t_name`, its `_world` arm). L §6.4.
///
/// # Errors
///
/// `LOAD-NAME unknown id <base>.<x>`; `LOAD-SCENARIO` for an item of a scenario the base
/// doesn't stack on, or one that isn't an input, a band or a def.
pub(super) fn base_name(m: &Reality, world: &BaseWorld, name: &str) -> Result<(), Stop> {
    let base = &world.id;
    let Some(kind) = m.names.get(name).copied() else {
        return Err(refused(Load::Name, format!("unknown id {base}.{name}")));
    };
    if scenario_item(kind)
        && let Some(scenario) = m.scenarios.of(name)
        && !world.chain.contains(scenario)
    {
        return Err(refused(
            Load::Scenario,
            format!("{base}.{name}: {name} is declared in scenario {scenario}, not in the base"),
        ));
    }
    if !matches!(kind, NameKind::Input | NameKind::Def | NameKind::Band) {
        return Err(refused(
            Load::Scenario,
            format!(
                "{base}.{name}: an expression row reads inputs, bands and defs, never a {}",
                name_word(kind)
            ),
        ));
    }
    Ok(())
}

impl Checker {
    /// Each scenario's expression row, a row of an input or a given of a def: typed as the
    /// target's value where its prefixed names are the base's, its reads hung off its own node,
    /// which the target reads (refcore's `m_check_scen_rows`). L §6.4.
    ///
    /// # Errors
    ///
    /// The row's typing refusal at its line (`LOAD-UNIT a row of <x> …`, `LOAD-NAME`,
    /// `LOAD-SCENARIO`).
    pub(super) fn check_scen_rows(&mut self, m: &Reality, files: &Files) -> Result<(), Stop> {
        for (id, input) in m.inputs.entries() {
            if input.observed {
                continue;
            }
            let ty = input.ty.clone().unwrap_or(Ty::Unseen);
            for row in &input.rows {
                self.scen_row(m, files, (NodeKind::Input, id), &ty, row)?;
            }
        }
        for (id, def) in m.defs.entries() {
            for row in &def.givens {
                self.scen_row(m, files, (NodeKind::Def, id), &def.ty, row)?;
            }
        }
        Ok(())
    }

    /// One row of `target`, of type `ty`, if it is an expression row (refcore's
    /// `m_check_scen_rows`, its loop's body).
    fn scen_row(
        &mut self,
        m: &Reality,
        files: &Files,
        (kind, id): (NodeKind, &str),
        ty: &Ty,
        row: &Row,
    ) -> Result<(), Stop> {
        let Value::Expr(e) = &row.value else {
            return Ok(());
        };
        let Some(scenario) = m.scenarios.of(&row.version) else {
            return Err(invariant(format!(
                "the expression row {} stands outside a scenario",
                row.sid
            )));
        };
        let node = Node::new(NodeKind::Srow, &row.sid);
        let file = files.of_version(&row.version)?;
        self.srows.push(SrowAt {
            node: node.clone(),
            target: id.to_owned(),
            scenario: scenario.to_owned(),
            at: (file.to_owned(), row.line),
        });
        let mut scope = Scope::new(node.clone(), Stands::Decl, file, row.line);
        scope.allow_lit = true;
        self.edge(&Node::new(kind, id), node, Rel::Any);
        self.base_world = Some(base_world(m, scenario));
        let typed = self.typed_as(m, &scope, e, ty, &format!("a row of {id}"), 0);
        self.base_world = None;
        typed.map(|_| ())
    }
}
