//! The pins: an `as if` and an ask's `with` applied, each pin's keys, date and value, and the `pinned:` lines they print.
//! Spec: L §5.11 (`as if` and its pins: a pin's value and keys, a hole pin), §5.18 (an ask's
//! `with` pins), §4.3 (the `law` and `wording` pins), §8.2 (`pinned:`, `world:`, a pin formed
//! per binding tuple), §6.4 (`as if {scenario s}` is x in s), §5.5 (`with a = opt` fixes an
//! axis, listed `pinned:`); E §3.2 (a pin given by an expression is evaluated first, without
//! that `as if`), §10 (an `as if` explained: the pins with their values and the derivation
//! inside them beside the one outside).
//! Twin: engine/tools/refcore/evaluate.py (`apply_pins`, `e_asif`, `asif_traced`,
//! `pin_lines`, `pin_value`, `coerce_to`, `_pertuple`).
//! Invariants: a pin's keys, date and value are evaluated in the context outside the pins; a
//! pin whose key or date is a hole makes the result that hole, nothing inside evaluated; the
//! layer a set of pins makes is interned in `context.rs`, and freed when its scope ends.

use std::collections::BTreeSet;

use super::context::{Pinned, world_named};
use super::keys::Given;
use super::state::{Ctx, Engine, Env, invariant, num_stop};
use super::trace::Kind as TraceKind;
use crate::base::Code;
use crate::hole::{Hole, spread};
use crate::model::{
    BindInfo, Class, Domain, KeyVal, NameKind, Number, PeriodWord, Reality, Shown, Ty, Unit, Value,
    currency_id, money_num, num_money,
};
use crate::num::Num;
use crate::syntax::{Expr, Key, Lit, PRef, Pin, PinForm, Stop};
use crate::time::Day;

impl Engine {
    /// `e as if {pins}`: e evaluated in the context the pins give, their values evaluated
    /// first in the context outside (refcore's `e_asif`); a pin whose key or date is a hole
    /// makes the value that hole, nothing inside evaluated; `{scenario s}` evaluates e in s
    /// (L §6.4), the moves recomputed in a timeline of their own where a pin or the world
    /// reaches them. L §5.11.
    ///
    /// # Errors
    ///
    /// The refusals the pins and e meet; `NotBuilt` for a sample pin, a world of
    /// another stack of scenarios, and a decision's running read that would change timeline.
    pub(super) fn as_if(
        &mut self,
        m: &Reality,
        (inner, pins): (&Expr, &[Pin]),
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        if self.tracing() {
            return self.as_if_traced(m, (inner, pins), env, ctx);
        }
        let inside = match self.pinned(m, pins, env, ctx, Pinning::AsIf)? {
            Err(hole) => return Ok(Value::Hole(hole)),
            Ok(inside) => inside,
        };
        self.open_layer(m, inside)?;
        let v = self.ev(m, inner, env, inside);
        self.end_layer(inside.layer, v.as_ref().err())?;
        v
    }

    /// `e as if {pins}` explained, under its node `as if {...}`: the pins with their values,
    /// each pin given by an expression with its derivation, read outside the `as if`; then the
    /// derivation inside the pins with its value; then the one outside them with its value,
    /// which brings nothing to the result's register and whose refusal stands in its place
    /// (refcore's `asif_traced`). E §10.
    fn as_if_traced(
        &mut self,
        m: &Reality,
        (inner, pins): (&Expr, &[Pin]),
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let node = self.trace_open(TraceKind::Group, "as if {...}");
        let group = self.trace_open(TraceKind::Group, "the pins, evaluated outside the as if");
        let pinned = self.pinned(m, pins, env, ctx, Pinning::AsIf);
        let inside = match pinned {
            Ok(Ok(inside)) => inside,
            Ok(Err(hole)) => {
                self.trace_close(group);
                self.trace_close(node);
                return Ok(Value::Hole(hole));
            }
            Err(stop) => {
                self.trace_close(group);
                self.trace_close(node);
                return Err(stop);
            }
        };
        match self.pin_lines(m, ctx, inside) {
            Ok(lines) => {
                for line in lines {
                    self.trace_leaf(format!("pin {line}"));
                }
            }
            Err(stop) => self.trace_kept(stop),
        }
        self.trace_close(group);
        let within = self.trace_open(TraceKind::Side, "inside the as if");
        let read = self.open_layer(m, inside).and_then(|()| {
            let v = self.ev(m, inner, env, inside);
            self.end_layer(inside.layer, v.as_ref().err())?;
            v
        });
        self.trace_close(within);
        // a read that stops leaves the side as it is and nothing outside (refcore's raise)
        let v = match read {
            Ok(v) => v,
            Err(stop) => {
                self.trace_close(node);
                return Err(stop);
            }
        };
        self.trace_value(within, &Ok(v.clone()));
        let without = self.trace_open(TraceKind::Side, "outside the as if");
        self.push();
        // the derivation outside is no read of the result: neither its register nor the ask
        // pins it takes are listed (L §8.2 One read set; P42)
        let keeping = core::mem::replace(&mut self.pins_read.keeping, false);
        let outside = self.ev(m, inner, env, ctx);
        self.pins_read.keeping = keeping;
        self.pop_alone();
        self.trace_close(without);
        self.trace_close(node);
        match outside {
            Err(Stop::Refused(r)) if matches!(r.code(), Code::Load(_)) => Err(Stop::Refused(r)),
            Err(Stop::Refused(r)) => {
                self.trace_value(without, &Err(Stop::Refused(r)));
                Ok(v)
            }
            Err(stop) => Err(stop),
            Ok(o) => {
                self.trace_value(without, &Ok(o));
                Ok(v)
            }
        }
    }

