//! The walk's booking of one movement on two statements: each end of a leg stated `same` booked
//! once, waiting for a decision, or the members' `HOLE-SAME`.
//! Spec: L §6.1 (What `same` does: a place two members book takes it once, by the latest in
//! the day's order; a decision's amount is known only at its point), §5.19 (One movement on two
//! statements; Booking an observation), §5.14.2, §8.1 (`HOLE-SAME`'s form), §8.4 (`same <p> =
//! <q>: <p> doesn't exist`); E §4.2 steps 1–3, §6.1 (one movement books each end once).
//! Twin: engine/tools/refcore/walk.py (`book_leg`, its `consol`; `same_wait_for`,
//! `same_book_key`, `same_release`, `inst_places`), moves.py (`movement`, `decs`).
//! Invariants: an end is booked once per group, by its latest member in the day's order where
//! the members agree; where they give it different amounts or days it is the `HOLE-SAME` hole
//! naming every member in the order of their ids; an end waiting for a decision is booked, or
//! refused, at the decision's point.

use std::collections::BTreeSet;

use super::observe::Obs;
use super::same::{DayCmp, and_list, compare_days, shown};
use crate::base::{Code, Refusal, Run};
use crate::eval::{Collected, Engine, EntryId, num_stop};
use crate::hole::Hole;
use crate::model::{Reality, Value};
use crate::syntax::Stop;
use crate::time::Day;
use crate::walk::{
    ALL, Amount, At, Change, Dated, End, Ends, Ent, Exists, Inst, LegVal, Status, Step, invariant,
};

/// What one end of a leg books under the `same` lines (refcore's `consol`). L §6.1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EndBook {
    /// As written: no other member of its group books that place.
    Written,
    /// Nothing here: the group's latest member there books it.
    Skip,
    /// At the point of the decision named, which hasn't run yet.
    Wait(String),
    /// The group's disagreement there, a hole movement (`HOLE-SAME`), or a side's hole day
    /// where the amounts agree (S1, P43 on #241).
    Hole(Hole),
    /// Instances that agree on the amount and differ on the day, at the first of their days:
    /// not booked here, the place that `HOLE-SAME` hole until the last (tally 2.3.1 R1,
    /// N245.21).
    Open(Hole),
    /// The same on a day between the first and the last: not booked, its register met there,
    /// so the result lists every side (tally 2.3.1, rulings 14 on G1's review N1).
    Between,
    /// The same at the last of their days: booked as written, ending that hole.
    Close(Hole),
}

/// A member of a `same` group as the walk meets it (refcore's `movement`, `decs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Member {
    /// An observed row.
    Obs(Obs),
    /// A plan's instance.
    Plan(Inst),
    /// A decision's instance: its point once it has run, none before.
    Decision(Inst, Option<At>),
}

impl Member {
    /// Its stable id. L §2.5.
    fn id(&self) -> &str {
        match self {
            Member::Obs(o) => &o.id,
            Member::Plan(i) | Member::Decision(i, _) => &i.id,
        }
    }

    /// Its legs. L §5.14 Legs.
    fn legs(&self) -> &[LegVal] {
        match self {
            Member::Obs(o) => &o.legs,
            Member::Plan(i) | Member::Decision(i, _) => &i.legs,
        }
    }

    /// Its day. L §6.1.
    fn day(&self) -> Option<Day> {
        match self {
            Member::Obs(o) => Some(o.day),
            Member::Plan(i) | Member::Decision(i, _) => i.day.as_ref().and_then(Dated::day),
        }
    }
}

/// An end waiting for a decision's point: the movement, its leg, the side, what it carries.
pub(crate) type Waiting = (String, LegVal, usize, Vec<EntryId>, Collected);

/// A leg's amount as a `HOLE-SAME` question prints it: the amount, `refused`, or the hole.
/// L §8.1.
fn amount_text(m: &Reality, lg: &LegVal) -> Result<String, Stop> {
    match (&lg.amount, &lg.unit) {
        (Amount::Qty(q), Some(unit)) => shown(m, q, unit),
        (Amount::Qty(_), None) => Err(invariant("an amount with no unit")),
        (Amount::Refused(_), _) => Ok("refused".to_owned()),
        (Amount::Hole(h), _) => Value::Hole(h.clone()).text(m).map_err(num_stop),
    }
}

