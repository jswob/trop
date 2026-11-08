//! The binding grammar: spans, bindings over an index, a kind, a key list or a range, and a `ties by` term.
//! Spec: L §2.3 (`span`, `binds`, `bind`, `range`, `tieby`), its parse rule 6 (an `ID in` at a
//! binding's place is a binding, never a membership test) and parse rule 7 (`ties by` takes
//! one term), and its depth limit.
//! Twin: engine/tools/refcore/parser.py (`span`, `_span_end`, `is_bind_start`, `bind`,
//! `bind_`, `binds`, `bindlist`, `tieby`, `tieby_term`).
//! Invariants: every loop takes a token or ends; each bracket is counted by `open` as refcore
//! counts it; a binding keeps its text as written.

use super::expr::Ends;
use super::lex::{Kind, Role};
use super::parse::{P, Stop, is_keyword};
use super::tree::{Bind, Domain, Expr, Postfix, Span};

impl P<'_> {
    /// `span = expr ".." [expr] | ".." [expr] | expr` (L §2.3).
    pub(super) fn span(&mut self) -> Result<Span, Stop> {
        if self.at("..", 0) {
            self.role(Some(self.i), Role::Bin);
        }
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
        if self.at("..", 0) {
            self.role(Some(self.i), Role::Bin);
        }
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
    pub(super) fn is_bind_start(&self) -> bool {
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

    /// `bind = ID "in" (qid | "[" keys "]" | range) ["on" e | "over" span] ["where" e]`.
    fn bind_inner(&mut self) -> Result<Bind, Stop> {
        let site = self.site();
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
            if self.at("..", 0) {
                self.role(Some(self.i), Role::Bin);
            }
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
            site,
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

    /// `ties by (post | "(" e "," e {"," e} ")")` (L §2.3 `tieby`). The term counts one
    /// depth level, as an `if` does, so a chain of reads each with its own `ties by` is
    /// bounded by `LOAD-DEPTH` (L §2.3, tally 2.1.2).
    pub(super) fn ties(&mut self) -> Result<Vec<Expr>, Stop> {
        self.expect("ties")?;
        self.expect("by")?;
        self.open()?;
        let items = self.tie_term()?;
        self.close()?;
        Ok(items)
    }

    /// The term after `ties by`: one postfix expression, or a bracketed list of two or more.
    fn tie_term(&mut self) -> Result<Vec<Expr>, Stop> {
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
}
