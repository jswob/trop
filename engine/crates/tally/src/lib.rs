//! The Tally engine: a reality goes in as bytes, a result comes out (ENGINE §1).
//! Spec: E §1 (the run function); the modules name their own sections.
//! Twin: engine/tools/refcore/cli.py.
//! Invariants: the library reads no clock, environment or file (R-38, R-39); [`run`] answers
//! a read, parse or load refusal and the asks a loaded reality's evaluator can read, and
//! `NotImplemented` for anything past the part built (milestones S and M0).
//!
//! Milestone S built it in groups (`engine/BUILD-LOG.md`, the S plan): the exact numbers,
//! [`num`]; the code catalogue and refusals, [`base`]; the entry and its read step, [`run`];
//! days, [`time`]; the lexer and the parser, [`syntax`]; load, [`load`], into the model,
//! [`model`]; the read rule, [`read`]; the evaluator and the memo, [`eval`]; holes, [`hole`];
//! the walk, its checkpoints and the written projection, [`walk`]; observations, readings,
//! adjustments and the residual ledger, [`book`]; the asks, [`ask`]; E §9 invariant 4, asks
//! move no money, `invariant.rs`. `engine/ARCHITECTURE.md` is the map.

#![forbid(unsafe_code)]

mod api;
#[cfg(feature = "counters")]
mod counters;
mod invariant;
mod oracle;

pub use api::{
    Answer, Asks, Given, Pin, Request, Source, load_report, reprint, reprint_in, run, run_in,
    run_with_memo,
};
#[cfg(feature = "counters")]
pub use counters::Counters;
#[cfg(feature = "counters")]
pub(crate) use counters::Unit;
pub use oracle::Days;

/// The foundations every other module stands on: the catalogue of codes (L §8, E §11) and
/// the refusal that carries one. The module's code is its files under `base/`.
pub mod base {
    mod code;
    mod refusal;
    mod symbol;
    mod table;

    pub use code::{Code, Hole, Load, Req, Run, Warn};
    pub use refusal::{Place, Reach, Refusal};
    pub use symbol::{Internal, Interner, Symbol};
    pub use table::Table;
}

/// The text of a reality (L §2): its lines and tokens, and the tree the parser makes of
/// them. The lexer and the parser are the `lexer-parser` area. The module's code is its
/// files under `syntax/`.
pub mod syntax {
    mod ask;
    mod binds;
    mod calls;
    mod canon;
    mod canon_order;
    mod decls;
    mod expr;
    mod item;
    mod lex;
    mod parens;
    mod parse;
    mod reads;
    mod spacing;
    mod tree;

    pub use canon::{printed_hash, reprint, reprint_among};
    pub(crate) use expr::is_builtin;
    pub(crate) use item::{parse_lines, prescan_units};
    pub use item::{parse_reality, parse_split};
    pub(crate) use lex::cannot;
    pub use lex::{Canon, Kind, Line, Mark, Paren, Read, Role, Tok, split_lines};
    pub use parse::Stop;
    pub use tree::{
        AddOp, Agg, AggFn, Ask, AskBody, Axis, Bal, Band, BandRow, Bind, Calendar, CalendarDay,
        Call, Certainty, CmpOp, Crps, DaySet, Def, Diff, Dist, DistFn, DistFnKind, DistKind,
        Domain, Edge, Enter, Expr, Field, FieldValue, File, Fill, FillLines, Flows, Gap, Governs,
        Header, HeaderKind, In, Index, IndexForm, IndexFrom, Input, InputForm, Instant, Interval,
        Item, ItemForm, Key, KindDecl, KindSpec, Known, LSpan, LawDate, Leg, Lit, Low, Move,
        MoveLines, MulOp, Number, OfQ, OfRow, Order, Orient, PRef, PartsClause, Pin, PinForm,
        Place, Postfix, Price, Proj, Prorata, Query, QueryFn, RChoice, Residuals, Rest, Row,
        RowKey, Rule, RuleValue, Section, Sign, Site, SiteKey, Solve, Source, Span, Step, Takes,
        Type, TypedInput, Unit, UnitDecl, UnitForm, UnitTerm, Valuation, Value, Version,
    };
}

