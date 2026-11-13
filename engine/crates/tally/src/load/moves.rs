//! Load's checker, its moves: a leg's places, their kinds and units, its amount; a decision's balance of its own day; a settlement's unit.
//! Spec: L §5.14 (a leg: places, amount of the leg's unit, its day), §5.10 (no leg to or from
//! an asset), §5.7 (a leg of a unit kind), §5.14.1 (a decision reads no balance of its own day
//! or later without a projection), §5.14.3 and §5.19 (a settlement in the adjusted leg's
//! unit), §8.3 (`LOAD-UNIT`, `-KIND`, `-CYCLE`).
//! Twin: engine/tools/refcore/check3.py (`m_check_leg`, `m_check_move`, its `LOAD-CYCLE` scan
//! of `bal(x, d)`); moves.py (`settle_sign`, its `LOAD-UNIT`, which refcore meets only when a
//! read settles the adjustment: the spec rules it a load refusal, issue #72).
//! Invariants: a leg is checked in refcore's order, its static amount type kept for `flows`
//! and the settlements; the settlements are checked after every other check of the walk, as
//! refcore meets them after load; a type load doesn't see refuses nothing.

use super::check::{Checker, DayRead, Scope};
use super::reads::is_place_type;
use super::same::instance_text;
use super::targets::literal_place;
use super::texts::{children, text};
use super::types::{Typed, numcls, refused, uerr, unit_type};
use crate::base::{Code, Load, Refusal};
use crate::model::{Fulfils, MoveInfo, NameKind, Reality, Ty};
use crate::syntax::{AddOp, Bal, Expr, Leg, Lit, Stop};
use std::collections::{BTreeMap, BTreeSet};

impl Checker {
    /// A leg: its places, each a place and none an asset, of one unit, the leg's own;
    /// its amount of the leg's unit, or of a unit of its unit kind; its own day; the leg's
    /// static amount type returned (refcore's `m_check_leg`). L §5.14, §5.10, §5.7.
    pub(super) fn check_leg(
        &mut self,
        m: &Reality,
        scope: &Scope,
        mv: &MoveInfo,
        leg: &Leg,
        unit: &str,
    ) -> Typed {
        let places = Ty::Key("places".to_owned());
        let assets = m.subtree("assets");
        let mut ends: Vec<String> = Vec::new();
        for (side, end) in [("from", &leg.from), ("to", &leg.to)] {
            let t = self.expr(m, scope, end, Some(&places), 0)?;
            if matches!(t, Ty::Hole) {
                continue;
            }
            if !is_place_type(m, &t) {
                return Err(scope.place(uerr(format!(
                    "leg {} of {}: {side} is {}, not a place",
                    leg.name,
                    mv.id,
                    t.show()
                ))));
            }
            let asset_kind = matches!(&t, Ty::Key(ix) if assets.contains(ix));
            let thing = literal_place(m, scope, end).map_err(|s| scope.place(s))?;
            let asset_thing = thing
                .as_ref()
                .and_then(|name| m.things.get(name))
                .and_then(|th| th.base.as_ref())
                .is_some_and(|base| assets.contains(base));
            if asset_kind || asset_thing {
                return Err(scope.place(refused(
                    Load::Kind,
                    format!("a leg to or from an asset ({} of {})", leg.name, mv.id),
                )));
            }
            ends.extend(thing);
        }
        let mut units = BTreeSet::new();
        for name in &ends {
            if let Some(held) = m.things.get(name).and_then(|thing| m.place_unit(thing))
                && held != "any"
            {
                units.insert(held);
            }
        }
        if units.len() > 1 {
            let held: Vec<&str> = units.iter().map(String::as_str).collect();
            return Err(scope.place(uerr(format!(
                "leg {} of {} runs between places of different units ({})",
                leg.name,
                mv.id,
                held.join(", ")
            ))));
        }
        let held = units.first().cloned();
        let fixed = m.currencies.contains_key(unit) || m.units.contains(unit);
        if unit != "any"
            && fixed
            && let Some(held) = &held
            && held != unit
        {
            return Err(scope.place(uerr(format!(
                "leg {} of {} is in {unit}; its places hold {held}",
                leg.name, mv.id
            ))));
        }
        let mut lt = unit_type(m, Some(unit)).map_err(|s| scope.place(s))?;
        if let (Ty::Dyn, Some(held)) = (&lt, &held) {
            lt = unit_type(m, Some(held)).map_err(|s| scope.place(s))?;
        }
        let unit_kind = (unit != "any" && unit != "one")
            && m.names.kind(unit) == Some(NameKind::Kind)
            && !m.units.contains(unit)
            && !m.currencies.contains_key(unit);
        let what = format!("the amount of leg {} of {}", leg.name, mv.id);
        let expect = (!matches!(lt, Ty::Dyn) && !unit_kind).then_some(&lt);
        let t = self.expr(m, scope, &leg.amount, expect, 0)?;
        self.leg_amounts
            .insert((mv.id.clone(), leg.name.clone()), t.clone());
        if unit_kind && !unit_kind_amount(m, unit, &t) {
            return Err(scope.place(uerr(format!(
                "{what} is {}, not a unit of {unit}",
                t.show()
            ))));
        }
        if !matches!(lt, Ty::Dyn) && !unit_kind {
            self.assign(m, scope, &lt, &leg.amount, &t, &what)
                .map_err(|stop| scope.place(stop))?;
        } else if !(matches!(numcls(m, &t), Some("money" | "num" | "int" | "flex"))
            || matches!(t, Ty::Dyn | Ty::Hole | Ty::Unseen))
        {
            return Err(scope.place(uerr(format!("{what} is {}", t.show()))));
        }
        if let Some(on) = &leg.on {
            let dt = self.expr(m, scope, on, None, 0)?;
            if !super::types::is_date(m, &dt) {
                return Err(scope.place(uerr(format!(
                    "on of leg {} of {} is {}, not a date",
                    leg.name,
                    mv.id,
                    dt.show()
                ))));
            }
        }
        Ok(lt)
    }
}

