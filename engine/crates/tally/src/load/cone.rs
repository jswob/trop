//! Load's random cone: the items that reach a sample, and each query, ask line, path clause and orientation checked against it.
//! Spec: E §3.1 step 10; L §5.16 (lifting, closed forms, a query over paths stands only in
//! an ask that states paths), §5.15 (`orient`), §5.18 (`paths N seed S [path P]`), §8.3
//! item 10 (`LOAD-RANDOM`, `-ASKONLY`, `-ORIENT`, `-PARSE`, `-DOMAIN`).
//! Twin: engine/tools/refcore/check2.py (`m_random_cone`, `m_line_random`, `m_is_random`,
//! `m_affine`, `m_closed_form`, `m_finish_random`, `reads_balances`, `m_reaches_any`).
//! Invariants: the cone is a least fixed point over the graph's edges, read roles and marks;
//! every walk over an expression or the graph is an explicit stack; the affine form's
//! recursion is counted.

use super::check::{Files, Scope, Stands, invariant, place_target};
use super::random::{QueryAt, strip};
use super::types::{children, refused, text};
use crate::base::Load;
use crate::model::{Graph, Mark, NameKind, Node, NodeKind, ReadRole, Reality, Rel, Target, Ty};
use crate::syntax::{Ask, AskBody, Expr, Postfix, QueryFn, Stop};
use std::collections::BTreeSet;

/// How deep the affine form's recursion goes before it is an invariant (R-16).
const AFFINE_DEPTH: u32 = 64;

/// The random cone: its items, the places its random movements book into (`any` when one
/// is a place load can't see) and their kinds (refcore's `rand`, `rplaces`, `rkinds`).
#[derive(Debug, Default)]
struct Cone {
    /// The random items.
    items: BTreeSet<Node>,
    /// The places random movements book into, by their things.
    places: BTreeSet<String>,
    /// Whether one books into a place load can't see.
    any_place: bool,
    /// The kinds of the random movements.
    kinds: BTreeSet<String>,
}

impl Cone {
    /// Whether a running read of `target` reads a place a random movement books into
    /// (refcore's `place_hit`).
    fn place_hit(&self, target: &Target) -> bool {
        if self.places.is_empty() && !self.any_place {
            return false;
        }
        if let Target::Thing(thing) = target
            && !self.any_place
        {
            return self.places.contains(thing);
        }
        true
    }

    /// Whether a running read of `target` reaches the cone: a random move's movements, a
    /// kind holding a random movement's kind, a booked place (refcore's `hit`).
    fn hit(&self, m: &Reality, target: &Target) -> bool {
        match target {
            Target::Move(id) => {
                self.items.contains(&Node::new(NodeKind::Move, id))
                    || self.items.contains(&Node::new(NodeKind::MoveOn, id))
            }
            Target::Kind(kind) => m.subtree(kind).iter().any(|k| self.kinds.contains(k)),
            Target::Thing(_) | Target::PlaceSec(_) | Target::Unknown => self.place_hit(target),
        }
    }

    /// Adds `item`, booking a move's legs' places and its kind (refcore's `add`, `book`).
    fn add(&mut self, m: &Reality, item: &Node) {
        self.items.insert(item.clone());
        if !matches!(item.kind, NodeKind::Move | NodeKind::MoveOn) {
            return;
        }
        let Some(mv) = m.moves.get(&item.id) else {
            return;
        };
        self.kinds.insert(mv.kind.clone());
        let scope = Scope::new(item.clone(), Stands::Rule, "", 0);
        for leg in &mv.decl.legs {
            for end in [&leg.from, &leg.to] {
                if let Target::Thing(thing) = place_target(m, &scope, end) {
                    self.places.insert(thing);
                } else {
                    self.any_place = true;
                }
            }
        }
    }
}

/// Whether a node kind is an item the cone holds (refcore's `ITEMS`).
fn is_item(node: &Node) -> bool {
    matches!(
        node.kind,
        NodeKind::Input | NodeKind::Def | NodeKind::Fill | NodeKind::Move | NodeKind::MoveOn
    )
}

