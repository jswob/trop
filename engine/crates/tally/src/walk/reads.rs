//! The walk's reads: a place's balance at an instant, its flows over a span, a move's flows, each with what it rests on in the register.
//! Spec: L §5.10 (`bal(x)`, `bal(x, d)`, `bal(x, d start)`, `bal(x, u, d)`; `HOLE-ANCHOR`),
//! §5.14 (`with uncertain`), §5.14.1 (running reads: a decision reads the walk at its point;
//! `RUN-FUTURE`, a plain read of its own day `RUN-CYCLE`), §5.14.3 (`flows(x, span)`,
//! `flows(m, span)`: their units, holes and refusals); §8.2 (the register: `unfulfilled:`,
//! `left out:`, `adjustment:`, `uncertain:`); E §4.2 (a refused leg refuses every read of it;
//! holes in the walk), E §4.3 (a read of a projection's walk rests on the walks it continues),
//! E §3.3 (the main walk of a timeline).
//! Twin: engine/tools/refcore/mreads.py (`place_arg`, `bal_unit`, `read_state`,
//! `merge_cum`, `e_bal`, `note_read`, `bal_day`, `with_uncertain`, `proj_uncertain`,
//! `e_flows`, `flows_place`, `flows_move`, `leg_refused`) and walk.py (`m_main_walk`).
//! Invariants: a read inside a decision sees the walk only up to its point; a read at the cut
//! sees the whole main walk; a hole a read meets is printed with every question it rests on.

use std::collections::BTreeSet;