impl Engine {
    /// The ends of leg `lg` of `mover` its booking keeps under the `same` lines: an end the
    /// group's latest member books instead, or one waiting for a decision's point, is dropped;
    /// one where the members disagree takes the `HOLE-SAME` hole (refcore's `book_leg`, its
    /// `consol`). L §6.1 (a place two of them book takes it once), §5.19, E §6.1 (one
    /// movement books each end once).
    ///
    /// # Errors
    ///
    /// `RUN-RECORD` for a `same` side that doesn't exist; a refusal reading a waiting
    /// decision's places.
    pub(crate) fn same_mask(
        &mut self,
        m: &Reality,
        id: usize,
        (mover, lg): (&Member, &LegVal),
        at: At,
        (reg, coll): (&[EntryId], &Collected),
        mut mask: Ends,
    ) -> Result<Ends, Stop> {
        let books = self.same_ends(m, id, (mover, lg))?;
        for (side, book) in books.into_iter().enumerate() {
            let (on, end) = if side == 0 {
                (&mut mask.from, &lg.from)
            } else {
                (&mut mask.to, &lg.to)
            };
            if !*on {
                continue;
            }
            match book {
                EndBook::Written => continue,
                EndBook::Skip => {}
                EndBook::Open(h) => {
                    self.same_entry(m, id, (mover, lg, end), (at, reg, coll), Change::Open(h))?;
                }
                EndBook::Between => {
                    self.same_entry(m, id, (mover, lg, end), (at, reg, coll), Change::Mark)?;
                }
                EndBook::Close(h) => {
                    self.same_entry(m, id, (mover, lg, end), (at, reg, coll), Change::Close(h))?;
                    // the last day books the end as written
                    continue;
                }
                EndBook::Wait(decision) => {
                    let waiting = (
                        mover.id().to_owned(),
                        lg.clone(),
                        side,
                        reg.to_vec(),
                        coll.clone(),
                    );
                    let walk = self.walk_mut(id)?;
                    walk.same_waiting
                        .entry(decision)
                        .or_insert_with(Vec::new)
                        .push(waiting);
                }
                EndBook::Hole(h) => {
                    if end.place().is_none() {
                        continue;
                    }
                    let unit = lg.unit.clone().unwrap_or_else(|| ALL.to_owned());
                    self.walk_mut(id)?.worth_add(&unit, None)?;
                    self.same_entry(m, id, (mover, lg, end), (at, reg, coll), Change::Hole(h))?;
                }
            }
            *on = false;
        }
        Ok(mask)
    }

    /// An entry `change` of `mover`'s leg `lg` at its end `end`: the group's `HOLE-SAME` hole,
    /// or a movement of two days' first day, a day between, or its last (tally 2.3.1 R1,
    /// N245.21). L §6.1, E §4.2.
    ///
    /// # Errors
    ///
    /// A refusal adding the entry.
    fn same_entry(
        &mut self,
        m: &Reality,
        id: usize,
        (mover, lg, end): (&Member, &LegVal, &End),
        (at, reg, coll): (At, &[EntryId], &Collected),
        change: Change,
    ) -> Result<(), Stop> {
        let Some(p) = end.place() else {
            return Ok(());
        };
        let mut ent = Ent::new(at, change);
        ent.mv = Some(mover.id().to_owned());
        ent.leg = Some(lg.name.clone());
        ent.reg = reg.to_vec();
        ent.coll = Some(coll.clone());
        let unit = lg.unit.clone().unwrap_or_else(|| ALL.to_owned());
        self.walk_mut(id)?.add(m, p, &unit, ent)
    }

    /// Instance `inst` as a member of its `same` group: a decision with its point once this
    /// walk ran it, else a plan's instance (refcore's `consol`, its `mv`). L §6.1.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a walk the engine lacks.
    pub(crate) fn mover_of(&self, m: &Reality, id: usize, inst: &Inst) -> Result<Member, Stop> {
        if !m.moves.get(&inst.move_id).is_some_and(|mv| mv.decision) {
            return Ok(Member::Plan(inst.clone()));
        }
        let walk = self.walk(id)?;
        let point = walk
            .run_ids
            .get(&inst.id)
            .and_then(|&ix| walk.runs.get(ix))
            .map(|run| run.point);
        Ok(Member::Decision(inst.clone(), point))
    }

