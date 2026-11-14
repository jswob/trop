//! An ask line's `windows each`: the line run once per combination of the days the windows it meets can take, and its results merged into maximal sub-windows.
//! Spec: L §4.4 (`windows each` prints one line per maximal sub-window, or combination of
//! sub-windows, over which the answer is constant, and one plain line when the answer doesn't
//! depend on the day; where the windows a line meets depend on the day an earlier window
//! takes, the combinations nest; the clause on an ask line holds for every window the line
//! meets through the whole walk; one run per sub-window combination it finds, bounded by the
//! windows' days), §5.5 (under an axis, each combination's result is its windows' lines, and
//! combinations whose lines are the same print under one head, the window part first), §5.18
//! (one line per binding tuple), §8.2.
//! Twin: engine/tools/refcore/ask.py (`each_lines`, `each_over_axes`, `each_rows`,
//! `each_nested`, `boxes`, `_box_cells`, `EachRes`).
//! Invariants: the runs go depth first, a run that meets a window its choice names no day of
//! run again once per day of the first such window, those runs before every run left; a run's
//! layer is open only while it runs; two runs give one answer when their lines print the same
//! text; the boxes are made per answer, in the order the answers were first given, each grown
//! greedily from its least day tuple, window by window in the order met, then sorted by their
//! first days; a line's register is every run's of its box together.

use std::collections::{BTreeSet, VecDeque};

use super::answer::{ask_pins_listed, env_text, form_line};
use super::axes::{Axes, grouped};
use super::check::Printed;
use super::register::{check_register_lines, register_lines};
use crate::base::{Code, Refusal};
use crate::eval::{Choice, Collected, Ctx, Engine, Entry, Env, Member, Met};
use crate::model::Reality;
use crate::syntax::{Ask, Stop};
use crate::time::Day;

/// One tuple's lines under `windows each`: each with its window part, ` [wd = a .. b]`, or
/// none for the plain line, and its result (refcore's `each_rows`, `(suffix, Line)`).
type Rows = Vec<(String, Printed)>;

/// One run of the line: its choice, the windows in the order met each with its day, and its
/// result.
struct Done {
    /// The choice.
    choice: Choice,
    /// The result.
    line: Printed,
}

/// The lines of ask `ask`, which states `windows each`, in context `ctx` with the `with`
/// pins' register `pinned` under each: one tuple's lines after another, each tuple's runs
/// fanned out over the axes the ask reaches (refcore's `_answer_in` with `each_mode`). L §4.4,
/// §5.5, §5.18.
///
/// # Errors
///
/// A load refusal met at run; `NotBuilt` for a construct not built yet.
pub(super) fn each_lines(
    m: &Reality,
    ask: &Ask,
    engine: &mut Engine,
    (ctx, pinned): (Ctx, &Collected),
) -> Result<Vec<String>, Stop> {
    let tuples = if ask.binds.is_empty() {
        vec![Member {
            env: Env::empty(),
            unknown: None,
            brought: Collected::default(),
        }]
    } else {
        // the tuples are bound once, outside the runs: a window met there takes its first
        // day (refcore's `members` under the line's empty choice)
        let bind_ctx = engine.with_windows(ctx, &Vec::new())?;
        engine.open_layer(m, bind_ctx)?;
        let members = engine.tuples(m, &ask.binds, bind_ctx);
        engine.end_layer(bind_ctx.layer, members.as_ref().err())?;
        match members? {
            Err(hole) => return Ok(vec![format!("{}: {}", ask.id, hole.text())]),
            Ok(tuples) => tuples,
        }
    };
    let axes = Axes::of(m, ask)?;
    let mut out = Vec::new();
    for tuple in tuples {
        let head = format!("{}{}", ask.id, env_text(m, &ask.binds, &tuple.env)?);
        if let Some(hole) = tuple.unknown {
            out.push(format!("{head}: {}", hole.text()));
            continue;
        }
        let lines = match &axes {
            Some(axes) => over_axes(m, ask, engine, (&tuple.env, ctx), axes)?,
            None => each_rows(m, ask, engine, &tuple.env, ctx)?
                .into_iter()
                .map(|(suffix, row)| (suffix, row, Vec::new()))
                .collect(),
        };
        // the movements a tuple binds aren't listed under its windows' lines (refcore's
        // `each_lines` takes no `tregs`)
        for (suffix, mut row, readings) in lines {
            out.extend(row.lines(&format!("{head}{suffix}")));
            row.met.merge(&ask_pins_listed(pinned, &row.met));
            for reading in readings {
                row.met.entries.insert(Entry {
                    word: "reading",
                    text: reading,
                    figure: String::new(),
                });
            }
            let register = match row.laws {
                Some(laws) => check_register_lines(&row.met, ctx, laws)?,
                None => register_lines(&row.met, ctx)?,
            };
            out.extend(register.into_iter().map(|l| format!("  {l}")));
        }
    }
    Ok(out)
}

