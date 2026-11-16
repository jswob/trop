//! `explain` of a walk-bound def: its cell at each walk point a decision read it, each explained as a def cell with its running reads.
//! Spec: E §10 (a walk-bound def: "the cell at each walk point where a decision read it, in
//! walk order, each with its point (day, stratum, movement, keys) and explained as a def cell
//! (above), its running reads as balances up to that point; with none, that nothing read it at
//! those keys"), §3.1 step 7 (a walk-bound def), §3.2 (its cells carry the walk point), §4.2
//! step 3 (a decision's running reads at its point); L §5.18 Explanation order ("A def
//! evaluated at walk points … is explained at each walk point that read it").
//! Twin: engine/tools/refcore/ask.py (`explain_walk_bound`); evaluate.py (`read_def`, its
//! `walk_cells`); mreads.py (`e_bal`, its running read's trace node).
//! Invariants: the cells are those the line's main walk kept as its decisions read them, in
//! the walk's order; each is traced again just before its decision booked, with a memo and a
//! register of its own, which bring nothing to the line's register and keep nothing in the
//! walk's cells.

use super::check::Printed;
use super::explain::{derivation, invariant, plain_text};
use crate::eval::{Collected, Ctx, Engine, Env, Item};
use crate::model::Reality;
use crate::read::origin;
use crate::syntax::{Expr, Key, Postfix, Stop};
use crate::walk::{At, Walk, WalkAt};

/// The line of `explain` of a walk-bound def at its keys: how many walk points read it, then
/// each cell with its point and its derivation there; a hole key is the line's value (refcore's
/// `explain_walk_bound`). E §10, L §5.18.
///
/// # Errors
///
/// A load refusal met at run; the main walk's refusal; `NotBuilt` for what the walk doesn't
/// build.
pub(super) fn walk_bound_line(
    m: &Reality,
    reference: &Expr,
    engine: &mut Engine,
    env: &Env,
    ctx: Ctx,
) -> Result<Printed, Stop> {
    let (id, written) = named(reference)?;
    let def = m
        .defs
        .get(id)
        .ok_or_else(|| invariant("an explained def load didn't build"))?;
    let keys = match engine.read_keys(m, (id, &def.binds), written, env, ctx)? {
        Ok(keys) => keys,
        Err(hole) => {
            let text = plain_text(m, &Item::Value(crate::model::Value::Hole(hole)))?;
            return Ok(printed(text, Vec::new()));
        }
    };
    // the walk's own register entries are its, not this line's
    engine.push();
    let main = engine.main_walk(m, ctx.cut);
    engine.pop_alone();
    let main = main?;
    let figure = origin(id, &keys);
    let read = engine.walk_cells.get(&(main, figure.clone())).cloned();
    let Some(cells) = read.filter(|cells| !cells.is_empty()) else {
        let line = format!("  nothing read {figure} at a walk point");
        return Ok(printed(
            "walk-bound, read at no walk point".to_owned(),
            vec![line],
        ));
    };
    let mut body = Vec::new();
    for (point, cell) in &cells {
        let value = match cell {
            Ok(v) => Item::Value(v.clone()),
            Err(r) => Item::Refused(r.clone()),
        };
        body.push(format!(
            "  {figure} at ({}, stratum {}, {}): {}",
            point.day.text(),
            point.stratum,
            reader(engine.walk(main)?, *point),
            plain_text(m, &value)?
        ));
        // the state just before the reader's own booking at its point, as the decision read
        // it: its law date, its own earlier observations left out (E §10: "its running reads
        // as balances up to that point"; "the traced value equals the stored one")
        let run = engine
            .walk(main)?
            .runs
            .iter()
            .find(|r| r.point == *point)
            .cloned();
        let law = run.as_ref().map_or(ctx.law, |r| r.inst.law.first());
        let there = Ctx {
            at: Some(WalkAt {
                walk: main,
                point: point.just_before(),
            }),
            ..engine.timeline_ctx(law, point.day.min(ctx.cut))
        };
        let prev = match &run {
            Some(run) => Some(engine.enter_run(m, main, run)?),
            None => None,
        };
        let lines = traced_at(m, engine, reference, env, there);
        if let Some(prev) = prev {
            engine.leave_run(main, prev)?;
        }
        for line in lines? {
            body.push(format!("    {line}"));
        }
    }
    let n = cells.len();
    let head = format!(
        "walk-bound, read at {n} walk point{}",
        if n == 1 { "" } else { "s" }
    );
    Ok(printed(head, body))
}

/// The derivation of `reference` read again at walk point `there`: traced with a memo and a
/// register of its own, kept nowhere (refcore's `explain_walk_bound`, its loop). E §10.
fn traced_at(
    m: &Reality,
    engine: &mut Engine,
    reference: &Expr,
    env: &Env,
    there: Ctx,
) -> Result<Vec<String>, Stop> {
    engine.rereading = true;
    engine.push();
    engine.begin_trace();
    let read = engine.line(m, core::slice::from_ref(reference), env, there);
    let trace = engine.end_trace();
    engine.pop_alone();
    engine.rereading = false;
    let trace = trace?;
    read?;
    derivation(m, &trace)
}

/// The decision that read a cell at `point`: the instance the walk ran there, else its move
/// (refcore's `byp`, else `dec_head`). E §10, §4.2 step 3.
fn reader(walk: &Walk, point: At) -> String {
    let at_act = |r: &&crate::walk::Run| {
        (r.point.day, r.point.stratum, r.point.rank) == (point.day, point.stratum, point.rank)
    };
    if let Some(run) = walk.runs.iter().find(|r| r.point == point) {
        return run.inst.id.clone();
    }
    walk.runs
        .iter()
        .find(at_act)
        .map_or_else(|| point.text(), |r| r.inst.move_id.clone())
}

/// The def a reference names and its keys as written: a name alone, or at keys (refcore's
/// `a.ref`). E §10.
fn named(reference: &Expr) -> Result<(&str, &[Key]), Stop> {
    if let Expr::Name(id) = reference {
        return Ok((id, &[]));
    }
    if let Expr::Post(base, post) = reference
        && let (Expr::Name(id), [Postfix::Keys(keys)]) = (base.as_ref(), post.as_slice())
    {
        return Ok((id, keys));
    }
    Err(invariant("a walk-bound reference that names no def"))
}

/// A line's head and the lines under it, with no register of its own.
fn printed(text: String, body: Vec<String>) -> Printed {
    Printed {
        text,
        body,
        bare: false,
        whole: false,
        met: Collected::default(),
        laws: None,
    }
}
