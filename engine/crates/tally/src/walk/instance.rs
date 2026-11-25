//! A move instance and its values: its day, legs, existence and status, its place in the
//! movement order, and the amount a booked plan gives again.
//! Spec: E §4.1 (the movement order: day, stratum, move id, keys in their domains' order; an
//! instance's status, step 3), E §9 item 5 (a booked plan's A₀), §6.3 Order; L §5.14 (moves:
//! keys, `on`, `when`, legs on their instance's day),
//! §8.1 (existence joined with a hole), §2.5.
//! Twin: engine/tools/refcore/moves.py (`Inst`, `Leg`, `inst_order`, `key_sort`, `body_key`);
//! refcore's `order_key`.
//! Invariants: two orders compare term by term, a hole day after every day; two legs are the
//! same leg wherever a hole amount was read; an instance's order is its day, its
//! move's stratum and id, then its keys in their domains' order.

use super::plan::invariant;
use super::tuples::Law;
use crate::base::Refusal;
use crate::eval::{Collected, Ctx, Env, order_text};
use crate::hole::{Hole, spread};
use crate::model::{Domain, KeyVal, MoveInfo, Reality, Value};
use crate::num::Num;
use crate::syntax::{Expr, Stop};
use crate::time::Day;
use core::cmp::Ordering;
use std::collections::BTreeMap;

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

    /// The day as a value: a date, or a hole (refcore's `mv_field`). L §5.14.
    #[must_use]
    pub fn value(&self) -> Value {
        match self {
            Dated::Day(d) => Value::Day(*d),
            Dated::Hole(h) => Value::Hole(h.clone()),
        }
    }

    /// The day as L §2.4 prints a value: a date in ISO, a hole as the hole (refcore's
    /// `day_text`). L §2.4 Values.
    #[must_use]
    pub fn text(&self) -> String {
        match self {
            Dated::Day(d) => d.text(),
            Dated::Hole(h) => h.text(),
        }
    }
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
    /// Its day: its instance's, a leg having no day of its own (tally 2.2, S-6), or the end of
    /// time for a refused instance with no day (`refused_places`, L §7.6 item 1). Kept on the
    /// leg because its readers (the day's acts, the ledger, `flows`, `same`, the members, fence
    /// F5) take the leg without its instance.
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

/// One leg of a booked A₀: its places, amount and unit. A leg is on its instance's day, which
/// is read at the ask's cut K, not at the booking cut, so a later K may move it (E §4.1 step
/// 2; refcore's `body_key`).
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
    /// Its law date by the move's `at`: a day, none, or a hole.
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
    /// What reading each leg met, by the leg's name (S3, P43 on #241).
    pub leg_reads: BTreeMap<String, Collected>,
    /// What decided whether it exists and its day, never its amounts: a binding's member's
    /// read set; none where its body wasn't read (L §8.2 l.2252; tally 2.3.1 pick P16).
    pub exist_coll: Option<Collected>,
    /// Its place in the movement order.
    pub order: Order,
    /// For a hole booking a later version answered: the cut that version is known from and
    /// the step's versions, whose evaluation its existence and legs are (E §5 step 1, §10).
    pub answered: Option<(Day, Vec<String>)>,
    /// Its tuple's existence, its keys' `where` read at the ask's cut K (E §4.1 step 1): a hole
    /// it leaves is no later version's to answer, so a re-measure starts from it (refcore's
    /// `exists0`; #236).
    pub exists0: Box<Exists>,
}

impl Inst {
    /// An instance of `mv` at `keys`, nothing evaluated yet. E §4.1.
    pub(super) fn new(mv: &MoveInfo, keys: Vec<KeyVal>, env: Env) -> Inst {
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
            leg_reads: BTreeMap::new(),
            exist_coll: None,
            order: Order {
                day: None,
                stratum: mv.stratum,
                move_id: mv.id.clone(),
                keys: Vec::new(),
            },
            answered: None,
            exists0: Box::new(Exists::Yes),
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

/// An instance's place in the movement order: its day, its move's stratum and id, its keys in
/// their domains' order (refcore's `inst_order`, `key_sort`). E §4.1.
pub(super) fn inst_order(m: &Reality, mv: &MoveInfo, inst: &Inst) -> Order {
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
