//! The asks that read provenance: `explain`'s derivation, `inputs-of`'s rows with their sources, `readers-of`'s readers.
//! Spec: L §5.18 (`explain ref`, `inputs-of ref`, `readers-of x`; Explanation order: the
//! def's rules with their states, the winner's causes depth first, each cause printed once and
//! referenced by its line after, a row with its version, source, locator, governs span or
//! instant and known date), §3.7 (a row's provenance); E §10 (`explain` re-evaluates the
//! cell's cone with tracing on; `inputs-of` is the transitive read set with each row's source,
//! a walk read standing for the rows behind what it counts; `readers-of` is the inverse of
//! the item graph, the direct readers first), §3.2 (the read set's rows).
//! Twin: engine/tools/refcore/ask.py (`evaluate`'s explain and inputs-of arms, `print_line`,
//! `row_line`, `readers_of`, `explain_lines`, `explain_node`); check.py
//! (`Checker.readers_of`).
//! Invariants: an explanation prints from its trace in the order refcore prints it, each node
//! numbered where it first prints and a cause met again referenced by its number, a row by
//! its stable id, a balance by its read; nothing recurses (a worklist, R-16); a balance, a
//! projection and a `low` are `walked.rs`'s, a balance among the causes prints in their
//! balance form (P63); a movement instance is `instance.rs`'s, by its move's name alone or
//! at keys; a walk-bound def stops `NotBuilt`; rows print in
//! the text order of their stable ids; readers by depth, each depth by id, an event kind's
//! bindings first (P63).

use std::collections::{BTreeMap, BTreeSet};

use super::answer::unresolved_word;
use super::check::Printed;
use super::instance::instance_line;
use super::walked::{Read, balance_line};
use crate::base::{Code, Refusal, Run};
use crate::eval::{Collected, Ctx, Engine, Env, Item, Trace, TraceKind, TracedBalance, num_stop};
use crate::model::{NameKind, Node, NodeKind, Reality, Value};
use crate::syntax::{Ask, Expr, PinForm, Postfix, Stop};

/// The line of `explain reference` of ask `ask` in scope `env` and context `ctx`: the value,
/// then its derivation, traced while the reference is evaluated again; under the ask's `with`
/// pins, the pins with their values, the derivation inside them and the one outside them
/// (refcore's `evaluate`, its explain arm, `explain_pins` and `print_line`). L §5.18, E §10.
///
/// # Errors
///
/// A load refusal met at run; `NotBuilt` for the walk's forms, G12's.
pub(super) fn explain_line(
    m: &Reality,
    (ask, reference): (&Ask, &Expr),
    engine: &mut Engine,
    env: &Env,
    ctx: Ctx,
) -> Result<Printed, Stop> {
    if let Expr::Bal(b) = reference {
        return balance_line(m, (ask, Read::Bal(b), reference), engine, env, ctx);
    }
    if let Expr::Low(l) = reference {
        return balance_line(m, (ask, Read::Low(l), reference), engine, env, ctx);
    }
    match walk_form(m, reference) {
        Form::Instance => return instance_line(m, (ask, reference), engine, env, ctx),
        Form::WalkBound => return Err(Stop::NotBuilt("explain of a walk-bound def")),
        Form::Traced => {}
    }
    let (item, met, trace) = traced(m, engine, reference, env, ctx)?;
    // a bare key literal no position resolved is refused where it is read (P39)
    if let Some(place) = m.asks_unresolved.get(&ask.id) {
        unresolved_word(core::slice::from_ref(&item), place)?;
    }
    let text = plain_text(m, &item)?;
    let trace = match engine.unpinned {
        Some(unpinned) if !ask.pins.is_empty() => outside_pins(
            m,
            (ask, reference),
            engine,
            env,
            (unpinned, ctx),
            (item, trace),
        )?,
        _ => trace,
    };
    Ok(Printed {
        text,
        body: derivation(m, &trace)?,
        bare: false,
        met,
        laws: None,
    })
}

