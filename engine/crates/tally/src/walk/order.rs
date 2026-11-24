//! A day's decisions: each move's instances due that day in their order, each run at its point against the balances as they stand, its legs booked and tested together.
//! Spec: E §4.2 step 3 (decisions by stratum, each at its point, computed at min(t, K) with
//! running reads; the legs it books from its point are one group of the sign rule), step 5;
//! L §5.14.1 (running reads), §5.14.2 (the order of a day's decisions: `ties by`, `RUN-TIE`),
//! §5.10 (the sign rule's point 2); E §4.2 step 3 and L §5.19 (a decision on a covered day
//! before K, or one an observation fulfils, is modelled past: its own observations booked at
//! its point, its legs not; it computes against the balances less its own observations
//! booked before its day); L §8.4 (one refused before its point, by its key's `where`, refuses
//! the places its legs give from its point; tally 2.1.6).
//! Twin: engine/tools/refcore/walk.py (`run_decisions`, `tie_order`, `run_one`,
//! `run_one_body`, `dec_bad`, `book_own_obs`).
//! Invariants: a decision reads the walk as it stands at its point, never a later point; a
//! run is recorded once, after its body is computed, so a run cut short past the counted depth
//! is run again from its start; the instances of one move run in the order of its `ties by`.

use core::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use super::day::ends;
use super::instance::{Amount, Dated, End, Exists, Inst, Status};
use super::plan::{invariant, kind_certainty};
use super::point::At;
use super::readings::Broke;
use super::state::{ALL, Change, Ent, Group, OwnLeg, Taken, WalkObs};
use crate::base::{Code, Refusal, Run as RunCode};
use crate::book::Obs;
use crate::eval::{Collected, Ctx, Engine, Entry};
use crate::hole::spread;
use crate::model::{KeyVal, MoveInfo, Reality, Target, Value};
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
    /// Every place it read, at its point or before it (E §10, `explain`).
    pub read: BTreeSet<String>,
    /// The places its legs name.
    pub changed: BTreeSet<String>,
    /// Whether its body was read at its point: not for one refused before its point (its
    /// key's `where`), nor for a cycle's member refused before it runs (`RUN-ORDER`,
    /// `RUN-TIE`), which had no point to read it at (E §4.2 step 3; E §10).
    pub ran: bool,
    /// What a refusal of its order tested (`RUN-ORDER`, `RUN-TIE`), which a read of it lists.
    pub tested: Tested,
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
        if self
            .walk(id)?
            .cycles_run
            .contains(&(move_id.to_owned(), day))
        {
            return Ok(());
        }
        let mut insts = self
            .walk(id)?
            .due
            .get(&(move_id.to_owned(), day))
            .cloned()
            .ok_or_else(|| invariant("a decision act with no instance due"))?;
        for group in &m.graph.order_groups {
            if group.contains(move_id)
                && group
                    .iter()
                    .any(|q| q != move_id && self.is_due(id, q, day))
            {
                return self.run_cycle(m, id, group, day, at);
            }
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

    /// The instances of an unordered cycle's decisions due on `day`, run once, at the act of
    /// the first of its moves to come: each move's instances in the order of its `ties by`,
    /// the refused ones first, then the rest in the order their reads give, each at its own
    /// point after the act's (refcore's `run_group`). L §5.14.2 (Cycles), E §4.2 step 3.
    fn run_cycle(
        &mut self,
        m: &Reality,
        id: usize,
        group: &BTreeSet<String>,
        day: Day,
        at: At,
    ) -> Result<(), Stop> {
        let judged = self.today(id)?.cycle.clone();
        let items = if let Some(items) = judged {
            items
        } else {
            let items = self.cycle_order(m, id, group, day, at)?;
            self.today_mut(id)?.cycle = Some(items.clone());
            items
        };
        let start = self.today(id)?.next_inst;
        for (n, item) in items.into_iter().enumerate().skip(start) {
            let n32 = u32::try_from(n).map_err(|_| invariant("an instance's rank"))?;
            let key = at
                .cycle_instance(n32)
                .ok_or_else(|| invariant("an instance's rank"))?;
            match item.refused {
                Some(err) => {
                    self.refused_run(
                        m,
                        id,
                        item.inst,
                        key,
                        (err, item.changes, &item.met, item.tested),
                    )?;
                }
                None => {
                    self.run_one(m, id, item.inst, key)?;
                }
            }
            self.today_mut(id)?.next_inst = n
                .checked_add(1)
                .ok_or_else(|| invariant("a count of runs"))?;
        }
        let walk = self.walk_mut(id)?;
        for q in group {
            walk.cycles_run.insert((q.clone(), day));
        }
        Ok(())
    }

    /// The order of an unordered cycle's instances due on `day`, read before any of them
    /// runs: one whose `when` is false, and stays so whatever another changes that day,
    /// orders nothing; one that reads a place another changes, or another's legs, runs after
    /// it, and a written `before` or `after` between two moves decides; two that read each
    /// other are refused, `RUN-ORDER`, or of one move go by its `ties by` (`RUN-TIE`
    /// without one); what no read orders runs by its move's id, then its place (refcore's
    /// `run_group`). L §5.14.2 (Cycles), E §4.2 step 3.
    fn cycle_order(
        &mut self,
        m: &Reality,
        id: usize,
        group: &BTreeSet<String>,
        day: Day,
        at: At,
    ) -> Result<Vec<CycleItem>, Stop> {
        let mut members = Vec::new();
        for q in group {
            let mq = m
                .moves
                .get(q)
                .ok_or_else(|| invariant("a cycle's move load didn't build"))?;
            let mut insts = self
                .walk(id)?
                .due
                .get(&(q.clone(), day))
                .cloned()
                .unwrap_or_else(Vec::new);
            if insts.len() > 1 && mq.decl.ties.is_some() {
                insts = self.tie_order(m, id, mq, insts, day)?;
            }
            for inst in insts {
                members.push(self.cycle_member(m, id, mq, inst, (day, at))?);
            }
        }
        let tied: BTreeSet<&str> = group
            .iter()
            .filter(|q| m.moves.get(q).is_some_and(has_ties))
            .map(String::as_str)
            .collect();
        let (order, bad) = cycle_ranks(&members, &tied, &m.graph.written_order, day)?;
        let mut refused: Vec<(usize, Refusal)> = bad.into_iter().collect();
        refused.sort_by(|a, b| rank_cmp(&members, a.0, b.0));
        // a refusal of the order rests on what every instance due read to be ordered, so the
        // pins and assumptions those reads took are listed with it (L §8.2 One read set;
        // tally 2.1.8 §4, #161; refcore's `run_group`, `ord_reads`)
        let mut rests = Collected::default();
        let mut whens = Vec::with_capacity(members.len());
        for member in &members {
            rests.merge(&member.met);
            whens.push((member.inst.move_id.clone(), member.inst.id.clone()));
        }
        let mut items = Vec::with_capacity(members.len());
        for (x, err) in refused {
            let member = members
                .get(x)
                .ok_or_else(|| invariant("a cycle's member past its list"))?;
            items.push(CycleItem {
                inst: member.inst.clone(),
                refused: Some(err),
                changes: member.changes.clone(),
                met: rests.clone(),
                tested: Tested::Whens(at, whens.clone()),
            });
        }
        for x in order {
            let member = members
                .get(x)
                .ok_or_else(|| invariant("a cycle's member past its list"))?;
            items.push(CycleItem {
                inst: member.inst.clone(),
                refused: None,
                changes: member.changes.clone(),
                met: Collected::default(),
                tested: Tested::Nothing,
            });
        }
        Ok(items)
    }

    /// What orders one instance of a cycle: the places its legs give that day, the places
    /// and moves it declares it reads, read at min(t, K) and its law date, none where one
    /// can't be read there; its `when` read at the act's point with running reads, and the
    /// places it read (refcore's `inst_places`, `run_group`'s `when`). L §5.14.1, §5.14.2.
    fn cycle_member(
        &mut self,
        m: &Reality,
        id: usize,
        mv: &MoveInfo,
        inst: Inst,
        (day, at): (Day, At),
    ) -> Result<CycleMember, Stop> {
        let cut = self.walk(id)?.cut;
        let ctx = self.timeline_ctx(inst.law.day(), day.min(cut));
        self.push();
        let read = self.cycle_reads(m, mv, &inst, ctx);
        let running = Ctx {
            at: Some(WalkAt {
                walk: id,
                point: at,
            }),
            ..ctx
        };
        let when = if read.is_ok() {
            self.when_before(m, id, mv, &inst, running)
        } else {
            Ok((false, BTreeSet::new()))
        };
        // what it read to be ordered, with what its instance read (refcore's `ord_reads`)
        let mut met = inst.coll.clone();
        met.merge(&self.pop_alone());
        let (changes, reads, moves) = read?;
        let (off, seen) = when?;
        Ok(CycleMember {
            inst,
            changes,
            reads,
            moves,
            off,
            seen,
            met,
        })
    }

    /// The places an instance's legs give, and the places and moves it declares it reads,
    /// each none where one can't be read (refcore's `inst_places`). L §5.14.1, §5.14.2.
    fn cycle_reads(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &Inst,
        ctx: Ctx,
    ) -> Result<Reads, Stop> {
        let changes = self.leg_places(m, mv, inst, ctx)?;
        let mut reads = BTreeSet::new();
        let mut moves = BTreeSet::new();
        for (read, target) in mv.decl.reads.iter().zip(&mv.read_targets) {
            match target {
                Target::Move(q) => {
                    moves.insert(q.clone());
                }
                Target::Kind(kind) => moves.extend(decisions_of(m, kind)),
                Target::PlaceSec(section) => reads.extend(things_of(m, section)),
                Target::Thing(_) | Target::Unknown => {
                    let Some(Value::Key(KeyVal::Thing(place))) =
                        self.run_read(m, read, &inst.env, ctx)?
                    else {
                        // a place that can't be read here meets every place; the reads after
                        // it aren't read (refcore's `inst_places`)
                        return Ok((changes, None, moves));
                    };
                    reads.insert(place);
                }
            }
        }
        Ok((changes, Some(reads), moves))
    }

    /// The places an instance's legs give; a hole place gives none, and none where one is
    /// neither a thing nor a hole, or refused (refcore's `inst_places`, its changes).
    /// L §5.14.2, §8.4.
    pub(crate) fn leg_places(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &Inst,
        ctx: Ctx,
    ) -> Result<Option<BTreeSet<String>>, Stop> {
        let mut changes = BTreeSet::new();
        for end in mv.decl.legs.iter().flat_map(|leg| [&leg.from, &leg.to]) {
            match self.run_read(m, end, &inst.env, ctx)? {
                Some(Value::Key(KeyVal::Thing(place))) => {
                    changes.insert(place);
                }
                Some(Value::Hole(_)) => {}
                Some(_) | None => return Ok(None),
            }
        }
        Ok(Some(changes))
    }

    /// Whether an instance's `when` is false at point `ctx.at`, read before any of the
    /// cycle runs, and the places it read there; a refusal there orders as true (refcore's
    /// `run_group`, its `when`). L §5.14.1, §5.14.2.
    fn when_before(
        &mut self,
        m: &Reality,
        id: usize,
        mv: &MoveInfo,
        inst: &Inst,
        ctx: Ctx,
    ) -> Result<(bool, BTreeSet<String>), Stop> {
        let Some(when) = &mv.decl.when else {
            return Ok((false, BTreeSet::new()));
        };
        let prev = self.walk_mut(id)?.live.replace(BTreeSet::new());
        let value = self.ev(m, when, &inst.env, ctx);
        let Some(live) = core::mem::replace(&mut self.walk_mut(id)?.live, prev) else {
            return Err(invariant("a when's live reads taken"));
        };
        match value {
            Ok(v) => Ok((v == Value::Bool(false), live)),
            Err(Stop::Refused(r)) if run_time(&r) => Ok((false, BTreeSet::new())),
            Err(stop) => Err(stop),
        }
    }

    /// The value of `e` at `ctx`, or none where a run-time refusal stops it; a load refusal
    /// met at run, `ENGINE-INVARIANT` and the walk's other stops go on (R-66; refcore's
    /// `try: ev … except TallyError`). L §8.4.
    fn run_read(
        &mut self,
        m: &Reality,
        e: &crate::syntax::Expr,
        env: &crate::eval::Env,
        ctx: Ctx,
    ) -> Result<Option<Value>, Stop> {
        match self.ev(m, e, env, ctx) {
            Ok(v) => Ok(Some(v)),
            Err(Stop::Refused(r)) if run_time(&r) => Ok(None),
            Err(stop) => Err(stop),
        }
    }

    /// A cycle's instance refused before it runs, `RUN-ORDER` or `RUN-TIE`: the places its
    /// legs give refused from its point, every place where none is known, and its run kept
    /// with no legs, so every read of it is refused too, and every link to it (refcore's
    /// `run_group`, its bad, and `refused_run`). L §5.14.2, §8.4; E §4.2 step 5; tally 2.3.1
    /// R2, C-9.
    fn refused_run(
        &mut self,
        m: &Reality,
        id: usize,
        mut inst: Inst,
        key: At,
        (err, changes, met, tested): (Refusal, Option<BTreeSet<String>>, &Collected, Tested),
    ) -> Result<(), Stop> {
        let walk = self.walk_mut(id)?;
        let refused = || {
            let mut ent = Ent::new(key, Change::Bad(err.clone()));
            ent.coll = Some(met.clone());
            ent
        };
        match changes.filter(|c| !c.is_empty()) {
            Some(places) => {
                for place in places {
                    let unit = m
                        .things
                        .get(&place)
                        .and_then(|t| m.place_unit(t))
                        .unwrap_or_else(|| ALL.to_owned());
                    walk.add(m, &place, &unit, refused())?;
                }
            }
            None => walk.add(m, ALL, ALL, refused())?,
        }
        inst.legs = Vec::new();
        inst.attrs = Vec::new();
        inst.err = Some(err.clone());
        inst.status = Status::Refused;
        // what the refusal rests on is what a read of the run lists (L §8.2; P20)
        inst.coll.merge_rows(met);
        let tid = inst.id.clone();
        walk.record(Run {
            inst,
            point: key,
            live: BTreeSet::new(),
            read: BTreeSet::new(),
            changed: BTreeSet::new(),
            ran: false,
            tested,
        });
        // a link to it rests on its refused existence (tally 2.3.1 R2, C-9)
        self.unlink_run(id, &tid, Some(&err))
    }

    /// Instances of one move that read the places another of them changes, with no `ties by`:
    /// their order changes the result, `RUN-TIE` at every place they change and on each of them
    /// (refcore's `run_decisions`). L §5.14.2, §8.
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
        // the tie refuses each tied instance, so every read of it is refused, its legs and
        // its flows, not only the balances it changes (L §8, §5.14.2; tally 2.1.7 R2, G7)
        // and rests on every tied run as it ran, which a read of each lists (L §8.2; P20)
        let tied = self.today(id)?.runs.clone();
        let tied_walk = self.walk_mut(id)?;
        let mut rests = Collected::default();
        let mut tested = Vec::with_capacity(tied.len());
        for ix in &tied {
            if let Some(run) = tied_walk.runs.get(*ix) {
                rests.merge(&run.inst.coll);
                tested.push(run.inst.id.clone());
            }
        }
        for ix in tied {
            let run = tied_walk
                .runs
                .get_mut(ix)
                .ok_or_else(|| invariant("a tied run past the walk's runs"))?;
            run.inst.err = Some(err.clone());
            run.inst.status = Status::Refused;
            run.inst.coll.merge_rows(&rests);
            run.tested = Tested::Runs(tested.clone());
            let (movement, legs) = (run.inst.id.clone(), run.inst.legs.clone());
            let read = run.inst.coll.clone();
            for lg in &legs {
                tied_walk.leg_bad(m, (&movement, lg, &read), &err, key)?;
            }
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
            let ctx = self.timeline_ctx(inst.law.day(), cut);
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
    /// `run_one_body`). One refused before its point (its key's `where`) reads nothing of its
    /// body; a refused one's legs' places are read at its point (L §8, §8.4; tally 2.1.7 R2,
    /// #104). E §4.2 step 3, L §5.14.1.
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
            ..self.timeline_ctx(None, t.min(cut))
        };
        let own = self.own_earlier_obs(m, id, &inst, t)?;
        let prev_less = core::mem::replace(&mut self.walk_mut(id)?.less_obs, own);
        let prev_live = self.walk_mut(id)?.live.replace(BTreeSet::new());
        let prev_reads = self.walk_mut(id)?.reads.replace(BTreeSet::new());
        self.push();
        let body = match &inst.err {
            // refused before its point (its key's `where`): nothing of its body is read but its
            // places (tally 2.1.7 R2, #104)
            Some(err) => Err(Stop::Refused(err.clone())),
            None => self.inst_body(
                m,
                mv,
                &inst,
                ctx.at_law(inst.law.day()),
                inst.exists.clone(),
            ),
        };
        let met = self.pop_alone();
        self.walk_mut(id)?.less_obs = prev_less;
        let Some(live) = core::mem::replace(&mut self.walk_mut(id)?.live, prev_live) else {
            return Err(invariant("a run's live reads taken"));
        };
        let Some(read) = core::mem::replace(&mut self.walk_mut(id)?.reads, prev_reads) else {
            return Err(invariant("a run's reads taken"));
        };
        let linked = self.obs_links_for(m, &inst.id, cut)?;
        let ran = inst.err.is_none();
        let mut run = Run {
            inst,
            point: key,
            live,
            read,
            changed: BTreeSet::new(),
            ran,
            tested: Tested::Nothing,
        };
        run.inst.coll.merge(&met);
        run.inst.obs = linked;
        run.inst.err = None;
        match body {
            Ok(b) => {
                run.inst.exists = b.exists;
                run.inst.legs = b.legs;
                run.inst.leg_reads = b.leg_reads;
                run.inst.attrs = b.attrs;
                run.inst.for_period = b.for_period;
                run.inst.fulfils = b.fulfils;
            }
            Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => run.inst.err = Some(r),
            Err(stop) => return Err(stop),
        }
        // a refused instance's legs' places are still read, at its point, so the refusal
        // reaches the places its legs give (L §8.4; tally 2.1.7 R2, #104)
        if run.inst.err.is_some() && run.inst.legs.is_empty() && !mv.decl.legs.is_empty() {
            let law = run.inst.law.day();
            self.refused_at_point(m, mv, &mut run.inst, ctx.at_law(law))?;
        }
        self.walk_mut(id)?.group = Some(Group::new());
        let ix = self.book_run(m, id, run, t)?;
        // the ends stated same as this decision that waited for its point (L §6.1)
        self.same_release(m, id, ix)?;
        Ok(ix)
    }

    /// The legs of decision instance `inst`, refused before its point, with their places read
    /// at its point (running reads, its law and facts at min(t, K)) and the refusal as their
    /// amount, so it refuses those places from its point in the day's order; none where a
    /// place can't be read there, a hole or a refusal, so it refuses every place (refcore's
    /// `dec_bad`, read at the decision's point). L §8.4 (tally 2.1.6), E §4.1 (the order of a
    /// day's refusals), E §4.2 step 3, Holes in the walk.
    fn refused_at_point(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &mut Inst,
        ctx: Ctx,
    ) -> Result<(), Stop> {
        self.push();
        let read = self.refused_places(m, mv, inst, ctx);
        inst.coll.merge(&self.pop_alone());
        // a place a run-time refusal stops is none here; a load refusal met at run and
        // ENGINE-INVARIANT go on (R-66)
        inst.legs = match read {
            Ok(Some(legs)) => legs,
            Ok(None) => Vec::new(),
            Err(stop) => return Err(stop),
        };
        Ok(())
    }

    /// What a run books by what its body gave (refcore's `run_one_body`, after the body); a
    /// refused run refuses the places its known legs give from its point, every place where
    /// none is known, and none for a move with no legs (L §8.4, tally 2.1.6; P16). E §4.2
    /// step 3, Holes in the walk.
    fn book_run(&mut self, m: &Reality, id: usize, mut run: Run, t: Day) -> Result<usize, Stop> {
        let key = run.point;
        let (cut, main) = {
            let walk = self.walk(id)?;
            (walk.cut, walk.proj.is_none())
        };
        let modelled = main
            && ((t < cut && self.touches_covered(m, &run.inst, cut)) || !run.inst.obs.is_empty());
        let legless = m
            .moves
            .get(&run.inst.move_id)
            .ok_or_else(|| invariant("a decision of a move load didn't build"))?
            .decl
            .legs
            .is_empty();
        // the walk and the run's register entries, apart: a past run notes `unfulfilled:`
        let walk = self
            .walks
            .get_mut(id)
            .ok_or_else(|| invariant("a walk the engine lacks"))?;
        let ix = walk.runs.len();
        if let Some(err) = run.inst.err.clone() {
            // refused: the places its legs give, every place where one can't be read, none for
            // a move with no legs (L §8.4, tally 2.1.6; P16)
            run.inst.status = Status::Refused;
            let known: Vec<_> = run
                .inst
                .legs
                .iter()
                .filter(|lg| !ends(lg).is_empty())
                .cloned()
                .collect();
            let mv_id = run.inst.id.clone();
            let read = run.inst.coll.clone();
            walk.record(run);
            if known.is_empty() && !legless {
                walk.add(m, ALL, ALL, Ent::new(key, Change::Bad(err.clone())))?;
            }
            for lg in &known {
                walk.leg_bad(m, (&mv_id, lg, &read), &err, key)?;
            }
            self.unlink_run(id, &mv_id, Some(&err))?;
            return Ok(ix);
        }
        if run.inst.exists == Exists::No {
            run.inst.status = Status::None;
            let inst = run.inst.clone();
            walk.record(run);
            self.unlink_run(id, &inst.id, None)?;
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
            reg.push(self.entries.intern(Entry {
                word: "unfulfilled",
                text: run.inst.id.clone(),
                figure: String::new(),
            })?);
        }
        let legs = run.inst.legs.clone();
        let inst = run.inst.clone();
        walk.record(run);
        for lg in &legs {
            self.book_leg(m, id, (&inst, lg), key, &reg)?;
        }
        self.book_own_obs(m, id, &inst, key)?;
        Ok(ix)
    }

    /// The links to decision instance `tid` of walk `id`, judged at its point, where its
    /// existence isn't true (its `when` false, or `why`, its run refused): each observation
    /// that fulfils it refused, in the walk too, so its legs book refused at that point; none
    /// in a projection's walk, whose links are the main walk's (E §4.2 step 3; L §5.19 Links,
    /// §8.4; tally 2.3.1 R6, R2, C-9).
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a walk the engine lacks.
    fn unlink_run(&mut self, id: usize, tid: &str, why: Option<&Refusal>) -> Result<(), Stop> {
        let (cut, main) = {
            let walk = self.walk(id)?;
            (walk.cut, walk.proj.is_none())
        };
        if !main {
            return Ok(());
        }
        let refused = self.unlink_at_point(tid, cut, why);
        let walk = self.walk_mut(id)?;
        for (obs, err) in refused {
            for wo in walk.obs.iter_mut().filter(|wo| wo.obs.id == obs) {
                wo.err = Some(err.clone());
            }
        }
        Ok(())
    }

    /// Puts walk `id`'s running reads back as decision run `run` read them at its point, its
    /// own observations booked before its day left out, for a cell traced again there; returns
    /// what [`Engine::leave_run`] puts back (`explain` of a walk-bound def: "the traced value
    /// equals the stored one"). E §10, §4.2 step 3, L §5.19.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a run with no day or a walk the engine lacks.
    pub(crate) fn enter_run(
        &mut self,
        m: &Reality,
        id: usize,
        run: &Run,
    ) -> Result<Option<Vec<OwnLeg>>, Stop> {
        let Some(Dated::Day(t)) = run.inst.day else {
            return Err(invariant("a decision run with no day"));
        };
        let own = self.own_earlier_obs(m, id, &run.inst, t)?;
        Ok(core::mem::replace(&mut self.walk_mut(id)?.less_obs, own))
    }

    /// Puts back walk `id`'s running reads as [`Engine::enter_run`] found them. E §4.2 step 3.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a walk the engine lacks.
    pub(crate) fn leave_run(&mut self, id: usize, prev: Option<Vec<OwnLeg>>) -> Result<(), Stop> {
        self.walk_mut(id)?.less_obs = prev;
        Ok(())
    }

    /// The legs of a decision's own observations booked on their days before its day, the
    /// cut and after the walk's start, each with the ends its row booked: its computation
    /// reads the balances less them (refcore's `own_obs`, `less_obs`). E §4.2 step 3, L §5.19.
    fn own_earlier_obs(
        &mut self,
        m: &Reality,
        id: usize,
        inst: &Inst,
        t: Day,
    ) -> Result<Option<Vec<OwnLeg>>, Stop> {
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
            for leg in &o.legs {
                let Dated::Day(d) = leg.day else {
                    continue;
                };
                if d >= t || d >= cut || d < start {
                    continue;
                }
                let from = self.end_booked(m, &o, &leg.from, (d, cut));
                let to = self.end_booked(m, &o, &leg.to, (d, cut));
                out.push(OwnLeg {
                    leg: leg.clone(),
                    from,
                    to,
                });
            }
        }
        Ok((!out.is_empty()).then_some(out))
    }

    /// Whether observation `o` books its end `end` on day `d`: a place no version covers
    /// that day, or one `o`'s own version covers (refcore's `own_obs`, its `obs_end_on`).
    /// L §5.19 Booking an observation.
    fn end_booked(&mut self, m: &Reality, o: &Obs, end: &End, (d, cut): (Day, Day)) -> bool {
        let Some(place) = end.place() else {
            return false;
        };
        self.obs_end_on(m, o, place, d, cut)
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
        // its observations of day t, each leg on its observation's day
        for wo in own.iter().filter(|wo| wo.obs.day == t) {
            for lg in &wo.obs.legs {
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

/// One instance of an unordered cycle as its order judged it: refused before it runs, or run
/// (refcore's `run_group`, its order and its bad). L §5.14.2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CycleItem {
    /// The instance.
    pub inst: Inst,
    /// Its refusal, `RUN-ORDER` or `RUN-TIE`, where it doesn't run.
    pub refused: Option<Refusal>,
    /// The places its legs give that day; none where one can't be read there.
    pub changes: Option<BTreeSet<String>>,
    /// What its refusal rests on: what every instance due read to be ordered; empty for one
    /// that runs.
    pub met: Collected,
    /// What its refusal tested; nothing for one that runs.
    pub tested: Tested,
}

/// What a refusal of the order tested, which a read of a refused run lists (L §8.2 One read
/// set; E §4.2 step 3; P20).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tested {
    /// No refusal of the order.
    Nothing,
    /// A cycle's refusal, `RUN-ORDER` or `RUN-TIE`: each member's `when` (its move's id, its
    /// instance's), read at the act's point, before any of them runs.
    Whens(At, Vec<(String, String)>),
    /// A tie of one move's runs, `RUN-TIE`: each tied run, by its instance, as it ran.
    Runs(Vec<String>),
}

/// What orders one instance of a cycle (refcore's `run_group`, its `info`). L §5.14.2.
struct CycleMember {
    /// The instance.
    inst: Inst,
    /// The places its legs give that day; none where one can't be read there.
    changes: Option<BTreeSet<String>>,
    /// The places it declares it reads; none where one can't be read there.
    reads: Option<BTreeSet<String>>,
    /// The decision moves whose legs it declares it reads.
    moves: BTreeSet<String>,
    /// Whether its `when` is false before any of the cycle runs.
    off: bool,
    /// The places its `when` read.
    seen: BTreeSet<String>,
    /// What it and its instance read to be ordered (refcore's `ord_reads`).
    met: Collected,
}

/// The places an instance's legs give, the places it declares it reads, each none where one
/// can't be read, and the moves whose legs it reads (refcore's `inst_places`).
type Reads = (
    Option<BTreeSet<String>>,
    Option<BTreeSet<String>>,
    BTreeSet<String>,
);

/// The order of a cycle's members and the refused ones with their refusal (refcore's
/// `run_group`, after its `info`): see `Engine::cycle_order`. L §5.14.2 (Cycles).
fn cycle_ranks(
    members: &[CycleMember],
    tied: &BTreeSet<&str>,
    written: &BTreeSet<(String, String)>,
    day: Day,
) -> Result<(Vec<usize>, BTreeMap<usize, Refusal>), Stop> {
    let mut after = cycle_after(members, written);
    let mut bad: BTreeMap<usize, Refusal> = BTreeMap::new();
    for (x, mx) in members.iter().enumerate() {
        let ys: Vec<usize> = set_of(&after, x)?.iter().copied().collect();
        for y in ys {
            if y < x || !set_of(&after, y)?.contains(&x) {
                continue;
            }
            let my = members
                .get(y)
                .ok_or_else(|| invariant("a cycle's member past its list"))?;
            let (a, b) = (mx.inst.move_id.as_str(), my.inst.move_id.as_str());
            if a == b && tied.contains(a) {
                // one move's instances go by its ties by
                if let Some(set) = after.get_mut(x) {
                    set.remove(&y);
                }
                continue;
            }
            let err = mutual(a, b, day);
            bad.entry(x).or_insert_with(|| err.clone());
            bad.entry(y).or_insert(err);
        }
    }
    let mut left: BTreeSet<usize> = (0..members.len())
        .filter(|x| !bad.contains_key(x))
        .collect();
    let mut order = Vec::with_capacity(left.len());
    while !left.is_empty() {
        let mut ranked: Vec<usize> = left.iter().copied().collect();
        ranked.sort_by(|a, b| rank_cmp(members, *a, *b));
        let mut ready = None;
        for x in ranked {
            if set_of(&after, x)?.is_disjoint(&left) {
                ready = Some(x);
                break;
            }
        }
        let Some(x) = ready else {
            let moves: BTreeSet<&str> = left
                .iter()
                .filter_map(|x| members.get(*x))
                .map(|mx| mx.inst.move_id.as_str())
                .collect();
            let names: Vec<&str> = moves.into_iter().collect();
            let err = refusal(
                RunCode::Order,
                format!(
                    "decisions {} read each other on {}",
                    names.join(" and "),
                    day.text()
                ),
            );
            for x in &left {
                bad.insert(*x, err.clone());
            }
            break;
        };
        order.push(x);
        left.remove(&x);
    }
    Ok((order, bad))
}

/// Two members that read each other: of one move with no `ties by`, `RUN-TIE`; of two moves,
/// `RUN-ORDER` (refcore's `run_group`, its bad). L §5.14.2.
fn mutual(a: &str, b: &str, day: Day) -> Refusal {
    if a == b {
        return refusal(
            RunCode::Tie,
            format!(
                "{a} has a tie at {} that changes the result, and no ties by",
                day.text()
            ),
        );
    }
    let (first, second) = if a < b { (a, b) } else { (b, a) };
    refusal(
        RunCode::Order,
        format!(
            "decisions {first} and {second} read each other on {}",
            day.text()
        ),
    )
}

/// Each member's members to run after: those whose places it reads, or whose legs, unless a
/// `when` false whatever they change takes either out; a written `before` or `after` between
/// their moves decides (refcore's `run_group`, its `after`). L §5.14.2 (Cycles).
fn cycle_after(
    members: &[CycleMember],
    written: &BTreeSet<(String, String)>,
) -> Vec<BTreeSet<usize>> {
    let off: Vec<bool> = members
        .iter()
        .enumerate()
        .map(|(x, mx)| mx.off && stays_off(members, x, mx))
        .collect();
    let on = |x: usize| off.get(x) == Some(&false);
    let mut after: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); members.len()];
    for (x, set) in after.iter_mut().enumerate() {
        let Some(mx) = members.get(x) else {
            continue;
        };
        for (y, my) in members.iter().enumerate() {
            if x != y && on(x) && on(y) && reads_of(mx, my) {
                set.insert(y);
            }
        }
    }
    for (x, mx) in members.iter().enumerate() {
        for (y, my) in members.iter().enumerate() {
            let pair = (mx.inst.move_id.clone(), my.inst.move_id.clone());
            if x == y || !on(x) || !on(y) || !written.contains(&pair) {
                continue;
            }
            if let Some(set) = after.get_mut(x) {
                set.insert(y);
            }
            if let Some(set) = after.get_mut(y) {
                set.remove(&x);
            }
        }
    }
    after
}

