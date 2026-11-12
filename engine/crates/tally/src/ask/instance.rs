//! `explain` of a movement instance: its class, day, booking cut, legs, derivation and adjustments, a decision's walk point and reads.
//! Spec: E §10 (`explain` of a movement instance: its class, day, booking cut and leg
//! derivations; for a decision, its stratum and walk point; each later version with its δ,
//! the adjustment it made and what settles or waives it), §4.1 (an instance dated at the cut
//! K, its body evaluated at its booking cut min(t, K), its status), §4.2 step 3 (a decision
//! run at its point with running reads), §5 (adjustments, their settlements and remainders);
//! L §5.18 (the explain row), §5.14 (moves, their legs and attributes), §5.14.3 (adjustments).
//! Twin: engine/tools/refcore/ask.py (`evaluate`, its moveref arm, `explain_instance`,
//! `point_text`).
//! Invariants: the instance is the one the line's cut sees, a decision's as the main walk ran
//! it; a plan's derivation is traced again at its booking cut with an empty memo; a line's
//! register is its value's read alone, what the lines read brings nothing; adjustments print
//! in their order (day, move, instance, leg), a settlement's movements in the text order of
//! their ids.

use super::answer::unresolved_word;
use super::check::Printed;
use super::explain::{derivation, invariant, plain_text};
use super::walked::{quantity_text, status_text};
use crate::book::Adjustment;
use crate::eval::{Ctx, Engine, Env, Item};
use crate::model::{MoveInfo, MvSrc, Reality, Value};
use crate::syntax::{Ask, Expr, Stop};
use crate::time::Day;
use crate::walk::{Amount, Dated, End, Exists, Inst, LegVal, Run, Seed, Walk};

/// The line of `explain` of a movement instance: the movement as the line reads it, then its
/// class, day, booking cut and status, its legs and attributes, and for a plan its
/// derivation at its booking cut and its adjustments (refcore's `evaluate`, its moveref arm,
/// and `explain_instance`). L §5.18, E §10.
///
/// # Errors
///
/// A load refusal met at run; `NotBuilt` for a derivation whose causes the walk computes.
pub(super) fn instance_line(
    m: &Reality,
    (ask, reference): (&Ask, &Expr),
    engine: &mut Engine,
    env: &Env,
    ctx: Ctx,
) -> Result<Printed, Stop> {
    let (items, met) = engine.line(m, core::slice::from_ref(reference), env, ctx)?;
    // a bare key literal no position resolved is refused where it is read (P39)
    if let Some(place) = m.asks_unresolved.get(&ask.id) {
        unresolved_word(&items, place)?;
    }
    let Ok([item]) = <[Item; 1]>::try_from(items) else {
        return Err(invariant("an explained line with other than one value"));
    };
    let text = plain_text(m, &item)?;
    // what the lines meet outside the derivation, the adjustments measured among them, joins
    // the line's register, as refcore's line collects it (the derivation keeps its own)
    engine.push();
    let lines = instance_lines(m, engine, &item, ctx);
    let mut met = met;
    met.merge(&engine.pop_alone());
    Ok(Printed {
        text,
        body: lines?.into_iter().map(|l| format!("  {l}")).collect(),
        bare: false,
        met,
        laws: None,
    })
}