/// `reference` read in scope `env` and context `ctx` as an ask's line reads it, with tracing
/// on: its value, the register entries it met, and its trace (refcore's `evaluate`, its
/// explain arm). E §10.
fn traced(
    m: &Reality,
    engine: &mut Engine,
    reference: &Expr,
    env: &Env,
    ctx: Ctx,
) -> Result<(Item, Collected, Trace), Stop> {
    engine.begin_trace();
    let read = engine.line(m, core::slice::from_ref(reference), env, ctx);
    let trace = engine.end_trace()?;
    if trace.walked {
        // a balance, a projection or a movement among the causes prints in the walk's own
        // form (E §10), G12's
        return Err(Stop::NotBuilt(
            "explain of a cause the walk computes: a balance, a projection, a movement",
        ));
    }
    let (items, met) = read?;
    let Ok([item]) = <[Item; 1]>::try_from(items) else {
        return Err(invariant("an explained line with other than one value"));
    };
    Ok((item, met, trace))
}

/// The trace of `explain reference` under ask `ask`'s `with` pins, `inside` its value and
/// trace in the line's context: the pins with their values, then `inside`, then the
/// reference traced again in `unpinned`, the line's context outside the pins with the axis
/// options the pins don't fix, which brings nothing to the line's register (refcore's
/// `explain_pins`). E §10.
fn outside_pins(
    m: &Reality,
    (ask, reference): (&Ask, &Expr),
    engine: &mut Engine,
    env: &Env,
    (unpinned, ctx): (Ctx, Ctx),
    (inside, trace): (Item, Trace),
) -> Result<Trace, Stop> {
    let fixed: BTreeSet<String> = ask
        .pins
        .iter()
        .filter_map(|pin| {
            if let PinForm::Set { target, .. } = &pin.form
                && m.names.kind(&target.id) == Some(NameKind::Axis)
            {
                Some(target.id.clone())
            } else {
                None
            }
        })
        .collect();
    let outside = engine.outside_pins((unpinned, ctx), &fixed)?;
    let pins = engine.pin_lines(m, outside, ctx)?;
    engine.open_layer(m, outside)?;
    let read = traced(m, engine, reference, env, outside);
    engine.end_layer(outside.layer, read.as_ref().err())?;
    let (value, _, outside_trace) = read?;
    trace.beside_pins(outside_trace, pins, (inside, value))
}

/// The form `explain` gives a reference by what it names (refcore's `evaluate`, its explain
/// arms). E §10.
enum Form {
    /// A movement instance: a move's name, at its keys.
    Instance,
    /// A walk-bound def's cell: its name, at its keys.
    WalkBound,
    /// Anything else: its derivation, traced.
    Traced,
}

/// The form `explain` gives `reference`: a move's or a walk-bound def's name, alone or at
/// keys, is the walk's; anything else is traced (refcore's `evaluate`, its explain arms).
/// E §10.
fn walk_form(m: &Reality, reference: &Expr) -> Form {
    // a name, or a name at keys
    let named = if let Expr::Post(base, post) = reference
        && matches!(post.as_slice(), [Postfix::Keys(_)])
    {
        base.as_ref()
    } else {
        reference
    };
    let Expr::Name(id) = named else {
        return Form::Traced;
    };
    match m.names.kind(id) {
        Some(NameKind::Move) => Form::Instance,
        Some(NameKind::Def) if m.graph.walk_bound.contains(id) => Form::WalkBound,
        _ => Form::Traced,
    }
}

/// The line of `inputs-of reference`: its head alone, then every row its evaluation read,
/// with its source, by stable id (refcore's `evaluate`, its inputs-of arm, `row_line`).
/// L §5.18, E §10, §3.2.
///
/// # Errors
///
/// A load refusal met at run; `NotBuilt` for a node not built yet.
pub(super) fn inputs_line(
    m: &Reality,
    (ask, reference): (&Ask, &Expr),
    engine: &mut Engine,
    env: &Env,
    ctx: Ctx,
) -> Result<Printed, Stop> {
    let (items, met) = engine.line(m, core::slice::from_ref(reference), env, ctx)?;
    // a bare key literal no position resolved is refused where it is read (P39)
    if let Some(place) = m.asks_unresolved.get(&ask.id) {
        unresolved_word(&items, place)?;
    }
    Ok(Printed {
        text: String::new(),
        body: row_lines(m, &met)?,
        bare: true,
        met,
        laws: None,
    })
}

