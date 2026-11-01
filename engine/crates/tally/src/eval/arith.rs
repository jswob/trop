//! Arithmetic and comparison on values: numbers with their classes and units, money, dates, periods and keys.
//! Spec: L §3.2 (the unit rules: `+ - * /`, money as a `num CUR`, inline figures), §3.3 (rate
//! arithmetic), §4.2 (date arithmetic, key arithmetic, differences of periods), §5.6 and §5.7
//! (keys and things: which have an order), §7.1 (exact numbers), §8.4 (what is refused per value).
//! Twin: engine/tools/refcore/evaluate.py (`arith`, `binop`, `key_arith`, `key_diff`,
//! `date_arith`, `cmp3`, `same_value`) and check.py (`_arith`, `arith_time`: the classes of
//! results, which the runtime takes from its operands here).
//! Invariants: holes, `none` and `never` are handled by the caller before these run; every
//! integer step is checked; a mismatch of units is refused, never converted silently.

use core::cmp::Ordering;

use super::state::{invariant, num_stop, refuse, refuse_load};
use crate::base::{Load, Run};
use crate::model::{
    Class, KeyVal, Number, Reality, Shown, Unit, Value, currency_id, currency_unit, money_num,
    num_money,
};
use crate::num::{Currency, Num};
use crate::syntax::Stop;
use crate::time::{Day, Period, PeriodKind};

/// An operator of arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// `+`.
    Add,
    /// `-`.
    Sub,
    /// `*`.
    Mul,
    /// `/`.
    Div,
}

impl Op {
    /// The operator as written.
    fn text(self) -> &'static str {
        match self {
            Op::Add => "+",
            Op::Sub => "-",
            Op::Mul => "*",
            Op::Div => "/",
        }
    }
}

/// A number's class with money among them: the static types refcore's `_arith` reads.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Kind {
    /// `money CUR`.
    Money(Currency),
    /// Any other number.
    Plain(Class),
}

/// A numeric operand: its value in its own unit and its class.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Operand {
    /// The value.
    q: Num,
    /// The class.
    kind: Kind,
}

impl Operand {
    /// The operand's unit: a currency's, or its class's (L §3.2).
    fn unit(&self, m: &Reality) -> Result<Unit, Stop> {
        match &self.kind {
            Kind::Money(cur) => currency_unit(m, *cur).map_err(num_stop),
            Kind::Plain(Class::Num(u, _) | Class::Rate(u)) => Ok(u.clone()),
            Kind::Plain(Class::Int | Class::Flex) => Ok(Unit::one()),
        }
    }

    /// Whether it is a bare figure that is an integer (refcore's `_intflex`).
    fn int_flex(&self) -> bool {
        self.kind == Kind::Plain(Class::Flex) && self.q.is_integer()
    }
}

/// The value of a number or money as an operand, or `None` for any other value.
fn operand(m: &Reality, v: &Value) -> Result<Option<Operand>, Stop> {
    if let Value::Money(money) = v {
        return Ok(Some(Operand {
            q: money_num(m, money).map_err(num_stop)?,
            kind: Kind::Money(money.currency()),
        }));
    }
    if let Value::Number(n) = v {
        return Ok(Some(Operand {
            q: n.q.clone(),
            kind: Kind::Plain(n.class.clone()),
        }));
    }
    Ok(None)
}

/// An operand of class `kind` and value `q` as a value: money in its minor units.
fn finish(m: &Reality, q: Num, kind: Kind) -> Result<Value, Stop> {
    match kind {
        Kind::Money(cur) => Ok(Value::Money(num_money(m, &q, cur).map_err(num_stop)?)),
        Kind::Plain(class) => Ok(Value::Number(Number { q, class })),
    }
}

/// `a op b` as L §8.4's refusal names it, `5 + true`.
fn shown(m: &Reality, a: &Value, op: Op, b: &Value) -> Result<String, Stop> {
    Ok(format!(
        "{} {} {}",
        a.text(m).map_err(num_stop)?,
        op.text(),
        b.text(m).map_err(num_stop)?
    ))
}

