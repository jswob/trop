//! Load's checker: one walk over every expression of the reality, each typed in refcore's order, each read recorded with how its keys relate to its reader's.
//! Spec: E §3.1 step 5 (nodes, edges with their key relations, a binding over a kind), step 7
//! (the marks), The order of the checks (one sort at a time: moves, defs, bands, asks, each
//! in section order); L §3 (types), §5.11 (a def's rules: `when` a bool, the value of the
//! def's type, `beats`), §5.9 (a band's rows), §5.14 (a move's day, legs and attributes),
//! §5.18 (an ask's `law` and `as known`), §2.1 and §7.1 (a four-digit figure's reading, the
//! literal rule), §8.3 (`LOAD-UNIT`, `-LITERAL`, `-LABEL`, `-CYCLE`, `-LEG`, `-LAWPERIOD`,
//! `-NAME`).
//! Twin: engine/tools/refcore/check.py (`Scope`, `Var`, `Checker.run`, `edge`, `relation`,
//! `def_scope`, `dom_type`, `check_def`, `_beats_cycle`, `check_band`, `check_ask`,
//! `check_prices`, `kl_finish`, the year-literal loop of `run`); check3.py (`m_check_move`,
//! `m_read_target`, `m_literal_place`, `m_place_target`, `m_move_changes`).
//! Invariants: the checks run in refcore's order and each refusal is put at the line its
//! check names; a form whose checks are M0's later groups' is typed `Unseen` and refused by
//! nothing; the price clauses are resolved before any item; every recursion is counted
//! (R-16), and the parser bounds every tree well below the count.

use super::across::PeriodRead;
use super::fills::{FillState, inline_fill_edges};
use super::keys::{KeyLit, keyed_place_thing};
use super::moves::{own_day_reads, settlements};
use super::pins::unloaded_world;
use super::prices::price_twice;
use super::random::{DrawKey, QueryAt, dist_whole};
use super::reads::{is_move_type, is_place_type};
use super::srows::{BaseWorld, SrowAt, ask_worlds, top_scenario};
use super::steps::refuse;
use super::types::{
    children, covered_spans, is_bool, is_date, key_value_kind, lit_rule, refused, uerr,
};
use crate::base::{Code, Load, Place as At, Reach, Refusal};
use crate::model::{
    BandInfo, BindInfo, DefInfo, Domain, Graph, KeyVal, KindSpec, Mark, MoveInfo, NameKind, Node,
    NodeKind, Reality, Rel, Shift, Target, Ty,
};
use crate::num::Num;
use crate::syntax::Bal;
use crate::syntax::{
    AddOp, Ask, AskBody, CmpOp, Domain as Written, Expr, Key, LawDate, Lit, Number, Order, Postfix,
    RChoice, RuleValue, Section, Stop,
};
use crate::time::PeriodKind;
use std::collections::{BTreeMap, BTreeSet};

/// How deep the walk goes into one expression. The parser's own limits (eight brackets or
/// `if`s, 64 chained reads) keep every tree far below it.
const MAX_DEPTH: u32 = 512;

/// Where an expression stands, for the literal rule (refcore's `Scope.where`). L §7.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Stands {
    /// A def's rule, a move, a price clause: no figure and no date but the inline ones.
    Rule,
    /// A def's `at e` and its key binds' ranges, read as a rule is.
    At,
    /// An ask: dates, and figures only where it allows them.
    Ask,
    /// A band's row: any figure.
    Band,
    /// A declaration's range: any literal.
    Decl,
}

/// What the literal rule is applied to. L §7.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Figure {
    /// A number.
    Num,
    /// A date or a period.
    Date,
}

/// How a four-digit figure, or a key-list variable of four-digit keys, is read. L §2.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Reading {
    /// As a year.
    Year,
    /// As an integer.
    Int,
}

/// Whether a read stands under an aggregate's binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Under {
    /// In the item's own expression.
    Item,
    /// Under an aggregate's binding.
    Aggregate,
}

/// How an axis is read: by its id (or as a mode), or as the key at a position over it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AxisRead {
    /// By its id, its option the current one; or as a rounding mode.
    ById,
    /// As the key at a key position over the axis.
    AtKey,
}

/// What a bound variable ranges over (refcore's `b._dom`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum VarDom {
    /// The keys of an index.
    Index(String),
    /// The things of a kind.
    Kind(String),
    /// The movements of an event kind.
    Event(String),
    /// The rows of an observed input.
    Obs(String),
    /// The options of an axis.
    Axis(String),
    /// A key list.
    List,
    /// A stepped range.
    Range,
    /// A band's argument, which ranges over nothing.
    Arg,
}

/// A bound variable (refcore's `Var`).
#[derive(Debug, Clone)]
pub(super) struct Var {
    /// Its name.
    pub(super) name: String,
    /// Its type.
    pub(super) ty: Ty,
    /// What it ranges over.
    pub(super) dom: VarDom,
    /// A key list's keys, as written; empty for a declaration's key bind.
    pub(super) keylist: Vec<Lit>,
    /// Which variable it is: a key list's index is fixed per variable.
    pub(super) id: usize,
    /// A range's end, as written, for a variable bound over a range (refcore's `v.node.dom.b`):
    /// a slot key whose range ends at a draw (L §5.16).
    pub(super) end: Option<Expr>,
}

impl Var {
    /// The kind whose things or movements it ranges over, where a leg may name them.
    fn kind(&self) -> Option<&str> {
        match &self.dom {
            VarDom::Kind(kind) | VarDom::Event(kind) => Some(kind),
            VarDom::Obs(_) => Some("events"),
            VarDom::Index(_) | VarDom::Axis(_) | VarDom::List | VarDom::Range | VarDom::Arg => None,
        }
    }
}

/// Where an expression is read: the reading item, its variables, how its reads relate to
/// its keys, and what its rules allow (refcore's `Scope`).
#[derive(Debug, Clone)]
pub(super) struct Scope {
    /// The item that reads.
    pub(super) item: Node,
    /// The variables in scope.
    pub(super) vars: BTreeMap<String, Var>,
    /// The item's own key variables.
    pub(super) keyvars: Vec<String>,
    /// Whether the read is under an aggregate's binding.
    pub(super) under: Under,
    /// A read at a day the reader states (`explicit`) or at each day asked (`calendar`),
    /// whatever its keys.
    pub(super) mode: Option<Rel>,
    /// The file and line the expression is written on, where a refusal met in it is put.
    pub(super) at: (String, u32),
    /// Where it stands, for the literal rule.
    pub(super) stands: Stands,
    /// Whether a figure is allowed here (a key offset, a `pos` comparison, a step).
    pub(super) allow_lit: bool,
    /// Whether it is in a rule with `v` (`missing()`).
    pub(super) cited: bool,
    /// Whether it is in a rule's `when`.
    pub(super) in_when: bool,
    /// The spans a `covered()` guard holds here (L §5.19).
    pub(super) covered: BTreeSet<String>,
    /// The key variables a condition tests here: a rule's `when` and its beaters', or the
    /// left of an `and`, or an `if`'s condition (refcore's `Scope.guards`). L §5.11.
    pub(super) guards: BTreeSet<String>,
    /// How a read of a random fact is taken here: plainly (its sample), as the distribution
    /// (refcore's `dist_role`, which a child scope drops), or inside a query over random
    /// values (refcore's `in_query`, which a child keeps and which outranks the role).
    /// L §5.16.
    pub(super) read_in: ReadIn,
}

/// How a read of a random fact is taken in a scope (refcore's `dist_role`, `in_query`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ReadIn {
    /// Its sample on the path.
    Plain,
    /// Its distribution.
    Dist,
    /// Inside a query over random values.
    Query,
}

impl Scope {
    /// The scope of `item`'s own expression, standing as `stands`, written at `line` of `file`.
    pub(super) fn new(item: Node, stands: Stands, file: &str, line: u32) -> Scope {
        Scope {
            item,
            vars: BTreeMap::new(),
            keyvars: Vec::new(),
            under: Under::Item,
            mode: None,
            at: (file.to_owned(), line),
            stands,
            allow_lit: false,
            cited: false,
            in_when: false,
            covered: BTreeSet::new(),
            guards: BTreeSet::new(),
            read_in: ReadIn::Plain,
        }
    }

    /// A child of this scope, as refcore's `child`: no figure allowed, no dist role.
    pub(super) fn child(&self) -> Scope {
        let mut scope = self.clone();
        scope.allow_lit = false;
        if scope.read_in == ReadIn::Dist {
            scope.read_in = ReadIn::Plain;
        }
        scope
    }

    /// This scope where a dist-typed read is the distribution, unless it is inside a query
    /// (refcore's `tc_dist`). L §5.16.
    pub(super) fn dist_scope(&self) -> Scope {
        let mut scope = self.child();
        if scope.read_in == ReadIn::Plain {
            scope.read_in = ReadIn::Dist;
        }
        scope
    }

    /// This scope, its expression on another line of the same file.
    pub(super) fn on_line(&self, line: u32) -> Scope {
        let mut scope = self.clone();
        scope.at.1 = line;
        scope
    }

    /// A refusal met in this scope's expression, at its line unless it has one.
    pub(super) fn place(&self, stop: Stop) -> Stop {
        match stop {
            Stop::Refused(refusal) => {
                Stop::Refused(refusal.placed(At::Line(self.at.0.clone(), self.at.1)))
            }
            other @ (Stop::NotBuilt(_) | Stop::Deep) => other,
        }
    }

    /// The file and line, for a refusal a key resolution puts itself.
    pub(super) fn file_line(&self) -> (String, u32) {
        self.at.clone()
    }

