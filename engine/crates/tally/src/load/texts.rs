//! Load's checker, its texts: an expression as a load refusal names it, and the expressions directly under each node.
//! Spec: L §8.3 (a load refusal names what it refuses as written), §5.12 (a fill's output read
//! as `F.taken[s, d]`), §5.19 (the spans a `covered()` guard names).
//! Twin: engine/tools/refcore/check.py (`_txt`); check2.py (`m_fill_ref`); check3.py (`_txt`,
//! `span_txt`, `m_covered_spans_of`).
//! Invariants: a text is refcore's byte for byte, made from the tree alone, never from a value;
//! `children` gives every expression directly under a node, so a walk over them sees the whole
//! tree.

use super::check::Scope;
use crate::model::{NameKind, Reality, Unit};
use crate::syntax::{
    AddOp, CmpOp, Domain, Expr, Field, FieldValue, Key, Lit, MulOp, Postfix, QueryFn, Span,
};

/// A binding's domain as the text writes it: an index or kind with its member's prefix, a
/// key list, a range with its step (refcore's `_txt`, its `dname`, `dlist`, `drange` arms;
/// L §2.4 Values, tally 2.1.7 R9).
pub(crate) fn domain_text(d: &Domain) -> String {
    match d {
        Domain::Name { id, qual: None } => id.clone(),
        Domain::Name { id, qual: Some(q) } => format!("{q}.{id}"),
        Domain::List(keys) => {
            let keys: Vec<String> = keys.iter().map(lit_txt).collect();
            format!("[{}]", keys.join(", "))
        }
        Domain::Range(range) => {
            let (from, to, step) = range.as_ref();
            let step = step
                .as_ref()
                .map_or_else(String::new, |s| format!(" step {}", text(s)));
            format!("{}..{}{step}", text(from), text(to))
        }
    }
}

/// An expression's text, as a message names it (refcore's `_txt`).
pub(crate) fn text(e: &Expr) -> String {
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
        Expr::Agg(agg) => format!("{}(...)", agg.func.word()),
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
pub(super) fn lit_txt(lit: &Lit) -> String {
    if let Lit::Num(n) = lit
        && let Some(unit) = &n.unit
    {
        let terms = unit
            .terms
            .iter()
            .map(|t| (t.base.clone(), t.power.clone()))
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
pub(super) fn fill_ref(
    m: &Reality,
    scope: &Scope,
    base: &Expr,
    posts: &[Postfix],
) -> Option<(String, Vec<Key>, Vec<Key>, usize)> {
    let Expr::Name(name) = base else {
        return None;
    };
    if scope.is_var(name) || m.names.kind(name) != Some(NameKind::Fill) {
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
pub(super) fn post_text(base: &Expr, posts: &[Postfix]) -> String {
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

/// A comparison's operator as written.
pub(super) fn cmp_sign(op: CmpOp) -> &'static str {
    match op {
        CmpOp::Eq => "==",
        CmpOp::Ne => "!=",
        CmpOp::Lt => "<",
        CmpOp::Le => "<=",
        CmpOp::Gt => ">",
        CmpOp::Ge => ">=",
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
pub(crate) fn children(e: &Expr) -> Vec<&Expr> {
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
