//! `bal(x, d) modelled`: the model's own balance of the days the statement covers, from x's last reading before d.
//! Spec: E §6.2 (`bal(x, d) modelled`: x's balance at its last reading before d, the opening
//! before the first, plus every booked movement of x after it through d, with the
//! modelled-past instances in place of the observations on covered days); L §5.19 (`bal(x, d)
//! modelled`: comparing it with the reading is the residual), §8.1 (`HOLE-ANCHOR`; a hole
//! movement's questions), §8.4 (a refused balance refuses what reads it).
//! Twin: engine/tools/refcore/mreads.py (`m_bal_modelled`).
//! Invariants: a reading on day d is never its start, so the reading it is compared with
//! isn't in it; after its start, the observations' entries and the readings' resets are left
//! out and the modelled-past instances' legs in x's unit through d that stand after the start
//! in the day walk's order are counted (a leg on the day of a reading at that day's end
//! stands before it); a hole entry or leg makes it a hole with every question met; a refused
//! entry or leg refuses it. `explain` lists what the value counts (`Counted`).

use super::plan::{Amount, Dated, Inst, LegVal, invariant, qty_value};
use super::point::{At, Step};
use super::state::{Change, Ent, Walk};
use crate::eval::Engine;
use crate::hole::{Hole, spread};
use crate::model::{Reality, Value};
use crate::syntax::Stop;
use crate::time::Day;

/// What `bal(x, d) modelled` counts in a main walk: where it starts, the log entries and the
/// modelled-past legs after that start through d. E §6.2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Counted {
    /// The reading it starts from, by its place in the place's log; none where it starts from
    /// the place's opening balance.
    pub(crate) start: Option<usize>,
    /// The entries of the place's log it counts, by their place in the log: every one after
    /// its start through the end of d but the observations' and the readings'; a refused one
    /// kept, as it refuses the balance.
    pub(crate) ents: Vec<usize>,
    /// The modelled-past instances' legs it counts, by position (the instance's, the leg's),
    /// each with its sign at the place (−1 out, +1 in).
    pub(crate) legs: Vec<(usize, usize, i64)>,
}

impl Walk {
    /// What `bal(x, d) modelled` of place `place` in unit `unit` counts in this main walk: its
    /// last reading dated before day d (an opening on day d where none is, the place's opening
    /// balance before the first), each booked entry since through the end of d but the
    /// observations' and the readings', and each modelled-past instance's leg at the place
    /// that stands after that reading in the day walk's order, through d (refcore's
    /// `m_bal_modelled`). E §6.2, §4.2 (a reading at a day's end stands after its movements).
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a log or an order past its entries.
    pub(crate) fn modelled_counted(
        &self,
        m: &Reality,
        (place, unit): (&str, &str),
        key: At,
    ) -> Result<Counted, Stop> {
        let (_, last) = self.state(m, place, unit, key);
        let log = self.entries(place, unit);
        let start = modelled_start(log, last, key.day);
        let first = match start {
            Some(at) => at
                .checked_add(1)
                .ok_or_else(|| invariant("a log past its entries"))?,
            None => 0,
        };
        let through = match last {
            Some(i) => i
                .checked_add(1)
                .ok_or_else(|| invariant("a log past its entries"))?,
            None => 0,
        };
        let obs: Vec<&str> = self.obs.iter().map(|wo| wo.obs.id.as_str()).collect();
        let since_start = log
            .get(first..through)
            .ok_or_else(|| invariant("a modelled balance past its log"))?;
        let mut ents = Vec::new();
        for (j, e) in (first..).zip(since_start) {
            if matches!(e.change, Change::Bad(_) | Change::PermBad(_)) || !booked_elsewhere(e, &obs)
            {
                ents.push(j);
            }
        }
        let since = match start {
            Some(at) => Some(
                log.get(at)
                    .ok_or_else(|| invariant("a reading past its log"))?
                    .at,
            ),
            None => None,
        };
        let legs = self.modelled_legs((place, unit), key, since)?;
        Ok(Counted { start, ents, legs })
    }

