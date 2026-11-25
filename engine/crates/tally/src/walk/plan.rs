//! The plan pass: each move's instances over its binding tuples, dated at the cut, evaluated at their booking cut, and classified.
//! Spec: E §4.1 (enumerate the instances, date them at K, evaluate them at min(t, K),
//! classify them, the movement order); E §5 (steps 0 and 1: dating at K,
//! the booking cut; a hole booking a later version answers is that version's, and an
//! explanation lists the answer's reads where its instance's own end, E §10);
//! L §5.14 (moves: keys, `on`, `at`, `for`, `when`, legs on their day; a day or a `when`
//! that is a hole makes a hole movement, a day or a law date that is a window a hole day,
//! L §4.4, tally 2.3 S6); L §8.4 (a refused key's `where`, `when`, attribute
//! or `for` makes a refused movement, its day and legs' places still read; tally 2.1.6);
//! L §6.4 (a move a scenario declares books nothing before its `from`). The binding tuples are
//! `tuples.rs`'s, a leg's places and amount `legs.rs`'s, the instance and its order
//! `instance.rs`'s, its body over its law dates `body.rs`'s.
//! Twin: engine/tools/refcore/moves.py (`move_instances`, `insts_of_tuples`,
//! `decision_instances`, `eval_inst`, `refused_places`, `status`, `scen_early`, `keys_seen`,
//! `answer_booking`).
//! Invariants: an instance's day is read at the ask's cut K and its body at its booking cut
//! min(t, K); a run-time refusal met evaluating an instance stays with that instance; a move's
//! instances are computed once per cut, and a move that reads its own instances is `RUN-CYCLE`.

use super::instance::{
    Amount, Booked, BookedLeg, Dated, End, Exists, Inst, LegVal, Seed, Status, inst_order,
};
use super::tuples::Tuple;
use crate::base::{Code, Load, Refusal, Run};
use crate::eval::{Ctx, Engine, Env, Snap, TracePart, one_day, thing_seen};
use crate::model::{
    Domain, KeyVal, MoveInfo, Node, NodeKind, Number, Reality, Value, World, currency_id,
    money_num, num_money,
};
use crate::num::Num;
use crate::syntax::{Certainty, Expr, Stop};
use crate::time::{Day, PeriodKind};
use std::rc::Rc;

impl Engine {
    /// A move's instances at cut `cut`, each dated at the cut and evaluated at its booking
    /// cut, sorted in the movement order, with their status (refcore's `move_instances`,
    /// `decision_instances`, `status`), shared with every other read of the same plan.
    /// E §4.1, §5 steps 0–1.
    ///
    /// # Errors
    ///
    /// `RUN-CYCLE` for a move that reads its own instances; the refusals the binding tuples
    /// meet; a load refusal met at run; `NotBuilt` past what S builds.
    pub(crate) fn move_instances(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        cut: Day,
    ) -> Result<Rc<Vec<Inst>>, Stop> {
        self.list_read(m, mv, cut)
    }

    /// A move's list at cut `cut`: the kept list; else the list made, what it booked kept with
    /// it or, where it stops, taken back, so every later read of it meets the same stop
    /// whatever was read before. E §4.1, §9 item 5 (P40).
    ///
    /// # Errors
    ///
    /// As [`Engine::move_instances`].
    fn list_read(&mut self, m: &Reality, mv: &MoveInfo, cut: Day) -> Result<Rc<Vec<Inst>>, Stop> {
        let key = (mv.id.clone(), cut);
        if let Some(hit) = self.plans.get(&key).map(Rc::clone) {
            return Ok(hit);
        }
        let mark = self.booked_open.len();
        self.making_open(mv)?;
        let made = self.make_list(m, mv, cut);
        self.making_close(mv)?;
        if self.plans.contains_key(&key) {
            self.booked_kept(mark);
        } else {
            self.booked_back(mark);
        }
        made
    }

    /// The list [`Engine::list_read`] makes: the instances, kept, then their statuses, what
    /// judging an instance's links meets that instance's own reads, which a reader that counts
    /// it takes and no other (L §8.2 One read set; E §3.2; tally 2.3.1 R2, item 6); then the
    /// hole bookings a later version answers. E §4.1, §5 steps 0–1.
    ///
    /// # Errors
    ///
    /// As [`Engine::move_instances`].
    fn make_list(&mut self, m: &Reality, mv: &MoveInfo, cut: Day) -> Result<Rc<Vec<Inst>>, Stop> {
        let key = (mv.id.clone(), cut);
        if self.plan_active.contains(&key) {
            self.narrow_open_to_days();
            return Err(Stop::Refused(Refusal::new(
                Code::Run(Run::Cycle),
                format!("{} reads its own instances", mv.id),
            )));
        }
        self.plan_active.insert(key.clone());
        let mut marks: Vec<(String, Snap)> = Vec::new();
        let made = if self.tracing() {
            self.traced_instances(m, mv, cut, &mut marks)
        } else {
            self.make_instances(m, mv, cut, None).map(|out| (out, None))
        };
        self.plan_active.remove(&key);
        let (mut out, back) = made?;
        // refcore computes a status when it is first read, after the instances are kept
        self.plans.insert(key.clone(), Rc::new(out.clone()));
        let mut due = false;
        for inst in &mut out {
            self.push();
            let status = self.status(m, mv, inst, cut);
            let met = self.pop_alone();
            inst.coll.merge(&met);
            inst.status = status?;
            due = due || crate::book::hole_booked_before(inst, cut);
        }
        if let Some(back) = &back {
            // the observations a status reads print nothing (refcore reads them untraced):
            // the plan isn't a walk the explanation lacks a form for
            self.trace_walked_back(*back);
        }
        let shared = Rc::new(out);
        if !due || mv.decision {
            self.plans.insert(key, Rc::clone(&shared));
            return Ok(shared);
        }
        self.plans.insert(key.clone(), Rc::clone(&shared));
        // a hole booking a later version answers is that version's here (E §5 step 1); a
        // stop other than a refusal leaves no list kept that it didn't answer
        if let Err(stop) = self.answer_hole_bookings(m, mv, cut) {
            self.plans.remove(&key);
            return Err(stop);
        }
        let answered = self.plans.get(&key).map_or(shared, Rc::clone);
        if back.is_some() {
            self.trace_answers(m, mv, (&answered, &marks), cut)?;
        }
        Ok(answered)
    }

    /// The answered instances of a list a read traces, each evaluated again at the cut of the
    /// version that answered it, traced, so the read names the rows that give the answer,
    /// listed where `marks` says the instance's own reads ended (refcore answers each booking
    /// where it makes the instance, `answer_booking`). E §10, §5 step 1; L §5.18.
    ///
    /// # Errors
    ///
    /// As [`Engine::make_instances`].
    fn trace_answers(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        (insts, marks): (&[Inst], &[(String, Snap)]),
        cut: Day,
    ) -> Result<(), Stop> {
        let outer = self.holding(Node::new(NodeKind::Move, &mv.id));
        self.trace_plan_open();
        let mut made = Ok(None);
        let mut done = Vec::new();
        for inst in insts {
            let Some((at, _)) = &inst.answered else {
                continue;
            };
            let since = self.trace_snap();
            let seed = Seed {
                env: inst.env.clone(),
                keys: inst.keys.clone(),
                exists: (*inst.exists0).clone(),
                at_cut: Some(*at),
                upto: None,
            };
            made = self.eval_inst(m, mv, &seed, self.timeline_ctx(None, cut));
            if made.is_err() {
                break;
            }
            let mark = marks.iter().find(|(id, _)| *id == inst.id);
            if let (Some(since), Some((_, mark))) = (since, mark) {
                self.trace_move_to(since, *mark, &mut done);
            }
        }
        // each answer is its instance's part of the pass (P20)
        self.trace_marks_moved(&done);
        let closed = self.trace_plan_close();
        self.holder = outer;
        made?;
        closed?;
        Ok(())
    }

    /// The day a list of move `mv`'s instances at cut `cut` stops at: a walk up to a day's,
    /// or none for the whole list, where [`Engine::plan_list`] takes it (refcore's
    /// `upto_tag`). E §4.1, §3.2.
    pub(crate) fn list_upto(&self, mv: &MoveInfo, cut: Day) -> Option<Day> {
        let upto = self.body_upto?;
        let whole = mv.decision || self.plans.contains_key(&(mv.id.clone(), cut));
        (!whole).then_some(upto)
    }

