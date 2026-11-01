//! Adjustments: a booked instance measured again at each later version's cut, each change of a leg recorded with the versions that made it.
//! Spec: E §5 (steps 1–5: the booked instance at its booking cut, each later step c with
//! t < c ≤ K, δ per leg, an adjustment recorded for δ ≠ 0 or a change of places, an adjustment
//! nothing settles is `HOLE-CHANGED` from its day); L §5.14.3 (adjustments, their names and
//! remainders), §8.1 (`HOLE-CHANGED`'s question), §8.2 (`adjustment:` in the register).
//! Twin: engine/tools/refcore/moves.py (`AdjMv`, `remainder`, `known_dates`, `adjust_move`,
//! `adjust_move_compute`, `step_versions`).
//! Invariants: a move's adjustments are computed once per cut, in the order of their day,
//! move, instance and leg; the versions known on a day are the versions', rows' and
//! observations' own; an adjustment's remainder is its change less its settlements, none once
//! waived (the links are `book/settle.rs`'s).

use std::collections::{BTreeMap, BTreeSet};

use crate::base::{Code, Refusal, Run};
use crate::eval::{Collected, Ctx, Engine};
use crate::hole::{Hole, spread};
use crate::model::{MoveInfo, Reality};
use crate::syntax::Stop;
use crate::time::Day;
use crate::walk::{Amount, End, Exists, Inst, LegVal, Seed, Status, invariant, qty_value};

/// One adjustment: an instance's leg changed by a later version (refcore's `AdjMv`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Adjustment {
    /// Its name, `adjustments[<inst>.<leg>, <version>]`, the first of the step's versions in
    /// text order (L §5.14.3, E §5 step 4).
    pub id: String,
    /// The adjusted instance's id.
    pub inst_id: String,
    /// The adjusted instance's keys.
    pub inst_keys: Vec<crate::model::KeyVal>,
    /// Its move.
    pub move_id: String,
    /// Its day.
    pub inst_day: Day,
    /// The leg, as booked or as the step gives it.
    pub leg: LegVal,
    /// The places the step gives the leg, when it changes them (L §5.14.3).
    pub new_ends: Vec<End>,
    /// The places the leg had before the step, when it changes them (L §5.14.3 How a
    /// settlement leg counts).
    pub before_ends: Vec<End>,
    /// The change δ: a quantity of the leg's unit, or a hole.
    pub delta: Amount,
    /// The step's day c, when the version became known.
    pub day: Day,
    /// The versions of the step, those the instance read first.
    pub vids: Vec<String>,
    /// What the step's evaluation met.
    pub coll: Collected,
    /// The movements that settle it, each leg with its sign (E §5 step 5).
    pub settled: Vec<Settled>,
    /// Whether a movement of a kind with no legs waives it (L §5.14.3).
    pub waived: bool,
}

/// A leg that settles an adjustment, or a waiver (refcore's `AdjMv.settled`). E §5 step 5.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settled {
    /// The settling movement's id.
    pub by: String,
    /// Its day.
    pub day: Option<Day>,
    /// +1 when the leg runs the adjusted leg's way, −1 the other way, 0 for a waiver or a leg
    /// that runs both ways.
    pub sign: i64,
    /// The leg's amount; 0 for a waiver.
    pub amount: Amount,
}

impl Adjustment {
    /// What remains of the change: δ less the settlements; none once waived (refcore's
    /// `AdjMv.remainder`). E §5 step 5.
    ///
    /// # Errors
    ///
    /// A settlement's refusal; a number error.
    pub fn remainder(&self) -> Result<Option<Amount>, Stop> {
        if self.waived {
            return Ok(None);
        }
        let mut r = self.delta.clone();
        for s in &self.settled {
            r = match (r, &s.amount) {
                (Amount::Hole(a), Amount::Hole(b)) => Amount::Hole(
                    spread(&[Some(&a), Some(b)]).ok_or_else(|| invariant("a hole remainder"))?,
                ),
                (Amount::Hole(a), Amount::Qty(_) | Amount::Refused(_)) => Amount::Hole(a),
                (Amount::Qty(_) | Amount::Refused(_), Amount::Hole(b)) => Amount::Hole(b.clone()),
                (Amount::Qty(_) | Amount::Refused(_), Amount::Refused(e)) => {
                    return Err(Stop::Refused(e.clone()));
                }
                (Amount::Refused(e), Amount::Qty(_)) => return Err(Stop::Refused(e)),
                (Amount::Qty(q), Amount::Qty(x)) => {
                    let part = x
                        .checked_mul(&crate::num::Num::int(s.sign))
                        .map_err(crate::eval::num_stop)?;
                    Amount::Qty(q.checked_sub(&part).map_err(crate::eval::num_stop)?)
                }
            };
        }
        Ok(Some(r))
    }

