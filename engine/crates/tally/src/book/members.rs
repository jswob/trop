//! Movements as values: the members of a binding over an event kind or an observed input, a
//! movement's fields, and its membership of a kind.
//! Spec: L §5.19 Bindings (a binding over an observed input ranges over its rows dated in the
//! span; one over an event kind holds the kind's observations, counting a planned instance and
//! the observations that fulfil it once: by the observations where they are booked, otherwise
//! by the instance), §5.14 (a movement's `day`, its legs as records, its attributes; a
//! movement whose day is a hole is an unknown member), §5.14.3 (an adjustment's `adjusts`,
//! `change` and day), §8.2 (`unfulfilled:`, `uncertain:`), §8.4 (`RUN-RECORD`).
//! Twin: engine/tools/refcore/mreads.py (`movements_of`, `movement_members`, `span_members`,
//! `obs_booked`, `mv_field`, `mv_record`, `mv_by_id`, `mv_kind_in`).
//! Invariants: members come in movement order, by day (a hole day after every day), then by
//! their order compared term by term as its types (E §4.1); a member's register reaches its reader only when the member
//! passes the binding's `where`; an uncertain instance is a member only `with uncertain`, and
//! brings its `uncertain:` line.

use std::collections::BTreeSet;

use super::adjust::Adjustment;
use super::observe::Obs;
use super::reading::Pad;
use super::same::{Fact, Groups, and_list, inst_side, kept_once, obs_side, sides_differ};
use crate::base::{Code, Load, Refusal, Run};
use crate::eval::{Collected, Ctx, Engine, Entry, Env};
use crate::hole::{Hole, spread};
use crate::model::{Fulfils, KeyVal, MoveInfo, MvRef, MvSrc, Reality, Span, Value};
use crate::num::Num;
use crate::syntax::{Bind, Certainty, Stop};
use crate::time::Day;
use crate::walk::{
    Amount, At, Dated, End, Exists, Inst, Law, LegVal, Status, Term, WalkAt, invariant,
    kind_certainty, qty_value,
};

/// What a binding over movements ranges over (refcore's `_dom`, `event` or `obs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MvDom {
    /// The rows of an observed input.
    Obs,
    /// The movements of an event kind.
    Event,
}

/// A movement a binding over movements may hold (refcore's `movements_of`, its items).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Moving {
    /// A plan's instance.
    Plan(Inst),
    /// An observation.
    Obs(Obs),
    /// An adjustment.
    Adj(Adjustment),
    /// A pad the walk booked, at its point.
    Pad(Pad, At),
}

/// A member of a binding over movements: the movement as a value, its unknown-member hole,
/// what it brings to a reader that keeps it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MvMember {
    /// The movement.
    pub value: Value,
    /// The hole of a day that is one.
    pub unknown: Option<Hole>,
    /// Its register and what made it.
    pub coll: Collected,
    /// Its day: a day or a hole (none was dropped before it became a member).
    pub day: Dated,
    /// Its stable id (L §2.5).
    pub id: String,
}

impl Moving {
    /// Its stable id. L §2.5.
    fn id(&self) -> &str {
        match self {
            Moving::Plan(i) => &i.id,
            Moving::Obs(o) => &o.id,
            Moving::Adj(a) => &a.id,
            Moving::Pad(p, _) => &p.id,
        }
    }

    /// Its day: a day, a hole, or none for an instance refused before its day was known.
    /// L §5.14.
    fn day(&self) -> Option<Dated> {
        match self {
            Moving::Plan(i) => i.day.clone(),
            Moving::Obs(o) => Some(Dated::Day(o.day)),
            Moving::Adj(a) => Some(Dated::Day(a.day)),
            Moving::Pad(_, at) => Some(Dated::Day(at.day)),
        }
    }

    /// Its place among a binding's members: its day, a hole day as the last day there is, so
    /// after every day, then its movement order, typed (tally 2.1.2). L §5.19 Bindings, §2.5;
    /// E §4.1.
    fn sort_key(&self) -> Result<(Day, Term), Stop> {
        let day = match self.day() {
            Some(Dated::Day(d)) => d,
            Some(Dated::Hole(_)) | None => Day::LAST,
        };
        let order = match self {
            Moving::Plan(i) => i.order.term()?,
            Moving::Obs(o) => o.order(),
            Moving::Adj(a) => a.order(),
            // a day's pads come after its plans and decisions, whatever the strata
            // (refcore's `PadMv.order`; E §4.1, tally 2.1.2, #39)
            Moving::Pad(p, at) => Term::Tuple(vec![
                Term::Day(at.day),
                Term::Int(PADS_STRATUM),
                Term::text("pads"),
                Term::text(&p.id),
            ]),
        };
        Ok((day, order))
    }

    /// The movement as a value (refcore's `mv()`). L §5.14.
    fn value(&self) -> Value {
        Value::Mv(Box::new(match self {
            Moving::Plan(i) => inst_ref(i),
            Moving::Obs(o) => MvRef {
                id: o.id.clone(),
                day: Some(o.day),
                src: MvSrc::Obs,
            },
            Moving::Adj(a) => MvRef {
                id: a.id.clone(),
                day: Some(a.day),
                src: MvSrc::Adj {
                    inst: a.inst_id.clone(),
                    move_id: a.move_id.clone(),
                    keys: a.inst_keys.clone(),
                    leg: a.leg.name.clone(),
                    vids: a.vids.clone(),
                },
            },
            Moving::Pad(p, at) => MvRef {
                id: p.id.clone(),
                day: Some(at.day),
                src: MvSrc::Pad,
            },
        }))
    }

    /// Its event kind. L §5.7.
    fn kind(&self, m: &Reality) -> Result<String, Stop> {
        match self {
            Moving::Plan(i) => m
                .moves
                .get(&i.move_id)
                .map(|mv| mv.kind.clone())
                .ok_or_else(|| invariant("an instance of a move load didn't build")),
            Moving::Obs(o) => Ok(o.kind.clone()),
            Moving::Adj(_) => Ok("adjustments".to_owned()),
            Moving::Pad(..) => Ok("pads".to_owned()),
        }
    }
}

/// A pad's stratum in its day's movement order: after every plan and decision (refcore's
/// `PADS_STRATUM`, 2⁶²). E §4.1.
const PADS_STRATUM: i64 = 4_611_686_018_427_387_904;

impl Engine {
    /// The members of binding `bind` over movements: those dated on its day or in its span,
    /// or the hole of a day or span that is one where the kind could have a member (refcore's
    /// `movement_members`, `bind_day`). L §5.19 Bindings, §5.7, §8.1.
    ///
    /// # Errors
    ///
    /// The refusals the day, the span or a member meets; `NotBuilt` past what S builds.
    pub(crate) fn movement_members(
        &mut self,
        m: &Reality,
        bind: &Bind,
        dom: (MvDom, &str),
        env: &Env,
        ctx: Ctx,
    ) -> Result<Result<Vec<MvMember>, Hole>, Stop> {
        let span = match (&bind.on, &bind.over) {
            (Some(on), _) => {
                let v = self.ev(m, on, env, ctx)?;
                if let Value::Hole(h) = v {
                    return self.unknown_day(m, (bind, env), dom, h, ctx);
                }
                let d = match crate::eval::one_day(m, &v, "the day of a binding")? {
                    Ok(d) => d,
                    Err(h) => return self.unknown_day(m, (bind, env), dom, h, ctx),
                };
                let next = d
                    .next()
                    .ok_or_else(|| invariant("a binding's day past the calendar"))?;
                Span::between(Some(d), Some(next))
            }
            (None, Some(over)) => match self.span_value(m, over, env, ctx)? {
                Err(h) => return self.unknown_day(m, (bind, env), dom, h, ctx),
                Ok(sp) => sp,
            },
            (None, None) => {
                return Err(Stop::NotBuilt(
                    "a binding over movements with no day or span",
                ));
            }
        };
        self.span_members(m, dom, &span, ctx).map(Ok)
    }

    /// A binding whose day or span is a hole: no member when no movement of the kind the cut
    /// knows could be one (none, or its `where` rules each out), else that hole (refcore's
    /// `bind_day`, `kind_has_member`). L §5.7, §8.1.
    fn unknown_day(
        &mut self,
        m: &Reality,
        (bind, env): (&Bind, &Env),
        dom: (MvDom, &str),
        h: Hole,
        ctx: Ctx,
    ) -> Result<Result<Vec<MvMember>, Hole>, Stop> {
        let mut movements = self.movements_of(m, dom, ctx)?;
        for x in &movements {
            // one left out by its `when` is no candidate: the pins its day took are read
            // (L §8.2; tally 2.1.8 §4, #161)
            if let Moving::Plan(i) = x
                && left_out(m, i)
            {
                let pins = i.coll.deciding();
                self.absorb(&pins);
            }
        }
        movements.retain(|x| !matches!(x, Moving::Plan(i) if left_out(m, i)));
        for x in &movements {
            // each could be a member: S1's refusal of one is the binding's (L §5.19, P42;
            // refcore's `possible_members`)
            if let Some(e) = self.s1_refusal(m, x.id(), ctx.cut)? {
                return Err(Stop::Refused(e));
            }
        }
        let cands = movements.iter().map(Moving::value).collect();
        Ok(if self.may_hold(m, (bind, env), cands, ctx)? {
            Err(h)
        } else {
            Ok(Vec::new())
        })
    }

    /// A planned movement a binding over span `sp` leaves out, by its day or its `when`: what
    /// decided so is the binding's read ([`Engine::uncounted_reads`]; L §8.2 One read set, "a
    /// day outside the span"; P35 on #241; P20).
    ///
    /// # Errors
    ///
    /// As [`Engine::uncounted_reads`].
    fn no_member(&mut self, m: &Reality, x: &Moving, sp: &Span, ctx: Ctx) -> Result<(), Stop> {
        if let Moving::Plan(i) = x {
            let reads = self.uncounted_reads(m, i, sp, ctx)?;
            self.absorb(&reads);
        }
        Ok(())
    }

