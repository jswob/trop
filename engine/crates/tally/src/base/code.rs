//! The catalogue of codes: every hole, refusal and warning code the engine prints, in one enum.
//! Spec: L §8.1 (holes), §8.3 (load), §8.4 (per value), §8.5 (warnings); E §11 (requests, the
//! invariant).
//! Twin: engine/tools/refcore/errors.py (refcore keeps its codes as strings).
//! Invariants: one variant per row of the spec's tables, each spelled as there; [`Code::ALL`]
//! lists each once, in the tables' order; every lookup is an exhaustive `match`.
//!
//! The codes are grouped by table, one enum per family, so each `match` stays short. The
//! builder's test `crates/tally/tests/catalogue.rs` compares [`Code::ALL`] with the tables (R-76).

/// A code of the catalogue: its family, then its row in that family's table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Code {
    /// A hole (L §8.1).
    Hole(Hole),
    /// A refusal at load (L §8.3).
    Load(Load),
    /// A refusal per value (L §8.4).
    Run(Run),
    /// A warning (L §8.5).
    Warn(Warn),
    /// A refused request (E §11).
    Req(Req),
    /// `ENGINE-INVARIANT`: a check of E §9 failed (E §11).
    EngineInvariant,
}

/// The holes of L §8.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hole {
    /// `HOLE-STATED`
    Stated,
    /// `HOLE-UNGOVERNED`
    Ungoverned,
    /// `HOLE-NOREADING`
    NoReading,
    /// `HOLE-NOTYET`
    NotYet,
    /// `HOLE-ANCHOR`
    Anchor,
    /// `HOLE-LOTS`
    Lots,
    /// `HOLE-PRICE`
    Price,
    /// `HOLE-CALENDAR`
    Calendar,
    /// `HOLE-IMPRECISE`
    Imprecise,
    /// `HOLE-CHANGED`
    Changed,
    /// `HOLE-SAME`
    Same,
    /// `HOLE-PRERUN`
    PreRun,
    /// `HOLE-PART`
    Part,
}

/// The refusals at load of L §8.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Load {
    /// `LOAD-PATH`
    Path,
    /// `LOAD-PARSE`
    Parse,
    /// `LOAD-HEADER`
    Header,
    /// `LOAD-VERSION`
    Version,
    /// `LOAD-PIN`
    Pin,
    /// `LOAD-RESERVED`
    Reserved,
    /// `LOAD-DUP`
    Dup,
    /// `LOAD-NAME`
    Name,
    /// `LOAD-SHADOW`
    Shadow,
    /// `LOAD-KEY`
    Key,
    /// `LOAD-AMBIG`
    Ambig,
    /// `LOAD-KIND`
    Kind,
    /// `LOAD-LIFESPAN`
    Lifespan,
    /// `LOAD-LEAF`
    Leaf,
    /// `LOAD-PLACEKIND`
    PlaceKind,
    /// `LOAD-PARTS`
    Parts,
    /// `LOAD-ALIVE`
    Alive,
    /// `LOAD-DOMAIN`
    Domain,
    /// `LOAD-UNIT`
    Unit,
    /// `LOAD-PRECISION`
    Precision,
    /// `LOAD-LITERAL`
    Literal,
    /// `LOAD-FIELD`
    Field,
    /// `LOAD-INSTANT`
    Instant,
    /// `LOAD-GOVERNS`
    Governs,
    /// `LOAD-KNOWN`
    Known,
    /// `LOAD-OVERLAP`
    Overlap,
    /// `LOAD-REPLACE`
    Replace,
    /// `LOAD-LABEL`
    Label,
    /// `LOAD-MOVES`
    Moves,
    /// `LOAD-GAPROLE`
    GapRole,
    /// `LOAD-BAND`
    Band,
    /// `LOAD-LAWDATE`
    LawDate,
    /// `LOAD-LAWPERIOD`
    LawPeriod,
    /// `LOAD-SCHEDULE`
    Schedule,
    /// `LOAD-CYCLE`
    Cycle,
    /// `LOAD-GUARD`
    Guard,
    /// `LOAD-TO`
    To,
    /// `LOAD-LEG`
    Leg,
    /// `LOAD-READS`
    Reads,
    /// `LOAD-ORDER`
    Order,
    /// `LOAD-PRICE`
    Price,
    /// `LOAD-COVERS`
    Covers,
    /// `LOAD-COVERED`
    Covered,
    /// `LOAD-SCENARIO`
    Scenario,
    /// `LOAD-WORLD`
    World,
    /// `LOAD-SAME`
    Same,
    /// `LOAD-RANDOM`
    Random,
    /// `LOAD-ASIF`
    AsIf,
    /// `LOAD-DEPTH`
    Depth,
    /// `LOAD-FORKS`
    Forks,
    /// `LOAD-ASKONLY`
    AskOnly,
    /// `LOAD-MISSING`
    Missing,
    /// `LOAD-ORIENT`
    Orient,
}

