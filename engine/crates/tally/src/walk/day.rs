//! One day of the walk: its acts in order, each leg booked at both ends, and the day's close.
//! Spec: E §4.2 (steps 0–5: readings, plans, decisions, the close, the checks of the places;
//! holes in the walk; the readings and the sign rule's groups are `walk/readings.rs`'s), §9
//! (items 1 and 6: conservation, things hold nothing outside their lifespans); L §5.10
//! (`WARN-NEGATIVE`), §5.14 (a leg of 0 books nothing; a leg outside a lifespan is
//! `RUN-LIFESPAN`), §5.19 (an observed leg before K, each end from the observation of the
//! version that covers its place that day, or from this one where none does; one that fulfils
//! a decision dated its day at the decision's point; a record refused refuses its places),
//! §8.5; E §6.1. A leg's ends refused or a hole are `walk/ends.rs`'s.
//! Twin: engine/tools/refcore/walk.py (`run_to`, `walk_day`, `act`, `book_leg`,
//! `book_obs_leg`, `book_obs_now`, `alive_day`, `close_day`).
//! Invariants: a leg is booked at both its ends or refused at both; a refused leg is taken back
//! and refuses its places from that point; the legs booked on a day sum to zero per unit; a
//! day's acts run in walk order, and a day cut short past the counted depth resumes at the act
//! it stopped at.

use std::collections::{BTreeMap, BTreeSet};

use super::agenda::{Act, Mv, Planned};
use super::fork::Forks;
use super::instance::{Amount, Dated, End, Inst, LegVal, Status};
use super::plan::{invariant, qty_value};
use super::point::At;
use super::readings::{Alone, Broke};
use super::state::{ALL, Change, Ent, Group, HoleLeg, Progress, Taken, Today, Walk, WalkObs};
use crate::base::{Code, Refusal, Run, Warn};
use crate::book::Member;
use crate::eval::{Collected, Engine, Entry, EntryId, GUARDED};
use crate::hole::{Hole, spread};
use crate::model::{Reality, World};
use crate::num::Num;
use crate::syntax::Stop;
use crate::time::Day;

/// A fault planted in a run, so a test sees the refusal an engine bug gives, `ENGINE-INVARIANT`
/// with its evidence (E §9, §11; INTERFACE §4, pick P8): only with the `fault-injection`
/// feature, which only builds with debug assertions have (R-51).
#[cfg(feature = "fault-injection")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    /// On the first day a walk books a leg whole, the day's close adds one minor unit (0.01)
    /// to that day's conservation sum before invariant 1 is checked, once per run (E §9 item 1;
    /// refcore's twin, `codes/m1_fault.py`).
    Conservation,
}

/// Which ends of a leg a booking books, and whether the leg counts whole for conservation
/// (refcore's `book_leg` `ends`, and its pads). E §6.1, E §9 item 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ends {
    /// Whether its `from` end is booked.
    pub from: bool,
    /// Whether its `to` end is booked.
    pub to: bool,
    /// Whether the leg counts whole even with one end booked: a pad, whose other end is its
    /// reading's reset.
    pub whole: bool,
}

impl Ends {
    /// Both ends (E §4.2 step 2).
    #[must_use]
    pub fn both() -> Ends {
        Ends {
            from: true,
            to: true,
            whole: false,
        }
    }
}

impl Engine {
    /// Walks timeline `id` to its last day: its plan pass first, then each day the day source
    /// gives, a day cut short past the counted depth resumed where it stopped (refcore's
    /// `run_to`). E §4.2, P §2.2.
    ///
    /// # Errors
    ///
    /// The refusal the walk met; `Deep` while a cell it needs waits on the work stack.
    pub(crate) fn advance(&mut self, m: &Reality, id: usize) -> Result<(), Stop> {
        self.advance_to(m, id, None)
    }

    /// Walks timeline `id` through day `last`, or to its last day for none: a later day is
    /// left until a read needs it, so a refusal past `last` refuses no read of it (refcore's
    /// `run_to`); a main walk's signature is kept once it is done (refcore's `w.sig`), and a
    /// scenario's checked against the world under it (refcore's `m_inv3`). E §4.2, §4.3, §9
    /// items 3 and 4, P §2.2.
    ///
    /// # Errors
    ///
    /// The refusal the walk met; `Deep` while a cell it needs waits on the work stack.
    pub(crate) fn advance_to(
        &mut self,
        m: &Reality,
        id: usize,
        last: Option<Day>,
    ) -> Result<(), Stop> {
        let mark = self.reads_apart();
        let walked = self.walk_through(m, id, last);
        self.reads_back(mark);
        walked
    }

