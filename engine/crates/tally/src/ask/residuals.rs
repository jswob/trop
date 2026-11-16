//! A `residuals` ask's result: its head, each group of the residual ledger with its totals and its lines under it, then the register; the ask's loop fans it out.
//! Spec: L §5.18 (`residuals x span [by f]`), E §6.3 (the ledger, its groups and totals), L §5.5
//! (tally 2.1.7 R8: an ask of any form fans out over the axes it reads, each combination a
//! result, equal results under one head), §8.2 (the register), §8.4 (a run-time refusal
//! prints as its tuple's or combination's line, alone), §8.1 (a place or span that is a hole
//! makes that line the hole, alone; tally 2.1.8 R13.1: a result is all the form prints for
//! one binding tuple, one combination, one window).
//! Twin: engine/tools/refcore/ask.py (`answer_residuals`, `one`) and check.py (`_txt`, a
//! span's text).
//! Invariants: the head names the place and the span as written, and ` by f` where the ask
//! groups by a field; each group prints its month and name (a field's value under `by f`),
//! then its model, observed and difference totals; every line sits under its group; at a
//! place of unit `any`, each unit's ledger prints under its own `ledger <unit>:` head, in the
//! order of the units' ids, its groups one step further in; the register is what reading the
//! ledger met, with the ask's own entries; the ask's loop (`answer.rs` `tuple_lines`) opens a
//! combination's layer only while its ledger is read.

use super::check::Printed;
use crate::base::Code;
use crate::book::{Ledger, UnitLedger, fig_text};
use crate::eval::{Collected, Ctx, Engine, Env};
use crate::hole::Hole;
use crate::model::Reality;
use crate::syntax::{Expr, Lit, Residuals, Span, Stop};

/// A `residuals` ask's result in context `ctx`, for one binding tuple and one combination of
/// the axes it reads (refcore's `answer_residuals`): the head, then each group's line and its
/// lines, its register with the ask's pins `pinned`; or the hole its place or span is, or the
/// run-time refusal its ledger met, that line alone with no register (tally 2.1.8 R13.1;
/// refcore's `one`). E §6.3, L §5.18, §5.5, §8.4.
///
/// # Errors
///
/// A load refusal met at run; `NotBuilt` past what the engine builds.
pub(super) fn residual_result(
    m: &Reality,
    r: &Residuals,
    engine: &mut Engine,
    (env, ctx, pinned): (&Env, Ctx, &Collected),
) -> Result<Printed, Stop> {
    let (text, carried) = match ledger_result(m, r, engine, (env, ctx, pinned)) {
        Ok(Ok(result)) => return Ok(result),
        Ok(Err(hole)) => (hole.text(), hole.carried().to_vec()),
        Err(Stop::Refused(refusal)) if !matches!(refusal.code(), Code::Load(_)) => {
            (format!("refused: {}", refusal.text()), Vec::new())
        }
        Err(stop) => return Err(stop),
    };
    Ok(Printed {
        text,
        body: Vec::new(),
        bare: false,
        whole: true,
        alone: true,
        met: Collected::default(),
        laws: None,
        carried,
        causes: None,
    })
}

/// The ledger of `r` in context `ctx` as a result: its head's text, its groups' lines under
/// it, what reading it met with the ask's pins `pinned`; or the text of the hole its place or
/// span is (refcore's `answer_residuals`). E §6.3, L §5.18, §8.1.
fn ledger_result(
    m: &Reality,
    r: &Residuals,
    engine: &mut Engine,
    (env, ctx, pinned): (&Env, Ctx, &Collected),
) -> Result<Result<Printed, Hole>, Stop> {
    engine.push();
    let ledger = engine.residuals(m, r, (env, ctx));
    let mut met = engine.pop_alone();
    met.merge(pinned);
    let (place, per_unit, ledgers) = match ledger? {
        Ledger::Hole(h) => return Ok(Err(h)),
        Ledger::Groups {
            place,
            per_unit,
            ledgers,
        } => (place, per_unit, ledgers),
    };
    let mut body = Vec::new();
    let mut carried = Vec::new();
    for one in &ledgers {
        // the unknown days its figures and days carry (refcore's `answer_residuals`, its
        // `vals`; tally 2.1.8 R13)
        for g in &one.groups {
            for fig in [&g.model, &g.observed, &g.diff] {
                fig.carried_into(&mut carried);
            }
            for line in &g.lines {
                line.carried_into(&mut carried);
            }
        }
        // a place of unit `any`: each unit's ledger under its own head (E §6.3)
        let indent = if per_unit {
            body.push(format!("  ledger {}:", one.unit));
            "    "
        } else {
            "  "
        };
        body.extend(group_lines(m, one, indent)?);
    }
    let by = r.by.as_ref().map_or(String::new(), |f| format!(" by {f}"));
    Ok(Ok(Printed {
        text: format!("residuals {place} {}{by}", span_text(&r.span)),
        body,
        bare: false,
        whole: true,
        alone: false,
        met,
        laws: None,
        carried,
        causes: None,
    }))
}

/// One ledger's lines, each indented by `indent`: each group's month and name with its totals,
/// then its lines under it (refcore's `answer_residuals`, its groups). E §6.3 Totals.
fn group_lines(m: &Reality, ledger: &UnitLedger, indent: &str) -> Result<Vec<String>, Stop> {
    let unit = &ledger.unit;
    let mut out = Vec::new();
    for g in &ledger.groups {
        let name = match g.key.month {
            Some(month) => format!("{} {}", month.label(), g.key.name),
            None => g.key.name.clone(),
        };
        out.push(format!(
            "{indent}{name}: model {}, observed {}, difference {}",
            fig_text(m, &g.model, unit)?,
            fig_text(m, &g.observed, unit)?,
            fig_text(m, &g.diff, unit)?
        ));
        out.extend(g.lines.iter().map(|l| format!("{indent}  {}", l.text)));
    }
    Ok(out)
}

/// A span as the ask wrote it: its first end, then ` .. ` and its last when `..` was
/// written (refcore's `_txt` of a span). L §5.18.
fn span_text(sp: &Span) -> String {
    let first = match &sp.from {
        Some(e) => end_text(e),
        None => String::new(),
    };
    if !sp.dots {
        return first;
    }
    match &sp.to {
        Some(e) => format!("{first} .. {}", end_text(e)),
        None => format!("{first} .."),
    }
}

/// A span's end as written: a name, a day, a month, a quarter, a key, or any other
/// expression's text, `end_of(2026-03)` (refcore's `_txt`). L §5.18.
fn end_text(e: &Expr) -> String {
    if let Expr::Name(n) = e {
        return n.clone();
    }
    if let Expr::Lit(lit) = e
        && matches!(
            lit,
            Lit::Day(_) | Lit::Month(..) | Lit::Quarter(..) | Lit::Key(_) | Lit::Name(_)
        )
    {
        return lit.text();
    }
    crate::load::expr_text(e)
}