    /// A move's instances computed while a read is traced, the first time the ask reaches
    /// them: what they read is kept from the reading cell, as the plan's always is, but traced,
    /// so the rows and cells their bindings, days, `when`s and amounts read print under the
    /// read (refcore traces the instances it computes while it explains, once per ask); with
    /// the trace's note of a walk read then. E §10, §3.2, §4.1; L §5.18 Explanation order.
    ///
    /// # Errors
    ///
    /// As [`Engine::make_instances`].
    fn traced_instances(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        cut: Day,
        marks: &mut Vec<(String, Snap)>,
    ) -> Result<(Vec<Inst>, Option<crate::eval::Walked>), Stop> {
        let outer = self.holding(Node::new(NodeKind::Move, &mv.id));
        self.trace_plan_open();
        let parts = self.trace_marks_open();
        let made = self.make_instances_held(m, mv, (cut, None), Some(marks));
        self.trace_marks_close((&mv.id, cut), parts);
        let back = self.trace_plan_close();
        self.holder = outer;
        Ok((made?, Some(back?)))
    }

    /// A decision's instances computed again while a read that counts its runs is traced,
    /// kept nowhere: the rows and cells its bindings and days read print under the read, as
    /// a plan's do where a read computes its instances first; its `when` and legs are its
    /// runs' (`trace_runs`). E §10, §4.1; L §5.18 Explanation order.
    ///
    /// # Errors
    ///
    /// As [`Engine::make_instances`].
    pub(crate) fn trace_days(&mut self, m: &Reality, mv: &MoveInfo, cut: Day) -> Result<(), Stop> {
        let outer = self.holding(Node::new(NodeKind::Move, &mv.id));
        self.trace_plan_open();
        let parts = self.trace_marks_open();
        let made = self.make_instances_held(m, mv, (cut, None), None);
        self.trace_marks_close((&mv.id, cut), parts);
        let closed = self.trace_plan_close();
        self.holder = outer;
        made?;
        closed?;
        Ok(())
    }

    /// A move's instances for the walk being planned: all of them, or, for a walk up to a
    /// day, a plan's dated on or before it, the others' bodies never evaluated; a decision, or
    /// a move whose instances are all known already, is taken whole (refcore's `plan_list`).
    /// E §4.1, §3.2.
    ///
    /// # Errors
    ///
    /// As [`Engine::move_instances`].
    pub(crate) fn plan_list(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        cut: Day,
    ) -> Result<Rc<Vec<Inst>>, Stop> {
        let Some(upto) = self.list_upto(mv, cut) else {
            return self.move_instances(m, mv, cut);
        };
        let key = (mv.id.clone(), cut, upto);
        if let Some(hit) = self.prefixes.plans.get(&key) {
            return Ok(Rc::clone(hit));
        }
        // what it books stays with it, or is taken back, as a whole list's (P40)
        let mark = self.booked_open.len();
        self.making_open(mv)?;
        let made = self.make_prefix(m, mv, cut, upto);
        self.making_close(mv)?;
        if self.prefixes.plans.contains_key(&key) {
            self.booked_kept(mark);
        } else {
            self.booked_back(mark);
        }
        made
    }

    /// The list [`Engine::plan_list`] makes for a walk up to day `upto`: its instances dated
    /// on or before it, kept, then their statuses and the hole bookings a later version
    /// answers (refcore's `plan_list`). E §4.1, §3.2, §5 step 1.
    ///
    /// # Errors
    ///
    /// As [`Engine::move_instances`].
    fn make_prefix(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        cut: Day,
        upto: Day,
    ) -> Result<Rc<Vec<Inst>>, Stop> {
        let active = (mv.id.clone(), cut);
        let key = (mv.id.clone(), cut, upto);
        if self.plan_active.contains(&active) {
            self.narrow_open_to_days();
            return Err(Stop::Refused(Refusal::new(
                Code::Run(Run::Cycle),
                format!("{} reads its own instances", mv.id),
            )));
        }
        self.plan_active.insert(active.clone());
        let made = self.make_instances(m, mv, cut, Some(upto));
        self.plan_active.remove(&active);
        let mut out = made?;
        // kept before their status, as the whole list is
        self.prefixes
            .plans
            .insert(key.clone(), Rc::new(out.clone()));
        let mut due = false;
        for inst in &mut out {
            // what judging its links meets is the instance's own, as in the whole list
            self.push();
            let status = self.status(m, mv, inst, cut);
            let met = self.pop_alone();
            inst.coll.merge(&met);
            inst.status = status?;
            due = due || crate::book::hole_booked_before(inst, cut);
        }
        let shared = Rc::new(out);
        if !due {
            self.prefixes.plans.insert(key, Rc::clone(&shared));
            return Ok(shared);
        }
        self.prefixes.plans.insert(key.clone(), Rc::clone(&shared));
        // a hole booking a later version answers is that version's here too, measured over
        // this list (E §5 step 1); a stop other than a refusal leaves no list kept that it
        // didn't answer
        if let Err(stop) = self.answer_hole_bookings(m, mv, cut) {
            self.prefixes.plans.remove(&key);
            return Err(stop);
        }
        Ok(self.prefixes.plans.get(&key).map_or(shared, Rc::clone))
    }

    /// An instance's status at cut `cut`, the observations that fulfil it judged first
    /// (refcore's `status`). E §4.1 step 3, L §5.19.
    pub(crate) fn status(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &mut Inst,
        cut: Day,
    ) -> Result<Status, Stop> {
        inst.obs = self.obs_links_for(m, &inst.id, cut)?;
        // an observation dated on K or later is seen but not applied, and nor is the link it
        // states, so the instance it fulfils stays in the plan (L §4.5, §5.19 Coverage; tally
        // 2.1.8 R12, #182)
        let applied = self.links_applied(m, &inst.obs, cut)?;
        let day = inst.day.as_ref().and_then(Dated::day);
        if inst.exists == Exists::No {
            return Ok(Status::None);
        }
        if let Some(d) = day
            && (d < m.start || d > m.end)
        {
            return Ok(Status::LeftOut);
        }
        if scenario_early(m, mv, inst) {
            return Ok(Status::LeftOut);
        }
        if inst.err.is_some() {
            return Ok(Status::Refused);
        }
        if applied && !mv.decision {
            return Ok(Status::Modelled);
        }
        let Some(d) = day else {
            return Ok(Status::Hole);
        };
        if matches!(inst.exists, Exists::Unknown(_)) {
            return Ok(Status::Hole);
        }
        if mv.decision {
            return Ok(Status::Decision);
        }
        if kind_certainty(m, &mv.kind) == Some(Certainty::Uncertain) && !applied {
            return Ok(Status::Uncertain);
        }
        if d < cut && self.touches_covered(m, inst, cut) {
            return Ok(Status::Modelled);
        }
        Ok(if d < cut { Status::Past } else { Status::Plan })
    }

    /// Whether one of the observations `obs` that fulfil an instance is applied at cut `cut`:
    /// dated before it (refcore's `m_status`, its `applied`). L §4.5 (what a day applies is
    /// applied only when dated before K; tally 2.1.8 R12), §5.19, E §4.1 step 3.
    ///
    /// # Errors
    ///
    /// As [`Engine::observations`].
    fn links_applied(&mut self, m: &Reality, obs: &[String], cut: Day) -> Result<bool, Stop> {
        if obs.is_empty() {
            return Ok(false);
        }
        Ok(self
            .observations(m, cut)?
            .iter()
            .any(|o| o.day < cut && obs.contains(&o.id)))
    }

    /// Every instance of a move at its binding tuples, sorted (refcore's `insts_of_tuples`).
    /// E §4.1 steps 1–2.
    pub(crate) fn make_instances(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        cut: Day,
        upto: Option<Day>,
    ) -> Result<Vec<Inst>, Stop> {
        let outer = self.holding(Node::new(NodeKind::Move, &mv.id));
        let mark = self.reads_apart();
        let made = self.make_instances_held(m, mv, (cut, upto), None);
        self.reads_back(mark);
        self.holder = outer;
        made
    }