/// `a op b` on two values that are neither holes, `none` nor `never`: numbers and money by
/// the unit rules, dates, periods and keys by date and key arithmetic (L §3.2, §4.2).
///
/// # Errors
///
/// The refusal of L §8.4 the operation meets (`RUN-DIV`, `RUN-NOT-INT`, `RUN-KEY`,
/// `RUN-DOMAIN`, `RUN-DIGITS`), or the unit refusal refcore raises (`LOAD-UNIT`).
pub fn arith(m: &Reality, op: Op, lhs: &Value, rhs: &Value) -> Result<Value, Stop> {
    if matches!(lhs, Value::Day(_) | Value::Period(_) | Value::Key(_))
        || matches!(rhs, Value::Day(_) | Value::Period(_) | Value::Key(_))
    {
        return time_arith(m, op, lhs, rhs);
    }
    let (Some(x), Some(y)) = (operand(m, lhs)?, operand(m, rhs)?) else {
        return Err(refuse_load(Load::Unit, shown(m, lhs, op, rhs)?));
    };
    let out = match op {
        Op::Add | Op::Sub => add_sub(m, op, x, y)?,
        Op::Mul | Op::Div => mul_div(m, op, x, y)?,
    };
    if let Value::Number(n) = &out
        && n.unit().day_and_period()
    {
        return Err(refuse_load(
            Load::Unit,
            format!(
                "{}: day is lhs dimension of its own (1 day / 1 year is refused, L §3.2)",
                shown(m, lhs, op, rhs)?
            ),
        ));
    }
    Ok(out)
}

/// `a ± b` (L §3.2): a bare figure takes the other operand's class (`%` beside a `num %`,
/// L §7.1), money meets money of its currency or a `num CUR`, rates meet rates of their period.
fn add_sub(m: &Reality, op: Op, lhs: Operand, rhs: Operand) -> Result<Value, Stop> {
    let (lhs, rhs) = pct_flex(lhs, rhs)?;
    let kind = match (&lhs.kind, &rhs.kind) {
        (Kind::Plain(Class::Flex), Kind::Plain(Class::Flex)) => {
            if lhs.int_flex() && rhs.int_flex() {
                Kind::Plain(Class::Int)
            } else {
                Kind::Plain(Class::Num(Unit::one(), Shown::Plain))
            }
        }
        (Kind::Plain(Class::Flex), other) | (other, Kind::Plain(Class::Flex)) => other.clone(),
        (Kind::Money(cur), Kind::Money(other_cur)) if cur == other_cur => Kind::Money(*cur),
        (Kind::Money(_), Kind::Money(_)) => {
            return Err(refuse_load(
                Load::Unit,
                "money in two currencies: different units".to_owned(),
            ));
        }
        (Kind::Plain(Class::Rate(per)), Kind::Plain(Class::Rate(other_per))) => {
            if per != other_per {
                return Err(refuse_load(
                    Load::Unit,
                    "rates of different periods".to_owned(),
                ));
            }
            Kind::Plain(Class::Rate(per.clone()))
        }
        (Kind::Plain(Class::Rate(_)), _) | (_, Kind::Plain(Class::Rate(_))) => {
            return Err(refuse_load(
                Load::Unit,
                "rate arithmetic outside L §3.3".to_owned(),
            ));
        }
        (Kind::Plain(Class::Int), Kind::Plain(Class::Int)) => Kind::Plain(Class::Int),
        (Kind::Money(cur), Kind::Plain(Class::Num(..)))
        | (Kind::Plain(Class::Num(..)), Kind::Money(cur)) => Kind::Plain(Class::Num(
            currency_unit(m, *cur).map_err(num_stop)?,
            Shown::Plain,
        )),
        (Kind::Money(_), Kind::Plain(Class::Int)) | (Kind::Plain(Class::Int), Kind::Money(_)) => {
            return Err(refuse_load(
                Load::Unit,
                "money and int: different units".to_owned(),
            ));
        }
        (
            Kind::Plain(Class::Num(_, Shown::Percent)),
            Kind::Plain(Class::Num(_, Shown::Percent)),
        ) => Kind::Plain(Class::Num(lhs.unit(m)?, Shown::Percent)),
        (Kind::Plain(_), Kind::Plain(_)) => Kind::Plain(Class::Num(lhs.unit(m)?, Shown::Plain)),
    };
    let factor = match (&lhs.kind, &rhs.kind) {
        (Kind::Plain(Class::Flex), _) | (_, Kind::Plain(Class::Flex)) => Num::int(1),
        _ => convert(m, &rhs, &lhs)?,
    };
    let scaled = rhs.q.checked_mul(&factor).map_err(num_stop)?;
    let q = match op {
        Op::Add => lhs.q.checked_add(&scaled),
        Op::Sub | Op::Mul | Op::Div => lhs.q.checked_sub(&scaled),
    }
    .map_err(num_stop)?;
    finish(m, q, kind)
}

