//! The tree the parser makes of a file: owned enums and structs, one per production it builds.
//! Spec: L §2.3 (the grammar), §2.4 (canonical order, for later), §3.1 (types).
//! Twin: engine/tools/refcore/parser.py (its `Node` kinds and fields) and node.py.
//! Invariants: chains of `or`, `and`, `+ -`, `* /` and postfixes are flat vectors, so no
//! derived walk over a long chain recurses (BUILDER §3); each line-level node keeps its line.
//!
//! A node's line is the line it was read from; an expression's place is its line's.

use crate::num::Num;
use crate::time::Day;

/// One term of a unit: a currency, a declared unit, a period word or `%`, with its power.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitTerm {
    /// The unit's name (a period word in its singular form).
    pub base: String,
    /// Its power as written, a whole number of any size, negative under `/` (load holds it
    /// to 64 in size, L §7.6 item 4).
    pub power: Num,
}

/// A unit written after a number or in a type: its terms in order (L §2.1 Units).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unit {
    /// The terms, in the order written.
    pub terms: Vec<UnitTerm>,
}

/// A number literal: its exact value and how it was written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Number {
    /// The value, the sign applied.
    pub value: Num,
    /// Whether it was written as an integer.
    pub whole: bool,
    /// Its unit, if written.
    pub unit: Option<Unit>,
    /// The text as written, with its sign.
    pub text: String,
    /// Whether it is four digits with no unit, so it may read as a year (L §2.1).
    pub four: bool,
    /// Whether it was written as a fraction `a/b` (L §2.1).
    pub fraction: bool,
}

/// A literal (L §2.3 `lit`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lit {
    /// A number with an optional unit.
    Num(Number),
    /// A day.
    Day(Day),
    /// A month: year and month.
    Month(u16, u8),
    /// A quarter: year and quarter.
    Quarter(u16, u8),
    /// A month-day: month and day.
    MonthDay(u8, u8),
    /// A key token (`ty2014-15`, `half-up`).
    Key(String),
    /// A currency code.
    Cur(String),
    /// `true` or `false`.
    Bool(bool),
    /// `inf`, or `-inf` when `neg`.
    Inf {
        /// Whether it is `-inf`.
        neg: bool,
    },
    /// The compounding code `nominal n`.
    Nominal(Num),
    /// A word read as an id or a key (`pl`, `ryczalt`).
    Name(String),
}

impl Lit {
    /// The literal's text as a key's source text compares it (refcore's `resolve_text`).
    /// L §2.5.
    #[must_use]
    pub fn text(&self) -> String {
        match self {
            Lit::Num(n) => n.text.clone(),
            Lit::Day(day) => day.text(),
            Lit::Month(year, month) => format!("{year:04}-{month:02}"),
            Lit::Quarter(year, quarter) => format!("{year:04}-Q{quarter}"),
            Lit::MonthDay(month, day) => format!("{month:02}-{day:02}"),
            Lit::Key(text) | Lit::Cur(text) | Lit::Name(text) => text.clone(),
            Lit::Bool(value) => value.to_string(),
            Lit::Inf { neg } => if *neg { "-inf" } else { "inf" }.to_owned(),
            Lit::Nominal(_) => "nominal".to_owned(),
        }
    }
}

/// A key position in brackets: an expression, or `*` for every key there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    /// `*`.
    Star,
    /// An expression.
    Expr(Expr),
}

/// One postfix of a chain: keys in brackets, or a field after `.`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Postfix {
    /// `[k, …]`.
    Keys(Vec<Key>),
    /// `.name`.
    Field(String),
}

/// An operator of a sum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddOp {
    /// `+`.
    Add,
    /// `-`.
    Sub,
}

/// An operator of a product.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MulOp {
    /// `*`.
    Mul,
    /// `/`.
    Div,
}

/// A comparison operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    /// `==`.
    Eq,
    /// `!=`.
    Ne,
    /// `<`.
    Lt,
    /// `<=`.
    Le,
    /// `>`.
    Gt,
    /// `>=`.
    Ge,
}

/// The aggregates of L §5.11.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggFn {
    /// `sum`.
    Sum,
    /// `count`.
    Count,
    /// `min`.
    Min,
    /// `max`.
    Max,
    /// `any`.
    Any,
    /// `all`.
    All,
    /// `first`.
    First,
    /// `last`.
    Last,
    /// `argmin`.
    Argmin,
    /// `argmax`.
    Argmax,
}

impl AggFn {
    /// The aggregate's word, as written and as messages name it. L §5.11.
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            AggFn::Sum => "sum",
            AggFn::Count => "count",
            AggFn::Min => "min",
            AggFn::Max => "max",
            AggFn::Any => "any",
            AggFn::All => "all",
            AggFn::First => "first",
            AggFn::Last => "last",
            AggFn::Argmin => "argmin",
            AggFn::Argmax => "argmax",
        }
    }
}

