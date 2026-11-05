//! Load's checker, its fills and `prorata`: each fill's sides, arc and order typed, a read of its one output, and a total shared pro rata.
//! Spec: L §5.12 (a fill: `supply` and `demand` share a unit, `arc` is a bool, `order` and
//! `ties by` over both sides' variables, no variable bound on both sides; its output
//! `F.taken[supply keys, demand keys]`, the fill's own keys first; its law dates are its
//! reader's), §5.6 (a fill's key bind is defined for every key), §8.3 (`LOAD-UNIT`,
//! `-SHADOW`, `-KEY`, `-PARSE`); §5.13 (`prorata`: a money total, one binding, a step of
//! the total's unit, `-LITERAL` in a rule but 1 with a unit, `minor(CUR)` or a row).
//! Twin: engine/tools/refcore/check2.py (`m_fill_scope`, `m_fill_sides`, `m_check_fill`,
//! `m_check_fill_body`, `m_fill_ref`, `m_fill_read_type`, `m_inline_fill_edges`,
//! `m_t_prorata`).
//! Invariants: each fill is typed once, in section order or first where a move reads its
//! output; a fill read inside its own supply stops "not built", as refcore does; a fill's
//! reads count as its reader's (the edges inlined after the walk, over an explicit stack).

use super::binds::BindAs;
use super::check::{Checker, Files, Scope, Stands, Var, VarDom};
use super::types::{Typed, converts, is_bool, numcls, refused, uerr, unit_of};
use crate::base::Load;
use crate::model::{Domain, Graph, Node, NodeKind, Reality, Rel, Shown, Ty, Unit};
use crate::num::Num;
use crate::syntax::{Bind, Expr, Key, Lit, Prorata, Rest, Step, Stop};
use std::collections::{BTreeMap, BTreeSet};

/// A fill's bindings as typed: its key binds' domains, its two sides' variables, its scope,
/// and the type of what it takes once its supply is typed (refcore's `fill_infos`).
#[derive(Debug, Clone)]
pub(super) struct FillSides {
    /// The domain of each key bind.
    fdoms: Vec<Domain>,
    /// The supply's variables, in the order bound.
    supply: Vec<Var>,
    /// The demand's variables, in the order bound.
    demand: Vec<Var>,
    /// The fill's own scope: its key variables.
    scope: Scope,
    /// The type of `taken`, once the supply is typed.
    taken: Option<Ty>,
}

/// The checker's fills: each one's sides, the fills typed and those being typed.
#[derive(Debug, Default)]
pub(super) struct FillState {
    /// Each fill's sides, typed once.
    sides: BTreeMap<String, FillSides>,
    /// The fills whose check has begun.
    done: BTreeSet<String>,
    /// The fills being typed now.
    busy: BTreeSet<String>,
    /// The file each fill's section stands in.
    files: BTreeMap<String, String>,
}

impl FillState {
    /// The checker's fills, before any is typed: each one's file.
    pub(super) fn of(m: &Reality, files: &Files) -> Result<FillState, Stop> {
        let mut out = FillState::default();
        for (id, _) in m.fills.entries() {
            out.files
                .insert(id.clone(), files.get("fill", id)?.to_owned());
        }
        Ok(out)
    }
}

/// The domain a key of a binding over `var`'s domain is read at: movements and observed rows
/// are read as any value, as a stepped range's (refcore's `key_pos` falls through for them).
fn key_domain(var: &Var) -> Domain {
    match &var.dom {
        VarDom::Index(ix) => Domain::Index(ix.clone()),
        VarDom::Kind(kind) => Domain::Kind(kind.clone()),
        VarDom::Axis(axis) => Domain::Axis(axis.clone()),
        VarDom::List => Domain::List(var.keylist.clone()),
        VarDom::Event(_) | VarDom::Obs(_) | VarDom::Range | VarDom::Arg => Domain::Range,
    }
}