/// The factor that brings `from`'s value into `to`'s unit; a zero with no unit meets any
/// (refcore's runtime `arith`); otherwise different units are refused (L §3.2).
fn convert(m: &Reality, from: &Operand, to: &Operand) -> Result<Num, Stop> {
    let fu = from.unit(m)?;
    let tu = to.unit(m)?;
    if fu == tu {
        return Ok(Num::int(1));
    }
    if let Some(f) = fu.conv(&tu).map_err(num_stop)? {
        return Ok(f);
    }
    if (fu.is_one() && from.q.is_zero()) || (tu.is_one() && to.q.is_zero()) {
        return Ok(Num::int(1));
    }
    Err(refuse_load(Load::Unit, "different units".to_owned()))
}

/// A bare figure beside a `num %` takes its unit: `1` is 1 % (L §7.1, refcore's `pct_flex`).
fn pct_flex(a: Operand, b: Operand) -> Result<(Operand, Operand), Stop> {
    let pct = |o: &Operand| matches!(o.kind, Kind::Plain(Class::Num(_, Shown::Percent)));
    let hundredth = Num::ratio(1, 100).map_err(num_stop)?;
    if a.kind == Kind::Plain(Class::Flex) && pct(&b) {
        let q = a.q.checked_mul(&hundredth).map_err(num_stop)?;
        let kind = b.kind.clone();
        return Ok((Operand { q, kind }, b));
    }
    if b.kind == Kind::Plain(Class::Flex) && pct(&a) {
        let q = b.q.checked_mul(&hundredth).map_err(num_stop)?;
        let kind = a.kind.clone();
        return Ok((a, Operand { q, kind }));
    }
    Ok((a, b))
}

/// A bare figure in `*` or `/` is dimensionless: an int when it is one (L §3.2).
fn unflex(o: Operand) -> Operand {
    if o.kind != Kind::Plain(Class::Flex) {
        return o;
    }
    let class = if o.q.is_integer() {
        Class::Int
    } else {
        Class::Num(Unit::one(), Shown::Plain)
    };
    Operand {
        q: o.q,
        kind: Kind::Plain(class),
    }
}

/// Whether a class is a plain dimensionless number or an int (what a rate may be scaled by).
fn scalar(kind: &Kind) -> bool {
    match kind {
        Kind::Plain(Class::Int) => true,
        Kind::Plain(Class::Num(u, _)) => u.is_one(),
        Kind::Money(_) | Kind::Plain(Class::Rate(_) | Class::Flex) => false,
    }
}

