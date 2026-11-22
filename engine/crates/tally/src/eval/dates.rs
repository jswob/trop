//! The date built-ins: period edges, a day's period, day counts, moved days, ages, weekdays and the year fraction, and which of them take a window.
//! Spec: L §4.2 (`start_of`, `end_of`, `year_of`, `quarter_of`, `month_of`, `day_of`,
//! `days_between`, `add_days`, `add_months`, `add_years`, `overlap_days`, `age`, `age_months`,
//! `weekday`), §4.4 (the readers that take a window: `add_days`, `add_months`, `add_years` the
//! stretch from their result at its first day to their result at its last; `year_of`,
//! `quarter_of`, `month_of`, `age`, `age_months`, `overlap_days` the value every combination of
//! their arguments' first and last days gives; every other read the hole), §3.3 (`dcf` with
//! `act365`), §2.1 (a four-digit figure where a period is read), §7.6 item 1.
//! Twin: engine/tools/refcore/evaluate.py (`c_start_of` to `c_weekday`, `as_date_or_iv`,
//! `period_arg`, `months_arg`, `lift`) and eval2.py (`m_c_dcf`, `m_dcf_q`, `m_as_day`).
//! Invariants: a hole argument makes the call that hole; a ranged date is read only by the
//! listed built-ins, each at its ends ([`super::windows::takes`]); any other built-in of one,
//! or a ranged date in a position no listed built-in reads it at, is `HOLE-IMPRECISE` naming
//! the built-in and each unknown day.

use super::arith::whole_of;
use super::expr::hole_of;
use super::state::{Ctx, Engine, Env, invariant, num_stop, refuse, refuse_load};
use super::state::{a_day, not_typed, past_days};
use super::time_arith::step_count;
use super::windows::{Takes, agreed, by_ends, corners, ranged, single, takes, unlisted};
use super::years::as_year;
use crate::base::{Load, Run};
use crate::model::{Class, Code, Number, Reality, Unit, Value, Window};
use crate::num::Num;
use crate::syntax::{Call, Expr, Stop};
use crate::time::{Day, Period, PeriodKind, age, age_months, overlap_days};

