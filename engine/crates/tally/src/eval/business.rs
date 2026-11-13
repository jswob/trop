//! The calendar built-ins: business days, their counts, a period's n-th last and a scheduled day's roll, over every day an added window takes.
//! Spec: L §4.2 (`is_business`, `add_business` and `RUN-ZERO`, `last_business`, `schedule`
//! and its rolls, ISDA 2006 section 4.12; the calendar read at the day asked; an added day
//! that is a hole, `HOLE-CALENDAR`, or a window, every day known but one), §4.4 (a
//! business-day function over a calendar whose `added` holds a window is evaluated for each
//! day the window can take, its value the interval of those results; for `is_business`
//! `HOLE-IMPRECISE` when they differ), §7.6 item 5 (a calendar with no business day), §8.4.
//! Twin: engine/tools/refcore/evaluate.py (`cal_at`, `business`, `over_added`,
//! `c_is_business`, `c_add_business`, `c_last_business`, `c_schedule`, `rolled`).
//! Invariants: every business-day function runs as passes, one per combination of the days
//! the added windows it meets can take, nested in the order met; a pass that meets a window
//! no choice names is run again once per day of it; the passes' state is restored after them.

use std::collections::{BTreeMap, VecDeque};

use super::arith::whole_of;
use super::builtin::read_parts;
use super::dates::period_arg;
use super::expr::hole_of;
use super::state::{Ctx, Engine, Env, invariant, not_typed, past_days, refuse, refuse_load};
use super::windows::{imprecise, interval, one_day, single};
use crate::base::{Load, Run};
use crate::hole::{Hole, spread};
use crate::model::{Code, Reality, Value, Window};
use crate::num::Num;
use crate::read::Asked;
use crate::syntax::{Call, Expr, Stop};
use crate::time::{Added, Day, Period};

/// An added window of a calendar, by the calendar's input, the entry's place in its `added`
/// and the window as written: two entries are two unknown days (tally 2.1.7 R3, F5).
type WindowId = (String, usize, String);

/// One pass of a business-day function: the day each added window takes in it, and the
/// windows it met that no choice names yet, with the days each can take (refcore's
/// `cal_choice`, `cal_met`). L §4.2, §4.4.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CalendarPass {
    /// The day each window takes in this pass.
    chosen: BTreeMap<WindowId, Day>,
    /// The windows met that no choice names, with their days in order.
    met: BTreeMap<WindowId, Vec<Day>>,
}

/// A scheduled day's roll to a business day (L §4.2 `schedule`; ISDA 2006 section 4.12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Roll {
    /// To the next business day.
    Following,
    /// To the one before.
    Preceding,
    /// To the next, unless it is in another month, then the one before.
    ModifiedFollowing,
}

/// A business-day function, as one pass evaluates it (refcore's closures under
/// `over_added`). L §4.2.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Business {
    /// `is_business(d, cal)`.
    Is(Day),
    /// `add_business(d, n, cal)`: the business days from a day, by steps of 1 or -1, so many
    /// of them (none past 64 bits).
    Add(Day, i64, Option<u64>),
    /// `last_business(p, n, cal)`: so many of them (none past 64 bits), and n as written.
    Last(Period, Option<u64>, String),
    /// `schedule`'s day, moved by its roll.
    Rolled(Day, Roll),
}

/// A business-day function of a day argument before its day is given: `is_business`, or
/// `add_business` with its count (refcore's closures under `per_day`). L §4.2, §4.4.
#[derive(Debug, Clone)]
enum Daily {
    /// `is_business(d, cal)`.
    Is,
    /// `add_business(d, n, cal)` with its n.
    Add(Num),
}