    /// The movements of the domain dated in span `sp`, an instance and the observations that
    /// fulfil it counted once, each with what it brings to a reader (refcore's
    /// `span_members`). L §5.19 Bindings, §5.14, §8.2.
    pub(crate) fn span_members(
        &mut self,
        m: &Reality,
        dom: (MvDom, &str),
        sp: &Span,
        ctx: Ctx,
    ) -> Result<Vec<MvMember>, Stop> {
        binds_future(m, dom, sp, ctx)?;
        let mut out: Vec<(Moving, Option<Hole>)> = Vec::new();
        let mut fulfilled: BTreeSet<String> = BTreeSet::new();
        for x in self.movements_of(m, dom, ctx)? {
            if let Moving::Plan(i) = &x
                && left_out(m, i)
            {
                self.no_member(m, &x, sp, ctx)?;
                continue;
            }
            // a binding rests on each movement it takes: S1's refusal is its own (L §5.19, P42)
            if let Some(e) = self.s1_refusal(m, x.id(), ctx.cut)?
                && rests_in(&x, sp)
            {
                return Err(Stop::Refused(e));
            }
            // a movement stated same is read from both sides: where their days fall on both
            // sides of the span, whether it is a member is their day's hole (N4, P36 on #241)
            let side = match &x {
                Moving::Plan(i) => Some(inst_side(i)),
                Moving::Obs(o) => Some(obs_side(o)),
                Moving::Adj(_) | Moving::Pad(..) => None,
            };
            if let Some(row) = side
                && let Some(h) = self.same_member_hole(m, row, (x.day().as_ref(), sp, ctx.cut))?
            {
                out.push((x, Some(h)));
                continue;
            }
            let mut st = match x.day() {
                None => {
                    if let Moving::Plan(i) = &x
                        && let Some(e) = &i.err
                    {
                        return Err(Stop::Refused(e.clone()));
                    }
                    continue;
                }
                Some(Dated::Hole(h)) => Some(h),
                Some(Dated::Day(d)) if sp.contains(d) => None,
                Some(Dated::Day(_)) => {
                    self.no_member(m, &x, sp, ctx)?;
                    continue;
                }
            };
            if let Moving::Plan(i) = &x {
                // a decision's run a binding takes prints what it read, explained (E §10)
                self.trace_counted(m, i, ctx)?;
                if let Exists::Unknown(h) = &i.exists {
                    st = spread(&[st.as_ref(), Some(h)]);
                }
                if let Some(e) = &i.err {
                    // a refused result lists what it read like a value (L §8.2 One read set;
                    // 208.2; P21)
                    self.absorb(&i.coll);
                    return Err(Stop::Refused(e.clone()));
                }
            }
            // an adjustment of a hole-day instance is a member only where that day falls before
            // its step's: its existence is the hole (E §5 step 1, tally 2.3 C-4)
            if let Moving::Adj(a) = &x
                && let Some(h) = &a.unknown
            {
                st = spread(&[st.as_ref(), Some(h)]);
            }
            if let Moving::Obs(o) = &x {
                // the links an observation's target reads are no cause the walk's forms
                // print: the member prints as its row where a read takes its leg (E §10; P26)
                let before = self.trace_walked();
                let target = self.obs_target(m, o, ctx.cut);
                self.trace_walked_back(before);
                // the verdict that its link names an instance rests on what decides that the
                // instance exists, its `where`s and `when`, printed where the binding judges it
                // (refcore's `link_tuple`; P31, P42, P43)
                if let Some(Fulfils::Instance(mid, keys)) = &o.fulfils
                    && let Some(mv) = m.moves.get(mid)
                    && ctx.at.is_none()
                {
                    self.trace_existence(m, mv, keys, ctx.cut)?;
                }
                if let Some(t) = target? {
                    fulfilled.insert(t);
                }
            }
            if dom.0 == MvDom::Event
                && let Some(h) = self.same_kind_hole(m, &x, dom.1, ctx)?
            {
                st = spread(&[st.as_ref(), Some(&h)]);
            }
            out.push((x, st));
        }
        let drop = self.counted_once(m, &out, &fulfilled, ctx.cut)?;
        out.retain(|(x, _)| !drop.contains(x.id()));
        let mut res = Vec::with_capacity(out.len());
        for (x, unknown) in out {
            let mut coll = Collected::default();
            match &x {
                Moving::Plan(i) => {
                    if unfulfilled(m, i, ctx.cut) {
                        coll.note(self.entry_id(entry("unfulfilled", &i.id))?);
                    }
                    if i.status == Status::Uncertain {
                        coll.note(self.entry_id(entry("uncertain", &i.id))?);
                    }
                    coll.merge(&i.coll);
                }
                Moving::Obs(o) => coll.merge(&o.coll),
                Moving::Adj(a) => coll.merge(&a.coll),
                // a binding that counts a pad lists it (L §8.2, the `pad:` row; N3 on #241)
                Moving::Pad(p, _) => {
                    coll.merge(&p.coll);
                    coll.merge(&p.reg);
                }
            }
            let day = x
                .day()
                .ok_or_else(|| invariant("a member of a binding with no day"))?;
            res.push(MvMember {
                value: x.value(),
                unknown,
                coll,
                day,
                id: x.id().to_owned(),
            });
        }
        Ok(res)
    }

    /// The members a binding over an event kind leaves out so that a planned instance and the
    /// observations that fulfil it count once: the instance where an observation is booked,
    /// with the observations that aren't; otherwise the observations (refcore's
    /// `span_members`, its drop). L §5.19 Bindings.
    fn counted_once(
        &mut self,
        m: &Reality,
        out: &[(Moving, Option<Hole>)],
        fulfilled: &BTreeSet<String>,
        cut: Day,
    ) -> Result<BTreeSet<String>, Stop> {
        let mut drop = BTreeSet::new();
        let ids: BTreeSet<&str> = out
            .iter()
            .filter_map(|(x, _)| match x {
                Moving::Plan(i) => Some(i.id.as_str()),
                Moving::Obs(_) | Moving::Adj(_) | Moving::Pad(..) => None,
            })
            .collect();
        for iid in fulfilled.iter().filter(|t| ids.contains(t.as_str())) {
            let mut obs: Vec<&Obs> = Vec::new();
            for (x, _) in out {
                if let Moving::Obs(o) = x
                    && self.obs_target(m, o, cut)?.as_deref() == Some(iid.as_str())
                {
                    obs.push(o);
                }
            }
            let mut booked: Vec<&str> = Vec::new();
            for o in &obs {
                if self.obs_booked(m, o, cut)? {
                    booked.push(&o.id);
                }
            }
            if !booked.is_empty() {
                drop.insert(iid.clone());
            }
            for o in obs {
                if !booked.contains(&o.id.as_str()) {
                    drop.insert(o.id.clone());
                }
            }
        }
        Ok(drop)
    }