    /// What each end of leg `lg` of `mover` books under the `same` lines (refcore's
    /// `consol`). L §6.1 What `same` does, §8.4 (`same <p> = <q>: <p> doesn't exist`).
    pub(super) fn same_ends(
        &mut self,
        m: &Reality,
        id: usize,
        (mover, lg): (&Member, &LegVal),
    ) -> Result<[EndBook; 2], Stop> {
        let cut = self.walk(id)?.cut;
        let groups = self.same_groups(m, cut)?;
        let Some(group) = groups.get(mover.id()) else {
            return Ok([EndBook::Written, EndBook::Written]);
        };
        let decides = matches!(mover, Member::Decision(..));
        let (mut others, mut waiting) = (Vec::new(), Vec::new());
        for x in group.iter().filter(|x| x.as_str() != mover.id()) {
            let Some(found) = self.walk_member(m, id, x)? else {
                let (left, right) = self
                    .first_line(m, (mover.id(), x), cut)?
                    .ok_or_else(|| invariant("a same group's member no same line names"))?;
                return Err(Stop::Refused(Refusal::new(
                    Code::Run(Run::Record),
                    format!("same {left} = {right}: {x} doesn't exist"),
                )));
            };
            if matches!(found, Member::Decision(_, None)) && !decides {
                waiting.push(found);
            } else {
                others.push(found);
            }
        }
        let mut out = [EndBook::Written, EndBook::Written];
        for (side, (end, slot)) in [&lg.from, &lg.to]
            .into_iter()
            .zip(out.iter_mut())
            .enumerate()
        {
            let Some(p) = end.place() else {
                continue;
            };
            *slot = match self.same_wait_for(m, id, (mover, lg), p, &waiting)? {
                Some(wait) => wait,
                None => self.same_end(m, id, (mover, lg, side), p, &others)?,
            };
        }
        Ok(out)
    }

    /// A member of a `same` group by id: a decision as this walk ran it, else a plan's or a
    /// decision's instance, else an observation; none where none is (refcore's `decs`,
    /// `movement`). L §6.1.
    fn walk_member(&mut self, m: &Reality, id: usize, x: &str) -> Result<Option<Member>, Stop> {
        let walk = self.walk(id)?;
        if let Some(run) = walk.run_ids.get(x).and_then(|&ix| walk.runs.get(ix)) {
            return Ok(Some(Member::Decision(run.inst.clone(), Some(run.point))));
        }
        let cut = walk.cut;
        let name = x.split_once('[').map_or(x, |(n, _)| n);
        if let Some(mv) = m.moves.get(name) {
            let insts = self.plan_list(m, mv, cut)?;
            if let Some(inst) = insts.iter().find(|i| i.id == x) {
                let found = if mv.decision {
                    Member::Decision(inst.clone(), None)
                } else {
                    Member::Plan(inst.clone())
                };
                return Ok(Some(found));
            }
        }
        let rows = self.observations(m, cut)?;
        Ok(rows.iter().find(|o| o.id == x).cloned().map(Member::Obs))
    }

    /// An end at place `p` of a movement stated `same` as a decision not yet run: waiting for
    /// the decision's point where it books `p` that day, the `HOLE-SAME` hole where their days
    /// differ, none where no waiting decision books `p` (refcore's `same_wait_for`). L §6.1
    /// (a decision's amount is known only at its point), §5.14.2.
    fn same_wait_for(
        &mut self,
        m: &Reality,
        id: usize,
        (mover, lg): (&Member, &LegVal),
        p: &str,
        waiting: &[Member],
    ) -> Result<Option<EndBook>, Stop> {
        for dec in waiting {
            let Member::Decision(inst, _) = dec else {
                continue;
            };
            let Some(dday) = inst.day.as_ref().and_then(Dated::day) else {
                return Ok(None);
            };
            if !self.decision_books(m, id, inst, (dday, p))? {
                continue;
            }
            if lg.day == Dated::Day(dday) {
                return Ok(Some(EndBook::Wait(inst.id.clone())));
            }
            let text = format!(
                "{} and {} are one, but the leg at {p} differs on {}: {} ({}) and its day {} ({})",
                mover.id(),
                inst.id,
                lg.day.text(),
                amount_text(m, lg)?,
                mover.id(),
                dday.text(),
                inst.id
            );
            let figure = format!("{}, {}", mover.id(), inst.id);
            return Ok(Some(EndBook::Hole(Hole::asked(&text, &figure))));
        }
        Ok(None)
    }

