//! The walk's agenda: the acts the plan pass hands the day walk, each at its point, and the days the walk steps through.
//! Spec: E §4.1 (what the plan pass hands the walk; the movement order, each term typed, which
//! ranks a day's plans; tally 2.1.6), §4.2 (the day walk's steps; a hole movement, and a
//! refused one reaching as it does, L §8.4; a decision's day), §4.3 (a projection's walk: the movements dated t
//! through d, those that depend on its holder left out and counted, no reading, observation or
//! adjustment; read at a decision's point, only the decisions after it that day), §5 step 5
//! (`HOLE-CHANGED` from the day of an adjustment nothing settles); §6.1 (each observed leg on
//! its day before K, in the day's first batch; `WARN-LEGS`); L §5.19 (readings: an `opening`
//! at `t start` anchors with t before K or t = S, a `stated` one before K anchors or pads;
//! the modelled past: an instance on a covered day before K, or one an observation fulfils, is
//! not booked, and one nothing fulfils marks its places `open`); P §2.2 (the event-driven walk steps only days that hold
//! an act, N-1 to N-14; the oracle steps every day).
//! Twin: engine/tools/refcore/walk.py (`Walk.prepare`, `plan_key`, `sched_leg`, `sched_hole`,
//! `sched_leg_hole`, `sched_refused`, `sched_adj`, `note_leftout`, `count_leftout`,
//! `dec_hole`, `ending`, and `proj_walk`'s first day) and moves.py (`readings`).
//! Invariants: every act is scheduled on its day at its point before the walk reaches that
//! day; the event walk steps the days with an act, the run's first day and each lifespan's
//! last; the oracle steps every day from S to E through the same steps; a projection's walk
//! books nothing of a move its holder's dependents hold.
//! PERF: P8-14 (days stepped: the event walk opens only the run's first day, the days with an
//! act and each lifespan's last, where refcore steps every day; X's replay steps 926 days in
//! 21 walks); plain twin: crates/tally/src/oracle.rs (`Days::Every`, every day from S to E
//! through the same step functions); the equivalence test: the walk oracle in the CLI's
//! tests, `crates/tally-cli/tests/walk_oracle.rs` (the kernel corpus and X's three S checks,
//! the same bytes under both).

use std::collections::{BTreeMap, BTreeSet};

use super::plan::{Amount, Dated, End, Exists, Inst, Order, Status};
use super::point::{At, Step};
use super::state::{Progress, Walk, WalkObs};
use crate::Days;
use crate::base::Refusal;
use crate::book::Seen;
use crate::eval::{Collected, Engine, Entry};
use crate::hole::Hole;
use crate::model::{KeyVal, MoveInfo, Reality, Value};
use crate::num::Num;
use crate::syntax::{Edge, Stop};
use crate::time::Day;

/// A movement the walk books: a plan instance, or a decision's run, by its place in the
/// walk's lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mv {
    /// A plan instance.
    Plan(usize),
    /// A decision's run.
    Run(usize),
}

/// A reading the cut sees, as the walk applies it (refcore's `readings` entries). L §5.19.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reading {
    /// The input: `opening` or `stated`.
    pub input: String,
    /// The place.
    pub place: String,
    /// Its unit.
    pub unit: String,
    /// Its value, or the hole it states.
    pub value: Result<Num, Hole>,
    /// What reading it met: its version, its `assumed`.
    pub coll: Collected,
    /// The version that states it.
    pub version: String,
    /// Its instant as text, `2026-09-30 end` (refcore's `instant_text`).
    pub instant: String,
}

/// The places a decision instance whose day is a hole makes holes, or none for every place
/// (refcore's `dec_hole`). E §4.2 Holes in the walk.
type HolePlaces = Option<BTreeSet<String>>;

