//! The parser over one line's tokens: its token helpers, units, literals and spans.
//! Spec: L §2.1 (units, numbers, signs), §2.2 (reserved words), §2.3 (`lit`, `slit`, `lspan`,
//! `known`, `unit`), §2.3's depth limit (`LOAD-DEPTH`).
//! Twin: engine/tools/refcore/parser.py (class `P`: `peek` to `cur`, `is_uterm_start` to
//! `type_unit`, `number` to `known`).
//! Invariants: every helper consumes a token or fails; brackets and `if`s are counted, never
//! measured (R-16); the error texts are refcore's word for word.
//!
//! The expressions are in `expr.rs`, the sections in `item.rs`; both extend this parser.

use super::lex::{Kind, Line, Tok, cannot};
use super::tree::{Known, LSpan, Lit, Number, Unit, UnitTerm};
use crate::base::{Code, Load, Place, Refusal};
use crate::num::Num;
use crate::time::Day;
use std::collections::BTreeSet;

/// Why parsing stopped: a refusal of the spec, or a construct the engine doesn't build yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stop {
    /// A refusal with its code (L §8.3).
    Refused(Refusal),
    /// A construct of the language the engine doesn't build yet, named.
    NotBuilt(&'static str),
}

impl From<Refusal> for Stop {
    fn from(refusal: Refusal) -> Stop {
        Stop::Refused(refusal)
    }
}

/// The keywords: reserved everywhere (L §2.2).
const KEYWORDS: [&str; 49] = [
    "model",
    "part",
    "scenario",
    "source",
    "version",
    "unit",
    "kind",
    "axis",
    "index",
    "place",
    "input",
    "band",
    "def",
    "given",
    "fill",
    "move",
    "ask",
    "tally",
    "note",
    "if",
    "then",
    "else",
    "and",
    "or",
    "not",
    "in",
    "for",
    "where",
    "with",
    "as",
    "when",
    "true",
    "false",
    "inf",
    "by",
    "at",
    "law",
    "known",
    "governs",
    "assumed",
    "none",
    "never",
    "check",
    "solve",
    "diff",
    "explain",
    "inputs-of",
    "readers-of",
    "residuals",
];

/// The deepest an expression may nest (L §2.3, `LOAD-DEPTH`).
const MAX_DEPTH: u8 = 8;

/// The deepest the parser recurses through reads chained by `from` or `ties by`, which the
/// spec's depth doesn't count; past it the line is left to a later milestone, never refused.
const MAX_NEST: u8 = 64;

/// Whether a token may have a space before it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Space {
    /// Spaced or touching.
    Free,
    /// Written touching the token before it.
    Touching,
}

/// Which words name a unit: the units the reality declares (after a number), or any word
/// that is no keyword (in a type, read by syntax only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Names {
    /// Periods and the declared units.
    Declared,
    /// Any word that is no keyword.
    Any,
}

/// Whether the parser took a `-` before a number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Minus {
    /// No sign was taken.
    None,
    /// A `-` was taken: the number is negated.
    Taken,
}

impl Minus {
    /// The sign as written in a number's text.
    fn text(self) -> &'static str {
        match self {
            Minus::None => "",
            Minus::Taken => "-",
        }
    }
}

/// Whether `word` is a keyword (L §2.2).
pub(crate) fn is_keyword(word: &str) -> bool {
    !word.is_empty() && KEYWORDS.contains(&word)
}

/// The period words and their singular forms (L §2.2).
pub(crate) fn period_word(word: &str) -> Option<&'static str> {
    match word {
        "day" | "days" => Some("day"),
        "month" | "months" => Some("month"),
        "quarter" | "quarters" => Some("quarter"),
        "year" | "years" => Some("year"),
        _ => None,
    }
}

/// A parser over the tokens of one line.
#[derive(Debug)]
pub(crate) struct P<'a> {
    /// The line's tokens.
    pub(super) toks: &'a [Tok],
    /// The next token's position.
    pub(super) i: usize,
    /// The file, as the run names it.
    file: &'a str,
    /// The line's number.
    pub(super) line: u32,
    /// The unit names a number may carry (refcore's `prescan_units`).
    units: &'a BTreeSet<String>,
    /// The brackets and `if`s open here.
    depth: u8,
    /// The parser's own recursion through chained reads.
    nest: u8,
    /// Inside an ask line's `for`: a closing `in ID` names the world (parse rule 4).
    pub(super) ask_for: bool,
}

