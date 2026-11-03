//! The built-ins S's realities call: the calendar's, dates and periods, `round`, `min` and `max`, `abs`, `pos`, `age`, `missing`, and a rate's `periodic` and `factor`.
//! Spec: L §4.2 (calendar built-ins: `start_of`, `end_of`, `day_of`, `year_of`, `month_of`,
//! `days_between`, `add_days`, `add_months`, `age`, `pos`, `is_business`, `add_business`),
//! §7.2 (`round` and the type of its result), §5.11 (`min`, `max`, `missing`), §3.3
//! (`periodic`, `factor`: exact when the power is an integer), §8.1 (`HOLE-CALENDAR`).
//! Twin: engine/tools/refcore/evaluate.py (`c_start_of` to `c_missing`, `business`,
//! `c_is_business`, `c_add_business`) and eval2.py (`m_rate_parts`, `m_periodic_real`).
//! Invariants: a hole argument makes the call that hole; a business day is read from the
//! calendar in force on that day; a power that isn't an integer stops `NotBuilt` (the
//! transcendental kernel isn't S's).

use super::arith::{Test, compare, int_of};
use super::expr::hole_of;
use super::read::{Given, make_keys};
use super::state::{Ctx, Engine, Env, invariant, num_stop, refuse, refuse_load, wrong_type};
use crate::base::{Load, Run};
use crate::model::{
    Class, Code, Domain, KeyVal, Number, Reality, Shown, Unit, Value, currency_unit, money_num,
    num_money,
};
use crate::num::{Mode, Num, NumError};
use crate::read::{Asked, Found, read_input};
use crate::syntax::{Call, Expr, Key, Postfix, Stop, Value as Written};
use crate::time::{Day, Period, PeriodKind, age};

/// The most days `add_business` steps before it stops: past it, no calendar of S's realities
/// has that many days off in a row.
const BUSINESS_STEPS: i64 = 4_000;

impl Engine {
    /// A call: a band's, or a built-in's (refcore's `e_call`). L §4.2, §5.9, §5.11, §7.2.
    pub(super) fn call(
        &mut self,
        m: &Reality,
        call: &Call,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        if m.bands.get(&call.name).is_some() {
            return self.band_call(m, call, env, ctx);
        }
        match call.name.as_str() {
            "start_of" | "end_of" | "year_of" | "month_of" | "quarter_of" => {
                self.period_edge(m, call, env, ctx)
            }
            "day_of" => self.day_of(m, call, env, ctx),
            "days_between" | "add_days" | "add_months" | "age" => self.day_count(m, call, env, ctx),
            "is_business" | "add_business" => self.business_call(m, call, env, ctx),
            "round" => self.round(m, call, env, ctx),
            "min" | "max" => self.extreme(m, call, env, ctx),
            "abs" => self.abs(m, call, env, ctx),
            "minor" => minor_of(m, call),
            "pos" => self.pos(m, call, env, ctx),
            "missing" => self.missing(m, call, env, ctx),
            "periodic" | "factor" => self.rate_call(m, call, env, ctx),
            _ => Err(Stop::NotBuilt("a built-in S's realities don't call")),
        }
    }

    /// The arguments of a call, evaluated in order.
    fn args(&mut self, m: &Reality, call: &Call, env: &Env, ctx: Ctx) -> Result<Vec<Value>, Stop> {
        let mut out = Vec::with_capacity(call.args.len());
        for arg in &call.args {
            out.push(self.ev(m, arg, env, ctx)?);
        }
        Ok(out)
    }

    /// `start_of(p)`, `end_of(p)` (the first day after p), `year_of(d)`, `month_of(d)`,
    /// `quarter_of(d)` (refcore's `c_start_of`, `c_end_of`, `_of`). L §4.2.
    fn period_edge(
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
        let period = period_arg(m, &v, name)?;
        let missing_day = || invariant("a period past the calendar's range");
        match name {
            "start_of" => Ok(Value::Day(period.start())),
            "end_of" => Ok(Value::Day(period.end())),
            "year_of" => Period::year_of(period.start())
                .map(Value::Period)
                .ok_or_else(missing_day),
            "month_of" => Period::month_of(period.start())
                .map(Value::Period)
                .ok_or_else(missing_day),
            _ => Period::quarter_of(period.start())
                .map(Value::Period)
                .ok_or_else(missing_day),
        }
    }