/// One act of the walk on a day (refcore's scheduled acts).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Act {
    /// A reading at the start of the day (E §4.2 step 0).
    Read(Box<Reading>),
    /// A booked leg (E §4.2 step 2, and a decision's leg with its own later day).
    Leg(Mv, usize),
    /// A leg of a movement whose existence is a hole, with that hole (E §4.2 Holes).
    LegHole(Mv, usize, Option<Hole>),
    /// A leg of a refused instance (E §4.2 step 5).
    LegBad(Mv, usize, Refusal),
    /// A leg whose day is a hole: a hole no reading answers, from the walk's start.
    Perm(Mv, usize, Hole),
    /// An adjustment nothing settles: its leg's places are `HOLE-CHANGED` from its day.
    Changed(String, String, Hole, Collected),
    /// An adjustment whose change is refused: the place, in every unit, is refused from its
    /// day, by the adjustment (E §5 step 5, tally 2.1.2).
    ChangeRefused(String, Refusal, String),
    /// A hole at one place in every unit from the walk's start, by a decision instance whose
    /// day is a hole, with its id and what it met (refcore's `placehole`).
    PlaceHole(String, Hole, String, Collected),
    /// A hole that holds for every place.
    GlobalHole(Hole),
    /// A refusal that holds for every place.
    GlobalBad(Refusal),
    /// The decisions of a move due that day (E §4.2 step 3).
    Decide(String, Day),
    /// An observed leg, by the observation's place in the walk's list and the leg's (E §4.2
    /// step 1).
    Obs(usize, usize),
    /// A register entry at a place, with what its movement met: a modelled-past instance
    /// nothing fulfils is `open` (L §5.19).
    Mark(String, String, Entry, Collected),
}

impl Walk {
    /// Puts an act on its day's agenda. E §4.2.
    pub(crate) fn at(&mut self, at: At, act: Act) {
        self.agenda
            .entry(at.day)
            .or_insert_with(Vec::new)
            .push((at, act));
    }

    /// The next day to walk after the last one walked, by the day source: the event walk's
    /// next day with an act (the run's first day and each lifespan's last are acts too), or
    /// the oracle's next day (P §2.2).
    #[must_use]
    pub fn next_day(&self, days: Days, endings: &BTreeMap<Day, Vec<String>>) -> Option<Day> {
        if days == Days::Every {
            return crate::oracle::every_day(self.done, self.start, self.end);
        }
        let first = match self.done {
            None => return Some(self.start),
            Some(d) => d.next()?,
        };
        let act = self.agenda.range(first..).next().map(|(d, _)| *d);
        let end = endings.range(first..).next().map(|(d, _)| *d);
        let next = match (act, end) {
            (Some(a), Some(b)) => a.min(b),
            (Some(a), None) => a,
            (None, Some(b)) => b,
            (None, None) => return None,
        };
        (next <= self.end).then_some(next)
    }

    /// Whether the walk has walked every day through `last` that holds an act: no day is
    /// open and its next day is later (refcore's `run_to`, which stops at its day). E §4.3.
    #[must_use]
    pub fn paused_after(&self, days: Days, last: Option<Day>) -> bool {
        let Some(last) = last else {
            return false;
        };
        self.today.is_none() && self.next_day(days, &self.endings).is_some_and(|d| d > last)
    }
}

impl Engine {
    /// The plan pass of a walk and its agenda: the readings the cut sees, every plan
    /// instance's legs by day and status, the adjustments nothing settles, the decision
    /// instances by day (refcore's `prepare`). E §4.1, §4.2, §5 step 5.
    ///
    /// # Errors
    ///
    /// The refusals the plan pass meets; `NotBuilt` for a reading after a movement or at a
    /// day with no edge. A written order never names a plan here: load refuses `before` a
    /// plan and drops `after` one, as plans come first (L §5.14.2).
    pub(crate) fn plan_walk(&mut self, m: &Reality, id: usize) -> Result<(), Stop> {
        let (cut, main, world) = {
            let walk = self.walk(id)?;
            (walk.cut, walk.proj.is_none(), walk.world)
        };
        let readings = if main {
            crate::book::readings(m, (cut, world))?
        } else {
            Vec::new()
        };
        let mut observed = Vec::new();
        let mut legs_warned = Vec::new();
        if main {
            legs_warned = self.disagreeing_links(m, cut)?;
            for obs in self.observations(m, cut)?.to_vec() {
                let decision_day = self.decision_fulfilled(m, &obs, cut)?;
                observed.push(WalkObs {
                    obs,
                    err: None,
                    decision_day,
                });
            }
        }
        let mut plans = Vec::new();
        let mut decisions = Vec::new();
        let mut hole_places = BTreeMap::new();
        for mv in m.moves_in(world) {
            let insts = self.plan_list(m, mv, cut)?.to_vec();
            if mv.decision {
                if !left_out_of(self.walk(id)?, &mv.id) {
                    hole_places.extend(self.hole_days(m, mv, &insts, cut)?);
                }
                decisions.push((mv.id.clone(), mv.stratum, insts));
            } else {
                plans.extend(insts);
            }
        }
        plans.sort_by(|a, b| a.order.cmp(&b.order));
        let orders = movement_orders(&plans, &decisions);
        let adjustments = if main {
            self.all_adjustments(m, cut)?
        } else {
            Vec::new()
        };
        for wo in &mut observed {
            wo.err = self.obs_err(m, &wo.obs, cut)?;
        }
        let walk = self.walk_mut(id)?;
        walk.orders = orders;
        for w in legs_warned {
            if !walk.warnings.contains(&w) {
                walk.warnings.push(w);
            }
        }
        schedule_readings(walk, &readings)?;
        schedule_observations(walk, observed)?;
        for inst in plans {
            if left_out_of(walk, &inst.move_id) {
                if inst.status != Status::None {
                    count_left_out(walk, &inst)?;
                }
                continue;
            }
            schedule_plan(walk, inst)?;
        }
        for (rank, (move_id, stratum, insts)) in ranked3(decisions).into_iter().enumerate() {
            for inst in insts {
                if left_out_of(walk, &move_id) {
                    count_left_out(walk, &inst)?;
                    continue;
                }
                let places = hole_places.remove(&inst.id);
                schedule_decision(m, walk, (&move_id, stratum, rank), (inst, places))?;
            }
        }
        for adj in adjustments {
            schedule_adjustment(m, walk, &adj)?;
        }
        if let Some(p) = walk.proj.as_mut() {
            // sorted, so a read finds its window's by halving (`left_out_lines`)
            for days in p.counted.values_mut() {
                days.sort();
            }
        }
        walk.endings = endings(m, walk);
        after_the_point(walk);
        walk.progress = Progress::Walking;
        Ok(())
    }

