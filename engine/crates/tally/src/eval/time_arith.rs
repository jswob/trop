//! Date, period and key arithmetic: a day moved by days or a duration, a period by its periods, a key by positions, and two periods' or keys' difference.
//! Spec: L §4.2 (date arithmetic, key arithmetic, differences of periods: a day moves by days
//! or a duration, a period by its periods, a key of an index by positions; two periods of one
//! kind differ by an int), §3.2 (a duration's unit), §7.6 item 1 (a day past the calendar),
//! §8.4 (what is refused per value).
//! Twin: engine/tools/refcore/evaluate.py (`date_arith`, `key_arith`, `key_diff`).
//! Invariants: holes, `none` and `never` are handled by the caller before these run; every
//! integer step is checked; a count past the 64-bit integers is past the calendar in its
//! direction, `RUN-LIMIT`.

use super::arith::{Op, shown, whole_of};
use super::state::{invariant, num_stop, past_days, refuse, refuse_load};
use crate::base::{Load, Run};
use crate::model::{Class, KeyVal, Number, Reality, Value};
use crate::num::Num;
use crate::syntax::Stop;
use crate::time::{Day, Period, PeriodKind, Step};

/// A count of days, months or business days to step a day by, an integer times `per`
/// (refcore's Python integer): past the 64-bit integers no day it reaches is in the
/// calendar, so it is `RUN-LIMIT` in its direction. L §4.2, §7.6 item 1.
///
/// # Errors
///
/// `int_of`'s for a value that isn't an integer; `RUN-LIMIT` past the 64-bit integers.
pub(super) fn step_count(m: &Reality, v: &Value, per: i64) -> Result<i64, Stop> {
    steps_of(&whole_of(m, v)?, per)
}

/// `count × per` as a step count of 64 bits; past them, `RUN-LIMIT` in its direction.
/// L §7.6 item 1.
fn steps_of(count: &Num, per: i64) -> Result<i64, Stop> {
    let steps = count.checked_mul(&Num::int(per)).map_err(num_stop)?;
    steps
        .to_i64()
        .ok_or_else(|| past_days(if steps.is_negative() { -1 } else { 1 }))
}

/// The period word of a duration `5 years`, `-1 day`, and its count, or `None` when the
/// number isn't one (L §4.2).
pub(super) fn duration(n: &Number) -> Option<(&str, &Num)> {
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
pub(super) fn time_arith(m: &Reality, op: Op, lhs: &Value, rhs: &Value) -> Result<Value, Stop> {
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
            key_shift(m, (index, name), &whole_of(m, rhs)?, sign, op)
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
        let months = match word {
            "day" | "month" => sign,
            "quarter" => sign
                .checked_mul(3)
                .ok_or_else(|| invariant("a step's sign"))?,
            _ => sign
                .checked_mul(12)
                .ok_or_else(|| invariant("a step's sign"))?,
        };
        let k = steps_of(count, months)?;
        let moved = if word == "day" {
            day.add_days(k)
        } else {
            day.add_months(k)
        };
        return Ok(Value::Day(moved.ok_or_else(|| past_days(k))?));
    }
    let k = step_count(m, rhs, sign)?;
    day.add_days(k).map(Value::Day).ok_or_else(|| past_days(k))
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
            if !count.is_integer() {
                return Err(refuse(
                    Run::NotInt,
                    format!("{} is not an integer", rhs.text(m).map_err(num_stop)?),
                ));
            }
            let (family, months) = match word {
                "day" => (PeriodKind::Day, 1),
                "month" => (PeriodKind::Month, 1),
                "quarter" => (PeriodKind::Month, 3),
                _ => (PeriodKind::Month, 12),
            };
            let amount = count.checked_mul(&Num::int(months)).map_err(num_stop)?;
            let periods = periods_in(m, period, family, &amount, (op, lhs, rhs))?;
            steps_of(&periods, sign)?
        }
        None => step_count(m, rhs, sign)?,
    };
    period
        .shift(steps)
        .map(Value::Period)
        .ok_or_else(|| past_days(steps))
}

/// A duration of `amount` days or months, an integer of any size, as a count of a period's
/// own periods; `RUN-DOMAIN` when it isn't a whole number of them, before any limit
/// (refcore's `date_arith`). L §4.1 `by n`, §4.2.
fn periods_in(
    m: &Reality,
    period: Period,
    family: PeriodKind,
    amount: &Num,
    (op, lhs, rhs): (Op, &Value, &Value),
) -> Result<Num, Stop> {
    if let Some(periods) = periods_of(period.step(), family, amount)? {
        return Ok(periods);
    }
    Err(refuse(
        Run::Domain,
        format!(
            "{}: not a whole number of its periods (L §4.2)",
            shown(m, lhs, op, rhs)?
        ),
    ))
}

