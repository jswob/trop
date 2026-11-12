//! Pins and an ask's axes: each pin checked, and the axes an ask reaches less those its pins fix, capped.
//! Spec: L §5.5 (axes: an ask reaches an axis through what it reads; its options fork the
//! answer; at most 64 combinations, `LOAD-FORKS`), §5.11 (`as if` and its pins), §5.18 (an
//! ask's `with` pins), §5.16 Pins (a sample pin), §6.4 (a pin's scenario), §3.2 and §7.1 (a
//! bare figure in a pin), §8.3 (`LOAD-ASIF`, `-SCENARIO`, `-KEY`, `-FIELD`, `-FORKS`).
//! Twin: engine/tools/refcore/check.py (`pin`, `pin_literal`, `check_graph`'s forks,
//! `reach_axes`); check2.py (`m_sample_pin`).
//! Invariants: the search is a loop over an explicit stack (R-16); the count of combinations
//! is exact (no cap on its own arithmetic before it is compared); a pin's checks run in
//! refcore's order, each refusal at the pin's line.

use super::check::{Checker, Figure, Files, Scope, Stands};
use super::random::sample_type;
use super::steps::refuse;
use super::texts::text;
use super::types::{is_date, lit_rule, numcls, refused, uerr};
use crate::base::{Code, Load, Reach, Refusal};
use crate::model::{BindInfo, Graph, NameKind, Node, NodeKind, Reality, Rel, Ty};
use crate::num::{Num, NumError};
use crate::syntax::{Expr, Key, Lit, PRef, Pin, PinForm, Stop};
use std::collections::{BTreeMap, BTreeSet};

/// The most axis combinations one ask may reach. L §5.5.
const FORKS_CAP: i64 = 64;

/// Each ask's axes, less those its pins fix; an ask that reaches more than 64 combinations
/// of options is `LOAD-FORKS` (refcore's `check_graph`, its forks). L §5.5.
///
/// # Errors
///
/// `LOAD-FORKS` at the first ask, in section order, past the cap.
pub(super) fn check_forks(
    m: &Reality,
    graph: &Graph,
    files: &Files,
) -> Result<BTreeMap<String, BTreeSet<String>>, Stop> {
    let mut out = BTreeMap::new();
    for ask in m.asks.iter().filter(|a| !m.asks_alone.contains_key(&a.id)) {
        let pinned: BTreeSet<&str> = ask
            .pins
            .iter()
            .filter_map(|pin| match &pin.form {
                PinForm::Set { target, .. } if m.axes.contains(&target.id) => {
                    Some(target.id.as_str())
                }
                PinForm::Set { .. }
                | PinForm::Law(_)
                | PinForm::Known(_)
                | PinForm::Wording(..)
                | PinForm::Scenario(_) => None,
            })
            .collect();
        let reached = reach_axes(m, graph, &Node::new(NodeKind::Ask, &ask.id));
        let forked: BTreeSet<String> = reached
            .into_iter()
            .filter(|axis| !pinned.contains(axis.as_str()))
            .collect();
        let mut n = Num::int(1);
        for axis in &forked {
            let options = m
                .axes
                .get(axis)
                .ok_or_else(|| super::check::invariant(format!("axis {axis} isn't declared")))?
                .options
                .len();
            let count = i64::try_from(options).map_err(|_| too_many(NumError::Digits))?;
            n = n.checked_mul(&Num::int(count)).map_err(too_many)?;
        }
        let over = n.compare(&Num::int(FORKS_CAP)).map_err(too_many)?.is_gt();
        if over {
            // refcore's `%d`: the whole count in plain digits.
            let shown = n.integer_text().ok_or_else(|| {
                super::check::invariant("a count of axis combinations, not whole".to_owned())
            })?;
            let message = format!(
                "ask {} reaches {shown} axis combinations; the cap is 64",
                ask.id
            );
            return Err(refuse(
                Load::Forks,
                message,
                files.get("ask", &ask.id)?,
                ask.line,
            ));
        }
        out.insert(ask.id.clone(), forked);
    }
    Ok(out)
}

/// A count of axis combinations past the number kernel's digit cap, which refcore prints whole
/// in its `LOAD-FORKS`: a capacity limit, "not built" until issue #89 rules; the kernel's own
/// invariant stays `ENGINE-INVARIANT`. L §5.5.
fn too_many(error: NumError) -> Stop {
    match error {
        NumError::Invariant => super::check::invariant(
            "the number kernel's invariant, counting combinations".to_owned(),
        ),
        NumError::Div | NumError::Digits | NumError::Domain => {
            Stop::NotBuilt("axis combinations past 4096 digits")
        }
    }
}