/// Whether `t` is an amount of a leg of unit kind `kind`: money of a declared currency for
/// `currencies`, a quantity of one of the kind's units, or a type load doesn't see (refcore's
/// `m_check_leg`, its `ukind` arm). L §5.7, §5.14.
fn unit_kind_amount(m: &Reality, kind: &str, t: &Ty) -> bool {
    if kind == "currencies"
        && let Ty::Money(cur) = t
        && m.currencies.contains_key(cur)
    {
        return true;
    }
    if matches!(t, Ty::Dyn | Ty::Hole | Ty::Flex(_) | Ty::Unseen) {
        return true;
    }
    let Ty::Num(unit, _) = t else {
        return false;
    };
    let sub = m.subtree(kind);
    matches!(unit.terms(), [(base, 1)] if m.unit_kind_keys.get(base).is_some_and(|k| sub.contains(k)))
}

/// `LOAD-CYCLE` for a decision whose `when`, legs or attributes read `bal(x, d)` of its own
/// day (`d` written as its `on`) without `start`, or of its `on` plus a positive figure
/// (refcore's `m_check_move`, its scan of the plain balances); the others, to fold where load
/// sees their days (its `cycle_cands`). L §5.14.1.
///
/// # Errors
///
/// That refusal, placed by the caller at the move's line.
pub(super) fn own_day_reads(mv: &MoveInfo) -> Result<Vec<DayRead>, Stop> {
    let decl = &mv.decl;
    let on = text(&decl.on);
    let mut exprs: Vec<&Expr> = Vec::new();
    exprs.extend(decl.when.as_ref());
    exprs.extend(decl.legs.iter().map(|leg| &leg.amount));
    exprs.extend(decl.legs.iter().map(|leg| &leg.from));
    exprs.extend(decl.legs.iter().map(|leg| &leg.to));
    exprs.extend(decl.attrs.iter().map(|(_, value)| value));
    let mut found = Vec::new();
    for e in exprs {
        let mut todo = vec![e];
        while let Some(x) = todo.pop() {
            if let Expr::Bal(bal) = x {
                found.extend(own_day_bal(mv, bal, &on)?);
            }
            let mut below = children(x);
            below.reverse();
            todo.extend(below);
        }
    }
    Ok(found)
}