/// The lines of the instance `item` names; one line where the cut sees none (refcore's
/// `explain_instance`). E §10.
fn instance_lines(
    m: &Reality,
    engine: &mut Engine,
    item: &Item,
    ctx: Ctx,
) -> Result<Vec<String>, Stop> {
    let found = match item {
        Item::Value(Value::Mv(v)) => match &v.src {
            MvSrc::Inst(move_id, _) => {
                let mv = m
                    .moves
                    .get(move_id)
                    .ok_or_else(|| invariant("a movement of a move load didn't build"))?;
                found_inst(m, engine, mv, &v.id, ctx)?.map(|inst| (mv, inst))
            }
            MvSrc::Obs | MvSrc::Adj { .. } | MvSrc::Kind(_) | MvSrc::Missing => None,
        },
        Item::Value(_) | Item::Refused(_) => None,
    };
    let Some((mv, inst)) = found else {
        let what = match item {
            Item::Value(Value::Mv(v)) => v.id.clone(),
            Item::Value(_) | Item::Refused(_) => plain_text(m, item)?,
        };
        return Ok(vec![format!("({what}: no instance at the cut)")]);
    };
    let mut out = Vec::new();
    let run = if mv.decision {
        let main = engine.main_walk(m, ctx.cut)?;
        let walk = engine.walk(main)?;
        let run = walk
            .run_ids
            .get(&inst.id)
            .and_then(|i| walk.runs.get(*i))
            .cloned();
        out.extend(decision_head(m, walk, (mv, &inst), run.as_ref())?);
        run
    } else {
        out.push(format!(
            "plan, day {}, booking cut {}, status {}",
            day_text(inst.day.as_ref()),
            inst.cut.map_or_else(|| "None".to_owned(), Day::text),
            status_of(&inst)?
        ));
        None
    };
    let src = run.as_ref().map_or(&inst, |r| &r.inst);
    match &inst.exists {
        Exists::Yes => {}
        Exists::No => out.push("exists: false".to_owned()),
        Exists::Unknown(h) => out.push(format!("exists: {}", h.text())),
    }
    if let Some(err) = &src.err {
        out.push(format!("refused: {}", err.text()));
    }
    for lg in &src.legs {
        out.push(leg_text(m, lg, inst.day.as_ref())?);
    }
    let mut attrs: Vec<&(String, Value)> = src.attrs.iter().collect();
    attrs.sort_by(|a, b| a.0.cmp(&b.0));
    for (name, value) in attrs {
        out.push(format!(
            "attr {name} = {}",
            plain_text(m, &Item::Value(value.clone()))?
        ));
    }
    if mv.decision {
        return Ok(out);
    }
    if let Some(cut) = inst.cut {
        out.extend(booked_derivation(m, engine, (mv, &inst), cut, ctx)?);
    }
    for adj in engine.all_adjustments(m, ctx.cut)? {
        if adj.inst_id == inst.id {
            out.push(adjustment_text(m, &adj)?);
        }
    }
    Ok(out)
}

/// Move `mv`'s instance `id` at the line's cut; none where the cut sees none (refcore's
/// `find_inst`). E §4.1.
fn found_inst(
    m: &Reality,
    engine: &mut Engine,
    mv: &MoveInfo,
    id: &str,
    ctx: Ctx,
) -> Result<Option<Inst>, Stop> {
    let insts = engine.move_instances(m, mv, ctx.cut)?;
    Ok(insts.iter().find(|i| i.id == id).cloned())
}

/// A decision's head: its day and status, its stratum and walk point, or that it didn't run,
/// and the places it read there (refcore's `explain_instance`, its decision). E §10, §4.2
/// step 3.
fn decision_head(
    m: &Reality,
    walk: &Walk,
    (mv, inst): (&MoveInfo, &Inst),
    run: Option<&Run>,
) -> Result<Vec<String>, Stop> {
    let status = status_of(run.map_or(inst, |r| &r.inst))?;
    let mut out = vec![format!(
        "decision, day {}, status {status}",
        day_text(inst.day.as_ref())
    )];
    let Some(run) = run else {
        out.push(format!("stratum {}, not run", mv.stratum));
        return Ok(out);
    };
    out.push(format!(
        "stratum {}, walk point {}",
        mv.stratum,
        point_text(m, walk, run)
    ));
    if !run.read.is_empty() {
        let read: Vec<&str> = run.read.iter().map(String::as_str).collect();
        out.push(format!("read at its walk point: {}", read.join(", ")));
    }
    Ok(out)
}

/// A run's walk point as `(day, stratum s, act)`: the act its move's `move|n`, or in an
/// unordered cycle's run `first|n|move`, `first` the cycle's first move by id (refcore's
/// `point_text`, its points from `run_decisions` and `run_group`). E §4.2 step 3, L §5.14.2.
fn point_text(m: &Reality, walk: &Walk, run: &Run) -> String {
    let at = run.point;
    let mv = &run.inst.move_id;
    let cycle = m
        .graph
        .order_groups
        .iter()
        .find(|g| g.contains(mv))
        .and_then(|g| g.first())
        .filter(|_| walk.cycles_run.contains(&(mv.clone(), at.day)));
    let act = match cycle {
        Some(first) => format!("{first}|{:06}|{mv}", at.n),
        None => format!("{mv}|{:06}", at.n),
    };
    format!("({}, stratum {}, {act})", at.day.text(), at.stratum)
}

