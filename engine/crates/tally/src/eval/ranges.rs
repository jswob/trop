//! A binding's stepped ranges of numbers, money or days, counted against the key limit before a value is built, and the keys read at their positions.
//! Spec: L §5.6 and §5.11 (a stepped range's values: days stepped, integers, or quantities
//! `a + i·s ≤ b` in the first end's unit), §2.3 `bind` (`a .. b [step s]`), §2.5 (at a
//! position keyed by a stepped range a key is one of its values, `LOAD-KEY` for a literal
//! where the range's ends and step are literals, `RUN-KEY` at run), §4.2 (a day moved by its
//! step), §8.1 (an end that is a hole makes the range, and a read at it, that hole), §7.6 item
//! 2 (past 1 000 000 keys a range is `RUN-LIMIT`, `LOAD-LIMIT` for a range of literals).
//! Twin: engine/tools/refcore/evaluate.py (`range_values` with its `too_many` and
//! `date_arith`, `range_member`, `key_values` at a range's position, `check_range_lits`),
//! loader.py (`range_literal`) and check3.py (`m_check_literal_ranges`).
//! Invariants: a range is counted before any value is built; a range of literals past the
//! limit is refused at load with the bytes an ask that steps through it prints; a key at a
//! range's position is one of its values, the same key whether a row writes it or a read
//! computes it.

use super::arith::{Kind, Op, Operand, arith, duration, finish, int_of, operand};
use super::bindings::Values;
use super::expr::literal;
use super::keys::Given;
use super::state::{
    Ctx, Engine, Env, Figures, invariant, num_stop, refuse, refuse_load, unit_stop,
};
use super::windows::{as_window, imprecise, one_day};
use crate::base::{Load, Run};
use crate::hole::{Hole, spread};
use crate::load::literal_end;
use crate::model::{
    BindInfo, Class, KeyVal, Number, RangeValue, Reality, Shown, Unit, Value, currency,
    currency_id, num_money,
};
use crate::num::{Num, NumError};
use crate::syntax::{Domain as Written, Expr, Lit, Stop};
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
    /// `NotBuilt` for a range of days with no step; the key's literal's refusal; an
    /// invariant.
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
        let value = written_value(m, key)?;
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

    /// The values of a range `a .. b [step s]`: days stepped, integers, or quantities `a + i·s
    /// ≤ b` in the first end's unit, counted before one is built, so past 1 000 000 keys it is
    /// `RUN-LIMIT` (refcore's `range_values`). L §5.6 (a stepped range), §7.6 item 2.
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
        Stepped::of(m, (&first, &last, by.as_ref()))?
            .values(m)
            .map(Values::Known)
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
        if let Some(h) = spread(&holes) {
            return Ok(Err(h));
        }
        for end in [&first, &last] {
            // a range end is one value: a window is the hole of its imprecision (L §4.4),
            // `never` no day (L §3.1; refcore's `date_of(z, 'a range end')`)
            if let Some(w) = as_window(end) {
                return Ok(Err(imprecise(m, w, "a range end")?));
            }
            if *end == Value::Never {
                return Err(refuse(Run::None, "never in a range end".to_owned()));
            }
        }
        Ok(Ok((first, last, by)))
    }

    /// A key given at the position of `bind`, a stepped range: the value, written or computed,
    /// checked to be one of the range's values, its ends and step read in `ctx` with the read's
    /// earlier keys bound (`earlier`); a window is `HOLE-IMPRECISE`, an end that is a hole
    /// makes the read that hole (refcore's `key_values` and `range_member`). L §2.5, §5.6,
    /// §8.1, §4.4.
    ///
    /// # Errors
    ///
    /// `RUN-NONE` for `none`; `<k> is not a value of <a> .. <b>[ step <s>]`, `LOAD-KEY` for a
    /// key literal where the range's ends and step are literals, else `RUN-KEY`; `RUN-LIMIT`
    /// for a range past 1 000 000 keys.
    pub(super) fn range_key(
        &mut self,
        m: &Reality,
        bind: &BindInfo,
        given: Given,
        (earlier, ctx): (&Env, Ctx),
    ) -> Result<Result<KeyVal, Hole>, Stop> {
        let (value, written) = match given {
            Given::Value(v) => (v, false),
            Given::Literal(lit) => (written_value(m, &lit)?, key_literal(&lit)),
        };
        if value == Value::None {
            return Err(refuse(Run::None, "a key is none here: a key".to_owned()));
        }
        if let Some(w) = as_window(&value) {
            return Ok(Err(imprecise(m, w, "a key")?));
        }
        let Written::Range(parts) = &bind.node.domain else {
            return Err(invariant("a bind over a stepped range written otherwise"));
        };
        let ends = match self.range_ends(m, parts, earlier, ctx)? {
            Ok(ends) => ends,
            Err(hole) => return Ok(Err(hole)),
        };
        if past_limit(m, &ends)? {
            return Err(refuse(Run::Limit, too_many_keys(m, &ends)?));
        }
        if !range_holds(m, &ends, &value)? {
            let text = format!(
                "{} is not a value of {}",
                value.text(m).map_err(num_stop)?,
                range_text(m, &ends)?
            );
            let seen = written
                && literal_end(&parts.0)
                && literal_end(&parts.1)
                && parts.2.as_ref().is_none_or(literal_end);
            return Err(if seen {
                refuse_load(Load::Key, text)
            } else {
                refuse(Run::Key, text)
            });
        }
        value_key(m, &value)?
            .map(Ok)
            .ok_or_else(|| invariant("a range's value that is no key"))
    }
}

