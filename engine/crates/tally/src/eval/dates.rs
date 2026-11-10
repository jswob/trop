//! The date built-ins: period edges, a day's period, day counts, moved days, ages, weekdays and the year fraction, each over a window too.
//! Spec: L §4.2 (`start_of`, `end_of`, `year_of`, `quarter_of`, `month_of`, `day_of`,
//! `days_between`, `add_days`, `add_months`, `add_years`, `overlap_days`, `age`, `age_months`,
//! `weekday`), §4.4 (a built-in on a window gives the interval of its values at the window's
//! first and last day; `weekday` is 1..7 on a window of seven days or more), §3.3 (`dcf` with
//! `act365`), §2.1 (a four-digit figure where a period is read), §7.6 item 1.
//! Twin: engine/tools/refcore/evaluate.py (`c_start_of` to `c_weekday`, `as_date_or_iv`,
//! `period_arg`, `months_arg`, `lift`) and eval2.py (`m_c_dcf`, `m_dcf_q`, `m_as_day`).
//! Invariants: a hole argument makes the call that hole; one window among the arguments is
//! lifted to its ends, two are `HOLE-IMPRECISE`; a built-in that is `none` at a window's end
//! is `HOLE-IMPRECISE`.

use std::collections::BTreeSet;

use super::arith::{step_count, whole_of};
use super::expr::hole_of;
use super::state::{Ctx, Engine, Env, invariant, num_stop, refuse, refuse_load};
use super::state::{a_day, not_typed, past_days};
use super::windows::{End, as_window, imprecise, interval, window_end};
use super::years::as_year;
use crate::base::{Load, Run};
use crate::model::{Class, Code, Number, Reality, Unit, Value, Window};
use crate::num::Num;
use crate::syntax::{Call, Expr, Stop};
use crate::time::{Day, Period, PeriodKind, age, age_months, overlap_days};

/// A date built-in on values that are no windows, which a window lifts to its ends (L §4.2,
/// §4.4; refcore's lambdas under `lift`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DateFn {
    /// A window's day as `start_of` gives it: the day.
    Start,
    /// A window's day as `end_of` gives it: the day after.
    End,
    /// `year_of(d)`.
    Year,
    /// `month_of(d)`.
    Month,
    /// `quarter_of(d)`.
    Quarter,
    /// `days_between(a, b)`.
    Between,
    /// `add_days(d, n)`.
    AddDays(i64),
    /// `add_months(d, n)`, and `add_years(d, n)` as 12n months.
    AddMonths(i64),
    /// `overlap_days(a, b, k)`.
    Overlap(Period),
    /// `age(b, d)`: `none` before b.
    Age,
    /// `age_months(b, d)`: `none` before b.
    AgeMonths,
}

impl Engine {
    /// `start_of(p)`, `end_of(p)` (the first day after p), `year_of(d)`, `month_of(d)`,
    /// `quarter_of(d)`, of a period, a day or a window (refcore's `c_start_of`, `c_end_of`,
    /// `_of`). L §4.2, §4.4.
    pub(super) fn period_edge(
        &mut self,
        m: &Reality,
        call: &Call,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let [v] = self
            .args(m, call, env, ctx)?
            .try_into()
            .map_err(|_| arity())?;
        if let Value::Hole(_) = v {
            return Ok(v);
        }
        let name = call.name.as_str();
        let f = match name {
            "start_of" => DateFn::Start,
            "end_of" => DateFn::End,
            "year_of" => DateFn::Year,
            "month_of" => DateFn::Month,
            "quarter_of" => DateFn::Quarter,
            _ => return Err(invariant("a period built-in dispatched by another name")),
        };
        if let Value::Window(_) = v {
            return lifted(m, f, &[v]);
        }
        if v == Value::Never && matches!(f, DateFn::Year | DateFn::Month | DateFn::Quarter) {
            // refcore's `_of` reads its argument through `date_of`
            return Err(refuse(Run::None, format!("never in {name}")));
        }
        let period = period_arg(m, (call.args.first(), &v), name)?;
        if f == DateFn::End {
            return Ok(Value::Day(period.end()));
        }
        apply(f, &[Value::Day(period.start())])
    }

