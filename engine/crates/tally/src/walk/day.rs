//! One day of the walk: its acts in order, each leg booked at both ends, the sign rule after each batch, and the day's close.
//! Spec: E §4.2 (steps 0–5: readings, plans, decisions, the close, the checks of the places;
//! holes in the walk), §9 (items 1 and 6: conservation, things hold nothing outside their
//! lifespans); L §5.10 (the sign rule and the points it is checked at; `WARN-NEGATIVE`),
//! §5.14 (a leg of 0 books nothing; a leg outside a lifespan is `RUN-LIFESPAN`), §5.19 (an
//! observed leg before K, each end from the observation of the version that covers its place
//! that day, or from this one where none does; one that fulfils a decision dated its day at
//! the decision's point; a record refused refuses its places), §8.5; E §6.1.
//! Twin: engine/tools/refcore/walk.py (`run_to`, `walk_day`, `check_batch`, `check_group`,
//! `act`, `leg_bad`, `book_leg`, `book_obs_leg`, `book_obs_now`, `apply_reading`,
//! `reading_group`, `sign_anchor`, `alive_day`, `close_day`).
//! Invariants: a leg is booked at both its ends or refused at both; a refused leg is taken back
//! and refuses its places from that point; the legs booked on a day sum to zero per unit; a
//! day's acts run in walk order, and a day cut short past the counted depth resumes at the act
//! it stopped at.

use std::collections::{BTreeMap, BTreeSet};

use super::agenda::{Act, Mv, Reading};
use super::plan::{Amount, End, Inst, LegVal, Status, invariant, qty_value};
use super::point::At;
use super::state::{ALL, Change, Ent, Group, Progress, Taken, Today, Walk, WalkObs};
use crate::base::{Code, Refusal, Run, Warn};
use crate::eval::{Collected, Engine, Entry};
use crate::hole::{Hole, spread};
use crate::model::Reality;
use crate::num::Num;
use crate::syntax::{Sign, Stop};
use crate::time::Day;

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

