//! Runtime values: what a row states and an expression gives, with the class each number carries, and how a value prints.
//! Spec: L §3.1 (the types and the three absences: a hole, `none`, `never`), §3.2 (units, money
//! as a subtype of `num CUR`, inline figures), §3.3 (rates), §2.4 and §5.18 (how values print).
//! Twin: engine/tools/refcore/values.py (`Num` with its `money`, `isint` and `rate` marks,
//! `Key`, `Thing`, `Mv`, `Code`, `Rec`, `Lst`, `Special`, `Cal`, `fmt`, `fmt_typed`,
//! `same_value`)
//! and evaluate.py (`OutKey`).
//! Invariants: a number's class is the static type refcore's checker gives the expression that
//! made it (check.py `_arith`), so the arithmetic decides it from its operands' classes; money
//! is a whole number of minor units; a hole is built only in the hole module (R-116).

use super::key::KeyVal;
use super::reality::Reality;
use super::ty::Shown;
use super::unit::Unit;
use crate::hole::Hole;
use crate::num::{Currency, Money, Num, NumError};
use crate::time::{Calendar, Day, Period};

/// The class of a number that isn't money: what refcore's static type says it is (L §3.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Class {
    /// `int`.
    Int,
    /// `num u`, shown plainly or with `%` (`num %`).
    Num(Unit, Shown),
    /// `rate`, with its period in its unit (`1/year`).
    Rate(Unit),
    /// A bare figure whose type its context decides (L §3.2 Inline figures): a bare `0`, `1`
    /// or `-1` in a def, any bare literal in a band row.
    Flex,
}

/// A number that isn't money: its exact value and its class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Number {
    /// The value.
    pub q: Num,
    /// Its class.
    pub class: Class,
}

impl Number {
    /// The integer `n`.
    #[must_use]
    pub fn int(n: i64) -> Number {
        Number {
            q: Num::int(n),
            class: Class::Int,
        }
    }

    /// The plain number `q` of unit `unit`.
    #[must_use]
    pub fn plain(q: Num, unit: Unit) -> Number {
        Number {
            q,
            class: Class::Num(unit, Shown::Plain),
        }
    }

    /// Its unit: none for an int or a bare figure (L §3.2).
    #[must_use]
    pub fn unit(&self) -> Unit {
        match &self.class {
            Class::Num(u, _) | Class::Rate(u) => u.clone(),
            Class::Int | Class::Flex => Unit::one(),
        }
    }
}

/// A code value: a word of a code type, or `nominal n` (L §3.1, §3.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Code {
    /// A code word: `half-up`, `act365`, `following`, `effective`.
    Word(String),
    /// `nominal n`.
    Nominal(i64),
}

/// A runtime value (L §3.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// A number that isn't money.
    Number(Number),
    /// Money: whole minor units of a currency.
    Money(Money),
    /// `true` or `false`.
    Bool(bool),
    /// A day.
    Day(Day),
    /// A period: a month, a quarter, a year, a day period.
    Period(Period),
    /// A month-day, `04-30`.
    MonthDay(u8, u8),
    /// A key of a list or open index, or a thing.
    Key(KeyVal),
    /// A text.
    Text(String),
    /// A code.
    Code(Code),
    /// A record of named fields, in its type's order.
    Record(Vec<(String, Value)>),
    /// A list.
    List(Vec<Value>),
    /// The keys an `argmin` or `argmax` over several bindings returns, one per binding in
    /// written order, `(k1, 1)`: printed, never compared or keyed, as load's types refuse
    /// (L §5.11 "the key or the tuple").
    Tuple(Vec<Value>),
    /// A business calendar, with the input it was read from.
    Calendar(Box<Calendar>),
    /// `none`: not applicable.
    None,
    /// `never`: later than every date.
    Never,
    /// A hole.
    Hole(Hole),
    /// A key literal not yet resolved: the index or kind of its position, or of the value it
    /// meets, decides which key it is (L §2.5).
    Word(String),
    /// A key arithmetic left outside its index: `RUN-KEY` where it stands as a key.
    OutKey {
        /// The index.
        index: String,
        /// The arithmetic as written, `low + 3`.
        text: String,
    },
    /// A movement: a move's instance, an observation, an adjustment, by its stable id; a
    /// movement is a thing of its event kind (L §5.14, §2.5).
    Mv(Box<MvRef>),
    /// A day not yet known, `window(a, b)`, or the stretch of days a listed reader gives of one
    /// (L §4.4).
    Window(Box<Window>),
}