    /// `day_of(m, n)`, the n-th day of month m clamped to its days; `day_of(y, md)`, a
    /// month-day in year y (refcore's `c_day_of`). L §4.2.
    pub(super) fn day_of(
        &mut self,
        m: &Reality,
        call: &Call,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let [a, b] = self
            .args(m, call, env, ctx)?
            .try_into()
            .map_err(|_| arity())?;
        if a.as_hole().is_some() || b.as_hole().is_some() {
            return hole_of(&[a.as_hole(), b.as_hole()]);
        }
        let period = period_arg(m, (call.args.first(), &a), "day_of")?;
        let anchored = || {
            refuse(
                Run::Domain,
                format!("day_of of {}, a key of an anchored index", period.label()),
            )
        };
        if !matches!(b, Value::MonthDay(..)) {
            let number = whole_of(m, &b)?;
            if period.kind() != PeriodKind::Month || period.start().day() != 1 {
                return Err(anchored());
            }
            if number.is_negative() || number.is_zero() {
                let text = number
                    .integer_text()
                    .ok_or_else(|| invariant("a day number"))?;
                return Err(refuse(
                    Run::Domain,
                    format!("day_of(m, {text}): a day number below 1"),
                ));
            }
            let last = period
                .end()
                .previous()
                .ok_or_else(|| invariant("a month with no last day"))?;
            // a number past 64 bits is past the month's last day too
            let day = match number.to_i64() {
                Some(whole) => u8::try_from(whole.min(i64::from(last.day())))
                    .map_err(|_| invariant("a day number"))?,
                None => last.day(),
            };
            return Day::from_ymd(last.year(), last.month(), day)
                .map(Value::Day)
                .ok_or_else(|| invariant("a day of a month"));
        }
        if period.kind() == PeriodKind::Year
            && (period.start().month(), period.start().day()) != (1, 1)
        {
            return Err(anchored());
        }
        let Value::MonthDay(month, day) = b else {
            return Err(invariant(
                "day_of(y, md) with no month-day: load's types refuse it first (LOAD-UNIT)",
            ));
        };
        Day::from_ymd(period.start().year(), month, day)
            .map(Value::Day)
            .ok_or_else(|| {
                refuse(
                    Run::Domain,
                    format!("day_of(y, {month:02}-{day:02}) in a year without that day"),
                )
            })
    }

    /// `days_between(a, b)`, `add_days(d, n)`, `add_months(d, n)`, `add_years(d, n)`,
    /// `age(b, d)`, `age_months(b, d)`, each over a window too (refcore's `c_days_between`,
    /// `c_add_days`, `c_add_months`, `c_add_years`, `c_age`). L §4.2, §4.4.
    pub(super) fn day_count(
        &mut self,
        m: &Reality,
        call: &Call,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let [a, b] = self
            .args(m, call, env, ctx)?
            .try_into()
            .map_err(|_| arity())?;
        if a.as_hole().is_some() || b.as_hole().is_some() {
            return hole_of(&[a.as_hole(), b.as_hole()]);
        }
        let name = call.name.as_str();
        let first = date_or_window(a, name)?;
        if matches!(name, "add_days" | "add_months" | "add_years")
            && let Some(w) = as_window(&b)
        {
            // a count that is an interval, where one value is needed (L §4.4)
            return Ok(Value::Hole(imprecise(m, w, name)?));
        }
        match name {
            "days_between" => lifted(m, DateFn::Between, &[first, date_or_window(b, name)?]),
            "add_days" => lifted(m, DateFn::AddDays(step_count(m, &b, 1)?), &[first]),
            "add_months" => lifted(m, DateFn::AddMonths(months_arg(m, &b, "month")?), &[first]),
            "add_years" => lifted(m, DateFn::AddMonths(months_arg(m, &b, "year")?), &[first]),
            "age_months" => lifted(m, DateFn::AgeMonths, &[first, date_or_window(b, name)?]),
            "age" => lifted(m, DateFn::Age, &[first, date_or_window(b, name)?]),
            _ => Err(invariant("a day-count built-in dispatched by another name")),
        }
    }