    /// [`Engine::make_instances`] with the move holding the projections its `at`, its
    /// bindings' `where` and its body read, which are the move's own (L §5.14.1: "the item
    /// holding the read"); what it reads is the plan's, shared by every reader (E §3.2, §4.1).
    /// While a read is traced, `marks` takes where each instance's own reads end, for its
    /// answer's (E §10).
    fn make_instances_held(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        (cut, upto): (Day, Option<Day>),
        mut marks: Option<&mut Vec<(String, Snap)>>,
    ) -> Result<Vec<Inst>, Stop> {
        let tctx = self.timeline_ctx(None, cut);
        let mut out = Vec::new();
        for tuple in self.move_tuples(m, mv, tctx)? {
            let made = self.tuple_instance(m, mv, (tuple, cut, upto), tctx, marks.as_deref_mut());
            out.extend(made?);
        }
        for inst in &mut out {
            inst.order = inst_order(m, mv, inst);
        }
        out.sort_by(|a, b| a.order.cmp(&b.order));
        Ok(out)
    }

    /// The instance of one binding tuple of a list at cut `cut`, as the plan pass makes it: a
    /// refused tuple's refused instance, a hole tuple's hole instance, else its day at the cut
    /// and its body at its booking cut, its booked amount checked; none where its day is
    /// `never` or `none` (refcore's `insts_of_tuples`, its loop). E §4.1 steps 1–2, §5 step 1.
    ///
    /// # Errors
    ///
    /// As [`Engine::refused_tuple`] and [`Engine::eval_inst`]; `ENGINE-INVARIANT` for a booked
    /// amount that changed.
    pub(super) fn tuple_instance(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        (tuple, cut, upto): (Tuple, Day, Option<Day>),
        tctx: Ctx,
        marks: Option<&mut Vec<(String, Snap)>>,
    ) -> Result<Option<Inst>, Stop> {
        if let Some(err) = tuple.refused.clone() {
            return self.refused_tuple(m, mv, (tuple, upto), err, tctx);
        }
        if let Some(hole) = tuple.domain_hole {
            let mut inst = Inst::new(mv, tuple.keys, tuple.env);
            inst.day = Some(Dated::Hole(hole.clone()));
            inst.exists = Exists::Unknown(hole);
            inst.coll = tuple.coll;
            return Ok(Some(inst));
        }
        let seed = Seed {
            env: tuple.env,
            keys: tuple.keys,
            exists: tuple.exists,
            at_cut: None,
            upto,
        };
        // where its reads begin and end, while a read traces the pass (P20)
        let id = self
            .trace_marking()
            .then(|| crate::book::inst_id(&mv.id, &seed.keys));
        if let Some(id) = &id {
            self.trace_part(id, TracePart::Begin);
        }
        let made = self.eval_inst(m, mv, &seed, tctx)?;
        if let Some(id) = &id {
            self.trace_part(id, TracePart::End);
        }
        let Some(mut inst) = made else {
            return Ok(None);
        };
        if !mv.decision
            && inst.err.is_none()
            && let Some(booking) = inst.cut
        {
            self.check_booked(&inst, (booking, cut), upto)?;
        }
        inst.coll.merge(&tuple.coll);
        if let Some(exist) = &mut inst.exist_coll {
            exist.merge(&tuple.coll);
        }
        if let (Some(marks), Some(mark)) = (marks, self.trace_snap()) {
            marks.push((inst.id.clone(), mark));
        }
        Ok(Some(inst))
    }

    /// The instance of a tuple whose key's `where` is refused: a refused instance, its day and
    /// legs read as any instance's, none where its `when` is false or its keys weren't there;
    /// a tuple whose later keys are unbound stands for itself alone
    /// ([`Engine::stopped_tuple`]), never for the move's list (refcore's `insts_of_tuples`,
    /// its refused tuples). L §5.14 Keys, An instance's existence is a cell, §8.4 (tally
    /// 2.1.6, 2.3.1 R6); E §4.1 step 2.
    ///
    /// # Errors
    ///
    /// A load refusal met at run.
    pub(super) fn refused_tuple(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        (tuple, upto): (Tuple, Option<Day>),
        err: Refusal,
        tctx: Ctx,
    ) -> Result<Option<Inst>, Stop> {
        if tuple.keys.len() < mv.binds.len() {
            return self.stopped_tuple(m, mv, (tuple, upto), err, tctx);
        }
        let seed = Seed {
            env: tuple.env,
            keys: tuple.keys,
            exists: Exists::Yes,
            at_cut: None,
            upto,
        };
        let Some(mut inst) = self.eval_inst(m, mv, &seed, tctx)? else {
            return Ok(None);
        };
        if inst.exists == Exists::No {
            return Ok(None);
        }
        inst.err = Some(err);
        inst.coll.merge(&tuple.coll);
        Ok(Some(inst))
    }

    /// The instance of a tuple refused before its later keys are bound: a refused movement
    /// standing for itself alone, its existence read on in its order, its day, then its
    /// `when`, each left unread where it reads a key the refusal left unbound: none where the
    /// day is `never` or `none` or the `when` false; a hole day reaches every span, a day
    /// refused or unread every place from the run's start; its legs' places read where they
    /// need no such key, else every place from its day (L §5.14 An instance's existence is a
    /// cell, §8.4; tally 2.3.1 R6, P40's list stop overturned). E §4.1 step 2.
    ///
    /// # Errors
    ///
    /// A load refusal met at run.
    fn stopped_tuple(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        (tuple, upto): (Tuple, Option<Day>),
        err: Refusal,
        tctx: Ctx,
    ) -> Result<Option<Inst>, Stop> {
        let mut inst = Inst::new(mv, tuple.keys, tuple.env);
        inst.err = Some(err);
        self.push();
        let made = self.stopped_parts(m, mv, &mut inst, (tctx, upto));
        let met = self.pop_alone();
        inst.coll.merge(&met);
        inst.coll.merge(&tuple.coll);
        Ok(made?.then_some(inst))
    }

    /// The parts [`Engine::stopped_tuple`] reads into refused instance `inst`; false where it
    /// doesn't exist. L §5.14, §8.4; E §5 steps 0–1.
    ///
    /// # Errors
    ///
    /// A load refusal met at run.
    fn stopped_parts(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &mut Inst,
        (tctx, upto): (Ctx, Option<Day>),
    ) -> Result<bool, Stop> {
        let bound = inst.keys.len();
        if !reads_unbound(mv, bound, &mv.decl.on) {
            self.push();
            let read = self.law_and_day(m, mv, inst, tctx);
            let day_coll = self.pop_alone();
            self.absorb(&day_coll);
            inst.day_coll = day_coll;
            let day = match read {
                Ok(Ok(Value::Never | Value::None)) => return Ok(false),
                Ok(Ok(v)) => day_dated(m, v, &inst.id).map(Some),
                Ok(Err(h)) => Ok(Some(Dated::Hole(h))),
                Err(stop) => Err(stop),
            };
            match day {
                Ok(day) => inst.day = day,
                // a refused day as a hole one: the where's refusal is the instance's (R1)
                Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => {}
                Err(stop) => return Err(stop),
            }
        }
        let day = match inst.day {
            // a hole day has no booking cut to read the rest at, as `eval_inst`'s
            Some(Dated::Hole(_)) => return Ok(true),
            Some(Dated::Day(d)) => Some(d),
            None => None,
        };
        if mv.decision {
            return Ok(true);
        }
        if let Some(d) = day {
            if upto.is_some_and(|u| d > u) {
                // a walk up to a day books nothing after it (refcore's `eval_inst`, `upto`)
                return Ok(false);
            }
            let cut = d.min(tctx.cut);
            inst.cut = Some(cut);
            if !keys_seen(m, mv, &inst.keys, (cut, self.timeline_world()))
                || !self.stopped_when(m, mv, inst, cut)?
            {
                return Ok(false);
            }
        }
        let places = mv
            .decl
            .legs
            .iter()
            .all(|lg| !reads_unbound(mv, bound, &lg.from) && !reads_unbound(mv, bound, &lg.to));
        if places {
            self.refused_legs(m, mv, inst, (tctx, None))?;
        }
        Ok(true)
    }

