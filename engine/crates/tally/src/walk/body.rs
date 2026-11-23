//! An instance's body: its existence, legs, attributes, period and what it fulfils, read at
//! its law date; a decision's runs read again for a traced read that counts them.
//! Spec: L §4.3, §5.14 (moves: `on`, `at e`, `when`, `for`, legs, attributes), §5.14.1,
//! §5.14.3 (`fulfils`), §8.1; E §5 step 1 (the body at its booking cut); E §10 (`explain`
//! re-evaluates the cone with tracing on; the traced value equals the stored one), L §5.18
//! Explanation order (the rows that made the value).
//! Twin: engine/tools/refcore/moves.py (`eval_inst`, `inst_body`); walk.py (`run_one_body`,
//! run while an `explain` traces).
//! Invariants: a body's move holds the projections it meets; a run read again is read at
//! its point as it ran, with a register and a read set of its own, kept nowhere, so tracing
//! changes no value and no line's register.

use super::instance::{Dated, Exists, Inst, LegVal, join};
use super::order::{Run as Ran, Tested, WalkAt};
use super::plan::{invariant, refuse, refuse_load, shown};
use super::point::At;
use super::tuples::Law;
use crate::base::{Code, Load, Run};
use crate::eval::{Collected, Ctx, Engine, Walked};
use crate::hole::Hole;
use crate::model::{MoveInfo, Node, NodeKind, Reality, Span, Value};
use crate::syntax::Stop;
use crate::time::Day;
use std::collections::BTreeMap;

/// What an instance's body gives: whether it exists, its legs, attributes and period.
pub(super) struct Body {
    /// Whether it exists.
    pub(super) exists: Exists,
    /// Its legs.
    pub(super) legs: Vec<LegVal>,
    /// Its attributes.
    pub(super) attrs: Vec<(String, Value)>,
    /// The period it is for.
    pub(super) for_period: Option<Value>,
    /// What its `fulfils` names.
    pub(super) fulfils: Option<Value>,
    /// What reading each leg met, by the leg's name: a read of one leg of a movement stated
    /// `same` holds the other side's reads of that leg only (S3, P43 on #241).
    pub(super) leg_reads: BTreeMap<String, Collected>,
}

impl Engine {
    /// An instance's law date by its move's `at`, kept on it, and its day, its `on` read at
    /// that date; a law date that is a hole is its day's hole (refcore's `eval_inst`). L §4.3,
    /// §5.14 `at e`.
    ///
    /// # Errors
    ///
    /// The refusals of the `at` and the `on`.
    pub(super) fn law_and_day(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &mut Inst,
        tctx: Ctx,
    ) -> Result<Result<Value, Hole>, Stop> {
        inst.law = self.move_law(m, mv, &inst.env, tctx)?;
        if let Law::Hole(h) = &inst.law {
            return Ok(Err(h.clone()));
        }
        let day = self.ev(m, &mv.decl.on, &inst.env, tctx.at_law(inst.law.day()))?;
        Ok(Ok(day))
    }

    /// An instance's body in context `ctx`, its law date and cut set: its `when`, period,
    /// legs and attributes, with its move the holder of the projections met (refcore's
    /// `body_over_law`, `inst_body`). L §5.14, §5.14.1, E §5 step 1.
    ///
    /// # Errors
    ///
    /// `RUN-NONE` for a `when` that is none; the refusals of its parts.
    pub(super) fn inst_body(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &Inst,
        ctx: Ctx,
        exists: Exists,
    ) -> Result<Body, Stop> {
        let outer = self.holding(Node::new(NodeKind::Move, &mv.id));
        let body = self.move_body(m, mv, inst, ctx, exists);
        self.holder = outer;
        body
    }

