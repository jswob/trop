//! The plan pass: each move's instances over its binding tuples, dated at the cut, evaluated at their booking cut, and classified.
//! Spec: E §4.1 (enumerate the instances, date them at K, evaluate them at min(t, K),
//! classify them, the movement order); E §5 (steps 0 and 1: dating at K, the booking cut);
//! L §5.14 (moves: keys, `on`, `at`, `for`, `when`, legs and their days; a day or a `when`
//! that is a hole makes a hole movement); L §8.4 (a refused key's `where`, `when`, attribute
//! or `for` makes a refused movement, its day and legs' places still read; tally 2.1.6);
//! L §6.4 (a move a scenario declares books nothing before its `from`). The binding tuples are
//! `tuples.rs`'s, a leg's places and amount `legs.rs`'s.
//! Twin: engine/tools/refcore/moves.py (`move_instances`, `insts_of_tuples`,
//! `decision_instances`, `eval_inst`, `refused_places`, `body_over_law`, `inst_body`,
//! `status`, `scen_early`, `inst_order`, `key_sort`, `keys_seen`).
//! Invariants: an instance's day is read at the ask's cut K and its body at its booking cut
//! min(t, K); a run-time refusal met evaluating an instance stays with that instance; a move's
//! instances are computed once per cut, and a move that reads its own instances is `RUN-CYCLE`.

use super::tuples::{Law, Tuple};
use crate::base::{Code, Load, Refusal, Run};
use crate::eval::{Collected, Ctx, Engine, Env, order_text, thing_seen};
use crate::hole::{Hole, spread};
use crate::model::{
    Domain, KeyVal, MoveInfo, Node, NodeKind, Number, Reality, Value, World, currency_id,
    money_num, num_money,
};
use crate::num::Num;
use crate::syntax::{Certainty, Stop};
use crate::time::{Day, PeriodKind};
use core::cmp::Ordering;
use std::rc::Rc;

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

/// A booked plan instance's amount A₀, its existence and legs, with the cut that first booked
/// it: what every later cut that books it at the same booking cut must give again (E §9
/// item 5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Booked {
    /// Whether it exists.
    pub exists: Exists,
    /// Its legs.
    pub legs: Vec<LegVal>,
    /// The cut that first booked it.
    pub cut: Day,
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
    /// The order as one typed term, `(day, stratum, move, ((kind, place, id), …))`, the last
    /// day there is for a hole day, so a binding's members and the residual ledger's last ties
    /// compare it term by term (refcore's `inst_order` under `order_key`; tally 2.1.2). E §4.1,
    /// §6.3 Order; L §2.5, §5.19 Bindings.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a key's place past `i64`.
    pub fn term(&self) -> Result<Term, Stop> {
        let mut keys = Vec::with_capacity(self.keys.len());
        for (kind, place, id) in &self.keys {
            let place = i64::try_from(*place).map_err(|_| invariant("a key's place past i64"))?;
            keys.push(Term::Tuple(vec![
                Term::Int(i64::from(*kind)),
                Term::Int(place),
                Term::text(id),
            ]));
        }
        let day = match self.day {
            Some(d) => d,
            None => Day::LAST,
        };
        Ok(Term::Tuple(vec![
            Term::Day(day),
            Term::Int(i64::from(self.stratum)),
            Term::text(&self.move_id),
            Term::Tuple(keys),
        ]))
    }
}

/// One term of a movement order, typed as refcore's `order_key` types it: a day, a number, a
/// text, or a tuple of terms. Two orders compare term by term: days as dates, strata and key
/// places as numbers, ids and stable ids by code point, a tuple term by term with a shorter
/// one first where it begins the other, and two terms of different types by their type, in
/// that order (tally 2.1.2: 2.1.1 compared the order's text, so 10 sorted before 2). E §4.1,
/// §6.3 Order; L §2.5.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    /// A day.
    Day(Day),
    /// A number: a stratum, a key's place in its index.
    Int(i64),
    /// A text: an id or a stable id.
    Text(String),
    /// A tuple of terms.
    Tuple(Vec<Term>),
}

