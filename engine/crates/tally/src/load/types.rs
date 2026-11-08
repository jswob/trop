//! Load's checker, its types: the static type of each expression as refcore's `tc` gives it, and the refusals its rules make on the way.
//! Spec: L §3.1–§3.3 (types, units, rates), §2.1 (four-digit figures), §2.5 (names, keys by
//! position), §4.2 (date and key arithmetic, keys as values), §5.7 (membership), §5.11
//! (aggregates), §7.1 (the literal rule), §8.3 (`LOAD-UNIT`, `-LITERAL`, `-KEY`, `-FIELD`,
//! `-AMBIG`, `-PARSE`, `-NAME`, `-PRECISION`), §8.4 (what isn't defined is refused, at load
//! where the types show it).
//! Twin: engine/tools/refcore/check.py (`tc`, `lit_rule`, `t_num` to `t_list`, `keylit`,
//! `resolve_key`, `key_value_kind`, `unordered_key`, `no_order`, `t_name`, `numcls`, `t_neg`,
//! `t_bin`, `arith`); check3.py (`m_unit_type`). The keyed reads (`read`, `key_pos`,
//! `key_conv`, `t_index`, `t_field`) are `keyed.rs`'s, the texts (`_txt`) `texts.rs`'s, the
//! operators' checks (`_arith` to `t_agg`) `ops.rs`'s and `compare.rs`'s, the calls `calls.rs`'s.
//! Invariants: each node is typed in refcore's order and its reads are recorded as it is
//! typed; a type load doesn't see yet (`Unseen`: a bound name's field, which a kind test may
//! narrow or `events` reads by name) is refused by no rule, `lifespan` included, as refcore
//! passes the type it knows there; `Dyn` is refcore's `DYN`, refused where refcore refuses it;
//! every recursion is counted (R-16).

use super::check::{AxisRead, Checker, Figure, Scope, Stands, Var, VarDom, deeper};
use super::keyed::{domain_of, resolve_lit};
use super::keys::{KeyLit, resolve_key};
use super::texts::{lit_txt, text};
use super::worlds::read_elsewhere;
use crate::base::{Code, Load, Refusal};
use crate::model::{
    CodeKind, Domain, IndexForm, KeyVal, NameKind, PeriodWord, Reality, Rel, Shown, Ty, Unit,
    UnitError,
};
use crate::num::{Num, NumError};
use crate::syntax::{
    AddOp, Expr, Field, FieldValue, Lit, MulOp, Number, Pin, PinForm, Postfix, Stop,
};
use crate::time::PeriodKind;

/// A typed expression's result.
pub(super) type Typed = Result<Ty, Stop>;

/// A load refusal with no place yet; the walk puts it at its line.
pub(super) fn refused(code: Load, message: String) -> Stop {
    Stop::Refused(Refusal::new(Code::Load(code), message))
}

/// `LOAD-UNIT` with its message (refcore's `uerr`).
pub(super) fn uerr(message: String) -> Stop {
    refused(Load::Unit, message)
}

/// A unit operation's refusal while typing (`UnitError::refusal`), placed where it was met;
/// a number error is `unit_stop`'s. L §7.6 item 4, §8.3 `LOAD-LIMIT`.
pub(super) fn unit_error(error: UnitError) -> Stop {
    error.refusal(unit_stop, None)
}

/// A number error met while typing a unit: a literal scaled by its unit past the digit cap is
/// "not built" (refcore's scales are unbounded); the number kernel's own invariant stays
/// `ENGINE-INVARIANT`. L §3.2, §7.1.
pub(super) fn unit_stop(error: NumError) -> Stop {
    match error {
        NumError::Invariant => Stop::Refused(Refusal::new(
            Code::EngineInvariant,
            "the unit algebra's invariant".to_owned(),
        )),
        NumError::Div | NumError::Digits | NumError::Domain => {
            Stop::NotBuilt("a unit past the 64-bit powers or the digit cap")
        }
    }
}

/// One of the four arithmetic operators, as a chain writes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Op {
    /// `+`.
    Add,
    /// `-`.
    Sub,
    /// `*`.
    Mul,
    /// `/`.
    Div,
}

impl Op {
    /// The operator as written.
    fn sign(self) -> &'static str {
        match self {
            Op::Add => "+",
            Op::Sub => "-",
            Op::Mul => "*",
            Op::Div => "/",
        }
    }

    /// Whether it is `+` or `-`.
    fn additive(self) -> bool {
        matches!(self, Op::Add | Op::Sub)
    }
}

/// One side of an operator: its type, and what refcore reads from the node itself (a
/// four-digit literal's entry, a bare variable, a bare literal, the side's text).
#[derive(Debug, Clone)]
pub(super) struct Operand {
    /// Its type.
    pub(super) ty: Ty,
    /// The four-digit literal it is, by its entry in the checker's list.
    pub(super) year4: Option<usize>,
    /// The variable it is, when it is a bare one.
    pub(super) var: Option<String>,
    /// The literal it is, when it is a bare one.
    pub(super) lit: Option<Lit>,
    /// Its text, as a message names it.
    pub(super) text: String,
}

