//! `bal(x, d) modelled`: the model's own balance of the days the statement covers, from x's last reading before d.
//! Spec: E §6.2 (`bal(x, d) modelled`: x's balance at its last reading before d, the opening
//! before the first, plus every booked movement of x after it through d, with the
//! modelled-past instances in place of the observations on covered days); L §5.19 (`bal(x, d)
//! modelled`: comparing it with the reading is the residual), §8.1 (`HOLE-ANCHOR`; a hole
//! movement's questions), §8.4 (a refused balance refuses what reads it).
//! Twin: engine/tools/refcore/mreads.py (`m_bal_modelled`).
//! Invariants: a reading on day d is never its start, so the reading it is compared with
//! isn't in it; after its start, the observations' entries and the readings' resets are left
//! out and the modelled-past instances' legs in x's unit through d are counted; a hole entry
//! or leg makes it a hole with every question met; a refused entry or leg refuses it.

use super::plan::{Amount, Dated, Inst, invariant, qty_value};
use super::point::At;
use super::state::{Change, Ent};
use crate::eval::Engine;
use crate::hole::{Hole, spread};
use crate::model::{Reality, Value};
use crate::syntax::Stop;
use crate::time::Day;

impl Engine {
    /// `bal(x, d) modelled` of place `place` in unit `unit`, from the main walk at the read's
    /// cut: the balance its last reading dated before day d states (an opening on day d where
    /// none is, the place's opening balance before the first), plus each booked entry since
    /// through the end of d but the observations' and the readings', plus each modelled-past
    /// instance's leg at the place dated after that reading through d; a hole with every
    /// question where an entry or a leg is a hole, `HOLE-ANCHOR` with no reading or opening
    /// (refcore's `m_bal_modelled`). E §6.2, L §5.19, §8.1.
    ///
    /// # Errors
    ///
    /// The refusal an entry or a modelled-past leg holds; the refusals of the walk.
    pub(crate) fn bal_modelled(
        &mut self,
        m: &Reality,
        (place, unit): (&str, &str),
        d: Day,
        cut: Day,
    ) -> Result<Value, Stop> {
        let id = self.main_walk(m, cut)?;
        let walk = self.walk(id)?;
        let (_, last) = walk.state(m, place, unit, At::end(d));
        let ents = walk.entries(place, unit);
        let start = modelled_start(ents, last, d);
        let (from, value) = match start {
            Some(at) => {
                let e = ents
                    .get(at)
                    .ok_or_else(|| invariant("a reading past its log"))?;
                let Change::Reset { val, qs } = &e.change else {
                    return Err(invariant("a modelled balance's start that is no reading"));
                };
                if let Some(h) = qs {
                    return Ok(Value::Hole(h.clone()));
                }
                (Some((at, e.at)), Some(val.clone()))
            }
            None => (None, walk.default_state(m, place, unit).val),
        };
        let Some(mut q) = value else {
            return Ok(Value::Hole(walk.anchor_question(place)));
        };
        let obs: Vec<&str> = walk.obs.iter().map(|wo| wo.obs.id.as_str()).collect();
        let first = match from {
            Some((at, _)) => at
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
        let mut holes: Vec<Hole> = Vec::new();
        let mut met = Vec::new();
        let since_start = ents
            .get(first..through)
            .ok_or_else(|| invariant("a modelled balance past its log"))?;
        for e in since_start {
            if let Change::Bad(err) = &e.change {
                return Err(Stop::Refused(err.clone()));
            }
            if booked_elsewhere(e, &obs) {
                continue;
            }
            match &e.change {
                Change::Delta(f) => q = q.checked_add(f).map_err(crate::eval::num_stop)?,
                Change::Hole(h) | Change::Perm(h) => holes.push(h.clone()),
                Change::Reset { .. } | Change::Bad(_) | Change::ImpOn(_) | Change::Mark => {}
            }
            if let Some(c) = &e.coll {
                met.push(c.clone());
            }
        }
        let since = from.map(|(_, at)| at);
        for (ii, li, sign) in modelled_legs(&walk.modelled, (place, unit), d, since) {
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
                Amount::Hole(h) => holes.push(h.clone()),
                Amount::Refused(r) => return Err(Stop::Refused(r.clone())),
                Amount::Qty(a) if sign < 0 => {
                    q = q.checked_sub(a).map_err(crate::eval::num_stop)?;
                }
                Amount::Qty(a) => {
                    q = q.checked_add(a).map_err(crate::eval::num_stop)?;
                }
            }
        }
        for c in &met {
            self.absorb(c);
        }
        if let Some(h) = spread(&holes.iter().map(Some).collect::<Vec<_>>()) {
            return Ok(Value::Hole(h));
        }
        qty_value(m, &q, unit)
    }
}

/// The legs of the modelled-past instances `modelled` at place `place` in unit `unit`, by
/// position (the instance's, the leg's), each with its sign there (−1 out, +1 in), dated (by
/// the leg's day, else the instance's first) after the reading at `since` through day `d`
/// (refcore's `m_bal_modelled`, its loop over `w.modelled`). E §6.2.
fn modelled_legs(
    modelled: &[Inst],
    (place, unit): (&str, &str),
    d: Day,
    since: Option<At>,
) -> Vec<(usize, usize, i64)> {
    let mut out = Vec::new();
    for (ii, inst) in modelled.iter().enumerate() {
        let inst_day = inst.day.as_ref().and_then(Dated::lo);
        for (li, lg) in inst.legs.iter().enumerate() {
            let Some(gd) = lg.day.day().or(inst_day) else {
                continue;
            };
            let after = since.is_none_or(|s| At::end(gd) > s);
            let in_unit = lg.unit.as_deref().is_none_or(|u| u == unit);
            if gd > d || !after || !in_unit {
                continue;
            }
            for (end, sign) in [(&lg.from, -1_i64), (&lg.to, 1_i64)] {
                if end.place() == Some(place) {
                    out.push((ii, li, sign));
                }
            }
        }
    }
    out
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