    /// Whether `name` is a bound variable here.
    pub(super) fn is_var(&self, name: &str) -> bool {
        self.vars.contains_key(name)
    }

    /// This scope, its reads made in `mode`.
    pub(super) fn with_mode(&self, mode: Rel) -> Scope {
        let mut scope = self.clone();
        scope.mode = Some(mode);
        scope
    }

    /// This scope under an aggregate's binding (refcore's `child`, which allows no figure).
    pub(super) fn in_aggregate(&self) -> Scope {
        let mut scope = self.clone();
        scope.under = Under::Aggregate;
        scope.allow_lit = false;
        if scope.read_in == ReadIn::Dist {
            scope.read_in = ReadIn::Plain;
        }
        scope
    }

    /// This scope, read by `item` instead.
    pub(super) fn for_item(&self, item: Node) -> Scope {
        let mut scope = self.clone();
        scope.item = item;
        scope
    }
}

/// A four-digit figure whose reading is decided later (refcore's `year4_nodes`).
#[derive(Debug, Clone)]
pub(super) struct Year4 {
    /// The scope it was typed in: its literal rule's.
    pub(super) scope: Scope,
    /// Its text.
    pub(super) text: String,
    /// Its value.
    pub(super) value: Num,
    /// The reading taken, once one is.
    pub(super) reading: Option<Reading>,
    /// Whether it was taken as a key of a period index, which the literal rule saw then.
    pub(super) keyed: bool,
}

/// A key-list variable read as a value, checked at the end against the index its key
/// positions fix (refcore's `kl_uses`).
#[derive(Debug, Clone)]
pub(super) struct KeyListUse {
    /// The variable's id.
    pub(super) var: usize,
    /// Its name.
    pub(super) name: String,
    /// How it was read.
    pub(super) reading: Reading,
    /// The file and line of the read.
    pub(super) at: (String, u32),
}

/// A def's key variables, each with the integer offsets of its key arithmetic (L §7.1).
pub(super) type Offsets = BTreeMap<String, BTreeSet<i64>>;

/// What one move's walk finds: the reads it declares and the places its legs change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MoveFacts {
    /// The move.
    pub(super) id: String,
    /// What its `reads` declares, in the order written.
    pub(super) read_targets: Vec<Target>,
    /// The places its legs change, each once.
    pub(super) changes: Vec<Target>,
}

/// A read at a key variable of a def shifted by a number, `x[k - 1]` (refcore's
/// `shift_reads` entry). L §5.11.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ShiftRead {
    /// The def that reads.
    pub(super) reader: Node,
    /// What it reads.
    pub(super) target: Node,
    /// The key variable shifted.
    pub(super) var: String,
    /// Which way.
    pub(super) shift: Shift,
    /// The offset as written, `1` of `x[k - 1]`, or `n` where no key is the variable and
    /// a number (refcore's `shift_offset`). L §8.3 `LOAD-GUARD`.
    pub(super) offset: String,
    /// Whether a condition on the variable guards the read.
    pub(super) guarded: bool,
    /// The file and line it is written on.
    pub(super) at: (String, u32),
}

/// A decision's plain `bal(x, d)`, which load folds where it sees the day (refcore's
/// `cycle_cands` entry). L §5.14.1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DayRead {
    /// The decision.
    pub(super) id: String,
    /// The place read, as written.
    pub(super) place: Expr,
    /// The day read, as written.
    pub(super) day: Expr,
    /// Whether it reads the start of the day (`start`).
    pub(super) start: bool,
}

/// A projection `bal(x, d) from t` read in a move, outside any query (refcore's `projs`
/// entry, a move's, not in `inq`). L §5.14.1, §5.16.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProjRead {
    /// The read, as written.
    pub(super) bal: Bal,
    /// The move that holds it.
    pub(super) holder: String,
    /// The place it projects.
    pub(super) target: Target,
    /// The file and line it is written on.
    pub(super) at: (String, u32),
}

/// What the walk finds beyond the graph, for the checks on it (refcore's `Checker` state).
#[derive(Debug, Default)]
pub(super) struct Walked {
    /// The item graph's edges and marks.
    pub(super) graph: Graph,
    /// Each move's declared reads and changes.
    pub(super) facts: Vec<MoveFacts>,
    /// The periods read at keys of time indexes.
    pub(super) periods: Vec<PeriodRead>,
    /// The shifted reads of defs, in the order typed.
    pub(super) shifts: Vec<ShiftRead>,
    /// Each def's reads inside its `at` (refcore's `at_reads`). L §4.3.
    pub(super) at_reads: BTreeMap<String, BTreeMap<Node, Vec<Rel>>>,
    /// Each move's direct reads of inputs, defs and bands inside its `at`, in the order
    /// typed. L §4.3, §5.14.
    pub(super) move_at: BTreeMap<String, Vec<Node>>,
    /// Each ask's reads in the order first made (refcore's edges, a dict in insertion
    /// order).
    pub(super) ask_reads: BTreeMap<String, Vec<Node>>,
    /// The decisions' plain balance reads, in the order typed.
    pub(super) day_reads: Vec<DayRead>,
    /// The asks refused alone, with their refusals (refcore's `asks_alone`). L §6.4, §8.
    pub(super) alone: BTreeMap<String, Refusal>,
    /// The queries and path scores over random values, in the order typed (refcore's
    /// `queries`). L §5.16.
    pub(super) queries: Vec<QueryAt>,
    /// The items that hold `crps` or `pdf` (refcore's `orient_items`). L §5.15.
    pub(super) orient_items: BTreeSet<Node>,
    /// The slot keys a range up to a value gives, in the order typed (refcore's
    /// `draw_keys`). L §5.16.
    pub(super) draws: Vec<DrawKey>,
    /// The moves' projections, in the order typed (refcore's `projs`). L §5.16.
    pub(super) projs: Vec<ProjRead>,
    /// The scenario's expression rows, in the order typed (refcore's `srow` items). L §6.4.
    pub(super) srows: Vec<SrowAt>,
    /// What each def, fill, band and expression row reads, in the order it first read each
    /// (refcore's edges, a dict in insertion order).
    pub(super) read_order: BTreeMap<Node, Vec<Node>>,
    /// The asks a scenario's reality doesn't hold: those whose lines are evaluated in other
    /// worlds (refcore's filter of `m.asks`). L §6.4.
    pub(super) dropped: BTreeSet<String>,
}

/// The checker's state over the walk (refcore's `Checker`).
#[derive(Debug, Default)]
pub(super) struct Checker {
    /// The item graph's edges and marks.
    pub(super) graph: Graph,
    /// The four-digit figures met, in the order typed.
    pub(super) year4: Vec<Year4>,
    /// Each key-list variable's index, fixed by its first key position.
    pub(super) kl_fixed: BTreeMap<usize, Domain>,
    /// Key-list variables read as values.
    pub(super) kl_uses: Vec<KeyListUse>,
    /// Each def's key offsets, by its id.
    offsets: BTreeMap<String, Offsets>,
    /// The variables made so far.
    vars: usize,
    /// Each checked move's legs' static amount types, by (move, leg) (refcore's `ln._lt`).
    pub(super) leg_types: BTreeMap<(String, String), Ty>,
    /// The periods read at keys of time indexes, in the order typed (refcore's
    /// `period_cands`).
    periods: Vec<PeriodRead>,
    /// The shifted reads of defs, in the order typed (refcore's `shift_reads`).
    shifts: Vec<ShiftRead>,
    /// Each def's reads inside its `at` (refcore's `at_reads`).
    at_reads: BTreeMap<String, BTreeMap<Node, Vec<Rel>>>,
    /// Each move's direct reads inside its `at`.
    move_at: BTreeMap<String, Vec<Node>>,
    /// The direct reads of inputs, defs and bands typed while an `at` is (refcore's `_r` of
    /// the nodes under it).
    direct: Option<Vec<Node>>,
    /// Each ask's reads in the order first made.
    ask_reads: BTreeMap<String, Vec<Node>>,
    /// The decisions' plain balance reads.
    day_reads: Vec<DayRead>,
    /// The fills: each one's sides, typed once (refcore's `fill_infos`).
    pub(super) fill: FillState,
    /// Whether the next of `to u`'s forms typed stands under it (refcore's `_under_to`).
    pub(super) under_to: bool,
    /// The random value X of an affine form under `cdf`, `pdf` or `pmf`, until typed
    /// (refcore's `_affx`).
    pub(super) aff_x: Option<Expr>,
    /// Whether the node being typed is that X.
    pub(super) affx_hit: bool,
    /// The queries and path scores over random values, in the order typed.
    pub(super) queries: Vec<QueryAt>,
    /// The items that hold `crps` or `pdf`.
    pub(super) orient_items: BTreeSet<Node>,
    /// The slot keys a range up to a value gives.
    pub(super) draws: Vec<DrawKey>,
    /// The moves' projections outside queries.
    pub(super) projs: Vec<ProjRead>,
    /// The scenario's expression rows typed so far.
    pub(super) srows: Vec<SrowAt>,
    /// What each def, fill, band and expression row reads, in the order first read.
    pub(super) read_order: BTreeMap<Node, Vec<Node>>,
    /// While a scenario's expression row is typed, the world its prefixed names are read in
    /// (refcore's `_world` mark).
    pub(super) base_world: Option<BaseWorld>,
}

/// One level deeper into an expression, refused as an invariant past the count (R-16).
pub(super) fn deeper(depth: u32) -> Result<u32, Stop> {
    let too_deep = || {
        Stop::Refused(Refusal::new(
            Code::EngineInvariant,
            "an expression nested deeper than the parser allows".to_owned(),
        ))
    };
    let next = depth.checked_add(1).ok_or_else(too_deep)?;
    if next > MAX_DEPTH {
        return Err(too_deep());
    }
    Ok(next)
}

