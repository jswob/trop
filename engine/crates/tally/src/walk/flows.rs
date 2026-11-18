//! The walk's flows reads: a place's flows over a span, with what it rests on in the register; a move's are `move_flows.rs`'s.
//! Spec: L §5.14.3 (`flows(x, span)`, `flows(m, span)`: their units, holes and refusals);
//! §5.19 (a pad's flow at its reading's place, on the reset); §8.2 (the register:
//! `unfulfilled:`, `left out:`, `adjustment:`, `open:`); E §4.2 (a refused leg refuses every
//! read of it), E §4.3 (a read of a projection's walk rests on the walks it continues).
//! Twin: engine/tools/refcore/mreads.py (`e_flows`, `flows_place`, `flows_place_walk`).
//! Invariants: a refused `flows` keeps the register it met before the refusal, as refcore
//! registers while it counts; a read of a projection's walk reads the walks it continues
//! through the point each continues from.
use super::instance::{Amount, Dated, DayWindow, Exists, HoleSrc, Inst, Status};
use super::plan::{invariant, qty_value};
use super::point::At;
use super::reads::{Sees, expr_text, refuse};
use super::state::{ALL, Change, Ent, Walk};
use crate::base::Run;
use crate::eval::{Collected, Ctx, Engine, Env};
use crate::hole::{Hole, spread};
use crate::model::{KeyVal, MoveInfo, Reality, Span, Value};
use crate::num::Num;
use crate::syntax::{Expr, Flows, Key, Postfix, Stop};
use crate::time::Day;
use std::collections::{BTreeMap, BTreeSet};