    /// The context of an ask's line of context `line` outside the ask's `with` pins:
    /// `unpinned`, the line's context before them, with the options of the axes the line fans
    /// out over, those in `fixed`, which the pins fix, left out (refcore's `explain_pins`, its
    /// `octx`). E §10, L §5.5.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a freed layer.
    pub(crate) fn outside_pins(
        &mut self,
        (unpinned, line): (Ctx, Ctx),
        fixed: &BTreeSet<String>,
    ) -> Result<Ctx, Stop> {
        let in_line = self.layer_data(line.layer)?;
        let options: Vec<(String, String)> = in_line
            .axes
            .iter()
            .filter(|(axis, _)| !fixed.contains(*axis))
            .map(|(axis, option)| (axis.clone(), option.clone()))
            .collect();
        self.with_options(unpinned, &options)
    }

    /// What the pins of `inside` change from `outside`, each as text: the value pins with
    /// their values, the law date, the cut, the wording pins, the axis options and the world
    /// (refcore's `pin_lines`). E §10.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a freed layer.
    pub(crate) fn pin_lines(
        &self,
        m: &Reality,
        outside: Ctx,
        inside: Ctx,
    ) -> Result<Vec<String>, Stop> {
        let (before, after) = (
            self.layer_data(outside.layer)?,
            self.layer_data(inside.layer)?,
        );
        // refcore's pins are sorted by their keys' text, the item's id first
        let mut new: Vec<&Pinned> = after
            .pins
            .iter()
            .filter(|p| !p.solve && !before.holds(p))
            .collect();
        new.sort_by(|a, b| (&a.item, &a.text).cmp(&(&b.item, &b.text)));
        let mut out: Vec<String> = new.iter().map(|p| p.text.clone()).collect();
        if let (true, Some(day)) = (inside.law != outside.law, inside.law) {
            out.push(format!("law {}", day.text()));
        }
        if inside.cut != outside.cut {
            out.push(format!("known {}", inside.cut.text()));
        }
        for (item, day) in &after.wording {
            if !before.wording.contains(&(item.clone(), *day)) {
                out.push(format!("wording {} for {item}", day.text()));
            }
        }
        for (axis, option) in &after.axes {
            if before.axes.get(axis) != Some(option) {
                out.push(format!("{axis} = {option}"));
            }
        }
        if inside.world != outside.world {
            let world = m
                .scenarios
                .scenario_of(inside.world)
                .unwrap_or(&m.header.id);
            out.push(format!("scenario {world}"));
        }
        Ok(out)
    }

