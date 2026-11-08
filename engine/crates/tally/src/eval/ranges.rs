//! A binding's stepped ranges: `a .. b [step s]` of integers or days, counted against the key limit before one key is built.
//! Spec: L §5.6 and §5.11 (a stepped range's values, days stepped or integers), §2.3 `bind`
//! (`a .. b [step s]`), §2.5 (whether a range holds a written key), §4.2 (a day moved by its
//! step), §8.1 (an end that is a hole makes the range that hole), §7.6 item 2 (past 1 000 000
//! keys a range is `RUN-LIMIT`, `LOAD-LIMIT` for a range of literals).
//! Twin: engine/tools/refcore/evaluate.py (`range_values` with its `too_many` and
//! `date_arith`, `range_member`, `check_range_lits`) and check3.py (`m_check_literal_ranges`).
//! Invariants: a range is counted before any key is built; a range of literals past the limit
//! is refused at load with the bytes an ask that steps through it prints.

use super::aggregate::{Values, day_of_binding};
use super::arith::{Op, arith, numbers_past, whole_of};
use super::expr::literal;
use super::state::{Ctx, Engine, Env, Figures, invariant, num_stop, refuse, refuse_load};
use crate::base::{Load, Run};
use crate::hole::{Hole, spread};
use crate::model::{Class, Number, Reality, Value};
use crate::num::Num;
use crate::syntax::{Expr, Lit, Stop};
use crate::time::Day;
use core::cmp::Ordering;

/// The most keys a stepped range binds: past it, `RUN-LIMIT` (L §7.6 item 2).
const RANGE_LIMIT: i64 = 1_000_000;

/// A range's ends and step, evaluated: `a`, `b` and `s` of `a .. b [step s]` (L §2.3 `bind`).
type Ends = (Value, Value, Option<Value>);

impl Engine {
    /// A range whose ends and step are literals, counted at load and never built: past
    /// 1 000 000 keys it is `LOAD-LIMIT` with the bytes an ask that steps through it prints
    /// (refcore's `m_check_literal_ranges`, its `range_values` with `lit`). L §7.6 item 2.
    ///
    /// # Errors
    ///
    /// `LOAD-LIMIT <a> .. <b>[ step <s>] holds more than 1 000 000 keys`; any other stop the
    /// count meets, as at run.
    pub(crate) fn literal_range(
        &mut self,
        m: &Reality,
        parts: &(Expr, Expr, Option<Expr>),
    ) -> Result<(), Stop> {
        let ctx = Ctx::new(None, m.on);
        let Ok(ends) = self.range_ends(m, parts, &Env::empty(), ctx)? else {
            return Ok(());
        };
        if past_limit(m, &ends)? {
            return Err(refuse_load(Load::Limit, too_many_keys(m, &ends)?));
        }
        Ok(())
    }

    /// What load says of a key written at a stepped range's position once it is done, the
    /// range's ends and step literals: `<k> is not a value of <a> .. <b>[ step <s>]` where the
    /// range doesn't hold it; none where it does, or where load can't tell, an end refused or
    /// the range past the key limit (refcore's `check_range_lits`, `range_member`, its
    /// `continue`). L §2.5, §5.6.
    ///
    /// # Errors
    ///
    /// `NotBuilt` for a range of numbers not integers or of days with no step; the key's
    /// literal's refusal; an invariant.
    pub(crate) fn range_key_missing(
        &mut self,
        m: &Reality,
        parts: &(Expr, Expr, Option<Expr>),
        key: &Lit,
    ) -> Result<Option<String>, Stop> {
        let ctx = Ctx::new(None, m.on);
        let ends = match self.range_ends(m, parts, &Env::empty(), ctx) {
            Ok(Ok(ends)) => ends,
            Ok(Err(_)) | Err(Stop::Refused(_)) => return Ok(None),
            Err(stop) => return Err(stop),
        };
        match past_limit(m, &ends) {
            Ok(false) => {}
            Ok(true) | Err(Stop::Refused(_)) => return Ok(None),
            Err(stop) => return Err(stop),
        }
        let value = match key {
            Lit::Num(n) if n.whole && n.unit.is_none() => Value::Number(Number {
                q: n.value.clone(),
                class: Class::Int,
            }),
            Lit::Num(_)
            | Lit::Day(_)
            | Lit::Month(..)
            | Lit::Quarter(..)
            | Lit::MonthDay(..)
            | Lit::Key(_)
            | Lit::Name(_)
            | Lit::Cur(_)
            | Lit::Bool(_)
            | Lit::Nominal(_)
            | Lit::Inf { .. } => literal(m, key, Figures::Rule)?,
        };
        match range_holds(m, &ends, &value) {
            Ok(true) | Err(Stop::Refused(_)) => Ok(None),
            Ok(false) => Ok(Some(format!(
                "{} is not a value of {}",
                value.text(m).map_err(num_stop)?,
                range_text(m, &ends)?
            ))),
            Err(stop) => Err(stop),
        }
    }