    /// [`Engine::advance_to`], what it reads the walk's own, not the reading cell's: every
    /// cell that reads the walk shares it (E §3.2, §3.3); once its plan pass is done, a
    /// projection's walk may continue a kept one (`fork.rs`, P §2.5). A walk that failed on a
    /// day refuses the reads that reach it, and a read through an earlier day reads the walk as
    /// it stood there, so a read's answer is the same whichever read walked the walk first
    /// (pick P54, #127). E §4.2, §4.3, §9 item 4; L §8.4.
    fn walk_through(&mut self, m: &Reality, id: usize, last: Option<Day>) -> Result<(), Stop> {
        loop {
            let walk = self.walk(id)?;
            let step = match &walk.progress {
                Progress::Done => return Ok(()),
                Progress::Failed(stop, on) => return before_failure(stop, *on, last),
                Progress::Planning => self
                    .plan_alone(m, id)
                    .and_then(|()| self.continue_planned(id)),
                Progress::Walking if walk.paused_after(self.days, last) => return Ok(()),
                Progress::Walking => self.step_day(m, id),
                Progress::Walked => self.close_walk(m, id),
            };
            match step {
                Ok(()) => {}
                // a walk cut short past the counted depth resumes where it stopped; one
                // given up for a cell outside it (`GUARDED`) is walked on when read again
                Err(Stop::Deep) => return Err(Stop::Deep),
                Err(Stop::NotBuilt(why)) if why == GUARDED => return Err(Stop::NotBuilt(why)),
                Err(stop) => {
                    let failed = self.walk_mut(id)?;
                    let on = match failed.progress {
                        Progress::Walking => failed.today.as_ref().map(|t| t.day),
                        Progress::Planning
                        | Progress::Walked
                        | Progress::Done
                        | Progress::Failed(..) => None,
                    };
                    failed.progress = Progress::Failed(stop.clone(), on);
                    return Err(stop);
                }
            }
        }
    }

    /// A walked walk's close, once: its warnings, its signature, and a scenario's E §9 item 3
    /// (refcore's `m_main_walk`); done only once all three hold, so a close cut short past the
    /// counted depth runs again on replay (R-16), and its refusal refuses every read of the
    /// walk. A walk up to a day keeps its warnings, since its days are the timeline's first
    /// days and a place's warning is its walk's, whichever read reached it (E §4.2 step 5);
    /// it has no signature and no item 3: it is no timeline's walk, only what a leg's read
    /// needs (refcore's `prefix_walk`). E §9 items 3 and 4, L §8.5, E §3.2.
    fn close_walk(&mut self, m: &Reality, id: usize) -> Result<(), Stop> {
        self.gather_warnings(id)?;
        if self.walk(id)?.upto.is_none() {
            self.keep_signature(id)?;
            self.scenario_unchanged(m, id)?;
        }
        self.walk_mut(id)?.progress = Progress::Done;
        Ok(())
    }

    /// A walk's plan pass, what its evaluation met kept apart from the read that started it
    /// (refcore's `prepare`, in its own collector). E §4.1.
    fn plan_alone(&mut self, m: &Reality, id: usize) -> Result<(), Stop> {
        self.push();
        let planned = self.plan_walk(m, id);
        self.pop_alone();
        planned
    }

    /// A walk's warnings, once it is done, joined to the engine's, each once (refcore's
    /// `main_walk`, its warnings). L §8.5.
    fn gather_warnings(&mut self, id: usize) -> Result<(), Stop> {
        let found = self.walk(id)?.warnings.clone();
        for w in found {
            if !self.walk_warnings.contains(&w) {
                self.walk_warnings.push(w);
            }
        }
        Ok(())
    }

    /// The next day of the walk, opened and walked, the checkpoints of the months it enters
    /// kept before it, and with forks on a projection's walk keeping the day's plan acts for a
    /// later read to compare (`fork.rs`); the walk done when the day source has no day left
    /// (refcore's `run_to`). E §4.2, P §2.2, §2.4, §2.5.
    fn step_day(&mut self, m: &Reality, id: usize) -> Result<(), Stop> {
        if self.walk(id)?.today.is_none() {
            let (days, forks) = (self.days, self.forks);
            let walk = self.walk_mut(id)?;
            let next = walk.next_day(days, &walk.endings);
            walk.keep_checkpoints(next)?;
            let Some(day) = next else {
                walk.progress = Progress::Walked;
                return Ok(());
            };
            let mut acts = walk.agenda.remove(&day).unwrap_or_else(Vec::new);
            acts.sort_by_key(|a| a.0);
            if forks == Forks::On && walk.proj.is_some() {
                walk.keep_walked(day, &acts);
            }
            #[cfg(feature = "counters")]
            let unit = if acts.is_empty() {
                crate::Unit::Day
            } else {
                crate::Unit::NonEmptyDay
            };
            walk.today = Some(Today {
                day,
                acts,
                next: 0,
                next_inst: 0,
                runs: Vec::new(),
                cycle: None,
                batch: Some(Group::new()),
                closing: None,
                anchors: BTreeMap::new(),
                touched: BTreeSet::new(),
            });
            #[cfg(feature = "counters")]
            self.counts.add(unit);
        }
        self.walk_today(m, id)
    }

