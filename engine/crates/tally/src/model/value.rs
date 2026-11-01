//! Runtime values: what a row states and an expression gives, with the class each number carries, and how a value prints.
//! Spec: L §3.1 (the types and the three absences: a hole, `none`, `never`), §3.2 (units, money
//! as a subtype of `num CUR`, inline figures), §3.3 (rates), §2.4 and §5.18 (how values print).
//! Twin: engine/tools/refcore/values.py (`Num` with its `money`, `isint` and `rate` marks,
//! `Key`, `Thing`, `Code`, `Rec`, `Lst`, `Special`, `Cal`, `fmt`, `fmt_typed`, `same_value`)
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
}

impl Value {
    /// The integer `n`.
    #[must_use]
    pub fn int(n: i64) -> Value {
        Value::Number(Number::int(n))
    }

    /// A key as a value (refcore's `keyval`): an integer key is an int, a day key a day, a
    /// period key a period (L §4.2 Keys as values).
    #[must_use]
    pub fn of_key(key: &KeyVal) -> Value {
        match key {
            KeyVal::Int(n) => Value::int(*n),
            KeyVal::Day(d) => Value::Day(*d),
            KeyVal::Period(p) => Value::Period(*p),
            KeyVal::Key { .. } | KeyVal::Thing(_) => Value::Key(key.clone()),
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
            Value::Calendar(cal) => Ok(cal.text()),
            Value::None => Ok("none".to_owned()),
            Value::Never => Ok("never".to_owned()),
            Value::Hole(h) => Ok(h.text()),
            Value::OutKey { text, .. } => Ok(text.clone()),
            Value::Word(word) => Ok(word.clone()),
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
/// (L §3.1, `LOAD-PRECISION` where load sees it).
///
/// # Errors
///
/// `Domain` for a fraction of the minor unit; as [`currency`].
pub fn num_money(m: &Reality, q: &Num, cur: Currency) -> Result<Money, NumError> {
    let (_, minor) = currency(m, cur)?;
    let units = q.checked_div(minor)?;
    if !units.is_integer() {
        return Err(NumError::Domain);
    }
    units.to_money(cur)
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
