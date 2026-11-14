//! Windows at run: a day not yet known through comparisons and arithmetic, its interval, and the hole a use that needs one value makes.
//! Spec: L §4.4 (arithmetic on a window gives an interval; a comparison is decided when the
//! whole interval lies on one side; a value compared with itself is decided; wherever one value
//! is needed, an undecided interval is `HOLE-IMPRECISE`), §3.1 (a window is a `date`), §8.1
//! (`HOLE-IMPRECISE`), §8.4.
//! Twin: engine/tools/refcore/evaluate.py (`iv_arith`, `iv_compare`, `imprecise`, `lift`'s end,
//! `e_neg` of an `Iv`, `c_window`, `c_one_of`, `date_of`); ask.py (`fmt_value` of an `Iv`).
//! Invariants: a window's low end is never above its high end; an operation on two windows,
//! and a use that needs one value, is the window's `HOLE-IMPRECISE`, asked of the row it came
//! from; a window keeps the text it was written with and its row's figure through every
//! operation that keeps it a window, but only a read of an item gives it a read, so a value
//! an operation computed is never decided against itself (#115, pick P28).

use core::cmp::Ordering;

use super::arith::{Op, Test, arith, compare, negate};
use super::expr::hole_of;
use super::state::{Ctx, Engine, Env, invariant, num_stop, refuse, refuse_load};
use crate::base::{Load, Run};
use crate::hole::Hole;
use crate::model::{KeyVal, ReadOf, Reality, Value, Window};
use crate::syntax::{Call, CmpOp, Stop};
use crate::time::{Day, PeriodKind};

/// The hole of window `w` met where one value is needed, by `what`: `?"date within <w> not
/// precise enough for <what>" (<its read>)` (refcore's `imprecise`). L §4.4, §8.1.
///
/// # Errors
///
/// A number error printing the window.
pub(super) fn imprecise(m: &Reality, w: &Window, what: &str) -> Result<Hole, Stop> {
    let window = w.question_text(m).map_err(num_stop)?;
    let text = format!("date within {window} not precise enough for {what}");
    Ok(Hole::asked(&text, w.figure()))
}

/// The window of a value, if it is one.
pub(super) fn as_window(v: &Value) -> Option<&Window> {
    if let Value::Window(w) = v {
        return Some(w);
    }
    None
}

/// An end of an interval: its lowest value or its highest (L §4.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum End {
    /// The lowest: a window's first day, `min`'s pick.
    Low,
    /// The highest: a window's last day, `max`'s pick.
    High,
}

/// A window's end `end`, any other value itself. L §4.4.
pub(super) fn window_end(v: &Value, end: End) -> Value {
    let Value::Window(w) = v else {
        return v.clone();
    };
    match end {
        End::Low => w.lo.clone(),
        End::High => w.hi.clone(),
    }
}

/// Value `v` as the value of one read of `item` at `keys` in `ctx`'s world, `last`'s day too
/// for `last(x, d)`: a window that is no read's value yet takes this read, so another read
/// alike is the same value (L §4.4: "Two reads of one item at the same keys, law date, cut
/// and pins are one value"). A window already a read's value keeps that read, so a def that
/// passes a read on is that read (L §4.4: "every def has one value in that world, whichever
/// def reads it"); any other value is itself.
pub(super) fn as_read(
    v: Value,
    (item, keys): (&str, &[KeyVal]),
    ctx: Ctx,
    last: Option<Day>,
) -> Value {
    let Value::Window(mut w) = v else {
        return v;
    };
    if w.read.is_none() {
        let world = match last {
            None => ctx.world(),
            Some(day) => format!("{}|last {}", ctx.world(), day.text()),
        };
        w.read = Some(ReadOf {
            item: item.to_owned(),
            keys: keys.to_vec(),
            world,
        });
    }
    Value::Window(w)
}