/// A stepped range of numbers or money in its first end's unit: its values are `lo + i·step`
/// up to `stop` for a whole i ≥ 0 (refcore's `range_values`, its quantities). L §5.6, §2.5.
struct Stepped {
    /// The first end.
    lo: Num,
    /// The last end, in the first end's unit.
    stop: Num,
    /// The step, in the first end's unit: 1 where none is written.
    step: Num,
    /// The values' unit, the first end's (a currency's for money).
    unit: Unit,
    /// The values' class: an int's where the ends and the step are integers of no unit,
    /// else the first end's, a plain number's for an int or a bare figure.
    kind: Kind,
}

impl Stepped {
    /// The range `a .. b [step s]` of numbers, in refcore's order of checks: with no step,
    /// its ends' integers stepped by 1; with one, a step that isn't positive is `RUN-DOMAIN`,
    /// then the last end and the step are brought into the first end's unit (refcore's
    /// `range_values`). L §5.6, §5.11.
    ///
    /// # Errors
    ///
    /// `int_of`'s for an end with no step that isn't an integer; `RUN-DOMAIN`; `LOAD-UNIT`
    /// for units that can't be brought together; "not built" for a stepped range of other
    /// values.
    fn of(m: &Reality, (first, last, by): (&Value, &Value, Option<&Value>)) -> Result<Self, Stop> {
        let Some(by) = by else {
            return Ok(Stepped {
                lo: range_int(m, first)?,
                stop: range_int(m, last)?,
                step: Num::int(1),
                unit: Unit::one(),
                kind: Kind::Plain(Class::Int),
            });
        };
        let (Some(a), Some(b), Some(s)) = (operand(m, first)?, operand(m, last)?, operand(m, by)?)
        else {
            return Err(Stop::NotBuilt("a stepped range of values not numbers"));
        };
        if s.q.is_negative() || s.q.is_zero() {
            let step = by.text(m).map_err(num_stop)?;
            return Err(refuse(
                Run::Domain,
                format!("a stepped range whose step {step} is not positive"),
            ));
        }
        let unit = a.unit(m)?;
        let (stop, step) = (in_unit(m, &b, &unit)?, in_unit(m, &s, &unit)?);
        let kind = if [first, last, by].iter().all(|v| is_int(v)) {
            Kind::Plain(Class::Int)
        } else {
            values_kind(a.kind)
        };
        Ok(Stepped {
            lo: a.q,
            stop,
            step,
            unit,
            kind,
        })
    }

    /// Whether it holds more than `limit` values, counted and never stepped, as
    /// `⌊(b − a) ÷ s⌋ + 1 > limit` (refcore's `range_values`, before `too_many`). L §7.6 item 2.
    fn past(&self, limit: u64) -> Result<bool, Stop> {
        self.lo
            .range_past(&self.stop, &self.step, limit)
            .map_err(num_stop)
    }