/// The two sides of a binary operator, and the node's text.
#[derive(Debug, Clone)]
pub(super) struct Sides {
    /// The left side.
    pub(super) left: Operand,
    /// The right side.
    pub(super) right: Operand,
    /// The whole node's text, `a + b`.
    pub(super) text: String,
}

/// Whether a figure is one of the inline figures 0, 1 and -1 (L §7.1).
pub(super) fn is_inline(v: &Num) -> bool {
    [0, 1, -1].iter().any(|i| *v == Num::int(*i))
}

/// The class of a number type (refcore's `numcls`): `int` for a key of an integer index.
pub(super) fn numcls(m: &Reality, t: &Ty) -> Option<&'static str> {
    match t.word() {
        word @ ("int" | "num" | "money" | "rate" | "flex" | "year4" | "dyn") => Some(word),
        "key" if key_value_kind(m, t) == Some("int") => Some("int"),
        _ => None,
    }
}

/// What a key of type `t` is as a value: `int`, `date` or `period` (refcore's
/// `key_value_kind`). L §4.2 Keys as values.
pub(super) fn key_value_kind(m: &Reality, t: &Ty) -> Option<&'static str> {
    let Ty::Key(ix) = t else {
        return None;
    };
    let index = m.indexes.get(ix)?;
    if index.is_int {
        return Some("int");
    }
    let dated = index
        .keys()
        .first()
        .is_none_or(|key| matches!(key, KeyVal::Day(_)));
    match index.period {
        Some(PeriodKind::Day) if dated => Some("date"),
        Some(_) => Some("period"),
        None => None,
    }
}

/// The period kind of a key's index.
pub(super) fn key_pk(m: &Reality, t: &Ty) -> Option<PeriodKind> {
    let Ty::Key(ix) = t else {
        return None;
    };
    m.indexes.get(ix).and_then(|index| index.period)
}

/// A period word's kind.
pub(super) fn word_pk(word: PeriodWord) -> PeriodKind {
    match word {
        PeriodWord::Month => PeriodKind::Month,
        PeriodWord::Quarter => PeriodKind::Quarter,
        PeriodWord::Year => PeriodKind::Year,
        PeriodWord::Day => PeriodKind::Day,
    }
}

/// A period kind's name.
pub(super) fn pk_name(pk: PeriodKind) -> &'static str {
    match pk {
        PeriodKind::Day => "day",
        PeriodKind::Month => "month",
        PeriodKind::Quarter => "quarter",
        PeriodKind::Year => "year",
    }
}

/// Whether `t` is a date: a date, `never`, or a key of a day index (refcore's `is_date`).
pub(super) fn is_date(m: &Reality, t: &Ty) -> bool {
    matches!(t, Ty::Date | Ty::Never | Ty::Unseen) || key_value_kind(m, t) == Some("date")
}

/// Whether `t` is a period, or a key of a period index (refcore's `is_period`).
pub(super) fn is_period(m: &Reality, t: &Ty) -> bool {
    matches!(t, Ty::Period(_) | Ty::Unseen) || key_value_kind(m, t) == Some("period")
}

/// Whether `t` is a bool, or a hole (refcore's `is_bool`).
pub(super) fn is_bool(t: &Ty) -> bool {
    matches!(t, Ty::Bool | Ty::Hole | Ty::Unseen)
}

/// The period kind of a period or of a key of a period index (refcore's `period_kind`).
pub(super) fn period_kind(m: &Reality, t: &Ty) -> Option<PeriodKind> {
    if let Ty::Period(word) = t {
        return Some(word_pk(*word));
    }
    key_pk(m, t)
}

/// Whether `t` is a duration: a num of one period word to the first power.
pub(super) fn is_duration(t: &Ty) -> bool {
    let Ty::Num(unit, _) = t else {
        return false;
    };
    matches!(unit.terms(), [(base, 1)] if crate::model::PERIODS.contains(&base.as_str()))
}

/// The unit of a num, money or dist type; 1 otherwise (refcore's `t.u`).
pub(super) fn unit_of(t: &Ty) -> Result<Unit, Stop> {
    match t {
        Ty::Num(unit, _) | Ty::Dist(unit) => Ok(unit.clone()),
        Ty::Money(cur) => Unit::make(vec![(cur.clone(), 1)]).map_err(unit_stop),
        Ty::Int
        | Ty::Bool
        | Ty::Text
        | Ty::Date
        | Ty::Dates
        | Ty::Period(_)
        | Ty::Key(_)
        | Ty::Rate
        | Ty::Calendar
        | Ty::Code(_)
        | Ty::MonthDay
        | Ty::Record(_)
        | Ty::List(_)
        | Ty::Tuple(_)
        | Ty::Flex(_)
        | Ty::Year4
        | Ty::KeyLit(_)
        | Ty::KeyList
        | Ty::Dyn
        | Ty::UnitKey
        | Ty::Unseen
        | Ty::None
        | Ty::Hole
        | Ty::Never
        | Ty::DistKeys(_)
        | Ty::KindName => Ok(Unit::one()),
    }
}