/// An expression (L §2.3 `expr`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    /// A literal.
    Lit(Lit),
    /// A text literal.
    Str(String),
    /// A name: an id, a bound variable or a key.
    Name(String),
    /// `none`.
    None,
    /// `never`.
    Never,
    /// `cut`, the ask's cut.
    Cut,
    /// `if c then a else b`.
    If(Box<[Expr; 3]>),
    /// `a or b or …`, two terms or more.
    Or(Vec<Expr>),
    /// `a and b and …`, two terms or more.
    And(Vec<Expr>),
    /// `not e`.
    Not(Box<Expr>),
    /// `a op b`.
    Cmp(Box<(Expr, CmpOp, Expr)>),
    /// `a in b [on d]`.
    In(Box<In>),
    /// `a ± b ± …`.
    Sum(Box<Expr>, Vec<(AddOp, Expr)>),
    /// `a × b ÷ …`.
    Prod(Box<Expr>, Vec<(MulOp, Expr)>),
    /// `-e`.
    Neg(Box<Expr>),
    /// A base with its postfixes: keys and fields, `a.b[k].c`.
    Post(Box<Expr>, Vec<Postfix>),
    /// A call of a built-in or a band.
    Call(Box<Call>),
    /// A leg's amount read of a party or batch, `m.cash.amount of p`.
    LegOf(Box<Expr>, Box<OfQ>),
    /// `e to u`.
    To(Box<Expr>, Box<Step>),
    /// `(e)`, kept for the canonical reprint.
    Paren(Box<Expr>),
    /// A record, `(a: 1, b: x)`.
    Record(Vec<Field>),
    /// A key list, `[a, b]`.
    List(Vec<Lit>),
    /// An aggregate.
    Agg(Box<Agg>),
    /// `last(x, d)`, a reading's carry.
    Last(Box<(Expr, Expr)>),
    /// A balance read.
    Bal(Box<Bal>),
    /// `flows(x, span …)`.
    Flows(Box<Flows>),
    /// `covered(x, span)`.
    Covered(Box<(Expr, Span)>),
    /// A calendar value.
    Calendar(Box<Calendar>),
    /// `value(x, d) in CUR [by p] [of q]`.
    Value(Box<Valuation>),
    /// A query: `mean`, `share` or `quantile` (L §5.11, §5.16).
    Query(Box<Query>),
    /// `low(x, span)` with its projection's clauses (L §5.14.1).
    Low(Box<Low>),
    /// `crps(…)`, a forecast's score (L §5.15).
    Crps(Box<Crps>),
    /// `cdf`, `pdf` or `pmf` of a random value at a point (L §5.16).
    DistFn(Box<DistFn>),
    /// A distribution (L §5.16).
    Dist(Box<Dist>),
    /// `prorata(…)` (L §5.13).
    Prorata(Box<Prorata>),
    /// `e as if {pins}` (L §5.11, parse rule 2).
    AsIf(Box<Expr>, Vec<Pin>),
}

/// Which query (L §5.11, §5.16).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryFn {
    /// `mean`.
    Mean,
    /// `share`.
    Share,
    /// `quantile`.
    Quantile,
}

/// `mean(e for binds)`, `share(e where c)`, `quantile(e …, q)`: over keys with `for`, else a
/// random value's query (L §2.3 `query`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    /// Which query.
    pub func: QueryFn,
    /// Its argument.
    pub arg: Expr,
    /// `for` and its bindings: a query over keys.
    pub binds: Option<Vec<Bind>>,
    /// `where c`.
    pub filter: Option<Expr>,
    /// A quantile's level.
    pub level: Option<Expr>,
}

/// `low(x, span)` and its projection's clauses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Low {
    /// The place.
    pub place: Expr,
    /// The span.
    pub span: Span,
    /// The projection's clauses.
    pub proj: Proj,
}

/// `crps(e [weight w] for bind, y)` or `crps(e, y)` (L §2.3 `crps`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Crps {
    /// The forecast.
    pub forecast: Expr,
    /// `weight w`.
    pub weight: Option<Expr>,
    /// `for bind`.
    pub bind: Option<Bind>,
    /// The outcome.
    pub outcome: Expr,
}

/// Which distribution function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistFnKind {
    /// `cdf`.
    Cdf,
    /// `pdf`.
    Pdf,
    /// `pmf`.
    Pmf,
}

/// `cdf(e, x)`, `pdf(e, x)`, `pmf(e, x)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistFn {
    /// Which function.
    pub func: DistFnKind,
    /// The random value.
    pub value: Expr,
    /// The point.
    pub at: Expr,
}

/// Which distribution (L §5.16).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistKind {
    /// `normal`.
    Normal,
    /// `lognormal`.
    Lognormal,
    /// `exponential`.
    Exponential,
    /// `gamma`.
    Gamma,
    /// `uniform`.
    Uniform,
    /// `poisson`.
    Poisson,
    /// `binomial`.
    Binomial,
    /// `negbin`.
    Negbin,
    /// `discrete(e for bind)`.
    Discrete,
}