    /// `HOLE-CHANGED`'s hole: the instance changed after it was paid (L §8.1, E §5 step 5).
    #[must_use]
    pub fn question(&self) -> Hole {
        Hole::asked(
            &format!(
                "{} on {} changed after it was paid (version {}); is back pay due, and when?",
                self.inst_id,
                self.inst_day.text(),
                self.vids.join(", ")
            ),
            &self.inst_id,
        )
    }

    /// The register entry's text, `<inst> by <δ> on <day> (version <v>), remaining <r>`; none
    /// when nothing remains (L §8.2 `adjustment:`, E §5 step 5).
    ///
    /// # Errors
    ///
    /// A number error printing the change.
    pub fn remainder_text(&self, m: &Reality) -> Result<Option<String>, Stop> {
        let Some(left) = self.remainder()? else {
            return Ok(None);
        };
        if let Amount::Qty(q) = &left
            && q.is_zero()
        {
            return Ok(None);
        }
        Ok(Some(format!(
            "{} by {} on {} (version {}), remaining {}",
            self.inst_id,
            self.amount_text(m, &self.delta)?,
            self.day.text(),
            self.vids.join(", "),
            self.amount_text(m, &left)?
        )))
    }

    /// An amount of the adjusted leg's unit as the register prints it (L §8.2).
    fn amount_text(&self, m: &Reality, a: &Amount) -> Result<String, Stop> {
        match a {
            Amount::Qty(q) => {
                let unit = self
                    .leg
                    .unit
                    .as_deref()
                    .ok_or_else(|| invariant("an adjustment with no unit"))?;
                qty_value(m, q, unit)?
                    .text(m)
                    .map_err(crate::eval::num_stop)
            }
            Amount::Hole(h) => Ok(h.text()),
            Amount::Refused(r) => Err(Stop::Refused(r.clone())),
        }
    }
}

impl Engine {
    /// The adjustments of one plan move's booked instances at cut `cut`, kept per move and cut
    /// (refcore's `adjust_move`). E §5 steps 1–4.
    ///
    /// # Errors
    ///
    /// `RUN-CYCLE` for adjustments that read themselves; the refusals of the measuring;
    /// `NotBuilt` for a version that answers a hole movement.
    pub(crate) fn adjust_move(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        cut: Day,
    ) -> Result<Vec<Adjustment>, Stop> {
        let key = (mv.id.clone(), cut);
        if let Some(hit) = self.adjustments.get(&key) {
            return Ok(hit.clone());
        }
        if super::settle::settlement_move(m, mv)? {
            self.adjustments.insert(key, Vec::new());
            return Ok(Vec::new());
        }
        if self.adjust_active.contains(&key) {
            return Err(Stop::Refused(Refusal::new(
                Code::Run(Run::Cycle),
                format!("the adjustments of {} read themselves", mv.id),
            )));
        }
        self.adjust_active.insert(key.clone());
        let made = self.measure_move(m, mv, cut);
        self.adjust_active.remove(&key);
        let out = made?;
        self.adjustments.insert(key, out.clone());
        Ok(out)
    }

    /// Each booked instance before the cut measured at each later step (refcore's
    /// `adjust_move_compute`). E §5 steps 1–4.
    fn measure_move(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        cut: Day,
    ) -> Result<Vec<Adjustment>, Stop> {
        let steps_all = known_dates(m);
        let mut out = Vec::new();
        for inst in self.move_instances(m, mv, cut)? {
            if !matches!(inst.status, Status::Past | Status::Hole | Status::None) {
                continue;
            }
            let Some(t) = inst.day.as_ref().and_then(crate::walk::Dated::day) else {
                continue;
            };
            if t >= cut {
                continue;
            }
            let steps: Vec<&(Day, BTreeSet<String>)> = steps_all
                .iter()
                .filter(|(c, _)| t < *c && *c <= cut)
                .collect();
            if steps.is_empty() {
                continue;
            }
            out.extend(self.measure_inst(m, mv, (&inst, t), &steps, cut)?);
        }
        out.sort_by(|a, b| {
            (a.day, &a.move_id, &a.inst_id, &a.leg.name).cmp(&(
                b.day,
                &b.move_id,
                &b.inst_id,
                &b.leg.name,
            ))
        });
        Ok(out)
    }

