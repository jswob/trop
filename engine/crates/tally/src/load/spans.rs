//! Load's spans and versions: a written span of days or periods, and each version's dates and source.
//! Spec: L §4.5 (`known`, a day or a window, after `on` refused), §4.6 (governs spans), §5.3
//! (versions: `cite`, `governs`, `known`, `enters`, `assumed`, `moves`); E §3.1 step 3.
//! Twin: engine/tools/refcore/model.py (`lit_period`, `lspan_to_span`, `VersionInfo`);
//! loader.py (`build_versions`).
//! Invariants: a span's end is the first day after it; a lone date or period is the span of
//! its days; every version cites a source and is known on or before `on`, a window's last day
//! too; a `moves` line moves a version known before its own, and no two lines known one day
//! move one version.

use super::check::invariant;
use super::keys::lit_text;
use super::steps::{Loader, refuse};
use crate::base::Load;
use crate::model::{Moved, NameKind, Span, VersionInfo};
use crate::syntax::{Expr, Governs, Known, LSpan, Lit, Section, Stop};
use crate::time::Period;
use std::collections::BTreeSet;

/// A written date or period as a period: a day, a month, a quarter, or a four-digit year
/// (refcore's `lit_period`). L §2.1, §4.6.
pub(super) fn lit_period(lit: &Lit, file: &str, line: u32) -> Result<Period, Stop> {
    let period = match lit {
        Lit::Day(day) => Period::day(*day),
        Lit::Month(year, month) => Period::month(*year, *month),
        Lit::Quarter(year, quarter) => Period::quarter(*year, *quarter),
        Lit::Num(n) if n.whole && n.unit.is_none() && n.four => n
            .value
            .to_i64()
            .and_then(|v| u16::try_from(v).ok())
            .and_then(Period::year),
        Lit::Num(_)
        | Lit::MonthDay(..)
        | Lit::Key(_)
        | Lit::Cur(_)
        | Lit::Bool(_)
        | Lit::Inf { .. }
        | Lit::Nominal(_)
        | Lit::Name(_) => {
            return Err(refuse(
                Load::Parse,
                format!(
                    "cannot parse: expected a date or a period, got {}",
                    lit_text(lit)
                ),
                file,
                line,
            ));
        }
    };
    period.ok_or(Stop::NotBuilt("a period past 9999-12-31"))
}

/// A written span: `a..b`, `a..`, `..b`, one period, or a lifespan's open hole end
/// (refcore's `lspan_to_span`). L §4.6, §5.7.
pub(super) fn lspan_to_span(span: &LSpan, file: &str, line: u32) -> Result<Span, Stop> {
    let from = span
        .from
        .as_ref()
        .map(|lit| lit_period(lit, file, line))
        .transpose()?;
    if !span.dots {
        let Some(one) = from else {
            return Err(invariant(
                "unreachable: the parser reads a span with no `..` as a date or a period"
                    .to_owned(),
            ));
        };
        return Ok(Span::between(Some(one.start()), Some(one.end())));
    }
    if let Some(question) = &span.to_hole {
        return Ok(Span::open_ended(from.map(Period::start), question.clone()));
    }
    let to = span
        .to
        .as_ref()
        .map(|lit| lit_period(lit, file, line))
        .transpose()?;
    Ok(Span::between(from.map(Period::start), to.map(Period::end)))
}

/// A `governs` clause as a span.
pub(super) fn governs_span(governs: &Governs, file: &str, line: u32) -> Result<Span, Stop> {
    match governs {
        Governs::Always => Ok(Span::always()),
        Governs::Span(span) => lspan_to_span(span, file, line),
    }
}