/// Whether two units convert into each other (refcore's `conv(a, b) is not None`).
pub(super) fn converts(a: &Unit, b: &Unit) -> Result<bool, Stop> {
    Ok(a.conv(b).map_err(unit_stop)?.is_some())
}

/// `num u`.
pub(super) fn num(unit: Unit) -> Ty {
    Ty::Num(unit, Shown::Plain)
}

/// `num %`, `num` printed with `%` (refcore's `pct`).
fn pct() -> Ty {
    Ty::Num(Unit::one(), Shown::Percent)
}

/// Whether a bare figure is an integer (refcore's `_intflex`).
pub(super) fn int_flex(t: &Ty) -> bool {
    matches!(t, Ty::Flex(n) if n.is_integer())
}

/// What a key type is when it has no order a rule can read: things, or an open index's
/// keys (refcore's `unordered_key`). L §5.7, §5.6.
fn unordered_key(m: &Reality, t: &Ty) -> Option<String> {
    let Ty::Key(ix) = t else {
        return None;
    };
    if m.kinds.get(ix).is_some() {
        return Some(format!("things of kind {ix}"));
    }
    let open = m.indexes.get(ix).is_some_and(|i| i.form == IndexForm::Open);
    open.then(|| format!("keys of the open index {ix}"))
}

/// `LOAD-UNIT` for an order on month-days, things or an open index's keys (refcore's
/// `no_order`). L §3.1, §5.6, §5.7, §8.4.
pub(super) fn no_order(m: &Reality, t: &Ty, what: &str) -> Result<(), Stop> {
    if matches!(t, Ty::MonthDay) {
        return Err(uerr(format!(
            "{what} orders month-days: they compare with == and != only"
        )));
    }
    if let Some(keys) = unordered_key(m, t) {
        return Err(uerr(format!(
            "{what} on {keys} has no meaning: they have no order a rule can read"
        )));
    }
    Ok(())
}

/// The literal rule: a figure or a date in a rule or an ask, but the inline figures and
/// the places that allow one (refcore's `lit_rule`). L §7.1.
pub(super) fn lit_rule(
    scope: &Scope,
    what: Figure,
    text: &str,
    value: Option<&Num>,
) -> Result<(), Stop> {
    let inline = value.is_some_and(is_inline);
    match scope.stands {
        Stands::Rule | Stands::At => {
            if inline || scope.allow_lit {
                return Ok(());
            }
            match what {
                Figure::Date => Err(refused(
                    Load::Literal,
                    "a date in a rule; make it a version".to_owned(),
                )),
                Figure::Num => Err(refused(
                    Load::Literal,
                    format!("figure {text} in a rule; make it a row"),
                )),
            }
        }
        Stands::Ask if what == Figure::Num && !scope.allow_lit && !inline => Err(refused(
            Load::Literal,
            format!("figure {text} in an ask; make it a row"),
        )),
        Stands::Ask | Stands::Band | Stands::Decl => Ok(()),
    }
}

/// The input a node reads when it is a read of an input, `x` or `x[k]` (refcore's `_r` of
/// `('read', 'input', x)`).
pub(super) fn input_read(m: &Reality, scope: &Scope, e: &Expr) -> Option<String> {
    let mut id = None;
    if let Expr::Name(name) = e {
        id = Some(name);
    }
    if let Expr::Post(base, posts) = e
        && let Expr::Name(name) = base.as_ref()
        && let [Postfix::Keys(_)] = posts.as_slice()
    {
        id = Some(name);
    }
    let id = id?;
    let input = !scope.is_var(id) && m.names.kind(id) == Some(NameKind::Input);
    input.then(|| id.clone())
}

/// A word that is no variable and no id, or a key, a day, a month, a month-day, a currency,
/// or a whole number with no unit: a key literal (refcore's `is_keylit_node`). L §2.5.
pub(super) fn is_keylit_node(m: &Reality, scope: &Scope, e: &Expr) -> bool {
    if let Expr::Name(name) = e {
        return !scope.is_var(name) && !m.names.contains_key(name);
    }
    let Expr::Lit(lit) = e else {
        return false;
    };
    match lit {
        Lit::Key(_) | Lit::Day(_) | Lit::Month(..) | Lit::MonthDay(..) | Lit::Cur(_) => true,
        Lit::Num(n) => n.whole && n.unit.is_none(),
        Lit::Quarter(..) | Lit::Bool(_) | Lit::Inf { .. } | Lit::Nominal(_) | Lit::Name(_) => false,
    }
}

/// A key written as an expression, for its resolution against a domain.
pub(super) fn key_lit_of(e: &Expr) -> Option<KeyLit> {
    KeyLit::of_expr(e)
}

impl Checker {
    /// An expression's type, `expect` the type its position expects, each refusal put at
    /// the scope's line (refcore's `tc`). L §3.
    pub(super) fn expr(
        &mut self,
        m: &Reality,
        scope: &Scope,
        e: &Expr,
        expect: Option<&Ty>,
        depth: u32,
    ) -> Typed {
        let depth = deeper(depth)?;
        self.typed(m, scope, e, expect, depth)
            .map_err(|stop| scope.place(stop))
    }