/// One tuple's lines over the combinations of `axes`: each combination's windows' lines are
/// its result, the combinations that give the same lines grouped as a fan-out groups them,
/// each group printing its first combination's lines, the window part then the axis part,
/// with the group's `reading:` entries (refcore's `each_over_axes`). L §5.5, §4.4.
fn over_axes(
    m: &Reality,
    ask: &Ask,
    engine: &mut Engine,
    (env, ctx): (&Env, Ctx),
    axes: &Axes,
) -> Result<Vec<(String, Printed, Vec<String>)>, Stop> {
    let combos = axes.combinations()?;
    let mut each: Vec<Rows> = Vec::with_capacity(combos.len());
    for combo in &combos {
        let inside = engine.with_options(ctx, &axes.options_of(combo)?)?;
        engine.open_layer(m, inside)?;
        let rows = each_rows(m, ask, engine, env, inside);
        engine.end_layer(inside.layer, rows.as_ref().err())?;
        each.push(rows?);
    }
    // a combination's result is its lines, compared as the text they print (refcore's
    // `EachRes.key`)
    let mut results = Vec::with_capacity(each.len());
    for (at, rows) in each.iter().enumerate() {
        let parts: Vec<String> = rows
            .iter()
            .map(|(suffix, row)| format!("{suffix}\u{0}{}", row.text))
            .collect();
        let text = parts.join("\u{0}");
        results.push((
            at,
            Printed {
                text,
                body: Vec::new(),
                bare: false,
                whole: false,
                met: Collected::default(),
                laws: None,
            },
        ));
    }
    let mut out = Vec::new();
    for line in grouped(axes, &combos, &results)? {
        let (combo, _) = results
            .get(line.first)
            .ok_or_else(|| invariant("a line's result past the results"))?;
        let rows = each
            .get(*combo)
            .ok_or_else(|| invariant("a result past the combinations"))?;
        let part = format!(" [{}]", axes.head(&line.product)?);
        let readings = axes.readings(&line.product)?;
        for (suffix, row) in rows {
            out.push((format!("{suffix}{part}"), row.clone(), readings.clone()));
        }
    }
    Ok(out)
}

/// The line of ask `ask` in scope `env` and context `ctx`, run once per combination of the
/// days the windows it meets can take, depth first, and merged: one plain line where every
/// run gives one answer, else the nested sub-windows where the runs met different windows,
/// else the maximal boxes of days (refcore's `each_rows`). L §4.4.
fn each_rows(
    m: &Reality,
    ask: &Ask,
    engine: &mut Engine,
    env: &Env,
    ctx: Ctx,
) -> Result<Rows, Stop> {
    let mut pending: VecDeque<Choice> = VecDeque::from([Vec::new()]);
    let mut done: Vec<Done> = Vec::new();
    let mut labels: Vec<Met> = Vec::new();
    while let Some(choice) = pending.pop_front() {
        let line = run_once(m, ask, engine, (env, ctx), &choice)?;
        let (line, met) = line;
        let new = met.into_iter().find(|w| !names(&choice, &w.id));
        let Some(new) = new else {
            done.push(Done { choice, line });
            continue;
        };
        for day in new.days.iter().rev() {
            let mut next = choice.clone();
            next.push((new.id.clone(), *day));
            pending.push_front(next);
        }
        if !labels.iter().any(|w| w.id == new.id) {
            labels.push(new);
        }
    }
    let [first, rest @ ..] = done.as_slice() else {
        return Err(invariant("windows each with no run"));
    };
    if rest.iter().all(|d| d.line.same(&first.line)) {
        let mut line = first.line.clone();
        for d in rest {
            line.met.merge(&d.line.met);
        }
        return Ok(vec![(String::new(), line)]);
    }
    let all: BTreeSet<&str> = labels.iter().map(|w| w.id.as_str()).collect();
    if done.iter().all(|d| meets_all(&d.choice, &all)) {
        boxed(&labels, &done)
    } else {
        nested(&labels, &done)
    }
}