impl Loader {
    /// Each version: its governs span, its source, its known date or window and its entry
    /// date; then each `moves` line, on the version it moves (refcore's `build_versions`).
    /// L §4.5, §5.3.
    pub(super) fn build_versions(&mut self) -> Result<(), Stop> {
        let mut moves = Vec::new();
        for (file, section) in &self.sections {
            let Section::Version(version) = section else {
                continue;
            };
            let id = &version.id;
            let line = version.line;
            let governs = version
                .governs
                .as_ref()
                .map(|g| governs_span(g, file, line))
                .transpose()?;
            if self.m.names.kind(&version.cite) != Some(NameKind::Source) {
                return Err(refuse(
                    Load::Name,
                    format!("unknown id {} (version {id} cites no source)", version.cite),
                    file,
                    line,
                )
                .naming(&version.cite));
            }
            let (known, window) = match version.known {
                Known::Day(day) => (day, None),
                Known::Window(first, last) => (first, Some(last)),
            };
            if window.is_some_and(|last| known > last) {
                return Err(refuse(
                    Load::Domain,
                    "window(a, b) with a after b (L §8.4)".to_owned(),
                    file,
                    line,
                ));
            }
            if window.unwrap_or(known) > self.m.on || known > self.m.on {
                return Err(refuse(
                    Load::Known,
                    format!("version {id} is known after the model's on date"),
                    file,
                    line,
                ));
            }
            let info = VersionInfo {
                id: id.clone(),
                cite: version.cite.clone(),
                governs,
                known,
                window,
                moved_by: Vec::new(),
                enters: version.enters,
                assumed: version.assumed.clone(),
                covers: Vec::new(),
                gap_roles: Vec::new(),
                file: file.clone(),
                line,
            };
            self.m.versions.push(&mut self.m.symbols, id, info)?;
            for (target, span) in &version.moves {
                moves.push((id.clone(), target.clone(), span.clone(), file.clone(), line));
            }
        }
        for (by, target, span, file, line) in moves {
            self.moves_line(&by, &target, &span, &file, line)?;
        }
        self.moves_known_apart()
    }

    /// One `moves V to SPAN` line of version `by`: V is a version known before `by`, named by
    /// its id; any other reference names no version, printed as written (refcore's
    /// `build_versions`, its moves; issue #88). L §5.3, §8.3 `LOAD-MOVES`.
    fn moves_line(
        &mut self,
        by: &str,
        target: &Expr,
        span: &LSpan,
        file: &str,
        line: u32,
    ) -> Result<(), Stop> {
        let Expr::Name(name) = target else {
            return Err(refuse(
                Load::Moves,
                format!(
                    "moves {}: unknown, or not known before this version",
                    super::texts::text(target)
                ),
                file,
                line,
            ));
        };
        let (known, window) = self
            .m
            .versions
            .get(by)
            .map(|v| (v.known, v.window))
            .ok_or_else(|| invariant(format!("version {by} isn't built")))?;
        let earlier = self.m.versions.get(name).is_some_and(|v| v.known < known);
        if !earlier {
            return Err(refuse(
                Load::Moves,
                format!("moves {name}: unknown, or not known before this version"),
                file,
                line,
            ));
        }
        let span = lspan_to_span(span, file, line)?;
        let moved = self
            .m
            .versions
            .get_mut(name)
            .ok_or_else(|| invariant(format!("version {name} isn't built")))?;
        moved.moved_by.push(Moved {
            by: by.to_owned(),
            known,
            window,
            span,
        });
        Ok(())
    }

    /// No version is moved by two lines known the same day (refcore's `build_versions`, its
    /// end). L §5.3.
    fn moves_known_apart(&self) -> Result<(), Stop> {
        for (id, version) in self.m.versions.entries() {
            let mut seen = BTreeSet::new();
            for line in &version.moved_by {
                if seen.insert(line.known) {
                    continue;
                }
                let mover = self
                    .m
                    .versions
                    .get(&line.by)
                    .ok_or_else(|| invariant(format!("version {} isn't built", line.by)))?;
                return Err(refuse(
                    Load::Moves,
                    format!("two moves of {id} known the same day"),
                    &mover.file,
                    mover.line,
                ));
            }
        }
        Ok(())
    }
}
