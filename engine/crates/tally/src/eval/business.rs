//! The calendar built-ins: business days, their counts, a period's n-th last and a scheduled day's roll.
//! Spec: L §4.2 (`is_business`, `add_business` and `RUN-ZERO`, `last_business`, `schedule`
//! and its rolls, ISDA 2006 section 4.12; the calendar read at the day asked; an added day
//! that is a hole, `HOLE-CALENDAR` where it decides a result), §4.4 (a business-day function
//! is off the readers' list, so a ranged date argument is the hole), §7.6 items 1 and 5 (the
//! calendar's edge; a calendar with no business day), §8 head (R1), §8.4; E §3.2 (law-date
//! classes).
//! Twin: engine/tools/refcore/evaluate.py (`cal_at`, `business`, `c_is_business`,
//! `c_add_business`, `c_last_business`, `c_schedule`, `rolled`).
//! Invariants: every business-day function reads its calendar once per day it asks, each
//! added day a day or a hole. An `add_business` whose count outnumbers the days to the edge
//! reads the calendar once per law-date class of its rows in day order, where those classes
//! stand for its days, and refuses with the first read's refusal, else `RUN-LIMIT`, as
//! refcore's walk day by day does; a hole on the way is left to that walk.

use std::collections::BTreeSet;

use super::arith::whole_of;
use super::builtin::read_parts;
use super::dates::period_arg;
use super::expr::hole_of;
use super::state::{Ctx, Engine, Env, given, invariant, not_typed, past_days, refuse, refuse_load};
use super::windows::{one_day, single, unlisted};
use crate::base::{Load, Run};
use crate::hole::{Hole, spread};
use crate::model::{Code, KeyVal, Reality, Value};
use crate::num::Num;
use crate::read::{Asked, row_span};
use crate::syntax::{Call, Expr, Postfix, Stop, Value as Written};
use crate::time::{Added, Day, Period};

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

/// A business-day function, its arguments read (refcore's `c_is_business`, `c_add_business`,
/// `c_last_business`, `c_schedule`). L §4.2.
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