    /// One instance measured at each step: δ per leg against the step before (refcore's
    /// `adjust_move_compute`, its steps). E §5 steps 1–4.
    fn measure_inst(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        (inst, t): (&Inst, Day),
        steps: &[&(Day, BTreeSet<String>)],
        cut: Day,
    ) -> Result<Vec<Adjustment>, Stop> {
        let mut out = Vec::new();
        let mut prev_legs = inst.legs.clone();
        let mut prev_ex = inst.exists == Exists::Yes;
        let hole0 = matches!(inst.exists, Exists::Unknown(_)) || inst.legs.iter().any(is_hole_leg);
        for (c, vids) in steps.iter().map(|s| (s.0, &s.1)) {
            let seed = Seed {
                env: inst.env.clone(),
                keys: inst.keys.clone(),
                exists: Exists::Yes,
                at_cut: Some(c),
            };
            let Some(j) = self.eval_inst(m, mv, &seed, Ctx::new(None, cut))? else {
                continue;
            };
            if j.err.is_some() {
                continue;
            }
            if hole0 {
                if matches!(j.exists, Exists::Unknown(_)) || j.legs.iter().any(is_hole_leg) {
                    continue;
                }
                return Err(Stop::NotBuilt(
                    "a version that answers a hole movement (E §5 step 1)",
                ));
            }
            if matches!(j.exists, Exists::Unknown(_)) {
                continue;
            }
            let ex = j.exists == Exists::Yes;
            let before: BTreeMap<&str, &LegVal> = if prev_ex {
                prev_legs.iter().map(|lg| (lg.name.as_str(), lg)).collect()
            } else {
                BTreeMap::new()
            };
            let after: BTreeMap<&str, &LegVal> = if ex {
                j.legs.iter().map(|lg| (lg.name.as_str(), lg)).collect()
            } else {
                BTreeMap::new()
            };
            let mut names: Vec<&str> = Vec::new();
            let all = inst
                .legs
                .iter()
                .chain(before.values().copied())
                .chain(after.values().copied());
            for lg in all {
                if !names.contains(&lg.name.as_str()) {
                    names.push(lg.name.as_str());
                }
            }
            let base = if inst.legs.is_empty() {
                &j.legs
            } else {
                &inst.legs
            };
            let pool: Vec<&LegVal> = base.iter().chain(before.values().copied()).collect();
            let step = Step {
                inst_id: inst.id.clone(),
                inst_keys: inst.keys.clone(),
                move_id: inst.move_id.clone(),
                inst_day: t,
                day: c,
                vids: step_versions(&j.coll, vids),
                coll: j.coll.clone(),
            };
            for ln in names {
                if let Some(adj) = leg_change(
                    &step,
                    &pool,
                    before.get(ln).copied(),
                    after.get(ln).copied(),
                    ln,
                )? {
                    out.push(adj);
                }
            }
            prev_legs.clone_from(&j.legs);
            prev_ex = ex;
        }
        Ok(out)
    }
}

/// One step of an instance's measuring: the instance, the step's day and versions, what its
/// evaluation met.
struct Step {
    /// The instance measured, by id.
    inst_id: String,
    /// Its keys.
    inst_keys: Vec<crate::model::KeyVal>,
    /// Its move.
    move_id: String,
    /// Its day.
    inst_day: Day,
    /// The step's day c.
    day: Day,
    /// Its versions.
    vids: Vec<String>,
    /// What it met.
    coll: Collected,
}

