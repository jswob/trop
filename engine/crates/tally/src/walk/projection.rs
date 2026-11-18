//! The written projection `bal(x, d) from t`, a walk of its own from day t without its holder's dependents.
//! Spec: L §5.14.1 (The projection is written: its start at t, or at a decision's point on t;
//! the movements dated t through d counted as known on t; those that depend on the holder left
//! out and listed; `with uncertain`; `HOLE-PRERUN` before S; `RUN-FUTURE`; `RUN-DOMAIN` for a
//! day before t), E §4.3 (steps 1–3; nesting is finite; the projection's own walk is the plain
//! twin of fence F5; the common case offered to F5 first, `lows.rs`), §3.2 (the projection in
//! a cell's context), L §8.2 (`left out:`).
//! Twin: engine/tools/refcore/mreads.py (`proj_bal`, `proj_regs`, `proj_uncertain`,
//! `proj_clauses`) and walk.py (`proj_walk`).
//! Invariants: a projection's walk starts each place as the walk it continues stood at its
//! point and books nothing of a move that depends on its holder; there is one walk per (walk
//! continued, point, cut, moves left out), walked only as far as a read needs; a decision
//! reads no projection from a day after its own; a read F5 answers is answered as this walk
//! would, and every read F5 leaves is read here unchanged.

use std::collections::{BTreeMap, BTreeSet};