/// The rows `met` read, one line each in their stable ids' order: `<sid> (source <s>
/// "<title>"[, locator "<l>"])` (refcore's `row_line`). L §5.18, §3.7.
fn row_lines(m: &Reality, met: &Collected) -> Result<Vec<String>, Stop> {
    let mut out = Vec::with_capacity(met.rows.len());
    for (sid, seen) in &met.rows {
        let cite = &m
            .versions
            .get(&seen.version)
            .ok_or_else(|| invariant("a row of a version load didn't build"))?
            .cite;
        let title = m
            .sources
            .get(cite)
            .map_or_else(String::new, |s| format!(" \"{}\"", s.title));
        let located = seen
            .locator
            .as_ref()
            .map_or_else(String::new, |l| format!(", locator \"{l}\""));
        out.push(format!("  {sid} (source {cite}{title}{located})"));
    }
    Ok(out)
}

/// The line of `readers-of reference`: every item with a path of edges to it in the item
/// graph, the direct readers first, then by depth, each depth by id; of an event kind, the
/// items holding a binding over it or over a kind it is a subkind of first, then their
/// readers so (a move of the kind is no reader: its kind is its type); `nothing reads x`
/// when none (refcore's `readers_of`, `Checker.readers_of`; ruling P63, #132). An index, a
/// kind of things, a unit, a version or a source is no node of the graph, so nothing reads
/// it. L §5.18, E §10, §3.1 step 5.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a reference that is no id.
pub(super) fn readers_line(m: &Reality, ask: &Ask, reference: &Expr) -> Result<String, Stop> {
    let Expr::Name(id) = reference else {
        return Err(invariant("readers-of of other than an id"));
    };
    let names = match node_of(m, id) {
        Of::Node(node) => readers(m, &[&node]),
        Of::Event => {
            let mut first: Vec<&Node> = m
                .graph
                .bound_kinds
                .iter()
                .filter(|(bound, _)| m.subtree(bound).contains(id.as_str()))
                .flat_map(|(_, items)| items.iter())
                .collect();
            first.sort_by(|a, b| by_id(a, b));
            first.dedup();
            let mut out: Vec<String> = first.iter().map(|n| n.id.clone()).collect();
            out.extend(readers(m, &first));
            out
        }
        Of::Nothing => Vec::new(),
    };
    if names.is_empty() {
        return Ok(format!("{}: nothing reads {id}", ask.id));
    }
    Ok(format!("{}: {}", ask.id, names.join(", ")))
}

/// What `readers-of` finds a name to be: a node of the graph, an event kind, or neither.
enum Of {
    /// A node of the item graph.
    Node(Node),
    /// An event kind: its readers are the items holding a binding over it.
    Event,
    /// No node: nothing reads it.
    Nothing,
}

/// What `id` names, for `readers-of`. E §10, §3.1 step 5.
fn node_of(m: &Reality, id: &str) -> Of {
    let kind = match m.names.kind(id) {
        Some(NameKind::Input) => NodeKind::Input,
        Some(NameKind::Def) => NodeKind::Def,
        Some(NameKind::Band) => NodeKind::Band,
        Some(NameKind::Place) => NodeKind::Place,
        Some(NameKind::Move) => NodeKind::Move,
        Some(NameKind::Axis) => NodeKind::Axis,
        Some(NameKind::Fill) => NodeKind::Fill,
        Some(NameKind::Ask) => NodeKind::Ask,
        Some(NameKind::Kind) if m.kinds.get(id).is_some_and(|k| k.certainty.is_some()) => {
            return Of::Event;
        }
        Some(
            NameKind::Kind
            | NameKind::Unit
            | NameKind::Index
            | NameKind::Version
            | NameKind::Source
            | NameKind::Cut,
        )
        | None => return Of::Nothing,
    };
    Of::Node(Node::new(kind, id))
}