/// A window: a stretch of dates and nothing else, the days a day not yet known can be, from
/// its first to its last (refcore's `Iv`). No number is a range. L §4.4.
#[derive(Debug, Clone)]
pub struct Window {
    /// The first day.
    pub lo: Day,
    /// The last day.
    pub hi: Day,
    /// The window as written, which a question names; none for a stretch no one window
    /// gives.
    pub src: Option<String>,
    /// The figure of the row it came from (`bought[l1]`), which its holes are asked of, kept
    /// through the operations on it (L §8.1).
    pub figure: Option<String>,
    /// The read of an item it is the value of; none for a value an operation computed. Two
    /// reads alike are one value, decided against itself (L §4.4).
    pub read: Option<ReadOf>,
    /// The unknown days it was computed from, in the order the line's reads met them: a read's
    /// own, an operation's operands' (L §4.4: "it carries the names it was computed from"),
    /// which a hole made of it carries too. No part of the value.
    pub carry: Vec<crate::hole::Unknown>,
}

impl PartialEq for Window {
    /// The same stretch, text, figure and read, whatever unknown days it carries (L §4.4).
    fn eq(&self, other: &Window) -> bool {
        self.lo == other.lo
            && self.hi == other.hi
            && self.src == other.src
            && self.figure == other.figure
            && self.read == other.read
    }
}

impl Eq for Window {}

impl Value {
    /// The unknown days the holes the value prints carry, added to `into` in order, each
    /// once: a hole's, and those of the holes a record or a list holds. A window prints as its
    /// stretch and gives none (refcore's `hole_days`). L §4.4, §8.2 `unknown day:`.
    pub fn hole_days_into(&self, into: &mut Vec<crate::hole::Unknown>) {
        let mut todo: Vec<&Value> = vec![self];
        while let Some(v) = todo.pop() {
            match v {
                Value::Hole(h) => crate::hole::carry_union(into, &h.carried()),
                Value::Record(fields) => todo.extend(fields.iter().rev().map(|(_, x)| x)),
                Value::List(items) | Value::Tuple(items) => todo.extend(items.iter().rev()),
                Value::Window(_)
                | Value::Number(_)
                | Value::Money(_)
                | Value::Bool(_)
                | Value::Day(_)
                | Value::Period(_)
                | Value::MonthDay(..)
                | Value::Key(_)
                | Value::Text(_)
                | Value::Code(_)
                | Value::Calendar(_)
                | Value::None
                | Value::Never
                | Value::Word(_)
                | Value::OutKey { .. }
                | Value::Mv(_) => {}
            }
        }
    }
}

/// One read of an item: the input or def, its keys and the cell it was read at. Two reads
/// of one item at the same keys, with the law date, cut and pins it depends on, are one
/// value, whatever variables bind the keys; equal windows from different reads stay
/// undecided (L §4.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadOf {
    /// The input or def read.
    pub item: String,
    /// Its keys, as made.
    pub keys: Vec<KeyVal>,
    /// The cell, as text: for a def, its cell's world (the law date, the cut, the pins the
    /// def's cone reaches, the walk point, `with uncertain`); for an input, the row its read
    /// rule took, which those decide (a pin of the input itself is its own value).
    pub cell: String,
}

impl Window {
    /// The window as a question names it: as written, else `window(lo, hi)` (refcore's
    /// `window_text`). L §4.4.
    #[must_use]
    pub fn question_text(&self) -> String {
        if let Some(src) = &self.src {
            return src.clone();
        }
        format!("window({}, {})", self.lo.text(), self.hi.text())
    }

    /// The figure a question about it is asked of: its row's, or none (L §8.1).
    #[must_use]
    pub fn figure(&self) -> &str {
        self.figure.as_ref().map_or("", String::as_str)
    }
}

/// A movement as a value: its stable id and what it is (refcore's `Mv`). L §5.14, §2.5.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MvRef {
    /// Its stable id: `rent_pay[2026-09]`, `bank[r17]`, `adjustments[spend[2026-03-01].cash, v2]`.
    pub id: String,
    /// Its day in the movement order, which places it as a key (refcore's `key_sort`).
    pub day: Option<Day>,
    /// What it is.
    pub src: MvSrc,
    /// Whether it is a binding's member of a decision whose day is a hole, whose legs the
    /// binding read as the plan has them (refcore's `hole_day_dec_body`, on the member's own
    /// record). L §5.19 Bindings.
    pub member: bool,
}

