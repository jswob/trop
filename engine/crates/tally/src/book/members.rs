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

use std::collections::{BTreeMap, BTreeSet};

use super::adjust::Adjustment;
use super::observe::Obs;
use super::reading::Pad;
use super::same::{Groups, and_list, kept_once, same_differ};
use crate::base::{Code, Load, Refusal, Run};
use crate::eval::{Collected, Ctx, Engine, Entry, Env};
use crate::hole::{Hole, spread};
use crate::model::{Fulfils, KeyVal, MoveInfo, MvRef, MvSrc, Reality, Span, Value};
use crate::num::Num;
use crate::syntax::{Bind, Call, Expr, Stop};
use crate::time::Day;
use crate::walk::{
    Amount, At, Dated, DayWindow, Depends, End, Exists, Inst, LegOut, LegVal, Status, Term, WalkAt,
    invariant, qty_value,
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
    /// For a row known in a window that holds the cut whose day and its instance's differ,
    /// that row's hole: which comes first is undecided (L §4.5, tally 2.1.8 R12, B6).
    pub two_days: Option<Hole>,
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
    /// after every day, then its movement order, typed (refcore's `(day_lo(x.day) or
    /// datetime.date.max, order_key(x.order))`; tally 2.1.2). L §5.19 Bindings, §2.5; E §4.1.
    fn sort_key(&self) -> Result<(Day, Term), Stop> {
        let day = match self.day() {
            Some(Dated::Day(d)) => d,
            Some(Dated::Window(w)) => w.lo,
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
        let movements = self.movements_of(m, dom, ctx)?;
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
        let mut two_days: BTreeMap<String, Hole> = BTreeMap::new();
        for x in self.movements_of(m, dom, ctx)? {
            // a binding rests on each movement it takes: S1's refusal is its own (L §5.19, P42)
            if let Some(e) = self.s1_refusal(m, x.id(), ctx.cut)?
                && rests_in(&x, sp)
            {
                return Err(Stop::Refused(e));
            }
            if let Moving::Obs(o) = &x
                && let Some(und) = &o.undecided
            {
                // a row known in a window that holds the cut is one movement read seen and
                // unseen: unseen, a row of an observed input is no row, and a movement of an
                // event kind is the instance it fulfils; a member either way is a member,
                // else an unknown one (L §4.5, tally 2.1.8 R12; refcore's `span_members`)
                let alt = self.unseen_day(m, o, dom.0, ctx)?;
                let (seen_in, unseen_in) =
                    (sp.contains(o.day), alt.is_some_and(|d| sp.contains(d)));
                if alt.is_some_and(|d| d != o.day) {
                    two_days.insert(o.id.clone(), und.clone());
                }
                if seen_in || unseen_in {
                    let unknown = (!(seen_in && unseen_in)).then(|| und.clone());
                    out.push((x, unknown));
                }
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
                Some(Dated::Day(_)) => continue,
                // a window inside the span is a member, one outside it isn't, one across its
                // edge may be (refcore's `span_members`; L §4.4)
                Some(Dated::Window(w)) => match window_member(&w, sp) {
                    Some(true) => None,
                    Some(false) => continue,
                    None => {
                        let text = format!(
                            "date within {} not precise enough for a binding over {}",
                            w.question, dom.1
                        );
                        Some(Hole::asked(&text, x.id()).carrying(&w.value.carry))
                    }
                },
            };
            if let Moving::Plan(i) = &x {
                if let Exists::Unknown(h) = &i.exists {
                    st = spread(&[st.as_ref(), Some(h)]);
                }
                if i.depends == Some(Depends::Covered)
                    && let Some(Dated::Window(w)) = &i.day
                {
                    // modelled past on some of its window's days, booked on the rest: whether
                    // it is a member is the window's hole (E §4.1 step 2; tally 2.1.8 item 77)
                    let text = format!(
                        "date within {} not precise enough for a binding over {}",
                        w.question, dom.1
                    );
                    let h = Hole::asked(&text, &i.id).carrying(&w.value.carry);
                    st = spread(&[st.as_ref(), Some(&h)]);
                }
                if let Some(e) = &i.err {
                    return Err(Stop::Refused(e.clone()));
                }
            }
            if let Moving::Obs(o) = &x
                && let Some(t) = self.obs_target(m, o, ctx.cut)?
            {
                fulfilled.insert(t);
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
                        coll.entries.insert(entry("unfulfilled", &i.id));
                    }
                    if i.status == Status::Uncertain {
                        coll.entries.insert(entry("uncertain", &i.id));
                    }
                    coll.merge(&i.coll);
                }
                Moving::Obs(o) => coll.merge(&o.coll),
                Moving::Adj(a) => coll.merge(&a.coll),
                Moving::Pad(p, _) => coll.merge(&p.coll),
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
                two_days: two_days.get(x.id()).cloned(),
            });
        }
        Ok(res)
    }

    /// The day an undecided row's movement has unseen: for a binding over an event kind, the
    /// day of the instance it fulfils, if any; for an observed input's, none, since unseen it
    /// is no row (refcore's `span_members`, `x.target`). L §4.5, §5.19.
    ///
    /// # Errors
    ///
    /// As [`Engine::move_instances`].
    fn unseen_day(
        &mut self,
        m: &Reality,
        o: &Obs,
        dom: MvDom,
        ctx: Ctx,
    ) -> Result<Option<Day>, Stop> {
        let (MvDom::Event, Some(tid)) = (dom, o.target_id()) else {
            return Ok(None);
        };
        Ok(match self.planned_by_id(m, &tid, ctx.cut)? {
            Some(Moving::Plan(i)) => i.day.as_ref().and_then(Dated::day),
            Some(Moving::Obs(_) | Moving::Adj(_) | Moving::Pad(..)) | None => None,
        })
    }

    /// `next(K, d)`: `first(m in K over d ..) ties by m.day`, the first movement of event kind
    /// K dated on or after d. A member whose day is a hole may come first, so the result is
    /// that hole; two on the first day are a tie (`RUN-TIE`); none at the cut is
    /// `HOLE-NOTYET`, since `d ..` reaches past every cut (refcore's `m_c_next`,
    /// `first_by_day`). L §5.14.1 (its table), §3.1, §8.1, §8.4; E §4.3.
    ///
    /// # Errors
    ///
    /// `RUN-TIE`; the refusals the day and the members meet.
    pub(crate) fn next_movement(
        &mut self,
        m: &Reality,
        call: &Call,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let [Expr::Name(kind), day] = call.args.as_slice() else {
            return Err(invariant("a next(K, d) load didn't check"));
        };
        let d = self.ev(m, day, env, ctx)?;
        if let Value::Hole(_) = d {
            return Ok(d);
        }
        let d = match crate::eval::one_day(m, &d, "the day of next")? {
            Ok(d) => d,
            Err(h) => return Ok(Value::Hole(h)),
        };
        let members =
            self.span_members(m, (MvDom::Event, kind), &Span::between(Some(d), None), ctx)?;
        // a member whose day is a hole may come first, and so may an undecided row's
        // movement where its two days differ, never a tie (L §4.5, tally 2.1.8 R12, B6)
        let mut hole_days: Vec<Option<&Hole>> = Vec::with_capacity(members.len());
        for x in &members {
            if matches!(x.day, Dated::Hole(_)) {
                hole_days.push(x.unknown.as_ref());
            }
        }
        hole_days.extend(members.iter().map(|x| x.two_days.as_ref()));
        if let Some(h) = spread(&hole_days) {
            return Ok(Value::Hole(h));
        }
        // its holes and refusals name the form as written, not its definition's bound
        // variable or the day's value (L §5 A form defined by another; tally 2.1.7 R6, SR-7)
        let written = crate::load::expr_text(&Expr::Call(Box::new(call.clone())));
        // the first last day, and every member that may fall on or before it: a window's
        // first and last days (refcore's `first_by_day`, `day_lo`, `day_hi`)
        let Some(top) = members.iter().filter_map(|x| x.day.hi()).min() else {
            let text = format!("{written}: no {kind} yet, and its span reaches past the cut");
            return Ok(Value::Hole(Hole::asked(&text, kind)));
        };
        let firsts: Vec<&MvMember> = members
            .iter()
            .filter(|x| matches!(x.day.lo(), Some(lo) if lo <= top))
            .collect();
        let unknown: Vec<Option<&Hole>> = firsts.iter().map(|x| x.unknown.as_ref()).collect();
        if let Some(h) = spread(&unknown) {
            return Ok(Value::Hole(h));
        }
        let [first] = firsts.as_slice() else {
            if let Some((w, id)) = firsts.iter().find_map(|x| match &x.day {
                Dated::Window(w) => Some((w, &x.id)),
                Dated::Day(_) | Dated::Hole(_) => None,
            }) {
                // which comes first is a comparison of windows, undecided (L §4.4)
                let text = format!(
                    "date within {} not precise enough for {written}",
                    w.question
                );
                return Ok(Value::Hole(Hole::asked(&text, id).carrying(&w.value.carry)));
            }
            let ids: Vec<&str> = firsts.iter().map(|x| x.id.as_str()).collect();
            return Err(Stop::Refused(Refusal::new(
                Code::Run(Run::Tie),
                format!(
                    "{written} has a tie at {} that changes the result, and its ties by \
                     doesn't break it",
                    ids.join(", ")
                ),
            )));
        };
        self.absorb(&first.coll);
        Ok(first.value.clone())
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
            let day = lg.day.day().unwrap_or(o.day);
            for p in [&lg.from, &lg.to].into_iter().filter_map(End::place) {
                if !self.covering(m, p, day, cut)?.is_empty() {
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
        let mut pins = Vec::new();
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
                    // the binding read its day to leave it out: the pins that day took are
                    // the binding's reads too (L §8.2 One read set; tally 2.1.8 §4, #161)
                    Status::None | Status::Modelled => pins.extend(
                        i.coll
                            .entries
                            .iter()
                            .filter(|e| e.word == "pinned")
                            .cloned(),
                    ),
                    Status::Uncertain if !ctx.unc => {}
                    Status::Uncertain
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
        for e in pins {
            self.note(e.word, e.text, e.figure);
        }
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
        }
        if sub.contains("pads") {
            // the walk's own movements, as it booked them (L §5.14.1 Running reads)
            let pads = self.walk_pads(m, ctx)?;
            out.extend(pads.into_iter().map(|(p, at)| Moving::Pad(p, at)));
        }
        out.extend(self.decided_members(m, &decided, ctx)?);
        let groups = self.same_groups(m, ctx.cut)?;
        let sides = self.same_sides(m, ctx.cut)?;
        in_order(once_by_same((&groups, &sides), out))
    }

    /// The runs of the decision moves `decided` that a binding over their kind ranges over,
    /// as the walk ran them: inside a decision, its walk's and those it continues', the ones
    /// before its point; else the main walk's of the read's timeline. One that doesn't exist
    /// or is modelled past is no member, nor is an uncertain one without `with uncertain`
    /// (refcore's `movements_of`, its decisions). L §5.14.1 Running reads, §5.19 Bindings.
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
            None => (self.main_walk(m, ctx.cut)?, None),
        };
        let mut out = Vec::new();
        for mv in decided {
            for inst in self.move_instances(m, mv, ctx.cut)?.iter() {
                let Some(run) = self.chain_run(walk, &inst.id)? else {
                    continue;
                };
                if limit.is_some_and(|point| run.point >= point) || run.inst.exists == Exists::No {
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
                Ok(all.into_iter().find(|a| a.id == v.id).map(Moving::Adj))
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
            None => (self.main_walk(m, ctx.cut)?, false),
        };
        match self.chain_run(walk, id)? {
            Some(run) => Ok(Some(Moving::Plan(run.inst.clone()))),
            None if running => Err(Stop::Refused(Refusal::new(
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
    /// # Errors
    ///
    /// `RUN-RECORD` for a movement the cut doesn't see; the movement's refusal; a refused
    /// leg; `LOAD-FIELD` for a field it lacks.
    pub(crate) fn mv_field(
        &mut self,
        m: &Reality,
        v: &MvRef,
        name: &str,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let Some(src) = self.mv_record(m, v, ctx)? else {
            return Err(Stop::Refused(Refusal::new(
                Code::Run(Run::Record),
                format!("{} doesn't exist", v.id),
            )));
        };
        let kind = src.kind(m)?;
        let is_leg = m.kind_legs(&kind).iter().any(|(n, _, _)| n == name);
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
            (Moving::Pad(p, _), _) => self.absorb(&p.coll),
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
        if let Moving::Obs(o) = &src
            && (name == "day" || is_leg)
            && let Some(h) = self.same_rows_hole(m, o, (name != "day").then_some(name), ctx.cut)?
        {
            return Ok(Value::Hole(h));
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

    /// The `HOLE-SAME` of observed row `o` where the other observed rows of its `same` group
    /// disagree with it on a leg's amount or day (on leg `leg` alone, where given): a read of
    /// the movement away from its places (refcore's `same_rows_hole`). L §5.19 One movement
    /// on two statements, §6.1, §8.1.
    fn same_rows_hole(
        &mut self,
        m: &Reality,
        o: &Obs,
        leg: Option<&str>,
        cut: Day,
    ) -> Result<Option<Hole>, Stop> {
        let groups = self.same_groups(m, cut)?;
        let Some(group) = groups.get(&o.id) else {
            return Ok(None);
        };
        let all = self.observations(m, cut)?;
        let mut others: Vec<&Obs> = Vec::new();
        for y in group.iter().filter(|y| **y != o.id) {
            others.extend(all.iter().find(|x| x.id == *y));
        }
        if others.is_empty() {
            return Ok(None);
        }
        same_differ(m, o, &others, leg)
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
                let insts = self.move_instances(m, mv, cut)?;
                Ok(insts
                    .iter()
                    .find(|i| i.keys == keys)
                    .map_or(Value::None, |i| Moving::Plan(i.clone()).value()))
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
            match self.leg_refused(m, (src.id(), name), day, ctx)? {
                Some(LegOut::Refused(err)) => return Err(Stop::Refused(err)),
                Some(LegOut::Hole(h)) => {
                    self.trace_walked_back(back);
                    return Ok(Value::Hole(h));
                }
                None => {}
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
            return Err(Stop::Refused(Refusal::new(
                Code::Run(Run::None),
                format!("{} doesn't exist: a field read of it", src.id()),
            )));
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
                day: Some(a.inst_day),
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
        Some(Dated::Day(_) | Dated::Window(_)) | None => None,
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
        day: i.day.as_ref().and_then(Dated::lo),
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

/// Whether a movement dated by window `w` is a member of a binding over span `sp`: true
/// inside it, false wholly outside it, none across its edge (refcore's `span_members`).
/// L §4.4, §5.19 Bindings.
fn window_member(w: &DayWindow, sp: &Span) -> Option<bool> {
    if sp.contains(w.lo) && sp.contains(w.hi) {
        return Some(true);
    }
    let after = sp.end().is_some_and(|end| w.lo >= end);
    let before = sp.start().is_some_and(|start| w.hi < start);
    (after || before).then_some(false)
}

/// Whether a binding over span `sp` rests on movement `x`, so meets its S1 refusal: one dated
/// on a day outside the span doesn't; one whose day is a window, a hole or refused may be a
/// member, so does (refcore's `span_members`, its `s1_refusal` test). L §5.19 One movement on
/// two statements, Bindings.
fn rests_in(x: &Moving, sp: &Span) -> bool {
    match x.day() {
        Some(Dated::Day(d)) => sp.contains(d),
        Some(Dated::Window(_) | Dated::Hole(_)) | None => true,
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
        return i.day.as_ref().and_then(Dated::lo).is_some_and(|d| d < cut);
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

/// Movements in movement order: by day, then by their typed order, ties as they came (refcore's
/// stable `sort` by `(day_lo(x.day) or datetime.date.max, order_key(x.order))`). L §5.19
/// Bindings; E §4.1.
fn in_order(movements: Vec<Moving>) -> Result<Vec<Moving>, Stop> {
    let mut keyed = Vec::with_capacity(movements.len());
    for mv in movements {
        keyed.push((mv.sort_key()?, mv));
    }
    keyed.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(keyed.into_iter().map(|(_, mv)| mv).collect())
}
