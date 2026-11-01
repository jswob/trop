//! The plan pass: each move's instances over its binding tuples, dated at the cut, evaluated at their booking cut, and classified.
//! Spec: E §4.1 (enumerate the instances, date them at K, evaluate them at min(t, K),
//! classify them, the movement order); E §5 (steps 0 and 1: dating at K, the booking cut);
//! L §5.14 (moves: keys, `on`, `at`, `for`, `when`, legs, their places, amounts and days; a
//! day or a `when` that is a hole makes a hole movement).
//! Twin: engine/tools/refcore/moves.py (`move_tuples`, `move_instances`, `insts_of_tuples`,
//! `decision_instances`, `eval_inst`, `body_over_law`, `inst_body`, `eval_leg`,
//! `leg_unit_static`, `leg_amount`, `status`, `inst_order`, `key_sort`, `keys_seen`).
//! Invariants: an instance's day is read at the ask's cut K and its body at its booking cut
//! min(t, K); a run-time refusal met evaluating an instance stays with that instance; a move's
//! instances are computed once per cut, and a move that reads its own instances is `RUN-CYCLE`.

use core::cmp::Ordering;

use crate::base::{Code, Load, Refusal, Run};
use crate::eval::{Collected, Ctx, Engine, Env, Member, order_text, reads_name, thing_seen};
use crate::hole::{Hole, spread};
use crate::model::{
    Domain, KeyVal, MoveInfo, Node, NodeKind, Number, Reality, Value, currency_id, money_num,
    num_money,
};
use crate::num::Num;
use crate::syntax::{Certainty, Leg, Stop};
use crate::time::{Day, PeriodKind};

/// A leg's end: a place by name, or a hole, a place nobody has told us (L §5.14 Legs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum End {
    /// A place.
    Place(String),
    /// A hole.
    Hole(Hole),
}

impl End {
    /// The place, when the end is one. L §5.14.
    #[must_use]
    pub fn place(&self) -> Option<&str> {
        match self {
            End::Place(p) => Some(p),
            End::Hole(_) => None,
        }
    }
}

/// A leg's amount: a quantity of its unit (zero or more), a hole, or the refusal it met
/// (L §5.14 Legs, §8.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Amount {
    /// A quantity, in the leg's unit.
    Qty(Num),
    /// A hole.
    Hole(Hole),
    /// A refusal.
    Refused(Refusal),
}

/// The day a movement or a leg falls on: a day, or a hole (L §5.14 `on d`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dated {
    /// A day.
    Day(Day),
    /// A hole.
    Hole(Hole),
}

impl Dated {
    /// The day, when it is one. L §5.14.
    #[must_use]
    pub fn day(&self) -> Option<Day> {
        match self {
            Dated::Day(d) => Some(*d),
            Dated::Hole(_) => None,
        }
    }
}

/// A leg of an instance, evaluated (refcore's `Leg`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegVal {
    /// The leg's name.
    pub name: String,
    /// The place it leaves.
    pub from: End,
    /// The place it enters.
    pub to: End,
    /// Its amount.
    pub amount: Amount,
    /// Its unit: a currency or a unit; none where a hole amount's unit can't be seen.
    pub unit: Option<String>,
    /// Its day.
    pub day: Dated,
}

/// Whether an instance exists: its `when` and its binding tuple's (L §5.14 `when`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Exists {
    /// It exists.
    Yes,
    /// Its `when` is false: it doesn't exist.
    No,
    /// A hole: its `when` or its tuple's `where` is one.
    Unknown(Hole),
}

/// An instance's status in a timeline (E §4.1 step 3, E §1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Not computed yet.
    Open,
    /// It doesn't exist.
    None,
    /// Dated outside [S, E]: kept for reads, never booked.
    LeftOut,
    /// Its evaluation was refused.
    Refused,
    /// A hole movement: its day or its existence is a hole.
    Hole,
    /// A decision's instance, run in the walk.
    Decision,
    /// Of an uncertain kind, no observation fulfilling it.
    Uncertain,
    /// The modelled past: fulfilled by an observation the cut sees, or on a covered day
    /// before the cut; kept for the ledger, never booked (E §4.1 step 3, §6.2).
    Modelled,
    /// Booked, before the cut.
    Past,
    /// Booked, from the cut on.
    Plan,
}

/// A movement's place in the order of the plan pass (E §4.1): its day, stratum, move id and
/// keys in their domains' order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    /// The day; none for a hole day, which sorts last.
    day: Option<Day>,
    /// The move's stratum.
    stratum: u32,
    /// The move's id.
    move_id: String,
    /// The keys' places: (0, a key's place in its index, the thing's stable id) or (1, 0, the
    /// key's text).
    keys: Vec<(u8, usize, String)>,
}

impl Ord for Order {
    /// The movement order of E §4.1: day, stratum, move id, keys in index order, things in
    /// the text order of their stable ids.
    fn cmp(&self, other: &Order) -> Ordering {
        let day = match (self.day, other.day) {
            (Some(a), Some(b)) => a.cmp(&b),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        };
        day.then_with(|| self.stratum.cmp(&other.stratum))
            .then_with(|| self.move_id.cmp(&other.move_id))
            .then_with(|| self.keys.cmp(&other.keys))
    }
}

