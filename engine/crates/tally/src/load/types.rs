//! Load's checker, its types: the static type of each expression as refcore's `tc` gives it, and the refusals its rules make on the way.
//! Spec: L §3.1–§3.3 (types, units, rates), §2.1 (four-digit figures), §2.5 (names, keys by
//! position), §4.2 (date and key arithmetic, keys as values), §4.6 (a period at a key of
//! another granularity), §5.7 (membership), §5.11 (aggregates), §7.1 (the literal rule), §8.3
//! (`LOAD-UNIT`, `-LITERAL`, `-KEY`, `-FIELD`, `-AMBIG`, `-PARSE`, `-NAME`, `-PRECISION`),
//! §8.4 (what isn't defined is refused, at load where the types show it).
//! Twin: engine/tools/refcore/check.py (`tc`, `lit_rule`, `t_num` to `t_list`, `keylit`,
//! `resolve_key`, `key_value_kind`, `unordered_key`, `no_order`, `key_pos`, `key_conv`,
//! `period_conv`, `t_name`, `read`, `t_index`, `t_field`, `numcls`, `t_neg`, `t_bin`, `arith`,
//! `_arith`, `fx_check`, `ambig`, `arith_time`, `t_not`, `t_and`, `t_if`, `unify`, `t_cmp`,
//! `kl_value`, `compare`, `t_in`, `t_agg`, `_txt`); check3.py (`m_unit_type`,
//! `m_movement_field`, `m_covered_spans_of`, `span_txt`).
//! Invariants: each node is typed in refcore's order and its reads are recorded as it is
//! typed; a form this checker doesn't type yet is `Unseen`, which no rule refuses, so a text
//! refcore loads is never refused for it; every recursion is counted (R-16).

