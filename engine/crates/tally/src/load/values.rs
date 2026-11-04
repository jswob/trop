//! Load's check of every row's value: each is converted to its target's type before any ask, as refcore converts it.
//! Spec: L §5.8 (a row's value has its input's type), §3.1 (values of each type), §3.2 (units,
//! a keyed unit, money in its minor unit), §2.1 and §7.1 (literals: `a/b` only where it
//! doesn't terminate), §4.6 (a reading's value, in its place's unit); E §3.1 step 4.
//! Twin: engine/tools/refcore/loader.py (`check_rows`, its conversion of each row, and
//! `place_value`).
//! Invariants: the rows are checked in refcore's order (inputs, the built-ins first, then
//! defs' givens, each in the order its rows were built); the conversion is the runtime's own
//! (`eval::convert`), so load refuses what a read would; a form whose value the runtime
//! doesn't build yet is left to the read, never guessed.

use super::steps::Loader;
use crate::base::{Code, Load, Place as At, Refusal};
use crate::eval::{ConvertFor, convert};
use crate::model::{KeyVal, Pat, Row, Shown, Ty, Unit};
use crate::syntax::{Lit, Section, Stop, Value as Written};
use std::collections::BTreeMap;

impl Loader {
    /// Each row's value converted to its target's type: inputs' rows, a balance reading by its
    /// place's unit, then defs' givens; an observed input's rows are its observations', and a
    /// distribution's row is an expression the checker types (refcore's `check_rows`, its
    /// conversion). L §5.8, §4.6.
    ///
    /// # Errors
    ///
    /// The value's refusal, at its row's line: `LOAD-UNIT`, `LOAD-PRECISION`, `LOAD-FIELD`,
    /// `LOAD-DOMAIN`, `LOAD-PARSE`, `LOAD-KEY` (a reading that states no place).
    pub(super) fn check_rows(&self) -> Result<(), Stop> {
        let files = self.version_files();
        for (id, input) in self.m.inputs.entries() {
            if input.observed {
                continue;
            }
            let balance = input.builtin && (id == "opening" || id == "stated");
            for row in &input.rows {
                let file = row_file(&files, row)?;
                let checked = if balance {
                    self.place_value(id, row)
                } else if let Some(ty) = input.ty.as_ref().filter(|t| !matches!(t, Ty::Dist(_))) {
                    let of = ConvertFor::row(&input.unit_factor, ty, &input.binds, &row.pattern);
                    converted(self, &row.value, ty, id, &of)
                } else {
                    Ok(())
                };
                checked.map_err(|stop| at_row(stop, &file, row.line))?;
            }
        }
        for (id, def) in self.m.defs.entries() {
            if matches!(def.ty, Ty::Dist(_)) {
                continue;
            }
            for row in &def.givens {
                let file = row_file(&files, row)?;
                let of = ConvertFor::row(&def.unit_factor, &def.ty, &def.binds, &row.pattern);
                converted(self, &row.value, &def.ty, id, &of)
                    .map_err(|stop| at_row(stop, &file, row.line))?;
            }
        }
        Ok(())
    }

    /// A balance reading's value: a quantity in its place's unit; a place of unit `any` states
    /// its unit in the row (refcore's `place_value`). L §4.6, §5.19.
    fn place_value(&self, input: &str, row: &Row) -> Result<(), Stop> {
        let Some(Pat::Key(KeyVal::Thing(place))) = row.pattern.first() else {
            return Err(load_refusal(
                Load::Key,
                format!("a row of {input} states its place"),
            ));
        };
        let number = match &row.value {
            Written::Hole(_) | Written::None => return Ok(()),
            Written::Lit(Lit::Num(number)) => number,
            Written::Never
            | Written::Str(_)
            | Written::Record(_)
            | Written::List(_)
            | Written::Calendar(_)
            | Written::Days(_)
            | Written::Lit(_)
            | Written::Dist(_)
            | Written::Expr(_) => {
                return Err(load_refusal(
                    Load::Unit,
                    format!("a value of the wrong type for {input}[{place}]"),
                ));
            }
        };
        let stated = self
            .m
            .things
            .get(place)
            .and_then(|thing| self.m.place_unit(thing))
            .filter(|unit| unit != "any");
        let unit = match stated {
            Some(unit) => unit,
            None => row_unit(input, place, number)?,
        };
        let ty = if self.m.currencies.contains_key(&unit) {
            Ty::Money(unit)
        } else {
            let one = Unit::make(vec![(unit, 1)]).map_err(super::steps::num_stop)?;
            Ty::Num(one, Shown::Plain)
        };
        converted(self, &row.value, &ty, input, &ConvertFor::plain())
    }

    /// The file each version's section stands in, by the version's id.
    fn version_files(&self) -> BTreeMap<String, String> {
        let mut files = BTreeMap::new();
        for (file, section) in &self.sections {
            if let Section::Version(version) = section {
                files.insert(version.id.clone(), file.clone());
            }
        }
        files
    }
}

/// The unit a reading of a place of unit `any` states in its row (refcore's `place_value`,
/// its `any` branch). L §4.6.
fn row_unit(input: &str, place: &str, number: &crate::syntax::Number) -> Result<String, Stop> {
    let Some(written) = &number.unit else {
        return Err(load_refusal(
            Load::Unit,
            format!("{input}[{place}] holds any unit; its row states the unit"),
        ));
    };
    let terms = written
        .terms
        .iter()
        .map(|t| (t.base.clone(), t.power))
        .collect();
    let (unit, _) = Unit::from_syntax(terms).map_err(super::steps::num_stop)?;
    match unit.terms() {
        [(base, 1)] => Ok(base.clone()),
        _ => Err(load_refusal(
            Load::Unit,
            format!("{} is not a unit of a place", number.text),
        )),
    }
}

/// A row's value converted, or left to the read where the runtime doesn't build its form.
fn converted(
    loader: &Loader,
    value: &Written,
    ty: &Ty,
    target: &str,
    of: &ConvertFor,
) -> Result<(), Stop> {
    match convert(&loader.m, value, ty, target, of) {
        Ok(_) | Err(Stop::NotBuilt(_)) => Ok(()),
        Err(stop @ (Stop::Refused(_) | Stop::Deep)) => Err(stop),
    }
}

/// The file of a row's version, which load built from a section of it.
fn row_file(files: &BTreeMap<String, String>, row: &Row) -> Result<String, Stop> {
    files.get(&row.version).cloned().ok_or_else(|| {
        Stop::Refused(Refusal::new(
            Code::EngineInvariant,
            "a row of a version with no section".to_owned(),
        ))
    })
}

/// A refusal met converting a row, put at the row's line when it names no place of its own.
fn at_row(stop: Stop, file: &str, line: u32) -> Stop {
    match stop {
        Stop::Refused(refusal) => Stop::Refused(refusal.placed(At::Line(file.to_owned(), line))),
        other @ (Stop::NotBuilt(_) | Stop::Deep) => other,
    }
}

/// A load refusal with no place yet.
fn load_refusal(code: Load, message: String) -> Stop {
    Stop::Refused(Refusal::new(Code::Load(code), message))
}
