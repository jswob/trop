//! Declarations and what versions state for them: inputs and their rows, defs and their rules, bands, places, moves, observations.
//! Spec: L §5.6 (key binds), §5.8 (inputs), §5.9 (bands), §5.10 (places), §5.11 (defs and
//! rules), §5.14 (moves), §5.19 (observations), §4.6 (rows: pattern, span, known, instant).
//! Twin: engine/tools/refcore/model.py (`BindInfo`, `InputInfo`, `RuleVer`, `DefInfo`,
//! `Table`, `BandRow`, `BandInfo`, `PlaceInfo`, `MoveInfo`, `ObsRow`, `Row`).
//! Invariants: a row's pattern has one position per key bind of its target; a rule's
//! versions are kept in the order versions state them.

use super::key::{KeyVal, Pat, Span};
use super::ty::Ty;
use crate::num::Num;
use crate::syntax::{Bind, Def, Edge, Expr, FieldValue, Lit, Move, Place, Price, Rule, RuleValue};
use crate::syntax::{LawDate, Value};
use crate::time::Day;

/// What a key bind ranges over (refcore's `BindInfo.dom`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Domain {
    /// The keys of an index.
    Index(String),
    /// The things of a kind.
    Kind(String),
    /// The options of an axis.
    Axis(String),
    /// A written list of keys.
    List(Vec<Lit>),
    /// A stepped range, its bind as written.
    Range,
}

/// A key bind of a declaration (refcore's `BindInfo`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindInfo {
    /// The bound variable.
    pub var: String,
    /// What it ranges over.
    pub domain: Domain,
    /// The bind as written.
    pub node: Bind,
}

/// Where a declaration's one period key is (refcore's `time_pos`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimePos {
    /// No key over a period index.
    None,
    /// One, at this position.
    At(usize),
    /// More than one.
    Multi,
}

/// A row's instant (L §4.6 Readings, §5.19).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowInstant {
    /// `at D [start|end]`.
    At(Day, Option<Edge>),
    /// `after x`.
    After(Expr),
}

/// A row a version states for an input, or a `given` for a def (refcore's `Row`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// The input or def it is a row of.
    pub target: String,
    /// One position per key bind.
    pub pattern: Vec<Pat>,
    /// How many positions it states.
    pub stated: usize,
    /// Its value, as written.
    pub value: Value,
    /// `assumed "…"`.
    pub assumed: Option<String>,
    /// The version that states it.
    pub version: String,
    /// Its own `governs` span.
    pub own_span: Option<Span>,
    /// Its own `known` date.
    pub own_known: Option<Day>,
    /// A reading's instant.
    pub instant: Option<RowInstant>,
    /// Its locator text.
    pub locator: Option<String>,
    /// Whether it is a `given`.
    pub given: bool,
    /// Whether it is a `replace` line.
    pub replace: bool,
    /// Its line.
    pub line: u32,
    /// Its stable id, `v1:rent[2026-03]` (L §2.5), set once its target's rows are all read.
    pub sid: String,
}

/// An input (refcore's `InputInfo`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputInfo {
    /// Its id.
    pub id: String,
    /// Its key binds.
    pub binds: Vec<BindInfo>,
    /// Its type; none for an observed input.
    pub ty: Option<Ty>,
    /// The factor a `%` in its unit brings.
    pub unit_factor: Num,
    /// Whether it is a reading.
    pub reading: bool,
    /// Whether it is observed.
    pub observed: bool,
    /// `question "…"`.
    pub question: Option<String>,
    /// Its rows, in the order versions state them.
    pub rows: Vec<Row>,
    /// Where its one period key is.
    pub time_pos: TimePos,
    /// Whether the language declares it (`opening`, `stated`, `account_place`).
    pub builtin: bool,
    /// `prices Y`.
    pub prices: Option<i64>,
    /// `by ix`.
    pub prices_by: Option<String>,
    /// An observed input's rows.
    pub observations: Vec<Observation>,
    /// Its line.
    pub line: Option<u32>,
}

impl InputInfo {
    /// An input with no key, type or row yet.
    #[must_use]
    pub fn new(id: &str, line: Option<u32>) -> InputInfo {
        InputInfo {
            id: id.to_owned(),
            binds: Vec::new(),
            ty: None,
            unit_factor: Num::int(1),
            reading: false,
            observed: false,
            question: None,
            rows: Vec::new(),
            time_pos: TimePos::None,
            builtin: line.is_none(),
            prices: None,
            prices_by: None,
            observations: Vec::new(),
            line,
        }
    }
}

/// A rule of a def as one version states it (refcore's `RuleVer`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleVer {
    /// The rule's label.
    pub label: String,
    /// The rule as written.
    pub rule: Rule,
    /// The version whose `replace rule` line states it.
    pub version: Option<String>,
    /// The version `v V` names.
    pub v: Option<String>,
    /// Its own `governs` span.
    pub span: Option<Span>,
    /// Whether it replaces an earlier one.
    pub replace: bool,
}