impl Daily {
    /// The function at day `day`: `add_business`'s n of 0 is `RUN-ZERO` there (refcore's
    /// `add_business_at`). L §4.2, §8.4.
    fn on(&self, day: Day, cal: &Expr) -> Result<Business, Stop> {
        let k = match self {
            Daily::Is => return Ok(Business::Is(day)),
            Daily::Add(k) => k,
        };
        if k.is_zero() {
            let text = crate::load::expr_text(cal);
            return Err(refuse(
                Run::Zero,
                format!("add_business({}, 0, {text})", day.text()),
            ));
        }
        let step = if k.is_negative() { -1 } else { 1 };
        // a count past 64 bits outnumbers the calendar's days: the walk reads the calendar
        // day by day until it meets 0001-01-01 or 9999-12-31 (L §7.6 item 1)
        let count = k.to_i64().map(i64::unsigned_abs);
        Ok(Business::Add(day, step, count))
    }
}

/// The days a window can take: a `one_of`'s, else every day from its first to its last.
/// L §4.4.
fn window_days(w: &Window) -> Result<Vec<Day>, Stop> {
    if let Some(days) = &w.days {
        return Ok(days.clone());
    }
    let (Value::Day(first), Value::Day(last)) = (&w.lo, &w.hi) else {
        return Err(not_typed());
    };
    let mut out = vec![*first];
    let mut day = *first;
    while day < *last {
        day = day.next().ok_or_else(|| past_days(1))?;
        out.push(day);
    }
    Ok(out)
}

/// A value's day, if it is one.
fn day_of(v: &Value) -> Option<Day> {
    if let Value::Day(d) = v {
        return Some(*d);
    }
    None
}

impl Engine {
    /// `is_business(d, cal)` (refcore's `c_is_business`). L §4.2, §4.4.
    pub(super) fn is_business(
        &mut self,
        m: &Reality,
        call: &Call,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let cal = call.args.last().ok_or_else(arity)?;
        let d = self.ev(m, call.args.first().ok_or_else(arity)?, env, ctx)?;
        if let Value::Hole(_) = d {
            return Ok(d);
        }
        if let Value::Window(w) = single(&d) {
            // a window argument: day by day, is_business isn't monotone in it (F4)
            return self.per_day(m, w.as_ref(), (cal, env, ctx), (&Daily::Is, call));
        }
        let day = match one_day(m, &d, "is_business")? {
            Ok(day) => day,
            Err(hole) => return Ok(Value::Hole(hole)),
        };
        self.over_added(m, (cal, env, ctx), (&Business::Is(day), call))
    }

    /// Business-day function `f` of a window argument `w`, evaluated for each day `w` can
    /// take, since it isn't monotone in it: one value where they agree, else the interval of
    /// the days, or for `is_business` `HOLE-IMPRECISE` (refcore's `per_day`; tally 2.1.7 R3,
    /// F4). L §4.4.
    fn per_day(
        &mut self,
        m: &Reality,
        w: &Window,
        (cal, env, ctx): (&Expr, &Env, Ctx),
        (f, call): (&Daily, &Call),
    ) -> Result<Value, Stop> {
        let mut outs: Vec<Value> = Vec::new();
        let mut holes: Vec<Hole> = Vec::new();
        for day in window_days(w)? {
            let v = self.over_added(m, (cal, env, ctx), (&f.on(day, cal)?, call))?;
            if let Value::Hole(h) = &v {
                holes.push(h.clone());
            } else {
                outs.push(v);
            }
        }
        if !holes.is_empty() {
            let operands: Vec<Option<&Hole>> = holes.iter().map(Some).collect();
            return hole_of(&operands);
        }
        let [first, rest @ ..] = outs.as_slice() else {
            return Err(invariant("a window with no day"));
        };
        if rest.iter().all(|v| v == first) {
            return Ok(first.clone());
        }
        if let Value::Bool(_) = first {
            let call_text = crate::load::expr_text(&Expr::Call(Box::new(call.clone())));
            return Ok(Value::Hole(imprecise(m, w, &call_text)?));
        }
        let (Some(lo), Some(hi)) = (
            outs.iter().filter_map(day_of).min(),
            outs.iter().filter_map(day_of).max(),
        ) else {
            return Err(invariant("a business-day function's day that is no day"));
        };
        interval(m, (Value::Day(lo), Value::Day(hi)), w)
    }