    /// The day being walked, from the act it stands at: each act in walk order, the first
    /// batch's sign rule before the first act past it, the closing readings', then the close
    /// (refcore's `walk_day`). E §4.2.
    fn walk_today(&mut self, m: &Reality, id: usize) -> Result<(), Stop> {
        loop {
            let today = self.today(id)?;
            let day = today.day;
            let Some((at, act)) = today.acts.get(today.next).cloned() else {
                break;
            };
            if !at.in_first_batch() && today.batch.is_some() {
                self.check_batch(m, id, day)?;
            }
            if at.at_or_after_end() && self.today(id)?.closing.is_none() {
                self.today_mut(id)?.closing = Some(Group::new());
            }
            if let Act::Decide(move_id, d) = &act {
                self.run_decisions(m, id, move_id, *d, at)?;
            } else {
                self.act(m, id, at, act)?;
            }
            let open = self.today_mut(id)?;
            open.next = open
                .next
                .checked_add(1)
                .ok_or_else(|| invariant("a day's acts"))?;
            open.next_inst = 0;
            open.runs.clear();
            open.cycle = None;
        }
        let day = self.today(id)?.day;
        if self.today(id)?.batch.is_some() {
            self.check_batch(m, id, day)?;
        }
        if let Some(closing) = self.today_mut(id)?.closing.take() {
            let at = At::closing_end(day);
            self.check_group(m, id, &closing, at, Taken::Closing(day), &Broke::Closing)?;
        }
        self.close_day(m, id, day)?;
        let walk = self.walk_mut(id)?;
        walk.done = Some(day);
        walk.today = None;
        Ok(())
    }

    /// One act of the day (refcore's `act`). E §4.2 steps 0–2, Holes in the walk, E §5 step 5.
    fn act(&mut self, m: &Reality, id: usize, at: At, act: Act) -> Result<(), Stop> {
        match act {
            Act::Read(reading) => self.apply_reading(m, id, at, &reading),
            Act::Obs(ix, leg) => self.book_obs_leg(m, id, ix, leg, at),
            Act::Mark(place, unit, entry, coll) => {
                let mut ent = Ent::new(at, Change::Mark);
                ent.reg = entry.into_iter().collect();
                ent.coll = Some(coll);
                self.walk_mut(id)?.add(m, &place, &unit, ent)
            }
            Act::Leg(mv, leg) => {
                let planned = self.movement(id, mv)?;
                let (movement, lg) = planned.leg(leg)?;
                if let Amount::Refused(r) = &lg.amount {
                    let walk = self.walk_mut(id)?;
                    return walk.leg_bad(m, (&movement.id, lg, &movement.coll), r, at);
                }
                let mut reg = Vec::new();
                if movement.status == Status::Past && movement.obs.is_empty() {
                    reg.push(self.entry_id(Entry {
                        word: "unfulfilled",
                        text: movement.id.clone(),
                        figure: String::new(),
                    })?);
                }
                self.book_leg(m, id, (movement, lg), at, &reg)
            }
            Act::LegHole(mv, leg, qs) => {
                let planned = self.movement(id, mv)?;
                let (movement, lg) = planned.leg(leg)?;
                let h = match &lg.amount {
                    Amount::Refused(r) => {
                        return self.walk_mut(id)?.leg_bad(
                            m,
                            (&movement.id, lg, &movement.coll),
                            r,
                            at,
                        );
                    }
                    Amount::Hole(a) => spread(&[qs.as_ref(), Some(a)]),
                    Amount::Qty(_) => qs,
                };
                let h = h.ok_or_else(|| invariant("a hole movement with no question"))?;
                self.walk_mut(id)?.hole_ends(
                    m,
                    (&movement.id, lg),
                    at,
                    &Change::Hole(h),
                    &movement.coll,
                )
            }
            Act::LegBad(mv, leg, err) => {
                let planned = self.movement(id, mv)?;
                let (movement, lg) = planned.leg(leg)?;
                let walk = self.walk_mut(id)?;
                if movement.day.as_ref().and_then(Dated::day).is_none() {
                    // its day is a hole or refused (tally 2.1.8, R11)
                    return walk.leg_bad_undated(m, (&movement.id, lg, &movement.coll), &err, at);
                }
                walk.leg_bad(m, (&movement.id, lg, &movement.coll), &err, at)
            }
            Act::Perm(mv, leg, h) => {
                let planned = self.movement(id, mv)?;
                let (movement, lg) = planned.leg(leg)?;
                self.walk_mut(id)?.hole_ends(
                    m,
                    (&movement.id, lg),
                    at,
                    &Change::Perm(h),
                    &movement.coll,
                )
            }
            Act::Changed(place, unit, h, coll) => {
                let mut ent = Ent::new(at, Change::Perm(h));
                ent.coll = Some(coll);
                self.walk_mut(id)?.add(m, &place, &unit, ent)
            }
            Act::ChangeRefused(place, err, adj) => {
                let mut ent = Ent::new(at, Change::Bad(err));
                ent.mv = Some(adj);
                self.walk_mut(id)?.add(m, &place, ALL, ent)
            }
            Act::PlaceHole(place, h, movement, coll) => {
                let mut ent = Ent::new(at, Change::Perm(h));
                ent.mv = Some(movement);
                ent.coll = Some(coll);
                self.walk_mut(id)?.add(m, &place, ALL, ent)
            }
            Act::GlobalHole(h) => {
                self.walk_mut(id)?
                    .add(m, ALL, ALL, Ent::new(at, Change::Perm(h)))
            }
            Act::GlobalBad(err) => {
                self.walk_mut(id)?
                    .add(m, ALL, ALL, Ent::new(at, Change::Bad(err)))
            }
            Act::Decide(..) => Err(invariant("a decision act taken as a plain act")),
        }
    }

