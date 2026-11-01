//! The written projection `bal(x, d) from t`: a walk of its own from day t, its holder's dependents left out.
//! Spec: L §5.14.1 (The projection is written: its start at t, or at a decision's point on t;
//! the movements dated t through d counted as known on t; those that depend on the holder left
//! out and listed; `with uncertain`; `HOLE-PRERUN` before S; `RUN-FUTURE`), E §4.3 (steps 1–3;
//! nesting is finite), §3.2 (the projection in a cell's context), L §8.2 (`left out:`).
//! Twin: engine/tools/refcore/mreads.py (`proj_bal`, `proj_regs`, `proj_uncertain`,
//! `proj_clauses`) and walk.py (`proj_walk`).
//! Invariants: a projection's walk starts each place as the walk it continues stood at its
//! point and books nothing of a move that depends on its holder; there is one walk per (walk
//! continued, point, cut, moves left out), walked only as far as a read needs; a decision
//! reads no projection from a day after its own.

use std::collections::{BTreeMap, BTreeSet};

use super::plan::invariant;
use super::point::At;
use super::state::{Projected, St, Walk};
use crate::base::{Code, Refusal, Run};
use crate::eval::{Ctx, Engine, Env};
use crate::hole::Hole;
use crate::model::{Reality, Value};
use crate::syntax::{Bal, Stop};
use crate::time::Day;

/// A projection's walk by what makes it: the walk it continues, the point it starts after,
/// its cut, the moves it leaves out (refcore's `proj_walk` key).
pub type ProjKey = (usize, At, Day, BTreeSet<String>);

impl Engine {
    /// `bal(x, d) from t` at day `d`: x's balance at the end of d in the walk continued from
    /// day t (refcore's `proj_bal`). L §5.14.1, E §4.3.
    ///
    /// # Errors
    ///
    /// `RUN-DOMAIN` for a day before the projection's start or a start that isn't a day;
    /// `RUN-FUTURE` for a decision's projection from a later day; the refusals of its walk;
    /// `NotBuilt` for a projection whose holder S doesn't track.
    pub(crate) fn projection(
        &mut self,
        m: &Reality,
        read: &Bal,
        (place, unit): (&str, &str),
        day: Day,
        (env, ctx): (&Env, Ctx),
    ) -> Result<Value, Stop> {
        let from = read
            .proj
            .from
            .as_ref()
            .ok_or_else(|| invariant("a projection with no start"))?;
        let start = self.ev(m, from, env, ctx)?;
        if let Value::Hole(_) = start {
            return Ok(start);
        }
        let start = crate::ask::date_of(&start, "the day of a projection")?;
        if day < start {
            return Err(refuse(
                Run::Domain,
                format!(
                    "bal({place}, {}) from {}: the day is before the projection's start",
                    day.text(),
                    start.text()
                ),
            ));
        }
        if start < m.start {
            return Ok(Value::Hole(Hole::asked("before the run's start", place)));
        }
        let left_out = self.holder_left_out(m)?;
        let id = self.proj_walk(m, ctx, start, left_out)?;
        if let Some(at) = ctx.at {
            self.note_live(at.walk, place)?;
        }
        self.walk_projection(m, id, day)?;
        let value = self.read_state(m, id, place, unit, At::end(day))?;
        self.proj_regs(id, start, day)?;
        if (read.proj.uncertain || self.uncertain) && !matches!(value, Value::Hole(_)) {
            return self.proj_uncertain(m, id, (place, unit), day, &value);
        }
        Ok(value)
    }

    /// The moves a projection leaves out: those that depend on the item holding it, none for
    /// an ask's line (refcore's `_leftout`, from the item graph). L §5.14.1, E §4.3 step 3.
    fn holder_left_out(&self, m: &Reality) -> Result<BTreeSet<String>, Stop> {
        let holder = self.holder.as_ref().ok_or(Stop::NotBuilt(
            "a projection outside a def, a band, a move or an ask",
        ))?;
        m.graph
            .left_out_of(holder)
            .cloned()
            .ok_or_else(|| invariant("a projection's holder the item graph lacks"))
    }

