//! Windows at run: a day not yet known through comparisons and arithmetic, its interval, and the hole a use that needs one value makes.
//! Spec: L §4.4 (arithmetic on a window gives an interval; a comparison is decided when the
//! whole interval lies on one side; a value compared with itself is decided; wherever one value
//! is needed, an undecided interval is `HOLE-IMPRECISE`), §3.1 (a window is a `date`), §8.1
//! (`HOLE-IMPRECISE`), §8.4.
//! Twin: engine/tools/refcore/evaluate.py (`iv_arith`, `iv_compare`, `imprecise`, `lift`'s end,
//! `e_neg` of an `Iv`, `c_window`, `c_one_of`, `date_of`); ask.py (`fmt_value` of an `Iv`).
//! Invariants: a window's low end is never above its high end; an operation on two windows
//! gives the interval of its values at their ends' corners; a use that needs one value is the
//! window's `HOLE-IMPRECISE`, asked of the row it came from; a window keeps the text it was written with and its row's figure through every
//! operation that keeps it a window, but only a read of an item gives it a read, so a value
//! an operation computed is never decided against itself (#115, pick P28); a window a read
//! gives carries its unknown day, named by that read, and every interval and hole made of it
//! carries it too, for the register's `unknown day:` (L §4.4, §8.2).

use core::cmp::Ordering;

use super::arith::{Op, Test, arith, compare, negate};
use super::expr::hole_of;
use super::state::{Ctx, Engine, Env, invariant, num_stop, refuse, refuse_load};
use crate::base::{Code, Load, Run};
use crate::hole::{Hole, Unknown};
use crate::model::{Class, KeyVal, Number, ReadOf, Reality, Value, Window};
use crate::num::Num;
use crate::syntax::{Call, CmpOp, Stop};
use crate::time::{Day, PeriodKind};