/// The axes `start` reaches: those each item on the way reads directly, and the axes it
/// reads by id; a read of the balances reaches every move and what the parts clauses read
/// (refcore's `reach_axes`). L §5.5; E §3.1 step 5, §3.3.
pub(crate) fn reach_axes(m: &Reality, graph: &Graph, start: &Node) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut stack = vec![start.clone()];
    let mut out = BTreeSet::new();
    while let Some(x) = stack.pop() {
        if !seen.insert(x.clone()) {
            continue;
        }
        if let Some(axes) = graph.axes_of.get(&x) {
            out.extend(axes.iter().cloned());
        }
        if x.kind == NodeKind::Axis {
            out.insert(x.id.clone());
        }
        if let Some(reads) = graph.edges.get(&x) {
            stack.extend(reads.keys().cloned());
        }
        if x.kind == NodeKind::Balances {
            for (id, _) in m.moves.entries() {
                stack.push(Node::new(NodeKind::Move, id));
            }
            stack.extend(graph.parts_reads.iter().cloned());
            for kind in &graph.parts_kinds {
                stack.push(Node::new(NodeKind::Parts, kind));
            }
        }
    }
    out
}

impl Checker {
    /// A pin of an ask or an `as if`: `law` and `known` a date; `wording e for [ids]` a date
    /// and known ids; `scenario s` a world of the run; `x[keys].f = v` an axis's option, or
    /// an input's or a def's value at keys, of its type (refcore's `pin`). L §5.11, §5.18,
    /// §6.4, §7.1.
    ///
    /// # Errors
    ///
    /// `LOAD-ASIF`, `LOAD-SCENARIO`, `LOAD-KEY`, `LOAD-FIELD`, `LOAD-UNIT`, `LOAD-LITERAL`,
    /// `LOAD-PRECISION`, at the pin's line; a member's pin stops "not built".
    pub(super) fn pin(
        &mut self,
        m: &Reality,
        scope: &Scope,
        pin: &Pin,
        depth: u32,
    ) -> Result<(), Stop> {
        let mut sc = scope.clone();
        if scope.stands == Stands::Ask {
            sc.allow_lit = true;
        }
        match &pin.form {
            PinForm::Law(value) | PinForm::Known(value) => {
                let t = self.expr(m, &sc, value, None, depth)?;
                if !is_date(m, &t) {
                    let word = if matches!(pin.form, PinForm::Law(_)) {
                        "law"
                    } else {
                        "known"
                    };
                    let message = format!("pin {word} takes a date, not {}", t.show());
                    return Err(sc.place(uerr(message)));
                }
                Ok(())
            }
            PinForm::Wording(value, ids) => {
                let t = self.expr(m, &sc, value, None, depth)?;
                if !is_date(m, &t) {
                    return Err(sc.place(uerr("wording takes a date".to_owned())));
                }
                for id in ids {
                    if id.contains('.') {
                        return Err(Stop::NotBuilt("a member's id in a wording pin"));
                    }
                    if !m.names.contains_key(id) {
                        return Err(sc.place(unknown(id)));
                    }
                }
                Ok(())
            }
            PinForm::Scenario(world) => {
                if m.all_worlds.contains(world) {
                    return Ok(());
                }
                if scope.stands == Stands::Ask && scope.item.kind == NodeKind::Ask {
                    return Err(sc.place(unloaded_world(&scope.item.id, world)));
                }
                Err(sc.place(unknown(world)))
            }
            PinForm::Set {
                target,
                value,
                sample,
            } => {
                if *sample {
                    return self.sample_pin(m, &sc, (target, value), depth);
                }
                self.set_pin(m, &sc, (target, value), depth)
            }
        }
    }

