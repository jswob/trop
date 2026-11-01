//! The walk's agenda: the acts the plan pass hands the day walk, each at its point, and the days the walk steps through.
//! Spec: E §4.1 (what the plan pass hands the walk; the movement order), §4.2 (the day walk's
//! steps; a hole movement; a decision's day), §5 step 5 (`HOLE-CHANGED` from an adjustment's
//! day); L §5.19 (readings: an `opening` at `t start` anchors with t before K or t = S);
//! P §2.2 (the event-driven walk steps only days that hold an act, N-1 to N-14; the oracle
//! steps every day).
//! Twin: engine/tools/refcore/walk.py (`Walk.prepare`, `sched_leg`, `sched_hole`,
//! `sched_leg_hole`, `sched_refused`, `sched_adj`, `note_leftout`, `dec_hole`, `ending`) and
//! moves.py (`readings`).
//! Invariants: every act is scheduled on its day at its point before the walk reaches that
//! day; the event walk steps the days with an act, the run's first day and each lifespan's
//! last; the oracle steps every day from S to E through the same steps.

use std::collections::{BTreeMap, BTreeSet};

use super::plan::{Amount, Dated, End, Exists, Inst, Status};
use super::point::{At, Step};
use super::state::{Progress, Walk};
use crate::Days;
use crate::base::Refusal;
use crate::eval::{Collected, Engine, Entry};
use crate::hole::Hole;
use crate::model::{Reality, RowInstant, Ty, Unit};
use crate::num::Num;
use crate::read::row_known;
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
    /// The input: `opening`.
    pub input: String,
    /// The place.
    pub place: String,
    /// Its unit.
    pub unit: String,
    /// Its value, or the hole it states.
    pub value: Result<Num, Hole>,
    /// What reading it met: its version, its `assumed`.
    pub coll: Collected,
}

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
    /// A hole that holds for every place.
    GlobalHole(Hole),
    /// A refusal that holds for every place.
    GlobalBad(Refusal),
    /// The decisions of a move due that day (E §4.2 step 3).
    Decide(String, Day),
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
}

impl Engine {
    /// The plan pass of a walk and its agenda: the readings the cut sees, every plan
    /// instance's legs by day and status, the adjustments nothing settles, the decision
    /// instances by day (refcore's `prepare`). E §4.1, §4.2, §5 step 5.
    ///
    /// # Errors
    ///
    /// The refusals the plan pass meets; `NotBuilt` for observations, coverage and `stated`
    /// readings (G8) and for a written order with a plan.
    pub(crate) fn plan_walk(&mut self, m: &Reality, id: usize) -> Result<(), Stop> {
        let cut = self.walk(id)?.cut;
        not_yet(m)?;
        let readings = readings(m, cut)?;
        let mut plans = Vec::new();
        let mut decisions = Vec::new();
        for mv in sorted_moves(m) {
            let insts = self.move_instances(m, mv, cut)?;
            if mv.decision {
                decisions.push((mv.id.clone(), mv.stratum, insts));
            } else {
                plans.extend(insts);
            }
        }
        plans.sort_by(|a, b| a.order.cmp(&b.order));
        let mut adjustments = Vec::new();
        for mv in sorted_moves(m).into_iter().filter(|mv| !mv.decision) {
            adjustments.extend(self.adjust_move(m, mv, cut)?);
        }
        let walk = self.walk_mut(id)?;
        let texts: BTreeSet<String> = plans.iter().map(|i| i.order.text()).collect();
        walk.order_texts = texts.into_iter().collect();
        schedule_readings(walk, &readings)?;
        for inst in plans {
            schedule_plan(walk, inst)?;
        }
        for (rank, (move_id, stratum, insts)) in ranked3(decisions).into_iter().enumerate() {
            for inst in insts {
                schedule_decision(m, walk, (&move_id, stratum, rank), inst)?;
            }
        }
        for adj in adjustments {
            schedule_adjustment(m, walk, &adj)?;
        }
        walk.endings = endings(m, walk);
        walk.progress = Progress::Walking;
        Ok(())
    }
}