/// `a * b`, `a / b` (L §3.2, §3.3): units multiply and divide; money times an int stays
/// money; a rate is scaled by a plain number, or times a duration gives a plain number.
fn mul_div(m: &Reality, op: Op, lhs: Operand, rhs: Operand) -> Result<Value, Stop> {
    let lhs = unflex(lhs);
    let rhs = unflex(rhs);
    if let Some(out) = rate_mul(m, op, &lhs, &rhs)? {
        return Ok(out);
    }
    if op == Op::Div {
        if rhs.q.is_zero() {
            return Err(refuse(Run::Div, "division by zero".to_owned()));
        }
        let (unit, factor) = lhs.unit(m)?.div(&rhs.unit(m)?).map_err(num_stop)?;
        let q = lhs.q.checked_div(&rhs.q).map_err(num_stop)?;
        let q = q.checked_mul(&factor).map_err(num_stop)?;
        return finish(m, q, Kind::Plain(Class::Num(unit, Shown::Plain)));
    }
    let q = lhs.q.checked_mul(&rhs.q).map_err(num_stop)?;
    let kind = match (&lhs.kind, &rhs.kind) {
        (Kind::Plain(Class::Int), Kind::Plain(Class::Int)) => Kind::Plain(Class::Int),
        (Kind::Money(cur), Kind::Plain(Class::Int))
        | (Kind::Plain(Class::Int), Kind::Money(cur)) => Kind::Money(*cur),
        _ => {
            let (unit, factor) = lhs.unit(m)?.mul(&rhs.unit(m)?).map_err(num_stop)?;
            let q = q.checked_mul(&factor).map_err(num_stop)?;
            return finish(m, q, Kind::Plain(Class::Num(unit, Shown::Plain)));
        }
    };
    finish(m, q, kind)
}

/// A rate in `*` (L §3.3): times a plain number it stays a rate; times a duration it is a
/// plain number; anything else is refused. `None` when neither operand is a rate.
fn rate_mul(m: &Reality, op: Op, lhs: &Operand, rhs: &Operand) -> Result<Option<Value>, Stop> {
    let (rate, other) = match (&lhs.kind, &rhs.kind) {
        (Kind::Plain(Class::Rate(u)), _) => (u, rhs),
        (_, Kind::Plain(Class::Rate(u))) => (u, lhs),
        _ => return Ok(None),
    };
    let refused = || refuse_load(Load::Unit, "rate arithmetic outside L §3.3".to_owned());
    if op != Op::Mul {
        return Err(refused());
    }
    let q = lhs.q.checked_mul(&rhs.q).map_err(num_stop)?;
    if scalar(&other.kind) {
        return finish(m, q, Kind::Plain(Class::Rate(rate.clone()))).map(Some);
    }
    let duration = other.unit(m)?;
    let one_period = matches!(duration.terms(), [(base, 1)]
        if matches!(base.as_str(), "day" | "month" | "quarter" | "year"));
    if !one_period {
        return Err(refused());
    }
    let (unit, factor) = rate.mul(&duration).map_err(num_stop)?;
    if !unit.is_one() {
        return Err(refuse_load(
            Load::Unit,
            "lhs rate times lhs duration of another period dimension".to_owned(),
        ));
    }
    let q = q.checked_mul(&factor).map_err(num_stop)?;
    finish(m, q, Kind::Plain(Class::Num(Unit::one(), Shown::Plain))).map(Some)
}

/// The negation of a number or money (L §3.2); `None` for any other value.
///
/// # Errors
///
/// A number error.
pub fn negate(v: &Value) -> Result<Option<Value>, Stop> {
    if let Value::Number(n) = v {
        return Ok(Some(Value::Number(Number {
            q: n.q.checked_neg().map_err(num_stop)?,
            class: n.class.clone(),
        })));
    }
    if let Value::Money(money) = v {
        return Ok(Some(Value::Money(money.checked_neg().map_err(num_stop)?)));
    }
    Ok(None)
}

