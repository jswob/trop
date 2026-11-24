//! Adjustments: a booked instance measured again at each later version's cut, each change of a leg recorded with the versions that made it.
//! Spec: E §5 (steps 1–5: the booked instance at its booking cut, one refused there never
//! re-measured, one whose day is a hole measured from the walk's start at every step after
//! it, its adjustments existing as that hole, one whose law date is a hole that hole at each
//! such step, tally 2.3 C-4; each later step c with t < c ≤ K, δ per leg, refused where the step's
//! re-measurement is refused or the leg's amount is on either side, else a hole where its
//! `when` is, tally 2.1.6, the next step measured against that state; an adjustment recorded
//! for δ ≠ 0, a change of places, or a δ that is a hole or
//! refused; an adjustment nothing settles is `HOLE-CHANGED` from its day, and one whose δ is
//! refused refuses its change and its remainder; tally 2.1.2); L §5.14.3 (adjustments, their
//! names and remainders), §8.1 (`HOLE-CHANGED`'s question), §8.2 (`adjustment:` in the
//! register), §8.4 (a refusal outranks a hole).
//! Twin: engine/tools/refcore/moves.py (`AdjMv`, `remainder`, `known_dates`, `adjust_move`,
//! `adjust_move_compute`, `step_legs`, `leg_refusal`, `adjust_made`, `step_versions`).
//! Invariants: a move's adjustments are computed once per cut, in the order of their day,
//! move, instance and leg; the versions known on a day are the versions', rows' and
//! observations' own; a booking refused at its booking cut records no adjustment; an
//! adjustment's change is a number, a hole or a refusal, and its remainder is its change less
//! its settlements, none once waived, refused whenever its change is (the links are
//! `book/settle.rs`'s).

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use crate::base::{Code, Refusal, Run};
use crate::eval::{Collected, Engine};
use crate::hole::{Hole, spread};
use crate::model::{Domain, MoveInfo, Node, NodeKind, Reality};
use crate::syntax::Stop;
use crate::time::Day;
use crate::walk::{
    Amount, Dated, End, Exists, Inst, Law, LegVal, Seed, Status, Term, invariant, qty_value,
};

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
    /// Its day; none where it is a hole.
    pub inst_day: Option<Day>,
    /// Its day as its question names it: the day, or its hole (refcore's `day_text`).
    pub inst_on: String,
    /// The leg, as booked or as the step gives it.
    pub leg: LegVal,
    /// The places the step gives the leg, when it changes them (L §5.14.3).
    pub new_ends: Vec<End>,
    /// The places the leg had before the step, when it changes them (L §5.14.3 How a
    /// settlement leg counts).
    pub before_ends: Vec<End>,
    /// The change δ: a quantity of the leg's unit, a hole, or refused, when the step or the
    /// state it is measured against was (E §5 step 3, tally 2.1.2).
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
    /// Whether its step made the instance exist, the state before it not existing: the leg's
    /// places are then this version's (E §5 step 1).
    pub made: bool,
    /// The hole its existence is: an instance whose day is a hole was booked before its step
    /// only where that day falls before the step's (E §5 step 1, tally 2.3 C-4).
    pub unknown: Option<Hole>,
}

