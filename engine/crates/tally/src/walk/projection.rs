//! The written projection `bal(x, d) from t`, a walk of its own from day t without its holder's dependents, and `low`, its least day.
//! Spec: L §5.14.1 (The projection is written: its start at t, or at a decision's point on t;
//! the movements dated t through d counted as known on t; those that depend on the holder left
//! out and listed; `with uncertain`; `HOLE-PRERUN` before S; `RUN-FUTURE`; `low` the minimum
//! of the end-of-day balances, `RUN-DOMAIN` for a day before t), E §4.3 (steps 1–3; nesting is
//! finite; `low` read day by day here, the plain twin of fence F5; the common case offered to
//! F5 first, `lows.rs`), §3.2 (the projection in a cell's context), L §8.2 (`left out:`).
//! Twin: engine/tools/refcore/mreads.py (`proj_bal`, `proj_regs`, `proj_uncertain`,
//! `proj_clauses`, `m_e_low`) and walk.py (`proj_walk`).
//! Invariants: a projection's walk starts each place as the walk it continues stood at its
//! point and books nothing of a move that depends on its holder; there is one walk per (walk
//! continued, point, cut, moves left out), walked only as far as a read needs; a decision
//! reads no projection from a day after its own; `low` is the first of equal lows, a hole
//! where any day's balance is one; a read F5 answers is answered as this walk would, and every
//! read F5 leaves is read here unchanged.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use super::agenda::left_out_lines;
use super::lows::{Base, Lows};
use super::plan::invariant;
use super::point::At;
use super::state::{Projected, St, Walk};
use crate::base::{Code, Load, Refusal, Run};
use crate::eval::{Ctx, Engine, Env};
use crate::hole::{Hole, spread};
use crate::model::{Reality, Value};
use crate::num::Num;
use crate::syntax::{Bal, Low, Stop};
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
        let uncertain = read.proj.uncertain || ctx.unc;
        if let Some(v) =
            self.common_case(m, (place, unit), (start, (day, day)), (uncertain, ctx))?
        {
            return Ok(v);
        }
        let left_out = self.holder_left_out(m)?;
        let id = self.proj_walk(m, ctx, start, left_out)?;
        if let Some(at) = ctx.at {
            self.note_live(at.walk, place)?;
        }
        self.walk_projection(m, id, day)?;
        let value = self.read_state(m, id, place, unit, At::end(day))?;
        self.proj_regs(id, start, day)?;
        // the read's own clause, else the ask line's, which no decision's context carries
        if (read.proj.uncertain || ctx.unc) && !matches!(value, Value::Hole(_)) {
            return self.proj_uncertain(m, id, (place, unit), day, &value);
        }
        Ok(value)
    }

    /// `low(x, a .. b) [from t] [clauses]`: the lowest end-of-day balance of x on the days a
    /// through b, the first of equal lows; with `from t` each day's balance in the walk
    /// continued from day t, else the plain balance, with the read's clauses either way; a
    /// hole where any day's balance is one, with every day's questions (refcore's `m_e_low`).
    /// L §5.14.1 (the built-ins' table), E §4.3.
    ///
    /// # Errors
    ///
    /// `RUN-EMPTY` for a window with no day (b before a: L §5.11, `min` over no keys);
    /// `RUN-DOMAIN` for a first day before the projection's start, a start that isn't a day;
    /// `LOAD-UNIT` for a place of unit `any`; `RUN-FUTURE` for a decision's projection from a
    /// later day; the refusals of the walk; `NotBuilt` for `windows each`.
    pub(crate) fn low(
        &mut self,
        m: &Reality,
        read: &Low,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let place = match self.place_arg(m, &read.place, env, ctx)? {
            Err(h) => return Ok(Value::Hole(h)),
            Ok(p) => p,
        };
        let unit = own_unit(m, &place)?;
        let span = match self.span_value(m, &read.span, env, ctx)? {
            Err(h) => return Ok(Value::Hole(h)),
            Ok(sp) => sp,
        };
        let first = span
            .start()
            .ok_or_else(|| invariant("a span with no start"))?;
        let last = super::flows::span_last(m, &span)?;
        if last < first {
            // `min(bal(x, d) … for d in a .. b step 1 day)` over no day: its terms are never read
            return Err(refuse(Run::Empty, "low over no keys".to_owned()));
        }
        if read.proj.windows_each {
            return Err(Stop::NotBuilt("`windows each` on a balance read"));
        }
        let days = Days { first, last };
        let balances = match &read.proj.from {
            Some(from) => {
                let start = self.ev(m, from, env, ctx)?;
                if let Value::Hole(_) = start {
                    return Ok(start);
                }
                let start = crate::ask::date_of(&start, "the day of a projection")?;
                if first < start {
                    return Err(refuse(
                        Run::Domain,
                        format!(
                            "low({place}, {} .. {}) from {}: a day before the projection's start",
                            first.text(),
                            last.text(),
                            start.text()
                        ),
                    ));
                }
                if start < m.start {
                    return Ok(Value::Hole(Hole::asked("before the run's start", &place)));
                }
                let uncertain = read.proj.uncertain || ctx.unc;
                let window = (start, (days.first, days.last));
                if let Some(v) = self.common_case(m, (&place, &unit), window, (uncertain, ctx))? {
                    return Ok(v);
                }
                self.projected_days(m, (&place, &unit), (start, days), (read, ctx))?
            }
            None => self.plain_days(m, (&place, &unit), days, (read, ctx))?,
        };
        lowest(balances)
    }

    /// The end-of-day balances of a place on the days of a `low`, in the walk continued from
    /// day `start`, its uncertain movements counted under `with uncertain` (refcore's
    /// `m_e_low`, its `proj` form). L §5.14.1, E §4.3.
    fn projected_days(
        &mut self,
        m: &Reality,
        (place, unit): (&str, &str),
        (start, days): (Day, Days),
        (read, ctx): (&Low, Ctx),
    ) -> Result<Vec<Value>, Stop> {
        // the read's own clause, else the ask line's, which no decision's context carries
        let uncertain = read.proj.uncertain || ctx.unc;
        let left_out = self.holder_left_out(m)?;
        let id = self.proj_walk(m, ctx, start, left_out)?;
        if let Some(at) = ctx.at {
            self.note_live(at.walk, place)?;
        }
        self.walk_projection(m, id, days.last)?;
        self.proj_regs(id, start, days.last)?;
        let mut out = Vec::new();
        let mut day = Some(days.first);
        while let Some(d) = day.filter(|d| *d <= days.last) {
            let v = self.read_state(m, id, place, unit, At::end(d))?;
            out.push(if uncertain && !matches!(v, Value::Hole(_)) {
                self.proj_uncertain(m, id, (place, unit), d, &v)?
            } else {
                v
            });
            day = d.next();
        }
        Ok(out)
    }

    /// The plain end-of-day balances of a place on the days of a `low` with no `from`, each
    /// with the clauses a plain `bal` takes: `with uncertain` (refcore's `m_e_low`, its plain
    /// form; pick P51, #125). L §5.14.1 (`low` "with the same clauses"), §5.14.
    fn plain_days(
        &mut self,
        m: &Reality,
        (place, unit): (&str, &str),
        days: Days,
        (read, ctx): (&Low, Ctx),
    ) -> Result<Vec<Value>, Stop> {
        let mut out = Vec::new();
        let mut day = Some(days.first);
        while let Some(d) = day.filter(|d| *d <= days.last) {
            let v = self.bal_day(m, (place, unit), d, At::end(d), ctx)?;
            out.push(if read.proj.uncertain {
                self.with_uncertain(m, (place, unit), d, &v, ctx)?
            } else {
                v
            });
            day = d.next();
        }
        Ok(out)
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
            self.note_live(at.walk, place)?;
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

    /// The walk of a projection from day `t`, from where [`Engine::proj_base`] starts it; its
    /// cut min(t, K) (refcore's `proj_walk`). E §4.3 step 1, L §5.14.1.
    fn proj_walk(
        &mut self,
        m: &Reality,
        ctx: Ctx,
        t: Day,
        left_out: BTreeSet<String>,
    ) -> Result<usize, Stop> {
        let (cut, base, point) = self.proj_base(m, ctx, t)?;
        let c = t.min(cut);
        let key: ProjKey = (base, point, c, left_out);
        if let Some(id) = self.proj_walks.get(&key) {
            return Ok(*id);
        }
        let walk = self.continued(m, (base, point), (c, t), key.3.clone())?;
        let id = self.walks.len();
        self.walks.push(walk);
        #[cfg(feature = "counters")]
        self.counts.add(crate::Unit::Walk);
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
        let mut first_reading = crate::book::first_readings(m, (c, from.world))?;
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
            world: from.world,
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
        let lines = left_out_lines(&p.counted, (t, d))?;
        for line in lines {
            self.note("left out", line, String::new());
        }
        Ok(())
    }
}