/// The hole of window `w` met where one value is needed, by `what`: `?"date within <w> not
/// precise enough for <what>" (<its read>)`, one question per unknown day where it rests on
/// two or more, each asked of the day's own name (refcore's `imprecise`, `imprecise_hole`).
/// L §4.4, §8.1 (R21 on #241).
///
/// # Errors
///
/// A number error printing the window.
pub(super) fn imprecise(m: &Reality, w: &Window, what: &str) -> Result<Hole, Stop> {
    let window = w.question_text(m).map_err(num_stop)?;
    Ok(crate::hole::imprecise(
        (&window, what),
        w.figure(),
        (&w.carry, None),
    ))
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
pub(crate) enum End {
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

/// Value `v` as the value of one read of `item` at `keys` at cell `cell` (a def's cell's
/// world, an input's row): a window that is no read's value yet takes this read, so another
/// read of that cell is the same value (L §4.4: "named by the read that gave it: its cell,
/// the item at its keys with the law date, cut and pins it depends on"). A window already a
/// read's value keeps that read, so a def that passes a read on is that read (L §4.4: "A
/// copy keeps the name"); any other value is itself.
pub(super) fn as_read(v: Value, (item, keys): (&str, &[KeyVal]), cell: String) -> Value {
    let Value::Window(mut w) = v else {
        return v;
    };
    if w.read.is_none() {
        w.read = Some(ReadOf {
            item: item.to_owned(),
            keys: keys.to_vec(),
            cell,
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
        carry: from.carry.clone(),
        whole: false,
    })))
}

/// The values `outs` an operation takes over windows' days: one value where they agree,
/// else their interval, with the text of window `from` and no read (refcore's
/// `interval_of`). L §4.4.
///
/// # Errors
///
/// As `compare`; `ENGINE-INVARIANT` for no value.
pub(super) fn interval_of(m: &Reality, outs: Vec<Value>, from: &Window) -> Result<Value, Stop> {
    let mut rest = outs.into_iter();
    let first = rest
        .next()
        .ok_or_else(|| invariant("an interval of no value"))?;
    let (mut lo, mut hi) = (first.clone(), first);
    for v in rest {
        if compare(m, &v, &lo, Test::Order)? == Ordering::Less {
            lo = v.clone();
        }
        if compare(m, &hi, &v, Test::Order)? == Ordering::Less {
            hi = v;
        }
    }
    interval(m, (lo, hi), from)
}

/// `a op b` where a side is a window (refcore's `iv_arith`): beside a value that is none, at
/// the window's ends, arithmetic being monotone in each side, one value where they agree; on
/// two windows, and where a divisor's interval holds 0, through [`value_choices`]: their
/// corners or whole numbers in turn, one window named on both sides as one day, a divisor's
/// values in turn or the hole where they can't be taken. A result of `+`, `-` or `*` of whole
/// numbers stays whole. L §4.4 l.668 ("from their ends where it is monotone in them
/// (arithmetic …)"), l.671; P30 (B3, N6).
///
/// # Errors
///
/// The refusal every value gives; any other stop of the arithmetic.
pub(super) fn window_arith(m: &Reality, op: Op, lhs: &Value, rhs: &Value) -> Result<Value, Stop> {
    let (lhs, rhs) = (single(lhs), single(rhs));
    let whole = matches!(op, Op::Add | Op::Sub | Op::Mul) && whole_valued(lhs) && whole_valued(rhs);
    let zero_divisor = op == Op::Div && holds_zero(m, rhs)?;
    let (w, lo, hi) = match (lhs, rhs) {
        (Value::Window(_), Value::Window(_)) => return arith_values(m, op, (lhs, rhs)),
        _ if zero_divisor => return arith_values(m, op, (lhs, rhs)),
        (Value::Window(a), b) => (a, arith(m, op, &a.lo, b)?, arith(m, op, &a.hi, b)?),
        (a, Value::Window(b)) => (b, arith(m, op, a, &b.lo)?, arith(m, op, a, &b.hi)?),
        // a window of one day is that day (F3)
        _ => return arith(m, op, lhs, rhs),
    };
    let (lo, hi) = match compare(m, &hi, &lo, Test::Order)? {
        // one value where its ends agree (L §4.4 l.668)
        Ordering::Equal => return Ok(lo),
        Ordering::Less => (hi, lo),
        Ordering::Greater => (lo, hi),
    };
    Ok(Value::Window(Box::new(Window {
        lo,
        hi,
        days: None,
        src: w.src.clone(),
        figure: w.figure.clone(),
        read: None,
        carry: w.carry.clone(),
        whole,
    })))
}

/// `a op b` through [`value_choices`] and [`values_interval`]: two windows, monotone in each,
/// or a divisor whose interval holds 0, whose values stand for it only taken in turn; whole
/// where `+`, `-` or `*` of whole numbers keeps it whole (refcore's `iv_arith`, its
/// `lift_values`). L §4.4 l.668, l.671; P30.
///
/// # Errors
///
/// As [`window_arith`].
fn arith_values(m: &Reality, op: Op, (lhs, rhs): (&Value, &Value)) -> Result<Value, Stop> {
    let whole = matches!(op, Op::Add | Op::Sub | Op::Mul) && whole_valued(lhs) && whole_valued(rhs);
    let shape = if op == Op::Div && holds_zero(m, rhs)? {
        Shape::Values
    } else {
        Shape::Monotone
    };
    let tuples = match value_choices(m, ("arithmetic", shape), &[lhs, rhs])? {
        Choices::Plain => return arith(m, op, lhs, rhs),
        Choices::Hole(h) => return Ok(Value::Hole(h)),
        Choices::Tuples(tuples) => tuples,
    };
    let mut outs = Vec::with_capacity(tuples.len());
    for t in &tuples {
        let [x, y] = t.as_slice() else {
            return Err(invariant("an operator with no two operands"));
        };
        outs.push(arith(m, op, x, y));
    }
    let v = values_interval(m, "arithmetic", outs, &[lhs, rhs])?;
    let Value::Window(mut w) = v else {
        return Ok(v);
    };
    w.whole = whole;
    Ok(Value::Window(w))
}

/// Whether `v` is an interval of numbers whose values hold 0 (refcore's `b.lo.q <= 0 <=
/// b.hi.q`): a divisor that divides by 0 on some of its values. L §4.4 l.671; P30.
///
/// # Errors
///
/// The refusals of comparing its ends with 0.
fn holds_zero(m: &Reality, v: &Value) -> Result<bool, Stop> {
    let Value::Window(w) = v else {
        return Ok(false);
    };
    if !of_numbers(w) {
        return Ok(false);
    }
    let zero = arith(m, Op::Sub, &w.lo, &w.lo)?;
    Ok(compare(m, &w.lo, &zero, Test::Order)? != Ordering::Greater
        && compare(m, &zero, &w.hi, Test::Order)? != Ordering::Greater)
}

/// Whether interval `w` is one of numbers or money, which has a 0 (refcore's `isinstance(lo,
/// Num)`), not of days or periods. L §4.4.
fn of_numbers(w: &Window) -> bool {
    matches!(
        (&w.lo, &w.hi),
        (
            Value::Number(_) | Value::Money(_),
            Value::Number(_) | Value::Money(_)
        )
    )
}

/// Whether every value `v` can hold is a whole number: an `int`, a number whose value is
/// whole, an interval of `int`s or one kept whole (refcore's `whole_valued`). L §4.4; P30.
fn whole_valued(v: &Value) -> bool {
    match v {
        Value::Number(n) => n.class == Class::Int || n.q.is_integer(),
        Value::Window(w) => whole_range(w).is_some(),
        Value::Money(_)
        | Value::Bool(_)
        | Value::Day(_)
        | Value::Period(_)
        | Value::MonthDay(..)
        | Value::Key(_)
        | Value::Text(_)
        | Value::Code(_)
        | Value::Record(_)
        | Value::List(_)
        | Value::Calendar(_)
        | Value::Hole(_)
        | Value::None
        | Value::Never
        | Value::Word(_)
        | Value::Mv(_)
        | Value::OutKey { .. } => false,
    }
}

/// The ends of interval `w` where every value it can hold is a whole number: its ends `int`s,
/// or it kept whole (`Window::whole`); none for any other interval (refcore's
/// `whole_valued`). L §4.4; P30.
fn whole_range(w: &Window) -> Option<(i64, i64)> {
    let (Value::Number(lo), Value::Number(hi)) = (&w.lo, &w.hi) else {
        return None;
    };
    let ints = lo.class == Class::Int && hi.class == Class::Int;
    if !(w.whole || ints) || !lo.q.is_integer() || !hi.q.is_integer() {
        return None;
    }
    Some((lo.q.to_i64()?, hi.q.to_i64()?))
}

/// How an operation stands in an argument that is an interval (L §4.4 l.668; P30).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Shape {
    /// Monotone in it and giving a value everywhere between its ends: its two ends stand for
    /// its values, with 0 where a number's interval crosses it (`abs`, an even `pow`: its
    /// shape).
    Monotone,
    /// Neither: only its values taken in turn stand for it.
    Values,
}