    /// `day_of(m, n)`, the n-th day of month m clamped to its days; `day_of(y, md)`, a
    /// month-day in year y (refcore's `c_day_of`). L §4.2.
    fn day_of(&mut self, m: &Reality, call: &Call, env: &Env, ctx: Ctx) -> Result<Value, Stop> {
        let [a, b] = self
            .args(m, call, env, ctx)?
            .try_into()
            .map_err(|_| arity())?;
        if a.as_hole().is_some() || b.as_hole().is_some() {
            return hole_of(&[a.as_hole(), b.as_hole()]);
        }
        let period = period_arg(m, &a, "day_of")?;
        let anchored = || {
            refuse(
                Run::Domain,
                format!("day_of of {}, a key of an anchored index", period.label()),
            )
        };
        if !matches!(b, Value::MonthDay(..)) {
            let k = int_of(m, &b)?;
            if period.kind() != PeriodKind::Month || period.start().day() != 1 {
                return Err(anchored());
            }
            if k < 1 {
                return Err(refuse(
                    Run::Domain,
                    format!("day_of(m, {k}): a day number below 1"),
                ));
            }
            let last = period
                .end()
                .previous()
                .ok_or_else(|| invariant("a month with no last day"))?;
            let day = u8::try_from(k.min(i64::from(last.day())))
                .map_err(|_| invariant("a day number"))?;
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
            return Err(refuse_load(
                Load::Unit,
                "day_of(y, md) takes a month-day".to_owned(),
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

    /// `days_between(a, b)`, `add_days(d, n)`, `add_months(d, n)`, `age(b, d)` (refcore's
    /// `c_days_between`, `c_add_days`, `c_add_months`, `c_age`). L §4.2.
    fn day_count(&mut self, m: &Reality, call: &Call, env: &Env, ctx: Ctx) -> Result<Value, Stop> {
        let [a, b] = self
            .args(m, call, env, ctx)?
            .try_into()
            .map_err(|_| arity())?;
        if a.as_hole().is_some() || b.as_hole().is_some() {
            return hole_of(&[a.as_hole(), b.as_hole()]);
        }
        let name = call.name.as_str();
        let first = date_arg(m, &a, name)?;
        let past = || invariant("a day past the calendar's range");
        match name {
            "days_between" => {
                let second = date_arg(m, &b, name)?;
                first.days_to(second).map(Value::int).ok_or_else(past)
            }
            "add_days" => first
                .add_days(int_of(m, &b)?)
                .map(Value::Day)
                .ok_or_else(past),
            "add_months" => first
                .add_months(months_arg(m, &b)?)
                .map(Value::Day)
                .ok_or_else(past),
            _ => {
                let second = date_arg(m, &b, name)?;
                Ok(age(first, second).map_or(Value::None, Value::int))
            }
        }
    }

    /// `is_business(d, cal)` and `add_business(d, n, cal)`: the calendar read in force on
    /// each day asked about (refcore's `c_is_business`, `c_add_business`, `business`). L §4.2.
    fn business_call(
        &mut self,
        m: &Reality,
        call: &Call,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let cal = call.args.last().ok_or_else(arity)?;
        let d = self.ev(m, call.args.first().ok_or_else(arity)?, env, ctx)?;
        if call.name == "is_business" {
            if let Value::Hole(_) = d {
                return Ok(d);
            }
            let day = date_arg(m, &d, "is_business")?;
            return self.business(m, cal, env, ctx, day);
        }
        let k = self.ev(m, call.args.get(1).ok_or_else(arity)?, env, ctx)?;
        if d.as_hole().is_some() || k.as_hole().is_some() {
            return hole_of(&[d.as_hole(), k.as_hole()]);
        }
        let day = date_arg(m, &d, "add_business")?;
        let k = int_of(m, &k)?;
        if k == 0 {
            return Err(refuse(
                Run::Zero,
                format!("add_business({}, 0, {})", day.text(), expr_name(cal)),
            ));
        }
        let step: i64 = if k > 0 { 1 } else { -1 };
        let mut left = k
            .checked_abs()
            .ok_or_else(|| invariant("a business-day count"))?;
        let mut current = day;
        let mut steps: i64 = 0;
        while left > 0 {
            steps = steps
                .checked_add(1)
                .ok_or_else(|| invariant("a business-day count"))?;
            if steps > BUSINESS_STEPS {
                return Err(Stop::NotBuilt(
                    "a calendar with thousands of days off in a row",
                ));
            }
            current = current
                .add_days(step)
                .ok_or_else(|| invariant("a day past the calendar's range"))?;
            let open = self.business(m, cal, env, ctx, current)?;
            let Value::Bool(is_open) = open else {
                return Ok(open);
            };
            if is_open {
                left = left
                    .checked_sub(1)
                    .ok_or_else(|| invariant("a business-day count"))?;
            }
        }
        Ok(Value::Day(current))
    }

    /// Whether `day` is a business day of the calendar `cal` reads, in force on that day
    /// (refcore's `cal_at`, `business`); the calendar's gap is `HOLE-CALENDAR`. L §4.2.
    fn business(
        &mut self,
        m: &Reality,
        cal: &Expr,
        env: &Env,
        ctx: Ctx,
        day: Day,
    ) -> Result<Value, Stop> {
        let Some((name, written)) = read_parts(cal) else {
            return Err(Stop::NotBuilt("a calendar that isn't an input's read"));
        };
        let input = m
            .inputs
            .get(name)
            .ok_or(Stop::NotBuilt("a calendar that isn't an input's read"))?;
        let given = self.given_keys(m, written, env, ctx)?;
        let keys = match make_keys(m, name, &input.binds, given, ctx.cut)? {
            Err(hole) => return Ok(Value::Hole(hole)),
            Ok(made) => made,
        };
        let value = self.read_input_at(m, input, &keys, ctx.at_law(Some(day)), Asked::Calendar)?;
        if let Value::Calendar(c) = &value {
            return Ok(Value::Bool(c.is_business(day)));
        }
        if let Value::Hole(_) = value {
            return Ok(value);
        }
        if value == Value::None {
            return Err(refuse(Run::None, "the calendar is none here".to_owned()));
        }
        Err(refuse_load(Load::Unit, format!("{name} is not a calendar")))
    }

    /// `round(x, u, mode)`: the multiple of u the mode chooses, money when u is a whole
    /// number of minor units of x's currency, an int for `round(x, 1, m)` of a plain number
    /// (refcore's `c_round` and check.py's `c_round` for the result's type). L §7.2.
    fn round(&mut self, m: &Reality, call: &Call, env: &Env, ctx: Ctx) -> Result<Value, Stop> {
        let [x, u, mode] = self
            .args(m, call, env, ctx)?
            .try_into()
            .map_err(|_| arity())?;
        if x.as_hole().is_some() || u.as_hole().is_some() || mode.as_hole().is_some() {
            return hole_of(&[x.as_hole(), u.as_hole(), mode.as_hole()]);
        }
        for v in [&x, &u, &mode] {
            if *v == Value::None {
                return Err(refuse(Run::None, "a value is none here: round".to_owned()));
            }
        }
        round_values(m, &x, &u, &mode)
    }

    /// `min(a, b, …)`, `max(a, b, …)`: a bare figure takes the unit of an argument with one
    /// (refcore's `c_min`, `coerce_flex`, `extreme`). L §5.11.
    fn extreme(&mut self, m: &Reality, call: &Call, env: &Env, ctx: Ctx) -> Result<Value, Stop> {
        let vals = self.args(m, call, env, ctx)?;
        let holes: Vec<Option<&crate::hole::Hole>> = vals.iter().map(Value::as_hole).collect();
        if holes.iter().any(Option::is_some) || vals.contains(&Value::None) {
            if vals.contains(&Value::None) {
                return Err(refuse(
                    Run::None,
                    format!("a value is none here: {}", call.name),
                ));
            }
            return hole_of(&holes);
        }
        let vals = coerce_flex(m, vals)?;
        let mut best: Option<Value> = None;
        for v in vals {
            best = Some(match best {
                None => v,
                Some(held) => {
                    let order = compare(m, &v, &held, Test::Order)?;
                    let better = match call.name.as_str() {
                        "min" => order == core::cmp::Ordering::Less,
                        _ => order == core::cmp::Ordering::Greater,
                    };
                    if better { v } else { held }
                }
            });
        }
        best.ok_or_else(arity)
    }

    /// `abs(x)` (refcore's `c_abs`). L §5.11.
    fn abs(&mut self, m: &Reality, call: &Call, env: &Env, ctx: Ctx) -> Result<Value, Stop> {
        let [v] = self
            .args(m, call, env, ctx)?
            .try_into()
            .map_err(|_| arity())?;
        if let Value::Hole(_) = v {
            return Ok(v);
        }
        if v == Value::None {
            return Err(refuse(Run::None, "a value is none here: abs".to_owned()));
        }
        if let Value::Money(money) = &v {
            if money.is_negative() {
                return Ok(Value::Money(money.checked_neg().map_err(num_stop)?));
            }
            return Ok(v);
        }
        let Value::Number(n) = v else {
            return Err(wrong_type(m, "abs of", &v));
        };
        if n.q.is_negative() {
            return Ok(Value::Number(Number {
                q: n.q.checked_neg().map_err(num_stop)?,
                class: n.class,
            }));
        }
        Ok(Value::Number(n))
    }

    /// `pos(k)`: a key's place in its index, from 1 (refcore's `c_pos`). L §4.2.
    fn pos(&mut self, m: &Reality, call: &Call, env: &Env, ctx: Ctx) -> Result<Value, Stop> {
        let arg = call.args.first().ok_or_else(arity)?;
        let v = self.ev(m, arg, env, ctx)?;
        if let Value::Hole(_) = v {
            return Ok(v);
        }
        if let Value::OutKey { index, text } = &v {
            return Err(refuse(Run::Key, format!("{text} is outside {index}")));
        }
        let from_var = if let Expr::Name(name) = arg
            && let Some(var) = env.get(name)
            && let Some(Domain::Index(ix)) = &var.domain
        {
            Some(ix.clone())
        } else {
            None
        };
        let index = if let Some(ix) = from_var {
            ix
        } else if let Value::Key(KeyVal::Key { index, .. }) = &v {
            index.clone()
        } else {
            return Err(Stop::NotBuilt("pos of a key whose index the run can't see"));
        };
        let ix = m
            .indexes
            .get(&index)
            .ok_or_else(|| invariant("a key of an index load didn't build"))?;
        let key = pos_key(m, &v)?;
        let at = ix
            .pos(&key)
            .ok_or_else(|| refuse(Run::Key, format!("{} is outside {}", key.text(), ix.id)))?;
        let at = i64::try_from(at)
            .map_err(|_| invariant("an index position"))?
            .checked_add(1)
            .ok_or_else(|| invariant("an index position"))?;
        Ok(Value::int(at))
    }

    /// `missing(r)`: whether the read r is a hole because no row gives it, or its row is a
    /// stated `?`; never itself a hole (refcore's `c_missing`). L §5.11.
    fn missing(&mut self, m: &Reality, call: &Call, env: &Env, ctx: Ctx) -> Result<Value, Stop> {
        let arg = call.args.first().ok_or_else(arity)?;
        let not_read = || Stop::NotBuilt("missing of a value that is not an input's read");
        let (name, written) = read_parts(arg).ok_or_else(not_read)?;
        let input = m.inputs.get(name).ok_or_else(not_read)?;
        let given = self.given_keys(m, written, env, ctx)?;
        let Ok(keys) = make_keys(m, name, &input.binds, given, ctx.cut)? else {
            return Ok(Value::Bool(false));
        };
        let value = self.read_input_at(m, input, &keys, ctx, Asked::Plain)?;
        if value.as_hole().is_none() {
            return Ok(Value::Bool(false));
        }
        let held = match read_input(m, input, &keys, self.clocks(ctx), Asked::Plain)? {
            Found::Hole(_) => true,
            Found::Row(at) => input
                .rows
                .get(at)
                .is_some_and(|row| matches!(row.value, Written::Hole(_))),
            Found::None => false,
        };
        if held {
            self.note_missing();
        }
        Ok(Value::Bool(held))
    }

    /// The keys of a read as written, each a value or a key literal (L §2.5).
    fn given_keys(
        &mut self,
        m: &Reality,
        keys: &[Key],
        env: &Env,
        ctx: Ctx,
    ) -> Result<Vec<Given>, Stop> {
        let mut given: Vec<Given> = Vec::with_capacity(keys.len());
        for key in keys {
            let Key::Expr(e) = key else {
                return Err(Stop::NotBuilt("`*` in a read's keys"));
            };
            given.push(self.given_key(m, e, env, ctx)?);
        }
        Ok(given)
    }

    /// `periodic(r, P, c)`, the rate per period P under compounding c, and `factor(r, n, P,
    /// c)`, (1 + periodic)^n; exact when the power is an integer (refcore's `rate_parts`,
    /// `periodic_real`). L §3.3.
    fn rate_call(&mut self, m: &Reality, call: &Call, env: &Env, ctx: Ctx) -> Result<Value, Stop> {
        let vals = self.args(m, call, env, ctx)?;
        let holes: Vec<Option<&crate::hole::Hole>> = vals.iter().map(Value::as_hole).collect();
        if holes.iter().any(Option::is_some) {
            return hole_of(&holes);
        }
        if vals.contains(&Value::None) {
            return Err(refuse(
                Run::None,
                format!("a value is none here: {}", call.name),
            ));
        }
        let (rate, count, period, code) = match vals.as_slice() {
            [rate, period, code] if call.name == "periodic" => (rate, None, period, code),
            [rate, count, period, code] => (rate, Some(count), period, code),
            _ => return Err(arity()),
        };
        let parts = rate_parts(m, rate, period, code, &call.name)?;
        let q = match (parts, count) {
            (Parts::Linear(per), None) => per,
            (Parts::Power(base, exponent), None) => power(&base, &exponent)?
                .checked_sub(&Num::int(1))
                .map_err(num_stop)?,
            (Parts::Linear(per), Some(times)) => {
                let base = Num::int(1).checked_add(&per).map_err(num_stop)?;
                power(&base, &plain_count(m, times)?)?
            }
            (Parts::Power(base, exponent), Some(times)) => {
                let exponent = exponent
                    .checked_mul(&plain_count(m, times)?)
                    .map_err(num_stop)?;
                power(&base, &exponent)?
            }
        };
        Ok(Value::Number(Number::plain(q, Unit::one())))
    }
}

/// A call with the wrong number of arguments: load refuses it, so here it is an invariant.
fn arity() -> Stop {
    invariant("a built-in called with the wrong number of arguments")
}

/// The name of a calendar argument as a message names it.
fn expr_name(e: &Expr) -> String {
    read_parts(e).map_or_else(|| "the calendar".to_owned(), |(name, _)| name.to_owned())
}

/// The keys a read of an input has when none are written.
const NO_KEYS: &[Key] = &[];

/// A read of an item as written, `x` or `x[k, …]`: its id and its keys.
fn read_parts(e: &Expr) -> Option<(&str, &[Key])> {
    if let Expr::Name(name) = e {
        return Some((name.as_str(), NO_KEYS));
    }
    if let Expr::Post(base, posts) = e
        && let (Expr::Name(name), [Postfix::Keys(keys)]) = (base.as_ref(), posts.as_slice())
    {
        return Some((name.as_str(), keys.as_slice()));
    }
    None
}

/// A value `pos` reads as a key of its index (refcore's `c_pos`). L §4.2.
fn pos_key(m: &Reality, v: &Value) -> Result<KeyVal, Stop> {
    if let Value::Number(_) = v {
        return Ok(KeyVal::Int(int_of(m, v)?));
    }
    if let Value::Day(d) = v {
        return Ok(KeyVal::Day(*d));
    }
    if let Value::Period(p) = v {
        return Ok(KeyVal::Period(*p));
    }
    if let Value::Key(k) = v {
        return Ok(k.clone());
    }
    Err(wrong_type(m, "pos of", v))
}

/// A period argument: a period, or a day as its day period (refcore's `period_arg`). L §4.2.
fn period_arg(m: &Reality, v: &Value, what: &str) -> Result<Period, Stop> {
    if let Value::Period(p) = v {
        return Ok(*p);
    }
    if let Value::Day(d) = v {
        return Period::day(*d).ok_or_else(|| invariant("a day period"));
    }
    if *v == Value::None {
        return Err(refuse(Run::None, format!("a value is none here: {what}")));
    }
    Err(wrong_type(m, &format!("{what}: not a period:"), v))
}

/// A date argument: a day, or a day period's day (refcore's `as_date_or_iv`). L §4.2.
fn date_arg(m: &Reality, v: &Value, what: &str) -> Result<Day, Stop> {
    if let Value::Day(d) = v {
        return Ok(*d);
    }
    if let Value::Period(p) = v
        && p.kind() == PeriodKind::Day
    {
        return Ok(p.start());
    }
    if *v == Value::None {
        return Err(refuse(Run::None, format!("a value is none here: {what}")));
    }
    if *v == Value::Never {
        return Err(refuse(Run::None, format!("never in {what}")));
    }
    Err(wrong_type(m, &format!("{what} of"), v))
}

/// The months of `add_months`'s count: an int, or a duration in months (refcore's
/// `months_arg`). L §4.2.
fn months_arg(m: &Reality, v: &Value) -> Result<i64, Stop> {
    if let Value::Number(Number {
        q,
        class: Class::Num(unit, _),
    }) = v
        && !unit.is_one()
    {
        if unit.terms() != [("month".to_owned(), 1)] {
            return Err(refuse_load(
                Load::Unit,
                format!("a duration of {} for add_months", unit.text()),
            ));
        }
        return int_of(m, &Value::Number(Number::plain(q.clone(), Unit::one())));
    }
    int_of(m, v)
}

/// `minor(CUR)`: the currency's minor unit as money (refcore's `c_minor`). L §7.2.
pub(super) fn minor_of(m: &Reality, call: &Call) -> Result<Value, Stop> {
    let Some(
        Expr::Name(cur) | Expr::Lit(crate::syntax::Lit::Cur(cur) | crate::syntax::Lit::Name(cur)),
    ) = call.args.first()
    else {
        return Err(arity());
    };
    let id = super::arith::currency_of(m, cur)?;
    Ok(Value::Money(crate::num::Money::from_minor(1, id)))
}

/// `round(x, u, mode)` of values that are neither holes nor `none` (refcore's `c_round`;
/// check.py's `c_round` for the result's type). L §7.2.
///
/// # Errors
///
/// `LOAD-UNIT` for a mode that isn't a rounding code or a step of another unit; `RUN-DOMAIN`
/// for a step that isn't positive.
pub(super) fn round_values(m: &Reality, x: &Value, u: &Value, mode: &Value) -> Result<Value, Stop> {
    let mode = rounding_mode(m, mode)?;
    let xs = Scalar::of(m, x)?;
    let us = Scalar::of(m, u)?;
    if us.q.compare(&Num::zero()).map_err(num_stop)? != core::cmp::Ordering::Greater {
        return Err(refuse(
            Run::Domain,
            format!(
                "round with a step {} that is not positive",
                u.text(m).map_err(num_stop)?
            ),
        ));
    }
    let (step_q, step_unit) = match &us.class {
        Some(Class::Flex) if xs.percent() => (
            us.q.checked_mul(&Num::ratio(1, 100).map_err(num_stop)?)
                .map_err(num_stop)?,
            xs.unit.clone(),
        ),
        Some(Class::Flex) => (us.q.clone(), xs.unit.clone()),
        Some(Class::Int | Class::Num(..) | Class::Rate(_)) | None => {
            (us.q.clone(), us.unit.clone())
        }
    };
    let factor = if step_unit == xs.unit {
        Num::int(1)
    } else if let Some(f) = step_unit.conv(&xs.unit).map_err(num_stop)? {
        f
    } else {
        return Err(refuse_load(
            Load::Unit,
            format!(
                "round: the step {} and {} have different units",
                u.text(m).map_err(num_stop)?,
                x.text(m).map_err(num_stop)?
            ),
        ));
    };
    let step = step_q.checked_mul(&factor).map_err(num_stop)?;
    let q = xs.q.round(&step, mode).map_err(num_stop)?;
    round_result(m, &xs, &us, u, q)
}

/// A rounding mode: a code word of the rounding family (L §7.2).
fn rounding_mode(m: &Reality, v: &Value) -> Result<Mode, Stop> {
    let (Value::Word(word) | Value::Code(Code::Word(word))) = v else {
        return Err(wrong_type(m, "round: the mode is", v));
    };
    Mode::from_word(word)
        .ok_or_else(|| refuse_load(Load::Unit, format!("round: the mode is {word}")))
}

/// A number or money as `round` reads it: its value, unit and class (none for money).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Scalar {
    /// The value in its unit.
    q: Num,
    /// Its unit.
    unit: Unit,
    /// Its class; none for money.
    class: Option<Class>,
}

impl Scalar {
    /// The scalar of a number or money.
    fn of(m: &Reality, v: &Value) -> Result<Scalar, Stop> {
        if let Value::Money(money) = v {
            return Ok(Scalar {
                q: money_num(m, money).map_err(num_stop)?,
                unit: currency_unit(m, money.currency()).map_err(num_stop)?,
                class: None,
            });
        }
        let Value::Number(n) = v else {
            return Err(wrong_type(m, "round of", v));
        };
        Ok(Scalar {
            q: n.q.clone(),
            unit: n.unit(),
            class: Some(n.class.clone()),
        })
    }

    /// Whether it is a `num %`.
    fn percent(&self) -> bool {
        matches!(self.class, Some(Class::Num(_, Shown::Percent)))
    }

    /// Whether it is an int or a bare figure.
    fn int_like(&self) -> bool {
        matches!(self.class, Some(Class::Int | Class::Flex))
    }
}

/// The class of `round`'s result (check.py's `c_round`). L §7.2 Its type.
fn round_result(
    m: &Reality,
    x: &Scalar,
    u: &Scalar,
    written: &Value,
    q: Num,
) -> Result<Value, Stop> {
    let currency = if let [(cur, 1)] = x.unit.terms()
        && m.currencies.contains_key(cur)
    {
        Some(cur.clone())
    } else {
        None
    };
    let step_flex = u.class == Some(Class::Flex);
    if let Some(cur) = &currency
        && (u.class.is_none() || (step_flex && whole_minor(m, &u.q, cur)?))
    {
        let id = super::arith::currency_of(m, cur)?;
        return match num_money(m, &q, id) {
            Ok(money) => Ok(Value::Money(money)),
            Err(_) => Err(refuse_load(
                Load::Unit,
                format!(
                    "round to money: the step {} is not a whole number of minor units",
                    written.text(m).map_err(num_stop)?
                ),
            )),
        };
    }
    if step_flex
        && u.q == Num::int(1)
        && x.unit.is_one()
        && !x.percent()
        && matches!(x.class, Some(Class::Num(..) | Class::Int | Class::Flex))
    {
        return Ok(Value::Number(Number {
            q,
            class: Class::Int,
        }));
    }
    let unit = if x.int_like() {
        Unit::one()
    } else {
        x.unit.clone()
    };
    Ok(Value::Number(Number::plain(q, unit)))
}

/// Whether step `step` is a whole number of currency `cur`'s minor units (L §7.2 Its type).
fn whole_minor(m: &Reality, step: &Num, cur: &str) -> Result<bool, Stop> {
    let minor = m
        .currencies
        .get(cur)
        .ok_or_else(|| invariant("a currency load didn't declare"))?;
    Ok(step.checked_div(minor).map_err(num_stop)?.is_integer())
}

/// `min`'s and `max`'s bare figures in the class of an argument with a unit or money, else
/// of any other number (refcore's `coerce_flex`, check.py's `flex_to`). L §3.2.
fn coerce_flex(m: &Reality, vals: Vec<Value>) -> Result<Vec<Value>, Stop> {
    let model = vals
        .iter()
        .find_map(unit_model)
        .or_else(|| vals.iter().find_map(number_model));
    let Some(model) = model else {
        return Ok(vals);
    };
    let mut out = Vec::with_capacity(vals.len());
    for v in vals {
        let Value::Number(Number {
            q,
            class: Class::Flex,
        }) = v
        else {
            out.push(v);
            continue;
        };
        out.push(match &model {
            Model::Money(cur) => Value::Money(num_money(m, &q, *cur).map_err(num_stop)?),
            Model::Class(class) => Value::Number(Number {
                q,
                class: class.clone(),
            }),
        });
    }
    Ok(out)
}

/// The class of an argument with money or a unit, which a bare figure beside it takes.
fn unit_model(v: &Value) -> Option<Model> {
    if let Value::Money(money) = v {
        return Some(Model::Money(money.currency()));
    }
    if let Value::Number(n) = v
        && (matches!(n.class, Class::Rate(_))
            || (!matches!(n.class, Class::Flex | Class::Int) && !n.unit().is_one()))
    {
        return Some(Model::Class(n.class.clone()));
    }
    None
}

/// The class of a number that isn't a bare figure.
fn number_model(v: &Value) -> Option<Model> {
    if let Value::Number(n) = v
        && n.class != Class::Flex
    {
        return Some(Model::Class(n.class.clone()));
    }
    None
}

/// The class a bare figure takes beside other arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Model {
    /// Money of a currency.
    Money(crate::num::Currency),
    /// A number's class.
    Class(Class),
}

/// The rate per period as a linear value or a power `base^e`, minus 1 (refcore's
/// `rate_parts`). L §3.3.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Parts {
    /// The rate per period itself.
    Linear(Num),
    /// `base^e - 1`.
    Power(Num, Num),
}

/// The months of a period word of the one dimension (L §3.2).
fn period_months(word: &str) -> Option<i64> {
    match word {
        "year" => Some(12),
        "quarter" => Some(3),
        "month" => Some(1),
        _ => None,
    }
}

/// `periodic`'s parts of rate `r` per period `p` under compounding `c` (refcore's
/// `rate_parts`). L §3.3.
fn rate_parts(
    m: &Reality,
    rate: &Value,
    period: &Value,
    code: &Value,
    what: &str,
) -> Result<Parts, Stop> {
    let Value::Number(Number {
        q: rq,
        class: Class::Rate(unit),
    }) = rate
    else {
        return Err(refuse_load(
            Load::Unit,
            format!("{what}: {} is not a rate", rate.text(m).map_err(num_stop)?),
        ));
    };
    let per = unit
        .period_of()
        .ok_or_else(|| refuse_load(Load::Unit, format!("{what}: the rate states no period")))?;
    let Value::Word(target) = period else {
        return Err(refuse_load(
            Load::Unit,
            format!("{what}: the period is no period word"),
        ));
    };
    let share = if target == "day" || per == "day" {
        if target != per {
            return Err(refuse(
                Run::Domain,
                format!(
                    "{what}: a {per} rate over a {target}: a day's share of a longer period is a day count (dcf), never a default (L §3.3)"
                ),
            ));
        }
        Num::int(1)
    } else {
        let months = period_months(target)
            .ok_or_else(|| refuse_load(Load::Unit, format!("{what}: {target} is no period")))?;
        let per_months = period_months(per).ok_or_else(|| invariant("a rate's period"))?;
        Num::ratio(months, per_months).map_err(num_stop)?
    };
    let one = Num::int(1);
    let plus = |x: &Num| one.checked_add(x).map_err(num_stop);
    if let Value::Code(Code::Nominal(times)) = code {
        let times = Num::int(*times);
        let base = plus(&rq.checked_div(&times).map_err(num_stop)?)?;
        return Ok(Parts::Power(
            base,
            times.checked_mul(&share).map_err(num_stop)?,
        ));
    }
    let (Value::Word(word) | Value::Code(Code::Word(word))) = code else {
        return Err(wrong_type(
            m,
            &format!("{what}: not a compounding code:"),
            code,
        ));
    };
    match word.as_str() {
        "simple" => Ok(Parts::Linear(rq.checked_mul(&share).map_err(num_stop)?)),
        "effective" => Ok(Parts::Power(plus(rq)?, share)),
        "discount" => {
            if *rq == one {
                return Err(refuse(
                    Run::Div,
                    "division by zero: a rate of discount of 1".to_owned(),
                ));
            }
            let interest = rq
                .checked_div(&one.checked_sub(rq).map_err(num_stop)?)
                .map_err(num_stop)?;
            Ok(Parts::Power(plus(&interest)?, share))
        }
        _ => Err(wrong_type(
            m,
            &format!("{what}: not a compounding code:"),
            code,
        )),
    }
}

/// `factor`'s n: a number with no unit (L §3.3).
fn plain_count(m: &Reality, n: &Value) -> Result<Num, Stop> {
    if let Value::Number(x) = n
        && x.unit().is_one()
        && !matches!(x.class, Class::Rate(_))
    {
        return Ok(x.q.clone());
    }
    Err(wrong_type(m, "factor: n is", n))
}

/// `base^e` exactly, for a whole e; a power that isn't whole needs the transcendental kernel,
/// which S doesn't build (L §3.3, §7.3).
fn power(base: &Num, e: &Num) -> Result<Num, Stop> {
    if !e.is_integer() {
        return Err(Stop::NotBuilt("a power that isn't an integer (`to u`)"));
    }
    let k = e.to_i64().ok_or_else(|| num_stop(NumError::Digits))?;
    if base.is_zero() && k < 0 {
        return Err(refuse(Run::Div, "division by zero".to_owned()));
    }
    let mut out = Num::int(1);
    let mut square = base.clone();
    let mut left = k.unsigned_abs();
    let halving = || invariant("a power's exponent");
    while left > 0 {
        if left.checked_rem_euclid(2).ok_or_else(halving)? == 1 {
            out = out.checked_mul(&square).map_err(num_stop)?;
        }
        left = left.checked_div_euclid(2).ok_or_else(halving)?;
        if left > 0 {
            square = square.checked_mul(&square).map_err(num_stop)?;
        }
    }
    if k < 0 {
        return Num::int(1).checked_div(&out).map_err(num_stop);
    }
    Ok(out)
}
