//! One day of the walk: its acts in order, each leg booked at both ends, the sign rule after each batch, and the day's close.
//! Spec: E §4.2 (steps 0–5: readings, plans, decisions, the close, the checks of the places;
//! holes in the walk), §9 (items 1 and 6: conservation, things hold nothing outside their
//! lifespans); L §5.10 (the sign rule and the points it is checked at; `WARN-NEGATIVE`),
//! §5.14 (a leg of 0 books nothing; a leg outside a lifespan is `RUN-LIFESPAN`), §8.5.
//! Twin: engine/tools/refcore/walk.py (`run_to`, `walk_day`, `check_batch`, `check_group`,
//! `act`, `leg_bad`, `book_leg`, `apply_reading`, `alive_day`, `close_day`).
//! Invariants: a leg is booked at both its ends or refused at both; a refused leg is taken back
//! and refuses its places from that point; the legs booked on a day sum to zero per unit; a
//! day's acts run in walk order, and a day cut short past the counted depth resumes at the act
//! it stopped at.

use std::collections::BTreeSet;

use super::agenda::{Act, Mv, Reading};
use super::plan::{Amount, End, Inst, LegVal, Status, invariant, qty_value};
use super::point::At;
use super::state::{ALL, Change, Ent, Group, Progress, Taken, Today, Walk};
use crate::base::{Code, Refusal, Run, Warn};
use crate::eval::{Collected, Engine, Entry};
use crate::hole::{Hole, spread};
use crate::model::Reality;
use crate::num::Num;
use crate::syntax::{Sign, Stop};
use crate::time::Day;

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
        loop {
            let walk = self.walk(id)?;
            let step = match &walk.progress {
                Progress::Done => return Ok(()),
                Progress::Failed(stop) => return Err(stop.clone()),
                Progress::Planning => self.plan_walk(m, id),
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
            }
        }
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

    /// The next day of the walk, opened and walked; the walk done when the day source has no
    /// day left (refcore's `run_to`). E §4.2, P §2.2.
    fn step_day(&mut self, m: &Reality, id: usize) -> Result<(), Stop> {
        if self.walk(id)?.today.is_none() {
            let days = self.days;
            let walk = self.walk_mut(id)?;
            let Some(day) = walk.next_day(days, &walk.endings) else {
                walk.progress = Progress::Done;
                return Ok(());
            };
            let mut acts = walk.agenda.remove(&day).unwrap_or_else(Vec::new);
            acts.sort_by_key(|a| a.0);
            walk.today = Some(Today {
                day,
                acts,
                next: 0,
                next_inst: 0,
                runs: Vec::new(),
                batch: Some(Group::new()),
                closing: None,
                touched: BTreeSet::new(),
            });
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
    /// legs at it, which are taken back (refcore's `check_group`). L §5.10, E §4.2 step 5.
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
                    walk.unbook(m, p, &unit, (mv, &leg.name), taken)?;
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
            Act::Read(reading) => self.walk_mut(id)?.apply_opening(m, at, &reading),
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
        let q = match &lg.amount {
            Amount::Refused(r) => return self.walk_mut(id)?.leg_bad(m, movement, lg, r, at),
            Amount::Hole(h) => {
                let walk = self.walk_mut(id)?;
                let unit = lg.unit.clone().unwrap_or_else(|| ALL.to_owned());
                for p in ends(lg) {
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
        let sides = [(&lg.from, -1_i64), (&lg.to, 1_i64)];
        let mut on: Vec<(String, i64)> = Vec::new();
        let mut err = None;
        let (mut kept, mut dropped) = (false, false);
        for (end, sign) in sides {
            let End::Place(p) = end else {
                kept = true;
                continue;
            };
            let thing = m
                .things
                .get(p)
                .ok_or_else(|| invariant("a leg's place load didn't build"))?;
            let span = m.lifespan(thing, cut);
            if let Some(sp) = &span
                && let Some(question) = sp.end_question()
                && sp.contains(at.day)
            {
                let mut ent = Ent::new(at, Change::Hole(Hole::asked(question, p)));
                ent.mv = Some(movement.to_owned());
                ent.leg = Some(lg.name.clone());
                ent.reg = reg.to_vec();
                ent.coll = Some(coll.clone());
                self.walk_mut(id)?.add(m, p, &unit, ent)?;
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
                "a leg booked outside every group of the sign rule (pads, G8)",
            ));
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
        for (p, sign) in &on {
            let f = signed(&q, *sign)?;
            if !dropped {
                let sum = walk.conserved.entry(unit.clone()).or_insert_with(Num::zero);
                *sum = sum.checked_add(&f).map_err(crate::eval::num_stop)?;
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
        let walk = self.walk_mut(id)?;
        let in_batch =
            at.in_first_batch() && walk.today.as_ref().is_some_and(|t| t.batch.is_some());
        let group = if in_batch {
            walk.today.as_mut().and_then(|t| t.batch.as_mut())
        } else if walk.group.is_some() {
            walk.group.as_mut()
        } else if at.at_or_after_end() {
            walk.today.as_mut().and_then(|t| t.closing.as_mut())
        } else {
            None
        };
        let Some(group) = group else {
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
        let money = accounts_and_cash(m);
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
    /// An opening reading: its place's balance set to it, the holes since answered, a stated
    /// hole's questions kept (refcore's `apply_reading`, an opening). L §5.19, E §4.2 step 0.
    fn apply_opening(&mut self, m: &Reality, at: At, reading: &Reading) -> Result<(), Stop> {
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
        self.add(m, &reading.place, &reading.unit, ent)
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
