//! Arithmetic and comparison on values: numbers with their classes and units, money, and the order of every value; days, periods and keys go to `time_arith.rs`.
//! Spec: L §3.2 (the unit rules: `+ - * /`, money as a `num CUR`, inline figures), §3.3 (rate
//! arithmetic), §5.6 and §5.7 (keys and things: which have an order), §7.1 (exact numbers),
//! §7.6 (Tally's limits: integers of any size), §8.4 (what is refused per value); L §4.2's
//! date and key arithmetic is `time_arith.rs`'s.
//! Twin: engine/tools/refcore/evaluate.py (`arith`, `binop`, `cmp3`, `same_value`) and
//! check.py (`_arith`, `arith_time`: the classes of results, which the runtime takes from its
//! operands here).
//! Invariants: holes, `none` and `never` are handled by the caller before these run; every
//! integer step is checked; a mismatch of units is refused, never converted silently.
use core::cmp::Ordering;

use super::keys::option_at;
use super::state::{invariant, num_stop, refuse, refuse_load, unit_stop};
use super::time_arith::time_arith;
use crate::base::{Load, Run};
use crate::model::{
    Class, KeyVal, Number, Reality, Shown, Unit, Value, currency_id, currency_unit, money_num,
    num_money,
};
use crate::num::{Currency, Num};
use crate::syntax::Stop;

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
    pub(super) fn text(self) -> &'static str {
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
pub(super) enum Kind {
    /// `money CUR`.
    Money(Currency),
    /// Any other number.
    Plain(Class),
}

/// A numeric operand: its value in its own unit and its class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Operand {
    /// The value.
    pub(super) q: Num,
    /// The class.
    pub(super) kind: Kind,
}