    /// The decision instances of `insts` whose day is a hole, each with the places its legs
    /// name (refcore's `prepare`, its decisions). E §4.2 Holes in the walk, L §5.14.
    fn hole_days(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        insts: &[Inst],
        cut: Day,
    ) -> Result<Vec<(String, HolePlaces)>, Stop> {
        let mut out = Vec::new();
        for inst in insts {
            if inst.exists != Exists::No && matches!(inst.day, Some(Dated::Hole(_))) {
                out.push((inst.id.clone(), self.hole_places(m, mv, inst, cut)?));
            }
        }
        Ok(out)
    }

    /// The places the legs of decision instance `inst`, whose day is a hole, name, read at the
    /// cut as its day is: each a hole from the walk's start, the others standing; a leg whose
    /// place is itself a hole books nothing known. None where they can't be read here: a key
    /// the hole left unbound, a place that is refused or no thing (refcore's `dec_hole`).
    /// E §4.2 Holes in the walk, L §5.14.
    fn hole_places(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &Inst,
        cut: Day,
    ) -> Result<HolePlaces, Stop> {
        if mv.decl.binds.iter().any(|b| inst.env.get(&b.var).is_none()) {
            return Ok(None);
        }
        let ctx = self.timeline_ctx(None, cut);
        let mut places = BTreeSet::new();
        for leg in &mv.decl.legs {
            for end in [&leg.from, &leg.to] {
                match self.read_or_refused(m, end, &inst.env, ctx)? {
                    Some(Value::Key(KeyVal::Thing(p))) => {
                        places.insert(p);
                    }
                    Some(Value::Hole(_)) => {}
                    Some(_) | None => return Ok(None),
                }
            }
        }
        Ok(Some(places))
    }
}

/// The movement orders of a walk's plan and decision instances, sorted in the typed movement
/// order, each once since an instance is its move and keys: the ranks of the day's plans
/// (refcore's `plan_key`; tally 2.1.6, #57). E §4.1.
fn movement_orders(plans: &[Inst], decisions: &[(String, u32, Vec<Inst>)]) -> Vec<Order> {
    let decided = decisions.iter().flat_map(|(_, _, insts)| insts);
    let mut orders: Vec<Order> = plans
        .iter()
        .chain(decided)
        .map(|i| i.order.clone())
        .collect();
    orders.sort();
    orders
}

/// Whether a projection's walk leaves out the movements of move `move_id`: they depend on
/// the projection's holder (L §5.14.1); a main walk leaves out none. E §4.3 step 3.
fn left_out_of(walk: &Walk, move_id: &str) -> bool {
    walk.proj
        .as_ref()
        .is_some_and(|p| p.left_out.contains(move_id))
}

