//! The rate built-ins: `periodic`, `factor` and `accrue`, a rate's arithmetic through a period and a compounding code.
//! Spec: L §3.3 (`periodic`, `factor`, `accrue`: exact where the power is an integer, else
//! `to u`; a day's share of a longer period is a day count; `accrue` takes a yearly rate,
//! states `simple`, `effective` and `nominal m`, and rounds nothing), §7.3 (a power that
//! isn't an integer needs `to u`), §8.4 (`RUN-DOMAIN` for an argument outside a row's
//! domain that load couldn't see), §4.4 (`accrue` off the list: a ranged date is the hole).
//! Twin: engine/tools/refcore/eval2.py (`m_trans_call`, `m_rate_parts`, `m_periodic_real`,
//! `m_accrue_real`, `m_dcf_q`).
//! Invariants: a hole argument makes the call that hole; a power by an integer is exact; a
//! power that isn't an integer stops `NotBuilt` (`to u`, the transcendental kernel, is M6's);
//! nothing here rounds.

use super::builtin::power_past_64_bits;
use super::dates::year_fraction_of;
use super::expr::hole_of;
use super::state::{Ctx, Engine, Env, invariant, num_stop, refuse, refuse_load, wrong_type};
use super::windows::{single, unlisted};
use crate::base::{Load, Run};
use crate::model::{Class, Code, Number, Reality, Unit, Value, currency_unit, money_num};
use crate::num::Num;
use crate::syntax::{Call, Stop};

impl Engine {
    /// `periodic(r, P, c)`, the rate per period P under compounding c, and `factor(r, n, P,
    /// c)`, (1 + periodic)^n; exact when the power is an integer (refcore's `rate_parts`,
    /// `periodic_real`). L §3.3.
    pub(super) fn rate_call(
        &mut self,
        m: &Reality,
        call: &Call,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
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

    /// `accrue(p, r, a, b, dc, c)`: the interest on principal p at yearly rate r from day a
    /// to day b, of p's unit: p × r × dcf(a, b, dc) under `simple`, p × ((1 + r)^dcf − 1)
    /// under `effective`, p × ((1 + r/m)^(m·dcf) − 1) under `nominal m`; exact where the power
    /// is an integer, otherwise it needs `to u`; nothing rounds. `discount` is outside the
    /// row's codes, `RUN-DOMAIN` (refcore's `m_accrue_real`); a rate whose period isn't a
    /// year, met at run, is `RUN-DOMAIN` too (L §8.4: an argument outside the row's domain
    /// load couldn't see). Off the list, so a ranged date is the hole. L §3.3, §4.4, §7.3.
    pub(super) fn accrue(
        &mut self,
        m: &Reality,
        call: &Call,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let vals = self.args(m, call, env, ctx)?;
        let holes: Vec<Option<&crate::hole::Hole>> = vals.iter().map(Value::as_hole).collect();
        if holes.iter().any(Option::is_some) {
            return hole_of(&holes);
        }
        if vals.contains(&Value::None) {
            return Err(refuse(Run::None, "a value is none here: accrue".to_owned()));
        }
        let [principal, rate, from, to, daycount, code] = vals.as_slice() else {
            return Err(arity());
        };
        if let Some(hole) = unlisted(&[from, to], "accrue") {
            return Ok(Value::Hole(hole));
        }
        let (p, unit) = principal_of(m, principal)?;
        let r = yearly(m, rate)?;
        let years = year_fraction_of(m, (&single(from), &single(to)), daycount)?;
        let how = compounding(m, "accrue", code)?;
        let interest = accrued((&p, &r), &years, how)?;
        Ok(Value::Number(Number::plain(interest, unit)))
    }
}

/// A call with the wrong number of arguments: load refuses it, so here it is an invariant.
fn arity() -> Stop {
    invariant("a built-in called with the wrong number of arguments")
}

/// `accrue`'s principal as a number with its unit: money in its currency's unit, a number
/// in its own, an int in none (refcore's `args[0].u`). L §3.3.
fn principal_of(m: &Reality, p: &Value) -> Result<(Num, Unit), Stop> {
    if let Value::Money(money) = p {
        return Ok((
            money_num(m, money).map_err(num_stop)?,
            currency_unit(m, money.currency()).map_err(num_stop)?,
        ));
    }
    if let Value::Number(x) = p {
        return Ok((x.q.clone(), x.unit()));
    }
    Err(wrong_type(m, "accrue: the principal is", p))
}

/// `accrue`'s rate, a yearly rate's figure (refcore's `m_accrue_real`): another period met
/// at run is `RUN-DOMAIN`, an argument outside the row's domain load couldn't see. L §3.3,
/// §8.4.
fn yearly(m: &Reality, rate: &Value) -> Result<Num, Stop> {
    let Value::Number(Number {
        q,
        class: Class::Rate(unit),
    }) = rate
    else {
        let shown = rate.text(m).map_err(num_stop)?;
        return Err(refuse_load(
            Load::Unit,
            format!("accrue: {shown} is not a rate"),
        ));
    };
    if unit.period_of() == Some("year") {
        return Ok(q.clone());
    }
    let shown = rate.text(m).map_err(num_stop)?;
    Err(refuse(
        Run::Domain,
        format!("accrue takes a yearly rate, not {shown} (L §3.3)"),
    ))
}

/// The interest on `p` at yearly rate `r` over `years` under compounding `how` (refcore's
/// `m_accrue_real`). L §3.3.
fn accrued((p, r): (&Num, &Num), years: &Num, how: Compounding) -> Result<Num, Stop> {
    let one = Num::int(1);
    let (base, exponent) = match how {
        Compounding::Simple => {
            let per = r.checked_mul(years).map_err(num_stop)?;
            return p.checked_mul(&per).map_err(num_stop);
        }
        Compounding::Effective => (one.checked_add(r).map_err(num_stop)?, years.clone()),
        Compounding::Nominal(times) => {
            let times = Num::int(times);
            let base = one
                .checked_add(&r.checked_div(&times).map_err(num_stop)?)
                .map_err(num_stop)?;
            (base, times.checked_mul(years).map_err(num_stop)?)
        }
        Compounding::Discount => {
            // accrue's row states simple, effective and nominal m (L §3.3); discount is
            // outside it (L §8.4)
            return Err(refuse(
                Run::Domain,
                "accrue under discount: its row states simple, effective and nominal m".to_owned(),
            ));
        }
    };
    let grown = power(&base, &exponent)?
        .checked_sub(&one)
        .map_err(num_stop)?;
    p.checked_mul(&grown).map_err(num_stop)
}

/// A compounding code (L §3.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Compounding {
    /// `simple`.
    Simple,
    /// `effective`.
    Effective,
    /// `discount`.
    Discount,
    /// `nominal m`.
    Nominal(i64),
}

