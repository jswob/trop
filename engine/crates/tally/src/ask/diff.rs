//! A `diff` ask's result: its value in two worlds, key by key changed, added, removed or unchanged, then the register of both.
//! Spec: L §5.18 (`diff(e [for binds]) in S against B`: e evaluated in world S and in world B,
//! keyed by the binding tuple, a diff line in the world where its `where` is a hole printed as
//! that hole), §5.5 (tally 2.1.7 R8: an ask of any form fans out over the axes it reads, a
//! `diff`'s result being its set of lines, equal results under one head, no head when it reads
//! no axis), §6.4 (asking a scenario: a world below the reality's own hides the layers over
//! it; two worlds on two stacks, each side in the reality that holds its stack), §8.2 (the
//! register: `world: s` for each world evaluated), §8.4 (a run-time refusal prints in its
//! value's place); E §3.3.
//! Twin: engine/tools/refcore/ask.py (`answer_diff`, `diff_worlds`, `diff_lines`,
//! `diff_print`, `rkey`, `fmt_value`, `fmt_bind`), cli.py (`cross_stack_diff`) and check.py
//! (`_txt`, the key of a diff without bindings).
//! Invariants: a cell per combination, in refcore's order, each with its combination's layer
//! open only while it is evaluated (the ask's loop, `answer.rs` `tuple_lines`, or here for a
//! diff across two stacks); in a cell each world is evaluated in its own timeline,
//! entered and left around it, in the order written (S, then B); a key is its tuple's
//! variables, or the expression's text with no bindings; S's lines come in their order, then
//! B's keys that S lacks; two values are unchanged only when they are the same value, a hole
//! or a refusal included; a cell's register is the ask's pins' and both evaluations' entries;
//! a cell across two stacks is each side's at the options of the axes that side reads, and
//! under `windows each` each side takes only the windows its answer carries (tally 2.1.8
//! R13.2), its runs being the pairs of the two sides' runs that take the same day of every
//! window both meet (L §4.4; tally 2.1.8 R13, #186; refcore prints the first day's).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::answer::{item_text, tuple_key};
use super::axes::{Axes, result_lines};
use super::check::Printed;
use super::register::register_lines;
use super::windows::{CellRuns, EACH_CAP, crossed_lines, each_next, past_cap};
use crate::eval::{Choice, Collected, Ctx, Engine, Entry, Env, Item, Met, world_named};
use crate::model::{Reality, Value};
use crate::syntax::{Ask, Diff, Stop};
use crate::walk::invariant;

/// A world's lines of a diff: each key with its value, in the order evaluated.
type Side = Vec<(String, Item)>;

/// The keys a diff counts by outcome.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Counts {
    /// In both worlds, with other values.
    changed: usize,
    /// Only in the world compared.
    added: usize,
    /// Only in the world compared against.
    removed: usize,
    /// In both worlds, with the same value.
    unchanged: usize,
}

/// One cell of a diff (refcore's `diff_worlds`): its combination of the axes the diff fans out
/// over, the lines of each world it evaluated, and what they read.
#[derive(Debug, Clone)]
struct Cell {
    /// Each axis's id with its option, the axes in id order; none when it fans out over none.
    options: Vec<(String, String)>,
    /// The lines of the world compared, where this cell evaluated it.
    ours: Option<Side>,
    /// The lines of the world compared against, where this cell evaluated it.
    theirs: Option<Side>,
    /// The register entries its evaluations met, the ask's pins' first.
    met: Collected,
    /// Under `windows each` across two stacks, its runs; none otherwise.
    runs: Runs,
}

/// Under `windows each`, a cell's runs: each choice of days for the windows its reads meet,
/// depth first, with the cell that choice gives, and the windows met in the order met
/// (refcore's `each_rows`, one side of a diff across two stacks at a time). L §4.4, §6.4.
#[derive(Debug, Clone, Default)]
struct Runs {
    /// Each choice with its cell.
    each: Vec<(Choice, Cell)>,
    /// The windows met, in the order met.
    windows: Vec<Met>,
}

/// A diff's cells: the axes they fan out over, one cell per combination in refcore's order,
/// and the line's context, whose law date and cut its register prints (refcore's
/// `answer_diff`, its `cells`). L §5.5.
#[derive(Debug)]
pub(super) struct Cells {
    /// The axes the cells fan out over; none when the diff reads none.
    axes: Option<Axes>,
    /// The cells, one per combination.
    each: Vec<Cell>,
    /// The line's context.
    ctx: Ctx,
}

