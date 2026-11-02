//! A `residuals` ask's lines: its head, each group of the residual ledger with its totals and its lines under it, then the register.
//! Spec: L §5.18 (`residuals x span [by f]`), E §6.3 (the ledger, its groups and totals), L §8.2
//! (the register), §8.4 (a run-time refusal prints as the ask's line), §8.1 (a place or span
//! that is a hole makes the ask line that hole).
//! Twin: engine/tools/refcore/ask.py (`answer_residuals`) and check.py (`_txt`, a span's text).
//! Invariants: the head names the place and the span as written; each group prints its month
//! and name, then its model, observed and difference totals; every line sits under its group;
//! the register is what reading the ledger met, with the ask's own entries.

use super::register::register_lines;
use crate::book::{Ledger, fig_text};
use crate::eval::{Collected, Ctx, Engine};
use crate::model::Reality;
use crate::syntax::{Ask, Expr, Lit, Residuals, Span, Stop};

/// A `residuals` ask's lines (refcore's `answer_residuals`): the head, then each group's line
/// and its lines, then the register. E §6.3, L §5.18.
///
/// # Errors
///
/// The refusals the ledger meets, which the ask prints as its line; `NotBuilt` past what S
/// builds.
pub(super) fn residual_lines(
    m: &Reality,
    ask: &Ask,
    r: &Residuals,
    engine: &mut Engine,
    (ctx, pinned): (Ctx, &Collected),
) -> Result<Vec<String>, Stop> {
    engine.push();
    let ledger = engine.residuals(m, r, ctx);
    let mut met = engine.pop_alone();
    let (place, unit, groups) = match ledger? {
        Ledger::Hole(h) => return Ok(vec![format!("{}: {}", ask.id, h.text())]),
        Ledger::Groups {
            place,
            unit,
            groups,
        } => (place, unit, groups),
    };
    let mut out = vec![format!(
        "{}: residuals {place} {}",
        ask.id,
        span_text(&r.span)?
    )];
    for g in &groups {
        let name = match g.key.month {
            Some(month) => format!("{} {}", month.label(), g.key.name),
            None => g.key.name.clone(),
        };
        out.push(format!(
            "  {name}: model {}, observed {}, difference {}",
            fig_text(m, &g.model, &unit)?,
            fig_text(m, &g.observed, &unit)?,
            fig_text(m, &g.diff, &unit)?
        ));
        out.extend(g.lines.iter().map(|l| format!("    {}", l.text)));
    }
    met.merge(pinned);
    out.extend(
        register_lines(&met, ctx)?
            .into_iter()
            .map(|l| format!("  {l}")),
    );
    Ok(out)
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