    /// The context inside `pins` (refcore's `apply_pins`): each pin's date, keys and value
    /// evaluated in `ctx`; `law D` the law date, `known D` the cut, `wording D for […]` the
    /// named items' wording, a value pin its target's cells. A pin given by an expression
    /// brings what it read to the reader's register, with its `pinned:` entry; an ask's value
    /// pins are listed whether read or not. A pin whose key, or a date pin whose date, is a
    /// hole makes the result that hole, with every such pin's questions. L §5.11, §5.18, §8.2.
    ///
    /// `scenario s` sets the world, listed `world: s` (L §6.4, §8.2).
    ///
    /// # Errors
    ///
    /// The refusals the pins meet; `NotBuilt` for a sample pin and for a world of
    /// another stack of scenarios.
    pub(crate) fn pinned(
        &mut self,
        m: &Reality,
        pins: &[Pin],
        env: &Env,
        ctx: Ctx,
        how: Pinning,
    ) -> Result<Result<Ctx, Hole>, Stop> {
        let mut data = self.layer_data(ctx.layer)?.clone();
        // this set's scope: inside every pin already in force (L §4.6's cascade, R4)
        let depth = data.next_layer()?;
        let mut inside = ctx;
        let mut holes: Vec<Hole> = Vec::new();
        for pin in pins {
            match &pin.form {
                PinForm::Scenario(id) => {
                    // `x as if {scenario s}` is x in s (L §6.4 Asking; refcore's `apply_pins_`)
                    inside.world = world_named(m, id)?;
                    self.note("world", id.clone(), String::new())?;
                }
                PinForm::Law(e) | PinForm::Known(e) | PinForm::Wording(e, _) => {
                    let Some(day) = self.date_pin(m, pin, e, env, ctx, &mut holes)? else {
                        continue;
                    };
                    match &pin.form {
                        PinForm::Law(_) => inside.law = Some(day),
                        PinForm::Known(_) => inside.cut = day,
                        PinForm::Wording(_, ids) => data.word(ids, day),
                        PinForm::Scenario(_) | PinForm::Set { .. } => {}
                    }
                }
                PinForm::Set {
                    target,
                    value,
                    sample,
                } => {
                    if *sample {
                        return Err(Stop::NotBuilt("a sample pin at run"));
                    }
                    if m.names.kind(&target.id) == Some(NameKind::Axis) {
                        // `ax = option` fixes the axis, listed under `pinned:` (L §5.5)
                        let option = axis_pinned(value)?;
                        let text = format!("{} = {option}", target.id);
                        data.axes.insert(target.id.clone(), option);
                        self.note("pinned", text, String::new())?;
                        continue;
                    }
                    let set = (target, value, pin.text.as_str());
                    if let Some(mut pinned) = self.value_pin(m, set, env, ctx, (how, &mut holes))? {
                        pinned.layer = depth;
                        data.pin(pinned);
                    }
                }
            }
        }
        let operands: Vec<Option<&Hole>> = holes.iter().map(Some).collect();
        if let Some(hole) = spread(&operands) {
            return Ok(Err(hole));
        }
        inside.layer = self.intern(data)?;
        Ok(Ok(inside))
    }

    /// A `law`, `known` or `wording` pin's day, its `pinned:` entry noted, with what an
    /// expression read; none, its hole kept in `holes`, for a date that is a hole (refcore's
    /// `apply_pins_`, its date arm). L §5.11, §8.2.
    fn date_pin(
        &mut self,
        m: &Reality,
        pin: &Pin,
        e: &Expr,
        env: &Env,
        ctx: Ctx,
        holes: &mut Vec<Hole>,
    ) -> Result<Option<Day>, Stop> {
        self.push();
        let v = self.ev(m, e, env, ctx);
        let met = self.pop_alone();
        let v = v?;
        if let Value::Hole(hole) = v {
            holes.push(hole);
            return Ok(None);
        }
        let word = match &pin.form {
            PinForm::Law(_) => "law",
            PinForm::Known(_) => "known",
            PinForm::Wording(..) | PinForm::Scenario(_) | PinForm::Set { .. } => "wording",
        };
        let day = match super::windows::one_day(m, &v, &format!("the {word} pin"))? {
            Ok(day) => day,
            Err(hole) => {
                holes.push(hole);
                return Ok(None);
            }
        };
        let expr = !matches!(
            e,
            Expr::Lit(Lit::Day(_) | Lit::Month(..) | Lit::Quarter(..) | Lit::Num(_))
        );
        if expr {
            self.absorb(&met);
        }
        // a cut after the clock is refused before its pin is noted (refcore's `apply_pins_`)
        let day = if matches!(pin.form, PinForm::Known(_)) {
            after_on(m, day)?
        } else {
            day
        };
        let written = if expr {
            crate::load::expr_text(e)
        } else {
            day.text()
        };
        let head = match &pin.form {
            PinForm::Wording(_, ids) => format!("wording {written} for [{}]", ids.join(", ")),
            PinForm::Law(_) | PinForm::Known(_) | PinForm::Scenario(_) | PinForm::Set { .. } => {
                format!("{word} {written}")
            }
        };
        let text = if expr {
            format!("{head} = {}", day.text())
        } else {
            head
        };
        self.note("pinned", text, String::new())?;
        Ok(Some(day))
    }