    /// `overlap_days(a, b, k)`: the days from a to b, both counted, that lie in period k; of
    /// two windows `HOLE-IMPRECISE` (refcore's `c_overlap_days`). L §4.2, §4.4.
    pub(super) fn overlap(
        &mut self,
        m: &Reality,
        call: &Call,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let [a, b, k] = self
            .args(m, call, env, ctx)?
            .try_into()
            .map_err(|_| arity())?;
        if let Some(h) = crate::hole::spread(&[a.as_hole(), b.as_hole(), k.as_hole()]) {
            return Ok(Value::Hole(h));
        }
        let from = date_or_window(a, "overlap_days")?;
        let to = date_or_window(b, "overlap_days")?;
        let period = period_arg(m, (call.args.get(2), &k), "overlap_days")?;
        if let (Value::Window(w), Value::Window(_)) = (&from, &to) {
            return Ok(Value::Hole(imprecise(m, w, "overlap_days of two windows")?));
        }
        lifted(m, DateFn::Overlap(period), &[from, to])
    }

    /// `weekday(d)`: 1 for Monday to 7 for Sunday; of a window the interval of its days'
    /// weekdays, 1..7 for a window of seven days or more (refcore's `c_weekday`). L §4.2, §4.4.
    pub(super) fn weekday(
        &mut self,
        m: &Reality,
        call: &Call,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let [d] = self
            .args(m, call, env, ctx)?
            .try_into()
            .map_err(|_| arity())?;
        if let Value::Hole(_) = d {
            return Ok(d);
        }
        let day = date_or_window(d, "weekday")?;
        if let Value::Window(w) = &day {
            return window_weekday(w);
        }
        Ok(Value::int(i64::from(a_date(&day)?.weekday())))
    }

    /// `dcf(a, b, dc)`: the year fraction from a to b under day count dc, exact; `act365`
    /// gives the days from a, counted, to b, not, over 365; b before a the negative of
    /// dcf(b, a) (refcore's `m_c_dcf`, `m_dcf_q`). L §3.3.
    pub(super) fn dcf(
        &mut self,
        m: &Reality,
        call: &Call,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let [a, b, dc] = self
            .args(m, call, env, ctx)?
            .try_into()
            .map_err(|_| arity())?;
        if let Some(h) = crate::hole::spread(&[a.as_hole(), b.as_hole(), dc.as_hole()]) {
            return Ok(Value::Hole(h));
        }
        let from = match dcf_day(m, &a)? {
            Ok(d) => d,
            Err(hole) => return Ok(Value::Hole(hole)),
        };
        let to = match dcf_day(m, &b)? {
            Ok(d) => d,
            Err(hole) => return Ok(Value::Hole(hole)),
        };
        let word = if let Value::Code(Code::Word(w)) | Value::Word(w) = &dc {
            w.clone()
        } else {
            dc.text(m).map_err(num_stop)?
        };
        if word != "act365" {
            return Err(refuse_load(
                Load::Unit,
                format!("dcf: {word} is not a daycount code"),
            ));
        }
        let days = from
            .days_to(to)
            .ok_or_else(|| invariant("a day count past the integers"))?;
        let q = Num::int(days)
            .checked_div(&Num::int(365))
            .map_err(num_stop)?;
        Ok(Value::Number(Number::plain(q, Unit::one())))
    }
}

/// A call with the wrong number of arguments: load refuses it, so here it is an invariant.
fn arity() -> Stop {
    invariant("a built-in called with the wrong number of arguments")
}