    /// Whether refused instance `inst`, dated, may exist by its move's `when` at booking cut
    /// `cut`: false only for a `when` that is false; one that reads a key left unbound isn't
    /// read. L §5.14 An instance's existence is a cell (tally 2.3.1 R6, P42).
    ///
    /// # Errors
    ///
    /// A load refusal met at run.
    fn stopped_when(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &Inst,
        cut: Day,
    ) -> Result<bool, Stop> {
        let Some(when) = &mv.decl.when else {
            return Ok(true);
        };
        if reads_unbound(mv, inst.keys.len(), when) {
            return Ok(true);
        }
        let ctx = self.timeline_ctx(inst.law.day(), cut);
        match self.ev(m, when, &inst.env, ctx) {
            Ok(w) => Ok(w != Value::Bool(false)),
            Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => Ok(true),
            Err(stop) => Err(stop),
        }
    }

    /// A booked plan amount never changes: an instance's A₀, its existence and legs as read at
    /// its booking cut `booking`, is the same under every cut that books it there, across the
    /// asks of a run (refcore's `inv5`). Its day, its legs' too, is read at the cut K, so it
    /// isn't part of A₀ (E §4.1 step 2). E §9 item 5.
    fn check_booked(
        &mut self,
        inst: &Inst,
        (booking, cut): (Day, Day),
        upto: Option<Day>,
    ) -> Result<(), Stop> {
        let key = (inst.id.clone(), booking, upto);
        let legs = booked_legs(&inst.legs);
        let Some(old) = self.booked.get(&key) else {
            let first = Booked {
                exists: inst.exists.clone(),
                legs,
                cut,
            };
            self.booked_open.push(key.clone());
            self.booked.insert(key, first);
            return Ok(());
        };
        if old.exists == inst.exists && old.legs == legs {
            return Ok(());
        }
        Err(invariant(&format!(
            "a booked plan amount changed: {} booked at {} differs between the cuts {} and {} \
             (E §9 invariant 5)",
            inst.id,
            booking.text(),
            old.cut.text(),
            cut.text()
        )))
    }