/// The refusals per value of L §8.4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Run {
    /// `RUN-NONE`
    None,
    /// `RUN-NOT-INT`
    NotInt,
    /// `RUN-KEY`
    Key,
    /// `RUN-CYCLE`
    Cycle,
    /// `RUN-CONFLICT`
    Conflict,
    /// `RUN-NO-RULE`
    NoRule,
    /// `RUN-LABEL`
    Label,
    /// `RUN-TIE`
    Tie,
    /// `RUN-EMPTY`
    Empty,
    /// `RUN-DIGITS`
    Digits,
    /// `RUN-DIV`
    Div,
    /// `RUN-DOMAIN`
    Domain,
    /// `RUN-UNDECIDED`
    Undecided,
    /// `RUN-PARAM`
    Param,
    /// `RUN-ZERO`
    Zero,
    /// `RUN-NEG`
    Neg,
    /// `RUN-ORDER`
    Order,
    /// `RUN-WEIGHTS`
    Weights,
    /// `RUN-PRORATA`
    Prorata,
    /// `RUN-PLACE`
    Place,
    /// `RUN-LIFESPAN`
    Lifespan,
    /// `RUN-FUTURE`
    Future,
    /// `RUN-SOLVE`
    Solve,
    /// `RUN-RECORD`
    Record,
}

/// The warnings of L §8.5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Warn {
    /// `WARN-NEGATIVE`
    Negative,
    /// `WARN-UNREAD`
    Unread,
    /// `WARN-LEGS`
    Legs,
    /// `WARN-LABELS`
    Labels,
    /// `WARN-STALE`
    Stale,
}

/// The refused requests of E §11.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Req {
    /// `REQ-ASK`
    Ask,
    /// `REQ-CLOCK`
    Clock,
    /// `REQ-PATHS`
    Paths,
}