impl Checker {
    /// Type fill `id`, once: in section order, or first where a move reads its output
    /// (refcore's `m_check_fill`). L §5.12.
    ///
    /// # Errors
    ///
    /// The fill's refusals, each at its line.
    pub(super) fn check_fill(&mut self, m: &Reality, id: &str) -> Result<(), Stop> {
        if !self.fill.done.insert(id.to_owned()) {
            return Ok(());
        }
        self.fill.busy.insert(id.to_owned());
        let body = self.check_fill_body(m, id);
        self.fill.busy.remove(id);
        body
    }

    /// A fill's bindings, bound once: its key binds over an index, a kind, a key list or a
    /// range, each defined for every key; each side's bindings in a scope of its own
    /// (refcore's `m_fill_sides`). L §5.12, §5.6.
    fn fill_sides(&mut self, m: &Reality, id: &str) -> Result<FillSides, Stop> {
        if let Some(sides) = self.fill.sides.get(id) {
            return Ok(sides.clone());
        }
        let fill = m
            .fills
            .get(id)
            .ok_or_else(|| super::check::invariant(format!("fill {id} isn't declared")))?;
        let file = self
            .fill
            .files
            .get(id)
            .cloned()
            .ok_or_else(|| super::check::invariant(format!("fill {id} has no section")))?;
        let mut scope = Scope::new(
            Node::new(NodeKind::Fill, id),
            Stands::Rule,
            &file,
            fill.line,
        );
        scope.keyvars = fill.binds.iter().map(|b| b.var.clone()).collect();
        let mut fdoms = Vec::new();
        for bind in &fill.binds {
            self.bind_as(m, &mut scope, bind, BindAs::KeyBind, 0)?;
            let var = bound(&scope, bind)?;
            if matches!(var.dom, VarDom::Event(_) | VarDom::Obs(_)) {
                return Err(Stop::NotBuilt("a keyed fill over movements"));
            }
            fdoms.push(key_domain(&var));
        }
        let mut sides = Vec::new();
        for (binds, line) in [
            (&fill.supply_binds, fill.lines.supply),
            (&fill.demand_binds, fill.lines.demand),
        ] {
            let mut side = scope.on_line(line);
            let mut vars = Vec::new();
            for bind in binds {
                self.bind(m, &mut side, bind, 0)?;
                vars.push(bound(&side, bind)?);
            }
            sides.push(vars);
        }
        let [supply, demand]: [Vec<Var>; 2] = sides
            .try_into()
            .map_err(|_| super::check::invariant(format!("fill {id} has two sides")))?;
        let made = FillSides {
            fdoms,
            supply,
            demand,
            scope,
            taken: None,
        };
        self.fill.sides.insert(id.to_owned(), made.clone());
        Ok(made)
    }

