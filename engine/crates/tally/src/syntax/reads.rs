//! The reads with a production of their own: queries, `low`, `crps`, distributions and their
//! functions, `prorata`, a random choice, and a reading row's `of`.
//! Spec: L §2.3 (`query`, `lowx`, `crps`, `distfn`, `dist`, `prorata`, `rchoice`, `ofrow`, `sref`),
//! §5.11 (queries over keys), §5.13 (`prorata`), §5.14.1 (`low`), §5.15 (`crps`), §5.16
//! (distributions, random values' queries), §5.19 (a reading's parts).
//! Twin: engine/tools/refcore/parser.py (`builtin`'s `low`, `crps`, `cdf`/`pdf`/`pmf`, `DISTS`
//! and `discrete` arms; `query`, `prorata`, `rchoice`, `sref`, `ofrow`).
//! Invariants: each bracket is counted by `open` as refcore counts it (an `ofrow` pair's
//! parentheses aren't, as in refcore); the error texts are refcore's word for word.

use super::expr::Ends;
use super::lex::Kind;
use super::parse::{P, Stop};
use super::tree::{
    Crps, Dist, DistFn, DistFnKind, DistKind, Expr, Key, Low, OfRow, Postfix, Prorata, Query,
    QueryFn, RChoice, Rest,
};

/// The query named `word`, if it is one (L §2.2).
pub(super) fn query_fn(word: &str) -> Option<QueryFn> {
    match word {
        "mean" => Some(QueryFn::Mean),
        "share" => Some(QueryFn::Share),
        "quantile" => Some(QueryFn::Quantile),
        _ => None,
    }
}

/// The distribution named `word`, if it is one (L §5.16).
pub(super) fn dist_kind(word: &str) -> Option<DistKind> {
    match word {
        "normal" => Some(DistKind::Normal),
        "lognormal" => Some(DistKind::Lognormal),
        "exponential" => Some(DistKind::Exponential),
        "gamma" => Some(DistKind::Gamma),
        "uniform" => Some(DistKind::Uniform),
        "poisson" => Some(DistKind::Poisson),
        "binomial" => Some(DistKind::Binomial),
        "negbin" => Some(DistKind::Negbin),
        "discrete" => Some(DistKind::Discrete),
        _ => None,
    }
}