/// An integer from a number that is one (L §3.2: a `num` that is an integer is accepted
/// where an `int` is); `RUN-NOT-INT` otherwise (refcore's `int_of`).
///
/// # Errors
///
/// `RUN-NOT-INT`; `RUN-NONE` for `none`; `LOAD-UNIT` for a value that isn't a number.
pub fn int_of(m: &Reality, v: &Value) -> Result<i64, Stop> {
    if let Value::Number(n) = v {
        if !n.q.is_integer() {
            return Err(refuse(
                Run::NotInt,
                format!("{} is not an integer", v.text(m).map_err(num_stop)?),
            ));
        }
        return n
            .q
            .to_i64()
            .ok_or_else(|| num_stop(crate::num::NumError::Domain));
    }
    if *v == Value::None {
        return Err(refuse(
            Run::None,
            "a value is none here: an integer argument".to_owned(),
        ));
    }
    Err(refuse_load(
        Load::Unit,
        format!("{} is not an integer", v.text(m).map_err(num_stop)?),
    ))
}

/// The period word of a duration `5 years`, `-1 day`, and its count, or `None` when the
/// number isn't one (L §4.2).
fn duration(n: &Number) -> Option<(&str, &Num)> {
    let Class::Num(unit, _) = &n.class else {
        return None;
    };
    match unit.terms() {
        [(base, 1)] if matches!(base.as_str(), "day" | "month" | "quarter" | "year") => {
            Some((base.as_str(), &n.q))
        }
        _ => None,
    }
}

/// Date, period and key arithmetic (L §4.2): a day moves by days or a duration, a period by
/// its periods, a key of an index by positions; two periods of one kind differ by an int.
fn time_arith(m: &Reality, op: Op, lhs: &Value, rhs: &Value) -> Result<Value, Stop> {
    let sign: i64 = match op {
        Op::Add => 1,
        Op::Sub => -1,
        Op::Mul | Op::Div => return Err(refuse_load(Load::Unit, shown(m, lhs, op, rhs)?)),
    };
    match (lhs, rhs) {
        (Value::Day(day), Value::Number(num)) => day_shift(m, *day, num, sign, rhs),
        (Value::Period(period), Value::Number(num)) => {
            period_shift(m, *period, num, sign, op, lhs, rhs)
        }
        (Value::Period(period), Value::Period(other)) if op == Op::Sub => {
            period_diff(*period, *other)
        }
        (Value::Key(KeyVal::Key { index, name }), Value::Number(_)) => {
            key_shift(m, index, name, int_of(m, rhs)?, sign, op)
        }
        (
            Value::Key(key @ KeyVal::Key { index, .. }),
            Value::Key(
                other_key @ KeyVal::Key {
                    index: other_index, ..
                },
            ),
        ) if op == Op::Sub && index == other_index => key_diff(m, index, key, other_key),
        _ => Err(refuse_load(Load::Unit, shown(m, lhs, op, rhs)?)),
    }
}

/// A day moved by an int of days or by a duration (refcore's `date_arith`). L §4.2.
fn day_shift(m: &Reality, day: Day, num: &Number, sign: i64, rhs: &Value) -> Result<Value, Stop> {
    if let Some((word, count)) = duration(num) {
        if !count.is_integer() {
            return Err(refuse(
                Run::NotInt,
                format!("{} is not an integer", rhs.text(m).map_err(num_stop)?),
            ));
        }
        let k = count
            .to_i64()
            .and_then(|k| k.checked_mul(sign))
            .ok_or_else(|| num_stop(crate::num::NumError::Domain))?;
        let moved = match word {
            "day" => day.add_days(k),
            "month" => day.add_months(k),
            "quarter" => k.checked_mul(3).and_then(|k| day.add_months(k)),
            _ => k.checked_mul(12).and_then(|k| day.add_months(k)),
        };
        return Ok(Value::Day(
            moved.ok_or_else(|| invariant("a day past the calendar's range"))?,
        ));
    }
    let k = int_of(m, rhs)?
        .checked_mul(sign)
        .ok_or_else(|| invariant("a day step past the integers"))?;
    day.add_days(k)
        .map(Value::Day)
        .ok_or_else(|| invariant("a day past the calendar's range"))
}