/// The value of date built-in `f` at `args`; where one is a window, the interval of its
/// values at the window's first and last day, `HOLE-IMPRECISE` where two are or where it is
/// `none` at an end (refcore's `lift`). L §4.4.
fn lifted(m: &Reality, f: DateFn, args: &[Value]) -> Result<Value, Stop> {
    let windows: Vec<&Window> = args.iter().filter_map(as_window).collect();
    let window = match windows.as_slice() {
        [] => return apply(f, args),
        [one] => *one,
        [first, ..] => {
            return Ok(Value::Hole(imprecise(
                m,
                first,
                "a built-in of two windows",
            )?));
        }
    };
    let lows: Vec<Value> = args.iter().map(|v| window_end(v, End::Low)).collect();
    let highs: Vec<Value> = args.iter().map(|v| window_end(v, End::High)).collect();
    let (low, high) = (apply(f, &lows)?, apply(f, &highs)?);
    if low == Value::None || high == Value::None {
        let what = "a built-in that is none on part of the window";
        return Ok(Value::Hole(imprecise(m, window, what)?));
    }
    interval(m, (low, high), window)
}

/// Date built-in `f` at days (refcore's lambdas under `lift`). L §4.2.
fn apply(f: DateFn, args: &[Value]) -> Result<Value, Stop> {
    let missing = || invariant("a period past the calendar's range");
    match (f, args) {
        (DateFn::Start, [d]) => Ok(Value::Day(a_date(d)?)),
        (DateFn::End, [d]) => a_day(a_date(d)?)?
            .next()
            .map(Value::Day)
            .ok_or_else(|| past_days(1)),
        (DateFn::Year, [d]) => Period::year_of(a_date(d)?)
            .map(Value::Period)
            .ok_or_else(missing),
        (DateFn::Month, [d]) => Period::month_of(a_date(d)?)
            .map(Value::Period)
            .ok_or_else(missing),
        (DateFn::Quarter, [d]) => Period::quarter_of(a_date(d)?)
            .map(Value::Period)
            .ok_or_else(missing),
        (DateFn::Between, [a, b]) => a_date(a)?
            .days_to(a_date(b)?)
            .map(Value::int)
            .ok_or_else(|| invariant("a day count past the integers")),
        (DateFn::AddDays(n), [d]) => a_date(d)?
            .add_days(n)
            .map(Value::Day)
            .ok_or_else(|| past_days(n)),
        (DateFn::AddMonths(n), [d]) => a_date(d)?
            .add_months(n)
            .map(Value::Day)
            .ok_or_else(|| past_days(n)),
        (DateFn::Overlap(period), [a, b]) => overlap_days(a_date(a)?, a_date(b)?, period)
            .map(Value::int)
            .ok_or_else(|| invariant("a day count past the integers")),
        (DateFn::Age, [b, d]) => {
            Ok(age(a_day(a_date(b)?)?, a_day(a_date(d)?)?).map_or(Value::None, Value::int))
        }
        (DateFn::AgeMonths, [b, d]) => {
            Ok(age_months(a_day(a_date(b)?)?, a_day(a_date(d)?)?).map_or(Value::None, Value::int))
        }
        _ => Err(arity()),
    }
}

/// The day of a value [`date_or_window`] made, which is no window. L §4.2.
fn a_date(v: &Value) -> Result<Day, Stop> {
    if let Value::Day(d) = v {
        return Ok(*d);
    }
    Err(not_typed())
}

/// The weekdays of window `w`'s days: one when they all fall on it, else their interval, with
/// the window's text and no read (refcore's `c_weekday`, its window). L §4.2, §4.4.
fn window_weekday(w: &Window) -> Result<Value, Stop> {
    let (first, last) = (a_date(&w.lo)?, a_date(&w.hi)?);
    let span = first
        .days_to(last)
        .ok_or_else(|| invariant("a window's length"))?;
    let all_week = w.days.is_none() && span >= 6;
    let weekdays: BTreeSet<u8> = if all_week {
        BTreeSet::from([1, 7])
    } else if let Some(days) = &w.days {
        days.iter().map(|d| d.weekday()).collect()
    } else {
        // stop at the last day before a step: a window may end on 9999-12-31
        let mut out = BTreeSet::from([last.weekday()]);
        let mut day = first;
        while day < last {
            out.insert(day.weekday());
            day = day.next().ok_or_else(|| past_days(1))?;
        }
        out
    };
    let (Some(lo), Some(hi)) = (weekdays.first(), weekdays.last()) else {
        return Err(invariant("a window with no day"));
    };
    if lo == hi {
        return Ok(Value::int(i64::from(*lo)));
    }
    Ok(Value::Window(Box::new(Window {
        lo: Value::int(i64::from(*lo)),
        hi: Value::int(i64::from(*hi)),
        days: None,
        src: w.src.clone(),
        figure: None,
        read: None,
    })))
}

