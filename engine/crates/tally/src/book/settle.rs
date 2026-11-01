//! Settlements: the movements that settle or waive an adjustment, each leg signed by the way
//! it runs, and every adjustment of a cut with its settlements linked.
//! Spec: E §5 steps 2 and 5 (a settlement isn't measured; settlements and waivers linked to
//! their adjustments; an adjustment nothing settles is `HOLE-CHANGED`), L §5.14.3 (a
//! settlement's `fulfils`, how a settlement leg counts, a waiver is a movement of a kind with
//! no legs), §5.19 (observed rows that fulfil an adjustment; `RUN-RECORD` for one no version
//! made by the cut).
//! Twin: engine/tools/refcore/moves.py (`settlement_move`, `adjustments`, `adjust_all`,
//! `settle`, `settle_sign`, `settle_candidates`) and check3.py (`fulfils_adj`).
//! Invariants: each plan move's adjustments are settled once per cut; a settlement move's
//! candidates bound to the adjustments being settled are made apart from its instances, never
//! kept as them; a waiver leaves no remainder.

use std::collections::BTreeSet;

use super::adjust::{Adjustment, Settled};
use super::observe::inst_id;
use crate::base::{Code, Load, Refusal, Run};
use crate::eval::Engine;
use crate::model::{Domain, Fulfils, MoveInfo, Reality, Value};
use crate::num::Num;
use crate::syntax::{Expr, Stop};
use crate::time::Day;
use crate::walk::{Amount, End, Exists, Inst, LegVal, invariant};

impl Engine {
    /// Every adjustment cut `cut` sees, unsettled: each plan move's but a settlement's, in
    /// the order of their day, move, instance and leg (refcore's `adjustments` with
    /// `settled=False`, `adjust_all`). E §5 steps 2–4.
    ///
    /// # Errors
    ///
    /// `RUN-CYCLE` for a measured move that reads adjustments; the refusals of the measuring.
    pub(crate) fn unsettled_adjustments(
        &mut self,
        m: &Reality,
        cut: Day,
    ) -> Result<Vec<Adjustment>, Stop> {
        if let Some(hit) = self.unsettled.get(&cut) {
            return Ok(hit.clone());
        }
        if self.adjust_all_active.contains(&cut) {
            return Err(Stop::Refused(Refusal::new(
                Code::Run(Run::Cycle),
                "the adjustments read themselves (a measured move reads adjustments)".to_owned(),
            )));
        }
        self.adjust_all_active.insert(cut);
        let made = self.measure_all(m, cut);
        self.adjust_all_active.remove(&cut);
        let out = made?;
        self.unsettled.insert(cut, out.clone());
        Ok(out)
    }

    /// Each plan move's adjustments but a settlement's, sorted (refcore's `adjust_all`).
    /// E §5 step 2.
    fn measure_all(&mut self, m: &Reality, cut: Day) -> Result<Vec<Adjustment>, Stop> {
        let mut out = Vec::new();
        for mv in plan_moves(m) {
            if settlement_move(m, mv)? {
                continue;
            }
            out.extend(self.adjust_move(m, mv, cut)?);
        }
        sort_adjustments(&mut out);
        Ok(out)
    }

    /// Every adjustment cut `cut` sees with its settlements and waivers linked; an observed
    /// row naming an adjustment no version made by the cut refused `RUN-RECORD` (refcore's
    /// `adjustments`). E §5 step 5, L §5.19 Links.
    ///
    /// # Errors
    ///
    /// As [`Engine::unsettled_adjustments`]; a settlement leg refused.
    pub(crate) fn all_adjustments(
        &mut self,
        m: &Reality,
        cut: Day,
    ) -> Result<Vec<Adjustment>, Stop> {
        self.unsettled_adjustments(m, cut)?;
        let mut out = Vec::new();
        for mv in plan_moves(m) {
            if settlement_move(m, mv)? {
                continue;
            }
            out.extend(self.settled_move(m, mv, cut)?);
        }
        sort_adjustments(&mut out);
        let mut names: BTreeSet<(String, String, String)> = BTreeSet::new();
        for a in &out {
            for vid in &a.vids {
                names.insert((a.inst_id.clone(), a.leg.name.clone(), vid.clone()));
            }
        }
        for o in self.observations(m, cut)?.to_vec() {
            let Some(Fulfils::Adjustment(mid, keys, leg, vid)) = &o.fulfils else {
                continue;
            };
            let iid = inst_id(mid, keys);
            if names.contains(&(iid.clone(), leg.clone(), vid.clone()))
                || self.obs_err(m, &o, cut)?.is_some()
            {
                continue;
            }
            let err = Refusal::new(
                Code::Run(Run::Record),
                format!(
                    "{} fulfils adjustments[{iid}.{leg}, {vid}], which no version made by the cut",
                    o.id
                ),
            );
            self.obs_errs.insert((cut, o.id.clone()), err);
        }
        Ok(out)
    }