use super::order::WalkAt;
use super::plan::{Amount, Dated, Exists, Inst, Status, invariant, qty_value};
use super::point::At;
use super::state::{ALL, Change, Progress, St, Walk};
use crate::base::{Code, Load, Refusal, Run};
use crate::eval::{Collected, Ctx, Engine, Entry, Env};
use crate::hole::{Hole, spread};
use crate::model::{KeyVal, MoveInfo, Reality, Span, Value};
use crate::num::Num;
use crate::syntax::{Bal, Expr, Flows, Key, Postfix, Stop};
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
            return Err(Stop::NotBuilt("a modelled balance (G8)"));
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
            return self.read_state(m, at.walk, &place, &unit, at.point);
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
    fn place_arg(
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
        let Some(pu) = own else {
            return Err(Stop::Refused(Refusal::new(
                Code::Load(Load::Unit),
                format!("bal({place}, d) of a place of unit any states its unit: bal(x, u, d)"),
            )));
        };
        Ok(Ok(pu))
    }

    /// A plain balance on a day: at the cut, from the main walk; inside a decision, a running
    /// read of a day before its own (refcore's `bal_day`). L §5.14.1.
    fn bal_day(
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
            return self.read_state(m, at.walk, place, unit, key);
        }
        let id = self.main_walk(m, ctx.cut)?;
        self.read_state(m, id, place, unit, key)
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
        for movement in &walk.uncertain {
            for leg in &movement.legs {
                let dated = leg
                    .day
                    .day()
                    .or_else(|| movement.day.as_ref().and_then(Dated::day));
                if counted(dated, window) {
                    acc.leg(place, movement, leg)?;
                }
            }
        }
        self.absorb(&acc.met);
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

    /// `flows(x, span)`, of a place or of a move (refcore's `e_flows`). L §5.14.3.
    ///
    /// # Errors
    ///
    /// The refusals of the walk and of the movements counted; `RUN-DOMAIN` for flows in no
    /// one unit; `NotBuilt` for a conversion and `for k` (G7).
    pub(crate) fn flows(
        &mut self,
        m: &Reality,
        f: &Flows,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let sp = match self.span_value(m, &f.span, env, ctx)? {
            Err(h) => return Ok(Value::Hole(h)),
            Ok(sp) => sp,
        };
        let named = match move_parts(&f.target) {
            Some((name, posts)) => m.moves.get(name).map(|mv| (mv, posts)),
            None => None,
        };
        if sp.start().is_none_or(|s| s < m.start) {
            let figure = match named {
                Some((mv, _)) => mv.id.clone(),
                None => expr_text(&f.target)?,
            };
            return Ok(Value::Hole(Hole::asked("before the run's start", &figure)));
        }
        if f.for_key.is_some() || f.convert.is_some() {
            return Err(Stop::NotBuilt("flows for a period or converted (G7)"));
        }
        let Some((mv, posts)) = named else {
            return match self.place_arg(m, &f.target, env, ctx)? {
                Err(h) => Ok(Value::Hole(h)),
                Ok(place) => self.flows_place(m, &place, &sp, ctx),
            };
        };
        let (keys, leg) = keys_and_leg(posts)?;
        self.flows_move(m, (mv, keys, leg), &sp, env, ctx)
    }

    /// A place's flows over a span: every movement booked there in it, holes and refusals
    /// included (refcore's `flows_place`). L §5.14.3, E §4.2.
    fn flows_place(
        &mut self,
        m: &Reality,
        place: &str,
        sp: &Span,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let last = span_last(m, sp)?;
        let mut limit = At::end(last);
        let id = match ctx.at {
            Some(at) => {
                let t = at.point.day;
                if last >= t {
                    self.note_live(at.walk, place)?;
                }
                if last > t {
                    return Err(refuse(
                        Run::Future,
                        format!(
                            "a decision on {} reads flows of {place} through {}, after its day",
                            t.text(),
                            last.text()
                        ),
                    ));
                }
                if last == t {
                    limit = at.point;
                }
                at.walk
            }
            None => self.main_walk(m, ctx.cut)?,
        };
        let start = sp
            .start()
            .ok_or_else(|| invariant("a span with no start"))?;
        let lo = At::before(start);
        let pu = m
            .things
            .get(place)
            .and_then(|t| m.place_unit(t))
            .ok_or_else(|| invariant("a place with no unit"))?;
        let walk = self.walk(id)?;
        let units = if pu == "any" {
            walk.units_of(place)
        } else {
            vec![pu.clone()]
        };
        let mut sums = Sums::default();
        sums.global(walk, lo, limit)?;
        for unit in units.iter().map(String::as_str).chain([ALL]) {
            sums.place_log(walk, place, unit, (lo, limit))?;
        }
        for (day, entry) in walk.adj_regs.get(place).into_iter().flatten() {
            if *day <= last {
                sums.met.entries.insert(entry.clone());
            }
        }
        self.absorb(&sums.met);
        if let Some(h) = spread(&sums.holes.iter().map(Some).collect::<Vec<_>>()) {
            return Ok(Value::Hole(h));
        }
        if pu == "any" {
            return Err(refuse(
                Run::Domain,
                format!("flows of {place}, a place of unit any: it sums no one unit"),
            ));
        }
        one_unit(m, place, &sums.totals, &units)
    }

    /// A move's flows over a span: its movements dated in it, a decision's as the walk ran
    /// them, the adjustments nothing settles a hole (refcore's `flows_move`). L §5.14.3, E §5.
    fn flows_move(
        &mut self,
        m: &Reality,
        (mv, keys, leg): (&MoveInfo, &[Key], Option<&str>),
        sp: &Span,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let want = match self.move_keys(m, mv, keys, env, ctx)? {
            Err(h) => return Ok(Value::Hole(h)),
            Ok(k) => k,
        };
        let last = span_last(m, sp)?;
        let insts = if mv.decision {
            self.decision_runs(m, mv, last, ctx)?
        } else {
            self.move_instances(m, mv, ctx.cut)?
        };
        let mut holes: Vec<Hole> = Vec::new();
        let mut total: Option<Num> = None;
        let mut units: BTreeSet<String> = BTreeSet::new();
        let mut met = Collected::default();
        for i in &insts {
            if !i.keys.starts_with(&want) {
                continue;
            }
            if matches!(i.status, Status::None | Status::Uncertain) {
                continue;
            }
            let day = match &i.day {
                Some(Dated::Hole(h)) => {
                    holes.push(h.clone());
                    continue;
                }
                Some(Dated::Day(d)) => *d,
                None => {
                    return Err(i
                        .err
                        .clone()
                        .map_or_else(|| invariant("an instance with no day"), Stop::Refused));
                }
            };
            if !sp.contains(day) {
                continue;
            }
            if let Some(err) = &i.err {
                return Err(Stop::Refused(err.clone()));
            }
            met.merge(&i.coll);
            if let Exists::Unknown(h) = &i.exists {
                holes.push(h.clone());
                continue;
            }
            if i.status == Status::Past {
                met.entries.insert(Entry {
                    word: "unfulfilled",
                    text: i.id.clone(),
                    figure: String::new(),
                });
            }
            for lg in i.legs.iter().filter(|lg| wanted(leg, &lg.name)) {
                let q = match &lg.amount {
                    Amount::Refused(r) => return Err(Stop::Refused(r.clone())),
                    Amount::Hole(h) => {
                        holes.push(h.clone());
                        continue;
                    }
                    Amount::Qty(q) => q,
                };
                if !q.is_zero()
                    && let Some(err) = self.leg_refused(m, (&i.id, &lg.name), ctx)?
                {
                    return Err(Stop::Refused(err));
                }
                total = Some(match total {
                    None => q.clone(),
                    Some(t) => t.checked_add(q).map_err(crate::eval::num_stop)?,
                });
                if let Some(u) = &lg.unit {
                    units.insert(u.clone());
                }
            }
        }
        if !mv.decision {
            for adj in self.adjust_move(m, mv, ctx.cut)? {
                if leg.is_some_and(|l| l != adj.leg.name) {
                    continue;
                }
                let adjusted = insts.iter().find(|i| i.id == adj.inst_id);
                if adjusted.is_some_and(|i| !i.keys.starts_with(&want)) {
                    continue;
                }
                if last >= adj.day {
                    holes.push(adj.question());
                }
            }
        }
        self.absorb(&met);
        if let Some(h) = spread(&holes.iter().map(Some).collect::<Vec<_>>()) {
            return Ok(Value::Hole(h));
        }
        if units.len() > 1 {
            let all: Vec<&str> = units.iter().map(String::as_str).collect();
            return Err(refuse(
                Run::Domain,
                format!("flows of {}: movements in {}", mv.id, all.join(" and ")),
            ));
        }
        let sum = match total {
            Some(t) => t,
            None => Num::zero(),
        };
        if let Some(u) = units.into_iter().next() {
            return qty_value(m, &sum, &u);
        }
        match static_unit(m, mv, leg) {
            Some(u) => qty_value(m, &sum, &u),
            None => Err(refuse(
                Run::Domain,
                format!(
                    "flows of {} over a span with no movement: its unit is unknown",
                    mv.id
                ),
            )),
        }
    }

    /// A move's keys as written in a flows read: all of them, or none (refcore's
    /// `flows_move`, its keys). L §5.14.3.
    fn move_keys(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        keys: &[Key],
        env: &Env,
        ctx: Ctx,
    ) -> Result<Result<Vec<KeyVal>, Hole>, Stop> {
        if keys.is_empty() {
            return Ok(Ok(Vec::new()));
        }
        if keys.len() != mv.binds.len() {
            return Err(Stop::NotBuilt("flows of a move at some of its keys"));
        }
        let mut given = Vec::with_capacity(keys.len());
        for key in keys {
            let Key::Expr(e) = key else {
                return Err(Stop::NotBuilt("`*` in a move's keys"));
            };
            given.push(self.given_key(m, e, env, ctx)?);
        }
        crate::eval::make_keys(m, &mv.id, &mv.binds, given, ctx.cut)
    }

    /// A decision move's runs a flows read counts: the main walk's, or inside a decision the
    /// running walk's before its point; an instance whose day is a hole counted as that hole
    /// (refcore's `flows_move`, its decisions). L §5.14.1, §5.14.3.
    fn decision_runs(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        last: Day,
        ctx: Ctx,
    ) -> Result<Vec<Inst>, Stop> {
        let (id, limit) = match ctx.at {
            Some(at) => {
                let t = at.point.day;
                if last > t {
                    return Err(refuse(
                        Run::Future,
                        format!(
                            "a decision on {} reads flows of {} through {}, after its day",
                            t.text(),
                            mv.id,
                            last.text()
                        ),
                    ));
                }
                (at.walk, (last == t).then_some(at.point))
            }
            None => (self.main_walk(m, ctx.cut)?, None),
        };
        let mut out = Vec::new();
        for i in self.move_instances(m, mv, ctx.cut)? {
            let walk = self.walk(id)?;
            let Some(run) = walk.run_ids.get(&i.id).and_then(|ix| walk.runs.get(*ix)) else {
                if matches!(i.day, Some(Dated::Hole(_))) && i.exists != Exists::No {
                    out.push(i);
                }
                continue;
            };
            if limit.is_some_and(|l| run.point >= l) {
                continue;
            }
            out.push(run.inst.clone());
        }
        Ok(out)
    }

    /// The refusal of a leg the walk refused: inside a decision, what its walk refused so far;
    /// at the ask's cut, the main walk's (refcore's `leg_refused`). E §4.2 step 5.
    fn leg_refused(
        &mut self,
        m: &Reality,
        key: (&str, &str),
        ctx: Ctx,
    ) -> Result<Option<Refusal>, Stop> {
        let wanted = (key.0.to_owned(), key.1.to_owned());
        if let Some(WalkAt { walk, .. }) = ctx.at {
            return Ok(self.walk(walk)?.bad_legs.get(&wanted).cloned());
        }
        let mut id = self.main_walks.get(&ctx.cut).copied();
        let busy = !self.plan_active.is_empty()
            || !self.adjust_active.is_empty()
            || !self.walking.is_empty();
        let unfinished = match id {
            None => true,
            Some(w) => {
                self.walk(w)?.progress == Progress::Walking
                    || self.walk(w)?.progress == Progress::Planning
            }
        };
        if !busy && ctx.cut == self.ask_cut && unfinished {
            let outer = self.guarded.replace(self.active.clone());
            let built = self.main_walk(m, ctx.cut);
            self.guarded = outer;
            id = match built {
                Ok(w) => Some(w),
                Err(Stop::Refused(_)) => return Ok(None),
                Err(stop) => return Err(stop),
            };
        }
        let Some(w) = id else {
            return Ok(None);
        };
        Ok(self.walk(w)?.bad_legs.get(&wanted).cloned())
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
            self.walks.push(Walk::new(cut, m.start, m.end));
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

/// What a place's flows gather: the totals per unit, the holes, the register.
#[derive(Default)]
struct Sums {
    /// The booked flows per unit.
    totals: std::collections::BTreeMap<String, Num>,
    /// The holes met.
    holes: Vec<Hole>,
    /// The register met.
    met: Collected,
}

impl Sums {
    /// What holds for the whole walk up to the span's end: a refusal from inside the span or
    /// from the walk's start, every hole (refcore's `flows_place`, its global log). L §5.14.3.
    fn global(&mut self, walk: &Walk, lo: At, limit: At) -> Result<(), Stop> {
        let first = At::before(walk.start);
        for e in walk.entries(ALL, ALL) {
            if e.at > limit {
                continue;
            }
            match &e.change {
                Change::Bad(r) if e.at > lo || e.at == first => {
                    return Err(Stop::Refused(r.clone()));
                }
                Change::Perm(h) | Change::Hole(h) => {
                    self.holes.push(h.clone());
                    if let Some(c) = &e.coll {
                        self.met.merge(c);
                    }
                }
                Change::Bad(_) | Change::Reset { .. } | Change::Delta(_) => {}
            }
        }
        Ok(())
    }

    /// One unit's log of the place over the span (refcore's `flows_place`, its logs). L §5.14.3.
    fn place_log(
        &mut self,
        walk: &Walk,
        place: &str,
        unit: &str,
        (lo, limit): (At, At),
    ) -> Result<(), Stop> {
        for e in walk.entries(place, unit) {
            if e.at <= lo {
                if let Change::Perm(h) = &e.change
                    && e.at <= limit
                {
                    self.holes.push(h.clone());
                }
                continue;
            }
            if e.at > limit {
                continue;
            }
            match &e.change {
                Change::Bad(r) => return Err(Stop::Refused(r.clone())),
                Change::Hole(h) | Change::Perm(h) => {
                    self.holes.push(h.clone());
                    self.met.entries.extend(e.reg.iter().cloned());
                    if let Some(c) = &e.coll {
                        self.met.merge(c);
                    }
                }
                Change::Delta(f) => {
                    self.met.entries.extend(e.reg.iter().cloned());
                    if let Some(c) = &e.coll {
                        self.met.merge(c);
                    }
                    let sum = self.totals.entry(unit.to_owned()).or_insert_with(Num::zero);
                    *sum = sum.checked_add(f).map_err(crate::eval::num_stop)?;
                }
                Change::Reset { .. } => {}
            }
        }
        Ok(())
    }
}

/// A place's flows in its one unit: `RUN-DOMAIN` for movements in two, or none whose unit is
/// known (refcore's `flows_place`, its end). L §5.14.3.
fn one_unit(
    m: &Reality,
    place: &str,
    totals: &std::collections::BTreeMap<String, Num>,
    units: &[String],
) -> Result<Value, Stop> {
    if totals.len() > 1 {
        let all: Vec<&str> = totals.keys().map(String::as_str).collect();
        return Err(refuse(
            Run::Domain,
            format!("flows of {place}: movements in {}", all.join(" and ")),
        ));
    }
    if let Some((u, q)) = totals.iter().next() {
        return qty_value(m, q, u);
    }
    if let [u] = units {
        return qty_value(m, &Num::zero(), u);
    }
    Err(refuse(
        Run::Domain,
        format!("flows of {place} over a span with no movement: its unit is unknown"),
    ))
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

/// A span's last day: the day before its end, or the run's last day (refcore's `last`). L §5.14.3.
fn span_last(m: &Reality, sp: &Span) -> Result<Day, Stop> {
    match sp.end() {
        Some(end) => end
            .previous()
            .ok_or_else(|| invariant("a span ending before the calendar")),
        None => Ok(m.end),
    }
}

/// A flows target written as a name with postfixes: a move's candidate name and its keys or
/// leg (refcore's `_form`). L §5.14.3.
fn move_parts(e: &Expr) -> Option<(&str, &[Postfix])> {
    if let Expr::Name(n) = e {
        return Some((n.as_str(), &[]));
    }
    if let Expr::Post(base, posts) = e
        && let Expr::Name(n) = base.as_ref()
    {
        return Some((n.as_str(), posts.as_slice()));
    }
    None
}

/// A move's keys and leg as a flows read writes them, `m[k]`, `m.leg`, `m[k].leg` (L §5.14.3).
fn keys_and_leg(posts: &[Postfix]) -> Result<(&[Key], Option<&str>), Stop> {
    match posts {
        [] => Ok((&[], None)),
        [Postfix::Keys(k)] => Ok((k.as_slice(), None)),
        [Postfix::Field(leg)] => Ok((&[], Some(leg.as_str()))),
        [Postfix::Keys(k), Postfix::Field(leg)] => Ok((k.as_slice(), Some(leg.as_str()))),
        [_, _, ..] => Err(Stop::NotBuilt("flows of a move's field")),
    }
}

/// A flows target's text where a message names it (refcore's `fmt_node`). L §8.4.
fn expr_text(e: &Expr) -> Result<String, Stop> {
    if let Expr::Name(n) = e {
        return Ok(n.clone());
    }
    Err(Stop::NotBuilt(
        "the text of an expression other than a name",
    ))
}

/// The unit of a move's flows load can see: its legs' kind unit when it is one, else their
/// places' one fixed unit (refcore's `_t`, `_leg_unit`). L §5.14.3.
fn static_unit(m: &Reality, mv: &MoveInfo, leg: Option<&str>) -> Option<String> {
    let kind_legs = m.kind_legs(&mv.kind);
    let mut found = BTreeSet::new();
    for lg in mv.decl.legs.iter().filter(|lg| wanted(leg, &lg.name)) {
        let unit = kind_legs
            .iter()
            .find(|(name, _, _)| *name == lg.name)
            .map(|(_, unit, _)| unit.clone());
        if let Some(u) = unit.filter(|u| m.currencies.contains_key(u) || m.units.contains(u)) {
            found.insert(u);
            continue;
        }
        for end in [&lg.from, &lg.to] {
            if let Expr::Name(p) = end
                && let Some(u) = m.things.get(p).and_then(|t| m.place_unit(t))
                && u != "any"
            {
                found.insert(u);
            }
        }
    }
    if found.len() == 1 {
        return found.into_iter().next();
    }
    None
}

/// A run-time refusal (L §8.4).
fn refuse(code: Run, message: String) -> Stop {
    Stop::Refused(Refusal::new(Code::Run(code), message))
}

/// Whether a leg's movement wanted by a flows read: every leg, or the one it names
/// (L §5.14.3). L §5.14.3.
fn wanted(leg: Option<&str>, name: &str) -> bool {
    match leg {
        None => true,
        Some(l) => l == name,
    }
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