/// What S doesn't walk yet: observations, coverage and `stated` readings (G8). E §4.1.
fn not_yet(m: &Reality) -> Result<(), Stop> {
    if m.inputs.entries().any(|(_, i)| !i.observations.is_empty()) {
        return Err(Stop::NotBuilt("observations in the walk (G8)"));
    }
    if m.versions.entries().any(|(_, v)| !v.covers.is_empty()) {
        return Err(Stop::NotBuilt("coverage in the walk (G8)"));
    }
    if m.inputs.get("stated").is_some_and(|i| !i.rows.is_empty()) {
        return Err(Stop::NotBuilt("stated readings in the walk (G8)"));
    }
    if m.graph
        .written_order
        .iter()
        .any(|(a, b)| is_plan(m, a) || is_plan(m, b))
    {
        return Err(Stop::NotBuilt("a written order with a plan"));
    }
    Ok(())
}

/// The moves, by id. E §4.1.
fn sorted_moves(m: &Reality) -> Vec<&crate::model::MoveInfo> {
    let mut out: Vec<&crate::model::MoveInfo> = m.moves.entries().map(|(_, mv)| mv).collect();
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

/// The decision moves by id (refcore's decision day keys). E §4.2 step 3.
fn ranked3(mut decisions: Vec<(String, u32, Vec<Inst>)>) -> Vec<(String, u32, Vec<Inst>)> {
    decisions.sort_by(|a, b| a.0.cmp(&b.0));
    decisions
}

/// A reading the cut sees: its day, its edge, its place, and what it states (none for a row
/// whose value is `none`).
type Seen = (Day, Edge, String, Option<Reading>);

/// The readings the cut sees: each `opening` row known by the cut, a restated one replaced
/// by the latest, sorted by place, instant and input (refcore's `readings`). L §5.19, §4.6.
fn readings(m: &Reality, cut: Day) -> Result<Vec<Seen>, Stop> {
    let Some(input) = m.inputs.get("opening") else {
        return Ok(Vec::new());
    };
    let mut keep: BTreeMap<(String, Day, bool), (Day, &crate::model::Row)> = BTreeMap::new();
    for row in &input.rows {
        let known = row_known(m, row)?;
        if known > cut {
            continue;
        }
        let Some(crate::model::Pat::Key(crate::model::KeyVal::Thing(place))) = row.pattern.first()
        else {
            return Err(Stop::NotBuilt("a reading of a place not named"));
        };
        let Some(RowInstant::At(day, Some(edge))) = &row.instant else {
            return Err(Stop::NotBuilt(
                "a reading at an instant other than a day's start or end",
            ));
        };
        let key = (place.clone(), *day, *edge == Edge::End);
        if keep.get(&key).is_none_or(|(k, _)| known > *k) {
            keep.insert(key, (known, row));
        }
    }
    let mut out = Vec::new();
    for ((place, day, end), (_, row)) in keep {
        let reading = reading_of(m, &place, row)?;
        out.push((
            day,
            if end { Edge::End } else { Edge::Start },
            place,
            reading,
        ));
    }
    Ok(out)
}

/// A reading's row as the walk applies it: its value in the place's unit, a stated hole, or
/// none for a row whose value is `none` (refcore's `apply_reading`, its value). L §5.19.
fn reading_of(m: &Reality, place: &str, row: &crate::model::Row) -> Result<Option<Reading>, Stop> {
    let unit = m
        .things
        .get(place)
        .and_then(|t| m.place_unit(t))
        .ok_or(Stop::NotBuilt("a reading of a place of unit any"))?;
    let ty = if m.currencies.contains_key(&unit) {
        Ty::Money(unit.clone())
    } else {
        let u = Unit::make(vec![(unit.clone(), 1)]).map_err(crate::eval::num_stop)?;
        Ty::Num(u, crate::model::Shown::Plain)
    };
    let mut coll = Collected::default();
    coll.versions.insert(row.version.clone());
    if let Some(reason) = &row.assumed {
        coll.entries.insert(Entry {
            word: "assumed",
            text: reason.clone(),
            figure: String::new(),
        });
    }
    let figure = format!("opening[{place}]");
    let value = match crate::eval::convert_row(m, &row.value, &ty)? {
        Err(text) => Err(Hole::asked(&text, &figure)),
        Ok(crate::model::Value::None) => return Ok(None),
        Ok(v) => Ok(super::plan::quantity(m, &v)?.0),
    };
    Ok(Some(Reading {
        input: "opening".to_owned(),
        place: place.to_owned(),
        unit,
        value,
        coll,
    }))
}

/// Puts each reading on its day: an opening at the run's start, or before it and before the
/// cut, at the start of the walk; one at another start or end only before the cut (refcore's
/// `prepare`, its readings). E §4.2 step 0, L §5.19.
fn schedule_readings(walk: &mut Walk, readings: &[Seen]) -> Result<(), Stop> {
    let (start, end, cut) = (walk.start, walk.end, walk.cut);
    for (rank, (day, edge, place, reading)) in readings.iter().enumerate() {
        let first = walk.first_reading.entry(place.clone()).or_insert(*day);
        *first = (*first).min(*day);
        let Some(reading) = reading else {
            continue;
        };
        let rank = u32::try_from(rank).map_err(|_| super::plan::invariant("a reading's rank"))?;
        if *day < start || *day > end {
            if *day < start && *day < cut {
                walk.at(At::before(start), Act::Read(Box::new(reading.clone())));
            }
            continue;
        }
        let at_start = *edge == Edge::Start;
        if !(*day == start && at_start) && *day >= cut {
            continue;
        }
        let step = if at_start { Step::Start } else { Step::End };
        let at = At {
            stratum: 1,
            rank,
            ..At::of(*day, step)
        };
        walk.at(at, Act::Read(Box::new(reading.clone())));
    }
    Ok(())
}

/// Puts a plan instance's legs on their days by its status: left out, uncertain, refused, a
/// hole, or booked (refcore's `prepare`, its plans). E §4.1 step 3, §4.2.
fn schedule_plan(walk: &mut Walk, inst: Inst) -> Result<(), Stop> {
    let rank = if inst.status == Status::Refused {
        walk.text_rank(&inst.move_id)?
    } else {
        walk.text_rank(&inst.order.text())?
    };
    match inst.status {
        Status::LeftOut => return note_left_out(walk, &inst),
        Status::None | Status::Decision | Status::Open => return Ok(()),
        Status::Uncertain => {
            walk.uncertain.push(inst);
            return Ok(());
        }
        Status::Refused | Status::Hole | Status::Past | Status::Plan => {}
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
        Status::LeftOut | Status::None | Status::Decision | Status::Open | Status::Uncertain => {}
    }
    Ok(())
}

/// Puts a decision instance on its day, at its move's stratum; one dated outside the run is
/// left out, one whose day is a hole makes its places holes from the walk's start (refcore's
/// `prepare`, its decisions, and `dec_hole`). E §4.2 step 3, Holes in the walk.
fn schedule_decision(
    m: &Reality,
    walk: &mut Walk,
    (move_id, stratum, rank): (&str, u32, usize),
    inst: Inst,
) -> Result<(), Stop> {
    if inst.exists == Exists::No {
        return Ok(());
    }
    let rank = u32::try_from(rank).map_err(|_| super::plan::invariant("a move's rank"))?;
    match inst.day.clone() {
        Some(Dated::Day(d)) => {
            if d < walk.start || d > walk.end {
                if d < m.start || d > m.end {
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
        Some(Dated::Hole(_)) => {
            return Err(Stop::NotBuilt("a decision whose day is a hole"));
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
    for place in &places {
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
    let day = inst.day.as_ref().and_then(Dated::day);
    let entry = walk
        .left_out
        .entry(inst.move_id.clone())
        .or_insert((0, None));
    entry.0 = entry
        .0
        .checked_add(1)
        .ok_or_else(|| super::plan::invariant("a count of instances"))?;
    entry.1 = match (entry.1, day) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (None, b) => b,
        (a, None) => a,
    };
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

/// Each place's last day of its lifespan inside the walk, under its cut (refcore's `ending`).
/// E §4.2 step 5.
#[must_use]
pub fn endings(m: &Reality, walk: &Walk) -> BTreeMap<Day, Vec<String>> {
    let mut out: BTreeMap<Day, Vec<String>> = BTreeMap::new();
    for (_, thing) in m.things.entries().filter(|(_, t)| m.is_place(t)) {
        let Some(end) = m.lifespan(thing, walk.cut).and_then(|sp| sp.end()) else {
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

/// A leg's amount as a quantity, when it is one. L §5.14.
#[must_use]
pub fn qty_of(amount: &Amount) -> Option<&Num> {
    match amount {
        Amount::Qty(q) => Some(q),
        Amount::Hole(_) | Amount::Refused(_) => None,
    }
}

/// Whether `id` names a plan move (L §5.14.2: a written order with a plan). L §5.14.2.
fn is_plan(m: &Reality, id: &str) -> bool {
    m.moves.get(id).is_some_and(|mv| !mv.decision)
}