/// An instance a projection's walk leaves out, kept by its day for the `left out:` line of
/// a read whose window holds it (refcore's `count_leftout`). E §4.3 step 3, L §8.2.
fn count_left_out(walk: &mut Walk, inst: &Inst) -> Result<(), Stop> {
    let p = walk
        .proj
        .as_mut()
        .ok_or_else(|| super::plan::invariant("an instance left out of a main walk"))?;
    left_out_day(&mut p.counted, inst);
    Ok(())
}

/// A left-out instance's day kept among its move's, for the `left out:` line of a read whose
/// window holds it; none for an instance with no day (refcore's `count_leftout`). Shared by
/// the projection's walk and fence F5 (`lows.rs`). E §4.3 step 3, L §8.2.
pub(super) fn left_out_day(counted: &mut BTreeMap<String, Vec<Day>>, inst: &Inst) {
    if let Some(day) = inst.day.as_ref().and_then(Dated::day) {
        counted
            .entry(inst.move_id.clone())
            .or_insert_with(Vec::new)
            .push(day);
    }
}

/// The `left out:` line of each left-out move with an instance dated t through `last`, each
/// move's days sorted, so a read finds its own by halving (refcore's `proj_regs`). Shared by
/// the projection's walk and fence F5. L §5.14.1, §8.2; E §4.3 step 3; P §2.3.
///
/// # Errors
///
/// `ENGINE-INVARIANT` where a window's days can't be taken from a move's.
pub(super) fn left_out_lines(
    counted: &BTreeMap<String, Vec<Day>>,
    (t, last): (Day, Day),
) -> Result<Vec<String>, Stop> {
    let mut lines = Vec::new();
    for (mid, days) in counted {
        let from = days.partition_point(|d| *d < t);
        let to = days.partition_point(|d| *d <= last);
        // none where t is past `last`, as no day lies in t through `last` then
        let inside = days
            .get(from..to.max(from))
            .ok_or_else(|| super::plan::invariant("a left-out move's days in a window"))?;
        if let Some(first) = inside.first() {
            lines.push(format!(
                "{mid}, {} instances, first {}",
                inside.len(),
                first.text()
            ));
        }
    }
    Ok(lines)
}

/// A projection's walk read at a decision's point: on its first day only the decisions after
/// that point are left, since what the day booked before it is in the start (refcore's
/// `proj_walk`). L §5.14.1, E §4.3 step 1.
fn after_the_point(walk: &mut Walk) {
    let Some(p) = &walk.proj else {
        return;
    };
    let (point, first) = (p.point, walk.start);
    if point <= At::start(first) {
        return;
    }
    if let Some(acts) = walk.agenda.get_mut(&first) {
        acts.retain(|(at, act)| matches!(act, Act::Decide(..)) && *at > point);
    }
}

/// The decision moves by id (refcore's decision day keys). E §4.2 step 3.
fn ranked3(mut decisions: Vec<(String, u32, Vec<Inst>)>) -> Vec<(String, u32, Vec<Inst>)> {
    decisions.sort_by(|a, b| a.0.cmp(&b.0));
    decisions
}

/// Puts each reading on its day: an opening at the run's start, or before it and before the
/// cut, at the start of the walk; one at another start or end only before the cut (refcore's
/// `prepare`, its readings; a `stated` one at or after the cut changes nothing, so it isn't
/// put on the walk). E §4.2 step 0, L §5.19.
fn schedule_readings(walk: &mut Walk, readings: &[Seen]) -> Result<(), Stop> {
    let (start, end, cut) = (walk.start, walk.end, walk.cut);
    for (rank, seen) in readings.iter().enumerate() {
        let (day, edge) = (&seen.day, &seen.edge);
        let first = walk.first_reading.entry(seen.place.clone()).or_insert(*day);
        *first = (*first).min(*day);
        let Some(reading) = &seen.reading else {
            continue;
        };
        let rank = u32::try_from(rank).map_err(|_| super::plan::invariant("a reading's rank"))?;
        let opening = reading.input == "opening";
        if *day < start || *day > end {
            if opening && *day < start && *day < cut {
                walk.at(At::before(start), Act::Read(Box::new(reading.clone())));
            }
            continue;
        }
        let at_start = *edge == Edge::Start;
        if !(opening && *day == start && at_start) && *day >= cut {
            continue;
        }
        walk.at(
            reading_point(*day, *edge, rank),
            Act::Read(Box::new(reading.clone())),
        );
    }
    Ok(())
}