    /// `add_business(d, n, cal)`: n business days after d, before it for a negative n,
    /// counted from the day after (refcore's `c_add_business`). L §4.2, §4.4.
    pub(super) fn add_business(
        &mut self,
        m: &Reality,
        call: &Call,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let cal = call.args.last().ok_or_else(arity)?;
        let d = self.ev(m, call.args.first().ok_or_else(arity)?, env, ctx)?;
        let k = self.ev(m, call.args.get(1).ok_or_else(arity)?, env, ctx)?;
        if d.as_hole().is_some() || k.as_hole().is_some() {
            return hole_of(&[d.as_hole(), k.as_hole()]);
        }
        if let Value::Window(w) = single(&d) {
            // a window argument: day by day, add_business isn't monotone in it (F4)
            let k = whole_of(m, &k)?;
            return self.per_day(m, w.as_ref(), (cal, env, ctx), (&Daily::Add(k), call));
        }
        let day = match one_day(m, &d, "add_business")? {
            Ok(day) => day,
            Err(hole) => return Ok(Value::Hole(hole)),
        };
        let k = whole_of(m, &k)?;
        let f = Daily::Add(k).on(day, cal)?;
        self.over_added(m, (cal, env, ctx), (&f, call))
    }

    /// `last_business(p, n, cal)`: the n-th last business day of period p; n below 1 and a
    /// period with fewer are `RUN-DOMAIN` (refcore's `c_last_business`). L §4.2, §4.4, §8.4.
    pub(super) fn last_business(
        &mut self,
        m: &Reality,
        call: &Call,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let [of, nth, cal] = call.args.as_slice() else {
            return Err(arity());
        };
        let (of, nth) = (self.ev(m, of, env, ctx)?, self.ev(m, nth, env, ctx)?);
        if of.as_hole().is_some() || nth.as_hole().is_some() {
            return hole_of(&[of.as_hole(), nth.as_hole()]);
        }
        let period = period_arg(m, (call.args.first(), &of), "last_business")?;
        let count = whole_of(m, &nth)?;
        let text = count.integer_text().ok_or_else(|| invariant("a count"))?;
        if count.is_negative() || count.is_zero() {
            return Err(refuse(
                Run::Domain,
                format!("last_business with n = {text} below 1"),
            ));
        }
        let last = Business::Last(period, count.to_i64().map(i64::unsigned_abs), text);
        self.over_added(m, (cal, env, ctx), (&last, call))
    }

    /// `schedule(p, e, roll, cal)`: day `e` of month `p`, none past its last day, moved by
    /// its roll to a business day of `cal` (refcore's `c_schedule`, `rolled`; ISDA 2006
    /// section 4.12). L §4.2, §4.4.
    pub(super) fn schedule(
        &mut self,
        m: &Reality,
        call: &Call,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let [month, dom, roll_arg, cal] = call.args.as_slice() else {
            return Err(arity());
        };
        let (month, dom, word) = (
            self.ev(m, month, env, ctx)?,
            self.ev(m, dom, env, ctx)?,
            self.ev(m, roll_arg, env, ctx)?,
        );
        if let Some(h) = spread(&[month.as_hole(), dom.as_hole(), word.as_hole()]) {
            return Ok(Value::Hole(h));
        }
        let period = period_arg(m, (call.args.first(), &month), "schedule")?;
        let start = period.start();
        // a day number past the month's, 64 bits or not, is none
        let Some(day) = whole_of(m, &dom)?
            .to_i64()
            .and_then(|n| u8::try_from(n).ok())
            .and_then(|n| Day::from_ymd(start.year(), start.month(), n))
            .filter(|d| period.contains(*d))
        else {
            return Ok(Value::None);
        };
        let (Value::Word(word) | Value::Code(Code::Word(word))) = &word else {
            return Err(not_typed());
        };
        let roll = match word.as_str() {
            "unadjusted" => return Ok(Value::Day(day)),
            "following" => Roll::Following,
            "preceding" => Roll::Preceding,
            "modified_following" => Roll::ModifiedFollowing,
            _ => {
                let message = format!("schedule: {word} is not a roll");
                return Err(refuse_load(Load::Unit, message));
            }
        };
        self.over_added(m, (cal, env, ctx), (&Business::Rolled(day, roll), call))
    }