    /// A movement of the walk, where its plan pass's list holds it: a leg is read there, never
    /// from a copy. E §4.2.
    fn movement(&self, id: usize, mv: Mv) -> Result<Planned, Stop> {
        self.walk(id)?
            .insts
            .get(mv.0)
            .cloned()
            .ok_or_else(|| invariant("an act of a movement the walk lacks"))
    }

    /// Books instance `inst`'s leg `lg`'s two ends: a hole at both for a hole amount, nothing
    /// for 0, `RUN-LIFESPAN` for an end outside its place's lifespan; in a group of the sign
    /// rule it waits for the group, otherwise it is tested at once (refcore's `book_leg`).
    /// Its `before from:` entry is dated by the instance's day, and a leg of an instance dated
    /// by no day has none (refcore's `before_from`). E §4.2 step 2, step 5, §9 item 3;
    /// L §5.14 Legs, §8.2 `before from:`.
    pub(super) fn book_leg(
        &mut self,
        m: &Reality,
        id: usize,
        (inst, lg): (&Inst, &LegVal),
        at: At,
        reg: &[EntryId],
    ) -> Result<(), Stop> {
        let (world, cut) = {
            let walk = self.walk(id)?;
            (walk.world, walk.cut)
        };
        let mut reg = reg.to_vec();
        // a leg is on its instance's day (L §8.2)
        if let Some(day) = inst.day.as_ref().and_then(Dated::day)
            && let Some(entry) = before_from(m, world, (&inst.move_id, day), &inst.coll)
        {
            reg.push(self.entry_id(entry)?);
        }
        let coll = &inst.coll;
        let mut mask = Ends::both();
        if self.stated_same(m, &inst.id, cut)? && !matches!(lg.amount, Amount::Refused(_)) {
            // one movement on two statements books each place once (L §6.1)
            let mover = self.mover_of(m, id, inst)?;
            mask = self.same_mask(m, id, (&mover, lg), at, (&reg, coll), mask)?;
            if !mask.from && !mask.to {
                return Ok(());
            }
        }
        self.book_leg_ends(m, id, (&inst.id, lg), at, (&reg, coll), mask)
    }