/// The point of a reading at the start or the end of `day`, `rank` its place among the
/// readings the cut sees (refcore's walk key of a reading). E §4.2 steps 0 and 4.
#[must_use]
pub fn reading_point(day: Day, edge: Edge, rank: u32) -> At {
    let step = match edge {
        Edge::Start => Step::Start,
        Edge::End => Step::End,
    };
    At {
        stratum: 1,
        rank,
        ..At::of(day, step)
    }
}

/// Puts each observed leg dated before the cut on its day, in the day's first batch, ordered
/// by `<id> <leg>` (refcore's `prepare`, its observations; one dated at or after the cut books
/// nothing, so it isn't put on the walk). E §4.2 step 1, L §5.19.
fn schedule_observations(walk: &mut Walk, observed: Vec<WalkObs>) -> Result<(), Stop> {
    let mut texts: BTreeSet<String> = BTreeSet::new();
    for wo in &observed {
        texts.extend(wo.obs.legs.iter().map(|lg| leg_text(&wo.obs.id, &lg.name)));
    }
    walk.obs_texts = texts.into_iter().collect();
    for wo in observed {
        let ix = walk.obs.len();
        for (li, lg) in wo.obs.legs.iter().enumerate() {
            let Some(d) = lg.day.day() else {
                continue;
            };
            if d < walk.start || d > walk.end || d >= walk.cut {
                continue;
            }
            let rank = walk.obs_rank(&leg_text(&wo.obs.id, &lg.name))?;
            let at = At {
                rank,
                ..At::of(d, Step::Observations)
            };
            walk.at(at, Act::Obs(ix, li));
        }
        walk.obs.push(wo);
    }
    Ok(())
}

/// An observed leg's text among a day's observations, `<id> <leg>` (refcore's walk key).
/// E §4.2 step 1.
fn leg_text(id: &str, leg: &str) -> String {
    format!("{id} {leg}")
}

/// A modelled-past instance: kept for the ledger, and one nothing fulfils marks its places
/// `open` from its legs' days (refcore's `prepare`, its modelled plans). E §4.1 step 3, L §5.19.
fn schedule_modelled(walk: &mut Walk, inst: Inst) -> Result<(), Stop> {
    if inst.obs.is_empty() {
        let rank = walk.plan_rank(&inst.order)?;
        for lg in &inst.legs {
            let Dated::Day(d) = lg.day else {
                continue;
            };
            if d < walk.start {
                continue;
            }
            let unit = lg
                .unit
                .clone()
                .unwrap_or_else(|| super::state::ALL.to_owned());
            for place in [&lg.from, &lg.to].into_iter().filter_map(End::place) {
                let entry = Entry {
                    word: "open",
                    text: inst.id.clone(),
                    figure: String::new(),
                };
                let act = Act::Mark(place.to_owned(), unit.clone(), entry, inst.coll.clone());
                walk.at(
                    At {
                        rank,
                        ..At::of(d, Step::Plans)
                    },
                    act,
                );
            }
        }
    }
    walk.modelled.push(inst);
    Ok(())
}

