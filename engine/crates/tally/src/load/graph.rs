//! The item graph's edges and marks: one walk over every expression of the reality, each read recorded with how its keys relate to its reader's.
//! Spec: E §3.1 step 5 (nodes, edges with their key relations, a binding over a kind), step 7
//! (the marks: running reads and projections); L §5.14 (a move's day, legs and reads),
//! §5.14.1 (running reads, the written projection), §5.10 (a value reads its prices), §4.2 (a
//! calendar is read at each day asked about).
//! Twin: engine/tools/refcore/check.py (`Checker.edge`, `read`, `relation`, `bind`, `t_agg`,
//! `t_in`, `band_call`, `band_ref`, `cal_arg`, `c_since`, `t_lastx`, `check_def`,
//! `check_band`, `check_ask`, `check_prices`); check3.py (`m_check_move`, `m_read_target`,
//! `m_t_bal`, `m_t_flows`, `m_t_value`, `m_price_edges`, `m_bind_movements`, `m_place_value`,
//! `m_move_value`, `m_literal_place`, `m_place_target`, `m_move_changes`, `m_resolve_price`,
//! `m_c_next`, `m_c_observed`, `m_legof_read`).
//! Invariants: the walk records what refcore's checker records on the forms S builds and
//! refuses nothing (the checker's refusals are M0's); its recursion is counted (R-16), and the
//! parser bounds every tree it walks well below the count.

use super::keys::keyed_place_thing;
use crate::base::{Code, Refusal};
use crate::model::{
    BindInfo, Domain, Graph, KindSpec, Mark, MoveInfo, NameKind, Node, NodeKind, Reality, Rel,
    Shift, Target, Thing, Ty,
};
use crate::syntax::{
    AddOp, Agg, Ask, AskBody, Bal, Bind, Call, Domain as Written, Expr, Field, FieldValue, Flows,
    In, Key, LawDate, Leg, Lit, MulOp, OfQ, Postfix, Price, RuleValue, Span, Step, Stop, Valuation,
};
use std::collections::{BTreeMap, BTreeSet};

/// How deep the walk goes into one expression. The parser's own limits (eight brackets or
/// `if`s, 64 chained reads) keep every tree far below it.
const MAX_DEPTH: u32 = 512;

/// The date functions whose names no band's call shadows (refcore's `DATE_FNS`).
const DATE_FNS: [&str; 21] = [
    "start_of",
    "end_of",
    "year_of",
    "quarter_of",
    "month_of",
    "day_of",
    "days_between",
    "add_days",
    "add_months",
    "add_years",
    "overlap_days",
    "age",
    "age_months",
    "weekday",
    "pos",
    "is_business",
    "add_business",
    "last_business",
    "schedule",
    "window",
    "one_of",
];

/// What a bound variable ranges over, as far as the graph needs: a kind's things or
/// movements (whose places a leg may name), or other keys.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Ranges {
    /// The things or movements of a kind.
    Kind(String),
    /// Keys of an index, a list or a range.
    Keys,
}

/// Whether a read stands under an aggregate's binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Under {
    /// In the item's own expression.
    Item,
    /// Under an aggregate's binding.
    Aggregate,
}

/// Where an expression is read: the reading item, its variables, and how its reads relate
/// to its keys (refcore's `Scope`, the fields the graph needs).
#[derive(Debug, Clone)]
struct Scope {
    /// The item that reads.
    item: Node,
    /// The variables in scope.
    vars: BTreeMap<String, Ranges>,
    /// The item's own key variables.
    keyvars: Vec<String>,
    /// Whether the read is under an aggregate's binding.
    under: Under,
    /// A read at a day the reader states (`explicit`) or at each day asked (`calendar`),
    /// whatever its keys.
    mode: Option<Rel>,
}

impl Scope {
    /// The scope of `item`'s own expression, with no variable yet.
    fn new(item: Node) -> Scope {
        Scope {
            item,
            vars: BTreeMap::new(),
            keyvars: Vec::new(),
            under: Under::Item,
            mode: None,
        }
    }

    /// The scope of a declaration keyed by `binds`: their variables are its keys.
    fn keyed(item: Node, binds: &[BindInfo]) -> Scope {
        let mut scope = Scope::new(item);
        for bind in binds {
            scope.vars.insert(bind.var.clone(), ranges_of(&bind.domain));
            scope.keyvars.push(bind.var.clone());
        }
        scope
    }

    /// Whether `name` is a bound variable here.
    fn is_var(&self, name: &str) -> bool {
        self.vars.contains_key(name)
    }

    /// This scope, its reads made in `mode`.
    fn with_mode(&self, mode: Rel) -> Scope {
        let mut scope = self.clone();
        scope.mode = Some(mode);
        scope
    }

    /// This scope under an aggregate's binding.
    fn in_aggregate(&self) -> Scope {
        let mut scope = self.clone();
        scope.under = Under::Aggregate;
        scope
    }

    /// This scope, read by `item` instead.
    fn for_item(&self, item: Node) -> Scope {
        let mut scope = self.clone();
        scope.item = item;
        scope
    }
}

/// What a declared key bind ranges over.
fn ranges_of(domain: &Domain) -> Ranges {
    match domain {
        Domain::Kind(kind) => Ranges::Kind(kind.clone()),
        Domain::Index(_) | Domain::Axis(_) | Domain::List(_) | Domain::Range => Ranges::Keys,
    }
}

/// What one move's walk finds: the reads it declares and the places its legs change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MoveFacts {
    /// The move.
    pub(super) id: String,
    /// What its `reads` declares, in the order written.
    pub(super) read_targets: Vec<Target>,
    /// The places its legs change, each once.
    pub(super) changes: Vec<Target>,
}

/// One level deeper into an expression, refused as an invariant past the count (R-16).
fn deeper(depth: u32) -> Result<u32, Stop> {
    let too_deep = || {
        Stop::Refused(Refusal::new(
            Code::EngineInvariant,
            "an expression nested deeper than the parser allows".to_owned(),
        ))
    };
    let next = depth.checked_add(1).ok_or_else(too_deep)?;
    if next > MAX_DEPTH {
        return Err(too_deep());
    }
    Ok(next)
}