    /// Whether decision instance `inst` books place `p` on its day: the places its legs give
    /// there, every place where one can't be read (refcore's `inst_places`, its changes).
    /// L §5.14.2, §6.1.
    fn decision_books(
        &mut self,
        m: &Reality,
        id: usize,
        inst: &Inst,
        (day, p): (Day, &str),
    ) -> Result<bool, Stop> {
        let mv = m
            .moves
            .get(&inst.move_id)
            .ok_or_else(|| invariant("a decision of a move load didn't build"))?;
        let cut = self.walk(id)?.cut;
        let ctx = self.timeline_ctx(inst.law.day(), day.min(cut));
        self.push();
        let read = self.leg_places(m, mv, inst, ctx);
        self.pop_alone();
        match read {
            Ok(Some(places)) => Ok(places.contains(p)),
            // a place that can't be read here meets every place
            Ok(None) | Err(Stop::Refused(_)) => Ok(true),
            Err(stop) => Err(stop),
        }
    }

    /// What place `p` takes from `mover`'s end there: as written where no other member
    /// books it; once, by the latest member in the day's order, where they agree; the
    /// `HOLE-SAME` hole where they give it different amounts or days, naming the leg (refcore's
    /// `consol`). L §6.1 What `same` does, §8.1; tally 2.2.1 237.2 (`leg fee`).
    fn same_end(
        &mut self,
        m: &Reality,
        id: usize,
        (mover, lg, side): (&Member, &LegVal, usize),
        p: &str,
        others: &[Member],
    ) -> Result<EndBook, Stop> {
        let mut touch: Vec<(&Member, &LegVal, usize)> = vec![(mover, lg, side)];
        for o in others {
            if matches!(o, Member::Plan(i) if i.status == Status::Modelled) {
                // a plan instance modelled past is never booked (L §5.19 Coverage)
                continue;
            }
            let mut theirs = Vec::new();
            for (li, sj) in self.touching(m, id, o, (p, lg.unit.as_deref()))? {
                let g = o
                    .legs()
                    .get(li)
                    .ok_or_else(|| invariant("a leg the member lacks"))?;
                theirs.push((g, sj));
            }
            // each leg is compared with the other side's of its name, where it states one: two
            // movements that agree in full leave no hole (L §6.1; N6, P36 on #241)
            if theirs.iter().any(|(g, _)| g.name == lg.name) {
                theirs.retain(|(g, _)| g.name == lg.name);
            }
            touch.extend(theirs.into_iter().map(|(g, sj)| (o, g, sj)));
        }
        let mut movers: Vec<&str> = touch.iter().map(|(x, _, _)| x.id()).collect();
        movers.dedup();
        if movers.len() < 2 {
            return Ok(EndBook::Written);
        }
        let amounts_agree = touch
            .iter()
            .skip(1)
            .all(|(_, g, sj)| *sj == side && g.amount == lg.amount);
        // the days by the one comparison of S1: a hole day's question, the undecided days'
        // questions for a balance and no `HOLE-SAME` of equal amounts (P43 on #241)
        let mut sides: Vec<(&str, &Dated)> =
            touch.iter().map(|(x, g, _)| (x.id(), &g.day)).collect();
        sides.sort_by_cached_key(|(x, _)| m.id_order(x));
        let (agree, two_days) = match compare_days(&sides) {
            DayCmp::Hole(h) if amounts_agree => return Ok(EndBook::Hole(h)),
            DayCmp::Agree => (amounts_agree, false),
            // instances, a plan's or a decision's, that agree on the amount and differ on the
            // day: `same`'s two days are its bounds (L §5.14 `on d`; tally 2.3.1 R1, N245.21,
            // rulings 14); an observed row books as written
            DayCmp::Differ(_) => (
                false,
                amounts_agree && !touch.iter().any(|(x, _, _)| matches!(x, Member::Obs(_))),
            ),
            DayCmp::Hole(_) => (false, false),
        };
        // the first and the last member in the day's order, the latest booking it
        let mut first: Option<(Option<At>, &str)> = None;
        let mut latest: Option<(Option<At>, &str)> = None;
        for (x, g, _) in touch.iter().filter(|_| agree || two_days) {
            let key = self.book_key(id, x, g)?;
            if latest.as_ref().is_none_or(|(best, _)| key > *best) {
                latest = Some((key, x.id()));
            }
            if first.as_ref().is_none_or(|(best, _)| key < *best) {
                first = Some((key, x.id()));
            }
        }
        let mine = latest.is_some_and(|(_, x)| x == mover.id());
        if agree {
            return Ok(if mine {
                EndBook::Written
            } else {
                EndBook::Skip
            });
        }
        let opens = first.is_some_and(|(_, x)| x == mover.id());
        // the members in the order of their ids, every one named (tally 2.1.8 S6, P50)
        let mut ordered = touch;
        ordered.sort_by_cached_key(|(x, _, _)| m.id_order(x.id()));
        let ids: Vec<String> = ordered.iter().map(|(x, _, _)| x.id().to_owned()).collect();
        let days: BTreeSet<Day> = ordered.iter().filter_map(|(_, g, _)| g.day.day()).collect();
        let days: Vec<String> = days.into_iter().map(Day::text).collect();
        let mut parts = Vec::with_capacity(ordered.len());
        for (x, g, _) in &ordered {
            parts.push(format!("{} ({})", amount_text(m, g)?, x.id()));
        }
        // the leg by its name where every side's has it, one question per leg that differs
        // (tally 2.2.1 237.2); the place's leg where their names differ
        let what = if ordered.iter().all(|(_, g, _)| g.name == lg.name) {
            format!("leg {}", lg.name)
        } else {
            format!("the leg at {p}")
        };
        let text = format!(
            "{} are one, but {what} differs on {}: {}",
            and_list(&ids),
            days.join(", "),
            and_list(&parts)
        );
        let hole = Hole::asked(&text, &ids.join(", "));
        Ok(match (two_days, mine, opens) {
            (true, true, _) => EndBook::Close(hole),
            (true, false, true) => EndBook::Open(hole),
            (true, false, false) => EndBook::Between,
            (false, _, _) => EndBook::Hole(hole),
        })
    }