/// Puts a plan instance's legs on their days by its status: left out, uncertain, refused, a
/// hole, modelled past, or booked, each at its rank in the typed movement order (refcore's
/// `prepare`, its plans; tally 2.1.6, #57). A refused instance refuses the places its legs
/// give from its day, and every place where a leg's place can't be read (L §8.4). E §4.1
/// step 3, §4.2.
fn schedule_plan(walk: &mut Walk, inst: Inst) -> Result<(), Stop> {
    let rank = walk.plan_rank(&inst.order)?;
    match inst.status {
        Status::LeftOut => return note_left_out(walk, &inst),
        Status::Modelled => return schedule_modelled(walk, inst),
        Status::None | Status::Decision | Status::Open => return Ok(()),
        Status::Uncertain => {
            walk.uncertain.push(inst);
            return Ok(());
        }
        Status::Refused | Status::Hole | Status::Past | Status::Plan => {}
    }
    let first = inst.day.as_ref().and_then(Dated::day);
    if walk.proj.is_some() && first.is_some_and(|d| d < walk.start) {
        // a projection counts the movements dated t through d (E §4.3 step 2)
        return Ok(());
    }
    let ix = walk.insts.len();
    let status = inst.status;
    walk.insts.push(inst.clone());
    let mv = Mv::Plan(ix);
    let point = |day: Day| At {
        rank,
        ..At::of(day, Step::Plans)
    };
    let start = walk.start;
    match status {
        Status::Refused => {
            let day = inst
                .day
                .as_ref()
                .and_then(Dated::day)
                .map_or(start, |d| d.max(start));
            let whole = inst
                .legs
                .iter()
                .all(|lg| lg.from.place().is_some() && lg.to.place().is_some());
            let err = inst
                .err
                .clone()
                .ok_or_else(|| super::plan::invariant("a refused instance with no refusal"))?;
            if whole && !inst.legs.is_empty() {
                for leg in 0..inst.legs.len() {
                    walk.at(point(day), Act::LegBad(mv, leg, err.clone()));
                }
            } else {
                walk.at(At::before(day), Act::GlobalBad(err));
            }
        }
        Status::Hole => {
            if let Some(Dated::Hole(h)) = &inst.day {
                for leg in 0..inst.legs.len() {
                    walk.at(At::before(start), Act::Perm(mv, leg, h.clone()));
                }
                if inst.legs.is_empty() {
                    walk.at(At::before(start), Act::GlobalHole(h.clone()));
                }
                return Ok(());
            }
            let qs = match &inst.exists {
                Exists::Unknown(h) => Some(h.clone()),
                Exists::Yes | Exists::No => None,
            };
            for (leg, lg) in inst.legs.iter().enumerate() {
                let day = lg
                    .day
                    .day()
                    .or_else(|| inst.day.as_ref().and_then(Dated::day))
                    .map_or(start, |d| d.max(start));
                walk.at(point(day), Act::LegHole(mv, leg, qs.clone()));
            }
        }
        Status::Past | Status::Plan => {
            for (leg, lg) in inst.legs.iter().enumerate() {
                match &lg.day {
                    Dated::Day(d) if *d >= start => walk.at(point(*d), Act::Leg(mv, leg)),
                    Dated::Day(_) => {}
                    Dated::Hole(h) => walk.at(At::before(start), Act::Perm(mv, leg, h.clone())),
                }
            }
        }
        Status::LeftOut
        | Status::None
        | Status::Decision
        | Status::Open
        | Status::Uncertain
        | Status::Modelled => {}
    }
    Ok(())
}

/// Puts a decision instance on its day, at its move's stratum; one dated outside the run is
/// left out, and so is one of a move a scenario declares dated before the scenario's `from`,
/// which books nothing there (L §6.4); one refused before its point (its key's `where`) is
/// put on its day as any instance is, so its refusal takes its place in the day's movement
/// order (E §4.1; L §8.4, tally 2.1.6), the scenario's rule first; one whose day is a hole
/// makes `places` holes from the walk's start, every place where they can't be read
/// (refcore's `prepare`, its decisions, and `dec_hole`). E §4.2 step 3, Holes in the walk;
/// L §6.4.
fn schedule_decision(
    m: &Reality,
    walk: &mut Walk,
    (move_id, stratum, rank): (&str, u32, usize),
    (inst, places): (Inst, Option<HolePlaces>),
) -> Result<(), Stop> {
    if inst.exists == Exists::No {
        return Ok(());
    }
    let rank = u32::try_from(rank).map_err(|_| super::plan::invariant("a move's rank"))?;
    match inst.day.clone() {
        Some(Dated::Day(d)) => {
            if super::plan::scenario_from(m, move_id).is_some_and(|from| d < from) {
                // a scenario's move books nothing before its from (L §6.4): left out
                if walk.proj.is_none() {
                    note_left_out(walk, &inst)?;
                }
                return Ok(());
            }
            if d < walk.start || d > walk.end {
                if walk.proj.is_none() && (d < m.start || d > m.end) {
                    note_left_out(walk, &inst)?;
                }
                return Ok(());
            }
            let key = (move_id.to_owned(), d);
            if !walk.due.contains_key(&key) {
                let at = At {
                    stratum,
                    rank,
                    ..At::of(d, Step::Decisions)
                };
                walk.at(at, Act::Decide(move_id.to_owned(), d));
            }
            walk.due.entry(key).or_insert_with(Vec::new).push(inst);
        }
        Some(Dated::Hole(h)) => {
            let at = At::before(walk.start);
            let places = places.ok_or_else(|| {
                super::plan::invariant("a decision's hole day with no places read")
            })?;
            let Some(places) = places else {
                walk.at(at, Act::GlobalHole(h));
                return Ok(());
            };
            for place in places {
                let act = Act::PlaceHole(place, h.clone(), inst.id.clone(), inst.coll.clone());
                walk.at(at, act);
            }
        }
        None => {
            let err = inst
                .err
                .clone()
                .ok_or_else(|| super::plan::invariant("an instance with no day and no refusal"))?;
            let start = walk.start;
            walk.at(At::before(start), Act::GlobalBad(err));
        }
    }
    Ok(())
}