    /// A fill's supply and demand, numbers of one unit, then its arc (a bool), its order and
    /// its ties over both sides' variables, none bound on both sides (refcore's
    /// `m_check_fill_body`). L §5.12.
    fn check_fill_body(&mut self, m: &Reality, id: &str) -> Result<(), Stop> {
        let sides = self.fill_sides(m, id)?;
        let fill = m
            .fills
            .get(id)
            .ok_or_else(|| super::check::invariant(format!("fill {id} isn't declared")))?;
        let head = sides.scope.clone();
        let supply = with_vars(&head.on_line(fill.lines.supply), &sides.supply);
        let st = self.expr(m, &supply, &fill.supply, None, 0)?;
        let cs = numcls(m, &st);
        let unseen = matches!(st, Ty::Unseen);
        if !unseen && matches!(cs, None | Some("rate")) {
            let message = format!("fill {id}: the supply is {}", st.show());
            return Err(head.place(uerr(message)));
        }
        let taken = if unseen || matches!(cs, Some("money" | "num")) {
            st.clone()
        } else {
            Ty::Num(Unit::one(), Shown::Plain)
        };
        if let Some(made) = self.fill.sides.get_mut(id) {
            made.taken = Some(taken);
        }
        let demand = with_vars(&head.on_line(fill.lines.demand), &sides.demand);
        let dt = self.expr(m, &demand, &fill.demand, None, 0)?;
        let cd = numcls(m, &dt);
        let seen = !unseen && !matches!(dt, Ty::Unseen);
        if seen {
            let su = side_unit(&st, cs)?;
            let du = side_unit(&dt, cd)?;
            let dynamic = cs == Some("dyn") || cd == Some("dyn");
            let shared = dynamic || converts(&su, &du)?;
            if matches!(cd, None | Some("rate")) || !shared {
                let message = format!(
                    "fill {id}: supply ({}) and demand ({}) must share a unit (L §5.12)",
                    st.show(),
                    dt.show()
                );
                return Err(head.place(uerr(message)));
            }
        }
        let mut both = head.clone();
        for (var, line) in sides
            .supply
            .iter()
            .map(|v| (v, fill.lines.supply))
            .chain(sides.demand.iter().map(|v| (v, fill.lines.demand)))
        {
            if both.is_var(&var.name) {
                let message = format!(
                    "bound variable {} is bound on both sides of fill {id}",
                    var.name
                );
                return Err(head.on_line(line).place(refused(Load::Shadow, message)));
            }
            both.vars.insert(var.name.clone(), var.clone());
        }
        let arc_scope = both.on_line(fill.lines.arc);
        let at = self.expr(m, &arc_scope, &fill.arc, None, 0)?;
        if !is_bool(&at) {
            let message = format!("fill {id}: arc is {}, not bool", at.show());
            return Err(head.place(uerr(message)));
        }
        let order_scope = both.on_line(fill.lines.order);
        for e in fill.order.iter().chain(fill.ties.iter().flatten()) {
            self.expr(m, &order_scope, e, None, 0)?;
        }
        Ok(())
    }

    /// A read of fill `id`'s output, `F[fkeys].taken[keys]`: the fill's own keys, then its
    /// supply's keys and its demand's, each at its binding's domain; an edge to the fill;
    /// its type the supply's (refcore's `m_fill_read_type`). L §5.12.
    ///
    /// # Errors
    ///
    /// `LOAD-KEY` for a wrong number of keys, the keys' own refusals; "not built" for a fill
    /// read inside its own supply.
    pub(super) fn fill_read(
        &mut self,
        m: &Reality,
        scope: &Scope,
        id: &str,
        (fkeys, keys): (&[Key], &[Key]),
        depth: u32,
    ) -> Typed {
        let sides = self.fill_sides(m, id)?;
        if fkeys.len() != sides.fdoms.len() {
            return Err(refused(
                Load::Key,
                format!("wrong number of keys for fill {id}"),
            ));
        }
        let tdoms: Vec<Domain> = sides
            .supply
            .iter()
            .chain(&sides.demand)
            .map(key_domain)
            .collect();
        if keys.len() != tdoms.len() {
            let message = format!(
                "wrong number of keys for {id}.taken: {}, not {} (supply keys, then demand keys)",
                keys.len(),
                tdoms.len()
            );
            return Err(refused(Load::Key, message));
        }
        for (key, domain) in fkeys
            .iter()
            .zip(&sides.fdoms)
            .chain(keys.iter().zip(&tdoms))
        {
            let Key::Expr(kn) = key else {
                return Err(refused(
                    Load::Parse,
                    "cannot parse: * stands only in a pin".to_owned(),
                ));
            };
            self.key_pos(m, scope, kn, domain, depth)?;
        }
        let rel = if fkeys.is_empty() {
            match (&scope.mode, scope.item.kind) {
                (Some(mode), _) => mode.clone(),
                (None, NodeKind::Def | NodeKind::Fill) => Rel::Same,
                (None, _) => Rel::Any,
            }
        } else {
            super::check::relation(scope, fkeys)
        };
        let target = Node::new(NodeKind::Fill, id);
        self.shifted(scope, &target, &rel);
        self.edge(&scope.item, target, rel);
        let taken = self.fill.sides.get(id).and_then(|s| s.taken.clone());
        if taken.is_none() && !self.fill.busy.contains(id) {
            self.check_fill(m, id)?;
        }
        self.fill
            .sides
            .get(id)
            .and_then(|s| s.taken.clone())
            .ok_or(Stop::NotBuilt("a fill read inside its own supply"))
    }
}