/// A distribution: `normal(a, b)` and the like, or `discrete(e for bind)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dist {
    /// Which distribution.
    pub func: DistKind,
    /// Its arguments (none for `discrete`).
    pub args: Vec<Expr>,
    /// `discrete`'s term.
    pub term: Option<Expr>,
    /// `discrete`'s binding.
    pub bind: Option<Bind>,
}

/// What `prorata` does with the rest (L §5.13).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rest {
    /// `rest none`.
    None,
    /// `rest last`.
    Last,
    /// `rest largest`.
    Largest,
}

/// `prorata(total, w for binds, k, u, MODE, rest none|last)` or
/// `prorata(total, w for binds, k, u, rest largest [ties by …])` (L §2.3 `prorata`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prorata {
    /// The total shared.
    pub total: Expr,
    /// The weight.
    pub weight: Expr,
    /// The bindings the weight runs over.
    pub binds: Vec<Bind>,
    /// The key whose share is read.
    pub key: Expr,
    /// The unit the shares round to.
    pub unit: Step,
    /// The rounding mode, unless the rest goes to the largest.
    pub mode: Option<Expr>,
    /// Where the rest goes.
    pub rest: Rest,
    /// `ties by` after `rest largest`.
    pub ties: Option<Vec<Expr>>,
}

/// A random clause's choice, `random at mean` or `random at quantile(q)` (L §5.16).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RChoice {
    /// `mean`.
    Mean,
    /// `quantile(q)`.
    Quantile(Expr),
}

/// A pin of an ask or an `as if` (L §2.3 `pin`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pin {
    /// What it pins.
    pub form: PinForm,
    /// The pin as written, for the register.
    pub text: String,
}

/// What a pin pins.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinForm {
    /// `law e`.
    Law(Expr),
    /// `known e`.
    Known(Expr),
    /// `wording e for [ids]`.
    Wording(Expr, Vec<String>),
    /// `scenario s`.
    Scenario(String),
    /// `pref = e`, or `sample pref = e`; `ID = key` sets an axis's option.
    Set {
        /// What is pinned.
        target: PRef,
        /// Its value.
        value: Expr,
        /// Whether `sample` was written.
        sample: bool,
    },
}

/// A pinned reference: `ID [["." ID] "[" pk {"," pk} "]"] {"." FIELD}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PRef {
    /// The id.
    pub id: String,
    /// The member, `a.x[k]`.
    pub qual: Option<String>,
    /// Its keys, `*` for every key there.
    pub keys: Option<Vec<Key>>,
    /// Its fields.
    pub fields: Vec<String>,
}

/// `solve f == g for x in [lo, hi] to u` (L §5.18).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Solve {
    /// The left side.
    pub left: Expr,
    /// The right side.
    pub right: Expr,
    /// What is solved for.
    pub target: Expr,
    /// The bracket's low end.
    pub lo: Lit,
    /// The bracket's high end.
    pub hi: Lit,
    /// The precision.
    pub unit: Step,
}

/// `value(x, d) in CUR [by p] [of q]`: a place's holding valued in a currency (L §2.3 `valuex`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Valuation {
    /// The place.
    pub place: Expr,
    /// The day.
    pub day: Expr,
    /// The currency it is valued in.
    pub currency: String,
    /// `by p` or `by last(p)`.
    pub by: Option<Price>,
    /// `of q`.
    pub of: Option<OfQ>,
}

/// `a in b [on d]`: a membership test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct In {
    /// The member.
    pub member: Expr,
    /// The set.
    pub set: Expr,
    /// The day, for a kind.
    pub on: Option<Expr>,
}

/// A call: a built-in's name or a band's id, with the prefixes that name a band.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    /// The name called.
    pub name: String,
    /// The prefixes before the band's id (`a.b(x)`).
    pub prefix: Vec<String>,
    /// The key of a keyed band, `b[k](x)`.
    pub band_key: Option<Key>,
    /// The arguments.
    pub args: Vec<Expr>,
}

/// An aggregate, `sum(t for k in ks)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Agg {
    /// Which aggregate.
    pub func: AggFn,
    /// Its term, absent for a bare binding.
    pub term: Option<Expr>,
    /// Its bindings.
    pub binds: Vec<Bind>,
    /// Its `ties by` terms.
    pub ties: Option<Vec<Expr>>,
}

/// A balance read, `bal(x[, d])` with its clauses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bal {
    /// The place read.
    pub place: Expr,
    /// The day (and a unit's day) arguments.
    pub args: Vec<Expr>,
    /// Whether `start` was written: the balance at the start of the day.
    pub start: bool,
    /// Whether `modelled` was written.
    pub modelled: bool,
    /// The projection clauses.
    pub proj: Proj,
    /// The party or batch read, `of q`.
    pub of: Option<OfQ>,
}

/// A projection's clauses (L §5.14.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proj {
    /// `from t`.
    pub from: Option<Expr>,
    /// `windows each`.
    pub windows_each: bool,
    /// `random at …`.
    pub random: Option<RChoice>,
    /// `with uncertain`.
    pub uncertain: bool,
}

