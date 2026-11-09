//! The walk's flows reads: a place's flows over a span and a move's flows, each with what it rests on in the register.
//! Spec: L §5.14.3 (`flows(x, span)`, `flows(m, span)`: their units, holes and refusals; a
//! move's flows at some of its keys; a modelled instance counted by the observations that stand
//! in for it, `open:` when none does; settlements dated in the span, `HOLE-CHANGED` where
//! nothing settles); §8.2 (the register: `unfulfilled:`, `left out:`, `adjustment:`, `open:`);
//! E §4.2 (a refused leg refuses every read of it), E §4.3 (a read of a projection's walk
//! rests on the walks it continues).
//! Twin: engine/tools/refcore/mreads.py (`e_flows`, `flows_place`, `flows_place_walk`,
//! `flows_move`, `leg_refused`).
//! Invariants: a refused `flows` keeps the register it met before the refusal, as refcore
//! registers while it counts; a leg refused by a walk a projection's walk continues is refused
//! in it too, and a decision run there counts in its `flows`.

use super::order::WalkAt;
use super::plan::{Amount, Dated, Exists, Inst, Status, invariant, qty_value};
use super::point::At;
use super::reads::{expr_text, refuse};
use super::state::{ALL, Change, Ent, Progress, Walk};
use crate::base::{Refusal, Run};
use crate::eval::{Collected, Ctx, Engine, Entry, Env};
use crate::hole::{Hole, spread};
use crate::model::{KeyVal, MoveInfo, Reality, Span, Value};
use crate::num::Num;
use crate::syntax::{Expr, Flows, Key, Postfix, Stop};
use crate::time::Day;
use std::collections::BTreeSet;