    /// Opens the making of a list of move `mv`, counted where it is a plan: a decision's
    /// existence traced inside one is a plan reaching a running read (E §3.1 step 7; tally
    /// 2.3.1 R6, P43).
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` past the counter.
    pub(super) fn making_open(&mut self, mv: &MoveInfo) -> Result<(), Stop> {
        if !mv.decision {
            self.plans_making = self
                .plans_making
                .checked_add(1)
                .ok_or_else(|| invariant("the plan lists being made"))?;
        }
        Ok(())
    }

    /// Closes what [`Engine::making_open`] opened. E §3.1 step 7.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a close with none open.
    pub(super) fn making_close(&mut self, mv: &MoveInfo) -> Result<(), Stop> {
        if !mv.decision {
            self.plans_making = self
                .plans_making
                .checked_sub(1)
                .ok_or_else(|| invariant("the plan lists being made"))?;
        }
        Ok(())
    }

    /// The amounts a list or an instance booked first since `mark`, once it is kept: they stay
    /// booked with it. E §9 item 5 (P40).
    pub(super) fn booked_kept(&mut self, mark: usize) {
        self.booked_open.truncate(mark);
    }

    /// The amounts a list booked first since `mark`, once it stopped: taken back, so nothing
    /// booked from a list that stopped stays in the walk, and a later read of the list, in
    /// this ask or a later one, makes it from the same bookings and meets the same stop
    /// whatever was read before. E §9 item 5, §4.1 (P40).
    fn booked_back(&mut self, mark: usize) {
        while self.booked_open.len() > mark
            && let Some(key) = self.booked_open.pop()
        {
            self.booked.remove(&key);
        }
    }

    /// One instance: its day at the cut K, then, for a plan, its body at its booking cut
    /// min(t, K) or at the seed's cut; none when its day is `never` or `none` (refcore's
    /// `eval_inst`). A run-time refusal stays with the instance, a refused movement, and a
    /// plan's legs are then read for their places (tally 2.1.6). E §5 steps 0–1, E §4.1
    /// step 2.
    ///
    /// # Errors
    ///
    /// A load refusal met at run; `NotBuilt` past what S builds.
    pub(crate) fn eval_inst(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        seed: &Seed,
        tctx: Ctx,
    ) -> Result<Option<Inst>, Stop> {
        let mut inst = Inst::new(mv, seed.keys.clone(), seed.env.clone());
        inst.exists0 = Box::new(seed.exists.clone());
        self.push();
        let made = match self.inst_parts(m, mv, &mut inst, tctx, seed) {
            Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => {
                inst.err = Some(r);
                self.refused_legs(m, mv, &mut inst, (tctx, seed.at_cut))
                    .map(|()| true)
            }
            made => made,
        };
        let met = self.pop_alone();
        inst.coll.merge(&met);
        Ok(made?.then_some(inst))
    }

    /// A refused plan instance's legs, its evaluation stopped before them: their places read
    /// at the cut its body is measured at (its booking cut, or a later version's for a
    /// re-measure, where refcore reads min(t, K)) and its law date, each leg's amount the refusal; none where a place
    /// can't be read here, so the walk refuses every place from its day (refcore's
    /// `eval_inst`, its refusal). A refused movement reaches as a hole movement does: its day
    /// and its legs' places are still read, since neither depends on whether it exists (L §8.4,
    /// tally 2.1.6). E §4.1 step 2, §4.2 Holes in the walk.
    ///
    /// # Errors
    ///
    /// A load refusal met at run.
    fn refused_legs(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &mut Inst,
        (tctx, at_cut): (Ctx, Option<Day>),
    ) -> Result<(), Stop> {
        if mv.decision {
            return Ok(());
        }
        let day = inst.day.as_ref().and_then(Dated::day);
        // the cut its body is measured at: a later version's, for a re-measure (E §5 step 1),
        // else min(t, K)
        let cut = match (at_cut, day) {
            (Some(c), _) => c,
            (None, Some(d)) => d.min(tctx.cut),
            (None, None) => tctx.cut,
        };
        if let Some(legs) =
            self.refused_places(m, mv, inst, self.timeline_ctx(inst.law.day(), cut))?
        {
            inst.legs = legs;
        }
        Ok(())
    }

    /// The legs of refused instance `inst` with their places read at `ctx` and no amount, the
    /// refusal in its place, each on the instance's day; none where a place isn't a thing
    /// here, a hole or a refusal, since such a leg could book anywhere (refcore's
    /// `refused_places`, and `dec_bad`'s places). An instance refused before its day may fall
    /// on any day: its legs are dated the end of time, no day (L §7.6 item 1), which falls
    /// before no scenario's `from` (L §6.4: refcore's legs there have no day). L §8.4, E §4.2
    /// Holes in the walk.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for an instance with no refusal; a load refusal met at run.
    pub(super) fn refused_places(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &Inst,
        ctx: Ctx,
    ) -> Result<Option<Vec<LegVal>>, Stop> {
        let err = inst
            .err
            .clone()
            .ok_or_else(|| invariant("a refused instance with no refusal"))?;
        let day = inst.day.clone().unwrap_or(Dated::Day(Day::END));
        let Some(places) = self.leg_ends(m, mv, inst, ctx)? else {
            return Ok(None);
        };
        let legs = mv
            .decl
            .legs
            .iter()
            .zip(places)
            .map(|(leg, (from, to))| LegVal {
                name: leg.name.clone(),
                from: End::Place(from),
                to: End::Place(to),
                amount: Amount::Refused(err.clone()),
                unit: None,
                day: day.clone(),
                src: None,
            })
            .collect();
        Ok(Some(legs))
    }

    /// The places each leg of instance `inst` gives, read at `ctx`; none where a place isn't
    /// a thing here, a hole or a refusal (refcore's `refused_places`). L §8.4, §8.2.
    ///
    /// # Errors
    ///
    /// A load refusal met at run.
    fn leg_ends(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &Inst,
        ctx: Ctx,
    ) -> Result<Option<Vec<(String, String)>>, Stop> {
        let mut out = Vec::with_capacity(mv.decl.legs.len());
        for leg in &mv.decl.legs {
            let Some(from) = self.read_or_refused(m, &leg.from, &inst.env, ctx)? else {
                return Ok(None);
            };
            let Some(to) = self.read_or_refused(m, &leg.to, &inst.env, ctx)? else {
                return Ok(None);
            };
            let (Value::Key(KeyVal::Thing(from)), Value::Key(KeyVal::Thing(to))) = (from, to)
            else {
                return Ok(None);
            };
            out.push((from, to));
        }
        Ok(Some(out))
    }

    /// The places, each with its unit, where the reads that decided instance `inst` doesn't
    /// exist (a false `when` or key `where`) join a balance's read set from its day: those
    /// its legs give, read at its booking cut (refcore's `note_none`). L §8.2 One read set
    /// (tally 2.1.7 R7, T's SR-8).
    ///
    /// # Errors
    ///
    /// A load refusal met at run.
    pub(super) fn decided_places(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &Inst,
        cut: Day,
    ) -> Result<Vec<(String, String)>, Stop> {
        let mut ends: Vec<(String, Option<String>)> = Vec::new();
        if inst.legs.is_empty() {
            let day = inst.day.as_ref().and_then(Dated::day);
            let ctx = self.timeline_ctx(inst.law.day(), day.map_or(cut, |d| d.min(cut)));
            let Some(places) = self.leg_ends(m, mv, inst, ctx)? else {
                return Ok(Vec::new());
            };
            for (from, to) in places {
                ends.push((from, None));
                ends.push((to, None));
            }
        }
        for lg in &inst.legs {
            for place in [&lg.from, &lg.to].into_iter().filter_map(End::place) {
                ends.push((place.to_owned(), lg.unit.clone()));
            }
        }
        let mut out = Vec::new();
        for (place, unit) in ends {
            let unit = match unit {
                Some(unit) => unit,
                None => m
                    .things
                    .get(&place)
                    .and_then(|thing| m.place_unit(thing))
                    .unwrap_or_else(|| super::state::ALL.to_owned()),
            };
            out.push((place, unit));
        }
        Ok(out)
    }

    /// The value of `e` at `ctx`, or none where it is refused at run (refcore's `try: ev …
    /// except TallyError`); a load refusal met at run and `ENGINE-INVARIANT` go on (R-66).
    /// L §8.4.
    pub(super) fn read_or_refused(
        &mut self,
        m: &Reality,
        e: &Expr,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Option<Value>, Stop> {
        match self.ev(m, e, env, ctx) {
            Ok(v) => Ok(Some(v)),
            Err(Stop::Refused(r)) if super::order::run_time(&r) => Ok(None),
            Err(stop) => Err(stop),
        }
    }

    /// The parts of an instance, evaluated into it; false when its day is `never` or `none`
    /// (refcore's `eval_inst`, its body). E §5 steps 0–1.
    fn inst_parts(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &mut Inst,
        tctx: Ctx,
        seed: &Seed,
    ) -> Result<bool, Stop> {
        // what reading its law date and its day met is what a read of its day brings, a pin
        // of the law date too (refcore's `day_coll`; the review of G1, N1)
        self.push();
        let read = self.law_and_day(m, mv, inst, tctx);
        let day_coll = self.pop_alone();
        self.absorb(&day_coll);
        inst.day_coll = day_coll;
        if self.trace_marking() {
            self.trace_part(&inst.id, TracePart::Day);
        }
        let v = match read? {
            Err(h) => {
                inst.day = Some(Dated::Hole(h.clone()));
                inst.exists = Exists::Unknown(h);
                return Ok(true);
            }
            Ok(v) => v,
        };
        if matches!(v, Value::Never | Value::None) {
            return Ok(false);
        }
        let day = day_dated(m, v, &inst.id)?;
        inst.day = Some(day.clone());
        inst.exists = seed.exists.clone();
        if mv.decision {
            return Ok(true);
        }
        if let (Some(upto), Some(d)) = (seed.upto, day.day())
            && d > upto
        {
            // a walk up to a day books nothing after it (refcore's `eval_inst`, `upto`)
            return Ok(false);
        }
        // the booking cut min(t, K) (E §5 step 1)
        let cut = match seed.at_cut {
            Some(c) => c,
            None => day.day().map_or(tctx.cut, |d| d.min(tctx.cut)),
        };
        inst.cut = Some(cut);
        // at a booking cut that doesn't see its keys the instance doesn't exist (E §5 steps 1–3)
        if !keys_seen(m, mv, &inst.keys, (cut, self.timeline_world())) {
            inst.exists = Exists::No;
            return Ok(true);
        }
        let ctx = self.timeline_ctx(inst.law.day(), cut);
        let body = self.inst_body(m, mv, inst, ctx, inst.exists.clone())?;
        inst.exists = body.exists;
        inst.legs = body.legs;
        inst.leg_reads = body.leg_reads;
        inst.exist_coll = Some(body.exist);
        inst.attrs = body.attrs;
        inst.for_period = body.for_period;
        inst.fulfils = body.fulfils;
        Ok(true)
    }
}

/// Whether `e` reads a key of move `mv` past its first `bound` bindings, which a tuple refused
/// before them leaves unbound (the walk's `move_law`, by the same `reads_var`). L §5.14 Keys.
fn reads_unbound(mv: &MoveInfo, bound: usize, e: &Expr) -> bool {
    mv.binds
        .iter()
        .skip(bound)
        .any(|b| crate::eval::reads_var(e, &b.var))
}

/// Whether the keys of an instance were there at cut `cut` in world `world`: an index's key
/// added by a later version or one the world doesn't see, or a thing entered after it, makes
/// the instance one that didn't exist then (refcore's `keys_seen`). E §5 steps 1–3; L §6.4.
fn keys_seen(m: &Reality, mv: &MoveInfo, keys: &[KeyVal], (cut, world): (Day, World)) -> bool {
    for (bind, key) in mv.binds.iter().zip(keys) {
        match &bind.domain {
            Domain::Index(id) => {
                let added = m.indexes.get(id).and_then(|ix| ix.added_by(key));
                let unseen = added.is_some_and(|v| !m.scenarios.sees(world, Some(v)));
                if unseen
                    || added
                        .and_then(|v| m.versions.get(v))
                        .is_some_and(|v| v.known > cut)
                {
                    return false;
                }
            }
            Domain::Kind(_) => {
                if let KeyVal::Thing(name) = key
                    && let Some(thing) = m.things.get(name)
                    && !thing_seen(m, thing, (cut, world))
                {
                    return false;
                }
            }
            Domain::Axis(_) | Domain::List(_) | Domain::Range => {}
        }
    }
    true
}

/// A₀'s legs: each leg as read at its booking cut, without its day, which is its instance's,
/// read at the cut K (refcore's `body_key`). E §4.1 step 2, E §9 item 5.
fn booked_legs(legs: &[LegVal]) -> Vec<BookedLeg> {
    let mut out = Vec::with_capacity(legs.len());
    for lg in legs {
        out.push(BookedLeg {
            name: lg.name.clone(),
            from: lg.from.clone(),
            to: lg.to.clone(),
            amount: lg.amount.clone(),
            unit: lg.unit.clone(),
        });
    }
    out
}

/// Whether `inst` is an instance of a move a scenario declares whose day falls before the
/// scenario's `from`: it books nothing there, so it is left out; load has refused the move
/// where it sees that day and that the instance exists (refcore's `scen_early`). L §6.4.
fn scenario_early(m: &Reality, mv: &MoveInfo, inst: &Inst) -> bool {
    let Some(from) = scenario_from(m, &mv.id) else {
        return false;
    };
    inst.day
        .as_ref()
        .and_then(Dated::day)
        .is_some_and(|d| d < from)
}

/// The `from` of the scenario whose file declares move `id`, or `None` for a move of the base
/// (refcore's `scen_from`). L §6.4.
#[must_use]
pub(super) fn scenario_from(m: &Reality, id: &str) -> Option<Day> {
    let scenario = m.scenarios.of(id)?;
    m.scenarios.from.get(scenario).copied()
}

/// An event kind's certainty, its own or its nearest parent's (refcore's `kind_certainty`).
/// L §5.7.
#[must_use]
pub fn kind_certainty(m: &Reality, kind: &str) -> Option<Certainty> {
    m.chain(kind).into_iter().find_map(|k| k.certainty)
}

/// A number or money as a quantity, with its unit where it is one unit to the first power
/// (refcore's `unit_of_value`). L §3.2.
pub(crate) fn quantity(m: &Reality, v: &Value) -> Result<(Num, Option<String>), Stop> {
    if let Value::Money(money) = v {
        let q = money_num(m, money).map_err(num_err)?;
        let (name, _) = crate::model::currency(m, money.currency()).map_err(num_err)?;
        return Ok((q, Some(name.to_owned())));
    }
    if let Value::Number(n) = v {
        return Ok((n.q.clone(), single_unit(n)));
    }
    Err(invariant("a quantity that isn't a number"))
}

/// A number's unit when it is one unit to the first power. L §3.2.
fn single_unit(n: &Number) -> Option<String> {
    match n.unit().terms() {
        [(name, 1)] => Some(name.clone()),
        _ => None,
    }
}

/// A quantity of `unit` as a value: money for a currency, a number of the unit otherwise
/// (refcore's `qty`). L §3.2.
///
/// # Errors
///
/// A number error: money finer than its minor unit, a unit past the cap.
pub fn qty_value(m: &Reality, q: &Num, unit: &str) -> Result<Value, Stop> {
    if m.currencies.contains_key(unit) {
        let cur = currency_id(m, unit).map_err(num_err)?;
        return Ok(Value::Money(num_money(m, q, cur).map_err(num_err)?));
    }
    let u = crate::model::Unit::make(vec![(unit.to_owned(), 1)]).map_err(num_err)?;
    Ok(Value::Number(Number::plain(q.clone(), u)))
}

/// The day of instance `id` that its `on` gives as `v`, neither `never` nor `none`: a day, a
/// hole, or a window's day, a hole where it is imprecise (refcore's `eval_inst`, its day).
/// L §4.4, §5.14, E §4.1 step 2.
///
/// # Errors
///
/// A load refusal met at run, for a value that isn't a day.
pub(super) fn day_dated(m: &Reality, v: Value, id: &str) -> Result<Dated, Stop> {
    // a window written in its own `on`, which no cell holds, is named by its movement
    // (L §8.2 `unknown day:`, "where no cell does, its movement"; R21 on #241)
    let v = crate::eval::held_by(v, (format!("move {id}"), id));
    Ok(match v {
        Value::Hole(h) => Dated::Hole(h),
        // a window: a hole day, `HOLE-IMPRECISE` for the day of the instance, as any
        // movement's whose day is a hole from there (L §4.4, §5.14; tally 2.3 S6)
        Value::Window(_) => match one_day(m, &v, &format!("the day of {id}"))? {
            Ok(d) => Dated::Day(d),
            Err(h) => Dated::Hole(h),
        },
        Value::Number(_)
        | Value::Money(_)
        | Value::Bool(_)
        | Value::Day(_)
        | Value::Period(_)
        | Value::MonthDay(..)
        | Value::Key(_)
        | Value::Text(_)
        | Value::Code(_)
        | Value::Record(_)
        | Value::List(_)
        | Value::Tuple(_)
        | Value::Calendar(_)
        | Value::None
        | Value::Never
        | Value::Word(_)
        | Value::Mv(_)
        | Value::OutKey { .. } => Dated::Day(date_of(&v)?),
    })
}

/// The day a value names: a day, or a day period's (refcore's `date_of`); a value that isn't
/// one is a load refusal met at run. L §4.2.
pub(super) fn date_of(v: &Value) -> Result<Day, Stop> {
    if let Value::Day(d) = v {
        return crate::eval::a_day(*d);
    }
    if let Value::Period(p) = v
        && p.kind() == PeriodKind::Day
    {
        return Ok(p.start());
    }
    Err(refuse_load(
        Load::Unit,
        "a day that isn't a date".to_owned(),
    ))
}

/// A value as it prints, for a message. L §8.4.
pub(super) fn shown(m: &Reality, v: &Value) -> Result<String, Stop> {
    v.text(m).map_err(num_err)
}

/// A number error as its refusal (L §7.1).
pub(super) fn num_err(e: crate::num::NumError) -> Stop {
    crate::eval::num_stop(e)
}

/// A run-time refusal (L §8.4).
pub(super) fn refuse(code: Run, message: String) -> Stop {
    Stop::Refused(Refusal::new(Code::Run(code), message))
}

/// A check of load's met at run, on a value load couldn't see: `RUN-*` in load's words at
/// the value, and the run goes on (L §8.4 R-load, tally 2.1.7 R5; refcore's `moves.py`
/// `refuse`, its `at_run`).
pub(super) fn refuse_load(code: Load, message: String) -> Stop {
    Stop::Refused(Refusal::new(code.at_run(), message))
}

/// A broken promise: `ENGINE-INVARIANT` (E §11).
pub(crate) fn invariant(what: &str) -> Stop {
    Stop::Refused(Refusal::new(Code::EngineInvariant, what.to_owned()))
}

#[cfg(test)]
mod tests {
    use crate::api::{Asks, Files, Given, Request, run};