    /// The main walk of cut `cut`, read by a read that counts the runs of the decision moves
    /// `decided` in it; where the read is traced, what it prints of them (`trace_decided`). A
    /// walk that stops is no cause either: its stop is the read's, an axis the timeline sets
    /// no option of included, which the trace outside an ask's pins fans out over (E §3.3,
    /// L §5.5). E §10, §4.2.
    ///
    /// # Errors
    ///
    /// As [`Engine::main_walk`] and [`Engine::trace_decided`].
    pub(crate) fn decided_walk(
        &mut self,
        m: &Reality,
        cut: Day,
        decided: &[&MoveInfo],
    ) -> Result<usize, Stop> {
        let before = self.trace_walked();
        let id = match self.main_walk(m, cut) {
            Ok(id) => id,
            Err(stop) => {
                self.trace_walked_back(before);
                return Err(stop);
            }
        };
        self.trace_decided(m, (before, id), decided)?;
        Ok(id)
    }

    /// What a traced read that counts the runs of the decision moves `decided` in main walk
    /// `id` prints of them once the walk is read: each move's instances again, with their
    /// days, once per explain for each move, as the plan pass lists them once; each run it
    /// counts prints where it is counted ([`Engine::trace_counted`]). The walk's own reads,
    /// the other moves' days and amounts, are none of the read's (L §5.18: the rows that made
    /// the value; L §8.2 One read set; tally 2.1.8 P50), so the note of a walk read goes back
    /// to `before`, as it stood before the read. E §10.
    ///
    /// # Errors
    ///
    /// As [`Engine::make_instances`].
    fn trace_decided(
        &mut self,
        m: &Reality,
        (before, id): (Walked, usize),
        decided: &[&MoveInfo],
    ) -> Result<(), Stop> {
        if !self.tracing() {
            return Ok(());
        }
        self.trace_walked_back(before);
        let cut = self.walk(id)?.cut;
        for mv in decided {
            if self.trace_first_days(id, &mv.id) {
                self.trace_days(m, mv, cut)?;
            }
        }
        Ok(())
    }

    /// What a read over span `sp` that counts the movements at its key (a `flows`, a binding
    /// over a kind) takes of due instance `i` there that it doesn't count, the one read set
    /// (L §8.2 One read set; E §3.2; P20): in the span, read to be left out (its `when`
    /// false), every read that decided so, a run's printed where traced
    /// ([`Engine::trace_counted`]); outside it, the day that put it there, never its `when`
    /// or amount.
    ///
    /// # Errors
    ///
    /// As [`Engine::trace_counted`].
    pub(crate) fn uncounted_reads(
        &mut self,
        m: &Reality,
        i: &Inst,
        sp: &Span,
        ctx: Ctx,
    ) -> Result<Collected, Stop> {
        if !matches!(&i.day, Some(Dated::Day(d)) if sp.contains(*d)) {
            return Ok(i.day_coll.outside());
        }
        self.trace_counted(m, i, ctx)?;
        Ok(i.coll.clone())
    }

    /// Instance `inst` of a decision, counted by a traced read at the ask's cut (a `flows`,
    /// a binding over its kind): its run's body read again at its point, once per explain,
    /// where it is first counted, so the explanation lists the one read set the value has,
    /// the runs at the read's keys and in its span alone (L §8.2 One read set, §5.18; E §10);
    /// a run whose order the walk refused, what the refusal tested ([`Tested`]): each member's
    /// `when` at the act's point, or each tied run as it ran (E §4.2 step 3; P20). None for a
    /// plan, inside a decision, or for a run whose body wasn't read at a point.
    ///
    /// # Errors
    ///
    /// As [`Engine::trace_run`].
    pub(crate) fn trace_counted(&mut self, m: &Reality, inst: &Inst, ctx: Ctx) -> Result<(), Stop> {
        if !m.moves.get(&inst.move_id).is_some_and(|mv| mv.decision) {
            return Ok(());
        }
        if ctx.at.is_some() || !self.tracing() {
            return Ok(());
        }
        let Some(id) = self.main_walks.get(&ctx.cut).copied() else {
            return Ok(());
        };
        let Some(run) = self.chain_run(id, &inst.id)?.cloned() else {
            return Ok(());
        };
        self.trace_ran(m, id, &run)?;
        match &run.tested {
            Tested::Nothing => Ok(()),
            Tested::Runs(tied) => {
                for i in tied {
                    if let Some(r) = self.chain_run(id, i)?.cloned() {
                        self.trace_ran(m, id, &r)?;
                    }
                }
                Ok(())
            }
            Tested::Whens(act, members) => {
                for (q, i) in members {
                    self.trace_when_at(m, (id, *act), (q, i))?;
                }
                Ok(())
            }
        }
    }