/// The item graph's edges and marks, and each move's declared reads and changes (refcore's
/// `Checker.run`, its walk: price clauses, inputs' ranges, moves by id, defs, bands, asks,
/// `prices … by`). E §3.1 steps 5, 7.
pub(super) fn walk(m: &Reality) -> Result<(Graph, Vec<MoveFacts>), Stop> {
    let mut graph = Graph::default();
    graph.priced(m)?;
    for (id, input) in m.inputs.entries() {
        let scope = Scope::keyed(Node::new(NodeKind::Input, id), &input.binds);
        graph.range_ends(m, &scope, &input.binds)?;
    }
    let mut moves: Vec<&MoveInfo> = m.moves.entries().map(|(_, mv)| mv).collect();
    moves.sort_by(|a, b| a.id.cmp(&b.id));
    let mut facts = Vec::new();
    for mv in moves {
        facts.push(graph.check_move(m, mv)?);
    }
    for (id, def) in m.defs.entries() {
        let scope = Scope::keyed(Node::new(NodeKind::Def, id), &def.binds);
        graph.range_ends(m, &scope, &def.binds)?;
        for bind in &def.binds {
            graph.opt(m, &scope, bind.node.filter.as_ref(), 0)?;
        }
        if let Some(LawDate::Expr(at)) = &def.at {
            graph.expr(m, &scope, at, 0)?;
        }
        for (_, versions) in &def.rules {
            for version in versions {
                graph.opt(m, &scope, version.rule.when.as_ref(), 0)?;
                graph.rule_value(m, &scope, &version.rule.value)?;
            }
        }
    }
    for (id, band) in m.bands.entries() {
        let mut scope = Scope::new(Node::new(NodeKind::Band, id));
        scope.vars.insert(band.arg.clone(), Ranges::Keys);
        for table in &band.tables {
            for row in &table.rows {
                graph.rule_value(m, &scope, &row.value)?;
            }
        }
    }
    for ask in &m.asks {
        graph.check_ask(m, ask)?;
    }
    graph.prices_by(m);
    Ok((graph, facts))
}

impl Graph {
    /// A keyed price clause reads its input at the keys it states, from the clause's own
    /// node; an unkeyed one is read where a place is valued (refcore's `m_check_priced`,
    /// `m_resolve_price`). L §5.10.
    fn priced(&mut self, m: &Reality) -> Result<(), Stop> {
        let mut kinds: Vec<(&String, &Price)> = Vec::new();
        for (id, kind) in m.kinds.entries() {
            if let Some(price) = &kind.priced {
                kinds.push((id, price));
            }
        }
        kinds.sort_by(|a, b| a.0.cmp(b.0));
        for (id, price) in kinds {
            let scope = Scope::new(Node::new(NodeKind::Price, &format!("kind {id}")));
            self.price_clause(m, &scope, price)?;
        }
        let mut things: Vec<&Thing> = m.things.entries().map(|(_, thing)| thing).collect();
        things.sort_by(|a, b| a.name.cmp(&b.name));
        let mut sections = BTreeSet::new();
        for thing in things {
            for entry in &thing.entries {
                let Some(price) = &entry.priced else {
                    continue;
                };
                let item = Node::new(NodeKind::Price, &format!("place {}", thing.name));
                let section = if entry.version.is_none() {
                    section_of(m, thing).and_then(|id| m.places.get(&id))
                } else {
                    None
                };
                let scope = match section {
                    Some(place) if !sections.insert(place.id.clone()) => continue,
                    Some(place) => Scope::keyed(item, &place.binds),
                    None => Scope::new(item),
                };
                self.price_clause(m, &scope, price)?;
            }
        }
        Ok(())
    }

    /// A price clause written with keys, `px[f]`, read at them.
    fn price_clause(&mut self, m: &Reality, scope: &Scope, price: &Price) -> Result<(), Stop> {
        if let Expr::Post(base, posts) = &price.input
            && let Expr::Name(_) = base.as_ref()
            && let [Postfix::Keys(_)] = posts.as_slice()
        {
            return self.expr(m, scope, &price.input, 0);
        }
        Ok(())
    }

    /// `prices Y by ix`: the input reads the index it is priced by (refcore's
    /// `check_prices`). L §3.4.
    fn prices_by(&mut self, m: &Reality) {
        for (id, input) in m.inputs.entries() {
            let Some(by) = &input.prices_by else {
                continue;
            };
            if let Some((node, _)) = input_or_def(m, by) {
                self.edge(&Node::new(NodeKind::Input, id), node, Rel::Any);
            }
        }
    }

    /// The ends and steps of a declaration's stepped ranges, read by the declaration.
    fn range_ends(&mut self, m: &Reality, scope: &Scope, binds: &[BindInfo]) -> Result<(), Stop> {
        for bind in binds {
            if let Written::Range(range) = &bind.node.domain {
                self.range(m, scope, range, 0)?;
            }
        }
        Ok(())
    }

    /// A range's ends and step.
    fn range(
        &mut self,
        m: &Reality,
        scope: &Scope,
        range: &(Expr, Expr, Option<Expr>),
        depth: u32,
    ) -> Result<(), Stop> {
        self.expr(m, scope, &range.0, depth)?;
        self.expr(m, scope, &range.1, depth)?;
        self.opt(m, scope, range.2.as_ref(), depth)
    }

    /// A rule's or a band row's value, unless it is a stated hole.
    fn rule_value(&mut self, m: &Reality, scope: &Scope, value: &RuleValue) -> Result<(), Stop> {
        match value {
            RuleValue::Expr(expr) => self.expr(m, scope, expr, 0),
            RuleValue::Hole(_) => Ok(()),
        }
    }

    /// A move: its binds, its day (read by its own day node), its `for`, `when`, legs,
    /// attributes, `fulfils` and `ties by`, and what it declares it reads (refcore's
    /// `m_check_move`, `m_move_changes`). L §5.14, §5.14.1.
    fn check_move(&mut self, m: &Reality, mv: &MoveInfo) -> Result<MoveFacts, Stop> {
        let decl = &mv.decl;
        let mut scope = Scope::new(Node::new(NodeKind::Move, &mv.id));
        for bind in &mv.binds {
            self.bind(m, &mut scope, &bind.node, 0)?;
        }
        scope.keyvars = mv.binds.iter().map(|bind| bind.var.clone()).collect();
        let day = scope.for_item(Node::new(NodeKind::MoveOn, &mv.id));
        self.edge(&scope.item, day.item.clone(), Rel::Any);
        self.expr(m, &day, &decl.on, 0)?;
        self.opt(m, &day, decl.at.as_ref(), 0)?;
        self.opt(m, &scope, decl.for_period.as_ref(), 0)?;
        self.opt(m, &scope, decl.when.as_ref(), 0)?;
        let mut changes = Vec::new();
        for leg in &decl.legs {
            self.leg(m, &scope, leg, 0)?;
            push_once(&mut changes, change_of(m, &scope, &leg.from));
            push_once(&mut changes, change_of(m, &scope, &leg.to));
        }
        for (_, value) in &decl.attrs {
            self.expr(m, &scope, value, 0)?;
        }
        self.opt(m, &scope, decl.fulfils.as_ref(), 0)?;
        if let Some(ties) = &decl.ties {
            self.all(m, &scope, ties, 0)?;
        }
        let mut read_targets = Vec::new();
        for read in &decl.reads {
            read_targets.push(self.read_target(m, &scope, read)?);
        }
        Ok(MoveFacts {
            id: mv.id.clone(),
            read_targets,
            changes,
        })
    }