impl Engine {
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
    /// included; inside a projection's walk, each walk of its chain gives its entries up to
    /// the point the walk after it starts after (refcore's `flows_place`). L §5.14.3, E §4.2,
    /// E §4.3 step 1 (tally 2.1.2).
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
        let chain = self.walk_chain(id, limit)?;
        let units = if pu == "any" {
            let mut all = BTreeSet::new();
            for (wid, _) in &chain {
                all.extend(self.walk(*wid)?.units_of(place));
            }
            all.into_iter().collect()
        } else {
            vec![pu.clone()]
        };
        let mut sums = Sums::default();
        for (wid, cut) in &chain {
            if let Err(stop) = sums.logs(self.walk(*wid)?, place, &units, (lo, *cut)) {
                // refcore registers as it reads the logs: a refusal keeps what came before it
                self.absorb(&sums.met);
                return Err(stop);
            }
        }
        let walk = self.walk(id)?;
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
    /// them, the adjustments nothing settles a hole (refcore's `flows_move`). What the count
    /// met reaches the reader even when the count is refused, as refcore registers while it
    /// counts. L §5.14.3, E §5, L §8.2.
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
            std::rc::Rc::new(self.decision_runs(m, mv, last, ctx)?)
        } else {
            self.move_instances(m, mv, ctx.cut)?
        };
        let mut acc = Counted::default();
        let counted = self.count_move(
            m,
            (mv, &want, leg),
            (insts.as_slice(), sp, last),
            ctx,
            &mut acc,
        );
        self.absorb(&acc.met);
        counted?;
        let Counted {
            total,
            mut units,
            holes,
            settle_unit,
            ..
        } = acc;
        if units.is_empty()
            && let Some(u) = settle_unit
        {
            units.insert(u);
        }
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

    /// A move's movements and adjustments counted into `acc`, in refcore's order: each
    /// instance's register and `unfulfilled:` before its legs, so a refusal that stops the
    /// count leaves in `acc` what it met before (refcore's `flows_move`, its loop, which
    /// registers as it goes). L §5.14.3, E §5 steps 5–6, L §8.2.
    fn count_move(
        &mut self,
        m: &Reality,
        (mv, want, leg): (&MoveInfo, &[KeyVal], Option<&str>),
        (insts, sp, last): (&[Inst], &Span, Day),
        ctx: Ctx,
        acc: &mut Counted,
    ) -> Result<(), Stop> {
        for i in insts {
            if !i.keys.starts_with(want) {
                continue;
            }
            if matches!(i.status, Status::None | Status::Uncertain) {
                continue;
            }
            let day = match &i.day {
                Some(Dated::Hole(h)) => {
                    acc.holes.push(h.clone());
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
            if i.status == Status::Modelled {
                self.count_standing_in(m, i, leg, ctx.cut, acc)?;
                continue;
            }
            if let Some(err) = &i.err {
                return Err(Stop::Refused(err.clone()));
            }
            acc.met.merge(&i.coll);
            if let Exists::Unknown(h) = &i.exists {
                acc.holes.push(h.clone());
                continue;
            }
            if i.status == Status::Past {
                acc.met.entries.insert(Entry {
                    word: "unfulfilled",
                    text: i.id.clone(),
                    figure: String::new(),
                });
            }
            for lg in i.legs.iter().filter(|lg| wanted(leg, &lg.name)) {
                let q = match &lg.amount {
                    Amount::Refused(r) => return Err(Stop::Refused(r.clone())),
                    Amount::Hole(h) => {
                        acc.holes.push(h.clone());
                        continue;
                    }
                    Amount::Qty(q) => q,
                };
                if !q.is_zero()
                    && let Some(err) = self.leg_refused(m, (&i.id, &lg.name), ctx)?
                {
                    return Err(Stop::Refused(err));
                }
                acc.add(q, lg.unit.as_ref())?;
            }
        }
        if !mv.decision {
            for adj in self.settled_move(m, mv, ctx.cut)? {
                if leg.is_none_or(|l| l == adj.leg.name) && adj.inst_keys.starts_with(want) {
                    count_adjustment(m, &adj, sp, last, acc)?;
                }
            }
        }
        Ok(())
    }

    /// A modelled instance's flows: the observations that stand in for it on its covered
    /// day, `open:` when none does (refcore's `flows_move`, its modelled past). L §5.14.3,
    /// §5.19.
    fn count_standing_in(
        &mut self,
        m: &Reality,
        i: &Inst,
        leg: Option<&str>,
        cut: Day,
        acc: &mut Counted,
    ) -> Result<(), Stop> {
        let obs = self.standing_in(m, i, cut)?;
        if obs.is_empty() {
            acc.met.entries.insert(Entry {
                word: "open",
                text: i.id.clone(),
                figure: String::new(),
            });
        }
        for o in &obs {
            acc.met.merge(&o.coll);
            for lg in o.legs.iter().filter(|lg| wanted(leg, &lg.name)) {
                let Amount::Qty(q) = &lg.amount else {
                    return Err(invariant("an observed leg with no amount"));
                };
                acc.add(q, lg.unit.as_ref())?;
            }
        }
        Ok(())
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
        let Some(binds) = mv.binds.get(..keys.len()) else {
            return Err(Stop::NotBuilt("flows of a move at more keys than it binds"));
        };
        let mut given = Vec::with_capacity(keys.len());
        for key in keys {
            let Key::Expr(e) = key else {
                return Err(Stop::NotBuilt("`*` in a move's keys"));
            };
            given.push(self.given_key(m, e, env, ctx)?);
        }
        crate::eval::make_keys(m, &mv.id, binds, given, ctx)
    }

    /// A decision move's runs a flows read counts: the main walk's, or inside a decision the
    /// running walk's and the walks' it continues, before its point; an instance whose day is
    /// a hole counted as that hole, and one whose day is refused as its refusal, in any span
    /// (L §8.4, tally 2.1.6); one refused by its key's `where` is a run (refcore's
    /// `flows_move`, its decisions). L §5.14.1, §5.14.3, E §4.3.
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
        for i in self.move_instances(m, mv, ctx.cut)?.iter() {
            let Some(run) = self.chain_run(id, &i.id)? else {
                if matches!(i.day, Some(Dated::Hole(_)) | None) && i.exists != Exists::No {
                    out.push(i.clone());
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

    /// The run of decision instance `inst` in walk `id` or in a walk it continues, the
    /// nearest first: a projection's walk counts the runs the walks it continues made
    /// (refcore's `flows_move`, its loop over `ww.base`). L §5.14.1, E §4.3 step 1.
    fn chain_run(&self, id: usize, inst: &str) -> Result<Option<&super::order::Run>, Stop> {
        for wid in self.chain(id)? {
            let walk = self.walk(wid)?;
            if let Some(run) = walk.run_ids.get(inst).and_then(|ix| walk.runs.get(*ix)) {
                return Ok(Some(run));
            }
        }
        Ok(None)
    }

    /// Walk `id` read through `limit`, then each walk it continues read through the point
    /// the walk after it starts after, the nearest first (refcore's `flows_place`, its
    /// chain). E §4.3 step 1, L §5.14.1.
    fn walk_chain(&self, id: usize, limit: At) -> Result<Vec<(usize, At)>, Stop> {
        let mut out = Vec::new();
        let mut cut = limit;
        for wid in self.chain(id)? {
            out.push((wid, cut));
            if let Some(p) = &self.walk(wid)?.proj {
                cut = p.point.min(cut);
            }
        }
        Ok(out)
    }

    /// The refusal of a leg the walk refused: inside a decision, what its walk and the walks
    /// it continues refused so far; at the ask's cut, the main walk's (refcore's
    /// `leg_refused`). E §4.2 step 5, §4.3.
    pub(crate) fn leg_refused(
        &mut self,
        m: &Reality,
        key: (&str, &str),
        ctx: Ctx,
    ) -> Result<Option<Refusal>, Stop> {
        let wanted = (key.0.to_owned(), key.1.to_owned());
        if let Some(WalkAt { walk, .. }) = ctx.at {
            return self.chain_refusal(walk, &wanted);
        }
        let mut id = self.main_walks.get(&ctx.cut).copied();
        let busy = !self.plan_active.is_empty()
            || !self.adjust_active.is_empty()
            || !self.walking.is_empty();
        let unfinished = match id {
            None => true,
            Some(w) => matches!(
                self.walk(w)?.progress,
                Progress::Planning | Progress::Walking | Progress::Walked
            ),
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
        self.chain_refusal(w, &wanted)
    }

    /// The refusal of leg `wanted` in walk `id` or in a walk it continues, the nearest first:
    /// a projection's walk keeps what the walks it continues have refused so far (refcore's
    /// `leg_refused`, its loop over `w.base`). E §4.2 step 5, §4.3 step 1.
    fn chain_refusal(&self, id: usize, wanted: &(String, String)) -> Result<Option<Refusal>, Stop> {
        for wid in self.chain(id)? {
            if let Some(refusal) = self.walk(wid)?.bad_legs.get(wanted) {
                return Ok(Some(refusal.clone()));
            }
        }
        Ok(None)
    }
}

/// What a move's flows gather: the total, the units its legs are in, the holes, the
/// register, the unit of the first settlement counted.
#[derive(Default)]
struct Counted {
    /// The total, once a leg is counted.
    total: Option<Num>,
    /// The units of the legs counted.
    units: BTreeSet<String>,
    /// The holes met.
    holes: Vec<Hole>,
    /// The register met.
    met: Collected,
    /// The unit of the first settlement leg counted.
    settle_unit: Option<String>,
}

impl Counted {
    /// A leg's amount counted, its unit noted. L §5.14.3.
    fn add(&mut self, q: &Num, unit: Option<&String>) -> Result<(), Stop> {
        self.total = Some(add_to(self.total.take(), q)?);
        if let Some(u) = unit {
            self.units.insert(u.clone());
        }
        Ok(())
    }
}

/// One adjustment of a move's flows: refused over a span reaching its day when its change
/// is; `HOLE-CHANGED` from its day when nothing settles it; else its remainder in the
/// register and its settlements dated in the span, each signed by the way it runs
/// (refcore's `flows_move`, its adjustments). E §5 steps 5–6, L §5.14.3.
fn count_adjustment(
    m: &Reality,
    adj: &crate::book::Adjustment,
    sp: &Span,
    last: Day,
    acc: &mut Counted,
) -> Result<(), Stop> {
    if let Amount::Refused(err) = &adj.delta {
        // tally 2.1.2: settled or not, as an unsettled one is the HOLE-CHANGED hole
        if last >= adj.day {
            return Err(Stop::Refused(err.clone()));
        }
        return Ok(());
    }
    if adj.settled.is_empty() {
        if last >= adj.day {
            acc.holes.push(adj.question());
        }
        return Ok(());
    }
    if last >= adj.day
        && let Some(text) = adj.remainder_text(m)?
    {
        acc.met.entries.insert(Entry {
            word: "adjustment",
            text,
            figure: String::new(),
        });
    }
    for s in &adj.settled {
        if s.sign == 0 || !s.day.is_some_and(|d| sp.contains(d)) {
            continue;
        }
        let q = match &s.amount {
            Amount::Refused(r) => return Err(Stop::Refused(r.clone())),
            Amount::Hole(h) => {
                acc.holes.push(h.clone());
                continue;
            }
            Amount::Qty(q) => q
                .checked_mul(&Num::int(s.sign))
                .map_err(crate::eval::num_stop)?,
        };
        acc.total = Some(add_to(acc.total.take(), &q)?);
        if acc.settle_unit.is_none() {
            acc.settle_unit.clone_from(&adj.leg.unit);
        }
    }
    Ok(())
}

/// A running total with `q` added. L §5.14.3.
fn add_to(total: Option<Num>, q: &Num) -> Result<Num, Stop> {
    match total {
        None => Ok(q.clone()),
        Some(t) => t.checked_add(q).map_err(crate::eval::num_stop),
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
    /// The walk's global log, then the place's log of each unit and of every unit, over the
    /// span; a refusal stops the reading, what it met before kept here (refcore's
    /// `flows_place`, its logs, which register as they go). L §5.14.3, §8.2.
    fn logs(
        &mut self,
        walk: &Walk,
        place: &str,
        units: &[String],
        (lo, limit): (At, At),
    ) -> Result<(), Stop> {
        self.global(walk, lo, limit)?;
        for unit in units.iter().map(String::as_str).chain([ALL]) {
            self.place_log(walk, place, unit, (lo, limit))?;
        }
        Ok(())
    }

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
                Change::Bad(_) | Change::Reset { .. } | Change::Delta(_) | Change::Mark => {}
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
                Change::Delta(f) => self.count(e, unit, Ok(f))?,
                // a pad's end at its reading's own place (E §6.1); an anchor's or an
                // opening's reset carries no flow, and `flows` doesn't count it
                Change::Reset { .. } => match &e.flow {
                    Some(Amount::Qty(f)) => self.count(e, unit, Ok(f))?,
                    Some(Amount::Hole(h)) => self.count(e, unit, Err(h))?,
                    Some(Amount::Refused(_)) | None => {}
                },
                Change::Mark => {}
            }
        }
        Ok(())
    }

    /// One booked flow of the span counted: its register, and its quantity added to its unit's
    /// total, or its hole kept (refcore's `flows_place_walk`, its `delta` and `reset` kinds).
    /// L §5.14.3, §8.2.
    fn count(&mut self, e: &Ent, unit: &str, flow: Result<&Num, &Hole>) -> Result<(), Stop> {
        self.met.entries.extend(e.reg.iter().cloned());
        if let Some(c) = &e.coll {
            self.met.merge(c);
        }
        match flow {
            Ok(f) => {
                let sum = self.totals.entry(unit.to_owned()).or_insert_with(Num::zero);
                *sum = sum.checked_add(f).map_err(crate::eval::num_stop)?;
            }
            Err(h) => self.holes.push(h.clone()),
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

/// A span's last day: the day before its end, or the run's last day (refcore's `last`). L §5.14.3.
pub(super) fn span_last(m: &Reality, sp: &Span) -> Result<Day, Stop> {
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

/// Whether a leg's movement wanted by a flows read: every leg, or the one it names
/// (L §5.14.3). L §5.14.3.
fn wanted(leg: Option<&str>, name: &str) -> bool {
    match leg {
        None => true,
        Some(l) => l == name,
    }
}

#[cfg(test)]
mod tests {
    use crate::Days;
    use crate::api::{Asks, Given, Pin, Request, run};

    /// A decision over three months whose day is refused in February (a division by zero),
    /// its flows read over January and over February.
    const REFUSED_DAY: &str = r#"model refused_day
  tally 2
  on 2026-02-15
  start 2026-01-01
  end 2026-03-31

source fixture "invented values"

unit PLN: currency minor 0.01

index month = month 2026-01..2026-03

place box: accounts PLN lives 2026-01-01..

place world: outside lives 2026-01-01..

input dec_dom: int

input fee_amt: money PLN

input small_put: money PLN

input divisor[m in month]: num

move split[m in month]: transfers
  on day_of(m, dec_dom * divisor[m] / divisor[m])
  at start_of(m)
  reads box
  leg cash from world to box amount if bal(box) >= small_put then small_put else fee_amt

ask q_fl: flows(split, 2026-01-01 .. 2026-01-31), flows(split, 2026-02-01 .. 2026-02-28) law cut

version facts
  cite fixture
  governs always
  known 2025-12-01
  dec_dom = 20 "the decision's day of the month"
  fee_amt = 3.00 "a fee"
  small_put = 1.00 "a small amount"
  divisor[2026-01] = 1 "answered"
  divisor[2026-02] = 0 "a division by zero, refused"
  divisor[2026-03] = 2 "answered"

version openings
  cite fixture
  governs always
  known 2025-12-31
  opening[box] = 0 at 2026-01-01 start
"#;

    #[test]
    fn flows_of_a_decision_whose_day_is_refused_is_refused_over_any_span() {
        // L §8.4 (tally 2.1.6): split[2026-02]'s day is refused, so it may fall in any span,
        // and every flows of its move is that refusal; refcore prints 0.00 PLN for February
        // (#104, refcore's flows_move counts an unrun instance only for a hole day)
        let given = vec![Given {
            path: "refused_day.tally".to_owned(),
            bytes: Some(REFUSED_DAY.as_bytes().to_vec()),
        }];
        let request = Request {
            asks: Asks::Named(vec!["q_fl".to_owned()]),
            pin: Pin::Canonical,
            days: Days::Events,
        };
        let answer = run(given, &request).expect("an answer");
        assert_eq!(
            answer.lines(),
            [
                "q_fl: (refused: RUN-DIV division by zero, refused: RUN-DIV division by zero)",
                "  unfulfilled: split[2026-01]",
                "  law date 2026-02-15, cut 2026-02-15"
            ],
            "both spans refused"
        );
    }
}