/// Whether member `x`'s false `when` read nothing another member changes that day, so it
/// stays false whatever runs first (refcore's `run_group`, its `off`). L §5.14.2.
fn stays_off(members: &[CycleMember], x: usize, mx: &CycleMember) -> bool {
    members
        .iter()
        .enumerate()
        .all(|(y, my)| y == x || !meets(&mx.seen, my.changes.as_ref()))
}

/// Whether a member reads what another changes that day, or its legs; a set that can't be
/// read meets every place (refcore's `run_group`, `reads_of`). L §5.14.2.
fn reads_of(x: &CycleMember, y: &CycleMember) -> bool {
    if x.moves.contains(&y.inst.move_id) {
        return true;
    }
    match (&x.reads, &y.changes) {
        (Some(reads), Some(changes)) => !reads.is_disjoint(changes),
        _ => true,
    }
}

/// Whether places `read` meet the places a member changes; changes that can't be read meet
/// every place. L §5.14.2.
fn meets(read: &BTreeSet<String>, changes: Option<&BTreeSet<String>>) -> bool {
    changes.is_none_or(|c| !read.is_disjoint(c))
}

/// Member `x`'s members to run after. L §5.14.2.
fn set_of(after: &[BTreeSet<usize>], x: usize) -> Result<&BTreeSet<usize>, Stop> {
    after
        .get(x)
        .ok_or_else(|| invariant("a cycle's member past its list"))
}

