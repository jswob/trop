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

use super::tuples::{Law, Tuple, imprecise_law};
use crate::base::{Code, Load, Refusal, Run};
use crate::eval::{Collected, Ctx, Engine, Env, LawClass, order_text, thing_seen};
use crate::hole::{Hole, spread};
use crate::model::{
    Domain, KeyVal, MoveInfo, Node, NodeKind, Number, Reality, Value, World, currency_id,
    money_num, num_money,
};
use crate::num::Num;
use crate::syntax::{Certainty, Expr, Stop};
use crate::time::{Day, PeriodKind};
use core::cmp::Ordering;
use std::collections::BTreeMap;
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

/// The day a movement or a leg falls on: a day, a window of days, or a hole (L §5.14 `on d`,
/// §4.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dated {
    /// A day.
    Day(Day),
    /// A window: the movement falls on one of its days, which no one knows yet.
    Window(DayWindow),
    /// A hole.
    Hole(Hole),
}

/// A window of days a movement or a leg falls on, with no `windows each` choosing a day: it
/// books at the window's last day, and a balance read inside the window is `HOLE-IMPRECISE`
/// (L §4.4; E §4.2 Holes in the walk).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayWindow {
    /// Its first day.
    pub lo: Day,
    /// Its last day.
    pub hi: Day,
    /// The window as a question names it (refcore's `window_text`).
    pub question: String,
    /// The window as L §2.4 prints the value.
    pub shown: String,
    /// The window as a value, which a read of the movement's `day` gives.
    pub value: Box<crate::model::Window>,
}

impl Dated {
    /// The day, when it is one. L §5.14.
    #[must_use]
    pub fn day(&self) -> Option<Day> {
        match self {
            Dated::Day(d) => Some(*d),
            Dated::Window(_) | Dated::Hole(_) => None,
        }
    }

    /// The first day it can be: the day, a window's first; none for a hole (refcore's
    /// `day_lo`). L §4.4, §5.14.
    #[must_use]
    pub fn lo(&self) -> Option<Day> {
        match self {
            Dated::Day(d) => Some(*d),
            Dated::Window(w) => Some(w.lo),
            Dated::Hole(_) => None,
        }
    }

    /// The last day it can be: the day, a window's last; none for a hole (refcore's
    /// `day_hi`). L §4.4, §5.14.
    #[must_use]
    pub fn hi(&self) -> Option<Day> {
        match self {
            Dated::Day(d) => Some(*d),
            Dated::Window(w) => Some(w.hi),
            Dated::Hole(_) => None,
        }
    }

    /// The day as a value: a date, a window, or a hole (refcore's `mv_field`). L §5.14.
    #[must_use]
    pub fn value(&self) -> Value {
        match self {
            Dated::Day(d) => Value::Day(*d),
            Dated::Window(w) => Value::Window(w.value.clone()),
            Dated::Hole(h) => Value::Hole(h.clone()),
        }
    }

    /// The day as L §2.4 prints a value: a date in ISO, a window as its value, a hole as the
    /// hole (refcore's `day_text`). L §2.4 Values.
    #[must_use]
    pub fn text(&self) -> String {
        match self {
            Dated::Day(d) => d.text(),
            Dated::Window(w) => w.shown.clone(),
            Dated::Hole(h) => h.text(),
        }
    }
}

/// The day window `w` gives a movement or a leg: none for a window of numbers (L §4.4).
///
/// # Errors
///
/// As [`Value::text`].
pub(super) fn day_window(m: &Reality, w: &crate::model::Window) -> Result<Option<Dated>, Stop> {
    let (Value::Day(lo), Value::Day(hi)) = (&w.lo, &w.hi) else {
        return Ok(None);
    };
    let question = w.question_text(m).map_err(crate::eval::num_stop)?;
    let value = Box::new(w.clone());
    let shown = Value::Window(value.clone())
        .text(m)
        .map_err(crate::eval::num_stop)?;
    Ok(Some(Dated::Window(DayWindow {
        lo: *lo,
        hi: *hi,
        question,
        shown,
        value,
    })))
}

/// A leg of an instance, evaluated (refcore's `Leg`).
#[derive(Debug, Clone, Eq)]
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
    /// Where a hole amount was read: a `partial` ask line reads it again for the holes it is
    /// linear in (L §5.18 Partial answers; refcore's `Leg.src`).
    pub src: Option<Box<HoleSrc>>,
}

/// Two legs are the same leg when they book the same: where a hole amount was read is how a
/// `partial` line reads it again, not what the leg books (E §9 invariant 5, L §5.18).
impl PartialEq for LegVal {
    /// The same name, places, amount, unit and day, wherever a hole amount was read. L §5.14
    /// Legs, E §9 invariant 5.
    fn eq(&self, other: &LegVal) -> bool {
        self.name == other.name
            && self.from == other.from
            && self.to == other.to
            && self.amount == other.amount
            && self.unit == other.unit
            && self.day == other.day
    }
}