/// The interval of the values `a` and `b` a built-in gives at window `from`'s ends: one value
/// when they are equal, else from the lower to the higher, with `from`'s text and figure and
/// no read, a value computed (refcore's `lift`, its end). L §4.4.
pub(super) fn interval(m: &Reality, (a, b): (Value, Value), from: &Window) -> Result<Value, Stop> {
    let (lo, hi) = match compare(m, &a, &b, Test::Order)? {
        Ordering::Equal => return Ok(a),
        Ordering::Less => (a, b),
        Ordering::Greater => (b, a),
    };
    Ok(Value::Window(Box::new(Window {
        lo,
        hi,
        days: None,
        src: from.src.clone(),
        figure: from.figure.clone(),
        read: None,
    })))
}

/// `a op b` where one side is a window: the op at the window's ends, in order, a window with
/// its text and figure and no read, a value computed; on two windows `HOLE-IMPRECISE`
/// (refcore's `iv_arith`). L §4.4.
pub(super) fn window_arith(m: &Reality, op: Op, lhs: &Value, rhs: &Value) -> Result<Value, Stop> {
    let (lhs, rhs) = (single(lhs), single(rhs));
    let (w, lo, hi) = match (lhs, rhs) {
        (Value::Window(a), Value::Window(_)) => {
            return Ok(Value::Hole(imprecise(m, a, "arithmetic on two windows")?));
        }
        (Value::Window(a), b) => (a, arith(m, op, &a.lo, b)?, arith(m, op, &a.hi, b)?),
        (a, Value::Window(b)) => (b, arith(m, op, a, &b.lo)?, arith(m, op, a, &b.hi)?),
        // a window of one day is that day (F3)
        _ => return arith(m, op, lhs, rhs),
    };
    let (lo, hi) = if compare(m, &hi, &lo, Test::Order)? == Ordering::Less {
        (hi, lo)
    } else {
        (lo, hi)
    };
    Ok(Value::Window(Box::new(Window {
        lo,
        hi,
        days: None,
        src: w.src.clone(),
        figure: w.figure.clone(),
        read: None,
    })))
}

/// `-w` of a window of numbers: the window of its ends negated, in order (refcore's `e_neg`
/// of an `Iv`); `None` for a window of days, which has no negative. L §3.2, §4.4.
pub(super) fn window_negated(w: &Window) -> Result<Option<Value>, Stop> {
    let (Some(lo), Some(hi)) = (negate(&w.hi)?, negate(&w.lo)?) else {
        return Ok(None);
    };
    Ok(Some(Value::Window(Box::new(Window {
        lo,
        hi,
        days: None,
        src: w.src.clone(),
        figure: w.figure.clone(),
        read: None,
    }))))
}

/// The comparison `a op b` where a side is a window: decided when the whole interval lies on
/// one side, two reads alike of one item decided as one value, else `HOLE-IMPRECISE` naming
/// the comparison as written, `text` (refcore's `iv_compare`; its identity is the input row,
/// the spec's the read, #115). L §4.4.
pub(super) fn window_compare(
    m: &Reality,
    op: CmpOp,
    (a, b): (&Value, &Value),
    text: &str,
) -> Result<Value, Stop> {
    if one_read(a, b) {
        return Ok(Value::Bool(matches!(op, CmpOp::Eq | CmpOp::Le | CmpOp::Ge)));
    }
    let (a, b) = (single(a), single(b));
    if as_window(a).is_none() && as_window(b).is_none() {
        // a window of one day is that day (F3)
        let order = compare(m, a, b, Test::Order)?;
        let holds = match op {
            CmpOp::Lt => order == Ordering::Less,
            CmpOp::Le => order != Ordering::Greater,
            CmpOp::Gt => order == Ordering::Greater,
            CmpOp::Ge => order != Ordering::Less,
            CmpOp::Eq => order == Ordering::Equal,
            CmpOp::Ne => order != Ordering::Equal,
        };
        return Ok(Value::Bool(holds));
    }
    let ((alo, ahi), (blo, bhi)) = (ends(a), ends(b));
    let order = |x: &Value, y: &Value| compare(m, x, y, Test::Order);
    match op {
        CmpOp::Lt | CmpOp::Le => {
            let strict = op == CmpOp::Lt;
            let below = order(ahi, blo)?;
            if below == Ordering::Less || (!strict && below == Ordering::Equal) {
                return Ok(Value::Bool(true));
            }
            let above = order(alo, bhi)?;
            if above == Ordering::Greater || (strict && above == Ordering::Equal) {
                return Ok(Value::Bool(false));
            }
        }
        CmpOp::Gt | CmpOp::Ge => {
            let strict = op == CmpOp::Gt;
            let above = order(alo, bhi)?;
            if above == Ordering::Greater || (!strict && above == Ordering::Equal) {
                return Ok(Value::Bool(true));
            }
            let below = order(ahi, blo)?;
            if below == Ordering::Less || (strict && below == Ordering::Equal) {
                return Ok(Value::Bool(false));
            }
        }
        CmpOp::Eq | CmpOp::Ne => {
            let apart = order(ahi, blo)? == Ordering::Less || order(bhi, alo)? == Ordering::Less;
            let apart = match (a, b) {
                (Value::Window(x), other) | (other, Value::Window(x))
                    if !matches!(other, Value::Window(_)) && x.days.is_some() =>
                {
                    !one_of_holds(x, other)
                }
                _ => apart,
            };
            if apart {
                return Ok(Value::Bool(op == CmpOp::Ne));
            }
        }
    }
    let w = as_window(a)
        .or_else(|| as_window(b))
        .ok_or_else(|| invariant("a window comparison with no window"))?;
    Ok(Value::Hole(imprecise(m, w, text)?))
}