/// How a refusal of the sign rule names what broke it (refcore's `check_group` messages).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Broke {
    /// The day's first batch.
    Batch(Day),
    /// A decision's legs, by its instance.
    Decision(String),
    /// The closing readings.
    Closing,
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
    /// `run_to`); a main walk's signature is kept once it is done (refcore's `w.sig`). E §4.2,
    /// §4.3, §9 item 4, P §2.2.
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
    /// cell that reads the walk shares it (E §3.2, §3.3). E §4.2, §4.3, §9 item 4.
    fn walk_through(&mut self, m: &Reality, id: usize, last: Option<Day>) -> Result<(), Stop> {
        loop {
            let walk = self.walk(id)?;
            let step = match &walk.progress {
                Progress::Done => return Ok(()),
                Progress::Failed(stop) => return Err(stop.clone()),
                Progress::Planning => self.plan_alone(m, id),
                Progress::Walking if walk.paused_after(self.days, last) => return Ok(()),
                Progress::Walking => self.step_day(m, id),
            };
            match step {
                Ok(()) => {}
                Err(Stop::Deep) => return Err(Stop::Deep),
                Err(stop) => {
                    self.walk_mut(id)?.progress = Progress::Failed(stop.clone());
                    return Err(stop);
                }
            }
            if self.walk(id)?.progress == Progress::Done {
                self.gather_warnings(id)?;
                self.keep_signature(id)?;
            }
        }
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
    /// kept before it; the walk done when the day source has no day left (refcore's `run_to`).
    /// E §4.2, P §2.2, §2.4.
    fn step_day(&mut self, m: &Reality, id: usize) -> Result<(), Stop> {
        if self.walk(id)?.today.is_none() {
            let days = self.days;
            let walk = self.walk_mut(id)?;
            let next = walk.next_day(days, &walk.endings);
            walk.keep_checkpoints(next)?;
            let Some(day) = next else {
                walk.progress = Progress::Done;
                return Ok(());
            };
            let mut acts = walk.agenda.remove(&day).unwrap_or_else(Vec::new);
            acts.sort_by_key(|a| a.0);
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

    /// The sign rule after the day's first batch, its legs taken together (refcore's
    /// `check_batch`). L §5.10, E §4.2 step 5.
    fn check_batch(&mut self, m: &Reality, id: usize, day: Day) -> Result<(), Stop> {
        let Some(batch) = self.today_mut(id)?.batch.take() else {
            return Ok(());
        };
        self.check_group(
            m,
            id,
            &batch,
            At::batch_end(day),
            Taken::Batch(day),
            &Broke::Batch(day),
        )
    }

    /// The sign rule at one of its points: each place tested against the balance the group
    /// leaves, and a place that breaks its kind's rule refused from there with the group's
    /// legs at it, which are taken back; an anchored place with no leg there is refused with
    /// its reading's message (refcore's `check_group`). L §5.10, §8.4, E §4.2 step 5.
    pub(super) fn check_group(
        &mut self,
        m: &Reality,
        id: usize,
        group: &Group,
        at: At,
        taken: Taken,
        broke: &Broke,
    ) -> Result<(), Stop> {
        let mut bad = Vec::new();
        for ((place, unit), legs) in group {
            let st = self.walk(id)?.pstate(m, place, unit, at);
            let Some(nv) = st.known() else {
                continue;
            };
            if breaks(m, place, nv) {
                bad.push((place.clone(), unit.clone(), nv.clone(), legs.clone()));
            }
        }
        for (place, unit, nv, legs) in bad {
            let shown = qty_value(m, &nv, &unit)?
                .text(m)
                .map_err(crate::eval::num_stop)?;
            if legs.is_empty() {
                // an anchored place with no leg of the group at it: refused from here, with
                // its reading's message (L §8.4; tally 2.1.2)
                let instant = self
                    .today(id)?
                    .anchors
                    .get(&(place.clone(), unit.clone()))
                    .cloned()
                    .ok_or_else(|| invariant("a place in a group with no leg and no anchor"))?;
                let err = Refusal::new(
                    Code::Run(Run::Place),
                    format!("{place} is {shown} after its reading at {instant}"),
                );
                self.walk_mut(id)?
                    .add(m, &place, &unit, Ent::new(at, Change::Bad(err)))?;
                continue;
            }
            let err = Refusal::new(
                Code::Run(Run::Place),
                broke_text(m, broke, &place, &shown, &legs),
            );
            for (mv, leg) in &legs {
                let walk = self.walk_mut(id)?;
                let key = (mv.clone(), leg.name.clone());
                if walk.bad_legs.contains_key(&key) {
                    continue;
                }
                walk.bad_legs.insert(key, err.clone());
                for p in ends(leg) {
                    walk.take_back(m, p, &unit, (mv, leg), taken)?;
                }
                for p in ends(leg) {
                    let mut ent = Ent::new(at, Change::Bad(err.clone()));
                    ent.mv = Some(mv.clone());
                    walk.add(m, p, &unit, ent)?;
                }
            }
        }
        Ok(())
    }

    /// One act of the day (refcore's `act`). E §4.2 steps 0–2, Holes in the walk, E §5 step 5.
    fn act(&mut self, m: &Reality, id: usize, at: At, act: Act) -> Result<(), Stop> {
        match act {
            Act::Read(reading) if reading.input == "opening" => {
                self.walk_mut(id)?.apply_opening(m, at, &reading)
            }
            Act::Read(reading) => self.apply_stated(m, id, at, &reading),
            Act::Obs(ix, leg) => self.book_obs_leg(m, id, ix, leg, at),
            Act::Mark(place, unit, entry, coll) => {
                let mut ent = Ent::new(at, Change::Mark);
                ent.reg = vec![entry];
                ent.coll = Some(coll);
                self.walk_mut(id)?.add(m, &place, &unit, ent)
            }
            Act::Leg(mv, leg) => {
                let (movement, lg) = self.movement_leg(id, mv, leg)?;
                let mut reg = Vec::new();
                if movement.status == Status::Past {
                    reg.push(Entry {
                        word: "unfulfilled",
                        text: movement.id.clone(),
                        figure: String::new(),
                    });
                }
                self.book_leg(m, id, (&movement.id, &lg), at, &reg, &movement.coll)
            }
            Act::LegHole(mv, leg, qs) => {
                let (movement, lg) = self.movement_leg(id, mv, leg)?;
                let h = match &lg.amount {
                    Amount::Refused(r) => {
                        return self.walk_mut(id)?.leg_bad(m, &movement.id, &lg, r, at);
                    }
                    Amount::Hole(a) => spread(&[qs.as_ref(), Some(a)]),
                    Amount::Qty(_) => qs,
                };
                let h = h.ok_or_else(|| invariant("a hole movement with no question"))?;
                self.walk_mut(id)?.hole_ends(
                    m,
                    (&movement.id, &lg),
                    at,
                    &Change::Hole(h),
                    &movement.coll,
                )
            }
            Act::LegBad(mv, leg, err) => {
                let (movement, lg) = self.movement_leg(id, mv, leg)?;
                self.walk_mut(id)?.leg_bad(m, &movement.id, &lg, &err, at)
            }
            Act::Perm(mv, leg, h) => {
                let (movement, lg) = self.movement_leg(id, mv, leg)?;
                self.walk_mut(id)?.hole_ends(
                    m,
                    (&movement.id, &lg),
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

    /// A movement of the walk and one of its legs. E §4.2.
    fn movement_leg(&self, id: usize, mv: Mv, leg: usize) -> Result<(Inst, LegVal), Stop> {
        let walk = self.walk(id)?;
        let inst = match mv {
            Mv::Plan(i) => walk.insts.get(i),
            Mv::Run(i) => walk.runs.get(i).map(|r| &r.inst),
        }
        .ok_or_else(|| invariant("an act of a movement the walk lacks"))?;
        let lg = inst
            .legs
            .get(leg)
            .cloned()
            .ok_or_else(|| invariant("an act of a leg the movement lacks"))?;
        Ok((inst.clone(), lg))
    }

    /// Books a leg's two ends: a hole at both for a hole amount, nothing for 0, `RUN-LIFESPAN`
    /// for an end outside its place's lifespan; in a group of the sign rule it waits for the
    /// group, otherwise it is tested at once (refcore's `book_leg`). E §4.2 step 2, step 5;
    /// L §5.14 Legs.
    pub(super) fn book_leg(
        &mut self,
        m: &Reality,
        id: usize,
        (movement, lg): (&str, &LegVal),
        at: At,
        reg: &[Entry],
        coll: &Collected,
    ) -> Result<(), Stop> {
        self.book_leg_ends(m, id, (movement, lg), at, (reg, coll), Ends::both())
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
        (reg, coll): (&[Entry], &Collected),
        mask: Ends,
    ) -> Result<(), Stop> {
        let q = match &lg.amount {
            Amount::Refused(r) => return self.walk_mut(id)?.leg_bad(m, movement, lg, r, at),
            Amount::Hole(h) => {
                #[cfg(feature = "counters")]
                self.counts.add(crate::Unit::Leg);
                let walk = self.walk_mut(id)?;
                let unit = lg.unit.clone().unwrap_or_else(|| ALL.to_owned());
                walk.worth_add(&unit, None)?;
                for (end, on_end) in [(&lg.from, mask.from), (&lg.to, mask.to)] {
                    let Some(p) = end.place().filter(|_| on_end) else {
                        continue;
                    };
                    let mut ent = Ent::new(at, Change::Hole(h.clone()));
                    ent.mv = Some(movement.to_owned());
                    ent.leg = Some(lg.name.clone());
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
            return Err(Stop::NotBuilt(
                "a leg booked outside every group of the sign rule (a reading after a movement)",
            ));
        }
        #[cfg(feature = "counters")]
        if err.is_none() {
            self.counts.add(crate::Unit::Leg);
        }
        let walk = self.walk_mut(id)?;
        if let Some(e) = err {
            walk.bad_legs
                .insert((movement.to_owned(), lg.name.clone()), e.clone());
            for p in ends(lg) {
                let mut ent = Ent::new(at, Change::Bad(e.clone()));
                ent.mv = Some(movement.to_owned());
                walk.add(m, p, &unit, ent)?;
            }
            return Ok(());
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
        if let Some(err) = &wo.err {
            return self.walk_mut(id)?.leg_bad(m, &wo.obs.id, lg, err, at);
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
        let mask = Ends {
            from,
            to,
            whole: false,
        };
        self.book_leg_ends(m, id, (&wo.obs.id, lg), at, (&[], &wo.obs.coll), mask)
    }

    /// Puts a leg's booked ends in the group of the sign rule it waits for: the day's first
    /// batch, the running decision's legs, or the closing readings; false when none is open
    /// (refcore's `book_leg`, its groups). L §5.10, E §4.2 step 5.
    fn group_legs(
        &mut self,
        id: usize,
        at: At,
        movement: &str,
        lg: &LegVal,
        on: &[(String, i64)],
        unit: &str,
    ) -> Result<bool, Stop> {
        let Some(group) = self.walk_mut(id)?.open_group(at) else {
            return Ok(false);
        };
        for (p, _) in on {
            group
                .entry((p.clone(), unit.to_owned()))
                .or_insert_with(Vec::new)
                .push((movement.to_owned(), lg.clone()));
        }
        Ok(true)
    }

    /// The day's close: conservation, `WARN-NEGATIVE` at the accounts and cash it touched, and
    /// `RUN-LIFESPAN` for a place whose lifespan ends holding something (refcore's
    /// `close_day`). E §4.2 steps 4–5, E §9 items 1 and 6.
    fn close_day(&mut self, m: &Reality, id: usize, day: Day) -> Result<(), Stop> {
        let at = At::close(day);
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
                let Some(v) = st.val.clone().filter(|v| !v.is_zero()) else {
                    continue;
                };
                if st.bad.is_some() {
                    continue;
                }
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

impl Walk {
    /// The group of the sign rule a booking at `at` waits for: the day's first batch, the
    /// running decision's legs, or the closing readings; none when none is open (refcore's
    /// `reading_group`, `book_leg`'s groups). L §5.10, E §4.2 step 5.
    pub(crate) fn open_group(&mut self, at: At) -> Option<&mut Group> {
        let in_batch =
            at.in_first_batch() && self.today.as_ref().is_some_and(|t| t.batch.is_some());
        if in_batch {
            return self.today.as_mut().and_then(|t| t.batch.as_mut());
        }
        if self.group.is_some() {
            return self.group.as_mut();
        }
        if at.at_or_after_end() {
            return self.today.as_mut().and_then(|t| t.closing.as_mut());
        }
        None
    }

    /// An anchored reading's place, an opening's included, tested at the reading's point by
    /// its kind's sign rule: a number at a `min 0` or `max 0` place joins the point's group
    /// with no leg, and the group's test refuses it there when it breaks the rule (refcore's
    /// `sign_anchor`; tally 2.1.2). L §5.10, §5.19; E §4.2 step 5.
    ///
    /// # Errors
    ///
    /// `NotBuilt` for a reading after a decision, where no group is open (M2).
    pub(crate) fn sign_anchor(&mut self, m: &Reality, at: At, r: &Reading) -> Result<(), Stop> {
        let ruled = matches!(place_sign(m, &r.place), Some(Sign::Min0 | Sign::Max0));
        if r.value.is_err() || !ruled {
            return Ok(());
        }
        let key = (r.place.clone(), r.unit.clone());
        let Some(group) = self.open_group(at) else {
            return Err(Stop::NotBuilt(
                "a reading after a decision (the sign rule's point 2)",
            ));
        };
        group.entry(key.clone()).or_insert_with(Vec::new);
        self.today
            .as_mut()
            .ok_or_else(|| invariant("a reading with no day open"))?
            .anchors
            .insert(key, r.instant.clone());
        Ok(())
    }

    /// An opening reading: its place's balance set to it, the holes since answered, a stated
    /// hole's questions kept; it always anchors, so its place is tested at its point (refcore's
    /// `apply_reading`, an opening). L §5.19, §5.10, E §4.2 step 0.
    fn apply_opening(&mut self, m: &Reality, at: At, reading: &Reading) -> Result<(), Stop> {
        if self.inside(m, &reading.place) {
            // E §9 item 2: an opening states the place's worth; its change counts as stated
            let expected = match &reading.value {
                Ok(v) => {
                    let prev = self.pstate(m, &reading.place, &reading.unit, at);
                    match (&prev.val, prev.known()) {
                        (None, _) => Some(v.clone()),
                        (Some(_), Some(p)) => {
                            Some(v.checked_sub(p).map_err(crate::eval::num_stop)?)
                        }
                        (Some(_), None) => None,
                    }
                }
                Err(_) => None,
            };
            self.worth_add(&reading.unit, expected.as_ref())?;
        }
        let change = match &reading.value {
            Ok(v) => Change::Reset {
                val: v.clone(),
                qs: None,
            },
            Err(h) => Change::Reset {
                val: Num::zero(),
                qs: Some(h.clone()),
            },
        };
        let mut ent = Ent::new(at, change);
        ent.coll = Some(reading.coll.clone());
        self.add(m, &reading.place, &reading.unit, ent)?;
        self.sign_anchor(m, at, reading)
    }

    /// Takes back a refused leg's entries at its end `p` (refcore's `unbook`); what came back
    /// across the boundary leaves the day's worth (E §9 item 2). E §4.2 step 5.
    fn take_back(
        &mut self,
        m: &Reality,
        p: &str,
        unit: &str,
        (movement, leg): (&str, &LegVal),
        taken: Taken,
    ) -> Result<(), Stop> {
        let back = self.unbook(m, p, unit, (movement, &leg.name), taken)?;
        let other = if leg.from.place() == Some(p) {
            &leg.to
        } else {
            &leg.from
        };
        let within = matches!(other, End::Place(o) if self.inside(m, o));
        if self.inside(m, p) && !within {
            let undone = back.checked_neg().map_err(crate::eval::num_stop)?;
            self.worth_add(unit, Some(&undone))?;
        }
        Ok(())
    }

    /// A refused leg: its places refused from here (refcore's `leg_bad`). E §4.2 step 5.
    pub(super) fn leg_bad(
        &mut self,
        m: &Reality,
        movement: &str,
        lg: &LegVal,
        err: &Refusal,
        at: At,
    ) -> Result<(), Stop> {
        let unit = lg.unit.clone().unwrap_or_else(|| ALL.to_owned());
        for p in ends(lg) {
            let mut ent = Ent::new(at, Change::Bad(err.clone()));
            ent.mv = Some(movement.to_owned());
            self.add(m, p, &unit, ent)?;
        }
        Ok(())
    }

    /// A hole movement's leg: a hole at both its places (refcore's `act`, `legq` and
    /// `perm`). E §4.2 Holes in the walk.
    fn hole_ends(
        &mut self,
        m: &Reality,
        (movement, lg): (&str, &LegVal),
        at: At,
        change: &Change,
        coll: &Collected,
    ) -> Result<(), Stop> {
        let unit = lg.unit.clone().unwrap_or_else(|| ALL.to_owned());
        for p in ends(lg) {
            let mut ent = Ent::new(at, change.clone());
            ent.mv = Some(movement.to_owned());
            ent.coll = Some(coll.clone());
            self.add(m, p, &unit, ent)?;
        }
        Ok(())
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

/// A quantity with a sign: −q out of a place, +q into it. E §4.2.
fn signed(q: &Num, sign: i64) -> Result<Num, Stop> {
    if sign < 0 {
        return q.checked_neg().map_err(crate::eval::num_stop);
    }
    Ok(q.clone())
}

/// A place's kind's sign rule (L §5.10).
fn place_sign(m: &Reality, place: &str) -> Option<Sign> {
    let base = m.things.get(place)?.base.as_deref()?;
    m.kind_sign(base)
}

/// Whether balance `v` breaks its place's sign rule (L §5.10).
fn breaks(m: &Reality, place: &str, v: &Num) -> bool {
    match place_sign(m, place) {
        Some(Sign::Min0) => v.is_negative(),
        Some(Sign::Max0) => !v.is_negative() && !v.is_zero(),
        Some(Sign::Signed) | None => false,
    }
}

/// ` units` for a holding: its balance is units, not money (refcore's message). L §5.10.
fn units_word(m: &Reality, place: &str) -> &'static str {
    let holds = m
        .things
        .get(place)
        .and_then(|t| t.base.as_ref())
        .is_some_and(|b| m.subtree("holdings").contains(b));
    if holds { " units" } else { "" }
}

/// The message of a refusal of the sign rule (refcore's `check_group` messages). L §5.10.
fn broke_text(
    m: &Reality,
    broke: &Broke,
    place: &str,
    shown: &str,
    legs: &[(String, LegVal)],
) -> String {
    let side = match place_sign(m, place) {
        Some(Sign::Max0) => "above 0".to_owned(),
        Some(Sign::Min0 | Sign::Signed) | None => format!("below 0{}", units_word(m, place)),
    };
    match broke {
        Broke::Batch(day) => format!(
            "{place} is {shown} after the day's observations and plans of {}",
            day.text()
        ),
        Broke::Decision(run) => format!("{run} takes {place} {side}"),
        Broke::Closing => {
            let ids: BTreeSet<&str> = legs.iter().map(|(mv, _)| mv.as_str()).collect();
            let all: Vec<&str> = ids.into_iter().collect();
            let plain = match place_sign(m, place) {
                Some(Sign::Max0) => "above 0",
                Some(Sign::Min0 | Sign::Signed) | None => "below 0",
            };
            format!("{} takes {place} {plain}", all.join(", "))
        }
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
    use crate::num::Num;

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