    /// The values of a range `a .. b [step s]`, days stepped or integers, counted before one
    /// is built, so past 1 000 000 keys it is `RUN-LIMIT` (refcore's `range_values`). L §5.6
    /// (a stepped range), §7.6 item 2.
    pub(super) fn range_values(
        &mut self,
        m: &Reality,
        parts: &(Expr, Expr, Option<Expr>),
        env: &Env,
        ctx: Ctx,
    ) -> Result<Values, Stop> {
        let ends = match self.range_ends(m, parts, env, ctx)? {
            Ok(ends) => ends,
            Err(h) => return Ok(Values::Unknown(h)),
        };
        if past_limit(m, &ends)? {
            return Err(refuse(Run::Limit, too_many_keys(m, &ends)?));
        }
        let (first, last, by) = ends;
        if let Value::Day(start) = first {
            let Some(by) = by else {
                return Err(Stop::NotBuilt("a range of days with no step"));
            };
            return day_range(m, start, (&last, &by)).map(Values::Known);
        }
        let stride = match &by {
            None => Num::int(1),
            Some(s) if [&first, &last, s].iter().all(|v| is_int(v)) => whole_of(m, s)?,
            Some(_) => return Err(Stop::NotBuilt("a stepped range of numbers not integers")),
        };
        let (lo, hi) = (whole_of(m, &first)?, whole_of(m, &last)?);
        let limit = u64::try_from(RANGE_LIMIT).map_err(|_| invariant("the key limit"))?;
        let Some(keys) = lo.range_keys(&hi, &stride, limit).map_err(num_stop)? else {
            return Err(invariant("a range counted within the key limit"));
        };
        Ok(Values::Known(keys.into_iter().map(int_member).collect()))
    }

    /// A range's ends and step evaluated, or the hole one of them is (L §8.1). L §5.6.
    fn range_ends(
        &mut self,
        m: &Reality,
        (from, to, step): &(Expr, Expr, Option<Expr>),
        env: &Env,
        ctx: Ctx,
    ) -> Result<Result<Ends, Hole>, Stop> {
        let first = self.ev(m, from, env, ctx)?;
        let last = self.ev(m, to, env, ctx)?;
        let by = match step {
            None => None,
            Some(e) => Some(self.ev(m, e, env, ctx)?),
        };
        let holes = [
            first.as_hole(),
            last.as_hole(),
            by.as_ref().and_then(Value::as_hole),
        ];
        Ok(match spread(&holes) {
            Some(h) => Err(h),
            None => Ok((first, last, by)),
        })
    }
}

/// An integer range's key as a binding's value, a member of no unknown. L §5.6.
fn int_member(q: Num) -> (Value, Option<Hole>) {
    (
        Value::Number(Number {
            q,
            class: Class::Int,
        }),
        None,
    )
}

/// Whether a value is an integer of no unit: an `int`, or a whole inline figure, which takes
/// the type of what it meets (L §3.2).
fn is_int(v: &Value) -> bool {
    matches!(v, Value::Number(n) if n.q.is_integer()
        && matches!(n.class, Class::Int | Class::Flex))
}

/// Whether a range holds more than 1 000 000 keys, counted and never built (refcore's
/// `range_values`, its checks before `too_many`, in its order): days by their step, numbers
/// by `(b − a) ÷ s`. L §5.6, §7.6 item 2.
fn past_limit(m: &Reality, (first, last, by): &Ends) -> Result<bool, Stop> {
    if let Value::Day(start) = first {
        return days_past(m, *start, last, by.as_ref());
    }
    for v in [first, last] {
        if *v == Value::None {
            return Err(refuse(
                Run::None,
                "a value is none here: a range end".to_owned(),
            ));
        }
    }
    let limit = u64::try_from(RANGE_LIMIT).map_err(|_| invariant("the key limit"))?;
    numbers_past(m, (first, last, by.as_ref()), limit)
}