/// Whether two values are windows that are two reads alike of one item, so one value (L §4.4:
/// "Two reads of one item at the same keys, law date, cut and pins are one value").
pub(super) fn one_read(a: &Value, b: &Value) -> bool {
    if let (Value::Window(x), Value::Window(y)) = (a, b) {
        return x.read.is_some() && x.read == y.read;
    }
    false
}

/// The order of two values where a side may be a window, where it is decided: below (above)
/// when the whole interval lies below (above) the other, equal for two reads alike of one
/// item; none where the intervals meet, an order an aggregate can't take (L §4.4; refcore's
/// `iv_compare`, its identity the input row's, #115).
///
/// # Errors
///
/// The refusals of comparing the values or their ends.
pub(super) fn decided_order(m: &Reality, a: &Value, b: &Value) -> Result<Option<Ordering>, Stop> {
    if one_read(a, b) {
        return Ok(Some(Ordering::Equal));
    }
    let (a, b) = (single(a), single(b));
    if as_window(a).is_none() && as_window(b).is_none() {
        return compare(m, a, b, Test::Order).map(Some);
    }
    let ((alo, ahi), (blo, bhi)) = (ends(a), ends(b));
    if compare(m, ahi, blo, Test::Order)? == Ordering::Less {
        return Ok(Some(Ordering::Less));
    }
    if compare(m, alo, bhi, Test::Order)? == Ordering::Greater {
        return Ok(Some(Ordering::Greater));
    }
    Ok(None)
}

/// A window of one day is that day; any other value is itself (refcore's `one_day`; tally
/// 2.1.7 R3, F3). L §4.4.
pub(super) fn single(v: &Value) -> &Value {
    if let Value::Window(w) = v
        && w.days.as_ref().is_none_or(|days| days.len() == 1)
        && w.lo == w.hi
    {
        return &w.lo;
    }
    v
}

/// A value's interval: a window's ends, any other value twice. L §4.4.
fn ends(v: &Value) -> (&Value, &Value) {
    if let Value::Window(w) = v {
        return (&w.lo, &w.hi);
    }
    (v, v)
}

/// Whether `one_of` window `w` holds the value `v` among its days. L §4.4.
fn one_of_holds(w: &Window, v: &Value) -> bool {
    match (&w.days, v) {
        (Some(days), Value::Day(d)) => days.contains(d),
        _ => false,
    }
}