use super::check::{AxisRead, Checker, Figure, Scope, Stands, Var, VarDom, deeper};
use super::keys::{KeyLit, resolve_key};
use crate::base::{Code, Load, Refusal};
use crate::model::{
    CodeKind, Domain, IndexForm, KeyVal, KindSpec, NameKind, Node, NodeKind, PeriodWord, Reality,
    Rel, Shown, Ty, Unit,
};
use crate::num::Num;
use crate::syntax::{
    AddOp, AggFn, CmpOp, Expr, Field, FieldValue, Key, Lit, MulOp, Number, Pin, PinForm, Postfix,
    QueryFn, Span, Stop,
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

/// A number error met while typing, an invariant of the unit algebra.
fn unit_stop(_: crate::num::NumError) -> Stop {
    Stop::Refused(Refusal::new(
        Code::EngineInvariant,
        "the unit algebra overflowed while typing".to_owned(),
    ))
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
        .keys
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
fn word_pk(word: PeriodWord) -> PeriodKind {
    match word {
        PeriodWord::Month => PeriodKind::Month,
        PeriodWord::Quarter => PeriodKind::Quarter,
        PeriodWord::Year => PeriodKind::Year,
    }
}

/// A period kind's name.
fn pk_name(pk: PeriodKind) -> &'static str {
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
fn num(unit: Unit) -> Ty {
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

/// An expression's text, as a message names it (refcore's `_txt`).
pub(super) fn text(e: &Expr) -> String {
    match e {
        Expr::Name(id) => id.clone(),
        Expr::Lit(lit) => lit_txt(lit),
        Expr::Str(s) => format!("\"{s}\""),
        Expr::None => "none".to_owned(),
        Expr::Never => "never".to_owned(),
        Expr::Cut => "cut".to_owned(),
        Expr::Paren(inner) => format!("({})", text(inner)),
        Expr::Neg(inner) => format!("-{}", text(inner)),
        Expr::Not(inner) => format!("not {}", text(inner)),
        Expr::Sum(first, rest) => {
            let signed: Vec<(&str, &Expr)> = rest
                .iter()
                .map(|(op, term)| (add_sign(*op), term))
                .collect();
            chain_text(first, &signed)
        }
        Expr::Prod(first, rest) => {
            let signed: Vec<(&str, &Expr)> = rest
                .iter()
                .map(|(op, factor)| (mul_sign(*op), factor))
                .collect();
            chain_text(first, &signed)
        }
        Expr::Or(terms) => joined(terms, "or"),
        Expr::And(terms) => joined(terms, "and"),
        Expr::Cmp(cmp) => format!("{} {} {}", text(&cmp.0), cmp_sign(cmp.1), text(&cmp.2)),
        Expr::If(parts) => {
            let [c, a, b] = parts.as_ref();
            format!("if {} then {} else {}", text(c), text(a), text(b))
        }
        Expr::In(test) => {
            let on = test
                .on
                .as_ref()
                .map_or_else(String::new, |on| format!(" on {}", text(on)));
            format!("{} in {}{on}", text(&test.member), text(&test.set))
        }
        Expr::Post(base, posts) => post_text(base, posts),
        Expr::Call(call) => format!("{}({})", call.name, list_text(&call.args)),
        Expr::Agg(agg) => format!("{}(...)", agg_word(agg.func)),
        Expr::Last(pair) => format!("last({}, {})", text(&pair.0), text(&pair.1)),
        Expr::AsIf(inner, _) => format!("{} as if {{...}}", text(inner)),
        Expr::Bal(bal) => {
            let mut s = format!("bal({}", text(&bal.place));
            for arg in &bal.args {
                s.push_str(", ");
                s.push_str(&text(arg));
            }
            if bal.start {
                s.push_str(" start");
            }
            s.push(')');
            if bal.modelled {
                s.push_str(" modelled");
            }
            if let Some(from) = &bal.proj.from {
                s.push_str(" from ");
                s.push_str(&text(from));
            }
            s
        }
        Expr::Value(value) => format!(
            "value({}, {}) in {}",
            text(&value.place),
            text(&value.day),
            value.currency
        ),
        Expr::List(_) => "list".to_owned(),
        Expr::Record(_) => "record".to_owned(),
        Expr::LegOf(..) => "legof".to_owned(),
        Expr::To(..) => "to".to_owned(),
        Expr::Flows(flows) => flows_text(flows),
        Expr::Covered(_) => "covered".to_owned(),
        Expr::Calendar(_) => "calendar".to_owned(),
        Expr::Query(q) => {
            let mut s = text(&q.arg);
            if q.binds.is_some() {
                s.push_str(" for ...");
            }
            if let Some(filter) = &q.filter {
                s.push_str(" where ");
                s.push_str(&text(filter));
            }
            if let Some(level) = &q.level {
                s.push_str(", ");
                s.push_str(&text(level));
            }
            let word = match q.func {
                QueryFn::Mean => "mean",
                QueryFn::Share => "share",
                QueryFn::Quantile => "quantile",
            };
            format!("{word}({s})")
        }
        Expr::Low(low) => {
            let mut s = format!(
                "low({}, {})",
                text(&low.place),
                super::reads::span_text(&low.span)
            );
            if let Some(from) = &low.proj.from {
                s.push_str(" from ");
                s.push_str(&text(from));
            }
            s
        }
        Expr::Crps(_) => "crps".to_owned(),
        Expr::DistFn(_) => "distfn".to_owned(),
        Expr::Dist(_) => "dist".to_owned(),
        Expr::Prorata(_) => "prorata".to_owned(),
    }
}

/// A `flows` read's text, its `for` and its conversion included (refcore's `_txt` of
/// `flows`).
fn flows_text(flows: &crate::syntax::Flows) -> String {
    let mut s = format!(
        "flows({}, {}",
        text(&flows.target),
        super::reads::span_text(&flows.span)
    );
    if let Some(key) = &flows.for_key {
        s.push_str(", for ");
        s.push_str(&text(key));
    }
    s.push(')');
    if let Some((cur, fx, mode)) = &flows.convert {
        s = format!("{s} in {cur} at {} {}", text(fx), text(mode));
    }
    s
}

/// A literal's text: a number with its unit as the unit algebra prints it.
fn lit_txt(lit: &Lit) -> String {
    if let Lit::Num(n) = lit
        && let Some(unit) = &n.unit
    {
        let terms = unit
            .terms
            .iter()
            .map(|t| (t.base.clone(), t.power))
            .collect();
        if let Ok((u, _)) = Unit::from_syntax(terms) {
            return format!("{} {}", n.text, u.text());
        }
    }
    lit.text()
}

/// A left-nested chain's text, `a + b - c`.
fn chain_text(first: &Expr, rest: &[(&str, &Expr)]) -> String {
    let mut out = text(first);
    for (sign, e) in rest {
        out = format!("{out} {sign} {}", text(e));
    }
    out
}

/// A chain of `and` or `or`.
fn joined(terms: &[Expr], word: &str) -> String {
    let shown: Vec<String> = terms.iter().map(text).collect();
    shown.join(&format!(" {word} "))
}

/// Expressions joined by `, `.
fn list_text(exprs: &[Expr]) -> String {
    let shown: Vec<String> = exprs.iter().map(text).collect();
    shown.join(", ")
}

/// A fill's output at the head of `base` and `posts`: `F.taken[s, d]` or `F[k].taken[s, d]`,
/// `F` a fill: its id, its own keys, the keys of `taken`, and how many postfixes they take
/// (refcore's `m_fill_ref`). L §5.12.
fn fill_ref(
    m: &Reality,
    scope: &Scope,
    base: &Expr,
    posts: &[Postfix],
) -> Option<(String, Vec<Key>, Vec<Key>, usize)> {
    let Expr::Name(name) = base else {
        return None;
    };
    if scope.is_var(name) || m.names.get(name) != Some(&NameKind::Fill) {
        return None;
    }
    let (fkeys, at) = match posts {
        [Postfix::Field(f), ..] if f == "taken" => (Vec::new(), 1),
        [Postfix::Keys(keys), Postfix::Field(f), ..] if f == "taken" => (keys.clone(), 2),
        _ => return None,
    };
    match posts.get(at) {
        Some(Postfix::Keys(keys)) => Some((name.clone(), fkeys, keys.clone(), at.checked_add(1)?)),
        Some(Postfix::Field(_)) | None => Some((name.clone(), fkeys, Vec::new(), at)),
    }
}

/// A base with its postfixes, `x[k].f`.
fn post_text(base: &Expr, posts: &[Postfix]) -> String {
    let mut out = text(base);
    for post in posts {
        match post {
            Postfix::Keys(keys) => {
                let shown: Vec<String> = keys.iter().map(key_text).collect();
                out = format!("{out}[{}]", shown.join(", "));
            }
            Postfix::Field(name) => out = format!("{out}.{name}"),
        }
    }
    out
}

/// A key position's text.
fn key_text(key: &Key) -> String {
    match key {
        Key::Star => "star".to_owned(),
        Key::Expr(e) => text(e),
    }
}

/// `+` or `-`.
fn add_sign(op: AddOp) -> &'static str {
    match op {
        AddOp::Add => "+",
        AddOp::Sub => "-",
    }
}

/// `*` or `/`.
fn mul_sign(op: MulOp) -> &'static str {
    match op {
        MulOp::Mul => "*",
        MulOp::Div => "/",
    }
}

/// A comparison's operator.
fn cmp_sign(op: CmpOp) -> &'static str {
    match op {
        CmpOp::Eq => "==",
        CmpOp::Ne => "!=",
        CmpOp::Lt => "<",
        CmpOp::Le => "<=",
        CmpOp::Gt => ">",
        CmpOp::Ge => ">=",
    }
}