/// A def (refcore's `DefInfo`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefInfo {
    /// Its id.
    pub id: String,
    /// Its key binds.
    pub binds: Vec<BindInfo>,
    /// Its type.
    pub ty: Ty,
    /// The factor a `%` in its unit brings.
    pub unit_factor: Num,
    /// `at reader` or `at e`.
    pub at: Option<LawDate>,
    /// Its rules by label, in the order written; each label's versions in order.
    pub rules: Vec<(String, Vec<RuleVer>)>,
    /// Its `given` rows.
    pub givens: Vec<Row>,
    /// Where its one period key is.
    pub time_pos: TimePos,
    /// The def as written.
    pub decl: Def,
}

/// An end of a band row's interval: a number in the band's argument unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BandEnd {
    /// A number.
    Num(Num),
}

/// A row of a band table (refcore's `BandRow`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BandRow {
    /// The low end; none for `-inf`.
    pub lo: Option<BandEnd>,
    /// Whether the low end is in.
    pub lo_closed: bool,
    /// The high end; none for `inf`.
    pub hi: Option<BandEnd>,
    /// Whether the high end is in.
    pub hi_closed: bool,
    /// Its value.
    pub value: RuleValue,
    /// Its locator text.
    pub locator: Option<String>,
    /// Whether it is a `replace` line.
    pub replace: bool,
    /// Its line.
    pub line: u32,
}

/// A band's table as one version states it (refcore's `Table`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    /// The version.
    pub version: String,
    /// A keyed band's key.
    pub key: Option<KeyVal>,
    /// Its rows.
    pub rows: Vec<BandRow>,
}

/// A band (refcore's `BandInfo`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BandInfo {
    /// Its id.
    pub id: String,
    /// Its key binds (at most one).
    pub binds: Vec<BindInfo>,
    /// Its argument's name.
    pub arg: String,
    /// Its argument's type.
    pub arg_type: Ty,
    /// Its value's type.
    pub ty: Ty,
    /// The factor a `%` in its unit brings.
    pub unit_factor: Num,
    /// Its tables, one per version and key.
    pub tables: Vec<Table>,
}

/// A `place` section (refcore's `PlaceInfo`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceInfo {
    /// Its id.
    pub id: String,
    /// Its kind.
    pub kind: String,
    /// Its key binds.
    pub binds: Vec<BindInfo>,
    /// The unit it states.
    pub unit: Option<String>,
    /// The price input it states.
    pub priced: Option<Price>,
    /// Its lifespan.
    pub lives: Span,
    /// Its things: the key values of each binding tuple, and the thing's name.
    pub things: Vec<(Vec<KeyVal>, String)>,
    /// The section as written.
    pub decl: Place,
}

/// A `move` section (refcore's `MoveInfo`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveInfo {
    /// Its id.
    pub id: String,
    /// Its event kind.
    pub kind: String,
    /// Its key binds.
    pub binds: Vec<BindInfo>,
    /// Whether it is a decision (it states `reads`).
    pub decision: bool,
    /// Its stratum (E §3.1 step 8).
    pub stratum: u32,
    /// The section as written.
    pub decl: Move,
}

/// An observed leg: its name, places, amount and own day.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObsLeg {
    /// The leg's name.
    pub name: String,
    /// The place it leaves.
    pub from: String,
    /// The place it enters.
    pub to: String,
    /// Its amount, exact, in its unit.
    pub amount: Num,
    /// Its unit: a currency or a unit.
    pub unit: String,
    /// Its own day.
    pub day: Option<Day>,
}

/// What an observation fulfils.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fulfils {
    /// A planned instance: the move and its keys.
    Instance(String, Vec<KeyVal>),
    /// An adjustment: the move, its keys, the leg and the version.
    Adjustment(String, Vec<KeyVal>, String, String),
}

/// A row of an observed input: a real movement (refcore's `ObsRow`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    /// The observed input.
    pub input: String,
    /// The row's id.
    pub key: String,
    /// The version that states it.
    pub version: String,
    /// Its own `known` date.
    pub own_known: Option<Day>,
    /// Its locator text.
    pub locator: Option<String>,
    /// Whether it is a `replace` line.
    pub replace: bool,
    /// Whether it is `none`: a removed observation.
    pub removed: bool,
    /// `assumed "…"`.
    pub assumed: Option<String>,
    /// Its day.
    pub day: Option<Day>,
    /// Its event kind.
    pub kind: Option<String>,
    /// Its legs, in the kind's order.
    pub legs: Vec<ObsLeg>,
    /// Its attributes, as written; the evaluator converts them to their types.
    pub attrs: Vec<(String, FieldValue)>,
    /// What it fulfils.
    pub fulfils: Option<Fulfils>,
    /// Its line.
    pub line: u32,
}

impl Observation {
    /// Its stable id, `bank[r20260901a]`.
    #[must_use]
    pub fn sid(&self) -> String {
        format!("{}[{}]", self.input, self.key)
    }
}