use super::agenda::left_out_lines;
use super::fork::Held;
use super::lows::{Base, Lows};
use super::plan::invariant;
use super::point::At;
use super::reads::Sees;
use super::state::{Projected, St, Walk};
use crate::base::{Code, Load, Refusal, Run};
use crate::eval::{Ctx, Engine, Env};
use crate::hole::Hole;
use crate::model::{Reality, Value};
use crate::syntax::{Bal, Stop};
use crate::time::{Day, Period};

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
        let start = match crate::eval::one_day(m, &start, "the day of a projection")? {
            Ok(start) => start,
            Err(h) => return Ok(Value::Hole(h)),
        };
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
            return Ok(Value::Hole(Hole::before_run(place)));
        }
        let uncertain = read.proj.uncertain || ctx.unc;
        // a `partial` line reads the projection's own walk, whose legs give its linear form
        if !ctx.partial
            && let Some(v) =
                self.common_case(m, (place, unit), (start, (day, day)), (uncertain, ctx))?
        {
            return Ok(v);
        }
        let left_out = self.holder_left_out(m)?;
        let id = self.proj_walk(m, ctx, (start, day), left_out)?;
        let value = self.projected_bal(m, id, (place, unit), (start, day), (read, ctx));
        self.end_projection(id, value.as_ref().err())?;
        value
    }

    /// [`Engine::projection`]'s read of its walk `id` from day `start`: x at the end of `day`,
    /// the moves it leaves out listed, its uncertain movements counted under `with uncertain`
    /// (refcore's `proj_bal`). L §5.14.1, E §4.3.
    fn projected_bal(
        &mut self,
        m: &Reality,
        id: usize,
        (place, unit): (&str, &str),
        (start, day): (Day, Day),
        (read, ctx): (&Bal, Ctx),
    ) -> Result<Value, Stop> {
        if let Some(at) = ctx.at {
            self.note_read(at.walk, place, Sees::AtPoint)?;
        }
        self.walk_projection(m, id, day)?;
        let mut value = self.read_state(m, id, place, unit, At::end(day))?;
        // a projection's balance is a sum as the plain one is (L §5.18; tally 2.1.8 R15)
        if ctx.partial
            && let Value::Hole(h) = value
        {
            value = self.partial_bal(m, id, (place, unit), At::end(day), h)?;
        }
        self.proj_regs(id, start, day)?;
        // the read's own clause, else the ask line's, which no decision's context carries; a
        // `partial` line's hole keeps the uncertain amounts in its known part (L §5.18, R15)
        if (read.proj.uncertain || ctx.unc) && (ctx.partial || !matches!(value, Value::Hole(_))) {
            return self.proj_uncertain(m, id, (place, unit), (day, &value), ctx);
        }
        Ok(value)
    }

    /// The plain twin's walk of a projection from day `t`, as the item holding it now leaves
    /// moves out, walked through day `through`: what `explain` of a projection reads, never
    /// F5's tree (refcore's `_explain_projection`, its `proj_walk` and `run_to`). E §10,
    /// E §4.3, L §5.14.1.
    ///
    /// # Errors
    ///
    /// `RUN-FUTURE` for a decision's projection from a later day; the refusals of its walk.
    pub(crate) fn projection_walk(
        &mut self,
        m: &Reality,
        ctx: Ctx,
        t: Day,
        through: Day,
    ) -> Result<usize, Stop> {
        let left_out = self.holder_left_out(m)?;
        let id = self.proj_walk(m, ctx, (t, through), left_out)?;
        self.walk_projection(m, id, through)?;
        Ok(id)
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

    /// E §4.3's common case read from fence F5's tree (`lows.rs`), where the engine reads it
    /// so and the read takes no `with uncertain`; none for the projection's own walk to read
    /// (P §2.3). E §4.3, L §5.14.1.
    fn common_case(
        &mut self,
        m: &Reality,
        (place, unit): (&str, &str),
        window: (Day, (Day, Day)),
        (uncertain, ctx): (bool, Ctx),
    ) -> Result<Option<Value>, Stop> {
        if uncertain || self.lows == Lows::Walk {
            return Ok(None);
        }
        let left_out = self.holder_left_out(m)?;
        let base = self.proj_base(m, ctx, window.0)?;
        if let Some(at) = ctx.at {
            self.note_read(at.walk, place, Sees::AtPoint)?;
        }
        self.common_low(m, (place, unit), window, base, &left_out)
    }

    /// Where the projection from day `t` starts: the main walk at the start of t, or the
    /// reading decision's walk, at its point on t or at the start of an earlier t, with that
    /// walk's cut (refcore's `proj_walk`, its start). E §4.3 step 1, L §5.14.1.
    fn proj_base(&mut self, m: &Reality, ctx: Ctx, t: Day) -> Result<Base, Stop> {
        match ctx.at {
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
                Ok((self.walk(at.walk)?.cut, at.walk, point))
            }
            None => Ok((ctx.cut, self.main_walk(m, ctx.cut)?, At::start(t))),
        }
    }

    /// The walk of a projection from day `t` through day `through`, from where
    /// [`Engine::proj_base`] starts it; its cut min(t, K): one a read holds now that plans
    /// that far, or with forks on one kept past its reads (`fork.rs`), else a new one; taken
    /// by the read until [`Engine::end_projection`] (refcore's `proj_walk`). E §4.3 step 1,
    /// L §5.14.1; P §2.3, §2.5, §3.5.
    fn proj_walk(
        &mut self,
        m: &Reality,
        ctx: Ctx,
        (t, through): (Day, Day),
        left_out: BTreeSet<String>,
    ) -> Result<usize, Stop> {
        let (cut, base, point) = self.proj_base(m, ctx, t)?;
        let c = t.min(cut);
        let key: ProjKey = (base, point, c, left_out);
        let held = self.proj_walks.get(&key).copied();
        let found = match held {
            Some(id) if self.is_parked(id) => self.meet_parked(id, through)?,
            Some(id) if self.walk(id)?.plans_through() >= through => Held::Kept(id),
            Some(_) | None => Held::New,
        };
        let id = match found {
            Held::Kept(id) => id,
            Held::New => self.new_proj_walk(m, key, t, horizon(m, through))?,
            Held::Continue(kept) => {
                let id = self.new_proj_walk(m, key, t, horizon(m, through))?;
                self.wait_to_continue(id, *kept);
                id
            }
        };
        let p = self
            .walk_mut(id)?
            .proj
            .as_mut()
            .ok_or_else(|| invariant("a projection's walk with no projection"))?;
        p.readers = p
            .readers
            .checked_add(1)
            .ok_or_else(|| invariant("a projection's readers past 32 bits"))?;
        Ok(id)
    }

    /// A new walk of projection `key` from day `t`, planned through day `through`, put under
    /// its key (refcore's `proj_walk`, its new `Walk`). E §4.3 step 1.
    ///
    /// # Errors
    ///
    /// The refusals the walk's first readings meet; `ENGINE-INVARIANT` for a walk the engine
    /// lacks.
    fn new_proj_walk(
        &mut self,
        m: &Reality,
        key: ProjKey,
        t: Day,
        through: Day,
    ) -> Result<usize, Stop> {
        let (base, point, c) = (key.0, key.1, key.2);
        let walk = self.continued(m, (base, point), (c, t, through), &key.3)?;
        let id = self.walks.len();
        self.walks.push(walk);
        #[cfg(feature = "counters")]
        self.counts.add(crate::Unit::Walk);
        self.proj_walks.insert(key, id);
        Ok(id)
    }

    /// A projection's read of walk `id` ended, with `stop` if it stopped: when no other read
    /// holds the walk it is freed, and its key with it, its memory released as its scope ends,
    /// or with forks on kept for the projection's next read (`fork.rs`, P §2.5); a read cut
    /// short past the counted depth or given up for a guarded cell keeps it, to walk on when
    /// read again (P §3.5, §1 item 9, D1-6; E §4.3; R-16).
    fn end_projection(&mut self, id: usize, stop: Option<&Stop>) -> Result<(), Stop> {
        let resumes = match stop {
            Some(Stop::Deep) => true,
            Some(Stop::NotBuilt(why)) => *why == crate::eval::GUARDED,
            Some(Stop::Refused(_)) | None => false,
        };
        let walk = self.walk_mut(id)?;
        let p = walk
            .proj
            .as_mut()
            .ok_or_else(|| invariant("a projection's walk with no projection"))?;
        p.readers = p
            .readers
            .checked_sub(1)
            .ok_or_else(|| invariant("a projection's read ended twice"))?;
        // a walk being walked has a reader still: every walk of a projection's walk is a
        // read's (`projected_bal`, explain's `projection_walk`), which holds it
        if resumes || p.readers > 0 {
            return Ok(());
        }
        self.park_projection(id)
    }

    /// A new projection's walk of cut `c` from day `t`, continuing walk `base` after `point`:
    /// each place starts as that walk stood there, and a place's first reading is the one its
    /// cut sees, else the walk's it continues (refcore's `Walk(base=…, point=…)` and
    /// `anchor_question`). E §4.3 step 1, L §8.1.
    fn continued(
        &mut self,
        m: &Reality,
        (base, point): (usize, At),
        (c, t, through): (Day, Day, Day),
        left_out: &BTreeSet<String>,
    ) -> Result<Walk, Stop> {
        let world = self.walk(base)?.world;
        let mut first_reading = crate::book::first_readings(m, (c, world))?;
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
        for (place, day) in &from.first_reading {
            first_reading.entry(place.clone()).or_insert(*day);
        }
        let projected = Projected {
            base,
            point,
            left_out: left_out.clone(),
            counted: BTreeMap::new(),
            levels,
            root,
            world: from.world,
            through,
            readers: 0,
        };
        // the window tests the walk it continues met still leave their holes open here
        let open_tests = from.open_tests.clone();
        let mut walk = Walk::projection(c, (t, m.end), projected, first_reading);
        walk.open_tests = open_tests;
        Ok(walk)
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
        let lines = left_out_lines(&p.counted, (t, d))?;
        for line in lines {
            self.note("left out", line, String::new());
        }
        Ok(())
    }
}