/// An adjustment on the walk: `HOLE-CHANGED` at its leg's places from its day when nothing
/// settles it, and its remainder in the register of every read of them (refcore's
/// `sched_adj`). E §5 step 5, L §8.2.
fn schedule_adjustment(
    m: &Reality,
    walk: &mut Walk,
    adj: &crate::book::Adjustment,
) -> Result<(), Stop> {
    let day = adj.day.max(walk.start);
    if let Amount::Refused(err) = &adj.delta {
        // tally 2.1.2: a refused change refuses the adjusted leg's places from its day,
        // where an unsettled one makes them HOLE-CHANGED (L §5.14.3, §8.4)
        for place in [&adj.leg.from, &adj.leg.to]
            .into_iter()
            .filter_map(End::place)
        {
            let act = Act::ChangeRefused(place.to_owned(), err.clone(), adj.id.clone());
            walk.at(At::before(day), act);
        }
        return Ok(());
    }
    let text = adj.remainder_text(m)?;
    let mut places: Vec<&str> = Vec::new();
    let all = [&adj.leg.from, &adj.leg.to]
        .into_iter()
        .chain(adj.new_ends.iter());
    for place in all.filter_map(End::place) {
        if !places.contains(&place) {
            places.push(place);
        }
    }
    let unit = adj
        .leg
        .unit
        .clone()
        .unwrap_or_else(|| super::state::ALL.to_owned());
    for place in places.iter().filter(|_| adj.settled.is_empty()) {
        let act = Act::Changed(
            (*place).to_owned(),
            unit.clone(),
            adj.question(),
            adj.coll.clone(),
        );
        walk.at(At::before(day), act);
    }
    let Some(text) = text else {
        return Ok(());
    };
    for place in places {
        let entry = Entry {
            word: "adjustment",
            text: text.clone(),
            figure: String::new(),
        };
        walk.adj_regs
            .entry(place.to_owned())
            .or_insert_with(Vec::new)
            .push((adj.day, entry));
    }
    Ok(())
}

/// An instance left out of the walk, counted for its move's `left out:` line and its places
/// (refcore's `note_leftout`). E §4.1 step 3, L §8.2.
fn note_left_out(walk: &mut Walk, inst: &Inst) -> Result<(), Stop> {
    one_more_left_out(&mut walk.left_out, inst)?;
    for leg in &inst.legs {
        for place in [&leg.from, &leg.to].into_iter().filter_map(End::place) {
            walk.left_out_at
                .entry(place.to_owned())
                .or_insert_with(BTreeSet::new)
                .insert(inst.move_id.clone());
        }
    }
    Ok(())
}

/// One more instance of a move left out of the run, counted with its first day, for its
/// `left out:` line (refcore's `note_leftout`). Shared by the walk and fence F5 (`lows.rs`).
/// L §8.2, E §4.1 step 3.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a count past what a `u32` holds.
pub(super) fn one_more_left_out(
    left_out: &mut BTreeMap<String, (u32, Option<Day>)>,
    inst: &Inst,
) -> Result<(), Stop> {
    let day = inst.day.as_ref().and_then(Dated::day);
    let entry = left_out.entry(inst.move_id.clone()).or_insert((0, None));
    entry.0 = entry
        .0
        .checked_add(1)
        .ok_or_else(|| super::plan::invariant("a count of instances"))?;
    entry.1 = match (entry.1, day) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (None, b) => b,
        (a, None) => a,
    };
    Ok(())
}

/// Each place's last day of its lifespan inside the walk, under its cut (refcore's `ending`).
/// E §4.2 step 5.
#[must_use]
pub fn endings(m: &Reality, walk: &Walk) -> BTreeMap<Day, Vec<String>> {
    let mut out: BTreeMap<Day, Vec<String>> = BTreeMap::new();
    for (_, thing) in m.things.entries().filter(|(_, t)| m.is_place(t)) {
        let Some(end) = m
            .lifespan(thing, (walk.cut, walk.world))
            .and_then(|sp| sp.end())
        else {
            continue;
        };
        let Some(last) = end.previous() else {
            continue;
        };
        out.entry(last)
            .or_insert_with(Vec::new)
            .push(thing.name.clone());
    }
    for names in out.values_mut() {
        names.sort();
    }
    out
}