/// Whether choice `choice` names a day of every window in `all`, and of no other.
fn meets_all(choice: &Choice, all: &BTreeSet<&str>) -> bool {
    choice.len() == all.len() && choice.iter().all(|(id, _)| all.contains(id.as_str()))
}

/// One run of the line under choice `choice`: its result and the windows it met that the
/// choice names no day of, in the order met. L §4.4.
fn run_once(
    m: &Reality,
    ask: &Ask,
    engine: &mut Engine,
    (env, ctx): (&Env, Ctx),
    choice: &Choice,
) -> Result<(Printed, Vec<Met>), Stop> {
    let outer = engine.each_met.replace(Vec::new());
    let line = run_inside(m, ask, engine, (env, ctx), choice);
    let met = core::mem::replace(&mut engine.each_met, outer)
        .ok_or_else(|| invariant("a run's windows met taken while it ran"))?;
    Ok((line?, met))
}

/// The line of ask `ask` in scope `env`, in context `ctx` under choice `choice`, its layer
/// open while it is evaluated. L §4.4.
fn run_inside(
    m: &Reality,
    ask: &Ask,
    engine: &mut Engine,
    (env, ctx): (&Env, Ctx),
    choice: &Choice,
) -> Result<Printed, Stop> {
    let inside = engine.with_windows(ctx, choice)?;
    engine.open_layer(m, inside)?;
    let line = form_line(m, ask, engine, env, inside);
    engine.end_layer(inside.layer, line.as_ref().err())?;
    line
}

/// The runs `done`, each of which met every window of `labels`, merged into boxes: per
/// answer, in the order the answers were first given, the day tuples covered by maximal boxes
/// of consecutive days grown greedily from the least tuple, window by window in the order met;
/// the boxes sorted by their first days (refcore's `each_rows`, its `boxes`). L §4.4.
fn boxed(labels: &[Met], done: &[Done]) -> Result<Rows, Stop> {
    let mut answers: Vec<Vec<usize>> = Vec::new();
    for at in 0..done.len() {
        let mut found = None;
        for (place, members) in answers.iter().enumerate() {
            if same_first(done, members, &[at])? {
                found = Some(place);
                break;
            }
        }
        match found.and_then(|place| answers.get_mut(place)) {
            Some(members) => members.push(at),
            None => answers.push(vec![at]),
        }
    }
    let mut rows: Vec<(Vec<(Day, Day)>, Printed)> = Vec::new();
    for members in &answers {
        let mut cells: Vec<(Vec<Day>, usize)> = Vec::with_capacity(members.len());
        for at in members {
            let d = done
                .get(*at)
                .ok_or_else(|| invariant("a run past the runs"))?;
            cells.push((tuple_of(labels, &d.choice)?, *at));
        }
        for bounds in boxes(labels, &cells)? {
            let mut line: Option<Printed> = None;
            for (tuple, at) in &cells {
                if !inside(tuple, &bounds) {
                    continue;
                }
                let d = done
                    .get(*at)
                    .ok_or_else(|| invariant("a run past the runs"))?;
                match &mut line {
                    Some(l) => l.met.merge(&d.line.met),
                    None => line = Some(d.line.clone()),
                }
            }
            let line = line.ok_or_else(|| invariant("a box no run fills"))?;
            rows.push((bounds, line));
        }
    }
    rows.sort_by_key(|(bounds, _)| first_days(bounds));
    let mut out = Vec::with_capacity(rows.len());
    for (bounds, line) in rows {
        let mut parts = Vec::with_capacity(bounds.len());
        for (w, (lo, hi)) in labels.iter().zip(&bounds) {
            parts.push(format!("{} = {}", w.label, span_text(*lo, *hi)));
        }
        out.push((format!(" [{}]", parts.join(", ")), line));
    }
    Ok(out)
}

