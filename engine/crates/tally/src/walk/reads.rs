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

use std::collections::BTreeSet;

use super::plan::{Amount, Dated, HoleSrc, Inst, Status, invariant, qty_value};
use super::point::At;
use super::state::{ALL, GapAt, St, Walk};
use crate::base::{Code, Load, Refusal, Run};
use crate::book::Stands;
use crate::eval::{Collected, Ctx, Engine, Entry, Env, Rows};
use crate::hole::{Hole, Linear, spread};
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
        // a projection's own `windows each` holds for that read: the line's run's choice
        // (L §4.4)
        let inside = if b.proj.windows_each {
            self.read_windows(ctx)?
        } else {
            None
        };
        let Some(inside) = inside else {
            return self.bal_read(m, b, env, ctx);
        };
        self.open_layer(m, inside)?;
        let v = self.bal_read(m, b, env, inside);
        self.end_layer(inside.layer, v.as_ref().err())?;
        v
    }

    /// [`Engine::bal`] in context `ctx`, a projection's own clause taken. L §5.10, §5.14.1.
    fn bal_read(&mut self, m: &Reality, b: &Bal, env: &Env, ctx: Ctx) -> Result<Value, Stop> {
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
        let Some(day_arg) = b.args.last() else {
            let Some(at) = ctx.at else {
                return Err(Stop::Refused(Refusal::new(
                    Code::Load(Load::Reads),
                    format!("bal({place}) with no day outside a decision"),
                )));
            };
            self.note_read(at.walk, &place, Sees::AtPoint)?;
            let v = self.read_state(m, at.walk, &place, &unit, at.point)?;
            let v = self.less_own_obs(m, at.walk, (&place, &unit), at.point, v)?;
            self.trace_running(m, &place, &v, at.point.day)?;
            return Ok(v);
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
        if b.modelled {
            if ctx.at.is_none() && self.tracing() {
                return Err(Stop::NotBuilt("explain of a modelled balance"));
            }
            // before S or after E it is the run's hole, as every balance (E §6.2, §2)
            if let Some(hole) = outside {
                return Ok(hole);
            }
            // the plain read's instant, its start included (tally 2.1.8 R16, B5)
            let key = if b.start { At::start(d) } else { At::end(d) };
            return self.bal_modelled(m, (&place, &unit), key, ctx.cut);
        }
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
        // traced wherever a read is: a walk reads apart, so inside a decision only a walk-bound
        // def's cell read again at its point traces it (E §10)
        if self.tracing() {
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
                    Load::Unit.at_run(),
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
        let v = self.read_state(m, id, place, unit, key)?;
        if ctx.partial
            && let Value::Hole(h) = v
        {
            return self.partial_bal(m, id, (place, unit), key, h);
        }
        Ok(v)
    }

    /// A balance read by a `partial` line: "its opening plus what is booked since" (L §5.19,
    /// §5.10), a sum, so the line prints its known part and each hole leg's amount, read again
    /// as the line reads it and signed by its direction at the place (L §5.18); a hole of any
    /// other making stays the plain hole (refcore's `m_partial_bal`).
    pub(super) fn partial_bal(
        &mut self,
        m: &Reality,
        id: usize,
        (place, unit): (&str, &str),
        key: At,
        v: Hole,
    ) -> Result<Value, Stop> {
        // a projection is its start balance plus the counted legs: its start's hole terms are
        // the walk it continues at the point, read the same way, up the chain (E §4.3; L §5.18,
        // R15; B9 of group 3's review); the chain's walks bound the steps (R-16)
        let mut at = (id, key);
        let mut known: Option<Value> = None;
        let mut legs: Vec<(Hole, bool)> = Vec::new();
        for _ in 0..=self.walks.len() {
            let walk = self.walk(at.0)?;
            let base = walk.proj.as_ref().map(|p| (p.base, p.point));
            if !walk.logs.contains_key(&(place.to_owned(), unit.to_owned())) {
                // a projection that books nothing at the place: its start, the base's balance
                let st = walk.default_state(m, place, unit);
                let plain = st.val.is_none() || st.bad.is_some() || st.perm.is_some();
                let (Some(next), false) = (base, plain || !st.imp.is_empty()) else {
                    return Ok(Value::Hole(v));
                };
                at = next;
                continue;
            }
            let Some(read) = walk.partial_legs(m, place, unit, at.1)? else {
                return Ok(Value::Hole(v));
            };
            for (into, src) in &read.legs {
                let Value::Hole(h) = self.ev(m, &src.amount, &src.env, src.ctx.partial())? else {
                    return Ok(Value::Hole(v));
                };
                legs.push((h, *into));
            }
            if !read.all_met {
                return Ok(Value::Hole(v));
            }
            // the known part is this walk's own: the start's is already in its value
            if known.is_none() {
                known = Some(qty_value(m, &read.known, unit)?);
            }
            let (Some(next), Some(_)) = (base, &read.start) else {
                let known = known.ok_or_else(|| invariant("a partial balance's known part"))?;
                let legs: Vec<(&Hole, bool)> = legs.iter().map(|(h, into)| (h, *into)).collect();
                return crate::eval::lin_balance(m, (&v, known), &legs);
            };
            at = next;
        }
        Err(invariant("a projection's chain longer than its walks"))
    }

    /// Each hole leg of `legs`, its hole, its direction at the place and where its amount was
    /// read, as a `partial` line reads it: read again under the line's clause, its linear part
    /// signed by its direction; none where one isn't linear, or is linear in other holes than
    /// its own (refcore's `leg_lin`). L §5.18 (tally 2.1.8 R15).
    pub(super) fn legs_linear(
        &mut self,
        m: &Reality,
        legs: &[(Hole, bool, HoleSrc)],
    ) -> Result<Option<Vec<Linear>>, Stop> {
        let mut out = Vec::with_capacity(legs.len());
        for (hole, into, src) in legs {
            let Value::Hole(h) = self.ev(m, &src.amount, &src.env, src.ctx.partial())? else {
                return Ok(None);
            };
            let Some(lin) = h.linear_part().filter(|_| h.same_questions(hole)) else {
                return Ok(None);
            };
            out.push(if *into {
                lin
            } else {
                crate::eval::lin_negated(m, lin)?
            });
        }
        Ok(Some(out))
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
        for own in legs {
            let lg = &own.leg;
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
            // only the ends its row booked: of two rows stated same, each books the end its
            // statement covers (L §5.19)
            if own.from && lg.from.place() == Some(place) {
                delta = delta.checked_add(q).map_err(crate::eval::num_stop)?;
            }
            if own.to && lg.to.place() == Some(place) {
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
    /// those it continues (refcore's `read_state`, `merge_cum`, `st_value`). The run's ends
    /// are tested here, once, so every walked read of a day outside them (`bal`, `value`, a
    /// running read) is the hole naming the end it passes (refcore's `run_ends`). L §5.10,
    /// §8.1 `HOLE-PRERUN`, `HOLE-POSTRUN`, §8.2; E §2, §4.3.
    pub(super) fn read_state(
        &mut self,
        m: &Reality,
        id: usize,
        place: &str,
        unit: &str,
        at: At,
    ) -> Result<Value, Stop> {
        if let Some(hole) = outside_run(m, at.day, place) {
            return Ok(Value::Hole(hole));
        }
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
            met.entries.insert(Entry {
                word: "left out",
                text: format!("{mid}, {n} instances, first {}", first.text()),
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
        self.uncertain_sum(m, id, (place, unit), read, (window, ctx))
    }

    /// A projection's balance `with uncertain`: plus the movements of uncertain kinds its
    /// walk keeps, dated t through d (refcore's `proj_uncertain`). L §5.14.1, E §4.3 step 2.
    pub(super) fn proj_uncertain(
        &mut self,
        m: &Reality,
        id: usize,
        (place, unit): (&str, &str),
        (day, read): (Day, &Value),
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let start = self.walk(id)?.start;
        let window = Window::Projected { start, day };
        self.uncertain_sum(m, id, (place, unit), read, (window, ctx))
    }

    /// A balance plus the uncertain movements walk `id` keeps at the place inside `window`,
    /// with `uncertain:` in the register; a hole where an amount is one, under `partial` a sum
    /// linear in the balance and each hole leg's amount (refcore's `with_uncertain`,
    /// `proj_uncertain`, `lin_fold`). L §5.14, §5.18 (tally 2.1.8 R15).
    fn uncertain_sum(
        &mut self,
        m: &Reality,
        id: usize,
        (place, unit): (&str, &str),
        read: &Value,
        (window, ctx): (Window, Ctx),
    ) -> Result<Value, Stop> {
        let walk = self.walk(id)?;
        let mut acc = Uncertain {
            add: Num::zero(),
            holes: Vec::new(),
            legs: Vec::new(),
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
        let Some(h) = spread(&all) else {
            let (q, _) = super::plan::quantity(m, read)?;
            let total = q.checked_add(&acc.add).map_err(crate::eval::num_stop)?;
            return qty_value(m, &total, unit);
        };
        if !ctx.partial || acc.legs.len() != acc.holes.len() {
            return Ok(Value::Hole(h));
        }
        // the balance's own known part and terms, where it is a hole that has them
        let (known, own) = if let Value::Hole(balance) = read {
            match balance.printed_linear() {
                Some(Linear {
                    known: Some(part),
                    terms,
                }) => (part.clone(), terms.clone()),
                Some(_) | None => return Ok(Value::Hole(h)),
            }
        } else {
            (read.clone(), Vec::new())
        };
        let Some(mut parts) = self.legs_linear(m, &acc.legs)? else {
            return Ok(Value::Hole(h));
        };
        parts.push(Linear {
            known: None,
            terms: own,
        });
        let (booked, _) = super::plan::quantity(m, &known)?;
        let total = booked
            .checked_add(&acc.add)
            .map_err(crate::eval::num_stop)?;
        crate::eval::lin_fold(m, &h, qty_value(m, &total, unit)?, parts)
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
        Ok(id)
    }

    /// The main walk of cut `cut`, walked to its end, unless it is being walked (a read from
    /// inside its own walk): none then. E §3.3, §4.2.
    ///
    /// # Errors
    ///
    /// As [`Engine::main_walk`].
    pub(crate) fn free_main_walk(&mut self, m: &Reality, cut: Day) -> Result<Option<usize>, Stop> {
        let busy = self.main_walks.get(&cut);
        if busy.is_some_and(|id| self.walking.contains(id)) {
            return Ok(None);
        }
        self.main_walk(m, cut).map(Some)
    }

    /// The status of decision instance `mid`'s run in the walk a read in context `ctx` sees,
    /// and whether observations the walk books stand in for it (none dated before S,
    /// [`crate::book::stands_in`]): the walk it reads at, or else the main walk of its cut,
    /// through the walks it continues; none where m hasn't run there, or where that main walk
    /// is being walked (refcore's `after_day`, its run and its `legs`). E §4.2 step 3, L §4.6
    /// Readings, §5.19 Before the run.
    ///
    /// # Errors
    ///
    /// As [`Engine::main_walk`].
    pub(crate) fn run_status(
        &mut self,
        m: &Reality,
        mid: &str,
        ctx: Ctx,
    ) -> Result<Option<(Status, bool)>, Stop> {
        let id = if let Some(at) = ctx.at {
            at.walk
        } else {
            let Some(id) = self.free_main_walk(m, ctx.cut)? else {
                return Ok(None);
            };
            id
        };
        for wid in self.chain(id)? {
            let walk = self.walk(wid)?;
            if let Some(ix) = walk.run_ids.get(mid) {
                let run = walk
                    .runs
                    .get(*ix)
                    .ok_or_else(|| invariant("a run by its id"))?;
                let (status, links, cut) = (run.inst.status, run.inst.obs.clone(), walk.cut);
                let obs = self.observations(m, cut)?;
                let stands = crate::book::stands_in(obs, &links, (m.start, m.end, cut));
                return Ok(Some((status, stands)));
            }
        }
        Ok(None)
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
    /// the cut, with what it rests on; `HOLE-NOREADING` where no reading of x falls that day;
    /// at a reading outside [S, E], `HOLE-PRERUN` or `HOLE-POSTRUN`, since the booked balance
    /// there is the run's hole (refcore's `m_c_gap`). L §5.19, E §6.1, L §8.1 (tally 2.1.8
    /// item 1).
    ///
    /// # Errors
    ///
    /// The refusal the booked balance at the reading is; `RUN-DOMAIN` where two units'
    /// readings stand at the instant (L §5.10); the refusals of the walk.
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
            // a reading at an instant outside [S, E]: the booked balance it is less is the
            // run's hole there, so the gap is too (L §5.19 gap, §8.1; tally 2.1.8 item 1)
            if let Some(h) = outside_run(m, d, &place)
                && self.stated_on(m, &place, d, ctx.cut)?
            {
                return Ok(Value::Hole(h));
            }
            let text = format!("stated[{place}] has no reading at {}", d.text());
            return Ok(Value::Hole(Hole::asked(&text, &format!("stated[{place}]"))));
        };
        // two units' readings at the instant gap reads: refused (L §5.10; tally 2.1.8 R14)
        let mut units: BTreeSet<String> = BTreeSet::new();
        for g in gaps.into_iter().flatten() {
            if g.instant == last.instant {
                units.insert(g.unit.clone());
            }
        }
        if units.len() > 1 {
            let units: Vec<String> = units.into_iter().collect();
            return Err(crate::read::two_units("gap", &place, &last.instant, &units));
        }
        self.absorb(&last.coll);
        match last.gap {
            Amount::Qty(q) => qty_value(m, &q, &last.unit),
            Amount::Hole(h) => Ok(Value::Hole(h)),
            Amount::Refused(r) => Err(Stop::Refused(r)),
        }
    }
}

impl Engine {
    /// Whether cut `cut` sees a `stated` reading of place `place` at an instant on day `day`
    /// (refcore's `m_c_gap`, its `readings`). L §5.19 gap, §4.6 Readings.
    ///
    /// # Errors
    ///
    /// As [`Engine::seen_readings`].
    fn stated_on(&mut self, m: &Reality, place: &str, day: Day, cut: Day) -> Result<bool, Stop> {
        let world = self.timeline_world();
        let after = self.after_days(m, cut)?;
        let seen = self.seen_readings(m, (cut, world), (Rows::Unkept, &after))?;
        Ok(seen.iter().any(|s| {
            s.input == "stated"
                && s.place == place
                && s.day == day
                && matches!(s.stands, Stands::Edge(_))
        }))
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
    /// The holes of them that are a leg's amount, each with its direction at the place and
    /// where it was read (L §5.18; tally 2.1.8 R15).
    legs: Vec<(Hole, bool, HoleSrc)>,
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
                    .or_else(|| movement.day.as_ref().and_then(Dated::lo));
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
                    if let Some(src) = &leg.src {
                        self.legs.push((h.clone(), sign > 0, HoleSrc::clone(src)));
                    }
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
    if let Some(h) = st.questions(missing) {
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