/// An aggregate's word.
fn agg_word(func: AggFn) -> &'static str {
    match func {
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

/// A span's text as a guard names it, `a..b` (refcore's `span_txt`).
pub(super) fn span_text(span: &Span) -> String {
    let from = span.from.as_ref().map_or_else(String::new, text);
    if !span.dots {
        return from;
    }
    let to = span.to.as_ref().map_or_else(String::new, text);
    format!("{from}..{to}")
}

/// The spans a condition's `covered()` guards (refcore's `m_covered_spans_of`). L §5.19.
pub(super) fn covered_spans(e: Option<&Expr>, out: &mut std::collections::BTreeSet<String>) {
    let Some(e) = e else {
        return;
    };
    let mut stack = vec![e];
    while let Some(next) = stack.pop() {
        if let Expr::Covered(pair) = next {
            out.insert(span_text(&pair.1));
        }
        stack.extend(children(next));
    }
}

/// The expressions directly under `e`, for a walk over a whole tree.
pub(super) fn children(e: &Expr) -> Vec<&Expr> {
    let mut out = Vec::new();
    match e {
        Expr::If(parts) => out.extend(parts.iter()),
        Expr::Or(terms) | Expr::And(terms) => out.extend(terms.iter()),
        Expr::Not(inner)
        | Expr::Neg(inner)
        | Expr::Paren(inner)
        | Expr::AsIf(inner, _)
        | Expr::LegOf(inner, _)
        | Expr::To(inner, _) => out.push(inner),
        Expr::Cmp(cmp) => out.extend([&cmp.0, &cmp.2]),
        Expr::In(test) => {
            out.extend([&test.member, &test.set]);
            out.extend(test.on.as_ref());
        }
        Expr::Sum(first, rest) => {
            out.push(first);
            out.extend(rest.iter().map(|(_, term)| term));
        }
        Expr::Prod(first, rest) => {
            out.push(first);
            out.extend(rest.iter().map(|(_, factor)| factor));
        }
        Expr::Record(fields) => {
            for field in fields {
                match field {
                    Field::Named(_, FieldValue::Expr(value)) => out.push(value),
                    Field::Named(
                        _,
                        FieldValue::Lit(_) | FieldValue::Hole(_) | FieldValue::None,
                    ) => {}
                    Field::Leg(leg) => {
                        out.extend([&leg.from, &leg.to, &leg.amount]);
                        out.extend(leg.on.as_ref());
                    }
                }
            }
        }
        Expr::Flows(flows) => out.push(&flows.target),
        Expr::Post(base, posts) => {
            out.push(base);
            for post in posts {
                if let Postfix::Keys(keys) = post {
                    out.extend(keys.iter().filter_map(key_expr));
                }
            }
        }
        Expr::Call(call) => out.extend(call.args.iter()),
        Expr::Agg(agg) => out.extend(agg.term.as_ref()),
        Expr::Last(pair) => out.extend([&pair.0, &pair.1]),
        Expr::Bal(bal) => {
            out.push(&bal.place);
            out.extend(bal.args.iter());
        }
        Expr::Covered(pair) => out.push(&pair.0),
        Expr::Value(value) => out.extend([&value.place, &value.day]),
        Expr::Lit(_)
        | Expr::Str(_)
        | Expr::Name(_)
        | Expr::None
        | Expr::Never
        | Expr::Cut
        | Expr::List(_)
        | Expr::Calendar(_)
        | Expr::Query(_)
        | Expr::Low(_)
        | Expr::Crps(_)
        | Expr::DistFn(_)
        | Expr::Dist(_)
        | Expr::Prorata(_) => {}
    }
    out
}

/// A key position's expression, unless it is `*`.
fn key_expr(key: &Key) -> Option<&Expr> {
    match key {
        Key::Star => None,
        Key::Expr(e) => Some(e),
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
    let input = !scope.is_var(id) && m.names.get(id) == Some(&NameKind::Input);
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
fn key_lit_of(e: &Expr) -> Option<KeyLit> {
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
            return Err(refused(Load::Name, format!("unknown id {ix}")));
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
    fn name(
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
        let Some(kind) = m.names.get(name).copied() else {
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

    /// A read of an input or a def at keys (refcore's `read`). L §2.5, §4.6; E §3.1 step 5.
    pub(super) fn read(
        &mut self,
        m: &Reality,
        scope: &Scope,
        id: &str,
        kind: NameKind,
        keys: &[Key],
        depth: u32,
    ) -> Typed {
        let affx = if core::mem::take(&mut self.affx_hit) {
            super::random::SampleOf::AffineX
        } else {
            super::random::SampleOf::Scope
        };
        let (binds, declared, observed, priced) = if kind == NameKind::Input {
            let Some(input) = m.inputs.get(id) else {
                return Err(Stop::NotBuilt("an input that isn't built"));
            };
            let priced = input.prices.is_some() && input.prices_by.is_some();
            (&input.binds, input.ty.clone(), input.observed, priced)
        } else {
            let Some(def) = m.defs.get(id) else {
                return Err(Stop::NotBuilt("a def that isn't built"));
            };
            (&def.binds, Some(def.ty.clone()), false, false)
        };
        let node_kind = if kind == NameKind::Input {
            NodeKind::Input
        } else {
            NodeKind::Def
        };
        if keys.len() != binds.len() && !observed {
            return Err(refused(
                Load::Key,
                format!(
                    "wrong number of keys for {id}: {}, not {}",
                    keys.len(),
                    binds.len()
                ),
            ));
        }
        if observed {
            let by_id = match keys {
                [Key::Expr(key)] => is_keylit_node(m, scope, key),
                _ => false,
            };
            if !by_id {
                return Err(refused(
                    Load::Key,
                    format!("an observation is read by its id: {id}[<id>]"),
                ));
            }
            self.edge(&scope.item, Node::new(NodeKind::Input, id), Rel::Any);
            return Ok(Ty::Key("events".to_owned()));
        }
        let mut env = Vec::new();
        for (key, bind) in keys.iter().zip(binds) {
            let Key::Expr(kn) = key else {
                return Err(refused(
                    Load::Parse,
                    "cannot parse: * stands only in a pin".to_owned(),
                ));
            };
            let lit = self.key_pos(m, scope, kn, &bind.domain, depth)?;
            let mut value = "$?".to_owned();
            if let Expr::Name(name) = kn
                && scope.is_var(name)
            {
                value = format!("${name}");
            }
            if let Some(named) = lit {
                value = named.text();
            }
            env.push((bind.var.clone(), value));
        }
        let target = Node::new(node_kind, id);
        let rel = super::check::relation(scope, keys);
        self.direct(&target);
        self.shifted(scope, &target, &rel);
        self.edge(&scope.item, target.clone(), rel);
        let mut ty = declared.unwrap_or(Ty::Unseen);
        if priced {
            ty = num(unit_of(&ty)?);
        }
        if let Ty::Num(unit, shown) = &ty
            && unit.terms().iter().any(|(base, _)| base.starts_with('$'))
        {
            ty = Ty::Num(unit.subst(&env).map_err(unit_stop)?, *shown);
        }
        if let Ty::Dist(unit) = &ty
            && unit.terms().iter().any(|(base, _)| base.starts_with('$'))
        {
            ty = Ty::Dist(unit.subst(&env).map_err(unit_stop)?);
        }
        if matches!(ty, Ty::Dist(_)) {
            return self.dist_read(m, scope, &target, &ty, affx);
        }
        self.note_read(scope, &target);
        Ok(ty)
    }

    /// A place read by its id, or a keyed place at keys (refcore's `m_place_value`).
    /// L §5.10.
    pub(super) fn place_value(
        &mut self,
        m: &Reality,
        scope: &Scope,
        id: &str,
        keys: &[Key],
        depth: u32,
    ) -> Typed {
        let Some(place) = m.places.get(id) else {
            return Err(Stop::NotBuilt("a place that isn't built"));
        };
        if keys.len() != place.binds.len() {
            return Err(refused(
                Load::Key,
                format!(
                    "wrong number of keys for place {id}: {}, not {}",
                    keys.len(),
                    place.binds.len()
                ),
            ));
        }
        for (key, bind) in keys.iter().zip(&place.binds) {
            let Key::Expr(kn) = key else {
                return Err(refused(
                    Load::Parse,
                    "cannot parse: * stands only in a pin".to_owned(),
                ));
            };
            self.key_pos(m, scope, kn, &bind.domain, depth)?;
        }
        self.edge(&scope.item, Node::new(NodeKind::Place, id), Rel::Any);
        Ok(Ty::Key(place.kind.clone()))
    }

    /// A movement instance named by its move and keys; a decision's is a running read of its
    /// legs (refcore's `m_move_value`). L §2.5, §5.14.1.
    pub(super) fn move_value(
        &mut self,
        m: &Reality,
        scope: &Scope,
        id: &str,
        keys: &[Key],
        depth: u32,
    ) -> Typed {
        let Some(mv) = m.moves.get(id) else {
            return Err(Stop::NotBuilt("a move that isn't built"));
        };
        if keys.len() != mv.binds.len() {
            return Err(refused(
                Load::Key,
                format!(
                    "wrong number of keys for {id}: {}, not {}",
                    keys.len(),
                    mv.binds.len()
                ),
            ));
        }
        for (key, bind) in keys.iter().zip(&mv.binds) {
            let Key::Expr(kn) = key else {
                return Err(refused(
                    Load::Parse,
                    "cannot parse: * stands only in a pin".to_owned(),
                ));
            };
            self.key_pos(m, scope, kn, &bind.domain, depth)?;
        }
        self.edge(&scope.item, Node::new(NodeKind::Move, id), Rel::Any);
        if mv.decision {
            self.mark(
                &scope.item,
                crate::model::Mark::Legs,
                crate::model::Target::Move(id.to_owned()),
            );
        }
        Ok(Ty::Key(mv.kind.clone()))
    }

    /// A key at a key position whose domain is `domain`: typed there, and checked against
    /// it; the key it names when written as a literal (refcore's `key_pos`). L §2.5, §4.2,
    /// §4.6.
    pub(super) fn key_pos(
        &mut self,
        m: &Reality,
        scope: &Scope,
        kn: &Expr,
        domain: &Domain,
        depth: u32,
    ) -> Result<Option<KeyVal>, Stop> {
        if let (Domain::Axis(axis), Expr::Name(name)) = (domain, kn)
            && !scope.is_var(name)
        {
            if name == axis {
                self.axis_read(&scope.item, axis, AxisRead::AtKey);
                return Ok(None);
            }
            if m.axes.contains(name) {
                return Err(refused(
                    Load::Key,
                    format!("{name} is used as a key of {axis}"),
                ));
            }
        }
        if let Domain::Index(ix) = domain
            && let Some(pk) = m.indexes.get(ix).and_then(|index| index.period)
        {
            let gran = granularity(kn);
            if gran.is_some_and(|g| g != pk) {
                let t = self.expr(m, scope, kn, None, depth)?;
                self.key_conv(m, scope, &t, domain, kn)?;
                return Ok(None);
            }
            let four = matches!(kn, Expr::Lit(Lit::Num(n)) if n.four);
            if four && pk != PeriodKind::Year {
                let t = self.expr(m, scope, kn, Some(&Ty::Period(PeriodWord::Year)), depth)?;
                self.key_conv(m, scope, &t, domain, kn)?;
                return Ok(None);
            }
        }
        let mut range_lit = false;
        if is_keylit_node(m, scope, kn) {
            match domain {
                Domain::Index(_) | Domain::Kind(_) | Domain::Axis(_) => {
                    let dated = matches!(
                        kn,
                        Expr::Lit(Lit::Day(_) | Lit::Month(..) | Lit::Quarter(..))
                    );
                    let year = matches!(kn, Expr::Lit(Lit::Num(n)) if n.four)
                        && matches!(domain, Domain::Index(ix) if m.indexes.get(ix).is_some_and(|i| i.period.is_some()));
                    if dated || year {
                        lit_rule(scope, Figure::Date, "", None)?;
                    }
                    return resolve_lit(m, scope, kn, domain).map(Some);
                }
                Domain::List(_) => return resolve_lit(m, scope, kn, domain).map(Some),
                Domain::Range => range_lit = true,
            }
        }
        let mut inner = scope.clone();
        if matches!(kn, Expr::Sum(..)) || range_lit {
            inner.allow_lit = true;
        }
        let expect = match domain {
            Domain::Index(ix) | Domain::Kind(ix) | Domain::Axis(ix) => Some(Ty::Key(ix.clone())),
            Domain::List(_) | Domain::Range => None,
        };
        let t = self.expr(m, &inner, kn, expect.as_ref(), depth)?;
        self.key_conv(m, scope, &t, domain, kn)?;
        Ok(None)
    }

    /// A key's type checked against its position's domain (refcore's `key_conv`). L §2.5,
    /// §4.2 Keys as values, §4.6.
    fn key_conv(
        &mut self,
        m: &Reality,
        scope: &Scope,
        t: &Ty,
        domain: &Domain,
        kn: &Expr,
    ) -> Result<(), Stop> {
        let var = if let Expr::Name(name) = kn {
            scope.vars.get(name)
        } else {
            None
        };
        let listed = var.is_some_and(|v| matches!(v.ty, Ty::KeyList));
        if matches!(t, Ty::Unseen) {
            return Ok(());
        }
        if matches!(t, Ty::KeyList) || (listed && matches!(domain, Domain::Index(_))) {
            if let Some(var) = var {
                self.fix_list(m, scope, var, domain)?;
            }
            return Ok(());
        }
        let not_a_key = |dom: &str| {
            refused(
                Load::Key,
                format!("{} ({}) is not a key of {dom}", text(kn), t.show()),
            )
        };
        match domain {
            Domain::Index(ix) => {
                let Some(index) = m.indexes.get(ix) else {
                    return Err(Stop::NotBuilt("an index that isn't built"));
                };
                if let Ty::Key(other) = t {
                    if key_of_index(m, (ix, index), other, kn)? == Conv::Period {
                        self.period_read(scope, ix, kn);
                    }
                    return Ok(());
                }
                if index.is_int && matches!(t, Ty::Int | Ty::Flex(_) | Ty::Year4) {
                    return Ok(());
                }
                if index.is_int && matches!(t, Ty::Num(unit, _) if unit.is_one()) {
                    return Ok(());
                }
                if index.period.is_some() {
                    if matches!(t, Ty::Date | Ty::Never) {
                        return Ok(());
                    }
                    if let Ty::Period(word) = t {
                        period_conv(index.period, ix, word_pk(*word))?;
                        self.period_read(scope, ix, kn);
                        return Ok(());
                    }
                }
                Err(not_a_key(ix))
            }
            Domain::Kind(kind) => {
                if let Ty::Key(other) = t
                    && m.kinds.get(other).is_some()
                    && (m.subtree(kind).contains(other) || m.subtree(other).contains(kind))
                {
                    return Ok(());
                }
                let unit_kind = m.kinds.get(kind).is_some_and(|k| k.spec == KindSpec::Unit);
                if matches!(t, Ty::UnitKey) && (unit_kind || kind == "currencies") {
                    return Ok(());
                }
                Err(not_a_key(kind))
            }
            Domain::Axis(axis) => {
                if matches!(t, Ty::Key(other) if other == axis) {
                    return Ok(());
                }
                Err(not_a_key(axis))
            }
            Domain::List(_) | Domain::Range => Ok(()),
        }
    }

    /// A key-list variable read at a key position: the first position fixes its index, a
    /// second index is `LOAD-KEY`, and each of its keys must be a key there (refcore's
    /// `key_conv`, its key-list arm). L §2.5.
    pub(super) fn fix_list(
        &mut self,
        m: &Reality,
        scope: &Scope,
        var: &Var,
        domain: &Domain,
    ) -> Result<(), Stop> {
        let name = domain_name(domain);
        match self.kl_fixed.get(&var.id) {
            Some(fixed) if fixed != domain => {
                return Err(refused(
                    Load::Key,
                    format!(
                        "{} is used as a key of {} and {}",
                        var.name,
                        domain_name(fixed),
                        name
                    ),
                ));
            }
            Some(_) => {}
            None => {
                self.kl_fixed.insert(var.id, domain.clone());
            }
        }
        for key in &var.keylist {
            resolve_lit(m, scope, &Expr::Lit(key.clone()), domain)?;
        }
        Ok(())
    }

    /// A base with its postfixes: a name read at keys, then fields (refcore's `t_index`,
    /// `t_field`). L §2.5, §3.1, §5.14.
    fn post(
        &mut self,
        m: &Reality,
        scope: &Scope,
        base: &Expr,
        posts: &[Postfix],
        expect: Option<&Ty>,
        depth: u32,
    ) -> Typed {
        let Some(first) = posts.first() else {
            return self.expr(m, scope, base, expect, depth);
        };
        if let Some((fill, fkeys, keys, used)) = fill_ref(m, scope, base, posts) {
            let ty = self.fill_read(m, scope, &fill, (&fkeys, &keys), depth)?;
            return Self::post_rest(m, scope, base, posts, (ty, used));
        }
        let ty = match first {
            Postfix::Keys(keys) => self.index(m, scope, base, keys, depth)?,
            Postfix::Field(field) => {
                let t = self.expr(m, scope, base, None, depth)?;
                Self::field(m, scope, Some(base), &t, field)?
            }
        };
        Self::post_rest(m, scope, base, posts, (ty, 1))
    }

    /// The postfixes after the first `done` of `posts`, the value so far of type `ty`: fields
    /// of it; a key there is no read (refcore's `t_index`, `t_field`). L §2.5, §3.1.
    fn post_rest(
        m: &Reality,
        scope: &Scope,
        base: &Expr,
        posts: &[Postfix],
        (mut ty, mut done): (Ty, usize),
    ) -> Typed {
        for post in posts.get(done..).unwrap_or(&[]) {
            let prefix = post_text(base, posts.get(..done).unwrap_or(&[]));
            match post {
                Postfix::Keys(_) => {
                    return Err(refused(
                        Load::Parse,
                        format!("cannot parse: {prefix} is not read with keys"),
                    ));
                }
                Postfix::Field(field) => ty = Self::field(m, scope, None, &ty, field)?,
            }
            done = done
                .checked_add(1)
                .ok_or(Stop::NotBuilt("a long postfix chain"))?;
        }
        Ok(ty)
    }

    /// `x[k]` with `x` a name (refcore's `t_index`). L §2.5.
    fn index(
        &mut self,
        m: &Reality,
        scope: &Scope,
        base: &Expr,
        keys: &[Key],
        depth: u32,
    ) -> Typed {
        if let Expr::Name(name) = base
            && !scope.is_var(name)
        {
            let Some(kind) = m.names.get(name).copied() else {
                return Err(refused(Load::Name, format!("unknown id {name}")));
            };
            return match kind {
                NameKind::Input | NameKind::Def => self.read(m, scope, name, kind, keys, depth),
                NameKind::Place => self.place_value(m, scope, name, keys, depth),
                NameKind::Move => self.move_value(m, scope, name, keys, depth),
                NameKind::Unit
                | NameKind::Kind
                | NameKind::Index
                | NameKind::Axis
                | NameKind::Band
                | NameKind::Fill
                | NameKind::Version
                | NameKind::Ask
                | NameKind::Source
                | NameKind::Cut => Err(refused(
                    Load::Parse,
                    format!(
                        "cannot parse: {} {name} is not read with keys",
                        name_word(kind)
                    ),
                )),
            };
        }
        Err(refused(
            Load::Parse,
            format!("cannot parse: {} is not read with keys", text(base)),
        ))
    }

    /// `x.f` of a value of type `t` (refcore's `t_field`). L §3.1, §5.7, §5.14.
    pub(super) fn field(
        m: &Reality,
        scope: &Scope,
        base: Option<&Expr>,
        t: &Ty,
        field: &str,
    ) -> Typed {
        if let (Some(Expr::Name(name)), Ty::KeyLit(_)) = (base, t)
            && !scope.is_var(name)
            && !m.names.contains_key(name)
        {
            return Err(refused(Load::Name, format!("unknown id {name}.{field}")));
        }
        if matches!(t, Ty::Unseen) {
            return Ok(Ty::Unseen);
        }
        if let Ty::Record(_) = t {
            return t.field(field).cloned().ok_or_else(|| {
                refused(
                    Load::Field,
                    format!("{field} is not a field of {}", t.show()),
                )
            });
        }
        if let Ty::Key(kind) = t
            && let Some(info) = m.kinds.get(kind)
        {
            if info.spec == KindSpec::Event {
                return Ok(movement_field(m, kind, field));
            }
            return Err(refused(
                Load::Field,
                format!("{field} is not a field of a thing of {kind}"),
            ));
        }
        Err(refused(
            Load::Field,
            format!("{field} is not a field of {}", t.show()),
        ))
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
fn granularity(e: &Expr) -> Option<PeriodKind> {
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
            terms.push((term.base.clone(), term.power));
        } else if crate::model::is_currency_code(base) {
            return Err(uerr(format!("money in an undeclared currency {base}")));
        } else {
            return Err(refused(Load::Name, format!("unknown id {base} (a unit)")));
        }
    }
    let (made, factor) = Unit::from_syntax(terms).map_err(unit_stop)?;
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
    if matches!(unit.terms.as_slice(), [t] if t.base == "%" && t.power == 1) {
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

/// The domain a key type's index, kind or axis is (refcore's `dom_of`).
pub(super) fn domain_of(m: &Reality, ix: &str) -> Option<Domain> {
    if m.indexes.get(ix).is_some() {
        return Some(Domain::Index(ix.to_owned()));
    }
    if m.kinds.get(ix).is_some() {
        return Some(Domain::Kind(ix.to_owned()));
    }
    (m.names.get(ix) == Some(&NameKind::Axis)).then(|| Domain::Axis(ix.to_owned()))
}

/// A domain's name, as a message names it.
fn domain_name(domain: &Domain) -> String {
    match domain {
        Domain::Index(id) | Domain::Kind(id) | Domain::Axis(id) => id.clone(),
        Domain::List(_) => "list".to_owned(),
        Domain::Range => "range".to_owned(),
    }
}

/// A key literal's key in `domain`, refused at the scope's line (refcore's
/// `resolve_key_literal`). L §2.5.
pub(super) fn resolve_lit(
    m: &Reality,
    scope: &Scope,
    node: &Expr,
    domain: &Domain,
) -> Result<KeyVal, Stop> {
    let Some(key) = key_lit_of(node) else {
        return Err(refused(
            Load::Key,
            format!("{} is not a key of {}", text(node), domain_name(domain)),
        ));
    };
    let (file, line) = scope.file_line();
    resolve_key(m, &key, domain, &file, line)
}

/// How a key stands at its key position: as a key there, or as a period read at a key of a
/// time index, which load folds at the end (refcore's `key_conv`, its `period` result).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Conv {
    /// A key of the position's index, an integer, or a day.
    Plain,
    /// A period no wider than the index's keys.
    Period,
}

/// A key of index `other` at a key of `ix`: the same index, two integer indexes, or two
/// period indexes where the period isn't wider (refcore's `key_conv`, its key arm). L §2.5,
/// §4.6.
fn key_of_index(
    m: &Reality,
    (ix, index): (&str, &crate::model::IndexInfo),
    other: &str,
    kn: &Expr,
) -> Result<Conv, Stop> {
    if other == ix {
        return Ok(Conv::Plain);
    }
    if let Some(theirs) = m.indexes.get(other) {
        if index.is_int && theirs.is_int {
            return Ok(Conv::Plain);
        }
        if let (Some(_), Some(pk)) = (index.period, theirs.period) {
            period_conv(index.period, ix, pk)?;
            return Ok(Conv::Period);
        }
    }
    Err(refused(
        Load::Key,
        format!("{} is used as a key of {other} and {ix}", text(kn)),
    ))
}

/// `LOAD-KEY` when a period is wider than the index's keys (refcore's `period_conv`).
/// L §4.6.
fn period_conv(index_pk: Option<PeriodKind>, ix: &str, pk: PeriodKind) -> Result<(), Stop> {
    let order = |p: Option<PeriodKind>| match p {
        Some(PeriodKind::Day) => 0,
        Some(PeriodKind::Month) => 1,
        Some(PeriodKind::Quarter) => 2,
        Some(PeriodKind::Year) => 3,
        None => 9,
    };
    if order(Some(pk)) > order(index_pk) {
        let keys = index_pk.map_or("?", pk_name);
        return Err(refused(
            Load::Key,
            format!(
                "a {} holds no key of {ix}, whose keys are {keys}s",
                pk_name(pk)
            ),
        ));
    }
    Ok(())
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
        || (m.names.get(u) == Some(&NameKind::Kind));
    if named {
        return Ok(num(Unit::make(vec![(u.to_owned(), 1)]).map_err(unit_stop)?));
    }
    Ok(Ty::Dyn)
}

/// A field of a movement of event kind `kind`: a leg, an attribute, `day`, `fulfils`; any
/// other is `Unseen` until kind tests narrow (refcore's `m_movement_field`). L §5.14.
fn movement_field(m: &Reality, kind: &str, field: &str) -> Ty {
    let places = || Ty::Key("places".to_owned());
    for (leg, unit, _) in m.kind_legs(kind) {
        if leg == field {
            let amount = unit_type(m, Some(&unit)).unwrap_or(Ty::Unseen);
            return Ty::Record(vec![
                ("from".to_owned(), places()),
                ("to".to_owned(), places()),
                ("amount".to_owned(), amount),
                ("day".to_owned(), Ty::Date),
            ]);
        }
    }
    for info in m.chain(kind) {
        for (attr, ty) in &info.attr_types {
            if attr == field {
                return ty.clone().unwrap_or(Ty::Dyn);
            }
        }
    }
    match field {
        "day" => Ty::Date,
        "fulfils" => Ty::Key("events".to_owned()),
        _ => Ty::Unseen,
    }
}