    /// An operand: the expression's type with what an operator reads from its node.
    pub(super) fn operand(
        &mut self,
        m: &Reality,
        scope: &Scope,
        e: &Expr,
        expect: Option<&Ty>,
        depth: u32,
    ) -> Result<Operand, Stop> {
        let ty = self.expr(m, scope, e, expect, depth)?;
        // Only the figure itself takes a reading: a bracket or a branch holding one passes
        // its type on, not its literal rule (refcore keeps the reading on the literal's node).
        let literal = matches!(e, Expr::Lit(Lit::Num(n)) if n.four);
        let year4 = if matches!(ty, Ty::Year4) && literal {
            self.year4.len().checked_sub(1)
        } else {
            None
        };
        let mut var = None;
        if let Expr::Name(name) = e
            && scope.is_var(name)
        {
            var = Some(name.clone());
        }
        let mut lit = None;
        if let Expr::Lit(written) = e {
            lit = Some(written.clone());
        }
        Ok(Operand {
            ty,
            year4,
            var,
            lit,
            text: text(e),
        })
    }

    /// One arm per node kind (refcore's `_tc`).
    fn typed(
        &mut self,
        m: &Reality,
        scope: &Scope,
        e: &Expr,
        expect: Option<&Ty>,
        depth: u32,
    ) -> Typed {
        if self.aff_x.as_ref().is_some_and(|x| x == e) {
            self.aff_x = None;
            self.affx_hit = true;
        }
        match e {
            Expr::Lit(Lit::Name(name)) | Expr::Name(name) => {
                self.name(m, scope, name, expect, depth)
            }
            Expr::Lit(lit) => self.lit(m, scope, lit, expect),
            Expr::Str(_) => text_literal(scope),
            Expr::None => Ok(Ty::None),
            Expr::Never => Ok(Ty::Never),
            Expr::Cut => Ok(Ty::Date),
            Expr::Calendar(_) => Err(refused(
                Load::Parse,
                "cannot parse: calendar(...) is a row value (L §2.3 value), not an atom of an expression"
                    .to_owned(),
            )),
            Expr::List(items) => self.list(m, scope, items, depth),
            Expr::If(parts) => self.conditional(m, scope, parts, expect, depth),
            Expr::Or(terms) | Expr::And(terms) => self.junction(m, scope, terms, depth),
            Expr::Not(inner) => self.not(m, scope, inner, depth),
            Expr::Paren(inner) => self.expr(m, scope, inner, expect, depth),
            Expr::Neg(inner) => self.neg(m, scope, inner, expect, depth),
            Expr::Cmp(cmp) => self.cmp(m, scope, &cmp.0, cmp.1, &cmp.2, depth),
            Expr::In(test) => self.membership(m, scope, test, depth),
            Expr::Sum(first, rest) => {
                let ops: Vec<(Op, &Expr)> = rest
                    .iter()
                    .map(|(op, term)| match op {
                        AddOp::Add => (Op::Add, term),
                        AddOp::Sub => (Op::Sub, term),
                    })
                    .collect();
                self.chain(m, scope, first, &ops, expect, depth)
            }
            Expr::Prod(first, rest) => {
                let ops: Vec<(Op, &Expr)> = rest
                    .iter()
                    .map(|(op, factor)| match op {
                        MulOp::Mul => (Op::Mul, factor),
                        MulOp::Div => (Op::Div, factor),
                    })
                    .collect();
                self.chain(m, scope, first, &ops, expect, depth)
            }
            Expr::Post(base, posts) => self.post(m, scope, base, posts, expect, depth),
            Expr::Record(fields) => self.record(m, scope, fields, expect, depth),
            Expr::Agg(agg) => self.aggregate(m, scope, agg, depth),
            Expr::Call(call) => self.call(m, scope, call, expect, depth),
            Expr::Last(pair) => self.last(m, scope, &pair.0, &pair.1, depth),
            Expr::LegOf(inner, of) => self.leg_of(m, scope, inner, of, expect, depth),
            Expr::To(inner, step) => self.to(m, scope, inner, step, expect, depth),
            Expr::Bal(bal) => self.bal(m, scope, bal, depth),
            Expr::Flows(flows) => self.flows(m, scope, flows, depth),
            Expr::Covered(covered) => self.covered(m, scope, &covered.0, &covered.1, depth),
            Expr::Value(value) => self.value(m, scope, value, depth),
            Expr::Query(query) => self.query(m, scope, query, depth),
            Expr::Low(low) => self.low(m, scope, low, depth),
            Expr::Crps(crps) => self.crps(m, scope, crps, depth),
            Expr::DistFn(dist_fn) => self.dist_fn(m, scope, dist_fn, depth),
            Expr::Dist(dist) => self.dist(m, scope, dist, expect, depth),
            Expr::Prorata(prorata) => self.prorata(m, scope, prorata, depth),
            Expr::AsIf(inner, pins) => self.as_if(m, scope, inner, pins, expect, depth),
        }
    }