/// The adjustment of one leg at one step: δ between the step before and this one, recorded for
/// δ ≠ 0, a hole, or a change of places (refcore's `adjust_move_compute`, a leg). E §5 steps
/// 3–4, L §5.14.3.
fn leg_change(
    step: &Step,
    pool: &[&LegVal],
    a: Option<&LegVal>,
    b: Option<&LegVal>,
    name: &str,
) -> Result<Option<Adjustment>, Stop> {
    let qa = a.map(|lg| &lg.amount);
    let qb = b.map(|lg| &lg.amount);
    if matches!(qa, Some(Amount::Refused(_))) || matches!(qb, Some(Amount::Refused(_))) {
        return Ok(None);
    }
    let Some(either) = b.or(a) else {
        return Ok(None);
    };
    let leg = pool
        .iter()
        .find(|lg| lg.name == name)
        .map(|lg| (*lg).clone())
        .ok_or_else(|| invariant("an adjusted leg in no pool"))?;
    let ha = qa.and_then(hole_of);
    let hb = qb.and_then(hole_of);
    if ha.is_some() || hb.is_some() {
        let h = spread(&[ha, hb]).ok_or_else(|| invariant("a hole change"))?;
        return Ok(Some(make(
            step,
            leg,
            Amount::Hole(h),
            (Vec::new(), Vec::new()),
        )));
    }
    if either.unit.is_none() {
        return Ok(None);
    }
    let da = qty_or_zero(qa);
    let db = qty_or_zero(qb);
    let moved = match (a, b) {
        (Some(x), Some(y)) => (&x.from, &x.to) != (&y.from, &y.to),
        (Some(_) | None, None) | (None, Some(_)) => false,
    };
    if db == da && !moved {
        return Ok(None);
    }
    let delta = db.checked_sub(&da).map_err(crate::eval::num_stop)?;
    let ends = match (moved, a, b) {
        (true, Some(x), Some(y)) => (
            vec![y.from.clone(), y.to.clone()],
            vec![x.from.clone(), x.to.clone()],
        ),
        (true, _, None | Some(_)) | (false, _, _) => (Vec::new(), Vec::new()),
    };
    let mut leg = leg;
    leg.unit.clone_from(&either.unit);
    Ok(Some(make(step, leg, Amount::Qty(delta), ends)))
}

/// An adjustment of `leg` by `delta` at a step, named by the step's versions, the first in
/// text order (E §5 step 4, L §5.14.3); for a change of places, the places the step gives the
/// leg and those it had before.
fn make(
    step: &Step,
    leg: LegVal,
    delta: Amount,
    (new_ends, before_ends): (Vec<End>, Vec<End>),
) -> Adjustment {
    let first = step.vids.first().map_or("", String::as_str);
    Adjustment {
        id: format!("adjustments[{}.{}, {first}]", step.inst_id, leg.name),
        inst_id: step.inst_id.clone(),
        inst_keys: step.inst_keys.clone(),
        before_ends,
        settled: Vec::new(),
        waived: false,
        move_id: step.move_id.clone(),
        inst_day: step.inst_day,
        leg,
        new_ends,
        delta,
        day: step.day,
        vids: step.vids.clone(),
        coll: step.coll.clone(),
    }
}

/// A leg's amount as a quantity, 0 where the leg isn't there (E §5 step 1: an instance that
/// doesn't exist moves 0).
fn qty_or_zero(a: Option<&Amount>) -> crate::num::Num {
    match a {
        Some(Amount::Qty(q)) => q.clone(),
        Some(Amount::Hole(_) | Amount::Refused(_)) | None => crate::num::Num::zero(),
    }
}

/// A leg amount's hole, when it is one. E §5 step 3.
fn hole_of(a: &Amount) -> Option<&Hole> {
    match a {
        Amount::Hole(h) => Some(h),
        Amount::Qty(_) | Amount::Refused(_) => None,
    }
}

/// Whether a leg's amount is a hole. E §5 step 1.
fn is_hole_leg(lg: &LegVal) -> bool {
    matches!(lg.amount, Amount::Hole(_))
}

/// The days versions became known, each with the versions known then: versions' `known`,
/// rows' own (refcore's `known_dates`). E §5 step 2.
fn known_dates(m: &Reality) -> Vec<(Day, BTreeSet<String>)> {
    let mut by: BTreeMap<Day, BTreeSet<String>> = BTreeMap::new();
    for (_, v) in m.versions.entries() {
        by.entry(v.known)
            .or_insert_with(BTreeSet::new)
            .insert(v.id.clone());
    }
    for (_, input) in m.inputs.entries() {
        for row in &input.rows {
            if let Some(k) = row.own_known {
                by.entry(k)
                    .or_insert_with(BTreeSet::new)
                    .insert(row.version.clone());
            }
        }
        for o in &input.observations {
            if let Some(k) = o.own_known {
                by.entry(k)
                    .or_insert_with(BTreeSet::new)
                    .insert(o.version.clone());
            }
        }
    }
    by.into_iter().collect()
}

/// The versions a step names: those of its versions the instance read, else all of them, in
/// text order (refcore's `step_versions`). E §5 step 4.
fn step_versions(coll: &Collected, vids: &BTreeSet<String>) -> Vec<String> {
    let hit: Vec<String> = vids.intersection(&coll.versions).cloned().collect();
    if hit.is_empty() {
        return vids.iter().cloned().collect();
    }
    hit
}
