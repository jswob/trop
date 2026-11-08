//! The expression grammar: precedence climbing over flat chains, postfixes, references, atoms, records, legs and types.
//! Spec: L §2.3 (`expr` to `atom`, `ofq`, `ref`, `sref`, `record`, `field`, `leg`, `type`,
//! `qid`, `rowkeys`), its seven parse rules, and its depth limit.
//! Twin: engine/tools/refcore/parser.py (`expr` to `call_args`, `ofq`, `sref`, `atom`,
//! `field`, `field_value`, `leg_body`, `type`, `qid`, `rowkey`).
//! Invariants: every loop takes a token or ends; each bracket and `if` is counted by `open`;
//! every form of the grammar parses (the bindings and spans are in `binds.rs`, the built-ins
//! and the reads with clauses in `calls.rs`, the reads M0 adds in `reads.rs`, the pins in
//! `ask.rs`).

use super::lex::{Kind, Mark, Paren, Read, Role};
use super::parse::{P, Stop, is_keyword};
use super::tree::{
    AddOp, Call, CmpOp, Expr, Field, FieldValue, In, Key, Leg, Lit, MulOp, Number, OfQ, Postfix,
    RowKey, Step, Type,
};
use crate::base::Load;
use crate::num::Num;

/// Where an expression stops early: at a membership's `in`, or at a leg's `to`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Ends {
    /// It runs to its natural end.
    Free,
    /// `in` ends it (a binding's domain, an ask's body).
    AtIn,
    /// `to` ends it (a leg's `from` place).
    AtTo,
}

/// What an operand stands under, for the parentheses L §2.4 always prints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Under {
    /// An arithmetic or logical operator, a `not` or a `-`.
    Operator,
    /// A comparison or a membership test.
    Comparison,
}

/// Whether a record is a row's, whose fields may be signed literals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Rec {
    /// A row's record.
    Row,
    /// An expression's record.
    Expr,
}

/// How a reference's keys are written: `ref`'s expressions, or `sref`'s literals (L §2.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeysOf {
    /// `expr`, or `*`.
    Exprs,
    /// `lit`.
    Lits,
}

/// The built-in names, read as built-ins only directly before `(` (L §2.2).
const BUILTINS: [&str; 75] = [
    "sum",
    "count",
    "min",
    "max",
    "any",
    "all",
    "first",
    "last",
    "argmin",
    "argmax",
    "share",
    "mean",
    "quantile",
    "prorata",
    "flows",
    "bal",
    "value",
    "low",
    "next",
    "covered",
    "round",
    "pow",
    "exp",
    "ln",
    "sqrt",
    "abs",
    "periodic",
    "factor",
    "dcf",
    "accrue",
    "days_between",
    "add_days",
    "add_months",
    "add_years",
    "start_of",
    "end_of",
    "year_of",
    "quarter_of",
    "month_of",
    "day_of",
    "overlap_days",
    "pos",
    "age",
    "age_months",
    "weekday",
    "add_business",
    "last_business",
    "is_business",
    "schedule",
    "calendar",
    "window",
    "one_of",
    "since",
    "band_lo",
    "band_hi",
    "marginal",
    "unit",
    "minor",
    "lifespan",
    "gap",
    "observed",
    "missing",
    "normal",
    "lognormal",
    "exponential",
    "gamma",
    "poisson",
    "binomial",
    "negbin",
    "uniform",
    "discrete",
    "cdf",
    "pdf",
    "pmf",
    "crps",
];

/// Whether `word` is a built-in's name (L §2.2).
pub(crate) fn is_builtin(word: &str) -> bool {
    !word.is_empty() && BUILTINS.contains(&word)
}

/// The comparison an operator token writes.
fn cmp_op(text: &str) -> Option<CmpOp> {
    match text {
        "==" => Some(CmpOp::Eq),
        "!=" => Some(CmpOp::Ne),
        "<" => Some(CmpOp::Lt),
        "<=" => Some(CmpOp::Le),
        ">" => Some(CmpOp::Gt),
        ">=" => Some(CmpOp::Ge),
        _ => None,
    }
}