/// The items that reach a sample of a random fact, through a read role, an edge, or a
/// running read of a place a random movement books into; `with_proj` counts projections
/// (refcore's `random_cone`). E §3.1 step 10; L §5.16, §8.3 item 10.
fn random_cone(m: &Reality, graph: &Graph, projections: Projections) -> Cone {
    let mut cone = Cone::default();
    let mut changed = true;
    while changed {
        changed = false;
        for (reader, reads) in &graph.rand_edges {
            if cone.items.contains(reader) || !is_item(reader) {
                continue;
            }
            let mut reaches = false;
            for (target, roles) in reads {
                let sample = roles.contains(&ReadRole::Sample);
                let live = roles.iter().any(|r| *r != ReadRole::Query);
                reaches |= live && (sample || cone.items.contains(target));
            }
            if reaches {
                cone.add(m, reader);
                changed = true;
            }
        }
        for (reader, reads) in &graph.edges {
            if cone.items.contains(reader) || !is_item(reader) {
                continue;
            }
            let roles = graph.rand_edges.get(reader);
            let mut reaches = false;
            for (target, rels) in reads {
                let only_query = roles
                    .and_then(|r| r.get(target))
                    .is_some_and(|r| only_queried(r));
                let read = rels.iter().any(|r| *r != Rel::Pin);
                reaches |= cone.items.contains(target) && read && !only_query;
            }
            if reaches {
                cone.add(m, reader);
                changed = true;
            }
        }
        if cone.places.is_empty() && !cone.any_place && cone.kinds.is_empty() {
            continue;
        }
        for (item, marks) in &graph.marks {
            if cone.items.contains(item) || !is_item(item) {
                continue;
            }
            let hit = marks.iter().any(|(mark, target)| match mark {
                Mark::Proj => projections == Projections::Counted && cone.place_hit(target),
                Mark::Bal | Mark::BalNow | Mark::Flows | Mark::Legs => {
                    if let Target::Move(id) = target {
                        cone.items.contains(&Node::new(NodeKind::Move, id))
                    } else {
                        cone.hit(m, target)
                    }
                }
                Mark::LegOf => false,
            });
            if hit {
                cone.add(m, item);
                changed = true;
            }
        }
    }
    cone
}

/// Whether an item reads a target only inside queries.
fn only_queried(roles: &[ReadRole]) -> bool {
    roles.len() == 1 && roles.contains(&ReadRole::Query)
}

/// The input, def or fill a name reads, if it names one.
fn read_node(m: &Reality, name: &str) -> Option<Node> {
    let kind = m.names.get(name)?;
    let node_kind = match kind {
        NameKind::Input => NodeKind::Input,
        NameKind::Def => NodeKind::Def,
        NameKind::Fill => NodeKind::Fill,
        NameKind::Unit
        | NameKind::Kind
        | NameKind::Index
        | NameKind::Axis
        | NameKind::Band
        | NameKind::Place
        | NameKind::Move
        | NameKind::Cut
        | NameKind::Version
        | NameKind::Ask
        | NameKind::Source => return None,
    };
    Some(Node::new(node_kind, name))
}

/// Whether a name reads an input or a def declared `dist`: its sample, outside a
/// distribution's role.
fn is_dist_fact(m: &Reality, name: &str) -> bool {
    m.inputs
        .get(name)
        .is_some_and(|i| matches!(i.ty, Some(Ty::Dist(_))))
        || m.defs
            .get(name)
            .is_some_and(|d| matches!(d.ty, Ty::Dist(_)))
}

/// Whether a query or a path score takes its argument over the paths, so a walk for
/// randomness skips it: a query over a random value, or `crps` of a random value.
fn over_paths(e: &Expr) -> bool {
    if let Expr::Query(q) = e {
        return q.binds.is_none();
    }
    if let Expr::Crps(c) = e {
        return c.bind.is_none() && !matches!(strip(strip_to(&c.forecast)), Expr::Dist(_));
    }
    false
}

/// The distribution inside a `to u`, or the expression itself.
fn strip_to(e: &Expr) -> &Expr {
    if let Expr::To(inner, _) = strip(e) {
        return inner;
    }
    e
}