impl PartialOrd for Order {
    /// As [`Ord`] (E §4.1).
    fn partial_cmp(&self, other: &Order) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Order {
    /// The order as a binding's members sort it on their day: the order's tuple written past
    /// its day, `<stratum>, '<move>', <keys>)` (refcore's `str(order)`). L §5.19 Bindings.
    #[must_use]
    pub fn member_text(&self) -> String {
        let text = self.text();
        let keys = text.split_once('|').map_or("()", |(_, k)| k);
        format!("{}, {}, {keys})", self.stratum, quoted(&self.move_id))
    }

    /// The movement's text among a day's plans, `<move>|<keys>`, the keys' places written as
    /// a tuple (refcore's `osort`, the walk key's text). E §4.1, §4.2.
    #[must_use]
    pub fn text(&self) -> String {
        let parts: Vec<String> = self
            .keys
            .iter()
            .map(|(a, b, s)| format!("({a}, {b}, {})", quoted(s)))
            .collect();
        let tuple = match parts.as_slice() {
            [] => "()".to_owned(),
            [one] => format!("({one},)"),
            [_, _, ..] => format!("({})", parts.join(", ")),
        };
        format!("{}|{tuple}", self.move_id)
    }
}

/// A text quoted as the walk key's tuple writes it: in single quotes, double ones when it
/// holds a single quote and no double one, with backslashes, line ends, tabs and the quote
/// escaped. E §4.1.
pub(crate) fn quoted(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::with_capacity(s.len());
    out.push(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

/// A move instance in one timeline (refcore's `Inst`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inst {
    /// The move.
    pub move_id: String,
    /// Its keys.
    pub keys: Vec<KeyVal>,
    /// The scope with its keys bound.
    pub env: Env,
    /// Its id, `rent_pay[2026-03]`.
    pub id: String,
    /// Its day, at the cut K; none when its evaluation was refused before it.
    pub day: Option<Dated>,
    /// Its law date by the move's `at`.
    pub law: Option<Day>,
    /// Its booking cut min(t, K), for a booked plan.
    pub cut: Option<Day>,
    /// The period it is for.
    pub for_period: Option<Value>,
    /// Whether it exists.
    pub exists: Exists,
    /// Its legs.
    pub legs: Vec<LegVal>,
    /// Its attributes.
    pub attrs: Vec<(String, Value)>,
    /// The refusal its evaluation met.
    pub err: Option<Refusal>,
    /// What its evaluation met.
    pub coll: Collected,
    /// Its status at the timeline's cut.
    pub status: Status,
    /// The observations that fulfil it at the timeline's cut, by id (L §5.19).
    pub obs: Vec<String>,
    /// What its `fulfils` names: an adjustment it settles, or none (L §5.14.3).
    pub fulfils: Option<Value>,
    /// What reading its day met (refcore's `day_coll`): what a read of its `day` brings.
    pub day_coll: Collected,
    /// Its place in the movement order.
    pub order: Order,
}

impl Inst {
    /// An instance of `mv` at `keys`, nothing evaluated yet. E §4.1.
    fn new(mv: &MoveInfo, keys: Vec<KeyVal>, env: Env) -> Inst {
        let id = if keys.is_empty() {
            mv.id.clone()
        } else {
            let texts: Vec<String> = keys.iter().map(KeyVal::text).collect();
            format!("{}[{}]", mv.id, texts.join(", "))
        };
        Inst {
            move_id: mv.id.clone(),
            keys,
            env,
            id,
            day: None,
            law: None,
            cut: None,
            for_period: None,
            exists: Exists::Yes,
            legs: Vec::new(),
            attrs: Vec::new(),
            err: None,
            coll: Collected::default(),
            status: Status::Open,
            obs: Vec::new(),
            fulfils: None,
            day_coll: Collected::default(),
            order: Order {
                day: None,
                stratum: mv.stratum,
                move_id: mv.id.clone(),
                keys: Vec::new(),
            },
        }
    }
}

/// What an instance is evaluated from: its tuple's scope, keys and existence, and the cut a
/// later version's step measures it at (refcore's `eval_inst` arguments).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seed {
    /// The scope with its keys bound.
    pub env: Env,
    /// Its keys.
    pub keys: Vec<KeyVal>,
    /// Its tuple's existence.
    pub exists: Exists,
    /// The cut it is measured at; none for its booking cut min(t, K).
    pub at_cut: Option<Day>,
}

/// A binding tuple of a move: its scope, keys, and the hole of a `where` that is one, or of a
/// domain or law date that is one (refcore's `move_tuples` statuses).
struct Tuple {
    /// The scope.
    env: Env,
    /// The keys.
    keys: Vec<KeyVal>,
    /// Whether it exists.
    exists: Exists,
    /// The hole of a domain or of a law date: the instance's day and existence are that hole.
    domain_hole: Option<Hole>,
}

/// What an instance's body gives: whether it exists, its legs, attributes and period.
pub(super) struct Body {
    /// Whether it exists.
    pub(super) exists: Exists,
    /// Its legs.
    pub(super) legs: Vec<LegVal>,
    /// Its attributes.
    pub(super) attrs: Vec<(String, Value)>,
    /// The period it is for.
    pub(super) for_period: Option<Value>,
    /// What its `fulfils` names.
    pub(super) fulfils: Option<Value>,
}

impl Engine {
    /// A move's instances at cut `cut`, each dated at the cut and evaluated at its booking
    /// cut, sorted in the movement order, with their status (refcore's `move_instances`,
    /// `decision_instances`, `status`). E §4.1, §5 steps 0–1.
    ///
    /// # Errors
    ///
    /// `RUN-CYCLE` for a move that reads its own instances; the refusals the binding tuples
    /// meet; a load refusal met at run; `NotBuilt` past what S builds.
    pub(crate) fn move_instances(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        cut: Day,
    ) -> Result<Vec<Inst>, Stop> {
        let key = (mv.id.clone(), cut);
        if let Some(hit) = self.plans.get(&key) {
            return Ok(hit.clone());
        }
        if self.plan_active.contains(&key) {
            return Err(Stop::Refused(Refusal::new(
                Code::Run(Run::Cycle),
                format!("{} reads its own instances", mv.id),
            )));
        }
        self.plan_active.insert(key.clone());
        let made = self.make_instances(m, mv, cut);
        self.plan_active.remove(&key);
        let mut out = made?;
        // refcore computes a status when it is first read, after the instances are kept
        self.plans.insert(key.clone(), out.clone());
        for inst in &mut out {
            inst.status = self.status(m, mv, inst, cut)?;
        }
        self.plans.insert(key, out.clone());
        Ok(out)
    }

