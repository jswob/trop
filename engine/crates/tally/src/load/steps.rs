//! Load: the parsed files become the reality, step by step in refcore's order, as far as M0 builds.
//! Spec: E §3.1 (steps 1–4); L §6.1 (one model header, one `tally`), §6.4 (a scenario over the
//! model or another scenario), §5.4 (units and currencies), §5.18 (asks), §8.5 (WARN-UNREAD
//! for a source).
//! Twin: engine/tools/refcore/loader.py (`Loader.build`, its pass 1, and `build_units`).
//! Invariants: each step reads only what earlier steps built; a form not built yet stops with
//! `NotBuilt`, never a guessed refusal; the load refusals made are the ones construction meets,
//! in refcore's words (the rest are M0's later groups').

use super::same::SameLine;
use crate::base::{Code, Load, Place as At, Refusal, Run, Table, Warn};
use crate::model::{Graph, Joint, PERIODS, Reality, Scenarios, UnitError, power_past_limit};
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
    /// The `same` lines of rows and movements, the header's then the versions' (refcore's
    /// `header.sames` and `sames`).
    pub(super) same_lines: Vec<SameLine>,
}

/// A load refusal at a line of a file. L §8.3.
pub(super) fn refuse(code: Load, message: String, file: &str, line: u32) -> Stop {
    Stop::Refused(Refusal::at(
        Code::Load(code),
        message,
        At::Line(file.to_owned(), line),
    ))
}

/// A unit operation's error met while loading a row: a power past 64 in size is `LOAD-LIMIT
/// unit <u> has a power past 64`, placed by the caller (refcore's `at`); a number error is
/// `num_stop`'s. L §7.6 item 4, §8.3 `LOAD-LIMIT`.
pub(super) fn unit_num_stop(error: UnitError) -> Stop {
    match error {
        UnitError::Power(base) => Stop::Refused(Refusal::new(
            Code::Load(Load::Limit),
            power_past_limit(&base),
        )),
        UnitError::Num(error) => num_stop(error),
    }
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
    load_among(files, &BTreeSet::new(), &BTreeMap::new(), None)
}

/// The reality the files state, loaded among the run's other worlds `worlds` (the scenarios
/// of a run with scenarios, which an ask of the base may name) (refcore's `load_texts` with
/// its `worlds`); `elsewhere`, for the base of a run with scenarios, the ids their files
/// declare, each with its scenario; `joint`, for a joint model's merged files, its members (the
/// asks a run may name `a.q`). E §3.1; L §6.1, §6.4.
///
/// # Errors
///
/// As [`load`].
pub(super) fn load_among(
    files: Vec<File>,
    worlds: &BTreeSet<String>,
    elsewhere: &BTreeMap<String, String>,
    joint: Option<&Joint>,
) -> Result<Reality, Stop> {
    let mut loader = Loader::new(files, joint)?;
    loader.m.all_worlds.clone_from(worlds);
    loader.m.elsewhere.clone_from(elsewhere);
    loader
        .m
        .all_worlds
        .extend(loader.m.scenarios.over.keys().cloned());
    loader.m.all_worlds.insert(loader.m.header.id.clone());
    loader.register_builtins();
    loader.register_names()?;
    loader.build_units()?;
    loader.build_kinds()?;
    loader.build_indexes()?;
    loader.build_axes()?;
    loader.build_versions()?;
    loader.unread_sources();
    loader.build_entries()?;
    loader.build_decls()?;
    loader.prescan_open_keys();
    loader.build_places()?;
    loader.joint_place_sames()?;
    loader.check_key_ids()?;
    loader.build_items()?;
    loader.check_rows()?;
    loader.finish_rows();
    super::overlap::check_obs(&loader.m)?;
    loader.check_sames()?;
    super::scenarios::check_scenarios(&loader.m)?;
    super::bands::check_bands(&loader.m)?;
    loader.collect_asks();
    loader.build_graph()?;
    Ok(loader.m)
}

impl Loader {
    /// The one model header, the files' tally version, the scenarios, and an empty reality under
    /// them, a joint model's header `same` lines first among the `same` lines (refcore's
    /// `Loader.build`, pass 1, and `joint_sames`' header pairs). L §6.1, §6.4.
    fn new(files: Vec<File>, joint: Option<&Joint>) -> Result<Loader, Stop> {
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
        let is_joint = !model.header.uses.is_empty() || !model.header.sames.is_empty();
        if is_joint && joint.is_none() {
            return Err(Stop::NotBuilt(
                "a joint model's files loaded without its members",
            ));
        }
        let header_sames: Vec<SameLine> = model
            .header
            .sames
            .iter()
            .map(|(left, right)| SameLine {
                left: left.clone(),
                right: right.clone(),
                file: model.path.clone(),
                line: model.header.line,
                version: None,
            })
            .collect();
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
                axes: Table::new(),
                fills: Table::new(),
                all_worlds: BTreeSet::new(),
                elsewhere: BTreeMap::new(),
                asks_alone: BTreeMap::new(),
                sames: Vec::new(),
                joint: joint.cloned(),
            },
            sections,
            currency_order: Vec::new(),
            same_lines: header_sames,
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

    /// The axes, each with its options; a key entered twice is `LOAD-DUP` (refcore's
    /// `build_axes`). L §5.5.
    fn build_axes(&mut self) -> Result<(), Stop> {
        for (file, section) in &self.sections {
            let Section::Axis(axis) = section else {
                continue;
            };
            let mut seen = BTreeSet::new();
            if !axis
                .options
                .iter()
                .all(|(key, _)| seen.insert(key.as_str()))
            {
                let message = format!("a key entered twice in axis {}", axis.id);
                return Err(refuse(Load::Dup, message, file, axis.line));
            }
            self.m.axes.push(&axis.id, axis.as_ref().clone());
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
        if let Some(from) = head.from {
            found.from.insert(head.id.clone(), from);
        }
        declared.push((head.id.as_str(), file.path.as_str(), head.line));
        for section in &file.sections {
            found
                .section
                .insert(section.id_line().0.to_owned(), head.id.clone());
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