    /// A value pin `x[keys].f = v`: its keys and value evaluated in `ctx`, the value in its
    /// target's type; an ask's noted `pinned:`; none, its hole kept in `holes`, for a key
    /// that is a hole (refcore's `apply_pins_`, its target arm, and `coerce_to`). A pin formed
    /// per binding tuple, its value or a key naming a binding in scope, prints once as written
    /// `pin`, with its bindings (`salary = gross[m] for m in months`, refcore's `_pertuple`).
    /// L §5.11, §8.2.
    ///
    /// # Errors
    ///
    /// The refusals its keys and value meet.
    fn value_pin(
        &mut self,
        m: &Reality,
        (target, written, pin): (&PRef, &Expr, &str),
        env: &Env,
        ctx: Ctx,
        (how, holes): (Pinning, &mut Vec<Hole>),
    ) -> Result<Option<Pinned>, Stop> {
        let id = target.id.as_str();
        let (binds, declared, mut factor) = match m.names.kind(id) {
            Some(NameKind::Input) => {
                let input = m
                    .inputs
                    .get(id)
                    .ok_or_else(|| invariant("a pinned input"))?;
                (&input.binds, input.ty.clone(), input.unit_factor.clone())
            }
            Some(NameKind::Def) => {
                let def = m.defs.get(id).ok_or_else(|| invariant("a pinned def"))?;
                (&def.binds, Some(def.ty.clone()), def.unit_factor.clone())
            }
            _ => return Err(invariant("a pin load didn't check")),
        };
        let keyed = |key: &Key| matches!(key, Key::Expr(e) if env.read_by(e));
        // an ask's pin in its own binding tuple's scope is that tuple's line's own pin, its
        // keys as they read (L §5.11 A pin's keys, §5.18): only a pin inside one line prints
        // once with its binding
        let inside_line = how != Pinning::Ask;
        let per_tuple =
            if inside_line && (env.read_by(written) || target.keys.iter().flatten().any(keyed)) {
                // L §8.2: a pin formed per binding tuple prints once, as written, with its
                // binding (refcore's `_pertuple`): the value's, then each key's
                let mut parts = vec![written];
                for key in target.keys.iter().flatten() {
                    if let Key::Expr(e) = key {
                        parts.push(e);
                    }
                }
                let named = env.binds_named(&parts);
                (!named.is_empty()).then(|| format!("{pin} for {}", named.join(", ")))
            } else {
                None
            };
        let mut keys = Vec::with_capacity(binds.len());
        let mut texts = Vec::with_capacity(binds.len());
        let all_written = match target.keys.as_deref() {
            Some(at) if at.len() == binds.len() && at.iter().all(is_written) => Some(at),
            Some(_) | None => None,
        };
        if let Some(at) = all_written {
            // every key written: made in one call, so a range's ends read the keys before it
            // (L §2.5, §5.11)
            let given = match self.written_keys(m, at, env, ctx)? {
                Err(hole) => {
                    holes.push(hole);
                    return Ok(None);
                }
                Ok(given) => given,
            };
            match self.make_keys(m, id, binds, given, ctx)? {
                Err(hole) => {
                    holes.push(hole);
                    return Ok(None);
                }
                Ok(made) => {
                    texts.extend(made.iter().map(KeyVal::text));
                    keys.extend(made.into_iter().map(Some));
                }
            }
        }
        // a `*` leaves its position open: each written key is made alone
        for (key, bind) in target.keys.iter().flatten().zip(binds) {
            if keys.len() == binds.len() {
                break;
            }
            let Key::Expr(e) = key else {
                keys.push(None);
                texts.push("*".to_owned());
                continue;
            };
            let given = self.given_key(m, e, env, ctx)?;
            match self.make_keys(m, id, core::slice::from_ref(bind), vec![given], ctx)? {
                Err(hole) => {
                    holes.push(hole);
                    return Ok(None);
                }
                Ok(made) => {
                    let [one] = made.as_slice() else {
                        return Err(invariant("a pin's key made more keys than one"));
                    };
                    texts.push(one.text());
                    keys.push(Some(one.clone()));
                }
            }
        }
        let mut ty = declared;
        for field in &target.fields {
            ty = ty.and_then(|t| t.field(field).cloned());
            // a field's type keeps its `%` as `num %` only (L §3.2)
            factor = percent_factor(ty.as_ref())?;
        }
        self.push();
        let v = self.ev(m, written, env, ctx);
        let met = self.pop_alone();
        let value = pin_typed(m, written, v?, (ty.as_ref(), &factor))?;
        let value = at_pin_keys(value, binds, &keys)?;
        if !plain_literal(m, written) {
            self.absorb(&met);
        }
        let mut head = id.to_owned();
        if !texts.is_empty() {
            head = format!("{head}[{}]", texts.join(", "));
        }
        for field in &target.fields {
            head = format!("{head}.{field}");
        }
        let text = match per_tuple {
            Some(text) => text,
            None => format!("{head} = {}", value.typed_text(m).map_err(num_stop)?),
        };
        if how == Pinning::Ask {
            // an ask's `with` pin of a cell is listed where a read of the line took it (L
            // §8.2 One read set; tally 2.1.8 §4, #161)
            self.note("pinned", text.clone(), String::new())?;
            self.note("pin cell", text.clone(), String::new())?;
        }
        Ok(Some(Pinned {
            solve: false,
            item: id.to_owned(),
            keys,
            fields: target.fields.clone(),
            value,
            text,
            layer: 0,
            tied: None,
        }))
    }