/// `add_business(day, k, cal)` as [`Engine::apply`] evaluates it: `k` of 0 is `RUN-ZERO`
/// (refcore's `add_business_at`). L §4.2, §8.4.
///
/// # Errors
///
/// `RUN-ZERO` for a count of 0.
fn business_days(day: Day, k: &Num, cal: &Expr) -> Result<Business, Stop> {
    if k.is_zero() {
        let text = crate::load::expr_text(cal);
        return Err(refuse(
            Run::Zero,
            format!("add_business({}, 0, {text})", day.text()),
        ));
    }
    let step = if k.is_negative() { -1 } else { 1 };
    // a count past 64 bits outnumbers the calendar's days: the walk reads the calendar day by
    // day until it meets 0001-01-01 or 9999-12-31 (L §7.6 item 1)
    let count = k.to_i64().map(i64::unsigned_abs);
    Ok(Business::Add(day, step, count))
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
        // off the list: a ranged date is the hole (L §4.4)
        let day = match one_day(m, &d, "is_business")? {
            Ok(day) => day,
            Err(hole) => return Ok(Value::Hole(hole)),
        };
        self.apply(m, (cal, env, ctx), &Business::Is(day))
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
        // refusals outrank holes (L §8; pick P12): each argument in written order, `none`
        // refused, a known one's own check run, a hole's or a window's waiting
        given(&d, "add_business")?;
        count_given(m, &k)?;
        if d.as_hole().is_some() || k.as_hole().is_some() {
            return hole_of(&[d.as_hole(), k.as_hole()]);
        }
        // off the list: a ranged date is the hole (L §4.4)
        let day = match one_day(m, &d, "add_business")? {
            Ok(day) => day,
            Err(hole) => return Ok(Value::Hole(hole)),
        };
        if let Some(hole) = unlisted(&[&k], "add_business") {
            return Ok(Value::Hole(hole));
        }
        let f = business_days(day, &whole_of(m, &k)?, cal)?;
        self.apply(m, (cal, env, ctx), &f)
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
        // refusals outrank holes (L §8; pick P12): each argument in written order, `none`
        // refused, a known one's own check run, a hole's or a window's waiting
        if !waits(&of) {
            period_arg(m, (call.args.first(), &single(&of)), "last_business")?;
        }
        if let Some(count) = count_given(m, &nth)?
            && (count.is_negative() || count.is_zero())
        {
            let text = count.integer_text().ok_or_else(|| invariant("a count"))?;
            let message = format!("last_business with n = {text} below 1");
            return Err(refuse(Run::Domain, message));
        }
        if of.as_hole().is_some() || nth.as_hole().is_some() {
            return hole_of(&[of.as_hole(), nth.as_hole()]);
        }
        // off the list: a ranged date is the hole (L §4.4)
        if let Some(hole) = unlisted(&[&of, &nth], "last_business") {
            return Ok(Value::Hole(hole));
        }
        let period = period_arg(m, (call.args.first(), &single(&of)), "last_business")?;
        let count = whole_of(m, &nth)?;
        let text = count.integer_text().ok_or_else(|| invariant("a count"))?;
        if count.is_negative() || count.is_zero() {
            return Err(refuse(
                Run::Domain,
                format!("last_business with n = {text} below 1"),
            ));
        }
        let last = Business::Last(period, count.to_i64().map(i64::unsigned_abs), text);
        self.apply(m, (cal, env, ctx), &last)
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
        // refusals outrank holes (L §8; pick P12): each argument in written order, `none`
        // refused, a known one's own check run, a hole's or a window's waiting
        if !waits(&month) {
            period_arg(m, (call.args.first(), &single(&month)), "schedule")?;
        }
        count_given(m, &dom)?;
        given(&word, "schedule")?;
        if let Some(h) = spread(&[month.as_hole(), dom.as_hole(), word.as_hole()]) {
            return Ok(Value::Hole(h));
        }
        // off the list: a ranged date is the hole (L §4.4)
        if let Some(hole) = unlisted(&[&month, &dom], "schedule") {
            return Ok(Value::Hole(hole));
        }
        let period = period_arg(m, (call.args.first(), &single(&month)), "schedule")?;
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
        self.apply(m, (cal, env, ctx), &Business::Rolled(day, roll))
    }

    /// Business-day function `f` over the calendar `cal` reads, at each day it asks: a day, a
    /// count's or a roll's, `true` or `false`, or the hole met on the way (refcore's
    /// `c_is_business`, `c_add_business`, `c_last_business`, `c_schedule`). L §4.2.
    fn apply(
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
    /// way (refcore's `c_add_business`, its `count`); a count past the days to the edge passes
    /// it whatever each day is (L §4.2: n business days take at least n days). L §4.2,
    /// §7.6 item 1.
    fn count_business(
        &mut self,
        m: &Reality,
        (cal, env, ctx): (&Expr, &Env, Ctx),
        (from, step, count): (Day, i64, Option<u64>),
    ) -> Result<Value, Stop> {
        if passes_edge(from, step, count) {
            return Err(self.edge_first(m, (cal, env, ctx), (from, step))?);
        }
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

    /// A shift that passes the calendar's edge from `from` by steps of `step`: the count can't
    /// end before the edge, so its outcome is the first refusal or hole the walk meets in day
    /// order, else the edge's `RUN-LIMIT` (L §8 head R1; tally 2.1.8 R11, pick P26). Both
    /// change only where a row's span starts or ends, so the walk reads the day after `from`
    /// and those days, in day order: of the calendar's rows and of an input a row of it reads
    /// as a whole ([`inputs_followed`]). A refusal
    /// there stands; a hole, read or added, is met first, and the edge's refusal outranks it
    /// (refcore's `edge_first`). Returns the refusal that stands. L §4.2, §7.6 item 1, §8.4.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for an input load didn't build.
    fn edge_first(
        &mut self,
        m: &Reality,
        (cal, env, ctx): (&Expr, &Env, Ctx),
        (from, step): (Day, i64),
    ) -> Result<Stop, Stop> {
        let Some(first) = from.add_days(step) else {
            return Ok(past_days(step));
        };
        let mut edges = BTreeSet::from([first]);
        if let Ok((name, _)) = self.calendar_input(m, (cal, env, ctx))? {
            for input in inputs_followed(m, &name) {
                row_edges(m, &input, ctx, &mut edges)?;
            }
        }
        let mut ahead: Vec<Day> = edges
            .into_iter()
            .filter(|day| if step > 0 { *day > from } else { *day < from })
            .collect();
        if step < 0 {
            ahead.reverse();
        }
        for day in ahead {
            match self.business(m, (cal, env, ctx), (day, true)) {
                Ok(read) if read.as_hole().is_some() => break,
                Ok(_) => {}
                Err(refused) => return Ok(refused),
            }
        }
        Ok(past_days(step))
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
    /// off by its rules or an added day; else an added hole is `HOLE-CALENDAR`. Where the
    /// search for one asks (`seek`), a calendar with none is `RUN-DOMAIN` (refcore's `cal_at`,
    /// `business`). L §4.2, §7.6 item 5.
    fn business(
        &mut self,
        m: &Reality,
        (cal, env, ctx): (&Expr, &Env, Ctx),
        (day, seek): (Day, bool),
    ) -> Result<Value, Stop> {
        let (owned, keys) = match self.calendar_input(m, (cal, env, ctx))? {
            Err(hole) => return Ok(Value::Hole(hole)),
            Ok(found) => found,
        };
        let name = owned.as_str();
        let value = self.calendar_on(m, (name, &keys), ctx, (day, seek))?;
        let Value::Calendar(c) = &value else {
            return Ok(value);
        };
        if c.off_by_rule(day) {
            return Ok(Value::Bool(false));
        }
        let mut unknown: Vec<Hole> = Vec::new();
        for added in c.added() {
            match added {
                Added::Day(off) if *off == day => return Ok(Value::Bool(false)),
                Added::Day(_) => {}
                Added::Hole(question) => unknown.push(Hole::asked(question, name)),
            }
        }
        if unknown.is_empty() {
            return Ok(Value::Bool(true));
        }
        let holes: Vec<Option<&Hole>> = unknown.iter().map(Some).collect();
        hole_of(&holes)
    }

    /// The input a calendar argument reads, by id, and its keys, or the hole of a key
    /// (refcore's `cal_at`, its read). L §4.2.
    fn calendar_input(
        &mut self,
        m: &Reality,
        (cal, env, ctx): (&Expr, &Env, Ctx),
    ) -> Result<Result<(String, Vec<KeyVal>), Hole>, Stop> {
        let Some((name, written)) = read_parts(cal) else {
            return Err(Stop::NotBuilt("a calendar that isn't an input's read"));
        };
        let input = m
            .inputs
            .get(name)
            .ok_or(Stop::NotBuilt("a calendar that isn't an input's read"))?;
        let given = self.given_keys(m, written, env, ctx)?;
        let keys = self.make_keys(m, name, &input.binds, given, ctx)?;
        Ok(keys.map(|made| (input.id.clone(), made)))
    }

    /// The calendar input `name` holds at `keys`, in force on `day`: a calendar, or the hole
    /// of its read; `none` is `RUN-NONE`; where the search for a business day asks (`seek`),
    /// a calendar with none is `RUN-DOMAIN` (refcore's `cal_at`). L §4.2, §7.6 item 5, §8.4.
    fn calendar_on(
        &mut self,
        m: &Reality,
        (name, keys): (&str, &[KeyVal]),
        ctx: Ctx,
        (day, seek): (Day, bool),
    ) -> Result<Value, Stop> {
        let input = m
            .inputs
            .get(name)
            .ok_or_else(|| invariant("a calendar input load didn't build"))?;
        let value = self.read_input_at(m, input, keys, ctx.at_law(Some(day)), Asked::Calendar)?;
        if let Value::Hole(_) = value {
            return Ok(value);
        }
        let Value::Calendar(c) = &value else {
            if value == Value::None {
                return Err(refuse(Run::None, "the calendar is none here".to_owned()));
            }
            return Err(invariant(
                "a calendar argument that isn't a calendar: load's types refuse it first (LOAD-UNIT)",
            ));
        };
        if seek && !c.has_business_day() {
            return Err(refuse(
                Run::Domain,
                format!("calendar {} has no business day", input.id),
            ));
        }
        Ok(value)
    }
}

/// The days input `name`'s rows start and end, each with the day before, into `edges`, in the
/// cut and world of `ctx` (refcore's `edge_first`, its bounds). L §4.6, §5.3.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for an input load didn't build.
fn row_edges(m: &Reality, name: &str, ctx: Ctx, edges: &mut BTreeSet<Day>) -> Result<(), Stop> {
    let input = m
        .inputs
        .get(name)
        .ok_or_else(|| invariant("a calendar input load didn't build"))?;
    for row in &input.rows {
        let Some(span) = row_span(m, row, (ctx.cut, ctx.world))? else {
            continue;
        };
        for day in [span.start(), span.end()].into_iter().flatten() {
            edges.insert(day);
            edges.extend(day.add_days(-1));
        }
    }
    Ok(())
}

/// How many inputs [`inputs_followed`] gathers, one read through the next.
const FOLLOWED: usize = 16;

/// Calendar input `name` and the inputs a read of it may follow, whose rows' spans are where
/// its reads can change: an input a row of it reads as a whole (`cq = bm.cr`, L §6.4); each
/// once, at most [`FOLLOWED`], in the order met.
fn inputs_followed(m: &Reality, name: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut pending = vec![name.to_owned()];
    while let Some(next) = pending.pop() {
        if out.contains(&next) || out.len() >= FOLLOWED {
            continue;
        }
        if let Some(input) = m.inputs.get(&next) {
            for row in &input.rows {
                pending.extend(read_through(&row.value).map(str::to_owned));
            }
        }
        out.push(next);
    }
    out
}

/// The input a row's value reads as a whole, where it is a plain read of one: `cr`, `cr[k]`
/// or the base's `bm.cr` (L §6.4); none for any other value.
fn read_through(value: &Written) -> Option<&str> {
    let Written::Expr(e) = value else {
        return None;
    };
    if let Expr::Post(base, posts) = e
        && let (Expr::Name(_), [Postfix::Field(name)]) = (base.as_ref(), posts.as_slice())
    {
        return Some(name.as_str());
    }
    read_parts(e).map(|(name, _)| name)
}

/// Whether `count` business days by steps of `step` from `from` outnumber the days to the
/// calendar's edge that way, so the shift passes it whatever each day is (L §4.2: n business
/// days take at least n days; a count past 64 bits outnumbers every calendar's days).
fn passes_edge(from: Day, step: i64, count: Option<u64>) -> bool {
    let room = if step > 0 {
        from.days_to(Day::LAST)
    } else {
        i64::from(from.number()).checked_sub(1)
    };
    // a day past the edge has no room to count
    match (count, room.filter(|days| *days >= 0)) {
        (None, _) => true,
        (Some(n), Some(days)) => i128::from(n) > i128::from(days),
        (Some(_), None) => false,
    }
}

/// Whether argument `v` waits in P12's order: a hole, or a window a later step takes, not a
/// value known now. L §8, §4.4.
fn waits(v: &Value) -> bool {
    v.as_hole().is_some() || matches!(single(v).as_ref(), Value::Window(_))
}

/// A count argument in written order (`add_business`'s and `last_business`'s n,
/// `schedule`'s day of the month; pick P12): `none` is `RUN-NONE a value is none here: an
/// integer argument`, a known value its integer; none for one that waits. L §8, §4.2.
///
/// # Errors
///
/// `whole_of`'s refusals of a value that is no integer.
fn count_given(m: &Reality, v: &Value) -> Result<Option<Num>, Stop> {
    if waits(v) {
        return Ok(None);
    }
    whole_of(m, v).map(Some)
}

/// A call with the wrong number of arguments: load refuses it, so here it is an invariant.
fn arity() -> Stop {
    invariant("a built-in called with the wrong number of arguments")
}