    /// Books the ends of a leg `ends` names, as [`Engine::book_leg`] books both: an
    /// observation books an end only where its own version covers the place, or none does; a
    /// pad books its other end, its reading's end being the reset (refcore's `book_leg`,
    /// `ends`). E §6.1, §4.2 step 2, step 5.
    pub(crate) fn book_leg_ends(
        &mut self,
        m: &Reality,
        id: usize,
        (movement, lg): (&str, &LegVal),
        at: At,
        (reg, coll): (&[EntryId], &Collected),
        mask: Ends,
    ) -> Result<(), Stop> {
        let q = match &lg.amount {
            Amount::Refused(r) => {
                return self.walk_mut(id)?.leg_bad(m, (movement, lg, coll), r, at);
            }
            Amount::Hole(h) => {
                #[cfg(feature = "counters")]
                self.counts.add(crate::Unit::Leg);
                let walk = self.walk_mut(id)?;
                let unit = lg.unit.clone().unwrap_or_else(|| ALL.to_owned());
                walk.worth_add(&unit, None)?;
                for (end, on_end, into) in [(&lg.from, mask.from, false), (&lg.to, mask.to, true)] {
                    let Some(p) = end.place().filter(|_| on_end) else {
                        continue;
                    };
                    let mut ent = Ent::new(at, Change::Hole(h.clone()));
                    ent.mv = Some(movement.to_owned());
                    ent.leg = Some(lg.name.clone());
                    ent.hole_leg = Some(HoleLeg {
                        into,
                        src: lg.src.clone(),
                    });
                    ent.reg = reg.to_vec();
                    ent.coll = Some(coll.clone());
                    walk.add(m, p, &unit, ent)?;
                }
                return Ok(());
            }
            Amount::Qty(q) => q.clone(),
        };
        if q.is_zero() {
            return Ok(());
        }
        let unit = lg
            .unit
            .clone()
            .ok_or_else(|| invariant("a booked leg with no unit"))?;
        let cut = self.walk(id)?.cut;
        let sides = [(&lg.from, -1_i64, mask.from), (&lg.to, 1_i64, mask.to)];
        let mut on: Vec<(String, i64)> = Vec::new();
        let mut err = None;
        let (mut kept, mut dropped) = (false, !(mask.from && mask.to));
        for (end, sign, on_end) in sides {
            if !on_end {
                continue;
            }
            let End::Place(p) = end else {
                // L §5.14 (tally 2.1.2): a leg whose place is a hole books its known side
                // only, so it is left out of conservation (E §9 item 1)
                kept = true;
                dropped = true;
                continue;
            };
            let thing = m
                .things
                .get(p)
                .ok_or_else(|| invariant("a leg's place load didn't build"))?;
            let span = m.lifespan(thing, (cut, self.timeline_world()));
            if let Some(sp) = &span
                && let Some(question) = sp.end_question()
                && sp.contains(at.day)
            {
                let mut ent = Ent::new(at, Change::Hole(Hole::asked(question, p)));
                ent.mv = Some(movement.to_owned());
                ent.leg = Some(lg.name.clone());
                ent.reg = reg.to_vec();
                ent.coll = Some(coll.clone());
                let walk = self.walk_mut(id)?;
                walk.worth_add(&unit, None)?;
                walk.add(m, p, &unit, ent)?;
                dropped = true;
                continue;
            }
            if !span.is_some_and(|sp| sp.contains(at.day)) {
                err = Some(Refusal::new(
                    Code::Run(Run::Lifespan),
                    format!(
                        "{movement}.{} moves into or out of {p} outside its lifespan",
                        lg.name
                    ),
                ));
                break;
            }
            kept = true;
            on.push((p.clone(), sign));
        }
        if err.is_none() && !kept {
            return Ok(());
        }
        if err.is_none() && !on.is_empty() && !self.group_legs(id, at, movement, lg, &on, &unit)? {
            // outside every group: tested as it is booked
            match self
                .walk(id)?
                .tested_alone(m, (movement, &q), &on, &unit, at)?
            {
                Alone::Passes => {}
                Alone::Refused(e) => err = Some(e),
            }
        }
        #[cfg(feature = "counters")]
        if err.is_none() {
            self.counts.add(crate::Unit::Leg);
        }
        let walk = self.walk_mut(id)?;
        if let Some(e) = err {
            return walk.refuse_leg(m, (movement, lg, coll), &e, at);
        }
        let whole = !dropped || mask.whole;
        for (p, sign) in &on {
            let f = signed(&q, *sign)?;
            if whole {
                let sum = walk.conserved.entry(unit.clone()).or_insert_with(Num::zero);
                *sum = sum.checked_add(&f).map_err(crate::eval::num_stop)?;
            }
            // E §9 item 2: a whole leg changes worth only across the boundary; one booked by
            // its ends is left out of the boundary's account, as from item 1
            let other = if *sign < 0 { &lg.to } else { &lg.from };
            let crosses = matches!(other, End::Place(o) if !walk.inside(m, o));
            if walk.inside(m, p) && (crosses || !whole) {
                walk.worth_add(&unit, Some(&f))?;
            }
            let mut ent = Ent::new(at, Change::Delta(f));
            ent.mv = Some(movement.to_owned());
            ent.leg = Some(lg.name.clone());
            ent.reg = reg.to_vec();
            ent.coll = Some(coll.clone());
            walk.add(m, p, &unit, ent)?;
        }
        Ok(())
    }

    /// An observed leg on its own day before the cut, unless its observation fulfils a decision
    /// instance dated that day, which books it at the instance's point (refcore's
    /// `book_obs_leg`). E §4.2 step 1, L §5.19 Booking an observation.
    fn book_obs_leg(
        &mut self,
        m: &Reality,
        id: usize,
        ix: usize,
        leg: usize,
        at: At,
    ) -> Result<(), Stop> {
        let walk = self.walk(id)?;
        let wo = walk
            .obs
            .get(ix)
            .cloned()
            .ok_or_else(|| invariant("an act of an observation the walk lacks"))?;
        let lg = wo
            .obs
            .legs
            .get(leg)
            .cloned()
            .ok_or_else(|| invariant("an act of a leg the observation lacks"))?;
        let day = lg
            .day
            .day()
            .ok_or_else(|| invariant("an observed leg with no day"))?;
        if day >= walk.cut || wo.decision_day == Some(day) {
            return Ok(());
        }
        self.book_obs_now(m, id, &wo, &lg, at)
    }