/// `amount` days or months (`family`) as a count of periods of `step`, the days or months
/// a period stands for: one key of its index, `by n` of them (refcore's `unit` and `n`);
/// `None` when it isn't a whole number of them. L §4.1 `by n`, §4.2.
pub(crate) fn periods_of(
    step: Step,
    family: PeriodKind,
    amount: &Num,
) -> Result<Option<Num>, Stop> {
    let (own, size) = match step {
        Step::Days(n) => (PeriodKind::Day, n),
        Step::Months(n) => (PeriodKind::Month, n),
    };
    if own != family {
        return Ok(None);
    }
    let periods = amount
        .checked_div(&Num::int(i64::from(size)))
        .map_err(num_stop)?;
    Ok(periods.is_integer().then_some(periods))
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

/// A key of a list index moved by `k` positions, an integer of any size (refcore's
/// `key_arith`); outside the index, an [`Value::OutKey`]. L §4.2, §7.6.
fn key_shift(
    m: &Reality,
    (index, name): (&str, &str),
    k: &Num,
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
    let step = k.checked_mul(&Num::int(sign)).map_err(num_stop)?;
    let to = Num::int(pos).checked_add(&step).map_err(num_stop)?;
    let at = to.to_i64().and_then(|at| usize::try_from(at).ok());
    if let Some(found) = at.and_then(|at| ix.keys().get(at)) {
        return Ok(Value::of_key(found));
    }
    let size = if k.is_negative() {
        k.checked_neg().map_err(num_stop)?
    } else {
        k.clone()
    };
    let size = size
        .integer_text()
        .ok_or_else(|| invariant("a key step that isn't whole"))?;
    Ok(Value::OutKey {
        index: index.to_owned(),
        text: format!("{name} {} {size}", op.text()),
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

#[cfg(test)]
mod tests {
    use super::{Op, period_diff, periods_of, steps_of};
    use crate::eval::past_days;
    use crate::model::Value;
    use crate::num::Num;
    use crate::time::{Period, PeriodKind, Step};

    #[test]
    fn a_duration_counts_the_periods_of_its_keys_by_n() {
        let huge = Num::int(i64::MAX)
            .checked_mul(&Num::int(4))
            .expect("past 64 bits");
        let month = (Step::Months(1), PeriodKind::Month);
        let by_two = (Step::Months(2), PeriodKind::Month);
        let years_by_two = (Step::Months(24), PeriodKind::Month);
        let weeks = (Step::Days(7), PeriodKind::Day);
        let cases = [
            (month, Num::int(5), Some(Num::int(5)), "a month by 1"),
            (month, huge.clone(), Some(huge.clone()), "by 1 past 64 bits"),
            (by_two, Num::int(1), None, "one month of two"),
            (by_two, Num::int(-6), Some(Num::int(-3)), "six months back"),
            (
                by_two,
                huge.clone(),
                huge.checked_div(&Num::int(2)).ok(),
                "even, past 64 bits",
            ),
            (
                by_two,
                huge.checked_add(&Num::int(1)).expect("odd"),
                None,
                "odd, past 64 bits",
            ),
            (years_by_two, Num::int(12), None, "one year of two"),
            (years_by_two, Num::int(48), Some(Num::int(2)), "four years"),
            (weeks, Num::int(14), Some(Num::int(2)), "two weeks"),
            (weeks, Num::int(1), None, "a day of a week"),
        ];
        for ((step, family), amount, periods, what) in cases {
            assert_eq!(periods_of(step, family, &amount), Ok(periods), "{what}");
        }
        assert_eq!(
            periods_of(Step::Days(1), PeriodKind::Month, &Num::int(1)),
            Ok(None),
            "months of a day"
        );
        assert_eq!(
            periods_of(Step::Months(12), PeriodKind::Day, &Num::int(365)),
            Ok(None),
            "days of a year"
        );
    }

    #[test]
    fn a_step_count_past_64_bits_is_past_the_calendar_in_its_direction() {
        let min = Num::int(i64::MIN);
        assert_eq!(steps_of(&Num::int(-3), -12), Ok(36), "in range");
        assert_eq!(steps_of(&min, 1), Ok(i64::MIN), "the least step");
        assert_eq!(steps_of(&min, -1), Err(past_days(1)), "minus the least");
        assert_eq!(
            steps_of(&Num::int(i64::MAX), 3),
            Err(past_days(1)),
            "forward"
        );
        assert_eq!(
            steps_of(&Num::int(i64::MAX), -3),
            Err(past_days(-1)),
            "back"
        );
    }

    #[test]
    fn months_differ_by_an_int() {
        let a = Period::month(2026, 3).expect("March");
        let b = Period::month(2025, 11).expect("November");
        assert_eq!(period_diff(a, b), Ok(Value::int(4)), "four months");
        assert_eq!(Op::Sub.text(), "-", "minus");
    }
}
