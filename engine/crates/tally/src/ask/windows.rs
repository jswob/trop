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
use super::axes::{Axes, Combo, grouped};
use super::check::Printed;
use super::register::{check_register_lines, register_lines};
use crate::base::{Code, Refusal, Run};
use crate::eval::{Choice, Collected, Ctx, Engine, Entry, Env, Member, Met};
use crate::model::Reality;
use crate::syntax::{Ask, Stop};
use crate::time::Day;

/// The most day combinations one line's `windows each` runs: past it the line is refused
/// `RUN-LIMIT` (pick P28, #174: refcore's point, until the spec names one cap for every
/// fan-out).
pub(super) const EACH_CAP: usize = 20_000;

/// One tuple's lines under `windows each`: each with its window part, ` [wd = a .. b]`, or
/// none for the plain line, its result, and the runs it stands for (refcore's `each_rows`,
/// `(suffix, Line)`).
type Rows = Vec<(String, Printed, Vec<usize>)>;

/// One tuple's lines over the axes: each with its window part then its axis part, its result,
/// and the `reading:` entries of its axis part (refcore's `each_over_axes`).
type Fanned = Vec<(String, Printed, Vec<String>)>;

/// One cell's runs of a diff across two stacks under `windows each`: each choice of days with
/// the result it gives, then the windows met in the order met (L §4.4, §6.4).
pub(super) type CellRuns = (Vec<(Choice, Printed)>, Vec<Met>);

/// One run of the line: its choice, the windows in the order met each with its day, and its
/// result.
struct Done {
    /// The choice.
    choice: Choice,
    /// The result: the line's, or under a binding every tuple's, compared as they print.
    line: Printed,
    /// Under a binding, each tuple's head and line, or the hole it prints.
    tuples: Vec<Tuple>,
}

impl Done {
    /// Its tuple of binding part `head`, where the run binds it.
    fn tuple(&self, head: &str) -> Option<&Tuple> {
        self.tuples.iter().find(|t| t.head == head)
    }
}

/// What one run gives: its result to compare, and under a binding each tuple's line.
type Ran = (Printed, Vec<Tuple>);

/// One box of days: each window's first and last day, its line, the runs it stands for.
type Boxed = (Vec<(Day, Day)>, Printed, Vec<usize>);

/// One box of a tuple's days: each window's first and last day, the runs it stands for.
type TupleBox = (Vec<(Day, Day)>, Vec<usize>);

/// One tuple of a binding taken inside a run: its head's binding part, and its line or the
/// hole its line prints (L §5.18).
struct Tuple {
    /// The binding part of its head, ` (m = g)`; empty for the binding's own hole.
    head: String,
    /// Its line, or the hole that stands for it.
    line: Result<Printed, String>,
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
    let axes = Axes::of(m, ask)?;
    let tuples = if ask.binds.is_empty() {
        vec![Member {
            env: engine.ask_env.clone(),
            unknown: None,
            brought: Collected::default(),
        }]
    } else {
        // the tuples are bound once, outside the runs, each tuple's line then split by the
        // windows it meets (refcore's `members` under the line's empty choice)
        let outer = engine.each_met.replace(Vec::new());
        let bind_ctx = under(engine, ask, ctx, &Vec::new())?;
        engine.open_layer(m, bind_ctx)?;
        let members = engine.tuples(m, &ask.binds, bind_ctx);
        engine.end_layer(bind_ctx.layer, members.as_ref().err())?;
        let met = core::mem::replace(&mut engine.each_met, outer)
            .ok_or_else(|| invariant("a binding's windows met taken while it ran"))?;
        if !met.is_empty() {
            // the binding meets a window: it binds inside each run, so the window splits
            // its members as it splits every value, under each axis combination too (L §4.4)
            return bound_lines(m, ask, engine, (ctx, pinned), axes.as_ref());
        }
        match members? {
            Err(hole) => return Ok(vec![format!("{}: {}", ask.id, hole.text())]),
            Ok(tuples) => tuples,
        }
    };
    let mut out = Vec::new();
    for tuple in tuples {
        let head = format!("{}{}", ask.id, env_text(m, &ask.binds, &tuple.env)?);
        if let Some(hole) = tuple.unknown {
            out.push(format!("{head}: {}", hole.text()));
            continue;
        }
        let lines = if let Some(axes) = &axes {
            over_axes(m, ask, engine, (&tuple.env, ctx, pinned), axes)?
        } else {
            let (rows, _, _) = each_rows(m, ask, engine, (&tuple.env, ctx, pinned), Shape::Line)?;
            rows.into_iter()
                .map(|(suffix, row, _)| (suffix, row, Vec::new()))
                .collect()
        };
        out.extend(fanned_lines(&head, lines, (&tuple.brought, pinned), ctx)?);
    }
    Ok(out)
}