impl Operand {
    /// The operand's unit: a currency's, or its class's (L §3.2).
    pub(super) fn unit(&self, m: &Reality) -> Result<Unit, Stop> {
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
pub(super) fn operand(m: &Reality, v: &Value) -> Result<Option<Operand>, Stop> {
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
pub(super) fn finish(m: &Reality, q: Num, kind: Kind) -> Result<Value, Stop> {
    match kind {
        Kind::Money(cur) => Ok(Value::Money(num_money(m, &q, cur).map_err(num_stop)?)),
        Kind::Plain(class) => Ok(Value::Number(Number { q, class })),
    }
}

/// `a op b` as L §8.4's refusal names it, `5 + true`.
pub(super) fn shown(m: &Reality, a: &Value, op: Op, b: &Value) -> Result<String, Stop> {
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
    if matches!(lhs, Value::Window(_)) || matches!(rhs, Value::Window(_)) {
        // the evaluator's arithmetic takes a window (`windows.rs`); a sum or a walk doesn't yet
        return Err(Stop::NotBuilt(
            "a window in arithmetic outside an expression",
        ));
    }
    if matches!(lhs, Value::Day(_) | Value::Period(_) | Value::Key(_))
        || matches!(rhs, Value::Day(_) | Value::Period(_) | Value::Key(_))
    {
        return time_arith(m, op, lhs, rhs);
    }
    let (Some(x), Some(y)) = (operand(m, lhs)?, operand(m, rhs)?) else {
        return Err(refuse_load(Load::Unit, shown(m, lhs, op, rhs)?));
    };
    let out = match op {
        Op::Add | Op::Sub => add_sub(m, op, x, y, (lhs, rhs))?,
        Op::Mul | Op::Div => mul_div(m, op, x, y, (lhs, rhs))?,
    };
    if let Value::Number(n) = &out
        && n.unit().day_and_period()
    {
        // load refuses `1 day / 1 year` (L §3.2); an operand whose unit only the run knows (a
        // leg of unit any) is refused where it is computed, with load's text (pick P66, #134)
        return Err(refuse_load(
            Load::Unit,
            format!(
                "{}: day is a dimension of its own (1 day / 1 year is refused, L §3.2)",
                shown(m, lhs, op, rhs)?
            ),
        ));
    }
    Ok(out)
}

/// `<a> and <b> have different units`, the operands as written; values in two keys of one
/// unit kind, which load saw only as that kind, `<a> and <b> are in different keys <u> and
/// <u'> of <K>` (L §3.2, §8.4 `RUN-DOMAIN`; refcore's `arith`, `cmp3`, `key_clash`).
fn different_units(m: &Reality, (a, b): (&Value, &Value)) -> Result<Stop, Stop> {
    let shown = (a.text(m).map_err(num_stop)?, b.text(m).map_err(num_stop)?);
    for x in unit_keys(m, a)? {
        for y in unit_keys(m, b)? {
            if let (Some(kx), Some(ky)) = (m.unit_kind_keys.get(&x), m.unit_kind_keys.get(&y))
                && x != y
                && kx == ky
            {
                return Ok(refuse(
                    Run::Domain,
                    format!(
                        "{} and {} are in different keys {x} and {y} of {kx}",
                        shown.0, shown.1
                    ),
                ));
            }
        }
    }
    Ok(refuse_load(
        Load::Unit,
        format!("{} and {} have different units", shown.0, shown.1),
    ))
}

/// The base units of a value's unit: money's currency, a number's terms.
fn unit_keys(m: &Reality, v: &Value) -> Result<Vec<String>, Stop> {
    if let Value::Money(x) = v {
        let (cur, _) = crate::model::currency(m, x.currency()).map_err(num_stop)?;
        return Ok(vec![cur.to_owned()]);
    }
    if let Value::Number(n) = v {
        let unit = n.unit();
        return Ok(unit.terms().iter().map(|(base, _)| base.clone()).collect());
    }
    Ok(Vec::new())
}

/// `a ± b` (L §3.2): a bare figure takes the other operand's class (`%` beside a `num %`,
/// L §7.1), money meets money of its currency or a `num CUR`, rates meet rates of their
/// period; the values as written for a refusal's text.
fn add_sub(
    m: &Reality,
    op: Op,
    lhs: Operand,
    rhs: Operand,
    values: (&Value, &Value),
) -> Result<Value, Stop> {
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
        (Kind::Money(_), Kind::Money(_)) => return Err(different_units(m, values)?),
        (Kind::Plain(Class::Rate(per)), Kind::Plain(Class::Rate(other_per))) => {
            if per != other_per {
                return Err(rate_periods(m, op, values)?);
            }
            Kind::Plain(Class::Rate(per.clone()))
        }
        (Kind::Plain(Class::Rate(_)), _) | (_, Kind::Plain(Class::Rate(_))) => {
            if lhs.unit(m)? != rhs.unit(m)? {
                return Err(rate_periods(m, op, values)?);
            }
            // load refuses a rate beside a number (L §3.3), so the run does, with its text
            return Err(outside_rates(m, op, values)?);
        }
        (Kind::Plain(Class::Int), Kind::Plain(Class::Int)) => Kind::Plain(Class::Int),
        (Kind::Money(cur), Kind::Plain(Class::Num(..)))
        | (Kind::Plain(Class::Num(..)), Kind::Money(cur)) => Kind::Plain(Class::Num(
            currency_unit(m, *cur).map_err(num_stop)?,
            Shown::Plain,
        )),
        (Kind::Money(cur), Kind::Plain(Class::Int)) if rhs.q.is_zero() => Kind::Money(*cur),
        (Kind::Plain(Class::Int), Kind::Money(cur)) if lhs.q.is_zero() => Kind::Money(*cur),
        (Kind::Money(_), Kind::Plain(Class::Int)) | (Kind::Plain(Class::Int), Kind::Money(_)) => {
            return Err(different_units(m, values)?);
        }
        (
            Kind::Plain(Class::Num(_, Shown::Percent)),
            Kind::Plain(Class::Num(_, Shown::Percent)),
        ) => Kind::Plain(Class::Num(lhs.unit(m)?, Shown::Percent)),
        (Kind::Plain(_), Kind::Plain(_)) => Kind::Plain(Class::Num(lhs.unit(m)?, Shown::Plain)),
    };
    let factor = match (&lhs.kind, &rhs.kind) {
        (Kind::Plain(Class::Flex), _) | (_, Kind::Plain(Class::Flex)) => Num::int(1),
        _ => convert(m, &rhs, &lhs, values)?,
    };
    let scaled = rhs.q.checked_mul(&factor).map_err(num_stop)?;
    let q = match op {
        Op::Add => lhs.q.checked_add(&scaled),
        Op::Sub | Op::Mul | Op::Div => lhs.q.checked_sub(&scaled),
    }
    .map_err(num_stop)?;
    finish(m, q, kind)
}

/// `<a> <op> <b>: rate arithmetic outside L §3.3`, load's text (refcore's `_arith`), met at
/// run where an operand's unit only the run knows (a leg of unit any; pick P66, #134).
fn outside_rates(m: &Reality, op: Op, (a, b): (&Value, &Value)) -> Result<Stop, Stop> {
    Ok(refuse_load(
        Load::Unit,
        format!("{}: rate arithmetic outside L §3.3", shown(m, a, op, b)?),
    ))
}

/// `rates of different periods: <a> <op> <b>` (L §3.3; refcore's `arith`).
fn rate_periods(m: &Reality, op: Op, (a, b): (&Value, &Value)) -> Result<Stop, Stop> {
    Ok(refuse_load(
        Load::Unit,
        format!("rates of different periods: {}", shown(m, a, op, b)?),
    ))
}

/// The factor that brings `from`'s value into `to`'s unit; a zero with no unit meets any
/// (refcore's runtime `arith`); otherwise different units are refused (L §3.2), the
/// written operands `values` named.
fn convert(
    m: &Reality,
    from: &Operand,
    to: &Operand,
    values: (&Value, &Value),
) -> Result<Num, Stop> {
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
    Err(different_units(m, values)?)
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
/// money; a rate is scaled by a plain number, or times a duration gives a plain number; the
/// values as written for a refusal's text.
fn mul_div(
    m: &Reality,
    op: Op,
    lhs: Operand,
    rhs: Operand,
    values: (&Value, &Value),
) -> Result<Value, Stop> {
    let lhs = unflex(lhs);
    let rhs = unflex(rhs);
    if let Some(out) = rate_mul(m, op, (&lhs, &rhs), values)? {
        return Ok(out);
    }
    if op == Op::Div {
        if rhs.q.is_zero() {
            return Err(refuse(Run::Div, "division by zero".to_owned()));
        }
        let (unit, factor) = lhs.unit(m)?.div(&rhs.unit(m)?).map_err(unit_stop)?;
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
            let (unit, factor) = lhs.unit(m)?.mul(&rhs.unit(m)?).map_err(unit_stop)?;
            let q = q.checked_mul(&factor).map_err(num_stop)?;
            return finish(m, q, Kind::Plain(Class::Num(unit, Shown::Plain)));
        }
    };
    finish(m, q, kind)
}

/// A rate in `*` (L §3.3): times a plain number it stays a rate; times a duration it is a
/// plain number; times a duration of another period dimension (a rate per year times days)
/// it is `LOAD-UNIT` with the product's unit, met where it is computed, since load can't see
/// a rate's period through its row (refcore's `binop`, `_rate_duration`); anything else is
/// refused. `None` when neither operand is a rate.
fn rate_mul(
    m: &Reality,
    op: Op,
    (lhs, rhs): (&Operand, &Operand),
    (a, b): (&Value, &Value),
) -> Result<Option<Value>, Stop> {
    let (rate, other) = match (&lhs.kind, &rhs.kind) {
        (Kind::Plain(Class::Rate(u)), _) => (u, rhs),
        (_, Kind::Plain(Class::Rate(u))) => (u, lhs),
        _ => return Ok(None),
    };
    // load refuses rate arithmetic outside L §3.3, so the run does, with its text
    if op != Op::Mul {
        return Err(outside_rates(m, op, (a, b))?);
    }
    let q = lhs.q.checked_mul(&rhs.q).map_err(num_stop)?;
    if scalar(&other.kind) {
        return finish(m, q, Kind::Plain(Class::Rate(rate.clone()))).map(Some);
    }
    let duration = other.unit(m)?;
    let one_period = matches!(duration.terms(), [(base, 1)]
        if matches!(base.as_str(), "day" | "month" | "quarter" | "year"));
    if !one_period {
        return Err(outside_rates(m, op, (a, b))?);
    }
    let (unit, factor) = lhs.unit(m)?.mul(&rhs.unit(m)?).map_err(unit_stop)?;
    if !unit.is_one() {
        return Err(refuse_load(
            Load::Unit,
            format!(
                "{}: a rate times a duration of another period dimension ({})",
                shown(m, a, op, b)?,
                unit.text()
            ),
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
/// `RUN-NOT-INT`; `RUN-NONE` for `none`; `NotBuilt` for a value that isn't a number.
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
    // no number is a range (L §4.4): a caller asks a ranged date for its hole first
    Err(Stop::NotBuilt(
        "an integer argument that isn't a number, met at run",
    ))
}

/// An integer of any size, as refcore's Python integer, where one names a count, a day of
/// a month or a range's end (`int_of`'s twin past 64 bits). L §3.2, §7.6.
///
/// # Errors
///
/// `int_of`'s for a value that isn't an integer.
pub(super) fn whole_of(m: &Reality, v: &Value) -> Result<Num, Stop> {
    if let Value::Number(n) = v
        && n.q.is_integer()
    {
        return Ok(n.q.clone());
    }
    int_of(m, v).map(Num::int)
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
    if matches!(lhs, Value::Window(_)) || matches!(rhs, Value::Window(_)) {
        // a comparison decides a window (`windows.rs`); an aggregate's order doesn't yet
        return Err(Stop::NotBuilt("a window ordered outside a comparison"));
    }
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
        let factor = compare_factor(m, &r, &l, (lhs, rhs))?;
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
            "a comparison of two records has no meaning (compare their fields)".to_owned(),
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
/// meets any (refcore's `cmp3`); the written operands `values` named in the refusal.
fn compare_factor(
    m: &Reality,
    from: &Operand,
    to: &Operand,
    values: (&Value, &Value),
) -> Result<Num, Stop> {
    let fu = from.unit(m)?;
    let tu = to.unit(m)?;
    if fu == tu || fu.is_one() || tu.is_one() {
        return Ok(Num::int(1));
    }
    match fu.conv(&tu).map_err(num_stop)? {
        Some(f) => Ok(f),
        None => Err(different_units(m, values)?),
    }
}

/// Two keys' order: by position in their index, an axis's options in the order written;
/// things and an open index's keys have none (L §5.5, §5.6, §5.7; `RUN-DOMAIN`).
fn key_order(m: &Reality, lhs: &KeyVal, rhs: &KeyVal, test: Test) -> Result<Ordering, Stop> {
    if lhs == rhs {
        return Ok(Ordering::Equal);
    }
    if test == Test::Equality {
        return Ok(Ordering::Greater);
    }
    match (lhs, rhs) {
        (
            KeyVal::Key { index, name: left },
            KeyVal::Key {
                index: j,
                name: right,
            },
        ) if index == j && m.axes.contains(index) => {
            let (Some(first), Some(second)) =
                (option_at(m, index, left), option_at(m, index, right))
            else {
                return Err(invariant("an axis's option it lacks"));
            };
            Ok(first.cmp(&second))
        }
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