/// A period moved by an int of its periods, or by a duration that is a whole number of them
/// (refcore's `date_arith`). L §4.2.
fn period_shift(
    m: &Reality,
    period: Period,
    num: &Number,
    sign: i64,
    op: Op,
    lhs: &Value,
    rhs: &Value,
) -> Result<Value, Stop> {
    let steps = match duration(num) {
        Some((word, count)) => {
            let whole = count
                .to_i64()
                .filter(|_| count.is_integer())
                .ok_or_else(|| {
                    refuse(
                        Run::NotInt,
                        "lhs duration that is not an integer".to_owned(),
                    )
                })?;
            let (family, months) = match word {
                "day" => (PeriodKind::Day, 1),
                "month" => (PeriodKind::Month, 1),
                "quarter" => (PeriodKind::Month, 3),
                _ => (PeriodKind::Month, 12),
            };
            let amount = whole
                .checked_mul(months)
                .ok_or_else(|| invariant("lhs duration past the integers"))?;
            periods_in(m, period, family, amount, op, lhs, rhs)?
        }
        None => int_of(m, rhs)?,
    };
    let steps = steps
        .checked_mul(sign)
        .ok_or_else(|| invariant("lhs period step"))?;
    period
        .shift(steps)
        .map(Value::Period)
        .ok_or_else(|| invariant("lhs period past the calendar's range"))
}

/// A duration as a count of a period's own periods: days of a day period, months of a
/// month, quarter or year; `RUN-DOMAIN` when it isn't a whole number of them. L §4.2.
fn periods_in(
    m: &Reality,
    period: Period,
    family: PeriodKind,
    amount: i64,
    op: Op,
    lhs: &Value,
    rhs: &Value,
) -> Result<i64, Stop> {
    let (own, months) = match period.kind() {
        PeriodKind::Day => (PeriodKind::Day, 1),
        PeriodKind::Month => (PeriodKind::Month, 1),
        PeriodKind::Quarter => (PeriodKind::Month, 3),
        PeriodKind::Year => (PeriodKind::Month, 12),
    };
    let whole = amount.checked_rem_euclid(months) == Some(0);
    if own != family || !whole {
        return Err(refuse(
            Run::Domain,
            format!(
                "{}: not lhs whole number of its periods (L §4.2)",
                shown(m, lhs, op, rhs)?
            ),
        ));
    }
    amount
        .checked_div_euclid(months)
        .ok_or_else(|| invariant("lhs period step"))
}

/// The int number of periods from `b` to `a`, two periods of one kind (refcore's `pdiff`,
/// `key_diff`). L §4.2 Differences.
fn period_diff(a: Period, b: Period) -> Result<Value, Stop> {
    let diff = match (a.kind(), b.kind()) {
        (PeriodKind::Year, PeriodKind::Year) => {
            i64::from(a.start().year()).checked_sub(i64::from(b.start().year()))
        }
        (PeriodKind::Month, PeriodKind::Month) => month_number(a)
            .zip(month_number(b))
            .and_then(|(x, y)| x.checked_sub(y)),
        (PeriodKind::Quarter, PeriodKind::Quarter) => month_number(a)
            .zip(month_number(b))
            .and_then(|(x, y)| x.checked_sub(y))
            .and_then(|d| d.checked_div_euclid(3)),
        _ => b.steps_to(a),
    };
    diff.map(Value::int)
        .ok_or_else(|| refuse(Run::Domain, "the difference of two periods".to_owned()))
}

/// The months from year 0 to a period's first month: year × 12 + month.
fn month_number(p: Period) -> Option<i64> {
    let start = p.start();
    i64::from(start.year())
        .checked_mul(12)?
        .checked_add(i64::from(start.month()))
}