    /// The legs of member `o` that touch place `p` in unit `unit`, by position, each with
    /// its side; an observed row's only where it books that end (refcore's `consol`, its
    /// `touch`). L §6.1, §5.19 Booking an observation.
    fn touching(
        &mut self,
        m: &Reality,
        id: usize,
        o: &Member,
        (p, unit): (&str, Option<&str>),
    ) -> Result<Vec<(usize, usize)>, Stop> {
        let cut = self.walk(id)?.cut;
        let mut out = Vec::new();
        for (li, g) in o.legs().iter().enumerate() {
            if g.unit.as_deref() != unit {
                continue;
            }
            for (sj, end) in [&g.from, &g.to].into_iter().enumerate() {
                if end.place() != Some(p) {
                    continue;
                }
                if let Member::Obs(row) = o
                    && !self.obs_end_on(m, row, p, row.day, cut)
                {
                    continue;
                }
                out.push((li, sj));
            }
        }
        Ok(out)
    }

    /// Where member `x`'s leg `lg` is booked in the day's order: observations before plans,
    /// plans before decisions, a decision at its point (refcore's `same_book_key`). E §4.2
    /// steps 1–3, L §6.1.
    fn book_key(&self, id: usize, x: &Member, lg: &LegVal) -> Result<Option<At>, Stop> {
        // a leg is on its member's day
        let Some(day) = x.day() else {
            return Ok(None);
        };
        let walk = self.walk(id)?;
        Ok(Some(match x {
            Member::Obs(o) => At {
                rank: walk.obs_rank(&format!("{} {}", o.id, lg.name))?,
                ..At::of(day, Step::Observations)
            },
            Member::Decision(_, Some(point)) if point.day == day => *point,
            Member::Decision(..) => At::of(day, Step::Decisions),
            Member::Plan(i) => At {
                rank: walk.plan_rank(&i.order)?,
                ..At::of(day, Step::Plans)
            },
        }))
    }