    /// What a `reads` line names: a move, all of a keyed place, an event kind, or the place
    /// an expression gives (refcore's `m_read_target`). L §5.14.1.
    fn read_target(&mut self, m: &Reality, scope: &Scope, read: &Expr) -> Result<Target, Stop> {
        if let Expr::Name(id) = read
            && !scope.is_var(id)
        {
            let named = m.names.get(id);
            if named == Some(&NameKind::Move) {
                return Ok(Target::Move(id.clone()));
            }
            let keyed = m
                .places
                .get(id)
                .is_some_and(|place| !place.binds.is_empty());
            if named == Some(&NameKind::Place) && keyed {
                return Ok(Target::PlaceSec(id.clone()));
            }
            let event = m
                .kinds
                .get(id)
                .is_some_and(|kind| kind.spec == KindSpec::Event);
            if named == Some(&NameKind::Kind) && event {
                return Ok(Target::Kind(id.clone()));
            }
        }
        self.expr(m, scope, read, 0)?;
        Ok(place_target(m, scope, read))
    }

    /// An ask: its bindings, `law`, `as known` and what it asks (refcore's `check_ask`).
    /// L §5.18.
    fn check_ask(&mut self, m: &Reality, ask: &Ask) -> Result<(), Stop> {
        let mut scope = Scope::new(Node::new(NodeKind::Ask, &ask.id));
        for bind in &ask.binds {
            self.bind(m, &mut scope, bind, 0)?;
        }
        self.opt(m, &scope, ask.law.as_ref(), 0)?;
        self.opt(m, &scope, ask.known.as_ref(), 0)?;
        match &ask.body {
            AskBody::Exprs(exprs) => self.all(m, &scope, exprs, 0),
            AskBody::Residuals(residuals) => {
                self.expr(m, &scope, &residuals.place, 0)?;
                self.span(m, &scope, &residuals.span, 0)
            }
        }
    }

    /// An expression, one arm per node kind (refcore's `tc`, the reads it records).
    fn expr(&mut self, m: &Reality, scope: &Scope, expr: &Expr, depth: u32) -> Result<(), Stop> {
        let depth = deeper(depth)?;
        match expr {
            Expr::Lit(_)
            | Expr::Str(_)
            | Expr::None
            | Expr::Never
            | Expr::Cut
            | Expr::List(_)
            | Expr::Calendar(_) => Ok(()),
            Expr::Name(name) => self.name(m, scope, name, &[], depth),
            Expr::If(parts) => self.all(m, scope, parts.as_slice(), depth),
            Expr::Or(terms) | Expr::And(terms) => self.all(m, scope, terms, depth),
            Expr::Not(inner) | Expr::Neg(inner) | Expr::Paren(inner) => {
                self.expr(m, scope, inner, depth)
            }
            Expr::Cmp(cmp) => self.pair(m, scope, &cmp.0, &cmp.2, depth),
            Expr::In(test) => self.membership(m, scope, test, depth),
            Expr::Sum(first, rest) => self.sum(m, scope, first, rest, depth),
            Expr::Prod(first, rest) => self.product(m, scope, first, rest, depth),
            Expr::Post(base, posts) => self.post(m, scope, base, posts, depth),
            Expr::Call(call) => self.call(m, scope, call, depth),
            Expr::LegOf(inner, of) => self.leg_of(m, scope, inner, of, depth),
            Expr::To(inner, step) => self.to(m, scope, inner, step, depth),
            Expr::Record(fields) => self.record(m, scope, fields, depth),
            Expr::Agg(agg) => self.aggregate(m, scope, agg, depth),
            Expr::Last(pair) => self.explicit(m, scope, &pair.0, Some(&pair.1), depth),
            Expr::Bal(bal) => self.bal(m, scope, bal, depth),
            Expr::Flows(flows) => self.flows(m, scope, flows, depth),
            Expr::Covered(covered) => self.covered(m, scope, &covered.0, &covered.1, depth),
            Expr::Value(value) => self.value(m, scope, value, depth),
        }
    }

    /// Each of `exprs`.
    fn all(&mut self, m: &Reality, scope: &Scope, exprs: &[Expr], depth: u32) -> Result<(), Stop> {
        for expr in exprs {
            self.expr(m, scope, expr, depth)?;
        }
        Ok(())
    }

    /// `expr`, where one is written.
    fn opt(
        &mut self,
        m: &Reality,
        scope: &Scope,
        expr: Option<&Expr>,
        depth: u32,
    ) -> Result<(), Stop> {
        match expr {
            Some(expr) => self.expr(m, scope, expr, depth),
            None => Ok(()),
        }
    }

    /// Two expressions.
    fn pair(
        &mut self,
        m: &Reality,
        scope: &Scope,
        left: &Expr,
        right: &Expr,
        depth: u32,
    ) -> Result<(), Stop> {
        self.expr(m, scope, left, depth)?;
        self.expr(m, scope, right, depth)
    }

    /// A flat chain of sums: its first term and the rest.
    fn sum(
        &mut self,
        m: &Reality,
        scope: &Scope,
        first: &Expr,
        rest: &[(AddOp, Expr)],
        depth: u32,
    ) -> Result<(), Stop> {
        self.expr(m, scope, first, depth)?;
        for (_, term) in rest {
            self.expr(m, scope, term, depth)?;
        }
        Ok(())
    }

    /// A flat chain of products: its first factor and the rest.
    fn product(
        &mut self,
        m: &Reality,
        scope: &Scope,
        first: &Expr,
        rest: &[(MulOp, Expr)],
        depth: u32,
    ) -> Result<(), Stop> {
        self.expr(m, scope, first, depth)?;
        for (_, factor) in rest {
            self.expr(m, scope, factor, depth)?;
        }
        Ok(())
    }

    /// A span's ends.
    fn span(&mut self, m: &Reality, scope: &Scope, span: &Span, depth: u32) -> Result<(), Stop> {
        self.opt(m, scope, span.from.as_ref(), depth)?;
        self.opt(m, scope, span.to.as_ref(), depth)
    }

    /// The key expressions of a read.
    fn keys(&mut self, m: &Reality, scope: &Scope, keys: &[Key], depth: u32) -> Result<(), Stop> {
        for key in keys {
            if let Key::Expr(expr) = key {
                self.expr(m, scope, expr, depth)?;
            }
        }
        Ok(())
    }

    /// A name, with the keys it is read at: an input or def read, a place, a movement
    /// instance, or a variable or key, which read nothing (refcore's `t_name`, `t_index`).
    fn name(
        &mut self,
        m: &Reality,
        scope: &Scope,
        name: &str,
        keys: &[Key],
        depth: u32,
    ) -> Result<(), Stop> {
        if scope.is_var(name) {
            return self.keys(m, scope, keys, depth);
        }
        match m.names.get(name).copied() {
            Some(NameKind::Input) => {
                self.read(m, scope, Node::new(NodeKind::Input, name), keys, depth)
            }
            Some(NameKind::Def) => self.read(m, scope, Node::new(NodeKind::Def, name), keys, depth),
            Some(NameKind::Place) => self.place_value(m, scope, name, keys, depth),
            Some(NameKind::Move) => self.move_value(m, scope, name, keys, depth),
            Some(NameKind::Axis) => Err(Stop::NotBuilt("axes")),
            Some(
                NameKind::Unit
                | NameKind::Kind
                | NameKind::Index
                | NameKind::Band
                | NameKind::Fill
                | NameKind::Version
                | NameKind::Ask
                | NameKind::Source
                | NameKind::Cut,
            )
            | None => self.keys(m, scope, keys, depth),
        }
    }