/// One day where one is needed, the one reader every such site uses (`what` names the site):
/// a day (the end of time is none), a day period's day; a window is its `HOLE-IMPRECISE` for
/// `what`; `none` and `never` refused (refcore's `date_of`). L §4.2, §4.4, §7.6 item 1.
///
/// # Errors
///
/// `RUN-NONE`, `RUN-LIMIT`; `LOAD-UNIT` for a value that is no date.
pub(crate) fn one_day(m: &Reality, v: &Value, what: &str) -> Result<Result<Day, Hole>, Stop> {
    let v = single(v);
    if let Value::Day(d) = v {
        return Ok(Ok(super::state::a_day(*d)?));
    }
    if let Value::Window(w) = v {
        return Ok(Err(imprecise(m, w, what)?));
    }
    if let Value::Period(p) = v
        && p.kind() == PeriodKind::Day
    {
        return Ok(Ok(p.start()));
    }
    if *v == Value::None {
        return Err(refuse(Run::None, format!("a value is none here: {what}")));
    }
    if *v == Value::Never {
        return Err(refuse(Run::None, format!("never in {what}")));
    }
    let shown = v.text(m).map_err(num_stop)?;
    Err(refuse_load(
        Load::Unit,
        format!("{what} is {shown}, not a date"),
    ))
}

impl Engine {
    /// `window(a, b)`: one day from a to b, not yet known (refcore's `c_window`). L §4.4, §8.4.
    pub(super) fn window_call(
        &mut self,
        m: &Reality,
        call: &Call,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let [a, b] = self
            .args(m, call, env, ctx)?
            .try_into()
            .map_err(|_| invariant("a built-in called with the wrong number of arguments"))?;
        if a.as_hole().is_some() || b.as_hole().is_some() {
            return hole_of(&[a.as_hole(), b.as_hole()]);
        }
        let first = match one_day(m, &a, "window")? {
            Ok(d) => d,
            Err(hole) => return Ok(Value::Hole(hole)),
        };
        let last = match one_day(m, &b, "window")? {
            Ok(d) => d,
            Err(hole) => return Ok(Value::Hole(hole)),
        };
        let written = format!("window({}, {})", first.text(), last.text());
        if first > last {
            return Err(refuse(Run::Domain, format!("{written} with a after b")));
        }
        Ok(Value::Window(Box::new(Window {
            lo: Value::Day(first),
            hi: Value::Day(last),
            days: None,
            src: Some(written),
            figure: None,
            read: None,
        })))
    }

    /// `one_of(ds)`: one of a list's days, not yet known (refcore's `c_one_of`). L §4.4, §8.4.
    pub(super) fn one_of_call(
        &mut self,
        m: &Reality,
        call: &Call,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let arg = call
            .args
            .first()
            .ok_or_else(|| invariant("a built-in called with the wrong number of arguments"))?;
        let v = self.ev(m, arg, env, ctx)?;
        if let Value::Hole(_) = v {
            return Ok(v);
        }
        let Value::List(items) = &v else {
            // load's types refuse it; refcore's run has no twin of this text
            return Err(Stop::NotBuilt(
                "one_of of a value that isn't a list, met at run",
            ));
        };
        let mut days = Vec::with_capacity(items.len());
        for item in items {
            match one_day(m, item, "one_of")? {
                Ok(d) => days.push(d),
                Err(hole) => return Ok(Value::Hole(hole)),
            }
        }
        if days.is_empty() {
            return Err(refuse(Run::Domain, "one_of of no days".to_owned()));
        }
        Ok(Value::Window(Box::new(super::convert::one_of(&days)?)))
    }
}

#[cfg(test)]
mod tests {
    use super::{Value, Window, ends, one_of_holds};
    use crate::time::Day;

    fn day(y: u16, m: u8, d: u8) -> Day {
        Day::from_ymd(y, m, d).expect("a real day")
    }

    #[test]
    fn a_one_of_holds_only_its_days() {
        let w = Window {
            lo: Value::Day(day(2026, 6, 1)),
            hi: Value::Day(day(2026, 6, 5)),
            days: Some(vec![day(2026, 6, 1), day(2026, 6, 5)]),
            src: None,
            figure: None,
            read: None,
        };
        assert!(
            one_of_holds(&w, &Value::Day(day(2026, 6, 5))),
            "a day of it"
        );
        assert!(!one_of_holds(&w, &Value::Day(day(2026, 6, 3))), "between");
        let plain = Value::Day(day(2026, 6, 3));
        let (lo, hi) = ends(&plain);
        assert_eq!((lo, hi), (&plain, &plain), "a value is its own interval");
    }
}