/// The item graph's edges and marks, and each move's declared reads and changes, the
/// reality type-checked on the way (refcore's `Checker.run`: the parts clauses, price
/// clauses, inputs' ranges, then moves, defs, bands and asks, one sort at a time, each in
/// section order, a scenario's reality checking only the asks its world evaluates; then
/// `prices … by`, the random rows, a scenario's expression rows, the four-digit figures'
/// literal rule and the key lists' readings). E §3.1 steps 5, 7, The order of the checks;
/// L §6.4.
///
/// # Errors
///
/// The checker's refusals, each at its line; `NotBuilt` for a form M0 doesn't build.
pub(super) fn walk(m: &Reality, sections: &[(String, Section)]) -> Result<Walked, Stop> {
    let files = Files::of(sections);
    price_twice(m, &files)?;
    let mut checker = Checker {
        fill: FillState::of(m, &files)?,
        ..Checker::default()
    };
    checker.check_parts(m, &files)?;
    checker.check_priced(m, &files)?;
    for (id, input) in m.inputs.entries() {
        if input
            .binds
            .iter()
            .any(|bind| matches!(bind.domain, Domain::Range))
        {
            let line = input
                .line
                .ok_or_else(|| invariant(format!("input {id} has no line")))?;
            let scope = Scope::new(
                Node::new(NodeKind::Input, id),
                Stands::Decl,
                files.get("input", id)?,
                line,
            );
            checker.keyed(m, scope, &input.binds)?;
        }
    }
    let mut facts = Vec::new();
    for (_, mv) in m.moves.entries() {
        facts.push(checker.check_move(m, mv, files.of_move(&mv.id)?)?);
    }
    for (id, _) in m.fills.entries() {
        checker.check_fill(m, id)?;
    }
    for (id, def) in m.defs.entries() {
        checker.check_def(m, def, files.of_def(id)?)?;
    }
    for (_, band) in m.bands.entries() {
        checker.check_band(m, band, &files)?;
    }
    // Every ask is checked only in the worlds its line is evaluated in: a scenario's reality
    // holds those that name its scenario (L §6.4, §5.18).
    let mut dropped = BTreeSet::new();
    if let Some(top) = top_scenario(m) {
        for ask in &m.asks {
            if !ask_worlds(m, &checker.graph, ask).contains(top) {
                dropped.insert(ask.id.clone());
            }
        }
    }
    let mut alone = BTreeMap::new();
    for ask in m.asks.iter().filter(|ask| !dropped.contains(&ask.id)) {
        match checker.check_ask(m, ask, files.of_ask(&ask.id)?) {
            Err(Stop::Refused(refusal)) if refused_alone(&refusal) => {
                alone.insert(ask.id.clone(), refusal);
            }
            other => other?,
        }
    }
    checker.prices_by(m, &files)?;
    checker.check_dist_rows(m, &files)?;
    if !m.scenarios.over.is_empty() {
        checker.check_scen_rows(m, &files)?;
    }
    checker.year4_finish()?;
    checker.kl_finish(m)?;
    inline_fill_edges(&mut checker.graph, &mut checker.read_order);
    settlements(m, &checker.leg_types)?;
    Ok(Walked {
        graph: checker.graph,
        facts,
        periods: checker.periods,
        shifts: checker.shifts,
        at_reads: checker.at_reads,
        move_at: checker.move_at,
        ask_reads: checker.ask_reads,
        day_reads: checker.day_reads,
        alone,
        queries: checker.queries,
        orient_items: checker.orient_items,
        draws: checker.draws,
        projs: checker.projs,
        srows: checker.srows,
        read_order: checker.read_order,
        dropped,
    })
}

/// Whether an ask's refusal refuses that ask alone: a world the run doesn't load (refcore's
/// `check_ask` caller, `LOAD-SCENARIO … neither the model nor a loaded scenario`). L §6.4, §8.
fn refused_alone(refusal: &Refusal) -> bool {
    refusal.code() == Code::Load(Load::Scenario) && refusal.reach() == Some(&Reach::UnloadedWorld)
}

/// Whether the order of a reader's reads is kept: a def's, a fill's, a band's or a scenario
/// row's, which a scenario row's reads are searched through (refcore's edges, in insertion
/// order). L §6.4.
pub(super) fn keeps_order(reader: &Node) -> bool {
    matches!(
        reader.kind,
        NodeKind::Def | NodeKind::Fill | NodeKind::Band | NodeKind::Srow
    )
}

/// An `ENGINE-INVARIANT` refusal.
pub(super) fn invariant(message: String) -> Stop {
    Stop::Refused(Refusal::new(Code::EngineInvariant, message))
}

/// The file each item's section stands in, by the item's id: a refusal names it.
pub(super) struct Files {
    /// `(section word, id)` to the file.
    by_item: BTreeMap<(&'static str, String), String>,
}

impl Files {
    /// The files of the sections' moves, defs, asks, versions, inputs, places and kinds.
    pub(super) fn of(sections: &[(String, Section)]) -> Files {
        let mut by_item = BTreeMap::new();
        for (file, section) in sections {
            let item = match section {
                Section::Move(mv) => ("move", mv.id.clone()),
                Section::Def(def) => ("def", def.id.clone()),
                Section::Ask(ask) => ("ask", ask.id.clone()),
                Section::Fill(fill) => ("fill", fill.id.clone()),
                Section::Version(version) => ("version", version.id.clone()),
                Section::Input(input) => ("input", input.id.clone()),
                Section::Place(place) => ("place", place.id.clone()),
                Section::Kind(kind) => ("kind", kind.id.clone()),
                Section::Source(_)
                | Section::Unit(_)
                | Section::Index(_)
                | Section::Axis(_)
                | Section::Band(_) => continue,
            };
            by_item.insert(item, file.clone());
        }
        Files { by_item }
    }

    /// The file of the section `word id`, which load built the item from.
    pub(super) fn get(&self, word: &'static str, id: &str) -> Result<&str, Stop> {
        self.by_item
            .get(&(word, id.to_owned()))
            .map(String::as_str)
            .ok_or_else(|| invariant(format!("{word} {id} has no section")))
    }

    /// A move's file.
    fn of_move(&self, id: &str) -> Result<&str, Stop> {
        self.get("move", id)
    }

    /// A def's file.
    fn of_def(&self, id: &str) -> Result<&str, Stop> {
        self.get("def", id)
    }

    /// An ask's file.
    fn of_ask(&self, id: &str) -> Result<&str, Stop> {
        self.get("ask", id)
    }

    /// A version's file, where its band rows and entries stand.
    pub(super) fn of_version(&self, id: &str) -> Result<&str, Stop> {
        self.get("version", id)
    }

    /// A load refusal at the declaration of input `id`, which stands at `line`.
    fn at_input(&self, id: &str, line: Option<u32>, code: Load, message: String) -> Stop {
        let file = match self.get("input", id) {
            Ok(file) => file.to_owned(),
            Err(stop) => return stop,
        };
        let Some(line) = line else {
            return invariant(format!("input {id} has no line"));
        };
        Stop::Refused(Refusal::at(Code::Load(code), message, At::Line(file, line)))
    }
}

impl Checker {
    /// A period read at key `kn` of time index `ix`, folded once the walk is done (refcore's
    /// `period_conv`, its `period_cands`). L §4.6.
    pub(super) fn period_read(&mut self, scope: &Scope, ix: &str, kn: &Expr) {
        self.periods.push(PeriodRead {
            index: ix.to_owned(),
            key: kn.clone(),
            reader: scope.item.clone(),
            at: scope.file_line(),
        });
    }

    /// An axis `axis` read by `item`: by its id or as a mode, an edge too; at a key position
    /// of the axis, no edge (refcore's `axes_of`, `edge`). L §5.5.
    pub(super) fn axis_read(&mut self, item: &Node, axis: &str, read: AxisRead) {
        self.graph
            .axes_of
            .entry(item.clone())
            .or_insert_with(BTreeSet::new)
            .insert(axis.to_owned());
        if read == AxisRead::ById {
            self.edge(item, Node::new(NodeKind::Axis, axis), Rel::Any);
        }
    }

    /// `reader` reads `target` with `rel`; an ask's reads, and a def's, a fill's, a band's and
    /// a scenario row's, are kept in the order first made.
    pub(super) fn edge(&mut self, reader: &Node, target: Node, rel: Rel) {
        let first = !self
            .graph
            .reads(reader)
            .is_some_and(|reads| reads.contains_key(&target));
        if first && keeps_order(reader) {
            self.read_order
                .entry(reader.clone())
                .or_insert_with(Vec::new)
                .push(target.clone());
        }
        if reader.kind == NodeKind::Ask {
            let reads = self
                .ask_reads
                .entry(reader.id.clone())
                .or_insert_with(Vec::new);
            if !reads.contains(&target) {
                reads.push(target.clone());
            }
        }
        self.graph.edge(reader, target, rel);
    }

    /// A read of an input, a def or a band, kept while an `at` is typed (refcore's `_r` of
    /// a read or a band's node). L §4.3.
    pub(super) fn direct(&mut self, target: &Node) {
        if let Some(reads) = &mut self.direct {
            reads.push(target.clone());
        }
    }

    /// A def's read at a shifted key variable, with whether a condition guards it (refcore's
    /// `shift_reads`). L §5.11.
    pub(super) fn shifted(&mut self, scope: &Scope, target: &Node, rel: &Rel, keys: &[Key]) {
        if let Rel::Shifted(var, shift) = rel {
            self.shifts.push(ShiftRead {
                reader: scope.item.clone(),
                target: target.clone(),
                var: var.clone(),
                shift: *shift,
                offset: shift_offset(keys, var),
                guarded: scope.guards.contains(var),
                at: scope.file_line(),
            });
        }
    }