impl Code {
    /// Every code, each once, in the order of the spec's tables: L §8.1, §8.3, §8.4, §8.5,
    /// then E §11.
    pub const ALL: [Code; 99] = [
        Code::Hole(Hole::Stated),
        Code::Hole(Hole::Ungoverned),
        Code::Hole(Hole::NoReading),
        Code::Hole(Hole::NotYet),
        Code::Hole(Hole::Anchor),
        Code::Hole(Hole::Lots),
        Code::Hole(Hole::Price),
        Code::Hole(Hole::Calendar),
        Code::Hole(Hole::Imprecise),
        Code::Hole(Hole::Changed),
        Code::Hole(Hole::Same),
        Code::Hole(Hole::PreRun),
        Code::Hole(Hole::Part),
        Code::Load(Load::Path),
        Code::Load(Load::Parse),
        Code::Load(Load::Header),
        Code::Load(Load::Version),
        Code::Load(Load::Pin),
        Code::Load(Load::Reserved),
        Code::Load(Load::Dup),
        Code::Load(Load::Name),
        Code::Load(Load::Shadow),
        Code::Load(Load::Key),
        Code::Load(Load::Ambig),
        Code::Load(Load::Kind),
        Code::Load(Load::Lifespan),
        Code::Load(Load::Leaf),
        Code::Load(Load::PlaceKind),
        Code::Load(Load::Parts),
        Code::Load(Load::Alive),
        Code::Load(Load::Domain),
        Code::Load(Load::Unit),
        Code::Load(Load::Precision),
        Code::Load(Load::Literal),
        Code::Load(Load::Field),
        Code::Load(Load::Instant),
        Code::Load(Load::Governs),
        Code::Load(Load::Known),
        Code::Load(Load::Overlap),
        Code::Load(Load::Replace),
        Code::Load(Load::Label),
        Code::Load(Load::Moves),
        Code::Load(Load::GapRole),
        Code::Load(Load::Band),
        Code::Load(Load::LawDate),
        Code::Load(Load::LawPeriod),
        Code::Load(Load::Schedule),
        Code::Load(Load::Cycle),
        Code::Load(Load::Guard),
        Code::Load(Load::To),
        Code::Load(Load::Leg),
        Code::Load(Load::Reads),
        Code::Load(Load::Order),
        Code::Load(Load::Price),
        Code::Load(Load::Covers),
        Code::Load(Load::Covered),
        Code::Load(Load::Scenario),
        Code::Load(Load::World),
        Code::Load(Load::Same),
        Code::Load(Load::Random),
        Code::Load(Load::AsIf),
        Code::Load(Load::Depth),
        Code::Load(Load::Forks),
        Code::Load(Load::AskOnly),
        Code::Load(Load::Missing),
        Code::Load(Load::Orient),
        Code::Run(Run::None),
        Code::Run(Run::NotInt),
        Code::Run(Run::Key),
        Code::Run(Run::Cycle),
        Code::Run(Run::Conflict),
        Code::Run(Run::NoRule),
        Code::Run(Run::Label),
        Code::Run(Run::Tie),
        Code::Run(Run::Empty),
        Code::Run(Run::Digits),
        Code::Run(Run::Div),
        Code::Run(Run::Domain),
        Code::Run(Run::Undecided),
        Code::Run(Run::Param),
        Code::Run(Run::Zero),
        Code::Run(Run::Neg),
        Code::Run(Run::Order),
        Code::Run(Run::Weights),
        Code::Run(Run::Prorata),
        Code::Run(Run::Place),
        Code::Run(Run::Lifespan),
        Code::Run(Run::Future),
        Code::Run(Run::Solve),
        Code::Run(Run::Record),
        Code::Warn(Warn::Negative),
        Code::Warn(Warn::Unread),
        Code::Warn(Warn::Legs),
        Code::Warn(Warn::Labels),
        Code::Warn(Warn::Stale),
        Code::Req(Req::Ask),
        Code::Req(Req::Clock),
        Code::Req(Req::Paths),
        Code::EngineInvariant,
    ];

    /// The code as the spec spells it, `LOAD-PATH`. L §8; E §11.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Code::Hole(hole) => hole.as_str(),
            Code::Load(load) => load.as_str(),
            Code::Run(run) => run.as_str(),
            Code::Warn(warn) => warn.as_str(),
            Code::Req(req) => req.as_str(),
            Code::EngineInvariant => "ENGINE-INVARIANT",
        }
    }
}

impl Hole {
    /// The code as L §8.1 spells it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Hole::Stated => "HOLE-STATED",
            Hole::Ungoverned => "HOLE-UNGOVERNED",
            Hole::NoReading => "HOLE-NOREADING",
            Hole::NotYet => "HOLE-NOTYET",
            Hole::Anchor => "HOLE-ANCHOR",
            Hole::Lots => "HOLE-LOTS",
            Hole::Price => "HOLE-PRICE",
            Hole::Calendar => "HOLE-CALENDAR",
            Hole::Imprecise => "HOLE-IMPRECISE",
            Hole::Changed => "HOLE-CHANGED",
            Hole::Same => "HOLE-SAME",
            Hole::PreRun => "HOLE-PRERUN",
            Hole::Part => "HOLE-PART",
        }
    }
}