    /// An instance's status at cut `cut`, the observations that fulfil it judged first
    /// (refcore's `status`). E §4.1 step 3, L §5.19.
    fn status(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &mut Inst,
        cut: Day,
    ) -> Result<Status, Stop> {
        inst.obs = self.obs_links_for(m, &inst.id, cut)?;
        let day = inst.day.as_ref().and_then(Dated::day);
        if inst.exists == Exists::No {
            return Ok(Status::None);
        }
        if let Some(d) = day
            && (d < m.start || d > m.end)
        {
            return Ok(Status::LeftOut);
        }
        if inst.err.is_some() {
            return Ok(Status::Refused);
        }
        if !inst.obs.is_empty() && !mv.decision {
            return Ok(Status::Modelled);
        }
        let Some(d) = day else {
            return Ok(Status::Hole);
        };
        if matches!(inst.exists, Exists::Unknown(_)) {
            return Ok(Status::Hole);
        }
        if mv.decision {
            return Ok(Status::Decision);
        }
        if kind_certainty(m, &mv.kind) == Some(Certainty::Uncertain) && inst.obs.is_empty() {
            return Ok(Status::Uncertain);
        }
        if d < cut && self.touches_covered(m, inst, cut) {
            return Ok(Status::Modelled);
        }
        Ok(if d < cut { Status::Past } else { Status::Plan })
    }

    /// Every instance of a move at its binding tuples, sorted (refcore's `insts_of_tuples`).
    /// E §4.1 steps 1–2.
    pub(crate) fn make_instances(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        cut: Day,
    ) -> Result<Vec<Inst>, Stop> {
        let tctx = Ctx::new(None, cut);
        let mut out = Vec::new();
        for tuple in self.move_tuples(m, mv, tctx)? {
            if let Some(hole) = tuple.domain_hole {
                let mut inst = Inst::new(mv, tuple.keys, tuple.env);
                inst.day = Some(Dated::Hole(hole.clone()));
                inst.exists = Exists::Unknown(hole);
                out.push(inst);
                continue;
            }
            let seed = Seed {
                env: tuple.env,
                keys: tuple.keys,
                exists: tuple.exists,
                at_cut: None,
            };
            if let Some(inst) = self.eval_inst(m, mv, &seed, tctx)? {
                out.push(inst);
            }
        }
        for inst in &mut out {
            inst.order = inst_order(m, mv, inst);
        }
        out.sort_by(|a, b| a.order.cmp(&b.order));
        Ok(out)
    }

    /// A move's binding tuples at the cut, depth first in binding order, each binding read at
    /// the move's law date with the keys bound before it (refcore's `move_tuples`). E §4.1
    /// step 1, L §5.14.
    fn move_tuples(&mut self, m: &Reality, mv: &MoveInfo, ctx: Ctx) -> Result<Vec<Tuple>, Stop> {
        let mut out = Vec::new();
        let mut work = vec![Tuple {
            env: Env::empty(),
            keys: Vec::new(),
            exists: Exists::Yes,
            domain_hole: None,
        }];
        while let Some(tuple) = work.pop() {
            let Some(bind) = mv.binds.get(tuple.keys.len()) else {
                out.push(tuple);
                continue;
            };
            let law = match self.move_law(m, mv, &tuple.env, ctx)? {
                Law::Hole(h) => {
                    out.push(Tuple {
                        domain_hole: Some(h),
                        ..tuple
                    });
                    continue;
                }
                Law::Day(d) => Some(d),
                Law::None => None,
            };
            let members = match self.members(
                m,
                core::slice::from_ref(&bind.node),
                &tuple.env,
                ctx.at_law(law),
            )? {
                Err(h) => {
                    out.push(Tuple {
                        domain_hole: Some(h),
                        ..tuple
                    });
                    continue;
                }
                Ok(members) => members,
            };
            let mut next = Vec::with_capacity(members.len());
            for member in members {
                next.push(extend(m, mv, &tuple, &bind.var, &bind.domain, member)?);
            }
            work.extend(next.into_iter().rev());
        }
        Ok(out)
    }