/// A `diff` ask's result in context `ctx` (refcore's `answer_diff`, `diff_print`): the
/// counts, a line per key changed, added or removed, the register of both worlds with the
/// ask's pins `pinned`; the ask's loop fans it out over the axes it reads, one combination's
/// cell each. L §5.18, §5.5, §6.4.
///
/// # Errors
///
/// A refusal met at run; `NotBuilt` past what the engine builds.
pub(super) fn diff_result(
    m: &Reality,
    diff: &Diff,
    engine: &mut Engine,
    (env, ctx, pinned): (&Env, Ctx, &Collected),
) -> Result<Printed, Stop> {
    let made = cell(m, diff, engine, (env, ctx, pinned), (Vec::new(), None))?;
    result(m, diff, &made)
}

/// A diff's cells, one per combination of the axes the ask fans out over: in each, the
/// diff's value in world `only` alone, the side this reality's stack holds, under `windows
/// each` once per choice of days (refcore's `answer_diff`, `diff_worlds`, `_diff_only`).
/// L §5.5, §5.18, §6.4, §4.4.
///
/// # Errors
///
/// A load refusal met at run; `RUN-LIMIT` past the cap of day combinations; `NotBuilt` past
/// what the engine builds.
pub(super) fn diff_cells(
    m: &Reality,
    (ask, diff): (&Ask, &Diff),
    engine: &mut Engine,
    (env, ctx, pinned): (&Env, Ctx, &Collected),
    only: &str,
) -> Result<Cells, Stop> {
    let axes = Axes::of(m, ask)?;
    let Some(fanned) = &axes else {
        let cell = side_cell(
            m,
            (ask, diff),
            engine,
            (env, ctx, pinned),
            (Vec::new(), only),
        )?;
        return Ok(Cells {
            axes,
            each: vec![cell],
            ctx,
        });
    };
    let combos = fanned.combinations()?;
    let mut each = Vec::with_capacity(combos.len());
    for combo in &combos {
        let options = fanned.options_of(combo)?;
        let inside = engine.with_options(ctx, &options)?;
        engine.open_layer(m, inside)?;
        let made = side_cell(
            m,
            (ask, diff),
            engine,
            (env, inside, pinned),
            (options, only),
        );
        engine.end_layer(inside.layer, made.as_ref().err())?;
        each.push(made?);
    }
    Ok(Cells { axes, each, ctx })
}

/// One side's cell at combination `options`: the diff's value in world `only`, or under
/// `windows each` its runs, one per choice of the days of the windows its answer carries,
/// the first its reads met taken first, depth first, each with its cell (refcore's
/// `each_rows`, `each_next`, one side at a time). L §4.4, §6.4; tally 2.1.8 R13.2.
fn side_cell(
    m: &Reality,
    (ask, diff): (&Ask, &Diff),
    engine: &mut Engine,
    (env, ctx, pinned): (&Env, Ctx, &Collected),
    (options, only): (Vec<(String, String)>, &str),
) -> Result<Cell, Stop> {
    if !ask.windows_each {
        return cell(m, diff, engine, (env, ctx, pinned), (options, Some(only)));
    }
    let mut pending: VecDeque<Choice> = VecDeque::from([Vec::new()]);
    let mut runs = Runs::default();
    while let Some(choice) = pending.pop_front() {
        let outer = engine.each_met.replace(Vec::new());
        let made = chosen_cell(m, diff, engine, (env, ctx, pinned), (&choice, only));
        let met = core::mem::replace(&mut engine.each_met, outer)
            .ok_or_else(|| invariant("a run's windows met taken while it ran"))?;
        let made = made?;
        // a run takes next only a window its answer carries (L §4.4; tally 2.1.8 R13.2)
        let mut values: Vec<&Item> = Vec::new();
        for side in [&made.ours, &made.theirs].into_iter().flatten() {
            values.extend(side.iter().map(|(_, v)| v));
        }
        let carried = super::check::carried_of(&values);
        let Some(new) = each_next(&met, &choice, (&carried, &[])) else {
            runs.each.push((choice, made));
            continue;
        };
        for day in new.days.iter().rev() {
            let mut next = choice.clone();
            next.push((new.id.clone(), *day));
            pending.push_front(next);
        }
        // the fan-out's cap, as a line's (pick P28, #174, interim)
        if pending
            .len()
            .checked_add(runs.each.len())
            .is_none_or(|n| n > EACH_CAP)
        {
            return Err(past_cap());
        }
        if !runs.windows.iter().any(|w| w.id == new.id) {
            runs.windows.push(new);
        }
    }
    Ok(Cell {
        options,
        ours: None,
        theirs: None,
        met: Collected::default(),
        runs,
    })
}

