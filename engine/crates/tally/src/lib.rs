//! The Tally engine: a reality goes in as bytes, a result comes out (ENGINE §1).
//! Spec: E §1 (the run function); the modules name their own sections.
//! Twin: engine/tools/refcore/cli.py.
//! Invariants: the library reads no clock, environment or file (R-38, R-39); [`run`] answers
//! a read or parse refusal, and `NotImplemented` past the parse until the checks and the
//! evaluator land (milestone S, groups G4 on).
//!
//! Milestone S is built in groups (`engine/BUILD-LOG.md`, the S plan). So far: the exact
//! numbers, [`num`]; the code catalogue and refusals, [`base`]; the entry and its read step,
//! [`run`]; days, [`time`]; the lexer and the parser, [`syntax`].

#![forbid(unsafe_code)]

mod api;

pub use api::{Answer, Asks, Given, Pin, Request, Source, run};

/// The foundations every other module stands on: the catalogue of codes (L §8, E §11) and
/// the refusal that carries one. The module's code is its files under `base/`.
pub mod base {
    mod code;
    mod refusal;

    pub use code::{Code, Hole, Load, Req, Run, Warn};
    pub use refusal::{Place, Refusal};
}

/// The text of a reality (L §2): its lines and tokens, and the tree the parser makes of
/// them. The lexer and the parser are the `lexer-parser` area. The module's code is its
/// files under `syntax/`.
pub mod syntax {
    mod expr;
    mod item;
    mod lex;
    mod parse;
    mod tree;

    pub use item::parse_reality;
    pub use lex::{Kind, Line, Tok, split_lines};
    pub use parse::Stop;
    pub use tree::{
        AddOp, Agg, AggFn, Ask, AskBody, Bal, Band, BandRow, Bind, Calendar, CalendarDay, Call,
        Certainty, CmpOp, DaySet, Def, Domain, Edge, Enter, Expr, Field, FieldValue, File, Flows,
        Gap, Governs, Header, HeaderKind, In, Index, IndexForm, IndexFrom, Input, InputForm,
        Instant, Interval, Item, ItemForm, Key, KindDecl, KindSpec, Known, LSpan, LawDate, Leg,
        Lit, Move, MulOp, Number, OfQ, Order, Place, Postfix, Price, Proj, Residuals, Row, RowKey,
        Rule, RuleValue, Section, Sign, Source, Span, Step, Type, TypedInput, Unit, UnitDecl,
        UnitForm, UnitTerm, Value, Version,
    };
}

/// Days, periods and the calendar (L §4). The module's code is its files under `time/`.
pub mod time {
    mod day;

    pub use day::Day;
}

/// Exact numbers (L §7.1, §7.2): `Num`, a reduced fraction held small or big; `Money`, a
/// whole number of a currency's minor units; rounding by the seven modes. The module's
/// code is its files under `num/`, the kernel's (gate 8); this block only names them.
pub mod num {
    mod big;
    mod gcd;
    mod money;
    mod rational;
    mod round;
    mod small;
    mod text;

    pub use money::{Currency, Money};
    pub use rational::{Num, NumError};
    pub use round::Mode;
}

#[cfg(all(feature = "fault-injection", not(debug_assertions)))]
compile_error!("the fault-injection feature exists only in builds with debug assertions (R-51)");
#[cfg(all(feature = "counters", not(debug_assertions)))]
compile_error!("the counters feature exists only in builds with debug assertions (R-51)");

/// The answer of an entry point past the part of the engine built so far.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotImplemented;

/// Splits and parses one file's bytes as a reality of that one file, named `input` (L §2.3,
/// §8.3); the library reads no files (R-39). The fuzz target `parse` calls it.
///
/// # Errors
///
/// The refusal met (bytes that aren't UTF-8 are `LOAD-PARSE`), or the first construct the
/// engine doesn't build yet.
pub fn parse(source: &[u8]) -> Result<Vec<syntax::File>, syntax::Stop> {
    let name = "input";
    let text = String::from_utf8(source.to_vec()).map_err(|_| base::Refusal::not_utf8(name))?;
    syntax::parse_reality(&[Source {
        path: name.to_owned(),
        text,
    }])
}

/// The work units PERFORMANCE §1.1 counts, behind the `counters` feature (D4-8,
/// BUILDER §13.2). The stub counts nothing, so every count is 0.
#[cfg(feature = "counters")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counters {
    /// Expression nodes evaluated (ev).
    pub ev: u64,
    /// Def cell reads that found the memo's value.
    pub def_hits: u64,
    /// Def cell reads that computed it.
    pub def_misses: u64,
    /// Input reads by the read rule.
    pub reads: u64,
    /// Distinct input reads.
    pub distinct_reads: u64,
    /// Days the day walk stepped through (walk days).
    pub days_stepped: u64,
    /// Stepped days with at least one act.
    pub nonempty_days: u64,
    /// Movement legs written to a place's log.
    pub legs_booked: u64,
    /// Groups of parts work at places that keep parts.
    pub parts_groups: u64,
    /// Timeline prefixes taken from a checkpoint instead of walked again.
    pub walks_shared: u64,
    /// Shared-layer cells a worker computed for itself.
    pub recomputed_locally: u64,
    /// Exact values made.
    pub num_results: u64,
}

#[cfg(feature = "counters")]
impl Counters {
    /// The `--stats` line on standard error: `stats: ` then `key=value` pairs, the form
    /// `engine/tools/perf counts` reads.
    #[must_use]
    pub fn line(&self) -> String {
        format!(
            "stats: ev={} def_hits={} def_misses={} reads={} distinct_reads={} days_stepped={} \
             nonempty_days={} legs_booked={} parts_groups={} walks_shared={} recomputed_locally={} \
             num_results={}\n",
            self.ev,
            self.def_hits,
            self.def_misses,
            self.reads,
            self.distinct_reads,
            self.days_stepped,
            self.nonempty_days,
            self.legs_booked,
            self.parts_groups,
            self.walks_shared,
            self.recomputed_locally,
            self.num_results,
        )
    }
}