    /// At decision run `ix`'s point, the ends that waited for it: refused where it doesn't
    /// exist; nothing where it books their place (its booking took the end once, or found the
    /// hole); else booked by their own movement now (refcore's `same_release`). L §6.1, §8.4.
    ///
    /// # Errors
    ///
    /// As [`Engine::book_leg_ends`].
    pub(crate) fn same_release(&mut self, m: &Reality, id: usize, ix: usize) -> Result<(), Stop> {
        let walk = self.walk_mut(id)?;
        if walk.same_waiting.is_empty() {
            return Ok(());
        }
        let run = walk
            .runs
            .get(ix)
            .ok_or_else(|| invariant("a run the walk lacks"))?;
        let Some(pending) = walk.same_waiting.remove(&run.inst.id) else {
            return Ok(());
        };
        let (run_id, point) = (run.inst.id.clone(), run.point);
        let none = run.inst.exists == Exists::No;
        let legs = if run.inst.status == Status::Modelled {
            Vec::new()
        } else {
            run.inst.legs.clone()
        };
        let after = point.with_sub(
            point
                .sub
                .checked_add(1)
                .ok_or_else(|| invariant("a point past the counted subpoints"))?,
        );
        let cut = self.walk(id)?.cut;
        for (mover, lg, side, reg, coll) in pending {
            if none {
                // the line that joins the waiting movement to the decision, as written, its
                // sides in the text's order (2.1.7's R9; 230.Q-5a, P48 N4 on #241)
                let (left, right) = self
                    .first_line(m, (&mover, &run_id), cut)?
                    .ok_or_else(|| invariant("a decision a mover waits for no same line names"))?;
                let err = Refusal::new(
                    Code::Run(Run::Record),
                    format!("same {left} = {right}: {run_id} doesn't exist"),
                );
                self.walk_mut(id)?
                    .leg_bad(m, (&mover, &lg, &coll), &err, point)?;
                continue;
            }
            let end = if side == 0 { &lg.from } else { &lg.to };
            let Some(p) = end.place() else {
                continue;
            };
            let hit = legs.iter().any(|g| {
                g.unit == lg.unit && (g.from.place() == Some(p) || g.to.place() == Some(p))
            });
            if hit {
                continue;
            }
            let ends = Ends {
                from: side == 0,
                to: side == 1,
                whole: false,
            };
            self.book_leg_ends(m, id, (&mover, &lg), after, (&reg, &coll), ends)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::api::{Asks, Files, Given, Request, run};

    /// Two planned movements stated `same`, of one amount, on 5 and 7 March.
    const TWO_DAYS: &str = r#"model two_days
  tally 2
  on 2026-06-10
  start 2026-01-01
  end 2026-12-31

source s "invented values"

unit PLN: currency minor 0.01

place wallet: accounts PLN lives 2025-01-01..

place world: outside lives 1900-01-01..

input d1: date

input d2: date

input a: money PLN

move in1: transfers
  on d1
  leg cash from world to wallet amount a

move in2: transfers
  on d2
  leg cash from world to wallet amount a

version v
  cite s
  governs always
  known 2025-12-01
  opening[wallet] = 0.00 PLN at 2026-01-01 start "s"
  d1 = 2026-03-05 "s"
  d2 = 2026-03-07 "s"
  a = 100.00 PLN "s"

version later
  cite s
  governs always
  known 2026-05-01
  same in1 = in2

ask b_before: bal(wallet, 2026-03-04)

ask b_between: bal(wallet, 2026-03-06)

ask b_after: bal(wallet, 2026-03-31)

ask f_both: flows(wallet, 2026-03)

ask f_one: flows(wallet, 2026-03-01 .. 2026-03-06)
"#;

    #[test]
    fn a_same_movement_of_two_days_is_known_after_both() {
        // L §5.14 `on d`, §6.1 (tally 2.3.1 R1, N245.21): `same`'s two days are its bounds, so
        // the balance is known before the first and after the last, the hole between them; a
        // span holding both days counts it, one holding one day is that hole
        let given = vec![Given {
            path: "two_days.tally".to_owned(),
            bytes: Some(TWO_DAYS.as_bytes().to_vec()),
        }];
        let request = Request::new(Asks::Every);
        let answer = run(Files::new(given), &request).expect("an answer");
        let hole = "?\"in1 and in2 are one, but leg cash differs on 2026-03-05, 2026-03-07: 100.00 PLN (in1) and 100.00 PLN (in2)\" (in1, in2)";
        let values: Vec<&str> = answer
            .lines()
            .iter()
            .filter(|line| line.starts_with("b_") || line.starts_with("f_"))
            .map(String::as_str)
            .collect();
        assert_eq!(
            values,
            [
                "b_after: 100.00 PLN".to_owned(),
                "b_before: 0.00 PLN".to_owned(),
                format!("b_between: {hole}"),
                "f_both: 100.00 PLN".to_owned(),
                format!("f_one: {hole}")
            ],
            "known outside the two days, the hole between"
        );
    }

    /// Three plans of one amount on three days, stated `same` in two lines.
    const THREE_DAYS: &str = r#"model three_days
  tally 2
  on 2026-06-10
  start 2026-01-01
  end 2026-12-31

source s "invented values"

unit PLN: currency minor 0.01

place wallet: accounts PLN lives 2025-01-01..

place world: outside lives 1900-01-01..

input d1: date

input d2: date

input d3: date

input a: money PLN

move p1: transfers
  on d1
  leg cash from world to wallet amount a

move p2: transfers
  on d2
  leg cash from world to wallet amount a

move p3: transfers
  on d3
  leg cash from world to wallet amount a

version v
  cite s
  governs always
  known 2025-12-01
  opening[wallet] = 0.00 PLN at 2026-01-01 start "s"
  d1 = 2026-02-03 "s"
  d2 = 2026-02-05 "s"
  d3 = 2026-02-07 "s"
  a = 25.00 PLN "s"
  same p1 = p2
  same p2 = p3

ask b_after: bal(wallet, 2026-02-28)
"#;

    #[test]
    fn a_three_sided_two_day_result_lists_every_side() {
        // tally 2.3.1, rulings 14 on G1's review N1: the middle side books nothing, but the
        // result lists it beside the first and the last (L §6.1, §8.2)
        let given = vec![Given {
            path: "three_days.tally".to_owned(),
            bytes: Some(THREE_DAYS.as_bytes().to_vec()),
        }];
        let answer = run(Files::new(given), &Request::new(Asks::Every)).expect("an answer");
        let lines: Vec<&str> = answer
            .lines()
            .iter()
            .map(String::as_str)
            .filter(|line| !line.starts_with("  cut"))
            .collect();
        assert_eq!(
            lines,
            [
                "b_after: 25.00 PLN",
                "  unfulfilled: p1",
                "  unfulfilled: p2",
                "  unfulfilled: p3"
            ],
            "every side the lines name"
        );
    }

    /// A decision on the first day, stated `same` as a plan on the second, of one amount.
    const DECISION_FIRST: &str = r#"model decision_first
  tally 2
  on 2026-06-10
  start 2026-01-01
  end 2026-12-31

source s "invented values"

unit PLN: currency minor 0.01

place wallet: accounts PLN lives 2025-01-01..

place world: outside lives 1900-01-01..

input d1: date

input d2: date

input a: money PLN

move top: transfers
  reads wallet
  on d1
  leg cash from world to wallet amount a + 0 * bal(wallet)

move pay: transfers
  on d2
  leg cash from world to wallet amount a

version v
  cite s
  governs always
  known 2025-12-01
  opening[wallet] = 0.00 PLN at 2026-01-01 start "s"
  d1 = 2026-04-02 "s"
  d2 = 2026-04-09 "s"
  a = 40.00 PLN "s"
  same top = pay

ask b_between: bal(wallet, 2026-04-05)

ask b_after: bal(wallet, 2026-04-30)
"#;

    #[test]
    fn a_decision_side_of_two_days_is_bounded_too() {
        // L §5.14 `on d`, §6.1 (tally 2.3.1, rulings 14 on G1's review B5): the two days
        // bound every instance, a decision's too: known after both, the hole between
        let given = vec![Given {
            path: "decision_first.tally".to_owned(),
            bytes: Some(DECISION_FIRST.as_bytes().to_vec()),
        }];
        let answer = run(Files::new(given), &Request::new(Asks::Every)).expect("an answer");
        let values: Vec<&str> = answer
            .lines()
            .iter()
            .filter(|line| line.starts_with("b_"))
            .map(String::as_str)
            .collect();
        assert_eq!(
            values,
            [
                "b_after: 40.00 PLN",
                "b_between: ?\"pay and top are one, but leg cash differs on 2026-04-02, 2026-04-09: 40.00 PLN (pay) and 40.00 PLN (top)\" (pay, top)"
            ],
            "known after both days"
        );
    }
}