    /// A move's law date by its `at`, with the keys bound so far: none while it reads a key
    /// not yet bound (refcore's `move_law`). L §4.3, §5.14.
    fn move_law(&mut self, m: &Reality, mv: &MoveInfo, env: &Env, ctx: Ctx) -> Result<Law, Stop> {
        let Some(at) = &mv.decl.at else {
            return Ok(Law::None);
        };
        if mv
            .binds
            .iter()
            .any(|b| env.get(&b.var).is_none() && reads_name(at, &b.var))
        {
            return Ok(Law::None);
        }
        let v = self.ev(m, at, env, ctx.at_law(None))?;
        if let Value::Hole(h) = v {
            return Ok(Law::Hole(h));
        }
        if v == Value::None {
            return Err(refuse(
                Run::None,
                format!("the law date of {} is none here", mv.id),
            ));
        }
        Ok(Law::Day(date_of(&v)?))
    }

    /// The binding tuple of `mv` at `keys` under cut `cut`, its `when` read at the cut: none
    /// when no tuple has those keys or its `when` is false; a `when` that is a hole or refused
    /// is taken as existing, its link judged where its instance is (refcore's `link_tuple`).
    /// L §5.19 Links, E §4.1 step 2.
    ///
    /// # Errors
    ///
    /// The refusals the binding tuples meet; a load refusal met at run.
    pub(crate) fn link_tuple(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        keys: &[KeyVal],
        cut: Day,
    ) -> Result<Option<Env>, Stop> {
        let tctx = Ctx::new(None, cut);
        for tuple in self.move_tuples(m, mv, tctx)? {
            if tuple.keys != keys {
                continue;
            }
            if tuple.exists == Exists::No {
                return Ok(None);
            }
            if let (Some(when), Exists::Yes, None) =
                (&mv.decl.when, &tuple.exists, &tuple.domain_hole)
            {
                match self.when_at_cut(m, mv, when, &tuple.env, tctx) {
                    Ok(false) => return Ok(None),
                    Ok(true) => {}
                    Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => {}
                    Err(stop) => return Err(stop),
                }
            }
            return Ok(Some(tuple.env));
        }
        Ok(None)
    }

    /// Whether a link's instance may exist by its `when` at the ask's cut: false only for a
    /// `when` that is false (refcore's `link_tuple`, its `when`). L §5.19.
    fn when_at_cut(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        when: &crate::syntax::Expr,
        env: &Env,
        tctx: Ctx,
    ) -> Result<bool, Stop> {
        let law = match self.move_law(m, mv, env, tctx)? {
            Law::Hole(_) => return Ok(true),
            Law::Day(d) => Some(d),
            Law::None => None,
        };
        let w = self.ev(m, when, env, tctx.at_law(law))?;
        Ok(w != Value::Bool(false))
    }

    /// The places the legs of `mv`'s instance at `env` give at the cut, no amount read; none
    /// where a place can't be read here, so the link is judged by its keys alone (refcore's
    /// `link_places`). L §5.19 Links.
    ///
    /// # Errors
    ///
    /// A load refusal met at run.
    pub(crate) fn link_places(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        env: &Env,
        cut: Day,
    ) -> Result<std::collections::BTreeSet<String>, Stop> {
        let tctx = Ctx::new(None, cut);
        let none = std::collections::BTreeSet::new();
        let law = match self.move_law(m, mv, env, tctx) {
            Ok(Law::Hole(_)) => return Ok(none),
            Ok(Law::Day(d)) => Some(d),
            Ok(Law::None) => None,
            Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => return Ok(none),
            Err(stop) => return Err(stop),
        };
        let mut places = std::collections::BTreeSet::new();
        for leg in &mv.decl.legs {
            for e in [&leg.from, &leg.to] {
                match self.ev(m, e, env, tctx.at_law(law)) {
                    Ok(Value::Key(KeyVal::Thing(name))) => {
                        places.insert(name);
                    }
                    Ok(_) => return Ok(none),
                    Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => {
                        return Ok(none);
                    }
                    Err(stop) => return Err(stop),
                }
            }
        }
        Ok(places)
    }

    /// One instance: its day at the cut K, then, for a plan, its body at its booking cut
    /// min(t, K) or at the seed's cut; none when its day is `never` or `none` (refcore's
    /// `eval_inst`). E §5 steps 0–1, E §4.1 step 2.
    ///
    /// # Errors
    ///
    /// A load refusal met at run; `NotBuilt` past what S builds; a run-time refusal stays
    /// with the instance.
    pub(crate) fn eval_inst(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        seed: &Seed,
        tctx: Ctx,
    ) -> Result<Option<Inst>, Stop> {
        let mut inst = Inst::new(mv, seed.keys.clone(), seed.env.clone());
        self.push();
        let made = self.inst_parts(m, mv, &mut inst, tctx, seed);
        let met = self.pop_alone();
        inst.coll.merge(&met);
        match made {
            Ok(true) => Ok(Some(inst)),
            Ok(false) => Ok(None),
            Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => {
                inst.err = Some(r);
                Ok(Some(inst))
            }
            Err(stop) => Err(stop),
        }
    }