/// A date built-in at days, which a listed reader takes at a window's ends (L §4.2, §4.4;
/// refcore's lambdas under `lift`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DateFn {
    /// A day's `start_of`: the day.
    Start,
    /// A day's `end_of`: the day after.
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
            return read_dates((f, name), &[v]);
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
    /// month-day in year y (refcore's `c_day_of`). L §4.2, §4.4.
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
        // off the list: a ranged date is the hole (L §4.4)
        if let Some(hole) = unlisted(&[&a, &b], "day_of") {
            return Ok(Value::Hole(hole));
        }
        // a window of one day is that day (F3)
        let (one_a, number) = (single(&a), single(&b));
        let period = period_arg(m, (call.args.first(), &one_a), "day_of")?;
        if !matches!(*number, Value::MonthDay(..)) {
            return day_of_month(m, period, &number);
        }
        if period.kind() == PeriodKind::Year
            && (period.start().month(), period.start().day()) != (1, 1)
        {
            return Err(anchored(period));
        }
        let Value::MonthDay(month, day) = *number else {
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
    /// `age(b, d)`, `age_months(b, d)`, each of a window by its rule (refcore's
    /// `c_days_between`, `c_add_days`, `c_add_months`, `c_add_years`, `c_age`). L §4.2, §4.4.
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
        if matches!(name, "add_days" | "add_months" | "add_years") {
            return moved_by(m, name, first, &b);
        }
        let f = match name {
            "days_between" => DateFn::Between,
            "age_months" => DateFn::AgeMonths,
            "age" => DateFn::Age,
            _ => return Err(invariant("a day-count built-in dispatched by another name")),
        };
        // each argument by its own ends, whatever names they carry: one window named twice is
        // two (L §4.4)
        read_dates((f, name), &[first, date_or_window(b, name)?])
    }

    /// `overlap_days(a, b, k)`: the days from a to b, both counted, that lie in period k; of
    /// windows, the count every combination of their ends gives (refcore's `c_overlap_days`).
    /// L §4.2, §4.4.
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
        // its period is read at no end: a ranged one is the hole (L §4.4)
        if ranged(&k).is_some() {
            return Ok(Value::Hole(hole_naming(&[&from, &to, &k], "overlap_days")?));
        }
        let period = period_arg(m, (call.args.get(2), &single(&k)), "overlap_days")?;
        read_dates((DateFn::Overlap(period), "overlap_days"), &[from, to])
    }

    /// `weekday(d)`: 1 for Monday to 7 for Sunday; off the list, so a ranged date is the hole
    /// (refcore's `c_weekday`). L §4.2, §4.4.
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
        if let Some(hole) = unlisted(&[&day], "weekday") {
            return Ok(Value::Hole(hole));
        }
        Ok(Value::int(i64::from(a_date(&single(&day))?.weekday())))
    }

    /// `dcf(a, b, dc)`: the year fraction from a to b under day count dc, exact; `act365`
    /// gives the days from a, counted, to b, not, over 365; b before a the negative of
    /// dcf(b, a); off the list, so a ranged date is the hole (refcore's `m_c_dcf`, `m_dcf_q`;
    /// C3). L §3.3, §4.4.
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
        // refusals outrank holes (L §8; pick P12): each argument in written order, a known
        // one's own check run (`none` among them), a hole or a window's waiting
        for day in [&a, &b] {
            let day = single(day);
            if day.as_hole().is_none() && !matches!(*day, Value::Window(_)) {
                dcf_day(m, &day)?;
            }
        }
        if dc.as_hole().is_none() {
            daycount_of(m, &dc)?;
        }
        if let Some(h) = crate::hole::spread(&[a.as_hole(), b.as_hole(), dc.as_hole()]) {
            return Ok(Value::Hole(h));
        }
        if let Some(hole) = unlisted(&[&a, &b], "dcf") {
            return Ok(Value::Hole(hole));
        }
        let q = year_fraction_of(m, (&single(&a), &single(&b)), &dc)?;
        Ok(Value::Number(Number::plain(q, Unit::one())))
    }
}

/// The year fraction from date a to date b under day count `dc`, exact: `act365` gives the
/// days from a, counted, to b, not, over 365, and b before a the negative of dcf(b, a)
/// (refcore's `m_dcf_q`). L §3.3.
///
/// # Errors
///
/// `LOAD-UNIT` for a value that is no date or no daycount code; `RUN-NONE` for `none`.
pub(super) fn year_fraction_of(
    m: &Reality,
    (a, b): (&Value, &Value),
    dc: &Value,
) -> Result<Num, Stop> {
    let (from, to) = (dcf_day(m, a)?, dcf_day(m, b)?);
    daycount_of(m, dc)?;
    let days = from
        .days_to(to)
        .ok_or_else(|| invariant("a day count past the integers"))?;
    Num::int(days).checked_div(&Num::int(365)).map_err(num_stop)
}