/// A box's first days, window by window: the key the boxes are sorted by.
fn first_days(bounds: &[(Day, Day)]) -> Vec<Day> {
    bounds.iter().map(|(lo, _)| *lo).collect()
}

/// The day each window of `labels` takes in `choice`, in the windows' order. L §4.4.
fn tuple_of(labels: &[Met], choice: &Choice) -> Result<Vec<Day>, Stop> {
    let mut out = Vec::with_capacity(labels.len());
    for w in labels {
        let day = choice
            .iter()
            .find(|(id, _)| *id == w.id)
            .map(|(_, day)| *day)
            .ok_or_else(|| invariant("a run that met a window its choice names no day of"))?;
        out.push(day);
    }
    Ok(out)
}

/// Whether day tuple `tuple` lies inside box `bounds`, window by window.
fn inside(tuple: &[Day], bounds: &[(Day, Day)]) -> bool {
    tuple
        .iter()
        .zip(bounds)
        .all(|(day, (lo, hi))| lo <= day && day <= hi)
}

/// A box's days for one window, `a` alone or `a .. b`.
fn span_text(lo: Day, hi: Day) -> String {
    if lo == hi {
        lo.text()
    } else {
        format!("{} .. {}", lo.text(), hi.text())
    }
}

/// The day tuples of `cells` covered by boxes of consecutive days of each window of
/// `labels`: from each tuple left in tuple order, each window in turn grown to its next day
/// while every tuple of the grown box is one of those left; the box's tuples are then taken
/// (refcore's `boxes`). L §4.4.
fn boxes(labels: &[Met], cells: &[(Vec<Day>, usize)]) -> Result<Vec<Vec<(Day, Day)>>, Stop> {
    let mut left: BTreeSet<Vec<Day>> = cells.iter().map(|(t, _)| t.clone()).collect();
    let sorted: Vec<Vec<Day>> = left.iter().cloned().collect();
    let mut out = Vec::new();
    for start in sorted {
        if !left.contains(&start) {
            continue;
        }
        let mut bounds: Vec<(Day, Day)> = start.iter().map(|d| (*d, *d)).collect();
        for (at, w) in labels.iter().enumerate() {
            loop {
                let Some((_, hi)) = bounds.get(at) else {
                    return Err(invariant("a box with fewer windows than met"));
                };
                let Some(next) = next_day(&w.days, *hi) else {
                    break;
                };
                let mut slice = bounds.clone();
                if let Some(edge) = slice.get_mut(at) {
                    *edge = (next, next);
                }
                if !box_tuples(labels, &slice)?.iter().all(|t| left.contains(t)) {
                    break;
                }
                if let Some(edge) = bounds.get_mut(at) {
                    edge.1 = next;
                }
            }
        }
        for t in box_tuples(labels, &bounds)? {
            left.remove(&t);
        }
        out.push(bounds);
    }
    Ok(out)
}

/// The day after `day` among a window's `days`, if any.
fn next_day(days: &[Day], day: Day) -> Option<Day> {
    let at = days.iter().position(|d| *d == day)?;
    days.get(at.checked_add(1)?).copied()
}