/// One run's cell: the diff's value in world `only` under choice `choice`, its layer open
/// while it is evaluated. L §4.4.
fn chosen_cell(
    m: &Reality,
    diff: &Diff,
    engine: &mut Engine,
    (env, ctx, pinned): (&Env, Ctx, &Collected),
    (choice, only): (&Choice, &str),
) -> Result<Cell, Stop> {
    let inside = engine.with_windows(ctx, choice)?;
    engine.open_layer(m, inside)?;
    let made = cell(
        m,
        diff,
        engine,
        (env, inside, pinned),
        (Vec::new(), Some(only)),
    );
    engine.end_layer(inside.layer, made.as_ref().err())?;
    made
}

/// The cells of a diff whose two worlds sit on two stacks of scenarios, from `ours`, the side
/// of the world compared, and `theirs`, the other's, each answered in the reality that holds
/// its stack: one cell per combination of the axes either side reads, each side's at the
/// options of the axes it reads, the register both sides' (refcore's `cross_stack_diff`).
/// L §5.5, §6.4.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a side with no cell at a combination's options; `RUN-LIMIT` past
/// [`EACH_CAP`] pairs of runs under `windows each`.
pub(super) fn crossed(m: &Reality, (ours, theirs): (&Cells, &Cells)) -> Result<Cells, Stop> {
    let mut ids: BTreeSet<String> = BTreeSet::new();
    for side in [ours, theirs] {
        ids.extend(side.axes.iter().flat_map(Axes::ids));
    }
    let axes = Axes::named(m, &ids)?;
    let every = match &axes {
        None => vec![Vec::new()],
        Some(fanned) => {
            let combos = fanned.combinations()?;
            let mut out = Vec::with_capacity(combos.len());
            for combo in &combos {
                out.push(fanned.options_of(combo)?);
            }
            out
        }
    };
    let mut each = Vec::with_capacity(every.len());
    for options in every {
        let left = ours.at(&options)?;
        let right = theirs.at(&options)?;
        let mut met = left.met.clone();
        met.merge(&right.met);
        each.push(Cell {
            options,
            ours: left.ours.clone(),
            theirs: right.theirs.clone(),
            met,
            runs: paired(&left.runs, &right.runs)?,
        });
    }
    Ok(Cells {
        axes,
        each,
        ctx: ours.ctx,
    })
}

/// The runs of a cell across two stacks under `windows each`: each run of the compared
/// world's side with each of the other side's whose choice takes the same day of every window
/// both meet, the choices joined, the compared world's windows first, as one stack's run
/// meets them (L §4.4, §6.4).
///
/// # Errors
///
/// `RUN-LIMIT` past [`EACH_CAP`] pairs, as a line's runs (L §7.6 item 6; pick P28, #174).
fn paired(left: &Runs, right: &Runs) -> Result<Runs, Stop> {
    let mut windows = left.windows.clone();
    for w in &right.windows {
        if !windows.iter().any(|x| x.id == w.id) {
            windows.push(w.clone());
        }
    }
    let mut each = Vec::new();
    for (ours, l) in &left.each {
        for (theirs, r) in &right.each {
            let Some(choice) = joined(ours, theirs) else {
                continue;
            };
            let mut met = l.met.clone();
            met.merge(&r.met);
            each.push((
                choice,
                Cell {
                    options: Vec::new(),
                    ours: l.ours.clone(),
                    theirs: r.theirs.clone(),
                    met,
                    runs: Runs::default(),
                },
            ));
            if each.len() > EACH_CAP {
                return Err(past_cap());
            }
        }
    }
    Ok(Runs { each, windows })
}

/// Choice `ours` then the days of `theirs` it names no day of; none where the two take
/// different days of one window.
fn joined(ours: &Choice, theirs: &Choice) -> Option<Choice> {
    let mut choice = ours.clone();
    for (id, day) in theirs {
        match ours.iter().find(|(named, _)| named == id) {
            Some((_, taken)) if taken != day => return None,
            Some(_) => {}
            None => choice.push((id.clone(), *day)),
        }
    }
    Some(choice)
}