    /// A plan over two bindings whose first key's `where` is refused in February (a
    /// division by zero), before its second binding.
    const LATER_KEY: &str = r#"model later_key
  tally 2
  on 2026-02-15
  start 2026-01-01
  end 2026-03-31

source fixture "invented values"

unit PLN: currency minor 0.01

index month = month 2026-01..2026-03

index tier = 1..2

place konto: accounts PLN lives 2026-01-01..

place box: accounts PLN lives 2026-01-01..

place world: outside lives 2026-01-01..

input dec_dom: int

input fee_amt: money PLN

input wdiv[m in month]: num

move topup[m in month where 1 / wdiv[m] > 0, t in tier]: transfers
  on day_of(m, dec_dom)
  leg cash from world to box amount fee_amt

ask q_bal: bal(konto, d), bal(box, d) for d in [2026-01-20, 2026-03-20]

version facts
  cite fixture
  governs always
  known 2025-12-01
  dec_dom = 20 "the day of the month"
  fee_amt = 3.00 "a fee"
  wdiv[2026-01] = 1 "answered"
  wdiv[2026-02] = 0 "a division by zero: the key's where is refused"
  wdiv[2026-03] = 2 "answered"
  opening[konto] = 100 at 2026-01-01 start
  opening[box] = 0 at 2026-01-01 start
"#;

    #[test]
    fn a_refused_where_before_a_later_binding_is_one_refused_movement_on_its_day() {
        // L §5.14 An instance's existence is a cell, §8.4 (tally 2.3.1 R6, P40's list stop
        // overturned): topup[2026-02]'s where is refused before its tier is bound; its day
        // and its leg's places need no tier, so it is one refused movement on 2026-02-20 at
        // box, and the other instances stand
        let given = vec![Given {
            path: "later_key.tally".to_owned(),
            bytes: Some(LATER_KEY.as_bytes().to_vec()),
        }];
        let request = Request::new(Asks::Named(vec!["q_bal".to_owned()]));
        let answer = run(Files::new(given), &request).expect("an answer");
        assert_eq!(
            answer.lines(),
            [
                "q_bal (d = 2026-01-20): (100.00 PLN, 6.00 PLN)",
                "  unfulfilled: topup[2026-01, 1]",
                "  unfulfilled: topup[2026-01, 2]",
                "  cut 2026-02-15",
                "q_bal (d = 2026-03-20): (100.00 PLN, refused: RUN-DIV division by zero)",
                "  unfulfilled: topup[2026-01, 1]",
                "  unfulfilled: topup[2026-01, 2]",
                "  cut 2026-02-15"
            ],
            "box refused from February's day, konto untouched"
        );
    }