    /// Whether an observation is booked: dated before the cut, its record not refused, one
    /// of its legs touching a place a version covers that day (refcore's `obs_booked`).
    /// L §5.19 Booking an observation, E §4.2 step 1.
    fn obs_booked(&mut self, m: &Reality, o: &Obs, cut: Day) -> Result<bool, Stop> {
        if o.day >= cut || self.obs_err(m, o, cut)?.is_some() {
            return Ok(false);
        }
        for lg in &o.legs {
            for p in [&lg.from, &lg.to].into_iter().filter_map(End::place) {
                if !self.covering(m, p, o.day, cut).is_empty() {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// Every movement a binding over the domain ranges over at the cut, in movement order:
    /// an observed input's rows; an event kind's plans (not the modelled past, an uncertain
    /// one only `with uncertain`), observations and adjustments; a settlement's candidates
    /// see only the adjustments being settled (refcore's `movements_of`). L §5.19 Bindings,
    /// E §5 step 5.
    fn movements_of(
        &mut self,
        m: &Reality,
        (dom, id): (MvDom, &str),
        ctx: Ctx,
    ) -> Result<Vec<Moving>, Stop> {
        let cut = ctx.cut;
        if dom == MvDom::Obs {
            let mut out = Vec::new();
            for o in self.observations(m, cut)? {
                if o.input == id {
                    out.push(Moving::Obs(o.clone()));
                }
            }
            return Ok(out);
        }
        let kind = id;
        let sub = m.subtree(kind);
        if let Some(adjs) = &self.adj_restrict
            && sub.contains("adjustments")
        {
            return in_order(adjs.iter().cloned().map(Moving::Adj).collect());
        }
        // the moves the read's world holds: a scenario's only in its world and those over it
        // (L §6.4)
        let mut moves = m.moves_in(ctx.world);
        moves.retain(|mv| sub.contains(&mv.kind));
        let mut out = Vec::new();
        let mut decided = Vec::new();
        let mut pins = Collected::default();
        for mv in moves {
            // a scenario's move the binding ranges over is a fact of it (E §9 item 3)
            if m.scenarios.of(&mv.id).is_some() {
                self.note_scenario(&mv.id, &mv.id);
            }
            if mv.decision {
                decided.push(mv);
                continue;
            }
            for i in self.move_instances(m, mv, cut)?.iter() {
                match i.status {
                    // the modelled past: the pins its day took are the binding's reads too
                    // (L §8.2 One read set; tally 2.1.8 §4, #161)
                    Status::Modelled => pins.merge(&i.coll.deciding()),
                    Status::Uncertain if !ctx.unc => {}
                    // an uncertain one unfulfilled is left out whatever its day (tally 2.3.1
                    // R1, N250.1)
                    Status::Hole if !ctx.unc && uncertain_on_hole_day(m, mv, i) => {}
                    // one that doesn't exist the binding read to leave out, which
                    // `span_members` lists (L §8.2 One read set; P20)
                    Status::None
                    | Status::Uncertain
                    | Status::Open
                    | Status::LeftOut
                    | Status::Refused
                    | Status::Hole
                    | Status::Decision
                    | Status::Past
                    | Status::Plan => out.push(Moving::Plan(i.clone())),
                }
            }
        }
        self.absorb(&pins);
        let before = self.trace_walked();
        let planned = out.len();
        for o in self.observations(m, cut)? {
            if sub.contains(&o.kind) {
                out.push(Moving::Obs(o.clone()));
            }
        }
        if out.len() == planned {
            // no observation is a member: the read holds no cause the walk's forms print
            // (E §10)
            self.trace_walked_back(before);
        }
        if sub.contains("adjustments") {
            out.extend(
                self.unsettled_adjustments(m, cut)?
                    .into_iter()
                    .map(Moving::Adj),
            );
            // those of an instance whose day is a hole, which may or may not exist (E §5 step
            // 1, tally 2.3 C-4)
            out.extend(
                self.unsure_adjustments(m, cut)?
                    .into_iter()
                    .map(Moving::Adj),
            );
        }
        if sub.contains("pads") {
            // the walk's own movements, as it booked them (L §5.14.1 Running reads)
            let pads = self.walk_pads(m, ctx)?;
            out.extend(pads.into_iter().map(|(p, at)| Moving::Pad(p, at)));
        }
        out.extend(self.decided_members(m, &decided, ctx)?);
        let groups = self.same_groups(m, ctx.cut)?;
        let sides = self.same_sides(m, ctx.cut)?;
        // one read to be left out stands for no `same` class: it is listed, never counted
        let (left, out): (Vec<Moving>, Vec<Moving>) = out
            .into_iter()
            .partition(|x| matches!(x, Moving::Plan(i) if left_out(m, i)));
        let mut kept = once_by_same((&groups, &sides), out);
        kept.extend(left);
        in_order(kept)
    }

    /// The runs of the decision moves `decided` that a binding over their kind ranges over,
    /// as the walk ran them: inside a decision, its walk's and those it continues', the ones
    /// before its point; else the main walk's of the read's timeline. One that doesn't exist
    /// or is modelled past is no member, nor is an uncertain one without `with uncertain`;
    /// one whose day is a hole, which no walk runs, is a member with that hole (refcore's
    /// `movements_of`, its decisions). L §5.14, §5.14.1 Running reads, §5.19 Bindings.
    fn decided_members(
        &mut self,
        m: &Reality,
        decided: &[&MoveInfo],
        ctx: Ctx,
    ) -> Result<Vec<Moving>, Stop> {
        if decided.is_empty() {
            return Ok(Vec::new());
        }
        let (walk, limit) = match ctx.at {
            Some(WalkAt { walk, point }) => (walk, Some(point)),
            // a traced read prints what the runs read, not the walk (E §10)
            None => (self.decided_walk(m, ctx.cut, decided)?, None),
        };
        let mut out = Vec::new();
        for mv in decided {
            for inst in self.move_instances(m, mv, ctx.cut)?.iter() {
                let Some(run) = self.chain_run(walk, &inst.id)? else {
                    out.extend(hole_day_member(m, (mv, inst), ctx));
                    continue;
                };
                if limit.is_some_and(|point| run.point >= point) {
                    continue;
                }
                if left_out(m, &run.inst) {
                    // read to be left out, which `span_members` lists (L §8.2; P20, P21)
                    out.push(Moving::Plan(run.inst.clone()));
                    continue;
                }
                match run.inst.status {
                    Status::Modelled | Status::None => {}
                    Status::Uncertain if !ctx.unc => {}
                    Status::Uncertain
                    | Status::Open
                    | Status::LeftOut
                    | Status::Refused
                    | Status::Hole
                    | Status::Decision
                    | Status::Past
                    | Status::Plan => out.push(Moving::Plan(run.inst.clone())),
                }
            }
        }
        Ok(out)
    }

    /// The record behind a movement value at the reader's cut: an instance, an observation or
    /// an adjustment; none when the cut sees none (refcore's `mv_record`, `mv_by_id`).
    /// L §5.14, §3.1.
    fn mv_record(&mut self, m: &Reality, v: &MvRef, ctx: Ctx) -> Result<Option<Moving>, Stop> {
        let cut = ctx.cut;
        match &v.src {
            MvSrc::Inst(move_id, _) => self.instance_by_id(m, move_id, &v.id, ctx),
            MvSrc::Obs => Ok(self.observation_by_id(m, &v.id, cut)?),
            MvSrc::Adj { .. } => {
                if let Some(r) = &self.adj_restrict
                    && let Some(a) = r.iter().find(|a| a.id == v.id)
                {
                    return Ok(Some(Moving::Adj(a.clone())));
                }
                let all = self.unsettled_adjustments(m, cut)?;
                if let Some(a) = all.into_iter().find(|a| a.id == v.id) {
                    return Ok(Some(Moving::Adj(a)));
                }
                // one of an instance whose day is a hole, a binding's member (tally 2.3 C-4)
                let unsure = self.unsure_adjustments(m, cut)?;
                Ok(unsure.into_iter().find(|a| a.id == v.id).map(Moving::Adj))
            }
            MvSrc::Kind(_) => Ok(None),
            MvSrc::Missing => {
                let name = v.id.split_once('[').map_or(v.id.as_str(), |(n, _)| n);
                if m.moves.get(name).is_some() {
                    return self.instance_by_id(m, name, &v.id, ctx);
                }
                self.observation_by_id(m, &v.id, cut)
            }
            MvSrc::Pad => {
                let pads = self.walk_pads(m, ctx)?;
                Ok(pads
                    .into_iter()
                    .find(|(p, _)| p.id == v.id)
                    .map(|(p, at)| Moving::Pad(p, at)))
            }
        }
    }

    /// The pads a binding over pads ranges over: inside a decision, those its walk and the
    /// walks it continues booked before its point; else the main walk's of the read's
    /// timeline (refcore's `movements_of`, its pads). L §5.14.1 Running reads, E §6.1.
    fn walk_pads(&mut self, m: &Reality, ctx: Ctx) -> Result<Vec<(Pad, At)>, Stop> {
        let (walk, limit) = match ctx.at {
            Some(WalkAt { walk, point }) => (walk, Some(point)),
            None => (self.main_walk(m, ctx.cut)?, None),
        };
        self.chain_pads(walk, limit)
    }

    /// A move's instance by id at the read's cut; a decision's as the walk ran it: inside a
    /// decision, in its walk or one it continues, `RUN-ORDER` where it hasn't run yet; else in
    /// the main walk of the read's timeline, or the instance as planned where that walk didn't
    /// run it (refcore's `mv_by_id`, a move, and `mv_record`). L §5.14, §5.14.1, §5.14.2.
    fn instance_by_id(
        &mut self,
        m: &Reality,
        move_id: &str,
        id: &str,
        ctx: Ctx,
    ) -> Result<Option<Moving>, Stop> {
        let mv = m
            .moves
            .get(move_id)
            .ok_or_else(|| invariant("a movement of a move load didn't build"))?;
        let insts = self.move_instances(m, mv, ctx.cut)?;
        let Some(inst) = insts.iter().find(|i| i.id == id).cloned() else {
            return Ok(None);
        };
        if !mv.decision {
            return Ok(Some(Moving::Plan(inst)));
        }
        let (walk, running) = match ctx.at {
            Some(WalkAt { walk, .. }) => (walk, true),
            // a traced read prints what the run read where it was counted, not the walk
            // (E §10)
            None => (self.decided_walk(m, ctx.cut, &[mv])?, false),
        };
        match self.chain_run(walk, id)? {
            Some(run) => Ok(Some(Moving::Plan(run.inst.clone()))),
            // one whose day is a hole runs on no day of the walk: it is never waited for, and
            // a field no walk read is its hole (L §5.14; the review of G1's fix, BF1, P10)
            None if running && unread_hole(m, &inst).is_none() => Err(Stop::Refused(Refusal::new(
                Code::Run(Run::Order),
                format!("decision {id} is read before it has run"),
            ))),
            None => Ok(Some(Moving::Plan(inst))),
        }
    }

    /// An observation by id at the cut (refcore's `mv_by_id`, an observation). L §5.19.
    fn observation_by_id(
        &mut self,
        m: &Reality,
        id: &str,
        cut: Day,
    ) -> Result<Option<Moving>, Stop> {
        let rows = self.observations(m, cut)?;
        Ok(rows.iter().find(|o| o.id == id).cloned().map(Moving::Obs))
    }

    /// `x.f` of a movement: its `day`, its `fulfils`, a leg as a record (`from`, `to`,
    /// `amount`, `day`), an attribute; an adjustment's `adjusts` and `change` (refcore's
    /// `mv_field`). L §5.14, §5.14.3, §5.19, §8.4.
    ///
    /// A field read of a move's instance that doesn't exist, `.day` included, is refused
    /// (L §5.14 An instance's existence is a cell; tally 2.3.1 R6, C-2).
    ///
    /// # Errors
    ///
    /// `RUN-NONE` for a move's instance that doesn't exist; `RUN-RECORD` for another movement
    /// the cut doesn't see; the movement's refusal; a refused leg; `LOAD-FIELD` for a field it
    /// lacks.
    pub(crate) fn mv_field(
        &mut self,
        m: &Reality,
        v: &MvRef,
        name: &str,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let Some(src) = self.mv_record(m, v, ctx)? else {
            let instance = match &v.src {
                MvSrc::Inst(..) => true,
                MvSrc::Missing => {
                    let id = v.id.split_once('[').map_or(v.id.as_str(), |(n, _)| n);
                    m.moves.get(id).is_some()
                }
                MvSrc::Obs | MvSrc::Adj { .. } | MvSrc::Kind(_) | MvSrc::Pad => false,
            };
            if instance {
                return Err(not_there(&v.id));
            }
            return Err(Stop::Refused(Refusal::new(
                Code::Run(Run::Record),
                format!("{} doesn't exist", v.id),
            )));
        };
        let kind = src.kind(m)?;
        let is_leg = m.kind_legs(&kind).iter().any(|(n, _, _)| n == name);
        // a leg's read of it is `leg_field`'s, which a later version may make exist
        if !is_leg
            && let Moving::Plan(i) = &src
            && (i.exists == Exists::No || i.status == Status::None)
            && !self.made_to_exist(m, i, ctx)?
        {
            return Err(not_there(src.id()));
        }
        match (&src, name) {
            // an adjustment's identity, its `day` and `adjusts`, carries none of its holes and
            // never its refusal, so a binding's `where` reading `a.adjusts` or `a.day` decides;
            // a refused adjustment refuses its change and every other field (E §5 step 5,
            // tally 2.1.2)
            (Moving::Adj(_), "day" | "adjusts") => {}
            (Moving::Adj(a), _) => {
                self.absorb(&a.coll);
                if let Amount::Refused(e) = &a.delta {
                    return Err(Stop::Refused(e.clone()));
                }
            }
            (Moving::Plan(i), "day") => self.absorb(&i.day_coll),
            (Moving::Plan(i), _) => self.absorb(&i.coll),
            (Moving::Obs(o), _) => self.absorb(&o.coll),
            // a binding that counts a pad lists it (L §8.2, the `pad:` row; 237.1 on #241)
            (Moving::Pad(p, _), _) => {
                self.absorb(&p.coll);
                self.absorb(&p.reg);
            }
        }
        match &src {
            Moving::Plan(i) => {
                if let Some(e) = &i.err {
                    return Err(Stop::Refused(e.clone()));
                }
            }
            Moving::Obs(o) => {
                if let Some(e) = self.obs_err(m, o, ctx.cut)? {
                    return Err(Stop::Refused(e));
                }
            }
            Moving::Adj(_) | Moving::Pad(..) => {}
        }
        // a field the plan pass didn't read, of an instance whose day is a hole (a decision,
        // which never runs without a day; one under a hole `at`), is that hole, read by its
        // name or through a binding alike (L §5.14; E §4.2 step 3, "a read of its legs is that
        // hole"; tally 2.3.1 R1, N250.8, C-13.9)
        if name != "day"
            && let Moving::Plan(i) = &src
            && let Some(h) = unread_hole(m, i)
        {
            return Ok(Value::Hole(h));
        }
        // a leg a later version changes, of an instance whose day is a hole, is read as that
        // hole where the version changes it, since its booking cut is a hole with its day: its
        // day, its amount where a version changes it, its places where one moves them, the
        // whole leg where one does both (L §5.14; pick P12, by C-4's measure, P10; tally 2.3.1
        // R1, N250.21)
        if is_leg
            && let Moving::Plan(i) = &src
            && let Some((h, changes)) = self.changed_leg(m, i, name, ctx.cut)?
        {
            return Ok(match i.legs.iter().find(|lg| lg.name == name) {
                Some(lg) if !changes.moved() => same_leg_record(lg, &h),
                Some(lg) if !changes.amount => moved_leg_record(m, lg, &h, changes)?,
                Some(_) | None => Value::Hole(h),
            });
        }
        if (name == "day" || is_leg)
            && let Some(h) =
                self.same_rows_hole(m, &src, (name != "day").then_some(name), ctx.cut)?
        {
            // per fact: a leg whose amount the sides disagree on is a record whose amount and
            // day are that hole, its `from` and `to` their own (P42, P43 on #241)
            let legs: &[LegVal] = match &src {
                Moving::Plan(i) => &i.legs,
                Moving::Obs(o) => &o.legs,
                Moving::Adj(_) | Moving::Pad(..) => &[],
            };
            return Ok(match legs.iter().find(|lg| lg.name == name) {
                Some(lg) if is_leg => same_leg_record(lg, &h),
                Some(_) | None => Value::Hole(h),
            });
        }
        match name {
            "day" => Ok(match src.day() {
                Some(day) => day.value(),
                None => Value::None,
            }),
            "fulfils" => self.fulfils_of(m, &src, ctx.cut),
            _ if is_leg => self.leg_field(m, &src, name, ctx),
            _ => attr_field(m, &src, name),
        }
    }

    /// The hole of leg `leg` of instance `inst`, whose day is a hole, where a later version
    /// known by cut `cut` changes it, and what the versions change: C-4's measure made an
    /// adjustment of it whose existence is that hole (`Engine::unsure_of`), which moves the
    /// leg's places or whose δ is no known 0; none for an instance with a day, or a leg no
    /// version changes (L §5.14, E §5 steps 1 to 3; pick P12; tally 2.3.1 N250.21).
    ///
    /// # Errors
    ///
    /// The refusals of the measuring.
    fn changed_leg(
        &mut self,
        m: &Reality,
        inst: &Inst,
        leg: &str,
        cut: Day,
    ) -> Result<Option<(Hole, Changes)>, Stop> {
        let Some(Dated::Hole(h)) = &inst.day else {
            return Ok(None);
        };
        let mv = m
            .moves
            .get(&inst.move_id)
            .ok_or_else(|| invariant("a movement of a move load didn't build"))?;
        let mut changed = false;
        let mut changes = Changes::default();
        // the measure is no cause the walk's forms print: the read gives the day's hole, which
        // prints as a hole (E §10; P26)
        let before = self.trace_walked();
        let unsure = self.unsure_of(m, mv, cut);
        self.trace_walked_back(before);
        for a in unsure? {
            if a.inst_id == inst.id && a.leg.name == leg {
                changed = true;
                // the ends as [from, to], before the step and after it; ends given with none
                // before them move both (E §5 step 2)
                match (a.before_ends.as_slice(), a.new_ends.as_slice()) {
                    (_, []) => {}
                    ([from0, to0], [from1, to1]) => {
                        changes.from = changes.from || from0 != from1;
                        changes.to = changes.to || to0 != to1;
                    }
                    _ => {
                        changes.from = true;
                        changes.to = true;
                    }
                }
                let same_amount = matches!(&a.delta, Amount::Qty(d) if d.is_zero());
                changes.amount = changes.amount || !same_amount;
            }
        }
        Ok(changed.then(|| (h.clone(), changes)))
    }

    /// A movement of a `same` group whose members state kinds not all inside the binding's
    /// kind `kind`: an unknown member of it, the `HOLE-SAME` naming every member with its
    /// kind, in id order; none where every kind the group states is inside `kind` (refcore's
    /// `same_kind_hole`). L §6.1 (one movement has one base kind), §8.1 (tally 2.1.8 S6).
    fn same_kind_hole(
        &mut self,
        m: &Reality,
        x: &Moving,
        kind: &str,
        ctx: Ctx,
    ) -> Result<Option<Hole>, Stop> {
        let groups = self.same_groups(m, ctx.cut)?;
        let Some(group) = groups.get(x.id()) else {
            return Ok(None);
        };
        let mut members: Vec<(Moving, String)> = Vec::new();
        for y in group {
            let found = match self.observation_by_id(m, y, ctx.cut)? {
                Some(o) => Some(o),
                None => self.planned_by_id(m, y, ctx.cut)?,
            };
            if let Some(member) = found {
                let k = member.kind(m)?;
                members.push((member, k));
            }
        }
        let mut kinds: Vec<&str> = Vec::new();
        for (_, k) in &members {
            if !kinds.contains(&k.as_str()) {
                kinds.push(k);
            }
        }
        let sub = m.subtree(kind);
        if kinds.len() < 2 || kinds.iter().all(|k| sub.contains(*k)) {
            return Ok(None);
        }
        let day = std::iter::once(x)
            .chain(members.iter().map(|(mv, _)| mv))
            .find_map(|mv| mv.day().as_ref().and_then(Dated::day))
            .map_or_else(|| "every day".to_owned(), Day::text);
        // every member, in the order of their ids (tally 2.1.8 S6)
        let ids: Vec<String> = members.iter().map(|(mv, _)| mv.id().to_owned()).collect();
        let parts: Vec<String> = members
            .iter()
            .map(|(mv, k)| format!("{k} ({})", mv.id()))
            .collect();
        let text = format!(
            "{} are one, but the kind differs on {day}: {}",
            and_list(&ids),
            and_list(&parts)
        );
        Ok(Some(Hole::asked(&text, &ids.join(", "))))
    }

    /// A plan's instance by id at the cut, a move named by the id's head (refcore's
    /// `plan_insts`, by id). L §5.14.
    fn planned_by_id(&mut self, m: &Reality, id: &str, cut: Day) -> Result<Option<Moving>, Stop> {
        let name = id.split_once('[').map_or(id, |(n, _)| n);
        let Some(mv) = m.moves.get(name).filter(|mv| !mv.decision) else {
            return Ok(None);
        };
        let insts = self.move_instances(m, mv, cut)?;
        Ok(insts.iter().find(|i| i.id == id).cloned().map(Moving::Plan))
    }

    /// The `HOLE-SAME` of movement `x`, an observed row or a planned instance, where the other
    /// members of its `same` group disagree with it on a leg's amount or day (on leg `leg`
    /// alone, where given): a read of the movement away from its places, "by every read of
    /// either" (refcore's `same_rows_hole`; tally 2.2.1's 219.1 for a planned side). L §5.19
    /// One movement on two statements, §6.1, §8.1.
    fn same_rows_hole(
        &mut self,
        m: &Reality,
        x: &Moving,
        leg: Option<&str>,
        cut: Day,
    ) -> Result<Option<Hole>, Stop> {
        let row = match x {
            Moving::Obs(o) => obs_side(o),
            Moving::Plan(i) => inst_side(i),
            Moving::Adj(_) | Moving::Pad(..) => return Ok(None),
        };
        // per fact: a day read takes the day alone, a leg's its amount (Q3, P36 on #241)
        let fact = match leg {
            Some(name) => Fact::Leg(name.to_owned()),
            None => Fact::Day,
        };
        self.same_hole(m, row, &fact, cut)
    }

    /// The `HOLE-SAME` of side `row` (an id, a day where it is one, legs) on fact `fact`, where
    /// the other members of its `same` group disagree with it, each side's reads of the fact
    /// taken; a side's hole day's hole (refcore's `same_rows_hole`; tally 2.2.1's 219.1, B2,
    /// N4, Q1, Q3). L §6.1, §8.1, §8.2.
    ///
    /// # Errors
    ///
    /// A number error printing an amount; a refusal reading the group or a member.
    pub(crate) fn same_hole(
        &mut self,
        m: &Reality,
        row: (&str, Option<Day>, &[LegVal]),
        fact: &Fact,
        cut: Day,
    ) -> Result<Option<Hole>, Stop> {
        let groups = self.same_groups(m, cut)?;
        let Some(group) = groups.get(row.0) else {
            return Ok(None);
        };
        // each other member, an observed row, else a planned instance at the cut
        let mut members: Vec<Moving> = Vec::new();
        for y in group.iter().filter(|y| y.as_str() != row.0) {
            let all = self.observations(m, cut)?;
            if let Some(o) = all.iter().find(|o| o.id == *y) {
                members.push(Moving::Obs(o.clone()));
            } else if let Some(p) = self.planned_by_id(m, y, cut)? {
                members.push(p);
            }
        }
        let mut others: Vec<(&str, Option<Day>, &[LegVal])> = Vec::new();
        // a fact of movements stated same is read from both sides together: each side's
        // reads are the read's, its day's for a day (L §6.1, §8.2; B2 on #241)
        for member in &members {
            match member {
                Moving::Obs(o) => {
                    self.absorb(&o.coll);
                    others.push(obs_side(o));
                }
                Moving::Plan(i) => {
                    self.absorb(&reads_of_fact(i, fact));
                    others.push(inst_side(i));
                }
                Moving::Adj(_) | Moving::Pad(..) => {}
            }
        }
        if others.is_empty() {
            return Ok(None);
        }
        sides_differ(m, row, &others, fact)
    }

    /// Whether span `sp` holds side `row` of a movement stated `same`, `day` its own day: none
    /// where every side's day is in the span or none is; else their day's hole, `HOLE-SAME` or
    /// a side's hole day's, never one side's day (L §6.1, §5.19 Bindings; N4, P36 on #241).
    ///
    /// # Errors
    ///
    /// A refusal reading the group or a member.
    pub(crate) fn same_member_hole(
        &mut self,
        m: &Reality,
        row: (&str, Option<Day>, &[LegVal]),
        (day, sp, cut): (Option<&Dated>, &Span, Day),
    ) -> Result<Option<Hole>, Stop> {
        let groups = self.same_groups(m, cut)?;
        let Some(group) = groups.get(row.0) else {
            return Ok(None);
        };
        let mut inside = vec![day_in(day, sp)];
        // the read holds every side's day reads, whatever the membership gives, so their pins
        // are listed (S2, F7, P43 on #241)
        for y in group.iter().filter(|y| y.as_str() != row.0) {
            let all = self.observations(m, cut)?;
            let row_of = all
                .iter()
                .find(|o| o.id == *y)
                .map(|o| (o.coll.clone(), o.day));
            if let Some((coll, row_day)) = row_of {
                self.absorb(&coll);
                inside.push(day_in(Some(&Dated::Day(row_day)), sp));
            } else if let Some(Moving::Plan(p)) = self.planned_by_id(m, y, cut)? {
                self.absorb(&p.day_coll);
                inside.push(day_in(p.day.as_ref(), sp));
            }
        }
        if inside.iter().all(|b| *b == Some(true)) || inside.iter().all(|b| *b == Some(false)) {
            return Ok(None);
        }
        self.same_hole(m, row, &Fact::Day, cut)
    }

    /// What a movement's `fulfils` names at the cut: the instance or the adjustment, or none
    /// (refcore's `mv_field`, `fulfils`). L §5.19, §5.14.3.
    fn fulfils_of(&mut self, m: &Reality, src: &Moving, cut: Day) -> Result<Value, Stop> {
        let fulfils = match src {
            Moving::Plan(i) => return Ok(i.fulfils.clone().unwrap_or(Value::None)),
            Moving::Adj(_) | Moving::Pad(..) => return Ok(Value::None),
            Moving::Obs(o) => o.fulfils.clone(),
        };
        match fulfils {
            None => Ok(Value::None),
            Some(Fulfils::Adjustment(mid, keys, leg, vid)) => {
                let iid = super::observe::inst_id(&mid, &keys);
                let all = self.unsettled_adjustments(m, cut)?;
                Ok(all
                    .into_iter()
                    .find(|a| a.inst_id == iid && a.leg.name == leg && a.vids.contains(&vid))
                    .map_or(Value::None, |a| Moving::Adj(a).value()))
            }
            Some(Fulfils::Instance(mid, keys)) => {
                let mv = m
                    .moves
                    .get(&mid)
                    .ok_or_else(|| invariant("a link to a move load didn't build"))?;
                // the instance by its existence alone, never its amounts: a field read of it
                // takes it again at the reader's cut (L §5.19 Links; P22)
                // the plan's list it reads is no cause the walk's forms print: the link prints
                // as the observation's row (E §10; P26)
                let before = self.trace_walked();
                let named = self.linked_instance(m, mv, &keys, cut);
                self.trace_walked_back(before);
                let named = named?;
                Ok(named.map_or(Value::None, |r| Value::Mv(Box::new(r))))
            }
        }
    }

    /// A movement's leg as a record, `from`, `to`, `amount`, `day`; a leg the walk refused is
    /// refused (refcore's `mv_field`, a leg). L §5.14, E §4.2 step 5.
    fn leg_field(
        &mut self,
        m: &Reality,
        src: &Moving,
        name: &str,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let legs: &[LegVal] = match src {
            Moving::Plan(i) => &i.legs,
            Moving::Obs(o) => &o.legs,
            Moving::Adj(_) => &[],
            Moving::Pad(p, _) => core::slice::from_ref(&p.leg),
        };
        if let Some(lg) = legs.iter().find(|lg| lg.name == name) {
            let day = lg
                .day
                .day()
                .or_else(|| src.day().as_ref().and_then(Dated::day));
            if let Amount::Refused(r) = &lg.amount {
                self.booked_walk(m, src, (name, day), ctx)?;
                return Err(Stop::Refused(r.clone()));
            }
            // the walk is a cause an explanation prints in its own form only where it refused
            // the leg; else the leg is its instance's, traced as the instance was (E §10)
            let back = self.trace_walked();
            if let Some(err) = self.leg_refused(m, (src.id(), name), day, ctx)? {
                return Err(Stop::Refused(err));
            }
            self.trace_walked_back(back);
            let record = leg_record(m, lg)?;
            // an observation a read counts is a cause, printed as its row (E §10)
            if let Moving::Obs(o) = src {
                self.trace_observation(m, (&o.input, &o.id), &record)?;
            }
            return Ok(record);
        }
        if let Moving::Plan(i) = src
            && (i.exists == Exists::No || i.status == Status::None)
        {
            if let Some(made) = self.made_to_exist_leg(m, i, name, ctx)? {
                return Ok(made);
            }
            return Err(not_there(src.id()));
        }
        if let Some(h) = body_hole(src) {
            return Ok(Value::Hole(h));
        }
        Err(Stop::Refused(Refusal::new(
            Code::Load(Load::Field),
            format!("{name} is not a field of {}", src.id()),
        )))
    }

    /// The walk a leg read of a booked instance meets before its amount, whose refusal is its
    /// answer: the walk the hole twin's read builds, so the run warns as its twin's does; a
    /// refusal met building it is left to the walk, the amount's refusal standing (refcore's
    /// `mv_field`, its `ask_walk`). L §8 (head), §8.5; E §4.2 step 5; tally 2.1.8 R11.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT`, or a part not built yet, met building the walk.
    fn booked_walk(
        &mut self,
        m: &Reality,
        src: &Moving,
        (name, day): (&str, Option<Day>),
        ctx: Ctx,
    ) -> Result<(), Stop> {
        let Moving::Plan(i) = src else {
            return Ok(());
        };
        if !matches!(i.status, Status::Past | Status::Plan) {
            return Ok(());
        }
        let back = self.trace_walked();
        let walked = self.leg_refused(m, (src.id(), name), day, ctx);
        self.trace_walked_back(back);
        match walked {
            Ok(_) | Err(Stop::Refused(_)) => Ok(()),
            Err(stop) => Err(stop),
        }
    }

    /// Whether a later version made plan instance `i`, which didn't exist at its booking cut,
    /// exist: one of the cut's adjustments of it, of any leg, whose step made it exist
    /// (refcore's `made_to_exist_leg`). E §5 steps 1 and 3, L §5.14.3.
    ///
    /// # Errors
    ///
    /// As [`Engine::adjust_move`].
    fn made_to_exist(&mut self, m: &Reality, i: &Inst, ctx: Ctx) -> Result<bool, Stop> {
        let mv = m
            .moves
            .get(&i.move_id)
            .ok_or_else(|| invariant("an instance of a move load didn't build"))?;
        if mv.decision {
            return Ok(false);
        }
        let adjustments = self.adjust_move(m, mv, ctx.cut)?;
        Ok(adjustments.iter().any(|a| a.inst_id == i.id && a.made))
    }

    /// Leg `name` of plan instance `i`, which didn't exist at its booking cut, as a later
    /// version made it exist: the leg of the last of the cut's adjustments of it whose step
    /// made it exist, with that version's places, booked at amount 0; none where no step made
    /// it exist (refcore's `made_to_exist_leg`). E §5 steps 1, 3 and 6, L §5.14.3.
    ///
    /// # Errors
    ///
    /// As [`Engine::adjust_move`].
    fn made_to_exist_leg(
        &mut self,
        m: &Reality,
        i: &Inst,
        name: &str,
        ctx: Ctx,
    ) -> Result<Option<Value>, Stop> {
        let mv = m
            .moves
            .get(&i.move_id)
            .ok_or_else(|| invariant("an instance of a move load didn't build"))?;
        if mv.decision {
            return Ok(None);
        }
        let adjustments = self.adjust_move(m, mv, ctx.cut)?;
        let Some(a0) = adjustments
            .iter()
            .rev()
            .find(|a| a.inst_id == i.id && a.leg.name == name && a.made)
        else {
            return Ok(None);
        };
        let (coll, mut leg) = (a0.coll.clone(), a0.leg.clone());
        self.absorb(&coll);
        // booked at A₀ = 0 (E §5 step 1); a later read sees A₀ plus the settlements (step 6)
        leg.amount = Amount::Qty(Num::zero());
        leg_record(m, &leg).map(Some)
    }

    /// Whether a movement value is of kind `kind`: its record's kind is in the kind's subtree;
    /// one the cut doesn't see is in no kind (refcore's `mv_kind_in`). L §5.7.
    ///
    /// # Errors
    ///
    /// The refusals finding its record meets.
    pub(crate) fn mv_kind_in(
        &mut self,
        m: &Reality,
        v: &MvRef,
        kind: &str,
        ctx: Ctx,
    ) -> Result<bool, Stop> {
        let Some(src) = self.mv_record(m, v, ctx)? else {
            return Ok(false);
        };
        Ok(m.subtree(kind).contains(&src.kind(m)?))
    }
}

/// An attribute of a movement: an instance's or an observation's, an adjustment's `adjusts`
/// and `change`; one it lacks is `RUN-DOMAIN` (refcore's `mv_field`, an attribute). L §5.14,
/// §5.14.3, §8.4.
fn attr_field(m: &Reality, src: &Moving, name: &str) -> Result<Value, Stop> {
    let found = match src {
        Moving::Plan(i) => i
            .attrs
            .iter()
            .find(|(a, _)| a == name)
            .map(|(_, v)| v.clone()),
        Moving::Obs(o) => o
            .values
            .iter()
            .find(|(a, _)| a == name)
            .map(|(_, v)| v.clone()),
        Moving::Adj(a) => match name {
            "adjusts" => Some(Value::Mv(Box::new(MvRef {
                id: a.inst_id.clone(),
                day: a.inst_day,
                src: MvSrc::Inst(a.move_id.clone(), a.inst_keys.clone()),
            }))),
            "change" => Some(amount_value(m, &a.delta, a.leg.unit.as_deref())?),
            _ => None,
        },
        // a pad has no attributes (refcore's `PadMv.attrs`)
        Moving::Pad(..) => None,
    };
    if let Some(v) = found {
        return Ok(v);
    }
    if let Some(h) = body_hole(src) {
        return Ok(Value::Hole(h));
    }
    // which movement lacks the attribute only the run knows: the value is refused, in load's
    // words, and the reality isn't (L §8.4 Load refuses what load sees; tally 2.1.8 §4, #162)
    Err(Stop::Refused(Refusal::new(
        Code::Run(Run::Domain),
        format!("{name} is not a field of {}", src.id()),
    )))
}

/// The hole of a movement whose body wasn't evaluated: its existence, law date or day a hole
/// (refcore's `_body_hole`). L §5.14.
fn body_hole(src: &Moving) -> Option<Hole> {
    let Moving::Plan(i) = src else {
        return None;
    };
    if let Exists::Unknown(h) = &i.exists {
        return Some(h.clone());
    }
    match &i.day {
        Some(Dated::Hole(h)) => Some(h.clone()),
        Some(Dated::Day(_)) | None => None,
    }
}

/// A leg as a record: `from`, `to`, `amount`, `day` (refcore's `Leg.rec`). L §5.14.
fn leg_record(m: &Reality, lg: &LegVal) -> Result<Value, Stop> {
    let end = |e: &End| match e {
        End::Place(p) => Value::Key(KeyVal::Thing(p.clone())),
        End::Hole(h) => Value::Hole(h.clone()),
    };
    let day = lg.day.value();
    Ok(Value::Record(vec![
        ("from".to_owned(), end(&lg.from)),
        ("to".to_owned(), end(&lg.to)),
        (
            "amount".to_owned(),
            amount_value(m, &lg.amount, lg.unit.as_deref())?,
        ),
        ("day".to_owned(), day),
    ]))
}

impl Engine {
    /// Adds, under the open node of a traced read, the observed rows among `by`, the
    /// movements settling the adjustments the read counts, each once, as an observation the
    /// read counts prints: its row, holding its first leg, or for a waiver, a movement with
    /// no leg, its day (E §10; L §5.14.3, §5.14; P37).
    ///
    /// # Errors
    ///
    /// As [`Engine::observations`]; `ENGINE-INVARIANT` for a row a load didn't build.
    pub(crate) fn trace_settling(
        &mut self,
        m: &Reality,
        by: &[String],
        cut: Day,
    ) -> Result<(), Stop> {
        if by.is_empty() || !self.tracing() {
            return Ok(());
        }
        let mut rows = Vec::new();
        for o in self.observations(m, cut)? {
            if by.contains(&o.id) && !rows.iter().any(|(_, id, _)| *id == o.id) {
                let first = o.legs.first().cloned();
                rows.push((o.input.clone(), o.id.clone(), (first, o.day)));
            }
        }
        for (input, id, (first, day)) in rows {
            let record = match &first {
                Some(lg) => leg_record(m, lg)?,
                None => Value::Record(vec![("day".to_owned(), Value::Day(day))]),
            };
            self.trace_observation(m, (&input, &id), &record)?;
        }
        Ok(())
    }
}

/// What instance `i` read for fact `fact` of a movement stated `same`: its day's reads, one
/// leg's, every leg's, or all it read; a leg whose reads no evaluation kept gives all (L §8.2
/// One read set; S3, P43 on #241).
fn reads_of_fact(i: &Inst, fact: &Fact) -> Collected {
    match fact {
        Fact::Day => i.day_coll.clone(),
        Fact::Leg(name) => i.leg_reads.get(name).unwrap_or(&i.coll).clone(),
        Fact::Amounts if !i.leg_reads.is_empty() => {
            let mut out = Collected::default();
            for reads in i.leg_reads.values() {
                out.merge(reads);
            }
            out
        }
        Fact::Amounts | Fact::Any => i.coll.clone(),
    }
}

/// A leg whose amount is a hole `same` as a record: its `from` and `to` its own, its `amount`
/// and `day` that hole; for a movement stated `same` whose sides disagree on the amount (L
/// §5.14, §6.1 `HOLE-SAME` per fact; P42 on #241), and for a hole-day instance's leg a later
/// version changes (pick P12).
fn same_leg_record(lg: &LegVal, same: &Hole) -> Value {
    let end = |e: &End| match e {
        End::Place(p) => Value::Key(KeyVal::Thing(p.clone())),
        End::Hole(h) => Value::Hole(h.clone()),
    };
    Value::Record(vec![
        ("from".to_owned(), end(&lg.from)),
        ("to".to_owned(), end(&lg.to)),
        ("amount".to_owned(), Value::Hole(same.clone())),
        ("day".to_owned(), Value::Hole(same.clone())),
    ])
}

/// What the later versions change of a leg of an instance whose day is a hole: its `from`,
/// its `to`, its amount (E §5 steps 2 and 3; tally 2.3.1 N250.21).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Changes {
    /// A version moves the leg's `from`.
    from: bool,
    /// A version moves the leg's `to`.
    to: bool,
    /// A version changes the leg's amount, or its δ isn't known to be 0.
    amount: bool,
}

impl Changes {
    /// Whether a version moves one of the leg's places (E §5 step 2).
    fn moved(self) -> bool {
        self.from || self.to
    }
}

/// A leg of an instance whose day is the hole `h`, whose places later versions move as
/// `changes` says and whose amount none changes, as a record: a place a version moves, and
/// its `day`, that hole; its other place and its `amount` its own, fields that need no unknown
/// (L §5.14; tally 2.3.1 R1, N250.21).
///
/// # Errors
///
/// The leg's amount refused.
fn moved_leg_record(m: &Reality, lg: &LegVal, h: &Hole, changes: Changes) -> Result<Value, Stop> {
    let end = |e: &End, moved: bool| match e {
        _ if moved => Value::Hole(h.clone()),
        End::Place(p) => Value::Key(KeyVal::Thing(p.clone())),
        End::Hole(own) => Value::Hole(own.clone()),
    };
    Ok(Value::Record(vec![
        ("from".to_owned(), end(&lg.from, changes.from)),
        ("to".to_owned(), end(&lg.to, changes.to)),
        (
            "amount".to_owned(),
            amount_value(m, &lg.amount, lg.unit.as_deref())?,
        ),
        ("day".to_owned(), Value::Hole(h.clone())),
    ]))
}

/// An amount as a value: a quantity of its unit, or its hole. L §5.14.
fn amount_value(m: &Reality, a: &Amount, unit: Option<&str>) -> Result<Value, Stop> {
    match a {
        Amount::Qty(q) => {
            let u = unit.ok_or_else(|| invariant("an amount with no unit"))?;
            qty_value(m, q, u)
        }
        Amount::Hole(h) => Ok(Value::Hole(h.clone())),
        Amount::Refused(r) => Err(Stop::Refused(r.clone())),
    }
}

/// A plan instance as a value. L §5.14.
pub(crate) fn inst_ref(i: &Inst) -> MvRef {
    MvRef {
        id: i.id.clone(),
        day: i.day.as_ref().and_then(Dated::day),
        src: MvSrc::Inst(i.move_id.clone(), i.keys.clone()),
    }
}

/// An observation's row key, its id without the input's name and brackets. L §5.19.
pub(super) fn row_key(o: &Obs) -> &str {
    o.id.strip_prefix(o.input.as_str())
        .and_then(|r| r.strip_prefix('['))
        .and_then(|r| r.strip_suffix(']'))
        .unwrap_or(&o.id)
}

/// Whether span `sp` holds a movement on `day`: a day in it or not; none for a hole day or
/// none. L §5.19 Bindings.
fn day_in(day: Option<&Dated>, sp: &Span) -> Option<bool> {
    match day? {
        Dated::Day(d) => Some(sp.contains(*d)),
        Dated::Hole(_) => None,
    }
}

/// Whether a binding over span `sp` rests on movement `x`, so meets its S1 refusal: one dated
/// on a day outside the span doesn't; one whose day is a hole or refused may be a
/// member, so does (refcore's `span_members`, its `s1_refusal` test). L §5.19 One movement on
/// two statements, Bindings.
fn rests_in(x: &Moving, sp: &Span) -> bool {
    match x.day() {
        Some(Dated::Day(d)) => sp.contains(d),
        Some(Dated::Hole(_)) | None => true,
    }
}

/// A running read's binding over a kind the walk itself books, decisions or pads, through a
/// day after the reader's: its future, `RUN-FUTURE` (refcore's `span_members`, its first
/// test). L §5.14.1 Running reads ("a span past the reader's day, in a binding or in flows,
/// is its future"), §8.4.
fn binds_future(m: &Reality, (dom, kind): (MvDom, &str), sp: &Span, ctx: Ctx) -> Result<(), Stop> {
    let Some(WalkAt { point, .. }) = ctx.at else {
        return Ok(());
    };
    let sub = m.subtree(kind);
    let walked = sub.contains("pads")
        || m.moves
            .entries()
            .any(|(_, mv)| mv.decision && sub.contains(&mv.kind));
    let t = point.day;
    let after = t
        .next()
        .ok_or_else(|| invariant("the day after a decision's"))?;
    if dom != MvDom::Event || !walked || sp.end().is_some_and(|end| end <= after) {
        return Ok(());
    }
    let through = match sp.end() {
        Some(end) => end
            .previous()
            .ok_or_else(|| invariant("a span that ends before the calendar"))?
            .text(),
        None => "the end".to_owned(),
    };
    Err(Stop::Refused(Refusal::new(
        Code::Run(Run::Future),
        format!(
            "a decision on {} binds over {kind} through {through}, after its day",
            t.text()
        ),
    )))
}

/// Whether a member brings `unfulfilled:` to its reader: a plan's past instance, or a
/// decision's run dated before the read's cut, that no observation fulfils (refcore's
/// `span_members`, its registers). L §5.19, §8.2.
fn unfulfilled(m: &Reality, i: &Inst, cut: Day) -> bool {
    if !i.obs.is_empty() {
        return false;
    }
    if m.moves.get(&i.move_id).is_some_and(|mv| mv.decision) {
        return i.day.as_ref().and_then(Dated::day).is_some_and(|d| d < cut);
    }
    i.status == Status::Past
}

/// A register entry with no figure (L §8.2).
fn entry(word: &'static str, text: &str) -> Entry {
    Entry {
        word,
        text: text.to_owned(),
        figure: String::new(),
    }
}

/// The movements `movements`, a group of `groups` stated `same` bound once, by its first
/// member in id order among them, `sides` the same lines' sides in id order (refcore's
/// `once_by_same`; pick P42). L §6.1, §5.19.
fn once_by_same((groups, sides): (&Groups, &[String]), movements: Vec<Moving>) -> Vec<Moving> {
    if groups.is_empty() {
        return movements;
    }
    let keep = {
        let ids: Vec<&str> = movements.iter().map(Moving::id).collect();
        kept_once(groups, sides, &ids)
    };
    movements
        .into_iter()
        .zip(keep)
        .filter_map(|(mv, kept)| kept.then_some(mv))
        .collect()
}

/// The hole of instance `inst` when the plan pass read none of its fields: its day is a hole
/// and it is a decision's, which never runs without a day, or its law date is a hole, under
/// which nothing of it is read; else none (L §5.14; the review of G1's fix, BF1, P10).
fn unread_hole(m: &Reality, inst: &Inst) -> Option<Hole> {
    let Some(Dated::Hole(h)) = &inst.day else {
        return None;
    };
    let decision = m.moves.get(&inst.move_id).is_some_and(|mv| mv.decision);
    (decision || matches!(inst.law, Law::Hole(_))).then(|| h.clone())
}

/// Whether instance `i` is one a binding reads to leave out: a plan's that doesn't exist (its
/// `when` is false, its keys aren't there), a decision's run whose `when` was false. L §8.2
/// One read set; P20, P21.
fn left_out(m: &Reality, i: &Inst) -> bool {
    let run = m.moves.get(&i.move_id).is_some_and(|mv| mv.decision);
    i.status == Status::None || (run && i.exists == Exists::No)
}

/// A decision's instance `inst` of move `mv` that no walk ran: one that may exist and whose
/// day is a hole runs on no day of the walk, so it is a member with that hole, never left
/// out, but one of an uncertain kind that nothing fulfils, which a binding counts only `with
/// uncertain` (the read `ctx`'s), whatever its day; else none (L §5.14: a movement whose day
/// is a hole makes every binding over a span it might fall in a hole; §5.14 Certainty; tally
/// 2.3 review of G1, B1; tally 2.3.1 R1, N250.1).
fn hole_day_member(m: &Reality, (mv, inst): (&MoveInfo, &Inst), ctx: Ctx) -> Option<Moving> {
    if !ctx.unc && inst.obs.is_empty() && uncertain_on_hole_day(m, mv, inst) {
        return None;
    }
    match inst.day {
        Some(Dated::Hole(_)) if inst.exists != Exists::No => Some(Moving::Plan(inst.clone())),
        Some(Dated::Hole(_) | Dated::Day(_)) | None => None,
    }
}

/// Whether instance `i` of move `mv` is of an uncertain kind and its day a hole: unfulfilled
/// (a plan's status `Hole` is one no observation the cut applies fulfils), a binding without
/// `with uncertain` leaves it out whatever its day (L §5.14 Certainty, §5.19 Bindings; tally
/// 2.3.1 R1, N250.1).
fn uncertain_on_hole_day(m: &Reality, mv: &MoveInfo, i: &Inst) -> bool {
    matches!(i.day, Some(Dated::Hole(_)))
        && kind_certainty(m, &mv.kind) == Some(Certainty::Uncertain)
}

/// Movements in movement order: by day, a hole day after every day, then by their typed order,
/// ties as they came (refcore's stable `sort`). L §5.19 Bindings; E §4.1.
fn in_order(movements: Vec<Moving>) -> Result<Vec<Moving>, Stop> {
    let mut keyed = Vec::with_capacity(movements.len());
    for mv in movements {
        keyed.push((mv.sort_key()?, mv));
    }
    keyed.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(keyed.into_iter().map(|(_, mv)| mv).collect())
}

/// The refusal of a field read of move instance `id`, which doesn't exist (L §5.14 An
/// instance's existence is a cell, §8.4 `RUN-NONE`; tally 2.3.1 R6, C-2).
fn not_there(id: &str) -> Stop {
    Stop::Refused(Refusal::new(
        Code::Run(Run::None),
        format!("{id} doesn't exist: a field read of it"),
    ))
}

#[cfg(test)]
mod tests {
    use crate::api::{Asks, Files, Given, Request, run};

    /// A sweep, a decision reading its place, on day `dd`: the day is written in for `DAY`.
    const SWEEP: &str = r#"model sweep
  tally 2
  on 2026-04-20
  start 2026-01-01
  end 2026-12-31

source fixture "invented values"

unit PLN: currency minor 0.01

kind sweeps: event certain
  leg cash: PLN

place sav: accounts PLN lives 2026-01-01..

place world: outside lives 1900-01-01..

input pay: money PLN

input dd: date

move sweep: sweeps
  on dd
  reads sav
  leg cash from sav to world amount min(bal(sav), pay)

version v
  cite fixture
  governs always
  known 2026-01-01
  pay = 100.00 PLN "its cap"
  dd = DAY "its day"
  opening[sav] = 500.00 PLN at 2026-01-01 start "own"

ask n: count(t in sweeps over 2026-05)

ask n_jan: count(t in sweeps over 2026-01)

ask s: sum(t.cash.amount for t in sweeps over 2026-05)

ask n_p: count(t in sweeps over 2026-05) with dd = 2026-05-05

ask w_to: count(t in sweeps over 2026-05 where t.cash.to == sav)

ask w_from: count(t in sweeps over 2026-05 where t.cash.from == sav)

ask d_to: sweep.cash.to
"#;

    /// A pay on day `pd`, written in for `DAY`, its amount raised by a version known on 15
    /// March, inside the run (`hd1`'s twin; T's pin oracle, `known_in`).
    const RAISE: &str = r#"model raise
  tally 2
  on 2026-04-20
  start 2026-01-01
  end 2026-12-31

source fixture "invented values"

unit PLN: currency minor 0.01

place konto: accounts PLN lives 2026-01-01..

place world: outside lives 1900-01-01..

input pay: money PLN

input pd: date

move pay_x: transfers
  on pd
  leg cash from world to konto amount pay

version v
  cite fixture
  governs always
  known 2026-01-01
  pay = 100.00 PLN "its amount"
  pd = DAY "its day"
  opening[konto] = 500.00 PLN at 2026-01-01 start "own"

version raise
  cite fixture
  governs always
  known 2026-03-15
  replace pay = 150.00 PLN "raised"

ask amt: pay_x.cash.amount

ask amt_p10: pay_x.cash.amount with pd = 2026-03-10

ask amt_p20: pay_x.cash.amount with pd = 2026-03-20

ask from: pay_x.cash.from
"#;

    /// A top-up, a decision that reads the sweeps of its month by their amounts, beside the
    /// sweep of [`SWEEP`] on day `dd`, written in for `DAY` (the review of G1's fix, BF1).
    const TOP: &str = r#"model top
  tally 2
  on 2026-04-20
  start 2026-01-01
  end 2026-12-31

source fixture "invented values"

unit PLN: currency minor 0.01

kind sweeps: event certain
  leg cash: PLN

kind tops: event certain
  leg cash: PLN

place sav: accounts PLN lives 2026-01-01..

place sv2: accounts PLN lives 2026-01-01..

place world: outside lives 1900-01-01..

input pay: money PLN

input dd: date

input dt: date

move sweep: sweeps
  on dd
  reads sav
  leg cash from sav to world amount min(bal(sav), pay)

move top: tops
  on dt
  reads sv2, sweep
  leg cash from world to sv2 amount sum(t.cash.amount for t in sweeps over month_of(dt)) + 0 * bal(sv2)

version v
  cite fixture
  governs always
  known 2026-01-01
  pay = 100.00 PLN "its cap"
  dt = 2026-05-31 "the top-up's day"
  dd = DAY "the sweep's day"
  opening[sav] = 500.00 PLN at 2026-01-01 start "own"
  opening[sv2] = 0.00 PLN at 2026-01-01 start "own"

ask f: flows(top, 2026-05)

ask b: bal(sv2, 2026-06-30)

ask f_p: flows(top, 2026-05) with dd = 2026-05-04
"#;

    /// An uncertain kind's decision and plan on day `dm`, written in for `DAY` (#250 row 1,
    /// the review of G1's fix, N-a's `b1e`).
    const MAYBE: &str = r#"model maybe
  tally 2
  on 2026-04-20
  start 2026-01-01
  end 2026-12-31

source fixture "invented values"

unit PLN: currency minor 0.01

kind maybes: event uncertain
  leg cash: PLN

place sv2: accounts PLN lives 2026-01-01..

place sv4: accounts PLN lives 2026-01-01..

place world: outside lives 1900-01-01..

input pay: money PLN

input dm: date

move mb: maybes
  on dm
  reads sv2
  leg cash from sv2 to world amount min(bal(sv2), pay)

move mp: maybes
  on dm
  leg cash from sv4 to world amount pay

version v
  cite fixture
  governs always
  known 2026-01-01
  pay = 100.00 PLN "its cap"
  dm = DAY "its day"
  opening[sv2] = 500.00 PLN at 2026-01-01 start "own"
  opening[sv4] = 500.00 PLN at 2026-01-01 start "own"

ask n: count(t in maybes over 2026-05)

ask s: sum(t.cash.amount for t in maybes over 2026-05)

ask nu: count(t in maybes over 2026-05) with uncertain
"#;

    /// Pays on day `w`, written in for `DAY`, whose second a version known inside the run
    /// moves to another place, its amount left (T's `t23-g1/surv-changed-amt`, #250).
    const MOVED: &str = r#"model moved
  tally 2
  on 2026-06-10
  start 2026-03-01
  end 2026-06-30

source fixture "invented values"

unit PLN: currency minor 0.01

index ks = [k1, k2]

place konto: accounts PLN lives 2026-01-01..

place other: accounts PLN lives 2026-01-01..

place world: outside lives 1900-01-01..

input amt[k in ks]: money PLN

input dest[k in ks]: key accounts

input w: date

input d_at: date

move pay[k in ks]: transfers
  on w
  at d_at
  leg cash from world to dest[k] amount amt[k]

version v
  cite fixture
  governs always
  known 2026-01-01
  d_at = 2026-03-01 "its law date"
  w = DAY "its day"
  amt[k1] = 100.00 PLN "its amount"
  amt[k2] = 100.00 PLN "its amount"
  dest[k1] = konto "its place"
  dest[k2] = konto "its place"
  opening[konto] = 500.00 PLN at 2026-03-01 start "own"
  opening[other] = 0.00 PLN at 2026-03-01 start "own"

version a1
  cite fixture
  governs 2026..
  known 2026-03-20
  replace amt[k1] = 150.00 PLN "raised"

version a2
  cite fixture
  governs 2026..
  known 2026-03-22
  replace dest[k2] = other "moved"

ask q2: pay[k2].cash

ask q1: pay[k1].cash
"#;

    /// The lines of ask `ask` over [`SWEEP`] with its day `day`.
    fn lines(day: &str, ask: &str) -> Vec<String> {
        lines_of(SWEEP, day, ask)
    }

    /// The lines of ask `ask` over text `text` with its day `day`.
    fn lines_of(text: &str, day: &str, ask: &str) -> Vec<String> {
        let given = vec![Given {
            path: "sweep.tally".to_owned(),
            bytes: Some(text.replace("DAY", day).into_bytes()),
        }];
        let request = Request::new(Asks::Named(vec![ask.to_owned()]));
        run(Files::new(given), &request)
            .expect("an answer")
            .lines()
            .to_vec()
    }

    #[test]
    fn a_decision_whose_day_is_a_hole_is_a_member_with_its_hole() {
        // L §5.14: a movement whose day is a hole makes every binding over a span it might fall
        // in a hole, a decision as any (tally 2.3 review of G1, B1): a window and a `?` alike
        let window = "?\"date within window(2026-05-04, 2026-05-06) not precise enough for the day of sweep\" (dd)";
        for ask in ["n", "n_jan", "s"] {
            assert_eq!(
                lines("window(2026-05-04, 2026-05-06)", ask),
                [
                    format!("{ask}: {window}"),
                    "  unknown day: dd".to_owned(),
                    "  cut 2026-04-20".to_owned()
                ],
                "{ask}: the window's hole"
            );
            assert_eq!(
                lines("?\"which day is the sweep?\"", ask),
                [
                    format!("{ask}: ?\"which day is the sweep?\" (dd)"),
                    "  cut 2026-04-20".to_owned()
                ],
                "{ask}: the question's hole"
            );
        }
        assert_eq!(
            lines("window(2026-05-04, 2026-05-06)", "n_p"),
            ["n_p: 1", "  pinned: dd = 2026-05-05", "  cut 2026-04-20"],
            "a pinned day"
        );
    }

    #[test]
    fn a_hole_day_decisions_places_are_its_hole_through_a_binding_too() {
        // E §4.2 step 3, "a read of its legs is that hole" (tally 2.3.1 R1, N250.8, C-13.9):
        // a hole-day decision's place is the day's hole by its name and through a binding
        // alike, so a `where` on it makes the member unknown, whether it would fail or take it
        let window = "?\"date within window(2026-05-04, 2026-05-06) not precise enough for the day of sweep\" (dd)";
        let day = "window(2026-05-04, 2026-05-06)";
        for ask in ["w_to", "w_from", "d_to"] {
            assert_eq!(
                lines_of(SWEEP, day, ask),
                [
                    format!("{ask}: {window}"),
                    "  unknown day: dd".to_owned(),
                    "  cut 2026-04-20".to_owned()
                ],
                "{ask}: the window's hole"
            );
        }
    }

    #[test]
    fn an_uncertain_movement_on_a_hole_day_is_left_out_without_with_uncertain() {
        // L §5.14 Certainty (tally 2.3.1 R1, N250.1): a binding counts an unfulfilled movement
        // of an uncertain kind only `with uncertain`, whatever its day, a decision's and a
        // plan's alike
        let day = "window(2026-05-04, 2026-05-06)";
        assert_eq!(
            lines_of(MAYBE, day, "n"),
            ["n: 0", "  cut 2026-04-20"],
            "none counted"
        );
        assert_eq!(
            lines_of(MAYBE, day, "s"),
            ["s: 0.00 PLN", "  cut 2026-04-20"],
            "none summed"
        );
        let nu = lines_of(MAYBE, day, "nu");
        assert!(
            nu.first()
                .is_some_and(|line| line.starts_with("nu: ?\"date within window")),
            "with uncertain, the day's hole: {nu:?}"
        );
    }

    #[test]
    fn a_hole_day_plans_leg_a_version_moves_keeps_its_amount() {
        // L §5.14 (tally 2.3.1 R1, N250.21): a version that moves k2's place and leaves its
        // amount makes the place it moves and the day the hole, its from and amount values;
        // one that changes k1's amount leaves its places
        let window = "?\"date within window(2026-03-05, 2026-03-25) not precise enough for the day of pay[k2]\" (w)";
        assert_eq!(
            lines_of(MOVED, "window(2026-03-05, 2026-03-25)", "q2"),
            [
                format!("q2: (from: world, to: {window}, amount: 100.00 PLN, day: {window})"),
                "  unknown day: w".to_owned(),
                "  cut 2026-06-10".to_owned()
            ],
            "the moved place's hole"
        );
        let k1 = "?\"date within window(2026-03-05, 2026-03-25) not precise enough for the day of pay[k1]\" (w)";
        assert_eq!(
            lines_of(MOVED, "window(2026-03-05, 2026-03-25)", "q1"),
            [
                format!("q1: (from: world, to: konto, amount: {k1}, day: {k1})"),
                "  unknown day: w".to_owned(),
                "  cut 2026-06-10".to_owned()
            ],
            "the changed amount's hole"
        );
    }

    #[test]
    fn a_field_of_a_hole_day_decision_is_its_hole_never_run_order() {
        // a hole-day decision is never waited for: read inside another decision, its leg's
        // amount, which no walk read, is its hole (L §5.14; the review of G1's fix, BF1, P10)
        let window = "?\"date within window(2026-05-04, 2026-05-06) not precise enough for the day of sweep\" (dd)";
        for ask in ["f", "b"] {
            assert_eq!(
                lines_of(TOP, "window(2026-05-04, 2026-05-06)", ask),
                [
                    format!("{ask}: {window}"),
                    "  unknown day: dd".to_owned(),
                    "  cut 2026-04-20".to_owned()
                ],
                "{ask}: the window's hole"
            );
            assert_eq!(
                lines_of(TOP, "?\"which day?\"", ask),
                [
                    format!("{ask}: ?\"which day?\" (dd)"),
                    "  cut 2026-04-20".to_owned()
                ],
                "{ask}: the question's hole"
            );
        }
        assert_eq!(
            lines_of(TOP, "window(2026-05-04, 2026-05-06)", "f_p"),
            [
                "f_p: 100.00 PLN",
                "  pinned: dd = 2026-05-04",
                "  cut 2026-04-20"
            ],
            "a pinned day"
        );
    }

    #[test]
    fn a_field_of_a_hole_day_instance_a_later_version_changes_is_its_hole() {
        // its booking cut is a hole with its day (L §5.14), so the amount a later version
        // changes isn't known; a field no version changes stays a value (pick P12)
        for (day, hole) in [
            (
                "window(2026-03-01, 2026-03-31)",
                vec![
                    "amt: ?\"date within window(2026-03-01, 2026-03-31) not precise enough for the day of pay_x\" (pd)",
                    "  unknown day: pd",
                    "  cut 2026-04-20",
                ],
            ),
            (
                "?\"which day is the pay?\"",
                vec!["amt: ?\"which day is the pay?\" (pd)", "  cut 2026-04-20"],
            ),
        ] {
            assert_eq!(lines_of(RAISE, day, "amt"), hole, "{day}");
            assert_eq!(
                lines_of(RAISE, day, "from"),
                ["from: world", "  cut 2026-04-20"],
                "{day}: its from"
            );
            assert_eq!(
                lines_of(RAISE, day, "amt_p10"),
                [
                    "amt_p10: 100.00 PLN",
                    "  pinned: pd = 2026-03-10",
                    "  cut 2026-04-20"
                ],
                "{day}: booked before the raise"
            );
            assert_eq!(
                lines_of(RAISE, day, "amt_p20"),
                [
                    "amt_p20: 150.00 PLN",
                    "  pinned: pd = 2026-03-20",
                    "  cut 2026-04-20"
                ],
                "{day}: booked after it"
            );
        }
    }

    /// A monthly spend skipped in February by its `when`, and in March by a `none` day: DAY
    /// is February's.
    const GONE: &str = r#"model gone
  tally 2
  on 2026-04-20
  start 2026-01-01
  end 2026-05-31

source s "probe"

unit PLN: currency minor 0.01

index month = month 2026-01..2026-05

place konto: accounts PLN lives 2026-01-01..

place shop: outside lives 1900-01-01..

input dd[m in month]: date

input skip_m: month

input amt: money PLN

move spend[m in month]: transfers
  on dd[m]
  when m != skip_m
  leg cash from konto to shop amount amt

ask d2: spend[2026-02].day

ask a2: spend[2026-02].cash.amount

ask d3: spend[2026-03].day

ask d4: spend[2026-04].day

ask isn: spend[2026-02] == none

version v
  cite s
  governs always
  known 2026-01-01
  skip_m = 2026-02 "s"
  amt = 10.00 PLN "s"
  dd[2026-01] = 2026-01-20 "s"
  dd[2026-02] = DAY "s"
  dd[2026-03] = none "s"
  dd[2026-04] = 2026-04-20 "s"
  dd[2026-05] = 2026-05-20 "s"
"#;

    #[test]
    fn a_field_read_of_an_instance_that_doesnt_exist_is_refused_its_day_too() {
        // L §5.14 An instance's existence is a cell (tally 2.3.1 R6, C-2): a false `when` and
        // a `none` day leave no instance; any field read of it is RUN-NONE, and the missing
        // instance equals nothing
        let gone =
            |id: &str| format!("refused: RUN-NONE spend[{id}] doesn't exist: a field read of it");
        for (ask, want) in [
            ("d2", format!("d2: {}", gone("2026-02"))),
            ("a2", format!("a2: {}", gone("2026-02"))),
            ("d3", format!("d3: {}", gone("2026-03"))),
            ("d4", "d4: 2026-04-20".to_owned()),
            ("isn", "isn: false".to_owned()),
        ] {
            assert_eq!(
                lines_of(GONE, "2026-02-20", ask),
                [want, "  cut 2026-04-20".to_owned()],
                "{ask}"
            );
        }
    }

    /// A two-key move whose first key's `where` is refused in February (DAY is unused).
    const PREFIX: &str = r#"model prefix
  tally 2
  on 2026-01-01
  start 2026-01-01
  end 2026-03-31

source s "probe"

unit PLN: currency minor 0.01

index month = month 2026-01..2026-03

index kinds = [a]

place konto: accounts PLN lives 2026-01-01..

place shop: outside lives 1900-01-01..

input dz[m in month]: int

input ten: int

input amt: money PLN

input d25: int

input fm: month

move spend[m in month where ten / dz[m] > 0, k in kinds]: transfers
  on day_of(m, d25)
  leg cash from konto to shop amount amt

ask dy: spend[fm, a].day

ask am: spend[fm, a].cash.amount

version v
  cite s
  governs always
  known 2026-01-01
  ten = 10 "s"
  amt = 10.00 PLN "s"
  d25 = 25 "s"
  fm = 2026-02 "s"
  dz[2026-01] = 1 "s"
  dz[2026-02] = 0 "s"
  dz[2026-03] = 1 "s"
"#;

    #[test]
    fn a_field_read_at_the_full_keys_of_a_tuple_refused_before_them_is_its_refusal() {
        // L §5.14 An instance's existence is a cell (l.1362; tally 2.3.1 R6, G2's review B2):
        // the tuple refused before its later key stands for every instance its keys begin, so
        // a field read at the full keys is its refusal, not a missing instance
        for ask in ["dy", "am"] {
            assert_eq!(
                lines_of(PREFIX, "", ask),
                [
                    format!("{ask}: refused: RUN-DIV division by zero"),
                    "  cut 2026-01-01".to_owned()
                ],
                "{ask}"
            );
        }
    }
}
