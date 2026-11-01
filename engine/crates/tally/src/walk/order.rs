//! A day's decisions: each move's instances due that day in their order, each run at its point against the balances as they stand, its legs booked and tested together.
//! Spec: E §4.2 step 3 (decisions by stratum, each at its point, computed at min(t, K) with
//! running reads; the legs it books from its point are one group of the sign rule), step 5;
//! L §5.14.1 (running reads), §5.14.2 (the order of a day's decisions: `ties by`, `RUN-TIE`),
//! §5.10 (the sign rule's point 2); E §4.2 step 3 and L §5.19 (a decision on a covered day
//! before K, or one an observation fulfils, is modelled past: its own observations booked at
//! its point, its legs not; it computes against the balances less its own observations
//! booked before its day).
//! Twin: engine/tools/refcore/walk.py (`run_decisions`, `tie_order`, `run_one`,
//! `run_one_body`, `book_own_obs`).
//! Invariants: a decision reads the walk as it stands at its point, never a later point; a
//! run is recorded once, after its body is computed, so a run cut short past the counted depth
//! is run again from its start; the instances of one move run in the order of its `ties by`.

use core::cmp::Ordering;
use std::collections::BTreeSet;

use super::agenda::{Act, Mv};
use super::day::{Broke, ends};
use super::plan::{Amount, Dated, Exists, Inst, LegVal, Status, invariant, kind_certainty};
use super::point::{At, Step};
use super::state::{ALL, Change, Ent, Group, Taken, WalkObs};
use crate::base::{Code, Refusal, Run as RunCode};
use crate::eval::{Ctx, Engine, Entry};
use crate::hole::spread;
use crate::model::{MoveInfo, Reality, Value};
use crate::syntax::{Certainty, Stop};
use crate::time::Day;

/// A decision instance as the walk ran it (refcore's run, a copy of the instance).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    /// The instance, its body as it ran: its legs, attributes, status.
    pub inst: Inst,
    /// Its point.
    pub point: At,
    /// The places it read as they stand at its point (L §5.14.2).
    pub live: BTreeSet<String>,
    /// The places its legs name.
    pub changed: BTreeSet<String>,
}

/// Where a running read stands: the walk and the point (refcore's `ctx.walk`, `ctx.wp`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WalkAt {
    /// The walk, by its place in the engine's list.
    pub walk: usize,
    /// The point.
    pub point: At,
}

impl Engine {
    /// The decisions of move `move_id` due on `day`: in the order of its `ties by`, each run
    /// at its point; instances that read each other's places with no `ties by` are `RUN-TIE`
    /// (refcore's `run_decisions`). E §4.2 step 3, L §5.14.2.
    pub(super) fn run_decisions(
        &mut self,
        m: &Reality,
        id: usize,
        move_id: &str,
        day: Day,
        at: At,
    ) -> Result<(), Stop> {
        let mv = m
            .moves
            .get(move_id)
            .ok_or_else(|| invariant("a decision act of a move load didn't build"))?;
        let mut insts = self
            .walk(id)?
            .due
            .get(&(move_id.to_owned(), day))
            .cloned()
            .ok_or_else(|| invariant("a decision act with no instance due"))?;
        let mut cycle = false;
        for group in m.graph.order_groups.iter().filter(|g| g.contains(move_id)) {
            for q in group {
                cycle = cycle || (q != move_id && self.is_due(id, q, day));
            }
        }
        if cycle {
            return Err(Stop::NotBuilt(
                "an unordered cycle of decisions due on one day",
            ));
        }
        if insts.len() > 1 && mv.decl.ties.is_some() {
            insts = self.tie_order(m, id, mv, insts, day)?;
        }
        let start = self.today(id)?.next_inst;
        for (n, inst) in insts.iter().enumerate().skip(start) {
            let n32 = u32::try_from(n).map_err(|_| invariant("an instance's rank"))?;
            let key = At { n: n32, ..at };
            let ix = self.run_one(m, id, inst.clone(), key)?;
            let today = self.today_mut(id)?;
            today.runs.push(ix);
            today.next_inst = n
                .checked_add(1)
                .ok_or_else(|| invariant("a count of runs"))?;
        }
        if insts.len() > 1 && mv.decl.ties.is_none() && mv.self_reads {
            self.self_tie(m, id, mv, at)?;
        }
        Ok(())
    }