/// Two nodes in the order a layer of readers prints: by id, then by their kinds' words.
fn by_id(a: &Node, b: &Node) -> core::cmp::Ordering {
    (a.id.as_str(), a.kind.word()).cmp(&(b.id.as_str(), b.kind.word()))
}

/// The ids of the items with a path of edges to one of `start`, by depth, each depth in the
/// order of its ids, then its kinds' words; `start` itself not among them. Each item prints
/// once, in the first depth that reaches one of its nodes: a move's day node is its move, and
/// a scenario's expression row its input or def (refcore's `Checker.readers_of`, which prints
/// nodes; ruling P64, #133). L §5.18 ("each once"), E §10, §3.1 step 5.
fn readers(m: &Reality, start: &[&Node]) -> Vec<String> {
    let mut inverse: BTreeMap<&Node, Vec<&Node>> = BTreeMap::new();
    for (reader, targets) in &m.graph.edges {
        for target in targets.keys() {
            match inverse.get_mut(target) {
                Some(readers) => readers.push(reader),
                None => {
                    inverse.insert(target, vec![reader]);
                }
            }
        }
    }
    let mut seen: BTreeSet<Node> = start.iter().map(|n| item_of(n, &inverse)).collect();
    let mut frontier: Vec<Node> = seen.iter().flat_map(|i| nodes_of(m, i)).collect();
    let mut out = Vec::new();
    while !frontier.is_empty() {
        let mut layer: Vec<Node> = Vec::new();
        for x in &frontier {
            for r in inverse.get(x).into_iter().flatten() {
                let item = item_of(r, &inverse);
                if !seen.contains(&item) {
                    seen.insert(item.clone());
                    layer.push(item);
                }
            }
        }
        layer.sort_by(by_id);
        out.extend(layer.iter().map(|n| n.id.clone()));
        frontier = layer.iter().flat_map(|i| nodes_of(m, i)).collect();
    }
    out
}

/// The item node `node` stands for: a move's day is its move, a scenario's expression row the
/// input or def that reads it, alone (`inverse`, the graph's readers), any other node itself
/// (ruling P64, #133). E §3.1 step 5, L §6.4.
fn item_of(node: &Node, inverse: &BTreeMap<&Node, Vec<&Node>>) -> Node {
    match node.kind {
        NodeKind::MoveOn => Node::new(NodeKind::Move, &node.id),
        NodeKind::Srow => inverse
            .get(node)
            .and_then(|readers| readers.first())
            .map_or_else(|| node.clone(), |n| (*n).clone()),
        NodeKind::Input
        | NodeKind::Def
        | NodeKind::Band
        | NodeKind::Move
        | NodeKind::Place
        | NodeKind::Ask
        | NodeKind::Price
        | NodeKind::Balances
        | NodeKind::Parts
        | NodeKind::Axis
        | NodeKind::Fill => node.clone(),
    }
}

/// The nodes of the graph that stand for `item`: itself, a move's day node, and the
/// expression rows an input or def reads. E §3.1 step 5, L §6.4.
fn nodes_of(m: &Reality, item: &Node) -> Vec<Node> {
    let mut nodes = vec![item.clone()];
    if item.kind == NodeKind::Move {
        nodes.push(Node::new(NodeKind::MoveOn, &item.id));
    }
    let rows = m
        .graph
        .edges
        .get(item)
        .into_iter()
        .flat_map(|targets| targets.keys());
    nodes.extend(rows.filter(|n| n.kind == NodeKind::Srow).cloned());
    nodes
}

/// One step of printing a derivation: a node at its depth, or a line made already.
enum Step {
    /// The node at this place, printed at this depth.
    Node(usize, usize),
    /// A line, printed as it is.
    Line(String),
}