/// An instance's status word. E §4.1 step 3.
fn status_of(inst: &Inst) -> Result<&'static str, Stop> {
    status_text(inst.status).ok_or_else(|| invariant("an explained instance with no status"))
}

/// A day or a hole as `fmt_value` prints it; `-` for none, an instance refused before its
/// day. L §5.14.
fn day_text(day: Option<&Dated>) -> String {
    match day {
        Some(Dated::Day(d)) => d.text(),
        Some(Dated::Hole(h)) => h.text(),
        None => "-".to_owned(),
    }
}

/// A leg: its name, places and amount, and its own day where it isn't its instance's
/// (refcore's `explain_instance`, its legs). L §5.14 Legs.
fn leg_text(m: &Reality, lg: &LegVal, day: Option<&Dated>) -> Result<String, Stop> {
    let amount = match &lg.amount {
        Amount::Qty(q) => {
            let unit = lg
                .unit
                .as_deref()
                .ok_or_else(|| invariant("a leg's amount with no unit"))?;
            quantity_text(m, q, unit)?
        }
        Amount::Hole(h) => h.text(),
        Amount::Refused(r) => format!("refused: {}", r.text()),
    };
    // a leg of an instance refused before its day has no day of its own
    let on = match day {
        Some(d) if *d != lg.day => format!(" on {}", day_text(Some(&lg.day))),
        Some(_) | None => String::new(),
    };
    Ok(format!(
        "leg {} from {} to {} amount {amount}{on}",
        lg.name,
        end_text(&lg.from),
        end_text(&lg.to)
    ))
}

/// A leg's end: its place, or its hole. L §5.14 Legs.
fn end_text(end: &End) -> String {
    match end {
        End::Place(p) => p.clone(),
        End::Hole(h) => h.text(),
    }
}

/// A plan instance's derivation at its booking cut: the instance evaluated again there with
/// tracing on, its day at the line's cut, every cause numbered once (refcore's
/// `explain_instance`, its `eval_inst` traced at `at_cut`). E §10, §4.1 step 2.
///
/// # Errors
///
/// A load refusal met at run; `NotBuilt` for a cause the walk computes.
fn booked_derivation(
    m: &Reality,
    engine: &mut Engine,
    (mv, inst): (&MoveInfo, &Inst),
    cut: Day,
    ctx: Ctx,
) -> Result<Vec<String>, Stop> {
    let seed = Seed {
        env: inst.env.clone(),
        keys: inst.keys.clone(),
        exists: Exists::Yes,
        at_cut: Some(cut),
        upto: None,
    };
    let tctx = engine.timeline_ctx(None, ctx.cut);
    engine.begin_trace();
    let made = engine.eval_inst(m, mv, &seed, tctx);
    let trace = engine.end_trace()?;
    made?;
    if trace.walked {
        return Err(Stop::NotBuilt(
            "explain of an instance whose derivation reads what the walk computes",
        ));
    }
    let sub = derivation(m, &trace)?;
    if sub.is_empty() {
        return Ok(sub);
    }
    let mut out = vec![format!("derivation at {}:", cut.text())];
    out.extend(sub);
    Ok(out)
}

/// An adjustment of the instance: its change, day and versions, what settles or waives it
/// and what remains; a refused change with its refusal (refcore's `explain_instance`, its
/// adjustments). E §5 steps 3–5, L §5.14.3.
///
/// # Errors
///
/// A settlement's refusal; a number error.
fn adjustment_text(m: &Reality, adj: &Adjustment) -> Result<String, Stop> {
    let versions = adj.vids.join(", ");
    if let Amount::Refused(r) = &adj.delta {
        return Ok(format!(
            "adjustment {}: change refused: {} on {} (version {versions})",
            adj.id,
            r.text(),
            adj.day.text()
        ));
    }
    let how = if adj.waived {
        "waived".to_owned()
    } else {
        let mut by: Vec<String> = adj.settled.iter().map(|s| s.by.clone()).collect();
        by.sort();
        if by.is_empty() {
            "settled by nothing".to_owned()
        } else {
            format!("settled by {}", by.join(", "))
        }
    };
    let remaining = match adj.remainder()? {
        None => "none".to_owned(),
        Some(left) => adj.amount_text(m, &left)?,
    };
    Ok(format!(
        "adjustment {}: change {} on {} (version {versions}), {how}; remaining {remaining}",
        adj.id,
        adj.amount_text(m, &adj.delta)?,
        adj.day.text()
    ))
}