/// `flows(x, span [, for k]) [in CUR at fx mode]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flows {
    /// The place or move.
    pub target: Expr,
    /// The span.
    pub span: Span,
    /// `for k`.
    pub for_key: Option<Expr>,
    /// The conversion: currency, rate and rounding mode.
    pub convert: Option<(String, Expr, Expr)>,
}

/// `of q`: a party, a batch, or the pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OfQ {
    /// One reference.
    Ref(Expr),
    /// `(a, b)`.
    Pair(Expr, Expr),
}

/// A unit for `to` and `round` (L §7.2 `UNIT`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// A positive number with a unit, or `1e-N`.
    Number(Number),
    /// `minor(CUR)`.
    Minor(String),
    /// A reference.
    Ref(Expr),
}

/// A span: `a .. b`, `a ..`, `.. b`, `..`, or one period.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// The start.
    pub from: Option<Expr>,
    /// The end.
    pub to: Option<Expr>,
    /// Whether `..` was written.
    pub dots: bool,
}

/// A binding, `k in ks [on d | over s] [where c]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bind {
    /// The bound variable.
    pub var: String,
    /// What it ranges over.
    pub domain: Domain,
    /// `on d`.
    pub on: Option<Expr>,
    /// `over s`.
    pub over: Option<Span>,
    /// `where c`.
    pub filter: Option<Expr>,
    /// The binding as written, for the register.
    pub text: String,
}

/// What a binding ranges over.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Domain {
    /// A key list, `[a, b]`.
    List(Vec<Lit>),
    /// A range, `a .. b [step s]`.
    Range(Box<(Expr, Expr, Option<Expr>)>),
    /// An index or kind, with a member's prefix.
    Name {
        /// The index or kind.
        id: String,
        /// The member, `a.ix`.
        qual: Option<String>,
    },
}

/// A record's field: a name and a value, or a leg.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Field {
    /// `name: value`.
    Named(String, FieldValue),
    /// `leg l from a to b amount q`.
    Leg(Box<Leg>),
}

/// A field's value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldValue {
    /// A signed literal of a row.
    Lit(Lit),
    /// `?"…"`.
    Hole(String),
    /// `none`.
    None,
    /// An expression.
    Expr(Expr),
}

/// A leg: `leg l from a to b amount q [on d]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Leg {
    /// The leg's name.
    pub name: String,
    /// The place it leaves.
    pub from: Expr,
    /// The place it enters.
    pub to: Expr,
    /// Its amount.
    pub amount: Expr,
    /// Its own day.
    pub on: Option<Expr>,
    /// Its line: a move's `leg` line, or the line of the row whose record holds it.
    pub line: u32,
}

/// A calendar value (L §4.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Calendar {
    /// The weekend days.
    pub weekends: Vec<String>,
    /// The yearly month-days.
    pub yearly: Vec<(u8, u8)>,
    /// The year the yearly days start.
    pub yearly_from: Option<Num>,
    /// The added days.
    pub added: Vec<CalendarDay>,
}

/// An added day of a calendar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalendarDay {
    /// A day.
    Day(Day),
    /// `?"…"`.
    Hole(String),
    /// A window or a choice of days.
    Set(DaySet),
}

/// A day known only within a set (L §4.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaySet {
    /// `window(a, b)`.
    Window(Day, Day),
    /// `one_of([a, …])`.
    OneOf(Vec<Day>),
}

/// A version's or row's span of literals (L §2.3 `lspan`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LSpan {
    /// The start.
    pub from: Option<Lit>,
    /// The end.
    pub to: Option<Lit>,
    /// Whether `..` was written.
    pub dots: bool,
    /// An end not yet known, `..?"…"` (a lifespan).
    pub to_hole: Option<String>,
}

/// `governs`: a span, or `always`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Governs {
    /// `always`.
    Always,
    /// A span.
    Span(LSpan),
}

/// `known`: a day, or a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Known {
    /// A day.
    Day(Day),
    /// `window(a, b)`.
    Window(Day, Day),
}

/// A type (L §3.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// `money CUR`.
    Money(String),
    /// `num [unit]`.
    Num(Option<Unit>),
    /// `dist [unit]`.
    Dist(Option<Unit>),
    /// `key ix`.
    Key(String),
    /// A type named by one word: `int`, `bool`, `date`, ….
    Word(String),
    /// A record type, `(a int, b date)`.
    Record(Vec<(String, Type)>),
}

/// A price: a reference, or `last(ref)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Price {
    /// The price input.
    pub input: Expr,
    /// Whether it is `last(…)`.
    pub last: bool,
}

/// A file: its header and its sections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    /// The path, as the run names it.
    pub path: String,
    /// The header.
    pub header: Header,
    /// The sections, in the order written.
    pub sections: Vec<Section>,
}

/// What a header declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderKind {
    /// `model`.
    Model,
    /// `part`.
    Part,
    /// `scenario`.
    Scenario,
}