    /// Business-day function `f` evaluated at each day the added windows of its calendar can
    /// take: one value when the passes agree; days that differ give their window; an
    /// `is_business` that differs is `HOLE-IMPRECISE`, asked of the calendar (refcore's
    /// `over_added`). L §4.2, §4.4.
    fn over_added(
        &mut self,
        m: &Reality,
        (cal, env, ctx): (&Expr, &Env, Ctx),
        (f, call): (&Business, &Call),
    ) -> Result<Value, Stop> {
        let outer = self.calendar_pass.take();
        let start = match &outer {
            Some(pass) => pass.chosen.clone(),
            None => BTreeMap::new(),
        };
        let found = self.passes(m, (cal, env, ctx), f, start);
        self.calendar_pass = outer;
        let (out, first_met) = found?;
        let holes: Vec<Option<&Hole>> = out.iter().map(Value::as_hole).collect();
        if let Some(h) = spread(&holes) {
            return Ok(Value::Hole(h));
        }
        let [first, rest @ ..] = out.as_slice() else {
            return Err(invariant("a business-day function with no pass"));
        };
        if rest.iter().all(|v| v == first) {
            return Ok(first.clone());
        }
        let (cal_id, _, written) = first_met.ok_or_else(|| invariant("passes with no window"))?;
        if let Value::Bool(_) = first {
            let call_text = crate::load::expr_text(&Expr::Call(Box::new(call.clone())));
            let text = format!("date within {written} not precise enough for {call_text}");
            return Ok(Value::Hole(Hole::asked(&text, &cal_id)));
        }
        let mut days = Vec::with_capacity(out.len());
        for v in &out {
            let Value::Day(d) = v else {
                return Err(invariant("a business-day function's day that is no day"));
            };
            days.push(*d);
        }
        let (Some(lo), Some(hi)) = (days.iter().min(), days.iter().max()) else {
            return Err(invariant("a business-day function with no pass"));
        };
        Ok(Value::Window(Box::new(Window {
            lo: Value::Day(*lo),
            hi: Value::Day(*hi),
            days: None,
            src: None,
            figure: None,
            read: None,
        })))
    }

    /// The passes of `f`, from the choices `start` names: each pass that meets a window no
    /// choice names is run again once per day of the first such window; the values of the
    /// others, in order, and the first window met (refcore's `over_added`, its loop).
    /// L §4.2, §4.4.
    fn passes(
        &mut self,
        m: &Reality,
        (cal, env, ctx): (&Expr, &Env, Ctx),
        f: &Business,
        start: BTreeMap<WindowId, Day>,
    ) -> Result<(Vec<Value>, Option<WindowId>), Stop> {
        let mut todo = VecDeque::from([start]);
        let mut out = Vec::new();
        let mut first_met: Option<WindowId> = None;
        while let Some(chosen) = todo.pop_front() {
            self.calendar_pass = Some(CalendarPass {
                chosen: chosen.clone(),
                met: BTreeMap::new(),
            });
            let v = self.pass(m, (cal, env, ctx), f)?;
            let met = self.calendar_pass.take().map(|p| p.met);
            if let Some((id, days)) = met.and_then(|met| met.into_iter().next()) {
                if first_met.is_none() {
                    first_met = Some(id.clone());
                }
                for d in days {
                    let mut next = chosen.clone();
                    next.insert(id.clone(), d);
                    todo.push_back(next);
                }
                continue;
            }
            out.push(v);
        }
        Ok((out, first_met))
    }