    /// A read of an input or def at keys, with its relation; an observation is read by its
    /// row's id, at any key (refcore's `read`). E §3.1 step 5.
    fn read(
        &mut self,
        m: &Reality,
        scope: &Scope,
        target: Node,
        keys: &[Key],
        depth: u32,
    ) -> Result<(), Stop> {
        let observed = target.kind == NodeKind::Input
            && m.inputs.get(&target.id).is_some_and(|input| input.observed);
        if observed {
            self.edge(&scope.item, target, Rel::Any);
            return Ok(());
        }
        self.keys(m, scope, keys, depth)?;
        self.edge(&scope.item, target, relation(scope, keys));
        Ok(())
    }

    /// A place read by its id, or a keyed place at keys (refcore's `m_place_value`). L §5.10.
    fn place_value(
        &mut self,
        m: &Reality,
        scope: &Scope,
        id: &str,
        keys: &[Key],
        depth: u32,
    ) -> Result<(), Stop> {
        self.keys(m, scope, keys, depth)?;
        self.edge(&scope.item, Node::new(NodeKind::Place, id), Rel::Any);
        Ok(())
    }

    /// A movement instance named by its move and keys; a decision's is a running read of its
    /// legs (refcore's `m_move_value`). L §2.5, §5.14.1.
    fn move_value(
        &mut self,
        m: &Reality,
        scope: &Scope,
        id: &str,
        keys: &[Key],
        depth: u32,
    ) -> Result<(), Stop> {
        self.keys(m, scope, keys, depth)?;
        self.edge(&scope.item, Node::new(NodeKind::Move, id), Rel::Any);
        if m.moves.get(id).is_some_and(|mv| mv.decision) {
            self.mark(&scope.item, Mark::Legs, Target::Move(id.to_owned()));
        }
        Ok(())
    }

    /// A base with its postfixes: a name read at its first keys, then fields, which read
    /// nothing (refcore's `t_index`, `t_field`).
    fn post(
        &mut self,
        m: &Reality,
        scope: &Scope,
        base: &Expr,
        posts: &[Postfix],
        depth: u32,
    ) -> Result<(), Stop> {
        let rest = if let Expr::Name(name) = base
            && !scope.is_var(name)
            && let [Postfix::Keys(keys), rest @ ..] = posts
        {
            self.name(m, scope, name, keys, depth)?;
            rest
        } else {
            self.expr(m, scope, base, depth)?;
            posts
        };
        for post in rest {
            if let Postfix::Keys(keys) = post {
                self.keys(m, scope, keys, depth)?;
            }
        }
        Ok(())
    }

    /// `a in b [on d]`: a kind or an index named as the set reads nothing (refcore's `t_in`).
    fn membership(
        &mut self,
        m: &Reality,
        scope: &Scope,
        test: &In,
        depth: u32,
    ) -> Result<(), Stop> {
        self.expr(m, scope, &test.member, depth)?;
        self.opt(m, scope, test.on.as_ref(), depth)?;
        if let Expr::Name(id) = &test.set
            && !scope.is_var(id)
            && matches!(m.names.get(id), Some(NameKind::Kind | NameKind::Index))
        {
            return Ok(());
        }
        self.expr(m, scope, &test.set, depth)
    }

    /// A call: a band's, or a built-in's, some of which read in a mode of their own
    /// (refcore's `t_call`).
    fn call(&mut self, m: &Reality, scope: &Scope, call: &Call, depth: u32) -> Result<(), Stop> {
        if !call.prefix.is_empty() {
            return Err(Stop::NotBuilt("a member's band"));
        }
        let named_band = !DATE_FNS.contains(&call.name.as_str())
            && m.names.get(&call.name) == Some(&NameKind::Band);
        if call.band_key.is_some() || named_band {
            return self.band_call(m, scope, call, depth);
        }
        let args = call.args.as_slice();
        match call.name.as_str() {
            "is_business" => self.calendar_call(m, scope, args, 1, depth),
            "add_business" | "last_business" => self.calendar_call(m, scope, args, 2, depth),
            "since" => self.explicit_first(m, scope, args, depth),
            "next" => self.next(m, scope, args, depth),
            "band_lo" | "band_hi" | "marginal" => self.band_ref(m, scope, args, depth),
            "observed" => self.observed(m, scope, args, depth),
            "gap" => self.gap(m, scope, args, depth),
            _ => self.all(m, scope, args, depth),
        }
    }

    /// `b(x)` or `b[k](x)`: the band at the reader's key, or at a computed one (refcore's
    /// `band_call`). L §5.9.
    fn band_call(
        &mut self,
        m: &Reality,
        scope: &Scope,
        call: &Call,
        depth: u32,
    ) -> Result<(), Stop> {
        if let Some(Key::Expr(key)) = &call.band_key {
            self.expr(m, scope, key, depth)?;
        }
        self.all(m, scope, &call.args, depth)?;
        let band = m
            .bands
            .get(&call.name)
            .filter(|_| m.names.get(&call.name) == Some(&NameKind::Band));
        if let Some(band) = band {
            let rel = if band.binds.is_empty() {
                Rel::Same
            } else {
                Rel::Computed
            };
            self.edge(&scope.item, Node::new(NodeKind::Band, &band.id), rel);
        }
        Ok(())
    }

    /// A business-day function: its calendar argument, at `at`, is read at each day asked
    /// about (refcore's `cal_arg`). L §4.2.
    fn calendar_call(
        &mut self,
        m: &Reality,
        scope: &Scope,
        args: &[Expr],
        at: usize,
        depth: u32,
    ) -> Result<(), Stop> {
        let calendar = scope.with_mode(Rel::Calendar);
        for (position, arg) in args.iter().enumerate() {
            let read_by = if position == at { &calendar } else { scope };
            self.expr(m, read_by, arg, depth)?;
        }
        Ok(())
    }

    /// `since(r, d)`: the reading read at the day stated (refcore's `c_since`). L §4.6.
    fn explicit_first(
        &mut self,
        m: &Reality,
        scope: &Scope,
        args: &[Expr],
        depth: u32,
    ) -> Result<(), Stop> {
        let Some((read, rest)) = args.split_first() else {
            return Ok(());
        };
        self.explicit(m, scope, read, None, depth)?;
        self.all(m, scope, rest, depth)
    }

