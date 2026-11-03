//! The expression grammar: precedence climbing over flat chains, postfixes, atoms and the reads.
//! Spec: L §2.3 (`expr` to `atom`, `binds`, `span`, `tieby`, `balx`, `valuex`, `flowsx`, `coveredx`,
//! `agg`, `calval`, `record`, `leg`, `type`), its seven parse rules, and its depth limit.
//! Twin: engine/tools/refcore/parser.py (`expr` to `call_args`, `span` to `leg_body`,
//! `builtin`, `agg`, `calval` to `dayset`, `price`, `type`, `bindlist`, `qid`, `rowkey`).
//! Invariants: every loop takes a token or ends; each bracket and `if` is counted by `open`;
//! a construct the engine doesn't build yet stops with `NotBuilt`, never a guessed refusal.

use super::lex::Kind;
use super::parse::{P, Stop, is_keyword};
use super::tree::{
    AddOp, Agg, AggFn, Bal, Bind, Calendar, CalendarDay, Call, CmpOp, DaySet, Domain, Expr, Field,
    FieldValue, Flows, In, Key, Leg, Lit, MulOp, Number, OfQ, Postfix, Price, Proj, RowKey, Span,
    Step, Type, Valuation,
};
use crate::base::Load;
use crate::num::Num;
use crate::time::Day;

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

/// Whether a record is a row's, whose fields may be signed literals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Rec {
    /// A row's record.
    Row,
    /// An expression's record.
    Expr,
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
fn is_builtin(word: &str) -> bool {
    !word.is_empty() && BUILTINS.contains(&word)
}