/// A file's header (L §2.3 `header`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    /// Model, part or scenario.
    pub kind: HeaderKind,
    /// Its id.
    pub id: String,
    /// A scenario's base.
    pub over: Option<String>,
    /// The tally version.
    pub tally: Num,
    /// `on`.
    pub on: Option<Day>,
    /// `start`.
    pub start: Option<Day>,
    /// `end`.
    pub end: Option<Day>,
    /// `use a = "file" sha256 "…"`.
    pub uses: Vec<(String, String, String)>,
    /// `same p = q`.
    pub sames: Vec<(Expr, Expr)>,
    /// Notes.
    pub notes: Vec<String>,
    /// `law`, a law part.
    pub law: bool,
    /// A scenario's `from`.
    pub from: Option<Day>,
    /// Its line.
    pub line: u32,
}

/// A section (L §2.3 `section`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Section {
    /// `source`.
    Source(Box<Source>),
    /// `version`.
    Version(Box<Version>),
    /// `unit`.
    Unit(Box<UnitDecl>),
    /// `kind`.
    Kind(Box<KindDecl>),
    /// `index`.
    Index(Box<Index>),
    /// `place`.
    Place(Box<Place>),
    /// `input`.
    Input(Box<Input>),
    /// `band`.
    Band(Box<Band>),
    /// `def`.
    Def(Box<Def>),
    /// `move`.
    Move(Box<Move>),
    /// `ask`.
    Ask(Box<Ask>),
    /// `axis`.
    Axis(Box<Axis>),
    /// `fill`.
    Fill(Box<Fill>),
}

impl Section {
    /// Its id and line.
    #[must_use]
    pub fn id_line(&self) -> (&str, u32) {
        match self {
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

    /// The key binds of its head: an item's, or an ask's `for` (refcore's `binds` of a
    /// section).
    #[must_use]
    pub fn binds(&self) -> &[Bind] {
        match self {
            Section::Place(s) => &s.binds,
            Section::Input(s) => &s.binds,
            Section::Band(s) => &s.binds,
            Section::Def(s) => &s.binds,
            Section::Move(s) => &s.binds,
            Section::Ask(s) => &s.binds,
            Section::Fill(s) => &s.binds,
            Section::Source(_)
            | Section::Version(_)
            | Section::Unit(_)
            | Section::Kind(_)
            | Section::Index(_)
            | Section::Axis(_) => &[],
        }
    }
}

/// `axis id` and its options, each a key and its label (L §5.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Axis {
    /// Its id.
    pub id: String,
    /// Its options, in the order written: the key and its label.
    pub options: Vec<(String, String)>,
    /// Its line.
    pub line: u32,
}

/// `fill id [binds]` and its lines (L §5.12).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fill {
    /// Its id.
    pub id: String,
    /// Its key binds.
    pub binds: Vec<Bind>,
    /// `v V`.
    pub v: Option<String>,
    /// Notes.
    pub notes: Vec<String>,
    /// `supply e`.
    pub supply: Expr,
    /// The supply's bindings.
    pub supply_binds: Vec<Bind>,
    /// `demand e`.
    pub demand: Expr,
    /// The demand's bindings.
    pub demand_binds: Vec<Bind>,
    /// `arc e`.
    pub arc: Expr,
    /// `order e, …`.
    pub order: Vec<Expr>,
    /// `ties by` after the order.
    pub ties: Option<Vec<Expr>>,
    /// Its line.
    pub line: u32,
    /// The lines of its `supply`, `demand`, `arc` and `order`, where a refusal in each is put.
    pub lines: FillLines,
}

/// The lines of a fill's `supply`, `demand`, `arc` and `order` (L §5.12).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FillLines {
    /// `supply e`.
    pub supply: u32,
    /// `demand e`.
    pub demand: u32,
    /// `arc e`.
    pub arc: u32,
    /// `order e, …`.
    pub order: u32,
}

/// `source id "title" [at "address"]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// Its id.
    pub id: String,
    /// Its title.
    pub title: String,
    /// Its address.
    pub at: Option<String>,
    /// Its line.
    pub line: u32,
}

/// `unit …`: a currency with its minor unit, or a named unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitForm {
    /// `unit CUR: currency minor m`.
    Currency(Lit),
    /// `unit id "label"`.
    Named(String),
}

/// A unit's declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitDecl {
    /// The currency code or the unit's id.
    pub id: String,
    /// Its form.
    pub form: UnitForm,
    /// Its line.
    pub line: u32,
}

/// Certain or uncertain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Certainty {
    /// `certain`.
    Certain,
    /// `uncertain`.
    Uncertain,
}

/// A place kind's sign rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    /// `signed`.
    Signed,
    /// `min 0`.
    Min0,
    /// `max 0`.
    Max0,
}