    /// Its values, a range counted within the key limit, in order (refcore's
    /// `range_values`). L §5.6.
    fn values(&self, m: &Reality) -> Result<Vec<(Value, Option<Hole>)>, Stop> {
        let limit = u64::try_from(RANGE_LIMIT).map_err(|_| invariant("the key limit"))?;
        if self.kind == Kind::Plain(Class::Int) {
            let Some(keys) = self
                .lo
                .range_keys(&self.stop, &self.step, limit)
                .map_err(num_stop)?
            else {
                return Err(invariant("a range counted within the key limit"));
            };
            return Ok(keys.into_iter().map(int_member).collect());
        }
        let mut out = Vec::new();
        let mut value = self.lo.clone();
        while value.compare(&self.stop).map_err(num_stop)? != Ordering::Greater {
            let next = value.checked_add(&self.step).map_err(num_stop)?;
            let member = match &self.kind {
                // a value between two minor units: refcore has no form for it yet
                Kind::Money(cur) => match num_money(m, &value, *cur) {
                    Ok(money) => Value::Money(money),
                    Err(NumError::Domain) => {
                        return Err(Stop::NotBuilt(
                            "a money range stepped finer than its minor unit",
                        ));
                    }
                    Err(other) => return Err(num_stop(other)),
                },
                Kind::Plain(_) => finish(m, value, self.kind.clone())?,
            };
            out.push((member, None));
            value = next;
        }
        Ok(out)
    }

    /// Whether `key` is one of its values: a number or money of its unit that is `lo +
    /// i·step` up to `stop` for a whole i ≥ 0, a value of another kind never (refcore's
    /// `range_member`; an int is a value of a range of numbers, L §2.5).
    fn holds(&self, m: &Reality, key: &Value) -> Result<bool, Stop> {
        let Some(k) = operand(m, key)? else {
            return Ok(false);
        };
        if k.unit(m)? != self.unit {
            return Ok(false);
        }
        let offset = k.q.checked_sub(&self.lo).map_err(num_stop)?;
        if offset.is_negative() || k.q.compare(&self.stop).map_err(num_stop)? == Ordering::Greater {
            return Ok(false);
        }
        Ok(offset
            .checked_div(&self.step)
            .map_err(num_stop)?
            .is_integer())
    }
}

/// The class of a stepped range's values that aren't all integers: the first end's, a plain
/// number's for an int or a bare figure (refcore's `Num(cur, a.u, money=a.money, isint=…)`).
/// L §5.6.
fn values_kind(first: Kind) -> Kind {
    match first {
        Kind::Plain(Class::Int | Class::Flex) => Kind::Plain(Class::Num(Unit::one(), Shown::Plain)),
        Kind::Plain(class @ (Class::Num(..) | Class::Rate(_))) => Kind::Plain(class),
        Kind::Money(currency) => Kind::Money(currency),
    }
}

/// A range end with no step as an integer, whatever its unit, money's amount among them
/// (refcore's `int_of` in `range_values`). L §5.11.
fn range_int(m: &Reality, v: &Value) -> Result<Num, Stop> {
    match operand(m, v)? {
        Some(o) if o.q.is_integer() => Ok(o.q),
        Some(_) => Err(refuse(
            Run::NotInt,
            format!("{} is not an integer", v.text(m).map_err(num_stop)?),
        )),
        None => int_of(m, v).map(Num::int),
    }
}

/// A range's last end or step in its first end's unit (refcore's `U.conv` in
/// `range_values`); `LOAD-UNIT` where they can't be brought together. L §3.2, §5.11.
fn in_unit(m: &Reality, o: &Operand, unit: &Unit) -> Result<Num, Stop> {
    let own = o.unit(m)?;
    let factor = if own == *unit {
        Some(Num::int(1))
    } else {
        own.conv(unit).map_err(num_stop)?
    };
    let Some(factor) = factor else {
        return Err(refuse_load(
            Load::Unit,
            "a range of quantities in different units".to_owned(),
        ));
    };
    o.q.checked_mul(&factor).map_err(num_stop)
}