/// The aggregate named `word`, if it is one.
fn agg_fn(word: &str) -> Option<AggFn> {
    match word {
        "sum" => Some(AggFn::Sum),
        "count" => Some(AggFn::Count),
        "min" => Some(AggFn::Min),
        "max" => Some(AggFn::Max),
        "any" => Some(AggFn::Any),
        "all" => Some(AggFn::All),
        "first" => Some(AggFn::First),
        "last" => Some(AggFn::Last),
        "argmin" => Some(AggFn::Argmin),
        "argmax" => Some(AggFn::Argmax),
        _ => None,
    }
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

/// The constructs with a production of their own that the engine doesn't build yet.
fn unbuilt(word: &str) -> Option<&'static str> {
    match word {
        "mean" | "share" | "quantile" => Some("queries (mean, share, quantile)"),
        "low" => Some("low"),
        "crps" => Some("crps"),
        "cdf" | "pdf" | "pmf" => Some("cdf, pdf and pmf"),
        "normal" | "lognormal" | "exponential" | "gamma" | "uniform" | "poisson" | "binomial"
        | "negbin" | "discrete" => Some("distributions"),
        "prorata" => Some("prorata"),
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
        let mut terms = vec![self.and_chain(ends)?];
        while self.accept("or")? {
            terms.push(self.and_chain(ends)?);
        }
        Ok(single_or(terms, Expr::Or))
    }

    /// `a and b and …`.
    fn and_chain(&mut self, ends: Ends) -> Result<Expr, Stop> {
        let mut terms = vec![self.not_term(ends)?];
        while self.accept("and")? {
            terms.push(self.not_term(ends)?);
        }
        Ok(single_or(terms, Expr::And))
    }

    /// `[not] cmp`: one `not` at most.
    fn not_term(&mut self, ends: Ends) -> Result<Expr, Stop> {
        if self.accept("not")? {
            return Ok(Expr::Not(Box::new(self.cmp(ends)?)));
        }
        self.cmp(ends)
    }

    /// A comparison or a membership test; neither chains (parse rule 1).
    fn cmp(&mut self, ends: Ends) -> Result<Expr, Stop> {
        let left = self.sum(ends)?;
        if let Some(op) = self
            .peek(0)
            .filter(|tok| tok.kind == Kind::Op)
            .and_then(|tok| cmp_op(&tok.text))
        {
            self.skip(1)?;
            let right = self.sum(ends)?;
            self.no_chain()?;
            return Ok(Expr::Cmp(Box::new((left, op, right))));
        }
        let world = self.ask_for && self.depth_is_zero() && self.world_in();
        if self.at("in", 0) && ends != Ends::AtIn && !world {
            self.skip(1)?;
            let set = self.sum(ends)?;
            let on = if self.accept("on")? {
                Some(self.sum(ends)?)
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
        let first = self.prod(ends)?;
        let mut rest = Vec::new();
        loop {
            let op = match self.op_text() {
                Some("+") => AddOp::Add,
                Some("-") => AddOp::Sub,
                Some(_) | None => break,
            };
            self.skip(1)?;
            rest.push((op, self.prod(ends)?));
        }
        if rest.is_empty() {
            return Ok(first);
        }
        Ok(Expr::Sum(Box::new(first), rest))
    }

    /// `a × b ÷ …`, kept flat.
    fn prod(&mut self, ends: Ends) -> Result<Expr, Stop> {
        let first = self.unary(ends)?;
        let mut rest = Vec::new();
        loop {
            let op = match self.op_text() {
                Some("*") => MulOp::Mul,
                Some("/") => MulOp::Div,
                Some(_) | None => break,
            };
            self.skip(1)?;
            rest.push((op, self.unary(ends)?));
        }
        if rest.is_empty() {
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
            return Ok(Expr::Neg(Box::new(self.post(ends)?)));
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

    /// `post`, inside its recursion count.
    fn post_inner(&mut self, ends: Ends) -> Result<Expr, Stop> {
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
            } else if self.at(".", 0) && !self.spaced(0) {
                self.skip(1)?;
                if !self.at_id(0) {
                    return Err(self.err("a field"));
                }
                fixes.push(Postfix::Field(self.next()?.text.clone()));
                if self.at("(", 0)
                    && !self.spaced(0)
                    && let Some(mut names) = name_chain(&base, &fixes)
                {
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
        if self.at("as", 0) && self.at("if", 1) {
            return Err(Stop::NotBuilt("as if"));
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

    /// A key position: `*`, or an expression.
    fn key_expr(&mut self) -> Result<Key, Stop> {
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
        let name = self.ident("a reference")?;
        let mut fixes = Vec::new();
        if self.at("[", 0) {
            fixes.push(Postfix::Keys(self.keys()?));
        }
        while self.at(".", 0) && !self.spaced(0) {
            self.skip(1)?;
            if !self.at_id(0) {
                return Err(self.err("a field"));
            }
            fixes.push(Postfix::Field(self.next()?.text.clone()));
            if self.at("[", 0) {
                fixes.push(Postfix::Keys(self.keys()?));
            }
        }
        if fixes.is_empty() {
            return Ok(Expr::Name(name));
        }
        Ok(Expr::Post(Box::new(Expr::Name(name)), fixes))
    }

    /// `( [expr {, expr}] )`.
    fn call_args(&mut self) -> Result<Vec<Expr>, Stop> {
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

    /// `span = expr ".." [expr] | ".." [expr] | expr` (L §2.3).
    pub(super) fn span(&mut self) -> Result<Span, Stop> {
        if self.accept("..")? {
            let to = if self.span_end() {
                None
            } else {
                Some(self.expr(Ends::Free)?)
            };
            return Ok(Span {
                from: None,
                to,
                dots: true,
            });
        }
        let from = self.expr(Ends::Free)?;
        if self.accept("..")? {
            let to = if self.span_end() {
                None
            } else {
                Some(self.expr(Ends::Free)?)
            };
            return Ok(Span {
                from: Some(from),
                to,
                dots: true,
            });
        }
        Ok(Span {
            from: Some(from),
            to: None,
            dots: false,
        })
    }

    /// Whether a span ends here, its end left open.
    fn span_end(&self) -> bool {
        let Some(tok) = self.peek(0) else {
            return true;
        };
        match tok.kind {
            Kind::Op => [",", ")", "]", "}"].contains(&tok.text.as_str()),
            Kind::Id => [
                "where", "by", "for", "law", "with", "as", "windows", "random", "paths", "path",
                "partial", "orient", "in", "from",
            ]
            .contains(&tok.text.as_str()),
            Kind::Key
            | Kind::Int(_)
            | Kind::Dec(_)
            | Kind::Str(_)
            | Kind::Day(_)
            | Kind::Month(..)
            | Kind::Quarter(..)
            | Kind::MonthDay(..)
            | Kind::Cur => false,
        }
    }

    /// Whether a binding starts here: an id that is no keyword, then `in` (parse rule 6).
    fn is_bind_start(&self) -> bool {
        self.peek(0)
            .is_some_and(|tok| tok.kind == Kind::Id && !is_keyword(&tok.text))
            && self.at("in", 1)
    }

    /// A binding, with its text as written.
    pub(super) fn bind(&mut self) -> Result<Bind, Stop> {
        let first = self.i;
        let mut bind = self.bind_inner()?;
        bind.text = self.source_text(first);
        Ok(bind)
    }

    /// The tokens from `first` to the current one, as written.
    fn source_text(&self, first: usize) -> String {
        let mut text = String::new();
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

    /// `bind = ID "in" (qid | "[" keys "]" | range) ["on" e | "over" span] ["where" e]`.
    fn bind_inner(&mut self) -> Result<Bind, Stop> {
        let var = self.ident("a bound variable")?;
        self.expect("in")?;
        let domain = if self.at("[", 0) {
            self.skip(1)?;
            self.open()?;
            let keys = self.lits_until("]")?;
            self.expect("]")?;
            self.close()?;
            Domain::List(keys)
        } else {
            let from = self.expr(Ends::AtIn)?;
            if self.accept("..")? {
                let to = self.expr(Ends::AtIn)?;
                let step = if self.accept("step")? {
                    Some(self.expr(Ends::AtIn)?)
                } else {
                    None
                };
                Domain::Range(Box::new((from, to, step)))
            } else {
                self.domain_name(from)?
            }
        };
        let (mut on, mut over) = (None, None);
        if self.accept("on")? {
            on = Some(self.expr(Ends::Free)?);
        } else if self.accept("over")? {
            over = Some(self.span()?);
        }
        let filter = if self.accept("where")? {
            Some(self.expr(Ends::Free)?)
        } else {
            None
        };
        Ok(Bind {
            var,
            domain,
            on,
            over,
            filter,
            text: String::new(),
        })
    }

    /// A binding's index or kind: `ix`, or a member's `a.ix`.
    fn domain_name(&self, expr: Expr) -> Result<Domain, Stop> {
        if let Expr::Name(id) = expr {
            return Ok(Domain::Name { id, qual: None });
        }
        if let Expr::Post(base, fixes) = expr
            && let (Expr::Name(qual), [Postfix::Field(id)]) = (*base, fixes.as_slice())
        {
            return Ok(Domain::Name {
                id: id.clone(),
                qual: Some(qual),
            });
        }
        Err(self.err("an index, a kind, a key list or a range"))
    }

    /// Literals separated by commas up to `close` (not taken), none if it is next.
    fn lits_until(&mut self, close: &str) -> Result<Vec<Lit>, Stop> {
        let mut lits = Vec::new();
        if !self.at(close, 0) {
            lits.push(self.lit()?);
            while self.accept(",")? {
                lits.push(self.lit()?);
            }
        }
        Ok(lits)
    }

    /// `binds = bind {"," bind}`, a comma continuing only before another `ID in`.
    pub(super) fn binds(&mut self) -> Result<Vec<Bind>, Stop> {
        let mut out = vec![self.bind()?];
        while self.at(",", 0) && self.at_id(1) && self.at("in", 2) {
            self.skip(1)?;
            out.push(self.bind()?);
        }
        Ok(out)
    }

    /// `"[" bind {"," bind} "]"`, a declaration's keys.
    pub(super) fn bind_list(&mut self) -> Result<Vec<Bind>, Stop> {
        self.expect("[")?;
        self.open()?;
        let mut out = vec![self.bind()?];
        while self.accept(",")? {
            out.push(self.bind()?);
        }
        self.expect("]")?;
        self.close()?;
        Ok(out)
    }

    /// `ties by (post | "(" e "," e {"," e} ")")` (L §2.3 `tieby`).
    pub(super) fn ties(&mut self) -> Result<Vec<Expr>, Stop> {
        self.expect("ties")?;
        self.expect("by")?;
        if self.at("(", 0) {
            let save = self.i;
            self.skip(1)?;
            self.open()?;
            let first = self.expr(Ends::Free)?;
            if self.accept(",")? {
                let mut items = vec![first, self.expr(Ends::Free)?];
                while self.accept(",")? {
                    items.push(self.expr(Ends::Free)?);
                }
                self.expect(")")?;
                self.close()?;
                return Ok(items);
            }
            self.close()?;
            self.i = save;
        }
        Ok(vec![self.post(Ends::Free)?])
    }

    /// `ofq = ref | "(" e "," e ")"`.
    fn ofq(&mut self) -> Result<OfQ, Stop> {
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

    /// A projection's clauses: `from`, `windows`, `random at`, `with uncertain`.
    fn proj(&mut self) -> Result<Proj, Stop> {
        let from = if self.accept("from")? {
            Some(self.post(Ends::Free)?)
        } else {
            None
        };
        let windows_each = if self.accept("windows")? {
            self.each()?;
            true
        } else {
            false
        };
        if self.at("random", 0) && self.at("at", 1) {
            return Err(Stop::NotBuilt("random at"));
        }
        let uncertain = self.with_uncertain()?;
        Ok(Proj {
            from,
            windows_each,
            uncertain,
        })
    }

    /// `with uncertain`, taken if next.
    pub(super) fn with_uncertain(&mut self) -> Result<bool, Stop> {
        if self.at("with", 0) && self.at("uncertain", 1) {
            self.skip(2)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// `wchoice = "each"`.
    pub(super) fn each(&mut self) -> Result<(), Stop> {
        if self.at("each", 0) {
            return self.skip(1);
        }
        Err(self.err("each"))
    }

    /// `MODE`: a rounding code's key token, or a reference.
    fn mode(&mut self) -> Result<Expr, Stop> {
        if self.kind(0) == Some(&Kind::Key) {
            return Ok(Expr::Lit(Lit::Key(self.next()?.text.clone())));
        }
        self.reference()
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
                let inner = self.expr(Ends::Free)?;
                self.expect(")")?;
                self.close()?;
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
        Ok(FieldValue::Expr(self.expr(Ends::Free)?))
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
        })
    }

    /// A built-in, `word` directly before `(`.
    fn builtin(&mut self, word: &str) -> Result<Expr, Stop> {
        if let Some(func) = agg_fn(word) {
            return self.agg(func);
        }
        if let Some(what) = unbuilt(word) {
            return Err(Stop::NotBuilt(what));
        }
        match word {
            "bal" => self.bal(),
            "flows" => self.flows(),
            "covered" => self.covered(),
            "value" => self.valuation(),
            "calendar" => Ok(Expr::Calendar(Box::new(self.calendar()?))),
            _ => {
                self.skip(1)?;
                let args = self.call_args()?;
                Ok(Expr::Call(Box::new(Call {
                    name: word.to_owned(),
                    prefix: Vec::new(),
                    band_key: None,
                    args,
                })))
            }
        }
    }

    /// `bal(x [, [e,] e [start]]) [of q] [proj | modelled]` (L §2.3 `balx`, parse rule 7).
    fn bal(&mut self) -> Result<Expr, Stop> {
        self.skip(1)?;
        self.expect("(")?;
        self.open()?;
        let place = self.reference()?;
        let mut args = Vec::new();
        let mut start = false;
        if self.accept(",")? {
            args.push(self.expr(Ends::Free)?);
            if self.accept(",")? {
                args.push(self.expr(Ends::Free)?);
            }
            start = self.accept("start")?;
        }
        self.expect(")")?;
        self.close()?;
        let of = if self.accept("of")? {
            Some(self.ofq()?)
        } else {
            None
        };
        let modelled = self.accept("modelled")?;
        let proj = if modelled {
            Proj {
                from: None,
                windows_each: false,
                uncertain: false,
            }
        } else if self.at("from", 0) {
            self.proj()?
        } else {
            Proj {
                from: None,
                windows_each: false,
                uncertain: self.with_uncertain()?,
            }
        };
        Ok(Expr::Bal(Box::new(Bal {
            place,
            args,
            start,
            modelled,
            proj,
            of,
        })))
    }

    /// `value(x, d) in CUR [by p | by last(p)] [of q]`: a value's `of` comes after its `by`
    /// (L §2.3 `valuex`, parse rule 7).
    fn valuation(&mut self) -> Result<Expr, Stop> {
        self.skip(1)?;
        self.expect("(")?;
        self.open()?;
        let place = self.reference()?;
        self.expect(",")?;
        let day = self.expr(Ends::Free)?;
        self.expect(")")?;
        self.close()?;
        self.expect("in")?;
        let currency = self.cur()?;
        let by = if self.accept("by")? {
            Some(self.price()?)
        } else {
            None
        };
        let of = if self.accept("of")? {
            Some(self.ofq()?)
        } else {
            None
        };
        Ok(Expr::Value(Box::new(Valuation {
            place,
            day,
            currency,
            by,
            of,
        })))
    }

    /// `flows(x, span [, for k]) [in CUR at fx MODE]` (L §2.3 `flowsx`, parse rule 4).
    fn flows(&mut self) -> Result<Expr, Stop> {
        self.skip(1)?;
        self.expect("(")?;
        self.open()?;
        let target = self.reference()?;
        self.expect(",")?;
        let span = self.span()?;
        let for_key = if self.accept(",")? {
            self.expect("for")?;
            Some(self.expr(Ends::Free)?)
        } else {
            None
        };
        self.expect(")")?;
        self.close()?;
        let convert = if self.at("in", 0) && self.kind(1) == Some(&Kind::Cur) {
            self.skip(1)?;
            let cur = self.cur()?;
            self.expect("at")?;
            let rate = self.reference()?;
            let mode = self.mode()?;
            Some((cur, rate, mode))
        } else {
            None
        };
        Ok(Expr::Flows(Box::new(Flows {
            target,
            span,
            for_key,
            convert,
        })))
    }

    /// `covered(x, span)` (L §2.3 `coveredx`).
    fn covered(&mut self) -> Result<Expr, Stop> {
        self.skip(1)?;
        self.expect("(")?;
        self.open()?;
        let place = self.expr(Ends::Free)?;
        self.expect(",")?;
        let span = self.span()?;
        self.expect(")")?;
        self.close()?;
        Ok(Expr::Covered(Box::new((place, span))))
    }

    /// An aggregate; `min` and `max` with plain arguments are calls; `last(x, d)` a carry.
    fn agg(&mut self, func: AggFn) -> Result<Expr, Stop> {
        self.skip(1)?;
        self.expect("(")?;
        self.open()?;
        let (term, binds) = if self.is_bind_start() {
            (None, self.binds()?)
        } else {
            let first = self.expr(Ends::Free)?;
            if self.accept("for")? {
                (Some(first), self.binds()?)
            } else if (func == AggFn::Min || func == AggFn::Max) && self.at(",", 0) {
                return self.plain_min_max(func, first);
            } else if func == AggFn::Last && self.accept(",")? {
                let day = self.expr(Ends::Free)?;
                self.expect(")")?;
                self.close()?;
                return Ok(Expr::Last(Box::new((first, day))));
            } else {
                return Err(self.err("\"for\" or a binding"));
            }
        };
        self.expect(")")?;
        self.close()?;
        let ties = if self.at("ties", 0) && self.at("by", 1) {
            Some(self.ties()?)
        } else {
            None
        };
        Ok(Expr::Agg(Box::new(Agg {
            func,
            term,
            binds,
            ties,
        })))
    }

    /// `min(a, b, …)` or `max(a, b, …)`: a call of two or more plain arguments.
    fn plain_min_max(&mut self, func: AggFn, first: Expr) -> Result<Expr, Stop> {
        let mut args = vec![first];
        while self.accept(",")? {
            args.push(self.expr(Ends::Free)?);
        }
        self.expect(")")?;
        self.close()?;
        let name = if func == AggFn::Min { "min" } else { "max" };
        Ok(Expr::Call(Box::new(Call {
            name: name.to_owned(),
            prefix: Vec::new(),
            band_key: None,
            args,
        })))
    }

    /// `calendar(weekends [...], [yearly [...] [from n]], [added [...]])` (L §4.2).
    pub(super) fn calendar(&mut self) -> Result<Calendar, Stop> {
        self.expect("calendar")?;
        self.expect("(")?;
        self.open()?;
        self.expect("weekends")?;
        self.expect("[")?;
        let mut weekends = vec![self.ident("a weekday")?];
        while self.accept(",")? {
            weekends.push(self.ident("a weekday")?);
        }
        self.expect("]")?;
        let mut yearly = Vec::new();
        let mut yearly_from = None;
        if self.at(",", 0) && self.at("yearly", 1) {
            self.skip(2)?;
            self.expect("[")?;
            if !self.at("]", 0) {
                yearly.push(self.month_day()?);
                while self.accept(",")? {
                    yearly.push(self.month_day()?);
                }
            }
            self.expect("]")?;
            if self.accept("from")? {
                yearly_from = Some(self.integer()?);
            }
        }
        let mut added = Vec::new();
        if self.at(",", 0) && self.at("added", 1) {
            self.skip(2)?;
            self.expect("[")?;
            if !self.at("]", 0) {
                added.push(self.calendar_day()?);
                while self.accept(",")? {
                    added.push(self.calendar_day()?);
                }
            }
            self.expect("]")?;
        }
        self.expect(")")?;
        self.close()?;
        Ok(Calendar {
            weekends,
            yearly,
            yearly_from,
            added,
        })
    }

    /// A month-day every year has (L §3.1: never `02-29`).
    fn month_day(&mut self) -> Result<(u8, u8), Stop> {
        let Some(tok) = self.peek(0) else {
            return Err(self.err("a month-day (MM-DD)"));
        };
        let Kind::MonthDay(month, day) = tok.kind else {
            return Err(self.err("a month-day (MM-DD)"));
        };
        if Day::from_ymd(2001, month, day).is_none() {
            return Err(self.cannot(&format!("{} is not a month-day every year has", tok.text)));
        }
        self.skip(1)?;
        Ok((month, day))
    }

    /// An added day: a day, a hole, a window or a choice.
    fn calendar_day(&mut self) -> Result<CalendarDay, Stop> {
        match self.peek(0).map(|tok| (&tok.kind, &tok.text)) {
            Some((Kind::Day(day), _)) => {
                self.skip(1)?;
                Ok(CalendarDay::Day(*day))
            }
            Some((Kind::MonthDay(..), text)) => {
                Err(self.cannot(&format!("a month-day {text} in added; added takes days")))
            }
            _ if self.at("?", 0) && self.at_str(1) => {
                self.skip(1)?;
                Ok(CalendarDay::Hole(self.string()?))
            }
            _ => Ok(CalendarDay::Set(self.day_set()?)),
        }
    }

    /// `window(a, b)` or `one_of([a, …])` (L §4.4).
    pub(super) fn day_set(&mut self) -> Result<DaySet, Stop> {
        if self.at("window", 0) && self.at("(", 1) {
            self.skip(2)?;
            let a = self.day()?;
            self.expect(",")?;
            let b = self.day()?;
            self.expect(")")?;
            return Ok(DaySet::Window(a, b));
        }
        if self.at("one_of", 0) && self.at("(", 1) {
            self.skip(2)?;
            self.expect("[")?;
            let mut days = vec![self.day()?];
            while self.accept(",")? {
                days.push(self.day()?);
            }
            self.expect("]")?;
            self.expect(")")?;
            return Ok(DaySet::OneOf(days));
        }
        Err(self.err("a window or one_of"))
    }

    /// `price = ref | "last" "(" ref ")"`.
    pub(super) fn price(&mut self) -> Result<Price, Stop> {
        if self.at("last", 0) && self.at("(", 1) {
            self.skip(2)?;
            let input = self.reference()?;
            self.expect(")")?;
            return Ok(Price { input, last: true });
        }
        Ok(Price {
            input: self.reference()?,
            last: false,
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
