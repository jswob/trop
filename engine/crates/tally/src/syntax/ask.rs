//! The ask line, its forms and clauses, the pins of an ask and of an `as if`, and `solve`.
//! Spec: L §2.3 (`ask`, `askbody`, `solvex`, `pin`, `pref`, parse rules 2 and 4), §5.18 (asks),
//! §5.11 (`as if`), §6.4 (`diff`, a world).
//! Twin: engine/tools/refcore/parser.py (`sec_ask`, `pin`, `pin_`, `qid_text`, `pref`,
//! `solvex`, `post`'s `as if` arm).
//! Invariants: an ask is one line; its clauses are taken in the grammar's order, each at most
//! once; the error texts are refcore's word for word.

use super::expr::Ends;
use super::lex::{Kind, Line};
use super::parse::{P, Stop};
use super::tree::{Ask, AskBody, Diff, Expr, Orient, PRef, Pin, PinForm, Residuals, Solve};

/// An ask's body expression; `in ID` after it names the world unless a comma or `on`
/// follows (parse rule 4).
fn ask_expr(p: &mut P<'_>) -> Result<Expr, Stop> {
    let save = p.i;
    let expr = p.expr(Ends::AtIn)?;
    if p.at("in", 0) && p.at_id(1) && (p.at(",", 2) || p.at("on", 2)) {
        p.i = save;
        return p.expr(Ends::Free);
    }
    Ok(expr)
}

/// What an ask asks, after `ask id:` (L §2.3 `askbody`).
fn ask_body(p: &mut P<'_>) -> Result<AskBody, Stop> {
    if p.accept("explain")? {
        let target = if p.at("bal", 0) || p.at("low", 0) {
            let word = p.peek(0).map_or("", |tok| tok.text.as_str());
            p.builtin(word)?
        } else {
            p.reference()?
        };
        return Ok(AskBody::Explain(target));
    }
    if p.accept("inputs-of")? {
        return Ok(AskBody::InputsOf(p.reference()?));
    }
    if p.accept("readers-of")? {
        return Ok(AskBody::ReadersOf(p.reference()?));
    }
    if p.accept("check")? {
        let left = p.sum(Ends::AtIn)?;
        p.expect("==")?;
        let right = p.sum(Ends::AtIn)?;
        p.no_chain()?;
        return Ok(AskBody::Check(left, right));
    }
    if p.accept("residuals")? {
        let place = p.reference()?;
        let span = p.span()?;
        let by = if p.accept("by")? {
            Some(p.ident("an id")?)
        } else {
            None
        };
        return Ok(AskBody::Residuals(Box::new(Residuals { place, span, by })));
    }
    if p.at("solve", 0) {
        return Ok(AskBody::Solve(Box::new(p.solve()?)));
    }
    if p.at("diff", 0) && p.at("(", 1) {
        return Ok(AskBody::Diff(Box::new(diff(p)?)));
    }
    let mut exprs = vec![ask_expr(p)?];
    while p.accept(",")? {
        exprs.push(ask_expr(p)?);
    }
    Ok(AskBody::Exprs(exprs))
}

/// `diff(e [for binds]) in s against b`, at `diff`.
fn diff(p: &mut P<'_>) -> Result<Diff, Stop> {
    p.skip(2)?;
    p.open()?;
    let expr = p.expr(Ends::Free)?;
    let binds = if p.accept("for")? {
        p.binds()?
    } else {
        Vec::new()
    };
    p.expect(")")?;
    p.close()?;
    p.expect("in")?;
    let world = p.ident("an id")?;
    p.expect("against")?;
    let against = p.ident("an id")?;
    Ok(Diff {
        expr,
        binds,
        world,
        against,
    })
}

/// `ask id: body [for binds] [in world] [with pins] [law e] [as known e] [windows each]
/// [with uncertain] [random at …] [paths N seed S] [path P] [partial] [orient …]`, one line
/// (L §2.3 `ask`).
pub(super) fn ask(p: &mut P<'_>, body: &[Line]) -> Result<Ask, Stop> {
    let id = p.ident("an id")?;
    p.expect(":")?;
    let ask_body = ask_body(p)?;
    let binds = if p.accept("for")? {
        p.ask_for = true;
        let binds = p.binds();
        p.ask_for = false;
        binds?
    } else {
        Vec::new()
    };
    let world = if p.at("in", 0) && p.at_id(1) {
        p.skip(1)?;
        Some(p.ident("an id")?)
    } else {
        None
    };
    let mut pins = Vec::new();
    if p.at("with", 0) && !p.at("uncertain", 1) {
        p.skip(1)?;
        pins.push(p.pin()?);
        while p.accept(",")? {
            pins.push(p.pin()?);
        }
    }
    let mut decl = Ask {
        id,
        body: ask_body,
        binds,
        world,
        pins,
        law: None,
        known: None,
        windows_each: false,
        uncertain: false,
        random: None,
        paths: None,
        path: None,
        partial: false,
        orient: None,
        line: p.line,
    };
    ask_clauses(p, &mut decl)?;
    if let Some(line) = body.first() {
        return Err(p.cannot_at("an ask is one line", line.number));
    }
    Ok(decl)
}

