//! Load: the parsed files become the reality, step by step in refcore's order, as far as M0 builds.
//! Spec: E §3.1 (steps 1–4); L §6.1 (one model header, one `tally`), §6.4 (a scenario over the
//! model or another scenario), §5.4 (units and currencies), §5.18 (asks), §8.5 (WARN-UNREAD
//! for a source).
//! Twin: engine/tools/refcore/loader.py (`Loader.build`, its pass 1, and `build_units`).
//! Invariants: each step reads only what earlier steps built; a form not built yet stops with
//! `NotBuilt`, never a guessed refusal; the load refusals made are the ones construction meets,
//! in refcore's words (the rest are M0's later groups').

use crate::base::{Code, Load, Place as At, Refusal, Run, Table, Warn};
use crate::model::{Graph, PERIODS, Reality, Scenarios};
use crate::num::{Num, NumError};
use crate::syntax::{File, HeaderKind, Lit, Section, Stop, UnitForm};
use std::collections::{BTreeMap, BTreeSet};

/// The reality being built, with the sections it is built from.
pub(super) struct Loader {
    /// The reality so far.
    pub(super) m: Reality,
    /// Every file's sections, in file order, each with its file's path.
    pub(super) sections: Vec<(String, Section)>,
    /// The currencies in the order declared.
    pub(super) currency_order: Vec<String>,
}

/// A load refusal at a line of a file. L §8.3.
pub(super) fn refuse(code: Load, message: String, file: &str, line: u32) -> Stop {
    Stop::Refused(Refusal::at(
        Code::Load(code),
        message,
        At::Line(file.to_owned(), line),
    ))
}

/// A number error met while loading: its code, as at a read (L §8.4).
pub(super) fn num_stop(error: NumError) -> Stop {
    let (code, message) = match error {
        NumError::Div => (Code::Run(Run::Div), "division by zero"),
        NumError::Digits => (Code::Run(Run::Digits), "a number past 4096 digits"),
        NumError::Domain => (Code::Run(Run::Domain), "outside its operation's domain"),
        NumError::Invariant => (Code::EngineInvariant, "the number kernel's invariant"),
    };
    Stop::Refused(Refusal::new(code, message.to_owned()))
}

/// The reality the files state (refcore's `Loader.build`, its constructive steps). E §3.1.
///
/// # Errors
///
/// A load refusal construction meets, or `NotBuilt` for a form S doesn't build.
pub fn load(files: Vec<File>) -> Result<Reality, Stop> {
    let mut loader = Loader::new(files)?;
    loader.register_builtins();
    loader.register_names()?;
    loader.build_units()?;
    loader.build_kinds()?;
    loader.build_indexes()?;
    loader.build_versions()?;
    loader.unread_sources();
    loader.build_entries()?;
    loader.build_decls()?;
    loader.prescan_open_keys();
    loader.build_places()?;
    loader.build_items()?;
    loader.check_rows()?;
    loader.finish_rows();
    loader.collect_asks();
    loader.build_graph()?;
    Ok(loader.m)
}

impl Loader {
    /// The one model header, the files' tally version, the scenarios, and an empty reality under
    /// them (refcore's `Loader.build`, pass 1). L §6.1, §6.4.
    fn new(files: Vec<File>) -> Result<Loader, Stop> {
        let models: Vec<&File> = files
            .iter()
            .filter(|f| f.header.kind == HeaderKind::Model)
            .collect();
        let [model] = models.as_slice() else {
            return Err(Stop::Refused(Refusal::new(
                Code::Load(Load::Header),
                format!(
                    "{} files have a model header; exactly one does",
                    models.len()
                ),
            )));
        };
        if files.iter().any(|f| f.header.tally != model.header.tally) {
            return Err(Stop::Refused(Refusal::new(
                Code::Load(Load::Version),
                "files declare different tally versions".to_owned(),
            )));
        }
        if !model.header.uses.is_empty() || !model.header.sames.is_empty() {
            return Err(Stop::NotBuilt("joint models"));
        }
        let scenarios = scenarios(&files, &model.header.id)?;
        let header = model.header.clone();
        let missing = Stop::NotBuilt("a model header without on, start or end");
        let start = header.start.ok_or(missing.clone())?;
        let end = header.end.ok_or(missing.clone())?;
        let on = header.on.ok_or(missing)?;
        let mut sections = Vec::new();
        for file in files {
            for section in file.sections {
                sections.push((file.path.clone(), section));
            }
        }
        Ok(Loader {
            m: Reality {
                header,
                start,
                end,
                on,
                names: BTreeMap::new(),
                sources: Table::new(),
                versions: Table::new(),
                units: BTreeSet::new(),
                currencies: BTreeMap::new(),
                unit_kind_keys: BTreeMap::new(),
                kinds: Table::new(),
                things: Table::new(),
                indexes: Table::new(),
                inputs: Table::new(),
                defs: Table::new(),
                bands: Table::new(),
                places: Table::new(),
                moves: Table::new(),
                asks: Vec::new(),
                warnings: Vec::new(),
                graph: Graph::default(),
                scenarios,
            },
            sections,
            currency_order: Vec::new(),
        })
    }