impl Cells {
    /// Merges `brought`, what a binding tuple brought (the movements it binds), into every
    /// cell's register and every run's (L §8.2).
    pub(super) fn bring(&mut self, brought: &Collected) {
        for cell in &mut self.each {
            cell.met.merge(brought);
            for (_, run) in &mut cell.runs.each {
                run.met.merge(brought);
            }
        }
    }

    /// The cell at the options of `options` of the axes this side reads.
    fn at(&self, options: &[(String, String)]) -> Result<&Cell, Stop> {
        let ids: BTreeSet<String> = self.axes.iter().flat_map(Axes::ids).collect();
        let wanted: Vec<(String, String)> = options
            .iter()
            .filter(|(axis, _)| ids.contains(axis))
            .cloned()
            .collect();
        self.each
            .iter()
            .find(|c| c.options == wanted)
            .ok_or_else(|| invariant("a diff's side with no cell at a combination"))
    }
}

/// A diff's lines from its cells under head `head`, the ask's id and a tuple's binding part:
/// its one result with no axis part when it reads no axis, else one line per distinct result
/// under a head naming its options, as a value's lines are (refcore's `diff_print`).
/// L §5.5, §5.18.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a cell with a side unevaluated.
pub(super) fn cells_lines(
    m: &Reality,
    (ask, diff, head): (&Ask, &Diff, &str),
    cells: &Cells,
) -> Result<Vec<String>, Stop> {
    if ask.windows_each {
        let mut each: Vec<CellRuns> = Vec::with_capacity(cells.each.len());
        for cell in &cells.each {
            let mut runs = Vec::with_capacity(cell.runs.each.len());
            for (choice, run) in &cell.runs.each {
                runs.push((choice.clone(), result(m, diff, run)?));
            }
            each.push((runs, cell.runs.windows.clone()));
        }
        return crossed_lines(head, cells.axes.as_ref(), each, cells.ctx);
    }
    let mut results = Vec::with_capacity(cells.each.len());
    for (at, cell) in cells.each.iter().enumerate() {
        results.push((at, result(m, diff, cell)?));
    }
    if let Some(axes) = &cells.axes {
        let combos = axes.combinations()?;
        // the ask's pins are in each cell's register already
        let pins = Collected::default();
        return result_lines((axes, &combos), head, &results, (cells.ctx, &pins));
    }
    let [(_, only)] = results.as_slice() else {
        return Err(invariant(
            "a diff that reads no axis with other than one cell",
        ));
    };
    let mut out = only.lines(head);
    let register = register_lines(&only.met, cells.ctx)?;
    out.extend(register.into_iter().map(|l| format!("  {l}")));
    Ok(out)
}

/// One cell: the diff's value in each world it compares, or in `only` alone, with the
/// combination `options` it is evaluated in (refcore's `diff_worlds`). L §5.18, §6.4.
fn cell(
    m: &Reality,
    diff: &Diff,
    engine: &mut Engine,
    (env, ctx, pinned): (&Env, Ctx, &Collected),
    (options, only): (Vec<(String, String)>, Option<&str>),
) -> Result<Cell, Stop> {
    let mut met = pinned.clone();
    let mut sides = [None, None];
    for (slot, world) in sides.iter_mut().zip([&diff.world, &diff.against]) {
        if only.is_none_or(|w| w == world.as_str()) {
            *slot = Some(side(m, (diff, world), engine, (env, ctx, &mut met))?);
        }
    }
    let [ours, theirs] = sides;
    Ok(Cell {
        options,
        ours,
        theirs,
        met,
        runs: Runs::default(),
    })
}