/// What a kind is (L §2.3 `kspec`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KindSpec {
    /// A kind with no spec.
    Plain,
    /// A subkind of `parent`.
    Sub {
        /// The parent kind.
        parent: String,
        /// Its certainty, if stated.
        certainty: Option<Certainty>,
    },
    /// `event certain|uncertain`.
    Event(Certainty),
    /// `unit`.
    Unit,
    /// `place …`.
    Place {
        /// Its unit (a currency, a unit, `any` or `one`).
        unit: String,
        /// Its sign rule.
        sign: Sign,
        /// Its price.
        priced: Option<Price>,
        /// Whether `outside`.
        outside: bool,
    },
}

/// A kind's declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KindDecl {
    /// Its id.
    pub id: String,
    /// What it is.
    pub spec: KindSpec,
    /// Its legs, with their units.
    pub legs: Vec<(String, String)>,
    /// Its attributes, with their types.
    pub attrs: Vec<(String, Type)>,
    /// Notes.
    pub notes: Vec<String>,
    /// The parts clauses, each with its line, in the order written (L §3.6, §5.7).
    pub parts: Vec<(PartsClause, u32)>,
    /// Its line.
    pub line: u32,
}

/// A parts clause of a place kind (L §2.3 `kline`, §3.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartsClause {
    /// `owned by D [ties by …]`: the arrival rule D.
    Owned {
        /// The arrival rule.
        rule: String,
        /// `ties by`.
        ties: Option<Vec<Expr>>,
    },
    /// `batched`.
    Batched,
    /// `step UNIT`.
    Step(Step),
    /// `takes K for W [v V] (order e, … [prorata [ties by …]] | prorata [ties by …])`.
    Takes(Box<Takes>),
}

/// A `takes` clause: which part an outflow takes, in which order (L §3.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Takes {
    /// The bound part.
    pub part: String,
    /// The bound outflow.
    pub outflow: String,
    /// `v V`.
    pub v: Option<String>,
    /// `order e, …`.
    pub order: Vec<Expr>,
    /// Whether `prorata` was written.
    pub prorata: bool,
    /// `ties by` after `prorata`.
    pub ties: Option<Vec<Expr>>,
}

/// An index's form (L §2.3 `index`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexForm {
    /// `: open`.
    Open,
    /// `= [a, b]`.
    List(Vec<Lit>),
    /// `= [period] lo .. hi [by n] [from f]`.
    Range {
        /// The period word, if written.
        period: Option<String>,
        /// The first key.
        lo: Lit,
        /// The last key.
        hi: Lit,
        /// `by n`.
        by: Option<Num>,
        /// `from MMDD` or `from n`.
        from: Option<IndexFrom>,
    },
}

/// An anchored index's `from`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexFrom {
    /// A month-day.
    MonthDay(u8, u8),
    /// A day of the month.
    Day(Num),
}

/// An index's declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Index {
    /// Its id.
    pub id: String,
    /// Its form.
    pub form: IndexForm,
    /// `v`.
    pub v: Option<String>,
    /// `assumed`.
    pub assumed: Option<String>,
    /// Notes.
    pub notes: Vec<String>,
    /// Its line.
    pub line: u32,
}

/// A place's declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    /// Its id.
    pub id: String,
    /// Its keys.
    pub binds: Vec<Bind>,
    /// Its kind.
    pub kind: String,
    /// Its unit.
    pub unit: Option<String>,
    /// Its price.
    pub priced: Option<Price>,
    /// Its lifespan.
    pub lives: LSpan,
    /// Notes.
    pub notes: Vec<String>,
    /// Its line.
    pub line: u32,
}

/// An input's form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputForm {
    /// `: observed`.
    Observed,
    /// A typed input.
    Typed(Box<TypedInput>),
}

/// A typed input's type and clauses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedInput {
    /// Its type.
    pub ty: Type,
    /// `reading`.
    pub reading: bool,
    /// `prices Y`.
    pub prices: Option<Lit>,
    /// `by index`.
    pub prices_by: Option<Expr>,
    /// `question "…"`.
    pub question: Option<String>,
}

/// An input's declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    /// Its id.
    pub id: String,
    /// Its keys.
    pub binds: Vec<Bind>,
    /// Its form.
    pub form: InputForm,
    /// Notes.
    pub notes: Vec<String>,
    /// Its line.
    pub line: u32,
}

/// A band's declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Band {
    /// Its id.
    pub id: String,
    /// Its key.
    pub binds: Vec<Bind>,
    /// Its argument's name.
    pub arg: String,
    /// Its argument's type.
    pub arg_type: Type,
    /// Its value's type.
    pub ty: Type,
    /// Notes.
    pub notes: Vec<String>,
    /// Its line.
    pub line: u32,
}

/// A def's law date.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LawDate {
    /// `at reader`.
    Reader,
    /// `at e`.
    Expr(Expr),
}

/// A rule's value: an expression or a stated hole.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleValue {
    /// `solve …`, in a rule with `v` (L §5.18).
    Solve(Box<Solve>),
    /// An expression.
    Expr(Expr),
    /// `?"…"`.
    Hole(String),
}