/// A key of a list index moved by `k` positions (refcore's `key_arith`); outside the index,
/// an [`Value::OutKey`]. L §4.2.
fn key_shift(
    m: &Reality,
    index: &str,
    name: &str,
    k: i64,
    sign: i64,
    op: Op,
) -> Result<Value, Stop> {
    let ix = m
        .indexes
        .get(index)
        .ok_or_else(|| invariant("a key of an index load didn't build"))?;
    let key = KeyVal::Key {
        index: index.to_owned(),
        name: name.to_owned(),
    };
    let pos = ix
        .pos(&key)
        .ok_or_else(|| invariant("a key its index doesn't hold"))?;
    let pos = i64::try_from(pos).map_err(|_| invariant("an index position"))?;
    let step = k.checked_mul(sign).ok_or_else(|| invariant("a key step"))?;
    let to = pos
        .checked_add(step)
        .ok_or_else(|| invariant("a key step"))?;
    if let Some(found) = usize::try_from(to).ok().and_then(|at| ix.keys.get(at)) {
        return Ok(Value::of_key(found));
    }
    Ok(Value::OutKey {
        index: index.to_owned(),
        text: format!("{name} {} {}", op.text(), k.unsigned_abs()),
    })
}

/// The int number of positions from key `b` to key `a` of one index (refcore's `key_diff`).
/// L §4.2.
fn key_diff(m: &Reality, index: &str, a: &KeyVal, b: &KeyVal) -> Result<Value, Stop> {
    let ix = m
        .indexes
        .get(index)
        .ok_or_else(|| invariant("a key of an index load didn't build"))?;
    let pa = ix.pos(a).and_then(|p| i64::try_from(p).ok());
    let pb = ix.pos(b).and_then(|p| i64::try_from(p).ok());
    pa.zip(pb)
        .and_then(|(x, y)| x.checked_sub(y))
        .map(Value::int)
        .ok_or_else(|| invariant("a key its index doesn't hold"))
}

/// What a comparison asks: an order, or only equality (`==`, `!=`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Test {
    /// `<`, `<=`, `>`, `>=`.
    Order,
    /// `==`, `!=`.
    Equality,
}

/// The order of two values that are neither holes nor `none` (refcore's `cmp3`): numbers by
/// value in one unit, bools, dates, periods, keys in index order; things, an open index's keys,
/// texts and codes by equality only (L §5.6, §5.7, §8.4).
///
/// # Errors
///
/// `RUN-DOMAIN` for an order that has no meaning; `LOAD-UNIT` for values of different units
/// or types.
pub fn compare(m: &Reality, lhs: &Value, rhs: &Value, test: Test) -> Result<Ordering, Stop> {
    if let (Some(l), Some(r)) = (operand(m, lhs)?, operand(m, rhs)?) {
        if let (Kind::Plain(Class::Rate(per)), Kind::Plain(Class::Rate(other_per))) =
            (&l.kind, &r.kind)
            && per != other_per
        {
            return Err(refuse_load(
                Load::Unit,
                "rates of different periods compared".to_owned(),
            ));
        }
        let factor = compare_factor(m, &r, &l)?;
        let scaled = r.q.checked_mul(&factor).map_err(num_stop)?;
        return l.q.compare(&scaled).map_err(num_stop);
    }
    match (lhs, rhs) {
        (Value::Bool(l), Value::Bool(r)) => Ok(l.cmp(r)),
        (Value::Day(l), Value::Day(r)) => Ok(l.cmp(r)),
        (Value::Period(l), Value::Period(r)) => Ok(l
            .start()
            .cmp(&r.start())
            .then_with(|| l.end().cmp(&r.end()))),
        (Value::Key(l), Value::Key(r)) => key_order(m, l, r, test),
        (Value::Text(l), Value::Text(r)) => Ok(if l == r {
            Ordering::Equal
        } else {
            Ordering::Greater
        }),
        (Value::Code(l), Value::Code(r)) => Ok(if l == r {
            Ordering::Equal
        } else {
            Ordering::Greater
        }),
        (Value::MonthDay(..), Value::MonthDay(..)) if test == Test::Order => {
            Err(refuse(Run::Domain, "an order on month-days".to_owned()))
        }
        (Value::MonthDay(l, r), Value::MonthDay(l3, r4)) => Ok(if l == l3 && r == r4 {
            Ordering::Equal
        } else {
            Ordering::Greater
        }),
        (Value::OutKey { index, text }, _) | (_, Value::OutKey { index, text }) => {
            Err(refuse(Run::Key, format!("{text} is outside {index}")))
        }
        (Value::Record(_), _) | (_, Value::Record(_)) => Err(refuse(
            Run::Domain,
            "lhs comparison of two records has no meaning (compare their fields)".to_owned(),
        )),
        _ => Err(refuse_load(
            Load::Unit,
            format!(
                "{} and {} compared",
                lhs.text(m).map_err(num_stop)?,
                rhs.text(m).map_err(num_stop)?
            ),
        )),
    }
}