/// Every day tuple of box `bounds`: the product of each window's days from its low end to its
/// high end (refcore's `_box_cells`).
fn box_tuples(labels: &[Met], bounds: &[(Day, Day)]) -> Result<Vec<Vec<Day>>, Stop> {
    let mut out: Vec<Vec<Day>> = vec![Vec::new()];
    for (w, (lo, hi)) in labels.iter().zip(bounds) {
        let days: Vec<Day> = w
            .days
            .iter()
            .copied()
            .filter(|d| lo <= d && d <= hi)
            .collect();
        if days.is_empty() {
            return Err(invariant("a box's window with no day"));
        }
        let mut next = Vec::new();
        for t in &out {
            for d in &days {
                let mut longer = t.clone();
                longer.push(*d);
                next.push(longer);
            }
        }
        out = next;
    }
    Ok(out)
}

/// The runs `done`, which met different windows by the days earlier windows took, as nested
/// sub-windows: under each run of days of the earlier window whose nested lines give the same
/// answers, the windows met there split in turn (refcore's `each_nested`; L §4.4: "the
/// combinations nest"). A loop over an explicit stack (R-16).
fn nested(labels: &[Met], done: &[Done]) -> Result<Rows, Stop> {
    let all: Vec<usize> = (0..done.len()).collect();
    let mut out = Vec::new();
    for (parts, members) in nest(labels, done, all)? {
        let mut line: Option<Printed> = None;
        for at in members {
            let d = done
                .get(at)
                .ok_or_else(|| invariant("a run past the runs"))?;
            match &mut line {
                Some(l) => l.met.merge(&d.line.met),
                None => line = Some(d.line.clone()),
            }
        }
        let line = line.ok_or_else(|| invariant("a nested line no run fills"))?;
        out.push((format!(" [{}]", parts.join(", ")), line));
    }
    Ok(out)
}

/// One level of the nesting: the runs, which share their first `depth` days, and the depth;
/// they split by the next window they met, day by day, runs of days whose lines below agree
/// merged.
type Level = (Vec<usize>, usize);

/// A level's nested lines: each its window parts and the runs it stands for.
type Nested = Vec<(Vec<String>, Vec<usize>)>;

/// The nested lines of the runs `members` of `done` from depth 0: each its window parts and
/// the runs it stands for (refcore's `each_nested`, its `nest`). L §4.4.
fn nest(labels: &[Met], done: &[Done], members: Vec<usize>) -> Result<Nested, Stop> {
    // each level's lines are made bottom up: a work stack of levels to open, and the lines
    // each opened level waits for
    let mut made: Vec<(Level, Nested)> = Vec::new();
    let mut stack: Vec<(Level, bool)> = vec![((members, 0), false)];
    while let Some(((runs, depth), opened)) = stack.pop() {
        if one_answer(done, &runs)? {
            made.push(((runs.clone(), depth), vec![(Vec::new(), runs)]));
            continue;
        }
        let (label, days) = level_window(labels, done, &runs, depth)?;
        let per = per_day(done, &runs, depth, &days)?;
        let next = after(depth)?;
        if !opened {
            stack.push(((runs.clone(), depth), true));
            for (_, sub) in &per {
                stack.push(((sub.clone(), next), false));
            }
            continue;
        }
        let mut below: Vec<(Day, Nested)> = Vec::with_capacity(per.len());
        for (day, sub) in per {
            let lines = made
                .iter()
                .find(|((m, d), _)| *m == sub && *d == next)
                .map(|(_, lines)| lines.clone())
                .ok_or_else(|| invariant("a nested level made out of order"))?;
            below.push((day, lines));
        }
        let lines = merged_days(done, &label, &below)?;
        made.push(((runs, depth), lines));
    }
    made.pop()
        .map(|(_, lines)| lines)
        .ok_or_else(|| invariant("a nesting with no level"))
}

/// Whether the runs `members` of `done` give one answer.
fn one_answer(done: &[Done], members: &[usize]) -> Result<bool, Stop> {
    let mut first: Option<&Printed> = None;
    for at in members {
        let d = done
            .get(*at)
            .ok_or_else(|| invariant("a run past the runs"))?;
        match first {
            Some(f) if !f.same(&d.line) => return Ok(false),
            Some(_) => {}
            None => first = Some(&d.line),
        }
    }
    Ok(true)
}