    /// The parts of an instance, evaluated into it; false when its day is `never` or `none`
    /// (refcore's `eval_inst`, its body). E §5 steps 0–1.
    fn inst_parts(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &mut Inst,
        tctx: Ctx,
        seed: &Seed,
    ) -> Result<bool, Stop> {
        let law = match self.move_law(m, mv, &inst.env, tctx)? {
            Law::Hole(h) => {
                inst.day = Some(Dated::Hole(h.clone()));
                inst.exists = Exists::Unknown(h);
                return Ok(true);
            }
            Law::Day(d) => Some(d),
            Law::None => None,
        };
        inst.law = law;
        self.push();
        let v = self.ev(m, &mv.decl.on, &inst.env, tctx.at_law(law));
        let day_coll = self.pop_alone();
        self.absorb(&day_coll);
        inst.day_coll = day_coll;
        let v = v?;
        if matches!(v, Value::Never | Value::None) {
            return Ok(false);
        }
        let day = match v {
            Value::Hole(h) => Dated::Hole(h),
            Value::Number(_)
            | Value::Money(_)
            | Value::Bool(_)
            | Value::Day(_)
            | Value::Period(_)
            | Value::MonthDay(..)
            | Value::Key(_)
            | Value::Text(_)
            | Value::Code(_)
            | Value::Record(_)
            | Value::List(_)
            | Value::Calendar(_)
            | Value::None
            | Value::Never
            | Value::Word(_)
            | Value::Mv(_)
            | Value::OutKey { .. } => Dated::Day(date_of(&v)?),
        };
        inst.day = Some(day.clone());
        inst.exists = seed.exists.clone();
        if mv.decision {
            return Ok(true);
        }
        let cut = match (seed.at_cut, &day) {
            (Some(c), _) => c,
            (None, Dated::Day(d)) => (*d).min(tctx.cut),
            (None, Dated::Hole(_)) => tctx.cut,
        };
        inst.cut = Some(cut);
        if !keys_seen(m, mv, &inst.keys, cut) {
            inst.exists = Exists::No;
            return Ok(true);
        }
        let body = self.inst_body(m, mv, inst, Ctx::new(law, cut), seed.exists.clone())?;
        inst.exists = body.exists;
        inst.legs = body.legs;
        inst.attrs = body.attrs;
        inst.for_period = body.for_period;
        inst.fulfils = body.fulfils;
        Ok(true)
    }

    /// An instance's body in context `ctx`, its law date and cut set: its `when`, period,
    /// legs and attributes, with its move the holder of the projections met (refcore's
    /// `body_over_law`, `inst_body`). L §5.14, §5.14.1, E §5 step 1.
    ///
    /// # Errors
    ///
    /// `RUN-NONE` for a `when` that is none; the refusals of its parts.
    pub(super) fn inst_body(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &Inst,
        ctx: Ctx,
        exists: Exists,
    ) -> Result<Body, Stop> {
        let outer = self.holding(Node::new(NodeKind::Move, &mv.id));
        let body = self.move_body(m, mv, inst, ctx, exists);
        self.holder = outer;
        body
    }

    /// An instance's body: its `when`, period, legs and attributes (refcore's
    /// `body_over_law`, `inst_body`). L §5.14, E §5 step 1.
    fn move_body(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &Inst,
        ctx: Ctx,
        exists: Exists,
    ) -> Result<Body, Stop> {
        let mut ex = exists;
        if mv.decl.fulfils.is_some() {
            crate::book::settlement_move(m, mv)?;
        }
        if let Some(when) = &mv.decl.when {
            let w = self.ev(m, when, &inst.env, ctx)?;
            if w == Value::None {
                return Err(refuse(
                    Run::None,
                    format!("the when of {} is none here", inst.id),
                ));
            }
            if w == Value::Bool(false) {
                let Some(Dated::Hole(h)) = &inst.day else {
                    return Ok(Body {
                        exists: Exists::No,
                        legs: Vec::new(),
                        attrs: Vec::new(),
                        for_period: None,
                        fulfils: None,
                    });
                };
                ex = join(ex, h);
            } else if let Value::Hole(h) = &w {
                ex = join(ex, h);
            } else if w != Value::Bool(true) {
                return Err(refuse_load(
                    Load::Unit,
                    format!("the when of {} is {}", inst.id, shown(m, &w)?),
                ));
            }
        }
        let for_period = match &mv.decl.for_period {
            Some(e) => Some(self.ev(m, e, &inst.env, ctx)?),
            None => None,
        };
        let kind_legs = m.kind_legs(&mv.kind);
        let mut legs = Vec::with_capacity(mv.decl.legs.len());
        for leg in &mv.decl.legs {
            let unit = kind_legs
                .iter()
                .find(|(name, _, _)| *name == leg.name)
                .map(|(_, unit, _)| unit.clone())
                .ok_or(Stop::NotBuilt(
                    "a leg its kind doesn't declare (M0's check)",
                ))?;
            legs.push(self.eval_leg(m, inst, leg, &unit, ctx)?);
        }
        let mut attrs = Vec::with_capacity(mv.decl.attrs.len());
        for (name, e) in &mv.decl.attrs {
            attrs.push((name.clone(), self.ev(m, e, &inst.env, ctx)?));
        }
        let fulfils = match &mv.decl.fulfils {
            Some(e) => Some(self.ev(m, e, &inst.env, ctx)?),
            None => None,
        };
        Ok(Body {
            exists: ex,
            legs,
            attrs,
            for_period,
            fulfils,
        })
    }