    /// Whether decisions of move `q` are due on `day` in walk `id`. L §5.14.2.
    fn is_due(&self, id: usize, q: &str, day: Day) -> bool {
        self.walks
            .get(id)
            .is_some_and(|w| w.due.contains_key(&(q.to_owned(), day)))
    }

    /// Instances of one move that read the places another of them changes, with no `ties by`:
    /// their order changes the result, `RUN-TIE` at every place they change (refcore's
    /// `run_decisions`). L §5.14.2.
    fn self_tie(&mut self, m: &Reality, id: usize, mv: &MoveInfo, at: At) -> Result<(), Stop> {
        let walk = self.walk(id)?;
        let runs: Vec<&Run> = self
            .today(id)?
            .runs
            .iter()
            .filter_map(|ix| walk.runs.get(*ix))
            .collect();
        let mut tie = false;
        for (a, ra) in runs.iter().enumerate() {
            for rb in runs
                .iter()
                .skip(a.checked_add(1).ok_or_else(|| invariant("a run's rank"))?)
            {
                let meets = !rb.live.is_disjoint(&ra.changed) || !ra.live.is_disjoint(&rb.changed);
                tie = tie || meets;
            }
        }
        if !tie {
            return Ok(());
        }
        let ids: Vec<&str> = runs.iter().map(|r| r.inst.id.as_str()).collect();
        let err = Refusal::new(
            Code::Run(RunCode::Tie),
            format!(
                "{} has a tie at {} that changes the result, and no ties by",
                mv.id,
                ids.join(", ")
            ),
        );
        let places: Vec<String> = runs
            .iter()
            .flat_map(|r| r.changed.iter().cloned())
            .collect();
        let key = At {
            n: u32::MAX,
            sub: 9,
            ..at
        };
        for place in places {
            let unit = m
                .things
                .get(&place)
                .and_then(|t| m.place_unit(t))
                .unwrap_or_else(|| ALL.to_owned());
            self.walk_mut(id)?
                .add(m, &place, &unit, Ent::new(key, Change::Bad(err.clone())))?;
        }
        Ok(())
    }

    /// One move's instances due on a day, in the order of its `ties by`: a hole there is
    /// `RUN-TIE`, and so are two instances it doesn't tell apart (refcore's `tie_order`).
    /// L §5.14.2.
    fn tie_order(
        &mut self,
        m: &Reality,
        id: usize,
        mv: &MoveInfo,
        insts: Vec<Inst>,
        day: Day,
    ) -> Result<Vec<Inst>, Stop> {
        let Some(ties) = &mv.decl.ties else {
            return Ok(insts);
        };
        let cut = self.walk(id)?.cut.min(day);
        let mut scored: Vec<(Vec<Value>, Inst)> = Vec::with_capacity(insts.len());
        for inst in insts {
            let ctx = Ctx::new(inst.law, cut);
            let mut vals = Vec::with_capacity(ties.len());
            for x in ties {
                let v = self.ev(m, x, &inst.env, ctx)?;
                if let Value::Hole(_) = v {
                    return Err(refuse(
                        RunCode::Tie,
                        format!("{}: its ties by is a hole", mv.id),
                    ));
                }
                vals.push(v);
            }
            scored.push((vals, inst));
        }
        let mut bad = None;
        scored.sort_by(|a, b| match tie_cmp(&a.0, &b.0) {
            Ok(o) => o,
            Err(stop) => {
                bad = Some(stop);
                Ordering::Equal
            }
        });
        if let Some(stop) = bad {
            return Err(stop);
        }
        for (a, b) in scored.iter().zip(scored.iter().skip(1)) {
            if tie_cmp(&a.0, &b.0)? == Ordering::Equal {
                return Err(refuse(
                    RunCode::Tie,
                    format!(
                        "{} has a tie at {} that changes the result, and its ties by doesn't break it",
                        mv.id,
                        day.text()
                    ),
                ));
            }
        }
        Ok(scored.into_iter().map(|(_, i)| i).collect())
    }