/// Two cycle members in the order no read gives: by move id, then by their place in the
/// list, each move's in its `ties by` order (refcore's `rank`). L §5.14.2.
fn rank_cmp(members: &[CycleMember], a: usize, b: usize) -> Ordering {
    (member_move(members, a), a).cmp(&(member_move(members, b), b))
}

/// The move of the cycle member at `x`. L §5.14.2.
fn member_move(members: &[CycleMember], x: usize) -> Option<&str> {
    members.get(x).map(|mx| mx.inst.move_id.as_str())
}

/// Whether a move states a `ties by`. L §5.14.2.
fn has_ties(mv: &MoveInfo) -> bool {
    mv.decl.ties.is_some()
}

/// The decision moves of event kind `kind` or a subkind. L §5.14.1 (`reads K`).
fn decisions_of(m: &Reality, kind: &str) -> Vec<String> {
    let sub = m.subtree(kind);
    m.moves
        .entries()
        .filter(|(_, mv)| mv.decision && sub.contains(&mv.kind))
        .map(|(_, mv)| mv.id.clone())
        .collect()
}

/// Every instance of keyed place `section`. L §5.14.1 (`reads x` of a keyed place).
fn things_of(m: &Reality, section: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (_, t) in m.things.entries() {
        if let Some(keys) = &t.place
            && keys.place == section
        {
            out.push(t.name.clone());
        }
    }
    out
}