/// A cell's result: its set of lines, the counts as its text and the lines `changed`,
/// `added` and `removed` under it (refcore's `diff_lines`). L §5.18, §5.5.
fn result(m: &Reality, diff: &Diff, cell: &Cell) -> Result<Printed, Stop> {
    let (Some(ours), Some(theirs)) = (&cell.ours, &cell.theirs) else {
        return Err(invariant("a diff's cell with a world unevaluated"));
    };
    // a key met twice in a world keeps its last value, as a dict of the lines does
    let ours_by: BTreeMap<&str, &Item> = ours.iter().map(|(k, v)| (k.as_str(), v)).collect();
    let theirs_by: BTreeMap<&str, &Item> = theirs.iter().map(|(k, v)| (k.as_str(), v)).collect();
    let mut counts = Counts::default();
    let mut lines = Vec::new();
    for (key, v) in ours {
        match theirs_by.get(key.as_str()) {
            Some(b) if *b == v => counts.unchanged = one_more(counts.unchanged)?,
            Some(b) => {
                counts.changed = one_more(counts.changed)?;
                lines.push(format!(
                    "  changed {key}: {} against {}",
                    item_text(m, v)?,
                    item_text(m, b)?
                ));
            }
            None => {
                counts.added = one_more(counts.added)?;
                lines.push(format!("  added {key}: {}", item_text(m, v)?));
            }
        }
    }
    for (key, v) in theirs {
        if !ours_by.contains_key(key.as_str()) {
            counts.removed = one_more(counts.removed)?;
            lines.push(format!("  removed {key}: {}", item_text(m, v)?));
        }
    }
    let text = format!(
        "diff in {} against {}: {} changed, {} added, {} removed, {} unchanged",
        diff.world, diff.against, counts.changed, counts.added, counts.removed, counts.unchanged
    );
    Ok(Printed {
        text,
        body: lines,
        bare: false,
        whole: true,
        alone: false,
        met: cell.met.clone(),
        laws: None,
        carried: super::check::carried_of(
            &ours
                .iter()
                .chain(theirs)
                .map(|(_, v)| v)
                .collect::<Vec<_>>(),
        ),
        causes: None,
    })
}

/// The diff's value in world `world`, a line per binding tuple, in that world's timeline; what
/// it met, `world: <world>` first, into `met` (refcore's `answer_diff`, one world). L §5.18, §6.4.
fn side(
    m: &Reality,
    (diff, world): (&Diff, &str),
    engine: &mut Engine,
    (env, ctx, met): (&Env, Ctx, &mut Collected),
) -> Result<Side, Stop> {
    met.entries.insert(Entry {
        word: "world",
        text: world.to_owned(),
        figure: String::new(),
    });
    let inside = ctx.in_world(world_named(m, world)?);
    engine.enter_timeline(m, inside)?;
    let lines = side_lines(m, diff, engine, (env, inside, met));
    engine.leave_timeline()?;
    lines
}

/// The lines of one world's side: the expression's text and its value with no bindings, else
/// a key per binding tuple, a tuple whose `where` is a hole printed as that hole, a domain that
/// is a hole one line of it (refcore's `answer_diff`, its loop). L §5.18.
fn side_lines(
    m: &Reality,
    diff: &Diff,
    engine: &mut Engine,
    (env, ctx, met): (&Env, Ctx, &mut Collected),
) -> Result<Side, Stop> {
    let text = crate::load::expr_text(&diff.expr);
    if diff.binds.is_empty() {
        return Ok(vec![(text, one(m, diff, engine, (env, ctx), met)?)]);
    }
    // the diff's own binding, inside the ask's tuple (L §5.18, tally 2.1.8 R13)
    let tuples = match engine.tuples(m, &diff.binds, (env, ctx))? {
        Err(hole) => return Ok(vec![(text, Item::Value(Value::Hole(hole)))]),
        Ok(tuples) => tuples,
    };
    let mut lines = Vec::with_capacity(tuples.len());
    for tuple in tuples {
        met.merge(&tuple.brought);
        let key = tuple_key(m, &diff.binds, &tuple.env)?;
        let value = match tuple.unknown {
            Some(hole) => Item::Value(Value::Hole(hole)),
            None => one(m, diff, engine, (&tuple.env, ctx), met)?,
        };
        lines.push((key, value));
    }
    Ok(lines)
}

/// The diff's expression at one tuple: its value or the run-time refusal it met, what it read
/// into `met` (refcore's `one`). L §5.18, §8.4.
fn one(
    m: &Reality,
    diff: &Diff,
    engine: &mut Engine,
    (env, ctx): (&Env, Ctx),
    met: &mut Collected,
) -> Result<Item, Stop> {
    let (items, read) = engine.line(m, core::slice::from_ref(&diff.expr), env, ctx)?;
    met.merge(&read);
    let mut items = items.into_iter();
    match (items.next(), items.next()) {
        (Some(item), None) => Ok(item),
        _ => Err(invariant("a diff's value as other than one item")),
    }
}

/// One more key of an outcome.
fn one_more(n: usize) -> Result<usize, Stop> {
    n.checked_add(1)
        .ok_or_else(|| invariant("a count of a diff's keys"))
}
