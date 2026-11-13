//! The walk's balance reads: a place's balance at an instant, plain, running or with uncertain, with what it rests on in the register.
//! Spec: L §5.10 (`bal(x)`, `bal(x, d)`, `bal(x, d start)`, `bal(x, u, d)`; `HOLE-ANCHOR`),
//! §5.14 (`with uncertain`), §5.14.1 (running reads: a decision reads the walk at its point;
//! `RUN-FUTURE`, a plain read of its own day `RUN-CYCLE`), §5.19 (a modelled decision reads
//! the balances less its own observations booked before its day); §8.2 (the register:
//! `unfulfilled:`, `left out:`, `adjustment:`, `uncertain:`, `open:`); E §4.2 (a refused leg
//! refuses every read of it; holes in the walk), E §4.3 (a read of a projection's walk rests on the walks it continues), E §3.3 (the
//! main walk of a timeline); §5.19 (`gap(x, d)`, `HOLE-NOREADING`). The flows reads
//! (L §5.14.3) are `flows.rs`'s, `value` (L §5.10) is `value.rs`'s.
//! Twin: engine/tools/refcore/mreads.py (`place_arg`, `bal_unit`, `read_state`,
//! `merge_cum`, `e_bal`, `note_read`, `bal_day`, `with_uncertain`, `proj_uncertain`,
//! `less_own_obs`, `m_c_gap`) and walk.py (`m_main_walk`).
//! Invariants: a read inside a decision sees the walk only up to its point; a read at the cut
//! sees the whole main walk; a hole a read meets is printed with every question it rests on;
//! a refused `with uncertain` keeps the register it met before the refusal, as refcore
//! registers while it counts.

use super::plan::{Amount, Dated, Inst, invariant, qty_value};
use super::point::At;
use super::state::{ALL, GapAt, St, Walk};
use crate::base::{Code, Load, Refusal, Run};
use crate::eval::{Collected, Ctx, Engine, Entry, Env};
use crate::hole::{Hole, spread};
use crate::model::{KeyVal, Reality, Value};
use crate::num::Num;
use crate::syntax::{Bal, Expr, Stop};
use crate::time::Day;

/// How a running read sees its place: as it stands at the decision's point, or only before
/// it (refcore's `note_read`, its `live`). L §5.14.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Sees {
    /// At the decision's point: it orders the day's decisions.
    AtPoint,
    /// Only before its point.
    Before,
}

impl Engine {
    /// A balance read (refcore's `e_bal`). L §5.10, §5.14.1.
    ///
    /// # Errors
    ///
    /// `LOAD-READS` for `bal(x)` outside a decision; `LOAD-UNIT` for a unit a place doesn't
    /// hold; the refusals of the walk; `NotBuilt` for what S doesn't build.
    pub(crate) fn bal(&mut self, m: &Reality, b: &Bal, env: &Env, ctx: Ctx) -> Result<Value, Stop> {
        let place = match self.place_arg(m, &b.place, env, ctx)? {
            Err(h) => return Ok(Value::Hole(h)),
            Ok(p) => p,
        };
        let unit = match self.bal_unit(m, b, &place, env, ctx)? {
            Err(h) => return Ok(Value::Hole(h)),
            Ok(u) => u,
        };
        if b.of.is_some() {
            return Err(Stop::NotBuilt("bal of a party or a batch (M5a)"));
        }
        if b.modelled {
            return Err(Stop::NotBuilt("a modelled balance (the ledger)"));
        }
        if b.proj.windows_each {
            return Err(Stop::NotBuilt("`windows each` on a balance read"));
        }
        let Some(day_arg) = b.args.last() else {
            let Some(at) = ctx.at else {
                return Err(Stop::Refused(Refusal::new(
                    Code::Load(Load::Reads),
                    format!("bal({place}) with no day outside a decision"),
                )));
            };
            self.note_read(at.walk, &place, Sees::AtPoint)?;
            let v = self.read_state(m, at.walk, &place, &unit, at.point)?;
            return self.less_own_obs(m, at.walk, (&place, &unit), at.point, v);
        };
        let d = self.ev(m, day_arg, env, ctx)?;
        if let Value::Hole(_) = d {
            return Ok(d);
        }
        // a window left a window (no `windows each`) is the hole of its imprecision for this
        // read, named with its place (refcore's `m_e_bal`); else the one-day reader's text
        let named;
        let what = if let Value::Window(_) = d {
            named = format!("the day of bal({place}, d)");
            named.as_str()
        } else {
            "the day of bal"
        };
        let d = match crate::eval::one_day(m, &d, what)? {
            Ok(d) => d,
            Err(imprecise) => return Ok(Value::Hole(imprecise)),
        };
        let outside = outside_run(m, d, &place).map(Value::Hole);
        if b.proj.from.is_some() {
            return match outside {
                Some(hole) => Ok(hole),
                None => self.projection(m, b, (&place, &unit), d, (env, ctx)),
            };
        }
        let key = if b.start { At::start(d) } else { At::end(d) };
        let walked = self.trace_walked();
        let got = match outside {
            Some(hole) => Ok(hole),
            None => self.bal_day(m, (&place, &unit), d, key, ctx).and_then(|v| {
                if b.proj.uncertain {
                    self.with_uncertain(m, (&place, &unit), d, &v, ctx)
                } else {
                    Ok(v)
                }
            }),
        };
        if ctx.at.is_none() && self.tracing() {
            self.trace_bal(m, walked, (place, unit, ctx.cut), (d, b), &got)?;
        }
        got
    }