    /// A pin's keys, every one written, as given in order; the first that is a hole makes
    /// the pin that hole, and the keys after it are not read (refcore's `apply_pins`, its
    /// `keyhole`). L §5.11, §8.1.
    fn written_keys(
        &mut self,
        m: &Reality,
        written: &[Key],
        env: &Env,
        ctx: Ctx,
    ) -> Result<Result<Vec<Given>, Hole>, Stop> {
        let mut given = Vec::with_capacity(written.len());
        for key in written {
            let Key::Expr(e) = key else {
                return Err(invariant("a pin's `*` key made with the written ones"));
            };
            let one = self.given_key(m, e, env, ctx)?;
            if let Given::Value(Value::Hole(hole)) = one {
                return Ok(Err(hole));
            }
            given.push(one);
        }
        Ok(Ok(given))
    }
}

/// Whether a pin's key is written, not `*` (L §5.11).
/// A cut computed from a value, `known d`, after the model's `on`: refused at that value,
/// `RUN-DOMAIN known <d> is after the model's on <on>` (L §8.4, E §11; R5, #112; refcore's
/// `after_on`). An ask line's literal cut is the request's `REQ-CLOCK`, checked first.
pub(crate) fn after_on(m: &Reality, d: Day) -> Result<Day, Stop> {
    if d <= m.on {
        return Ok(d);
    }
    Err(Stop::Refused(crate::base::Refusal::new(
        Code::Run(crate::base::Run::Domain),
        format!("known {} is after the model's on {}", d.text(), m.on.text()),
    )))
}

fn is_written(key: &Key) -> bool {
    matches!(key, Key::Expr(_))
}

/// The option an axis pin `ax = option` names, a key literal load checked (refcore's
/// `p.option`). L §5.5.
fn axis_pinned(value: &Expr) -> Result<String, Stop> {
    let (Expr::Name(option) | Expr::Lit(Lit::Key(option))) = value else {
        return Err(invariant(
            "an axis pin that names no option, which load refuses",
        ));
    };
    Ok(option.clone())
}

/// What applies a set of pins: an ask's `with`, whose value pins are listed whether read or
/// not, or an `as if` (L §5.18, §8.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Pinning {
    /// An ask's `with` pins.
    Ask,
    /// An `as if`'s pins.
    AsIf,
}

/// Whether a pin's value is written plainly, so what it read stays off the register: a
/// literal, a negation, a text, or a name that reads no input or def (refcore's
/// `apply_pins_`, its kinds). L §8.2.
fn plain_literal(m: &Reality, e: &Expr) -> bool {
    if let Expr::Name(name) = e {
        return !matches!(m.names.kind(name), Some(NameKind::Input | NameKind::Def));
    }
    matches!(e, Expr::Lit(_) | Expr::Neg(_) | Expr::Str(_))
}