/// A rule of a def.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    /// Its label.
    pub label: String,
    /// The rules it beats.
    pub beats: Vec<String>,
    /// `v V`.
    pub v: Option<String>,
    /// Its own `governs`.
    pub governs: Option<LSpan>,
    /// Its value.
    pub value: RuleValue,
    /// `when c`.
    pub when: Option<Expr>,
    /// Its line.
    pub line: u32,
}

/// A def's declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Def {
    /// Its id.
    pub id: String,
    /// Its keys.
    pub binds: Vec<Bind>,
    /// Its type.
    pub ty: Type,
    /// `at`.
    pub at: Option<LawDate>,
    /// Its rules (one, `base`, for the short form).
    pub rules: Vec<Rule>,
    /// Notes.
    pub notes: Vec<String>,
    /// Whether it was written in the short form.
    pub short: bool,
    /// Its line.
    pub line: u32,
}

/// `before` or `after` another decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    /// `before`.
    Before,
    /// `after`.
    After,
}

/// A move's declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Move {
    /// Its id.
    pub id: String,
    /// Its keys.
    pub binds: Vec<Bind>,
    /// Its kind (`a.k` for a member's).
    pub kind: String,
    /// `on`.
    pub on: Expr,
    /// `at`.
    pub at: Option<Expr>,
    /// `for`.
    pub for_period: Option<Expr>,
    /// `when`.
    pub when: Option<Expr>,
    /// `reads`.
    pub reads: Vec<Expr>,
    /// `before` and `after`.
    pub order: Vec<(Order, String)>,
    /// `ties by`.
    pub ties: Option<Vec<Expr>>,
    /// `attr a = e`.
    pub attrs: Vec<(String, Expr)>,
    /// Its legs.
    pub legs: Vec<Leg>,
    /// `fulfils`.
    pub fulfils: Option<Expr>,
    /// Notes.
    pub notes: Vec<String>,
    /// The line of each of its clauses.
    pub lines: MoveLines,
    /// Its line.
    pub line: u32,
}

/// The line each of a move's clauses stands on; a clause not written keeps the move's line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveLines {
    /// `on`.
    pub on: u32,
    /// `at`.
    pub at: u32,
    /// `for`.
    pub for_period: u32,
    /// `when`.
    pub when: u32,
    /// `reads`.
    pub reads: u32,
    /// `ties by`.
    pub ties: u32,
    /// `fulfils`.
    pub fulfils: u32,
    /// Each `attr` line, in the order written.
    pub attrs: Vec<u32>,
}

impl MoveLines {
    /// Every clause at the move's own line.
    #[must_use]
    pub fn at_move(line: u32) -> MoveLines {
        MoveLines {
            on: line,
            at: line,
            for_period: line,
            when: line,
            reads: line,
            ties: line,
            fulfils: line,
            attrs: Vec::new(),
        }
    }
}

/// What an ask asks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AskBody {
    /// Values, `e, e, …`.
    Exprs(Vec<Expr>),
    /// `residuals x span [by id]`.
    Residuals(Box<Residuals>),
    /// `explain ref`, `explain bal(…)`, `explain low(…)` (L §5.18).
    Explain(Expr),
    /// `inputs-of ref`.
    InputsOf(Expr),
    /// `readers-of ref`.
    ReadersOf(Expr),
    /// `check a == b`.
    Check(Expr, Expr),
    /// `solve …`.
    Solve(Box<Solve>),
    /// `diff(e [for binds]) in s against b`.
    Diff(Box<Diff>),
}

/// `diff(e [for binds]) in s against b` (L §5.18, §6.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diff {
    /// The value compared.
    pub expr: Expr,
    /// Its bindings.
    pub binds: Vec<Bind>,
    /// The world compared.
    pub world: String,
    /// The world it is compared against.
    pub against: String,
}

/// An ask's `orient` clause (L §5.15).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orient {
    /// `orient lower`.
    Lower,
    /// `orient higher`.
    Higher,
}

/// `residuals x span [by id]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Residuals {
    /// The place.
    pub place: Expr,
    /// The span.
    pub span: Span,
    /// `by id`.
    pub by: Option<String>,
}

/// An ask (L §5.18).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ask {
    /// Its id.
    pub id: String,
    /// What it asks.
    pub body: AskBody,
    /// `for` bindings.
    pub binds: Vec<Bind>,
    /// `in world`.
    pub world: Option<String>,
    /// `with` pins.
    pub pins: Vec<Pin>,
    /// `law e`.
    pub law: Option<Expr>,
    /// `as known e`.
    pub known: Option<Expr>,
    /// `windows each`.
    pub windows_each: bool,
    /// `with uncertain`.
    pub uncertain: bool,
    /// `random at …`.
    pub random: Option<RChoice>,
    /// `paths N seed S`.
    pub paths: Option<(Num, Num)>,
    /// `path P`.
    pub path: Option<Num>,
    /// `partial`.
    pub partial: bool,
    /// `orient lower|higher`.
    pub orient: Option<Orient>,
    /// Its line.
    pub line: u32,
}

