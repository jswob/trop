//! The plan pass's legs: each leg's two places, its day and its amount as a quantity of its unit.
//! Spec: L §5.14 Legs (a leg's `from` and `to` a place or a hole, its amount a hole or a
//! quantity of the kind's leg unit, `RUN-NONE`, `RUN-DOMAIN`, `RUN-NEG`), §5.10 (places of
//! another unit are `LOAD-UNIT`, money finer than its minor unit `LOAD-UNIT`), §3.2 (a
//! quantity's unit); E §4.1 step 3 (an instance's legs evaluated at its booking cut).
//! Twin: engine/tools/refcore/moves.py (`eval_leg`, `leg_unit_static`, `leg_amount`).
//! Invariants: a run-time refusal met reading a leg's amount stays with that leg; a load
//! refusal met at run refuses the run; a hole amount's unit is the kind's leg unit where it
//! is one, else a place's.

use super::plan::{
    Amount, Dated, End, Inst, LegVal, date_of, invariant, num_err, quantity, refuse, refuse_load,
    shown,
};
use crate::base::{Code, Load, Refusal, Run};
use crate::eval::{Ctx, Engine};
use crate::model::{KeyVal, Reality, Value};
use crate::num::Num;
use crate::syntax::{Leg, Stop};

impl Engine {
    /// One leg: its places, its day and its amount in its unit; a run-time refusal of the
    /// amount stays with the leg (refcore's `eval_leg`). L §5.14 Legs.
    pub(super) fn eval_leg(
        &mut self,
        m: &Reality,
        inst: &Inst,
        leg: &Leg,
        kind_unit: &str,
        ctx: Ctx,
    ) -> Result<LegVal, Stop> {
        let from_value = self.ev(m, &leg.from, &inst.env, ctx)?;
        let to_value = self.ev(m, &leg.to, &inst.env, ctx)?;
        let from = leg_end(m, inst, leg, from_value, "from")?;
        let to = leg_end(m, inst, leg, to_value, "to")?;
        let day = match &leg.on {
            None => inst
                .day
                .clone()
                .ok_or_else(|| invariant("a leg of an instance with no day"))?,
            Some(e) => {
                let v = self.ev(m, e, &inst.env, ctx)?;
                if v == Value::None {
                    return Err(refuse(
                        Run::None,
                        format!("the day of leg {} of {} is none here", leg.name, inst.id),
                    ));
                }
                if let Value::Window(_) = v {
                    // a leg dated by a window is a movement dated by one (L §4.4), as the
                    // instance's own day: not a load refusal, and not built
                    return Err(Stop::NotBuilt("a movement dated by a window"));
                }
                if let Value::Hole(h) = v {
                    Dated::Hole(h)
                } else {
                    Dated::Day(date_of(&v)?)
                }
            }
        };
        let read = self.ev(m, &leg.amount, &inst.env, ctx);
        let ends = (&from, &to);
        let (amount, unit) = match read {
            Ok(v) => amount_of(m, &inst.id, &leg.name, kind_unit, &v, ends)?,
            Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => {
                (Amount::Refused(r), None)
            }
            Err(stop) => return Err(stop),
        };
        Ok(LegVal {
            name: leg.name.clone(),
            from,
            to,
            amount,
            unit,
            day,
        })
    }
}

/// A leg's end read as `v`: a place, or a hole; `RUN-NONE` for none, `RUN-DOMAIN` for a value
/// that isn't a place (refcore's `eval_leg`, its places). L §5.14 Legs.
fn leg_end(m: &Reality, inst: &Inst, leg: &Leg, v: Value, side: &str) -> Result<End, Stop> {
    if v == Value::None {
        return Err(refuse(
            Run::None,
            format!(
                "the {side} place of leg {} of {} is none here",
                leg.name, inst.id
            ),
        ));
    }
    if let Value::Hole(h) = v {
        return Ok(End::Hole(h));
    }
    if let Value::Key(KeyVal::Thing(name)) = &v
        && let Some(thing) = m.things.get(name)
        && m.is_place(thing)
    {
        if thing
            .base
            .as_ref()
            .is_some_and(|b| m.subtree("assets").contains(b))
        {
            return Err(refuse_load(
                Load::Kind,
                format!("a leg to or from an asset ({} of {})", leg.name, inst.id),
            ));
        }
        return Ok(End::Place(name.clone()));
    }
    Err(refuse(
        Run::Domain,
        format!(
            "the {side} of leg {} of {} is {}, not a place",
            leg.name,
            inst.id,
            shown(m, &v)?
        ),
    ))
}