/// Whether an expression's value changes with the path: a sample read, a read of a random
/// item, outside the queries and path scores that take it over the paths (refcore's
/// `is_random`). L §5.16.
fn is_random(m: &Reality, items: &BTreeSet<Node>, e: &Expr) -> bool {
    let scope = Scope::new(Node::new(NodeKind::Ask, ""), Stands::Ask, "", 0);
    // The X of each affine form under `cdf`, `pdf` or `pmf`, read as its distribution, not
    // its sample; each node of the walk carries the form whose value holds it, and a node of
    // that value equal to its X is X, as an affine form's value holds no other node equal
    // to X (refcore leaves the node's `_sample` unset).
    let mut exes: Vec<Expr> = Vec::new();
    let mut stack: Vec<(&Expr, Option<usize>)> = vec![(e, None)];
    while let Some((x, within)) = stack.pop() {
        if over_paths(x) {
            continue;
        }
        let is_exempt = within
            .and_then(|at| exes.get(at))
            .is_some_and(|dist_x| dist_x == x);
        let mut name = None;
        if let Expr::Name(read) = x {
            name = Some(read);
        } else if is_exempt
            && let Expr::Post(base, _) = x
            && let Expr::Name(read) = &**base
        {
            name = Some(read);
        }
        if let Some(name) = name {
            if !is_exempt && is_dist_fact(m, name) {
                return true;
            }
            if read_node(m, name).is_some_and(|n| items.contains(&n)) {
                return true;
            }
        }
        if is_exempt && let Expr::Post(_, posts) = x {
            stack.extend(post_keys(posts).into_iter().map(|key| (key, within)));
            continue;
        }
        let mut value_of = None;
        if let Expr::DistFn(f) = x
            && let Some(dist_x) = super::random::affine_x(m, &scope, &f.value)
        {
            value_of = Some((&f.value, exes.len()));
            exes.push(dist_x);
        }
        for child in all_children(x) {
            let tag = if let Some((value, at)) = value_of
                && core::ptr::eq(child, value)
            {
                Some(at)
            } else {
                within
            };
            stack.push((child, tag));
        }
    }
    false
}

/// The children a walk for randomness or balances visits: `children`'s, and the parts of
/// the random forms, `low` and `prorata` it leaves out (refcore's walks visit every field
/// of a node; a binding's domain names keys, never a random value).
pub(super) fn all_children(e: &Expr) -> Vec<&Expr> {
    let mut out = children(e);
    if let Expr::DistFn(f) = e {
        out.extend([&f.value, &f.at]);
    } else if let Expr::Query(q) = e {
        out.push(&q.arg);
        out.extend(q.filter.as_ref());
        out.extend(q.level.as_ref());
    } else if let Expr::Crps(c) = e {
        out.extend([&c.forecast, &c.outcome]);
        out.extend(c.weight.as_ref());
    } else if let Expr::Dist(d) = e {
        out.extend(d.args.iter());
        out.extend(d.term.as_ref());
    } else if let Expr::Low(low) = e {
        out.push(&low.place);
    } else if let Expr::Prorata(p) = e {
        out.extend([&p.total, &p.weight, &p.key]);
        out.extend(p.mode.as_ref());
    }
    out
}

/// The key expressions of a read's postfixes.
fn post_keys(posts: &[Postfix]) -> Vec<&Expr> {
    let mut out = Vec::new();
    for post in posts {
        if let Postfix::Keys(keys) = post {
            for key in keys {
                if let crate::syntax::Key::Expr(key) = key {
                    out.push(key);
                }
            }
        }
    }
    out
}

/// Whether an expression reads a balance, a low, flows or a value (refcore's
/// `reads_balances`).
fn reads_balances(e: &Expr) -> bool {
    let mut stack = vec![e];
    while let Some(x) = stack.pop() {
        if matches!(
            x,
            Expr::Bal(_) | Expr::Low(_) | Expr::Flows(_) | Expr::Value(_)
        ) {
            return true;
        }
        stack.extend(all_children(x));
    }
    false
}

/// The random fact `e` reads directly, its sample (refcore's `is_fact`).
fn fact_read(m: &Reality, e: &Expr) -> Option<String> {
    let x = strip(e);
    let base = if let Expr::Post(base, posts) = x
        && posts.iter().all(|p| matches!(p, Postfix::Keys(_)))
    {
        &**base
    } else {
        x
    };
    let Expr::Name(name) = base else {
        return None;
    };
    is_dist_fact(m, name).then(|| name.clone())
}

