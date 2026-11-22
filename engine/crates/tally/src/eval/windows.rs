//! Windows at run: a day not yet known, the closed list of readers that take it, and the hole every other read of it is.
//! Spec: L §4.4 (a window is a stretch of dates; these readers take it, and no other: date to
//! date by the ends, a comparison decided where it holds for every day, a plain result where
//! every combination of the arguments' first and last days gives the same, a def's `at`, a
//! print, a `with` pin; every other read is `HOLE-IMPRECISE` naming each unknown day; no number
//! is a range), §3.1 (a window is a `date`), §8.1 (`HOLE-IMPRECISE`), §8.4.
//! Twin: engine/tools/refcore/evaluate.py (`iv_compare`, `imprecise`, `lift`, `c_window`,
//! `c_one_of`, `date_of`); ask.py (`fmt_value` of an `Iv`).
//! Invariants: a window's first day is never after its last; a listed reader of a ranged date
//! reads it at its ends only, two evaluations a ranged argument, each argument by its own ends
//! whatever name it carries; every other read of one is its `HOLE-IMPRECISE`, asked of the row
//! it came from, made here; a window keeps the text it was written with and its row's figure
//! through every reader that keeps it a stretch, but only a read of an item gives it a read, so
//! a stretch a reader computed is never decided against itself (#115, pick P28); a window a read
//! gives carries its unknown day, named by that read, and every stretch and hole made of it
//! carries it too, for the register's `unknown day:` (L §4.4, §8.2).

use core::cmp::Ordering;
use std::borrow::Cow;

use super::arith::{Op, Test, arith, compare};
use super::expr::hole_of;
use super::state::{Ctx, Engine, Env, invariant, num_stop, refuse, refuse_load};
use crate::base::{Code, Load, Run};
use crate::hole::{Hole, Unknown};
use crate::model::{KeyVal, ReadOf, Reality, Value, Window};
use crate::syntax::{Call, CmpOp, Stop};
use crate::time::{Day, PeriodKind};

/// How a listed reader takes a ranged date (L §4.4's table, "These readers take it, and no
/// other").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Takes {
    /// Date to date, by the ends: the stretch from its result at the first day to its result
    /// at the last.
    Ends,
    /// A comparison of dates, and `argmin`, `argmax`, which order by it: decided where it
    /// holds for every day both sides can take; a window against its own name is one day.
    Order,
    /// A plain result: its value, `none` too, where every combination of its arguments' first
    /// and last days gives the same.
    Agreed,
    /// A def's `at`: one read per law-date class.
    LawDate,
    /// A print: the stretch.
    Print,
    /// A `with` pin of the cell: the pinned day.
    Pin,
}

/// The readers that take a ranged date, by the word the text writes them with, each with how
/// it takes one: the one list (L §4.4's table). `+` and `-` are `date ± duration`, which is
/// `add_days`, `add_months` or `add_years` (L §4.2); a period's key is `month_of`'s rule
/// (`keys.rs`); a def's `at` is read in `rules.rs`, a print in `model/value.rs`, a pin in
/// `pins.rs`. A reader not here is the hole wherever it meets one ([`unlisted`], [`one_day`]):
/// `weekday`, `days_between`, `dcf` among them (C3).
const READERS: [(&str, Takes); 25] = [
    ("add_days", Takes::Ends),
    ("add_months", Takes::Ends),
    ("add_years", Takes::Ends),
    ("+", Takes::Ends),
    ("-", Takes::Ends),
    ("min", Takes::Ends),
    ("max", Takes::Ends),
    ("<", Takes::Order),
    ("<=", Takes::Order),
    (">", Takes::Order),
    (">=", Takes::Order),
    ("==", Takes::Order),
    ("!=", Takes::Order),
    ("argmin", Takes::Order),
    ("argmax", Takes::Order),
    ("year_of", Takes::Agreed),
    ("quarter_of", Takes::Agreed),
    ("month_of", Takes::Agreed),
    ("age", Takes::Agreed),
    ("age_months", Takes::Agreed),
    ("overlap_days", Takes::Agreed),
    ("a key", Takes::Agreed),
    ("at", Takes::LawDate),
    ("a print", Takes::Print),
    ("with", Takes::Pin),
];