/// The days of a `low`, first through last. L §5.14.1.
#[derive(Debug, Clone, Copy)]
struct Days {
    /// The first day, a.
    first: Day,
    /// The last day, b.
    last: Day,
}

/// The lowest of a `low`'s daily balances, the first of equal ones; the holes of every day
/// where any is one (refcore's `m_e_low`, its end). L §5.14.1, §8.1.
fn lowest(balances: Vec<Value>) -> Result<Value, Stop> {
    let holes: Vec<Option<&Hole>> = balances.iter().map(Value::as_hole).collect();
    if let Some(h) = spread(&holes) {
        return Ok(Value::Hole(h));
    }
    let mut best: Option<(Value, Num)> = None;
    for v in balances {
        // one place in one unit: money by its minor units, a number by its value
        let q = if let Value::Money(x) = &v {
            x.minor_units()
        } else if let Value::Number(n) = &v {
            n.q.clone()
        } else {
            return Err(invariant("a day's balance that is no quantity"));
        };
        let lower = match &best {
            None => true,
            Some((_, b)) => q.compare(b).map_err(crate::eval::num_stop)? == Ordering::Less,
        };
        if lower {
            best = Some((v, q));
        }
    }
    best.map(|(v, _)| v)
        .ok_or_else(|| invariant("a low over no day"))
}

/// A place's own unit, for a read that names none: `LOAD-UNIT` for a place of unit `any`
/// (refcore's `m_bal_unit`, with no unit written). L §5.10.
pub(super) fn own_unit(m: &Reality, place: &str) -> Result<String, Stop> {
    m.things
        .get(place)
        .and_then(|t| m.place_unit(t))
        .filter(|u| u != "any")
        .ok_or_else(|| {
            Stop::Refused(Refusal::new(
                Code::Load(Load::Unit),
                format!("bal({place}, d) of a place of unit any states its unit: bal(x, u, d)"),
            ))
        })
}

/// A run-time refusal (L §8.4).
fn refuse(code: Run, message: String) -> Stop {
    Stop::Refused(Refusal::new(Code::Run(code), message))
}