/// The compounding code `code` names, for built-in `what`: anything else is `LOAD-UNIT
/// <what>: <c> is not a compounding code` (refcore's `rate_parts`, `m_accrue_real`). L §3.3.
fn compounding(m: &Reality, what: &str, code: &Value) -> Result<Compounding, Stop> {
    if let Value::Code(Code::Nominal(times)) = code {
        return Ok(Compounding::Nominal(*times));
    }
    let (Value::Word(word) | Value::Code(Code::Word(word))) = code else {
        return Err(no_compounding(m, what, code)?);
    };
    match word.as_str() {
        "simple" => Ok(Compounding::Simple),
        "effective" => Ok(Compounding::Effective),
        "discount" => Ok(Compounding::Discount),
        _ => Err(no_compounding(m, what, code)?),
    }
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

/// `<what>: <c> is not a compounding code` (refcore's `rate_parts`). L §3.3.
fn no_compounding(m: &Reality, what: &str, code: &Value) -> Result<Stop, Stop> {
    Ok(refuse_load(
        Load::Unit,
        format!(
            "{what}: {} is not a compounding code",
            code.text(m).map_err(num_stop)?
        ),
    ))
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
    let Some(per) = unit.period_of() else {
        let shown = rate.text(m).map_err(num_stop)?;
        return Err(refuse_load(
            Load::Unit,
            format!("{what}: {shown} states no period"),
        ));
    };
    let Value::Word(target) = period else {
        return Err(invariant(
            "a period argument that isn't a period word: load's types refuse it first (LOAD-UNIT)",
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
        let months = period_months(target).ok_or_else(|| {
            invariant("a period word that is no period: load takes only day, month, quarter, year")
        })?;
        let per_months = period_months(per).ok_or_else(|| invariant("a rate's period"))?;
        Num::ratio(months, per_months).map_err(num_stop)?
    };
    let one = Num::int(1);
    let plus = |x: &Num| one.checked_add(x).map_err(num_stop);
    match compounding(m, what, code)? {
        Compounding::Nominal(times) => {
            let times = Num::int(times);
            let base = plus(&rq.checked_div(&times).map_err(num_stop)?)?;
            Ok(Parts::Power(
                base,
                times.checked_mul(&share).map_err(num_stop)?,
            ))
        }
        Compounding::Simple => Ok(Parts::Linear(rq.checked_mul(&share).map_err(num_stop)?)),
        Compounding::Effective => Ok(Parts::Power(plus(rq)?, share)),
        Compounding::Discount => {
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

/// `base^e` exactly, for a whole e; a power that isn't whole needs `to u`, the
/// transcendental kernel, which M6 builds (L §3.3, §7.3).
fn power(base: &Num, e: &Num) -> Result<Num, Stop> {
    if !e.is_integer() {
        return Err(Stop::NotBuilt("a power that isn't an integer (`to u`)"));
    }
    let Some(k) = e.to_i64() else {
        return power_past_64_bits(base, e);
    };
    base.checked_pow(k).map_err(num_stop)
}
