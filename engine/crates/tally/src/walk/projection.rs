//! The written projection `bal(x, d) from t`, a walk of its own from day t without its holder's dependents, and the `min` of it read as one range minimum.
//! Spec: L §5.14.1 (The projection is written: its start at t, or at a decision's point on t;
//! the movements dated t through d counted as known on t; those that depend on the holder left
//! out and listed; `with uncertain`; `HOLE-PRERUN` before S; `RUN-FUTURE`; `RUN-DOMAIN` for a
//! day before t), E §4.3 (steps 1–3; nesting is finite; the projection's own walk is the plain
//! twin of fence F5; the common case offered to F5 first, `lows.rs`; the `min` of `bal(x, d)
//! from t` over the days a .. b a range minimum: its shape recognised here, `lowest_shape`,
//! and read by `range_minimum` with the fence on or off, from F5's tree or, F5's plain twin,
//! one walk read day by day; the aggregate `min`, its terms read one by one, eval/aggregate.rs,
//! only on a `partial` line or in a traced read), §3.2 (the projection in a cell's context),
//! L §5.11 (`min`), §8.2 (`left out:`).
//! Twin: engine/tools/refcore/mreads.py (`proj_bal`, `proj_regs`, `proj_uncertain`,
//! `proj_clauses`), walk.py (`proj_walk`) and, for the recognised `min`, evaluate.py
//! (`e_agg`).
//! Invariants: a projection's walk starts each place as the walk it continues stood at its
//! point and books nothing of a move that depends on its holder; there is one walk per (walk
//! continued, point, cut, moves left out), walked only as far as a read needs; a decision
//! reads no projection from a day after its own; a read F5 answers is answered as this walk
//! would, and every read F5 leaves is read here unchanged; a recognised `min` gives the
//! aggregate's value, hole or refusal, and one `range_minimum` leaves is the aggregate's, read
//! from the same range's values.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use super::agenda::left_out_lines;
use super::fork::Held;
use super::lows::{Base, Lows};
use super::plan::invariant;
use super::point::At;
use super::reads::Sees;
use super::state::{Projected, St, Walk};
use crate::base::{Code, Load, Refusal, Run};
use crate::eval::{Ctx, Engine, Env, hole_of, min_max, reads_name};
use crate::hole::Hole;
use crate::model::{Reality, Value};
use crate::num::Num;
use crate::syntax::{Agg, AggFn, Bal, Domain, Expr, Lit, Stop};
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

    /// Whether a `min` of [`lowest_shape`]'s shape is read as one range minimum: always, with
    /// the fence on or off, as `low` was (F5's tree under `Lows::Tree`, its plain twin's one
    /// walk read day by day under `Lows::Walk`); never on a `partial` line or in a traced
    /// read, which read each day's projection as the aggregate's terms do (L §5.18, E §10).
    /// E §4.3.
    pub(crate) fn reads_lowest(&self, ctx: Ctx) -> bool {
        !ctx.partial && !self.tracing()
    }

    /// A `min` of [`lowest_shape`]'s shape, its term `read`, read as one range minimum over
    /// its range's values `days`, read once as its binding reads them: its balances
    /// ([`Engine::lowest_balances`]) folded as the aggregate folds its terms, a hole where any
    /// is one. None where the aggregate reads it term by term from the same values: a range
    /// of no day or of other values, or what `lowest_balances` leaves. L §5.11, E §4.3.
    ///
    /// # Errors
    ///
    /// The refusals of [`Engine::lowest_balances`] and of the fold (L §5.11's `min`).
    pub(crate) fn range_minimum(
        &mut self,
        m: &Reality,
        read: &Bal,
        days: &[(Value, Option<Hole>)],
        (env, ctx): (&Env, Ctx),
    ) -> Result<Option<Value>, Stop> {
        let Some(ends) = first_and_last(days) else {
            return Ok(None);
        };
        let Some(balances) = self.lowest_balances(m, read, ends, (env, ctx))? else {
            return Ok(None);
        };
        let holes: Vec<Option<&Hole>> = balances
            .iter()
            .filter_map(Value::as_hole)
            .map(Some)
            .collect();
        if !holes.is_empty() {
            return hole_of(&holes).map(Some);
        }
        min_max(m, balances, crate::eval::End::Low).map(Some)
    }

    /// The end-of-day balances of a `min` of [`lowest_shape`]'s shape, its term `read`, on its
    /// days `first` through `last`, its place and start read once: in E §4.3's common case
    /// one balance, the least, from fence F5's tree; else each day's, from one walk continued
    /// from day t, walked day by day. None where the aggregate reads it day by day as
    /// written, so that every hole and refusal its days meet is the aggregate's own: a day
    /// outside the run, a place or a start that is a hole or a window, a start before the run
    /// or after the first day. L §5.14.1, E §4.3.
    ///
    /// # Errors
    ///
    /// The refusals the aggregate's first term meets: its place's, its unit's (`LOAD-UNIT`),
    /// its start's, `RUN-FUTURE` for a decision's projection from a later day; the refusals of
    /// the walk.
    fn lowest_balances(
        &mut self,
        m: &Reality,
        read: &Bal,
        (first, last): (Day, Day),
        (env, ctx): (&Env, Ctx),
    ) -> Result<Option<Vec<Value>>, Stop> {
        let Some(from) = &read.proj.from else {
            return Ok(None);
        };
        if first < m.start || last > m.end {
            return Ok(None);
        }
        let Ok(place) = self.place_arg(m, &read.place, env, ctx)? else {
            return Ok(None);
        };
        let unit = own_unit(m, &place)?;
        let start = self.ev(m, from, env, ctx)?;
        if let Value::Hole(_) = start {
            return Ok(None);
        }
        let Ok(start) = crate::eval::one_day(m, &start, "the day of a projection")? else {
            return Ok(None);
        };
        if first < start || start < m.start {
            return Ok(None);
        }
        let uncertain = read.proj.uncertain || ctx.unc;
        let window = (start, (first, last));
        if let Some(v) = self.common_case(m, (&place, &unit), window, (uncertain, ctx))? {
            return Ok(Some(vec![v]));
        }
        let left_out = self.holder_left_out(m)?;
        let id = self.proj_walk(m, ctx, (start, last), left_out)?;
        let balances = self.days_read(
            m,
            id,
            (&place, &unit),
            (start, (first, last)),
            (uncertain, ctx),
        );
        self.end_projection(id, balances.as_ref().err())?;
        balances.map(Some)
    }

    /// [`Engine::lowest_balances`]' read of its walk `id` from day `start`: each day's end
    /// balance from `first` through `last`, the walk walked through that day first, with that
    /// day's `left out:` lines, its uncertain movements counted under `with uncertain`, as the
    /// `min`'s terms read them one by one (refcore's `proj_bal`). L §5.14.1, E §4.3.
    fn days_read(
        &mut self,
        m: &Reality,
        id: usize,
        (place, unit): (&str, &str),
        (start, (first, last)): (Day, (Day, Day)),
        (uncertain, ctx): (bool, Ctx),
    ) -> Result<Vec<Value>, Stop> {
        if let Some(at) = ctx.at {
            self.note_read(at.walk, place, Sees::AtPoint)?;
        }
        // a day's balance reads the walk through that day (L §8, a scan; tally 2.1.7 R1, #127):
        // a walk that fails on a day refuses the `min` there, after the days before it are read
        let mut out = Vec::new();
        let mut day = Some(first);
        while let Some(d) = day.filter(|d| *d <= last) {
            self.walk_projection(m, id, d)?;
            self.proj_regs(id, start, d)?;
            let v = self.read_state(m, id, place, unit, At::end(d))?;
            out.push(if uncertain && !matches!(v, Value::Hole(_)) {
                self.proj_uncertain(m, id, (place, unit), (d, &v), ctx)?
            } else {
                v
            });
            day = d.next();
        }
        Ok(out)
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
    /// walk's cut (refcore's `proj_walk`, its start). An earlier t before the start of the
    /// projection's walk the decision runs in is a day that walk takes from the walk it
    /// continues, so the projection starts there, up the chain (E §4.3 step 1: "a read in it
    /// of days before its start … takes those days from that walk"). E §4.3 step 1,
    /// L §5.14.1.
    fn proj_base(&mut self, m: &Reality, ctx: Ctx, t: Day) -> Result<Base, Stop> {
        let Some(at) = ctx.at else {
            return Ok((ctx.cut, self.main_walk(m, ctx.cut)?, At::start(t)));
        };
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
        if t == day {
            return Ok((self.walk(at.walk)?.cut, at.walk, at.point));
        }
        let mut id = at.walk;
        for wid in self.chain(at.walk)? {
            id = wid;
            if self.walk(wid)?.start <= t {
                break;
            }
        }
        Ok((self.walk(id)?.cut, id, At::start(t)))
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
            Some(id) => self.outgrown(id),
            None => Held::New,
        };
        let id = match found {
            Held::Kept(id) => id,
            Held::New => self.new_proj_walk(m, key, t, self.planned_through(m, through))?,
            Held::Continue(kept) => {
                let id = self.new_proj_walk(m, key, t, self.planned_through(m, through))?;
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
        let lines = left_out_lines(&p.counted, (t, (d, d)))?;
        for line in lines {
            self.note("left out", line, String::new());
        }
        Ok(())
    }
}

/// A range's ends and step as written, `a .. b step s` (L §2.3 `bind`).
pub(crate) type RangeText = (Expr, Expr, Option<Expr>);

/// The range and the term of a `min` E §4.3 reads as one range minimum, `min(bal(x, d) from
/// t [with uncertain] for d in a .. b step 1 day)`: one binding, a range of days stepped by
/// one day with no `where`, `on` or `over`; its term the projection's balance of the bound
/// day at its end, whose place and start don't read the bound day; no `ties by` or `random
/// at`. None for any other `min`, which is the aggregate, term by term (L §5.11). L §5.14.1,
/// E §4.3.
pub(crate) fn lowest_shape(agg: &Agg) -> Option<(&RangeText, &Bal)> {
    let (AggFn::Min, [bind], Some(Expr::Bal(read)), None) =
        (agg.func, agg.binds.as_slice(), &agg.term, &agg.ties)
    else {
        return None;
    };
    let (Domain::Range(range), None, None, None) =
        (&bind.domain, &bind.on, &bind.over, &bind.filter)
    else {
        return None;
    };
    let ([Expr::Name(day)], Some(from), None) = (read.args.as_slice(), &read.proj.from, &read.of)
    else {
        return None;
    };
    let clauses = !read.start && !read.modelled && read.proj.random.is_none();
    let free = !reads_name(&read.place, &bind.var) && !reads_name(from, &bind.var);
    let shape = *day == bind.var && clauses && free && one_day_step(range.2.as_ref());
    shape.then_some((&**range, &**read))
}

/// Whether a range's step is written `1 day`. L §2.3 `bind`, §5.6.
fn one_day_step(step: Option<&Expr>) -> bool {
    let Some(Expr::Lit(Lit::Num(n))) = step else {
        return false;
    };
    let one = |q: &Num| q.compare(&Num::int(1)) == Ok(Ordering::Equal);
    let day = n.unit.as_ref().is_some_and(
        |u| matches!(u.terms.as_slice(), [term] if term.base == "day" && one(&term.power)),
    );
    day && one(&n.value)
}

/// The first and last of a range's days, where every value is a day with no hole of its own;
/// none for a range of no day or of other values. L §5.6.
fn first_and_last(values: &[(Value, Option<Hole>)]) -> Option<(Day, Day)> {
    let mut days = values.iter().map(|value| match value {
        (Value::Day(d), None) => Some(*d),
        _ => None,
    });
    let first = days.next()??;
    let mut last = first;
    for d in days {
        last = d?;
    }
    Some((first, last))
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