    /// A read at a day the reader states, then that day (refcore's `t_lastx`). L §4.6.
    fn explicit(
        &mut self,
        m: &Reality,
        scope: &Scope,
        read: &Expr,
        day: Option<&Expr>,
        depth: u32,
    ) -> Result<(), Stop> {
        self.expr(m, &scope.with_mode(Rel::Explicit), read, depth)?;
        self.opt(m, scope, day, depth)
    }

    /// `next(K, d)`: every move of K or a subkind (refcore's `m_c_next`). L §5.14.1.
    fn next(&mut self, m: &Reality, scope: &Scope, args: &[Expr], depth: u32) -> Result<(), Stop> {
        let Some((kind, rest)) = args.split_first() else {
            return Ok(());
        };
        if let Expr::Name(id) = kind
            && m.names.get(id) == Some(&NameKind::Kind)
        {
            let sub = m.subtree(id);
            for (move_id, mv) in m.moves.entries() {
                if sub.contains(&mv.kind) {
                    self.edge(&scope.item, Node::new(NodeKind::Move, move_id), Rel::Next);
                }
            }
        }
        self.all(m, scope, rest, depth)
    }

    /// `band_lo(b, x)`, `band_hi(b, x)`, `marginal(b, x)`: the band, then the argument
    /// (refcore's `band_ref`). L §5.9.
    fn band_ref(
        &mut self,
        m: &Reality,
        scope: &Scope,
        args: &[Expr],
        depth: u32,
    ) -> Result<(), Stop> {
        let Some((band, rest)) = args.split_first() else {
            return Ok(());
        };
        if let Some((id, posts)) = named_parts(band)
            && m.names.get(id) == Some(&NameKind::Band)
        {
            for post in posts {
                if let Postfix::Keys(keys) = post {
                    self.keys(m, scope, keys, depth)?;
                }
            }
            self.edge(&scope.item, Node::new(NodeKind::Band, id), Rel::Any);
        } else {
            self.expr(m, scope, band, depth)?;
        }
        self.all(m, scope, rest, depth)
    }

    /// `observed(m[k])`: the planned instance, and every observed input (refcore's
    /// `m_c_observed`). L §5.19.
    fn observed(
        &mut self,
        m: &Reality,
        scope: &Scope,
        args: &[Expr],
        depth: u32,
    ) -> Result<(), Stop> {
        self.all(m, scope, args, depth)?;
        let instance = args.first().and_then(named_parts);
        if instance.is_some_and(|(id, _)| m.names.get(id) == Some(&NameKind::Move)) {
            for (id, input) in m.inputs.entries() {
                if input.observed {
                    self.edge(&scope.item, Node::new(NodeKind::Input, id), Rel::Observed);
                }
            }
        }
        Ok(())
    }

    /// `gap(x, d)`: a balance read (refcore's `m_c_gap`). L §5.19.
    fn gap(&mut self, m: &Reality, scope: &Scope, args: &[Expr], depth: u32) -> Result<(), Stop> {
        self.all(m, scope, args, depth)?;
        self.edge(&scope.item, Node::balances(), Rel::Bal);
        Ok(())
    }

    /// `m.leg.amount of q`: booking state, read as a balance is (refcore's `m_t_legof`,
    /// `m_legof_read`). L §5.14.1 (R-10).
    fn leg_of(
        &mut self,
        m: &Reality,
        scope: &Scope,
        inner: &Expr,
        of: &OfQ,
        depth: u32,
    ) -> Result<(), Stop> {
        self.expr(m, scope, inner, depth)?;
        self.of(m, scope, Some(of), depth)?;
        self.edge(&scope.item, Node::balances(), Rel::Bal);
        self.mark(&scope.item, Mark::LegOf, Target::Unknown);
        Ok(())
    }

    /// `of q`: the party, the batch, or the pair.
    fn of(&mut self, m: &Reality, scope: &Scope, of: Option<&OfQ>, depth: u32) -> Result<(), Stop> {
        match of {
            None => Ok(()),
            Some(OfQ::Ref(expr)) => self.expr(m, scope, expr, depth),
            Some(OfQ::Pair(party, batch)) => self.pair(m, scope, party, batch, depth),
        }
    }

    /// `e to u`: the value and a unit written as a reference.
    fn to(
        &mut self,
        m: &Reality,
        scope: &Scope,
        inner: &Expr,
        step: &Step,
        depth: u32,
    ) -> Result<(), Stop> {
        self.expr(m, scope, inner, depth)?;
        match step {
            Step::Ref(expr) => self.expr(m, scope, expr, depth),
            Step::Number(_) | Step::Minor(_) => Ok(()),
        }
    }

    /// A record's values and legs.
    fn record(
        &mut self,
        m: &Reality,
        scope: &Scope,
        fields: &[Field],
        depth: u32,
    ) -> Result<(), Stop> {
        for field in fields {
            match field {
                Field::Named(_, FieldValue::Expr(expr)) => self.expr(m, scope, expr, depth)?,
                Field::Named(_, FieldValue::Lit(_) | FieldValue::Hole(_) | FieldValue::None) => {}
                Field::Leg(leg) => self.leg(m, scope, leg, depth)?,
            }
        }
        Ok(())
    }

    /// A leg: its places, its amount and its own day (refcore's `m_check_leg`). L §5.14.
    fn leg(&mut self, m: &Reality, scope: &Scope, leg: &Leg, depth: u32) -> Result<(), Stop> {
        self.pair(m, scope, &leg.from, &leg.to, depth)?;
        self.expr(m, scope, &leg.amount, depth)?;
        self.opt(m, scope, leg.on.as_ref(), depth)
    }

    /// An aggregate: its bindings, its term and its `ties by`, under the bindings (refcore's
    /// `t_agg`). L §5.11.
    fn aggregate(&mut self, m: &Reality, scope: &Scope, agg: &Agg, depth: u32) -> Result<(), Stop> {
        let mut inner = scope.in_aggregate();
        for bind in &agg.binds {
            self.bind(m, &mut inner, bind, depth)?;
        }
        self.opt(m, &inner, agg.term.as_ref(), depth)?;
        if let Some(ties) = &agg.ties {
            self.all(m, &inner, ties, depth)?;
        }
        Ok(())
    }

    /// A binding: what it ranges over, then its `on`, `over` and `where`, with its variable
    /// in scope (refcore's `bind`, `bind_finish`). L §5.6.
    fn bind(
        &mut self,
        m: &Reality,
        scope: &mut Scope,
        bind: &Bind,
        depth: u32,
    ) -> Result<(), Stop> {
        let ranges = match &bind.domain {
            Written::List(_) => Ranges::Keys,
            Written::Range(range) => {
                self.range(m, scope, range, depth)?;
                Ranges::Keys
            }
            Written::Name { qual: Some(_), .. } => return Err(Stop::NotBuilt("a member's prefix")),
            Written::Name { id, qual: None } => self.bind_name(m, scope, id),
        };
        scope.vars.insert(bind.var.clone(), ranges);
        self.opt(m, scope, bind.on.as_ref(), depth)?;
        if let Some(over) = &bind.over {
            self.span(m, scope, over, depth)?;
        }
        self.opt(m, scope, bind.filter.as_ref(), depth)
    }