#[cfg(test)]
mod tests {
    use crate::Days;
    use crate::api::{Asks, Given, Pin, Request, run};

    /// The printed lines of ask `ask` over `files`, under the event walk.
    fn answer(files: &[(&str, &str)], ask: &str) -> Vec<String> {
        let given = files
            .iter()
            .map(|(path, text)| Given {
                path: (*path).to_owned(),
                bytes: Some(text.as_bytes().to_vec()),
            })
            .collect();
        let request = Request {
            asks: Asks::Named(vec![ask.to_owned()]),
            pin: Pin::Canonical,
            days: Days::Events,
        };
        run(given, &request).expect("an answer").lines().to_vec()
    }

    /// A decision dated 3 January whose leg is dated 10 January, and a plan dated 10 January,
    /// each moving into `gone` after its lifespan ends: both refuse `src` that day.
    const LATER_LEG: &str = r#"model later_leg
  tally 2
  on 2025-12-31
  start 2025-01-01
  end 2025-12-31

source card "a card"

unit DUK: currency minor 0.01

input amt: money DUK

input d0: date

input d1: date

place src: accounts DUK lives 2025-01-01..

place gone: accounts DUK lives 2025-01-01..2025-01-05

move a: transfers
  on d0
  leg cash from src to gone amount amt

move sweep: transfers
  on d1
  reads src
  leg cash from src to gone amount min(amt, bal(src)) on d0

ask q: bal(src, 2025-01-31)

version facts
  cite card
  governs always
  known 2025-01-01
  amt = 1.00 DUK
  d0 = 2025-01-10
  d1 = 2025-01-03
  opening[src] = 500.00 DUK at 2025-01-01 start
  opening[gone] = 0 DUK at 2025-01-01 start
"#;

    #[test]
    fn a_decisions_later_leg_takes_its_place_among_the_days_plans_in_the_movement_order() {
        // E §4.1 (tally 2.1.6): the decision's instance is dated 3 January, before the plan's
        // 10 January, so its leg on 10 January is booked first there and src keeps its
        // refusal; ranked as the plans are scheduled, the plan's would be first
        assert_eq!(
            answer(&[("later_leg.tally", LATER_LEG)], "q"),
            [
                "q: refused: RUN-LIFESPAN sweep.cash moves into or out of gone outside its lifespan",
                "  cut 2025-12-31"
            ],
            "the decision's refusal, first in the typed movement order"
        );
    }

    /// The base of a scenario whose two decisions are dated on its `from` and after it.
    const BASE: &str = r#"model hh
  tally 2
  on 2025-12-31
  start 2025-12-30
  end 2026-01-05

source s "a card"

unit PLN: currency minor 0.01

place konto: accounts PLN lives 2025-01-01..

place world: outside lives 1900-01-01..

input from_day: date

input late_day: date

input top_amt: money PLN

input lift_amt: money PLN

ask q: bal(konto, 2026-01-05)

version v
  cite s
  governs always
  known 2025-12-01
  from_day = 2026-01-01
  late_day = 2026-01-03
  top_amt = 1.00 PLN
  lift_amt = 2.00 PLN
  opening[konto] = 10.00 PLN at 2025-12-30 start
"#;

    const SCENARIO: &str = r"scenario boost over hh
  tally 2
  from 2026-01-01

move top: transfers
  on from_day
  reads konto
  leg cash from world to konto amount min(top_amt, bal(konto))

move lift: transfers
  on late_day
  reads konto
  leg cash from world to konto amount min(lift_amt, bal(konto))

ask q_boost: bal(konto, 2026-01-05) in boost
";

    #[test]
    fn a_scenarios_decision_books_on_its_from_and_after_it() {
        // L §6.4: a move a scenario declares books nothing before its from, and books on the
        // from itself and after it: 10.00 + 1.00 (on the from) + 2.00 (two days after)
        assert_eq!(
            answer(
                &[("model.tally", BASE), ("scen.tally", SCENARIO)],
                "q_boost"
            ),
            ["q_boost: 13.00 PLN", "  world: boost", "  cut 2025-12-31"],
            "both decisions booked"
        );
    }
}