/// The random facts read directly in an affine form, a sum of known multiples of them
/// plus a known constant; `None` when it isn't one (refcore's `affine`). L §5.16.
fn affine(
    m: &Reality,
    items: &BTreeSet<Node>,
    e: &Expr,
    depth: u32,
) -> Result<Option<Vec<String>>, Stop> {
    let depth = depth
        .checked_add(1)
        .filter(|d| *d <= AFFINE_DEPTH)
        .ok_or_else(|| invariant("an affine form nested past its count".to_owned()))?;
    let x = strip(e);
    if let Some(name) = fact_read(m, x) {
        return Ok(Some(vec![name]));
    }
    if !is_random(m, items, x) {
        return Ok(Some(Vec::new()));
    }
    if let Expr::Neg(inner) = x {
        return affine(m, items, inner, depth);
    }
    if let Expr::Sum(first, rest) = x {
        let mut out = Vec::new();
        for term in core::iter::once(&**first).chain(rest.iter().map(|(_, t)| t)) {
            let Some(found) = affine(m, items, term, depth)? else {
                return Ok(None);
            };
            out.extend(found);
        }
        return Ok(Some(out));
    }
    if let Expr::Prod(first, rest) = x {
        let mut k = rest.len();
        while let Some(at) = k.checked_sub(1) {
            let Some((op, factor)) = rest.get(at) else {
                return Err(invariant("a product's factor out of range".to_owned()));
            };
            let prefix_random = is_random(m, items, first)
                || rest.iter().take(at).any(|(_, f)| is_random(m, items, f));
            let factor_random = is_random(m, items, factor);
            if *op == crate::syntax::MulOp::Mul {
                if !prefix_random {
                    return affine(m, items, factor, depth);
                }
                if factor_random {
                    return Ok(None);
                }
            } else if factor_random {
                return Ok(None);
            }
            k = at;
        }
        return affine(m, items, first, depth);
    }
    Ok(None)
}

/// `LOAD-RANDOM`: a query with no closed form needs `paths N seed S`.
fn no_closed(q: &str, why: &str) -> Stop {
    refused(
        Load::Random,
        format!("{q} has no closed form{why}; state paths N seed S"),
    )
}

/// A query's closed form, decided at load; `LOAD-RANDOM` when there is none (refcore's
/// `closed_form`). L §5.16, its table.
fn closed_form(m: &Reality, items: &BTreeSet<Node>, form: &Expr) -> Result<(), Stop> {
    let q = text(form);
    let Expr::Query(query) = form else {
        return Err(no_closed(&q, ""));
    };
    if query.filter.is_some() {
        return Err(no_closed(&q, " (a query with where)"));
    }
    let arg = strip(&query.arg);
    let xs = if query.func == QueryFn::Share {
        let Expr::Cmp(cmp) = arg else {
            return Err(no_closed(&q, ""));
        };
        let (left, _, right) = &**cmp;
        let lr = is_random(m, items, left);
        let rr = is_random(m, items, right);
        if lr && rr {
            return Err(no_closed(&q, " (random values on both sides)"));
        }
        let side = if lr { left } else { right };
        match affine(m, items, side, 0)? {
            Some(xs) if xs.len() == 1 => xs,
            Some(_) | None => {
                return Err(no_closed(
                    &q,
                    " (one random fact, read directly, in an affine form, is needed)",
                ));
            }
        }
    } else {
        let Some(xs) = affine(m, items, arg, 0)?.filter(|xs| !xs.is_empty()) else {
            return Err(no_closed(&q, ""));
        };
        if query.func == QueryFn::Quantile && xs.len() != 1 {
            return Err(no_closed(&q, " (a quantile of more than one random fact)"));
        }
        xs
    };
    for name in xs {
        if read_node(m, &name).is_some_and(|n| items.contains(&n)) {
            return Err(no_closed(&q, &format!(" ({name} has a random parameter)")));
        }
    }
    Ok(())
}