/// A literal pin's value `v` in its target's declared type `ty`, whose unit brings `factor`;
/// none where `written` is no literal the type takes (refcore's `pin_literal`, `typed`, and
/// its checker's `_keyat`, `as_year`). L §3.2 Inline figures, §2.1, §2.5.
/// - A bare figure (through brackets and a minus) on money or an int is that money or int;
///   on a `num` it is the figure times the `%` its unit holds: `5` on `num %` is 5 %, on
///   `num %/year` 0.05 1/year, as a row's figure is.
/// - A figure with its unit on a `num` of that unit is that figure shown as the type shows it
///   (`50 %` on a plain `num` is 0.5; tally 2.3.1 R3).
/// - A four-digit figure on a `year` is that year.
/// - A key literal (a name, a four-digit year, a period) on a `key` is the key of that index,
///   kind or axis.
fn written_typed(
    m: &Reality,
    written: &Expr,
    v: &Value,
    (ty, factor): (&Ty, &Num),
) -> Result<Option<Value>, Stop> {
    let Value::Number(n) = v else {
        return pinned_key(m, written, v, ty);
    };
    if !super::band::bare_figure(written) {
        // a figure with its unit on a `num` of that unit is the row's figure, shown as the
        // type shows it: `with ratio = 50 %` on a plain `num` is 0.5, as the row `ratio = 50 %`
        // is (L §5.11 A pin is a row; tally 2.3.1 R3, N256.6)
        let Ty::Num(unit, shown) = ty else {
            return Ok(None);
        };
        return Ok((unit_figure(written) && n.unit() == *unit).then(|| {
            Value::Number(Number {
                q: n.q.clone(),
                class: Class::Num(unit.clone(), *shown),
            })
        }));
    }
    if let Ty::Num(unit, shown) = ty {
        return Ok(Some(Value::Number(Number {
            q: n.q.checked_mul(factor).map_err(num_stop)?,
            class: Class::Num(unit.clone(), *shown),
        })));
    }
    if matches!(ty, Ty::Money(_) | Ty::Int) {
        return super::expr::typed(m, &n.q, ty);
    }
    if matches!(ty, Ty::Period(PeriodWord::Year)) {
        return Ok(super::years::as_year(Some(written), v).map(Value::Period));
    }
    pinned_key(m, written, v, ty)
}

/// A key literal `written` pinned on a `key` target `ty`, a bare key `v` read as a word or a
/// literal: the key of its index, kind or axis that the literal names (refcore's `assign`,
/// its `keylit` arm, and a four-digit year as a key of a year index, L §2.1, §2.5); none for
/// any other written value or type.
fn pinned_key(m: &Reality, written: &Expr, v: &Value, ty: &Ty) -> Result<Option<Value>, Stop> {
    let Ty::Key(id) = ty else {
        return Ok(None);
    };
    let lit = if let Value::Word(word) = v {
        Lit::Name(word.clone())
    } else if let Expr::Lit(lit) = written.unbracketed() {
        lit.clone()
    } else {
        return Ok(None);
    };
    let domain = if m.axes.contains(id) {
        Domain::Axis(id.clone())
    } else if m.kinds.get(id).is_some() {
        Domain::Kind(id.clone())
    } else {
        Domain::Index(id.clone())
    };
    let key = super::keys::literal_key(m, &lit, &domain)?;
    Ok(key.map(|k| Value::of_key(&k)))
}

/// Whether a pin's value is a figure with its unit, through brackets and a minus (`50 %`,
/// `-(5 %/year)`), as a row writes one. L §3.2 Inline figures, §5.11 A pin is a row.
fn unit_figure(e: &Expr) -> bool {
    let mut at = e;
    while let Expr::Paren(x) | Expr::Neg(x) = at {
        at = x;
    }
    matches!(at, Expr::Lit(Lit::Num(n)) if n.unit.is_some())
}

/// The factor a `num %` type's `%` brings, 1/100; 1 for any other type. L §3.2.
fn percent_factor(ty: Option<&Ty>) -> Result<Num, Stop> {
    if matches!(ty, Some(Ty::Num(_, Shown::Percent))) {
        return Num::ratio(1, 100).map_err(num_stop);
    }
    Ok(Num::int(1))
}