/// The day a projection's walk read through `through` is planned through: on to the last day
/// of that day's month (P §2.4's interval), within the run, so that a projection read day by
/// day plans once a month, not once a read, while its walk stays within a month of its window
/// (P §3.5, pick P54). Planned further, a walk walks no further than a read asks and books
/// the same through it: past the window only instances the walk never reaches change (E §4.1,
/// §4.3; pick P73).
fn horizon(m: &Reality, through: Day) -> Day {
    match Period::month_of(through).and_then(|month| month.end().previous()) {
        Some(last) => last.min(m.end).max(through),
        None => through,
    }
}

/// A place's own unit, for a read that names none: `LOAD-UNIT` for a place of unit `any`
/// (refcore's `m_bal_unit`, with no unit written). L §5.10.
pub(crate) fn own_unit(m: &Reality, place: &str) -> Result<String, Stop> {
    m.things
        .get(place)
        .and_then(|t| m.place_unit(t))
        .filter(|u| u != "any")
        .ok_or_else(|| {
            Stop::Refused(Refusal::new(
                Load::Unit.at_run(),
                format!("bal({place}, d) of a place of unit any states its unit: bal(x, u, d)"),
            ))
        })
}

/// A run-time refusal (L §8.4).
fn refuse(code: Run, message: String) -> Stop {
    Stop::Refused(Refusal::new(Code::Run(code), message))
}