    /// One pass of business-day function `f` (refcore's closures under `over_added`).
    /// L §4.2.
    fn pass(
        &mut self,
        m: &Reality,
        (cal, env, ctx): (&Expr, &Env, Ctx),
        f: &Business,
    ) -> Result<Value, Stop> {
        match f {
            Business::Is(day) => self.business(m, (cal, env, ctx), (*day, false)),
            Business::Add(day, step, count) => {
                self.count_business(m, (cal, env, ctx), (*day, *step, *count))
            }
            Business::Last(period, count, text) => {
                self.last_of(m, (cal, env, ctx), (*period, *count, text))
            }
            Business::Rolled(day, roll) => self.rolled(m, (cal, env, ctx), (*day, *roll)),
        }
    }

    /// The `count`-th business day after `from` by steps of `step`, or the hole met on the
    /// way (refcore's `c_add_business`, its `count`). L §4.2.
    fn count_business(
        &mut self,
        m: &Reality,
        (cal, env, ctx): (&Expr, &Env, Ctx),
        (from, step, count): (Day, i64, Option<u64>),
    ) -> Result<Value, Stop> {
        let mut left = count;
        let mut current = from;
        while left != Some(0) {
            let open = self.next_business(m, (cal, env, ctx), (current, step))?;
            let Value::Day(found) = open else {
                return Ok(open);
            };
            current = found;
            if let Some(n) = left.as_mut() {
                *n = n
                    .checked_sub(1)
                    .ok_or_else(|| invariant("a business-day count"))?;
            }
        }
        Ok(Value::Day(current))
    }

    /// The `count`-th last business day of `period`, counting back from its last day, or the
    /// hole met on the way; a period with fewer is `RUN-DOMAIN`, naming n as written,
    /// `text` (refcore's `c_last_business`, its `count`). L §4.2, §8.4.
    fn last_of(
        &mut self,
        m: &Reality,
        (cal, env, ctx): (&Expr, &Env, Ctx),
        (period, count, text): (Period, Option<u64>, &str),
    ) -> Result<Value, Stop> {
        let mut left = count;
        let mut current = period.end().previous();
        while let Some(day) = current.filter(|d| period.contains(*d)) {
            let open = self.business(m, (cal, env, ctx), (day, false))?;
            let Value::Bool(is_open) = open else {
                return Ok(open);
            };
            if is_open {
                if let Some(n) = left.as_mut() {
                    *n = n
                        .checked_sub(1)
                        .ok_or_else(|| invariant("a business-day count"))?;
                }
                if left == Some(0) {
                    return Ok(Value::Day(day));
                }
            }
            current = day.previous();
        }
        Err(refuse(
            Run::Domain,
            format!(
                "last_business({}, {text}): the period has fewer business days",
                period.label()
            ),
        ))
    }

    /// `schedule`'s day moved by its roll to a business day, or the hole met on the way
    /// (refcore's `rolled`; ISDA 2006 section 4.12). L §4.2.
    fn rolled(
        &mut self,
        m: &Reality,
        (cal, env, ctx): (&Expr, &Env, Ctx),
        (day, roll): (Day, Roll),
    ) -> Result<Value, Stop> {
        let open = self.business(m, (cal, env, ctx), (day, true))?;
        if open != Value::Bool(false) {
            return Ok(if open == Value::Bool(true) {
                Value::Day(day)
            } else {
                open
            });
        }
        let step = if roll == Roll::Preceding { -1 } else { 1 };
        let found = self.next_business(m, (cal, env, ctx), (day, step))?;
        if let Value::Day(f) = found
            && roll == Roll::ModifiedFollowing
            && f.month() != day.month()
        {
            return self.next_business(m, (cal, env, ctx), (day, -1));
        }
        Ok(found)
    }

    /// The first business day after `from` (before it, `step` -1), never `from` itself, or
    /// the hole of the calendar's gap met on the way: a calendar with a business day has one
    /// within 400 years of every day (refcore's `walk` of `rolled`, `count`). L §4.2, §7.6
    /// item 5.
    fn next_business(
        &mut self,
        m: &Reality,
        (cal, env, ctx): (&Expr, &Env, Ctx),
        (from, step): (Day, i64),
    ) -> Result<Value, Stop> {
        let mut current = from;
        loop {
            current = current.add_days(step).ok_or_else(|| past_days(step))?;
            let open = self.business(m, (cal, env, ctx), (current, true))?;
            let Value::Bool(is_open) = open else {
                return Ok(open);
            };
            if is_open {
                return Ok(Value::Day(current));
            }
        }
    }