impl<'a> P<'a> {
    /// A parser at the start of `toks`, a line of `file`.
    pub(super) fn new(
        toks: &'a [Tok],
        file: &'a str,
        line: u32,
        units: &'a BTreeSet<String>,
    ) -> P<'a> {
        P {
            toks,
            i: 0,
            file,
            line,
            units,
            depth: 0,
            nest: 0,
            ask_for: false,
        }
    }

    /// The line's place, for a refusal.
    pub(super) fn place(&self) -> Place {
        Place::Line(self.file.to_owned(), self.line)
    }

    /// A refusal with `code` and `message` at this line.
    pub(super) fn refuse(&self, code: Load, message: String) -> Stop {
        Stop::Refused(Refusal::at(Code::Load(code), message, self.place()))
    }

    /// `LOAD-PARSE cannot parse: <what>` at this line.
    pub(super) fn cannot(&self, what: &str) -> Stop {
        Stop::Refused(cannot(what, self.place()))
    }

    /// `LOAD-PARSE cannot parse: <what>` at another line of this file.
    pub(super) fn cannot_at(&self, what: &str, line: u32) -> Stop {
        Stop::Refused(cannot(what, Place::Line(self.file.to_owned(), line)))
    }

    /// A parser over another line of this file, after that line's first word.
    pub(super) fn under(&self, line: &'a Line) -> P<'a> {
        let mut p = P::new(&line.toks, self.file, line.number, self.units);
        p.i = 1;
        p
    }

    /// The token `k` places ahead.
    pub(super) fn peek(&self, k: usize) -> Option<&'a Tok> {
        self.toks.get(self.i.checked_add(k)?)
    }

    /// Whether the line is read to its end.
    pub(super) fn done(&self) -> bool {
        self.i >= self.toks.len()
    }

    /// Steps past `n` tokens.
    pub(super) fn skip(&mut self, n: usize) -> Result<(), Stop> {
        self.i = self
            .i
            .checked_add(n)
            .ok_or_else(|| self.cannot("a line too long"))?;
        Ok(())
    }

    /// The next token, taken; `expected more text` at the end.
    pub(super) fn next(&mut self) -> Result<&'a Tok, Stop> {
        let Some(tok) = self.peek(0) else {
            return Err(self.err("more text"));
        };
        self.skip(1)?;
        Ok(tok)
    }

    /// Whether the token `k` ahead is the word or operator `s`.
    pub(super) fn at(&self, s: &str, k: usize) -> bool {
        self.peek(k).is_some_and(|tok| {
            (tok.kind == Kind::Id || tok.kind == Kind::Key || tok.kind == Kind::Op) && tok.text == s
        })
    }

    /// The kind of the token `k` ahead.
    pub(super) fn kind(&self, k: usize) -> Option<&'a Kind> {
        self.peek(k).map(|tok| &tok.kind)
    }

    /// Whether the token `k` ahead is an id.
    pub(super) fn at_id(&self, k: usize) -> bool {
        self.kind(k) == Some(&Kind::Id)
    }

    /// Whether the token `k` ahead is a string.
    pub(super) fn at_str(&self, k: usize) -> bool {
        self.kind(k).is_some_and(|kind| match kind {
            Kind::Str(_) => true,
            Kind::Id
            | Kind::Key
            | Kind::Int(_)
            | Kind::Dec(_)
            | Kind::Day(_)
            | Kind::Month(..)
            | Kind::Quarter(..)
            | Kind::MonthDay(..)
            | Kind::Cur
            | Kind::Op => false,
        })
    }

    /// Whether the token `k` ahead is an integer.
    pub(super) fn at_int(&self, k: usize) -> bool {
        self.kind(k).is_some_and(|kind| match kind {
            Kind::Int(_) => true,
            Kind::Id
            | Kind::Key
            | Kind::Dec(_)
            | Kind::Str(_)
            | Kind::Day(_)
            | Kind::Month(..)
            | Kind::Quarter(..)
            | Kind::MonthDay(..)
            | Kind::Cur
            | Kind::Op => false,
        })
    }

    /// Whether the token `k` ahead is white-space separated from the one before it.
    pub(super) fn spaced(&self, k: usize) -> bool {
        self.peek(k).is_some_and(|tok| tok.ws)
    }

    /// Takes the word or operator `s` if it is next.
    pub(super) fn accept(&mut self, s: &str) -> Result<bool, Stop> {
        if self.at(s, 0) {
            self.skip(1)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Takes the word or operator `s`, or refuses: `expected "s"`.
    pub(super) fn expect(&mut self, s: &str) -> Result<(), Stop> {
        if self.accept(s)? {
            return Ok(());
        }
        Err(self.err(&format!("\"{s}\"")))
    }

    /// `cannot parse: expected <what>, got <the next token or end of line>`.
    pub(super) fn err(&self, what: &str) -> Stop {
        let got = self.peek(0).map_or("end of line", |tok| tok.text.as_str());
        self.cannot(&format!("expected {what}, got {got}"))
    }

    /// Refuses unless the line is read to its end.
    pub(super) fn end(&self) -> Result<(), Stop> {
        if self.done() {
            return Ok(());
        }
        Err(self.err("end of line"))
    }

    /// One more bracket or `if` open: `LOAD-DEPTH` past 8 (L §2.3).
    pub(super) fn open(&mut self) -> Result<(), Stop> {
        self.depth = self
            .depth
            .checked_add(1)
            .ok_or_else(|| self.cannot("depth"))?;
        if self.depth > MAX_DEPTH {
            return Err(self.refuse(Load::Depth, "expression nested deeper than 8".to_owned()));
        }
        Ok(())
    }

    /// One bracket or `if` closed.
    pub(super) fn close(&mut self) -> Result<(), Stop> {
        self.depth = self
            .depth
            .checked_sub(1)
            .ok_or_else(|| self.cannot("a bracket closed twice"))?;
        Ok(())
    }

    /// One more level of the parser's own recursion through a chained read.
    pub(super) fn enter(&mut self) -> Result<(), Stop> {
        self.nest = self.nest.checked_add(1).ok_or(Stop::NotBuilt(
            "reads chained deeper than the parser counts",
        ))?;
        if self.nest > MAX_NEST {
            return Err(Stop::NotBuilt(
                "reads chained deeper than the parser counts",
            ));
        }
        Ok(())
    }

    /// That level left.
    pub(super) fn leave(&mut self) -> Result<(), Stop> {
        self.nest = self
            .nest
            .checked_sub(1)
            .ok_or_else(|| self.cannot("a read left twice"))?;
        Ok(())
    }

    /// An id; a keyword there is `LOAD-RESERVED` (L §2.2).
    pub(super) fn ident(&mut self, what: &str) -> Result<String, Stop> {
        let Some(tok) = self.peek(0) else {
            return Err(self.err(what));
        };
        if tok.kind != Kind::Id {
            return Err(self.err(what));
        }
        if is_keyword(&tok.text) {
            return Err(self.refuse(Load::Reserved, format!("{} is a keyword", tok.text)));
        }
        self.skip(1)?;
        Ok(tok.text.clone())
    }

    /// Whether no bracket or `if` is open here.
    pub(super) fn depth_is_zero(&self) -> bool {
        self.depth == 0
    }

    /// A rule label: any word of the id pattern, keywords included.
    pub(super) fn label(&mut self) -> Result<String, Stop> {
        if !self.at_id(0) {
            return Err(self.err("a rule label"));
        }
        Ok(self.next()?.text.clone())
    }

    /// A string's text.
    pub(super) fn string(&mut self) -> Result<String, Stop> {
        match self.kind(0) {
            Some(Kind::Str(text)) => {
                self.skip(1)?;
                Ok(text.clone())
            }
            Some(_) | None => Err(self.err("a string")),
        }
    }

    /// A day.
    pub(super) fn day(&mut self) -> Result<Day, Stop> {
        match self.kind(0) {
            Some(Kind::Day(day)) => {
                self.skip(1)?;
                Ok(*day)
            }
            Some(_) | None => Err(self.err("a day")),
        }
    }

    /// An integer.
    pub(super) fn integer(&mut self) -> Result<Num, Stop> {
        match self.kind(0) {
            Some(Kind::Int(n)) => {
                self.skip(1)?;
                Ok(n.clone())
            }
            Some(_) | None => Err(self.err("an integer")),
        }
    }

    /// A currency code.
    pub(super) fn cur(&mut self) -> Result<String, Stop> {
        if self.kind(0) != Some(&Kind::Cur) {
            return Err(self.err("a currency code"));
        }
        Ok(self.next()?.text.clone())
    }

    // -- units (L §2.1 Units, §2.3 unit) --

    /// Whether a unit term starts `k` ahead: a currency, `%`, or a unit's or period's name
    /// (with `Names::Any`, any word that is no keyword).
    pub(super) fn is_uterm_start(&self, k: usize, space: Space, names: Names) -> bool {
        let Some(tok) = self.peek(k) else {
            return false;
        };
        if space == Space::Touching && tok.ws {
            return false;
        }
        match tok.kind {
            Kind::Cur => true,
            Kind::Op => tok.text == "%",
            Kind::Id if names == Names::Any => !is_keyword(&tok.text),
            Kind::Id => period_word(&tok.text).is_some() || self.units.contains(&tok.text),
            Kind::Key
            | Kind::Int(_)
            | Kind::Dec(_)
            | Kind::Str(_)
            | Kind::Day(_)
            | Kind::Month(..)
            | Kind::Quarter(..)
            | Kind::MonthDay(..) => false,
        }
    }

    /// One unit term with its power, `PLN^2`.
    fn uterm(&mut self) -> Result<UnitTerm, Stop> {
        let tok = self.next()?;
        let base = match (&tok.kind, period_word(&tok.text)) {
            (Kind::Id, Some(singular)) => singular.to_owned(),
            _ => tok.text.clone(),
        };
        let mut power = 1_i64;
        if self.at("^", 0) && !self.spaced(0) {
            self.skip(1)?;
            let negative = self.accept("-")?;
            let n = self.integer()?;
            let n = n.to_i64().ok_or_else(|| self.cannot("a power too large"))?;
            power = if negative {
                n.checked_neg()
                    .ok_or_else(|| self.cannot("a power too large"))?
            } else {
                n
            };
        }
        Ok(UnitTerm { base, power })
    }

    /// The terms after the first, joined by touching `*` or `/`.
    fn unit_rest(&mut self, mut terms: Vec<UnitTerm>) -> Result<Unit, Stop> {
        while self
            .peek(0)
            .is_some_and(|t| t.kind == Kind::Op && (t.text == "*" || t.text == "/"))
            && !self.spaced(0)
            && self.is_uterm_start(1, Space::Touching, Names::Any)
        {
            let divide = self.next()?.text == "/";
            let term = self.uterm()?;
            let power = if divide {
                term.power
                    .checked_neg()
                    .ok_or_else(|| self.cannot("a power too large"))?
            } else {
                term.power
            };
            terms.push(UnitTerm {
                base: term.base,
                power,
            });
        }
        Ok(Unit { terms })
    }

    /// `1/u`: a unit that starts with the reciprocal of a term.
    fn reciprocal(&mut self) -> Result<UnitTerm, Stop> {
        self.skip(2)?;
        let term = self.uterm()?;
        Ok(UnitTerm {
            power: term
                .power
                .checked_neg()
                .ok_or_else(|| self.cannot("a power too large"))?,
            base: term.base,
        })
    }

    /// Whether `1/` starts here; with `Space::Touching`, only with a unit term touching it.
    fn at_reciprocal(&self, term: Space) -> bool {
        let one = match self.kind(0) {
            Some(Kind::Int(n)) => *n == Num::int(1),
            Some(_) | None => false,
        };
        let term_touches =
            || !self.spaced(1) && self.is_uterm_start(2, Space::Touching, Names::Any);
        one && self.at("/", 1) && (term == Space::Free || term_touches())
    }

    /// A unit written after a number, or none (L §2.1 Units).
    pub(super) fn unit_after_number(&mut self) -> Result<Option<Unit>, Stop> {
        let first = if self.is_uterm_start(0, Space::Free, Names::Declared) {
            self.uterm()?
        } else if self.at_reciprocal(Space::Touching) {
            self.reciprocal()?
        } else {
            return Ok(None);
        };
        Ok(Some(self.unit_rest(vec![first])?))
    }

    /// A unit in a type, read by syntax only; a type's clause word ends it (parse rule 3).
    pub(super) fn type_unit(&mut self) -> Result<Option<Unit>, Stop> {
        let clause = self.peek(0).is_some_and(|tok| {
            tok.kind == Kind::Id
                && ["question", "reading", "prices", "lives", "priced"].contains(&tok.text.as_str())
        });
        if clause {
            return Ok(None);
        }
        let first = if self.is_uterm_start(0, Space::Free, Names::Any) {
            self.uterm()?
        } else if self.at_reciprocal(Space::Free) {
            self.reciprocal()?
        } else {
            return Ok(None);
        };
        Ok(Some(self.unit_rest(vec![first])?))
    }

    // -- literals (L §2.3 lit, slit) --

    /// `value`, negated when a `-` was taken before it.
    fn signed(&self, value: Num, minus: Minus, what: &str) -> Result<Num, Stop> {
        match minus {
            Minus::None => Ok(value),
            Minus::Taken => value.checked_neg().map_err(|_| self.cannot(what)),
        }
    }

    /// A number with an optional unit, with no sign before it.
    pub(super) fn unsigned_number(&mut self) -> Result<Number, Stop> {
        self.number(Minus::None)
    }

    /// A number with an optional unit, after the `-` of `minus` if one was taken.
    fn number(&mut self, minus: Minus) -> Result<Number, Stop> {
        let tok = self.next()?;
        let (value, whole) = match &tok.kind {
            Kind::Int(n) => (n.clone(), true),
            Kind::Dec(n) => (n.clone(), false),
            Kind::Id
            | Kind::Key
            | Kind::Str(_)
            | Kind::Day(_)
            | Kind::Month(..)
            | Kind::Quarter(..)
            | Kind::MonthDay(..)
            | Kind::Cur
            | Kind::Op => return Err(self.err("a number")),
        };
        let value = self.signed(value, minus, "a number")?;
        let unit = self.unit_after_number()?;
        let four = whole && tok.text.len() == 4 && tok.text.bytes().all(|b| b.is_ascii_digit());
        Ok(Number {
            value,
            whole,
            four: four && unit.is_none(),
            unit,
            text: format!("{}{}", minus.text(), tok.text),
            fraction: false,
        })
    }

    /// A literal: a number with a unit, a date, a key, a code word, `true`/`false`, `inf`;
    /// a `-` before a number or `inf` is its sign.
    pub(super) fn lit(&mut self) -> Result<Lit, Stop> {
        if self.at("-", 0) {
            match self.kind(1) {
                Some(Kind::Int(_) | Kind::Dec(_)) => {
                    self.skip(1)?;
                    return self.lit_after(Minus::Taken);
                }
                Some(Kind::Id) if self.at("inf", 1) => {
                    self.skip(2)?;
                    return Ok(Lit::Inf { neg: true });
                }
                Some(_) | None => {}
            }
        }
        self.lit_after(Minus::None)
    }

    /// A literal with no sign before it (a currency's minor unit).
    pub(super) fn lit_unsigned(&mut self) -> Result<Lit, Stop> {
        self.lit_after(Minus::None)
    }

    /// A literal, after the `-` of `minus` if one was taken.
    fn lit_after(&mut self, minus: Minus) -> Result<Lit, Stop> {
        let Some(tok) = self.peek(0) else {
            return Err(self.err("a literal"));
        };
        match &tok.kind {
            Kind::Int(_) if self.at_fraction() => self.fraction(minus),
            Kind::Int(_) | Kind::Dec(_) => Ok(Lit::Num(self.number(minus)?)),
            Kind::Day(day) => self.take(Lit::Day(*day)),
            Kind::Month(y, m) => self.take(Lit::Month(*y, *m)),
            Kind::Quarter(y, q) => self.take(Lit::Quarter(*y, *q)),
            Kind::MonthDay(m, d) => self.take(Lit::MonthDay(*m, *d)),
            Kind::Key => self.take(Lit::Key(tok.text.clone())),
            Kind::Cur => self.take(Lit::Cur(tok.text.clone())),
            Kind::Id => self.word_lit(&tok.text),
            Kind::Str(_) | Kind::Op => Err(self.err("a literal")),
        }
    }

    /// Takes one token and answers `lit`.
    fn take(&mut self, lit: Lit) -> Result<Lit, Stop> {
        self.skip(1)?;
        Ok(lit)
    }

    /// A literal written as a word: `true`, `false`, `inf`, `nominal n`, or a name.
    fn word_lit(&mut self, word: &str) -> Result<Lit, Stop> {
        match word {
            "true" => self.take(Lit::Bool(true)),
            "false" => self.take(Lit::Bool(false)),
            "inf" => self.take(Lit::Inf { neg: false }),
            "nominal" if self.at_int(1) => {
                self.skip(1)?;
                Ok(Lit::Nominal(self.integer()?))
            }
            _ if is_keyword(word) => Err(self.err("a literal")),
            _ => self.take(Lit::Name(word.to_owned())),
        }
    }

    /// Whether `a/b` written touching, two integers, starts here (L §2.1 Fractions).
    fn at_fraction(&self) -> bool {
        self.at("/", 1) && self.at_int(2) && !self.spaced(1) && !self.spaced(2)
    }

    /// A fraction `a/b`, allowed only in rows (L §2.1).
    fn fraction(&mut self, minus: Minus) -> Result<Lit, Stop> {
        let numer_text = self
            .peek(0)
            .map_or(String::new(), |t| t.text.replace(' ', ""));
        let numer = self.integer()?;
        self.skip(1)?;
        let denom_text = self
            .peek(0)
            .map_or(String::new(), |t| t.text.replace(' ', ""));
        let denom = self.integer()?;
        if denom.is_zero() {
            return Err(self.cannot("fraction with denominator 0"));
        }
        let value = numer
            .checked_div(&denom)
            .map_err(|_| self.cannot("fraction with denominator 0"))?;
        let value = self.signed(value, minus, "a fraction")?;
        let unit = self.unit_after_number()?;
        Ok(Lit::Num(Number {
            value,
            whole: false,
            unit,
            text: format!("{}{numer_text}/{denom_text}", minus.text()),
            four: false,
            fraction: true,
        }))
    }

    /// A version's span of literals: `a .. b`, `a ..`, `.. b`, or `a` (L §2.3 `lspan`).
    pub(super) fn lspan(&mut self) -> Result<LSpan, Stop> {
        if self.accept("..")? {
            let to = self.lit()?;
            return Ok(LSpan {
                from: None,
                to: Some(to),
                dots: true,
                to_hole: None,
            });
        }
        let from = self.lit()?;
        if self.accept("..")? {
            let to = if self.done() || self.at_clause_after_span() {
                None
            } else {
                Some(self.lit()?)
            };
            return Ok(LSpan {
                from: Some(from),
                to,
                dots: true,
                to_hole: None,
            });
        }
        Ok(LSpan {
            from: Some(from),
            to: None,
            dots: false,
            to_hole: None,
        })
    }

    /// An entry's lifespan: an `lspan`, or `a ..?"…"`, an end not yet known.
    pub(super) fn life_span(&mut self) -> Result<LSpan, Stop> {
        let save = self.i;
        if !self.at("..", 0) {
            let from = self.lit()?;
            if self.at("..", 0) && self.at("?", 1) && self.at_str(2) {
                self.skip(2)?;
                return Ok(LSpan {
                    from: Some(from),
                    to: None,
                    dots: true,
                    to_hole: Some(self.string()?),
                });
            }
            self.i = save;
        }
        self.lspan()
    }

    /// Whether what follows `a ..` is the line's next clause rather than an end.
    fn at_clause_after_span(&self) -> bool {
        let Some(tok) = self.peek(0) else {
            return true;
        };
        match &tok.kind {
            Kind::Str(_) => true,
            Kind::Op | Kind::Key => tok.text == "," || tok.text == "=",
            Kind::Id => {
                ["known", "at", "after", "governs", "priced", "lives"].contains(&tok.text.as_str())
            }
            Kind::Int(_)
            | Kind::Dec(_)
            | Kind::Day(_)
            | Kind::Month(..)
            | Kind::Quarter(..)
            | Kind::MonthDay(..)
            | Kind::Cur => false,
        }
    }

    /// `known`: a day, or `window(a, b)` (L §2.3).
    pub(super) fn known(&mut self) -> Result<Known, Stop> {
        if self.at("window", 0) && self.at("(", 1) {
            self.skip(2)?;
            let a = self.day()?;
            self.expect(",")?;
            let b = self.day()?;
            self.expect(")")?;
            return Ok(Known::Window(a, b));
        }
        Ok(Known::Day(self.day()?))
    }
}