/// An ask's clauses after its pins, in the grammar's order, to the line's end.
fn ask_clauses(p: &mut P<'_>, decl: &mut Ask) -> Result<(), Stop> {
    if p.accept("law")? {
        decl.law = Some(p.expr(Ends::Free)?);
    }
    if p.at("as", 0) && p.at("known", 1) {
        p.skip(2)?;
        decl.known = Some(p.expr(Ends::Free)?);
    }
    if p.accept("windows")? {
        p.each()?;
        decl.windows_each = true;
    }
    decl.uncertain = p.with_uncertain()?;
    if p.at("random", 0) && p.at("at", 1) {
        p.skip(2)?;
        decl.random = Some(p.rchoice()?);
    }
    if p.accept("paths")? {
        let paths = p.integer()?;
        p.expect("seed")?;
        decl.paths = Some((paths, p.integer()?));
    }
    if p.accept("path")? {
        decl.path = Some(p.integer()?);
    }
    decl.partial = p.accept("partial")?;
    if p.accept("orient")? {
        decl.orient = Some(if p.accept("lower")? {
            Orient::Lower
        } else if p.accept("higher")? {
            Orient::Higher
        } else {
            return Err(p.err("lower or higher"));
        });
    }
    p.end()
}

impl P<'_> {
    /// `{pin, …}` after `as if` (parse rule 2: it binds to the reference before it).
    pub(super) fn asif_pins(&mut self) -> Result<Vec<Pin>, Stop> {
        self.expect("{")?;
        self.open()?;
        let mut pins = vec![self.pin()?];
        while self.accept(",")? {
            pins.push(self.pin()?);
        }
        self.expect("}")?;
        self.close()?;
        Ok(pins)
    }

    /// A pin, with its text as written (L §2.3 `pin`).
    pub(super) fn pin(&mut self) -> Result<Pin, Stop> {
        let first = self.i;
        let form = self.pin_form()?;
        Ok(Pin {
            form,
            text: self.source_text(first),
        })
    }

    /// What a pin pins: `law e`, `known e`, `wording e for [ids]`, `scenario s`, or
    /// `[sample] pref = e` (`ID = key` an axis's option, decided at resolution).
    fn pin_form(&mut self) -> Result<PinForm, Stop> {
        if self.accept("law")? {
            return Ok(PinForm::Law(self.expr(Ends::Free)?));
        }
        if self.accept("known")? {
            return Ok(PinForm::Known(self.expr(Ends::Free)?));
        }
        if self.accept("wording")? {
            let day = self.expr(Ends::Free)?;
            self.expect("for")?;
            self.expect("[")?;
            let mut ids = vec![self.qid_text()?];
            while self.accept(",")? {
                ids.push(self.qid_text()?);
            }
            self.expect("]")?;
            return Ok(PinForm::Wording(day, ids));
        }
        if self.at("scenario", 0) && self.at_id(1) {
            self.skip(1)?;
            return Ok(PinForm::Scenario(self.ident("an id")?));
        }
        let sample = self.at("sample", 0) && self.at_id(1) && !self.at("=", 1);
        if sample {
            self.skip(1)?;
        }
        let target = self.pref()?;
        self.expect("=")?;
        let value = self.expr(Ends::Free)?;
        Ok(PinForm::Set {
            target,
            value,
            sample,
        })
    }

    /// `ID ["." ID]`, as text.
    fn qid_text(&mut self) -> Result<String, Stop> {
        let first = self.ident("an id")?;
        if self.at(".", 0) && !self.spaced(0) {
            self.skip(1)?;
            return Ok(format!("{first}.{}", self.ident("an id")?));
        }
        Ok(first)
    }

    /// `pref = ID [["." ID] "[" pk {"," pk} "]"] {"." FIELD}` (L §2.3).
    fn pref(&mut self) -> Result<PRef, Stop> {
        let mut id = self.ident("a pinned id")?;
        let mut qual = None;
        let member_key = self.at(".", 0)
            && !self.spaced(0)
            && self.kind(1) == Some(&Kind::Id)
            && self.at("[", 2);
        if member_key {
            self.skip(1)?;
            qual = Some(id);
            id = self.ident("an id")?;
        }
        let keys = if self.at("[", 0) {
            self.skip(1)?;
            self.open()?;
            let mut keys = vec![self.key_expr()?];
            while self.accept(",")? {
                keys.push(self.key_expr()?);
            }
            self.expect("]")?;
            self.close()?;
            Some(keys)
        } else {
            None
        };
        let mut fields = Vec::new();
        while self.at(".", 0) && !self.spaced(0) {
            self.skip(1)?;
            let tok = self.next()?;
            if tok.kind != Kind::Id {
                return Err(self.err("a field"));
            }
            fields.push(tok.text.clone());
        }
        Ok(PRef {
            id,
            qual,
            keys,
            fields,
        })
    }

    /// `solve f == g for x in [lo, hi] to u` (L §2.3 `solvex`).
    pub(super) fn solve(&mut self) -> Result<Solve, Stop> {
        self.expect("solve")?;
        let left = self.sum(Ends::Free)?;
        self.expect("==")?;
        let right = self.sum(Ends::Free)?;
        self.no_chain()?;
        self.expect("for")?;
        let target = self.reference()?;
        self.expect("in")?;
        self.expect("[")?;
        let lo = self.lit()?;
        self.expect(",")?;
        let hi = self.lit()?;
        self.expect("]")?;
        self.expect("to")?;
        let unit = self.step_unit()?;
        Ok(Solve {
            left,
            right,
            target,
            lo,
            hi,
            unit,
        })
    }
}