    /// Runs a decision instance at its point, then tests the sign rule after its legs taken
    /// together, a decision's legs at a place that breaks it refused (refcore's `run_one`).
    /// E §4.2 steps 3 and 5, L §5.10 point 2.
    fn run_one(&mut self, m: &Reality, id: usize, inst: Inst, key: At) -> Result<usize, Stop> {
        let prev = self.walk_mut(id)?.group.take();
        let ran = self.run_one_body(m, id, inst, key);
        let walk = self.walk_mut(id)?;
        let group = core::mem::replace(&mut walk.group, prev);
        let ix = ran?;
        if let Some(group) = group
            && !group.is_empty()
        {
            let run_id = self
                .walk(id)?
                .runs
                .get(ix)
                .map(|r| r.inst.id.clone())
                .ok_or_else(|| invariant("a run just made"))?;
            self.check_group(
                m,
                id,
                &group,
                key.with_sub(2),
                Taken::Act(key),
                &Broke::Decision(run_id),
            )?;
        }
        Ok(ix)
    }

    /// A decision instance's run: its body at its point with running reads, then, by what it
    /// gives, its legs booked, a hole at its places, or a refusal there (refcore's
    /// `run_one_body`). E §4.2 step 3, L §5.14.1.
    fn run_one_body(&mut self, m: &Reality, id: usize, inst: Inst, key: At) -> Result<usize, Stop> {
        let mv = m
            .moves
            .get(&inst.move_id)
            .ok_or_else(|| invariant("a decision of a move load didn't build"))?;
        let Some(Dated::Day(t)) = inst.day else {
            return Err(invariant("a decision run with no day"));
        };
        let cut = self.walk(id)?.cut;
        let ctx = Ctx {
            at: Some(WalkAt {
                walk: id,
                point: key,
            }),
            ..Ctx::new(inst.law, t.min(cut))
        };
        let own = self.own_earlier_obs(m, id, &inst, t)?;
        let prev_less = core::mem::replace(&mut self.walk_mut(id)?.less_obs, own);
        let prev_live = self.walk_mut(id)?.live.replace(BTreeSet::new());
        self.push();
        let body = self.inst_body(m, mv, &inst, ctx, inst.exists.clone());
        let met = self.pop_alone();
        self.walk_mut(id)?.less_obs = prev_less;
        let Some(live) = core::mem::replace(&mut self.walk_mut(id)?.live, prev_live) else {
            return Err(invariant("a run's live reads taken"));
        };
        let linked = self.obs_links_for(m, &inst.id, cut)?;
        let mut run = Run {
            inst,
            point: key,
            live,
            changed: BTreeSet::new(),
        };
        run.inst.coll.merge(&met);
        run.inst.obs = linked;
        match body {
            Ok(b) => {
                run.inst.exists = b.exists;
                run.inst.legs = b.legs;
                run.inst.attrs = b.attrs;
                run.inst.for_period = b.for_period;
                run.inst.fulfils = b.fulfils;
            }
            Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => run.inst.err = Some(r),
            Err(stop) => return Err(stop),
        }
        self.walk_mut(id)?.group = Some(Group::new());
        self.book_run(m, id, run, t)
    }