/// The variable `bind` left in `scope`.
fn bound(scope: &Scope, bind: &Bind) -> Result<Var, Stop> {
    scope.vars.get(&bind.var).cloned().ok_or_else(|| {
        super::check::invariant(format!("the bound variable {} isn't in scope", bind.var))
    })
}

/// `scope` with `vars` in it.
fn with_vars(scope: &Scope, vars: &[Var]) -> Scope {
    let mut out = scope.clone();
    for var in vars {
        out.vars.insert(var.name.clone(), var.clone());
    }
    out
}

/// A side's unit: its own for money or a num, none for an int or a figure.
fn side_unit(t: &Ty, class: Option<&str>) -> Result<Unit, Stop> {
    if matches!(class, Some("money" | "num")) {
        unit_of(t)
    } else {
        Ok(Unit::one())
    }
}

/// A fill's terms are read at the law date of its reader: each read of a fill is also a read
/// of what the fill reads, through fills it reads in turn, a same-key or shifted read made
/// `computed` (refcore's `m_inline_fill_edges`). L §5.12 Law dates.
pub(super) fn inline_fill_edges(graph: &mut Graph) {
    let readers: Vec<Node> = graph
        .edges
        .keys()
        .filter(|r| r.kind != NodeKind::Fill)
        .cloned()
        .collect();
    for reader in readers {
        let Some(targets) = graph.edges.get(&reader) else {
            continue;
        };
        let fills: Vec<Node> = targets
            .keys()
            .filter(|t| t.kind == NodeKind::Fill)
            .cloned()
            .collect();
        for fill in fills {
            for (target, rels) in fill_reads(graph, &fill) {
                for rel in rels {
                    graph.edge(&reader, target.clone(), rel);
                }
            }
        }
    }
}

/// What fill `start` reads, through the fills it reads (each once), a same-key or shifted
/// read made `computed` (refcore's `fill_reads`).
fn fill_reads(graph: &Graph, start: &Node) -> BTreeMap<Node, Vec<Rel>> {
    let mut out: BTreeMap<Node, Vec<Rel>> = BTreeMap::new();
    let mut seen = BTreeSet::from([start.clone()]);
    let mut stack = vec![start.clone()];
    while let Some(fill) = stack.pop() {
        let Some(reads) = graph.edges.get(&fill) else {
            continue;
        };
        for (target, rels) in reads {
            if target.kind == NodeKind::Fill {
                if seen.insert(target.clone()) {
                    stack.push(target.clone());
                }
                continue;
            }
            let held = out.entry(target.clone()).or_insert_with(Vec::new);
            for rel in rels {
                let rel = if matches!(rel, Rel::Same | Rel::Shifted(..)) {
                    Rel::Computed
                } else {
                    rel.clone()
                };
                if !held.contains(&rel) {
                    held.push(rel);
                }
            }
        }
    }
    out
}