    /// One leg: its places, its day and its amount in its unit; a run-time refusal of the
    /// amount stays with the leg (refcore's `eval_leg`). L §5.14 Legs.
    fn eval_leg(
        &mut self,
        m: &Reality,
        inst: &Inst,
        leg: &Leg,
        kind_unit: &str,
        ctx: Ctx,
    ) -> Result<LegVal, Stop> {
        let from_value = self.ev(m, &leg.from, &inst.env, ctx)?;
        let to_value = self.ev(m, &leg.to, &inst.env, ctx)?;
        let from = leg_end(m, inst, leg, from_value, "from")?;
        let to = leg_end(m, inst, leg, to_value, "to")?;
        let day = match &leg.on {
            None => inst
                .day
                .clone()
                .ok_or_else(|| invariant("a leg of an instance with no day"))?,
            Some(e) => {
                let v = self.ev(m, e, &inst.env, ctx)?;
                if v == Value::None {
                    return Err(refuse(
                        Run::None,
                        format!("the day of leg {} of {} is none here", leg.name, inst.id),
                    ));
                }
                if let Value::Hole(h) = v {
                    Dated::Hole(h)
                } else {
                    Dated::Day(date_of(&v)?)
                }
            }
        };
        let read = self.ev(m, &leg.amount, &inst.env, ctx);
        let ends = (&from, &to);
        let (amount, unit) = match read {
            Ok(v) => amount_of(m, &inst.id, &leg.name, kind_unit, &v, ends)?,
            Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => {
                (Amount::Refused(r), None)
            }
            Err(stop) => return Err(stop),
        };
        Ok(LegVal {
            name: leg.name.clone(),
            from,
            to,
            amount,
            unit,
            day,
        })
    }
}

/// A leg's end read as `v`: a place, or a hole; `RUN-NONE` for none, `RUN-DOMAIN` for a value
/// that isn't a place (refcore's `eval_leg`, its places). L §5.14 Legs.
fn leg_end(m: &Reality, inst: &Inst, leg: &Leg, v: Value, side: &str) -> Result<End, Stop> {
    if v == Value::None {
        return Err(refuse(
            Run::None,
            format!(
                "the {side} place of leg {} of {} is none here",
                leg.name, inst.id
            ),
        ));
    }
    if let Value::Hole(h) = v {
        return Ok(End::Hole(h));
    }
    if let Value::Key(KeyVal::Thing(name)) = &v
        && let Some(thing) = m.things.get(name)
        && m.is_place(thing)
    {
        if thing
            .base
            .as_ref()
            .is_some_and(|b| m.subtree("assets").contains(b))
        {
            return Err(refuse_load(
                Load::Kind,
                format!("a leg to or from an asset ({} of {})", leg.name, inst.id),
            ));
        }
        return Ok(End::Place(name.clone()));
    }
    Err(refuse(
        Run::Domain,
        format!(
            "the {side} of leg {} of {} is {}, not a place",
            leg.name,
            inst.id,
            shown(m, &v)?
        ),
    ))
}

/// A leg's amount read as `v`: a hole, `RUN-NONE` for none, a quantity in its unit, else
/// `RUN-DOMAIN`; a run-time refusal stays with the leg (refcore's `eval_leg`, its amount).
/// L §5.14 Legs.
fn amount_of(
    m: &Reality,
    inst_id: &str,
    leg: &str,
    kind_unit: &str,
    v: &Value,
    ends: (&End, &End),
) -> Result<(Amount, Option<String>), Stop> {
    if let Value::Hole(h) = v {
        return Ok((
            Amount::Hole(h.clone()),
            leg_unit_static(m, kind_unit, ends.0, ends.1),
        ));
    }
    if *v == Value::None {
        let r = Refusal::new(
            Code::Run(Run::None),
            format!("the amount of leg {leg} of {inst_id} is none here"),
        );
        return Ok((Amount::Refused(r), None));
    }
    if !matches!(v, Value::Money(_) | Value::Number(_)) {
        let r = Refusal::new(
            Code::Run(Run::Domain),
            format!("the amount of leg {leg} of {inst_id} is {}", shown(m, v)?),
        );
        return Ok((Amount::Refused(r), None));
    }
    match leg_amount(m, inst_id, leg, kind_unit, v, ends.0, ends.1) {
        Ok((q, u)) => Ok((Amount::Qty(q), Some(u))),
        Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => {
            Ok((Amount::Refused(r), None))
        }
        Err(stop) => Err(stop),
    }
}

/// A move's law date: none, a day, or a hole.
enum Law {
    /// No `at`, or one that reads a key not yet bound.
    None,
    /// A day.
    Day(Day),
    /// A hole.
    Hole(Hole),
}

/// A tuple extended by one binding's member: its variable bound, its key made a key of the
/// binding's domain, its hole joined (refcore's `move_tuples`, `go`). L §5.14.
fn extend(
    m: &Reality,
    mv: &MoveInfo,
    tuple: &Tuple,
    var: &str,
    domain: &Domain,
    member: Member,
) -> Result<Tuple, Stop> {
    let value = member
        .env
        .get(var)
        .map(|v| v.value.clone())
        .ok_or_else(|| invariant("a member without its variable"))?;
    let listed = matches!(domain, Domain::List(_) | Domain::Range);
    let key = if let Value::Key(k) = &value {
        k.clone()
    } else if let Value::Mv(r) = &value {
        KeyVal::Mv(r.clone())
    } else if let (Value::Number(n), true) = (&value, listed) {
        KeyVal::Int(
            n.q.to_i64()
                .ok_or(Stop::NotBuilt("a key that isn't an integer"))?,
        )
    } else if let (Value::Day(d), true) = (&value, listed) {
        KeyVal::Day(*d)
    } else {
        crate::eval::to_key(m, &mv.id, &value, domain)?
    };
    let mut keys = tuple.keys.clone();
    keys.push(key);
    let exists = match member.unknown {
        None => tuple.exists.clone(),
        Some(h) => join(tuple.exists.clone(), &h),
    };
    Ok(Tuple {
        env: member.env,
        keys,
        exists,
        domain_hole: None,
    })
}

/// Existence joined with a hole: the union of the questions (L §8.1).
fn join(ex: Exists, h: &Hole) -> Exists {
    match ex {
        Exists::Unknown(own) => match spread(&[Some(&own), Some(h)]) {
            Some(both) => Exists::Unknown(both),
            None => Exists::Unknown(own),
        },
        Exists::Yes | Exists::No => Exists::Unknown(h.clone()),
    }
}

/// Whether the keys of an instance were there at cut `cut`: an index's key added by a later
/// version, or a thing entered after it, makes the instance one that didn't exist then
/// (refcore's `keys_seen`). E §5 steps 1–3.
fn keys_seen(m: &Reality, mv: &MoveInfo, keys: &[KeyVal], cut: Day) -> bool {
    for (bind, key) in mv.binds.iter().zip(keys) {
        match &bind.domain {
            Domain::Index(id) => {
                let added = m
                    .indexes
                    .get(id)
                    .and_then(|ix| ix.added_by(key))
                    .and_then(|v| m.versions.get(v));
                if added.is_some_and(|v| v.known > cut) {
                    return false;
                }
            }
            Domain::Kind(_) => {
                if let KeyVal::Thing(name) = key
                    && let Some(thing) = m.things.get(name)
                    && !thing_seen(m, thing, cut)
                {
                    return false;
                }
            }
            Domain::Axis(_) | Domain::List(_) | Domain::Range => {}
        }
    }
    true
}

/// An event kind's certainty, its own or its nearest parent's (refcore's `kind_certainty`).
/// L §5.7.
#[must_use]
pub fn kind_certainty(m: &Reality, kind: &str) -> Option<Certainty> {
    m.chain(kind).into_iter().find_map(|k| k.certainty)
}

/// An instance's place in the movement order: its day, its move's stratum and id, its keys in
/// their domains' order (refcore's `inst_order`, `key_sort`). E §4.1.
fn inst_order(m: &Reality, mv: &MoveInfo, inst: &Inst) -> Order {
    let mut keys = Vec::with_capacity(inst.keys.len());
    for (bind, key) in mv.binds.iter().zip(&inst.keys) {
        let mut place = None;
        if let Domain::Index(id) = &bind.domain
            && let Some(ix) = m.indexes.get(id)
        {
            place = match ix.pos(key) {
                Some(p) => Some((0, p, String::new())),
                None => Some((0, 0, String::new())),
            };
        } else if let (Domain::Kind(_), KeyVal::Thing(name)) = (&bind.domain, key) {
            place = m.things.get(name).map(|t| (0, 0, order_text(t)));
        } else if let KeyVal::Mv(r) = key {
            let day = r.day.map_or_else(String::new, Day::text);
            place = Some((0, 0, format!("{day} {}", r.id)));
        }
        keys.push(place.unwrap_or_else(|| (1, 0, key.text())));
    }
    Order {
        day: inst.day.as_ref().and_then(Dated::day),
        stratum: mv.stratum,
        move_id: mv.id.clone(),
        keys,
    }
}

/// A hole amount's unit, where it can be seen: the kind's leg unit when it is one, else a
/// place's (refcore's `leg_unit_static`). L §5.14.
fn leg_unit_static(m: &Reality, kind_unit: &str, from: &End, to: &End) -> Option<String> {
    if m.currencies.contains_key(kind_unit) || m.units.contains(kind_unit) {
        return Some(kind_unit.to_owned());
    }
    place_units(m, from, to).into_iter().next()
}

/// A leg's amount as a quantity of its unit: places of another unit are `LOAD-UNIT`, a
/// negative amount `RUN-NEG`, money finer than its minor unit `LOAD-UNIT` (refcore's
/// `leg_amount`). L §5.10, §5.14.
fn leg_amount(
    m: &Reality,
    inst_id: &str,
    leg: &str,
    kind_unit: &str,
    amount: &Value,
    from: &End,
    to: &End,
) -> Result<(Num, String), Stop> {
    let units = place_units(m, from, to);
    let (q, own) = quantity(m, amount)?;
    let unit = if m.currencies.contains_key(kind_unit) || m.units.contains(kind_unit) {
        kind_unit.to_owned()
    } else {
        let found = match (&own, units.iter().next()) {
            (Some(u), _) => Some(u.clone()),
            (None, Some(u)) if q.is_zero() && units.len() == 1 => Some(u.clone()),
            (None, _) => None,
        };
        let Some(u) = found else {
            return Err(refuse_load(
                Load::Unit,
                format!(
                    "the amount of leg {leg} of {inst_id} ({}) is in no unit of a place",
                    shown(m, amount)?
                ),
            ));
        };
        if kind_unit != "any"
            && m.kinds.get(kind_unit).is_some()
            && m.unit_kind_keys.get(&u).map(String::as_str) != Some(kind_unit)
            && !(kind_unit == "currencies" && m.currencies.contains_key(&u))
        {
            return Err(refuse_load(
                Load::Unit,
                format!(
                    "the amount of leg {leg} of {inst_id} is in {u}, not a unit of {kind_unit}"
                ),
            ));
        }
        u
    };
    if !units.is_empty() && (units.len() != 1 || !units.contains(&unit)) {
        let all: Vec<&str> = units.iter().map(String::as_str).collect();
        return Err(refuse_load(
            Load::Unit,
            format!(
                "leg {leg} of {inst_id} moves {unit} between places of {}",
                all.join(", ")
            ),
        ));
    }
    if let Some(u) = &own
        && *u != unit
    {
        return Err(refuse_load(
            Load::Unit,
            format!("the amount of leg {leg} of {inst_id} is in {u}, not {unit}"),
        ));
    }
    if q.is_negative() {
        return Err(refuse(
            Run::Neg,
            format!(
                "negative amount {} for leg {leg} of {inst_id}",
                shown(m, amount)?
            ),
        ));
    }
    if let Some(minor) = m.currencies.get(&unit)
        && !q.checked_div(minor).map_err(num_err)?.is_integer()
    {
        return Err(refuse_load(
            Load::Unit,
            format!(
                "the amount of leg {leg} of {inst_id} ({}) is not money of {unit}: round it",
                shown(m, amount)?
            ),
        ));
    }
    Ok((q, unit))
}

/// The units of a leg's places, but `any` (refcore's `leg_amount`, the places' units). L §5.14.
fn place_units(m: &Reality, from: &End, to: &End) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    for p in [from, to].into_iter().filter_map(End::place) {
        if let Some(u) = m.things.get(p).and_then(|t| m.place_unit(t))
            && u != "any"
        {
            out.insert(u);
        }
    }
    out
}

