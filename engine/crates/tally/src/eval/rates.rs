//! The rate built-ins: `periodic`, `factor` and `accrue`, a rate's arithmetic through a period and a compounding code.
//! Spec: L §3.3 (`periodic`, `factor`, `accrue`: exact where the power is an integer, else
//! `to u`; a day's share of a longer period is a day count; `accrue` takes a yearly rate,
//! states `simple`, `effective` and `nominal m`, and rounds nothing), §7.3 (a power that
//! isn't an integer needs `to u`), §8.4 (`RUN-DOMAIN` for an argument outside a row's
//! domain that load couldn't see), §4.4 (`accrue` off the list: a ranged date is the hole).
//! Twin: engine/tools/refcore/eval2.py (`m_trans_call`, `m_rate_parts`, `m_periodic_real`,
//! `m_accrue_real`, `m_dcf_q`).
//! Invariants: a refusal outranks a hole: `none`, then each known argument's own refusal,
//! then a hole argument makes the call that hole (L §8); a power by an integer is exact; a
//! power that isn't an integer stops `NotBuilt` (`to u`, the transcendental kernel, is M6's);
//! nothing here rounds.

use super::builtin::power_past_64_bits;
use super::dates::{daycount_of, year_fraction_of};
use super::expr::hole_of;
use super::state::{
    Ctx, Engine, Env, day_given, given, invariant, num_stop, refuse, refuse_load, wrong_type,
};
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
        let (rate, count, period, code) = match vals.as_slice() {
            [rate, period, code] if call.name == "periodic" => (rate, None, period, code),
            [rate, count, period, code] => (rate, Some(count), period, code),
            _ => return Err(arity()),
        };
        let what = call.name.as_str();
        // refusals outrank holes (L §8; pick P12): each argument in written order, `none`
        // refused, a known one's own check run, a hole's waiting; the rate's check reads the
        // period word, which load fixes
        let share = given(rate, what)?
            .then(|| rate_share(m, rate, period, what))
            .transpose()?;
        let times = match count {
            Some(n) => given(n, what)?.then(|| plain_count(m, n)).transpose()?,
            None => None,
        };
        let how = given(code, what)?
            .then(|| compounding(m, what, code).and_then(nominal_positive))
            .transpose()?;
        let parts = match (share, how) {
            (Some((rq, share)), Some(how)) => Some(parts_under(&rq, share, how)?),
            _ => None,
        };
        let holes: Vec<Option<&crate::hole::Hole>> = vals.iter().map(Value::as_hole).collect();
        if holes.iter().any(Option::is_some) {
            return hole_of(&holes);
        }
        let parts = parts.ok_or_else(|| invariant("a known rate's parts"))?;
        let q = match (parts, times) {
            (Parts::Linear(per), None) => per,
            (Parts::Power(base, exponent), None) => power(&base, &exponent)?
                .checked_sub(&Num::int(1))
                .map_err(num_stop)?,
            (Parts::Linear(per), Some(times)) => {
                let base = Num::int(1).checked_add(&per).map_err(num_stop)?;
                power(&base, &times)?
            }
            (Parts::Power(base, exponent), Some(times)) => {
                let exponent = exponent.checked_mul(&times).map_err(num_stop)?;
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
        let [principal, rate, from, to, daycount, code] = vals.as_slice() else {
            return Err(arity());
        };
        let what = "accrue";
        // refusals outrank holes (L §8; pick P12): each argument in written order, `none`
        // refused, a known one's own check run, a hole's waiting
        let principal = given(principal, what)?
            .then(|| principal_of(m, principal))
            .transpose()?;
        let r = given(rate, what)?.then(|| yearly(m, rate)).transpose()?;
        day_given(from, what)?;
        day_given(to, what)?;
        if given(daycount, what)? {
            daycount_of(m, daycount)?;
        }
        let how = given(code, what)?
            .then(|| compounding(m, what, code).and_then(accrual))
            .transpose()?;
        let holes: Vec<Option<&crate::hole::Hole>> = vals.iter().map(Value::as_hole).collect();
        if holes.iter().any(Option::is_some) {
            return hole_of(&holes);
        }
        if let Some(hole) = unlisted(&[from, to], "accrue") {
            return Ok(Value::Hole(hole));
        }
        let (Some((p, unit)), Some(r), Some(how)) = (principal, r, how) else {
            return Err(invariant("accrue's known arguments"));
        };
        let years = year_fraction_of(m, (&single(from), &single(to)), daycount)?;
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
pub(super) fn principal_of(m: &Reality, p: &Value) -> Result<(Num, Unit), Stop> {
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

/// The interest on `p` at yearly rate `r` over `years` under `how` (refcore's
/// `m_accrue_real`). L §3.3.
fn accrued((p, r): (&Num, &Num), years: &Num, how: Accrual) -> Result<Num, Stop> {
    let one = Num::int(1);
    let (base, exponent) = match how {
        Accrual::Simple => {
            let per = r.checked_mul(years).map_err(num_stop)?;
            return p.checked_mul(&per).map_err(num_stop);
        }
        Accrual::Effective => (one.checked_add(r).map_err(num_stop)?, years.clone()),
        Accrual::Nominal(times) => {
            let times = Num::int(times);
            let base = one
                .checked_add(&r.checked_div(&times).map_err(num_stop)?)
                .map_err(num_stop)?;
            (base, times.checked_mul(years).map_err(num_stop)?)
        }
    };
    let grown = power(&base, &exponent)?
        .checked_sub(&one)
        .map_err(num_stop)?;
    p.checked_mul(&grown).map_err(num_stop)
}

/// The compounding codes `accrue`'s row states (L §3.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Accrual {
    /// `simple`.
    Simple,
    /// `effective`.
    Effective,
    /// `nominal m`, m a positive int.
    Nominal(i64),
}

/// The accrual compounding `how` gives: `discount` is outside `accrue`'s row, `RUN-DOMAIN`
/// (L §3.3, §8.4; refcore's `m_accrue_real`), and so is a `nominal m` with m below 1.
fn accrual(how: Compounding) -> Result<Accrual, Stop> {
    match nominal_positive(how)? {
        Compounding::Simple => Ok(Accrual::Simple),
        Compounding::Effective => Ok(Accrual::Effective),
        Compounding::Nominal(times) => Ok(Accrual::Nominal(times)),
        // accrue's row states simple, effective and nominal m (L §3.3); discount is
        // outside it (L §8.4)
        Compounding::Discount => Err(refuse(
            Run::Domain,
            "accrue under discount: its row states simple, effective and nominal m".to_owned(),
        )),
    }
}

/// Compounding `how`, unless it is a `nominal m` whose m is below 1: `nominal m` takes a
/// positive int, so the code is refused where it is read, `RUN-DOMAIN`, whatever the other
/// arguments are, a hole too (L §3.3, §8.4; L §8, a check on a code is made once the code is
/// known; tally 2.3.1 S16, N255.1).
fn nominal_positive(how: Compounding) -> Result<Compounding, Stop> {
    if let Compounding::Nominal(times) = how
        && times < 1
    {
        return Err(refuse(
            Run::Domain,
            format!("nominal {times}: m is a positive int"),
        ));
    }
    Ok(how)
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

/// Rate `r`'s figure and the share of its period that period `p` is, a day only over a day
/// (refcore's `rate_parts`, its rate and period checks). L §3.3.
fn rate_share(m: &Reality, rate: &Value, period: &Value, what: &str) -> Result<(Num, Num), Stop> {
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
    Ok((rq.clone(), share))
}

/// `periodic`'s parts of rate figure `rq` over `share` of its period under compounding `how`
/// (refcore's `rate_parts`, its compounding): a discount rate of 1 divides by zero. L §3.3.
fn parts_under(rq: &Num, share: Num, how: Compounding) -> Result<Parts, Stop> {
    let one = Num::int(1);
    let plus = |x: &Num| one.checked_add(x).map_err(num_stop);
    match how {
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

#[cfg(test)]
mod tests {
    use crate::api::{Asks, Files, Given, Request, run};

    /// `accrue` and `periodic` under `nominal 0`, its principal a hole too, and under
    /// `nominal 4` (`fixes/g1/a9_nominal`'s twin, N255.1).
    const NOMINAL: &str = r#"model nominal
  tally 2
  on 2026-10-25
  start 2026-01-01
  end 2027-12-31

source s "invented values"

input r: rate

input d0: date

input d1: date

input p: num

input ph: num

input c0: compounding

input cn: compounding

ask q_0: accrue(p, r, d0, d1, act365, c0)

ask q_h: accrue(ph, r, d0, d1, act365, c0)

ask q_p: periodic(r, month, c0)

ask q_n: accrue(p, r, d0, d1, act365, cn)

version v
  cite s
  governs always
  known 2026-01-01
  r = 10 %/year "s"
  p = 1000 "s"
  ph = ?"which principal?" "s"
  d0 = 2026-01-01 "s"
  d1 = 2027-01-01 "s"
  c0 = nominal 0 "s"
  cn = nominal 4 "s"
"#;

    #[test]
    fn a_nominal_code_below_one_is_refused_where_it_is_read() {
        // L §3.3 "m a positive int", §8.4 (tally 2.3.1 S16, N255.1): RUN-DOMAIN, whatever
        // the other arguments are, a hole too; nominal 4 computes
        let given = vec![Given {
            path: "nominal.tally".to_owned(),
            bytes: Some(NOMINAL.as_bytes().to_vec()),
        }];
        let request = Request::new(Asks::Every);
        let answer = run(Files::new(given), &request).expect("an answer");
        let values: Vec<&str> = answer
            .lines()
            .iter()
            .filter(|line| line.starts_with("q_"))
            .map(String::as_str)
            .collect();
        let refused = "refused: RUN-DOMAIN nominal 0: m is a positive int";
        assert_eq!(
            values,
            [
                format!("q_0: {refused}"),
                format!("q_h: {refused}"),
                "q_n: 103.812890625".to_owned(),
                format!("q_p: {refused}")
            ],
            "nominal 0 refused, nominal 4 computed"
        );
    }
}