impl P<'_> {
    /// An expression: `if c then a else b`, or an `or` chain (L §2.3 `expr`).
    pub(super) fn expr(&mut self, ends: Ends) -> Result<Expr, Stop> {
        if !self.at("if", 0) {
            return self.or_chain(ends);
        }
        self.skip(1)?;
        self.open()?;
        let cond = self.expr(Ends::Free)?;
        self.expect("then")?;
        let then = self.expr(ends)?;
        self.expect("else")?;
        let other = self.expr(ends)?;
        self.close()?;
        Ok(Expr::If(Box::new([cond, then, other])))
    }

    /// `a or b or …`.
    fn or_chain(&mut self, ends: Ends) -> Result<Expr, Stop> {
        let mut froms = vec![self.i];
        let mut terms = vec![self.and_chain(ends)?];
        while self.accept("or")? {
            froms.push(self.i);
            terms.push(self.and_chain(ends)?);
        }
        self.force_terms(&terms, &froms);
        if self.blind
            && terms.len() > 1
            && let Some(Expr::Or(head)) = terms.first_mut()
        {
            let mut joined = core::mem::take(head);
            joined.extend(terms.into_iter().skip(1));
            terms = joined;
        }
        Ok(single_or(terms, Expr::Or))
    }

    /// `a and b and …`.
    fn and_chain(&mut self, ends: Ends) -> Result<Expr, Stop> {
        let mut froms = vec![self.i];
        let mut terms = vec![self.not_term(ends)?];
        while self.accept("and")? {
            froms.push(self.i);
            terms.push(self.not_term(ends)?);
        }
        self.force_terms(&terms, &froms);
        if self.blind
            && terms.len() > 1
            && let Some(Expr::And(head)) = terms.first_mut()
        {
            let mut joined = core::mem::take(head);
            joined.extend(terms.into_iter().skip(1));
            terms = joined;
        }
        Ok(single_or(terms, Expr::And))
    }

    /// The terms of an `or` or `and` chain of two terms or more, each from its first token,
    /// marked for the reprint's parentheses.
    fn force_terms(&self, terms: &[Expr], froms: &[usize]) {
        if terms.len() > 1 {
            for (term, from) in terms.iter().zip(froms) {
                self.force(term, *from, Under::Operator);
            }
        }
    }

    /// Marks an operand of an operator node for the reprint (L §2.4, refcore's `forced`): a
    /// `… to u` (and, in a comparison or a membership, a converted `flows` or a `value`) is
    /// always parenthesised, so its parentheses stay, and written bare it gets them. The
    /// operand runs from the token at `from` to the one taken last.
    fn force(&self, operand: &Expr, from: usize, under: Under) {
        let always = |expr: &Expr| {
            matches!(expr, Expr::To(..))
                || (under == Under::Comparison
                    && (matches!(expr, Expr::Value(_))
                        || matches!(expr, Expr::Flows(f) if f.convert.is_some())))
        };
        if let Expr::Paren(inner) = operand {
            if always(inner) {
                self.mark(Some(from), |m| m.paren = Paren::Forced);
            }
        } else if always(operand) {
            self.mark(Some(from), |m| m.wrap_open = true);
            self.mark(self.back(), |m| m.wrap_close = true);
        }
    }

    /// `[not] cmp`: one `not` at most.
    fn not_term(&mut self, ends: Ends) -> Result<Expr, Stop> {
        if self.accept("not")? {
            let from = self.i;
            let term = self.cmp(ends)?;
            self.force(&term, from, Under::Operator);
            return Ok(Expr::Not(Box::new(term)));
        }
        self.cmp(ends)
    }

    /// A comparison or a membership test; neither chains (parse rule 1).
    fn cmp(&mut self, ends: Ends) -> Result<Expr, Stop> {
        let from = self.i;
        let left = self.sum(ends)?;
        if let Some(op) = self
            .peek(0)
            .filter(|tok| tok.kind == Kind::Op)
            .and_then(|tok| cmp_op(&tok.text))
        {
            self.force(&left, from, Under::Comparison);
            self.skip(1)?;
            let right_from = self.i;
            let right = self.sum(ends)?;
            self.force(&right, right_from, Under::Comparison);
            self.no_chain()?;
            return Ok(Expr::Cmp(Box::new((left, op, right))));
        }
        let world = self.ask_for && self.depth_is_zero() && self.world_in();
        if self.at("in", 0) && ends != Ends::AtIn && !world {
            self.force(&left, from, Under::Comparison);
            self.skip(1)?;
            let set_from = self.i;
            let set = self.sum(ends)?;
            self.force(&set, set_from, Under::Comparison);
            let on = if self.accept("on")? {
                let on_from = self.i;
                let on = self.sum(ends)?;
                self.force(&on, on_from, Under::Comparison);
                Some(on)
            } else {
                None
            };
            self.no_chain()?;
            return Ok(Expr::In(Box::new(In {
                member: left,
                set,
                on,
            })));
        }
        Ok(left)
    }

    /// `in ID` that ends an ask line or comes before its later clauses: the world
    /// (parse rule 4).
    pub(super) fn world_in(&self) -> bool {
        if !(self.at("in", 0) && self.at_id(1)) {
            return false;
        }
        match self.peek(2) {
            None => true,
            Some(tok) => {
                tok.kind == Kind::Id
                    && [
                        "with", "law", "as", "windows", "random", "paths", "path", "partial",
                        "orient",
                    ]
                    .contains(&tok.text.as_str())
            }
        }
    }

    /// Refuses a second comparison after one (parse rule 1).
    pub(super) fn no_chain(&self) -> Result<(), Stop> {
        let chained = self
            .peek(0)
            .is_some_and(|tok| tok.kind == Kind::Op && cmp_op(&tok.text).is_some());
        if chained {
            return Err(self.cannot("comparisons don't chain"));
        }
        Ok(())
    }

    /// `a ± b ± …`, kept flat.
    pub(super) fn sum(&mut self, ends: Ends) -> Result<Expr, Stop> {
        let from = self.i;
        let first = self.prod(ends)?;
        let mut rest = Vec::new();
        loop {
            let op = match self.op_text() {
                Some("+") => AddOp::Add,
                Some("-") => AddOp::Sub,
                Some(_) | None => break,
            };
            if rest.is_empty() {
                self.force(&first, from, Under::Operator);
            }
            self.role(Some(self.i), Role::Bin);
            self.skip(1)?;
            let term_from = self.i;
            let term = self.prod(ends)?;
            self.force(&term, term_from, Under::Operator);
            rest.push((op, term));
        }
        if rest.is_empty() {
            return Ok(first);
        }
        let mut first = first;
        if self.blind
            && let Expr::Sum(_, head) = &mut first
        {
            // refcore nests `+` and `-` to the left: `(a + b) + c` is `a + b + c`
            head.extend(rest);
            return Ok(first);
        }
        Ok(Expr::Sum(Box::new(first), rest))
    }

    /// `a × b ÷ …`, kept flat.
    fn prod(&mut self, ends: Ends) -> Result<Expr, Stop> {
        let from = self.i;
        let first = self.unary(ends)?;
        let mut rest = Vec::new();
        loop {
            let op = match self.op_text() {
                Some("*") => MulOp::Mul,
                Some("/") => MulOp::Div,
                Some(_) | None => break,
            };
            if rest.is_empty() {
                self.force(&first, from, Under::Operator);
            }
            self.role(Some(self.i), Role::Bin);
            self.skip(1)?;
            let term_from = self.i;
            let term = self.unary(ends)?;
            self.force(&term, term_from, Under::Operator);
            rest.push((op, term));
        }
        if rest.is_empty() {
            return Ok(first);
        }
        let mut first = first;
        if self.blind
            && let Expr::Prod(_, head) = &mut first
        {
            // refcore nests `*` and `/` to the left: `(a * b) / c` is `a * b / c`
            head.extend(rest);
            return Ok(first);
        }
        Ok(Expr::Prod(Box::new(first), rest))
    }

    /// The next token's text when it is an operator token.
    fn op_text(&self) -> Option<&str> {
        self.peek(0)
            .filter(|tok| tok.kind == Kind::Op)
            .map(|tok| tok.text.as_str())
    }

    /// `[-] post`: one prefix `-` at most.
    fn unary(&mut self, ends: Ends) -> Result<Expr, Stop> {
        if self.accept("-")? {
            let from = self.i;
            let term = self.post(ends)?;
            self.force(&term, from, Under::Operator);
            return Ok(Expr::Neg(Box::new(term)));
        }
        self.post(ends)
    }

    /// An atom with its postfixes, then `of`, `to` and `as if` (L §2.3 `post`).
    pub(super) fn post(&mut self, ends: Ends) -> Result<Expr, Stop> {
        self.enter()?;
        let expr = self.post_inner(ends)?;
        self.leave()?;
        Ok(expr)
    }

    /// `post`, inside its recursion count. The `as if` nesting it holds is counted on the way:
    /// an `as if` is one deeper than the deepest inside its expression or its pins (refcore's
    /// `asif_depth`, L §2.3).
    fn post_inner(&mut self, ends: Ends) -> Result<Expr, Stop> {
        let outer = self.asif.replace(0);
        let expr = self.post_plain(ends)?;
        let inner = self.asif.get();
        if !(self.at("as", 0) && self.at("if", 1)) {
            self.asif.set(outer.max(inner));
            return Ok(expr);
        }
        self.skip(2)?;
        self.asif.set(0);
        let pins = self.asif_pins()?;
        let depth = inner
            .max(self.asif.get())
            .checked_add(1)
            .ok_or_else(|| self.cannot("as if nested too deep"))?;
        self.asif.set(outer.max(depth));
        Ok(Expr::AsIf(Box::new(expr), pins))
    }

    /// An atom with its postfixes, then `of` and `to`.
    fn post_plain(&mut self, ends: Ends) -> Result<Expr, Stop> {
        let mut base = self.atom()?;
        let mut fixes: Vec<Postfix> = Vec::new();
        loop {
            if self.at("[", 0) {
                let chain = name_chain(&base, &fixes);
                let keys = self.keys()?;
                if self.at("(", 0)
                    && let Some(mut names) = chain
                {
                    let [band_key] = <[Key; 1]>::try_from(keys)
                        .map_err(|_| self.err("one key of a keyed band"))?;
                    let name = names.pop().ok_or_else(|| self.cannot("a band"))?;
                    base = self.band_call(name, names, Some(band_key))?;
                    fixes = Vec::new();
                    continue;
                }
                fixes.push(Postfix::Keys(keys));
            } else if self.at(".", 0) {
                // Any spacing reads: `a .b` and `a. b` are `a.b`, and `a.b (x)` a call, as
                // `f (x)` is (L §2.4, §2.3 parse rule 3; tally 2.1.6, #81). The word right
                // after `.` is a field whatever its word, so its `[` prints touching.
                self.skip(1)?;
                if !self.at_id(0) {
                    return Err(self.err("a field"));
                }
                self.mark(Some(self.i), Mark::read_as_field);
                fixes.push(Postfix::Field(self.next()?.text.clone()));
                if self.at("(", 0)
                    && let Some(mut names) = name_chain(&base, &fixes)
                {
                    // The call's name is read as an id, so the canonical form prints it touching
                    // its `(` (L §2.4 Expressions, tally 2.1.4; refcore's `tok.nm = True`).
                    self.mark(self.back(), |m| m.read = Read::Name);
                    let name = names.pop().ok_or_else(|| self.cannot("a band"))?;
                    base = self.band_call(name, names, None)?;
                    fixes = Vec::new();
                }
            } else {
                break;
            }
        }
        let ends_in_field = fixes.last().is_some_and(|fix| match fix {
            Postfix::Field(_) => true,
            Postfix::Keys(_) => false,
        });
        let mut expr = if fixes.is_empty() {
            base
        } else {
            Expr::Post(Box::new(base), fixes)
        };
        if ends_in_field && self.accept("of")? {
            expr = Expr::LegOf(Box::new(expr), Box::new(self.ofq()?));
        }
        if ends != Ends::AtTo && self.accept("to")? {
            expr = Expr::To(Box::new(expr), Box::new(self.step_unit()?));
        }
        Ok(expr)
    }

    /// A call of a band by its prefixed name.
    fn band_call(
        &mut self,
        name: String,
        prefix: Vec<String>,
        band_key: Option<Key>,
    ) -> Result<Expr, Stop> {
        let args = self.call_args()?;
        Ok(Expr::Call(Box::new(Call {
            name,
            prefix,
            band_key,
            args,
        })))
    }

    /// `[k, …]`, each a key expression or `*`.
    fn keys(&mut self) -> Result<Vec<Key>, Stop> {
        self.expect("[")?;
        self.open()?;
        let mut keys = vec![self.key_expr()?];
        while self.accept(",")? {
            keys.push(self.key_expr()?);
        }
        self.expect("]")?;
        self.close()?;
        Ok(keys)
    }

    /// A key position: `*`, or an expression; in a record field's value an expression only, as
    /// a `ref`'s keys are (refcore's `key_expr`, L §2.3 `ref`, issue #88).
    pub(super) fn key_expr(&mut self) -> Result<Key, Stop> {
        if self.at("*", 0) && self.field_ref {
            return Err(self.err("an expression"));
        }
        if self.accept("*")? {
            return Ok(Key::Star);
        }
        Ok(Key::Expr(self.expr(Ends::Free)?))
    }

    /// `UNIT`: a positive number (or `1e-N`), `minor(CUR)`, or a reference (L §7.2).
    pub(super) fn step_unit(&mut self) -> Result<Step, Stop> {
        let Some(tok) = self.peek(0) else {
            return Err(self.err("a unit step"));
        };
        if tok.kind == Kind::Key
            && let Some(places) = tok.text.strip_prefix("1e-")
            && !places.is_empty()
            && places.bytes().all(|b| b.is_ascii_digit())
        {
            let power = places
                .parse::<u32>()
                .map_err(|_| self.cannot("a step too fine"))?;
            let value = Num::int(1)
                .checked_div(&ten_to(power).map_err(|_| self.cannot("a step too fine"))?)
                .map_err(|_| self.cannot("a step too fine"))?;
            self.skip(1)?;
            return Ok(Step::Number(Number {
                value,
                whole: false,
                unit: None,
                text: tok.text.clone(),
                four: false,
                fraction: false,
            }));
        }
        match tok.kind {
            Kind::Int(_) | Kind::Dec(_) => return Ok(Step::Number(self.unsigned_number()?)),
            Kind::Id
            | Kind::Key
            | Kind::Str(_)
            | Kind::Day(_)
            | Kind::Month(..)
            | Kind::Quarter(..)
            | Kind::MonthDay(..)
            | Kind::Cur
            | Kind::Op => {}
        }
        if self.at("minor", 0) && self.at("(", 1) {
            self.skip(2)?;
            let cur = self.cur()?;
            self.expect(")")?;
            return Ok(Step::Minor(cur));
        }
        Ok(Step::Ref(self.reference()?))
    }

    /// `ref = ID ["[" keys "]"] {"." FIELD ["[" keys "]"]}` (L §2.3).
    pub(super) fn reference(&mut self) -> Result<Expr, Stop> {
        self.reference_keyed(KeysOf::Exprs, "a reference")
    }

    /// A `same` side, `sref` (L §2.3): a reference whose keys are literals (`a.konto`,
    /// `a.a_stmt[a_g1]`, `b.rent[2026-03]`, §6.1) or places keyed by literals
    /// (`opening[konto[ana]]`), in refcore's `ref` shape (one key list after the id and after
    /// each field). Any other key, `*` or a computed one, is `LOAD-PARSE`, as the grammar
    /// reads it.
    pub(super) fn same_ref(&mut self) -> Result<Expr, Stop> {
        self.reference_keyed(KeysOf::Lits, "a reference")
    }

    /// An `of` row's `sref` (L §2.3 `ofrow`): the `same` side's form, read by the one `sref`
    /// parser (issue #88), "an id" where its first word isn't one (refcore's `sref`).
    pub(super) fn sref(&mut self) -> Result<Expr, Stop> {
        self.reference_keyed(KeysOf::Lits, "an id")
    }

    /// A reference, its keys written as `keys` says; `what` names its first word.
    fn reference_keyed(&mut self, keys: KeysOf, what: &str) -> Result<Expr, Stop> {
        let name = self.ident(what)?;
        let mut fixes = Vec::new();
        if self.at("[", 0) {
            fixes.push(Postfix::Keys(self.ref_keys(keys)?));
        }
        while self.at(".", 0) && !self.spaced(0) {
            self.skip(1)?;
            if !self.at_id(0) {
                return Err(self.err("a field"));
            }
            fixes.push(Postfix::Field(self.next()?.text.clone()));
            if self.at("[", 0) {
                fixes.push(Postfix::Keys(self.ref_keys(keys)?));
            }
        }
        if fixes.is_empty() {
            return Ok(Expr::Name(name));
        }
        Ok(Expr::Post(Box::new(Expr::Name(name)), fixes))
    }

    /// A reference's key list, as `keys` says it is written.
    fn ref_keys(&mut self, keys: KeysOf) -> Result<Vec<Key>, Stop> {
        match keys {
            KeysOf::Exprs => self.keys(),
            KeysOf::Lits => self.lit_keys(),
        }
    }

    /// `"[" lit {"," lit} "]"`: an `sref`'s keys (L §2.3).
    fn lit_keys(&mut self) -> Result<Vec<Key>, Stop> {
        self.expect("[")?;
        self.open()?;
        let mut keys = vec![self.lit_key()?];
        while self.accept(",")? {
            keys.push(self.lit_key()?);
        }
        self.expect("]")?;
        self.close()?;
        Ok(keys)
    }

    /// One key of an `sref`: a literal with no sign, or a place keyed by things, `konto[ana]`
    /// (L §2.3 `sk`, tally 2.1.6), the reference `konto[ana]` refcore's `skey` reads; a sign,
    /// `*`, a range or an expression is `LOAD-PARSE` where the grammar stops (issue #88).
    fn lit_key(&mut self) -> Result<Key, Stop> {
        if !(self.at_id(0) && self.at("[", 1) && !self.spaced(1)) {
            return Ok(Key::Expr(Expr::Lit(self.lit_unsigned()?)));
        }
        let name = self.ident("an id")?;
        self.skip(1)?;
        let mut keys = vec![Key::Expr(Expr::Lit(self.lit_unsigned()?))];
        while self.accept(",")? {
            keys.push(Key::Expr(Expr::Lit(self.lit_unsigned()?)));
        }
        self.expect("]")?;
        let place = Expr::Post(Box::new(Expr::Name(name)), vec![Postfix::Keys(keys)]);
        Ok(Key::Expr(place))
    }

    /// `( [expr {, expr}] )`.
    pub(super) fn call_args(&mut self) -> Result<Vec<Expr>, Stop> {
        self.expect("(")?;
        self.open()?;
        let mut args = Vec::new();
        if !self.at(")", 0) {
            args.push(self.expr(Ends::Free)?);
            while self.accept(",")? {
                args.push(self.expr(Ends::Free)?);
            }
        }
        self.expect(")")?;
        self.close()?;
        Ok(args)
    }

    /// The tokens from `first` to the current one, as written.
    pub(super) fn source_text(&self, first: usize) -> String {
        let mut text = String::new();
        if self.blind {
            return text;
        }
        for (n, tok) in self
            .toks
            .iter()
            .enumerate()
            .skip(first)
            .take_while(|(n, _)| *n < self.i)
        {
            if n > first && tok.ws {
                text.push(' ');
            }
            text.push_str(&tok.text);
        }
        text
    }

    /// Literals separated by commas up to `close` (not taken), none if it is next.
    pub(super) fn lits_until(&mut self, close: &str) -> Result<Vec<Lit>, Stop> {
        let mut lits = Vec::new();
        if !self.at(close, 0) {
            lits.push(self.lit()?);
            while self.accept(",")? {
                lits.push(self.lit()?);
            }
        }
        Ok(lits)
    }

    /// `ofq = ref | "(" e "," e ")"`.
    pub(super) fn ofq(&mut self) -> Result<OfQ, Stop> {
        if !self.at("(", 0) {
            return Ok(OfQ::Ref(self.reference()?));
        }
        self.skip(1)?;
        self.open()?;
        let a = self.expr(Ends::Free)?;
        self.expect(",")?;
        let b = self.expr(Ends::Free)?;
        self.expect(")")?;
        self.close()?;
        Ok(OfQ::Pair(a, b))
    }

    /// An atom (L §2.3 `atom`).
    fn atom(&mut self) -> Result<Expr, Stop> {
        let Some(tok) = self.peek(0) else {
            return Err(self.err("an expression"));
        };
        match &tok.kind {
            Kind::Int(_)
            | Kind::Dec(_)
            | Kind::Day(_)
            | Kind::Month(..)
            | Kind::Quarter(..)
            | Kind::MonthDay(..)
            | Kind::Key
            | Kind::Cur => Ok(Expr::Lit(self.lit_unsigned()?)),
            Kind::Str(text) => {
                self.skip(1)?;
                Ok(Expr::Str(text.clone()))
            }
            Kind::Op => self.op_atom(&tok.text),
            Kind::Id => self.word_atom(&tok.text),
        }
    }

    /// An atom that starts with an operator: a parenthesis, a record, a key list.
    fn op_atom(&mut self, op: &str) -> Result<Expr, Stop> {
        match op {
            "(" => {
                self.skip(1)?;
                self.open()?;
                if self.at_id(0) && self.at(":", 1) {
                    let fields = self.fields(Rec::Expr)?;
                    self.expect(")")?;
                    self.close()?;
                    return Ok(Expr::Record(fields));
                }
                self.mark(self.back(), |m| m.paren = Paren::Plain);
                let inner = self.expr(Ends::Free)?;
                self.expect(")")?;
                self.close()?;
                if self.blind {
                    return Ok(inner);
                }
                Ok(Expr::Paren(Box::new(inner)))
            }
            "[" => {
                self.skip(1)?;
                self.open()?;
                let items = self.lits_until("]")?;
                self.expect("]")?;
                self.close()?;
                Ok(Expr::List(items))
            }
            "?" => Err(self.cannot("a hole stands only as a whole value")),
            _ => Err(self.err("an expression")),
        }
    }

    /// An atom that starts with a word.
    fn word_atom(&mut self, word: &str) -> Result<Expr, Stop> {
        match word {
            "none" => return self.take_expr(Expr::None),
            "never" => return self.take_expr(Expr::Never),
            "cut" if !self.at("(", 1) => return self.take_expr(Expr::Cut),
            "true" | "false" | "inf" => return Ok(Expr::Lit(self.lit_unsigned()?)),
            "nominal" if self.at_int(1) => return Ok(Expr::Lit(self.lit_unsigned()?)),
            _ => {}
        }
        if self.at("(", 1) && is_builtin(word) {
            return self.builtin(word);
        }
        if is_keyword(word) {
            return Err(self.keyword_atom(word));
        }
        self.mark(Some(self.i), |m| m.read = Read::Name);
        self.skip(1)?;
        if self.at("(", 0) {
            let args = self.call_args()?;
            return Ok(Expr::Call(Box::new(Call {
                name: word.to_owned(),
                prefix: Vec::new(),
                band_key: None,
                args,
            })));
        }
        Ok(Expr::Name(word.to_owned()))
    }

    /// Takes one token and answers `expr`.
    fn take_expr(&mut self, expr: Expr) -> Result<Expr, Stop> {
        self.skip(1)?;
        Ok(expr)
    }

    /// The refusal of a keyword where an expression starts.
    fn keyword_atom(&self, word: &str) -> Stop {
        if word == "if" {
            return self.err("a parenthesised if");
        }
        let clause = [
            "and", "or", "then", "else", "for", "where", "when", "at", "with", "as", "by", "in",
            "not", "law", "known", "governs", "assumed",
        ];
        if clause.contains(&word) {
            return self.err("an expression");
        }
        self.refuse(Load::Reserved, format!("{word} is a keyword"))
    }

    /// The fields of a record, separated by commas.
    pub(super) fn fields(&mut self, rec: Rec) -> Result<Vec<Field>, Stop> {
        let mut fields = vec![self.field(rec)?];
        while self.accept(",")? {
            fields.push(self.field(rec)?);
        }
        Ok(fields)
    }

    /// A field: `name: value`, or a leg; a positional record is `LOAD-FIELD` (L §3.1).
    fn field(&mut self, rec: Rec) -> Result<Field, Stop> {
        let named = self.at_id(0) && self.at(":", 1);
        if !named && !self.at("leg", 0) {
            return Err(self.refuse(Load::Field, "positional record; name its fields".to_owned()));
        }
        if !named {
            return Ok(Field::Leg(Box::new(self.leg()?)));
        }
        let name = self.next()?.text.clone();
        self.expect(":")?;
        Ok(Field::Named(name, self.field_value(rec)?))
    }

    /// A field's value: a row's signed literal, a hole, `none`, or an expression.
    fn field_value(&mut self, rec: Rec) -> Result<FieldValue, Stop> {
        let signed_number = self.at("-", 0)
            && self.kind(1).is_some_and(|kind| match kind {
                Kind::Int(_) | Kind::Dec(_) => true,
                Kind::Id
                | Kind::Key
                | Kind::Str(_)
                | Kind::Day(_)
                | Kind::Month(..)
                | Kind::Quarter(..)
                | Kind::MonthDay(..)
                | Kind::Cur
                | Kind::Op => false,
            });
        if rec == Rec::Row && signed_number {
            let save = self.i;
            let lit = self.lit()?;
            if self.at(",", 0) || self.at(")", 0) {
                return Ok(FieldValue::Lit(lit));
            }
            self.i = save;
        }
        if self.at("?", 0) && self.at_str(1) {
            self.skip(1)?;
            return Ok(FieldValue::Hole(self.string()?));
        }
        if self.accept("none")? {
            return Ok(FieldValue::None);
        }
        let outer = self.field_ref;
        self.field_ref = true;
        let value = self.expr(Ends::Free);
        self.field_ref = outer;
        Ok(FieldValue::Expr(value?))
    }

    /// `leg l from a to b amount q [on d]` (L §2.3 `leg`).
    pub(super) fn leg(&mut self) -> Result<Leg, Stop> {
        self.expect("leg")?;
        let name = self.ident("a leg name")?;
        self.expect("from")?;
        let from = self.expr(Ends::AtTo)?;
        self.expect("to")?;
        let to = self.expr(Ends::Free)?;
        self.expect("amount")?;
        let amount = self.expr(Ends::Free)?;
        let on = if self.accept("on")? {
            Some(self.expr(Ends::Free)?)
        } else {
            None
        };
        Ok(Leg {
            name,
            from,
            to,
            amount,
            on,
            line: self.line,
        })
    }

    /// A type (L §3.1).
    pub(super) fn ty(&mut self) -> Result<Type, Stop> {
        if self.accept("(")? {
            let mut fields = Vec::new();
            loop {
                let name = self.next()?;
                if name.kind != Kind::Id {
                    return Err(self.err("a field name"));
                }
                fields.push((name.text.clone(), self.ty()?));
                if !self.accept(",")? {
                    break;
                }
            }
            self.expect(")")?;
            return Ok(Type::Record(fields));
        }
        if !self.at_id(0) {
            return Err(self.err("a type"));
        }
        let word = self.next()?.text.clone();
        match word.as_str() {
            "money" => Ok(Type::Money(self.cur()?)),
            "num" => Ok(Type::Num(self.type_unit()?)),
            "dist" => Ok(Type::Dist(self.type_unit()?)),
            "key" => {
                let mut id = self.ident("an index or kind")?;
                if self.at(".", 0) && !self.spaced(0) {
                    self.skip(1)?;
                    id = format!("{id}.{}", self.ident("an id")?);
                }
                Ok(Type::Key(id))
            }
            "int" | "bool" | "text" | "date" | "dates" | "month" | "monthday" | "quarter"
            | "year" | "rate" | "calendar" | "rounding" | "daycount" | "roll" | "compounding" => {
                Ok(Type::Word(word))
            }
            _ => {
                self.i = self.i.checked_sub(1).ok_or_else(|| self.cannot("a type"))?;
                Err(self.err("a type"))
            }
        }
    }

    /// `qid = ID ["." ID]`: an id with a member's prefix.
    pub(super) fn qid(&mut self) -> Result<(Option<String>, String), Stop> {
        let first = self.ident("an id")?;
        if self.at(".", 0) && !self.spaced(0) && self.at_id(1) {
            self.skip(1)?;
            return Ok((Some(first), self.ident("an id")?));
        }
        Ok((None, first))
    }

    /// A row's key: `*`, a place keyed by things, `a .. b`, or a key (L §2.3 `rk`).
    pub(super) fn row_key(&mut self) -> Result<RowKey, Stop> {
        if self.accept("*")? {
            return Ok(RowKey::Star);
        }
        if self.at_id(0) && self.at("[", 1) {
            let name = self.ident("an id")?;
            self.skip(1)?;
            let mut keys = vec![self.lit()?];
            while self.accept(",")? {
                keys.push(self.lit()?);
            }
            self.expect("]")?;
            return Ok(RowKey::Place(name, keys));
        }
        let first = self.lit()?;
        if self.accept("..")? {
            return Ok(RowKey::Range(first, self.lit()?));
        }
        Ok(RowKey::Lit(first))
    }
}

/// One term alone, or the chain `make` builds of two or more.
fn single_or(mut terms: Vec<Expr>, make: fn(Vec<Expr>) -> Expr) -> Expr {
    if terms.len() == 1
        && let Some(only) = terms.pop()
    {
        return only;
    }
    make(terms)
}

/// The names of a name or a chain of fields over a name (`a.b.c`), or none.
fn name_chain(base: &Expr, fixes: &[Postfix]) -> Option<Vec<String>> {
    let Expr::Name(first) = base else {
        return None;
    };
    let mut names = vec![first.clone()];
    for fix in fixes {
        match fix {
            Postfix::Field(name) => names.push(name.clone()),
            Postfix::Keys(_) => return None,
        }
    }
    Some(names)
}

/// `10^power`, exactly.
fn ten_to(power: u32) -> Result<Num, crate::num::NumError> {
    let mut value = Num::int(1);
    for _ in 0..power {
        value = value.checked_mul(&Num::int(10))?;
    }
    Ok(value)
}