/// The lines of one tuple's rows `lines` under head `head`: each row's window and axis parts
/// after the head, its result, then its register: what it read, what its tuple brought
/// `brought` (the movements it binds), the ask's pins `pinned` and its `reading:` entries; a
/// row that prints alone has none (L §8.2, §4.4).
fn fanned_lines(
    head: &str,
    lines: Fanned,
    (brought, pinned): (&Collected, &Collected),
    ctx: Ctx,
) -> Result<Vec<String>, Stop> {
    let mut out = Vec::new();
    for (suffix, mut row, readings) in lines {
        out.extend(row.lines(&format!("{head}{suffix}")));
        if row.alone {
            continue;
        }
        row.met.merge(brought);
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
    Ok(out)
}

/// The lines of a diff whose worlds sit on two stacks, under `windows each`: each cell's runs
/// `cells` merged into rows as a line's runs are, one block per run of sub-windows that agree,
/// one unlabelled block where all agree; over the axes either side reads, the cells grouped as
/// a fan-out groups them; each block under the ask's id, its window part then its axis part,
/// with its register (L §4.4, §5.5, §6.4; tally 2.1.8 R13, #186).
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a cell with no run.
pub(super) fn crossed_lines(
    ask: &Ask,
    axes: Option<&Axes>,
    cells: Vec<CellRuns>,
    ctx: Ctx,
) -> Result<Vec<String>, Stop> {
    let mut each: Vec<Rows> = Vec::with_capacity(cells.len());
    for (runs, labels) in cells {
        let done: Vec<Done> = runs
            .into_iter()
            .map(|(choice, line)| Done {
                choice,
                line,
                tuples: Vec::new(),
            })
            .collect();
        each.push(rows_of(&done, &labels)?);
    }
    let lines = match axes {
        Some(axes) => grouped_rows(axes, &axes.combinations()?, &each)?,
        None => each
            .into_iter()
            .flatten()
            .map(|(suffix, row, _)| (suffix, row, Vec::new()))
            .collect(),
    };
    // a cell's register holds the ask's pins already
    let none = Collected::default();
    fanned_lines(&ask.id, lines, (&none, &none), ctx)
}

/// One line of a keyed `windows each` line whose binding meets a window: its head after the
/// ask's id, the binding part then the window part; its line, or the hole its tuple prints;
/// and what its runs read together (L §5.18, §4.4, §8.2).
struct BoundRow {
    /// The binding part, ` (k = a)`.
    tuple: String,
    /// The window part, ` [d[a] = 2026-03-01]`; empty for the plain line.
    suffix: String,
    /// Its line, or the hole that stands for it.
    line: Result<Printed, String>,
    /// What the runs it stands for read.
    met: Collected,
}

/// The lines of ask `ask`, which states `windows each` and binds keys, its binding taken
/// inside each run: per sub-window, each tuple's line with the sub-window after its binding,
/// or none where the sub-window binds nothing, as an empty binding prints (L §4.4, §5.18;
/// pick P29); over the axes the ask reaches, each combination's lines, grouped as a fan-out
/// groups them (L §5.5). A line's register is what its tuple's runs read (L §8.2).
///
/// # Errors
///
/// A load refusal met at run; `NotBuilt` for a construct not built yet.
fn bound_lines(
    m: &Reality,
    ask: &Ask,
    engine: &mut Engine,
    (ctx, pinned): (Ctx, &Collected),
    axes: Option<&Axes>,
) -> Result<Vec<String>, Stop> {
    if let Some(axes) = axes {
        return bound_over_axes(m, ask, engine, (ctx, pinned), axes);
    }
    let mut out = Vec::new();
    for row in bound_rows(m, ask, engine, (ctx, pinned))? {
        out.extend(row_lines(ask, row, ("", &[]), (ctx, pinned))?);
    }
    Ok(out)
}

/// The rows of ask `ask`'s keyed `windows each` line in context `ctx` with the ask's pins
/// `pinned`, its binding taken inside each run ([`bound_lines`]). L §4.4, §5.18.
fn bound_rows(
    m: &Reality,
    ask: &Ask,
    engine: &mut Engine,
    (ctx, pinned): (Ctx, &Collected),
) -> Result<Vec<BoundRow>, Stop> {
    let (rows, done, labels) =
        each_rows(m, ask, engine, (&Env::empty(), ctx, pinned), Shape::Bound)?;
    let all: BTreeSet<&str> = labels.iter().map(|w| w.id.as_str()).collect();
    if rows.len() > 1 && done.iter().all(|d| meets_all(&d.choice, &all)) {
        return tuple_rows(&done, &labels);
    }
    let mut out = Vec::new();
    for (suffix, _, runs) in rows {
        let first = runs
            .first()
            .and_then(|at| done.get(*at))
            .ok_or_else(|| invariant("a line no run fills"))?;
        for (k, tuple) in first.tuples.iter().enumerate() {
            let met = if tuple.line.is_ok() {
                runs_met(&done, &runs, k)
            } else {
                Collected::default()
            };
            out.push(BoundRow {
                tuple: tuple.head.clone(),
                suffix: suffix.clone(),
                line: tuple.line.clone(),
                met,
            });
        }
    }
    Ok(out)
}

/// What the line of the `k`th tuple of each of the runs `runs` of `done` read, together
/// (L §8.2).
fn runs_met(done: &[Done], runs: &[usize], k: usize) -> Collected {
    let mut met = Collected::default();
    for at in runs {
        if let Some(Tuple { line: Ok(l), .. }) = done.get(*at).and_then(|d| d.tuples.get(k)) {
            met.merge(&l.met);
        }
    }
    met
}

/// The lines of bound row `row` of ask `ask`, the axis part `part` after its head and the
/// axes' `readings` in its register: its line, then its register with the `with` pins'; a
/// tuple's hole alone (L §5.18, §5.5, §8.2).
fn row_lines(
    ask: &Ask,
    row: BoundRow,
    (part, readings): (&str, &[String]),
    (ctx, pinned): (Ctx, &Collected),
) -> Result<Vec<String>, Stop> {
    let head = format!("{}{}{}{part}", ask.id, row.tuple, row.suffix);
    let line = match row.line {
        Err(hole) => return Ok(vec![format!("{head}: {hole}")]),
        Ok(line) => line,
    };
    let mut out = line.lines(&head);
    let mut met = row.met;
    met.merge(&ask_pins_listed(pinned, &met));
    for reading in readings {
        met.entries.insert(Entry {
            word: "reading",
            text: reading.clone(),
            figure: String::new(),
        });
    }
    let register = match line.laws {
        Some(laws) => check_register_lines(&met, ctx, laws)?,
        None => register_lines(&met, ctx)?,
    };
    out.extend(register.into_iter().map(|l| format!("  {l}")));
    Ok(out)
}

/// [`bound_lines`] over the combinations of `axes`: each combination's rows, run in its
/// options' layer; per tuple, in the order the combinations bind them, its rows are a
/// combination's result, the combinations that give one result grouped as a fan-out groups
/// them, each group printing its first combination's rows, the window part then the axis
/// part, each row's register every grouped combination's (refcore's `each_over_axes`, as
/// [`over_axes`] groups a tuple bound once). L §5.5, §4.4; pick P29.
fn bound_over_axes(
    m: &Reality,
    ask: &Ask,
    engine: &mut Engine,
    (ctx, pinned): (Ctx, &Collected),
    axes: &Axes,
) -> Result<Vec<String>, Stop> {
    let combos = axes.combinations()?;
    let mut each: Vec<Vec<BoundRow>> = Vec::with_capacity(combos.len());
    for combo in &combos {
        let inside = engine.with_options(ctx, &axes.options_of(combo)?)?;
        engine.open_layer(m, inside)?;
        let rows = bound_rows(m, ask, engine, (inside, pinned));
        engine.end_layer(inside.layer, rows.as_ref().err())?;
        each.push(rows?);
    }
    let mut out = Vec::new();
    for tuple in &tuple_heads(&each)? {
        let mut results = Vec::new();
        let mut given: Vec<Vec<&BoundRow>> = Vec::new();
        for (combo, rows) in each.iter().enumerate() {
            let mine: Vec<&BoundRow> = rows.iter().filter(|r| r.tuple == *tuple).collect();
            if !mine.is_empty() {
                results.push((combo, compared(&mine)));
                given.push(mine);
            }
        }
        for line in grouped(axes, &combos, &results)? {
            let first = given
                .get(line.first)
                .ok_or_else(|| invariant("a line's rows past the results"))?;
            let part = format!(" [{}]", axes.head(&line.product)?);
            let readings = axes.readings(&line.product)?;
            for (at, row) in first.iter().enumerate() {
                let row = BoundRow {
                    tuple: row.tuple.clone(),
                    suffix: row.suffix.clone(),
                    line: row.line.clone(),
                    met: members_met(&given, &line.members, at),
                };
                out.extend(row_lines(ask, row, (&part, &readings), (ctx, pinned))?);
            }
        }
    }
    Ok(out)
}

/// What the `at`th row of each grouped combination `members` (places in `given`) read,
/// together: a grouped line's register (L §8.2, §5.5).
fn members_met(given: &[Vec<&BoundRow>], members: &[usize], at: usize) -> Collected {
    let mut met = Collected::default();
    for member in members {
        if let Some(same) = given.get(*member).and_then(|rows| rows.get(at)) {
            met.merge(&same.met);
        }
    }
    met
}

/// Every combination's tuple heads in order: each combination lists its rows in binding
/// order, so a tuple a combination adds goes right after the last of its tuples already
/// placed, or first ([`binding_order`]'s rule). L §5.18, §5.5.
fn tuple_heads(each: &[Vec<BoundRow>]) -> Result<Vec<String>, Stop> {
    let mut heads: Vec<String> = Vec::new();
    for rows in each {
        let mut at = 0;
        for row in rows {
            match heads.iter().position(|h| *h == row.tuple) {
                Some(p) => at = p,
                None => heads = placed_at(heads, at, &row.tuple),
            }
            at = at
                .checked_add(1)
                .ok_or_else(|| invariant("a row past the integers"))?;
        }
    }
    Ok(heads)
}

/// One tuple's rows in a combination, as a fan-out compares them: each row's window part
/// and its line's text, or its hole marked apart from a line's (refcore's `EachRes.key`).
/// L §5.5.
fn compared(rows: &[&BoundRow]) -> Printed {
    let parts: Vec<String> = rows
        .iter()
        .map(|row| match &row.line {
            Ok(l) => format!("{}\u{0}{}\u{0}{}", row.suffix, l.text, l.body.join("\u{0}")),
            Err(hole) => format!("{}\u{0}?{hole}", row.suffix),
        })
        .collect();
    Printed {
        text: parts.join("\u{1}"),
        body: Vec::new(),
        bare: false,
        whole: false,
        alone: false,
        met: Collected::default(),
        laws: None,
    }
}

/// The rows of a keyed `windows each` line whose binding meets a window, every run having met
/// every window: tuple by tuple, in the order the runs bind them, each tuple's lines split only
/// by the windows its own line or its membership depends on, its sub-windows merged as a line's
/// are; none where a sub-window doesn't bind it (L §4.4, §5.18; pick P29). A row's register is
/// what its tuple's runs read (L §8.2).
fn tuple_rows(done: &[Done], labels: &[Met]) -> Result<Vec<BoundRow>, Stop> {
    let heads = binding_order(done)?;
    let mut days = Vec::with_capacity(done.len());
    for d in done {
        days.push(tuple_of(labels, &d.choice)?);
    }
    let mut out = Vec::new();
    for head in &heads {
        let head = head.as_str();
        let keys: Vec<Option<String>> = done.iter().map(|d| tuple_key(d, head)).collect();
        let kept = windows_that_matter(labels.len(), &days, &keys)?;
        let sub: Vec<Met> = kept
            .iter()
            .filter_map(|at| labels.get(*at).cloned())
            .collect();
        let mut projected: Vec<Vec<Day>> = Vec::with_capacity(days.len());
        for t in &days {
            projected.push(kept.iter().filter_map(|at| t.get(*at).copied()).collect());
        }
        for (bounds, runs) in tuple_boxes(&sub, &projected, &keys)? {
            let suffix = if sub.is_empty() {
                String::new()
            } else {
                let parts: Vec<String> = sub
                    .iter()
                    .zip(&bounds)
                    .map(|(w, (lo, hi))| format!("{} = {}", w.label, span_text(*lo, *hi)))
                    .collect();
                format!(" [{}]", parts.join(", "))
            };
            out.push(tuple_box_row(done, (head, &suffix), &runs)?);
        }
    }
    Ok(out)
}

/// What tuple `head` gives in run `d`, as it prints; none where the run doesn't bind it.
fn tuple_key(d: &Done, head: &str) -> Option<String> {
    let t = d.tuple(head)?;
    Some(match &t.line {
        Ok(l) => format!(
            "{}\u{0}{}\u{0}{}",
            l.text,
            u8::from(l.bare),
            l.body.join("\u{0}")
        ),
        Err(hole) => format!("?{hole}"),
    })
}

/// The windows, by their place in the order met, on whose day a tuple's result `keys` (one per
/// run, the runs' days `days`) depends: two runs whose days differ in that window alone give
/// different results. L §4.4 (one line where the answer doesn't depend on the day).
fn windows_that_matter(
    count: usize,
    days: &[Vec<Day>],
    keys: &[Option<String>],
) -> Result<Vec<usize>, Stop> {
    let mut kept = Vec::new();
    for at in 0..count {
        let mut seen: std::collections::BTreeMap<Vec<Day>, &Option<String>> =
            std::collections::BTreeMap::new();
        let mut matters = false;
        for (t, key) in days.iter().zip(keys) {
            if t.get(at).is_none() {
                return Err(invariant("a run's days past the windows met"));
            }
            let mut rest: Vec<Day> = Vec::with_capacity(t.len());
            for (i, d) in t.iter().enumerate() {
                if i != at {
                    rest.push(*d);
                }
            }
            match seen.get(&rest) {
                Some(other) if *other != key => {
                    matters = true;
                    break;
                }
                Some(_) => {}
                None => {
                    seen.insert(rest, key);
                }
            }
        }
        if matters {
            kept.push(at);
        }
    }
    Ok(kept)
}

/// A tuple's boxes over the windows `sub` it depends on: per result, in the order first given,
/// the maximal boxes of its runs' days `projected` (as [`boxed`] grows them), sorted by their
/// first days, each with the runs inside it; a result where the tuple isn't bound has none.
fn tuple_boxes(
    sub: &[Met],
    projected: &[Vec<Day>],
    keys: &[Option<String>],
) -> Result<Vec<TupleBox>, Stop> {
    let mut answers: Vec<&Option<String>> = Vec::new();
    for key in keys {
        if !answers.contains(&key) {
            answers.push(key);
        }
    }
    let mut rows = Vec::new();
    for answer in answers {
        if answer.is_none() {
            continue;
        }
        let mut cells: Vec<(Vec<Day>, usize)> = Vec::new();
        for (at, (t, key)) in projected.iter().zip(keys).enumerate() {
            if key == answer && !cells.iter().any(|(c, _)| c == t) {
                cells.push((t.clone(), at));
            }
        }
        for bounds in boxes(sub, &cells)? {
            let mut runs: Vec<usize> = Vec::new();
            for (at, (t, key)) in projected.iter().zip(keys).enumerate() {
                if key == answer && inside(t, &bounds) {
                    runs.push(at);
                }
            }
            rows.push((bounds, runs));
        }
    }
    rows.sort_by_key(|(bounds, _)| first_days(bounds));
    Ok(rows)
}

/// Every run's tuple heads in binding order (one line per binding tuple, L §2.5): each run
/// lists its tuples in the binding's order, so a head a run adds goes right after the last
/// head of that run already placed, or first. Two heads no run holds together are ordered by
/// no run: the one a later run adds goes before. L §2.5, §4.4.
fn binding_order(done: &[Done]) -> Result<Vec<String>, Stop> {
    let mut heads: Vec<String> = Vec::new();
    for d in done {
        let mut at = 0;
        for t in &d.tuples {
            match heads.iter().position(|h| *h == t.head) {
                Some(p) => at = p,
                None => heads = placed_at(heads, at, &t.head),
            }
            at = at
                .checked_add(1)
                .ok_or_else(|| invariant("a binding past the integers"))?;
        }
    }
    Ok(heads)
}

/// `heads` with `head` placed before position `at` (at the end past it).
fn placed_at(heads: Vec<String>, at: usize, head: &str) -> Vec<String> {
    let past = at >= heads.len();
    let mut out = Vec::new();
    for (i, h) in heads.into_iter().enumerate() {
        if i == at {
            out.push(head.to_owned());
        }
        out.push(h);
    }
    if past {
        out.push(head.to_owned());
    }
    out
}

/// One box's row of tuple `head`: its first run's line under `suffix`, with every run's
/// register, or the hole that stands for it (L §5.18, §8.2).
fn tuple_box_row(
    done: &[Done],
    (head, suffix): (&str, &str),
    runs: &[usize],
) -> Result<BoundRow, Stop> {
    let first = runs
        .first()
        .and_then(|at| done.get(*at))
        .and_then(|d| d.tuple(head))
        .ok_or_else(|| invariant("a tuple's box no run binds"))?;
    let mut met = Collected::default();
    if first.line.is_ok() {
        for at in runs {
            let same = done.get(*at).and_then(|d| d.tuple(head));
            if let Some(Tuple { line: Ok(l), .. }) = same {
                met.merge(&l.met);
            }
        }
    }
    Ok(BoundRow {
        tuple: head.to_owned(),
        suffix: suffix.to_owned(),
        line: first.line.clone(),
        met,
    })
}

/// One run's binding and each tuple's line in context `ctx` with the ask's pins `pinned`:
/// the tuples, and the run's result to compare, every tuple's head and line as they print
/// (L §5.18, §4.4).
fn bound_run(
    m: &Reality,
    ask: &Ask,
    engine: &mut Engine,
    (ctx, pinned): (Ctx, &Collected),
) -> Result<(Printed, Vec<Tuple>), Stop> {
    let mut tuples = Vec::new();
    match engine.tuples(m, &ask.binds, ctx)? {
        Err(hole) => tuples.push(Tuple {
            head: String::new(),
            line: Err(hole.text()),
        }),
        Ok(members) => {
            for member in members {
                let head = env_text(m, &ask.binds, &member.env)?;
                let line = if let Some(hole) = member.unknown {
                    Err(hole.text())
                } else {
                    let mut line = form_line(m, ask, engine, (&member.env, ctx, pinned))?;
                    line.met.merge(&member.brought);
                    Ok(line)
                };
                tuples.push(Tuple { head, line });
            }
        }
    }
    let mut parts = Vec::with_capacity(tuples.len());
    let mut met = Collected::default();
    for t in &tuples {
        match &t.line {
            Ok(l) => {
                parts.push(format!(
                    "{}\u{0}{}\u{0}{}",
                    t.head,
                    l.text,
                    l.body.join("\u{0}")
                ));
                met.merge(&l.met);
            }
            Err(hole) => parts.push(format!("{}\u{0}?{hole}", t.head)),
        }
    }
    let line = Printed {
        text: parts.join("\u{1}"),
        body: Vec::new(),
        bare: false,
        whole: false,
        alone: false,
        met,
        laws: None,
    };
    Ok((line, tuples))
}

/// One tuple's lines over the combinations of `axes`: each combination's windows' lines are
/// its result, the combinations that give the same lines grouped as a fan-out groups them,
/// each group printing its first combination's lines, the window part then the axis part,
/// with the group's `reading:` entries (refcore's `each_over_axes`). L §5.5, §4.4.
fn over_axes(
    m: &Reality,
    ask: &Ask,
    engine: &mut Engine,
    (env, ctx, pinned): (&Env, Ctx, &Collected),
    axes: &Axes,
) -> Result<Fanned, Stop> {
    let combos = axes.combinations()?;
    let mut each: Vec<Rows> = Vec::with_capacity(combos.len());
    for combo in &combos {
        let inside = engine.with_options(ctx, &axes.options_of(combo)?)?;
        engine.open_layer(m, inside)?;
        let rows = each_rows(m, ask, engine, (env, inside, pinned), Shape::Line);
        engine.end_layer(inside.layer, rows.as_ref().err())?;
        each.push(rows?.0);
    }
    grouped_rows(axes, &combos, &each)
}

/// The rows `each` of the combinations `combos` of `axes`, one combination's rows its result,
/// grouped as [`over_axes`] groups them. L §5.5, §4.4.
fn grouped_rows(axes: &Axes, combos: &[Combo], each: &[Rows]) -> Result<Fanned, Stop> {
    // a combination's result is its lines, compared as the results they print, an
    // `inputs-of`'s rows included (refcore's `EachRes.key`, `rkey`)
    let mut results = Vec::with_capacity(each.len());
    for (at, rows) in each.iter().enumerate() {
        let parts: Vec<String> = rows
            .iter()
            .map(|(suffix, row, _)| format!("{suffix}\u{0}{}", row.key()))
            .collect();
        let text = parts.join("\u{0}");
        results.push((
            at,
            Printed {
                text,
                body: Vec::new(),
                bare: false,
                whole: false,
                alone: false,
                met: Collected::default(),
                laws: None,
            },
        ));
    }
    let mut out = Vec::new();
    for line in grouped(axes, combos, &results)? {
        let (combo, _) = results
            .get(line.first)
            .ok_or_else(|| invariant("a line's result past the results"))?;
        let rows = each
            .get(*combo)
            .ok_or_else(|| invariant("a result past the combinations"))?;
        let part = format!(" [{}]", axes.head(&line.product)?);
        let readings = axes.readings(&line.product)?;
        for (row_at, (suffix, row, _)) in rows.iter().enumerate() {
            // a grouped line's register is every combination's in it (L §8.2, §5.5)
            let mut row = row.clone();
            for at in &line.members {
                let (other, _) = results
                    .get(*at)
                    .ok_or_else(|| invariant("a member's result past the results"))?;
                if let Some((_, same, _)) = each.get(*other).and_then(|r| r.get(row_at)) {
                    row.met.merge(&same.met);
                }
            }
            out.push((format!("{suffix}{part}"), row, readings.clone()));
        }
    }
    Ok(out)
}

/// The line of ask `ask` in scope `env` and context `ctx` with the ask's pins `pinned`, run
/// once per combination of the days the windows it meets can take, depth first, and merged:
/// one plain line where every run gives one answer, else the nested sub-windows where the
/// runs met different windows, else the maximal boxes of days (refcore's `each_rows`). Under
/// a binding (`Shape::Bound`) each run binds the line's keys itself. With the rows, the runs,
/// and the windows met in the order met. A ledger's or a diff's result is its block of lines,
/// so its rows are blocks, each under its window part (L §4.4; tally 2.1.8 R13, #186).
///
/// # Errors
///
/// A refusal a run meets; `RUN-LIMIT` past the cap.
fn each_rows(
    m: &Reality,
    ask: &Ask,
    engine: &mut Engine,
    (env, ctx, pinned): (&Env, Ctx, &Collected),
    shape: Shape,
) -> Result<(Rows, Vec<Done>, Vec<Met>), Stop> {
    let mut pending: VecDeque<Choice> = VecDeque::from([Vec::new()]);
    let mut done: Vec<Done> = Vec::new();
    let mut labels: Vec<Met> = Vec::new();
    while let Some(choice) = pending.pop_front() {
        let ((line, tuples), met) = run_once(m, ask, engine, (env, ctx, pinned), (&choice, shape))?;
        let new = met.into_iter().find(|w| !names(&choice, &w.id));
        let Some(new) = new else {
            done.push(Done {
                choice,
                line,
                tuples,
            });
            continue;
        };
        for day in new.days.iter().rev() {
            let mut next = choice.clone();
            next.push((new.id.clone(), *day));
            pending.push_front(next);
        }
        // the fan-out's cap, refcore's point (pick P28, #174, interim)
        if pending
            .len()
            .checked_add(done.len())
            .is_none_or(|n| n > EACH_CAP)
        {
            return Err(past_cap());
        }
        if !labels.iter().any(|w| w.id == new.id) {
            labels.push(new);
        }
    }
    let rows = rows_of(&done, &labels)?;
    Ok((rows, done, labels))
}

/// The rows the runs `done` give, the windows they met `labels`: one plain line where every
/// run gives one answer, else the nested sub-windows where the runs met different windows,
/// else the maximal boxes of days (refcore's `each_rows`, its end). L §4.4.
fn rows_of(done: &[Done], labels: &[Met]) -> Result<Rows, Stop> {
    let [first, rest @ ..] = done else {
        return Err(invariant("windows each with no run"));
    };
    if rest.iter().all(|d| d.line.same(&first.line)) {
        let mut line = first.line.clone();
        for d in rest {
            line.met.merge(&d.line.met);
        }
        let runs = (0..done.len()).collect();
        return Ok(vec![(String::new(), line, runs)]);
    }
    let all: BTreeSet<&str> = labels.iter().map(|w| w.id.as_str()).collect();
    if done.iter().all(|d| meets_all(&d.choice, &all)) {
        boxed(labels, done)
    } else {
        nested(labels, done)
    }
}

/// `RUN-LIMIT` past [`EACH_CAP`] day combinations (pick P28, #174, interim).
pub(super) fn past_cap() -> Stop {
    Stop::Refused(Refusal::new(
        Code::Run(Run::Limit),
        "windows each over more than 20 000 day combinations".to_owned(),
    ))
}

/// Whether choice `choice` names a day of every window in `all`, and of no other.
fn meets_all(choice: &Choice, all: &BTreeSet<&str>) -> bool {
    choice.len() == all.len() && choice.iter().all(|(id, _)| all.contains(id.as_str()))
}

/// Context `ctx` under choice `choice`: every read of the line takes it where the line
/// states `windows each`, else only the reads that state it (refcore's `each` and
/// `each-read` modes). L §4.4.
fn under(engine: &mut Engine, ask: &Ask, ctx: Ctx, choice: &Choice) -> Result<Ctx, Stop> {
    if ask.windows_each {
        engine.with_windows(ctx, choice)
    } else {
        engine.with_read_windows(ctx, choice)
    }
}

/// What one run of the line evaluates (L §4.4, §5.18).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shape {
    /// The line in its scope.
    Line,
    /// The line's binding, then each tuple's line.
    Bound,
}

