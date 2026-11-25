//! The walk's flows reads: a place's flows over a span, with what it rests on in the register; a move's are `move_flows.rs`'s.
//! Spec: L §5.14.3 (`flows(x, span)`, `flows(m, span)`: their units, holes and refusals);
//! §5.19 (a pad's flow at its reading's place, on the reset); §8.2 (the register:
//! `unfulfilled:`, `left out:`, `adjustment:`, `open:`); E §4.2 (a refused leg refuses every
//! read of it), E §4.3 (a read of a projection's walk rests on the walks it continues).
//! Twin: engine/tools/refcore/mreads.py (`e_flows`, `flows_place`, `flows_place_walk`).
//! Invariants: a refused `flows` keeps the register it met before the refusal, as refcore
//! registers while it counts; a read of a projection's walk reads the walks it continues
//! through the point each continues from.
use super::instance::{Amount, Dated, Exists, HoleSrc, Inst, Status};
use super::plan::{invariant, qty_value};
use super::point::At;
use super::reads::{Sees, refuse};
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
        // the hole names the target as written, its keys and field too, each key a binding
        // gave by its value (L §8.1; 235.6 on #241)
        if sp.start().is_none_or(|s| s < m.start) {
            let figure = crate::eval::written_bound(m, &f.target, env)?;
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
        // a movement the span doesn't count: what decided it is in the read set, whatever the
        // outcome (L §8.2 One read set; P35 on #241)
        let met = self.walk(id)?.uncounted_in(place, (start, last));
        let mut sums = Sums {
            totals: BTreeMap::new(),
            holes: Vec::new(),
            legs: Vec::new(),
            met,
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
                sums.met.note(*entry);
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
                let Some(day) = i.day.as_ref().and_then(Dated::day) else {
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
        // a row that names a tuple is for the named tuple's period, read from its keys whether
        // or not it exists (L §5.14 l.1363; tally 2.3.1 rulings 22 (1), the critic's P1-C6)
        let named: Vec<(String, String, Vec<KeyVal>)> = self
            .observations(m, ctx.cut)?
            .iter()
            .filter_map(|o| match &o.fulfils {
                Some(crate::model::Fulfils::Instance(mid, keys)) => {
                    Some((o.id.clone(), mid.clone(), keys.clone()))
                }
                Some(crate::model::Fulfils::Adjustment(..)) | None => None,
            })
            .collect();
        for (id, mid, keys) in named {
            let Some(mv) = m.moves.get(&mid) else {
                continue;
            };
            if self.named_for(m, mv, &keys, ctx.cut)?.as_ref() == Some(k) {
                out.insert(id);
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

/// What a place's flows gather: the totals per unit, the holes, the register.
struct Sums {
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
                    self.meet(e);
                    return Err(Stop::Refused(r.clone()));
                }
                Change::Perm(h) | Change::Hole(h) | Change::Open(h) => {
                    self.holes.push(h.clone());
                    if let Some(c) = &e.coll {
                        self.met.merge(c);
                    }
                }
                Change::Bad(_)
                | Change::PermBad(_)
                | Change::Reset { .. }
                | Change::Delta(_)
                | Change::Close(_)
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
        // the holes no reading answers met before the span
        let mut open: Vec<Hole> = Vec::new();
        for e in walk.entries(place, unit) {
            if e.at <= lo {
                match &e.change {
                    Change::Perm(h) if e.at <= limit => open.push(h.clone()),
                    // a refused movement whose day is a hole or refused may fall in the span,
                    // as a hole day's (tally 2.1.8, R11)
                    Change::PermBad(r) if e.at <= limit => {
                        self.meet(e);
                        return Err(Stop::Refused(r.clone()));
                    }
                    // a movement of two days, by its ends below
                    Change::Perm(_)
                    | Change::PermBad(_)
                    | Change::Bad(_)
                    | Change::Reset { .. }
                    | Change::Delta(_)
                    | Change::Hole(_)
                    | Change::Open(_)
                    | Change::Close(_)
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
                // a refused result lists what it read like a value (L §8.2; B1 on #241)
                self.meet(e);
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
                    self.meet(e);
                }
                Change::Delta(f) => self.count(e, unit, Ok(f))?,
                // a pad's end at its reading's own place (E §6.1); an anchor's or an
                // opening's reset carries no flow, and `flows` doesn't count it
                Change::Reset { .. } => match &e.flow {
                    Some(Amount::Qty(f)) => self.count(e, unit, Ok(f))?,
                    Some(Amount::Hole(h)) => self.count(e, unit, Err(h))?,
                    Some(Amount::Refused(_)) | None => {}
                },
                // a mark in the span (an open instance, one that doesn't exist): what a balance
                // lists of it, `flows` lists (S5, P43 on #241; L §8.2 One read set); a
                // movement of two days, by its ends below, its last day booking it as written
                Change::Mark | Change::Open(_) => self.meet(e),
                Change::Close(_) => {}
            }
        }
        self.holes.extend(open);
        self.holes
            .extend(one_end_inside(walk.entries(place, unit), (lo, limit)));
        Ok(())
    }

    /// A plan's instance dated after the run's end counted at the place: each leg dated in
    /// the span, signed by the way it runs there (refcore's `flows_post_e`, its plans). E §2,
    /// L §5.14.3.
    fn unbooked(&mut self, i: &Inst, place: &str, (first, last): (Day, Day)) -> Result<(), Stop> {
        for lg in &i.legs {
            let Some(day) = lg.day.day() else {
                continue;
            };
            if day < first || day > last {
                continue;
            }
            let (out, into) = (lg.from.place() == Some(place), lg.to.place() == Some(place));
            if !out && !into {
                continue;
            }
            self.met.merge(&i.coll);
            if let Some(err) = &i.err {
                return Err(Stop::Refused(err.clone()));
            }
            if let Exists::Unknown(h) = &i.exists {
                self.holes.push(h.clone());
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

    /// An entry the span meets, its register and its reads into the read set, whatever it
    /// gives: a flow, a hole or a refusal (L §8.2 One read set; P35 on #241).
    fn meet(&mut self, e: &Ent) {
        self.met.note_all(&e.reg);
        if let Some(c) = &e.coll {
            self.met.merge(c);
        }
    }

    /// One booked flow of the span counted: its register, and its quantity added to its unit's
    /// total, or its hole kept (refcore's `flows_place_walk`, its `delta` and `reset` kinds).
    /// L §5.14.3, §8.2.
    fn count(&mut self, e: &Ent, unit: &str, flow: Result<&Num, &Hole>) -> Result<(), Stop> {
        self.meet(e);
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

/// The holes of the movements of several days in log `ents`, each a [`Change::Open`] and its
/// [`Change::Close`], whose days the span after `lo` up to `limit` meets without holding
/// them all: one movement whose day is a hole from its first day to its last, it may fall in
/// the span or out of it, a span strictly between its days included; one with all its days in
/// the span counts as its last day books it, one whose days the span misses not at all; one
/// whose last side ended no hole is the hole where its first day is in the span (L §6.1,
/// §5.14 `on d`: `same`'s days are its bounds; tally 2.3.1 R1, N245.21, rulings 17
/// statement B).
fn one_end_inside(ents: &[Ent], (lo, limit): (At, At)) -> Vec<Hole> {
    let inside = |at: At| lo < at && at <= limit;
    let mut out = Vec::new();
    for e in ents {
        let Change::Open(h) = &e.change else {
            continue;
        };
        let closed = ents
            .iter()
            .find(|c| c.at > e.at && matches!(&c.change, Change::Close(x) if x == h));
        let may_fall = match closed {
            Some(c) => e.at <= limit && lo < c.at && !(inside(e.at) && inside(c.at)),
            None => inside(e.at),
        };
        if may_fall {
            out.push(h.clone());
        }
    }
    out
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
