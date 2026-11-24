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
use super::keys::index_key_of;
use super::random::sample_type;
use super::steps::refuse;
use super::texts::text;
use super::types::{is_date, lit_rule, numcls, refused, uerr};
use crate::base::{Code, Load, Reach, Refusal};
use crate::eval::{ConvertFor, convert};
use crate::model::{BindInfo, Graph, NameKind, Node, NodeKind, Reality, Rel, Ty};
use crate::num::{Num, NumError};
use crate::syntax::{Ask, Expr, Key, Lit, PRef, Pin, PinForm, Stop, Value as Written};
use crate::time::PeriodKind;
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
        let mut counts = Vec::with_capacity(forked.len());
        for axis in &forked {
            let options = m
                .axes
                .get(axis)
                .ok_or_else(|| super::check::invariant(format!("axis {axis} isn't declared")))?
                .options
                .len();
            counts.push(options);
        }
        if let Some(message) = forks_past_cap(&ask.id, &counts)? {
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

/// `LOAD-FORKS`'s message for ask `id` whose axes have `counts` options each, where their
/// combinations pass 64, counted exactly; none within the cap (refcore's `check_graph`, its
/// forks). L §5.5 Refusals.
///
/// # Errors
///
/// `NotBuilt` for a count past the number kernel's digits.
pub(super) fn forks_past_cap(id: &str, counts: &[usize]) -> Result<Option<String>, Stop> {
    let mut n = Num::int(1);
    for options in counts {
        let count = i64::try_from(*options).map_err(|_| too_many(NumError::Digits))?;
        n = n.checked_mul(&Num::int(count)).map_err(too_many)?;
    }
    if !n.compare(&Num::int(FORKS_CAP)).map_err(too_many)?.is_gt() {
        return Ok(None);
    }
    // refcore's `%d`: the whole count in plain digits.
    let shown = n.integer_text().ok_or_else(|| {
        super::check::invariant("a count of axis combinations, not whole".to_owned())
    })?;
    Ok(Some(format!(
        "ask {id} reaches {shown} axis combinations; the cap is 64"
    )))
}

/// Each ask's file, for a refusal at its line met once every world is loaded: a base diff,
/// whose cone is both its worlds' (`LOAD-FORKS`). L §5.5, §6.4.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for an ask with no section.
pub(super) fn ask_files(m: &Reality, files: &Files) -> Result<BTreeMap<String, String>, Stop> {
    let mut out = BTreeMap::new();
    for ask in &m.asks {
        out.insert(ask.id.clone(), files.get("ask", &ask.id)?.to_owned());
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
            let st = sample_type(m, &self.files, &pinned, &ty).map_err(|stop| sc.place(stop))?;
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
                self.bind_key(m, sc, kn, bind, depth)?;
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
        let (binds, declared, node, factor) = match kind {
            NameKind::Input => {
                let input = m.inputs.at(m.names.input_id(name)?)?;
                if input.builtin && (name == "opening" || name == "stated") {
                    // A pin has no instant, and a balance reading anchors the walk at its
                    // own (L §8.3, S-231.Q2): a what-if balance is a scenario's row.
                    let message = format!("a pin of balance reading {name} states no instant");
                    return Err(sc.place(refused(Load::Instant, message)));
                }
                let factor = &input.unit_factor;
                (&input.binds, input.ty.clone(), NodeKind::Input, factor)
            }
            NameKind::Def => {
                let def = m.defs.at(m.names.def_id(name)?)?;
                let factor = &def.unit_factor;
                (&def.binds, Some(def.ty.clone()), NodeKind::Def, factor)
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
        pin_key(m, value, &ty, name).map_err(|stop| sc.place(stop))?;
        if !pin_literal(m, sc, value, &ty)? {
            let one = Num::int(1);
            let factor = if target.fields.is_empty() {
                factor
            } else {
                &one
            };
            self.typed_pin(m, sc, (name, value), (&ty, factor), depth)?;
        }
        self.edge(&sc.item, pinned, Rel::Pin);
        Ok(())
    }

    /// A pin's value typed at its target's type `ty`, whose unit brings `factor`, and assigned
    /// to it; a literal it can't take is refused in its row's words (refcore's `pin`, its `tc`
    /// and `assign`, after `pin_row`; G3's review N1). L §5.11 A pin is a row, §3.2.
    fn typed_pin(
        &mut self,
        m: &Reality,
        sc: &Scope,
        (name, value): (&str, &Expr),
        (ty, factor): (&Ty, &Num),
        depth: u32,
    ) -> Result<(), Stop> {
        if matches!(ty, Ty::Dist(_)) {
            self.expr(m, sc, value, None, depth)?;
            return Ok(());
        }
        let t = self.expr(m, sc, value, Some(ty), depth)?;
        let what = format!("the pin of {name}");
        let Err(stop) = self.assign(m, sc, ty, value, &t, &what) else {
            return Ok(());
        };
        let words = row_refusal(m, value, ty, name, factor)?;
        Err(sc.place(words.unwrap_or(stop)))
    }
}

/// A bare figure, month, quarter or day pinned on a key of an index is read as the same
/// literal written in a row of it: one the index lacks is refused `LOAD-KEY`, a four-digit
/// figure in refcore's words, the rest in the row's (refcore's `assign`, its key arm; P16).
/// L §3.2 Inline figures, §2.5.
fn pin_key(m: &Reality, node: &Expr, ty: &Ty, name: &str) -> Result<(), Stop> {
    let (Ty::Key(id), Expr::Lit(lit)) = (ty, node) else {
        return Ok(());
    };
    let read_as_key = matches!(lit, Lit::Month(..) | Lit::Quarter(..) | Lit::Day(_))
        || matches!(lit, Lit::Num(n) if n.whole && n.unit.is_none());
    if !read_as_key || m.names.kind(id) != Some(NameKind::Index) {
        return Ok(());
    }
    let index = m.indexes.at(m.names.index_id(id)?)?;
    if index_key_of(index, lit).is_some() {
        return Ok(());
    }
    let four = matches!(lit, Lit::Num(n) if n.four);
    let period = match lit {
        Lit::Month(..) => Some(PeriodKind::Month),
        Lit::Quarter(..) => Some(PeriodKind::Quarter),
        Lit::Day(_) => Some(PeriodKind::Day),
        Lit::Num(_)
        | Lit::MonthDay(..)
        | Lit::Key(_)
        | Lit::Cur(_)
        | Lit::Bool(_)
        | Lit::Inf { .. }
        | Lit::Nominal(_)
        | Lit::Name(_) => None,
    };
    let message = if four && !index.is_int {
        format!("the pin of {name} is year4, not a key of {id}")
    } else if (index.is_int && period.is_none()) || (period.is_some() && index.period == period) {
        format!("{} is not a key of {id}", text(node))
    } else {
        return Ok(());
    };
    Err(refused(Load::Key, message))
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

/// The refusal the row of a pin's value meets where that value is one literal (a figure,
/// signed or in brackets; a date, a month or a quarter, in brackets) on a target of type
/// `ty`, whose unit brings `factor`: the row reader's own words (refcore's `pin_row`, its
/// `convert_value`; G3's review N1). None for any other value, a record, a distribution or a
/// unit keyed by `$v`, and where the row stands. L §5.11 A pin is a row, §3.2, §8.3.
fn row_refusal(
    m: &Reality,
    value: &Expr,
    ty: &Ty,
    name: &str,
    factor: &Num,
) -> Result<Option<Stop>, Stop> {
    let keyed = matches!(ty, Ty::Num(unit, _)
        if unit.terms().iter().any(|(base, _)| base.starts_with('$')));
    if keyed || matches!(ty, Ty::Dist(_) | Ty::Record(_)) {
        return Ok(None);
    }
    let (inner, negative) = bare(value);
    let Expr::Lit(lit) = inner else {
        return Ok(None);
    };
    let row = match lit {
        Lit::Num(n) if negative => {
            let mut signed = n.clone();
            signed.value = n.value.checked_neg().map_err(super::steps::num_stop)?;
            signed.text = format!("-{}", n.text);
            Lit::Num(signed)
        }
        Lit::Num(_) | Lit::Day(_) | Lit::Month(..) | Lit::Quarter(..) if !negative => lit.clone(),
        Lit::Num(_)
        | Lit::Day(_)
        | Lit::Month(..)
        | Lit::Quarter(..)
        | Lit::MonthDay(..)
        | Lit::Key(_)
        | Lit::Cur(_)
        | Lit::Bool(_)
        | Lit::Inf { .. }
        | Lit::Nominal(_)
        | Lit::Name(_) => return Ok(None),
    };
    let of = ConvertFor {
        factor: factor.clone(),
        keys: Vec::new(),
    };
    match convert(m, &Written::Lit(row), ty, name, &of) {
        Err(stop @ Stop::Refused(_)) => Ok(Some(stop)),
        Ok(_) | Err(Stop::NotBuilt(_) | Stop::Deep) => Ok(None),
    }
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

/// Two value pins of one set on one cell, of one width, tie in L §4.6's cascade: refused at
/// load where load sees both pins' keys, written alike with no variable of the scope in them,
/// `LOAD-OVERLAP pins <x>[k] = a and <x>[k] = b of <x> overlap at [k]`; pins whose keys only
/// the run gives are the cell's, `RUN-CONFLICT` (refcore's `pin_ties`; tally 2.1.7 R4).
/// L §4.6, §8.3.
///
/// # Errors
///
/// That `LOAD-OVERLAP`, unplaced.
pub(super) fn pin_ties(m: &Reality, scope: &Scope, pins: &[Pin]) -> Result<(), Stop> {
    let sets: Vec<(&PRef, &Expr)> = pins
        .iter()
        .filter_map(|pin| match &pin.form {
            PinForm::Set {
                target,
                value,
                sample: false,
            } if m.names.kind(&target.id) != Some(NameKind::Axis) => Some((target, value)),
            PinForm::Set { .. }
            | PinForm::Law(_)
            | PinForm::Known(_)
            | PinForm::Wording(..)
            | PinForm::Scenario(_) => None,
        })
        .collect();
    let mut rest = sets.as_slice();
    while let [(p, pv), tail @ ..] = rest {
        for (q, qv) in tail {
            if p.id != q.id || p.qual != q.qual || p.fields != q.fields {
                continue;
            }
            let (Some(pk), Some(qk)) = (seen_keys(scope, p), seen_keys(scope, q)) else {
                continue;
            };
            if pk != qk {
                continue;
            }
            let ks = pk.join(", ");
            let mut cell = p.id.clone();
            if !pk.is_empty() {
                cell = format!("{cell}[{ks}]");
            }
            for field in &p.fields {
                cell = format!("{cell}.{field}");
            }
            return Err(refused(
                Load::Overlap,
                format!(
                    "pins {cell} = {} and {cell} = {} of {} overlap at [{ks}]",
                    text(pv),
                    text(qv),
                    p.id
                ),
            ));
        }
        rest = tail;
    }
    Ok(())
}

/// A line's `law D` and `as known D`, as a clause or a `with` pin, are its pins of the law date
/// and the cut: two of one on ask `ask`'s line are two pins of one cell, `LOAD-OVERLAP pins
/// <a> and <b> of the law date overlap at no keys` (`of the cut`), its `with` pins first, then
/// its clause (refcore's `check_ask`; tally 2.3.1 R3, N245.4). L §5.11 A pin is a row, §4.6,
/// §8.3.
///
/// # Errors
///
/// That `LOAD-OVERLAP`, unplaced.
pub(super) fn line_pins(ask: &Ask) -> Result<(), Stop> {
    for law in [true, false] {
        let mut same: Vec<String> = ask
            .pins
            .iter()
            .filter_map(|pin| match &pin.form {
                PinForm::Law(e) if law => Some(format!("law {}", text(e))),
                PinForm::Known(e) if !law => Some(format!("known {}", text(e))),
                PinForm::Law(_)
                | PinForm::Known(_)
                | PinForm::Wording(..)
                | PinForm::Scenario(_)
                | PinForm::Set { .. } => None,
            })
            .collect();
        let (clause, word, cell) = if law {
            (&ask.law, "law", "the law date")
        } else {
            (&ask.known, "as known", "the cut")
        };
        same.extend(clause.iter().map(|e| format!("{word} {}", text(e))));
        if let [first, second, ..] = same.as_slice() {
            return Err(refused(
                Load::Overlap,
                format!("pins {first} and {second} of {cell} overlap at no keys"),
            ));
        }
    }
    Ok(())
}

/// A pin's keys as written, when load sees them: none where one reads a variable of the
/// scope, which the run gives (refcore's `pin_ties`, its `seen`). L §4.6.
fn seen_keys(scope: &Scope, target: &PRef) -> Option<Vec<String>> {
    let Some(keys) = &target.keys else {
        return Some(Vec::new());
    };
    let mut out = Vec::with_capacity(keys.len());
    for key in keys {
        match key {
            Key::Star => out.push("*".to_owned()),
            Key::Expr(e) => {
                let names = crate::eval::names_in(e)?;
                if names.iter().any(|n| scope.is_var(n)) {
                    return None;
                }
                out.push(text(e));
            }
        }
    }
    Some(out)
}

/// `LOAD-SCENARIO ask <q> names <w>, neither the model nor a loaded scenario`: it refuses only
/// that ask (L §6.4, §8.3; refcore's `check_ask`, `pin`).
pub(super) fn unloaded_world(ask: &str, world: &str) -> Stop {
    let message = format!("ask {ask} names {world}, neither the model nor a loaded scenario");
    Stop::Refused(Refusal::new(Code::Load(Load::Scenario), message).reaching(Reach::UnloadedWorld))
}