    /// `e as if {pins}`: each pin checked, then `e`, its reads inside `{law D}` made at that
    /// law date (refcore's `t_asif`). L §5.11.
    fn as_if(
        &mut self,
        m: &Reality,
        scope: &Scope,
        inner: &Expr,
        pins: &[Pin],
        expect: Option<&Ty>,
        depth: u32,
    ) -> Typed {
        let mut under = scope.clone();
        for pin in pins {
            self.pin(m, scope, pin, depth)?;
            if matches!(pin.form, PinForm::Law(_)) && under.mode.is_none() {
                under.mode = Some(Rel::LawPin);
            }
        }
        self.expr(m, &under, inner, expect, depth)
    }

    /// A literal (refcore's `t_num`, `t_date`, `t_month`, `t_quarter`, `t_mmdd`, `t_key`,
    /// `t_cur`, `t_bool`, `t_inf`, `t_code`). L §2.1, §7.1.
    fn lit(&mut self, m: &Reality, scope: &Scope, lit: &Lit, expect: Option<&Ty>) -> Typed {
        match lit {
            Lit::Num(n) => self.number(m, scope, n, expect),
            Lit::Day(_) => {
                lit_rule(scope, Figure::Date, "", None)?;
                Ok(Ty::Date)
            }
            Lit::Month(..) => {
                lit_rule(scope, Figure::Date, "", None)?;
                Ok(Ty::Period(PeriodWord::Month))
            }
            Lit::Quarter(..) => {
                lit_rule(scope, Figure::Date, "", None)?;
                Ok(Ty::Period(PeriodWord::Quarter))
            }
            Lit::MonthDay(month, day) => {
                if !month_day_ok(*month, *day) {
                    return Err(refused(
                        Load::Parse,
                        format!(
                            "cannot parse: {} is not a month-day every year has",
                            lit.text()
                        ),
                    ));
                }
                lit_rule(scope, Figure::Date, "", None)?;
                Ok(Ty::MonthDay)
            }
            Lit::Key(_) | Lit::Name(_) => {
                Self::keylit(m, scope, &Expr::Lit(lit.clone()), &lit.text(), expect)
            }
            Lit::Cur(cur) => {
                if !m.currencies.contains_key(cur) {
                    return Err(uerr(format!("money in an undeclared currency {cur}")));
                }
                Ok(Ty::Key("currencies".to_owned()))
            }
            Lit::Bool(_) => Ok(Ty::Bool),
            Lit::Inf { .. } => Err(refused(
                Load::Parse,
                "cannot parse: inf stands only in an interval".to_owned(),
            )),
            Lit::Nominal(_) => Ok(Ty::Code(CodeKind::Compounding)),
        }
    }

    /// A number literal: its unit, the literal rule, precision, and the context's reading of
    /// a bare figure (refcore's `t_num`). L §2.1, §3.2, §7.1.
    fn number(&mut self, m: &Reality, scope: &Scope, n: &Number, expect: Option<&Ty>) -> Typed {
        if let Some(unit) = &n.unit {
            return number_with_unit(m, scope, n, unit, expect);
        }
        if n.fraction {
            return Err(refused(
                Load::Parse,
                "cannot parse: a fraction is allowed only in rows".to_owned(),
            ));
        }
        if let Some(Ty::Key(ix)) = expect
            && n.whole
            && m.indexes.get(ix).is_some_and(|index| index.is_int)
        {
            let (file, line) = scope.file_line();
            resolve_key(
                m,
                &KeyLit::Lit(Lit::Num(n.clone())),
                &Domain::Index(ix.clone()),
                &file,
                line,
            )?;
            return Ok(Ty::Key(ix.clone()));
        }
        let year_expected = matches!(expect, Some(Ty::Period(PeriodWord::Year)));
        if scope.stands == Stands::Band
            && !(n.whole && n.four && matches!(expect, Some(Ty::Period(_))))
        {
            return Ok(Ty::Flex(n.value.clone()));
        }
        if n.whole && is_inline(&n.value) {
            return Ok(Ty::Flex(n.value.clone()));
        }
        if n.whole && n.four {
            if year_expected {
                lit_rule(scope, Figure::Date, &n.text, None)?;
                return Ok(Ty::Period(PeriodWord::Year));
            }
            if matches!(expect, Some(Ty::Int | Ty::Num(..))) {
                lit_rule(scope, Figure::Num, &n.text, Some(&n.value))?;
                return Ok(Ty::Int);
            }
            self.push_year4(scope, n);
            return Ok(Ty::Year4);
        }
        lit_rule(scope, Figure::Num, &n.text, Some(&n.value))?;
        if n.whole {
            return Ok(Ty::Int);
        }
        Ok(num(Unit::one()))
    }