    /// What a run books by what its body gave (refcore's `run_one_body`, after the body).
    /// E §4.2 step 3, Holes in the walk.
    fn book_run(&mut self, m: &Reality, id: usize, mut run: Run, t: Day) -> Result<usize, Stop> {
        let key = run.point;
        let (cut, main) = {
            let walk = self.walk(id)?;
            (walk.cut, walk.proj.is_none())
        };
        let modelled = main
            && ((t < cut && self.touches_covered(m, &run.inst, cut)) || !run.inst.obs.is_empty());
        let walk = self.walk_mut(id)?;
        let ix = walk.runs.len();
        if let Some(err) = run.inst.err.clone() {
            run.inst.status = Status::Refused;
            let known: Vec<_> = run
                .inst
                .legs
                .iter()
                .filter(|lg| !ends(lg).is_empty())
                .cloned()
                .collect();
            let mv_id = run.inst.id.clone();
            walk.record(run);
            if known.is_empty() {
                walk.add(m, ALL, ALL, Ent::new(key, Change::Bad(err.clone())))?;
            }
            for lg in &known {
                walk.leg_bad(m, &mv_id, lg, &err, key)?;
            }
            return Ok(ix);
        }
        if run.inst.exists == Exists::No {
            run.inst.status = Status::None;
            let inst = run.inst.clone();
            walk.record(run);
            self.book_own_obs(m, id, &inst, key)?;
            return Ok(ix);
        }
        for lg in &run.inst.legs {
            for p in ends(lg) {
                run.changed.insert(p.to_owned());
            }
        }
        if modelled {
            run.inst.status = Status::Modelled;
            let inst = run.inst.clone();
            walk.modelled.push(inst.clone());
            walk.record(run);
            self.book_own_obs(m, id, &inst, key)?;
            return Ok(ix);
        }
        if let Exists::Unknown(h) = run.inst.exists.clone() {
            run.inst.status = Status::Hole;
            let legs = run.inst.legs.clone();
            let (mv_id, coll) = (run.inst.id.clone(), run.inst.coll.clone());
            walk.record(run);
            for lg in &legs {
                let amount = match &lg.amount {
                    Amount::Hole(a) => Some(a),
                    Amount::Qty(_) | Amount::Refused(_) => None,
                };
                let q = spread(&[Some(&h), amount]).ok_or_else(|| invariant("a hole run"))?;
                let unit = lg.unit.clone().unwrap_or_else(|| ALL.to_owned());
                for p in ends(lg) {
                    let mut ent = Ent::new(key, Change::Hole(q.clone()));
                    ent.mv = Some(mv_id.clone());
                    ent.coll = Some(coll.clone());
                    walk.add(m, p, &unit, ent)?;
                }
            }
            return Ok(ix);
        }
        let kind = m.moves.get(&run.inst.move_id).map(|mv| mv.kind.clone());
        if kind.is_some_and(|k| kind_certainty(m, &k) == Some(Certainty::Uncertain))
            && run.inst.obs.is_empty()
        {
            run.inst.status = Status::Uncertain;
            walk.uncertain.push(run.inst.clone());
            walk.record(run);
            return Ok(ix);
        }
        run.inst.status = if t < cut { Status::Past } else { Status::Plan };
        let mut reg = Vec::new();
        if t < cut && run.inst.obs.is_empty() {
            reg.push(Entry {
                word: "unfulfilled",
                text: run.inst.id.clone(),
                figure: String::new(),
            });
        }
        let legs = run.inst.legs.clone();
        let (mv_id, coll) = (run.inst.id.clone(), run.inst.coll.clone());
        let later_rank = walk.text_rank(&mv_id)?;
        let inst = run.inst.clone();
        walk.record(run);
        for (li, lg) in legs.iter().enumerate() {
            if let Dated::Day(ld) = lg.day
                && ld > t
            {
                let at = At {
                    rank: later_rank,
                    ..At::of(ld, Step::Plans)
                };
                self.walk_mut(id)?.at(at, Act::Leg(Mv::Run(ix), li));
                continue;
            }
            self.book_leg(m, id, (&mv_id, lg), key, &reg, &coll)?;
        }
        self.book_own_obs(m, id, &inst, key)?;
        Ok(ix)
    }