/// What an operation over intervals is read at (refcore's `lift_values`, its choices).
pub(super) enum Choices {
    /// No argument is an interval: the operation on its arguments as they are.
    Plain,
    /// The argument tuples it is read at, every combination in order.
    Tuples(Vec<Vec<Value>>),
    /// An interval whose values can't be taken in turn, where its ends don't stand for them:
    /// not decided for the whole interval, `HOLE-IMPRECISE`.
    Hole(Hole),
}

/// The tuples taken in turn before an interval is its hole (refcore's `LIFT_CAP`; P30: a cap
/// past which the values aren't taken, never a fallback to the ends).
const VALUES_CAP: u64 = 100_000;

/// The whole numbers interval `w` holds, in order, in its low end's class, with `size` the
/// tuples counted so far, times their count; none past [`VALUES_CAP`] (refcore's
/// `lift_values`, its whole numbers). L §4.4; P30.
fn whole_values(w: &Window, size: &mut u64) -> Option<Vec<Value>> {
    let (first, last) = whole_range(w)?;
    let Value::Number(lo) = &w.lo else {
        return None;
    };
    // an interval's ends are in order, so its width is never negative
    let Ok(width) = u64::try_from(last.checked_sub(first)?) else {
        return None;
    };
    let count = width.checked_add(1)?;
    let total = size.checked_mul(count)?;
    if total > VALUES_CAP {
        return None;
    }
    *size = total;
    let mut values = Vec::new();
    for k in first..=last {
        values.push(Value::Number(Number {
            q: Num::int(k),
            class: lo.class.clone(),
        }));
    }
    Some(values)
}