    /// `item` holds a running read, a projection or `bal(x)` with no day of `target`.
    pub(super) fn mark(&mut self, item: &Node, mark: Mark, target: Target) {
        self.graph.mark(item, mark, target);
    }

    /// A new variable (refcore's `Var`), with its own id.
    pub(super) fn new_var(
        &mut self,
        name: &str,
        ty: Ty,
        dom: VarDom,
        keylist: Vec<Lit>,
    ) -> Result<Var, Stop> {
        let id = self.vars;
        self.vars = id
            .checked_add(1)
            .ok_or_else(|| invariant("too many bound variables".to_owned()))?;
        Ok(Var {
            name: name.to_owned(),
            ty,
            dom,
            keylist,
            id,
            end: None,
        })
    }

    /// A four-digit figure typed with no reading yet, its literal rule's scope kept
    /// (refcore's `year4_nodes`). L §2.1.
    pub(super) fn push_year4(&mut self, scope: &Scope, n: &Number) {
        self.year4.push(Year4 {
            scope: scope.clone(),
            text: n.text.clone(),
            value: n.value.clone(),
            reading: None,
            keyed: false,
        });
    }

    /// A four-digit figure's reading, its literal rule applied at the end. L §2.1.
    pub(super) fn decide(&mut self, entry: Option<usize>, reading: Reading) {
        if let Some(y) = entry.and_then(|at| self.year4.get_mut(at)) {
            y.reading = Some(reading);
        }
    }

    /// A four-digit figure's reading, its literal rule applied now (refcore's
    /// `year4_as_number`, `ambig`). L §2.1, §7.1.
    pub(super) fn year4_now(&mut self, entry: Option<usize>, reading: Reading) -> Result<(), Stop> {
        let Some(y) = entry.and_then(|at| self.year4.get_mut(at)) else {
            return Ok(());
        };
        y.reading = Some(reading);
        let y = y.clone();
        year4_rule(&y).map_err(|stop| y.scope.place(stop))
    }

    /// The literal rule on each four-digit figure by the reading it took, at its line
    /// (refcore's `run`, its loop over `year4_nodes`). L §2.1, §7.1.
    fn year4_finish(&self) -> Result<(), Stop> {
        for y in &self.year4 {
            if !y.keyed {
                year4_rule(y).map_err(|stop| y.scope.place(stop))?;
            }
        }
        Ok(())
    }

    /// A key-list variable read as an int or a year whose key positions fixed an index of
    /// no such keys (refcore's `kl_finish`). L §2.5, §2.1.
    fn kl_finish(&self, m: &Reality) -> Result<(), Stop> {
        for read in &self.kl_uses {
            let Some(Domain::Index(ix)) = self.kl_fixed.get(&read.var) else {
                continue;
            };
            let Some(index) = m.indexes.get(ix) else {
                continue;
            };
            let place = At::Line(read.at.0.clone(), read.at.1);
            let wrong = match read.reading {
                Reading::Int if !index.is_int => "an int",
                Reading::Year if !(index.is_int || index.period == Some(PeriodKind::Year)) => {
                    "a year"
                }
                Reading::Int | Reading::Year => continue,
            };
            return Err(Stop::Refused(Refusal::at(
                Code::Load(Load::Unit),
                format!("{} is a key of {ix}, not {wrong}", read.name),
                place,
            )));
        }
        Ok(())
    }

    /// The key offsets of def `item`'s rules, which a `pos` guard may hold (L §7.1).
    pub(super) fn offsets_of(&self, item: &Node) -> Option<&Offsets> {
        if item.kind != NodeKind::Def {
            return None;
        }
        self.offsets.get(&item.id)
    }

    /// `prices Y by ix`: `ix` is an input or a def keyed by one time index with a key for the
    /// year `Y`, and the input reads it (refcore's `check_prices`). L §3.4.
    fn prices_by(&mut self, m: &Reality, files: &Files) -> Result<(), Stop> {
        for (id, input) in m.inputs.entries() {
            let (Some(by), Some(year)) = (&input.prices_by, input.prices) else {
                continue;
            };
            let refuse =
                |code: Load, message: String| files.at_input(id, input.line, code, message);
            let binds = match m.names.kind(by) {
                None => return Err(refuse(Load::Name, format!("unknown id {by}")).naming(by)),
                Some(NameKind::Input) => m.inputs.get(by).map(|i| i.binds.as_slice()),
                Some(NameKind::Def) => m.defs.get(by).map(|d| d.binds.as_slice()),
                Some(_) => None,
            };
            let Some([bind]) = binds else {
                let message =
                    format!("prices by {by}: not an input or a def keyed by one time index");
                return Err(refuse(Load::Key, message));
            };
            let index = match &bind.domain {
                Domain::Index(ix) => m.indexes.get(ix).filter(|ix| ix.period.is_some()),
                Domain::Kind(_) | Domain::Axis(_) | Domain::List(_) | Domain::Range => None,
            };
            let Some(index) = index else {
                let message = format!("prices by {by}: not keyed by a time index");
                return Err(refuse(Load::Key, message));
            };
            let label = format!("{year:04}");
            let has_year = index.keys.iter().any(|key| {
                matches!(key, KeyVal::Period(p) if p.kind() == PeriodKind::Year && p.label() == label)
            });
            if !has_year {
                let message =
                    format!("prices {year} by {by} needs an index with a key for the year {year}");
                return Err(refuse(Load::Key, message));
            }
            if let Some((node, _)) = input_or_def(m, by) {
                self.edge(&Node::new(NodeKind::Input, id), node, Rel::Any);
            }
        }
        Ok(())
    }

    /// A declaration's scope, each key bind's variable typed by its domain (refcore's
    /// `def_scope`, `dom_type`): a stepped range's ends and step are typed here. L §5.6.
    pub(super) fn keyed(
        &mut self,
        m: &Reality,
        mut scope: Scope,
        binds: &[BindInfo],
    ) -> Result<Scope, Stop> {
        scope.keyvars = binds.iter().map(|bind| bind.var.clone()).collect();
        for bind in binds {
            let (ty, dom) = match &bind.domain {
                Domain::Index(ix) => (Ty::Key(ix.clone()), VarDom::Index(ix.clone())),
                Domain::Kind(kind) => (Ty::Key(kind.clone()), VarDom::Kind(kind.clone())),
                Domain::Axis(axis) => (Ty::Key(axis.clone()), VarDom::Axis(axis.clone())),
                Domain::List(_) => (Ty::KeyList, VarDom::List),
                Domain::Range => {
                    let Written::Range(range) = &bind.node.domain else {
                        return Err(invariant(format!(
                            "the range of {} is not written",
                            bind.var
                        )));
                    };
                    (self.range_type(m, &scope, range, 0)?, VarDom::Range)
                }
            };
            let mut var = self.new_var(&bind.var, ty, dom, Vec::new())?;
            if let Written::Range(range) = &bind.node.domain {
                var.end = Some(range.1.clone());
            }
            scope.vars.insert(bind.var.clone(), var);
        }
        Ok(scope)
    }

    /// A def's scope at `line` (refcore's `def_scope`).
    pub(super) fn def_scope(
        &mut self,
        m: &Reality,
        def: &DefInfo,
        stands: Stands,
        at: (&str, u32),
    ) -> Result<Scope, Stop> {
        let scope = Scope::new(Node::new(NodeKind::Def, &def.id), stands, at.0, at.1);
        self.keyed(m, scope, &def.binds)
    }

    /// A def: its key binds' `where`, its `at`, its rules' labels and `beats`, then each
    /// rule's `when` (a bool) and value (of the def's type) (refcore's `check_def`).
    /// L §5.11, §4.3, §2.3.
    fn check_def(&mut self, m: &Reality, def: &DefInfo, file: &str) -> Result<(), Stop> {
        let line = def.decl.line;
        let at_scope = self.def_scope(m, def, Stands::At, (file, line))?;
        for bind in &def.binds {
            if let Some(filter) = &bind.node.filter {
                let scope = self.def_scope(m, def, Stands::Rule, (file, line))?;
                let t = self.expr(m, &scope, filter, None, 0)?;
                if !is_bool(&t) {
                    return Err(scope.place(uerr(format!(
                        "where of a key bind of {} is {}, not bool",
                        def.id,
                        t.show()
                    ))));
                }
            }
        }
        let node = Node::new(NodeKind::Def, &def.id);
        let before = self
            .graph
            .edges
            .get(&node)
            .map_or_else(BTreeMap::new, Clone::clone);
        if let Some(LawDate::Expr(at)) = &def.at {
            let t = self.expr(m, &at_scope, at, None, 0)?;
            law_date(m, &t, at, &format!("at of {}", def.id)).map_err(|s| at_scope.place(s))?;
        }
        if def.at.is_some() {
            let made = new_reads(&before, self.graph.edges.get(&node));
            self.at_reads.insert(def.id.clone(), made);
        }
        let beats = labels(def, file)?;
        beats_cycle(&def.id, &beats, (file, line))?;
        self.offsets.insert(def.id.clone(), key_offsets(def));
        let guards = rule_guards(def, &beats);
        for (label, versions) in &def.rules {
            for version in versions {
                let rule = &version.rule;
                let mut scope = self.def_scope(m, def, Stands::Rule, (file, rule.line))?;
                scope.cited = version.v.is_some();
                scope.guards = guards.get(label).map_or_else(BTreeSet::new, Clone::clone);
                covered_spans(rule.when.as_ref(), &mut scope.covered);
                if let Some(when) = &rule.when {
                    self.when(m, &scope, &scope, when, &format!("when of rule {label}"))?;
                }
                let what = format!("rule {label} of {}", def.id);
                self.rule_value(m, &scope, &rule.value, &def.ty, &what)?;
            }
        }
        Ok(())
    }