    /// Whether `day` is a business day of the calendar `cal` reads, in force on that day:
    /// off by its rules or an added day; an added hole is `HOLE-CALENDAR`; an added window
    /// holding the day is off when this pass's choice for it, or its first day where the pass
    /// meets it newly, is the day. Where the search for one asks (`seek`), a calendar with
    /// none is `RUN-DOMAIN` (refcore's `cal_at`, `business`). L §4.2, §4.4, §7.6 item 5.
    fn business(
        &mut self,
        m: &Reality,
        (cal, env, ctx): (&Expr, &Env, Ctx),
        (day, seek): (Day, bool),
    ) -> Result<Value, Stop> {
        let Some((name, written)) = read_parts(cal) else {
            return Err(Stop::NotBuilt("a calendar that isn't an input's read"));
        };
        let input = m
            .inputs
            .get(name)
            .ok_or(Stop::NotBuilt("a calendar that isn't an input's read"))?;
        let given = self.given_keys(m, written, env, ctx)?;
        let keys = match self.make_keys(m, name, &input.binds, given, ctx)? {
            Err(hole) => return Ok(Value::Hole(hole)),
            Ok(made) => made,
        };
        let value = self.read_input_at(m, input, &keys, ctx.at_law(Some(day)), Asked::Calendar)?;
        let Value::Calendar(c) = &value else {
            if let Value::Hole(_) = value {
                return Ok(value);
            }
            if value == Value::None {
                return Err(refuse(Run::None, "the calendar is none here".to_owned()));
            }
            return Err(invariant(
                "a calendar argument that isn't a calendar: load's types refuse it first (LOAD-UNIT)",
            ));
        };
        if seek && !c.has_business_day(day) {
            return Err(refuse(
                Run::Domain,
                format!("calendar {name} has no business day"),
            ));
        }
        if c.off_by_rule(day) {
            return Ok(Value::Bool(false));
        }
        let mut unknown: Vec<Hole> = Vec::new();
        for (entry, added) in c.added().iter().enumerate() {
            if *added == Added::Day(day) {
                return Ok(Value::Bool(false));
            }
            if let Added::Hole(question) = added {
                unknown.push(Hole::asked(question, name));
            }
            if added.may_be(day) && self.window_takes((name, entry), added, day)? {
                return Ok(Value::Bool(false));
            }
        }
        if unknown.is_empty() {
            return Ok(Value::Bool(true));
        }
        let holes: Vec<Option<&Hole>> = unknown.iter().map(Some).collect();
        hole_of(&holes)
    }

    /// Whether calendar `name`'s added window `added`, which may be `day`, takes it in this
    /// pass: the pass's choice for it, or its first day where the pass meets it newly, which
    /// the pass records (refcore's `business`, its `cal_choice`). L §4.2, §4.4.
    fn window_takes(
        &mut self,
        (name, entry): (&str, usize),
        added: &Added,
        day: Day,
    ) -> Result<bool, Stop> {
        let pass = self
            .calendar_pass
            .as_mut()
            .ok_or_else(|| invariant("a calendar's added window read outside a pass"))?;
        let id = (name.to_owned(), entry, added.text());
        if let Some(chosen) = pass.chosen.get(&id) {
            return Ok(*chosen == day);
        }
        let days = added
            .days()
            .ok_or_else(|| invariant("a window's days past the calendar"))?;
        let first = days.first().copied();
        pass.met.entry(id).or_insert(days);
        Ok(first == Some(day))
    }
}

/// A call with the wrong number of arguments: load refuses it, so here it is an invariant.
fn arity() -> Stop {
    invariant("a built-in called with the wrong number of arguments")
}