/// Whether a refusal is a run-time one, which a read that may fail takes as unknown there;
/// a load refusal met at run and `ENGINE-INVARIANT` stop the run (R-66). L §8.3, §8.4.
pub(super) fn run_time(r: &Refusal) -> bool {
    !matches!(r.code(), Code::Load(_) | Code::EngineInvariant)
}

/// A run-time refusal (L §8.4).
fn refusal(code: RunCode, message: String) -> Refusal {
    Refusal::new(Code::Run(code), message)
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

#[cfg(test)]
mod tests {
    use crate::api::{Asks, Files, Given, Request, run};

    /// Two decisions over three months whose key's `where` is refused in February (a
    /// division by zero): `split`'s body is answered there, `spill`'s refused too, by its
    /// `when`.
    const REFUSED_WHERE: &str = r#"model refused_where
  tally 2
  on 2026-02-15
  start 2026-01-01
  end 2026-03-31

source fixture "invented values"

unit PLN: currency minor 0.01

index month = month 2026-01..2026-03

place konto: accounts PLN lives 2026-01-01..

place box: accounts PLN lives 2026-01-01..

place bin: accounts PLN lives 2026-01-01..

place world: outside lives 2026-01-01..

input dec_dom: int

input fee_amt: money PLN

input wdiv[m in month]: num

input adiv[m in month]: num

move split[m in month where 1 / wdiv[m] > 0]: transfers
  on day_of(m, dec_dom)
  at start_of(m)
  reads box
  leg cash from world to box amount fee_amt + 0 * bal(box)

move spill[m in month where 1 / wdiv[m] > 0]: transfers
  on day_of(m, dec_dom)
  at start_of(m)
  reads bin
  when 1 / adiv[m] > 0
  leg cash from world to bin amount fee_amt + 0 * bal(bin)

ask q_bal: bal(konto, d), bal(box, d), bal(bin, d) for d in [2026-01-20, 2026-02-19, 2026-02-20, 2026-03-20] law cut

version facts
  cite fixture
  governs always
  known 2025-12-01
  dec_dom = 20 "the decisions' day of the month"
  fee_amt = 3.00 "a fee"
  wdiv[2026-01] = 1 "answered"
  wdiv[2026-02] = 0 "a division by zero: the key's where is refused"
  wdiv[2026-03] = 2 "answered"
  adiv[2026-01] = 1 "answered"
  adiv[2026-02] = 0 "a division by zero: spill's when is refused too"
  adiv[2026-03] = 2 "answered"

version openings
  cite fixture
  governs always
  known 2025-12-31
  opening[konto] = 100 at 2026-01-01 start
  opening[box] = 0 at 2026-01-01 start
  opening[bin] = 0 at 2026-01-01 start
"#;

    #[test]
    fn a_decision_refused_by_its_keys_where_refuses_the_places_its_legs_give_at_its_point() {
        // L §8.4 (tally 2.1.6): split[2026-02] and spill[2026-02] are refused by their key's
        // where; each refuses the place its leg gives from its point, box and bin, whether its
        // body is answered (split) or refused too (spill, its leg's place read at its point),
        // and konto stands; refcore drops both instances (#104)
        let given = vec![Given {
            path: "where.tally".to_owned(),
            bytes: Some(REFUSED_WHERE.as_bytes().to_vec()),
        }];
        let request = Request::new(Asks::Named(vec!["q_bal".to_owned()]));
        let answer = run(Files::new(given), &request).expect("an answer");
        let values: Vec<&str> = answer
            .lines()
            .iter()
            .filter(|line| line.starts_with("q_bal "))
            .map(String::as_str)
            .collect();
        assert_eq!(
            values,
            [
                "q_bal (d = 2026-01-20): (100.00 PLN, 3.00 PLN, 3.00 PLN)",
                "q_bal (d = 2026-02-19): (100.00 PLN, 3.00 PLN, 3.00 PLN)",
                "q_bal (d = 2026-02-20): (100.00 PLN, refused: RUN-DIV division by zero, refused: RUN-DIV division by zero)",
                "q_bal (d = 2026-03-20): (100.00 PLN, refused: RUN-DIV division by zero, refused: RUN-DIV division by zero)"
            ],
            "box and bin refused from 20 February, konto standing"
        );
    }

    /// "Pay when the balance drops" (L §5.14.1), a February bank row fulfilling its instance
    /// on its own day: the critic's `dec_link.tally`, its limit written in for `LIM`.
    const DEC_LINK: &str = r#"model dec_link
  tally 2
  on 2026-04-20
  start 2026-01-01
  end 2026-05-31

source s "probe"

unit PLN: currency minor 0.01

index month = month 2026-01..2026-05

place konto: accounts PLN lives 2026-01-01..

place save: accounts PLN lives 2026-01-01..

place shop: outside lives 1900-01-01..

input bank: observed

input lim: money PLN

input amt: money PLN

input d5: int

move topup[m in month]: transfers
  reads konto
  on day_of(m, d5)
  when bal(konto) < lim
  leg cash from save to konto amount amt

ask loose: sum(o.cash.amount for o in bank over 2026-02 where o.fulfils == none)

ask b: bal(konto, 2026-03-31)

ask f: flows(topup, 2026-02)

version v
  cite s
  governs always
  known 2026-01-01
  opening[konto] = 100.00 PLN at 2026-01-01 start "s"
  opening[save] = 5 000.00 PLN at 2026-01-01 start "s"
  lim = LIM "s"
  amt = 400.00 PLN "s"
  d5 = 5 "s"

version feb
  cite s
  governs 2026-02
  known 2026-03-01
  covers konto
  bank[r1] = (on: 2026-02-05, kind: transfers, leg cash from save to konto amount 400.00 PLN, fulfils: topup[2026-02]) "the top-up"
  bank[r2] = (on: 2026-02-10, kind: transfers, leg cash from konto to shop amount 30.00 PLN) "loose"
"#;

    /// The value lines of `asks` run in that order on `DEC_LINK` with limit `lim`.
    fn dec_link(lim: &str, asks: &[&str]) -> Vec<String> {
        let given = vec![Given {
            path: "dec_link.tally".to_owned(),
            bytes: Some(DEC_LINK.replace("LIM", lim).into_bytes()),
        }];
        let asks = asks.iter().map(|a| (*a).to_owned()).collect();
        let answer = run(Files::new(given), &Request::new(Asks::Named(asks))).expect("an answer");
        answer
            .lines()
            .iter()
            .filter(|line| !line.starts_with(' ') && !line.starts_with("warning"))
            .cloned()
            .collect()
    }

    #[test]
    fn a_link_to_a_decision_is_judged_at_its_walk_point_whatever_reads_it_first() {
        // L §5.14 An instance's existence is a cell, §5.19 Links; E §4.2 step 3 (tally 2.3.1
        // R6, C-9): the `when` reads the running balance, so the link is judged at the
        // instance's point, never refused at load; with the limit at 600.00 PLN it runs and
        // the row stands for it, at 500.00 PLN it doesn't, and the row is refused
        for order in [
            ["loose", "b", "f"],
            ["b", "f", "loose"],
            ["f", "loose", "b"],
        ] {
            let mut on = dec_link("600.00 PLN", &order);
            on.sort();
            assert_eq!(
                on,
                ["b: 870.00 PLN", "f: 400.00 PLN", "loose: 30.00 PLN"],
                "{order:?}"
            );
            let refused = "refused: RUN-RECORD observation bank[r1] fulfils topup[2026-02], \
                           which doesn't exist or doesn't touch its places";
            let mut off = dec_link("500.00 PLN", &order);
            off.sort();
            assert_eq!(
                off,
                [
                    format!("b: {refused}"),
                    format!("f: {refused}"),
                    format!("loose: {refused}")
                ],
                "{order:?}"
            );
        }
    }

    #[test]
    fn a_link_to_a_decision_whose_run_is_refused_is_refused_with_it() {
        // L §5.19 Links, §8.4, §5.14.2; E §4.2 step 3 (tally 2.3.1 R2, C-9): `spend` and
        // `topup` read each other on each 5th with no order, so `topup[2026-02]`'s existence
        // is refused at its point, and the row that names it, read for its `fulfils`, is
        // refused with it, its explanation too
        let spend = "move spend[m in month]: transfers\n  reads konto\n  on day_of(m, d5)\n  \
                     leg cash from konto to shop amount 1.00 PLN + 0 * bal(konto)\n\n\
                     def lo[m in month]: money PLN at start_of(m)\n  rule guessed = 0.00 PLN\n  \
                     rule seen beats guessed = sum(o.cash.amount for o in bank over m \
                     where o.fulfils == none) when covered(konto, m)\n\n\
                     ask el: explain lo[2026-02]\n\nask loose:";
        let text = DEC_LINK
            .replace("LIM", "600.00 PLN")
            .replace("ask loose:", spend);
        let given = vec![Given {
            path: "dec_order.tally".to_owned(),
            bytes: Some(text.into_bytes()),
        }];
        let asks = vec!["loose".to_owned(), "el".to_owned()];
        let answer = run(Files::new(given), &Request::new(Asks::Named(asks))).expect("an answer");
        let order = "refused: RUN-ORDER decisions spend and topup read each other on 2026-02-05";
        let lines = answer.lines();
        assert_eq!(lines.first(), Some(&format!("loose: {order}")), "{lines:?}");
        assert!(
            lines.iter().any(|l| *l == format!("el: {order}")),
            "{lines:?}"
        );
    }
}
