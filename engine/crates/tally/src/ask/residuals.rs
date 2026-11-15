//! A `residuals` ask's lines: its head, each group of the residual ledger with its totals and its lines under it, per combination of the axes it reads, then the register.
//! Spec: L §5.18 (`residuals x span [by f]`), E §6.3 (the ledger, its groups and totals), L §5.5
//! (tally 2.1.7 R8: an ask of any form fans out over the axes it reads, each combination a
//! result, equal results under one head), §8.2 (the register), §8.4 (a run-time refusal
//! prints as the ask's line, or as its combination's), §8.1 (a place or span that is a hole
//! makes the ask line that hole).
//! Twin: engine/tools/refcore/ask.py (`answer_residuals`) and check.py (`_txt`, a span's text);
//! refcore doesn't fan a ledger out (its interpreter guard), so the fanned layout is a value's.
//! Invariants: the head names the place and the span as written, and ` by f` where the ask
//! groups by a field; each group prints its month and name (a field's value under `by f`),
//! then its model, observed and difference totals; every line sits under its group; at a
//! place of unit `any`, each unit's ledger prints under its own `ledger <unit>:` head, in the
//! order of the units' ids, its groups one step further in; the register is what reading the
//! ledger met, with the ask's own entries; a combination's layer is open only while its
//! ledger is read.

use super::axes::{Axes, result_lines};
use super::check::Printed;
use super::register::register_lines;
use crate::base::Code;
use crate::book::{Ledger, UnitLedger, fig_text};
use crate::eval::{Collected, Ctx, Engine};
use crate::model::Reality;
use crate::syntax::{Ask, Expr, Lit, Residuals, Span, Stop};

/// A `residuals` ask's lines (refcore's `answer_residuals`): the head, then each group's line
/// and its lines, then the register; over the axes it reads, one such result per distinct
/// ledger across their combinations, under a head naming its options. E §6.3, L §5.18, §5.5.
///
/// # Errors
///
/// The refusals the ledger meets, which the ask prints as its line; `NotBuilt` past what the
/// engine builds.
pub(super) fn residual_lines(
    m: &Reality,
    ask: &Ask,
    r: &Residuals,
    engine: &mut Engine,
    (ctx, pinned): (Ctx, &Collected),
) -> Result<Vec<String>, Stop> {
    let Some(axes) = Axes::of(m, ask)? else {
        let result = match ledger_result(m, r, engine, (ctx, pinned))? {
            Ok(result) => result,
            Err(hole) => return Ok(vec![format!("{}: {hole}", ask.id)]),
        };
        let mut out = result.lines(&ask.id);
        let register = register_lines(&result.met, ctx)?;
        out.extend(register.into_iter().map(|l| format!("  {l}")));
        return Ok(out);
    };
    let combos = axes.combinations()?;
    let mut results = Vec::with_capacity(combos.len());
    for (at, combo) in combos.iter().enumerate() {
        let inside = engine.with_options(ctx, &axes.options_of(combo)?)?;
        engine.open_layer(m, inside)?;
        let read = combination_result(m, r, engine, (inside, pinned));
        engine.end_layer(inside.layer, read.as_ref().err())?;
        results.push((at, read?));
    }
    // the ask's pins are in each result's register already
    let pins = Collected::default();
    result_lines((&axes, &combos), &ask.id, &results, (ctx, &pins))
}

/// The ledger of `r` in context `ctx` as a result: its head's text, its groups' lines under
/// it, what reading it met with the ask's pins `pinned`; or the text of the hole its place or
/// span is (refcore's `answer_residuals`). E §6.3, L §5.18, §8.1.
fn ledger_result(
    m: &Reality,
    r: &Residuals,
    engine: &mut Engine,
    (ctx, pinned): (Ctx, &Collected),
) -> Result<Result<Printed, String>, Stop> {
    engine.push();
    let ledger = engine.residuals(m, r, ctx);
    let mut met = engine.pop_alone();
    met.merge(pinned);
    let (place, per_unit, ledgers) = match ledger? {
        Ledger::Hole(h) => return Ok(Err(h.text())),
        Ledger::Groups {
            place,
            per_unit,
            ledgers,
        } => (place, per_unit, ledgers),
    };
    let mut body = Vec::new();
    for one in &ledgers {
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
        text: format!("residuals {place} {}{by}", span_text(&r.span)?),
        body,
        bare: false,
        whole: true,
        met,
        laws: None,
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

/// One combination's result: its ledger, or the hole of its place or span, or the run-time
/// refusal it met, each in its own line with what it read (L §5.5, §8.4).
fn combination_result(
    m: &Reality,
    r: &Residuals,
    engine: &mut Engine,
    (ctx, pinned): (Ctx, &Collected),
) -> Result<Printed, Stop> {
    let text = match ledger_result(m, r, engine, (ctx, pinned)) {
        Ok(Ok(result)) => return Ok(result),
        Ok(Err(hole)) => hole,
        Err(Stop::Refused(refusal)) if !matches!(refusal.code(), Code::Load(_)) => {
            format!("refused: {}", refusal.text())
        }
        Err(stop) => return Err(stop),
    };
    Ok(Printed {
        text,
        body: Vec::new(),
        bare: false,
        whole: true,
        met: pinned.clone(),
        laws: None,
    })
}

/// A span as the ask wrote it: its first end, then ` .. ` and its last when `..` was
/// written (refcore's `_txt` of a span). L §5.18.
fn span_text(sp: &Span) -> Result<String, Stop> {
    let first = match &sp.from {
        Some(e) => end_text(e)?,
        None => String::new(),
    };
    if !sp.dots {
        return Ok(first);
    }
    match &sp.to {
        Some(e) => Ok(format!("{first} .. {}", end_text(e)?)),
        None => Ok(format!("{first} ..")),
    }
}

/// A span's end as written: a name, or a day, a month, a quarter or a key (refcore's `_txt`).
/// L §5.18.
fn end_text(e: &Expr) -> Result<String, Stop> {
    if let Expr::Name(n) = e {
        return Ok(n.clone());
    }
    if let Expr::Lit(lit) = e
        && matches!(
            lit,
            Lit::Day(_) | Lit::Month(..) | Lit::Quarter(..) | Lit::Key(_) | Lit::Name(_)
        )
    {
        return Ok(lit.text());
    }
    Err(Stop::NotBuilt(
        "the text of a span's end other than a name, a day, a month or a quarter",
    ))
}