/// A value of a stepped range as a key: an integer of no unit an integer key, a day or a
/// period itself, any other number of its unit, or money, by its value and its text; none for
/// a value of another kind (refcore's `vkey` of a range's value). L §2.5, §5.6.
///
/// # Errors
///
/// A number error from its text.
pub(crate) fn value_key(m: &Reality, v: &Value) -> Result<Option<KeyVal>, Stop> {
    let value = match v {
        Value::Day(day) => return Ok(Some(KeyVal::Day(*day))),
        Value::Period(period) => return Ok(Some(KeyVal::Period(*period))),
        Value::Money(money) => Value::Money(money.clone()),
        Value::Number(n) => {
            let unit = n.unit();
            let plain = matches!(n.class, Class::Int | Class::Flex | Class::Num(..));
            if plain
                && unit.is_one()
                && let Some(int) = n.q.to_i64().filter(|_| n.q.is_integer())
            {
                return Ok(Some(KeyVal::Int(int)));
            }
            // a rate is keyed as the number of its unit it equals, as a written `2 %/year`
            // is: a key at a stepped range is tested by its number (L §2.5, P65)
            Value::Number(Number {
                q: n.q.clone(),
                class: Class::Num(unit, Shown::Plain),
            })
        }
        Value::Bool(_)
        | Value::MonthDay(..)
        | Value::Key(_)
        | Value::Text(_)
        | Value::Code(_)
        | Value::Record(_)
        | Value::List(_)
        | Value::Calendar(_)
        | Value::None
        | Value::Never
        | Value::Hole(_)
        | Value::Word(_)
        | Value::OutKey { .. }
        | Value::Mv(_)
        | Value::Window(_) => return Ok(None),
    };
    let text = value.text(m).map_err(num_stop)?;
    Ok(Some(KeyVal::Value(Box::new(RangeValue { value, text }))))
}

/// A key literal written at a stepped range's position as the key of its value: a number,
/// money, a day, a month or a quarter (refcore's `range_literal`); none for a literal that is
/// no value of a range, which its caller refuses. L §2.5.
///
/// # Errors
///
/// The literal's unit's refusal; a number error.
pub(crate) fn range_literal_key(m: &Reality, lit: &Lit) -> Result<Option<KeyVal>, Stop> {
    match lit {
        Lit::Num(_) | Lit::Day(_) | Lit::Month(..) | Lit::Quarter(..) => {
            value_key(m, &written_value(m, lit)?)
        }
        Lit::MonthDay(..)
        | Lit::Key(_)
        | Lit::Cur(_)
        | Lit::Bool(_)
        | Lit::Inf { .. }
        | Lit::Nominal(_)
        | Lit::Name(_) => Ok(None),
    }
}

/// A key literal at a stepped range's position as a value: a whole figure of no unit an int,
/// money finer than its minor unit a figure of the currency's unit, which no range of money
/// holds (refcore's `range_literal`, its `Int` and `Num(q, u, money=True)`). L §2.5, §3.2.
fn written_value(m: &Reality, lit: &Lit) -> Result<Value, Stop> {
    let Lit::Num(n) = lit else {
        return literal(m, lit, Figures::Rule);
    };
    let Some(unit) = &n.unit else {
        let class = if n.whole {
            Class::Int
        } else {
            Class::Num(Unit::one(), Shown::Plain)
        };
        return Ok(Value::Number(Number {
            q: n.value.clone(),
            class,
        }));
    };
    let terms: Vec<(String, Num)> = unit
        .terms
        .iter()
        .map(|t| (t.base.clone(), t.power.clone()))
        .collect();
    let (u, factor) = Unit::from_syntax(terms).map_err(unit_stop)?;
    let q = n.value.checked_mul(&factor).map_err(num_stop)?;
    if u.is_currency()
        && let [(name, _)] = u.terms()
    {
        let id = currency_id(m, name).map_err(num_stop)?;
        let (_, minor) = currency(m, id).map_err(num_stop)?;
        if q.checked_div(minor).map_err(num_stop)?.is_integer() {
            return num_money(m, &q, id).map(Value::Money).map_err(num_stop);
        }
    }
    Ok(Value::Number(Number {
        q,
        class: Class::Num(u, Shown::Plain),
    }))
}

