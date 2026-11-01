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
//! Invariants: members come in movement order, by day, then by the text of their order; a
//! member's register reaches its reader only when the member passes the binding's `where`; the
//! walk's own movements (pads, decisions) and `with uncertain` members stop `NotBuilt`.

use std::collections::BTreeSet;

use super::adjust::Adjustment;
use super::observe::Obs;
use crate::base::{Code, Load, Refusal, Run};
use crate::eval::{Collected, Ctx, Engine, Entry, Env};
use crate::hole::{Hole, spread};
use crate::model::{Fulfils, KeyVal, MvRef, MvSrc, Reality, Span, Value};
use crate::syntax::{Bind, Stop};
use crate::time::Day;
use crate::walk::{Amount, Dated, End, Exists, Inst, LegVal, Status, invariant, qty_value};

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
}

impl Moving {
    /// Its stable id. L §2.5.
    fn id(&self) -> &str {
        match self {
            Moving::Plan(i) => &i.id,
            Moving::Obs(o) => &o.id,
            Moving::Adj(a) => &a.id,
        }
    }

    /// Its day: a day, a hole, or none for an instance refused before its day was known.
    /// L §5.14.
    fn day(&self) -> Option<Dated> {
        match self {
            Moving::Plan(i) => i.day.clone(),
            Moving::Obs(o) => Some(Dated::Day(o.day)),
            Moving::Adj(a) => Some(Dated::Day(a.day)),
        }
    }