/// A pin's value at the keys it names, every key named: a per-unit price's figure is in CUR
/// per the key's unit, as a def cell's is (L §3.2 Per-unit prices of a unit kind). A pin with
/// a `*` keeps the type's `$u`.
fn at_pin_keys(v: Value, binds: &[BindInfo], keys: &[Option<KeyVal>]) -> Result<Value, Stop> {
    let named: Option<Vec<KeyVal>> = keys.iter().cloned().collect();
    if let (Value::Number(n), Some(named)) = (&v, named) {
        return Ok(Value::Number(super::read::at_keys(
            n.clone(),
            binds,
            &named,
        )?));
    }
    Ok(v)
}

/// A pin's value in its target's type: a literal `written` takes the target's declared type
/// (`written_typed`); a number pinned on money is money where it is whole minor units, an
/// int or money pinned on a `num` takes its unit (refcore's `coerce_to`). L §3.2 Inline
/// figures, §5.11.
fn pin_typed(
    m: &Reality,
    written: &Expr,
    v: Value,
    (ty, factor): (Option<&Ty>, &Num),
) -> Result<Value, Stop> {
    if let Some(ty) = ty
        && let Some(typed) = written_typed(m, written, &v, (ty, factor))?
    {
        return Ok(typed);
    }
    match (ty, v) {
        (Some(Ty::Money(cur)), Value::Number(n)) => {
            let minor = m
                .currencies
                .get(cur)
                .ok_or_else(|| invariant("a currency load didn't declare"))?;
            if !n.q.checked_div(minor).map_err(num_stop)?.is_integer() {
                return Ok(Value::Number(n));
            }
            let id = currency_id(m, cur).map_err(num_stop)?;
            Ok(Value::Money(num_money(m, &n.q, id).map_err(num_stop)?))
        }
        (Some(Ty::Num(unit, shown)), Value::Number(n))
            if matches!(n.class, Class::Int | Class::Flex) =>
        {
            Ok(Value::Number(Number {
                q: n.q,
                class: Class::Num(unit.clone(), *shown),
            }))
        }
        // a figure with a period's unit pinned on a rate is that rate, as a row's figure is
        // (`10 %/year` is 0.1 /year; L §3.3, pick P16)
        (Some(Ty::Rate), Value::Number(n))
            if matches!(n.class, Class::Num(..)) && n.unit().period_of().is_some() =>
        {
            let unit = n.unit();
            Ok(Value::Number(Number {
                q: n.q,
                class: Class::Rate(unit),
            }))
        }
        (Some(Ty::Num(..)), Value::Money(money)) => {
            let q = money_num(m, &money).map_err(num_stop)?;
            let unit: Unit = crate::model::currency_unit(m, money.currency()).map_err(num_stop)?;
            Ok(Value::Number(Number::plain(q, unit)))
        }
        (_, v) => Ok(v),
    }
}

#[cfg(test)]
mod tests {
    use crate::api::{Asks, Files, Given, Request, run};

    /// A line's law date and cut, pinned (the writer's `lawpin`, tally 2.3.1 R3).
    const LAWPIN: &str = r#"model lawpin
  tally 2
  on 2026-06-30
  start 2026-01-01
  end 2026-12-31

source s "probe"
unit PLN: currency minor 0.01
input rate: money PLN
input ratio: num

def cost: money PLN at reader
  rule one = rate

ASKS

version v
  cite s
  governs 2026-01-01..2026-06-30
  known 2026-01-01
  rate = 10.00 PLN "s"
  ratio = 1 "s"

version v2
  cite s
  governs 2026-07-01..
  known 2026-01-01
  rate = 20.00 PLN "s"
"#;

    /// Every ask of `LAWPIN` with `asks` in place of its asks, its lines.
    fn ran(asks: &str) -> Vec<String> {
        let text = LAWPIN.replace("ASKS", asks);
        let given = vec![Given {
            path: "lawpin.tally".to_owned(),
            bytes: Some(text.into_bytes()),
        }];
        run(Files::new(given), &Request::new(Asks::Every))
            .expect("an answer")
            .lines()
            .to_vec()
    }