impl P<'_> {
    /// `mean(e (for binds | [where c]))`, `share(…)`, `quantile(e …, q)` (L §2.3 `query`).
    pub(super) fn query(&mut self, func: QueryFn) -> Result<Expr, Stop> {
        self.skip(1)?;
        self.expect("(")?;
        self.open()?;
        let arg = self.expr(Ends::Free)?;
        let (mut binds, mut filter) = (None, None);
        if self.accept("for")? {
            binds = Some(self.binds()?);
        } else if self.accept("where")? {
            filter = Some(self.expr(Ends::Free)?);
        }
        let level = if func == QueryFn::Quantile {
            self.expect(",")?;
            Some(self.expr(Ends::Free)?)
        } else {
            None
        };
        self.expect(")")?;
        self.close()?;
        Ok(Expr::Query(Box::new(Query {
            func,
            arg,
            binds,
            filter,
            level,
        })))
    }

    /// `low(x, span)` and its projection's clauses (L §2.3 `lowx`).
    pub(super) fn low(&mut self) -> Result<Expr, Stop> {
        self.skip(1)?;
        self.expect("(")?;
        self.open()?;
        let place = self.reference()?;
        self.expect(",")?;
        let span = self.span()?;
        self.expect(")")?;
        self.close()?;
        let proj = self.proj()?;
        Ok(Expr::Low(Box::new(Low { place, span, proj })))
    }

    /// `crps(e [weight w] for bind, y)` or `crps(e, y)` (L §2.3 `crps`).
    pub(super) fn crps(&mut self) -> Result<Expr, Stop> {
        self.skip(1)?;
        self.expect("(")?;
        self.open()?;
        let forecast = self.expr(Ends::Free)?;
        let (mut weight, mut bind) = (None, None);
        if self.accept("weight")? {
            weight = Some(self.expr(Ends::Free)?);
            self.expect("for")?;
            bind = Some(self.bind()?);
        } else if self.accept("for")? {
            bind = Some(self.bind()?);
        }
        self.expect(",")?;
        let outcome = self.expr(Ends::Free)?;
        self.expect(")")?;
        self.close()?;
        Ok(Expr::Crps(Box::new(Crps {
            forecast,
            weight,
            bind,
            outcome,
        })))
    }

    /// `cdf(e, x)`, `pdf(e, x)`, `pmf(e, x)`: two arguments, checked once both are read.
    pub(super) fn dist_fn(&mut self, func: DistFnKind) -> Result<Expr, Stop> {
        self.skip(1)?;
        let args = self.call_args()?;
        let Ok([value, at]) = <[Expr; 2]>::try_from(args) else {
            return Err(self.err("two arguments"));
        };
        Ok(Expr::DistFn(Box::new(DistFn { func, value, at })))
    }

    /// A distribution: `normal(a, b)` and the like, or `discrete(e for bind)` (L §2.3 `dist`).
    pub(super) fn dist(&mut self, func: DistKind) -> Result<Expr, Stop> {
        self.skip(1)?;
        if func != DistKind::Discrete {
            let args = self.call_args()?;
            return Ok(Expr::Dist(Box::new(Dist {
                func,
                args,
                term: None,
                bind: None,
            })));
        }
        self.expect("(")?;
        self.open()?;
        let term = self.expr(Ends::Free)?;
        self.expect("for")?;
        let bind = self.bind()?;
        self.expect(")")?;
        self.close()?;
        Ok(Expr::Dist(Box::new(Dist {
            func,
            args: Vec::new(),
            term: Some(term),
            bind: Some(bind),
        })))
    }

    /// `prorata(total, w for binds, k, u, MODE, rest none|last)` or
    /// `prorata(total, w for binds, k, u, rest largest [ties by …])` (L §2.3 `prorata`).
    pub(super) fn prorata(&mut self) -> Result<Expr, Stop> {
        self.skip(1)?;
        self.expect("(")?;
        self.open()?;
        let total = self.expr(Ends::Free)?;
        self.expect(",")?;
        let weight = self.expr(Ends::Free)?;
        self.expect("for")?;
        let binds = self.binds()?;
        self.expect(",")?;
        let key = self.expr(Ends::Free)?;
        self.expect(",")?;
        let unit = self.step_unit()?;
        self.expect(",")?;
        let (mut mode, mut ties) = (None, None);
        let rest = if self.accept("rest")? {
            self.expect("largest")?;
            if self.at("ties", 0) {
                ties = Some(self.ties()?);
            }
            Rest::Largest
        } else {
            mode = Some(self.mode()?);
            self.expect(",")?;
            self.expect("rest")?;
            if self.accept("none")? {
                Rest::None
            } else {
                self.expect("last")?;
                Rest::Last
            }
        };
        self.expect(")")?;
        self.close()?;
        Ok(Expr::Prorata(Box::new(Prorata {
            total,
            weight,
            binds,
            key,
            unit,
            mode,
            rest,
            ties,
        })))
    }

    /// `rchoice = "mean" | "quantile" "(" expr ")"`, after `random at` (L §5.16).
    pub(super) fn rchoice(&mut self) -> Result<RChoice, Stop> {
        if self.accept("mean")? {
            return Ok(RChoice::Mean);
        }
        if !self.at("quantile", 0) {
            return Err(self.err("mean or quantile(q)"));
        }
        self.skip(1)?;
        self.expect("(")?;
        self.open()?;
        let level = self.expr(Ends::Free)?;
        self.expect(")")?;
        self.close()?;
        Ok(RChoice::Quantile(level))
    }

    /// `sref = ID {"." FIELD | "[" lit {"," lit} "]"}`: a kind, thing, row or movement, its keys
    /// written as literals (L §2.3).
    pub(super) fn sref(&mut self) -> Result<Expr, Stop> {
        let name = self.ident("an id")?;
        let mut fixes = Vec::new();
        loop {
            if self.at(".", 0) && !self.spaced(0) {
                self.skip(1)?;
                let tok = self.next()?;
                if tok.kind != Kind::Id {
                    return Err(self.err("a field"));
                }
                fixes.push(Postfix::Field(tok.text.clone()));
            } else if self.accept("[")? {
                let mut keys = vec![Key::Expr(Expr::Lit(self.lit()?))];
                while self.accept(",")? {
                    keys.push(Key::Expr(Expr::Lit(self.lit()?)));
                }
                self.expect("]")?;
                fixes.push(Postfix::Keys(keys));
            } else {
                break;
            }
        }
        if fixes.is_empty() {
            return Ok(Expr::Name(name));
        }
        Ok(Expr::Post(Box::new(Expr::Name(name)), fixes))
    }

    /// `ofrow = sref | "(" sref "," sref ")"`: a party, a batch, or the pair (L §2.3, §3.6).
    pub(super) fn of_row(&mut self) -> Result<OfRow, Stop> {
        if !self.accept("(")? {
            return Ok(OfRow::One(self.sref()?));
        }
        let first = self.sref()?;
        self.expect(",")?;
        let second = self.sref()?;
        self.expect(")")?;
        Ok(OfRow::Pair(first, second))
    }
}