    /// A binding over a name: an observed input's rows, an event kind's movements, a place
    /// kind's places, or keys (refcore's `bind`, `m_bind_movements`). E §3.1 step 5.
    fn bind_name(&mut self, m: &Reality, scope: &Scope, id: &str) -> Ranges {
        let named = m.names.get(id);
        let observed = m.inputs.get(id).is_some_and(|input| input.observed);
        if named == Some(&NameKind::Input) && observed {
            let input = Node::new(NodeKind::Input, id);
            self.edge(&scope.item, input.clone(), Rel::Bind);
            self.edge(&scope.item, input, Rel::Named);
            return Ranges::Kind("events".to_owned());
        }
        let Some(kind) = m.kinds.get(id).filter(|_| named == Some(&NameKind::Kind)) else {
            return Ranges::Keys;
        };
        match kind.spec {
            KindSpec::Event => self.bind_movements(m, scope, id),
            KindSpec::Place => self.bind_places(m, scope, id),
            KindSpec::Plain | KindSpec::Unit => {}
        }
        Ranges::Kind(id.to_owned())
    }

    /// A binding over an event kind reads every move of it or a subkind, and every observed
    /// input; one over a decision's movements or over `pads` is a running read (refcore's
    /// `m_bind_movements`). L §5.14.1.
    fn bind_movements(&mut self, m: &Reality, scope: &Scope, kind: &str) {
        let sub = m.subtree(kind);
        for (id, mv) in m.moves.entries() {
            if !sub.contains(&mv.kind) {
                continue;
            }
            self.edge(&scope.item, Node::new(NodeKind::Move, id), Rel::Bind);
            if mv.decision {
                self.mark(&scope.item, Mark::Legs, Target::Move(id.clone()));
            }
        }
        if sub.contains("pads") {
            self.mark(&scope.item, Mark::Legs, Target::Kind("pads".to_owned()));
        }
        for (id, input) in m.inputs.entries() {
            if input.observed {
                self.edge(&scope.item, Node::new(NodeKind::Input, id), Rel::Bind);
            }
        }
    }

    /// A binding over a place kind reads every place section of it or a subkind.
    fn bind_places(&mut self, m: &Reality, scope: &Scope, kind: &str) {
        let sub = m.subtree(kind);
        for (id, place) in m.places.entries() {
            if sub.contains(&place.kind) {
                self.edge(&scope.item, Node::new(NodeKind::Place, id), Rel::Bind);
            }
        }
    }

    /// A balance read: the place, its days, its clauses; marked as a running read, a
    /// projection or `bal(x)` with no day (refcore's `m_t_bal`, `m_check_proj`).
    /// L §5.14.1; E §3.1 step 7.
    fn bal(&mut self, m: &Reality, scope: &Scope, bal: &Bal, depth: u32) -> Result<(), Stop> {
        self.expr(m, scope, &bal.place, depth)?;
        self.all(m, scope, &bal.args, depth)?;
        self.opt(m, scope, bal.proj.from.as_ref(), depth)?;
        if !bal.modelled {
            let mark = self.bal_mark(scope, bal);
            self.mark(&scope.item, mark, place_target(m, scope, &bal.place));
        }
        self.edge(&scope.item, Node::balances(), Rel::Bal);
        self.of(m, scope, bal.of.as_ref(), depth)
    }

    /// A balance read's mark: a projection, `bal(x)` with no day, or a running read at a
    /// day; `bal(x, d) modelled` reads the modelled past and takes none (L §5.19).
    fn bal_mark(&mut self, scope: &Scope, bal: &Bal) -> Mark {
        if bal.proj.from.is_some() {
            self.holders.insert(scope.item.clone());
            return Mark::Proj;
        }
        if bal.args.is_empty() {
            self.balnow.insert(scope.item.clone());
            return Mark::BalNow;
        }
        Mark::Bal
    }

    /// `flows(x, span …)`: of a move (its leg named, or its one leg), or of a place
    /// (refcore's `m_t_flows`). L §5.14.3, §5.14.1.
    fn flows(&mut self, m: &Reality, scope: &Scope, flows: &Flows, depth: u32) -> Result<(), Stop> {
        if let Some((id, keys)) = move_keys(&flows.target)
            && !scope.is_var(id)
            && m.names.get(id) == Some(&NameKind::Move)
        {
            self.keys(m, scope, keys, depth)?;
            self.edge(&scope.item, Node::new(NodeKind::Move, id), Rel::Flows);
            if m.moves.get(id).is_some_and(|mv| mv.decision) {
                self.mark(&scope.item, Mark::Flows, Target::Move(id.to_owned()));
            }
        } else {
            self.expr(m, scope, &flows.target, depth)?;
            let target = place_target(m, scope, &flows.target);
            self.mark(&scope.item, Mark::Flows, target);
            self.edge(&scope.item, Node::balances(), Rel::Flows);
        }
        self.span(m, scope, &flows.span, depth)?;
        self.opt(m, scope, flows.for_key.as_ref(), depth)?;
        if let Some((_, rate, mode)) = &flows.convert {
            self.pair(m, scope, rate, mode, depth)?;
        }
        Ok(())
    }

    /// `covered(x, span)`: the place and the span (refcore's `m_c_covered`). L §5.19.
    fn covered(
        &mut self,
        m: &Reality,
        scope: &Scope,
        place: &Expr,
        span: &Span,
        depth: u32,
    ) -> Result<(), Stop> {
        self.expr(m, scope, place, depth)?;
        self.span(m, scope, span, depth)
    }

    /// `value(x, d) in CUR [by p] [of q]`: the place, the day, the prices its places are
    /// valued by, and a balance read (refcore's `m_t_value`, `m_price_edges`). L §5.10.
    fn value(
        &mut self,
        m: &Reality,
        scope: &Scope,
        value: &Valuation,
        depth: u32,
    ) -> Result<(), Stop> {
        self.pair(m, scope, &value.place, &value.day, depth)?;
        for price in price_inputs(m, scope, &value.place) {
            self.edge(&scope.item, price, Rel::Explicit);
        }
        if let Some(by) = &value.by {
            self.by(m, scope, by, &value.day, depth)?;
        }
        self.edge(&scope.item, Node::balances(), Rel::Bal);
        self.mark(&scope.item, Mark::Bal, place_target(m, scope, &value.place));
        self.of(m, scope, value.of.as_ref(), depth)
    }