    /// Its place among a binding's members: its day, a hole day last, then its order's text
    /// past the day (refcore's `(day_lo(x.day), str(x.order))`). L §5.19 Bindings.
    fn sort_key(&self) -> (Option<Day>, String) {
        let day = self.day().as_ref().and_then(Dated::day);
        let text = match self {
            Moving::Plan(i) => i.order.member_text(),
            Moving::Obs(o) => {
                let key = row_key(o);
                format!("-1, {}, {})", quoted(&o.input), quoted(key))
            }
            Moving::Adj(a) => format!(
                "-1, {}, {}, {})",
                quoted(&a.move_id),
                quoted(&a.inst_id),
                quoted(&a.leg.name)
            ),
        };
        (day, text)
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
        }
    }
}

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
                    return self.unknown_day(m, bind, dom, h, ctx);
                }
                let d = crate::ask::date_of(&v, "the day of a binding")?;
                let next = d
                    .next()
                    .ok_or_else(|| invariant("a binding's day past the calendar"))?;
                Span::between(Some(d), Some(next))
            }
            (None, Some(over)) => match self.span_value(m, over, env, ctx)? {
                Err(h) => return self.unknown_day(m, bind, dom, h, ctx),
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

    /// A binding whose day or span is a hole: no member when the kind has none, else that
    /// hole (refcore's `bind_day`, `kind_has_member`). L §5.7, §8.1.
    fn unknown_day(
        &mut self,
        m: &Reality,
        bind: &Bind,
        dom: (MvDom, &str),
        h: Hole,
        ctx: Ctx,
    ) -> Result<Result<Vec<MvMember>, Hole>, Stop> {
        if self.movements_of(m, dom, ctx)?.is_empty() {
            return Ok(Ok(Vec::new()));
        }
        if bind.filter.is_some() {
            return Err(Stop::NotBuilt(
                "a binding whose day is a hole and whose where may rule its members out",
            ));
        }
        Ok(Err(h))
    }

    /// The movements of the domain dated in span `sp`, an instance and the observations that
    /// fulfil it counted once, each with what it brings to a reader (refcore's
    /// `span_members`). L §5.19 Bindings, §5.14, §8.2.
    fn span_members(
        &mut self,
        m: &Reality,
        dom: (MvDom, &str),
        sp: &Span,
        ctx: Ctx,
    ) -> Result<Vec<MvMember>, Stop> {
        let mut out: Vec<(Moving, Option<Hole>)> = Vec::new();
        let mut fulfilled: BTreeSet<String> = BTreeSet::new();
        for x in self.movements_of(m, dom, ctx)? {
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
            };
            if let Moving::Plan(i) = &x {
                if let Exists::Unknown(h) = &i.exists {
                    st = spread(&[st.as_ref(), Some(h)]);
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
            out.push((x, st));
        }
        let drop = self.counted_once(m, &out, &fulfilled, ctx.cut)?;
        out.retain(|(x, _)| !drop.contains(x.id()));
        let mut res = Vec::with_capacity(out.len());
        for (x, unknown) in out {
            let mut coll = Collected::default();
            match &x {
                Moving::Plan(i) => {
                    if i.status == Status::Past && i.obs.is_empty() {
                        coll.entries.insert(entry("unfulfilled", &i.id));
                    }
                    if i.status == Status::Uncertain {
                        coll.entries.insert(entry("uncertain", &i.id));
                    }
                    coll.merge(&i.coll);
                }
                Moving::Obs(o) => coll.merge(&o.coll),
                Moving::Adj(a) => coll.merge(&a.coll),
            }
            res.push(MvMember {
                value: x.value(),
                unknown,
                coll,
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
                Moving::Obs(_) | Moving::Adj(_) => None,
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
                if !self.covering(m, p, day, cut).is_empty() {
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
            let mut out: Vec<Moving> = adjs.iter().cloned().map(Moving::Adj).collect();
            out.sort_by_key(Moving::sort_key);
            return Ok(out);
        }
        let decided = m
            .moves
            .entries()
            .any(|(_, mv)| mv.decision && sub.contains(&mv.kind));
        if sub.contains("pads") || decided {
            return Err(Stop::NotBuilt(
                "a binding over pads or decisions (the walk's own movements)",
            ));
        }
        let mut moves: Vec<&crate::model::MoveInfo> = m
            .moves
            .entries()
            .map(|(_, mv)| mv)
            .filter(|mv| !mv.decision && sub.contains(&mv.kind))
            .collect();
        moves.sort_by(|a, b| a.id.cmp(&b.id));
        let mut out = Vec::new();
        for mv in moves {
            for i in self.move_instances(m, mv, cut)? {
                match i.status {
                    Status::None | Status::Modelled | Status::Uncertain if !self.uncertain => {}
                    Status::Uncertain => {
                        return Err(Stop::NotBuilt(
                            "a binding over uncertain movements under `with uncertain`",
                        ));
                    }
                    Status::None | Status::Modelled => {}
                    Status::Open
                    | Status::LeftOut
                    | Status::Refused
                    | Status::Hole
                    | Status::Decision
                    | Status::Past
                    | Status::Plan => out.push(Moving::Plan(i)),
                }
            }
        }
        for o in self.observations(m, cut)? {
            if sub.contains(&o.kind) {
                out.push(Moving::Obs(o.clone()));
            }
        }
        if sub.contains("adjustments") {
            out.extend(
                self.unsettled_adjustments(m, cut)?
                    .into_iter()
                    .map(Moving::Adj),
            );
        }
        out.sort_by_key(Moving::sort_key);
        Ok(out)
    }

    /// The record behind a movement value at the reader's cut: an instance, an observation or
    /// an adjustment; none when the cut sees none (refcore's `mv_record`, `mv_by_id`).
    /// L §5.14, §3.1.
    fn mv_record(&mut self, m: &Reality, v: &MvRef, ctx: Ctx) -> Result<Option<Moving>, Stop> {
        let cut = ctx.cut;
        match &v.src {
            MvSrc::Inst(move_id, _) => self.instance_by_id(m, move_id, &v.id, cut),
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
                    return self.instance_by_id(m, name, &v.id, cut);
                }
                self.observation_by_id(m, &v.id, cut)
            }
        }
    }

    /// A move's instance by id at the cut (refcore's `mv_by_id`, a move). L §5.14.
    fn instance_by_id(
        &mut self,
        m: &Reality,
        move_id: &str,
        id: &str,
        cut: Day,
    ) -> Result<Option<Moving>, Stop> {
        let mv = m
            .moves
            .get(move_id)
            .ok_or_else(|| invariant("a movement of a move load didn't build"))?;
        if mv.decision {
            return Err(Stop::NotBuilt("a read of a decision's movement (the walk)"));
        }
        let insts = self.move_instances(m, mv, cut)?;
        Ok(insts.into_iter().find(|i| i.id == id).map(Moving::Plan))
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
            (Moving::Adj(_), "day" | "adjusts") => {}
            (Moving::Plan(i), "day") => self.absorb(&i.day_coll),
            (Moving::Plan(i), _) => self.absorb(&i.coll),
            (Moving::Obs(o), _) => self.absorb(&o.coll),
            (Moving::Adj(a), _) => self.absorb(&a.coll),
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
            Moving::Adj(_) => {}
        }
        match name {
            "day" => Ok(match src.day() {
                Some(Dated::Day(d)) => Value::Day(d),
                Some(Dated::Hole(h)) => Value::Hole(h),
                None => Value::None,
            }),
            "fulfils" => self.fulfils_of(m, &src, ctx.cut),
            _ if is_leg => self.leg_field(m, &src, name, ctx),
            _ => attr_field(m, &src, name),
        }
    }

    /// What a movement's `fulfils` names at the cut: the instance or the adjustment, or none
    /// (refcore's `mv_field`, `fulfils`). L §5.19, §5.14.3.
    fn fulfils_of(&mut self, m: &Reality, src: &Moving, cut: Day) -> Result<Value, Stop> {
        let fulfils = match src {
            Moving::Plan(i) => return Ok(i.fulfils.clone().unwrap_or(Value::None)),
            Moving::Adj(_) => return Ok(Value::None),
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
                    .into_iter()
                    .find(|i| i.keys == keys)
                    .map_or(Value::None, |i| Moving::Plan(i).value()))
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
        };
        if let Some(lg) = legs.iter().find(|lg| lg.name == name) {
            if let Amount::Refused(r) = &lg.amount {
                return Err(Stop::Refused(r.clone()));
            }
            if let Some(err) = self.leg_refused(m, (src.id(), name), ctx)? {
                return Err(Stop::Refused(err));
            }
            return leg_record(m, lg);
        }
        if let Moving::Plan(i) = src
            && (i.exists == Exists::No || i.status == Status::None)
        {
            return Err(Stop::NotBuilt(
                "a leg of an instance a later version made exist",
            ));
        }
        if let Some(h) = body_hole(src) {
            return Ok(Value::Hole(h));
        }
        Err(Stop::Refused(Refusal::new(
            Code::Load(Load::Field),
            format!("{name} is not a field of {}", src.id()),
        )))
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
/// and `change` (refcore's `mv_field`, an attribute). L §5.14, §5.14.3.
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
    };
    if let Some(v) = found {
        return Ok(v);
    }
    if let Some(h) = body_hole(src) {
        return Ok(Value::Hole(h));
    }
    Err(Stop::Refused(Refusal::new(
        Code::Load(Load::Field),
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
    let day = match &lg.day {
        Dated::Day(d) => Value::Day(*d),
        Dated::Hole(h) => Value::Hole(h.clone()),
    };
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
fn inst_ref(i: &Inst) -> MvRef {
    MvRef {
        id: i.id.clone(),
        day: i.day.as_ref().and_then(Dated::day),
        src: MvSrc::Inst(i.move_id.clone(), i.keys.clone()),
    }
}

/// An observation's row key, its id without the input's name and brackets. L §5.19.
fn row_key(o: &Obs) -> &str {
    o.id.strip_prefix(o.input.as_str())
        .and_then(|r| r.strip_prefix('['))
        .and_then(|r| r.strip_suffix(']'))
        .unwrap_or(&o.id)
}

/// A register entry with no figure (L §8.2).
fn entry(word: &'static str, text: &str) -> Entry {
    Entry {
        word,
        text: text.to_owned(),
        figure: String::new(),
    }
}

/// A text as the order's tuple writes it (E §4.1).
fn quoted(s: &str) -> String {
    crate::walk::quoted(s)
}