/// The factor that brings `from` into `to`'s unit for a comparison; a number with no unit
/// meets any (refcore's `cmp3`).
fn compare_factor(m: &Reality, from: &Operand, to: &Operand) -> Result<Num, Stop> {
    let fu = from.unit(m)?;
    let tu = to.unit(m)?;
    if fu == tu || fu.is_one() || tu.is_one() {
        return Ok(Num::int(1));
    }
    fu.conv(&tu)
        .map_err(num_stop)?
        .ok_or_else(|| refuse_load(Load::Unit, "different units compared".to_owned()))
}

/// Two keys' order: by position in their index; things and an open index's keys have none
/// (L §5.6, §5.7; `RUN-DOMAIN`).
fn key_order(m: &Reality, lhs: &KeyVal, rhs: &KeyVal, test: Test) -> Result<Ordering, Stop> {
    if lhs == rhs {
        return Ok(Ordering::Equal);
    }
    if test == Test::Equality {
        return Ok(Ordering::Greater);
    }
    match (lhs, rhs) {
        (KeyVal::Key { index, .. }, KeyVal::Key { index: j, .. }) if index == j => {
            let ix = m
                .indexes
                .get(index)
                .ok_or_else(|| invariant("a key of an index load didn't build"))?;
            if ix.form == crate::model::IndexForm::Open {
                return Err(refuse(
                    Run::Domain,
                    format!("an order on keys of the open index {index} has no meaning"),
                ));
            }
            Ok(ix.pos(lhs).cmp(&ix.pos(rhs)))
        }
        (KeyVal::Thing(x), KeyVal::Thing(y)) => Err(refuse(
            Run::Domain,
            format!("an order on things ({x}, {y}) has no meaning"),
        )),
        _ => Ok(Ordering::Greater),
    }
}

/// Whether two values are equal as values: two numbers of one quantity whatever their int or
/// money mark (refcore's `same_value`). L §5.11 (an `if` whose branches are equal).
///
/// # Errors
///
/// A number error.
pub fn same_value(m: &Reality, lhs: &Value, rhs: &Value) -> Result<bool, Stop> {
    if let (Some(l), Some(r)) = (operand(m, lhs)?, operand(m, rhs)?) {
        let rates = |o: &Operand| matches!(o.kind, Kind::Plain(Class::Rate(_)));
        if rates(&l) != rates(&r) {
            return Ok(false);
        }
        let Some(factor) = l.unit(m)?.conv(&r.unit(m)?).map_err(num_stop)? else {
            return Ok(false);
        };
        let scaled = l.q.checked_mul(&factor).map_err(num_stop)?;
        return Ok(scaled == r.q);
    }
    Ok(lhs == rhs)
}

/// The currency of money of currency `name` (L §5.4).
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a currency load didn't declare.
pub fn currency_of(m: &Reality, name: &str) -> Result<Currency, Stop> {
    currency_id(m, name).map_err(num_stop)
}

#[cfg(test)]
mod tests {
    use super::{Op, period_diff};
    use crate::model::Value;
    use crate::time::Period;

    #[test]
    fn months_differ_by_an_int() {
        let a = Period::month(2026, 3).expect("March");
        let b = Period::month(2025, 11).expect("November");
        assert_eq!(period_diff(a, b), Ok(Value::int(4)), "four months");
        assert_eq!(Op::Sub.text(), "-", "minus");
    }
}
