//! The walk's balance reads: a place's balance at an instant, plain, running or with uncertain, with what it rests on in the register.
//! Spec: L §5.10 (`bal(x)`, `bal(x, d)`, `bal(x, d start)`, `bal(x, u, d)`; `HOLE-ANCHOR`),
//! §5.14 (`with uncertain`), §5.14.1 (running reads: a decision reads the walk at its point;
//! `RUN-FUTURE`, a plain read of its own day `RUN-CYCLE`), §5.19 (a modelled decision reads
//! the balances less its own observations booked before its day); §8.2 (the register:
//! `unfulfilled:`, `left out:`, `adjustment:`, `uncertain:`, `open:`); E §4.2 (a refused leg
//! refuses every read of it; holes in the walk), E §4.3 (a read of a projection's walk rests on the walks it continues), E §3.3 (the
//! main walk of a timeline). The flows reads (L §5.14.3) are `flows.rs`'s.
//! Twin: engine/tools/refcore/mreads.py (`place_arg`, `bal_unit`, `read_state`,
//! `merge_cum`, `e_bal`, `note_read`, `bal_day`, `with_uncertain`, `proj_uncertain`,
//! `less_own_obs`) and walk.py (`m_main_walk`).
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
use crate::model::{Domain, KeyVal, Priced, Reality, Value};
use crate::num::Num;
use crate::syntax::{Bal, Expr, Price, Stop, Valuation};
use crate::time::Day;

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
            return Err(Stop::NotBuilt("bal of a party or a batch (G9)"));
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
            self.note_live(at.walk, &place)?;
            let v = self.read_state(m, at.walk, &place, &unit, at.point)?;
            return self.less_own_obs(m, at.walk, (&place, &unit), at.point, v);
        };
        let d = self.ev(m, day_arg, env, ctx)?;
        if let Value::Hole(_) = d {
            return Ok(d);
        }
        let d = crate::ask::date_of(&d, "the day of bal")?;
        if b.proj.from.is_some() {
            return self.projection(m, b, (&place, &unit), d, (env, ctx));
        }
        let key = if b.start { At::start(d) } else { At::end(d) };
        let v = self.bal_day(m, (&place, &unit), d, key, ctx)?;
        if b.proj.uncertain {
            return self.with_uncertain(m, (&place, &unit), d, &v, ctx);
        }
        Ok(v)
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
    fn bal_unit(
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

    /// Notes a running decision's read of a place: where it sees the place at its point, for
    /// the order of the day's decisions (refcore's `note_read`). L §5.14.2.
    pub(super) fn note_live(&mut self, id: usize, place: &str) -> Result<(), Stop> {
        let walk = self.walk_mut(id)?;
        if let Some(set) = &mut walk.live {
            set.insert(place.to_owned());
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
        let mut next = Some((id, at));
        let mut links = 0_usize;
        while let Some((wid, point)) = next {
            links = links
                .checked_add(1)
                .filter(|n| *n <= self.walks.len())
                .ok_or_else(|| invariant("a chain of walks that loops"))?;
            next = self.chain_link(m, (wid, point), (place, unit), &mut met)?;
        }
        self.absorb(&met);
        state_value(m, &st, unit, &anchor)
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
    fn with_uncertain(
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
        let d = crate::ask::date_of(&d, "the day of gap")?;
        let id = self.main_walk(m, ctx.cut)?;
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

    /// `value(x, d) in CUR [by p]`: the place's positions at the end of day d at their prices
    /// in CUR, multiplied exactly: a position in CUR at its amount, a place priced whole at its
    /// price input, every other position at its unit's price; a hole naming a position and
    /// its price where it has none (`HOLE-PRICE`), never left in its own unit (refcore's
    /// `m_e_value`). L §5.10 (Value, A priced place), §8.1.
    ///
    /// # Errors
    ///
    /// `LOAD-PRICE`, `LOAD-UNIT`, `LOAD-KEY` for a price load lets by and the run meets;
    /// `RUN-DOMAIN` for a price in another unit, `RUN-NONE` for a written price that is none;
    /// the refusals of the walk; `NotBuilt` for `of q` and a price clause with keys.
    pub(crate) fn valuation(
        &mut self,
        m: &Reality,
        read: &Valuation,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let place = match self.place_arg(m, &read.place, env, ctx)? {
            Err(h) => return Ok(Value::Hole(h)),
            Ok(p) => p,
        };
        let thing = m
            .things
            .get(&place)
            .ok_or_else(|| invariant("a place that is no thing"))?;
        let d = self.ev(m, &read.day, env, ctx)?;
        if let Value::Hole(_) = d {
            return Ok(d);
        }
        let day = crate::ask::date_of(&d, "the day of value")?;
        let valued = Valued {
            place: place.clone(),
            day,
            priced: m.price_of(thing),
            by: read.by.as_ref().map(|by| by_form(m, env, by)),
        };
        if let Some(price) = &valued.priced
            && let Some((id, Priced::Whole(whole))) = m.clause_mode(price)
        {
            return self.value_whole(m, (read, &valued), (price, &id, &whole), ctx);
        }
        if read.of.is_some() {
            return Err(Stop::NotBuilt("value of a party or a batch (G9)"));
        }
        // a place of unit any: the units its log holds, in the walk the read sees
        let units = if let Some(u) = m.place_unit(thing).filter(|u| u != "any") {
            vec![u]
        } else {
            let id = match ctx.at {
                Some(at) => at.walk,
                None => self.main_walk(m, ctx.cut)?,
            };
            self.walk(id)?.units_of(&place)
        };
        let cur = read.currency.as_str();
        let mut total = Num::zero();
        let mut holes = Vec::new();
        for u in units {
            let q = self.bal_day(m, (&place, &u), day, At::end(day), ctx)?;
            if let Value::Hole(h) = q {
                holes.push(h);
                continue;
            }
            let (q, _) = super::plan::quantity(m, &q)?;
            if u == cur {
                total = total.checked_add(&q).map_err(crate::eval::num_stop)?;
                continue;
            }
            if q.is_zero() {
                continue;
            }
            match self.position_price(m, (read, &valued), &u, env, ctx)? {
                Err(h) => holes.push(h),
                Ok(p) => {
                    let worth = q.checked_mul(&p).map_err(crate::eval::num_stop)?;
                    total = total.checked_add(&worth).map_err(crate::eval::num_stop)?;
                }
            }
        }
        if let Some(h) = spread(&holes.iter().map(Some).collect::<Vec<_>>()) {
            return Ok(Value::Hole(h));
        }
        in_currency(cur, total)
    }

    /// A place priced whole, `priced p` with p a value in a currency: p at the value's day,
    /// in CUR through `by p` when p is in another (refcore's `m_e_value`, its `whole` arm).
    /// L §5.10 (A priced place).
    fn value_whole(
        &mut self,
        m: &Reality,
        (read, valued): (&Valuation, &Valued),
        (price, id, whole): (&Price, &str, &str),
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let amount = match self.priced_read(m, (read, valued), (price, id), None, ctx)? {
            Err(h) => return Ok(Value::Hole(h)),
            Ok(q) => q,
        };
        let cur = read.currency.as_str();
        if whole == cur {
            return in_currency(cur, amount);
        }
        let (pid, last) = match &valued.by {
            Some(By::UnkeyedLast(pid)) => (pid.as_str(), true),
            Some(By::Keyed(pid, last)) => (pid.as_str(), *last),
            Some(By::Written) | None => {
                return Err(Stop::Refused(Refusal::new(
                    Code::Load(Load::Price),
                    format!(
                        "value({}) in {cur} holds units in {whole} and states no by",
                        valued.place
                    ),
                )));
            }
        };
        match self.named_price(m, (read, valued), (pid, last), whole, ctx)? {
            Err(h) => Ok(Value::Hole(h)),
            Ok(p) => {
                let worth = amount.checked_mul(&p).map_err(crate::eval::num_stop)?;
                in_currency(cur, worth)
            }
        }
    }

    /// The price in CUR of one unit `u` of a position on the value's day: by the `by` written,
    /// else by the place's price clause (refcore's `m_e_value`, its loop). L §5.10.
    fn position_price(
        &mut self,
        m: &Reality,
        (read, valued): (&Valuation, &Valued),
        u: &str,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Result<Num, Hole>, Stop> {
        let cur = read.currency.as_str();
        let (place, day) = (valued.place.as_str(), valued.day);
        match (&valued.by, &read.by) {
            (Some(By::Written), Some(by)) => {
                // an expression of type num CUR/u with its keys, read at the value's day
                let at = Ctx {
                    law: Some(day),
                    ..ctx
                };
                let p = if by.last {
                    self.last_read(m, (&by.input, &read.day), env, at)?
                } else {
                    self.ev(m, &by.input, env, at)?
                };
                if let Value::Hole(h) = p {
                    return Ok(Err(h));
                }
                if p == Value::None {
                    let text = format!("the price of {u} in {cur} on {} is none here", day.text());
                    return Err(refuse(Run::None, text));
                }
                per_unit_price(m, &p, (place, cur, u)).map(Ok)
            }
            (Some(By::UnkeyedLast(pid)), _) => {
                // one unit's price: the position's unit must be its unit
                let p = self.price_at(m, (pid, Vec::new()), (day, true), ctx)?;
                if matches!(p, Value::Hole(_) | Value::None) {
                    return priced_value(m, (read, valued), p, &format!("{pid}[]"), u);
                }
                per_unit_price(m, &p, (place, cur, u)).map(Ok)
            }
            (Some(By::Keyed(pid, last)), _) => {
                self.named_price(m, (read, valued), (pid, *last), u, ctx)
            }
            (Some(By::Written), None) | (None, _) => {
                self.priced_position(m, (read, valued), u, ctx)
            }
        }
    }

    /// A position in unit `u` of a place with no `by`: at the place's own price clause, which
    /// prices `u` in CUR (refcore's `m_e_value`, its `priced` arm). L §5.10.
    fn priced_position(
        &mut self,
        m: &Reality,
        (read, valued): (&Valuation, &Valued),
        u: &str,
        ctx: Ctx,
    ) -> Result<Result<Num, Hole>, Stop> {
        let (cur, place) = (read.currency.as_str(), valued.place.as_str());
        let Some(price) = &valued.priced else {
            return Err(Stop::Refused(Refusal::new(
                Code::Load(Load::Price),
                format!("value({place}) in {cur} holds units in {u} and states no by"),
            )));
        };
        let Some((id, Priced::PerUnit(pc, per))) = m.clause_mode(price) else {
            return Err(invariant("a place priced by no price load let by"));
        };
        if pc != cur || per != u {
            return Err(Stop::Refused(Refusal::new(
                Code::Load(Load::Unit),
                format!("value({place}) in {cur}: {place} is priced in {pc}/{per}, not {cur}/{u}"),
            )));
        }
        self.priced_read(m, (read, valued), (price, &id), Some(u), ctx)
    }

    /// A price named by `by p` or `by last(p)`, an input or def keyed by unit, read at unit
    /// (or currency) `u` on the value's day; a hole naming `u` where it is `none` (refcore's
    /// `m_price_read`). L §5.10, §4.6, §8.1.
    fn named_price(
        &mut self,
        m: &Reality,
        (read, valued): (&Valuation, &Valued),
        (pid, last): (&str, bool),
        u: &str,
        ctx: Ctx,
    ) -> Result<Result<Num, Hole>, Stop> {
        let (given, figure) = if key_count(m, pid).is_some_and(|n| n != 0) {
            (
                vec![Value::Key(KeyVal::Thing(u.to_owned()))],
                format!("{pid}[{u}]"),
            )
        } else {
            (Vec::new(), format!("{pid}[]"))
        };
        let p = self.price_at(m, (pid, given), (valued.day, last), ctx)?;
        priced_value(m, (read, valued), p, &figure, u)
    }

    /// A place's own price clause read on the value's day: at the place's thing, or at its
    /// position's unit `u`, when the price input is keyed by a kind holding one; a hole naming
    /// the place or `u` where it is `none` (refcore's `m_priced_read`). L §5.10, §4.6, §8.1.
    fn priced_read(
        &mut self,
        m: &Reality,
        (read, valued): (&Valuation, &Valued),
        (price, pid): (&Price, &str),
        u: Option<&str>,
        ctx: Ctx,
    ) -> Result<Result<Num, Hole>, Stop> {
        if !matches!(price.input, Expr::Name(_)) {
            return Err(Stop::NotBuilt("a price clause written with its keys"));
        }
        let place = valued.place.as_str();
        let key = match first_domain(m, pid) {
            None => None,
            Some(Domain::Kind(kind)) if m.thing_in(place, &kind) => Some(place),
            Some(Domain::Kind(kind)) if unit_in(m, u, &kind) => u,
            Some(domain) => {
                let over = match &domain {
                    Domain::Kind(id) | Domain::Index(id) | Domain::Axis(id) => id.as_str(),
                    Domain::List(_) | Domain::Range => "",
                };
                return Err(Stop::Refused(Refusal::new(
                    Code::Load(Load::Key),
                    format!(
                        "{place} is priced by {pid}, keyed by {over}, which holds neither {place} nor its unit"
                    ),
                )));
            }
        };
        let (given, figure) = match key {
            Some(k) => (
                vec![Value::Key(KeyVal::Thing(k.to_owned()))],
                format!("{pid}[{k}]"),
            ),
            None => (Vec::new(), format!("{pid}[]")),
        };
        let p = self.price_at(m, (pid, given), (valued.day, price.last), ctx)?;
        let who = match u {
            Some(u) => u,
            None => place,
        };
        priced_value(m, (read, valued), p, &figure, who)
    }
}

/// What a `value` reads besides the read as written: its place, its day, the place's price
/// clause (on its entry or its kind, L §5.10) and the form of the read's `by`.
struct Valued {
    /// The place.
    place: String,
    /// The day.
    day: Day,
    /// The place's price clause.
    priced: Option<Price>,
    /// The form of the read's `by p`.
    by: Option<By>,
}

/// How a value's `by p` reads a price (refcore's `n._price`, `_price_kind`,
/// `_price_unkeyed`, `_price_expr`, which load's `m_t_value` sets). L §5.10.
enum By {
    /// `by last(px)`, px an unkeyed input: one unit's price, through its carry.
    UnkeyedLast(String),
    /// `by px` or `by last(px)`, px an input or def keyed by unit: read at the position's
    /// unit, through its carry with `last`.
    Keyed(String, bool),
    /// An expression of type `num CUR/u` written with its keys, `by px[month_of(d)]`, or
    /// `by last(px[k])`: read at the value's day, as the read writes it.
    Written,
}

/// The form of a `by p` (load's `Checker::by`, at run). L §5.10.
fn by_form(m: &Reality, env: &Env, by: &Price) -> By {
    if let Expr::Name(id) = &by.input
        && env.get(id).is_none()
    {
        match key_count(m, id) {
            Some(0) if by.last => return By::UnkeyedLast(id.clone()),
            Some(n) if n != 0 => return By::Keyed(id.clone(), by.last),
            Some(_) | None => {}
        }
    }
    By::Written
}

/// How many keys an input or a def takes; none for another id. L §5.8, §5.11.
fn key_count(m: &Reality, id: &str) -> Option<usize> {
    if let Some(input) = m.inputs.get(id) {
        return Some(input.binds.len());
    }
    m.defs.get(id).map(|def| def.binds.len())
}

/// The domain of an input's or a def's first key; none for no key or another id.
/// L §5.8, §5.11.
fn first_domain(m: &Reality, id: &str) -> Option<Domain> {
    if let Some(input) = m.inputs.get(id) {
        return input.binds.first().map(|b| b.domain.clone());
    }
    m.defs
        .get(id)
        .and_then(|def| def.binds.first())
        .map(|b| b.domain.clone())
}

/// Whether a position's unit `u` is a thing of `kind`, for a price keyed by that kind.
/// L §5.7, §5.10.
fn unit_in(m: &Reality, u: Option<&str>, kind: &str) -> bool {
    let Some(u) = u else {
        return false;
    };
    m.things.get(u).is_some() && m.thing_in(u, kind)
}

/// The gap of a day's last applied reading: of the latest instant, the last applied there
/// (refcore sorts them by instant, stably, and takes the last). L §5.19 `gap(x, d)`.
fn latest_gap(gaps: &[GapAt]) -> Option<&GapAt> {
    gaps.iter().max_by_key(|g| g.at)
}

/// A price read's quantity; `none` a hole naming the position and the price at its keys,
/// `figure` (refcore's `no price` hole). L §5.10, §8.1 `HOLE-PRICE`.
fn priced_value(
    m: &Reality,
    (read, valued): (&Valuation, &Valued),
    p: Value,
    figure: &str,
    who: &str,
) -> Result<Result<Num, Hole>, Stop> {
    if let Value::Hole(h) = p {
        return Ok(Err(h));
    }
    if p == Value::None {
        let text = format!(
            "{who} has no price in {} on {}",
            read.currency,
            valued.day.text()
        );
        return Ok(Err(Hole::asked(&text, figure)));
    }
    Ok(Ok(super::plan::quantity(m, &p)?.0))
}

/// A price written as an expression, or read unkeyed, must be in CUR per the position's unit
/// (refcore's `m_e_value`, its unit test). L §5.10.
fn per_unit_price(
    m: &Reality,
    p: &Value,
    (place, cur, u): (&str, &str, &str),
) -> Result<Num, Stop> {
    let fits = if let Value::Number(n) = p {
        let mut terms = n.unit().terms().to_vec();
        terms.sort();
        let mut want = vec![(cur.to_owned(), 1), (u.to_owned(), -1)];
        want.sort();
        terms == want
    } else {
        false
    };
    if !fits {
        let shown = p.text(m).map_err(crate::eval::num_stop)?;
        return Err(refuse(
            Run::Domain,
            format!("value({place}) in {cur}: the price {shown} is not in {cur}/{u}"),
        ));
    }
    Ok(super::plan::quantity(m, p)?.0)
}

/// A value's result: a `num CUR`, never rounded (L §5.10: `value` multiplies exactly).
fn in_currency(cur: &str, total: Num) -> Result<Value, Stop> {
    let unit = crate::model::Unit::make(vec![(cur.to_owned(), 1)])
        .map_err(|_| invariant("a currency's unit"))?;
    Ok(Value::Number(crate::model::Number::plain(total, unit)))
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
fn state_value(m: &Reality, st: &St, unit: &str, anchor: &Hole) -> Result<Value, Stop> {
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