/// A number or money as a quantity, with its unit where it is one unit to the first power
/// (refcore's `unit_of_value`). L §3.2.
pub(crate) fn quantity(m: &Reality, v: &Value) -> Result<(Num, Option<String>), Stop> {
    if let Value::Money(money) = v {
        let q = money_num(m, money).map_err(num_err)?;
        let (name, _) = crate::model::currency(m, money.currency()).map_err(num_err)?;
        return Ok((q, Some(name.to_owned())));
    }
    if let Value::Number(n) = v {
        return Ok((n.q.clone(), single_unit(n)));
    }
    Err(invariant("a quantity that isn't a number"))
}

/// A number's unit when it is one unit to the first power. L §3.2.
fn single_unit(n: &Number) -> Option<String> {
    match n.unit().terms() {
        [(name, 1)] => Some(name.clone()),
        _ => None,
    }
}

/// A quantity of `unit` as a value: money for a currency, a number of the unit otherwise
/// (refcore's `qty`). L §3.2.
///
/// # Errors
///
/// A number error: money finer than its minor unit, a unit past the cap.
pub fn qty_value(m: &Reality, q: &Num, unit: &str) -> Result<Value, Stop> {
    if m.currencies.contains_key(unit) {
        let cur = currency_id(m, unit).map_err(num_err)?;
        return Ok(Value::Money(num_money(m, q, cur).map_err(num_err)?));
    }
    let u = crate::model::Unit::make(vec![(unit.to_owned(), 1)]).map_err(num_err)?;
    Ok(Value::Number(Number::plain(q.clone(), u)))
}