    /// The walk of a projection from day `t`: from the main walk at the start of t, or from
    /// the reading decision's walk, at its point on t or at the start of an earlier t; its
    /// cut min(t, K) (refcore's `proj_walk`). E §4.3 step 1, L §5.14.1.
    fn proj_walk(
        &mut self,
        m: &Reality,
        ctx: Ctx,
        t: Day,
        left_out: BTreeSet<String>,
    ) -> Result<usize, Stop> {
        let (cut, base, point) = match ctx.at {
            Some(at) => {
                let day = at.point.day;
                if t > day {
                    return Err(refuse(
                        Run::Future,
                        format!(
                            "a decision on {} reads a projection from {}, after its day",
                            day.text(),
                            t.text()
                        ),
                    ));
                }
                let point = if t == day { at.point } else { At::start(t) };
                (self.walk(at.walk)?.cut, at.walk, point)
            }
            None => (ctx.cut, self.main_walk(m, ctx.cut)?, At::start(t)),
        };
        let c = t.min(cut);
        let key: ProjKey = (base, point, c, left_out);
        if let Some(id) = self.proj_walks.get(&key) {
            return Ok(*id);
        }
        let walk = self.continued(m, (base, point), (c, t), key.3.clone())?;
        let id = self.walks.len();
        self.walks.push(walk);
        self.proj_walks.insert(key, id);
        Ok(id)
    }

    /// A new projection's walk of cut `c` from day `t`, continuing walk `base` after `point`:
    /// each place starts as that walk stood there, and a place's first reading is the one its
    /// cut sees, else the walk's it continues (refcore's `Walk(base=…, point=…)` and
    /// `anchor_question`). E §4.3 step 1, L §8.1.
    fn continued(
        &self,
        m: &Reality,
        (base, point): (usize, At),
        (c, t): (Day, Day),
        left_out: BTreeSet<String>,
    ) -> Result<Walk, Stop> {
        let from = self.walk(base)?;
        let mut states: BTreeMap<(String, String), St> = BTreeMap::new();
        for (place, unit) in from.logs.keys() {
            let (st, _) = from.state(m, place, unit, point);
            states.insert((place.clone(), unit.clone()), st);
        }
        let mut levels = vec![states];
        let root = match &from.proj {
            None => (from.cut, from.start),
            Some(p) => {
                levels.extend(p.levels.iter().cloned());
                p.root
            }
        };
        let mut first_reading = super::agenda::first_readings(m, c)?;
        for (place, day) in &from.first_reading {
            first_reading.entry(place.clone()).or_insert(*day);
        }
        let projected = Projected {
            base,
            point,
            left_out,
            counted: BTreeMap::new(),
            levels,
            root,
        };
        Ok(Walk::projection(c, (t, m.end), projected, first_reading))
    }

    /// A projection's walk walked through day `d` (refcore's `run_to`). E §4.3.
    ///
    /// # Errors
    ///
    /// The refusal the walk met; `Deep` while a cell it needs waits on the work stack;
    /// `NotBuilt` for a read of the walk while it is walked.
    fn walk_projection(&mut self, m: &Reality, id: usize, d: Day) -> Result<(), Stop> {
        if self.walking.contains(&id) {
            return Err(Stop::NotBuilt("a read of a walk while it is walked"));
        }
        self.walking.insert(id);
        let walked = self.advance_to(m, id, Some(d));
        self.walking.remove(&id);
        walked
    }

    /// The `left out:` line of each move the projection leaves out with an instance dated in
    /// its window t through d (refcore's `proj_regs`). L §5.14.1, §8.2; E §4.3 step 3.
    fn proj_regs(&mut self, id: usize, t: Day, d: Day) -> Result<(), Stop> {
        let p = self
            .walk(id)?
            .proj
            .as_ref()
            .ok_or_else(|| invariant("a projection read of a main walk"))?;
        let mut lines = Vec::new();
        for (mid, days) in &p.counted {
            let inside: Vec<Day> = days
                .iter()
                .copied()
                .filter(|x| t <= *x && *x <= d)
                .collect();
            if let Some(first) = inside.iter().min() {
                lines.push(format!(
                    "{mid}, {} instances, first {}",
                    inside.len(),
                    first.text()
                ));
            }
        }
        for line in lines {
            self.note("left out", line, String::new());
        }
        Ok(())
    }
}

/// A run-time refusal (L §8.4).
fn refuse(code: Run, message: String) -> Stop {
    Stop::Refused(Refusal::new(Code::Run(code), message))
}