/// The window the runs `members` met at position `depth`, as the first of them met it: its
/// label and its days.
fn level_window(
    labels: &[Met],
    done: &[Done],
    members: &[usize],
    depth: usize,
) -> Result<(String, Vec<Day>), Stop> {
    let first = members
        .first()
        .and_then(|at| done.get(*at))
        .ok_or_else(|| invariant("a nesting level with no run"))?;
    let (id, _) = first
        .choice
        .get(depth)
        .ok_or_else(|| invariant("runs that differ with no window left"))?;
    let w = labels
        .iter()
        .find(|w| w.id == *id)
        .ok_or_else(|| invariant("a window met with no label"))?;
    Ok((w.label.clone(), w.days.clone()))
}

/// The runs `members` by the day they took at position `depth`, the days in order, a day no
/// run took left out.
fn per_day(
    done: &[Done],
    members: &[usize],
    depth: usize,
    days: &[Day],
) -> Result<Vec<(Day, Vec<usize>)>, Stop> {
    let mut out = Vec::new();
    for day in days {
        let mut sub = Vec::new();
        for at in members {
            let d = done
                .get(*at)
                .ok_or_else(|| invariant("a run past the runs"))?;
            if d.choice.get(depth).is_some_and(|(_, took)| took == day) {
                sub.push(*at);
            }
        }
        if !sub.is_empty() {
            out.push((*day, sub));
        }
    }
    Ok(out)
}

/// The lines of one level from the lines below each of its days `below`: consecutive days
/// whose lines below give the same parts and answers merged, each such run printing the
/// window `label` over its days before the parts below (refcore's `nest`, its merge).
fn merged_days(done: &[Done], label: &str, below: &[(Day, Nested)]) -> Result<Nested, Stop> {
    let mut out = Vec::new();
    let mut i = 0;
    while let Some((first_day, first)) = below.get(i) {
        let mut j = i;
        while let Some((_, next)) = below.get(after(j)?)
            && same_lines(done, first, next)?
        {
            j = after(j)?;
        }
        let run = below
            .get(i..=j)
            .ok_or_else(|| invariant("a run of days past the days"))?;
        let last_day = run.last().map_or(*first_day, |(d, _)| *d);
        for (k, (parts, _)) in first.iter().enumerate() {
            let mut runs = Vec::new();
            for (_, lines) in run {
                if let Some((_, members)) = lines.get(k) {
                    runs.extend(members.iter().copied());
                }
            }
            let mut named = vec![format!("{label} = {}", span_text(*first_day, last_day))];
            named.extend(parts.iter().cloned());
            out.push((named, runs));
        }
        i = after(j)?;
    }
    Ok(out)
}

/// The place after `at`.
fn after(at: usize) -> Result<usize, Stop> {
    at.checked_add(1)
        .ok_or_else(|| invariant("a place past the integers"))
}

/// Whether the first runs of `a` and of `b` give one answer.
fn same_first(done: &[Done], a: &[usize], b: &[usize]) -> Result<bool, Stop> {
    let (Some(x), Some(y)) = (a.first(), b.first()) else {
        return Err(invariant("a nested line no run fills"));
    };
    let (Some(x), Some(y)) = (done.get(*x), done.get(*y)) else {
        return Err(invariant("a run past the runs"));
    };
    Ok(x.line.same(&y.line))
}

/// Whether choice `choice` names a day of window `id`.
fn names(choice: &Choice, id: &str) -> bool {
    choice.iter().any(|(named, _)| named == id)
}

/// Whether two days' lines below give the same parts and answers (refcore's `sig`).
fn same_lines(done: &[Done], a: &Nested, b: &Nested) -> Result<bool, Stop> {
    if a.len() != b.len() {
        return Ok(false);
    }
    for ((pa, ra), (pb, rb)) in a.iter().zip(b) {
        if pa != pb || !same_first(done, ra, rb)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// `ENGINE-INVARIANT` for a broken invariant of `windows each`.
fn invariant(what: &str) -> Stop {
    Stop::Refused(Refusal::new(Code::EngineInvariant, what.to_owned()))
}