/// Whether an ask line's value is random: it reaches the cone outside a query, or its own
/// balance, flows or projection reads a place a random movement books into (refcore's
/// `line_random`). E §3.1 step 10; L §8.3 item 10.
fn line_random(m: &Reality, cone: &Cone, ask: &Ask, e: &Expr) -> bool {
    if is_random(m, &cone.items, e) {
        return true;
    }
    let scope = Scope::new(Node::new(NodeKind::Ask, &ask.id), Stands::Ask, "", 0);
    let mut stack = vec![e];
    while let Some(x) = stack.pop() {
        if over_paths(x) {
            continue;
        }
        let mut read = None;
        if let Expr::Bal(bal) = x {
            read = Some((bal.proj.from.is_some(), place_target(m, &scope, &bal.place)));
        } else if let Expr::Low(low) = x {
            read = Some((low.proj.from.is_some(), place_target(m, &scope, &low.place)));
        } else if let Expr::Flows(flows) = x {
            read = Some((false, flows_target(m, &scope, &flows.target)));
        }
        if let Some((proj, target)) = read
            && (!proj || ask.random.is_none())
            && cone.hit(m, &target)
        {
            return true;
        }
        stack.extend(all_children(x));
    }
    false
}

/// What `flows(x, …)` marks: a decision's movements, or a place.
fn flows_target(m: &Reality, scope: &Scope, target: &Expr) -> Target {
    if let Expr::Name(id) = strip(target)
        && let Some(mv) = m.moves.get(id)
    {
        if mv.decision {
            return Target::Move(id.clone());
        }
        return Target::Unknown;
    }
    place_target(m, scope, target)
}

/// Whether `start` reaches one of `targets` over the edges and the read roles (refcore's
/// `reaches_any`).
fn reaches_any(graph: &Graph, start: &Node, targets: &BTreeSet<Node>) -> bool {
    let mut seen = BTreeSet::new();
    let mut stack = vec![start.clone()];
    while let Some(x) = stack.pop() {
        if !seen.insert(x.clone()) {
            continue;
        }
        if targets.contains(&x) {
            return true;
        }
        if let Some(reads) = graph.edges.get(&x) {
            stack.extend(reads.keys().cloned());
        }
        if let Some(reads) = graph.rand_edges.get(&x) {
            stack.extend(reads.keys().cloned());
        }
    }
    false
}

/// `LOAD-RANDOM` for a query of a value that is not random.
fn not_random(q: &crate::syntax::Query) -> Stop {
    let word = match q.func {
        QueryFn::Mean => "mean",
        QueryFn::Share => "share",
        QueryFn::Quantile => "quantile",
    };
    refused(
        Load::Random,
        format!(
            "{word} of a value that is not random: a query is asked of a random value (L §5.16)"
        ),
    )
}

/// Whether a query's argument and `where` read nothing random and no balance (refcore's
/// `finish_random`, its first test).
fn unrandom(m: &Reality, items: &BTreeSet<Node>, q: &crate::syntax::Query) -> bool {
    let filter_random = match &q.filter {
        Some(f) => is_random(m, items, f),
        None => false,
    };
    !is_random(m, items, &q.arg) && !filter_random && !reads_balances(&q.arg)
}

/// A whole number as refcore's `%d` prints it; an ask's path counts are parsed integers.
fn int_text(n: &crate::num::Num) -> Result<String, Stop> {
    n.to_i64()
        .map(|v| v.to_string())
        .ok_or_else(|| invariant("a path count past 64 bits".to_owned()))
}

/// Whether the cone counts projections: an ask line with its own random clause gives each
/// projection it reaches that clause, so its cone leaves them out (refcore's `with_proj`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Projections {
    /// Projections count.
    Counted,
    /// Projections are left out.
    Left,
}