impl Load {
    /// The code as L §8.3 spells it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Load::Path => "LOAD-PATH",
            Load::Parse => "LOAD-PARSE",
            Load::Header => "LOAD-HEADER",
            Load::Version => "LOAD-VERSION",
            Load::Pin => "LOAD-PIN",
            Load::Reserved => "LOAD-RESERVED",
            Load::Dup => "LOAD-DUP",
            Load::Name => "LOAD-NAME",
            Load::Shadow => "LOAD-SHADOW",
            Load::Key => "LOAD-KEY",
            Load::Ambig => "LOAD-AMBIG",
            Load::Kind => "LOAD-KIND",
            Load::Lifespan => "LOAD-LIFESPAN",
            Load::Leaf => "LOAD-LEAF",
            Load::PlaceKind => "LOAD-PLACEKIND",
            Load::Parts => "LOAD-PARTS",
            Load::Alive => "LOAD-ALIVE",
            Load::Domain => "LOAD-DOMAIN",
            Load::Unit => "LOAD-UNIT",
            Load::Precision => "LOAD-PRECISION",
            Load::Literal => "LOAD-LITERAL",
            Load::Field => "LOAD-FIELD",
            Load::Instant => "LOAD-INSTANT",
            Load::Governs => "LOAD-GOVERNS",
            Load::Known => "LOAD-KNOWN",
            Load::Overlap => "LOAD-OVERLAP",
            Load::Replace => "LOAD-REPLACE",
            Load::Label => "LOAD-LABEL",
            Load::Moves => "LOAD-MOVES",
            Load::GapRole => "LOAD-GAPROLE",
            Load::Band => "LOAD-BAND",
            Load::LawDate => "LOAD-LAWDATE",
            Load::LawPeriod => "LOAD-LAWPERIOD",
            Load::Schedule => "LOAD-SCHEDULE",
            Load::Cycle => "LOAD-CYCLE",
            Load::Guard => "LOAD-GUARD",
            Load::To => "LOAD-TO",
            Load::Leg => "LOAD-LEG",
            Load::Reads => "LOAD-READS",
            Load::Order => "LOAD-ORDER",
            Load::Price => "LOAD-PRICE",
            Load::Covers => "LOAD-COVERS",
            Load::Covered => "LOAD-COVERED",
            Load::Scenario => "LOAD-SCENARIO",
            Load::World => "LOAD-WORLD",
            Load::Same => "LOAD-SAME",
            Load::Random => "LOAD-RANDOM",
            Load::AsIf => "LOAD-ASIF",
            Load::Depth => "LOAD-DEPTH",
            Load::Forks => "LOAD-FORKS",
            Load::AskOnly => "LOAD-ASKONLY",
            Load::Missing => "LOAD-MISSING",
            Load::Orient => "LOAD-ORIENT",
        }
    }
}

impl Run {
    /// The code as L §8.4 spells it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Run::None => "RUN-NONE",
            Run::NotInt => "RUN-NOT-INT",
            Run::Key => "RUN-KEY",
            Run::Cycle => "RUN-CYCLE",
            Run::Conflict => "RUN-CONFLICT",
            Run::NoRule => "RUN-NO-RULE",
            Run::Label => "RUN-LABEL",
            Run::Tie => "RUN-TIE",
            Run::Empty => "RUN-EMPTY",
            Run::Digits => "RUN-DIGITS",
            Run::Div => "RUN-DIV",
            Run::Domain => "RUN-DOMAIN",
            Run::Undecided => "RUN-UNDECIDED",
            Run::Param => "RUN-PARAM",
            Run::Zero => "RUN-ZERO",
            Run::Neg => "RUN-NEG",
            Run::Order => "RUN-ORDER",
            Run::Weights => "RUN-WEIGHTS",
            Run::Prorata => "RUN-PRORATA",
            Run::Place => "RUN-PLACE",
            Run::Lifespan => "RUN-LIFESPAN",
            Run::Future => "RUN-FUTURE",
            Run::Solve => "RUN-SOLVE",
            Run::Record => "RUN-RECORD",
        }
    }
}

impl Warn {
    /// The code as L §8.5 spells it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Warn::Negative => "WARN-NEGATIVE",
            Warn::Unread => "WARN-UNREAD",
            Warn::Legs => "WARN-LEGS",
            Warn::Labels => "WARN-LABELS",
            Warn::Stale => "WARN-STALE",
        }
    }
}

impl Req {
    /// The code as E §11 spells it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Req::Ask => "REQ-ASK",
            Req::Clock => "REQ-CLOCK",
            Req::Paths => "REQ-PATHS",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Code;

    #[test]
    fn every_code_is_listed_once() {
        for (i, code) in Code::ALL.iter().enumerate() {
            let again = Code::ALL
                .iter()
                .skip(i.checked_add(1).expect("i + 1"))
                .any(|c| c.as_str() == code.as_str());
            assert!(!again, "{} is listed twice", code.as_str());
        }
    }

    #[test]
    fn a_code_is_its_family_then_a_word() {
        for code in Code::ALL {
            let text = code.as_str();
            let family = text.split('-').next().expect("a family");
            assert!(
                ["HOLE", "LOAD", "RUN", "WARN", "REQ", "ENGINE"].contains(&family),
                "{text}"
            );
            assert!(
                text.bytes().all(|b| b.is_ascii_uppercase() || b == b'-'),
                "{text}"
            );
        }
    }
}