/// Whether the days `start .. end step s` are more than 1 000 000, counted and never stepped:
/// a step of days moves every day by its first step's days, and one of months moves a day
/// by 28 days at least, so a million of them pass the calendar. Past the limit, the
/// 1 000 001st day is stepped from, as refcore's loop steps from it before it counts
/// (refcore's `range_values` for days). L §4.2, §5.6, §7.6 item 2.
fn days_past(m: &Reality, start: Day, end: &Value, by: Option<&Value>) -> Result<bool, Stop> {
    let Some(by) = by else {
        return Err(Stop::NotBuilt("a range of days with no step"));
    };
    let last = day_of_binding(end)?;
    if start > last {
        return Ok(false);
    }
    let days = start
        .days_to(day_step(m, start, by)?)
        .ok_or_else(|| invariant("a step's days"))?;
    let reach = days
        .checked_mul(RANGE_LIMIT)
        .ok_or_else(|| invariant("a range's reach"))?;
    let span = start
        .days_to(last)
        .ok_or_else(|| invariant("a range's days"))?;
    if span < reach {
        return Ok(false);
    }
    let key = start
        .add_days(reach)
        .ok_or_else(|| invariant("a day of the range"))?;
    day_step(m, key, by)?;
    Ok(true)
}

/// `<a> .. <b> [step <s>] holds more than 1 000 000 keys`, the ends and step as the range's
/// values print (refcore's `too_many`). L §7.6 item 2.
fn too_many_keys(m: &Reality, ends: &Ends) -> Result<String, Stop> {
    Ok(format!(
        "{} holds more than 1 000 000 keys",
        range_text(m, ends)?
    ))
}

/// `<a> .. <b>[ step <s>]`, the ends and step as the range's values print. L §2.3 `bind`.
fn range_text(m: &Reality, (a, b, s): &Ends) -> Result<String, Stop> {
    let step = match s {
        Some(s) => format!(" step {}", s.text(m).map_err(num_stop)?),
        None => String::new(),
    };
    let (a, b) = (a.text(m).map_err(num_stop)?, b.text(m).map_err(num_stop)?);
    Ok(format!("{a} .. {b}{step}"))
}

/// Whether a range counted within the key limit holds `key`, as refcore's set of its values
/// does (`range_member`): days by stepping to it, integers by `a + i·s` for a whole `i ≥ 0`
/// up to `b`, a value of another kind never. L §2.5, §5.6.
fn range_holds(m: &Reality, (first, last, by): &Ends, key: &Value) -> Result<bool, Stop> {
    if let Value::Day(start) = first {
        let Some(by) = by else {
            return Err(Stop::NotBuilt("a range of days with no step"));
        };
        let Value::Day(day) = key else {
            return Ok(false);
        };
        let last = day_of_binding(last)?;
        let mut current = *start;
        while current < *day && current <= last {
            current = day_step(m, current, by)?;
        }
        return Ok(current == *day && current <= last);
    }
    let stride = match by {
        None => Num::int(1),
        Some(s) if [first, last, s].iter().all(|v| is_int(v)) => whole_of(m, s)?,
        Some(_) => return Err(Stop::NotBuilt("a stepped range of numbers not integers")),
    };
    let (lo, hi) = (whole_of(m, first)?, whole_of(m, last)?);
    let Value::Number(Number {
        q,
        class: Class::Int,
    }) = key
    else {
        return Ok(false);
    };
    let offset = q.checked_sub(&lo).map_err(num_stop)?;
    if offset.is_negative() || q.compare(&hi).map_err(num_stop)? == Ordering::Greater {
        return Ok(false);
    }
    Ok(offset.checked_div(&stride).map_err(num_stop)?.is_integer())
}

/// The days from `start` to `end`, stepped, a range counted within the key limit (refcore's
/// `range_values` for days). L §5.11.
fn day_range(
    m: &Reality,
    start: Day,
    (end, step): (&Value, &Value),
) -> Result<Vec<(Value, Option<Hole>)>, Stop> {
    let last = day_of_binding(end)?;
    let mut out = Vec::new();
    let mut current = start;
    while current <= last {
        out.push((Value::Day(current), None));
        current = day_step(m, current, step)?;
    }
    Ok(out)
}

/// A day of a range moved by its step (refcore's `date_arith` in `range_values`): a step
/// that isn't a duration is `LOAD-UNIT`, one that doesn't move forward `RUN-DOMAIN`. L §4.2,
/// §5.11.
fn day_step(m: &Reality, day: Day, step: &Value) -> Result<Day, Stop> {
    let Value::Day(next) = arith(m, Op::Add, &Value::Day(day), step)? else {
        // `arith` refuses a day moved by anything but a duration first
        return Err(Stop::NotBuilt(
            "a range of days stepped by no duration, met at run",
        ));
    };
    if next <= day {
        return Err(refuse(
            Run::Domain,
            "a stepped range whose step is not positive".to_owned(),
        ));
    }
    Ok(next)
}