    /// Books an observed leg now: refused at both ends when its record is; else each end from
    /// the observation of the version that covers its place that day, an end no version
    /// covers from this one (refcore's `book_obs_now`). E §6.1, L §5.19, §8.4.
    pub(super) fn book_obs_now(
        &mut self,
        m: &Reality,
        id: usize,
        wo: &WalkObs,
        lg: &LegVal,
        at: At,
    ) -> Result<(), Stop> {
        let refused = wo.err.as_ref().or(match &lg.amount {
            Amount::Refused(r) => Some(r),
            Amount::Qty(_) | Amount::Hole(_) => None,
        });
        if let Some(err) = refused {
            return self
                .walk_mut(id)?
                .leg_bad(m, (&wo.obs.id, lg, &wo.obs.coll), err, at);
        }
        let cut = self.walk(id)?.cut;
        let day = lg
            .day
            .day()
            .ok_or_else(|| invariant("an observed leg with no day"))?;
        let mut on = [true, true];
        for (end, slot) in [&lg.from, &lg.to].into_iter().zip(on.iter_mut()) {
            if let Some(p) = end.place() {
                *slot = self.obs_end_on(m, &wo.obs, p, day, cut);
            }
        }
        let [from, to] = on;
        let mut mask = Ends {
            from,
            to,
            whole: false,
        };
        if self.stated_same(m, &wo.obs.id, cut)? {
            // one movement on two statements books each place once (L §6.1)
            let mover = Member::Obs(wo.obs.clone());
            mask = self.same_mask(m, id, (&mover, lg), at, (&[], &wo.obs.coll), mask)?;
            if !mask.from && !mask.to {
                return Ok(());
            }
        }
        self.book_leg_ends(m, id, (&wo.obs.id, lg), at, (&[], &wo.obs.coll), mask)
    }

    /// The planted fault, once per run: on the first day a walk that is no projection's books
    /// a leg whole, one minor unit (0.01) added to the day's conservation sum in its first unit,
    /// so invariant 1 refuses as on an engine bug (refcore's twin, `codes/m1_fault.py`; INTERFACE
    /// §4). E §9 item 1, §11.
    #[cfg(feature = "fault-injection")]
    fn plant_fault(&mut self, id: usize) -> Result<(), Stop> {
        let Some(Fault::Conservation) = self.plant else {
            return Ok(());
        };
        let walk = self.walk_mut(id)?;
        if walk.proj.is_some() {
            return Ok(());
        }
        let Some(sum) = walk.conserved.values_mut().next() else {
            return Ok(());
        };
        let minor = Num::ratio(1, 100).map_err(crate::eval::num_stop)?;
        *sum = sum.checked_add(&minor).map_err(crate::eval::num_stop)?;
        self.plant = None;
        Ok(())
    }

    /// The day's close: conservation, `WARN-NEGATIVE` at the accounts and cash it touched, and
    /// `RUN-LIFESPAN` for a place whose lifespan ends holding something, each tested only where
    /// the balance is known (refcore's `close_day`). E §4.2 steps 4–5, E §9 items 1 and 6; L §8
    /// (tally 2.3.1 R1, #248).
    fn close_day(&mut self, m: &Reality, id: usize, day: Day) -> Result<(), Stop> {
        let at = At::close(day);
        #[cfg(feature = "fault-injection")]
        self.plant_fault(id)?;
        let walk = self.walk_mut(id)?;
        for (unit, sum) in &walk.conserved {
            if !sum.is_zero() {
                let shown = qty_value(m, sum, unit)?
                    .text(m)
                    .map_err(crate::eval::num_stop)?;
                return Err(invariant(&format!(
                    "conservation: the legs booked on {} in {unit} sum to {shown}, not 0 (E §9 invariant 1)",
                    day.text()
                )));
            }
        }
        walk.conserved.clear();
        let touched = match &walk.today {
            Some(t) => t.touched.clone(),
            None => BTreeSet::new(),
        };
        check_worth(m, walk, day, &touched)?;
        let money = if walk.proj.is_none() {
            accounts_and_cash(m)
        } else {
            BTreeSet::new()
        };
        for place in &touched {
            let Some(thing) = m.things.get(place) else {
                continue;
            };
            if !thing.base.as_ref().is_some_and(|b| money.contains(b)) {
                continue;
            }
            for unit in walk.units_of(place) {
                let st = walk.pstate(m, place, &unit, at);
                let Some(v) = st.known() else {
                    continue;
                };
                let neg = v.is_negative();
                let key = (place.clone(), unit);
                if neg && walk.negative.get(&key) != Some(&true) {
                    walk.warnings.push((
                        Warn::Negative,
                        format!("{place} is below zero from {}", day.text()),
                    ));
                }
                walk.negative.insert(key, neg);
            }
        }
        let ending = walk.endings.get(&day).cloned().unwrap_or_else(Vec::new);
        for place in ending {
            for unit in walk.units_of(&place) {
                let st = walk.pstate(m, &place, &unit, at);
                // on a balance that is a hole the check isn't made: the balance stays its hole
                // (L §8, what an operation decides; tally 2.3.1 R1, #248's face (b)); a refused
                // one keeps its refusal
                let Some(v) = st.known().filter(|v| !v.is_zero()).cloned() else {
                    continue;
                };
                let shown = qty_value(m, &v, &unit)?
                    .text(m)
                    .map_err(crate::eval::num_stop)?;
                let err = Refusal::new(
                    Code::Run(Run::Lifespan),
                    format!("{place} ends on {} holding {shown}", day.text()),
                );
                walk.add(m, &place, &unit, Ent::new(at, Change::Bad(err)))?;
            }
        }
        Ok(())
    }