    /// Run `run` of walk `id` read again where it ran, once per explain (E §10).
    ///
    /// # Errors
    ///
    /// As [`Engine::trace_run`].
    fn trace_ran(&mut self, m: &Reality, id: usize, run: &Ran) -> Result<(), Stop> {
        let Some(mv) = m.moves.get(&run.inst.move_id) else {
            return Err(invariant("a run's move load didn't build"));
        };
        if !run.ran || !self.trace_first_run(id, &run.inst.id) {
            return Ok(());
        }
        self.trace_run(m, (id, mv), run)
    }

    /// The `when` of instance `inst_id` of decision `move_id`, read again at the act's point `act` of
    /// walk `id`, before any of its cycle runs, traced, kept nowhere, once per explain: what
    /// a refusal of the cycle's order tested of it (E §4.2 step 3, §10; L §5.14.2; P20).
    ///
    /// # Errors
    ///
    /// A load refusal met at run; `ENGINE-INVARIANT` for a move the load didn't build.
    fn trace_when_at(
        &mut self,
        m: &Reality,
        (id, act): (usize, At),
        (move_id, inst_id): (&str, &str),
    ) -> Result<(), Stop> {
        let Some(mv) = m.moves.get(move_id) else {
            return Err(invariant("a cycle's move load didn't build"));
        };
        let Some(when) = &mv.decl.when else {
            return Ok(());
        };
        // its own key, apart from its run's, as the cycle read it before any ran
        if !self.trace_first_run(id, &format!("{inst_id} before {}", act.day.text())) {
            return Ok(());
        }
        let cut = self.walk(id)?.cut;
        let insts = self.move_instances(m, mv, cut)?;
        let Some(inst) = insts.iter().find(|x| x.id == inst_id) else {
            return Ok(());
        };
        let Some(Dated::Day(t)) = inst.day else {
            return Ok(());
        };
        let there = Ctx {
            at: Some(WalkAt {
                walk: id,
                point: act,
            }),
            ..self.timeline_ctx(inst.law.day(), t.min(cut))
        };
        let outer = core::mem::replace(&mut self.rereading, true);
        let holder = self.holding(Node::new(NodeKind::Move, &mv.id));
        self.push();
        self.trace_plan_open();
        let read = self.ev(m, when, &inst.env, there);
        let closed = self.trace_plan_close();
        self.pop_alone();
        self.holder = holder;
        self.rereading = outer;
        closed?;
        match read {
            Err(Stop::Refused(r)) if matches!(r.code(), Code::Load(_)) => Err(Stop::Refused(r)),
            Ok(_) | Err(Stop::Refused(_)) => Ok(()),
            Err(stop) => Err(stop),
        }
    }