/// Interval `w`'s two ends, and 0 between them where it is an interval of numbers that
/// crosses 0 (refcore's `lift_values`, its `mono` choices). L §4.4 l.668.
///
/// # Errors
///
/// The refusals of comparing its ends with 0.
fn interval_ends(m: &Reality, w: &Window) -> Result<Vec<Value>, Stop> {
    if !of_numbers(w) {
        return Ok(vec![w.lo.clone(), w.hi.clone()]);
    }
    let zero = arith(m, Op::Sub, &w.lo, &w.lo)?;
    let crosses = compare(m, &w.lo, &zero, Test::Order)? == Ordering::Less
        && compare(m, &zero, &w.hi, Test::Order)? == Ordering::Less;
    if crosses {
        return Ok(vec![w.lo.clone(), zero, w.hi.clone()]);
    }
    Ok(vec![w.lo.clone(), w.hi.clone()])
}

/// Whether windows `a` and `b` are one window named twice: two reads alike of one item's
/// window of days, one unknown day (refcore's `lift_values`, its named windows). L §4.4
/// l.667; P30 (N9).
fn one_window(a: &Window, b: &Window) -> bool {
    a.of_days() && b.of_days() && a.read.is_some() && a.read == b.read
}

/// The tuples of `args` at each day of window `w`, the day in every argument that is a
/// window: one unknown day taken day by day (refcore's `per_day`). L §4.4; P30.
///
/// # Errors
///
/// As [`window_days`].
fn day_tuples(w: &Window, args: &[&Value]) -> Result<Vec<Vec<Value>>, Stop> {
    let days = window_days(w)?;
    let mut tuples = Vec::with_capacity(days.len());
    for d in days {
        let mut t = Vec::with_capacity(args.len());
        for v in args {
            t.push(match as_window(v) {
                Some(_) => Value::Day(d),
                None => (*v).clone(),
            });
        }
        tuples.push(t);
    }
    Ok(tuples)
}

/// The argument tuples an operator or a built-in is read at where some of `args` are
/// intervals: the one mechanism every operation over an interval goes through (refcore's
/// `lift_values`; P30). One window named in two arguments is one day, its days in turn, as in
/// a comparison. Otherwise, where the operation is [`Shape::Monotone`] in an interval, at its
/// two ends and 0 where a number's interval crosses it, whatever its size; where it isn't, at
/// the whole numbers it holds, in turn, up to 100 000 tuples, and beyond them or where they
/// can't be taken in turn the hole for `what`, carrying every interval's unknown days. Every
/// other argument as it is, every combination in order; [`Choices::Plain`] where none is an
/// interval. L §4.4 l.667 to l.671 ("each operation gives the interval of its values over the
/// days its own arguments can take"); P41 (F1, F8).
///
/// # Errors
///
/// The refusals of comparing an interval's ends with 0, or of its days.
pub(super) fn value_choices(
    m: &Reality,
    (what, shape): (&str, Shape),
    args: &[&Value],
) -> Result<Choices, Stop> {
    let args: Vec<&Value> = args.iter().map(|v| single(v)).collect();
    let windows: Vec<&Window> = args.iter().filter_map(|v| as_window(v)).collect();
    let [first, rest @ ..] = windows.as_slice() else {
        return Ok(Choices::Plain);
    };
    if !rest.is_empty() && rest.iter().all(|w| one_window(first, w)) {
        return Ok(Choices::Tuples(day_tuples(first, &args)?));
    }
    let mut tuples: Vec<Vec<Value>> = vec![Vec::new()];
    let mut size: u64 = 1;
    for v in &args {
        let options = match (as_window(v), shape) {
            (None, _) => vec![(*v).clone()],
            // its ends stand for its values whatever its size (P41, F1)
            (Some(w), Shape::Monotone) => interval_ends(m, w)?,
            (Some(w), Shape::Values) => match whole_values(w, &mut size) {
                Some(values) => values,
                None => return Ok(Choices::Hole(imprecise_all(m, &windows, what)?)),
            },
        };
        let count = tuples
            .len()
            .checked_mul(options.len())
            .ok_or_else(|| invariant("an argument tuple count past the integers"))?;
        let mut next = Vec::with_capacity(count);
        for t in &tuples {
            for o in &options {
                next.push([t.as_slice(), core::slice::from_ref(o)].concat());
            }
        }
        tuples = next;
    }
    Ok(Choices::Tuples(tuples))
}