/// The lines of a trace's derivation, as refcore prints them (`explain_lines`,
/// `explain_node`): each node numbered `[n]` where it first prints, its depth two spaces a
/// level; a def cell with its value and law date, its lines, each rule in its labels' order
/// with its state, under the rule the causes its `when` and, for the winner, its value read,
/// then what the cell read beside its rules; a row with its value and provenance; a balance
/// in the balance form, from the walk it read (P63); a node met again as `(<label>: see
/// [n])`, a row by its stable id, a balance by its read. L §5.18, E §10.
pub(super) fn derivation(m: &Reality, trace: &Trace) -> Result<Vec<String>, Stop> {
    let mut out = Vec::new();
    let mut numbered: BTreeMap<usize, usize> = BTreeMap::new();
    let mut rows: BTreeMap<String, usize> = BTreeMap::new();
    // the balances printed, by their read: a timeline has no order (R-47), so they're found
    // by equality
    let mut balances: Vec<(&TracedBalance, usize)> = Vec::new();
    let mut count: usize = 0;
    let top = trace.top()?;
    let mut todo: Vec<Step> = top.iter().rev().map(|c| Step::Node(*c, 1)).collect();
    while let Some(step) = todo.pop() {
        let (at, depth) = match step {
            Step::Line(line) => {
                out.push(line);
                continue;
            }
            Step::Node(at, depth) => (at, depth),
        };
        let node = trace.node(at)?;
        let ind = indent(depth);
        // a row is met again by its stable id; a balance only as the same read, its unit, cut
        // and timeline too, since its label has none of them (P63)
        let met = match &node.kind {
            TraceKind::Row | TraceKind::BandRow => rows.get(&node.label).copied(),
            TraceKind::Balance(read) => balances.iter().find(|(b, _)| *b == read).map(|(_, n)| *n),
            TraceKind::Def(_) | TraceKind::Root | TraceKind::Group | TraceKind::Side => {
                numbered.get(&at).copied()
            }
        };
        if let Some(no) = met {
            out.push(format!("{ind}({}: see [{no}])", node.label));
            continue;
        }
        count = count
            .checked_add(1)
            .ok_or_else(|| invariant("a count of a derivation's lines"))?;
        match &node.kind {
            TraceKind::Row | TraceKind::BandRow => {
                rows.insert(node.label.clone(), count);
            }
            TraceKind::Balance(read) => balances.push((read, count)),
            TraceKind::Def(_) | TraceKind::Root | TraceKind::Group | TraceKind::Side => {
                numbered.insert(at, count);
            }
        }
        let (head, below) = node_lines(m, trace, (at, depth), count)?;
        out.extend(head);
        todo.extend(below.into_iter().rev());
    }
    Ok(out)
}

/// The indent of a line at depth `depth`: two spaces a level.
fn indent(depth: usize) -> String {
    core::iter::repeat_n("  ", depth).collect()
}

