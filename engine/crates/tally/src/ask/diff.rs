//! A `diff` ask's lines: its value in two worlds, key by key changed, added, removed or unchanged, per combination of the axes it reads, then the register of both.
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
//! open only while it is evaluated; in a cell each world is evaluated in its own timeline,
//! entered and left around it, in the order written (S, then B); a key is its tuple's
//! variables, or the expression's text with no bindings; S's lines come in their order, then
//! B's keys that S lacks; two values are unchanged only when they are the same value, a hole
//! or a refusal included; a cell's register is the ask's pins' and both evaluations' entries;
//! a cell across two stacks is each side's at the options of the axes that side reads.

use std::collections::{BTreeMap, BTreeSet};

use super::answer::{item_text, tuple_key};
use super::axes::{Axes, result_lines};
use super::check::Printed;
use super::register::register_lines;
use crate::eval::{Collected, Ctx, Engine, Entry, Env, Item, world_named};
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

/// A `diff` ask's lines (refcore's `answer_diff`, `diff_print`): per distinct result across
/// the combinations of the axes it reads, the head with the counts, a line per key changed,
/// added or removed, then the register of both worlds. L §5.18, §5.5, §6.4.
///
/// # Errors
///
/// A load refusal met at run; `NotBuilt` past what the engine builds.
pub(super) fn diff_lines(
    m: &Reality,
    (ask, diff): (&Ask, &Diff),
    engine: &mut Engine,
    (ctx, pinned): (Ctx, &Collected),
) -> Result<Vec<String>, Stop> {
    let cells = diff_cells(m, (ask, diff), engine, (ctx, pinned), None)?;
    cells_lines(m, (ask, diff), &cells)
}

/// A diff's cells, one per combination of the axes the ask fans out over: in each, the
/// diff's value in each world it compares, or in world `only` alone, the side this reality's
/// stack holds (refcore's `answer_diff`, `diff_worlds`, `_diff_only`). L §5.5, §5.18, §6.4.
///
/// # Errors
///
/// A load refusal met at run; `NotBuilt` past what the engine builds.
pub(super) fn diff_cells(
    m: &Reality,
    (ask, diff): (&Ask, &Diff),
    engine: &mut Engine,
    (ctx, pinned): (Ctx, &Collected),
    only: Option<&str>,
) -> Result<Cells, Stop> {
    let axes = Axes::of(m, ask)?;
    let Some(fanned) = &axes else {
        let cell = cell(m, diff, engine, (ctx, pinned), (Vec::new(), only))?;
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
        let made = cell(m, diff, engine, (inside, pinned), (options, only));
        engine.end_layer(inside.layer, made.as_ref().err())?;
        each.push(made?);
    }
    Ok(Cells { axes, each, ctx })
}

/// The cells of a diff whose two worlds sit on two stacks of scenarios, from `ours`, the side
/// of the world compared, and `theirs`, the other's, each answered in the reality that holds
/// its stack: one cell per combination of the axes either side reads, each side's at the
/// options of the axes it reads, the register both sides' (refcore's `cross_stack_diff`).
/// L §5.5, §6.4.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a side with no cell at a combination's options.
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
        let left = cell_at(ours, &options)?;
        let right = cell_at(theirs, &options)?;
        let mut met = left.met.clone();
        met.merge(&right.met);
        each.push(Cell {
            options,
            ours: left.ours.clone(),
            theirs: right.theirs.clone(),
            met,
        });
    }
    Ok(Cells {
        axes,
        each,
        ctx: ours.ctx,
    })
}

/// The cell of side `side` at the options of `options` of the axes it reads.
fn cell_at<'a>(side: &'a Cells, options: &[(String, String)]) -> Result<&'a Cell, Stop> {
    let ids: BTreeSet<String> = side.axes.iter().flat_map(Axes::ids).collect();
    let wanted: Vec<(String, String)> = options
        .iter()
        .filter(|(axis, _)| ids.contains(axis))
        .cloned()
        .collect();
    side.each
        .iter()
        .find(|c| c.options == wanted)
        .ok_or_else(|| invariant("a diff's side with no cell at a combination"))
}

/// A diff's lines from its cells: its one result without a head when it reads no axis, else
/// one line per distinct result under a head naming its options, as a value's lines are
/// (refcore's `diff_print`). L §5.5, §5.18.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a cell with a side unevaluated.
pub(super) fn cells_lines(
    m: &Reality,
    (ask, diff): (&Ask, &Diff),
    cells: &Cells,
) -> Result<Vec<String>, Stop> {
    let mut results = Vec::with_capacity(cells.each.len());
    for (at, cell) in cells.each.iter().enumerate() {
        results.push((at, result(m, diff, cell)?));
    }
    if let Some(axes) = &cells.axes {
        let combos = axes.combinations()?;
        // the ask's pins are in each cell's register already
        let pins = Collected::default();
        return result_lines((axes, &combos), &ask.id, &results, (cells.ctx, &pins));
    }
    let [(_, only)] = results.as_slice() else {
        return Err(invariant(
            "a diff that reads no axis with other than one cell",
        ));
    };
    let mut out = only.lines(&ask.id);
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
    (ctx, pinned): (Ctx, &Collected),
    (options, only): (Vec<(String, String)>, Option<&str>),
) -> Result<Cell, Stop> {
    let mut met = pinned.clone();
    let mut sides = [None, None];
    for (slot, world) in sides.iter_mut().zip([&diff.world, &diff.against]) {
        if only.is_none_or(|w| w == world.as_str()) {
            *slot = Some(side(m, (diff, world), engine, (ctx, &mut met))?);
        }
    }
    let [ours, theirs] = sides;
    Ok(Cell {
        options,
        ours,
        theirs,
        met,
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
        met: cell.met.clone(),
        laws: None,
    })
}

/// The diff's value in world `world`, a line per binding tuple, in that world's timeline; what
/// it met, `world: <world>` first, into `met` (refcore's `answer_diff`, one world). L §5.18, §6.4.
fn side(
    m: &Reality,
    (diff, world): (&Diff, &str),
    engine: &mut Engine,
    (ctx, met): (Ctx, &mut Collected),
) -> Result<Side, Stop> {
    met.entries.insert(Entry {
        word: "world",
        text: world.to_owned(),
        figure: String::new(),
    });
    let inside = ctx.in_world(world_named(m, world)?);
    engine.enter_timeline(m, inside)?;
    let lines = side_lines(m, diff, engine, (inside, met));
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
    (ctx, met): (Ctx, &mut Collected),
) -> Result<Side, Stop> {
    let text = crate::load::expr_text(&diff.expr);
    if diff.binds.is_empty() {
        return Ok(vec![(
            text,
            one(m, diff, engine, (&Env::empty(), ctx), met)?,
        )]);
    }
    let tuples = match engine.tuples(m, &diff.binds, ctx)? {
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