    /// A `when`: a bool, typed in `scope` as a `when` is (`missing()` allowed in a cited
    /// rule's), refused at `owner`'s line (refcore's `check_def`, `m_check_move`, their
    /// `when`). L §5.11, §5.14.
    fn when(
        &mut self,
        m: &Reality,
        scope: &Scope,
        owner: &Scope,
        when: &Expr,
        what: &str,
    ) -> Result<(), Stop> {
        let mut in_when = scope.clone();
        in_when.in_when = true;
        let t = self.expr(m, &in_when, when, None, 0)?;
        if !is_bool(&t) {
            return Err(owner.place(uerr(format!("{what} is {}, not bool", t.show()))));
        }
        Ok(())
    }

    /// A rule's or a band row's value, of the type `ty` (refcore's `check_def`, its value
    /// arm; `check_band`). L §5.11, §5.9.
    fn rule_value(
        &mut self,
        m: &Reality,
        scope: &Scope,
        value: &RuleValue,
        ty: &Ty,
        what: &str,
    ) -> Result<(), Stop> {
        match value {
            RuleValue::Hole(_) => Ok(()),
            RuleValue::Solve(solve) => {
                if !scope.cited {
                    let message = "solve outside an ask (allowed in a rule with v)".to_owned();
                    return Err(scope.place(refused(Load::AskOnly, message)));
                }
                self.check_solve(m, scope, solve, 0)
                    .map(|_| ())
                    .map_err(|stop| scope.place(stop))
            }
            RuleValue::Expr(e) => {
                if matches!(ty, Ty::Dist(_)) {
                    let t = self.expr(m, &scope.dist_scope(), e, Some(ty), 0)?;
                    self.assign(m, scope, ty, e, &t, what)
                        .map_err(|stop| scope.place(stop))?;
                    return dist_whole(e, what).map_err(|stop| scope.place(stop));
                }
                let t = self.expr(m, scope, e, Some(ty), 0)?;
                self.assign(m, scope, ty, e, &t, what)
                    .map_err(|stop| scope.place(stop))
            }
        }
    }

    /// A band: each row's value of the band's type, its argument in scope (refcore's
    /// `check_band`). L §5.9.
    fn check_band(&mut self, m: &Reality, band: &BandInfo, files: &Files) -> Result<(), Stop> {
        for table in &band.tables {
            let file = files.of_version(&table.version)?;
            for row in &table.rows {
                let mut scope = Scope::new(
                    Node::new(NodeKind::Band, &band.id),
                    Stands::Band,
                    file,
                    row.line,
                );
                let arg =
                    self.new_var(&band.arg, band.arg_type.clone(), VarDom::Arg, Vec::new())?;
                scope.vars.insert(band.arg.clone(), arg);
                let what = format!("a row of band {}", band.id);
                self.rule_value(m, &scope, &row.value, &band.ty, &what)?;
            }
        }
        Ok(())
    }

    /// An ask: the world it names a world of the run; an ask of a scenario's file names one;
    /// its bindings, its pins, `law` and `as known` (dates), a quantile's level, then what it
    /// asks (refcore's `check_ask`). L §5.18, §6.4.
    fn check_ask(&mut self, m: &Reality, ask: &Ask, file: &str) -> Result<(), Stop> {
        let mut scope = Scope::new(
            Node::new(NodeKind::Ask, &ask.id),
            Stands::Ask,
            file,
            ask.line,
        );
        if let Some(world) = &ask.world
            && !m.all_worlds.contains(world)
        {
            return Err(scope.place(unloaded_world(&ask.id, world)));
        }
        let diff = matches!(ask.body, AskBody::Diff(_));
        if m.scenarios.of(&ask.id).is_some() && ask.world.is_none() && !diff {
            let message = format!("ask {} in a scenario file names no world", ask.id);
            return Err(scope.place(refused(Load::World, message)));
        }
        for bind in &ask.binds {
            self.bind(m, &mut scope, bind, 0)?;
        }
        for pin in &ask.pins {
            self.pin(m, &scope, pin, 0)?;
        }
        for (clause, word) in [(&ask.law, "law"), (&ask.known, "as known")] {
            if let Some(e) = clause {
                let t = self.expr(m, &scope, e, None, 0)?;
                if !is_date(m, &t) {
                    return Err(scope.place(uerr(format!(
                        "{word} of ask {} is {}, not a date",
                        ask.id,
                        t.show()
                    ))));
                }
            }
        }
        if let Some(RChoice::Quantile(level)) = &ask.random {
            let mut levelled = scope.clone();
            levelled.allow_lit = true;
            self.expr(m, &levelled, level, None, 0)?;
        }
        self.ask_body(m, &scope, ask)
    }

    /// What an ask asks: values; `check a == b`; `explain` or `inputs-of` a reference;
    /// `readers-of` an id; `solve`; a `diff` between two worlds of the run; residuals
    /// (refcore's `check_ask`, its forms). L §5.18, §6.4.
    fn ask_body(&mut self, m: &Reality, scope: &Scope, ask: &Ask) -> Result<(), Stop> {
        match &ask.body {
            AskBody::Exprs(exprs) => {
                for e in exprs {
                    self.expr(m, scope, e, None, 0)?;
                }
                Ok(())
            }
            AskBody::Check(left, right) => {
                let compared = self.cmp(m, scope, left, CmpOp::Eq, right, 0);
                compared.map(|_| ()).map_err(|stop| scope.place(stop))
            }
            AskBody::Explain(reference) | AskBody::InputsOf(reference) => {
                self.expr(m, scope, reference, None, 0)?;
                let explain = matches!(ask.body, AskBody::Explain(_));
                let plain = matches!(reference, Expr::Name(_) | Expr::Post(..));
                let read = explain && matches!(reference, Expr::Bal(_) | Expr::Low(_));
                if !plain && !read {
                    let word = if explain { "explain" } else { "inputs-of" };
                    let message = format!("cannot parse: {word} takes a reference");
                    return Err(scope.place(refused(Load::Parse, message)));
                }
                Ok(())
            }
            AskBody::ReadersOf(reference) => {
                let Expr::Name(id) = reference else {
                    let message = "cannot parse: readers-of takes an id".to_owned();
                    return Err(scope.place(refused(Load::Parse, message)));
                };
                if !m.names.contains_key(id) {
                    return Err(
                        scope.place(refused(Load::Name, format!("unknown id {id}")).naming(id))
                    );
                }
                Ok(())
            }
            AskBody::Solve(solve) => self
                .check_solve(m, scope, solve, 0)
                .map(|_| ())
                .map_err(|stop| scope.place(stop)),
            AskBody::Diff(diff) => {
                for world in [&diff.world, &diff.against] {
                    if !m.all_worlds.contains(world) {
                        return Err(scope.place(unloaded_world(&ask.id, world)));
                    }
                }
                let mut inner = scope.clone();
                for bind in &diff.binds {
                    self.bind(m, &mut inner, bind, 0)?;
                }
                self.expr(m, &inner, &diff.expr, None, 0).map(|_| ())
            }
            AskBody::Residuals(residuals) => {
                self.expr(m, scope, &residuals.place, None, 0)?;
                self.span(m, scope, &residuals.span, 0)
            }
        }
    }

    /// A move: its binds, its day (read by its own day node), `at`, `for`, `when`, its legs
    /// and attributes against its kind's, `fulfils`, `ties by`, what it declares it reads,
    /// and its written order (refcore's `m_check_move`, `m_move_changes`). L §5.14, §5.14.1,
    /// §5.14.2.
    fn check_move(&mut self, m: &Reality, mv: &MoveInfo, file: &str) -> Result<MoveFacts, Stop> {
        let decl = &mv.decl;
        let lines = &decl.lines;
        let mut scope = Scope::new(
            Node::new(NodeKind::Move, &mv.id),
            Stands::Rule,
            file,
            decl.line,
        );
        for bind in &mv.binds {
            self.bind(m, &mut scope, &bind.node, 0)?;
        }
        scope.keyvars = mv.binds.iter().map(|bind| bind.var.clone()).collect();
        let day = scope.for_item(Node::new(NodeKind::MoveOn, &mv.id));
        self.edge(&scope.item, day.item.clone(), Rel::Any);
        let t = self.expr(m, &day.on_line(lines.on), &decl.on, None, 0)?;
        if !(is_date(m, &t) || matches!(t, Ty::Hole)) {
            let message = format!("on of move {} is {}, not a date", mv.id, t.show());
            return Err(scope.place(uerr(message)));
        }
        if let Some(at) = &decl.at {
            self.direct = Some(Vec::new());
            let law = self.expr(m, &day.on_line(lines.at), at, None, 0);
            if let Some(reads) = self.direct.take() {
                self.move_at.insert(mv.id.clone(), reads);
            }
            law_date(m, &law?, at, &format!("at of move {}", mv.id)).map_err(|s| scope.place(s))?;
        }
        if let Some(period) = &decl.for_period {
            self.expr(m, &scope.on_line(lines.for_period), period, None, 0)?;
        }
        if let Some(when) = &decl.when {
            let what = format!("when of move {}", mv.id);
            self.when(m, &scope.on_line(lines.when), &scope, when, &what)?;
        }
        leg_names(m, mv).map_err(|s| scope.place(s))?;
        let legs = m.kind_legs(&mv.kind);
        let mut changes = Vec::new();
        for leg in &decl.legs {
            if let Some((_, unit, _)) = legs.iter().find(|(name, ..)| *name == leg.name) {
                let lt = self.check_leg(m, &scope.on_line(leg.line), mv, leg, unit)?;
                self.leg_types.insert((mv.id.clone(), leg.name.clone()), lt);
            }
            let on_leg = scope.on_line(leg.line);
            for end in [&leg.from, &leg.to] {
                let change = change_of(m, &scope, end).map_err(|s| on_leg.place(s))?;
                push_once(&mut changes, change);
            }
        }
        self.attrs(m, &scope, mv)?;
        if let Some(fulfils) = &decl.fulfils {
            let ft = self.expr(m, &scope.on_line(lines.fulfils), fulfils, None, 0)?;
            if !is_move_type(m, &ft) {
                let message = format!("fulfils of {} names {}, not a movement", mv.id, ft.show());
                return Err(scope.place(uerr(message)));
            }
        }
        if let Some(ties) = &decl.ties {
            for tie in ties {
                self.expr(m, &scope.on_line(lines.ties), tie, None, 0)?;
            }
        }
        let mut read_targets = Vec::new();
        for read in &decl.reads {
            read_targets.push(self.read_target(m, &scope.on_line(lines.reads), mv, read)?);
        }
        for (order, other) in &decl.order {
            if m.names.kind(other) != Some(NameKind::Move) {
                let word = match order {
                    Order::Before => "before",
                    Order::After => "after",
                };
                let message = format!("unknown move {other} ({word} {other})");
                return Err(scope.place(refused(Load::Name, message)));
            }
        }
        if !decl.reads.is_empty() {
            let found = own_day_reads(mv).map_err(|stop| scope.place(stop))?;
            self.day_reads.extend(found);
        }
        Ok(MoveFacts {
            id: mv.id.clone(),
            read_targets,
            changes,
        })
    }