    /// A plain balance read at cut `cut`, traced as a node `explain` prints in the balance
    /// form from the main walk it read, a refused one alone; none for a refusal of load's or
    /// a form not built (ruling P63, #132). E §10.
    fn trace_bal(
        &mut self,
        m: &Reality,
        walked: crate::eval::Walked,
        (place, unit, cut): (String, String, Day),
        (day, b): (Day, &Bal),
        got: &Result<Value, Stop>,
    ) -> Result<(), Stop> {
        let (item, walk) = match got {
            Ok(v) => (
                crate::eval::Item::Value(v.clone()),
                Some(self.main_walk(m, cut)?),
            ),
            Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => {
                (crate::eval::Item::Refused(r.clone()), None)
            }
            Err(_) => return Ok(()),
        };
        let read = crate::eval::TracedBalance {
            place,
            unit,
            day,
            start: b.start,
            uncertain: b.proj.uncertain,
            cut,
            timeline: self.timeline_key(),
        };
        self.trace_balance(m, walked, read, (item, walk));
        Ok(())
    }

    /// A read's place: a place, or the hole it is (refcore's `place_arg`). L §5.10.
    pub(crate) fn place_arg(
        &mut self,
        m: &Reality,
        e: &Expr,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Result<String, Hole>, Stop> {
        let v = self.ev(m, e, env, ctx)?;
        if let Value::Hole(h) = v {
            return Ok(Err(h));
        }
        if v == Value::None {
            return Err(refuse(
                Run::None,
                format!("{} is none here: a place", expr_text(e)?),
            ));
        }
        if let Value::Key(KeyVal::Thing(name)) = &v
            && m.things.get(name).is_some_and(|t| m.is_place(t))
        {
            return Ok(Ok(name.clone()));
        }
        let shown = v.text(m).map_err(crate::eval::num_stop)?;
        Err(refuse(Run::Domain, format!("{shown} is not a place")))
    }

    /// A balance read's unit: the one written, or the place's (refcore's `bal_unit`).
    /// L §5.10 Kind and unit.
    pub(crate) fn bal_unit(
        &mut self,
        m: &Reality,
        b: &Bal,
        place: &str,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Result<String, Hole>, Stop> {
        let own = m
            .things
            .get(place)
            .and_then(|t| m.place_unit(t))
            .filter(|u| u != "any");
        if let [unit_arg, _] = b.args.as_slice() {
            let u = self.ev(m, unit_arg, env, ctx)?;
            if let Value::Hole(h) = u {
                return Ok(Err(h));
            }
            let name = match &u {
                Value::Key(k) => k.text(),
                Value::Window(_) => return Err(Stop::NotBuilt("a unit read as a window")),
                Value::Number(_)
                | Value::Money(_)
                | Value::Bool(_)
                | Value::Day(_)
                | Value::Period(_)
                | Value::MonthDay(..)
                | Value::Text(_)
                | Value::Code(_)
                | Value::Record(_)
                | Value::List(_)
                | Value::Calendar(_)
                | Value::None
                | Value::Never
                | Value::Word(_)
                | Value::Hole(_)
                | Value::Mv(_)
                | Value::OutKey { .. } => u.text(m).map_err(crate::eval::num_stop)?,
            };
            if let Some(pu) = &own
                && *pu != name
            {
                return Err(Stop::Refused(Refusal::new(
                    Code::Load(Load::Unit),
                    format!("bal({place}, {name}, d): {place} holds {pu}"),
                )));
            }
            return Ok(Ok(name));
        }
        super::projection::own_unit(m, place).map(Ok)
    }

    /// A plain balance on a day: at the cut, from the main walk; inside a decision, a running
    /// read of a day before its own (refcore's `bal_day`). L §5.14.1.
    pub(super) fn bal_day(
        &mut self,
        m: &Reality,
        (place, unit): (&str, &str),
        d: Day,
        key: At,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        if let Some(at) = ctx.at {
            let t = at.point.day;
            self.note_read(at.walk, place, Sees::Before)?;
            if key.day > t || (key.day == t && key.at_or_after_decisions()) {
                if d > t {
                    return Err(refuse(
                        Run::Future,
                        format!(
                            "a decision on {} reads {place} at {} after its day",
                            t.text(),
                            d.text()
                        ),
                    ));
                }
                let reader = self.running_move(at.walk)?;
                return Err(refuse(
                    Run::Cycle,
                    format!(
                        "{reader} reads itself through bal({place}, {}): a decision's plain read of its own day",
                        d.text()
                    ),
                ));
            }
            let v = self.read_state(m, at.walk, place, unit, key)?;
            return self.less_own_obs(m, at.walk, (place, unit), key, v);
        }
        let id = self.main_walk(m, ctx.cut)?;
        self.read_state(m, id, place, unit, key)
    }

    /// A running read of a modelled decision, against the booked balance less its own
    /// observations booked on their days before its day (refcore's `less_own_obs`).
    /// E §4.2 step 3, L §5.19.
    fn less_own_obs(
        &self,
        m: &Reality,
        id: usize,
        (place, unit): (&str, &str),
        key: At,
        v: Value,
    ) -> Result<Value, Stop> {
        let Some(legs) = &self.walk(id)?.less_obs else {
            return Ok(v);
        };
        if !matches!(v, Value::Money(_) | Value::Number(_)) {
            return Ok(v);
        }
        let mut delta = Num::zero();
        for lg in legs {
            let Some(o) = lg.day.day() else {
                continue;
            };
            let (Amount::Qty(q), true) = (&lg.amount, key >= At::of(o, super::point::Step::Plans))
            else {
                continue;
            };
            if unit != ALL && lg.unit.as_deref().is_some_and(|u| u != ALL && u != unit) {
                continue;
            }
            if lg.from.place() == Some(place) {
                delta = delta.checked_add(q).map_err(crate::eval::num_stop)?;
            }
            if lg.to.place() == Some(place) {
                delta = delta.checked_sub(q).map_err(crate::eval::num_stop)?;
            }
        }
        if delta.is_zero() {
            return Ok(v);
        }
        let d = qty_value(m, &delta, unit)?;
        crate::eval::arith(m, crate::eval::Op::Add, &v, &d)
    }

    /// The move of the decision running in walk `id`. L §5.14.1.
    fn running_move(&self, id: usize) -> Result<String, Stop> {
        let walk = self.walk(id)?;
        let today = walk
            .today
            .as_ref()
            .ok_or_else(|| invariant("a running read with no day open"))?;
        match today.acts.get(today.next) {
            Some((_, super::agenda::Act::Decide(mv, _))) => Ok(mv.clone()),
            Some(_) | None => Err(invariant("a running read outside a decision")),
        }
    }

    /// Notes a running decision's read of a place: every read, for `explain`'s "read at its
    /// walk point" (E §10), and where it sees the place at its point, for the order of the
    /// day's decisions (refcore's `note_read`). L §5.14.2.
    pub(super) fn note_read(&mut self, id: usize, place: &str, sees: Sees) -> Result<(), Stop> {
        let walk = self.walk_mut(id)?;
        if let Some(set) = &mut walk.reads {
            set.insert(place.to_owned());
        }
        match sees {
            Sees::AtPoint => {
                if let Some(set) = &mut walk.live {
                    set.insert(place.to_owned());
                }
            }
            Sees::Before => {}
        }
        Ok(())
    }

    /// A place's balance at a point of a walk, with its register: the entries since its last
    /// reading, in a projection's walk those of the walks it continues before its point, the
    /// adjustments with a remainder there, the moves left out that touch it, in this walk and
    /// those it continues (refcore's `read_state`, `merge_cum`, `st_value`). L §5.10, §8.2;
    /// E §4.3.
    pub(super) fn read_state(
        &mut self,
        m: &Reality,
        id: usize,
        place: &str,
        unit: &str,
        at: At,
    ) -> Result<Value, Stop> {
        let walk = self.walk(id)?;
        let st = walk.pstate(m, place, unit, at);
        let anchor = walk.anchor_question(place);
        let mut met = Collected::default();
        for (day, entry) in walk.adj_regs.get(place).into_iter().flatten() {
            if *day <= at.day {
                met.entries.insert(entry.clone());
            }
        }
        self.chain_register(m, (id, at), (place, unit), &mut met)?;
        self.absorb(&met);
        state_value(m, &st, unit, &anchor)
    }

    /// A balance's register through point `at` of walk `id` and of each walk it continues
    /// before its point, until one met a reading there (refcore's `read_state`, its chain).
    /// L §8.2, E §4.3 step 1.
    pub(super) fn chain_register(
        &mut self,
        m: &Reality,
        (id, at): (usize, At),
        (place, unit): (&str, &str),
        met: &mut Collected,
    ) -> Result<(), Stop> {
        let mut next = Some((id, at));
        let mut links = 0_usize;
        while let Some((wid, point)) = next {
            links = links
                .checked_add(1)
                .filter(|n| *n <= self.walks.len())
                .ok_or_else(|| invariant("a chain of walks that loops"))?;
            next = self.chain_link(m, (wid, point), (place, unit), met)?;
        }
        Ok(())
    }

    /// One walk of a read's chain: the moves left out there that touch the place, the register
    /// of its entries since the last reset through `at`; the walk it continues and the point,
    /// when no reset was seen (refcore's `read_state`, its chain, and `merge_cum`). L §8.2,
    /// E §4.3 step 1.
    fn chain_link(
        &mut self,
        m: &Reality,
        (id, at): (usize, At),
        (place, unit): (&str, &str),
        met: &mut Collected,
    ) -> Result<Option<(usize, At)>, Stop> {
        let walk = self.walk(id)?;
        for mid in walk.left_out_at.get(place).into_iter().flatten() {
            let (n, first) = walk
                .left_out
                .get(mid)
                .ok_or_else(|| invariant("a move left out with no count"))?;
            let first = first.map_or_else(|| "None".to_owned(), Day::text);
            met.entries.insert(Entry {
                word: "left out",
                text: format!("{mid}, {n} instances, first {first}"),
                figure: String::new(),
            });
        }
        let (st, last) = walk.state(m, place, unit, at);
        let reset_seen = last.is_some() && st.rst.is_some();
        let continued = walk.proj.as_ref().map(|p| (p.base, p.point));
        if let Some(i) = last {
            let cum = self.walk_mut(id)?.cum(place, unit, i);
            met.merge(&cum);
        }
        Ok(continued.filter(|_| !reset_seen))
    }

    /// `bal(x, d) with uncertain`: the balance plus the movements of uncertain kinds dated
    /// through d since its last reading (refcore's `with_uncertain`). L §5.14.
    pub(super) fn with_uncertain(
        &mut self,
        m: &Reality,
        (place, unit): (&str, &str),
        day: Day,
        read: &Value,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let id = self.main_walk(m, ctx.cut)?;
        let walk = self.walk(id)?;
        let (st, _) = walk.state(m, place, unit, At::end(day));
        let since = st
            .rst
            .and_then(|r| walk.entries(place, unit).get(r))
            .map(|e| e.at);
        let window = Window::SinceReading { day, since };
        self.uncertain_sum(m, id, (place, unit), read, window)
    }

    /// A projection's balance `with uncertain`: plus the movements of uncertain kinds its
    /// walk keeps, dated t through d (refcore's `proj_uncertain`). L §5.14.1, E §4.3 step 2.
    pub(super) fn proj_uncertain(
        &mut self,
        m: &Reality,
        id: usize,
        (place, unit): (&str, &str),
        day: Day,
        read: &Value,
    ) -> Result<Value, Stop> {
        let start = self.walk(id)?.start;
        let window = Window::Projected { start, day };
        self.uncertain_sum(m, id, (place, unit), read, window)
    }

    /// A balance plus the uncertain movements walk `id` keeps at the place inside `window`,
    /// with `uncertain:` in the register; a hole where an amount is one (refcore's
    /// `with_uncertain`, `proj_uncertain`). L §5.14.
    fn uncertain_sum(
        &mut self,
        m: &Reality,
        id: usize,
        (place, unit): (&str, &str),
        read: &Value,
        window: Window,
    ) -> Result<Value, Stop> {
        let walk = self.walk(id)?;
        let mut acc = Uncertain {
            add: Num::zero(),
            holes: Vec::new(),
            met: Collected::default(),
        };
        let added = acc.movements(&walk.uncertain, place, window);
        // refcore registers as it adds: a refusal keeps the `uncertain:` entries before it
        self.absorb(&acc.met);
        added?;
        let mut all: Vec<Option<&Hole>> = acc.holes.iter().map(Some).collect();
        if let Value::Hole(h) = read {
            all.push(Some(h));
        }
        if let Some(h) = spread(&all) {
            return Ok(Value::Hole(h));
        }
        let (q, _) = super::plan::quantity(m, read)?;
        qty_value(
            m,
            &q.checked_add(&acc.add).map_err(crate::eval::num_stop)?,
            unit,
        )
    }

    /// Walk `id`, then each walk it continues, the nearest first: a projection's walk
    /// continues the walk it starts from (refcore's loops over `w.base`). E §4.3 step 1,
    /// L §5.14.1.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a chain that loops, one longer than the walks there are.
    pub(super) fn chain(&self, id: usize) -> Result<Vec<usize>, Stop> {
        let mut chain = Vec::new();
        let mut next = Some(id);
        while let Some(wid) = next {
            if chain.len() >= self.walks.len() {
                return Err(invariant("a chain of walks that loops"));
            }
            chain.push(wid);
            next = self.walk(wid)?.proj.as_ref().map(|p| p.base);
        }
        Ok(chain)
    }

    /// The main walk of the timeline of cut `cut`, from S to E, walked to its end (refcore's
    /// `main_walk`). E §3.3, §4.2.
    ///
    /// # Errors
    ///
    /// The refusal the walk met; `Deep` while a cell it needs waits on the work stack;
    /// `NotBuilt` for a read of a walk while it is walked.
    pub(crate) fn main_walk(&mut self, m: &Reality, cut: Day) -> Result<usize, Stop> {
        self.trace_walk();
        let id = if let Some(id) = self.main_walks.get(&cut) {
            *id
        } else {
            let id = self.walks.len();
            self.walks
                .push(Walk::new(cut, (m.start, m.end), self.timeline_world()));
            #[cfg(feature = "counters")]
            self.counts.add(crate::Unit::Walk);
            self.main_walks.insert(cut, id);
            id
        };
        if self.walking.contains(&id) {
            return Err(Stop::NotBuilt("a read of a walk while it is walked"));
        }
        self.walking.insert(id);
        self.push();
        let walked = self.advance(m, id);
        self.pop_alone();
        self.walking.remove(&id);
        walked?;
        // a scenario's movements before its from that read its facts, listed with every
        // result read from the walk (E §9 item 3)
        for text in self.walk(id)?.listed.clone() {
            self.note("before from", text, String::new());
        }
        Ok(id)
    }

    /// The walk of the timeline of cut `cut` from S through `day`, its plan pass listing no
    /// plan's instance dated after it: what a leg read that day needs while the main walk
    /// needs a cell still being computed outside it. Kept per day, a walk to a later day
    /// serving an earlier one; none where it is refused at run (refcore's `prefix_walk`,
    /// `plan_list`). E §3.2 (evaluation is demand-driven), §4.1, §4.2 step 5.
    ///
    /// # Errors
    ///
    /// A load refusal met at run and `ENGINE-INVARIANT`, which stop the read (R-66); `Deep`
    /// while a cell it needs waits on the work stack; `NotBuilt` for a leg whose day is a
    /// hole, and for a read of a walk while it is walked.
    pub(crate) fn prefix_walk(
        &mut self,
        m: &Reality,
        cut: Day,
        day: Option<Day>,
    ) -> Result<Option<usize>, Stop> {
        let Some(day) = day else {
            return Err(Stop::NotBuilt(
                "a leg's read whose walk needs a cell still being computed, its day a hole",
            ));
        };
        let kept = self
            .prefixes
            .walks
            .range((cut, day)..)
            .next()
            .filter(|((c, _), _)| *c == cut)
            .map(|((_, upto), id)| (*upto, *id));
        let (upto, id) = if let Some(found) = kept {
            found
        } else {
            let id = self.walks.len();
            let end = day.min(m.end);
            let mut walk = Walk::new(cut, (m.start, end), self.timeline_world());
            walk.upto = Some(day);
            self.walks.push(walk);
            #[cfg(feature = "counters")]
            self.counts.add(crate::Unit::Walk);
            self.prefixes.walks.insert((cut, day), id);
            (day, id)
        };
        if self.walking.contains(&id) {
            return Err(Stop::NotBuilt("a read of a walk while it is walked"));
        }
        let outer = self.body_upto.replace(upto);
        self.walking.insert(id);
        self.push();
        let walked = self.advance(m, id);
        self.pop_alone();
        self.walking.remove(&id);
        self.body_upto = outer;
        match walked {
            Ok(()) => Ok(Some(id)),
            Err(Stop::Refused(r)) if super::order::run_time(&r) => Ok(None),
            Err(stop) => Err(stop),
        }
    }

    /// `gap(x, d)`: the gap at the last `stated` reading of x on day d in the main walk at
    /// the cut, with what it rests on; `HOLE-NOREADING` where no reading of x falls that day
    /// (refcore's `m_c_gap`). L §5.19, E §6.1, L §8.1.
    ///
    /// # Errors
    ///
    /// The refusal the booked balance at the reading is; the refusals of the walk.
    pub(crate) fn gap(
        &mut self,
        m: &Reality,
        (place_arg, day_arg): (&Expr, &Expr),
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let place = match self.place_arg(m, place_arg, env, ctx)? {
            Err(h) => return Ok(Value::Hole(h)),
            Ok(p) => p,
        };
        let d = self.ev(m, day_arg, env, ctx)?;
        if let Value::Hole(_) = d {
            return Ok(d);
        }
        let d = match crate::eval::one_day(m, &d, "the day of gap")? {
            Ok(d) => d,
            Err(h) => return Ok(Value::Hole(h)),
        };
        // a decision inside the main walk reads the gaps it has met so far, the walk as it
        // stands (refcore's `main_walk` of a running walk)
        let id = match self.main_walks.get(&ctx.cut) {
            Some(id) if self.walking.contains(id) => *id,
            _ => self.main_walk(m, ctx.cut)?,
        };
        let gaps = self.walk(id)?.gaps.get(&(place.clone(), d));
        let Some(last) = gaps.map(Vec::as_slice).and_then(latest_gap).cloned() else {
            let text = format!("stated[{place}] has no reading at {}", d.text());
            return Ok(Value::Hole(Hole::asked(&text, &format!("stated[{place}]"))));
        };
        self.absorb(&last.coll);
        match last.gap {
            Amount::Qty(q) => qty_value(m, &q, &last.unit),
            Amount::Hole(h) => Ok(Value::Hole(h)),
            Amount::Refused(r) => Err(Stop::Refused(r)),
        }
    }
}

/// The gap of a day's last applied reading: of the latest instant, the last applied there
/// (refcore sorts them by instant, stably, and takes the last). L §5.19 `gap(x, d)`.
fn latest_gap(gaps: &[GapAt]) -> Option<&GapAt> {
    gaps.iter().max_by_key(|g| g.at)
}

/// What `with uncertain` adds to a balance: the uncertain movements' signed amounts, their
/// holes, their register.
struct Uncertain {
    /// The amounts added.
    add: Num,
    /// The holes met.
    holes: Vec<Hole>,
    /// The register met.
    met: Collected,
}

impl Uncertain {
    /// The legs at `place` of the uncertain movements dated inside `window`, each added; a
    /// refusal stops the sum, what it met before kept here (refcore's `with_uncertain`, its
    /// loop). L §5.14, §8.2.
    fn movements(&mut self, movements: &[Inst], place: &str, window: Window) -> Result<(), Stop> {
        for movement in movements {
            for leg in &movement.legs {
                let dated = leg
                    .day
                    .day()
                    .or_else(|| movement.day.as_ref().and_then(Dated::day));
                if counted(dated, window) {
                    self.leg(place, movement, leg)?;
                }
            }
        }
        Ok(())
    }

    /// One uncertain movement's leg at `place`: `uncertain:` in the register, its amount
    /// signed by its direction (refcore's `with_uncertain`, a leg). L §5.14.
    fn leg(&mut self, place: &str, movement: &Inst, leg: &super::plan::LegVal) -> Result<(), Stop> {
        for (end, sign) in [(&leg.from, -1_i64), (&leg.to, 1_i64)] {
            if end.place() != Some(place) {
                continue;
            }
            self.met.entries.insert(Entry {
                word: "uncertain",
                text: movement.id.clone(),
                figure: String::new(),
            });
            self.met.merge(&movement.coll);
            let q = match &leg.amount {
                Amount::Hole(h) => {
                    self.holes.push(h.clone());
                    continue;
                }
                Amount::Refused(r) => return Err(Stop::Refused(r.clone())),
                Amount::Qty(q) => q,
            };
            let signed = if sign < 0 {
                q.checked_neg()
            } else {
                Ok(q.clone())
            };
            let signed = signed.map_err(crate::eval::num_stop)?;
            self.add = self
                .add
                .checked_add(&signed)
                .map_err(crate::eval::num_stop)?;
        }
        Ok(())
    }
}

/// A state's value: the refusal, a hole with every question it rests on, or the balance
/// (refcore's `st_value`). L §5.10, §8.1.
pub(crate) fn state_value(m: &Reality, st: &St, unit: &str, anchor: &Hole) -> Result<Value, Stop> {
    if let Some(r) = &st.bad {
        return Err(Stop::Refused(r.clone()));
    }
    let missing = st.val.is_none().then_some(anchor);
    if let Some(h) = spread(&[st.hole.as_ref(), st.perm.as_ref(), missing]) {
        return Ok(Value::Hole(h));
    }
    let v = st
        .val
        .as_ref()
        .ok_or_else(|| invariant("a balance with no value"))?;
    qty_value(m, v, unit)
}

/// The walk holds the days S to E: a walked value of a day outside them is the hole naming
/// the end it passes (refcore's `m_bal_at`). L §8.1 `HOLE-PRERUN`, `HOLE-POSTRUN`; E §2.
pub(super) fn outside_run(m: &Reality, d: Day, figure: &str) -> Option<Hole> {
    if d < m.start {
        Some(Hole::before_run(figure))
    } else if d > m.end {
        Some(Hole::after_run(figure))
    } else {
        None
    }
}

/// A flows target's text where a message names it (refcore's `fmt_node`). L §8.4.
pub(super) fn expr_text(e: &Expr) -> Result<String, Stop> {
    if let Expr::Name(n) = e {
        return Ok(n.clone());
    }
    Err(Stop::NotBuilt(
        "the text of an expression other than a name",
    ))
}

/// A run-time refusal (L §8.4).
pub(super) fn refuse(code: Run, message: String) -> Stop {
    Stop::Refused(Refusal::new(Code::Run(code), message))
}

/// Which uncertain movements a balance read `with uncertain` counts (L §5.14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Window {
    /// A plain balance through `day`: those dated by then and after the place's last
    /// reading, `since`.
    SinceReading {
        /// The day read.
        day: Day,
        /// The point of the place's last reading.
        since: Option<At>,
    },
    /// A projection's from `start` through `day` (L §5.14.1).
    Projected {
        /// The projection's first day, t.
        start: Day,
        /// The day read.
        day: Day,
    },
}

/// Whether an uncertain movement dated `dated` counts in a balance read with `window`
/// (refcore's `with_uncertain`, `proj_uncertain`). L §5.14, §5.14.1.
fn counted(dated: Option<Day>, window: Window) -> bool {
    let Some(d) = dated else {
        return false;
    };
    match window {
        Window::SinceReading { day, since: None } => d <= day,
        Window::SinceReading {
            day,
            since: Some(s),
        } => d <= day && At::end(d) > s,
        Window::Projected { start, day } => start <= d && d <= day,
    }
}