impl Checker {
    /// `prorata(total, w for k in D, key, u, MODE, rest none|last)` or `…, rest largest [ties
    /// by …])`: a money total shared by one binding's weights, the step of the total's unit,
    /// a whole share money (refcore's `t_prorata`). L §5.13.
    pub(super) fn prorata(&mut self, m: &Reality, scope: &Scope, p: &Prorata, depth: u32) -> Typed {
        let tt = self.expr(m, scope, &p.total, None, depth)?;
        if matches!(tt, Ty::Unseen) {
            return Ok(tt);
        }
        let c = numcls(m, &tt);
        let total_unit = unit_of(&tt)?;
        let cur = match total_unit.terms() {
            [(cur, 1)] if matches!(c, Some("money" | "num")) && total_unit.is_currency() => {
                cur.clone()
            }
            _ => {
                return Err(uerr(format!(
                    "prorata of {}: the total is money or a num CUR (L §5.13)",
                    tt.show()
                )));
            }
        };
        let [bind] = p.binds.as_slice() else {
            return Err(refused(
                Load::Parse,
                "cannot parse: prorata takes one binding".to_owned(),
            ));
        };
        let mut inner = scope.in_aggregate();
        self.bind(m, &mut inner, bind, depth)?;
        let wt = self.expr(m, &inner, &p.weight, None, depth)?;
        let weighs = matches!(numcls(m, &wt), Some("num" | "int" | "flex" | "money"));
        if !weighs && !matches!(wt, Ty::Unseen) {
            return Err(uerr(format!(
                "prorata: the weight is {}, not a number",
                wt.show()
            )));
        }
        let var = inner
            .vars
            .get(&bind.var)
            .cloned()
            .ok_or_else(|| super::check::invariant(format!("{} isn't bound", bind.var)))?;
        if let VarDom::Index(d) | VarDom::Kind(d) = &var.dom {
            let key = Ty::Key(d.clone());
            let kt = self.expr(m, scope, &p.key, Some(&key), depth)?;
            self.assign(m, scope, &key, &p.key, &kt, "the key of prorata")?;
        } else {
            self.key_pos(m, scope, &p.key, &key_domain(&var), depth)?;
        }
        let mut stepped = scope.child();
        stepped.allow_lit = scope.stands == Stands::Ask;
        let ut = match &p.unit {
            Step::Number(n) if n.unit.is_none() && !n.fraction => {
                self.expr(
                    m,
                    &stepped,
                    &Expr::Lit(Lit::Num(n.clone())),
                    Some(&tt),
                    depth,
                )?;
                Ty::Flex(n.value.clone())
            }
            Step::Number(n) => self.expr(
                m,
                &stepped,
                &Expr::Lit(Lit::Num(n.clone())),
                Some(&tt),
                depth,
            )?,
            Step::Minor(minor) => Ty::Money(minor.clone()),
            Step::Ref(e) => self.expr(m, &stepped, e, Some(&tt), depth)?,
        };
        let uc = numcls(m, &ut);
        if uc.is_none() && !matches!(ut, Ty::Unseen) {
            return Err(uerr(format!("prorata: the step is {}", ut.show())));
        }
        if matches!(scope.stands, Stands::Rule | Stands::At) {
            let ok = match &p.unit {
                Step::Number(n) => n.value == Num::int(1),
                Step::Minor(_) => true,
                Step::Ref(e) => super::types::input_read(m, scope, e).is_some(),
            };
            if !ok {
                return Err(refused(
                    Load::Literal,
                    "the step of prorata is 1 with a unit, minor(CUR) or a row".to_owned(),
                ));
            }
        }
        if uc != Some("flex") && !matches!(ut, Ty::Unseen) {
            let su = if matches!(uc, Some("num" | "money")) {
                unit_of(&ut)?
            } else {
                Unit::one()
            };
            if !converts(&su, &total_unit)? {
                return Err(uerr(format!(
                    "prorata: the step {} and the total {} have different units",
                    ut.show(),
                    tt.show()
                )));
            }
        }
        if matches!(p.rest, Rest::None | Rest::Last) {
            let mode = p
                .mode
                .as_ref()
                .ok_or_else(|| super::check::invariant("prorata with no mode".to_owned()))?;
            self.mode_arg(m, scope, "prorata", mode, depth)?;
        }
        for tie in p.ties.iter().flatten() {
            self.expr(m, &inner, tie, None, depth)?;
        }
        let whole = if let Ty::Flex(n) = &ut {
            let minor = m
                .currencies
                .get(&cur)
                .ok_or_else(|| super::check::invariant(format!("currency {cur} isn't declared")))?;
            n.checked_div(minor)
                .map_err(super::steps::num_stop)?
                .is_integer()
        } else {
            uc == Some("money")
        };
        if whole && (p.rest != Rest::Last || c == Some("money")) {
            return Ok(Ty::Money(cur));
        }
        Ok(Ty::Num(total_unit, Shown::Plain))
    }
}
