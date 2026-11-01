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
    /// Its power, negative under `/`.
    pub power: i64,
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
    /// Its line.
    pub line: u32,
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
    /// Its line.
    pub line: u32,
}

/// What an ask asks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AskBody {
    /// Values, `e, e, …`.
    Exprs(Vec<Expr>),
    /// `residuals x span [by id]`.
    Residuals(Box<Residuals>),
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
    /// `law e`.
    pub law: Option<Expr>,
    /// `as known e`.
    pub known: Option<Expr>,
    /// `windows each`.
    pub windows_each: bool,
    /// `with uncertain`.
    pub uncertain: bool,
    /// `partial`.
    pub partial: bool,
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
    /// An expression (a scenario's row).
    Expr(Expr),
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
    /// The key of a keyed band.
    pub key: Option<RowKey>,
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