/// One run of the line under choice `choice`: its result, its tuples under a binding
/// (`Shape::Bound`), and the windows it met that the choice names no day of, in the order
/// met. L §4.4.
fn run_once(
    m: &Reality,
    ask: &Ask,
    engine: &mut Engine,
    (env, ctx, pinned): (&Env, Ctx, &Collected),
    (choice, shape): (&Choice, Shape),
) -> Result<(Ran, Vec<Met>), Stop> {
    let outer = engine.each_met.replace(Vec::new());
    let line = run_inside(m, ask, engine, (env, ctx, pinned), (choice, shape));
    let met = core::mem::replace(&mut engine.each_met, outer)
        .ok_or_else(|| invariant("a run's windows met taken while it ran"))?;
    Ok((line?, met))
}

/// The line of ask `ask` in scope `env`, or under a binding (`Shape::Bound`) its tuples'
/// lines, in context `ctx` under choice `choice` with the ask's pins `pinned`, its layer open
/// while it is evaluated. L §4.4.
fn run_inside(
    m: &Reality,
    ask: &Ask,
    engine: &mut Engine,
    (env, ctx, pinned): (&Env, Ctx, &Collected),
    (choice, shape): (&Choice, Shape),
) -> Result<(Printed, Vec<Tuple>), Stop> {
    let inside = under(engine, ask, ctx, choice)?;
    engine.open_layer(m, inside)?;
    let line = match shape {
        Shape::Bound => bound_run(m, ask, engine, (inside, pinned)),
        Shape::Line => {
            form_line(m, ask, engine, (env, inside, pinned)).map(|line| (line, Vec::new()))
        }
    };
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
    let mut rows: Vec<Boxed> = Vec::new();
    for members in &answers {
        let mut cells: Vec<(Vec<Day>, usize)> = Vec::with_capacity(members.len());
        for at in members {
            let d = done
                .get(*at)
                .ok_or_else(|| invariant("a run past the runs"))?;
            cells.push((tuple_of(labels, &d.choice)?, *at));
        }
        for bounds in boxes(labels, &cells)? {
            let mut runs = Vec::new();
            for (tuple, at) in &cells {
                if inside(tuple, &bounds) {
                    runs.push(*at);
                }
            }
            rows.push((bounds, merged(done, &runs)?, runs));
        }
    }
    rows.sort_by_key(|(bounds, _, _)| first_days(bounds));
    let mut out = Vec::with_capacity(rows.len());
    for (bounds, line, runs) in rows {
        let mut parts = Vec::with_capacity(bounds.len());
        for (w, (lo, hi)) in labels.iter().zip(&bounds) {
            parts.push(format!("{} = {}", w.label, span_text(*lo, *hi)));
        }
        out.push((format!(" [{}]", parts.join(", ")), line, runs));
    }
    Ok(out)
}

/// The line the runs `runs` of `done` give together: the first's, with every run's register
/// (L §4.4, §8.2).
fn merged(done: &[Done], runs: &[usize]) -> Result<Printed, Stop> {
    let mut line: Option<Printed> = None;
    for at in runs {
        let d = done
            .get(*at)
            .ok_or_else(|| invariant("a run past the runs"))?;
        match &mut line {
            Some(l) => l.met.merge(&d.line.met),
            None => line = Some(d.line.clone()),
        }
    }
    line.ok_or_else(|| invariant("a line no run fills"))
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
        let line = merged(done, &members)?;
        out.push((format!(" [{}]", parts.join(", ")), line, members));
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
pub(super) fn names(choice: &Choice, id: &str) -> bool {
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