    /// Run `run` of decision `mv` in walk `id`, its body read again at its point as it ran
    /// (its law date, the state just before its own booking, its own earlier observations
    /// left out), traced, kept nowhere: its `when`, legs and attributes print their reads,
    /// its running reads as balances up to that point (E §10: "the traced value equals the
    /// stored one"). E §10, §4.2 step 3; L §5.14.1.
    ///
    /// # Errors
    ///
    /// A load refusal met at run; `ENGINE-INVARIANT` for a run with no day.
    fn trace_run(
        &mut self,
        m: &Reality,
        (id, mv): (usize, &MoveInfo),
        run: &Ran,
    ) -> Result<(), Stop> {
        let Some(Dated::Day(t)) = run.inst.day else {
            return Err(invariant("a decision run with no day"));
        };
        let cut = self.walk(id)?.cut;
        let insts = self.move_instances(m, mv, cut)?;
        let Some(inst) = insts.iter().find(|i| i.id == run.inst.id) else {
            return Ok(());
        };
        let there = Ctx {
            at: Some(WalkAt {
                walk: id,
                point: run.point.just_before(),
            }),
            ..self.timeline_ctx(None, t.min(cut))
        };
        let prev = self.enter_run(m, id, run)?;
        let outer = core::mem::replace(&mut self.rereading, true);
        self.push();
        self.trace_plan_open();
        let law = inst.law.day();
        let body = self.inst_body(m, mv, inst, there.at_law(law), inst.exists.clone());
        let closed = self.trace_plan_close();
        self.pop_alone();
        self.rereading = outer;
        self.leave_run(id, prev)?;
        closed?;
        match body {
            Err(Stop::Refused(r)) if matches!(r.code(), Code::Load(_)) => Err(Stop::Refused(r)),
            Ok(_) | Err(Stop::Refused(_)) => Ok(()),
            Err(stop) => Err(stop),
        }
    }

    /// An instance's body: its `when`, period, legs and attributes (refcore's
    /// `body_over_law`, `inst_body`). L §5.14, E §5 step 1.
    fn move_body(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &Inst,
        ctx: Ctx,
        exists: Exists,
    ) -> Result<Body, Stop> {
        let mut ex = exists;
        if mv.decl.fulfils.is_some() {
            crate::book::settlement_move(m, mv)?;
        }
        if let Some(when) = &mv.decl.when {
            let w = self.ev(m, when, &inst.env, ctx)?;
            if w == Value::None {
                return Err(refuse(
                    Run::None,
                    format!("the when of {} is none here", inst.id),
                ));
            }
            if w == Value::Bool(false) {
                let Some(Dated::Hole(h)) = &inst.day else {
                    return Ok(Body {
                        exists: Exists::No,
                        legs: Vec::new(),
                        attrs: Vec::new(),
                        for_period: None,
                        fulfils: None,
                        leg_reads: BTreeMap::new(),
                    });
                };
                ex = join(ex, h);
            } else if let Value::Hole(h) = &w {
                ex = join(ex, h);
            } else if w != Value::Bool(true) {
                return Err(refuse_load(
                    Load::Unit,
                    format!("the when of {} is {}", inst.id, shown(m, &w)?),
                ));
            }
        }
        let for_period = match &mv.decl.for_period {
            Some(e) => Some(self.ev(m, e, &inst.env, ctx)?),
            None => None,
        };
        let kind_legs = m.kind_legs(&mv.kind);
        let mut legs = Vec::with_capacity(mv.decl.legs.len());
        let mut leg_reads = BTreeMap::new();
        for leg in &mv.decl.legs {
            let unit = kind_legs
                .iter()
                .find(|(name, _, _)| *name == leg.name)
                .map(|(_, unit, _)| unit.clone())
                .ok_or(Stop::NotBuilt(
                    "a leg its kind doesn't declare (M0's check)",
                ))?;
            self.push();
            let read = self.eval_leg(m, inst, leg, &unit, ctx);
            let reads = self.pop_alone();
            self.absorb(&reads);
            legs.push(read?);
            leg_reads.insert(leg.name.clone(), reads);
        }
        let mut attrs = Vec::with_capacity(mv.decl.attrs.len());
        for (name, e) in &mv.decl.attrs {
            attrs.push((name.clone(), self.ev(m, e, &inst.env, ctx)?));
        }
        let fulfils = match &mv.decl.fulfils {
            Some(e) => Some(self.ev(m, e, &inst.env, ctx)?),
            None => None,
        };
        Ok(Body {
            exists: ex,
            legs,
            attrs,
            for_period,
            fulfils,
            leg_reads,
        })
    }
}
