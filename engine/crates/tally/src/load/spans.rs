//! Load's spans and versions: a written span of days or periods, and each version's dates and source.
//! Spec: L §4.5 (`known`, after `on` refused), §4.6 (governs spans), §5.3 (versions: `cite`,
//! `governs`, `known`, `enters`, `assumed`); E §3.1 step 3.
//! Twin: engine/tools/refcore/model.py (`lit_period`, `lspan_to_span`, `VersionInfo`);
//! loader.py (`build_versions`).
//! Invariants: a span's end is the first day after it; a lone date or period is the span of
//! its days; every version cites a source and is known on or before `on`.

use super::keys::lit_text;
use super::steps::{Loader, refuse};
use crate::base::Load;
use crate::model::{NameKind, Span, VersionInfo};
use crate::syntax::{Governs, Known, LSpan, Lit, Section, Stop};
use crate::time::Period;

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
            return Err(Stop::NotBuilt("a span with no day"));
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
    /// Each version: its source, its governs span, its known and entry dates (refcore's
    /// `build_versions`). L §4.5, §5.3.
    pub(super) fn build_versions(&mut self) -> Result<(), Stop> {
        for (file, section) in &self.sections {
            let Section::Version(version) = section else {
                continue;
            };
            let id = &version.id;
            if self.m.names.get(&version.cite) != Some(&NameKind::Source) {
                return Err(refuse(
                    Load::Name,
                    format!("unknown id {} (version {id} cites no source)", version.cite),
                    file,
                    version.line,
                ));
            }
            let governs = version
                .governs
                .as_ref()
                .map(|g| governs_span(g, file, version.line))
                .transpose()?;
            let known = match version.known {
                Known::Day(day) => day,
                Known::Window(..) => return Err(Stop::NotBuilt("a known window")),
            };
            if known > self.m.on {
                return Err(refuse(
                    Load::Known,
                    format!("version {id} is known after the model's on date"),
                    file,
                    version.line,
                ));
            }
            let info = VersionInfo {
                id: id.clone(),
                cite: version.cite.clone(),
                governs,
                known,
                enters: version.enters,
                assumed: version.assumed.clone(),
                covers: Vec::new(),
                gap_roles: Vec::new(),
                file: file.clone(),
                line: version.line,
            };
            self.m.versions.push(id, info);
        }
        Ok(())
    }
}
