//! The evaluator's state: a read's context, the variables in scope, the register entries a cell collects, and the engine that holds the memo.
//! Spec: E §3.2 (every value is a cell: an item, its keys and a context; the memo maps cells to
//! results, a value or a refusal, with the register entries the cell met); L §4.3 (the law
//! date), §4.5 (the cut), §8.2 (the register).
//! Twin: engine/tools/refcore/evaluate.py (`Ctx`, `Coll`, `Engine` and its collector stack).
//! Invariants: a cell is computed once per context and its result, refusal included, is kept;
//! a cell read while it is being computed is `RUN-CYCLE`; recursion through def cells is
//! counted, and past its limit the cell demanded waits on the work stack (R-16).

use std::collections::{BTreeMap, BTreeSet};

use crate::base::{Code, Refusal, Run};
use crate::model::{Domain, KeyVal, Reality, Value};
use crate::num::NumError;
use crate::syntax::Stop;
use crate::time::Day;

/// The deepest chain of def cells computed by plain calls; past it the cell demanded waits on
/// the work stack (R-16, BUILDER §4). U-15, measured at S: a debug build uses about 19 KB of
/// stack per def cell, so 32 cells take about 0.6 MB of the 2 MiB thread T's runner gives a
/// run, leaving the rest for expressions between cells and the walk's frames.
pub(super) const DEPTH_LIMIT: u32 = 32;

/// A read's context (refcore's `Ctx`, the parts S builds).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ctx {
    /// The law date d; none for a read with no law date (L §4.3).
    pub law: Option<Day>,
    /// The cut K: what the read sees (L §4.5).
    pub cut: Day,
}

impl Ctx {
    /// The context with law date `law`.
    #[must_use]
    pub fn at_law(self, law: Option<Day>) -> Ctx {
        Ctx { law, ..self }
    }

    /// The context as a memo key's text.
    pub(super) fn key(self) -> String {
        let law = self.law.map_or_else(|| "-".to_owned(), Day::text);
        format!("{law}|{}", self.cut.text())
    }
}

/// A bound variable: its name, its value, and the domain it ranges over when it is a key
/// (refcore's static `key ix` type of the variable).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Var {
    /// The name.
    pub name: String,
    /// The value.
    pub value: Value,
    /// The index or kind it is a key of.
    pub domain: Option<Domain>,
}

/// What a bare literal is where it stands: in a def, a rule or an ask only `0`, `1` and `-1`
/// are bare figures; in a band's row every bare literal is (L §3.2 Inline figures, §5.9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Figures {
    /// A def, a rule, a move or an ask.
    Rule,
    /// A band's row.
    Band,
}

/// The variables in scope: each name with its value, the latest bound last.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Env {
    /// The bindings, the innermost last.
    vars: Vec<Var>,
    /// How bare literals read here.
    pub figures: Figures,
}

impl Env {
    /// No variables, in a def's scope.
    #[must_use]
    pub fn empty() -> Env {
        Env {
            vars: Vec::new(),
            figures: Figures::Rule,
        }
    }

    /// A band row's scope: its argument bound, every bare literal a figure (L §5.9).
    #[must_use]
    pub fn band(arg: &str, x: Value) -> Env {
        Env {
            vars: vec![Var {
                name: arg.to_owned(),
                value: x,
                domain: None,
            }],
            figures: Figures::Band,
        }
    }

    /// The variable `name`, the innermost binding first.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Var> {
        self.vars.iter().rev().find(|v| v.name == name)
    }

    /// This scope with `name` bound to `value` too.
    #[must_use]
    pub fn with(&self, name: &str, value: Value, domain: Option<Domain>) -> Env {
        let mut out = self.clone();
        out.bind(name, value, domain);
        out
    }

    /// Binds `name` to `value` in this scope.
    pub fn bind(&mut self, name: &str, value: Value, domain: Option<Domain>) {
        self.vars.push(Var {
            name: name.to_owned(),
            value,
            domain,
        });
    }
}

/// A register entry a read meets (L §8.2): its word (`assumed`, `held`, `given`, `missing`),
/// its text and the figure it was met at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The register word.
    pub word: &'static str,
    /// Its text.
    pub text: String,
    /// The figure, or the empty text.
    pub figure: String,
}

impl Ord for Entry {
    /// The set's order: the word, the text, the figure, by code point.
    fn cmp(&self, other: &Entry) -> core::cmp::Ordering {
        self.word
            .cmp(other.word)
            .then_with(|| self.text.cmp(&other.text))
            .then_with(|| self.figure.cmp(&other.figure))
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Entry) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// What a cell's evaluation met: its register entries (refcore's `Coll.reg`), and whether a
/// `missing(…)` read directly in it held (refcore's `Coll.missing`, never merged outward).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Collected {
    /// The register entries.
    pub entries: BTreeSet<Entry>,
    /// Whether a `missing(r)` evaluated in this collector, not in a cell below it, was true.
    pub missing: bool,
}