    /// The legs of the modelled-past instances at place `place` in unit `unit`, by position
    /// (the instance's, the leg's), each with its sign there (−1 out, +1 in), dated (by the
    /// instance's day, a window's first) through day `d` and standing after the reading at
    /// `since` in the day walk's order (refcore's `m_bal_modelled`, its loop over
    /// `w.modelled`). E §6.2, §4.2 step 3.
    fn modelled_legs(
        &self,
        (place, unit): (&str, &str),
        key: At,
        since: Option<At>,
    ) -> Result<Vec<(usize, usize, i64)>, Stop> {
        let mut out = Vec::new();
        for (ii, inst) in self.modelled.iter().enumerate() {
            // its legs are on its day, a window's first
            let Some(gd) = inst.day.as_ref().and_then(Dated::lo) else {
                continue;
            };
            for (li, lg) in inst.legs.iter().enumerate() {
                let in_unit = lg.unit.as_deref().is_none_or(|u| u == unit);
                if gd > key.day || !in_unit {
                    continue;
                }
                // through the plain read's instant: `start` leaves day d's legs out
                let point = self.leg_point(inst, gd)?;
                if point > key || since.is_some_and(|s| point <= s) {
                    continue;
                }
                for sign in signs_at(lg, place) {
                    out.push((ii, li, sign));
                }
            }
        }
        Ok(out)
    }

    /// Where a modelled-past instance's leg dated `gd` stands in the day walk's order, as the
    /// walk books a leg: a decision's on its own day at its run's point, any other at its
    /// day's plans, at the instance's place in the movement order (`walk/order.rs` `book_run`,
    /// `walk/agenda.rs` `schedule_modelled`). E §4.2 steps 2 and 3.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a movement order past its length.
    fn leg_point(&self, inst: &Inst, gd: Day) -> Result<At, Stop> {
        if let Some(run) = self
            .runs
            .iter()
            .find(|r| r.point.day == gd && r.inst.id == inst.id)
        {
            return Ok(run.point);
        }
        Ok(At {
            rank: self.plan_rank(&inst.order)?,
            ..At::of(gd, Step::Plans)
        })
    }
}

impl Engine {
    /// `bal(x, d) modelled` of place `place` in unit `unit`, from the main walk at the read's
    /// cut: what [`Walk::modelled_counted`] counts, added to the balance its start states; a
    /// hole with every question where an entry or a leg is a hole, `HOLE-ANCHOR` with no
    /// reading or opening (refcore's `m_bal_modelled`). E §6.2, L §5.19, §8.1.
    ///
    /// # Errors
    ///
    /// The refusal an entry or a modelled-past leg holds; the refusals of the walk.
    pub(crate) fn bal_modelled(
        &mut self,
        m: &Reality,
        (place, unit): (&str, &str),
        key: At,
        cut: Day,
    ) -> Result<Value, Stop> {
        let id = self.main_walk(m, cut)?;
        let walk = self.walk(id)?;
        let counted = walk.modelled_counted(m, (place, unit), key)?;
        let ents = walk.entries(place, unit);
        let value = match counted.start {
            Some(at) => {
                let reading = ents
                    .get(at)
                    .ok_or_else(|| invariant("a reading past its log"))?;
                let Change::Reset { val, qs } = &reading.change else {
                    return Err(invariant("a modelled balance's start that is no reading"));
                };
                if let Some(hole) = qs {
                    return Ok(Value::Hole(hole.clone()));
                }
                Some(val.clone())
            }
            None => walk.default_state(m, place, unit).val,
        };
        let Some(mut q) = value else {
            return Ok(Value::Hole(walk.anchor_question(place)));
        };
        let mut holes: Vec<Hole> = Vec::new();
        let mut met = Vec::new();
        for at in &counted.ents {
            let entry = ents
                .get(*at)
                .ok_or_else(|| invariant("a modelled balance past its log"))?;
            match &entry.change {
                Change::Bad(err) | Change::PermBad(err) => return Err(Stop::Refused(err.clone())),
                Change::Delta(flow) => q = q.checked_add(flow).map_err(crate::eval::num_stop)?,
                Change::Hole(hole) | Change::Perm(hole) => holes.push(hole.clone()),
                Change::Reset { .. } | Change::ImpOn(_) | Change::Unperm(_) | Change::Mark => {}
            }
            if let Some(coll) = &entry.coll {
                met.push(coll.clone());
            }
        }
        for (ii, li, sign) in counted.legs {
            let inst = walk
                .modelled
                .get(ii)
                .ok_or_else(|| invariant("a modelled instance the walk lacks"))?;
            let lg = inst
                .legs
                .get(li)
                .ok_or_else(|| invariant("a leg the instance lacks"))?;
            met.push(inst.coll.clone());
            match &lg.amount {
                Amount::Hole(hole) => holes.push(hole.clone()),
                Amount::Refused(refusal) => return Err(Stop::Refused(refusal.clone())),
                Amount::Qty(amount) if sign < 0 => {
                    q = q.checked_sub(amount).map_err(crate::eval::num_stop)?;
                }
                Amount::Qty(amount) => {
                    q = q.checked_add(amount).map_err(crate::eval::num_stop)?;
                }
            }
        }
        for coll in &met {
            self.absorb(coll);
        }
        if let Some(hole) = spread(&holes.iter().map(Some).collect::<Vec<_>>()) {
            return Ok(Value::Hole(hole));
        }
        qty_value(m, &q, unit)
    }
}