/// An operation's value over intervals, from `outs`, its value at each of [`value_choices`]'
/// tuples of `args`, in order: one value where they agree, else their interval, a window
/// among them taken by its ends, a computed value with the first interval's text, carrying
/// every interval's unknown days; a hole among them is the value, a `none` among them the
/// hole; a run refusal at every tuple is the first, at only some of them the first interval's
/// `HOLE-IMPRECISE` for `what` (refcore's `lift_values`). L §4.4 l.668, l.671 ("and for a
/// refusal that only some of its days give").
///
/// # Errors
///
/// The first run refusal where every tuple gives one; any other stop as it came.
pub(super) fn values_interval(
    m: &Reality,
    what: &str,
    outs: Vec<Result<Value, Stop>>,
    args: &[&Value],
) -> Result<Value, Stop> {
    let windows: Vec<&Window> = args.iter().filter_map(|v| as_window(single(v))).collect();
    let [first, rest @ ..] = windows.as_slice() else {
        return Err(invariant("an interval's value with no interval"));
    };
    let mut values = Vec::with_capacity(outs.len());
    let (mut first_refusal, mut refused) = (None, false);
    for out in outs {
        match out {
            Ok(Value::Hole(h)) => return Ok(Value::Hole(h)),
            Ok(Value::None) => {
                let none = "a built-in that is none on part of the window";
                return Ok(Value::Hole(imprecise_all(m, &windows, none)?));
            }
            Ok(Value::Window(w)) => values.extend([w.lo, w.hi]),
            Ok(v) => values.push(v),
            Err(Stop::Refused(r)) if matches!(r.code(), Code::Run(_)) => {
                refused = true;
                first_refusal.get_or_insert(r);
            }
            Err(other) => return Err(other),
        }
    }
    if values.is_empty() {
        let refusal = first_refusal.ok_or_else(|| invariant("an interval of no value"))?;
        return Err(Stop::Refused(refusal));
    }
    if refused {
        return Ok(Value::Hole(imprecise_all(m, &windows, what)?));
    }
    let mut from = (*first).clone();
    for w in rest {
        crate::hole::carry_union(&mut from.carry, &w.carry);
    }
    interval_of(m, values, &from)
}