/// Whether a key literal is one refcore's `is_keylit_node` reads as a key at a range's
/// position (its `_rangelit`): a day, a month, a month-day, a key, a currency, a name, or an
/// integer of no unit; a quarter and other figures are values. L §2.5.
fn key_literal(lit: &Lit) -> bool {
    match lit {
        Lit::Num(n) => n.whole && n.unit.is_none(),
        Lit::Day(_)
        | Lit::Month(..)
        | Lit::MonthDay(..)
        | Lit::Key(_)
        | Lit::Cur(_)
        | Lit::Name(_) => true,
        Lit::Quarter(..) | Lit::Bool(_) | Lit::Inf { .. } | Lit::Nominal(_) => false,
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
    Stepped::of(m, (first, last, by.as_ref()))?.past(limit)
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
    let last = range_last(m, end)?;
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
/// does (`range_member`): days stepped by days as `a + i·s`, days stepped by months by
/// stepping to it, numbers by `a + i·s` for a whole `i ≥ 0` up to `b`, a value of another
/// kind never. L §2.5, §5.6, §4.2.
fn range_holds(m: &Reality, (first, last, by): &Ends, key: &Value) -> Result<bool, Stop> {
    if let Value::Day(start) = first {
        let Some(by) = by else {
            return Err(Stop::NotBuilt("a range of days with no step"));
        };
        let Value::Day(day) = key else {
            return Ok(false);
        };
        let last = range_last(m, last)?;
        if *day <= *start || *start > last {
            return Ok(*day == *start && *start <= last);
        }
        // the first step, refused as stepping refuses it (no duration, or not forward)
        let next = day_step(m, *start, by)?;
        if steps_by_days(by) {
            return holds_by_days(m, (*start, next, last), by, *day);
        }
        let mut current = next;
        while current < *day && current <= last {
            current = day_step(m, current, by)?;
        }
        return Ok(current == *day && current <= last);
    }
    Stepped::of(m, (first, last, by.as_ref()))?.holds(m, key)
}

/// Whether a range's step moves a day by days alone: a whole number of days, or a count
/// (refcore's `date_arith`). L §4.2.
fn steps_by_days(by: &Value) -> bool {
    let Value::Number(n) = by else {
        return false;
    };
    duration(n).is_none_or(|(word, _)| word == "day")
}

/// Whether day `day`, after `start`, is one of a range's days `start + i·s` up to `last`, the
/// step s the days from `start` to `next`: the value before `day` found by its index, then
/// the range's step from it, refused where stepping one by one refuses (past the end of
/// time). One read of a long range costs a step, not the range (G13 review NB5). L §2.5,
/// §4.2, §5.6.
fn holds_by_days(
    m: &Reality,
    (start, next, last): (Day, Day, Day),
    by: &Value,
    day: Day,
) -> Result<bool, Stop> {
    let bad = || invariant("a forward step of days: `day_step` refuses one that isn't");
    let step = start.days_to(next).ok_or_else(bad)?;
    let gap = start.days_to(day).ok_or_else(bad)?;
    // Python's (gap - 1) // step * step: the offset of the range's last day before `day`
    let before = gap
        .checked_sub(1)
        .and_then(|g| g.checked_div_euclid(step))
        .and_then(|i| i.checked_mul(step))
        .ok_or_else(bad)?;
    let below = start.add_days(before).ok_or_else(bad)?;
    if below > last {
        return Ok(false);
    }
    let reached = if before == 0 {
        next
    } else {
        day_step(m, below, by)?
    };
    Ok(reached == day && reached <= last)
}

/// The days from `start` to `end`, stepped, a range counted within the key limit (refcore's
/// `range_values` for days). L §5.11.
fn day_range(
    m: &Reality,
    start: Day,
    (end, step): (&Value, &Value),
) -> Result<Vec<(Value, Option<Hole>)>, Stop> {
    let last = range_last(m, end)?;
    let mut out = Vec::new();
    let mut current = start;
    while current <= last {
        out.push((Value::Day(current), None));
        current = day_step(m, current, step)?;
    }
    Ok(out)
}

/// A day range's last day (refcore's `date_of(z, 'a range end')`): `never` is `RUN-NONE`; a
/// window `range_ends` made the read's hole first. L §5.6.
fn range_last(m: &Reality, end: &Value) -> Result<Day, Stop> {
    one_day(m, end, "a range end")?
        .map_err(|_| invariant("a window as a range end, which `range_ends` makes a hole first"))
}

/// A day of a range moved by its step (refcore's `date_arith` in `range_values`): a step
/// that isn't a duration is `LOAD-UNIT`, an interval of durations included, which moves a
/// day to no one day (L §4.4); one that doesn't move forward `RUN-DOMAIN`. L §4.2, §5.11.
fn day_step(m: &Reality, day: Day, step: &Value) -> Result<Day, Stop> {
    if let Value::Window(_) = step {
        let shown = step.text(m).map_err(num_stop)?;
        return Err(refuse_load(Load::Unit, format!("{} + {shown}", day.text())));
    }
    let Value::Day(next) = arith(m, Op::Add, &Value::Day(day), step)? else {
        return Err(invariant(
            "a day stepped to no day: `arith` refuses a step that isn't a duration first",
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