/// A period argument: a period, a day as its day period, or a four-digit figure written as
/// the argument, which the checker read as a year (refcore's `period_arg`, and the `_year`
/// mark of `tdate_or_period`); another value, `never` or a window among them, is refcore's
/// `LOAD-UNIT <what> is <v>, not a period`, met at run. L §2.1, §4.2, §8.4.
pub(super) fn period_arg(
    m: &Reality,
    (arg, v): (Option<&Expr>, &Value),
    what: &str,
) -> Result<Period, Stop> {
    if let Value::Period(p) = v {
        return Ok(*p);
    }
    if let Some(year) = as_year(arg, v) {
        return Ok(year);
    }
    if let Value::Day(d) = v {
        return Period::day(a_day(*d)?).ok_or_else(|| invariant("a day period"));
    }
    if *v == Value::None {
        return Err(refuse(Run::None, format!("a value is none here: {what}")));
    }
    let shown = v.text(m).map_err(num_stop)?;
    Err(refuse_load(
        Load::Unit,
        format!("{what} is {shown}, not a period"),
    ))
}

/// A date argument, or a window: a day, a day period's day (refcore's `as_date_or_iv`).
/// L §4.2, §4.4.
fn date_or_window(v: Value, what: &str) -> Result<Value, Stop> {
    if let Value::Day(_) | Value::Window(_) = v {
        return Ok(v);
    }
    if let Value::Period(p) = v
        && p.kind() == PeriodKind::Day
    {
        return Ok(Value::Day(p.start()));
    }
    if v == Value::None {
        return Err(refuse(Run::None, format!("a value is none here: {what}")));
    }
    if v == Value::Never {
        return Err(refuse(Run::None, format!("never in {what}")));
    }
    Err(not_typed())
}

/// A date of `dcf`: a day, a day period's day; a window is `HOLE-IMPRECISE` (refcore's
/// `m_as_day`). L §3.3, §4.4.
fn dcf_day(m: &Reality, v: &Value) -> Result<Result<Day, crate::hole::Hole>, Stop> {
    if let Value::Window(w) = v {
        return Ok(Err(imprecise(m, w, "dcf")?));
    }
    if let Value::Period(p) = v
        && p.kind() == PeriodKind::Day
    {
        return Ok(Ok(p.start()));
    }
    if let Value::Day(d) = v {
        return Ok(Ok(*d));
    }
    if *v == Value::None {
        return Err(refuse(Run::None, "a value is none here: dcf".to_owned()));
    }
    let shown = v.text(m).map_err(num_stop)?;
    Err(refuse_load(
        Load::Unit,
        format!("dcf: {shown} is not a date"),
    ))
}

/// The months of `add_months`'s or `add_years`'s count: an int, or a duration of its period,
/// a year 12 months (refcore's `months_arg`). L §4.2.
fn months_arg(m: &Reality, v: &Value, per: &str) -> Result<i64, Stop> {
    let mult = if per == "year" { 12 } else { 1 };
    if let Value::Number(Number {
        q,
        class: Class::Num(unit, _),
    }) = v
        && !unit.is_one()
    {
        if unit.terms() != [(per.to_owned(), 1)] {
            return Err(refuse_load(
                Load::Unit,
                format!("a duration of {} for add_{per}s", unit.text()),
            ));
        }
        return step_count(
            m,
            &Value::Number(Number::plain(q.clone(), Unit::one())),
            mult,
        );
    }
    step_count(m, v, mult)
}