    /// The day being walked. E §4.2.
    pub(super) fn today(&self, id: usize) -> Result<&Today, Stop> {
        self.walk(id)?
            .today
            .as_ref()
            .ok_or_else(|| invariant("a day walked with no day open"))
    }

    /// The day being walked, to change. E §4.2.
    pub(super) fn today_mut(&mut self, id: usize) -> Result<&mut Today, Stop> {
        self.walk_mut(id)?
            .today
            .as_mut()
            .ok_or_else(|| invariant("a day walked with no day open"))
    }
}

/// Net worth moves only at the boundary or at an anchor, per unit so no price is needed: the
/// day's change in the balances of the places that aren't outside equals the legs across the
/// boundary and those booked by their ends, the pads from outside, the openings and the
/// anchors' gaps; a unit some hole leaves unknown that day is not compared, and a projection's
/// walk is not checked (refcore's `inv2_check`). E §9 item 2.
fn check_worth(
    m: &Reality,
    walk: &mut Walk,
    day: Day,
    touched: &BTreeSet<String>,
) -> Result<(), Stop> {
    let expected = core::mem::take(&mut walk.worth);
    if walk.proj.is_some() {
        return Ok(());
    }
    let before = At::end(
        day.previous()
            .ok_or_else(|| invariant("a walked day with no day before it"))?,
    );
    let mut actual: BTreeMap<String, Option<Num>> = BTreeMap::new();
    for place in touched {
        if !walk.inside(m, place) {
            continue;
        }
        for unit in walk.units_of(place) {
            let now = walk.pstate(m, place, &unit, At::end(day));
            let then = walk.pstate(m, place, &unit, before);
            let slot = actual.entry(unit).or_insert_with(|| Some(Num::zero()));
            *slot = match (slot.take(), now.known(), then.known()) {
                (Some(sum), Some(a), Some(b)) => {
                    let change = a.checked_sub(b).map_err(crate::eval::num_stop)?;
                    Some(sum.checked_add(&change).map_err(crate::eval::num_stop)?)
                }
                _ => None,
            };
        }
    }
    let units: BTreeSet<&String> = actual.keys().chain(expected.keys()).collect();
    // a unit on one side only moved by nothing on the other: a sum over no entries is 0
    let nothing = Some(Num::zero());
    for unit in units {
        let x = match actual.get(unit) {
            Some(x) => x,
            None => &nothing,
        };
        let y = match expected.get(unit) {
            Some(y) => y,
            None => &nothing,
        };
        let (Some(x), Some(y)) = (x, y) else {
            continue;
        };
        if x != y {
            let shown = |q: &Num| -> Result<String, Stop> {
                qty_value(m, q, unit)?
                    .text(m)
                    .map_err(crate::eval::num_stop)
            };
            return Err(invariant(&format!(
                "net worth: the places that aren't outside change by {} in {unit} on {}, and the \
                 boundary, pads and gaps by {} (E §9 invariant 2)",
                shown(x)?,
                day.text(),
                shown(y)?
            )));
        }
    }
    Ok(())
}

/// The places a leg names. L §5.14.
pub(super) fn ends(lg: &LegVal) -> Vec<&str> {
    [&lg.from, &lg.to]
        .into_iter()
        .filter_map(End::place)
        .collect()
}