/// Days, periods and the calendar (L §4). The module's code is its files under `time/`.
pub mod time {
    mod calendar;
    mod day;
    mod period;

    pub use calendar::{Added, Calendar, FirstYear, weekday_number};
    pub use day::Day;
    pub use period::{Period, PeriodKind, Step, age, age_months, overlap_days};
}

/// The resolved reality load builds (E §3.1): units and types, kinds and things, indexes,
/// versions and their rows, and the declarations. The module's code is its files under
/// `model/`.
pub mod model {
    mod decl;
    mod graph;
    mod ids;
    mod index;
    mod key;
    mod kind;
    mod reality;
    mod ty;
    mod unit;
    mod value;
    mod world;

    pub use decl::{
        BandEnd, BandInfo, BandRow, BindInfo, DefInfo, Domain, Fulfils, InputInfo, MoveInfo,
        ObsLeg, Observation, PlaceInfo, Row, RowInstant, RuleVer, Table as BandTable, TimePos,
    };
    pub use graph::{Graph, Mark, Node, NodeKind, ReadRole, Rel, Shift, Target};
    pub use ids::{
        AxisId, BandId, DefId, FillId, IndexId, InputId, KindId, MoveId, Name, Names, PlaceId,
        SourceId, ThingId, VersionId,
    };
    pub use index::{GapRole, IndexForm, IndexInfo, Moved, OwnTime, VersionInfo};
    pub use key::{KeyVal, Pat, Span, pattern_text};
    pub use kind::{Entry, KindInfo, KindSpec, Labels, PRELUDE_KINDS, PlaceKeys, Thing, prelude};
    pub use reality::{
        BUILTIN_INDEXES, BUILTIN_INPUTS, Joint, MemberAsks, NameKind, Reality, SamePair,
    };
    pub use ty::{CodeKind, PeriodWord, Shown, Ty};
    pub use unit::{PERIODS, Scaled, Unit, UnitError, is_currency_code};
    pub use value::{
        Class, Code, MvRef, MvSrc, Number, Value, Window, currency, currency_id, currency_unit,
        money_num, num_money,
    };
    pub use world::Scenarios;
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
    mod bindings;
    mod builtin;
    mod business;
    mod convert;
    mod dates;
    mod expr;
    mod extremes;
    mod memo;
    mod ranges;
    mod read;
    mod rules;
    mod stack;
    mod state;
    mod ty;
    mod windows;
    mod years;

    pub use aggregate::Member;
    pub use arith::{Op, Test, arith, compare, currency_of, int_of, negate, same_value};
    pub(crate) use bindings::{order_text, reads_name};
    pub(crate) use convert::{ConvertFor, Converted, convert, convert_row, field_literal};
    pub use memo::Memo;
    pub(crate) use read::{make_keys, thing_seen, to_key};
    pub use stack::Item;
    pub use state::{Collected, Ctx, Engine, Entry, Env};
    pub(crate) use state::{a_day, num_stop, past_days};
    pub(crate) use years::year_form;
}

/// The walk (E §4): the plan pass's instances and their order (E §4.1), the day walk's
/// agenda, days, place states and the sign rule (E §4.2), the decisions at their points, the
/// balance and flows reads, and the written projection's own walk (E §4.3). The kernel's
/// (gate 8); the module's code is its files under `walk/`.
pub mod walk {
    mod agenda;
    mod day;
    mod flows;
    mod legs;
    mod order;
    mod plan;
    mod point;
    mod projection;
    mod reads;
    mod state;
    mod tuples;