/// A day count code `dc`: `act365`, the one L §3.3 states; anything else is `LOAD-UNIT dcf:
/// <dc> is not a daycount code` (refcore's `m_dcf_q`). L §3.3.
///
/// # Errors
///
/// `LOAD-UNIT` for any other value.
pub(super) fn daycount_of(m: &Reality, dc: &Value) -> Result<(), Stop> {
    let word = if let Value::Code(Code::Word(w)) | Value::Word(w) = dc {
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
    Ok(())
}

/// A call with the wrong number of arguments: load refuses it, so here it is an invariant.
fn arity() -> Stop {
    invariant("a built-in called with the wrong number of arguments")
}

/// The hole of a read off the list of the ranged dates among `args`, for `what` (L §4.4).
///
/// # Errors
///
/// `ENGINE-INVARIANT` where none is ranged.
fn hole_naming(args: &[&Value], what: &str) -> Result<crate::hole::Hole, Stop> {
    unlisted(args, what).ok_or_else(|| invariant("a ranged argument's hole with no window"))
}

/// `add_days(d, n)`, `add_months(d, n)` or `add_years(d, n)`, `name`, of a day or a window
/// `first` and a count `b`; a count is read at no end, so a ranged one is the hole (L §4.2,
/// §4.4: no number is a range).
///
/// # Errors
///
/// As [`moved`].
fn moved_by(m: &Reality, name: &str, first: Value, b: &Value) -> Result<Value, Stop> {
    if ranged(b).is_some() {
        return Ok(Value::Hole(hole_naming(&[&first, b], name)?));
    }
    moved(m, name, first, &single(b))
}

/// `add_days(d, n)`, `add_months(d, n)` or `add_years(d, n)`, `name`, of a day or a window
/// `first` and a count `n` (refcore's `c_add_days`, `c_add_months`, `c_add_years`, their
/// lambdas). L §4.2, §4.4.
///
/// # Errors
///
/// The refusals of reading the count; `RUN-LIMIT` past the calendar.
fn moved(m: &Reality, name: &str, first: Value, n: &Value) -> Result<Value, Stop> {
    let f = match name {
        "add_days" => DateFn::AddDays(step_count(m, n, 1)?),
        "add_months" => DateFn::AddMonths(months_arg(m, n, "month")?),
        "add_years" => DateFn::AddMonths(months_arg(m, n, "year")?),
        _ => return Err(invariant("a moved day dispatched by another name")),
    };
    read_dates((f, name), &[first])
}

/// `day_of` of a key of an anchored index: `RUN-DOMAIN` (L §4.2).
fn anchored(period: Period) -> Stop {
    refuse(
        Run::Domain,
        format!("day_of of {}, a key of an anchored index", period.label()),
    )
}

/// `day_of(m, n)`: the n-th day of month m, clamped to its days (refcore's `day_of_month`).
/// L §4.2.
///
/// # Errors
///
/// `RUN-DOMAIN` for a key of an anchored index or a number below 1; the refusals of reading a
/// whole number.
fn day_of_month(m: &Reality, period: Period, b: &Value) -> Result<Value, Stop> {
    let number = whole_of(m, b)?;
    if period.kind() != PeriodKind::Month || period.start().day() != 1 {
        return Err(anchored(period));
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
        Some(whole) => {
            u8::try_from(whole.min(i64::from(last.day()))).map_err(|_| invariant("a day number"))?
        }
        None => last.day(),
    };
    Day::from_ymd(last.year(), last.month(), day)
        .map(Value::Day)
        .ok_or_else(|| invariant("a day of a month"))
}

/// Date built-in `f`, called `name`, at `args`: at days, its value; where some are ranged,
/// by how the list says `name` takes them ([`takes`]): date to date, the stretch from its
/// value at the window's first day to its value at its last; a plain result, the value every
/// combination of the ranged arguments' first and last days gives, each by its own ends; off
/// the list, the hole for `name` naming each unknown day (refcore's `lift`). L §4.4.
fn read_dates((f, name): (DateFn, &str), args: &[Value]) -> Result<Value, Stop> {
    if !args.iter().any(|v| matches!(v, Value::Window(_))) {
        return apply(f, args);
    }
    // a window of one day is that day (F3)
    let ones: Vec<Value> = args.iter().map(|v| single(v).into_owned()).collect();
    let refs: Vec<&Value> = ones.iter().collect();
    let windows: Vec<&Window> = refs.iter().filter_map(|v| ranged(v)).collect();
    let Some(first) = windows.first() else {
        return apply(f, &ones);
    };
    match takes(name) {
        Some(Takes::Ends) => at_ends((f, name), &refs, first),
        Some(Takes::Agreed) => at_corners((f, name), &refs, &windows),
        None => hole_naming(&refs, name).map(Value::Hole),
        Some(Takes::Order | Takes::LawDate | Takes::Print | Takes::Pin) => Err(invariant(
            "a date built-in the list names as no date reader",
        )),
    }
}

/// Date built-in `f`, called `name`, a date-to-date reader (`Takes::Ends`), at `refs`, one of
/// them ranged, `first`: the stretch from its value at the window's first day to its value at
/// its last ([`by_ends`]). L §4.4.
///
/// # Errors
///
/// As [`by_ends`]; `ENGINE-INVARIANT` for a reader of more than one ranged date.
fn at_ends((f, name): (DateFn, &str), refs: &[&Value], first: &Window) -> Result<Value, Stop> {
    let mut outs = corners(refs).into_iter().map(|t| apply(f, &t));
    let (Some(lo), Some(hi), None) = (outs.next(), outs.next(), outs.next()) else {
        return Err(invariant("a date-to-date reader of more than one date"));
    };
    by_ends((lo, hi), first, name)
}

/// Date built-in `f`, called `name`, a plain result (`Takes::Agreed`), at `refs`: the value
/// every combination of the ranged arguments' first and last days gives ([`agreed`]), each
/// window of `windows` by its own ends. L §4.4.
///
/// # Errors
///
/// As [`agreed`].
fn at_corners(
    (f, name): (DateFn, &str),
    refs: &[&Value],
    windows: &[&Window],
) -> Result<Value, Stop> {
    let outs = corners(refs).iter().map(|t| apply(f, t)).collect();
    agreed(outs, windows, name)
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

/// A period argument: a period, a day as its day period, or a four-digit figure written as
/// the argument, which the checker read as a year (refcore's `period_arg`, and the `_year`
/// mark of `tdate_or_period`); another value, `never` among them, is refcore's
/// `LOAD-UNIT <what> is <v>, not a period`, met at run. A caller asks for a ranged date's
/// hole first. L §2.1, §4.2, §8.4.
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

/// The day a date of `dcf` is: a day, a day period's day (refcore's `m_as_day`). L §3.3.
fn dcf_day(m: &Reality, v: &Value) -> Result<Day, Stop> {
    if let Value::Period(p) = v
        && p.kind() == PeriodKind::Day
    {
        return Ok(p.start());
    }
    if let Value::Day(d) = v {
        return Ok(*d);
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

#[cfg(test)]
mod tests {
    use super::{DateFn, read_dates};
    use crate::model::{Value, Window};
    use crate::syntax::Stop;
    use crate::time::Day;

    fn day(y: u16, m: u8, d: u8) -> Day {
        Day::from_ymd(y, m, d).expect("a real day")
    }

    fn window((lo, hi): (Day, Day), name: &str) -> Value {
        Value::Window(Box::new(Window {
            lo,
            hi,
            src: Some(format!("window({}, {})", lo.text(), hi.text())),
            figure: Some(name.to_owned()),
            read: None,
            carry: vec![crate::hole::Unknown {
                id: format!("row {name}"),
                label: name.to_owned(),
                text: format!("window({}, {})", lo.text(), hi.text()),
            }],
        }))
    }

    fn text(v: &Result<Value, Stop>) -> String {
        match v {
            Ok(Value::Hole(h)) => h.text(),
            Ok(Value::Day(d)) => d.text(),
            Ok(Value::Window(w)) => format!("window({}, {})", w.lo.text(), w.hi.text()),
            Ok(Value::Number(n)) => format!("{:?}", n.q.to_i64()),
            Ok(Value::Period(p)) => p.label(),
            Ok(Value::None) => "none".to_owned(),
            Ok(_) => "another value".to_owned(),
            Err(e) => format!("{e:?}"),
        }
    }

    #[test]
    fn a_listed_reader_reads_a_window_at_its_ends() {
        let wb = window((day(2024, 2, 27), day(2024, 3, 2)), "wb");
        let wc = window((day(2024, 1, 5), day(2024, 1, 25)), "wc");
        // date to date: the stretch of its value at the first day and at the last
        let moved = read_dates(
            (DateFn::AddMonths(12), "add_months"),
            core::slice::from_ref(&wb),
        );
        assert_eq!(text(&moved), "window(2025-02-27, 2025-03-02)", "add_months");
        // a plain result: the value the ends agree on, else the hole naming the reader
        let year = read_dates((DateFn::Year, "year_of"), core::slice::from_ref(&wb));
        assert_eq!(text(&year), "2024", "year_of");
        let month = read_dates((DateFn::Month, "month_of"), core::slice::from_ref(&wb));
        assert_eq!(
            text(&month),
            "?\"date within window(2024-02-27, 2024-03-02) not precise enough for month_of\" (wb)",
            "month_of"
        );
        let age = read_dates((DateFn::Age, "age"), &[Value::Day(day(2010, 6, 1)), wc]);
        assert_eq!(text(&age), "Some(13)", "age");
        // none too, where every end gives it
        let late = window((day(2026, 11, 2), day(2026, 11, 9)), "wd");
        let none = read_dates((DateFn::Age, "age"), &[late, Value::Day(day(2026, 1, 1))]);
        assert_eq!(text(&none), "none", "age before birth");
    }

    #[test]
    fn a_reader_off_the_list_is_the_hole() {
        let wd = window((day(2026, 11, 2), day(2026, 11, 9)), "wd");
        let start = read_dates((DateFn::Start, "start_of"), core::slice::from_ref(&wd));
        assert_eq!(
            text(&start),
            "?\"date within window(2026-11-02, 2026-11-09) not precise enough for start_of\" (wd)",
            "start_of"
        );
        let between = read_dates(
            (DateFn::Between, "days_between"),
            &[wd, Value::Day(day(2026, 12, 1))],
        );
        assert!(
            text(&between).contains("not precise enough for days_between"),
            "days_between: {}",
            text(&between)
        );
        // a window of one day is that day
        let one = window((day(2026, 11, 4), day(2026, 11, 4)), "w1");
        let one_day = read_dates((DateFn::Start, "start_of"), &[one]);
        assert_eq!(text(&one_day), "2026-11-04", "one day");
    }

    #[test]
    fn one_window_in_two_arguments_goes_by_each_arguments_ends() {
        let w = window((day(2026, 3, 2), day(2026, 3, 20)), "w");
        // days_between(w, w), age(w, w): not one day, each argument by its own ends
        let between = read_dates((DateFn::Between, "days_between"), &[w.clone(), w.clone()]);
        let age = read_dates((DateFn::Age, "age"), &[w.clone(), w.clone()]);
        let hole = |what: &str| {
            format!(
                "?\"date within window(2026-03-02, 2026-03-20) not precise enough for {what}\" (w)"
            )
        };
        assert_eq!(text(&between), hole("days_between"), "days_between(w, w)");
        assert_eq!(text(&age), hole("age"), "age(w, w)");
        // its corners: four, first argument outermost
        let refs = [&w, &w];
        let corners = super::corners(&refs);
        assert_eq!(corners.len(), 4, "two ranged arguments, four corners");
    }

    #[test]
    fn a_listed_reader_refused_at_every_end_is_the_first_refusal() {
        let late = window((day(9999, 12, 20), day(9999, 12, 30)), "late");
        let past = read_dates(
            (DateFn::AddDays(30), "add_days"),
            core::slice::from_ref(&late),
        );
        assert!(
            matches!(past, Err(Stop::Refused(_))),
            "refused at both ends: {}",
            text(&past)
        );
        let some = read_dates((DateFn::AddDays(5), "add_days"), &[late]);
        assert!(
            text(&some).contains("not precise enough for add_days"),
            "refused at one end: {}",
            text(&some)
        );
    }
}