impl Term {
    /// A text term. E §4.1.
    #[must_use]
    pub fn text(s: &str) -> Term {
        Term::Text(s.to_owned())
    }

    /// An observation's or an adjustment's order, `(day, -1, id, …)`: before every plan of
    /// its day, then by its ids (refcore's `ObsMv.order`, `AdjMv.order`). E §4.1, §6.3 Order.
    #[must_use]
    pub fn booked(day: Day, ids: &[&str]) -> Term {
        let mut terms = vec![Term::Day(day), Term::Int(-1)];
        terms.extend(ids.iter().map(|id| Term::text(id)));
        Term::Tuple(terms)
    }

    /// The term's type, as refcore's `order_key` ranks it: a day, a number, a text, a tuple.
    /// E §4.1.
    fn rank(&self) -> u8 {
        match self {
            Term::Day(_) => 1,
            Term::Int(_) => 2,
            Term::Text(_) => 3,
            Term::Tuple(_) => 4,
        }
    }
}

impl Ord for Term {
    /// Term by term as its type (refcore's `order_key`). E §4.1, §6.3 Order.
    fn cmp(&self, other: &Term) -> Ordering {
        match (self, other) {
            (Term::Day(a), Term::Day(b)) => a.cmp(b),
            (Term::Int(a), Term::Int(b)) => a.cmp(b),
            (Term::Text(a), Term::Text(b)) => a.cmp(b),
            (Term::Tuple(a), Term::Tuple(b)) => a.cmp(b),
            (Term::Day(_) | Term::Int(_) | Term::Text(_) | Term::Tuple(_), _) => {
                self.rank().cmp(&other.rank())
            }
        }
    }
}