/// A hole amount's expression, the scope and the context it was read in (refcore's
/// `Leg.src`). L §5.18 Partial answers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoleSrc {
    /// The leg's amount as written.
    pub amount: Expr,
    /// Its instance's scope.
    pub env: Env,
    /// The context it was read in.
    pub ctx: Ctx,
}

/// The booked plan amounts of one timeline, by (instance, booking cut, the day a walk up to a
/// day stops at): a walk up to a day books its partial plan list while cells outside it are
/// still being computed, so its A₀ are checked against its own kind only, never against the
/// whole walk's (E §9 item 5, §3.2, §4.1; refcore's `inv5` keys by the timeline).
pub type Bookings = BTreeMap<(String, Day, Option<Day>), Booked>;

/// A booked plan instance's amount A₀, its existence and legs as read at its booking cut,
/// with the cut that first booked it: what every later cut that books it at the same booking
/// cut must give again (E §9 item 5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Booked {
    /// Whether it exists.
    pub exists: Exists,
    /// Its legs.
    pub legs: Vec<BookedLeg>,
    /// The cut that first booked it.
    pub cut: Day,
}

/// One leg of a booked A₀: its places, amount and unit, and its own day where it states one.
/// A leg without its own `on` takes its instance's day, which is read at the ask's cut K,
/// not at the booking cut, so a later K may move it (E §4.1 step 2; refcore's `body_key`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookedLeg {
    /// The leg's name.
    pub name: String,
    /// The place it leaves.
    pub from: End,
    /// The place it enters.
    pub to: End,
    /// Its amount.
    pub amount: Amount,
    /// Its unit.
    pub unit: Option<String>,
    /// Its own day; none where it takes its instance's.
    pub own_day: Option<Dated>,
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
    /// The order as one typed term, `(day, stratum, move, ((kind, place, id), …))`, a hole
    /// day after every day, 9999-12-31 included, so a binding's members and the residual ledger's last ties
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
            Some(d) => Term::Day(d),
            None => Term::End,
        };
        Ok(Term::Tuple(vec![
            day,
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
    /// A hole day's place: after every day, 9999-12-31 included (refcore's `END`; tally 2.1.6,
    /// #48).
    End,
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

    /// The term's type, as refcore's `order_key` ranks it: a day (a hole day's place among
    /// them), a number, a text, a tuple. E §4.1.
    fn rank(&self) -> u8 {
        match self {
            Term::Day(_) | Term::End => 1,
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
            // a hole day after every day (E §4.1, tally 2.1.6)
            (Term::End, Term::End) => Ordering::Equal,
            (Term::Day(_), Term::End) => Ordering::Less,
            (Term::End, Term::Day(_)) => Ordering::Greater,
            (Term::Int(a), Term::Int(b)) => a.cmp(b),
            (Term::Text(a), Term::Text(b)) => a.cmp(b),
            (Term::Tuple(a), Term::Tuple(b)) => a.cmp(b),
            (Term::Day(_) | Term::End | Term::Int(_) | Term::Text(_) | Term::Tuple(_), _) => {
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
    /// Its law date by the move's `at`: a day, or a window it is read at each day of.
    pub law: Law,
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
    /// For a hole booking a later version answered: the cut that version is known from and
    /// the step's versions, whose evaluation its existence and legs are (E §5 step 1, §10).
    pub answered: Option<(Day, Vec<String>)>,
    /// Where its class depends on its window's day: a hole movement from the window's first
    /// day, its day kept (refcore's `class_q`, `class_member`; tally 2.1.8 item 77).
    pub depends: Option<Depends>,
}

/// Why a window-dated instance's class (E §4.1 step 3) is one on some of the window's days
/// and another on the rest (E §4.1 step 2, L §4.4; tally 2.1.8 item 77; refcore's
/// `window_class_depends`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Depends {
    /// Inside the run on some days, before its start on the rest.
    Start,
    /// Modelled past on some days, booked on the rest: whether it is a binding's member
    /// depends on the day too.
    Covered,
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
            law: Law::None,
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
            answered: None,
            depends: None,
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
    /// For a walk up to a day, that day: an instance dated after it is none, its body never
    /// evaluated (refcore's `eval_inst`, `upto`). E §4.1.
    pub upto: Option<Day>,
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
        let made = if self.tracing() {
            self.traced_instances(m, mv, cut)
        } else {
            self.make_instances(m, mv, cut, None).map(|out| (out, None))
        };
        self.plan_active.remove(&key);
        let (mut out, back) = made?;
        // refcore computes a status when it is first read, after the instances are kept
        self.plans.insert(key.clone(), Rc::new(out.clone()));
        let mut due = false;
        for inst in &mut out {
            inst.status = self.status(m, mv, inst, cut)?;
            due = due || crate::book::hole_booked_before(inst, cut);
        }
        if let Some(back) = &back {
            // the observations a status reads print nothing (refcore reads them untraced):
            // the plan isn't a walk the explanation lacks a form for
            self.trace_walked_back(*back);
        }
        let shared = Rc::new(out);
        if !due || mv.decision {
            self.plans.insert(key, Rc::clone(&shared));
            return Ok(shared);
        }
        self.plans.insert(key.clone(), Rc::clone(&shared));
        // a hole booking a later version answers is that version's here (E §5 step 1); a
        // stop other than a refusal leaves no list kept that it didn't answer
        if let Err(stop) = self.answer_hole_bookings(m, mv, cut) {
            self.plans.remove(&key);
            return Err(stop);
        }
        let answered = self.plans.get(&key).map_or(shared, Rc::clone);
        if back.is_some() {
            self.trace_answers(m, mv, &answered, cut)?;
        }
        Ok(answered)
    }

    /// The answered instances of a list a read traces, each evaluated again at the cut of the
    /// version that answered it, traced, so the read names the rows that give the answer
    /// (E §10, §5 step 1; L §5.18).
    ///
    /// # Errors
    ///
    /// As [`Engine::make_instances`].
    fn trace_answers(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        insts: &[Inst],
        cut: Day,
    ) -> Result<(), Stop> {
        let outer = self.holding(Node::new(NodeKind::Move, &mv.id));
        self.trace_plan_open();
        let mut made = Ok(None);
        for inst in insts {
            let Some((at, _)) = &inst.answered else {
                continue;
            };
            let seed = Seed {
                env: inst.env.clone(),
                keys: inst.keys.clone(),
                exists: Exists::Yes,
                at_cut: Some(*at),
                upto: None,
            };
            made = self.eval_inst(m, mv, &seed, self.timeline_ctx(None, cut));
            if made.is_err() {
                break;
            }
        }
        let closed = self.trace_plan_close();
        self.holder = outer;
        made?;
        closed?;
        Ok(())
    }

    /// The day a list of move `mv`'s instances at cut `cut` stops at: a walk up to a day's,
    /// or none for the whole list, where [`Engine::plan_list`] takes it (refcore's
    /// `upto_tag`). E §4.1, §3.2.
    pub(crate) fn list_upto(&self, mv: &MoveInfo, cut: Day) -> Option<Day> {
        let upto = self.body_upto?;
        let whole = mv.decision
            || mv.decl.legs.iter().any(|leg| leg.on.is_some())
            || self.plans.contains_key(&(mv.id.clone(), cut));
        (!whole).then_some(upto)
    }

    /// A move's instances computed while a read is traced, the first time the ask reaches
    /// them: what they read is kept from the reading cell, as the plan's always is, but traced,
    /// so the rows and cells their bindings, days, `when`s and amounts read print under the
    /// read (refcore traces the instances it computes while it explains, once per ask); with
    /// the trace's note of a walk read then. E §10, §3.2, §4.1; L §5.18 Explanation order.
    ///
    /// # Errors
    ///
    /// As [`Engine::make_instances`].
    fn traced_instances(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        cut: Day,
    ) -> Result<(Vec<Inst>, Option<crate::eval::Walked>), Stop> {
        let outer = self.holding(Node::new(NodeKind::Move, &mv.id));
        self.trace_plan_open();
        let made = self.make_instances_held(m, mv, cut, None);
        let back = self.trace_plan_close();
        self.holder = outer;
        Ok((made?, Some(back?)))
    }

    /// A move's instances for the walk being planned: all of them, or, for a walk up to a
    /// day, a plan's dated on or before it, the others' bodies never evaluated; a decision, a
    /// move whose legs state their own days, or one whose instances are all known already, is
    /// taken whole (refcore's `plan_list`). E §4.1, §3.2.
    ///
    /// # Errors
    ///
    /// As [`Engine::move_instances`].
    pub(crate) fn plan_list(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        cut: Day,
    ) -> Result<Rc<Vec<Inst>>, Stop> {
        let Some(upto) = self.list_upto(mv, cut) else {
            return self.move_instances(m, mv, cut);
        };
        let active = (mv.id.clone(), cut);
        let key = (mv.id.clone(), cut, upto);
        if let Some(hit) = self.prefixes.plans.get(&key) {
            return Ok(Rc::clone(hit));
        }
        if self.plan_active.contains(&active) {
            self.narrow_open_to_days();
            return Err(Stop::Refused(Refusal::new(
                Code::Run(Run::Cycle),
                format!("{} reads its own instances", mv.id),
            )));
        }
        self.plan_active.insert(active.clone());
        let made = self.make_instances(m, mv, cut, Some(upto));
        self.plan_active.remove(&active);
        let mut out = made?;
        // kept before their status, as the whole list is
        self.prefixes
            .plans
            .insert(key.clone(), Rc::new(out.clone()));
        let mut due = false;
        for inst in &mut out {
            inst.status = self.status(m, mv, inst, cut)?;
            due = due || crate::book::hole_booked_before(inst, cut);
        }
        let shared = Rc::new(out);
        if !due {
            self.prefixes.plans.insert(key, Rc::clone(&shared));
            return Ok(shared);
        }
        self.prefixes.plans.insert(key.clone(), Rc::clone(&shared));
        // a hole booking a later version answers is that version's here too, measured over
        // this list (E §5 step 1); a stop other than a refusal leaves no list kept that it
        // didn't answer
        if let Err(stop) = self.answer_hole_bookings(m, mv, cut) {
            self.prefixes.plans.remove(&key);
            return Err(stop);
        }
        Ok(self.prefixes.plans.get(&key).map_or(shared, Rc::clone))
    }

    /// An instance's status at cut `cut`, the observations that fulfil it judged first
    /// (refcore's `status`). E §4.1 step 3, L §5.19.
    pub(crate) fn status(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &mut Inst,
        cut: Day,
    ) -> Result<Status, Stop> {
        inst.obs = self.obs_links_for(m, &inst.id, cut)?;
        // an observation dated on K or later is seen but not applied, and nor is the link it
        // states, so the instance it fulfils stays in the plan (L §4.5, §5.19 Coverage; tally
        // 2.1.8 R12, #182)
        let applied = self.links_applied(m, &inst.obs, cut)?;
        // a window's first day and its last (refcore's `day_lo`, `day_hi`)
        let (day, last) = match &inst.day {
            Some(dated) => (dated.lo(), dated.hi()),
            None => (None, None),
        };
        inst.depends = None;
        if inst.exists == Exists::No {
            return Ok(Status::None);
        }
        if let (Some(d), Some(hi)) = (day, last)
            && (hi < m.start || d > m.end)
        {
            return Ok(Status::LeftOut);
        }
        if scenario_early(m, mv, inst) {
            return Ok(Status::LeftOut);
        }
        if inst.err.is_some() {
            return Ok(Status::Refused);
        }
        if applied && !mv.decision {
            return Ok(Status::Modelled);
        }
        let Some(d) = day else {
            return Ok(Status::Hole);
        };
        if matches!(inst.exists, Exists::Unknown(_)) {
            return Ok(Status::Hole);
        }
        if !mv.decision
            && let Some(face) = self.class_depends(m, inst, cut)?
        {
            // booked on some of its window's days and not on the rest: a hole movement from
            // the window's first day, its day kept (E §4.1 step 2; tally 2.1.8 item 77)
            inst.depends = Some(face);
            return Ok(Status::Hole);
        }
        if mv.decision {
            return Ok(Status::Decision);
        }
        if kind_certainty(m, &mv.kind) == Some(Certainty::Uncertain) && !applied {
            return Ok(Status::Uncertain);
        }
        if d < cut && self.touches_covered(m, inst, (cut, None))? {
            return Ok(Status::Modelled);
        }
        Ok(if d < cut { Status::Past } else { Status::Plan })
    }

    /// Whether plan instance `inst`, dated by a window, takes its class one way on some of
    /// the window's days and another on the rest, at cut `cut`: inside the run or before its
    /// start, and modelled past or booked, each day of the window asked (refcore's
    /// `window_class_depends`). E §4.1 steps 2–3, L §4.4; tally 2.1.8 item 77.
    ///
    /// # Errors
    ///
    /// As [`Engine::touches_covered`]; `ENGINE-INVARIANT` for a day past the calendar.
    fn class_depends(
        &mut self,
        m: &Reality,
        inst: &Inst,
        cut: Day,
    ) -> Result<Option<Depends>, Stop> {
        let Some(Dated::Window(w)) = &inst.day else {
            return Ok(None);
        };
        if w.lo == w.hi {
            return Ok(None);
        }
        if w.lo < m.start && m.start <= w.hi {
            return Ok(Some(Depends::Start));
        }
        if w.lo >= cut {
            return Ok(None);
        }
        // every day asked: coverage may change inside the window (E §4.1 step 2). No day from
        // K on is modelled, and the answer is known at the first two days that disagree, so
        // the days asked stop there: at most the days from the window's first to K, however
        // far the window reaches (to 9999-12-31)
        let (mut some, mut every) = (false, true);
        let mut d = w.lo;
        loop {
            if d >= cut {
                every = false;
                break;
            }
            let modelled = self.touches_covered(m, inst, (cut, Some(d)))?;
            some = some || modelled;
            every = every && modelled;
            if (some && !every) || d >= w.hi {
                break;
            }
            d = d
                .next()
                .ok_or_else(|| invariant("a window's day past the calendar"))?;
        }
        Ok((some && !every).then_some(Depends::Covered))
    }

    /// Whether one of the observations `obs` that fulfil an instance is applied at cut `cut`:
    /// dated before it (refcore's `m_status`, its `applied`). L §4.5 (what a day applies is
    /// applied only when dated before K; tally 2.1.8 R12), §5.19, E §4.1 step 3.
    ///
    /// # Errors
    ///
    /// As [`Engine::observations`].
    fn links_applied(&mut self, m: &Reality, obs: &[String], cut: Day) -> Result<bool, Stop> {
        if obs.is_empty() {
            return Ok(false);
        }
        Ok(self
            .observations(m, cut)?
            .iter()
            .any(|o| o.day < cut && obs.contains(&o.id)))
    }

    /// Every instance of a move at its binding tuples, sorted (refcore's `insts_of_tuples`).
    /// E §4.1 steps 1–2.
    pub(crate) fn make_instances(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        cut: Day,
        upto: Option<Day>,
    ) -> Result<Vec<Inst>, Stop> {
        let outer = self.holding(Node::new(NodeKind::Move, &mv.id));
        let mark = self.reads_apart();
        let made = self.make_instances_held(m, mv, cut, upto);
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
        upto: Option<Day>,
    ) -> Result<Vec<Inst>, Stop> {
        let tctx = self.timeline_ctx(None, cut);
        let mut out = Vec::new();
        for tuple in self.move_tuples(m, mv, tctx)? {
            if let Some(err) = tuple.refused.clone() {
                out.extend(self.refused_tuple(m, mv, (tuple, upto), err, tctx)?);
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
                upto,
            };
            if let Some(mut inst) = self.eval_inst(m, mv, &seed, tctx)? {
                if !mv.decision
                    && inst.err.is_none()
                    && let Some(booking) = inst.cut
                {
                    self.check_booked(mv, &inst, (booking, cut), upto)?;
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
        (tuple, upto): (Tuple, Option<Day>),
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
            upto,
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

    /// A booked plan amount never changes: an instance's A₀, its existence and legs as read at
    /// its booking cut, is the same under every cut that books it at booking cut `booking`,
    /// across the asks of a run (refcore's `inv5`). Its day is read at the cut K, so a leg's
    /// day counts only where the leg states its own (E §4.1 step 2). E §9 item 5.
    fn check_booked(
        &mut self,
        mv: &MoveInfo,
        inst: &Inst,
        (booking, cut): (Day, Day),
        upto: Option<Day>,
    ) -> Result<(), Stop> {
        let key = (inst.id.clone(), booking, upto);
        let legs = booked_legs(mv, &inst.legs);
        let Some(old) = self.booked.get(&key) else {
            let first = Booked {
                exists: inst.exists.clone(),
                legs,
                cut,
            };
            self.booked.insert(key, first);
            return Ok(());
        };
        if old.exists == inst.exists && old.legs == legs {
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
                self.refused_legs(m, mv, &mut inst, (tctx, seed.at_cut))
                    .map(|()| true)
            }
            made => made,
        };
        let met = self.pop_alone();
        inst.coll.merge(&met);
        Ok(made?.then_some(inst))
    }

    /// A refused plan instance's legs, its evaluation stopped before them: their places read
    /// at the cut its body is measured at (its booking cut, or a later version's for a
    /// re-measure, where refcore reads min(t, K)) and its law date, each leg's amount the refusal; none where a place
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
        (tctx, at_cut): (Ctx, Option<Day>),
    ) -> Result<(), Stop> {
        if mv.decision {
            return Ok(());
        }
        let day = inst.day.as_ref().and_then(Dated::lo);
        // the cut its body is measured at: a later version's, for a re-measure (E §5 step 1),
        // else min(t, K)
        let cut = match (at_cut, day) {
            (Some(c), _) => c,
            (None, Some(d)) => d.min(tctx.cut),
            (None, None) => tctx.cut,
        };
        if let Some(legs) =
            self.refused_places(m, mv, inst, self.timeline_ctx(inst.law.first(), cut))?
        {
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
        let Some(places) = self.leg_ends(m, mv, inst, ctx)? else {
            return Ok(None);
        };
        let legs = mv
            .decl
            .legs
            .iter()
            .zip(places)
            .map(|(leg, (from, to))| LegVal {
                name: leg.name.clone(),
                from: End::Place(from),
                to: End::Place(to),
                amount: Amount::Refused(err.clone()),
                unit: None,
                day: day.clone(),
                src: None,
            })
            .collect();
        Ok(Some(legs))
    }

    /// The places each leg of instance `inst` gives, read at `ctx`; none where a place isn't
    /// a thing here, a hole or a refusal (refcore's `refused_places`). L §8.4, §8.2.
    ///
    /// # Errors
    ///
    /// A load refusal met at run.
    fn leg_ends(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &Inst,
        ctx: Ctx,
    ) -> Result<Option<Vec<(String, String)>>, Stop> {
        let mut out = Vec::with_capacity(mv.decl.legs.len());
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
            out.push((from, to));
        }
        Ok(Some(out))
    }

    /// The places, each with its unit, where the reads that decided instance `inst` doesn't
    /// exist (a false `when` or key `where`) join a balance's read set from its day: those
    /// its legs give, read at its booking cut (refcore's `note_none`). L §8.2 One read set
    /// (tally 2.1.7 R7, T's SR-8).
    ///
    /// # Errors
    ///
    /// A load refusal met at run.
    pub(super) fn decided_places(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &Inst,
        cut: Day,
    ) -> Result<Vec<(String, String)>, Stop> {
        let mut ends: Vec<(String, Option<String>)> = Vec::new();
        if inst.legs.is_empty() {
            let day = inst.day.as_ref().and_then(Dated::lo);
            let ctx = self.timeline_ctx(inst.law.first(), day.map_or(cut, |d| d.min(cut)));
            let Some(places) = self.leg_ends(m, mv, inst, ctx)? else {
                return Ok(Vec::new());
            };
            for (from, to) in places {
                ends.push((from, None));
                ends.push((to, None));
            }
        }
        for lg in &inst.legs {
            for place in [&lg.from, &lg.to].into_iter().filter_map(End::place) {
                ends.push((place.to_owned(), lg.unit.clone()));
            }
        }
        let mut out = Vec::new();
        for (place, unit) in ends {
            let unit = match unit {
                Some(unit) => unit,
                None => m
                    .things
                    .get(&place)
                    .and_then(|thing| m.place_unit(thing))
                    .unwrap_or_else(|| super::state::ALL.to_owned()),
            };
            out.push((place, unit));
        }
        Ok(out)
    }

    /// The value of `e` at `ctx`, or none where it is refused at run (refcore's `try: ev …
    /// except TallyError`); a load refusal met at run and `ENGINE-INVARIANT` go on (R-66).
    /// L §8.4.
    pub(super) fn read_or_refused(
        &mut self,
        m: &Reality,
        e: &Expr,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Option<Value>, Stop> {
        match self.ev(m, e, env, ctx) {
            Ok(v) => Ok(Some(v)),
            Err(Stop::Refused(r)) if super::order::run_time(&r) => Ok(None),
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
            law @ (Law::Day(_) | Law::Window(..) | Law::None) => law,
        };
        inst.law = law.clone();
        self.push();
        let v = self.day_over_law(m, mv, inst, tctx);
        let day_coll = self.pop_alone();
        self.absorb(&day_coll);
        inst.day_coll = day_coll;
        let v = v?;
        if matches!(v, Value::Never | Value::None) {
            return Ok(false);
        }
        let day = match v {
            Value::Hole(h) => Dated::Hole(h),
            // a window no `windows each` chose a day of (L §4.4)
            Value::Window(w) => match day_window(m, &w)? {
                Some(day) => day,
                None => Dated::Day(date_of(&Value::Window(w))?),
            },
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
        if let (Some(upto), Some(d)) = (seed.upto, day.lo())
            && d > upto
        {
            // a walk up to a day books nothing after it, nor a movement whose window's
            // first day falls after it (refcore's `eval_inst`, `upto`; L §4.4)
            return Ok(false);
        }
        // the booking cut min(t, K); a window's at its first day and its last (refcore's
        // `cuts`)
        let (cut, last) = match (seed.at_cut, day.lo(), day.hi()) {
            (Some(c), _, _) => (c, c),
            (None, Some(lo), Some(hi)) => (lo.min(tctx.cut), hi.min(tctx.cut)),
            (None, _, _) => (tctx.cut, tctx.cut),
        };
        inst.cut = Some(cut);
        let world = self.timeline_world();
        let first = keys_seen(m, mv, &inst.keys, (cut, world));
        let seen = if last == cut {
            (first, first)
        } else {
            (first, keys_seen(m, mv, &inst.keys, (last, world)))
        };
        if seen == (false, false) {
            inst.exists = Exists::No;
            return Ok(true);
        }
        // at a booking cut that doesn't see its keys the instance has no body (E §5 steps 1–3)
        let mut body = if seen.0 {
            self.body_over_law(m, mv, inst, self.timeline_ctx(None, cut))?
        } else {
            no_body()
        };
        if let (Dated::Window(w), true) = (&day, last != cut) {
            // a version known inside the window of a windowed instance (L §5.14.3): each
            // part is the window's hole where the two booking cuts differ, so a key added or
            // dropped inside the window leaves its existence and its legs to the day
            // (refcore's `agreed_body`; L §4.4, tally 2.1.8 #169-e)
            let text = format!(
                "date within {} not precise enough for the booking cut of {}",
                w.question, inst.id
            );
            let hole = Hole::asked(&text, &inst.id).carrying(&w.value.carry);
            let other = if seen.1 {
                self.body_over_law(m, mv, inst, self.timeline_ctx(None, last))?
            } else {
                no_body()
            };
            if !same_body(&body, &other) {
                body = imprecise_body(body, &other, &hole);
            }
        }
        inst.exists = body.exists;
        inst.legs = body.legs;
        inst.attrs = body.attrs;
        inst.for_period = body.for_period;
        inst.fulfils = body.fulfils;
        Ok(true)
    }

    /// An instance's day: its `on` read at its law date, and under a window `at` at each law
    /// date the window can take, `HOLE-IMPRECISE` where they differ (refcore's `eval_inst`,
    /// its `lds`). L §4.3, §5.14 `at e`.
    fn day_over_law(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &Inst,
        tctx: Ctx,
    ) -> Result<Value, Stop> {
        let (Some(window), Some(day)) = (inst.law.window(), inst.law.first()) else {
            return self.ev(m, &mv.decl.on, &inst.env, tctx.at_law(inst.law.first()));
        };
        // once per law-date class: a day its class holds gives what it gave (L §4.4's cost)
        let (first, mut class) = self.on_at(m, mv, inst, (tctx, day))?;
        for at in inst.law.rest() {
            if class.holds(*at) {
                continue;
            }
            let (other, next) = self.on_at(m, mv, inst, (tctx, *at))?;
            if other != first {
                return Ok(Value::Hole(imprecise_law(window, &inst.id)));
            }
            class = next;
        }
        Ok(first)
    }

    /// An instance's `on` read at law date `day`, with the law dates that give the same
    /// (E §3.2). L §4.3, §5.14 `at e`.
    fn on_at(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &Inst,
        (tctx, day): (Ctx, Day),
    ) -> Result<(Value, LawClass), Stop> {
        let mark = self.open_law_class(day);
        let value = self.ev(m, &mv.decl.on, &inst.env, tctx.at_law(Some(day)));
        let class = self.close_law_class(mark, day);
        Ok((value?, class?))
    }

    /// An instance's body at its law date in context `ctx`, its cut set, from its tuple's
    /// existence; under a window `at`, at each law date the window can take: one body where
    /// all agree, else each leg's amount `HOLE-IMPRECISE`, and its existence where that
    /// differs too (refcore's `body_over_law`). L §4.3, §5.14 `at e`, E §5 step 1.
    ///
    /// # Errors
    ///
    /// The refusals of its parts.
    pub(super) fn body_over_law(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &Inst,
        ctx: Ctx,
    ) -> Result<Body, Stop> {
        let exists = inst.exists.clone();
        let (Some(window), Some(day)) = (inst.law.window(), inst.law.first()) else {
            return self.inst_body(m, mv, inst, ctx.at_law(inst.law.first()), exists);
        };
        // once per law-date class: a day its class holds gives what it gave (L §4.4's cost)
        let (first, mut class) = self.body_at(m, mv, inst, (ctx, day))?;
        for at in inst.law.rest() {
            if class.holds(*at) {
                continue;
            }
            let (other, next) = self.body_at(m, mv, inst, (ctx, *at))?;
            if !same_body(&other, &first) {
                return Ok(imprecise_body(
                    first,
                    &other,
                    &imprecise_law(window, &inst.id),
                ));
            }
            class = next;
        }
        Ok(first)
    }

    /// An instance's body at law date `day`, with the law dates that give the same (E §3.2).
    /// L §4.3, §5.14 `at e`.
    fn body_at(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &Inst,
        (ctx, day): (Ctx, Day),
    ) -> Result<(Body, LawClass), Stop> {
        let mark = self.open_law_class(day);
        let body = self.inst_body(m, mv, inst, ctx.at_law(Some(day)), inst.exists.clone());
        let class = self.close_law_class(mark, day);
        Ok((body?, class?))
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

/// The body of an instance at a booking cut that doesn't see its keys: it doesn't exist, and
/// moves, states and fulfils nothing (refcore's `(False, [], {}, None, None)`). E §5 steps
/// 1–3.
fn no_body() -> Body {
    Body {
        exists: Exists::No,
        legs: Vec::new(),
        attrs: Vec::new(),
        for_period: None,
        fulfils: None,
    }
}

/// Whether two bodies are the same, read at two law dates or two cuts: their existence, each
/// leg's name, places, amount and day, their attributes, `for` and `fulfils` (refcore's
/// `body_key`, its legs). L §4.3, E §5 step 1.
fn same_body(a: &Body, b: &Body) -> bool {
    a.exists == b.exists
        && same_legs(&a.legs, &b.legs)
        && a.attrs == b.attrs
        && a.for_period == b.for_period
        && a.fulfils == b.fulfils
}

/// Whether two bodies' legs are the same: each leg's name, places, amount and day (refcore's
/// `body_key`). L §4.3.
fn same_legs(a: &[LegVal], b: &[LegVal]) -> bool {
    a.len() == b.len()
        && a.iter().zip(b).all(|(x, y)| {
            x.name == y.name
                && x.from == y.from
                && x.to == y.to
                && x.amount == y.amount
                && x.day == y.day
        })
}

/// The body of a window `at` whose law dates give two bodies: the first's, each leg's amount
/// the hole `hole` (the second's legs where the first has none) unless only attributes, `for`
/// or `fulfils` differ, its existence the hole too where the two differ, and each attribute,
/// `for` and `fulfils` the two give differently the hole (refcore's `body_over_law`). L §4.3,
/// §8.1, E §5 step 1.
fn imprecise_body(first: Body, other: &Body, hole: &Hole) -> Body {
    let legs = if first.exists == other.exists && same_legs(&first.legs, &other.legs) {
        first.legs
    } else {
        let src = if first.legs.is_empty() {
            &other.legs
        } else {
            &first.legs
        };
        src.iter()
            .map(|lg| LegVal {
                amount: Amount::Hole(hole.clone()),
                src: None,
                ..lg.clone()
            })
            .collect()
    };
    let exists = if first.exists == other.exists {
        first.exists
    } else {
        Exists::Unknown(hole.clone())
    };
    let mut attrs = first.attrs;
    for (name, value) in &mut attrs {
        let theirs = other.attrs.iter().find(|(n, _)| n == name).map(|(_, v)| v);
        if theirs != Some(&*value) {
            *value = Value::Hole(hole.clone());
        }
    }
    Body {
        exists,
        legs,
        attrs,
        for_period: imprecise_part(first.for_period, other.for_period.as_ref(), hole),
        fulfils: imprecise_part(first.fulfils, other.fulfils.as_ref(), hole),
    }
}

/// A body's `for` or `fulfils` read two ways: the first where both agree, else the hole
/// `hole` (L §4.3, §8.1).
fn imprecise_part(first: Option<Value>, other: Option<&Value>, hole: &Hole) -> Option<Value> {
    if first.as_ref() == other {
        first
    } else {
        Some(Value::Hole(hole.clone()))
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

/// A₀'s legs: each leg as read at its booking cut, its day kept only where it states its own
/// `on`, since one it takes from its instance is read at the cut K (refcore's `body_key`).
/// E §4.1 step 2, E §9 item 5.
fn booked_legs(mv: &MoveInfo, legs: &[LegVal]) -> Vec<BookedLeg> {
    let mut out = Vec::with_capacity(legs.len());
    for lg in legs {
        let own = mv
            .decl
            .legs
            .iter()
            .any(|decl| decl.name == lg.name && decl.on.is_some());
        out.push(BookedLeg {
            name: lg.name.clone(),
            from: lg.from.clone(),
            to: lg.to.clone(),
            amount: lg.amount.clone(),
            unit: lg.unit.clone(),
            own_day: own.then(|| lg.day.clone()),
        });
    }
    out
}

/// Whether `inst` is an instance of a move a scenario declares whose day, or a leg's own day,
/// falls before the scenario's `from`: it books nothing there, so it is left out; load has
/// refused the move where it sees that day and that the instance exists (refcore's
/// `scen_early`). L §6.4.
fn scenario_early(m: &Reality, mv: &MoveInfo, inst: &Inst) -> bool {
    let Some(from) = scenario_from(m, &mv.id) else {
        return false;
    };
    // a window's first day (refcore's `day_lo`)
    let legs = inst.legs.iter().filter_map(|leg| leg.day.lo());
    let day = inst.day.as_ref().and_then(Dated::lo);
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
        // a window's first day (refcore's `inst_order`, `day_lo`)
        day: inst.day.as_ref().and_then(Dated::lo),
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

/// A check of load's met at run, on a value load couldn't see: `RUN-*` in load's words at
/// the value, and the run goes on (L §8.4 R-load, tally 2.1.7 R5; refcore's `moves.py`
/// `refuse`, its `at_run`).
pub(super) fn refuse_load(code: Load, message: String) -> Stop {
    Stop::Refused(Refusal::new(code.at_run(), message))
}

/// A broken promise: `ENGINE-INVARIANT` (E §11).
pub(crate) fn invariant(what: &str) -> Stop {
    Stop::Refused(Refusal::new(Code::EngineInvariant, what.to_owned()))
}

/// The hole a window-dated instance whose class depends on the window's day puts to a
/// balance its legs reach: the window's, asked of the instance (refcore's `class_q`;
/// L §4.4, E §4.1 step 2; tally 2.1.8 item 77).
#[must_use]
pub(crate) fn class_hole(inst: &Inst, w: &DayWindow) -> Hole {
    let text = format!(
        "date within {} not precise enough for a balance",
        w.question
    );
    Hole::asked(&text, &inst.id).carrying(&w.value.carry)
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
    fn a_unit_fault_met_reading_a_refused_plans_place_is_refused_at_that_value() {
        // L §3.3, §8.4 (tally 2.1.7 R5): a unit check met at run, here reading the place of
        // February's refused instance, is refused at that value, `RUN-*`, not the run: as
        // its twin above, every place refused from 20 February (refcore's bytes)
        assert_eq!(
            answer(&[("loadrun.tally", PLACE_LOAD_REFUSAL)], "q_bal"),
            answer(&[("place.tally", REFUSED_PLACE)], "q_bal"),
            "every place refused from 20 February"
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