    /// A move's attributes: each its kind declares, once, of its type, and none missing
    /// (refcore's `m_check_move`, its attributes). L §5.14.
    fn attrs(&mut self, m: &Reality, scope: &Scope, mv: &MoveInfo) -> Result<(), Stop> {
        let declared = attr_types(m, &mv.kind);
        let lines = &mv.decl.lines.attrs;
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for (at, (attr, value)) in mv.decl.attrs.iter().enumerate() {
            let Some((_, ty)) = declared.iter().find(|(name, _)| name == attr) else {
                let message = format!("attribute {attr} is missing or unknown ({})", mv.id);
                return Err(scope.place(refused(Load::Leg, message)));
            };
            if !seen.insert(attr) {
                let message = format!("attribute {attr} twice in {}", mv.id);
                return Err(scope.place(refused(Load::Leg, message)));
            }
            let line = lines
                .get(at)
                .copied()
                .ok_or_else(|| invariant(format!("an attr of move {} with no line", mv.id)))?;
            let Some(ty) = ty else {
                return Err(invariant(format!(
                    "attribute {attr} of {} has no type",
                    mv.id
                )));
            };
            let t = self.expr(m, &scope.on_line(line), value, Some(ty), 0)?;
            let what = format!("attribute {attr} of {}", mv.id);
            self.assign(m, scope, ty, value, &t, &what)
                .map_err(|stop| scope.place(stop))?;
        }
        if let Some((attr, _)) = declared
            .iter()
            .find(|(name, _)| !seen.contains(name.as_str()))
        {
            let message = format!("attribute {attr} is missing or unknown ({})", mv.id);
            return Err(scope.place(refused(Load::Leg, message)));
        }
        Ok(())
    }