    pub use agenda::{Act, Mv, Reading, endings, reading_point};
    pub use day::{Broke, Ends};
    pub use order::{Run, WalkAt};
    pub use plan::{
        Amount, Booked, Dated, End, Exists, Inst, LegVal, Order, Seed, Status, Term,
        kind_certainty, qty_value,
    };
    pub(crate) use plan::{invariant, quantity};
    pub use point::{At, Step};
    pub use projection::ProjKey;
    pub use state::{
        ALL, Change, Checkpoint, Ent, Group, Log, Progress, Projected, St, Taken, Today, Walk,
        WalkObs,
    };
}

/// The book (E §5, §6.1, §6.3): observations the cut sees, the places versions cover and the
/// links to the instances observations fulfil; a booked instance measured again at each later
/// version's cut, its adjustments and their settlements; the residual ledger. The kernel's
/// (gate 8); the module's code is its files under `book/`.
pub mod book {
    mod adjust;
    mod ledger;
    mod members;
    mod observe;
    mod reading;
    mod settle;

    pub use adjust::{Adjustment, Settled};
    pub use ledger::{Fig, Group, GroupKey, Ledger, Line, fig_text};
    pub(crate) use members::{MvDom, MvMember};
    pub use observe::{Coverage, Linked, Obs, inst_id};
    pub use reading::{Pad, Seen, first_readings, reading_rank, readings};
    pub(crate) use settle::settlement_move;
}

/// The asks (L §5.18): each ask's lines with the register under them (L §8.2), then the
/// run's warnings (L §8.5). The module's code is its files under `ask/`.
pub mod ask {
    mod answer;
    mod register;
    mod residuals;
    mod worlds;

    pub(crate) use answer::date_of;
    pub use answer::{Answered, answer};
    pub use register::{register_lines, warning_lines};
    pub use worlds::{answer_worlds, report_lines};
}

/// Load (E §3.1 steps 1–4): the run's files split and told into worlds, the base and each
/// scenario's; the parsed files become each world's reality, constructively; the load refusals
/// made are those construction meets, the rest are M0's later groups'.
pub mod load {
    mod across;
    mod bands;
    mod binds;
    mod calls;
    mod check;
    mod compare;
    mod cone;
    mod decls;
    mod defs;
    mod exact;
    mod fills;
    mod indexes;
    mod joins;
    mod joint;
    mod keyed;
    mod keys;
    mod kinds;
    mod lawdates;
    mod merge;
    mod moves;
    mod names;
    mod observed;
    mod ops;
    mod overlap;
    mod parts;
    mod pins;
    mod places;
    mod prefixed;
    mod prices;
    mod random;
    mod reads;
    mod rows;
    mod same;
    mod scenarios;
    mod spans;
    mod srows;
    mod steps;
    mod strata;
    mod targets;
    mod texts;
    mod types;
    mod values;
    mod worlds;

    pub use joint::member_paths;
    pub(crate) use joint::reprint_members;
    pub use keys::index_key_of;
    pub use steps::load;
    pub(crate) use texts::text as expr_text;
    pub use worlds::{Loaded, Split, Worlds, load_run, split_files};
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
    pub use gap::{
        Asked, Found, Question, member_span, no_candidate, origin, read_input, read_last,
    };
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

/// Reprints one file's bytes in the canonical format, as a reality of that one file named
/// `input` (L §2.4); the fuzz target `format` calls it: a text it prints reprints as itself.
///
/// # Errors
///
/// The refusal met (bytes that aren't UTF-8 are `LOAD-PARSE`), or a form whose order the
/// engine doesn't build yet.
pub fn format_file(source: &[u8]) -> Result<String, syntax::Stop> {
    let name = "input";
    let text = String::from_utf8(source.to_vec()).map_err(|_| base::Refusal::not_utf8(name))?;
    let printed = syntax::reprint(&[Source {
        path: name.to_owned(),
        text,
    }])?;
    Ok(printed
        .into_iter()
        .map(|(_, canonical)| canonical)
        .collect())
}