impl Engine {
    /// `flows(x, span)`, of a place or of a move (refcore's `e_flows`). L §5.14.3.
    ///
    /// # Errors
    ///
    /// The refusals of the walk and of the movements counted; `RUN-DOMAIN` for flows in no
    /// one unit; `NotBuilt` for flows of a move's field.
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
            return Ok(Value::Hole(Hole::before_run(&figure)));
        }
        // `, for k`: only the movements for period k (L §5.14.3); a hole k is the line's
        let fk = match &f.for_key {
            Some(e) => {
                let k = self.ev(m, e, env, ctx)?;
                if let Value::Hole(h) = k {
                    return Ok(Value::Hole(h));
                }
                Some(k)
            }
            None => None,
        };
        let Some((mv, posts)) = named else {
            return match self.place_arg(m, &f.target, env, ctx)? {
                Err(h) => Ok(Value::Hole(h)),
                Ok(place) => self.flows_place(m, &place, (&sp, fk.as_ref()), ctx),
            };
        };
        let (keys, leg) = keys_and_leg(posts)?;
        self.flows_move(m, (mv, keys, leg), (&sp, fk.as_ref()), env, ctx)
    }

    /// A place's flows over a span: every movement booked there in it, holes and refusals
    /// included; inside a projection's walk, each walk of its chain gives its entries up to
    /// the point the walk after it starts after (refcore's `flows_place`). L §5.14.3, E §4.2,
    /// E §4.3 step 1 (tally 2.1.2).
    fn flows_place(
        &mut self,
        m: &Reality,
        place: &str,
        (sp, fk): (&Span, Option<&Value>),
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let last = span_last(m, sp)?;
        let mut limit = At::end(last);
        let id = match ctx.at {
            Some(at) => {
                let t = at.point.day;
                let sees = if last >= t {
                    Sees::AtPoint
                } else {
                    Sees::Before
                };
                self.note_read(at.walk, place, sees)?;
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
        let keep = match fk {
            Some(k) => Some(self.movements_for(m, k, &chain, ctx)?),
            None => None,
        };
        let mut sums = Sums {
            span: (start, last),
            totals: BTreeMap::new(),
            holes: Vec::new(),
            legs: Vec::new(),
            met: Collected::default(),
            keep,
        };
        for (wid, cut) in &chain {
            if let Err(stop) = sums.logs(self.walk(*wid)?, place, &units, (lo, *cut)) {
                // refcore registers as it reads the logs: a refusal keeps what came before it
                self.absorb(&sums.met);
                return Err(stop);
            }
        }
        if last > m.end
            && let Err(stop) = self.after_run(m, place, (start, last), ctx, &mut sums)
        {
            self.absorb(&sums.met);
            return Err(stop);
        }
        let walk = self.walk(id)?;
        for (day, entry) in walk.adj_regs.get(place).into_iter().flatten() {
            if *day <= last {
                sums.met.entries.insert(entry.clone());
            }
        }
        self.absorb(&sums.met);
        if let Some(h) = spread(&sums.holes.iter().map(Some).collect::<Vec<_>>()) {
            // a sum of its movements: under `partial`, linear in each hole leg's amount where
            // every hole is one, in one unit (L §5.18; tally 2.1.8 R15)
            let one = match sums.totals.first_key_value() {
                Some((u, q)) if sums.totals.len() == 1 => Some((u.clone(), q.clone())),
                Some(_) => None,
                None => Some((pu.clone(), Num::zero())),
            };
            if let Some((unit, q)) =
                one.filter(|_| ctx.partial && pu != "any" && sums.legs.len() == sums.holes.len())
                && let Some(parts) = self.legs_linear(m, &sums.legs)?
            {
                return crate::eval::lin_fold(m, &h, qty_value(m, &q, &unit)?, parts);
            }
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

    /// A place's flows over the days after the run's end E, which the walk doesn't book
    /// (refcore's `flows_post_e`): a plan's instance there counts its legs at the place, a
    /// decision's is the hole `after the run's end`, its amount reading a walk that doesn't
    /// hold its day. E §2 (movements outside [S, E]), L §8.1 `HOLE-POSTRUN`.
    fn after_run(
        &mut self,
        m: &Reality,
        place: &str,
        days: (Day, Day),
        ctx: Ctx,
        sums: &mut Sums,
    ) -> Result<(), Stop> {
        let mut moves = m.moves_in(self.timeline_world());
        // the plans' instances first, as the walk lists them (refcore's `post_e`)
        moves.sort_by_key(|mv| mv.decision);
        for mv in moves {
            for i in self.move_instances(m, mv, ctx.cut)?.iter() {
                // a window's first day (refcore's `note_post_e`, `day_lo`)
                let Some(day) = i.day.as_ref().and_then(Dated::lo) else {
                    continue;
                };
                if day <= m.end || i.status != Status::LeftOut {
                    continue;
                }
                if sums.keep.as_ref().is_some_and(|keep| !keep.contains(&i.id)) {
                    continue;
                }
                if !mv.decision {
                    sums.unbooked(i, place, days)?;
                } else if day >= days.0 && day <= days.1 && self.may_touch(m, mv, i, place, ctx)? {
                    sums.holes.push(Hole::after_run(&i.id));
                }
            }
        }
        Ok(())
    }

    /// Whether a decision's instance may move a place: a leg names it, or a leg's place can't
    /// be read (refcore's `flows_post_e`, its places). L §5.14 Legs.
    fn may_touch(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        i: &Inst,
        place: &str,
        ctx: Ctx,
    ) -> Result<bool, Stop> {
        for leg in &mv.decl.legs {
            for end in [&leg.from, &leg.to] {
                match self.ev(m, end, &i.env, ctx) {
                    Ok(Value::Key(KeyVal::Thing(name))) if name == place => return Ok(true),
                    Ok(Value::Key(KeyVal::Thing(_))) => {}
                    Ok(_) | Err(Stop::Refused(_)) => return Ok(true),
                    Err(stop) => return Err(stop),
                }
            }
        }
        Ok(false)
    }

    /// The movements for period `k` by id: the plans' instances at the cut and the decisions'
    /// runs in the walks of `chain` whose `for` is k (refcore's `mv_for`). L §5.14.3.
    fn movements_for(
        &mut self,
        m: &Reality,
        k: &Value,
        chain: &[(usize, At)],
        ctx: Ctx,
    ) -> Result<BTreeSet<String>, Stop> {
        let mut out = BTreeSet::new();
        for mv in m.moves_in(self.timeline_world()) {
            if mv.decision {
                continue;
            }
            for i in self.move_instances(m, mv, ctx.cut)?.iter() {
                if i.for_period.as_ref() == Some(k) {
                    out.insert(i.id.clone());
                }
            }
        }
        for (wid, _) in chain {
            for run in &self.walk(*wid)?.runs {
                if run.inst.for_period.as_ref() == Some(k) {
                    out.insert(run.inst.id.clone());
                }
            }
        }
        Ok(out)
    }

    /// The run of decision instance `inst` in walk `id` or in a walk it continues, the
    /// nearest first: a projection's walk counts the runs the walks it continues made
    /// (refcore's `flows_move`, its loop over `ww.base`). L §5.14.1, E §4.3 step 1.
    pub(crate) fn chain_run(
        &self,
        id: usize,
        inst: &str,
    ) -> Result<Option<&super::order::Run>, Stop> {
        for wid in self.chain(id)? {
            let walk = self.walk(wid)?;
            if let Some(run) = walk.run_ids.get(inst).and_then(|ix| walk.runs.get(*ix)) {
                return Ok(Some(run));
            }
        }
        Ok(None)
    }

    /// The pads walk `id` and the walks it continues booked, each once, the nearest first; a
    /// running read's only those before its point `limit` (refcore's `movements_of`, its
    /// pads). L §5.14.1 Running reads, E §6.1.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a walk the engine lacks.
    pub(crate) fn chain_pads(
        &self,
        id: usize,
        limit: Option<At>,
    ) -> Result<Vec<(crate::book::Pad, At)>, Stop> {
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        let mut out = Vec::new();
        for wid in self.chain(id)? {
            for (pad, at) in &self.walk(wid)?.pads {
                if !seen.insert(pad.id.as_str()) || limit.is_some_and(|point| *at >= point) {
                    continue;
                }
                out.push((pad.clone(), *at));
            }
        }
        Ok(out)
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
}

/// `HOLE-IMPRECISE` of flows over a span movement `movement`, dated by window `w`, may fall
/// in or out of (refcore's `flows_place`). L §4.4, §8.1.
fn imprecise_flows(w: &DayWindow, place: &str, movement: &str) -> Hole {
    let text = format!(
        "date within {} not precise enough for flows of {place}",
        w.question
    );
    Hole::asked(&text, movement).carrying(&w.value.carry)
}

/// Whether a leg's amount is zero, so its day can't change a sum (L §4.4).
fn zero_amount(amount: &Amount) -> bool {
    matches!(amount, Amount::Qty(q) if q.is_zero())
}

/// Whether window `w` meets the span `(first, last)` without lying inside it: the movement
/// it dates may fall in the span or out of it (L §4.4).
fn crosses(w: &DayWindow, (first, last): (Day, Day)) -> bool {
    w.lo <= last && w.hi >= first && !(first <= w.lo && w.hi <= last)
}

/// What a place's flows gather: the totals per unit, the holes, the register.
struct Sums {
    /// The span's first and last day, which a movement dated by a window must fall inside.
    span: (Day, Day),
    /// The booked flows per unit.
    totals: BTreeMap<String, Num>,
    /// The holes met.
    holes: Vec<Hole>,
    /// The holes of them that are a booked leg's amount, each with its direction at the place
    /// and where it was read, which a `partial` line reads again (L §5.18; tally 2.1.8 R15).
    legs: Vec<(Hole, bool, HoleSrc)>,
    /// The register met.
    met: Collected,
    /// Under `, for k`, the movements for period k by id; none counts every movement.
    keep: Option<BTreeSet<String>>,
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

    /// What holds for the whole walk up to the span's end: a refusal from inside the span, its
    /// first point included (tally 2.1.7 R2, #107), or from the walk's start, every hole
    /// (refcore's `flows_place`, its global log). L §5.14.3, §8.
    fn global(&mut self, walk: &Walk, lo: At, limit: At) -> Result<(), Stop> {
        let first = At::before(walk.start);
        for e in walk.entries(ALL, ALL) {
            if e.at > limit {
                continue;
            }
            match &e.change {
                Change::Bad(r) | Change::PermBad(r) if e.at >= lo || e.at == first => {
                    return Err(Stop::Refused(r.clone()));
                }
                Change::Perm(h) | Change::Hole(h) => {
                    self.holes.push(h.clone());
                    if let Some(c) = &e.coll {
                        self.met.merge(c);
                    }
                }
                Change::Bad(_)
                | Change::PermBad(_)
                | Change::Unperm(_)
                | Change::Reset { .. }
                | Change::Delta(_)
                | Change::ImpOn(_)
                | Change::Mark => {}
            }
        }
        Ok(())
    }

    /// Whether `, for k` leaves entry `e` out: a movement not for period k, or a reset (a pad's
    /// end), which is for no period (refcore's `other_for`). L §5.14.3.
    fn other_period(&self, e: &Ent) -> bool {
        let Some(keep) = &self.keep else {
            return false;
        };
        matches!(e.change, Change::Reset { .. })
            || e.mv.as_ref().is_some_and(|id| !keep.contains(id))
    }

    /// One unit's log of the place over the span (refcore's `flows_place`, its logs). L §5.14.3.
    fn place_log(
        &mut self,
        walk: &Walk,
        place: &str,
        unit: &str,
        (lo, limit): (At, At),
    ) -> Result<(), Stop> {
        // the movements dated by a window met by the span's end: each whose window crosses
        // the span is its hole, booked in the walk or past its end (L §4.4; refcore's `wins`)
        let mut windows: BTreeMap<&str, (&DayWindow, &str)> = BTreeMap::new();
        // the holes of windows open before the span: one whose window ended there too is
        // decided, its movement dated before the span (L §4.4, P29; B5 of group 3's review)
        let mut open: Vec<Hole> = Vec::new();
        for e in walk.entries(place, unit) {
            // a zero leg's day can't change the sum: decided (L §4.4)
            if let (Change::ImpOn(_), Some((wid, w)), Some(mv)) = (&e.change, &e.window, &e.mv)
                && e.at <= limit
                && !e.flow.as_ref().is_some_and(zero_amount)
            {
                windows.insert(wid, (w, mv));
            }
            if e.at <= lo {
                match &e.change {
                    Change::Perm(h) if e.at <= limit => open.push(h.clone()),
                    Change::Unperm(h) => open.retain(|o| o != h),
                    // a refused movement whose day is a hole or refused may fall in the span,
                    // as a hole day's (tally 2.1.8, R11)
                    Change::PermBad(r) if e.at <= limit => return Err(Stop::Refused(r.clone())),
                    Change::Perm(_)
                    | Change::PermBad(_)
                    | Change::Bad(_)
                    | Change::Reset { .. }
                    | Change::Delta(_)
                    | Change::Hole(_)
                    | Change::ImpOn(_)
                    | Change::Mark => {}
                }
                continue;
            }
            // the log runs in point order: the holes met before the span come first
            self.holes.append(&mut open);
            if e.at > limit {
                continue;
            }
            if let Change::Bad(r) | Change::PermBad(r) = &e.change {
                return Err(Stop::Refused(r.clone()));
            }
            if self.other_period(e) {
                continue;
            }
            if let (Change::Hole(h), Some(leg)) = (&e.change, &e.hole_leg)
                && let Some(src) = &leg.src
            {
                self.legs.push((h.clone(), leg.into, HoleSrc::clone(src)));
            }
            match &e.change {
                Change::Bad(r) | Change::PermBad(r) => return Err(Stop::Refused(r.clone())),
                Change::Hole(h) | Change::Perm(h) => {
                    self.holes.push(h.clone());
                    self.met.entries.extend(e.reg.iter().cloned());
                    if let Some(c) = &e.coll {
                        self.met.merge(c);
                    }
                }
                Change::Delta(f) => match &e.window {
                    // a window that crosses the span is its hole, below (L §4.4)
                    Some((_, w)) if crosses(w, self.span) => {}
                    Some(_) | None => self.count(e, unit, Ok(f))?,
                },
                // a pad's end at its reading's own place (E §6.1); an anchor's or an
                // opening's reset carries no flow, and `flows` doesn't count it
                Change::Reset { .. } => match &e.flow {
                    Some(Amount::Qty(f)) => self.count(e, unit, Ok(f))?,
                    Some(Amount::Hole(h)) => self.count(e, unit, Err(h))?,
                    Some(Amount::Refused(_)) | None => {}
                },
                Change::ImpOn(_) | Change::Unperm(_) | Change::Mark => {}
            }
        }
        self.holes.extend(open);
        for (w, movement) in windows.into_values() {
            if crosses(w, self.span) {
                self.holes.push(imprecise_flows(w, place, movement));
            }
        }
        Ok(())
    }

    /// A plan's instance dated after the run's end counted at the place: each leg dated in
    /// the span, signed by the way it runs there (refcore's `flows_post_e`, its plans). E §2,
    /// L §5.14.3.
    fn unbooked(&mut self, i: &Inst, place: &str, (first, last): (Day, Day)) -> Result<(), Stop> {
        for lg in &i.legs {
            // a leg dated by a window counts where the window meets the span (refcore's
            // `flows_post_e`, `day_lo`, `day_hi`)
            let (Some(lo), Some(hi)) = (lg.day.lo(), lg.day.hi()) else {
                continue;
            };
            if hi < first || lo > last {
                continue;
            }
            let (out, into) = (lg.from.place() == Some(place), lg.to.place() == Some(place));
            if !out && !into {
                continue;
            }
            if let Some(err) = &i.err {
                return Err(Stop::Refused(err.clone()));
            }
            self.met.merge(&i.coll);
            if let Exists::Unknown(h) = &i.exists {
                self.holes.push(h.clone());
                continue;
            }
            // a leg whose window crosses the span may fall in it or out, unless it moves
            // nothing (L §4.4)
            if let Dated::Window(w) = &lg.day
                && crosses(w, (first, last))
                && !zero_amount(&lg.amount)
            {
                self.holes.push(imprecise_flows(w, place, &i.id));
                continue;
            }
            let q = match &lg.amount {
                Amount::Refused(r) => return Err(Stop::Refused(r.clone())),
                Amount::Hole(h) => {
                    self.holes.push(h.clone());
                    continue;
                }
                Amount::Qty(q) => q,
            };
            let unit = lg
                .unit
                .clone()
                .ok_or_else(|| invariant("a counted leg with no unit"))?;
            let sum = self.totals.entry(unit).or_insert_with(Num::zero);
            if out {
                *sum = sum.checked_sub(q).map_err(crate::eval::num_stop)?;
            }
            if into {
                *sum = sum.checked_add(q).map_err(crate::eval::num_stop)?;
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
    totals: &BTreeMap<String, Num>,
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
pub(crate) fn span_last(m: &Reality, sp: &Span) -> Result<Day, Stop> {
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