impl PartialOrd for Term {
    /// As [`Ord`] (E §4.1).
    fn partial_cmp(&self, other: &Term) -> Option<Ordering> {
        Some(self.cmp(other))
    }
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
    /// `decision_instances`, `status`), shared with every other read of the same plan.
    /// E §4.1, §5 steps 0–1.
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
    ) -> Result<Rc<Vec<Inst>>, Stop> {
        let key = (mv.id.clone(), cut);
        if let Some(hit) = self.plans.get(&key) {
            return Ok(Rc::clone(hit));
        }
        if self.plan_active.contains(&key) {
            self.narrow_open_to_days();
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
        self.plans.insert(key.clone(), Rc::new(out.clone()));
        for inst in &mut out {
            inst.status = self.status(m, mv, inst, cut)?;
        }
        let shared = Rc::new(out);
        self.plans.insert(key, Rc::clone(&shared));
        Ok(shared)
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
        if scenario_early(m, mv, inst) {
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
        let outer = self.holding(Node::new(NodeKind::Move, &mv.id));
        let mark = self.reads_apart();
        let made = self.make_instances_held(m, mv, cut);
        self.reads_back(mark);
        self.holder = outer;
        made
    }

    /// [`Engine::make_instances`] with the move holding the projections its `at`, its
    /// bindings' `where` and its body read, which are the move's own (L §5.14.1: "the item
    /// holding the read"); what it reads is the plan's, shared by every reader (E §3.2, §4.1).
    fn make_instances_held(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        cut: Day,
    ) -> Result<Vec<Inst>, Stop> {
        let tctx = self.timeline_ctx(None, cut);
        let mut out = Vec::new();
        for tuple in self.move_tuples(m, mv, tctx)? {
            if let Some(err) = tuple.refused.clone() {
                out.extend(self.refused_tuple(m, mv, tuple, err, tctx)?);
                continue;
            }
            if let Some(hole) = tuple.domain_hole {
                let mut inst = Inst::new(mv, tuple.keys, tuple.env);
                inst.day = Some(Dated::Hole(hole.clone()));
                inst.exists = Exists::Unknown(hole);
                inst.coll = tuple.coll;
                out.push(inst);
                continue;
            }
            let seed = Seed {
                env: tuple.env,
                keys: tuple.keys,
                exists: tuple.exists,
                at_cut: None,
            };
            if let Some(mut inst) = self.eval_inst(m, mv, &seed, tctx)? {
                if !mv.decision
                    && inst.err.is_none()
                    && let Some(booking) = inst.cut
                {
                    self.check_booked(&inst, booking, cut)?;
                }
                inst.coll.merge(&tuple.coll);
                out.push(inst);
            }
        }
        for inst in &mut out {
            inst.order = inst_order(m, mv, inst);
        }
        out.sort_by(|a, b| a.order.cmp(&b.order));
        Ok(out)
    }

    /// The instance of a tuple whose last key's `where` is refused: a refused instance, its
    /// day and legs read as any instance's, none where its `when` is false or its keys weren't
    /// there; a tuple whose later keys are unbound has no day or places to read, so the move's
    /// instances are refused, every place from the run's start (refcore's `insts_of_tuples`,
    /// its refused tuples). L §5.14 Keys, §8.4 (tally 2.1.6); E §4.1 step 2.
    ///
    /// # Errors
    ///
    /// The refusal, for a tuple whose later keys are unbound; a load refusal met at run.
    fn refused_tuple(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        tuple: Tuple,
        err: Refusal,
        tctx: Ctx,
    ) -> Result<Option<Inst>, Stop> {
        if tuple.keys.len() < mv.binds.len() {
            return Err(Stop::Refused(err));
        }
        let seed = Seed {
            env: tuple.env,
            keys: tuple.keys,
            exists: Exists::Yes,
            at_cut: None,
        };
        let Some(mut inst) = self.eval_inst(m, mv, &seed, tctx)? else {
            return Ok(None);
        };
        if inst.exists == Exists::No {
            return Ok(None);
        }
        inst.err = Some(err);
        inst.coll.merge(&tuple.coll);
        Ok(Some(inst))
    }

    /// A booked plan amount never changes: an instance's A₀, its existence and legs, is the
    /// same under every cut that books it at booking cut `booking`, across the asks of a run
    /// (refcore's `inv5`). E §9 item 5.
    fn check_booked(&mut self, inst: &Inst, booking: Day, cut: Day) -> Result<(), Stop> {
        let key = (inst.id.clone(), booking);
        let Some(old) = self.booked.get(&key) else {
            let first = Booked {
                exists: inst.exists.clone(),
                legs: inst.legs.clone(),
                cut,
            };
            self.booked.insert(key, first);
            return Ok(());
        };
        if old.exists == inst.exists && old.legs == inst.legs {
            return Ok(());
        }
        Err(invariant(&format!(
            "a booked plan amount changed: {} booked at {} differs between the cuts {} and {} \
             (E §9 invariant 5)",
            inst.id,
            booking.text(),
            old.cut.text(),
            cut.text()
        )))
    }

    /// One instance: its day at the cut K, then, for a plan, its body at its booking cut
    /// min(t, K) or at the seed's cut; none when its day is `never` or `none` (refcore's
    /// `eval_inst`). A run-time refusal stays with the instance, a refused movement, and a
    /// plan's legs are then read for their places (tally 2.1.6). E §5 steps 0–1, E §4.1
    /// step 2.
    ///
    /// # Errors
    ///
    /// A load refusal met at run; `NotBuilt` past what S builds.
    pub(crate) fn eval_inst(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        seed: &Seed,
        tctx: Ctx,
    ) -> Result<Option<Inst>, Stop> {
        let mut inst = Inst::new(mv, seed.keys.clone(), seed.env.clone());
        self.push();
        let made = match self.inst_parts(m, mv, &mut inst, tctx, seed) {
            Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => {
                inst.err = Some(r);
                self.refused_legs(m, mv, &mut inst, tctx).map(|()| true)
            }
            made => made,
        };
        let met = self.pop_alone();
        inst.coll.merge(&met);
        Ok(made?.then_some(inst))
    }

    /// A refused plan instance's legs, its evaluation stopped before them: their places read
    /// at its booking cut and law date, each leg's amount the refusal; none where a place
    /// can't be read here, so the walk refuses every place from its day (refcore's
    /// `eval_inst`, its refusal). A refused movement reaches as a hole movement does: its day
    /// and its legs' places are still read, since neither depends on whether it exists (L §8.4,
    /// tally 2.1.6). E §4.1 step 2, §4.2 Holes in the walk.
    ///
    /// # Errors
    ///
    /// A load refusal met at run.
    fn refused_legs(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &mut Inst,
        tctx: Ctx,
    ) -> Result<(), Stop> {
        if mv.decision {
            return Ok(());
        }
        let day = inst.day.as_ref().and_then(Dated::day);
        let cut = day.map_or(tctx.cut, |d| d.min(tctx.cut));
        if let Some(legs) = self.refused_places(m, mv, inst, self.timeline_ctx(inst.law, cut))? {
            inst.legs = legs;
        }
        Ok(())
    }

    /// The legs of refused instance `inst` with their places read at `ctx` and no amount, the
    /// refusal in its place, each on the instance's day; none where a place isn't a thing
    /// here, a hole or a refusal, since such a leg could book anywhere (refcore's
    /// `refused_places`, and `dec_bad`'s places). An instance refused before its day may fall
    /// on any day: its legs are dated the end of time, no day (L §7.6 item 1), which falls
    /// before no scenario's `from` (L §6.4: refcore's legs there have no day). L §8.4, E §4.2
    /// Holes in the walk.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for an instance with no refusal; a load refusal met at run.
    pub(super) fn refused_places(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &Inst,
        ctx: Ctx,
    ) -> Result<Option<Vec<LegVal>>, Stop> {
        let err = inst
            .err
            .clone()
            .ok_or_else(|| invariant("a refused instance with no refusal"))?;
        let day = inst.day.clone().unwrap_or(Dated::Day(Day::END));
        let mut legs = Vec::with_capacity(mv.decl.legs.len());
        for leg in &mv.decl.legs {
            let Some(from) = self.read_or_refused(m, &leg.from, &inst.env, ctx)? else {
                return Ok(None);
            };
            let Some(to) = self.read_or_refused(m, &leg.to, &inst.env, ctx)? else {
                return Ok(None);
            };
            let (Value::Key(KeyVal::Thing(from)), Value::Key(KeyVal::Thing(to))) = (from, to)
            else {
                return Ok(None);
            };
            legs.push(LegVal {
                name: leg.name.clone(),
                from: End::Place(from),
                to: End::Place(to),
                amount: Amount::Refused(err.clone()),
                unit: None,
                day: day.clone(),
            });
        }
        Ok(Some(legs))
    }

    /// The value of `e` at `ctx`, or none where it is refused at run (refcore's `try: ev …
    /// except TallyError`, a load refusal raised). L §8.4.
    fn read_or_refused(
        &mut self,
        m: &Reality,
        e: &crate::syntax::Expr,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Option<Value>, Stop> {
        match self.ev(m, e, env, ctx) {
            Ok(v) => Ok(Some(v)),
            Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => Ok(None),
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
            Value::Window(_) => return Err(Stop::NotBuilt("a movement dated by a window")),
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
        if !keys_seen(m, mv, &inst.keys, (cut, self.timeline_world())) {
            inst.exists = Exists::No;
            return Ok(true);
        }
        let body = self.inst_body(
            m,
            mv,
            inst,
            self.timeline_ctx(law, cut),
            seed.exists.clone(),
        )?;
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
}

/// Existence joined with a hole: the union of the questions (L §8.1).
pub(super) fn join(ex: Exists, h: &Hole) -> Exists {
    match ex {
        Exists::Unknown(own) => match spread(&[Some(&own), Some(h)]) {
            Some(both) => Exists::Unknown(both),
            None => Exists::Unknown(own),
        },
        Exists::Yes | Exists::No => Exists::Unknown(h.clone()),
    }
}

/// Whether the keys of an instance were there at cut `cut` in world `world`: an index's key
/// added by a later version or one the world doesn't see, or a thing entered after it, makes
/// the instance one that didn't exist then (refcore's `keys_seen`). E §5 steps 1–3; L §6.4.
fn keys_seen(m: &Reality, mv: &MoveInfo, keys: &[KeyVal], (cut, world): (Day, World)) -> bool {
    for (bind, key) in mv.binds.iter().zip(keys) {
        match &bind.domain {
            Domain::Index(id) => {
                let added = m.indexes.get(id).and_then(|ix| ix.added_by(key));
                let unseen = added.is_some_and(|v| !m.scenarios.sees(world, Some(v)));
                if unseen
                    || added
                        .and_then(|v| m.versions.get(v))
                        .is_some_and(|v| v.known > cut)
                {
                    return false;
                }
            }
            Domain::Kind(_) => {
                if let KeyVal::Thing(name) = key
                    && let Some(thing) = m.things.get(name)
                    && !thing_seen(m, thing, (cut, world))
                {
                    return false;
                }
            }
            Domain::Axis(_) | Domain::List(_) | Domain::Range => {}
        }
    }
    true
}

/// Whether `inst` is an instance of a move a scenario declares whose day, or a leg's own day,
/// falls before the scenario's `from`: it books nothing there, so it is left out; load has
/// refused the move where it sees that day and that the instance exists (refcore's
/// `scen_early`). L §6.4.
fn scenario_early(m: &Reality, mv: &MoveInfo, inst: &Inst) -> bool {
    let Some(from) = scenario_from(m, &mv.id) else {
        return false;
    };
    let legs = inst.legs.iter().filter_map(|leg| leg.day.day());
    let day = inst.day.as_ref().and_then(Dated::day);
    day.into_iter().chain(legs).any(|d| d < from)
}

/// The `from` of the scenario whose file declares move `id`, or `None` for a move of the base
/// (refcore's `scen_from`). L §6.4.
#[must_use]
pub(super) fn scenario_from(m: &Reality, id: &str) -> Option<Day> {
    let scenario = m.scenarios.of(id)?;
    m.scenarios.from.get(scenario).copied()
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
pub(super) fn date_of(v: &Value) -> Result<Day, Stop> {
    if let Value::Day(d) = v {
        return crate::eval::a_day(*d);
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
pub(super) fn shown(m: &Reality, v: &Value) -> Result<String, Stop> {
    v.text(m).map_err(num_err)
}

/// A number error as its refusal (L §7.1).
pub(super) fn num_err(e: crate::num::NumError) -> Stop {
    crate::eval::num_stop(e)
}

/// A run-time refusal (L §8.4).
pub(super) fn refuse(code: Run, message: String) -> Stop {
    Stop::Refused(Refusal::new(Code::Run(code), message))
}

/// A load refusal met at run (L §8.3).
pub(super) fn refuse_load(code: Load, message: String) -> Stop {
    Stop::Refused(Refusal::new(Code::Load(code), message))
}

/// A broken promise: `ENGINE-INVARIANT` (E §11).
pub(crate) fn invariant(what: &str) -> Stop {
    Stop::Refused(Refusal::new(Code::EngineInvariant, what.to_owned()))
}

#[cfg(test)]
mod tests {
    use crate::Days;
    use crate::api::{Asks, Given, Pin, Request, run};

    /// A plan over two bindings whose first key's `where` is refused in February (a
    /// division by zero), before its second binding.
    const LATER_KEY: &str = r#"model later_key
  tally 2
  on 2026-02-15
  start 2026-01-01
  end 2026-03-31

source fixture "invented values"

unit PLN: currency minor 0.01

index month = month 2026-01..2026-03

index tier = 1..2

place konto: accounts PLN lives 2026-01-01..

place box: accounts PLN lives 2026-01-01..

place world: outside lives 2026-01-01..

input dec_dom: int

input fee_amt: money PLN

input wdiv[m in month]: num

move topup[m in month where 1 / wdiv[m] > 0, t in tier]: transfers
  on day_of(m, dec_dom)
  leg cash from world to box amount fee_amt

ask q_bal: bal(konto, d), bal(box, d) for d in [2026-01-20, 2026-03-20]

version facts
  cite fixture
  governs always
  known 2025-12-01
  dec_dom = 20 "the day of the month"
  fee_amt = 3.00 "a fee"
  wdiv[2026-01] = 1 "answered"
  wdiv[2026-02] = 0 "a division by zero: the key's where is refused"
  wdiv[2026-03] = 2 "answered"
  opening[konto] = 100 at 2026-01-01 start
  opening[box] = 0 at 2026-01-01 start
"#;

    #[test]
    fn a_refused_where_before_a_later_binding_refuses_every_place_from_the_start() {
        // L §5.14 Keys, §8.4 (tally 2.1.6): topup[2026-02]'s where is refused before its tier
        // is bound, so it has no day or places to read and the move's instances are refused,
        // every place from the run's start, as refcore's insts_of_tuples raises
        let given = vec![Given {
            path: "later_key.tally".to_owned(),
            bytes: Some(LATER_KEY.as_bytes().to_vec()),
        }];
        let request = Request {
            asks: Asks::Named(vec!["q_bal".to_owned()]),
            pin: Pin::Canonical,
            days: Days::Events,
        };
        let answer = run(given, &request).expect("an answer");
        assert_eq!(
            answer.lines(),
            [
                "q_bal (d = 2026-01-20): (refused: RUN-DIV division by zero, refused: RUN-DIV division by zero)",
                "  cut 2026-02-15",
                "q_bal (d = 2026-03-20): (refused: RUN-DIV division by zero, refused: RUN-DIV division by zero)",
                "  cut 2026-02-15"
            ],
            "every place refused from the start"
        );
    }

    /// A plan over three months whose `when` is refused in February, and its leg's place
    /// there too.
    const REFUSED_PLACE: &str = r#"model refused_place
  tally 2
  on 2026-02-15
  start 2026-01-01
  end 2026-03-31

source fixture "invented values"

unit PLN: currency minor 0.01

index month = month 2026-01..2026-03


place konto: accounts PLN lives 2026-01-01..

place box: accounts PLN lives 2026-01-01..

place world: outside lives 2026-01-01..

input dec_dom: int

input fee_amt: money PLN

input wdiv[m in month]: num

move topup[m in month]: transfers
  on day_of(m, dec_dom)
  when 1 / wdiv[m] > 0
  leg cash from world to (if 1 / wdiv[m] > 0 then box else konto) amount fee_amt

ask q_bal: bal(konto, d), bal(box, d) for d in [2026-01-20, 2026-02-19, 2026-02-20]

version facts
  cite fixture
  governs always
  known 2025-12-01
  dec_dom = 20 "the day of the month"
  fee_amt = 3.00 "a fee"
  wdiv[2026-01] = 1 "answered"
  wdiv[2026-02] = 0 "a division by zero: the when and the place are refused"
  wdiv[2026-03] = 2 "answered"
  opening[konto] = 100 at 2026-01-01 start
  opening[box] = 0 at 2026-01-01 start
"#;

    /// The same plan whose leg's place, read only for February's refused instance, meets a
    /// load refusal at run (a rate per year times a day).
    const PLACE_LOAD_REFUSAL: &str = r#"model place_load_refusal
  tally 2
  on 2026-02-15
  start 2026-01-01
  end 2026-03-31

source fixture "invented values"

unit PLN: currency minor 0.01

index month = month 2026-01..2026-03


place konto: accounts PLN lives 2026-01-01..

place box: accounts PLN lives 2026-01-01..

place world: outside lives 2026-01-01..

input dec_dom: int

input fee_amt: money PLN

input r5: rate

input wdiv[m in month]: num

move topup[m in month]: transfers
  on day_of(m, dec_dom)
  when 1 / wdiv[m] > 0
  leg cash from world to (if wdiv[m] == 0 then (if r5 * 1 day > 0 then konto else box) else box) amount fee_amt

ask q_bal: bal(konto, d), bal(box, d) for d in [2026-01-20, 2026-02-19, 2026-02-20]

version facts
  cite fixture
  governs always
  known 2025-12-01
  dec_dom = 20 "the day of the month"
  fee_amt = 3.00 "a fee"
  r5 = 5 %/year "five per cent a year"
  wdiv[2026-01] = 1 "answered"
  wdiv[2026-02] = 0 "a division by zero: the when and the place are refused"
  wdiv[2026-03] = 2 "answered"
  opening[konto] = 100 at 2026-01-01 start
  opening[box] = 0 at 2026-01-01 start
"#;

    /// A base whose scenario declares a plan over the days, kept by a `when` load can't see,
    /// that exists on 31 December, before the scenario's `from`.
    const EARLY_BASE: &str = r#"model hh
  tally 2
  on 2026-01-02
  start 2025-12-30
  end 2026-01-05

source s "T's fixture card (invented)"

unit PLN: currency minor 0.01

place konto: accounts PLN lives 2025-01-01..

place world: outside lives 1900-01-01..

input from_day: date

input top_amt: money PLN

input lift_amt: money PLN

ask q: bal(konto, 2026-01-05), top_amt, lift_amt

version v
  cite s
  governs always
  known 2025-12-01
  from_day = 2025-12-31
  top_amt = 1.00 PLN
  lift_amt = 2.00 PLN
  opening[konto] = 10.00 PLN at 2025-12-30 start
"#;

    const EARLY_SCENARIO: &str = r"scenario topup over hh
  tally 2
  from 2026-01-01

move top[d in days]: transfers
  on d
  when d >= from_day
  leg cash from world to konto amount top_amt

ask q_top: bal(konto, 2026-01-05) in topup
";

    /// The printed lines of ask `ask` over `files`, under the event walk.
    fn answer(files: &[(&str, &str)], ask: &str) -> Vec<String> {
        let given = files
            .iter()
            .map(|(path, text)| Given {
                path: (*path).to_owned(),
                bytes: Some(text.as_bytes().to_vec()),
            })
            .collect();
        let request = Request {
            asks: Asks::Named(vec![ask.to_owned()]),
            pin: Pin::Canonical,
            days: Days::Events,
        };
        run(given, &request).expect("an answer").lines().to_vec()
    }

    #[test]
    fn a_refused_plan_whose_legs_place_is_refused_refuses_every_place_from_its_day() {
        // L §8.4 (tally 2.1.6): topup[2026-02]'s when is refused, and its leg's place can't be
        // read there either, so it refuses every place from its day, 20 February; January's
        // instance books, and nothing is refused before
        assert_eq!(
            answer(&[("place.tally", REFUSED_PLACE)], "q_bal"),
            [
                "q_bal (d = 2026-01-20): (100.00 PLN, 3.00 PLN)",
                "  unfulfilled: topup[2026-01]",
                "  cut 2026-02-15",
                "q_bal (d = 2026-02-19): (100.00 PLN, 3.00 PLN)",
                "  unfulfilled: topup[2026-01]",
                "  cut 2026-02-15",
                "q_bal (d = 2026-02-20): (refused: RUN-DIV division by zero, refused: RUN-DIV division by zero)",
                "  unfulfilled: topup[2026-01]",
                "  cut 2026-02-15"
            ],
            "every place refused from 20 February"
        );
    }

    #[test]
    fn a_load_refusal_met_reading_a_refused_plans_place_refuses_the_run() {
        // L §3.3, §8.3: a load refusal met at run, here reading the place of February's
        // refused instance, refuses the run in the base's world, as anywhere else
        assert_eq!(
            answer(&[("loadrun.tally", PLACE_LOAD_REFUSAL)], "q_bal"),
            [
                "refused: LOAD-UNIT 0.05 /year * 1 day: a rate times a duration of another period dimension (day/year)"
            ],
            "the run refused"
        );
    }

    #[test]
    fn a_scenarios_plan_before_its_from_is_left_out_at_run() {
        // L §6.4: top[2025-12-31] exists (from_day is 31 December), but scenario topup's from
        // is 1 January, so it books nothing there: 10.00 + 5 × 1.00, and the note
        assert_eq!(
            answer(
                &[("model.tally", EARLY_BASE), ("a.tally", EARLY_SCENARIO)],
                "q_top"
            ),
            [
                "q_top: 15.00 PLN",
                "  world: topup",
                "  unfulfilled: top[2026-01-01]",
                "  left out: top, 1 instances, first 2025-12-31",
                "  cut 2026-01-02"
            ],
            "the instance before the from left out"
        );
    }
}