/// A row's key (L §2.3 `rk`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowKey {
    /// `*`.
    Star,
    /// A key.
    Lit(Lit),
    /// `a .. b`.
    Range(Lit, Lit),
    /// A place keyed by things, `konto[ana]`.
    Place(String, Vec<Lit>),
}

/// A row's value (L §2.3 `value`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// `?"…"`.
    Hole(String),
    /// `never`.
    Never,
    /// `none`.
    None,
    /// A text.
    Str(String),
    /// A record.
    Record(Vec<Field>),
    /// A key list.
    List(Vec<Lit>),
    /// A calendar.
    Calendar(Calendar),
    /// A window or a choice of days.
    Days(DaySet),
    /// A signed literal.
    Lit(Lit),
    /// A distribution, with its `to u` if written (L §5.16): an `Expr::Dist`, or an
    /// `Expr::To` around one.
    Dist(Expr),
    /// An expression (a scenario's row).
    Expr(Expr),
}

/// A reading row's `of`: a party, a batch, or the pair (L §2.3 `ofrow`, §5.19).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OfRow {
    /// One reference, `of ana`.
    One(Expr),
    /// `of (ana, b1)`.
    Pair(Expr, Expr),
}

/// A reading's instant (L §4.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instant {
    /// `at D [start|end]`.
    At(Day, Option<Edge>),
    /// `after m`.
    After(Expr),
}

/// The start or end of a day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    /// `start`.
    Start,
    /// `end`.
    End,
}

/// A row of a version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// The member's prefix.
    pub qual: Option<String>,
    /// The target.
    pub target: String,
    /// Its keys.
    pub keys: Option<Vec<RowKey>>,
    /// `of q`: the parts a reading states (L §5.19).
    pub of: Option<OfRow>,
    /// Its value.
    pub value: Value,
    /// `assumed "…"`.
    pub assumed: Option<String>,
    /// Its own `governs`.
    pub governs: Option<Governs>,
    /// Its own `known`.
    pub known: Option<Known>,
    /// Its instant.
    pub instant: Option<Instant>,
    /// Its locator.
    pub locator: Option<String>,
    /// `given`.
    pub given: bool,
    /// `replace`.
    pub replace: bool,
}

/// A band's interval (L §5.9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interval {
    /// The lower end, `None` for `-inf`.
    pub lo: Option<Lit>,
    /// The upper end, `None` for `inf`.
    pub hi: Option<Lit>,
    /// Whether the lower end is in.
    pub lo_closed: bool,
    /// Whether the upper end is in.
    pub hi_closed: bool,
}

/// A row of a band's table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BandRow {
    /// The member's prefix.
    pub qual: Option<String>,
    /// The band.
    pub target: String,
    /// The key of a keyed band: one literal, no sign (L §2.3 `bandrow`, tally 2.1.6).
    pub key: Option<Lit>,
    /// The interval.
    pub interval: Interval,
    /// The value.
    pub value: RuleValue,
    /// Its locator.
    pub locator: Option<String>,
    /// `replace`.
    pub replace: bool,
}

/// A key entering a kind or an index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Enter {
    /// The member's prefix.
    pub qual: Option<String>,
    /// The kind or index.
    pub target: String,
    /// The keys entered.
    pub keys: Vec<Lit>,
    /// Their unit.
    pub unit: Option<String>,
    /// Their price.
    pub priced: Option<Price>,
    /// Their lifespan.
    pub lives: Option<LSpan>,
    /// `replace`.
    pub replace: bool,
}

/// What an item of a version is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemForm {
    /// A row.
    Row(Box<Row>),
    /// A band row.
    Band(Box<BandRow>),
    /// An entry.
    Enter(Box<Enter>),
    /// `same p = q`.
    Same(Box<(Expr, Expr)>),
    /// `replace rule d.l …`.
    Rule {
        /// The member's prefix.
        qual: Option<String>,
        /// The def.
        def: String,
        /// The rule.
        rule: Box<Rule>,
    },
}

/// An item of a version, with its line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// What it is.
    pub form: ItemForm,
    /// Its line.
    pub line: u32,
}

/// A balance gap's role: `pad x from y` or `anchor x`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gap {
    /// `pad x from y`.
    Pad(Expr, Expr),
    /// `anchor x`.
    Anchor(Expr),
}

/// A version (L §5.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    /// Its id.
    pub id: String,
    /// `cite`.
    pub cite: String,
    /// `governs`.
    pub governs: Option<Governs>,
    /// `known`.
    pub known: Known,
    /// `enters`.
    pub enters: Option<Day>,
    /// `assumed`.
    pub assumed: Option<String>,
    /// `moves V to span`, each the version moved and its new span (L §5.3).
    pub moves: Vec<(Expr, LSpan)>,
    /// `covers`.
    pub covers: Vec<(Expr, Option<LSpan>)>,
    /// `pad` and `anchor`.
    pub gaps: Vec<Gap>,
    /// Notes.
    pub notes: Vec<String>,
    /// Its items.
    pub items: Vec<Item>,
    /// Its line.
    pub line: u32,
}