    #[test]
    fn a_refused_tuple_whose_day_needs_a_later_key_refuses_its_places_from_the_start() {
        // L §5.14, §8.4 (tally 2.3.1 R6): the day reads the tier the refusal left unbound, so
        // it can't be read, a refused day as a hole one: box is refused from the run's start;
        // its leg's places are still read, so konto stands
        let text = LATER_KEY.replace("on day_of(m, dec_dom)", "on day_of(m, dec_dom + t)");
        let given = vec![Given {
            path: "later_key.tally".to_owned(),
            bytes: Some(text.into_bytes()),
        }];
        let request = Request::new(Asks::Named(vec!["q_bal".to_owned()]));
        let answer = run(Files::new(given), &request).expect("an answer");
        assert_eq!(
            answer.lines(),
            [
                "q_bal (d = 2026-01-20): (100.00 PLN, refused: RUN-DIV division by zero)",
                "  cut 2026-02-15",
                "q_bal (d = 2026-03-20): (100.00 PLN, refused: RUN-DIV division by zero)",
                "  unfulfilled: topup[2026-01, 1]",
                "  unfulfilled: topup[2026-01, 2]",
                "  cut 2026-02-15"
            ],
            "box refused from the start"
        );
    }

    /// A plan over three months whose `when` is refused in February, and its leg's place
    /// there too.
    const REFUSED_PLACE: &str = r#"model refused_place
  tally 2
  on 2026-02-15
  start 2026-01-01
  end 2026-03-31

source fixture "invented values"

unit PLN: currency minor 0.01

index month = month 2026-01..2026-03


place konto: accounts PLN lives 2026-01-01..

place box: accounts PLN lives 2026-01-01..

place world: outside lives 2026-01-01..

input dec_dom: int

input fee_amt: money PLN

input wdiv[m in month]: num

move topup[m in month]: transfers
  on day_of(m, dec_dom)
  when 1 / wdiv[m] > 0
  leg cash from world to (if 1 / wdiv[m] > 0 then box else konto) amount fee_amt

ask q_bal: bal(konto, d), bal(box, d) for d in [2026-01-20, 2026-02-19, 2026-02-20]

version facts
  cite fixture
  governs always
  known 2025-12-01
  dec_dom = 20 "the day of the month"
  fee_amt = 3.00 "a fee"
  wdiv[2026-01] = 1 "answered"
  wdiv[2026-02] = 0 "a division by zero: the when and the place are refused"
  wdiv[2026-03] = 2 "answered"
  opening[konto] = 100 at 2026-01-01 start
  opening[box] = 0 at 2026-01-01 start
"#;

    /// The same plan whose leg's place, read only for February's refused instance, meets a
    /// load refusal at run (a rate per year times a day).
    const PLACE_LOAD_REFUSAL: &str = r#"model place_load_refusal
  tally 2
  on 2026-02-15
  start 2026-01-01
  end 2026-03-31

source fixture "invented values"

unit PLN: currency minor 0.01

index month = month 2026-01..2026-03


place konto: accounts PLN lives 2026-01-01..

place box: accounts PLN lives 2026-01-01..

place world: outside lives 2026-01-01..

input dec_dom: int

input fee_amt: money PLN

input r5: rate

input wdiv[m in month]: num

move topup[m in month]: transfers
  on day_of(m, dec_dom)
  when 1 / wdiv[m] > 0
  leg cash from world to (if wdiv[m] == 0 then (if r5 * 1 day > 0 then konto else box) else box) amount fee_amt

ask q_bal: bal(konto, d), bal(box, d) for d in [2026-01-20, 2026-02-19, 2026-02-20]

version facts
  cite fixture
  governs always
  known 2025-12-01
  dec_dom = 20 "the day of the month"
  fee_amt = 3.00 "a fee"
  r5 = 5 %/year "five per cent a year"
  wdiv[2026-01] = 1 "answered"
  wdiv[2026-02] = 0 "a division by zero: the when and the place are refused"
  wdiv[2026-03] = 2 "answered"
  opening[konto] = 100 at 2026-01-01 start
  opening[box] = 0 at 2026-01-01 start
"#;

    /// A base whose scenario declares a plan over the days, kept by a `when` load can't see,
    /// that exists on 31 December, before the scenario's `from`.
    const EARLY_BASE: &str = r#"model hh
  tally 2
  on 2026-01-02
  start 2025-12-30
  end 2026-01-05

source s "T's fixture card (invented)"

unit PLN: currency minor 0.01

place konto: accounts PLN lives 2025-01-01..

place world: outside lives 1900-01-01..

input from_day: date

input top_amt: money PLN

input lift_amt: money PLN

ask q: bal(konto, 2026-01-05), top_amt, lift_amt

version v
  cite s
  governs always
  known 2025-12-01
  from_day = 2025-12-31
  top_amt = 1.00 PLN
  lift_amt = 2.00 PLN
  opening[konto] = 10.00 PLN at 2025-12-30 start
"#;

    const EARLY_SCENARIO: &str = r"scenario topup over hh
  tally 2
  from 2026-01-01

move top[d in days]: transfers
  on d
  when d >= from_day
  leg cash from world to konto amount top_amt

ask q_top: bal(konto, 2026-01-05) in topup
";

    /// The printed lines of ask `ask` over `files`, under the event walk.
    fn answer(files: &[(&str, &str)], ask: &str) -> Vec<String> {
        let given = files
            .iter()
            .map(|(path, text)| Given {
                path: (*path).to_owned(),
                bytes: Some(text.as_bytes().to_vec()),
            })
            .collect();
        let request = Request::new(Asks::Named(vec![ask.to_owned()]));
        run(Files::new(given), &request)
            .expect("an answer")
            .lines()
            .to_vec()
    }

    #[test]
    fn a_refused_plan_whose_legs_place_is_refused_refuses_every_place_from_its_day() {
        // L §8.4 (tally 2.1.6): topup[2026-02]'s when is refused, and its leg's place can't be
        // read there either, so it refuses every place from its day, 20 February; January's
        // instance books, and nothing is refused before
        assert_eq!(
            answer(&[("place.tally", REFUSED_PLACE)], "q_bal"),
            [
                "q_bal (d = 2026-01-20): (100.00 PLN, 3.00 PLN)",
                "  unfulfilled: topup[2026-01]",
                "  cut 2026-02-15",
                "q_bal (d = 2026-02-19): (100.00 PLN, 3.00 PLN)",
                "  unfulfilled: topup[2026-01]",
                "  cut 2026-02-15",
                "q_bal (d = 2026-02-20): (refused: RUN-DIV division by zero, refused: RUN-DIV division by zero)",
                "  unfulfilled: topup[2026-01]",
                "  cut 2026-02-15"
            ],
            "every place refused from 20 February"
        );
    }

    #[test]
    fn a_unit_fault_met_reading_a_refused_plans_place_is_refused_at_that_value() {
        // L §3.3, §8.4 (tally 2.1.7 R5): a unit check met at run, here reading the place of
        // February's refused instance, is refused at that value, `RUN-*`, not the run: as
        // its twin above, every place refused from 20 February (refcore's bytes)
        assert_eq!(
            answer(&[("loadrun.tally", PLACE_LOAD_REFUSAL)], "q_bal"),
            answer(&[("place.tally", REFUSED_PLACE)], "q_bal"),
            "every place refused from 20 February"
        );
    }

    #[test]
    fn a_scenarios_plan_before_its_from_is_left_out_at_run() {
        // L §6.4: top[2025-12-31] exists (from_day is 31 December), but scenario topup's from
        // is 1 January, so it books nothing there: 10.00 + 5 × 1.00, and the note
        assert_eq!(
            answer(
                &[("model.tally", EARLY_BASE), ("a.tally", EARLY_SCENARIO)],
                "q_top"
            ),
            [
                "q_top: 15.00 PLN",
                "  world: topup",
                "  unfulfilled: top[2026-01-01]",
                "  left out: top, 1 instances, first 2025-12-31",
                "  cut 2026-01-02"
            ],
            "the instance before the from left out"
        );
    }