/// What a movement value is, found again at the reader's cut (refcore's `Mv.src`). L §5.14.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MvSrc {
    /// A move's instance: the move and its keys.
    Inst(String, Vec<KeyVal>),
    /// An observation, by its row's id.
    Obs,
    /// An adjustment: the adjusted instance's id and its move, the leg, the step's versions.
    Adj {
        /// The adjusted instance's id.
        inst: String,
        /// Its move.
        move_id: String,
        /// Its keys.
        keys: Vec<KeyVal>,
        /// The adjusted leg.
        leg: String,
        /// The step's versions.
        vids: Vec<String>,
    },
    /// A movement known by its id alone, found again at the reader's cut: a move's instance no
    /// binding tuple gives there, or a movement an observed row's attribute names (L §3.1).
    Missing,
    /// A pad the walk booked, by its id (E §6.1).
    Pad,
    /// A movement of an event kind as the type sampler sees one: never a run's value.
    Kind(String),
}

impl Value {
    /// The integer `n`.
    #[must_use]
    pub fn int(n: i64) -> Value {
        Value::Number(Number::int(n))
    }

    /// A key as a value (refcore's `keyval`): an integer key is an int, a day key a day, a
    /// period key a period, a stepped range's value that value (L §4.2 Keys as values, §2.5).
    #[must_use]
    pub fn of_key(key: &KeyVal) -> Value {
        match key {
            KeyVal::Int(n) => Value::int(*n),
            KeyVal::Day(d) => Value::Day(*d),
            KeyVal::Period(p) => Value::Period(*p),
            KeyVal::Key { .. } | KeyVal::Thing(_) => Value::Key(key.clone()),
            KeyVal::Mv(mv) => Value::Mv(mv.clone()),
            KeyVal::Value(v) => v.value.clone(),
        }
    }

    /// The hole, if it is one.
    #[must_use]
    pub fn as_hole(&self) -> Option<&Hole> {
        if let Value::Hole(h) = self {
            return Some(h);
        }
        None
    }

    /// The value as L §5.18 prints it (refcore's `fmt`).
    ///
    /// # Errors
    ///
    /// A number error, or `Invariant` for money in a currency load didn't declare.
    pub fn text(&self, m: &Reality) -> Result<String, NumError> {
        match self {
            Value::Number(n) => number_text(n),
            Value::Money(money) => money_text(m, money),
            Value::Bool(b) => Ok(if *b { "true" } else { "false" }.to_owned()),
            Value::Day(d) => Ok(d.text()),
            Value::Period(p) => Ok(p.label()),
            Value::MonthDay(month, day) => Ok(format!("{month:02}-{day:02}")),
            Value::Key(key) => Ok(key.text()),
            Value::Text(s) => Ok(format!(
                "\"{}\"",
                s.replace('\\', "\\\\").replace('"', "\\\"")
            )),
            Value::Code(Code::Word(w)) => Ok(w.clone()),
            Value::Code(Code::Nominal(n)) => Ok(format!("nominal {n}")),
            Value::Record(fields) => record_text(m, fields),
            Value::List(items) => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(item.text(m)?);
                }
                Ok(format!("[{}]", out.join(", ")))
            }
            Value::Tuple(items) => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(item.text(m)?);
                }
                Ok(format!("({})", out.join(", ")))
            }
            Value::Calendar(cal) => Ok(cal.text()),
            Value::None => Ok("none".to_owned()),
            Value::Never => Ok("never".to_owned()),
            Value::Hole(h) => Ok(h.text()),
            Value::OutKey { text, .. } => Ok(text.clone()),
            Value::Word(word) => Ok(word.clone()),
            Value::Mv(mv) => Ok(mv.id.clone()),
            Value::Window(w) => Ok(window_text(w)),
        }
    }

    /// The value as its static type prints it: a `num %` shows its per cent (refcore's
    /// `fmt_typed`). L §2.4, §3.2.
    ///
    /// # Errors
    ///
    /// As [`Value::text`].
    pub fn typed_text(&self, m: &Reality) -> Result<String, NumError> {
        if let Value::Number(Number {
            q,
            class: Class::Num(unit, Shown::Percent),
        }) = self
            && unit.is_one()
        {
            return Ok(format!("{} %", q.checked_mul(&Num::int(100))?.to_text()?));
        }
        self.text(m)
    }
}

/// A window as it prints: `window(lo, hi)`, its stretch (refcore's `fmt` of an `Iv`).
/// L §4.4, §5.18.
fn window_text(w: &Window) -> String {
    format!("window({}, {})", w.lo.text(), w.hi.text())
}

/// `(a: x, b: y)`.
fn record_text(m: &Reality, fields: &[(String, Value)]) -> Result<String, NumError> {
    let mut out = Vec::with_capacity(fields.len());
    for (name, value) in fields {
        out.push(format!("{name}: {}", value.text(m)?));
    }
    Ok(format!("({})", out.join(", ")))
}