    /// A value's `by p`: a keyed price input or def, or a reading through `last`, read at
    /// the value's day; any other price is an expression read as written (refcore's
    /// `m_t_value`, its `by` branches). L §5.10.
    fn by(
        &mut self,
        m: &Reality,
        scope: &Scope,
        by: &Price,
        day: &Expr,
        depth: u32,
    ) -> Result<(), Stop> {
        if let Expr::Name(id) = &by.input
            && !scope.is_var(id)
            && let Some((node, keyed)) = input_or_def(m, id)
            && (keyed || by.last)
        {
            self.edge(&scope.item, node, Rel::Explicit);
            return Ok(());
        }
        if by.last {
            return self.explicit(m, scope, &by.input, Some(day), depth);
        }
        self.expr(m, scope, &by.input, depth)
    }
}

/// How a read's keys relate to its reader's: only a def's reads relate; a read in a mode
/// takes the mode (refcore's `relation`). E §3.1 step 5.
fn relation(scope: &Scope, keys: &[Key]) -> Rel {
    if let Some(mode) = &scope.mode {
        return mode.clone();
    }
    if scope.item.kind != NodeKind::Def {
        return Rel::Any;
    }
    let bound_here = |key: &Key| {
        if let Key::Expr(Expr::Name(name)) = key {
            scope.is_var(name) && !scope.keyvars.contains(name)
        } else {
            false
        }
    };
    if scope.under == Under::Aggregate && keys.iter().any(bound_here) {
        return Rel::Computed;
    }
    let mut shift = None;
    for key in keys {
        match key_relation(scope, key) {
            KeyRel::Same => {}
            KeyRel::Shifted(var, way) if shift.is_none() => shift = Some(Rel::Shifted(var, way)),
            KeyRel::Shifted(..) | KeyRel::Computed => return Rel::Computed,
        }
    }
    shift.unwrap_or(Rel::Same)
}

/// How one key of a read relates to its reader's keys.
#[derive(Debug, Clone, PartialEq, Eq)]
enum KeyRel {
    /// A key variable of the reader.
    Same,
    /// A key variable shifted by a number.
    Shifted(String, Shift),
    /// Anything else.
    Computed,
}

/// One key's relation to the reader's keys (refcore's `relation`, its loop).
fn key_relation(scope: &Scope, key: &Key) -> KeyRel {
    let Key::Expr(expr) = key else {
        return KeyRel::Computed;
    };
    if let Expr::Name(name) = expr
        && scope.keyvars.contains(name)
    {
        return KeyRel::Same;
    }
    if let Expr::Sum(base, rest) = expr
        && let Expr::Name(name) = base.as_ref()
        && scope.keyvars.contains(name)
        && let [(op, Expr::Lit(Lit::Num(_)))] = rest.as_slice()
    {
        let way = match op {
            AddOp::Add => Shift::Later,
            AddOp::Sub => Shift::Earlier,
        };
        return KeyRel::Shifted(name.clone(), way);
    }
    KeyRel::Computed
}

/// A name with its postfixes, `x` or `x[k].f`.
fn named_parts(expr: &Expr) -> Option<(&str, &[Postfix])> {
    if let Expr::Name(id) = expr {
        return Some((id, &[]));
    }
    if let Expr::Post(base, posts) = expr
        && let Expr::Name(id) = base.as_ref()
    {
        return Some((id, posts));
    }
    None
}

/// A `flows` target written as a move's may be: `m`, `m[k]`, `m.leg` or `m[k].leg`, with
/// its name and keys.
fn move_keys(target: &Expr) -> Option<(&str, &[Key])> {
    let (id, posts) = named_parts(target)?;
    match posts {
        [] | [Postfix::Field(_)] => Some((id, &[])),
        [Postfix::Keys(keys)] | [Postfix::Keys(keys), Postfix::Field(_)] => Some((id, keys)),
        _ => None,
    }
}

/// The input or def `id`, and whether it is keyed.
fn input_or_def(m: &Reality, id: &str) -> Option<(Node, bool)> {
    match m.names.get(id)? {
        NameKind::Input => {
            let input = m.inputs.get(id)?;
            Some((Node::new(NodeKind::Input, id), !input.binds.is_empty()))
        }
        NameKind::Def => {
            let def = m.defs.get(id)?;
            Some((Node::new(NodeKind::Def, id), !def.binds.is_empty()))
        }
        NameKind::Unit
        | NameKind::Kind
        | NameKind::Index
        | NameKind::Axis
        | NameKind::Band
        | NameKind::Place
        | NameKind::Fill
        | NameKind::Move
        | NameKind::Version
        | NameKind::Ask
        | NameKind::Source
        | NameKind::Cut => None,
    }
}

/// The place section a thing comes from: its keyed place's, or the unkeyed place it is.
fn section_of(m: &Reality, thing: &Thing) -> Option<String> {
    if let Some(keys) = &thing.place {
        return Some(keys.place.clone());
    }
    let unkeyed = m.names.get(&thing.name) == Some(&NameKind::Place);
    unkeyed.then(|| thing.name.clone())
}

/// The thing a place expression names literally: an unkeyed place's id, a thing of a place
/// kind, or a keyed place at literal keys (refcore's `m_literal_place`). L §5.10.
fn literal_place(m: &Reality, scope: &Scope, expr: &Expr) -> Option<String> {
    let (id, posts) = named_parts(expr)?;
    if scope.is_var(id) {
        return None;
    }
    let named = m.names.get(id);
    if named.is_none() && posts.is_empty() {
        let thing = m.things.get(id)?;
        return m.is_place(thing).then(|| id.to_owned());
    }
    let place = m
        .places
        .get(id)
        .filter(|_| named == Some(&NameKind::Place))?;
    if posts.is_empty() {
        let unkeyed = place.things.iter().find(|(keys, _)| keys.is_empty());
        return unkeyed.map(|(_, thing)| thing.clone());
    }
    let [Postfix::Keys(keys)] = posts else {
        return None;
    };
    let mut lits = Vec::new();
    for key in keys {
        lits.push(key_literal(m, scope, key)?);
    }
    // A key no index or kind holds is refused by M0's checks (`LOAD-KEY`); here it names no
    // literal place.
    match keyed_place_thing(m, id, &lits, "", 0) {
        Ok(thing) => thing,
        Err(_) => None,
    }
}

/// A key written as a literal at a key position: a word that is no variable and no id, a
/// key, a day, a month, a month-day, a currency, or a whole number with no unit (refcore's
/// `is_keylit_node`). L §2.5.
fn key_literal(m: &Reality, scope: &Scope, key: &Key) -> Option<Lit> {
    let Key::Expr(expr) = key else {
        return None;
    };
    if let Expr::Name(name) = expr {
        let literal = !scope.is_var(name) && !m.names.contains_key(name);
        return literal.then(|| Lit::Name(name.clone()));
    }
    let Expr::Lit(lit) = expr else {
        return None;
    };
    let keylike = match lit {
        Lit::Key(_) | Lit::Day(_) | Lit::Month(..) | Lit::MonthDay(..) | Lit::Cur(_) => true,
        Lit::Num(number) => number.whole && number.unit.is_none(),
        Lit::Quarter(..) | Lit::Name(_) | Lit::Bool(_) | Lit::Inf { .. } | Lit::Nominal(_) => false,
    };
    keylike.then(|| lit.clone())
}