    /// One plan move's adjustments with their settlements linked, kept per move and cut
    /// (refcore's `adjust_move` then `settle`). E §5 step 5.
    ///
    /// # Errors
    ///
    /// As [`Engine::adjust_move`]; a settlement leg refused.
    pub(crate) fn settled_move(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        cut: Day,
    ) -> Result<Vec<Adjustment>, Stop> {
        let key = (mv.id.clone(), cut);
        if let Some(hit) = self.settled.get(&key) {
            return Ok(hit.clone());
        }
        let mut own = self.adjust_move(m, mv, cut)?;
        self.settle(m, &mut own, cut)?;
        self.settled.insert(key, own.clone());
        Ok(own)
    }

    /// Links the settlements and waivers of `adjs`: each settlement move's instance whose
    /// `fulfils` names one, then each observed row that names one (refcore's `settle`).
    /// E §5 step 5, L §5.14.3, §5.19.
    fn settle(&mut self, m: &Reality, adjs: &mut [Adjustment], cut: Day) -> Result<(), Stop> {
        if adjs.is_empty() {
            return Ok(());
        }
        for mv in plan_moves(m) {
            if mv.decl.fulfils.is_none() {
                continue;
            }
            for s in self.settle_candidates(m, mv, adjs, cut)? {
                link_instance(m, mv, &s, adjs)?;
            }
        }
        let observed = self.observations(m, cut)?.to_vec();
        for a in adjs.iter_mut() {
            for vid in a.vids.clone() {
                link_observed(a, &vid, &observed)?;
            }
        }
        Ok(())
    }

    /// The instances of settlement move `mv` that may settle `adjs`: when its `fulfils` is the
    /// variable of its one binding over adjustments, only those bound to `adjs`, made apart;
    /// otherwise its instances (refcore's `settle_candidates`). E §5 step 5, E §4.1.
    fn settle_candidates(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        adjs: &[Adjustment],
        cut: Day,
    ) -> Result<Vec<Inst>, Stop> {
        if !bound_to_adjustments(m, mv) {
            return self.move_instances(m, mv, cut);
        }
        let key = (format!("{}, its settlements", mv.id), cut);
        if self.plan_active.contains(&key) {
            return Err(Stop::Refused(Refusal::new(
                Code::Run(Run::Cycle),
                format!("{} reads its own instances", mv.id),
            )));
        }
        self.plan_active.insert(key.clone());
        let before = self.adj_restrict.replace(adjs.to_vec());
        let made = self.make_instances(m, mv, cut);
        self.adj_restrict = before;
        self.plan_active.remove(&key);
        made
    }
}