/// A leg's amount read as `v`: a hole, `RUN-NONE` for none, a quantity in its unit, else
/// `RUN-DOMAIN`; a run-time refusal stays with the leg (refcore's `eval_leg`, its amount).
/// L §5.14 Legs.
fn amount_of(
    m: &Reality,
    inst_id: &str,
    leg: &str,
    kind_unit: &str,
    v: &Value,
    ends: (&End, &End),
) -> Result<(Amount, Option<String>), Stop> {
    if let Value::Hole(h) = v {
        return Ok((
            Amount::Hole(h.clone()),
            leg_unit_static(m, kind_unit, ends.0, ends.1),
        ));
    }
    if *v == Value::None {
        let r = Refusal::new(
            Code::Run(Run::None),
            format!("the amount of leg {leg} of {inst_id} is none here"),
        );
        return Ok((Amount::Refused(r), None));
    }
    if !matches!(v, Value::Money(_) | Value::Number(_)) {
        let r = Refusal::new(
            Code::Run(Run::Domain),
            format!("the amount of leg {leg} of {inst_id} is {}", shown(m, v)?),
        );
        return Ok((Amount::Refused(r), None));
    }
    match leg_amount(m, inst_id, leg, kind_unit, v, ends.0, ends.1) {
        Ok((q, u)) => Ok((Amount::Qty(q), Some(u))),
        Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => {
            Ok((Amount::Refused(r), None))
        }
        Err(stop) => Err(stop),
    }
}

/// A hole amount's unit, where it can be seen: the kind's leg unit when it is one, else a
/// place's (refcore's `leg_unit_static`). L §5.14.
fn leg_unit_static(m: &Reality, kind_unit: &str, from: &End, to: &End) -> Option<String> {
    if m.currencies.contains_key(kind_unit) || m.units.contains(kind_unit) {
        return Some(kind_unit.to_owned());
    }
    place_units(m, from, to).into_iter().next()
}

/// A leg's amount as a quantity of its unit: places of another unit are `LOAD-UNIT`, a
/// negative amount `RUN-NEG`, money finer than its minor unit `LOAD-UNIT` (refcore's
/// `leg_amount`). L §5.10, §5.14.
fn leg_amount(
    m: &Reality,
    inst_id: &str,
    leg: &str,
    kind_unit: &str,
    amount: &Value,
    from: &End,
    to: &End,
) -> Result<(Num, String), Stop> {
    let units = place_units(m, from, to);
    let (q, own) = quantity(m, amount)?;
    let unit = if m.currencies.contains_key(kind_unit) || m.units.contains(kind_unit) {
        kind_unit.to_owned()
    } else {
        let found = match (&own, units.iter().next()) {
            (Some(u), _) => Some(u.clone()),
            (None, Some(u)) if q.is_zero() && units.len() == 1 => Some(u.clone()),
            (None, _) => None,
        };
        let Some(u) = found else {
            return Err(refuse_load(
                Load::Unit,
                format!(
                    "the amount of leg {leg} of {inst_id} ({}) is in no unit of a place",
                    shown(m, amount)?
                ),
            ));
        };
        if kind_unit != "any"
            && m.kinds.get(kind_unit).is_some()
            && m.unit_kind_keys.get(&u).map(String::as_str) != Some(kind_unit)
            && !(kind_unit == "currencies" && m.currencies.contains_key(&u))
        {
            return Err(refuse_load(
                Load::Unit,
                format!(
                    "the amount of leg {leg} of {inst_id} is in {u}, not a unit of {kind_unit}"
                ),
            ));
        }
        u
    };
    if !units.is_empty() && (units.len() != 1 || !units.contains(&unit)) {
        let all: Vec<&str> = units.iter().map(String::as_str).collect();
        return Err(refuse_load(
            Load::Unit,
            format!(
                "leg {leg} of {inst_id} moves {unit} between places of {}",
                all.join(", ")
            ),
        ));
    }
    if let Some(u) = &own
        && *u != unit
    {
        return Err(refuse_load(
            Load::Unit,
            format!("the amount of leg {leg} of {inst_id} is in {u}, not {unit}"),
        ));
    }
    if q.is_negative() {
        return Err(refuse(
            Run::Neg,
            format!(
                "negative amount {} for leg {leg} of {inst_id}",
                shown(m, amount)?
            ),
        ));
    }
    if let Some(minor) = m.currencies.get(&unit)
        && !q.checked_div(minor).map_err(num_err)?.is_integer()
    {
        return Err(refuse_load(
            Load::Unit,
            format!(
                "the amount of leg {leg} of {inst_id} ({}) is not money of {unit}: round it",
                shown(m, amount)?
            ),
        ));
    }
    Ok((q, unit))
}

/// The units of a leg's places, but `any` (refcore's `leg_amount`, the places' units). L §5.14.
fn place_units(m: &Reality, from: &End, to: &End) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    for p in [from, to].into_iter().filter_map(End::place) {
        if let Some(u) = m.things.get(p).and_then(|t| m.place_unit(t))
            && u != "any"
        {
            out.insert(u);
        }
    }
    out
}