/// A move's measure over one of its lists at a cut: its adjustments, and the hole bookings
/// its versions answered, as the instances they become (refcore's `adjust_move`'s cache).
/// E §5 steps 1–4.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Measured {
    /// The adjustments, in their order.
    pub adjustments: Vec<Adjustment>,
    /// The answered hole bookings.
    pub answered: Vec<Inst>,
    /// The adjustments of its instances whose day is a hole, each existing as that hole
    /// (E §5 step 1, tally 2.3 C-4): only a binding over `adjustments` reads them, every
    /// place they book being that hole day's already.
    pub unsure: Vec<Adjustment>,
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
    /// Its movement order, `(day, -1, move, instance, leg)` (refcore's `AdjMv.order`; tally
    /// 2.1.2). E §4.1, §6.3 Order; L §5.19 Bindings.
    #[must_use]
    pub fn order(&self) -> Term {
        Term::booked(self.day, &[&self.move_id, &self.inst_id, &self.leg.name])
    }

    /// What remains of the change: δ less the settlements; none once waived; a refused
    /// change refuses, settled or not (refcore's `AdjMv.remainder`; tally 2.1.2). E §5 step 5.
    ///
    /// # Errors
    ///
    /// The change's refusal; a settlement's refusal; a number error.
    pub fn remainder(&self) -> Result<Option<Amount>, Stop> {
        if let Amount::Refused(err) = &self.delta {
            return Err(Stop::Refused(err.clone()));
        }
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
                self.inst_on,
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
    pub(crate) fn amount_text(&self, m: &Reality, a: &Amount) -> Result<String, Stop> {
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
    /// The adjustments of one plan move's booked instances at cut `cut`, kept per move, cut
    /// and the list they measure (the whole list, or a walk up to a day's); a hole booking a
    /// later version answers becomes, in that list, what that version gives, a settlement's
    /// too, which has no adjustment (refcore's `adjust_move`, its key with `upto_tag`,
    /// `answer_booking`). E §5 steps 1–4, §3.2 (tally 2.1.8 §4, item 28).
    ///
    /// # Errors
    ///
    /// `RUN-CYCLE` for adjustments that read themselves; the refusals of the measuring.
    pub(crate) fn adjust_move(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        cut: Day,
    ) -> Result<Vec<Adjustment>, Stop> {
        let upto = self.list_upto(mv, cut);
        let key = (mv.id.clone(), cut, upto);
        if let Some(hit) = self.adjustments.get(&key) {
            return Ok(hit.adjustments.clone());
        }
        if self.adjust_active.contains(&key) {
            self.narrow_open_to_days();
            return Err(Stop::Refused(Refusal::new(
                Code::Run(Run::Cycle),
                format!("the adjustments of {} read themselves", mv.id),
            )));
        }
        self.adjust_active.insert(key.clone());
        // what the measuring reads is the adjustments', shared by every reader (E §3.2)
        let mark = self.reads_apart();
        let made = self.measure_move(m, mv, cut);
        self.reads_back(mark);
        self.adjust_active.remove(&key);
        let measured = made?;
        self.answer_holes(&mv.id, (cut, upto), &measured.answered);
        let out = measured.adjustments.clone();
        self.adjustments.insert(key, measured);
        Ok(out)
    }

    /// The adjustments of plan move `mv`'s instances whose day is a hole at cut `cut`, each
    /// existing as that hole ([`Measured::unsure`]; E §5 step 1, tally 2.3 C-4).
    ///
    /// # Errors
    ///
    /// As [`Engine::adjust_move`].
    pub(crate) fn unsure_of(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        cut: Day,
    ) -> Result<Vec<Adjustment>, Stop> {
        self.adjust_move(m, mv, cut)?;
        let key = (mv.id.clone(), cut, self.list_upto(mv, cut));
        let measured = self
            .adjustments
            .get(&key)
            .ok_or_else(|| invariant("a move's measure not kept"))?;
        Ok(measured.unsure.clone())
    }

    /// Plan move `mv`'s list at cut `cut` holds a hole booking before the cut: the move's
    /// adjustments over that list, so a later version that answers it does so there, whatever
    /// reads it first; a measure made before the list was gives it its answers too; a refusal the
    /// measuring meets stays its own readers'. A cycle it meets kept the open cells at their
    /// own law dates: they read the list with its holes unanswered, which turns on the measure
    /// in progress (E §5 step 1, steps 2–4; E §3.2).
    ///
    /// # Errors
    ///
    /// A stop other than a refusal that measuring the move meets.
    pub(crate) fn answer_hole_bookings(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        cut: Day,
    ) -> Result<(), Stop> {
        let upto = self.list_upto(mv, cut);
        let key = (mv.id.clone(), cut, upto);
        if self.adjust_active.contains(&key) {
            return Ok(());
        }
        if let Some(hit) = self.adjustments.get(&key) {
            let answered = hit.answered.clone();
            self.answer_holes(&mv.id, (cut, upto), &answered);
            return Ok(());
        }
        // the plan's own measure is no walk read of an explanation the instances are read
        // under: its causes are the plan's, as its status's are (E §10)
        let back = self.trace_walked();
        let measured = self.adjust_move(m, mv, cut);
        self.trace_walked_back(back);
        match measured {
            Ok(_) | Err(Stop::Refused(_)) => Ok(()),
            Err(stop) => Err(stop),
        }
    }

    /// The move's list at cut `cut` that the answers' measure took, with each hole booking a
    /// later version answered by `answered` in its place: the whole list, and the lists up to
    /// a day kept, or the one list up to a day `upto` (refcore's in-place `i.exists, i.legs =
    /// j.exists, j.legs`). E §5 step 1.
    fn answer_holes(&mut self, move_id: &str, (cut, upto): (Day, Option<Day>), answered: &[Inst]) {
        if answered.is_empty() {
            return;
        }
        if let Some(day) = upto {
            let key = (move_id.to_owned(), cut, day);
            if let Some(list) = self.prefixes.plans.get(&key) {
                let patched = with_answers(list, answered);
                self.prefixes.plans.insert(key, patched);
            }
            return;
        }
        let key = (move_id.to_owned(), cut);
        if let Some(list) = self.plans.get(&key) {
            let patched = with_answers(list, answered);
            self.plans.insert(key, patched);
        }
        for ((id, at, _), list) in &mut self.prefixes.plans {
            if id == move_id && *at == cut {
                *list = with_answers(list, answered);
            }
        }
    }

    /// Instances `made` of move `mv` apart from its list (a settlement's bound to the
    /// adjustments it settles), each hole booking before cut `cut` that a later version
    /// answers in its answer's place, so a settlement's hole booking is answered like any
    /// instance's (refcore's `answer_booking`). E §5 step 1 (tally 2.1.8 §4, item 28).
    ///
    /// # Errors
    ///
    /// The refusals of the measuring.
    pub(crate) fn answer_apart(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        made: Vec<Inst>,
        cut: Day,
    ) -> Result<Vec<Inst>, Stop> {
        let steps_all = known_dates(m);
        let mut out = Vec::with_capacity(made.len());
        for inst in made {
            let t = inst.day.as_ref().and_then(Dated::day);
            let Some(t) = t.filter(|t| *t < cut && is_hole_booking(&inst)) else {
                out.push(inst);
                continue;
            };
            let steps: Vec<&(Day, BTreeSet<String>)> = steps_all
                .iter()
                .filter(|(c, _)| t < *c && *c <= cut)
                .collect();
            let mut answered = Vec::new();
            self.measure_inst(m, mv, (&inst, t), (&steps, cut), &mut answered)?;
            out.push(answered.pop().unwrap_or(inst));
        }
        Ok(out)
    }

    /// Each booked instance before the cut measured at each later step, with the hole
    /// bookings a step answers, as the instances they become, and each instance whose day is
    /// a hole measured at every step (refcore's `adjust_move_compute`). E §5 steps 1–4.
    fn measure_move(&mut self, m: &Reality, mv: &MoveInfo, cut: Day) -> Result<Measured, Stop> {
        let steps_all = known_dates(m);
        let mut out = Vec::new();
        let mut answered = Vec::new();
        let mut unsure = Vec::new();
        // a settlement's hole booking is answered like any instance's; steps 2–3 leave the
        // rest of it unmeasured (E §5 steps 1–2; tally 2.1.8 §4, item 28)
        let settles = super::settle::settlement_move(m, mv)?;
        // at a past cut only the instances this measure takes (fence F7)
        for inst in self.measured_list(m, mv, cut)?.iter() {
            if !matches!(inst.status, Status::Past | Status::Hole | Status::None) {
                continue;
            }
            if settles && !is_hole_booking(inst) {
                continue;
            }
            let t = match &inst.day {
                Some(Dated::Day(d)) => *d,
                Some(Dated::Hole(h)) if !settles && inst.exists != Exists::No => {
                    unsure.extend(self.measure_hole_day(m, mv, (inst, h), (&steps_all, cut))?);
                    continue;
                }
                Some(Dated::Hole(_)) | None => continue,
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
            let made = self.measure_inst(m, mv, (inst, t), (&steps, cut), &mut answered)?;
            if !settles {
                out.extend(made);
            }
        }
        out.sort_by(|a, b| {
            (a.day, &a.move_id, &a.inst_id, &a.leg.name).cmp(&(
                b.day,
                &b.move_id,
                &b.inst_id,
                &b.leg.name,
            ))
        });
        Ok(Measured {
            adjustments: out,
            answered,
            unsure,
        })
    }

    /// Plan instance `inst` whose day is hole `h`, measured at the known dates `steps_all` up
    /// to cut `cut`: κ₀ is that hole, a day of the walk, so the instance as the walk's start
    /// gives it stands for its booking, and each known date after the start that changes it
    /// makes an adjustment whose existence is the hole, the day falling before that step's or
    /// not; one whose law date is a hole is that hole, changed or not at each such date (E §5
    /// step 1, tally 2.3 C-4; the review of G1, B2).
    ///
    /// # Errors
    ///
    /// The refusals of the measuring.
    fn measure_hole_day(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        (inst, h): (&Inst, &Hole),
        (steps_all, cut): (&[(Day, BTreeSet<String>)], Day),
    ) -> Result<Vec<Adjustment>, Stop> {
        // the day falls in the walk: the versions known by its start are the booking's, each
        // known after it and by the cut a step that may fall after the day
        let start = m.start;
        let later: Vec<&(Day, BTreeSet<String>)> = steps_all
            .iter()
            .filter(|(c, _)| start < *c && *c <= cut)
            .collect();
        if later.is_empty() {
            return Ok(Vec::new());
        }
        // a hole `at`: the instance is that hole, its legs read at no law date, so whether a
        // step that touches what its move names changes it is the hole too (the reviews of
        // G1, B2, and its fix, BF2)
        if matches!(inst.law, Law::Hole(_)) {
            return Ok(law_hole_adjustments(m, mv, (inst, h), &later));
        }
        let seed = Seed {
            env: inst.env.clone(),
            keys: inst.keys.clone(),
            exists: (*inst.exists0).clone(),
            at_cut: Some(start),
            upto: None,
        };
        let Some(booked) = self.eval_inst(m, mv, &seed, self.timeline_ctx(None, cut))? else {
            return Ok(Vec::new());
        };
        // what a step answers of a hole booking here is dropped: the list keeps the instance
        // as its hole day, which no version answers
        let mut answered = Vec::new();
        let made = self.measure_inst(m, mv, (&booked, start), (&later, cut), &mut answered)?;
        Ok(made
            .into_iter()
            .map(|mut a| {
                a.inst_day = None;
                a.inst_on = h.text();
                a.unknown = Some(h.clone());
                a
            })
            .collect())
    }

    /// One instance measured at each step: δ per leg against the state before (refcore's
    /// `adjust_move_compute`, its steps). A booking refused at κ₀ stays refused and records
    /// nothing; a hole booking stays a hole until a step answers it, which `answered` gets as
    /// the instance at the cut, and one answered by a refusal stays refused, measured no more
    /// (tally 2.1.8 S9); a refused step makes each leg's adjustment refused, a step
    /// whose `when` is a hole makes each a hole but a leg's whose amount is refused on either
    /// side, refused (tally 2.1.6), and the step after either is measured against it (tally
    /// 2.1.2). E §5 steps 1–4.
    fn measure_inst(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        (inst, t): (&Inst, Day),
        (steps, cut): (&[&(Day, BTreeSet<String>)], Day),
        answered: &mut Vec<Inst>,
    ) -> Result<Vec<Adjustment>, Stop> {
        let mut out = Vec::new();
        // E §5 step 1 (tally 2.1.2, "refused stays refused"): its re-measurements record nothing
        if inst.legs.iter().any(is_refused_leg) {
            return Ok(out);
        }
        let mut prev = Prev {
            legs: inst.legs.clone(),
            exists: inst.exists == Exists::Yes,
            unknown: None,
        };
        let mut hole0 = is_hole_booking(inst);
        for (c, vids) in steps.iter().map(|s| (s.0, &s.1)) {
            let seed = Seed {
                env: inst.env.clone(),
                keys: inst.keys.clone(),
                // its keys' `where` is read at K, so a hole it leaves is no version's to
                // answer (E §4.1 step 1; refcore's `exists0`, #236)
                exists: (*inst.exists0).clone(),
                at_cut: Some(c),
                upto: None,
            };
            let Some(j) = self.eval_inst(m, mv, &seed, self.timeline_ctx(None, cut))? else {
                continue;
            };
            let step = Step {
                inst_id: inst.id.clone(),
                inst_keys: inst.keys.clone(),
                move_id: inst.move_id.clone(),
                inst_day: t,
                inst_on: t.text(),
                day: c,
                vids: step_versions(&j.coll, vids),
                coll: j.coll.clone(),
            };
            if hole0 && let Some(err) = &j.err {
                // the answer is its booking from then on: one answered by a refusal stays
                // refused and, as a refused A0, no later version measures it again (E §5 step
                // 1; tally 2.1.8 S9)
                let mut answer = inst.clone();
                answer.err = Some(err.clone());
                answer.coll.merge(&j.coll);
                answer.answered = Some((c, step.vids.clone()));
                answer.status = self.status(m, mv, &mut answer, cut)?;
                answered.push(answer);
                return Ok(out);
            }
            if let Some(err) = &j.err {
                // a version answers a hole movement with a value or a refusal (E §5 step 1; L
                // §8, a refusal outranks a hole: tally 2.1.7 R2, #94): each leg's adjustment
                // is refused from that version's known, and the next step's (step 3)
                let refused = Amount::Refused(err.clone());
                out.extend(each_leg(&step, inst, &prev.legs, &j.legs, &refused));
                prev.unknown = Some(Unknown::Refused(err.clone()));
                continue;
            }
            if let Some(was) = prev.unknown.take() {
                let made = match &was {
                    Unknown::Refused(err) => {
                        let refused = Amount::Refused(err.clone());
                        each_leg(&step, inst, &prev.legs, &j.legs, &refused)
                    }
                    Unknown::Hole(h) => against_hole(&step, inst, &prev.legs, &j, h)?,
                };
                out.extend(made);
                prev = Prev::after(&j);
                continue;
            }
            if hole0 {
                if matches!(j.exists, Exists::Unknown(_)) || j.legs.iter().any(is_hole_leg) {
                    continue;
                }
                // a version answers the hole movement: its existence and amounts are the
                // instance's under cuts from its known, and the steps after measure against
                // them (E §5 step 1)
                let mut answer = inst.clone();
                answer.exists = j.exists.clone();
                answer.legs.clone_from(&j.legs);
                answer.coll.merge(&j.coll);
                answer.answered = Some((c, step.vids.clone()));
                answer.status = self.status(m, mv, &mut answer, cut)?;
                answered.push(answer);
                hole0 = false;
                prev = Prev::after(&j);
                continue;
            }
            if let Exists::Unknown(h) = &j.exists {
                // tally 2.1.2: a `when` that is a hole makes each leg's adjustment that hole,
                // but a leg refused on either side (tally 2.1.6)
                let before = prev.exists.then_some(prev.legs.as_slice());
                out.extend(hole_when(&step, inst, (&prev.legs, before), &j.legs, h));
                prev = Prev::after(&j);
                continue;
            }
            out.extend(measured(&step, inst, &prev, &j)?);
            prev = Prev::after(&j);
        }
        Ok(out)
    }
}

/// The state the next step of a measuring is measured against: the legs and existence of
/// the last state measured, and whether it was refused or its existence a hole (refcore's
/// `prev`, `prev_ex`, `prev_bad`, `prev_hole`). E §5 step 3.
struct Prev {
    /// Its legs.
    legs: Vec<LegVal>,
    /// Whether it existed.
    exists: bool,
    /// Its refusal, or the hole its existence is.
    unknown: Option<Unknown>,
}

impl Prev {
    /// The state a step leaves: its legs, whether it exists, and its existence's hole. E §5
    /// step 3.
    fn after(j: &Inst) -> Prev {
        Prev {
            legs: j.legs.clone(),
            exists: j.exists == Exists::Yes,
            unknown: match &j.exists {
                Exists::Unknown(h) => Some(Unknown::Hole(h.clone())),
                Exists::Yes | Exists::No => None,
            },
        }
    }
}

/// What a step measured against an unknown state finds: a refusal, or a hole (tally 2.1.2).
/// E §5 step 3.
enum Unknown {
    /// The state was refused.
    Refused(Refusal),
    /// The state's existence is a hole.
    Hole(Hole),
}

/// Each leg's adjustment at a step whose δ is unknown, refused or a hole, the same for every
/// leg (refcore's `adjust_made` over `step_legs`; tally 2.1.2). E §5 steps 3–4.
fn each_leg(
    step: &Step,
    inst: &Inst,
    prev: &[LegVal],
    now: &[LegVal],
    delta: &Amount,
) -> Vec<Adjustment> {
    step_legs(inst, prev, now)
        .into_iter()
        .map(|lg| make(step, lg, delta.clone(), (Vec::new(), Vec::new())))
        .collect()
}

/// Each leg's adjustment at a step whose `when` is a hole: that hole, as a hole amount
/// makes it, but for a leg whose amount is refused at this step or in the state measured
/// against when it existed, whose δ is that refusal, which outranks a hole (refcore's
/// `adjust_move_compute`, its hole `when`, `leg_refusal`; tally 2.1.6). E §5 steps 3–4,
/// L §8.4.
fn hole_when(
    step: &Step,
    inst: &Inst,
    (prev, before): (&[LegVal], Option<&[LegVal]>),
    now: &[LegVal],
    h: &Hole,
) -> Vec<Adjustment> {
    let mut out = Vec::new();
    for lg in step_legs(inst, prev, now) {
        let delta = match leg_refusal(before, now, &lg.name) {
            Some(err) => Amount::Refused(err),
            None => Amount::Hole(h.clone()),
        };
        out.push(make(step, lg, delta, (Vec::new(), Vec::new())));
    }
    out
}

/// The refusal of leg `name`'s amount at this step, or else in the state measured against;
/// none where both are a number or a hole (refcore's `leg_refusal`; tally 2.1.6). E §5
/// step 3: "a leg's amount on either side".
fn leg_refusal(before: Option<&[LegVal]>, now: &[LegVal], name: &str) -> Option<Refusal> {
    now.iter()
        .chain(before.into_iter().flatten())
        .filter(|lg| lg.name == name)
        .find_map(|lg| refusal_of(Some(&lg.amount)).cloned())
}

/// A step measured against a state whose existence is a hole: each leg's adjustment a hole
/// holding that hole, this step's hole amount at the leg and its own existence's hole, but
/// for a leg whose amount is refused on either side, whose δ is that refusal (refcore's
/// `adjust_move_compute`, `prev_hole`; tally 2.1.2, and 2.1.6 for the refused leg). E §5
/// steps 3–4, L §8.4.
fn against_hole(
    step: &Step,
    inst: &Inst,
    prev: &[LegVal],
    j: &Inst,
    was: &Hole,
) -> Result<Vec<Adjustment>, Stop> {
    let mut out = Vec::new();
    for lg in step_legs(inst, prev, &j.legs) {
        if let Some(err) = leg_refusal(Some(prev), &j.legs, &lg.name) {
            out.push(make(
                step,
                lg,
                Amount::Refused(err),
                (Vec::new(), Vec::new()),
            ));
            continue;
        }
        let mut holes = vec![Some(was)];
        let now = j.legs.iter().filter(|x| x.name == lg.name);
        holes.extend(now.map(|x| hole_of(&x.amount)));
        if let Exists::Unknown(own) = &j.exists {
            holes.push(Some(own));
        }
        let delta = Amount::Hole(spread(&holes).ok_or_else(|| invariant("a hole change"))?);
        out.push(make(step, lg, delta, (Vec::new(), Vec::new())));
    }
    Ok(out)
}

/// The legs a step measures again, one per name: the booked instance's, the previous
/// state's and this step's, each where it is first met (refcore's `step_legs`). E §5 step 3.
fn step_legs(inst: &Inst, prev: &[LegVal], now: &[LegVal]) -> Vec<LegVal> {
    let mut out: Vec<LegVal> = Vec::new();
    for lg in inst.legs.iter().chain(prev).chain(now) {
        if !out.iter().any(|o| o.name == lg.name) {
            out.push(lg.clone());
        }
    }
    out
}

/// A step measured against a known state: δ per leg of the two states compared, the
/// previous state's legs when it existed and this step's when it exists (refcore's
/// `adjust_move_compute`, its legs). E §5 steps 3–4.
fn measured(step: &Step, inst: &Inst, prev: &Prev, j: &Inst) -> Result<Vec<Adjustment>, Stop> {
    let before: BTreeMap<&str, &LegVal> = if prev.exists {
        prev.legs.iter().map(|lg| (lg.name.as_str(), lg)).collect()
    } else {
        BTreeMap::new()
    };
    let after: BTreeMap<&str, &LegVal> = if j.exists == Exists::Yes {
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
    // this step makes the instance exist: its legs' places are this version's (E §5 step 1)
    let made = !prev.exists && j.exists == Exists::Yes;
    let mut out = Vec::new();
    for ln in names {
        let (a, b) = (before.get(ln).copied(), after.get(ln).copied());
        if let Some(adj) = leg_change(step, &pool, (a, b, made), ln)? {
            out.push(adj);
        }
    }
    Ok(out)
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
    /// Its day as its question names it.
    inst_on: String,
    /// The step's day c.
    day: Day,
    /// Its versions.
    vids: Vec<String>,
    /// What it met.
    coll: Collected,
}

/// The adjustment of one leg at one step: δ between the step before (`a`) and this one (`b`),
/// recorded for δ ≠ 0, a hole, or a change of places; a δ in number marked `made` where the
/// step made the instance exist (refcore's `adjust_move_compute`, a leg, `adj.made`). E §5
/// steps 1, 3–4, L §5.14.3.
fn leg_change(
    step: &Step,
    pool: &[&LegVal],
    (a, b, made): (Option<&LegVal>, Option<&LegVal>, bool),
    name: &str,
) -> Result<Option<Adjustment>, Stop> {
    let qa = a.map(|lg| &lg.amount);
    let qb = b.map(|lg| &lg.amount);
    if let Some(err) = refusal_of(qb).or_else(|| refusal_of(qa)) {
        // tally 2.1.2: a leg's amount refused on either side refuses its adjustment, and the
        // next step, measured against this state, is refused too
        let leg = pool_leg(pool, name)?;
        let delta = Amount::Refused(err.clone());
        return Ok(Some(make(step, leg, delta, (Vec::new(), Vec::new()))));
    }
    let Some(either) = b.or(a) else {
        return Ok(None);
    };
    let leg = pool_leg(pool, name)?;
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
    let mut adj = make(step, leg, Amount::Qty(delta), ends);
    adj.made = made;
    Ok(Some(adj))
}

/// The adjustments of plan instance `inst` of `mv`, whose law date is hole `h`, at the known
/// dates `steps`: nothing of the instance was read, so a step touches its read set where
/// one of its versions states a row of what its move names ([`named_reads`]); there each leg
/// of the move has an adjustment whose existence, places and change are the hole, named by
/// all the step's versions, and elsewhere none (E §5 steps 1 and 2; L §5.14 `at e`, an `at`
/// that gives a window a hole `at`, §4.4; the reviews of G1, B2, and its fix, BF2, P10).
fn law_hole_adjustments(
    m: &Reality,
    mv: &MoveInfo,
    (inst, h): (&Inst, &Hole),
    steps: &[&(Day, BTreeSet<String>)],
) -> Vec<Adjustment> {
    let reads = named_reads(m, mv);
    let mut out = Vec::new();
    for (day, versions) in steps {
        if !reads.touched_by(m, versions) {
            continue;
        }
        let step = Step {
            inst_id: inst.id.clone(),
            inst_keys: inst.keys.clone(),
            move_id: mv.id.clone(),
            inst_day: *day,
            inst_on: h.text(),
            day: *day,
            vids: versions.iter().cloned().collect(),
            coll: Collected::default(),
        };
        for leg in &mv.decl.legs {
            let unread = LegVal {
                name: leg.name.clone(),
                from: End::Hole(h.clone()),
                to: End::Hole(h.clone()),
                amount: Amount::Hole(h.clone()),
                unit: None,
                day: Dated::Hole(h.clone()),
                src: None,
            };
            let mut adj = make(
                &step,
                unread,
                Amount::Hole(h.clone()),
                (Vec::new(), Vec::new()),
            );
            adj.inst_day = None;
            adj.unknown = Some(h.clone());
            out.push(adj);
        }
    }
    out
}

/// What a move names, its read set where nothing of an instance was read: the inputs its
/// `on`, `at`, `when`, legs and attributes read through the item graph, defs and the items
/// they read included, and the indexes of its keys (E §5 step 2; the review of G1's fix,
/// BF2, P10).
struct NamedReads {
    /// Whether it reaches the balances, which every movement books into: then every version
    /// touches it.
    all: bool,
    /// The inputs it reaches.
    inputs: BTreeSet<String>,
    /// The indexes whose keys it reads: its keys', and every index where it reaches a def.
    indexes: BTreeSet<String>,
    /// Whether it reaches a def, which may read any index's keys.
    defs: bool,
}

impl NamedReads {
    /// Whether one of `versions` states a row or a reading of an input it reaches, or adds a
    /// key to an index it reads (E §5 step 2).
    fn touched_by(&self, m: &Reality, versions: &BTreeSet<String>) -> bool {
        if self.all {
            return true;
        }
        for input in self.inputs.iter().filter_map(|id| m.inputs.get(id)) {
            let rows = input.rows.iter().map(|r| &r.version);
            let readings = input.observations.iter().map(|o| &o.version);
            if rows.chain(readings).any(|v| versions.contains(v)) {
                return true;
            }
        }
        for (_, ix) in m.indexes.entries() {
            let read = self.defs || self.indexes.contains(&ix.id);
            if read && ix.added.iter().any(|(_, v)| versions.contains(v)) {
                return true;
            }
        }
        false
    }
}

/// The read set move `mv` names ([`NamedReads`]): from its node and its day's node, every
/// item the item graph says they read, and on through what those read, as a worklist
/// (E §3.1 step 5, §5 step 2).
fn named_reads(m: &Reality, mv: &MoveInfo) -> NamedReads {
    let mut out = NamedReads {
        all: false,
        inputs: BTreeSet::new(),
        indexes: BTreeSet::new(),
        defs: false,
    };
    for bind in &mv.binds {
        if let Domain::Index(ix) = &bind.domain {
            out.indexes.insert(ix.clone());
        }
    }
    let mut seen: BTreeSet<Node> = BTreeSet::new();
    let mut work = vec![
        Node::new(NodeKind::Move, &mv.id),
        Node::new(NodeKind::MoveOn, &mv.id),
    ];
    while let Some(node) = work.pop() {
        if !seen.insert(node.clone()) {
            continue;
        }
        match node.kind {
            NodeKind::Balances => out.all = true,
            NodeKind::Input => {
                out.inputs.insert(node.id.clone());
            }
            NodeKind::Def => out.defs = true,
            NodeKind::Band
            | NodeKind::Move
            | NodeKind::MoveOn
            | NodeKind::Place
            | NodeKind::Ask
            | NodeKind::Price
            | NodeKind::Parts
            | NodeKind::Axis
            | NodeKind::Fill
            | NodeKind::Srow => {}
        }
        if let Some(reads) = m.graph.edges.get(&node) {
            work.extend(reads.keys().cloned());
        }
    }
    out
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
        made: false,
        unknown: None,
        move_id: step.move_id.clone(),
        inst_day: Some(step.inst_day),
        inst_on: step.inst_on.clone(),
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

/// A leg amount's refusal, when it is one. E §5 step 3.
fn refusal_of(a: Option<&Amount>) -> Option<&Refusal> {
    match a {
        Some(Amount::Refused(r)) => Some(r),
        Some(Amount::Qty(_) | Amount::Hole(_)) | None => None,
    }
}

/// The leg of name `name` among the legs a step compares, as the adjustment names it. E §5
/// step 4.
fn pool_leg(pool: &[&LegVal], name: &str) -> Result<LegVal, Stop> {
    pool.iter()
        .find(|lg| lg.name == name)
        .map(|lg| (*lg).clone())
        .ok_or_else(|| invariant("an adjusted leg in no pool"))
}

/// Whether a leg's amount is a hole. E §5 step 1.
fn is_hole_leg(lg: &LegVal) -> bool {
    matches!(lg.amount, Amount::Hole(_))
}

/// Whether a leg's amount is refused. E §5 step 1.
fn is_refused_leg(lg: &LegVal) -> bool {
    matches!(lg.amount, Amount::Refused(_))
}

/// Whether an instance is a hole movement booked before cut `cut`: dated before it, past (or
/// a hole, or none so far), its booking a hole (refcore's `hole0`). E §5 step 1.
pub(crate) fn hole_booked_before(inst: &Inst, cut: Day) -> bool {
    let day = inst.day.as_ref().and_then(Dated::day);
    matches!(inst.status, Status::Past | Status::Hole | Status::None)
        && day.is_some_and(|t| t < cut)
        && is_hole_booking(inst)
}

/// A move's instance list with each instance a version answered replaced by its answer
/// (refcore mutates the cached instance). E §5 step 1.
fn with_answers(list: &Rc<Vec<Inst>>, answered: &[Inst]) -> Rc<Vec<Inst>> {
    let mut out = list.to_vec();
    for inst in &mut out {
        if let Some(a) = answered.iter().find(|a| a.id == inst.id) {
            a.clone_into(inst);
        }
    }
    Rc::new(out)
}

/// Whether a booking is a hole movement: its existence or a leg's amount a hole, no leg's
/// refused (refcore's `hole0`). E §5 step 1.
pub(crate) fn is_hole_booking(inst: &Inst) -> bool {
    let hole = matches!(inst.exists, Exists::Unknown(_)) || inst.legs.iter().any(is_hole_leg);
    hole && !inst.legs.iter().any(is_refused_leg)
}

/// The days versions became known, each with the versions known then: versions' `known`,
/// rows' own (refcore's `known_dates`). E §5 step 2.
pub(crate) fn known_dates(m: &Reality) -> Vec<(Day, BTreeSet<String>)> {
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
    let Some(read) = coll.versions() else {
        return vids.iter().cloned().collect();
    };
    let hit: Vec<String> = vids.intersection(read).cloned().collect();
    if hit.is_empty() {
        return vids.iter().cloned().collect();
    }
    hit
}

#[cfg(test)]
mod tests {
    use crate::api::{Asks, Files, Given, Request, run};

    /// A pay on a window of March, its amount raised by a version known on 20 March (the
    /// critic's `n1_q`, its window twin).
    const RAISED: &str = r#"model raised
  tally 2
  on 2026-06-10
  start 2026-03-01
  end 2026-06-30

source fixture "invented values"

unit PLN: currency minor 0.01

place konto: accounts PLN lives 2026-01-01..

place world: outside lives 1900-01-01..

input amt: money PLN

input w: date

input d_at: date

move pay: transfers
  on w
  at d_at
  leg cash from world to konto amount amt

version v
  cite fixture
  governs always
  known 2026-01-01
  w = window(2026-03-05, 2026-03-20) "a day in March"
  d_at = 2026-03-01 "its law date"
  opening[konto] = 500.00 PLN at 2026-03-01 start "own"

version a0
  cite fixture
  governs 2026..
  known 2026-01-01
  amt = 100.00 PLN "first"

version a1
  cite fixture
  governs 2026..
  known 2026-03-20
  replace amt = 150.00 PLN "raised"

ask n: count(a in adjustments over 2026-03 .. 2026-04)

ask n_early: count(a in adjustments over 2026-03 .. 2026-04) with w = 2026-03-05

ask n_late: count(a in adjustments over 2026-03 .. 2026-04) with w = 2026-03-20

ask n_may: count(a in adjustments over 2026-05 .. 2026-06)

ask s: sum(1 for a in adjustments over 2026-03 .. 2026-04)

ask n_k: count(a in adjustments over 2026-03 .. 2026-04) as known 2026-03-19
"#;

    /// A pay on a day of March whose law date `w` is written in for `LAW`, its amount raised
    /// by a version known on 20 March (the review's `f9`).
    const AT_HOLE: &str = r#"model at_hole
  tally 2
  on 2026-06-10
  start 2026-03-01
  end 2026-06-30

source fixture "invented values"

unit PLN: currency minor 0.01

place konto: accounts PLN lives 2026-01-01..

place world: outside lives 1900-01-01..

input amt: money PLN

input w: date

input dd: date

move pay: transfers
  on dd
  at w
  leg cash from world to konto amount amt

version v
  cite fixture
  governs always
  known 2026-01-01
  dd = 2026-03-10 "its day"
  w = LAW "its law date"
  opening[konto] = 500.00 PLN at 2026-03-01 start "own"

version a0
  cite fixture
  governs 2026..
  known 2026-01-01
  amt = 100.00 PLN "first"

version a1
  cite fixture
  governs 2026..
  known 2026-03-20
  replace amt = 150.00 PLN "raised"

ask n: count(a in adjustments over 2026-03 .. 2026-04)

ask n_p: count(a in adjustments over 2026-03 .. 2026-04) with w = 2026-03-03

ask n_k: count(a in adjustments over 2026-03 .. 2026-04) as known 2026-03-19
"#;

    /// A pay on a day of March whose law date `w` is written in for `LAW`, and a version
    /// known on 20 March that changes only an input the pay doesn't read (the review of G1's
    /// fix, `b2b`).
    const UNREAD_CHANGE: &str = r#"model unread_change
  tally 2
  on 2026-06-10
  start 2026-03-01
  end 2026-06-30

source fixture "invented values"

unit PLN: currency minor 0.01

place konto: accounts PLN lives 2026-01-01..

place world: outside lives 1900-01-01..

input amt: money PLN

input other: money PLN

input w: date

input dd: date

move pay: transfers
  on dd
  at w
  leg cash from world to konto amount amt

version v
  cite fixture
  governs always
  known 2026-01-01
  w = LAW "its law date"
  amt = 100.00 PLN "its amount"
  dd = 2026-03-10 "its day"
  other = 1.00 PLN "another figure"
  opening[konto] = 500.00 PLN at 2026-03-01 start "own"

version b_mid
  cite fixture
  governs always
  known 2026-03-20
  replace other = 4.00 PLN "changed"

ask n: count(a in adjustments over ..)

ask o: other
"#;

    /// A pay on a window of March, its amount raised by a version known in February, before
    /// the walk's start.
    const BEFORE_START: &str = r#"model before_start
  tally 2
  on 2026-06-10
  start 2026-03-01
  end 2026-06-30

source fixture "invented values"

unit PLN: currency minor 0.01

place konto: accounts PLN lives 2026-01-01..

place world: outside lives 1900-01-01..

input amt: money PLN

input w: date

move pay: transfers
  on w
  leg cash from world to konto amount amt

version v
  cite fixture
  governs always
  known 2026-01-01
  amt = 100.00 PLN "first"
  w = window(2026-03-05, 2026-03-20) "a day in March"
  opening[konto] = 500.00 PLN at 2026-03-01 start "own"

version a1
  cite fixture
  governs always
  known 2026-02-01
  replace amt = 150.00 PLN "raised before the start"

ask n_all: count(a in adjustments over ..)
"#;

    /// The lines of ask `ask` over text `text`.
    fn lines_of(text: &str, ask: &str) -> Vec<String> {
        let given = vec![Given {
            path: "adjust.tally".to_owned(),
            bytes: Some(text.as_bytes().to_vec()),
        }];
        let request = Request::new(Asks::Named(vec![ask.to_owned()]));
        run(Files::new(given), &request)
            .expect("an answer")
            .lines()
            .to_vec()
    }

    /// The lines of ask `ask` over [`RAISED`].
    fn lines(ask: &str) -> Vec<String> {
        lines_of(RAISED, ask)
    }

    #[test]
    fn an_adjustment_of_a_hole_at_instance_exists_as_its_hole() {
        // E §5 step 1 (tally 2.3 C-4; the review of G1, B2): a hole `at` makes the instance and
        // its day holes, so a later version may change it: a binding over adjustments that
        // could hold it is the hole, for a window and a `?` alike, where every pin gives 1
        let window = "?\"date within window(2026-03-01, 2026-03-05) not precise enough for the law date of pay\" (w)";
        let at_window = AT_HOLE.replace("LAW", "window(2026-03-01, 2026-03-05)");
        assert_eq!(
            lines_of(&at_window, "n"),
            [
                format!("n: {window}"),
                "  unknown day: w".to_owned(),
                "  cut 2026-06-10".to_owned()
            ],
            "the window's hole"
        );
        let at_question = AT_HOLE.replace("LAW", "?\"which law date?\"");
        assert_eq!(
            lines_of(&at_question, "n"),
            ["n: ?\"which law date?\" (w)", "  cut 2026-06-10"],
            "the question's hole"
        );
        assert_eq!(
            lines_of(&at_window, "n_p"),
            ["n_p: 1", "  pinned: w = 2026-03-03", "  cut 2026-06-10"],
            "a pinned law date"
        );
        assert_eq!(
            lines_of(&at_window, "n_k"),
            ["n_k: 0", "  cut 2026-03-19"],
            "before the version is known"
        );
    }

    #[test]
    fn a_version_that_touches_nothing_a_hole_at_move_reads_adjusts_nothing() {
        // E §5 step 2: only a version that touches the read set can change the instance; under
        // a hole `at` that set is the reads its move names (the review of G1's fix, BF2, P10)
        for law in ["window(2026-03-01, 2026-03-05)", "?\"which law date?\""] {
            assert_eq!(
                lines_of(&UNREAD_CHANGE.replace("LAW", law), "n"),
                ["n: 0", "  cut 2026-06-10"],
                "{law}"
            );
        }
    }

    #[test]
    fn a_version_known_before_the_start_adjusts_no_hole_day_instance() {
        // a hole day falls in the walk, after its start: a version known before it is in the
        // booking, as every pinned day has it (E §5 step 1, tally 2.3 C-4)
        assert_eq!(
            lines_of(BEFORE_START, "n_all"),
            ["n_all: 0", "  cut 2026-06-10"],
            "no adjustment"
        );
    }

    #[test]
    fn an_adjustment_of_a_hole_day_instance_exists_as_its_hole() {
        // E §5 step 1 (tally 2.3 C-4): pay's day is a hole, so a1 makes an adjustment on 20
        // March only where the day falls before it: a binding over adjustments that could hold
        // it is the hole, where a pinned day before gives 1 and the day itself 0
        let imprecise = "?\"date within window(2026-03-05, 2026-03-20) not precise enough for the day of pay\" (w)";
        for ask in ["n", "s"] {
            assert_eq!(
                lines(ask),
                [
                    format!("{ask}: {imprecise}"),
                    "  unknown day: w".to_owned(),
                    "  cut 2026-06-10".to_owned()
                ],
                "{ask}: the day's hole"
            );
        }
        assert_eq!(
            lines("n_early"),
            ["n_early: 1", "  pinned: w = 2026-03-05", "  cut 2026-06-10"],
            "a day before the version's"
        );
        // on the version's own day there is no adjustment, so nothing read w (as refcore)
        assert_eq!(
            lines("n_late"),
            ["n_late: 0", "  cut 2026-06-10"],
            "the version's own day"
        );
        // a span the adjustment's day isn't in, and a cut before the version, hold none
        assert_eq!(lines("n_may"), ["n_may: 0", "  cut 2026-06-10"], "after it");
        assert_eq!(
            lines("n_k"),
            ["n_k: 0", "  cut 2026-03-19"],
            "before it is known"
        );
    }
}