/// One plain `bal(x, d)` of a decision: `LOAD-CYCLE` when `d` is its own day without `start`,
/// or a later one; otherwise the read, for the fold.
fn own_day_bal(mv: &MoveInfo, bal: &Bal, on: &str) -> Result<Option<DayRead>, Stop> {
    let Some(day) = bal.args.last() else {
        return Ok(None);
    };
    if bal.modelled || bal.proj.from.is_some() {
        return Ok(None);
    }
    let later = later_day(day, on);
    if !((text(day) == on && !bal.start) || later) {
        return Ok(Some(DayRead {
            id: mv.id.clone(),
            place: bal.place.clone(),
            day: day.clone(),
            start: bal.start,
        }));
    }
    let when = if later { "a later day" } else { "its own day" };
    Err(refused(
        Load::Cycle,
        format!(
            "{} reads bal({}, {}), {when}: its own result or its future (write the projection)",
            mv.id,
            text(&bal.place),
            text(day)
        ),
    ))
}

/// Whether `day` is written as the move's `on` plus a positive figure, `on + 1 day`.
fn later_day(day: &Expr, on: &str) -> bool {
    let Expr::Sum(first, rest) = day else {
        return false;
    };
    let Some(((AddOp::Add, Expr::Lit(Lit::Num(n))), head)) = rest.split_last() else {
        return false;
    };
    let left = if head.is_empty() {
        text(first)
    } else {
        text(&Expr::Sum(first.clone(), head.to_vec()))
    };
    left == on && !n.value.is_negative() && !n.value.is_zero()
}

/// `LOAD-UNIT` for an observed row that settles an adjustment, `fulfils:
/// adjustments[m[keys].leg, V]`, with a leg in another unit than the leg it settles, where
/// load sees that leg's unit (refcore's `settle_sign`, at load by the spec's ruling, issue
/// #72). L §5.14.3, §5.19, §8.3.
///
/// # Errors
///
/// That refusal, with no place, as refcore prints it.
pub(super) fn settlements(
    m: &Reality,
    leg_types: &BTreeMap<(String, String), Ty>,
) -> Result<(), Stop> {
    for (_, input) in m.inputs.entries() {
        for obs in &input.observations {
            let Some(Fulfils::Adjustment(mid, keys, leg, vid)) = &obs.fulfils else {
                continue;
            };
            let Some(adjusted) = leg_types
                .get(&(mid.clone(), leg.clone()))
                .and_then(static_unit)
            else {
                continue;
            };
            let Some(settling) = obs.legs.iter().find(|lg| lg.unit != adjusted) else {
                continue;
            };
            let iid = instance_text(mid, keys);
            return Err(Stop::Refused(Refusal::new(
                Code::Load(Load::Unit),
                format!(
                    "settlement {} of adjustments[{iid}.{leg}, {vid}] is in {}, not {adjusted}",
                    obs.sid(),
                    settling.unit
                ),
            )));
        }
    }
    Ok(())
}

/// The one unit a leg's static amount type names: a currency, or a unit to the first power.
fn static_unit(t: &Ty) -> Option<String> {
    if let Ty::Money(cur) = t {
        return Some(cur.clone());
    }
    let Ty::Num(unit, _) = t else {
        return None;
    };
    let [(base, 1)] = unit.terms() else {
        return None;
    };
    Some(base.clone())
}