/// The plan moves, by id (refcore's `plan_moves`). E §4.1.
fn plan_moves(m: &Reality) -> Vec<&MoveInfo> {
    let mut out: Vec<&MoveInfo> = m
        .moves
        .entries()
        .map(|(_, mv)| mv)
        .filter(|mv| !mv.decision)
        .collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

/// Whether `mv` is a settlement, never measured: its `fulfils` names an adjustment, the
/// variable of a binding over a kind adjustments is of (refcore's `settlement_move`,
/// `fulfils_adj`). E §5 step 2, L §5.14.3.
///
/// # Errors
///
/// `NotBuilt` for a `fulfils` that is any other expression.
pub(crate) fn settlement_move(m: &Reality, mv: &MoveInfo) -> Result<bool, Stop> {
    let Some(f) = &mv.decl.fulfils else {
        return Ok(false);
    };
    let Expr::Name(var) = f else {
        return Err(Stop::NotBuilt(
            "a move's fulfils that isn't the variable of its binding over adjustments",
        ));
    };
    let bound = mv.binds.iter().find(|b| b.var == *var);
    match bound.map(|b| &b.domain) {
        Some(Domain::Kind(k)) if m.subtree("adjustments").contains(k) => Ok(true),
        _ => Err(Stop::NotBuilt(
            "a move's fulfils that isn't the variable of its binding over adjustments",
        )),
    }
}

/// Whether a settlement move's `fulfils` is the variable of its one binding over a kind whose
/// subtree holds adjustments, so its candidates are those bound to the adjustments settled
/// (refcore's `settle_candidates`, its test). E §5 step 5.
fn bound_to_adjustments(m: &Reality, mv: &MoveInfo) -> bool {
    let over: Vec<&str> = mv
        .binds
        .iter()
        .filter(|b| match &b.domain {
            Domain::Kind(k) => m.subtree(k).contains("adjustments"),
            Domain::Index(_) | Domain::Axis(_) | Domain::List(_) | Domain::Range => false,
        })
        .map(|b| b.var.as_str())
        .collect();
    match (&mv.decl.fulfils, over.as_slice()) {
        (Some(Expr::Name(var)), [only]) => var == only,
        _ => false,
    }
}

/// The order of adjustments: day, move, instance, leg (refcore's `AdjMv.order`). E §5.
fn sort_adjustments(out: &mut [Adjustment]) {
    out.sort_by(|a, b| {
        (a.day, &a.move_id, &a.inst_id, &a.leg.name).cmp(&(
            b.day,
            &b.move_id,
            &b.inst_id,
            &b.leg.name,
        ))
    });
}

/// A settlement move's instance linked to the adjustment its `fulfils` names, when that is
/// one of `adjs` and the instance exists (refcore's `settle`, its moves). E §5 step 5.
fn link_instance(
    m: &Reality,
    mv: &MoveInfo,
    s: &Inst,
    adjs: &mut [Adjustment],
) -> Result<(), Stop> {
    if s.exists != Exists::Yes || s.err.is_some() {
        return Ok(());
    }
    let Some(Value::Mv(named)) = &s.fulfils else {
        return Ok(());
    };
    let Some(a) = adjs.iter_mut().find(|a| a.id == named.id) else {
        return Ok(());
    };
    let day = s.day.as_ref().and_then(crate::walk::Dated::day);
    if m.kind_legs(&mv.kind).is_empty() {
        waive(a, &s.id, day);
        return Ok(());
    }
    link(a, &s.id, day, &s.legs)
}

/// The observed rows that name adjustment `a` as `adjustments[<inst>.<leg>, <vid>]` linked
/// to it: a row with no legs waives it (refcore's `settle`, its observed rows). E §5 step 5,
/// L §5.14.3, §5.19.
fn link_observed(
    a: &mut Adjustment,
    vid: &str,
    observed: &[super::observe::Obs],
) -> Result<(), Stop> {
    for o in observed {
        let Some(Fulfils::Adjustment(mid, keys, leg, v)) = &o.fulfils else {
            continue;
        };
        if v != vid || *leg != a.leg.name || inst_id(mid, keys) != a.inst_id {
            continue;
        }
        if o.legs.is_empty() {
            waive(a, &o.id, Some(o.day));
            continue;
        }
        link(a, &o.id, Some(o.day), &o.legs)?;
    }
    Ok(())
}

/// A settling movement's legs added to adjustment `a`, each with its sign (refcore's
/// `settle`). E §5 step 5, L §5.14.3.
fn link(a: &mut Adjustment, by: &str, day: Option<Day>, legs: &[LegVal]) -> Result<(), Stop> {
    for lg in legs {
        let sign = settle_sign(lg, a, by)?;
        a.settled.push(Settled {
            by: by.to_owned(),
            day,
            sign,
            amount: lg.amount.clone(),
        });
    }
    Ok(())
}

/// A waiver: the adjustment settled with nothing left (L §5.14.3, E §5 step 5).
fn waive(a: &mut Adjustment, by: &str, day: Option<Day>) {
    a.waived = true;
    a.settled.push(Settled {
        by: by.to_owned(),
        day,
        sign: 0,
        amount: Amount::Qty(Num::zero()),
    });
}

/// The sign of a settlement leg: +1 when it runs the adjusted leg's way, −1 the other way, 0
/// for a leg between two of a moved leg's `from` or `to` places (refcore's `settle_sign`).
/// E §5 step 5, L §5.14.3 How a settlement leg counts.
fn settle_sign(lg: &LegVal, a: &Adjustment, by: &str) -> Result<i64, Stop> {
    if let (Some(u), Some(au)) = (&lg.unit, &a.leg.unit)
        && u != au
    {
        return Err(Stop::Refused(Refusal::new(
            Code::Load(Load::Unit),
            format!("settlement {by} of {} is in {u}, not {au}", a.id),
        )));
    }
    let neither = || {
        Stop::Refused(Refusal::new(
            Code::Run(Run::Record),
            format!(
                "settlement {by} of {} touches neither place of its leg",
                a.id
            ),
        ))
    };
    if let ([bf, bt], [nf, nt]) = (a.before_ends.as_slice(), a.new_ends.as_slice()) {
        let (froms, tos): ([&End; 2], [&End; 2]) = ([bf, nf], [bt, nt]);
        if froms.iter().any(|f| tos.contains(f)) {
            return Err(Stop::Refused(Refusal::new(
                Code::Run(Run::Record),
                format!("settlement {by} of {}: the leg is turned round", a.id),
            )));
        }
        let fwd = froms.contains(&&lg.from) || tos.contains(&&lg.to);
        let back = tos.contains(&&lg.from) || froms.contains(&&lg.to);
        return match (fwd, back) {
            (true, true) => Ok(0),
            (true, false) => Ok(1),
            (false, true) => Ok(-1),
            (false, false) => Err(neither()),
        };
    }
    if !a.before_ends.is_empty() || !a.new_ends.is_empty() {
        return Err(invariant("a moved leg without its two places on each side"));
    }
    if lg.from == a.leg.from || lg.to == a.leg.to {
        return Ok(1);
    }
    if lg.from == a.leg.to || lg.to == a.leg.from {
        return Ok(-1);
    }
    Err(neither())
}