/// [`imprecise`] of the first of `windows`, carrying the unknown days of every one: a hole an
/// operation over them makes names every window its operands carry (L §4.4 l.667, l.671;
/// P41, F8).
///
/// # Errors
///
/// As [`imprecise`]; `ENGINE-INVARIANT` for no window.
fn imprecise_all(m: &Reality, windows: &[&Window], what: &str) -> Result<Hole, Stop> {
    let [first, rest @ ..] = windows else {
        return Err(invariant("an interval's hole with no interval"));
    };
    let mut from = (*first).clone();
    for w in rest {
        crate::hole::carry_union(&mut from.carry, &w.carry);
    }
    imprecise(m, &from, what)
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
        carry: w.carry.clone(),
        whole: false,
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

/// The days window `w` can take: a `one_of`'s, else every day from its first to its last.
/// L §4.4.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a window of numbers; `RUN-LIMIT` past the calendar's edge.
pub(crate) fn window_days(w: &Window) -> Result<Vec<Day>, Stop> {
    if let Some(days) = &w.days {
        return Ok(days.clone());
    }
    let (Value::Day(first), Value::Day(last)) = (&w.lo, &w.hi) else {
        return Err(super::state::not_typed());
    };
    let mut out = vec![*first];
    let mut day = *first;
    while day < *last {
        day = day.next().ok_or_else(|| super::state::past_days(1))?;
        out.push(day);
    }
    Ok(out)
}

/// Value `v` named by the read that gave it: a window of days is the one unknown day `id` (a
/// read's row and figure, or the window as written) labelled `label`, and carries that day
/// alone, whatever it carried before, a written window's own text included; any other value
/// is left as it is (refcore's `window_id`, `iv_carry`: the read's new `Iv`, no carry kept).
/// L §4.4 (one unknown day, named by its read), §8.2 (`unknown day:`).
pub(super) fn named_window(v: Value, (id, label): (String, &str)) -> Value {
    let Value::Window(mut w) = v else {
        return v;
    };
    let (Value::Day(lo), Value::Day(hi)) = (&w.lo, &w.hi) else {
        return Value::Window(w);
    };
    // its question names the window as written, else by its days (refcore's `window_text`)
    let text = match &w.src {
        Some(src) => src.clone(),
        None => format!("window({}, {})", lo.text(), hi.text()),
    };
    w.carry = vec![Unknown {
        id,
        label: label.to_owned(),
        text,
    }];
    Value::Window(w)
}

/// Value `value`, a window no cell holds or copies yet (written in the text, or computed from
/// windows written there), as the cell `figure` that holds it, its unknown day `id` named by
/// that cell and its holes asked of it; a window that came from a read, or was computed from
/// one or from a calendar's entry, keeps the names and the days it carries; any other value is
/// itself (refcore's `named`, `pinned_read`, a move's `on`). L §4.4 ("a copy keeps the name",
/// "a computed value … carries the names it was computed from"), §8.2 `unknown day:` (R21:
/// "the cell a caller pins, wherever a cell holds or copies the day"; P30, P32, P26).
pub(crate) fn held_by(value: Value, (id, figure): (String, &str)) -> Value {
    let Value::Window(mut w) = value else {
        return value;
    };
    if w.figure.is_some() || !names_only_written(&w.carry) {
        return Value::Window(w);
    }
    w.figure = Some(figure.to_owned());
    named_window(Value::Window(w), (id, figure))
}

/// The unknown day a window written in the text is, its id: the window as written (refcore's
/// `window_id` of an `Iv` with no origin). L §4.4.
fn written_id(text: &str) -> String {
    format!("src {text}")
}

/// Whether the unknown days `carry` holds are only windows written in the text, each named by
/// its own text, so the value carries no name of its own: no read's, no calendar entry's
/// (refcore's `Iv.origin is None`). L §4.4.
pub(super) fn names_only_written(carry: &[Unknown]) -> bool {
    carry.iter().all(|u| u.id.starts_with(&written_id("")))
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
        let window = Value::Window(Box::new(Window {
            lo: Value::Day(first),
            hi: Value::Day(last),
            days: None,
            src: Some(written.clone()),
            figure: None,
            read: None,
            carry: Vec::new(),
            whole: false,
        }));
        Ok(named_window(window, (written_id(&written), &written)))
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
        let window = super::convert::one_of(&days)?;
        let written = window
            .src
            .clone()
            .ok_or_else(|| invariant("a one_of with no text"))?;
        Ok(named_window(
            Value::Window(Box::new(window)),
            (written_id(&written), &written),
        ))
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
            carry: Vec::new(),
            whole: false,
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