    #[test]
    fn two_pins_of_the_law_date_on_a_line_overlap() {
        // L §5.11 A pin is a row: a clause and a `with law` pin are two pins of one cell
        let clause = ran("ask q4: cost with law 2026-02-01 law 2026-09-01");
        assert_eq!(
            clause,
            [
                "refused: LOAD-OVERLAP pins law 2026-02-01 and law 2026-09-01 of the law date \
                 overlap at no keys (lawpin.tally:15)"
            ],
            "the with pin first, then the clause"
        );
        let two = ran("ask q5: cost with law 2026-09-01, law 2026-02-01");
        assert_eq!(
            two.first().map(String::as_str),
            Some(
                "refused: LOAD-OVERLAP pins law 2026-09-01 and law 2026-02-01 of the law date \
                 overlap at no keys (lawpin.tally:15)"
            ),
            "two with pins"
        );
        let cut = ran("ask q7: cost with known 2026-03-01 as known 2026-05-01");
        assert_eq!(
            cut.first().map(String::as_str),
            Some(
                "refused: LOAD-OVERLAP pins known 2026-03-01 and as known 2026-05-01 of the cut \
                 overlap at no keys (lawpin.tally:15)"
            ),
            "the cut's two pins"
        );
    }

    #[test]
    fn two_pins_of_one_cell_in_an_as_if_overlap() {
        // L §5.11 "two pins of one cell" (G3's review N6, the lead's ruling): as the line's
        for (ask, refusal) in [
            (
                "ask a10: cost as if {law 2026-02-01, law 2026-09-01}",
                "refused: LOAD-OVERLAP pins law 2026-02-01 and law 2026-09-01 of the law date \
                 overlap at no keys (lawpin.tally:15)",
            ),
            (
                "ask a11: cost as if {known 2026-02-01, known 2026-03-01} law 2026-09-01",
                "refused: LOAD-OVERLAP pins known 2026-02-01 and known 2026-03-01 of the cut \
                 overlap at no keys (lawpin.tally:15)",
            ),
        ] {
            assert_eq!(ran(ask).first().map(String::as_str), Some(refusal), "{ask}");
        }
    }

    #[test]
    fn a_law_pin_and_a_cut_pin_on_one_line_stand() {
        // one pin of each cell, and an `as if {law D}` inside an expression is that as if's
        let lines = ran("ask t2: cost as if {law 2026-02-01} law 2026-09-01 as known 2026-05-01");
        assert_eq!(
            lines,
            [
                "t2: 10.00 PLN",
                "  pinned: law 2026-02-01",
                "  law date 2026-09-01, cut 2026-05-01",
                "warning: WARN-UNREAD input ratio is read by nothing",
            ],
            "no overlap"
        );
    }

    #[test]
    fn a_literal_cut_after_on_in_the_line_is_req_clock() {
        // E §11: an `as if {known D}` in the line's body, first, before the clause
        let lines = ran("ask q8: cost as if {known 2026-03-01} law 2026-03-01\n\
             ask q10: (cost as if {known 2026-09-01}) + cost law 2026-03-01 as known 2026-08-01");
        assert_eq!(
            lines,
            ["refused: REQ-CLOCK known 2026-09-01 is after the model's on 2026-06-30"],
            "the run's one result"
        );
    }

    #[test]
    fn a_figure_with_its_unit_pinned_is_the_row() {
        // L §5.11 A pin is a row: `50 %` on a plain num is 0.5, as the row is (N256.6)
        let lines = ran("ask r1: ratio + ratio with ratio = 50 % law 2026-03-01");
        assert_eq!(
            lines.get(..2),
            Some(["r1: 1".to_owned(), "  pinned: ratio = 0.5".to_owned()].as_slice()),
            "the declared type's figure"
        );
    }

    #[test]
    fn a_literal_pin_of_the_wrong_type_is_refused_as_its_row() {
        // L §5.11 A pin is a row (G3's review N1): the row reader's words, not the pin's
        for (ask, refusal) in [
            (
                "ask m2: ratio with ratio = 5.00 PLN",
                "refused: LOAD-UNIT 5.00 PLN and num have different units (lawpin.tally:15)",
            ),
            (
                "ask m4: ratio with ratio = (5 %/year)",
                "refused: LOAD-UNIT 5 1/year and num have different units (lawpin.tally:15)",
            ),
            (
                "ask m3: ratio with ratio = 2026-01-01",
                "refused: LOAD-UNIT a value of the wrong type: a date for num (lawpin.tally:15)",
            ),
        ] {
            assert_eq!(ran(ask).first().map(String::as_str), Some(refusal), "{ask}");
        }
    }
}