/// The keyed place section a place expression reads, `p` or `p[k]` (refcore's `_r` of
/// `place`).
fn place_section(m: &Reality, scope: &Scope, expr: &Expr) -> Option<String> {
    let (id, posts) = named_parts(expr)?;
    let read = posts.is_empty() || matches!(posts, [Postfix::Keys(_)]);
    let place = !scope.is_var(id) && m.names.get(id) == Some(&NameKind::Place);
    (read && place).then(|| id.to_owned())
}

/// What a running read or a declared read of a place names (refcore's `m_place_target`).
/// E §3.1 step 7.
fn place_target(m: &Reality, scope: &Scope, expr: &Expr) -> Target {
    if let Some(thing) = literal_place(m, scope, expr) {
        return Target::Thing(thing);
    }
    place_section(m, scope, expr).map_or(Target::Unknown, Target::PlaceSec)
}

/// What a leg's place changes: the place written, every place of a keyed place, every
/// place of the kind its type names, or a place load can't see (refcore's
/// `m_move_changes`). L §5.14.2.
fn change_of(m: &Reality, scope: &Scope, end: &Expr) -> Target {
    if let Some(thing) = literal_place(m, scope, end) {
        return Target::Thing(thing);
    }
    if let Some(section) = place_section(m, scope, end) {
        return Target::PlaceSec(section);
    }
    place_kind(m, scope, end).map_or(Target::Unknown, Target::Kind)
}

/// The kind a place expression's type names, where load sees it: a variable bound over a
/// kind, a place, a thing, or an input or def of type `key K` (refcore's `_t.ix`).
fn place_kind(m: &Reality, scope: &Scope, expr: &Expr) -> Option<String> {
    let mut inner = expr;
    while let Expr::Paren(nested) = inner {
        inner = nested;
    }
    let (id, posts) = named_parts(inner)?;
    if posts.len() > 1 || posts.iter().any(|post| matches!(post, Postfix::Field(_))) {
        return None;
    }
    if let Some(ranges) = scope.vars.get(id) {
        return match ranges {
            Ranges::Kind(kind) => Some(kind.clone()),
            Ranges::Keys => None,
        };
    }
    let named = m.names.get(id)?;
    let ty = match named {
        NameKind::Place => return m.places.get(id).map(|place| place.kind.clone()),
        NameKind::Input => m.inputs.get(id).and_then(|input| input.ty.as_ref()),
        NameKind::Def => m.defs.get(id).map(|def| &def.ty),
        NameKind::Unit
        | NameKind::Kind
        | NameKind::Index
        | NameKind::Axis
        | NameKind::Band
        | NameKind::Fill
        | NameKind::Move
        | NameKind::Version
        | NameKind::Ask
        | NameKind::Source
        | NameKind::Cut => None,
    };
    if let Some(Ty::Key(kind)) = ty
        && m.kinds.contains(kind)
    {
        return Some(kind.clone());
    }
    None
}

/// The price inputs a value of `place` reads: those of the place it names, or of every
/// thing of the kind its type names (refcore's `m_t_value`'s candidates, `m_price_edges`).
/// L §5.10.
fn price_inputs(m: &Reality, scope: &Scope, place: &Expr) -> Vec<Node> {
    let mut things: Vec<&Thing> = Vec::new();
    if let Some(name) = literal_place(m, scope, place) {
        things.extend(m.things.get(&name));
    } else if let Some(kind) = place_kind(m, scope, place) {
        let sub = m.subtree(&kind);
        for (_, thing) in m.things.entries() {
            if thing.base.as_ref().is_some_and(|base| sub.contains(base)) {
                things.push(thing);
            }
        }
    }
    let mut out = Vec::new();
    for thing in things {
        let Some(price) = m.price_of(thing) else {
            continue;
        };
        let Some((id, _)) = named_parts(&price.input) else {
            continue;
        };
        if let Some((node, _)) = input_or_def(m, id) {
            out.push(node);
        }
    }
    out
}

/// Adds `target` to `targets` unless it is there.
fn push_once(targets: &mut Vec<Target>, target: Target) {
    if !targets.contains(&target) {
        targets.push(target);
    }
}

#[cfg(test)]
mod tests {
    use super::{KeyRel, Ranges, Scope, Under, key_relation, relation};
    use crate::model::{Node, NodeKind, Rel, Shift};
    use crate::num::Num;
    use crate::syntax::{AddOp, Expr, Key, Lit, Number};

    fn def_scope() -> Scope {
        let mut scope = Scope::new(Node::new(NodeKind::Def, "d"));
        scope.vars.insert("k".to_owned(), Ranges::Keys);
        scope.keyvars.push("k".to_owned());
        scope
    }

    fn one() -> Expr {
        Expr::Lit(Lit::Num(Number {
            value: Num::int(1),
            whole: true,
            unit: None,
            text: "1".to_owned(),
            four: false,
            fraction: false,
        }))
    }

    #[test]
    fn a_def_relates_its_reads_keys_to_its_own() {
        let scope = def_scope();
        let at_k = Key::Expr(Expr::Name("k".to_owned()));
        let earlier = Key::Expr(Expr::Sum(
            Box::new(Expr::Name("k".to_owned())),
            vec![(AddOp::Sub, one())],
        ));
        assert_eq!(relation(&scope, &[]), Rel::Same, "no keys");
        assert_eq!(
            relation(&scope, core::slice::from_ref(&at_k)),
            Rel::Same,
            "at k"
        );
        assert_eq!(
            relation(&scope, core::slice::from_ref(&earlier)),
            Rel::Shifted("k".to_owned(), Shift::Earlier),
            "at k - 1"
        );
        assert_eq!(
            relation(&scope, &[earlier.clone(), earlier]),
            Rel::Computed,
            "two shifts"
        );
        assert_eq!(relation(&scope, &[Key::Star]), Rel::Computed, "a star");
        assert_eq!(key_relation(&scope, &at_k), KeyRel::Same, "at k itself");
        let mut under = scope.in_aggregate();
        under.vars.insert("x".to_owned(), Ranges::Keys);
        assert_eq!(under.under, Under::Aggregate, "under the aggregate");
        let bound = Key::Expr(Expr::Name("x".to_owned()));
        assert_eq!(
            relation(&under, &[bound]),
            Rel::Computed,
            "a bound variable"
        );
        assert_eq!(
            relation(
                &scope.with_mode(Rel::Explicit),
                core::slice::from_ref(&at_k)
            ),
            Rel::Explicit,
            "a mode wins"
        );
        let mover = Scope::new(Node::new(NodeKind::Move, "mv"));
        assert_eq!(
            relation(&mover, &[at_k]),
            Rel::Any,
            "a move relates nothing"
        );
    }
}