    /// A key literal, resolved by the type its position expects (refcore's `keylit`).
    /// L §2.5.
    pub(super) fn keylit(
        m: &Reality,
        scope: &Scope,
        node: &Expr,
        written: &str,
        expect: Option<&Ty>,
    ) -> Typed {
        match expect {
            Some(Ty::Key(ix)) => {
                Self::resolve_at(m, scope, node, ix)?;
                Ok(Ty::Key(ix.clone()))
            }
            Some(Ty::Code(kind)) => {
                if CodeKind::of_word(written) == Some(*kind) && written != "nominal" {
                    return Ok(Ty::Code(*kind));
                }
                Err(uerr(format!("{written} is not a {} code", kind.name())))
            }
            Some(_) | None => Ok(Ty::KeyLit(written.to_owned())),
        }
    }

    /// A key literal at a key of `ix` (refcore's `resolve_key`). L §2.5.
    fn resolve_at(m: &Reality, scope: &Scope, node: &Expr, ix: &str) -> Result<(), Stop> {
        let Some(domain) = domain_of(m, ix) else {
            return Err(refused(Load::Name, format!("unknown id {ix}")).naming(ix));
        };
        if let Expr::Name(name) = node
            && scope.is_var(name)
        {
            return Err(refused(
                Load::Key,
                format!("{name} is a bound variable, not a key"),
            ));
        }
        if matches!(
            node,
            Expr::Lit(Lit::Day(_) | Lit::Month(..) | Lit::Quarter(..))
        ) {
            lit_rule(scope, Figure::Date, "", None)?;
        }
        resolve_lit(m, scope, node, &domain).map(|_| ())
    }

    /// A name (refcore's `t_name`): a variable, an input or def read, a place, a movement
    /// instance, the cut, or a key literal; any other id is no value here. L §2.5.
    pub(super) fn name(
        &mut self,
        m: &Reality,
        scope: &Scope,
        name: &str,
        expect: Option<&Ty>,
        depth: u32,
    ) -> Typed {
        if let Some(var) = scope.vars.get(name) {
            return Ok(var_type(var));
        }
        let Some(kind) = m.names.kind(name) else {
            if !matches!(expect, Some(Ty::Key(_) | Ty::Code(_))) {
                read_elsewhere(m, name)?;
            }
            return Self::keylit(m, scope, &Expr::Name(name.to_owned()), name, expect);
        };
        match kind {
            NameKind::Input | NameKind::Def => self.read(m, scope, name, kind, &[], depth),
            NameKind::Place => self.place_value(m, scope, name, &[], depth),
            NameKind::Move => self.move_value(m, scope, name, &[], depth),
            NameKind::Axis => {
                self.axis_read(&scope.item, name, AxisRead::ById);
                Ok(Ty::Key(name.to_owned()))
            }
            NameKind::Cut => Ok(Ty::Date),
            NameKind::Index | NameKind::Kind => Err(refused(
                Load::Parse,
                format!(
                    "cannot parse: {} {name} stands only in a binding or a membership test",
                    name_word(kind)
                ),
            )),
            NameKind::Band => Err(refused(
                Load::Parse,
                format!("cannot parse: band {name} is called with ( )"),
            )),
            NameKind::Fill => Err(refused(
                Load::Parse,
                format!("cannot parse: fill {name} is read as {name}.taken[...]"),
            )),
            NameKind::Version | NameKind::Source | NameKind::Unit | NameKind::Ask => Err(refused(
                Load::Parse,
                format!("cannot parse: {} {name} is not a value", name_word(kind)),
            )),
        }
    }

    /// A list of literals: `dates` when every item is a date (refcore's `t_list`).
    fn list(&mut self, m: &Reality, scope: &Scope, items: &[Lit], depth: u32) -> Typed {
        for item in items {
            self.expr(m, scope, &lit_expr(item), None, depth)?;
        }
        if !items.is_empty() && items.iter().all(|item| matches!(item, Lit::Day(_))) {
            return Ok(Ty::Dates);
        }
        Ok(Ty::List(Box::new(Ty::Unseen)))
    }

    /// A record's fields, each typed by the field its expected type gives (refcore's
    /// `t_record`). L §3.1.
    fn record(
        &mut self,
        m: &Reality,
        scope: &Scope,
        fields: &[Field],
        expect: Option<&Ty>,
        depth: u32,
    ) -> Typed {
        let mut out = Vec::new();
        for field in fields {
            match field {
                Field::Leg(leg) => {
                    return Err(refused(
                        Load::Field,
                        format!(
                            "leg {}: a leg is a field only of an observed row, not of a record",
                            leg.name
                        ),
                    ));
                }
                Field::Named(name, value) => {
                    let wanted = expect.and_then(|t| t.field(name));
                    let t = match value {
                        FieldValue::Hole(_) => wanted.cloned().unwrap_or(Ty::Hole),
                        FieldValue::None => Ty::None,
                        FieldValue::Lit(lit) => {
                            self.expr(m, scope, &lit_expr(lit), wanted, depth)?
                        }
                        FieldValue::Expr(e) => self.expr(m, scope, e, wanted, depth)?,
                    };
                    out.push((name.clone(), t));
                }
            }
        }
        Ok(Ty::Record(out))
    }