    /// `sample x[keys] = v`, `sample x[*] = e`, `sample x[keys].u = u`: a random fact's
    /// sample at its addresses, of its sample type, or its uniform, a plain number (refcore's
    /// `sample_pin`). L §5.16 Pins.
    fn sample_pin(
        &mut self,
        m: &Reality,
        sc: &Scope,
        (target, value): (&PRef, &Expr),
        depth: u32,
    ) -> Result<(), Stop> {
        if target.qual.is_some() {
            return Err(Stop::NotBuilt("a member's pin"));
        }
        let name = &target.id;
        let Some(kind) = m.names.kind(name) else {
            return Err(sc.place(unknown(name)));
        };
        let fact = if kind == NameKind::Input {
            m.inputs
                .get(name)
                .and_then(|input| Some((&input.binds, input.ty.clone()?, NodeKind::Input)))
        } else if kind == NameKind::Def {
            m.defs
                .get(name)
                .map(|def| (&def.binds, def.ty.clone(), NodeKind::Def))
        } else {
            None
        };
        let Some((binds, ty, node)) =
            fact.filter(|(_, ty, _)| matches!(ty, Ty::Dist(_) | Ty::DistKeys(_)))
        else {
            let message = format!(
                "sample {name}: a sample pin names a random fact (an input or def of type dist)"
            );
            return Err(sc.place(uerr(message)));
        };
        let uniform = !target.fields.is_empty();
        if uniform && target.fields != ["u"] {
            let message = format!(
                "{} is not a field of a sample (u is)",
                target.fields.join(".")
            );
            return Err(sc.place(refused(Load::Field, message)));
        }
        self.pin_keys(m, sc, target, binds, "address", depth)?;
        let pinned = Node::new(node, name);
        if uniform {
            let t = self.expr(m, sc, value, None, depth)?;
            let united = matches!(&t, Ty::Num(unit, _) if !unit.is_one());
            if !matches!(numcls(m, &t), Some("num" | "flex" | "int")) || united {
                let message = format!("sample {name}.u is {}, not a plain number", t.show());
                return Err(sc.place(uerr(message)));
            }
        } else {
            let st = sample_type(m, &pinned, &ty).map_err(|stop| sc.place(stop))?;
            if !pin_literal(m, sc, value, &st)? {
                let t = self.expr(m, sc, value, Some(&st), depth)?;
                let what = format!("the sample pin of {name}");
                self.assign(m, sc, &st, value, &t, &what)
                    .map_err(|stop| sc.place(stop))?;
            }
        }
        self.edge(&sc.item, pinned, Rel::Pin);
        Ok(())
    }

    /// A pin's keys against its target's bindings: as many as it binds, or none and `[*]`
    /// asked for, each typed at its position (refcore's `pin` and `sample_pin`, their key
    /// checks); `each` names what one key pins, an `address` or a `key`. L §5.11, §5.16.
    fn pin_keys(
        &mut self,
        m: &Reality,
        sc: &Scope,
        target: &PRef,
        binds: &[BindInfo],
        each: &str,
        depth: u32,
    ) -> Result<(), Stop> {
        let name = &target.id;
        match &target.keys {
            Some(keys) if keys.len() != binds.len() => {
                let message = format!("wrong number of keys for {name}");
                return Err(sc.place(refused(Load::Key, message)));
            }
            None if !binds.is_empty() => {
                let message =
                    format!("wrong number of keys for {name}: write [*] to pin every {each}");
                return Err(sc.place(refused(Load::Key, message)));
            }
            Some(_) | None => {}
        }
        for (key, bind) in target.keys.iter().flatten().zip(binds) {
            if let Key::Expr(kn) = key {
                self.key_pos(m, sc, kn, &bind.domain, depth)?;
            }
        }
        Ok(())
    }