/// How reader `reader` takes a ranged date, or none for a reader off the list (L §4.4).
pub(super) fn takes(reader: &str) -> Option<Takes> {
    READERS
        .iter()
        .find(|(name, _)| *name == reader)
        .map(|(_, how)| *how)
}

/// The hole of window `w` met where one value is needed, by `what`: `?"date within <w> not
/// precise enough for <what>" (<its read>)`, one question per unknown day where it rests on
/// two or more, each asked of the day's own name (refcore's `imprecise`, `imprecise_hole`).
/// L §4.4, §8.1 (R21 on #241).
pub(super) fn imprecise(w: &Window, what: &str) -> Hole {
    crate::hole::imprecise((&w.question_text(), what), w.figure(), (&w.carry, None))
}

/// The window of a value, if it is one.
pub(super) fn as_window(v: &Value) -> Option<&Window> {
    if let Value::Window(w) = v {
        return Some(w);
    }
    None
}

/// An end of a stretch: its first day or its last (L §4.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum End {
    /// The lowest: a window's first day, `min`'s pick.
    Low,
    /// The highest: a window's last day, `max`'s pick.
    High,
}

/// A window's end `end` as a day, any other value itself. L §4.4.
pub(super) fn window_end(v: &Value, end: End) -> Value {
    let Value::Window(w) = v else {
        return v.clone();
    };
    match end {
        End::Low => Value::Day(w.lo),
        End::High => Value::Day(w.hi),
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

/// The stretch from day `lo` to day `hi` a reader computed from window `from`: one day where
/// they agree, else a window with `from`'s text, figure and unknown days and no read, a value
/// computed (refcore's `lift`, its end). L §4.4.
pub(super) fn stretch((lo, hi): (Day, Day), from: &Window) -> Value {
    if lo == hi {
        return Value::Day(lo);
    }
    Value::Window(Box::new(Window {
        lo: lo.min(hi),
        hi: hi.max(lo),
        days: None,
        src: from.src.clone(),
        figure: from.figure.clone(),
        read: None,
        carry: from.carry.clone(),
    }))
}

/// Whether a stop is a refusal per value, which a listed reader may meet at some ends and not
/// at others (L §4.4, §8.4).
fn refused_per_value(stop: &Stop) -> bool {
    matches!(stop, Stop::Refused(r) if matches!(r.code(), Code::Run(_)))
}

/// A listed reader of date to date (`Takes::Ends`) of window `from`, from its results at the
/// window's first day and its last, `ends`: the stretch between them; refused at both, the
/// first's refusal; refused at one, the hole for `what`; any other stop as it came. L §4.4
/// ("Refused at every end, it is the first's refusal").
///
/// # Errors
///
/// The first end's refusal where both refuse; any other stop of either.
pub(super) fn by_ends(
    ends: (Result<Value, Stop>, Result<Value, Stop>),
    from: &Window,
    what: &str,
) -> Result<Value, Stop> {
    match ends {
        (Ok(Value::Day(lo)), Ok(Value::Day(hi))) => Ok(stretch((lo, hi), from)),
        (Ok(_), Ok(_)) => Err(invariant("a date-to-date reader that gives no day")),
        (Err(first), Err(last)) => {
            if refused_per_value(&first) && !refused_per_value(&last) {
                return Err(last);
            }
            Err(first)
        }
        (Err(e), Ok(_)) | (Ok(_), Err(e)) => {
            if !refused_per_value(&e) {
                return Err(e);
            }
            Ok(Value::Hole(imprecise(from, what)))
        }
    }
}

/// Every combination of the first and last days of the ranged dates among `args`, each other
/// argument as it is, the first argument's ends outermost: two tuples for one ranged
/// argument, four for two, whatever names they carry (L §4.4: "A reader takes each argument by
/// its own ends").
pub(super) fn corners(args: &[&Value]) -> Vec<Vec<Value>> {
    let mut tuples: Vec<Vec<Value>> = vec![Vec::new()];
    for v in args {
        let options = if let Value::Window(w) = v {
            vec![Value::Day(w.lo), Value::Day(w.hi)]
        } else {
            vec![(*v).clone()]
        };
        let mut next = Vec::new();
        for t in &tuples {
            for o in &options {
                next.push([t.as_slice(), core::slice::from_ref(o)].concat());
            }
        }
        tuples = next;
    }
    tuples
}

/// A listed reader of a plain result (`Takes::Agreed`) from `outs`, its results at each of
/// [`corners`]: the value every one gives, `none` too; refused at every one, the first's
/// refusal; refused at some, or two that differ, the hole for `what` naming each of
/// `windows`; any other stop as it came. L §4.4 ("its value, `none` too, where every
/// combination of its arguments' first and last days gives the same").
///
/// # Errors
///
/// The first refusal where every combination refuses; any other stop.
pub(super) fn agreed(
    outs: Vec<Result<Value, Stop>>,
    windows: &[&Window],
    what: &str,
) -> Result<Value, Stop> {
    let (mut value, mut first_refusal, mut differ) = (None::<Value>, None::<Stop>, false);
    for out in outs {
        match out {
            Ok(v) => match &value {
                None => value = Some(v),
                Some(held) => differ |= *held != v,
            },
            Err(e) if refused_per_value(&e) => {
                first_refusal.get_or_insert(e);
            }
            Err(other) => return Err(other),
        }
    }
    match (value, first_refusal) {
        (Some(v), None) if !differ => Ok(v),
        (None, Some(refusal)) => Err(refusal),
        (Some(_), _) => imprecise_all(windows, what)
            .map(Value::Hole)
            .ok_or_else(|| invariant("a reader's ends that differ with no window")),
        (None, None) => Err(invariant("a reader read at no combination of ends")),
    }
}

/// The hole of a read off the list (L §4.4: "Every other read is the hole"): `HOLE-IMPRECISE`
/// for `what`, naming each ranged date among `args` (a window of one day is that day); none
/// where no argument is ranged. With [`one_day`], the one place such a read is answered.
pub(super) fn unlisted(args: &[&Value], what: &str) -> Option<Hole> {
    let windows: Vec<&Window> = args.iter().filter_map(|v| ranged(v)).collect();
    imprecise_all(&windows, what)
}

/// The window of a ranged date: a window of more than one day; none for any other value, a
/// window of one day among them, which is that day (F3). L §4.4.
pub(super) fn ranged(v: &Value) -> Option<&Window> {
    as_window(v).filter(|w| !of_one_day(w))
}

/// Whether window `w` is of one day. L §4.4 ("A window of one day is that day").
fn of_one_day(w: &Window) -> bool {
    w.days.as_ref().is_none_or(|days| days.len() == 1) && w.lo == w.hi
}

/// `a op b` where a side is a window: `date ± duration`, a day moved by a number or a
/// duration, is listed, `add_days`'s rule (L §4.2: "date ± duration moves by that duration"),
/// the stretch from its result at the window's first day to its result at its last; any other
/// arithmetic on a ranged date is the hole for `arithmetic` (L §4.4, "No number is a range").
///
/// # Errors
///
/// The refusal both ends give; any other stop of the arithmetic.
pub(super) fn window_arith(
    m: &Reality,
    op: Op,
    (left, right): (&Value, &Value),
) -> Result<Value, Stop> {
    let (one_l, one_r) = (single(left), single(right));
    let (lhs, rhs) = (one_l.as_ref(), one_r.as_ref());
    if let (Value::Window(w), Value::Number(_)) = (lhs, rhs)
        && takes(op.text()) == Some(Takes::Ends)
    {
        let at = |d: Day| arith(m, op, &Value::Day(d), rhs);
        return by_ends((at(w.lo), at(w.hi)), w, "arithmetic");
    }
    match unlisted(&[lhs, rhs], "arithmetic") {
        Some(hole) => Ok(Value::Hole(hole)),
        // a window of one day is that day (F3)
        None => arith(m, op, lhs, rhs),
    }
}

/// [`imprecise`] of the first of `windows`, carrying the unknown days of every one, each once:
/// a hole a read makes of them names every window its operands carry (L §4.4 l.667, l.671;
/// P41, F8); none for no window.
fn imprecise_all(windows: &[&Window], what: &str) -> Option<Hole> {
    let [first, rest @ ..] = windows else {
        return None;
    };
    let mut from = (*first).clone();
    for w in rest {
        crate::hole::carry_union(&mut from.carry, &w.carry);
    }
    Some(imprecise(&from, what))
}

/// The comparison `a op b` where a side is a window, a listed reader (`Takes::Order`): decided
/// when it holds for every day both sides can take, two reads alike of one item decided as one
/// day, else `HOLE-IMPRECISE` naming the comparison as written, `text` (refcore's `iv_compare`;
/// its identity is the input row, the spec's the read, #115). L §4.4.
pub(super) fn window_compare(
    m: &Reality,
    op: CmpOp,
    (left, right): (&Value, &Value),
    text: &str,
) -> Result<Value, Stop> {
    if one_read(left, right) {
        return Ok(Value::Bool(matches!(op, CmpOp::Eq | CmpOp::Le | CmpOp::Ge)));
    }
    let (one_a, one_b) = (single(left), single(right));
    let (a, b) = (one_a.as_ref(), one_b.as_ref());
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
            let below = order(&ahi, &blo)?;
            if below == Ordering::Less || (!strict && below == Ordering::Equal) {
                return Ok(Value::Bool(true));
            }
            let above = order(&alo, &bhi)?;
            if above == Ordering::Greater || (strict && above == Ordering::Equal) {
                return Ok(Value::Bool(false));
            }
        }
        CmpOp::Gt | CmpOp::Ge => {
            let strict = op == CmpOp::Gt;
            let above = order(&alo, &bhi)?;
            if above == Ordering::Greater || (!strict && above == Ordering::Equal) {
                return Ok(Value::Bool(true));
            }
            let below = order(&ahi, &blo)?;
            if below == Ordering::Less || (strict && below == Ordering::Equal) {
                return Ok(Value::Bool(false));
            }
        }
        CmpOp::Eq | CmpOp::Ne => {
            let apart =
                order(&ahi, &blo)? == Ordering::Less || order(&bhi, &alo)? == Ordering::Less;
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
    Ok(Value::Hole(imprecise(w, text)))
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
/// when the whole stretch lies below (above) the other, equal for two reads alike of one
/// item; none where the stretches meet, an order an aggregate can't take (L §4.4; refcore's
/// `iv_compare`, its identity the input row's, #115).
///
/// # Errors
///
/// The refusals of comparing the values or their ends.
pub(super) fn decided_order(
    m: &Reality,
    left: &Value,
    right: &Value,
) -> Result<Option<Ordering>, Stop> {
    if one_read(left, right) {
        return Ok(Some(Ordering::Equal));
    }
    let (one_a, one_b) = (single(left), single(right));
    let (a, b) = (one_a.as_ref(), one_b.as_ref());
    if as_window(a).is_none() && as_window(b).is_none() {
        return compare(m, a, b, Test::Order).map(Some);
    }
    let ((alo, ahi), (blo, bhi)) = (ends(a), ends(b));
    if compare(m, &ahi, &blo, Test::Order)? == Ordering::Less {
        return Ok(Some(Ordering::Less));
    }
    if compare(m, &alo, &bhi, Test::Order)? == Ordering::Greater {
        return Ok(Some(Ordering::Greater));
    }
    Ok(None)
}

/// A window of one day is that day; any other value is itself (refcore's `one_day`; tally
/// 2.1.7 R3, F3). L §4.4.
pub(super) fn single(v: &Value) -> Cow<'_, Value> {
    if let Value::Window(w) = v
        && of_one_day(w)
    {
        return Cow::Owned(Value::Day(w.lo));
    }
    Cow::Borrowed(v)
}

/// A value's stretch: a window's first and last days, any other value twice. L §4.4.
fn ends(v: &Value) -> (Value, Value) {
    if let Value::Window(w) = v {
        return (Value::Day(w.lo), Value::Day(w.hi));
    }
    (v.clone(), v.clone())
}

/// Whether `one_of` window `w` holds the value `v` among its days. L §4.4.
fn one_of_holds(w: &Window, v: &Value) -> bool {
    match (&w.days, v) {
        (Some(days), Value::Day(d)) => days.contains(d),
        _ => false,
    }
}

/// One day where one is needed, the one reader every such site uses (`what` names the reader
/// that needs it): a day (the end of time is none), a day period's day; a window, a read off
/// the list (L §4.4: "Every other read is the hole"), is its `HOLE-IMPRECISE` for `what`;
/// `none` and `never` refused (refcore's `date_of`). L §4.2, §4.4, §7.6 item 1.
///
/// # Errors
///
/// `RUN-NONE`, `RUN-LIMIT`; `LOAD-UNIT` for a value that is no date.
pub(crate) fn one_day(m: &Reality, value: &Value, what: &str) -> Result<Result<Day, Hole>, Stop> {
    let one = single(value);
    let v = one.as_ref();
    if let Value::Day(d) = v {
        return Ok(Ok(super::state::a_day(*d)?));
    }
    if let Value::Window(w) = v {
        return Ok(Err(imprecise(w, what)));
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

/// The days window `w` can take, which a def's `at` reads a law-date class at a time: a
/// `one_of`'s, else every day from its first to its last. L §4.4, §4.3.
///
/// # Errors
///
/// `RUN-LIMIT` past the calendar's edge.
pub(crate) fn window_days(w: &Window) -> Result<Vec<Day>, Stop> {
    if let Some(days) = &w.days {
        return Ok(days.clone());
    }
    let mut out = vec![w.lo];
    let mut day = w.lo;
    while day < w.hi {
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
    let (lo, hi) = (w.lo, w.hi);
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
            lo: first,
            hi: last,
            days: None,
            src: Some(written.clone()),
            figure: None,
            read: None,
            carry: Vec::new(),
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
    use super::{Takes, Value, Window, ends, one_of_holds, takes, unlisted};
    use crate::time::Day;

    fn day(y: u16, m: u8, d: u8) -> Day {
        Day::from_ymd(y, m, d).expect("a real day")
    }

    #[test]
    fn the_list_names_each_reader_with_its_class_and_no_other() {
        // L §4.4's table: listed readers by class; weekday, days_between, dcf, round, abs,
        // pow, a band's argument off it (C3, C-5)
        for (name, how) in [
            ("add_years", Takes::Ends),
            ("-", Takes::Ends),
            ("max", Takes::Ends),
            ("<=", Takes::Order),
            ("argmin", Takes::Order),
            ("quarter_of", Takes::Agreed),
            ("age_months", Takes::Agreed),
            ("overlap_days", Takes::Agreed),
        ] {
            assert_eq!(takes(name), Some(how), "{name}");
        }
        for name in [
            "weekday",
            "days_between",
            "dcf",
            "start_of",
            "round",
            "abs",
            "pow",
            "*",
            "is_business",
            "a band argument",
        ] {
            assert_eq!(takes(name), None, "{name} is off the list");
        }
        // the default: the hole naming each ranged argument's day, a one-day window none
        let ranged = Value::Window(Box::new(Window {
            lo: day(2026, 6, 1),
            hi: day(2026, 6, 5),
            days: None,
            src: Some("window(2026-06-01, 2026-06-05)".to_owned()),
            figure: Some("w".to_owned()),
            read: None,
            carry: Vec::new(),
        }));
        let hole = unlisted(&[&ranged, &Value::int(2)], "weekday").expect("the hole");
        assert_eq!(
            hole.text(),
            "?\"date within window(2026-06-01, 2026-06-05) not precise enough for weekday\" (w)",
            "the reader's word"
        );
        let one = Value::Window(Box::new(Window {
            lo: day(2026, 6, 1),
            hi: day(2026, 6, 1),
            days: None,
            src: None,
            figure: None,
            read: None,
            carry: Vec::new(),
        }));
        assert!(
            unlisted(&[&one], "weekday").is_none(),
            "one day is that day"
        );
    }

    #[test]
    fn a_one_of_holds_only_its_days() {
        let w = Window {
            lo: day(2026, 6, 1),
            hi: day(2026, 6, 5),
            days: Some(vec![day(2026, 6, 1), day(2026, 6, 5)]),
            src: None,
            figure: None,
            read: None,
            carry: Vec::new(),
        };
        assert!(
            one_of_holds(&w, &Value::Day(day(2026, 6, 5))),
            "a day of it"
        );
        assert!(!one_of_holds(&w, &Value::Day(day(2026, 6, 3))), "between");
        let plain = Value::Day(day(2026, 6, 3));
        let (lo, hi) = ends(&plain);
        assert_eq!(
            (lo, hi),
            (plain.clone(), plain),
            "a value is its own stretch"
        );
    }
}