/// The day a value names: a day, or a day period's (refcore's `date_of`); a value that isn't
/// one is a load refusal met at run. L §4.2.
fn date_of(v: &Value) -> Result<Day, Stop> {
    if let Value::Day(d) = v {
        return Ok(*d);
    }
    if let Value::Period(p) = v
        && p.kind() == PeriodKind::Day
    {
        return Ok(p.start());
    }
    Err(refuse_load(
        Load::Unit,
        "a day that isn't a date".to_owned(),
    ))
}

/// A value as it prints, for a message. L §8.4.
fn shown(m: &Reality, v: &Value) -> Result<String, Stop> {
    v.text(m).map_err(num_err)
}

/// A number error as its refusal (L §7.1).
fn num_err(e: crate::num::NumError) -> Stop {
    crate::eval::num_stop(e)
}

/// A run-time refusal (L §8.4).
fn refuse(code: Run, message: String) -> Stop {
    Stop::Refused(Refusal::new(Code::Run(code), message))
}

/// A load refusal met at run (L §8.3).
fn refuse_load(code: Load, message: String) -> Stop {
    Stop::Refused(Refusal::new(Code::Load(code), message))
}

/// A broken promise: `ENGINE-INVARIANT` (E §11).
pub(crate) fn invariant(what: &str) -> Stop {
    Stop::Refused(Refusal::new(Code::EngineInvariant, what.to_owned()))
}
