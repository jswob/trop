//! The Tally engine: a reality goes in as bytes, a result comes out (ENGINE §1).
//! Spec: E §1 (the run function); the modules name their own sections.
//! Twin: engine/tools/refcore/cli.py.
//! Invariants: the library reads no clock, environment or file (R-38, R-39); [`run`] answers
//! a read or parse refusal and the asks a loaded reality's evaluator can read, and
//! `NotImplemented` for a load refusal and anything past the part built (milestone S).
//!
//! Milestone S is built in groups (`engine/BUILD-LOG.md`, the S plan). So far: the exact
//! numbers, [`num`]; the code catalogue and refusals, [`base`]; the entry and its read step,
//! [`run`]; days, [`time`]; the lexer and the parser, [`syntax`]; load, [`load`]; the read
//! rule, [`read`]; the evaluator, [`eval`]; holes, [`hole`]; the asks, [`ask`].

#![forbid(unsafe_code)]

mod api;

pub use api::{Answer, Asks, Given, Pin, Request, Source, run};

/// The foundations every other module stands on: the catalogue of codes (L §8, E §11) and
/// the refusal that carries one. The module's code is its files under `base/`.
pub mod base {
    mod code;
    mod refusal;
    mod table;

    pub use code::{Code, Hole, Load, Req, Run, Warn};
    pub use refusal::{Place, Refusal};
    pub use table::Table;
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
        UnitForm, UnitTerm, Valuation, Value, Version,
    };
}

/// Days, periods and the calendar (L §4). The module's code is its files under `time/`.
pub mod time {
    mod calendar;
    mod day;
    mod period;

    pub use calendar::{Calendar, weekday_number};
    pub use day::Day;
    pub use period::{Period, PeriodKind, Step, age, age_months, overlap_days};
}

/// The resolved reality load builds (E §3.1): units and types, kinds and things, indexes,
/// versions and their rows, and the declarations. The module's code is its files under
/// `model/`.
pub mod model {
    mod decl;
    mod graph;
    mod index;
    mod key;
    mod kind;
    mod reality;
    mod ty;
    mod unit;
    mod value;

    pub use decl::{
        BandEnd, BandInfo, BandRow, BindInfo, DefInfo, Domain, Fulfils, InputInfo, MoveInfo,
        ObsLeg, Observation, PlaceInfo, Row, RowInstant, RuleVer, Table as BandTable, TimePos,
    };
    pub use graph::{Graph, Mark, Node, NodeKind, Rel, Shift, Target};
    pub use index::{GapRole, IndexForm, IndexInfo, OwnTime, VersionInfo};
    pub use key::{KeyVal, Pat, Span, pattern_text};
    pub use kind::{Entry, KindInfo, KindSpec, PRELUDE_KINDS, PlaceKeys, Thing, prelude};
    pub use reality::{BUILTIN_INDEXES, BUILTIN_INPUTS, NameKind, Reality};
    pub use ty::{CodeKind, PeriodWord, Shown, Ty};
    pub use unit::{PERIODS, Scaled, Unit, is_currency_code};
    pub use value::{
        Class, Code, Number, Value, currency, currency_id, currency_unit, money_num, num_money,
    };
}

/// Holes (L §8.1): a missing fact and the questions it carries, built only here (R-116), with
/// the one rule every operator applies to them. The kernel's (gate 8); the module's code is
/// its files under `hole/`.
pub mod hole {
    mod questions;

    pub use questions::{Hole, spread};
}

/// The evaluator (E §3.2): expressions over the reality, def cells in the memo, rules, bands,
/// aggregates and built-ins. The module's code is its files under `eval/`.
pub mod eval {
    mod aggregate;
    mod arith;
    mod band;
    mod builtin;
    mod expr;
    mod read;
    mod rules;
    mod stack;
    mod state;
    mod ty;

    pub use aggregate::Member;
    pub use arith::{Op, Test, arith, compare, currency_of, int_of, negate, same_value};
    pub use stack::Item;
    pub(crate) use state::num_stop;
    pub use state::{Collected, Ctx, Engine, Entry, Env};
}

/// The asks (L §5.18): each ask's lines with the register under them (L §8.2), then the
/// run's warnings (L §8.5). The module's code is its files under `ask/`.
pub mod ask {
    mod answer;
    mod register;

    pub use answer::answer;
    pub use register::{register_lines, warning_lines};
}

/// Load (E §3.1 steps 1–4): the parsed files become the reality, constructively; the load
/// refusals S makes are those construction meets, the rest are M0's.
pub mod load {
    mod decls;
    mod graph;
    mod indexes;
    mod keys;
    mod kinds;
    mod observed;
    mod places;
    mod rows;
    mod spans;
    mod steps;
    mod strata;

    pub use keys::index_key_of;
    pub use steps::load;
}

/// The read rule (L §4.6): which row a read of an input, a def's givens, a dated rule or a
/// band takes, and what a read with none gives. The kernel's (gate 8); the module's code
/// is its files under `read/`, a plain scan (the row index is fence F3, opened when a
/// budget row asks).
pub mod read {
    mod bands;
    mod gap;
    mod rows;
    mod rules;

    pub use bands::{BandAt, band_no_row, band_no_table, band_origin, band_rows, table_held};
    pub use gap::{Asked, Found, Question, member_span, no_candidate, origin, read_input};
    pub use rows::{
        Clocks, Point, Test, candidates, choose, held, lawless, need_law, row_known, row_place,
        row_span, time_point,
    };
    pub use rules::rule_in_force;
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