    /// Three moves: `pay_w` on a window of three days, its twin `pay_x` on a day that is a
    /// hole, and `pay_one` on a window of one day, each into its own place.
    const HOLE_DAY: &str = r#"model hole_day
  tally 2
  on 2026-04-20
  start 2026-01-01
  end 2026-12-31

source fixture "invented values"

unit PLN: currency minor 0.01

place konto: accounts PLN lives 2026-01-01..

place other: accounts PLN lives 2026-01-01..

place sav: accounts PLN lives 2026-01-01..

place world: outside lives 1900-01-01..

input pay: money PLN

input w: date

input pd: date

input one: date

move pay_w: transfers
  on w
  leg cash from world to konto amount pay

move pay_x: transfers
  on pd
  leg cash from world to other amount pay

move pay_one: transfers
  on one
  leg cash from world to sav amount pay

version v
  cite fixture
  governs always
  known 2026-01-01
  pay = 100.00 PLN "a pay"
  w = window(2026-04-01, 2026-04-03) "a day of April"
  pd = ?"which day is the pay?" "not known"
  one = window(2026-03-10, 2026-03-10) "one day"
  opening[konto] = 500.00 PLN at 2026-01-01 start "own"
  opening[other] = 500.00 PLN at 2026-01-01 start "own"
  opening[sav] = 500.00 PLN at 2026-01-01 start "own"

ask k: bal(konto, 2026-01-15), bal(konto, 2026-12-31)

ask o: bal(other, 2026-01-15), bal(other, 2026-12-31)

ask k_pin: bal(konto, 2026-12-31) with w = 2026-04-02

ask k_day: pay_w.day, pay_w.cash.amount

ask s: bal(sav, 2026-03-09), bal(sav, 2026-03-10)
"#;

    #[test]
    fn a_window_on_a_moves_day_makes_it_a_hole_day() {
        // L §4.4, §5.14 (tally 2.3 S6): pay_w's day is the hole `HOLE-IMPRECISE` for the day
        // of pay_w, naming w, so konto is that hole on every day, before the window too, as
        // other is pay_x's; its amount stays a value; a pin of w books it on that day
        let imprecise = "?\"date within window(2026-04-01, 2026-04-03) not precise enough for the day of pay_w\" (w)";
        assert_eq!(
            answer(&[("hole_day.tally", HOLE_DAY)], "k"),
            [
                format!("k: ({imprecise}, {imprecise})"),
                "  unknown day: w".to_owned(),
                "  cut 2026-04-20".to_owned()
            ],
            "konto is the hole day's on every day"
        );
        assert_eq!(
            answer(&[("hole_day.tally", HOLE_DAY)], "o"),
            [
                "o: (?\"which day is the pay?\" (pd), ?\"which day is the pay?\" (pd))",
                "  cut 2026-04-20"
            ],
            "as other is the twin's hole day's"
        );
        assert_eq!(
            answer(&[("hole_day.tally", HOLE_DAY)], "k_day"),
            [
                format!("k_day: ({imprecise}, 100.00 PLN)"),
                "  unknown day: w".to_owned(),
                "  cut 2026-04-20".to_owned()
            ],
            "the day is the hole, the amount a value"
        );
        assert_eq!(
            answer(&[("hole_day.tally", HOLE_DAY)], "k_pin"),
            [
                "k_pin: 600.00 PLN",
                "  pinned: w = 2026-04-02",
                "  unfulfilled: pay_w",
                "  cut 2026-04-20"
            ],
            "a pin of w books it"
        );
        // a window of one day is that day: pay_one books on 10 March
        assert_eq!(
            answer(&[("hole_day.tally", HOLE_DAY)], "s"),
            [
                "s: (500.00 PLN, 600.00 PLN)",
                "  unfulfilled: pay_one",
                "  cut 2026-04-20"
            ],
            "booked on its one day"
        );
    }

    /// A move whose `at` is a window of February.
    const HOLE_LAW: &str = r#"model hole_law
  tally 2
  on 2026-04-20
  start 2026-01-01
  end 2026-12-31

source fixture "invented values"

unit PLN: currency minor 0.01

place konto: accounts PLN lives 2026-01-01..

place world: outside lives 1900-01-01..

input pay: money PLN

input wl: date

input d: date

move pay_at: transfers
  on d
  at wl
  leg cash from world to konto amount pay

version v
  cite fixture
  governs always
  known 2026-01-01
  pay = 100.00 PLN "a pay"
  wl = window(2026-02-01, 2026-02-28) "a law date of February"
  d = 2026-03-10 "its day"
  opening[konto] = 500.00 PLN at 2026-01-01 start "own"

ask k: bal(konto, 2026-01-15), pay_at.day

ask k_pin: bal(konto, 2026-12-31) with wl = 2026-02-10

ask day_pin: pay_at.day with wl = 2026-02-10
"#;

    #[test]
    fn a_days_read_brings_its_law_dates_pin() {
        // what reading an instance's law date met is what a read of its day brings, as
        // refcore's `day_coll` (the review of G1, N1): a plain law date, and a window's
        let plain = HOLE_LAW.replace("window(2026-02-01, 2026-02-28)", "2026-02-15");
        for (what, text) in [("a plain law date", plain.as_str()), ("a window", HOLE_LAW)] {
            assert_eq!(
                answer(&[("law.tally", text)], "day_pin"),
                [
                    "day_pin: 2026-03-10",
                    "  pinned: wl = 2026-02-10",
                    "  cut 2026-04-20"
                ],
                "{what}"
            );
        }
    }

    #[test]
    fn a_window_at_makes_a_hole_law_date() {
        // L §4.4, §5.14 `at e` (tally 2.3 S6): pay_at's law date is the hole for the law date
        // of pay_at, so the instance and its day are that hole, as for an `at` that is a hole;
        // a pin of wl reads one law date
        let imprecise = "?\"date within window(2026-02-01, 2026-02-28) not precise enough for the law date of pay_at\" (wl)";
        assert_eq!(
            answer(&[("hole_law.tally", HOLE_LAW)], "k"),
            [
                format!("k: ({imprecise}, {imprecise})"),
                "  unknown day: wl".to_owned(),
                "  cut 2026-04-20".to_owned()
            ],
            "the balance and the day are the law date's hole"
        );
        assert_eq!(
            answer(&[("hole_law.tally", HOLE_LAW)], "k_pin"),
            [
                "k_pin: 600.00 PLN",
                "  pinned: wl = 2026-02-10",
                "  unfulfilled: pay_at",
                "  cut 2026-04-20"
            ],
            "a pin of wl books it"
        );
    }

    /// Monthly rent whose January `when` is an assumed row OK, a January bank row fulfilling
    /// it: the planner's `first.tally`.
    const FIRST: &str = r#"model first
  tally 2
  on 2026-05-20
  start 2026-01-01
  end 2026-06-30

source s "invented values"

unit PLN: currency minor 0.01

index month = month 2026-01..2026-06

place pot: accounts PLN lives 2026-01-01..

place landlord: outside lives 1900-01-01..

input bank: observed

input rent_amt: money PLN

input ok[m in month]: bool

input d5: int

move rent[m in month]: transfers
  on day_of(m, d5)
  at start_of(m)
  when ok[m]
  leg cash from pot to landlord amount rent_amt

ask a: flows(rent, 2026-01)

ask b: flows(rent, 2026-03)

ask c: bal(pot, 2026-03-31)

version v
  cite s
  governs always
  known 2026-01-01
  rent_amt = 100.00 PLN "s"
  d5 = 5 "s"
  ok[2026-01] = OK
  ok[2026-02] = true "ok feb"
  ok[2026-03] = true "ok mar"
  ok[2026-04] = true "ok apr"
  ok[2026-05] = true "ok may"
  ok[2026-06] = true "ok jun"
  opening[pot] = 9000.00 PLN at 2026-01-01 start "s"

version jan
  cite s
  governs 2026-01
  known 2026-01-21
  covers pot
  bank[a1] = (on: 2026-01-05, kind: transfers, leg cash from pot to landlord amount 100.00 PLN, fulfils: rent[2026-01]) "names jan"
"#;

    #[test]
    fn a_binding_over_the_kind_carries_no_link_read_of_an_instance_outside_its_span() {
        // L §8.2 l.2252 (tally 2.3.1 R2, item 6; G2's review B4): January's modelled instance
        // and its row lie outside March's span, so March's sum over the kind lists none of
        // January's link reads, as March's flows lists none
        let text = FIRST
            .replace("OK", "true assumed \"jan is a guess\" \"ok jan\"")
            .replace(
                "ask c:",
                "ask sm: sum(t.cash.amount for t in transfers over 2026-03)\n\nask c:",
            );
        let files = [("first.tally", text.as_str())];
        assert_eq!(
            answer(&files, "sm"),
            [
                "sm: 100.00 PLN",
                "  unfulfilled: rent[2026-03]",
                "  cut 2026-05-20"
            ],
            "March's sum doesn't rest on January's link"
        );
    }
}
