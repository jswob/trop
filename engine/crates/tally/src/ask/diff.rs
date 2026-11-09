//! A `diff` ask's lines: its value in two worlds, key by key changed, added, removed or unchanged, then the register of both.
//! Spec: L §5.18 (`diff(e [for binds]) in S against B`: e evaluated in world S and in world B,
//! keyed by the binding tuple, a diff line in the world where its `where` is a hole printed as
//! that hole), §6.4 (asking a scenario: a world below the reality's own hides the layers over
//! it), §8.2 (the register: `world: s` for each world evaluated), §8.4 (a run-time refusal
//! prints in its value's place).
//! Twin: engine/tools/refcore/ask.py (`answer_diff`, `diff_combine`, `rkey`, `fmt_value`,
//! `fmt_bind`) and check.py (`_txt`, the key of a diff without bindings).
//! Invariants: each world is evaluated in its own timeline, entered and left around it, in the
//! order written (S, then B); a key is its tuple's variables, or the expression's text with no
//! bindings; S's lines come in their order, then B's keys that S lacks; two values are
//! unchanged only when they are the same value, a hole or a refusal included; the register is
//! the ask's pins' and both evaluations' entries.

use std::collections::BTreeMap;

use super::answer::{item_text, tuple_key};
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

/// A `diff` ask's lines (refcore's `answer_diff`, `diff_combine`): the head with the counts,
/// a line per key changed, added or removed, then the register of both worlds. L §5.18, §6.4.
///
/// # Errors
///
/// A load refusal met at run; `NotBuilt` for a world of another stack of scenarios and past
/// what M1 builds.
pub(super) fn diff_lines(
    m: &Reality,
    (ask, diff): (&Ask, &Diff),
    engine: &mut Engine,
    (ctx, pinned): (Ctx, &Collected),
) -> Result<Vec<String>, Stop> {
    let mut met = pinned.clone();
    let ours = side(m, (diff, &diff.world), engine, (ctx, &mut met))?;
    let theirs = side(m, (diff, &diff.against), engine, (ctx, &mut met))?;
    // a key met twice in a world keeps its last value, as a dict of the lines does
    let ours_by: BTreeMap<&str, &Item> = ours.iter().map(|(k, v)| (k.as_str(), v)).collect();
    let theirs_by: BTreeMap<&str, &Item> = theirs.iter().map(|(k, v)| (k.as_str(), v)).collect();
    let mut counts = Counts::default();
    let mut lines = Vec::new();
    for (key, v) in &ours {
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
    for (key, v) in &theirs {
        if !ours_by.contains_key(key.as_str()) {
            counts.removed = one_more(counts.removed)?;
            lines.push(format!("  removed {key}: {}", item_text(m, v)?));
        }
    }
    let mut out = vec![format!(
        "{}: diff in {} against {}: {} changed, {} added, {} removed, {} unchanged",
        ask.id,
        diff.world,
        diff.against,
        counts.changed,
        counts.added,
        counts.removed,
        counts.unchanged
    )];
    out.extend(lines);
    out.extend(
        register_lines(&met, ctx)?
            .into_iter()
            .map(|l| format!("  {l}")),
    );
    Ok(out)
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