/// The signs of leg `lg`'s ends at place `place`: −1 where it leaves, +1 where it enters
/// (refcore's `m_bal_modelled`, its `(g.frm, -1), (g.to, 1)`). E §6.2.
fn signs_at(lg: &LegVal, place: &str) -> Vec<i64> {
    [(&lg.from, -1_i64), (&lg.to, 1_i64)]
        .into_iter()
        .filter(|(end, _)| end.place() == Some(place))
        .map(|(_, sign)| sign)
        .collect()
}

/// Where a modelled balance on day `d` starts in a place's log `ents`, whose last entry
/// through the end of d is at `last`: the last reading dated before d, else an opening on
/// day d; none where neither is, so it starts from the place's opening balance (refcore's
/// `m_bal_modelled`, its start). E §6.2, L §5.19 ("a reading on day d is what the modelled
/// balance is compared with, not where it starts").
fn modelled_start(ents: &[Ent], last: Option<usize>, d: Day) -> Option<usize> {
    let before = ents.partition_point(|e| e.at < At::before(d));
    if let Some(i) = before.checked_sub(1)
        && let Some(rst) = ents.get(i).and_then(|e| e.st.rst)
    {
        return Some(rst);
    }
    let last = last?;
    (0..=last).rev().find(|j| opening_on(ents, *j, d))
}

/// Whether entry `j` of a place's log `ents` is an opening reading's reset on day `d`.
/// L §5.19.
fn opening_on(ents: &[Ent], j: usize, d: Day) -> bool {
    ents.get(j).is_some_and(|e| e.at.day == d && is_opening(e))
}

/// Whether an entry is an opening reading's reset: the walk's reset with no movement and no
/// register entry, as `walk/readings.rs`'s `apply_opening` books it (a stated anchor's names
/// itself `anchor:`, a pad's its movement). L §5.19.
fn is_opening(e: &Ent) -> bool {
    matches!(e.change, Change::Reset { .. }) && e.mv.is_none() && e.reg.is_empty()
}

/// Whether an entry after a modelled balance's start is left out of it: an observation's,
/// which the modelled past stands in for, or a reading's reset, which it is compared with
/// (refcore's `m_bal_modelled`). E §6.2.
fn booked_elsewhere(e: &Ent, obs: &[&str]) -> bool {
    matches!(e.change, Change::Reset { .. }) || e.mv.as_deref().is_some_and(|mv| obs.contains(&mv))
}

#[cfg(test)]
mod tests {
    use super::is_opening;
    use crate::num::Num;
    use crate::time::Day;
    use crate::walk::{At, Change, Ent};

    #[test]
    fn an_opening_is_a_reset_with_no_movement_and_no_register() {
        let d = Day::from_ymd(2025, 1, 1).expect("a day");
        let reset = Change::Reset {
            val: Num::zero(),
            qs: None,
        };
        let opening = Ent::new(At::start(d), reset.clone());
        assert!(is_opening(&opening), "an opening");
        let mut pad = Ent::new(At::start(d), reset);
        pad.mv = Some("pads:v:x@2025-01-01 start".to_owned());
        assert!(!is_opening(&pad), "a pad's reset");
        assert!(!is_opening(&Ent::new(At::start(d), Change::Mark)), "a mark");
    }
}