/// A number as it prints: an int grouped, a rate with its period, a unit after (L §5.18).
fn number_text(n: &Number) -> Result<String, NumError> {
    match &n.class {
        Class::Int => n.q.to_fixed(0),
        Class::Flex if n.q.is_integer() => n.q.to_fixed(0),
        Class::Flex => n.q.to_text(),
        Class::Rate(unit) => {
            let period = unit.period_of().ok_or(NumError::Invariant)?;
            Ok(format!("{} /{period}", n.q.to_text()?))
        }
        Class::Num(unit, _) if unit.is_one() => n.q.to_text(),
        Class::Num(unit, _) => Ok(format!("{} {}", n.q.to_text()?, unit.text())),
    }
}

/// The name and minor unit of a currency, by its id: its place among the declared
/// currencies, in code point order (L §5.4).
///
/// # Errors
///
/// `Invariant` for an id load gave no currency.
pub fn currency(m: &Reality, cur: Currency) -> Result<(&str, &Num), NumError> {
    let at = usize::try_from(cur.id()).map_err(|_| NumError::Invariant)?;
    m.currencies
        .iter()
        .nth(at)
        .map(|(name, minor)| (name.as_str(), minor))
        .ok_or(NumError::Invariant)
}

/// The id of the currency `name`, its place among the declared currencies (L §5.4).
///
/// # Errors
///
/// `Invariant` for a currency load didn't declare.
pub fn currency_id(m: &Reality, name: &str) -> Result<Currency, NumError> {
    let at = m
        .currencies
        .keys()
        .position(|c| c == name)
        .ok_or(NumError::Invariant)?;
    let id = u32::try_from(at).map_err(|_| NumError::Invariant)?;
    Ok(Currency::new(id))
}

/// Money as it prints: its minor-unit digits and its currency, `6 064.17 PLN` (L §5.18).
fn money_text(m: &Reality, money: &Money) -> Result<String, NumError> {
    let (name, minor) = currency(m, money.currency())?;
    Ok(format!("{} {name}", money.to_text(minor)?))
}

/// Money as a number of its currency's units, `12.34` (L §3.2: money is a `num CUR`).
///
/// # Errors
///
/// As [`currency`].
pub fn money_num(m: &Reality, money: &Money) -> Result<Num, NumError> {
    let (_, minor) = currency(m, money.currency())?;
    money.to_num(minor)
}

/// `q` units of currency `cur` as money; `Domain` when it isn't a whole number of minor units
/// (L §3.1, `LOAD-PRECISION` where load sees it). The cap is `q`'s, counted on its reduced
/// fraction, never on its minor units (L §7.1).
///
/// # Errors
///
/// `Domain` for a fraction of the minor unit; as [`currency`].
pub fn num_money(m: &Reality, q: &Num, cur: Currency) -> Result<Money, NumError> {
    let (_, minor) = currency(m, cur)?;
    q.to_money_in(minor, cur)
}

/// `q` units of currency `cur` as money, exactly: whole minor units as [`num_money`] gives
/// them, else money holding a fraction of one, which only `solve`'s centre makes (L §5.18
/// step 3; pick P36).
///
/// # Errors
///
/// As [`currency`]; a number error.
pub fn num_money_part(m: &Reality, q: &Num, cur: Currency) -> Result<Money, NumError> {
    let (_, minor) = currency(m, cur)?;
    q.to_money_part(minor, cur)
}

/// The unit of a currency, `PLN` to the first power (L §3.2).
///
/// # Errors
///
/// As [`currency`].
pub fn currency_unit(m: &Reality, cur: Currency) -> Result<Unit, NumError> {
    let (name, _) = currency(m, cur)?;
    Unit::make(vec![(name.to_owned(), 1)])
}

#[cfg(test)]
mod tests {
    use super::{Class, Code, Number, Value};
    use crate::model::{KeyVal, Shown, Unit};
    use crate::num::Num;
    use crate::time::Period;

    #[test]
    fn keys_as_values() {
        assert_eq!(Value::of_key(&KeyVal::Int(3)), Value::int(3), "an int key");
        let p = Period::month(2026, 3).expect("a month");
        assert_eq!(
            Value::of_key(&KeyVal::Period(p)),
            Value::Period(p),
            "a period"
        );
        let n = Number::plain(Num::int(2), Unit::one());
        assert_eq!(n.class, Class::Num(Unit::one(), Shown::Plain), "plain");
        assert_eq!(Code::Nominal(12), Code::Nominal(12), "a code");
    }
}