impl Collected {
    /// Adds the register entries `other` met (its `missing` stays its own).
    pub fn merge(&mut self, other: &Collected) {
        self.entries.extend(other.entries.iter().cloned());
    }
}

/// A memo cell: the result of a def at its keys in one context, with what it met.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Cell {
    /// The value, or the refusal.
    pub result: Result<Value, Stop>,
    /// What its evaluation met.
    pub met: Collected,
}

/// A def cell waiting on the work stack: demanded past the counted depth, it is computed from
/// the outermost read, with the cells that were being computed above it still in progress.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Task {
    /// The def.
    pub def: String,
    /// Its keys.
    pub keys: Vec<KeyVal>,
    /// The cell's context, its law date set by the def's `at`.
    pub ctx: Ctx,
    /// The cells in progress that unwound so it could be computed from depth 0: they stay in
    /// progress until it is, then the read waiting on it replays them.
    pub unwound: Vec<String>,
}

/// The evaluator: the memo, the collector stack and the work stack (refcore's `Engine`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Engine {
    /// The memo: a cell's key to its result.
    pub(super) memo: BTreeMap<String, Cell>,
    /// The cells being computed.
    pub(super) active: BTreeSet<String>,
    /// The collectors, the innermost last; never empty.
    pub(super) stack: Vec<Collected>,
    /// The def cells open now.
    pub(super) depth: u32,
    /// The cells demanded past the counted depth, the latest last (R-16).
    pub(super) work: Vec<Task>,
    /// The ask's cut, past which a row read for a later day is `held` (L §8.2).
    pub(super) ask_cut: Day,
}

impl Engine {
    /// An engine with an empty memo, for an ask whose cut is `ask_cut`.
    #[must_use]
    pub fn new(ask_cut: Day) -> Engine {
        Engine {
            memo: BTreeMap::new(),
            active: BTreeSet::new(),
            stack: vec![Collected::default()],
            depth: 0,
            work: Vec::new(),
            ask_cut,
        }
    }

    /// Records a register entry in the innermost collector (L §8.2).
    pub(super) fn note(&mut self, word: &'static str, text: String, figure: String) {
        if let Some(top) = self.stack.last_mut() {
            top.entries.insert(Entry { word, text, figure });
        }
    }

    /// Marks that a `missing(…)` read in the innermost collector held (L §8.2 `missing`).
    pub(super) fn note_missing(&mut self) {
        if let Some(top) = self.stack.last_mut() {
            top.missing = true;
        }
    }

    /// Opens a collector.
    pub(super) fn push(&mut self) {
        self.stack.push(Collected::default());
    }

    /// Closes the innermost collector and returns what it met, without adding it to the one
    /// outside.
    pub(super) fn pop_alone(&mut self) -> Collected {
        if self.stack.len() > 1
            && let Some(top) = self.stack.pop()
        {
            return top;
        }
        Collected::default()
    }

    /// Adds what a closed collector met to the innermost one.
    pub(super) fn absorb(&mut self, met: &Collected) {
        if let Some(top) = self.stack.last_mut() {
            top.merge(met);
        }
    }

    /// What the outermost collector has met so far: the register of the reads made.
    #[must_use]
    pub fn collected(&self) -> Collected {
        match self.stack.first() {
            Some(outer) => outer.clone(),
            None => Collected::default(),
        }
    }
}

/// A refusal with its code and message (L §8.4).
pub(super) fn refuse(code: Run, message: String) -> Stop {
    Stop::Refused(Refusal::new(Code::Run(code), message))
}

/// A load refusal met at run, as refcore raises it there (L §8.3).
pub(super) fn refuse_load(code: crate::base::Load, message: String) -> Stop {
    Stop::Refused(Refusal::new(Code::Load(code), message))
}

/// A value of a type its place doesn't take: `LOAD-UNIT` with `what` and the value as it
/// prints (refcore raises it where load's checker would have).
pub(super) fn wrong_type(m: &Reality, what: &str, v: &Value) -> Stop {
    match v.text(m) {
        Ok(text) => refuse_load(crate::base::Load::Unit, format!("{what} {text}")),
        Err(error) => num_stop(error),
    }
}

/// A broken promise of load's or the evaluator's: `ENGINE-INVARIANT` (E §11).
pub(super) fn invariant(what: &str) -> Stop {
    Stop::Refused(Refusal::new(Code::EngineInvariant, what.to_owned()))
}

/// A number error as the refusal refcore raises (L §7.1, §8.4).
pub(crate) fn num_stop(error: NumError) -> Stop {
    match error {
        NumError::Div => refuse(Run::Div, "division by zero".to_owned()),
        NumError::Digits => refuse(
            Run::Digits,
            "a value passes 4 096 digits; round where the law rounds".to_owned(),
        ),
        NumError::Domain => refuse(Run::Domain, "outside its operation's domain".to_owned()),
        NumError::Invariant => invariant("the number kernel's invariant"),
    }
}