/// The `before from:` entry of a leg an instance of move `move_id` books on `day`, where its
/// read set holds a fact of a scenario of the walk's world whose `from` is after `day`: listed
/// under each result whose reads reach the leg, in that branch and every branch over it; none
/// for any other leg (refcore's `before_from`). L §8.2 `before from:`, §6.4; E §9 item 3
/// (tally 2.1.7 R7).
fn before_from(
    m: &Reality,
    world: World,
    (move_id, day): (&str, Day),
    coll: &Collected,
) -> Option<Entry> {
    let listed = coll.listed()?;
    let items: BTreeSet<&str> = listed
        .iter()
        .filter(|(section, _)| changed_after(m, world, section, day))
        .map(|(_, item)| item.as_str())
        .collect();
    if items.is_empty() {
        return None;
    }
    let items: Vec<&str> = items.into_iter().collect();
    Some(Entry {
        word: "before from",
        text: format!("{move_id} on {} reads {}", day.text(), items.join(", ")),
        figure: String::new(),
    })
}

/// Whether `section` is of a scenario world `world` sees whose `from` is after `day` (refcore's
/// `before_from`, its `changed`). L §6.4.
fn changed_after(m: &Reality, world: World, section: &str, day: Day) -> bool {
    let Some(scenario) = m.scenarios.of(section) else {
        return false;
    };
    m.scenarios.sees(world, Some(section))
        && m.scenarios
            .from
            .get(scenario)
            .is_some_and(|from| day < *from)
}

/// A quantity with a sign: −q out of a place, +q into it. E §4.2.
pub(super) fn signed(q: &Num, sign: i64) -> Result<Num, Stop> {
    if sign < 0 {
        return q.checked_neg().map_err(crate::eval::num_stop);
    }
    Ok(q.clone())
}

/// A failed walk's answer to a read through day `last`: the walk as it stood, for a read that
/// ends before the day the walk failed on; its refusal for one that reaches it, for a read to
/// the walk's end, and for a failure of no day, its plan pass or its close (pick P54, #127).
/// E §4.3, L §8.4.
fn before_failure(stop: &Stop, on: Option<Day>, last: Option<Day>) -> Result<(), Stop> {
    match (on, last) {
        (Some(failed), Some(read)) if read < failed => Ok(()),
        (Some(_) | None, Some(_) | None) => Err(stop.clone()),
    }
}

/// The kinds whose places warn below zero: accounts and cash, with their subkinds (L §5.10).
fn accounts_and_cash(m: &Reality) -> BTreeSet<String> {
    let mut out = m.subtree("accounts");
    out.extend(m.subtree("cash"));
    out
}

#[cfg(test)]
mod tests {
    use super::signed;
    use crate::api::{Asks, Files, Given, Request, run};
    use crate::num::Num;

    /// A place whose lifespan ends on 30 June, emptied by a movement whose amount is a hole,
    /// and the same pinned to an amount that leaves something (`critic/life_amt`, #248).
    const LIFE: &str = r#"model life
  tally 2
  on 2026-06-30
  start 2026-01-01
  end 2026-12-31

source s "invented values"

unit PLN: currency minor 0.01

place konto: accounts PLN lives 2026-01-01..

place spare: accounts PLN lives 2026-01-01..2026-06-30

input out_amt: money PLN

input out_day: date

move empty: transfers
  on out_day
  leg cash from spare to konto amount out_amt

ask s_end: bal(spare, 2026-06-30)

ask s_p60: bal(spare, 2026-06-30) with out_amt = 60.00 PLN

version v
  cite s
  governs always
  known 2026-01-01
  opening[spare] = 100.00 PLN at 2026-01-01 start "s"
  opening[konto] = 0.00 PLN at 2026-01-01 start "s"
  out_amt = ?"how much was moved out?" "s"
  out_day = 2026-03-10 "s"
"#;

    #[test]
    fn a_lifespan_end_on_a_hole_balance_is_the_balances_hole() {
        // L §8 (tally 2.3.1 R1, #248's face (b)): the check isn't made on a balance that is a
        // hole, so the balance prints its hole; on a known one it refuses
        let given = vec![Given {
            path: "life.tally".to_owned(),
            bytes: Some(LIFE.as_bytes().to_vec()),
        }];
        let request = Request::new(Asks::Every);
        let answer = run(Files::new(given), &request).expect("an answer");
        let values: Vec<&str> = answer
            .lines()
            .iter()
            .filter(|line| line.starts_with("s_"))
            .map(String::as_str)
            .collect();
        assert_eq!(
            values,
            [
                "s_end: ?\"how much was moved out?\" (out_amt)",
                "s_p60: refused: RUN-LIFESPAN spare ends on 2026-06-30 holding 40.00 PLN"
            ],
            "the hole, and the known balance's refusal"
        );
    }

    #[test]
    fn a_sign_turns_a_quantity_round() {
        assert_eq!(
            signed(&Num::int(4), -1).expect("signed"),
            Num::int(-4),
            "out"
        );
        assert_eq!(signed(&Num::int(4), 1).expect("signed"), Num::int(4), "in");
    }
}