/// The cone marked, then each query (a closed form, or an ask's paths), each ask line (no
/// random value outside a query), each ask's paths clause, and each ask that reaches `crps`
/// or `pdf` (`orient`) checked (refcore's `finish_random`). E §3.1 step 10; L §5.16, §5.15,
/// §5.18.
///
/// # Errors
///
/// `LOAD-RANDOM`, `-ASKONLY`, `-PARSE`, `-DOMAIN` or `-ORIENT` at its line.
pub(super) fn finish_random(
    m: &Reality,
    queries: &[QueryAt],
    orient_items: &BTreeSet<Node>,
    files: &Files,
) -> Result<(), Stop> {
    let cone = random_cone(m, &m.graph, Projections::Counted);
    let asks: Vec<&Ask> = m
        .asks
        .iter()
        .filter(|a| !m.asks_alone.contains_key(&a.id))
        .collect();
    for at in queries {
        let place =
            |stop: Stop| Scope::new(at.item.clone(), at.stands, &at.at.0, at.at.1).place(stop);
        let ask = if at.item.kind == NodeKind::Ask {
            asks.iter().find(|a| a.id == at.item.id)
        } else {
            None
        };
        let query = if let Expr::Query(q) = &at.form {
            Some(q)
        } else {
            None
        };
        let flat = query.is_some_and(|q| unrandom(m, &cone.items, q));
        if ask.is_none()
            && let Some(q) = query
        {
            if flat {
                return Err(place(not_random(q)));
            }
            match closed_form(m, &cone.items, &at.form) {
                Err(Stop::Refused(r)) if r.code() == crate::base::Code::Load(Load::Random) => {
                    let message = format!("{} has no closed form outside an ask", text(&at.form));
                    return Err(place(refused(Load::AskOnly, message)));
                }
                other => other.map_err(place)?,
            }
            continue;
        }
        if ask.is_some_and(|a| a.paths.is_some()) {
            continue;
        }
        if at.item.kind != NodeKind::Ask && matches!(at.form, Expr::Crps(_)) {
            let message = "crps of a random value outside a paths ask".to_owned();
            return Err(place(refused(Load::Random, message)));
        }
        if flat && let Some(q) = query {
            return Err(place(not_random(q)));
        }
        closed_form(m, &cone.items, &at.form).map_err(place)?;
    }
    let base_cone = random_cone(m, &m.graph, Projections::Left);
    for ask in &asks {
        let file = files.get("ask", &ask.id)?;
        let place = |stop: Stop| {
            Scope::new(
                Node::new(NodeKind::Ask, &ask.id),
                Stands::Ask,
                file,
                ask.line,
            )
            .place(stop)
        };
        let exprs: Vec<&Expr> = match &ask.body {
            AskBody::Exprs(es) => es.iter().collect(),
            AskBody::Check(l, r) => vec![l, r],
            AskBody::Explain(e) | AskBody::InputsOf(e) => vec![e],
            AskBody::Solve(s) => vec![&s.left, &s.right],
            AskBody::Residuals(_) | AskBody::ReadersOf(_) | AskBody::Diff(_) => Vec::new(),
        };
        let used = if ask.random.is_some() {
            &base_cone
        } else {
            &cone
        };
        for e in exprs {
            if ask.path.is_none() && line_random(m, used, ask, e) {
                let message = format!("ask {}: {} is random; apply a query", ask.id, text(e));
                return Err(place(refused(Load::Random, message)));
            }
        }
        if let (Some(p), None) = (&ask.path, &ask.paths) {
            let message = format!("cannot parse: path {} without paths N seed S", int_text(p)?);
            return Err(place(refused(Load::Parse, message)));
        }
        if let Some((n, _)) = &ask.paths {
            let one = crate::num::Num::int(1);
            if n.compare(&one).map_err(super::steps::num_stop)? == core::cmp::Ordering::Less {
                let message = format!("paths {}: N is at least 1", int_text(n)?);
                return Err(place(refused(Load::Domain, message)));
            }
            if let Some(p) = &ask.path {
                let below = p.compare(&one).map_err(super::steps::num_stop)?;
                let above = p.compare(n).map_err(super::steps::num_stop)?;
                if below == core::cmp::Ordering::Less || above == core::cmp::Ordering::Greater {
                    let message = format!("path {} is outside 1..{}", int_text(p)?, int_text(n)?);
                    return Err(place(refused(Load::Domain, message)));
                }
            }
        }
    }
    for ask in &asks {
        if ask.orient.is_some() {
            continue;
        }
        let node = Node::new(NodeKind::Ask, &ask.id);
        if reaches_any(&m.graph, &node, orient_items) {
            let file = files.get("ask", &ask.id)?;
            let message = format!("ask {} reaches crps or pdf without orient", ask.id);
            let scope = Scope::new(node, Stands::Ask, file, ask.line);
            return Err(scope.place(refused(Load::Orient, message)));
        }
    }
    Ok(())
}