    /// Currencies with their minor units, and named units (refcore's `build_units`). L §5.4.
    fn build_units(&mut self) -> Result<(), Stop> {
        for (file, section) in &self.sections {
            let Section::Unit(unit) = section else {
                continue;
            };
            match &unit.form {
                UnitForm::Currency(minor) => {
                    let Some(value) = positive_plain(minor) else {
                        return Err(refuse(
                            Load::Unit,
                            format!("the minor unit of {} must be a positive number", unit.id),
                            file,
                            unit.line,
                        ));
                    };
                    self.m.currencies.insert(unit.id.clone(), value);
                    self.currency_order.push(unit.id.clone());
                }
                UnitForm::Named(_) => {
                    if PERIODS.contains(&unit.id.as_str()) {
                        return Err(refuse(
                            Load::Dup,
                            format!("{} is declared twice (a period word)", unit.id),
                            file,
                            unit.line,
                        ));
                    }
                    self.m.units.insert(unit.id.clone());
                }
            }
        }
        Ok(())
    }

    /// The asks, in section order. L §5.18.
    fn collect_asks(&mut self) {
        for (_, section) in &self.sections {
            if let Section::Ask(ask) = section {
                self.m.asks.push(ask.as_ref().clone());
            }
        }
    }

    /// `source s is read by nothing` for each source no version cites, in source order
    /// (refcore's `build_versions`, its end). L §8.5.
    fn unread_sources(&mut self) {
        for (id, _) in self.m.sources.entries() {
            let cited = self.m.versions.entries().any(|(_, v)| v.cite == *id);
            if !cited {
                self.m
                    .warnings
                    .push((Warn::Unread, format!("source {id} is read by nothing")));
            }
        }
    }
}

/// A number literal with no unit, above zero.
fn positive_plain(lit: &Lit) -> Option<Num> {
    let Lit::Num(n) = lit else {
        return None;
    };
    let positive = n.unit.is_none() && !n.value.is_negative() && !n.value.is_zero();
    positive.then(|| n.value.clone())
}

/// The scenarios the files declare (refcore's `Loader.build`, pass 1): each over the model
/// (`None`) or over another scenario of these files, with the sections its file holds. L §6.4.
fn scenarios(files: &[File], model: &str) -> Result<Scenarios, Stop> {
    let mut found = Scenarios::default();
    let mut declared = Vec::new();
    for file in files
        .iter()
        .filter(|f| f.header.kind == HeaderKind::Scenario)
    {
        let head = &file.header;
        if found.over.contains_key(&head.id) {
            let message = format!("scenario {} declared twice", head.id);
            return Err(refuse(Load::Scenario, message, &file.path, head.line));
        }
        let over = head.over.clone().filter(|over| over != model);
        found.over.insert(head.id.clone(), over);
        declared.push((head.id.as_str(), file.path.as_str(), head.line));
        for section in &file.sections {
            found
                .section
                .insert(section_id(section).0.to_owned(), head.id.clone());
        }
    }
    for (id, path, line) in declared {
        if let Some(Some(over)) = found.over.get(id)
            && !found.over.contains_key(over)
        {
            let message = format!("scenario {id} is over {over}, neither the model nor a scenario");
            return Err(refuse(Load::Scenario, message, path, line));
        }
    }
    Ok(found)
}

/// A section's id and line.
pub(super) fn section_id(section: &Section) -> (&str, u32) {
    match section {
        Section::Source(s) => (&s.id, s.line),
        Section::Version(s) => (&s.id, s.line),
        Section::Unit(s) => (&s.id, s.line),
        Section::Kind(s) => (&s.id, s.line),
        Section::Index(s) => (&s.id, s.line),
        Section::Place(s) => (&s.id, s.line),
        Section::Input(s) => (&s.id, s.line),
        Section::Band(s) => (&s.id, s.line),
        Section::Def(s) => (&s.id, s.line),
        Section::Move(s) => (&s.id, s.line),
        Section::Ask(s) => (&s.id, s.line),
        Section::Axis(s) => (&s.id, s.line),
        Section::Fill(s) => (&s.id, s.line),
    }
}