    /// `-x` (refcore's `t_neg`): a number's own class, `-1` an inline figure.
    fn neg(
        &mut self,
        m: &Reality,
        scope: &Scope,
        inner: &Expr,
        expect: Option<&Ty>,
        depth: u32,
    ) -> Typed {
        if let Expr::Lit(Lit::Num(n)) = inner
            && n.unit.is_none()
            && n.whole
            && n.value == Num::int(1)
        {
            return Ok(Ty::Flex(Num::int(-1)));
        }
        let t = self.expr(m, scope, inner, expect, depth)?;
        if matches!(t, Ty::Unseen) {
            return Ok(t);
        }
        match numcls(m, &t) {
            None => Err(uerr(format!("unary minus on {}", t.show()))),
            Some("flex") => {
                if let Ty::Flex(n) = &t {
                    return Ok(Ty::Flex(n.checked_neg().map_err(unit_stop)?));
                }
                Ok(t)
            }
            Some("year4") => Ok(Ty::Int),
            Some(_) if matches!(t, Ty::Key(_)) => Ok(Ty::Int),
            Some(_) => Ok(t),
        }
    }

    /// A chain of `+ -` or `* /`, nested to the left (refcore's `t_bin`). L §3.2, §4.2,
    /// §7.1.
    fn chain(
        &mut self,
        m: &Reality,
        scope: &Scope,
        first: &Expr,
        rest: &[(Op, &Expr)],
        expect: Option<&Ty>,
        depth: u32,
    ) -> Typed {
        let additive = rest.first().is_some_and(|(op, _)| op.additive());
        let mut left =
            self.operand(m, scope, first, if additive { expect } else { None }, depth)?;
        for (at, (op, right_expr)) in rest.iter().enumerate() {
            let op = *op;
            let first_var = left.var.as_ref().and_then(|v| scope.vars.get(v));
            let ranged = first_var
                .is_some_and(|v| matches!(v.dom, VarDom::Index(_) | VarDom::Range | VarDom::List));
            let offset = op.additive()
                && (matches!(left.ty, Ty::Key(_)) || ranged)
                && matches!(right_expr, Expr::Lit(Lit::Num(n)) if n.whole && n.unit.is_none());
            let mut sub = scope.clone();
            if offset {
                sub.allow_lit = true;
            }
            let right_expect = if op.additive() && !offset {
                expect
            } else {
                None
            };
            let mut right = self.operand(m, &sub, right_expr, right_expect, depth)?;
            if matches!(left.ty, Ty::KeyLit(_)) && at == 0 {
                left = self.operand(m, scope, first, Some(&right.ty), depth)?;
            }
            if matches!(right.ty, Ty::KeyLit(_)) {
                right = self.operand(m, &sub, right_expr, Some(&left.ty), depth)?;
            }
            let lt = left.ty.clone();
            let rt = right.ty.clone();
            let text = format!("{} {} {}", left.text, op.sign(), right.text);
            let sides = Sides {
                left,
                right,
                text: text.clone(),
            };
            let ty = self.arith(m, scope, op, (&lt, &rt), &sides)?;
            left = Operand {
                ty,
                year4: None,
                var: None,
                lit: None,
                text,
            };
        }
        Ok(left.ty)
    }

    /// An operator's type, `day` mixed with a period refused (refcore's `arith`). L §3.2.
    pub(super) fn arith(
        &mut self,
        m: &Reality,
        scope: &Scope,
        op: Op,
        (lt, rt): (&Ty, &Ty),
        sides: &Sides,
    ) -> Typed {
        let t = self.arith_core(m, scope, op, (lt, rt), sides)?;
        if let Ty::Num(unit, _) = &t
            && unit.day_and_period()
        {
            return Err(uerr(format!(
                "{} {} {}: day is a dimension of its own (1 day / 1 year is refused, L §3.2)",
                lt.show(),
                op.sign(),
                rt.show()
            )));
        }
        Ok(t)
    }
}

/// The granularity of a date, month or quarter literal.
pub(super) fn granularity(e: &Expr) -> Option<PeriodKind> {
    let Expr::Lit(lit) = e else {
        return None;
    };
    match lit {
        Lit::Day(_) => Some(PeriodKind::Day),
        Lit::Month(..) => Some(PeriodKind::Month),
        Lit::Quarter(..) => Some(PeriodKind::Quarter),
        Lit::Num(_)
        | Lit::MonthDay(..)
        | Lit::Key(_)
        | Lit::Cur(_)
        | Lit::Bool(_)
        | Lit::Inf { .. }
        | Lit::Nominal(_)
        | Lit::Name(_) => None,
    }
}

/// Whether a month and a day are a month-day every year has (refcore's `monthday_ok`).
fn month_day_ok(month: u8, day: u8) -> bool {
    let last = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => 28,
        _ => 0,
    };
    day >= 1 && day <= last
}

/// `LOAD-LITERAL` for a text literal in a rule (refcore's `t_str`). L §7.1.
fn text_literal(scope: &Scope) -> Typed {
    if matches!(scope.stands, Stands::Rule | Stands::At) {
        return Err(refused(
            Load::Literal,
            "a text literal in a rule; make it a row".to_owned(),
        ));
    }
    Ok(Ty::Text)
}