    /// What a `reads` line names: a move, all of a keyed place, an event kind, or the place
    /// an expression gives, `LOAD-READS` for anything else (refcore's `m_read_target`).
    /// L §5.14.1.
    fn read_target(
        &mut self,
        m: &Reality,
        scope: &Scope,
        mv: &MoveInfo,
        read: &Expr,
    ) -> Result<Target, Stop> {
        if let Expr::Name(id) = read
            && !scope.is_var(id)
        {
            let named = m.names.kind(id);
            if named == Some(NameKind::Move) {
                return Ok(Target::Move(id.clone()));
            }
            let keyed = m
                .places
                .get(id)
                .is_some_and(|place| !place.binds.is_empty());
            if named == Some(NameKind::Place) && keyed {
                return Ok(Target::PlaceSec(id.clone()));
            }
            let event = m
                .kinds
                .get(id)
                .is_some_and(|kind| kind.spec == KindSpec::Event);
            if named == Some(NameKind::Kind) && event {
                return Ok(Target::Kind(id.clone()));
            }
        }
        let t = self.expr(m, scope, read, Some(&Ty::Key("places".to_owned())), 0)?;
        if !is_place_type(m, &t) {
            let message = format!(
                "{} declares reads {}, which is not a place or a move",
                mv.id,
                super::types::text(read)
            );
            return Err(scope.place(refused(Load::Reads, message)));
        }
        place_target(m, scope, read).map_err(|s| scope.place(s))
    }
}

/// The literal rule on a four-digit figure by its reading. L §2.1, §7.1.
fn year4_rule(y: &Year4) -> Result<(), Stop> {
    match y.reading {
        Some(Reading::Year) => lit_rule(&y.scope, Figure::Date, &y.text, None),
        Some(Reading::Int) => lit_rule(&y.scope, Figure::Num, &y.text, Some(&y.value)),
        None => Ok(()),
    }
}

/// A law date `at e`: a date, not a period (refcore's `check_def`, `m_check_move`, their
/// `at`). L §4.3.
fn law_date(m: &Reality, t: &Ty, at: &Expr, what: &str) -> Result<(), Stop> {
    if matches!(t, Ty::Period(_)) || key_value_kind(m, t) == Some("period") {
        return Err(refused(
            Load::LawPeriod,
            format!("at {}: a period is not a law date", super::types::text(at)),
        ));
    }
    if !is_date(m, t) {
        return Err(uerr(format!("{what} is {}, not a date", t.show())));
    }
    Ok(())
}

/// A def's rules' `beats`: each names a rule of the def, at its line; the labels each beats
/// (refcore's `check_def`, its labels). L §5.11.
fn labels(def: &DefInfo, file: &str) -> Result<BTreeMap<String, BTreeSet<String>>, Stop> {
    let names: BTreeSet<&str> = def.rules.iter().map(|(label, _)| label.as_str()).collect();
    let mut beats: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (label, versions) in &def.rules {
        let mut beaten = BTreeSet::new();
        for version in versions {
            for beat in &version.rule.beats {
                if !names.contains(beat.as_str()) {
                    return Err(Stop::Refused(Refusal::at(
                        Code::Load(Load::Label),
                        format!("beats names no rule {beat}"),
                        At::Line(file.to_owned(), version.rule.line),
                    )));
                }
                beaten.insert(beat.clone());
            }
        }
        if !beaten.is_empty() {
            beats.insert(label.clone(), beaten);
        }
    }
    Ok(beats)
}

/// `LOAD-CYCLE` when a def's `beats` run in a cycle, the path from the first label in text
/// order, at the def's line (refcore's `_beats_cycle`, its depth-first walk as a worklist).
/// L §5.11, §8.3.
fn beats_cycle(
    def: &str,
    beats: &BTreeMap<String, BTreeSet<String>>,
    at: (&str, u32),
) -> Result<(), Stop> {
    let mut done: BTreeSet<String> = BTreeSet::new();
    for start in beats.keys() {
        if done.contains(start) {
            continue;
        }
        let mut path: Vec<(String, Vec<String>)> = vec![(start.clone(), beaten(beats, start))];
        while let Some((_, pending)) = path.last_mut() {
            let Some(next) = pending.pop() else {
                if let Some((finished, _)) = path.pop() {
                    done.insert(finished);
                }
                continue;
            };
            if path.iter().any(|(label, _)| *label == next) {
                let mut names: Vec<String> = path.into_iter().map(|(label, _)| label).collect();
                names.push(next);
                return Err(refuse(
                    Load::Cycle,
                    format!("a beats cycle in {def}: {}", names.join(" -> ")),
                    at.0,
                    at.1,
                ));
            }
            if !done.contains(&next) {
                let below = beaten(beats, &next);
                path.push((next, below));
            }
        }
    }
    Ok(())
}

/// The reads `after` holds that `before` didn't, each with its new relations: what an `at`
/// read (refcore's `check_def`, its `at_reads`). L §4.3.
fn new_reads(
    before: &BTreeMap<Node, Vec<Rel>>,
    after: Option<&BTreeMap<Node, Vec<Rel>>>,
) -> BTreeMap<Node, Vec<Rel>> {
    let mut out = BTreeMap::new();
    for (target, rels) in after.into_iter().flatten() {
        let old: &[Rel] = before.get(target).map_or(&[], Vec::as_slice);
        let added: Vec<Rel> = rels
            .iter()
            .filter(|rel| !old.contains(rel))
            .cloned()
            .collect();
        if !added.is_empty() {
            out.insert(target.clone(), added);
        }
    }
    out
}

/// Each rule's guards: the key variables its `when` tests, and those of the `when` of each
/// rule that beats it (refcore's `check_def`, its `when_vars` and `guard`). L §5.11.
fn rule_guards(
    def: &DefInfo,
    beats: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeMap<String, BTreeSet<String>> {
    let keyvars: Vec<String> = def.binds.iter().map(|bind| bind.var.clone()).collect();
    let mut when_vars: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for (label, versions) in &def.rules {
        let mut vars = BTreeSet::new();
        for version in versions {
            if let Some(when) = &version.rule.when {
                vars.extend(tested_vars(when, &keyvars));
            }
        }
        when_vars.insert(label, vars);
    }
    let mut out = BTreeMap::new();
    for (label, _) in &def.rules {
        let mut guard = when_vars
            .get(label.as_str())
            .map_or_else(BTreeSet::new, Clone::clone);
        for (beater, beaten) in beats {
            if beaten.contains(label)
                && let Some(vars) = when_vars.get(beater.as_str())
            {
                guard.extend(vars.iter().cloned());
            }
        }
        out.insert(label.clone(), guard);
    }
    out
}

/// The key variables a condition tests: each named in a side of a comparison or an `in`
/// (refcore's `tested_vars`). L §5.11.
pub(super) fn tested_vars(e: &Expr, keyvars: &[String]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut stack = vec![e];
    while let Some(next) = stack.pop() {
        let mut sides: Vec<&Expr> = Vec::new();
        if let Expr::Cmp(cmp) = next {
            sides.extend([&cmp.0, &cmp.2]);
        }
        if let Expr::In(test) = next {
            sides.extend([&test.member, &test.set]);
        }
        for side in sides {
            let mut below = vec![side];
            while let Some(x) = below.pop() {
                if let Expr::Name(name) = x
                    && keyvars.contains(name)
                {
                    out.insert(name.clone());
                }
                below.extend(children(x));
            }
        }
        stack.extend(children(next));
    }
    out
}

/// The labels `label` beats, last first, so that popping takes them in text order.
fn beaten(beats: &BTreeMap<String, BTreeSet<String>>, label: &str) -> Vec<String> {
    let Some(set) = beats.get(label) else {
        return Vec::new();
    };
    set.iter().rev().cloned().collect()
}

/// The integer offsets of a def's key arithmetic, `k + 1` or `k - 2` with `k` a key
/// variable (refcore's `check_def`, `item_offsets`). L §7.1.
fn key_offsets(def: &DefInfo) -> Offsets {
    let vars: BTreeSet<&str> = def.binds.iter().map(|bind| bind.var.as_str()).collect();
    let mut out = Offsets::new();
    let mut todo: Vec<&Expr> = Vec::new();
    for (_, versions) in &def.rules {
        for version in versions {
            todo.extend(version.rule.when.as_ref());
            if let RuleValue::Expr(e) = &version.rule.value {
                todo.push(e);
            }
        }
    }
    while let Some(e) = todo.pop() {
        if let Expr::Sum(first, rest) = e
            && let Expr::Name(var) = first.as_ref()
            && vars.contains(var.as_str())
            && let Some((_, Expr::Lit(Lit::Num(n)))) = rest.first()
            && n.whole
            && n.unit.is_none()
            && let Some(offset) = n.value.to_i64()
        {
            if let Some(set) = out.get_mut(var) {
                set.insert(offset);
            } else {
                out.insert(var.clone(), BTreeSet::from([offset]));
            }
        }
        todo.extend(children(e));
    }
    out
}

/// A move's legs: each its kind declares, once, and none missing (refcore's
/// `m_check_move`, its legs). L §5.14.
fn leg_names(m: &Reality, mv: &MoveInfo) -> Result<(), Stop> {
    let declared = m.kind_legs(&mv.kind);
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for leg in &mv.decl.legs {
        if !declared.iter().any(|(name, ..)| *name == leg.name) {
            return Err(refused(
                Load::Leg,
                format!("leg {} is not a leg of {}", leg.name, mv.kind),
            ));
        }
        if !seen.insert(&leg.name) {
            return Err(refused(
                Load::Leg,
                format!("leg {} twice in {}", leg.name, mv.id),
            ));
        }
    }
    if let Some((name, ..)) = declared
        .iter()
        .find(|(name, ..)| !seen.contains(name.as_str()))
    {
        return Err(refused(Load::Leg, format!("{} lacks leg {name}", mv.id)));
    }
    Ok(())
}

/// An event kind's attributes with their types, its parents' first (refcore's
/// `kind_attrs`). L §5.7.
fn attr_types(m: &Reality, kind: &str) -> Vec<(String, Option<Ty>)> {
    let mut out = Vec::new();
    for info in m.chain(kind).into_iter().rev() {
        out.extend(info.attr_types.iter().cloned());
    }
    out
}

/// How a read's keys relate to its reader's: only a def's reads relate; a read in a mode
/// takes the mode (refcore's `relation`). E §3.1 step 5.
pub(super) fn relation(scope: &Scope, keys: &[Key]) -> Rel {
    if let Some(mode) = &scope.mode {
        return mode.clone();
    }
    if !matches!(scope.item.kind, NodeKind::Def | NodeKind::Fill) {
        return Rel::Any;
    }
    let bound_here = |key: &Key| {
        if let Key::Expr(Expr::Name(name)) = key {
            scope.is_var(name) && !scope.keyvars.contains(name)
        } else {
            false
        }
    };
    if scope.under == Under::Aggregate && keys.iter().any(bound_here) {
        return Rel::Computed;
    }
    let mut shift = None;
    for key in keys {
        match key_relation(scope, key) {
            KeyRel::Same => {}
            KeyRel::Shifted(var, way) if shift.is_none() => shift = Some(Rel::Shifted(var, way)),
            KeyRel::Shifted(..) | KeyRel::Computed => return Rel::Computed,
        }
    }
    shift.unwrap_or(Rel::Same)
}

/// How one key of a read relates to its reader's keys.
#[derive(Debug, Clone, PartialEq, Eq)]
enum KeyRel {
    /// A key variable of the reader.
    Same,
    /// A key variable shifted by a number.
    Shifted(String, Shift),
    /// Anything else.
    Computed,
}

/// The offset of a recurrence read's key `k ± n` as written, the first key that is the
/// variable `var` plus or minus a number, else `n` (refcore's `shift_offset`). L §8.3
/// `LOAD-GUARD`.
fn shift_offset(keys: &[Key], var: &str) -> String {
    keys.iter()
        .find_map(|key| {
            if let Key::Expr(Expr::Sum(base, rest)) = key
                && let Expr::Name(name) = base.as_ref()
                && name == var
                && let [(_, Expr::Lit(Lit::Num(number)))] = rest.as_slice()
            {
                Some(number.text.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "n".to_owned())
}

/// One key's relation to the reader's keys (refcore's `relation`, its loop).
fn key_relation(scope: &Scope, key: &Key) -> KeyRel {
    let Key::Expr(expr) = key else {
        return KeyRel::Computed;
    };
    if let Expr::Name(name) = expr
        && scope.keyvars.contains(name)
    {
        return KeyRel::Same;
    }
    if let Expr::Sum(base, rest) = expr
        && let Expr::Name(name) = base.as_ref()
        && scope.keyvars.contains(name)
        && let [(op, Expr::Lit(Lit::Num(_)))] = rest.as_slice()
    {
        let way = match op {
            AddOp::Add => Shift::Later,
            AddOp::Sub => Shift::Earlier,
        };
        return KeyRel::Shifted(name.clone(), way);
    }
    KeyRel::Computed
}

/// A name with its postfixes, `x` or `x[k].f`.
pub(super) fn named_parts(expr: &Expr) -> Option<(&str, &[Postfix])> {
    if let Expr::Name(id) = expr {
        return Some((id, &[]));
    }
    if let Expr::Post(base, posts) = expr
        && let Expr::Name(id) = base.as_ref()
    {
        return Some((id, posts));
    }
    None
}

/// A `flows` target written as a move's may be: `m`, `m[k]`, `m.leg` or `m[k].leg`, with
/// its name and keys.
pub(super) fn move_keys(target: &Expr) -> Option<(&str, &[Key])> {
    let (id, posts) = named_parts(target)?;
    match posts {
        [] | [Postfix::Field(_)] => Some((id, &[])),
        [Postfix::Keys(keys)] | [Postfix::Keys(keys), Postfix::Field(_)] => Some((id, keys)),
        _ => None,
    }
}

/// The input or def `id`, and whether it is keyed.
pub(super) fn input_or_def(m: &Reality, id: &str) -> Option<(Node, bool)> {
    match m.names.kind(id)? {
        NameKind::Input => {
            let input = m.inputs.get(id)?;
            Some((Node::new(NodeKind::Input, id), !input.binds.is_empty()))
        }
        NameKind::Def => {
            let def = m.defs.get(id)?;
            Some((Node::new(NodeKind::Def, id), !def.binds.is_empty()))
        }
        NameKind::Unit
        | NameKind::Kind
        | NameKind::Index
        | NameKind::Axis
        | NameKind::Band
        | NameKind::Place
        | NameKind::Fill
        | NameKind::Move
        | NameKind::Version
        | NameKind::Ask
        | NameKind::Source
        | NameKind::Cut => None,
    }
}

/// The thing a place expression names literally: an unkeyed place's id, a thing of a place
/// kind, or a keyed place at literal keys (refcore's `m_literal_place`). L §5.10.
pub(super) fn literal_place(
    m: &Reality,
    scope: &Scope,
    expr: &Expr,
) -> Result<Option<String>, Stop> {
    match literal_form(m, scope, expr) {
        None => Ok(None),
        Some(Literal::Thing(thing)) => Ok(Some(thing)),
        Some(Literal::Keyed(id, lits)) => match keyed_place_thing(m, &id, &keyed(lits), "", 0) {
            Ok(thing) => Ok(thing),
            // A key no index or kind holds is refused where the key is typed (`LOAD-KEY`);
            // here it names no literal place. `LOAD-KEY` is the one refusal of a key.
            Err(Stop::Refused(r)) if r.code() == Code::Load(Load::Key) => Ok(None),
            Err(stop) => Err(stop),
        },
    }
}

/// Literal keys as keys written at a place's positions.
fn keyed(lits: Vec<Lit>) -> Vec<KeyLit> {
    lits.into_iter().map(KeyLit::Lit).collect()
}

/// A place expression read literally, before a keyed place's keys are looked up.
enum Literal {
    /// The thing it names.
    Thing(String),
    /// A keyed place, at keys written as literals.
    Keyed(String, Vec<Lit>),
}

/// What a place expression names literally, its keys not yet looked up (refcore's
/// `m_literal_place`, up to its `key_thing`). L §5.10.
fn literal_form(m: &Reality, scope: &Scope, expr: &Expr) -> Option<Literal> {
    let (id, posts) = named_parts(expr)?;
    if scope.is_var(id) {
        return None;
    }
    let named = m.names.kind(id);
    if named.is_none() && posts.is_empty() {
        let thing = m.things.get(id)?;
        return m.is_place(thing).then(|| Literal::Thing(id.to_owned()));
    }
    let place = m
        .places
        .get(id)
        .filter(|_| named == Some(NameKind::Place))?;
    if posts.is_empty() {
        let unkeyed = place.things.iter().find(|(keys, _)| keys.is_empty());
        return unkeyed.map(|(_, thing)| Literal::Thing(thing.clone()));
    }
    let [Postfix::Keys(keys)] = posts else {
        return None;
    };
    let mut lits = Vec::new();
    for key in keys {
        lits.push(key_literal(m, scope, key)?);
    }
    Some(Literal::Keyed(id.to_owned(), lits))
}

/// A key written as a literal at a key position: a word that is no variable and no id, a
/// key, a day, a month, a month-day, a currency, or a whole number with no unit (refcore's
/// `is_keylit_node`). L §2.5.
fn key_literal(m: &Reality, scope: &Scope, key: &Key) -> Option<Lit> {
    let Key::Expr(expr) = key else {
        return None;
    };
    if let Expr::Name(name) = expr {
        let literal = !scope.is_var(name) && !m.names.contains_key(name);
        return literal.then(|| Lit::Name(name.clone()));
    }
    let Expr::Lit(lit) = expr else {
        return None;
    };
    let keylike = match lit {
        Lit::Key(_) | Lit::Day(_) | Lit::Month(..) | Lit::MonthDay(..) | Lit::Cur(_) => true,
        Lit::Num(number) => number.whole && number.unit.is_none(),
        Lit::Quarter(..) | Lit::Name(_) | Lit::Bool(_) | Lit::Inf { .. } | Lit::Nominal(_) => false,
    };
    keylike.then(|| lit.clone())
}

/// The keyed place section a place expression reads, `p` or `p[k]` (refcore's `_r` of
/// `place`).
pub(super) fn place_section(m: &Reality, scope: &Scope, expr: &Expr) -> Option<String> {
    let (id, posts) = named_parts(expr)?;
    let read = posts.is_empty() || matches!(posts, [Postfix::Keys(_)]);
    let place = !scope.is_var(id) && m.names.kind(id) == Some(NameKind::Place);
    (read && place).then(|| id.to_owned())
}

/// What a running read or a declared read of a place names (refcore's `m_place_target`).
/// E §3.1 step 7.
pub(super) fn place_target(m: &Reality, scope: &Scope, expr: &Expr) -> Result<Target, Stop> {
    if let Some(thing) = literal_place(m, scope, expr)? {
        return Ok(Target::Thing(thing));
    }
    Ok(place_section(m, scope, expr).map_or(Target::Unknown, Target::PlaceSec))
}

/// What a leg's place changes: the place written, every place of a keyed place, every
/// place of the kind its type names, or a place load can't see (refcore's
/// `m_move_changes`). L §5.14.2.
fn change_of(m: &Reality, scope: &Scope, end: &Expr) -> Result<Target, Stop> {
    if let Some(thing) = literal_place(m, scope, end)? {
        return Ok(Target::Thing(thing));
    }
    if let Some(section) = place_section(m, scope, end) {
        return Ok(Target::PlaceSec(section));
    }
    Ok(place_kind(m, scope, end).map_or(Target::Unknown, Target::Kind))
}

/// The kind a place expression's type names, where load sees it: a variable bound over a
/// kind, a place, a thing, or an input or def of type `key K` (refcore's `_t.ix`).
fn place_kind(m: &Reality, scope: &Scope, expr: &Expr) -> Option<String> {
    let mut inner = expr;
    while let Expr::Paren(nested) = inner {
        inner = nested;
    }
    let (id, posts) = named_parts(inner)?;
    if posts.len() > 1 || posts.iter().any(|post| matches!(post, Postfix::Field(_))) {
        return None;
    }
    if let Some(var) = scope.vars.get(id) {
        return var.kind().map(str::to_owned);
    }
    let named = m.names.kind(id)?;
    let ty = match named {
        NameKind::Place => return m.places.get(id).map(|place| place.kind.clone()),
        NameKind::Input => m.inputs.get(id).and_then(|input| input.ty.as_ref()),
        NameKind::Def => m.defs.get(id).map(|def| &def.ty),
        NameKind::Unit
        | NameKind::Kind
        | NameKind::Index
        | NameKind::Axis
        | NameKind::Band
        | NameKind::Fill
        | NameKind::Move
        | NameKind::Version
        | NameKind::Ask
        | NameKind::Source
        | NameKind::Cut => None,
    };
    if let Some(Ty::Key(kind)) = ty
        && m.kinds.contains(kind)
    {
        return Some(kind.clone());
    }
    None
}

/// Adds `target` to `targets` unless it is there.
fn push_once(targets: &mut Vec<Target>, target: Target) {
    if !targets.contains(&target) {
        targets.push(target);
    }
}

#[cfg(test)]
mod tests {
    use super::{KeyRel, Scope, Stands, Under, Var, VarDom, beats_cycle, key_relation, relation};
    use crate::model::{Node, NodeKind, Rel, Shift, Ty};
    use crate::num::Num;
    use crate::syntax::{AddOp, Expr, Key, Lit, Number, Stop};
    use std::collections::{BTreeMap, BTreeSet};

    fn var(name: &str) -> Var {
        Var {
            name: name.to_owned(),
            ty: Ty::Int,
            dom: VarDom::Range,
            keylist: Vec::new(),
            id: 0,
            end: None,
        }
    }

    fn def_scope() -> Scope {
        let mut scope = Scope::new(Node::new(NodeKind::Def, "d"), Stands::Rule, "a.tally", 1);
        scope.vars.insert("k".to_owned(), var("k"));
        scope.keyvars.push("k".to_owned());
        scope
    }

    fn one() -> Expr {
        Expr::Lit(Lit::Num(Number {
            value: Num::int(1),
            whole: true,
            unit: None,
            text: "1".to_owned(),
            four: false,
            fraction: false,
        }))
    }

    #[test]
    fn a_def_relates_its_reads_keys_to_its_own() {
        let scope = def_scope();
        let at_k = Key::Expr(Expr::Name("k".to_owned()));
        let earlier = Key::Expr(Expr::Sum(
            Box::new(Expr::Name("k".to_owned())),
            vec![(AddOp::Sub, one())],
        ));
        assert_eq!(relation(&scope, &[]), Rel::Same, "no keys");
        assert_eq!(
            relation(&scope, core::slice::from_ref(&at_k)),
            Rel::Same,
            "at k"
        );
        assert_eq!(
            relation(&scope, core::slice::from_ref(&earlier)),
            Rel::Shifted("k".to_owned(), Shift::Earlier),
            "at k - 1"
        );
        assert_eq!(
            relation(&scope, &[earlier.clone(), earlier]),
            Rel::Computed,
            "two shifts"
        );
        assert_eq!(relation(&scope, &[Key::Star]), Rel::Computed, "a star");
        assert_eq!(key_relation(&scope, &at_k), KeyRel::Same, "at k itself");
        let mut under = scope.in_aggregate();
        under.vars.insert("x".to_owned(), var("x"));
        assert_eq!(under.under, Under::Aggregate, "under the aggregate");
        let bound = Key::Expr(Expr::Name("x".to_owned()));
        assert_eq!(
            relation(&under, &[bound]),
            Rel::Computed,
            "a bound variable"
        );
        assert_eq!(
            relation(
                &scope.with_mode(Rel::Explicit),
                core::slice::from_ref(&at_k)
            ),
            Rel::Explicit,
            "a mode wins"
        );
        let mover = Scope::new(Node::new(NodeKind::Move, "mv"), Stands::Rule, "a.tally", 1);
        assert_eq!(
            relation(&mover, &[at_k]),
            Rel::Any,
            "a move relates nothing"
        );
    }

    #[test]
    fn a_beats_cycle_is_named_from_the_first_label() {
        let set = |labels: &[&str]| -> BTreeSet<String> {
            labels.iter().map(|l| (*l).to_owned()).collect()
        };
        let mut beats = BTreeMap::new();
        beats.insert("base".to_owned(), set(&["waived"]));
        beats.insert("waived".to_owned(), set(&["base"]));
        let refused = match beats_cycle("fee", &beats, ("a.tally", 3)) {
            Err(Stop::Refused(refusal)) => refusal.text(),
            other => format!("{other:?}"),
        };
        assert_eq!(
            refused, "LOAD-CYCLE a beats cycle in fee: base -> waived -> base (a.tally:3)",
            "the cycle"
        );
        let mut chain = BTreeMap::new();
        chain.insert("a".to_owned(), set(&["b", "c"]));
        chain.insert("b".to_owned(), set(&["c"]));
        assert!(
            beats_cycle("fee", &chain, ("a.tally", 3)).is_ok(),
            "a chain is no cycle"
        );
    }
}