/// A node's own lines, numbered `no` at its depth, and the steps that print under it in
/// order (refcore's `explain_node`); a balance's in the balance form, from the walk it read,
/// but for a refused one (ruling P63, #132). L §5.18, E §10.
fn node_lines(
    m: &Reality,
    trace: &Trace,
    (at, depth): (usize, usize),
    no: usize,
) -> Result<(Vec<String>, Vec<Step>), Stop> {
    let node = trace.node(at)?;
    let ind = indent(depth);
    let mut head = Vec::new();
    let mut below = Vec::new();
    match &node.kind {
        TraceKind::Row => {
            head.push(format!(
                "{ind}[{no}] row {} = {}",
                node.label,
                value_text(m, node.value.as_ref())?
            ));
            head.extend(node.lines.iter().map(|l| format!("{ind}    {l}")));
        }
        TraceKind::Balance(_) => {
            head.push(format!(
                "{ind}[{no}] {} = {}",
                node.label,
                value_text(m, node.value.as_ref())?
            ));
            // its balance form, rendered from the walk it read when it was read
            head.extend(node.lines.iter().map(|l| format!("{ind}    {l}")));
        }
        TraceKind::BandRow => head.push(format!("{ind}[{no}] band row {}", node.label)),
        TraceKind::Def(law) => {
            let law = law.map_or_else(String::new, |d| format!(", law date {}", d.text()));
            head.push(format!(
                "{ind}[{no}] {} = {}{law}",
                node.label,
                value_text(m, node.value.as_ref())?
            ));
            head.extend(node.lines.iter().map(|l| format!("{ind}    {l}")));
            let inner = depth
                .checked_add(3)
                .ok_or_else(|| invariant("a derivation's depth"))?;
            // a def cell's rules show once its candidates are known, each with its state
            let shown = node
                .rules
                .values()
                .filter(|r| node.shown && !r.state.is_empty());
            for rule in shown {
                below.push(Step::Line(format!(
                    "{ind}    rule {}: {}",
                    rule.label, rule.state
                )));
                let groups = [
                    rule.when,
                    rule.value.filter(|_| rule.state.starts_with("won")),
                ];
                for group in groups.into_iter().flatten() {
                    let children = &trace.node(group)?.children;
                    below.extend(children.iter().map(|c| Step::Node(*c, inner)));
                }
            }
            let beside = depth
                .checked_add(2)
                .ok_or_else(|| invariant("a derivation's depth"))?;
            below.extend(node.children.iter().map(|c| Step::Node(*c, beside)));
        }
        TraceKind::Root | TraceKind::Group | TraceKind::Side => {
            let value = match (&node.kind, &node.value) {
                (TraceKind::Side, Some(item)) => format!(": {}", side_text(m, item)?),
                _ => String::new(),
            };
            head.push(format!("{ind}[{no}] {}{value}", node.label));
            let inner = depth
                .checked_add(1)
                .ok_or_else(|| invariant("a derivation's depth"))?;
            below.extend(node.children.iter().map(|c| Step::Node(*c, inner)));
        }
    }
    Ok((head, below))
}

/// A traced value as `explain` prints it, by no static type (refcore's `fmt_value` with
/// none): a value, a hole with its questions, a refusal; `?` where none was kept.
fn value_text(m: &Reality, item: Option<&Item>) -> Result<String, Stop> {
    match item {
        Some(item) => plain_text(m, item),
        None => Ok("?".to_owned()),
    }
}

/// A line's value by no static type, as `explain` prints its head (refcore's `fmt_value`
/// with none): a key outside its index and a refusal as `refused: <code> <message>`.
/// L §5.18, §8.4.
pub(super) fn plain_text(m: &Reality, item: &Item) -> Result<String, Stop> {
    match item {
        Item::Refused(r) => Ok(format!("refused: {}", r.text())),
        Item::Value(Value::OutKey { index, text }) => Ok(format!(
            "refused: {} {text} is outside {index}",
            Code::Run(Run::Key).as_str()
        )),
        Item::Value(Value::Window(w)) if !w.of_days() => {
            let window = w.question_text(m).map_err(num_stop)?;
            let text = format!("date within {window} not precise enough for a print");
            Ok(crate::hole::Hole::asked(&text, w.figure()).text())
        }
        Item::Value(v) => v.text(m).map_err(num_stop),
    }
}

/// An `as if`'s value inside or outside its pins as `explain` prints it beside its label: a
/// refusal as `refused: <code> <message>`, a hole with its questions, any other value as
/// L §5.18 prints it (refcore's `_tfmt`). E §10.
fn side_text(m: &Reality, item: &Item) -> Result<String, Stop> {
    match item {
        Item::Refused(r) => Ok(format!("refused: {}", r.text())),
        Item::Value(v) => v.text(m).map_err(num_stop),
    }
}

/// An `ENGINE-INVARIANT` refusal: an explanation's own structure broke.
pub(super) fn invariant(what: &str) -> Stop {
    Stop::Refused(Refusal::new(Code::EngineInvariant, what.to_owned()))
}