/// A number with a unit: the unit's names, the literal rule, a currency's precision, `%`
/// and a period unit where a rate is expected (refcore's `t_num`, its unit arm). L §3.2,
/// §3.3, §7.1.
fn number_with_unit(
    m: &Reality,
    scope: &Scope,
    n: &Number,
    unit: &crate::syntax::Unit,
    expect: Option<&Ty>,
) -> Typed {
    let mut terms = Vec::new();
    for term in &unit.terms {
        let base = term.base.as_str();
        let known = base == "%"
            || m.currencies.contains_key(base)
            || m.units.contains(base)
            || crate::model::PERIODS.contains(&base)
            || m.unit_kind_keys.contains_key(base);
        if known {
            terms.push((term.base.clone(), term.power.clone()));
        } else if crate::model::is_currency_code(base) {
            return Err(uerr(format!("money in an undeclared currency {base}")));
        } else {
            return Err(refused(Load::Name, format!("unknown id {base} (a unit)")).naming(base));
        }
    }
    let (made, factor) = Unit::from_syntax(terms).map_err(unit_error)?;
    let scaled = n.value.checked_mul(&factor).map_err(unit_stop)?;
    let shown = lit_txt(&Lit::Num(n.clone()));
    let ruled = if is_inline(&scaled) {
        &scaled
    } else {
        &n.value
    };
    lit_rule(scope, Figure::Num, &shown, Some(ruled))?;
    if let [(cur, 1)] = made.terms()
        && made.is_currency()
    {
        let Some(minor) = m.currencies.get(cur) else {
            return Err(uerr(format!("money in an undeclared currency {cur}")));
        };
        if !scaled.checked_div(minor).map_err(unit_stop)?.is_integer() {
            return Err(refused(
                Load::Precision,
                format!("{} is finer than the minor unit", n.text),
            ));
        }
        return Ok(Ty::Money(cur.clone()));
    }
    if matches!(unit.terms.as_slice(), [t] if t.base == "%" && t.power == Num::int(1)) {
        return Ok(pct());
    }
    if matches!(expect, Some(Ty::Rate)) && made.period_of().is_some() {
        return Ok(Ty::Rate);
    }
    Ok(num(made))
}

/// A literal as an expression: a word is a name.
pub(super) fn lit_expr(lit: &Lit) -> Expr {
    match lit {
        Lit::Name(name) => Expr::Name(name.clone()),
        Lit::Num(_)
        | Lit::Day(_)
        | Lit::Month(..)
        | Lit::Quarter(..)
        | Lit::MonthDay(..)
        | Lit::Key(_)
        | Lit::Cur(_)
        | Lit::Bool(_)
        | Lit::Inf { .. }
        | Lit::Nominal(_) => Expr::Lit(lit.clone()),
    }
}

/// The type a variable gives where it is read: a key list of days is a date, of months a
/// month, of quarters a quarter (refcore's `t_name`, its variable arm). L §2.5, §4.2.
fn var_type(var: &Var) -> Ty {
    if matches!(var.ty, Ty::KeyList) && !var.keylist.is_empty() {
        if var.keylist.iter().all(|k| matches!(k, Lit::Day(_))) {
            return Ty::Date;
        }
        if var.keylist.iter().all(|k| matches!(k, Lit::Month(..))) {
            return Ty::Period(PeriodWord::Month);
        }
        if var.keylist.iter().all(|k| matches!(k, Lit::Quarter(..))) {
            return Ty::Period(PeriodWord::Quarter);
        }
    }
    var.ty.clone()
}

/// A name kind's word, as a message names it.
pub(super) fn name_word(kind: NameKind) -> &'static str {
    match kind {
        NameKind::Unit => "unit",
        NameKind::Kind => "kind",
        NameKind::Index => "index",
        NameKind::Axis => "axis",
        NameKind::Band => "band",
        NameKind::Place => "place",
        NameKind::Fill => "fill",
        NameKind::Move => "move",
        NameKind::Version => "version",
        NameKind::Ask => "ask",
        NameKind::Source => "source",
        NameKind::Cut => "cut",
        NameKind::Input => "input",
        NameKind::Def => "def",
    }
}

/// The static type of a quantity of unit `u`: money for a currency, `num u` for a unit or a
/// unit kind, `Dyn` for `any` (refcore's `m_unit_type`). L §5.14.
pub(super) fn unit_type(m: &Reality, u: Option<&str>) -> Result<Ty, Stop> {
    let Some(u) = u.filter(|u| *u != "any" && *u != "one") else {
        return Ok(Ty::Dyn);
    };
    if m.currencies.contains_key(u) {
        return Ok(Ty::Money(u.to_owned()));
    }
    let named = m.units.contains(u)
        || m.unit_kind_keys.contains_key(u)
        || (m.names.kind(u) == Some(NameKind::Kind));
    if named {
        return Ok(num(Unit::make(vec![(u.to_owned(), 1)]).map_err(unit_stop)?));
    }
    Ok(Ty::Dyn)
}