    /// `x[keys].f = v`: an axis's option, or an input's or a def's value (refcore's `pin`,
    /// its target arm). L §5.11, §5.5.
    fn set_pin(
        &mut self,
        m: &Reality,
        sc: &Scope,
        (target, value): (&PRef, &Expr),
        depth: u32,
    ) -> Result<(), Stop> {
        if target.qual.is_some() {
            return Err(Stop::NotBuilt("a member's pin"));
        }
        let name = &target.id;
        let Some(kind) = m.names.kind(name) else {
            return Err(sc.place(unknown(name)));
        };
        if kind == NameKind::Axis {
            return axis_pin(m, sc, target, value);
        }
        let (binds, declared, node) = match kind {
            NameKind::Input => {
                let input = m.inputs.at(m.names.input_id(name)?)?;
                (&input.binds, input.ty.clone(), NodeKind::Input)
            }
            NameKind::Def => {
                let def = m.defs.at(m.names.def_id(name)?)?;
                (&def.binds, Some(def.ty.clone()), NodeKind::Def)
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
            | NameKind::Cut => {
                let message = format!("pin on {name}, which is not an input, a def or an axis");
                return Err(sc.place(refused(Load::AsIf, message)));
            }
        };
        self.pin_keys(m, sc, target, binds, "key", depth)?;
        let mut ty = declared.unwrap_or(Ty::Unseen);
        for field in &target.fields {
            let Some(inner) = ty.field(field).cloned() else {
                let message = format!("{field} is not a field of {}", ty.show());
                return Err(sc.place(refused(Load::Field, message)));
            };
            ty = inner;
        }
        let pinned = Node::new(node, name);
        if !pin_literal(m, sc, value, &ty)? {
            if matches!(ty, Ty::Dist(_)) {
                self.expr(m, sc, value, None, depth)?;
            } else {
                let t = self.expr(m, sc, value, Some(&ty), depth)?;
                let what = format!("the pin of {name}");
                self.assign(m, sc, &ty, value, &t, &what)
                    .map_err(|stop| sc.place(stop))?;
            }
        }
        self.edge(&sc.item, pinned, Rel::Pin);
        Ok(())
    }
}

/// An axis pin, `ax = option`: no keys or fields, an option of the axis (refcore's `pin`,
/// its axis arm). L §5.5.
fn axis_pin(m: &Reality, sc: &Scope, target: &PRef, value: &Expr) -> Result<(), Stop> {
    let name = &target.id;
    if target.keys.is_some() || !target.fields.is_empty() {
        let message = "an axis pin names its option".to_owned();
        return Err(sc.place(refused(Load::AsIf, message)));
    }
    let option = if let Expr::Name(x) | Expr::Lit(Lit::Key(x)) = value {
        Some(x)
    } else {
        None
    };
    let named = option.is_some_and(|x| super::keys::is_option(m, name, x));
    if !named {
        let message = format!("{} is not a key of {name}", text(value));
        return Err(sc.place(refused(Load::Key, message)));
    }
    Ok(())
}

/// A bare figure in a pin takes its target's declared type and unit: the literal rule, a
/// fraction only in rows, a rate states its period, an int is whole, money no finer than its
/// minor unit (refcore's `pin_literal`). L §3.2 Inline figures, §7.1.
pub(super) fn pin_literal(m: &Reality, sc: &Scope, node: &Expr, ty: &Ty) -> Result<bool, Stop> {
    if !matches!(ty, Ty::Money(_) | Ty::Num(..) | Ty::Int | Ty::Rate) {
        return Ok(false);
    }
    let (inner, negative) = bare(node);
    let Expr::Lit(Lit::Num(n)) = inner else {
        return Ok(false);
    };
    if n.unit.is_some() {
        return Ok(false);
    }
    if n.fraction {
        let message = "cannot parse: a fraction is allowed only in rows".to_owned();
        return Err(sc.place(refused(Load::Parse, message)));
    }
    let v = if negative {
        n.value.checked_neg().map_err(super::steps::num_stop)?
    } else {
        n.value.clone()
    };
    lit_rule(sc, Figure::Num, &n.text, Some(&v)).map_err(|stop| sc.place(stop))?;
    if *ty == Ty::Rate {
        let message = "a bare figure as a rate: a rate states its period (L §3.3)".to_owned();
        return Err(sc.place(uerr(message)));
    }
    if *ty == Ty::Int && !v.is_integer() {
        return Err(sc.place(uerr(format!("{} is not an int", text(node)))));
    }
    if let Ty::Money(cur) = ty {
        let minor = m
            .currencies
            .get(cur)
            .ok_or_else(|| super::check::invariant(format!("currency {cur} isn't declared")))?;
        let units = v.checked_div(minor).map_err(super::steps::num_stop)?;
        if !units.is_integer() {
            let message = format!("{} is finer than the minor unit", text(node));
            return Err(sc.place(refused(Load::Precision, message)));
        }
    }
    Ok(true)
}

/// An expression under its parentheses and minus signs, and whether the signs negate it.
pub(super) fn bare(node: &Expr) -> (&Expr, bool) {
    let mut inner = node;
    let mut negative = false;
    loop {
        if let Expr::Paren(e) = inner {
            inner = e;
        } else if let Expr::Neg(e) = inner {
            negative = !negative;
            inner = e;
        } else {
            return (inner, negative);
        }
    }
}

/// `LOAD-ASIF pin on unknown id <x>`. L §8.3.
fn unknown(id: &str) -> Stop {
    refused(Load::AsIf, format!("pin on unknown id {id}"))
}

/// `LOAD-SCENARIO ask <q> names <w>, neither the model nor a loaded scenario`: it refuses only
/// that ask (L §6.4, §8.3; refcore's `check_ask`, `pin`).
pub(super) fn unloaded_world(ask: &str, world: &str) -> Stop {
    let message = format!("ask {ask} names {world}, neither the model nor a loaded scenario");
    Stop::Refused(Refusal::new(Code::Load(Load::Scenario), message).reaching(Reach::UnloadedWorld))
}