    /// The legs of a decision's own observations booked on their days before its day, the
    /// cut and after the walk's start: its computation reads the balances less them (refcore's
    /// `run_one_body`, `less_obs`). E §4.2 step 3, L §5.19.
    fn own_earlier_obs(
        &mut self,
        m: &Reality,
        id: usize,
        inst: &Inst,
        t: Day,
    ) -> Result<Option<Vec<LegVal>>, Stop> {
        let walk = self.walk(id)?;
        if walk.proj.is_some() {
            return Ok(None);
        }
        let (cut, start) = (walk.cut, walk.start);
        let mut out = Vec::new();
        for o in self.standing_in(m, inst, cut)? {
            if self.obs_err(m, &o, cut)?.is_some() {
                continue;
            }
            out.extend(
                o.legs
                    .into_iter()
                    .filter(|lg| matches!(lg.day, Dated::Day(d) if d < t && d < cut && d >= start)),
            );
        }
        Ok((!out.is_empty()).then_some(out))
    }

    /// A decision's own observations dated its day before the cut, booked just after its
    /// point, in its group of the sign rule (refcore's `book_own_obs`). E §4.2 steps 1 and 3,
    /// L §5.19.
    fn book_own_obs(&mut self, m: &Reality, id: usize, inst: &Inst, key: At) -> Result<(), Stop> {
        let walk = self.walk(id)?;
        let Some(Dated::Day(t)) = inst.day else {
            return Ok(());
        };
        if walk.proj.is_some() || t >= walk.cut {
            return Ok(());
        }
        let own: Vec<WalkObs> = walk
            .obs
            .iter()
            .filter(|wo| inst.obs.contains(&wo.obs.id))
            .cloned()
            .collect();
        let sub = key
            .sub
            .checked_add(1)
            .ok_or_else(|| invariant("a point past the counted subpoints"))?;
        for wo in &own {
            for lg in wo.obs.legs.iter().filter(|lg| lg.day.day() == Some(t)) {
                self.book_obs_now(m, id, wo, lg, key.with_sub(sub))?;
            }
        }
        Ok(())
    }
}

impl super::state::Walk {
    /// Records a run: its place in the list, by its instance's id. E §4.2.
    fn record(&mut self, run: Run) {
        let ix = self.runs.len();
        self.run_ids.insert(run.inst.id.clone(), ix);
        self.runs.push(run);
    }
}

/// Two `ties by` keys in order: numbers and money by value, days by date, texts and keys by
/// code point (refcore's `vkey`); keys of other kinds aren't built. L §5.14.2.
fn tie_cmp(a: &[Value], b: &[Value]) -> Result<Ordering, Stop> {
    for (x, y) in a.iter().zip(b) {
        let o = value_cmp(x, y)?;
        if o != Ordering::Equal {
            return Ok(o);
        }
    }
    Ok(a.len().cmp(&b.len()))
}

/// One `ties by` term's order. L §5.14.2.
fn value_cmp(x: &Value, y: &Value) -> Result<Ordering, Stop> {
    if let (Value::Number(p), Value::Number(q)) = (x, y)
        && p.unit() == q.unit()
    {
        return p.q.compare(&q.q).map_err(crate::eval::num_stop);
    }
    if let (Value::Money(p), Value::Money(q)) = (x, y)
        && p.currency() == q.currency()
    {
        return p.compare(q).map_err(crate::eval::num_stop);
    }
    if let (Value::Day(p), Value::Day(q)) = (x, y) {
        return Ok(p.cmp(q));
    }
    if let (Value::Text(p), Value::Text(q)) = (x, y) {
        return Ok(p.cmp(q));
    }
    if let (Value::Key(p), Value::Key(q)) = (x, y) {
        return Ok(p.text().cmp(&q.text()));
    }
    if let (Value::Bool(p), Value::Bool(q)) = (x, y) {
        return Ok(p.cmp(q));
    }
    Err(Stop::NotBuilt("a ties by over values of two kinds"))
}

/// A run-time refusal (L §8.4).
fn refuse(code: RunCode, message: String) -> Stop {
    Stop::Refused(Refusal::new(Code::Run(code), message))
}
