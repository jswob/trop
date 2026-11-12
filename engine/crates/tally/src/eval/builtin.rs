//! The built-ins' dispatch and the number built-ins: `round`, `min` and `max`, `abs`, `pow`, `pos`, `missing`, `minor`, and a rate's `periodic` and `factor`.
//! Spec: L §4.2 (the dates' and calendars' built-ins, in `dates.rs` and `business.rs`; `pos`),
//! §7.2 (`round` and the type of its result; a bare step takes the operand's unit), §5.11
//! (`min`, `max`, `missing`), §3.3 (`periodic`, `factor`: exact when the power is an integer),
//! §7.1, §7.3 (`pow` by an integer, exact; a power that isn't one needs `to u`), §4.4 (`min`
//! and `max` of windows, `round` of one).
//! Twin: engine/tools/refcore/evaluate.py (`e_call`, `c_min`, `c_abs`, `c_pow`,
//! `c_minor`, `c_round`, `c_pos`, `c_missing`) and eval2.py (`m_rate_parts`,
//! `m_periodic_real`).
//! Invariants: a hole argument makes the call that hole; a power by an integer is exact and
//! its unit raised to it; a power that isn't an integer stops `NotBuilt` (the transcendental
//! kernel is M6's).

use super::arith::int_of;
use super::expr::hole_of;
use super::extremes::min_max;
use super::keys::Given;
use super::state::{
    Ctx, Engine, Env, invariant, not_typed, num_stop, refuse, refuse_load, wrong_type,
};
use super::windows::{End, as_window, imprecise};
use crate::base::{Load, Run};
use crate::model::{
    Class, Code, Domain, KeyVal, Number, Reality, Shown, Unit, Value, currency_unit, money_num,
    num_money,
};
use crate::num::{Mode, Num, NumError};
use crate::read::{Asked, Found, read_input};
use crate::syntax::{Call, Expr, Key, Lit, Postfix, Stop, Value as Written};

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
            "days_between" | "add_days" | "add_months" | "add_years" | "age" | "age_months" => {
                self.day_count(m, call, env, ctx)
            }
            "overlap_days" => self.overlap(m, call, env, ctx),
            "weekday" => self.weekday(m, call, env, ctx),
            "dcf" => self.dcf(m, call, env, ctx),
            "window" => self.window_call(m, call, env, ctx),
            "one_of" => self.one_of_call(m, call, env, ctx),
            "is_business" => self.is_business(m, call, env, ctx),
            "add_business" => self.add_business(m, call, env, ctx),
            "last_business" => self.last_business(m, call, env, ctx),
            "schedule" => self.schedule(m, call, env, ctx),
            "round" => self.round(m, call, env, ctx),
            "min" | "max" => self.extreme(m, call, env, ctx),
            "abs" => self.abs(m, call, env, ctx),
            "pow" => self.pow(m, call, env, ctx),
            "minor" => minor_of(m, call),
            "marginal" => self.marginal(m, call, env, ctx),
            "pos" => self.pos(m, call, env, ctx),
            "missing" => self.missing(m, call, env, ctx),
            "since" => self.since_read(m, call, env, ctx),
            "gap" => self.gap_call(m, call, env, ctx),
            "periodic" | "factor" => self.rate_call(m, call, env, ctx),
            "next" => self.next_movement(m, call, env, ctx),
            "lifespan" => self.lifespan_of(m, call, env, ctx),
            _ => Err(Stop::NotBuilt("a built-in M1's realities don't call")),
        }
    }

    /// `gap(x, d)`: its two arguments, read by the walk (refcore's `m_c_gap`). L §5.19.
    fn gap_call(&mut self, m: &Reality, call: &Call, env: &Env, ctx: Ctx) -> Result<Value, Stop> {
        let [place, day] = call.args.as_slice() else {
            return Err(arity());
        };
        self.gap(m, (place, day), env, ctx)
    }

    /// The arguments of a call, evaluated in order.
    pub(super) fn args(
        &mut self,
        m: &Reality,
        call: &Call,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Vec<Value>, Stop> {
        let mut out = Vec::with_capacity(call.args.len());
        for arg in &call.args {
            out.push(self.ev(m, arg, env, ctx)?);
        }
        Ok(out)
    }

    /// `round(x, u, mode)`: the multiple of u the mode chooses, money when u is a whole
    /// number of minor units of x's currency, an int for `round(x, 1, m)` of a plain number
    /// (refcore's `c_round` and check.py's `c_round` for the result's type); a step written as
    /// a bare figure takes the operand's unit (check.py's `_stepflex`). L §7.1, §7.2, §4.4.
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
        if let Value::Window(w) = &x {
            return Ok(Value::Hole(imprecise(m, w, "round")?));
        }
        round_values(m, &x, &bare_step(call.args.get(1), u), &mode)
    }

    /// `min(a, b, …)`, `max(a, b, …)`: a bare figure takes the unit of an argument with one;
    /// of windows, the window of the extremes of their ends (refcore's `c_min`, `coerce_flex`,
    /// `agg_minmax`, `extreme`). L §5.11, §4.4.
    fn extreme(&mut self, m: &Reality, call: &Call, env: &Env, ctx: Ctx) -> Result<Value, Stop> {
        let snap = self.trace_snap();
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
        if vals.is_empty() {
            return Err(arity());
        }
        let pick = if call.name == "min" {
            End::Low
        } else {
            End::High
        };
        let vals = coerce_flex(m, vals)?;
        let Some(snap) = snap else {
            return min_max(m, vals, pick);
        };
        let extreme = min_max(m, vals.clone(), pick)?;
        // explained by its first winning term, the arguments in the order written (L §5.18)
        let terms: Vec<(&Expr, &Env, Value)> = call
            .args
            .iter()
            .zip(vals)
            .map(|(a, v)| (a, env, v))
            .collect();
        self.trace_winner(m, (snap, &extreme), &terms, ctx)?;
        Ok(extreme)
    }

    /// `pow(x, n)`: x to the integer power n, exact, its unit to the n-th power, a `num`; a
    /// power that isn't an integer needs the transcendental kernel (refcore's `c_pow`).
    /// L §3.3, §7.1, §7.3.
    fn pow(&mut self, m: &Reality, call: &Call, env: &Env, ctx: Ctx) -> Result<Value, Stop> {
        let [x, k] = self
            .args(m, call, env, ctx)?
            .try_into()
            .map_err(|_| arity())?;
        let some_hole = x.as_hole().is_some() || k.as_hole().is_some();
        for v in [&x, &k] {
            if *v == Value::None {
                return Err(refuse(Run::None, "a value is none here: pow".to_owned()));
            }
        }
        if some_hole {
            return hole_of(&[x.as_hole(), k.as_hole()]);
        }
        if let Some(w) = as_window(&x).or_else(|| as_window(&k)) {
            // an interval where one value is needed (L §4.4), as `round` of one is
            return Ok(Value::Hole(imprecise(m, w, "pow")?));
        }
        let not_number = || Stop::NotBuilt("pow of a value that is no number, met at run");
        let (base, unit) = if let Value::Money(money) = &x {
            (
                money_num(m, money).map_err(num_stop)?,
                currency_unit(m, money.currency()).map_err(num_stop)?,
            )
        } else if let Value::Number(n) = &x {
            (n.q.clone(), n.unit())
        } else {
            return Err(not_number());
        };
        let Value::Number(power) = &k else {
            return Err(not_number());
        };
        if !power.q.is_integer() {
            return Err(Stop::NotBuilt(
                "a power that isn't an integer (the transcendental kernel)",
            ));
        }
        let q = int_power(m, (&x, &base), &power.q)?;
        let unit = if unit.is_one() {
            unit
        } else {
            unit.power(&power.q)
                .map_err(|e| e.refusal(num_stop, None))?
        };
        Ok(Value::Number(Number::plain(q, unit)))
    }

    /// `lifespan(k)`: thing k's lifespan as the cut sees it, a record of its first day `from`
    /// and its last `to`, `never` for an open end and the stated hole for an end not yet known
    /// (`HOLE-STATED`); a thing added after the cut is `RUN-KEY` (refcore's `c_lifespan`).
    /// L §5.7.
    fn lifespan_of(
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
        if v == Value::None {
            return Err(refuse(
                Run::None,
                "a value is none here: lifespan".to_owned(),
            ));
        }
        let Value::Key(KeyVal::Thing(name)) = v else {
            // a movement bound over an event kind has no lifespan (L §5.7): load lets it
            // through, and neither the spec nor refcore gives its run's text
            return Err(Stop::NotBuilt(
                "lifespan of a value that is no thing, met at run",
            ));
        };
        let thing = m
            .things
            .get(&name)
            .ok_or_else(|| invariant("a thing load didn't build"))?;
        // a lifespan is a fact (L §5.7): its read is recorded, so a scenario that changes it
        // lists the movements before its `from` that read it (E §9 item 3)
        self.note_thing(m, thing, ctx.world);
        let Some(span) = m.lifespan(thing, (ctx.cut, ctx.world)) else {
            return Err(refuse(Run::Key, format!("{name} was added after the cut")));
        };
        let from = span
            .start()
            .ok_or_else(|| invariant("a lifespan with no start (LOAD-LIFESPAN)"))?;
        let to = match (span.end_question(), span.end()) {
            (Some(question), _) => Value::Hole(crate::hole::Hole::asked(question, &name)),
            (None, None) => Value::Never,
            (None, Some(end)) => Value::Day(
                end.previous()
                    .ok_or_else(|| invariant("a lifespan ending before the first day"))?,
            ),
        };
        Ok(Value::Record(vec![
            ("from".to_owned(), Value::Day(from)),
            ("to".to_owned(), to),
        ]))
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
        if let Some(w) = as_window(&v) {
            // an interval where one value is needed (L §4.4), as `round` of one is
            return Ok(Value::Hole(imprecise(m, w, "abs")?));
        }
        if let Value::Money(money) = &v {
            if money.is_negative() {
                return Ok(Value::Money(money.checked_neg().map_err(num_stop)?));
            }
            return Ok(v);
        }
        let Value::Number(n) = v else {
            return Err(Stop::NotBuilt(
                "abs of a value that is no number, met at run",
            ));
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
        let index = if let Some(ix) = written_index(arg, env) {
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
        let Ok(keys) = self.make_keys(m, name, &input.binds, given, ctx)? else {
            return Ok(Value::Bool(false));
        };
        self.movement_keys(m, &input.binds, &keys, ctx)?;
        let value = self.read_input_at(m, input, &keys, ctx, Asked::Plain)?;
        if value.as_hole().is_none() {
            return Ok(Value::Bool(false));
        }
        let held = match read_input(m, input, &keys, self.clocks(ctx, &input.id)?, Asked::Plain)? {
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
    pub(super) fn given_keys(
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

/// `round`'s step as its operand meets it: a figure written bare, with no unit and not as a
/// fraction, takes the operand's unit (check.py's `_stepflex`). L §7.1, §7.2.
fn bare_step(written: Option<&Expr>, u: Value) -> Value {
    if let (Some(Expr::Lit(Lit::Num(n))), Value::Number(step)) = (written, &u)
        && n.unit.is_none()
        && !n.fraction
    {
        return Value::Number(Number {
            q: step.q.clone(),
            class: Class::Flex,
        });
    }
    u
}

/// `base^k` for a whole `k`, as refcore's `c_pow` screens and computes it: past the digits its
/// parts' bits promise, `RUN-DIGITS` before the power; 0 to a power below 0 `RUN-DIV`; 1, 0
/// and -1 to a power past 64 bits by their closed forms. L §3.3, §7.1.
fn int_power(m: &Reality, (x, base): (&Value, &Num), k: &Num) -> Result<Num, Stop> {
    let k_text = k.integer_text().ok_or_else(|| invariant("a whole power"))?;
    let bits = base.bits();
    if bits > 1 && passes_cap(bits, k)? {
        let x_text = x.text(m).map_err(num_stop)?;
        return Err(refuse(
            Run::Digits,
            format!("pow({x_text}, {k_text}) passes 4 096 digits; round where the law rounds"),
        ));
    }
    if base.is_zero() && k.is_negative() {
        return Err(refuse(
            Run::Div,
            format!("division by zero: pow(0, {k_text})"),
        ));
    }
    match k.to_i64() {
        Some(whole) => base.checked_pow(whole).map_err(num_stop),
        None => power_past_64_bits(base, k),
    }
}

/// Whether `(bits - 1) × |k| × log₁₀ 2` passes 4 096 + 10 digits, log₁₀ 2 as refcore's
/// 0.30102999 (refcore's `c_pow`, its screen); a screen past the cap itself passes. L §7.1.
fn passes_cap(bits: u64, k: &Num) -> Result<bool, Stop> {
    let below = bits
        .checked_sub(1)
        .and_then(|b| i64::try_from(b).ok())
        .ok_or_else(|| invariant("a part's bits"))?;
    let size = if k.is_negative() {
        k.checked_neg().map_err(num_stop)?
    } else {
        k.clone()
    };
    let log2 = Num::ratio(30_102_999, 100_000_000).map_err(num_stop)?;
    let digits = Num::int(below)
        .checked_mul(&size)
        .and_then(|d| d.checked_mul(&log2));
    match digits {
        Ok(d) => Ok(d.compare(&Num::int(4_106)).map_err(num_stop)? == core::cmp::Ordering::Greater),
        Err(NumError::Digits) => Ok(true),
        Err(other) => Err(num_stop(other)),
    }
}

/// The keys a read of an input has when none are written.
const NO_KEYS: &[Key] = &[];

/// A read of an item as written, `x` or `x[k, …]`: its id and its keys.
pub(super) fn read_parts(e: &Expr) -> Option<(&str, &[Key])> {
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

/// The index of `pos`'s argument as written: a variable bound over an index, or a key moved
/// by steps from one, `k ± n`, which keeps its index's type (refcore's `n.args[0]._t.ix`;
/// check.py types `k + n` as k's key). L §4.2, §5.11 Recurrences.
fn written_index(arg: &Expr, env: &Env) -> Option<String> {
    let mut at = arg;
    while let Expr::Paren(x) | Expr::Sum(x, _) = at {
        at = x;
    }
    let Expr::Name(name) = at else {
        return None;
    };
    let Some(Domain::Index(ix)) = &env.get(name)?.domain else {
        return None;
    };
    Some(ix.clone())
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
    Err(not_typed())
}

/// `minor(CUR)`: the currency's minor unit as money (refcore's `c_minor`). L §7.2.
pub(super) fn minor_of(m: &Reality, call: &Call) -> Result<Value, Stop> {
    let Some(Expr::Name(cur) | Expr::Lit(Lit::Cur(cur) | Lit::Name(cur))) = call.args.first()
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
    let mode = rounding_mode(m, "round", mode)?;
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

/// A rounding mode of function `what` (`round`, `prorata`): a code word of the rounding
/// family, or an axis at its option where every option is one (L §7.2, §5.13, §5.5).
pub(super) fn rounding_mode(m: &Reality, what: &str, v: &Value) -> Result<Mode, Stop> {
    let word = if let Value::Word(word) | Value::Code(Code::Word(word)) = v {
        word
    } else if let Value::Key(KeyVal::Key { index, name }) = v
        && m.axes.contains(index)
    {
        name
    } else {
        return Err(wrong_type(m, &format!("{what}: the mode is"), v));
    };
    Mode::from_word(word)
        .ok_or_else(|| refuse_load(Load::Unit, format!("{what}: the mode is {word}")))
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
            return Err(not_typed());
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
    if let Value::Code(Code::Nominal(times)) = code {
        let times = Num::int(*times);
        let base = plus(&rq.checked_div(&times).map_err(num_stop)?)?;
        return Ok(Parts::Power(
            base,
            times.checked_mul(&share).map_err(num_stop)?,
        ));
    }
    let (Value::Word(word) | Value::Code(Code::Word(word))) = code else {
        return Err(no_compounding(m, what, code)?);
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
        _ => Err(no_compounding(m, what, code)?),
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
    let Some(k) = e.to_i64() else {
        return power_past_64_bits(base, e);
    };
    base.checked_pow(k).map_err(num_stop)
}

/// `base^e` for a whole `e` past 64 bits, as refcore's Python integers: 1, 0 and -1 keep their
/// size; any other base has a part of 2 or more, which such a power takes past the 4 096-digit
/// cap, `RUN-DIGITS`. L §3.3, §7.1.
fn power_past_64_bits(base: &Num, e: &Num) -> Result<Num, Stop> {
    let one = Num::int(1);
    if *base == one {
        return Ok(one);
    }
    if base.is_zero() {
        if e.is_negative() {
            return Err(refuse(Run::Div, "division by zero".to_owned()));
        }
        return Ok(Num::zero());
    }
    if *base == Num::int(-1) {
        let even = e.checked_div(&Num::int(2)).map_err(num_stop)?.is_integer();
        return Ok(if even { one } else { Num::int(-1) });
    }
    Err(num_stop(NumError::Digits))
}
