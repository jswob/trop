//! The values a binding ranges over: an index's keys the cut sees, a key list's, a kind's things alive on a day or over a span.
//! Spec: L §2.5 (binding order: an index's order, a kind's things by the text of `kind:key`;
//! a key list's literals as keys of the index its key positions fix), §5.7 (a binding over a
//! kind `on d` or `over s`, alive by its memberships), §5.6 (an index's keys), §4.5 (the keys
//! and things a cut sees), §7.6 item 1 (the end of time is no day to end a span on). The
//! aggregates over these values are `aggregate.rs`'s; a stepped range's keys `ranges.rs`'s;
//! movements `book/members.rs`'s.
//! Twin: engine/tools/refcore/evaluate.py (`bind_values`, `span_value`, `span_start`,
//! `span_end`, `bind_day`, `kind_has_member`) and model.py (`alive_on`, `alive_over`,
//! `kind_order_key`, `keys_at`).
//! Invariants: values come in binding order; a day or span that is a hole makes the domain
//! that hole, unless no thing of the kind could be a member; a key a version added after the
//! cut is no value.

use super::read::{lacks_at_run, literal_key};
use super::state::{Ctx, Engine, Env, a_day, invariant, not_typed, refuse, wrong_type};
use super::years::span_year;
use crate::base::Run;
use crate::hole::Hole;
use crate::model::{Class, Domain, IndexInfo, KeyVal, Number, Reality, Span, Thing, Value};
use crate::syntax::{
    Bind, Domain as Over, Expr, Field, FieldValue, Key, Lit, Postfix, Span as SpanText, Stop,
};
use crate::time::{Day, Period};
use std::collections::BTreeSet;

/// One binding's values, each with its own unknown-member hole, or the hole of a domain that
/// is unknown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Values {
    /// The values, in binding order.
    Known(Vec<(Value, Option<Hole>)>),
    /// The domain is a hole.
    Unknown(Hole),
    /// Movements, each with what it brings to a reader that keeps it.
    Moves(Vec<crate::book::MvMember>),
}

impl Engine {
    /// The values one binding ranges over in scope `env` (refcore's `bind_values`). L §5.11,
    /// §2.5, §5.7.
    pub(super) fn bind_values(
        &mut self,
        m: &Reality,
        bind: &Bind,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Values, Stop> {
        match &bind.domain {
            Over::Name { qual: Some(_), .. } => {
                Err(Stop::NotBuilt("a binding over a member's index or kind"))
            }
            Over::Name { id, qual: None } => {
                if let Some(ix) = m.indexes.get(id) {
                    if bind.on.is_some() || bind.over.is_some() {
                        return Err(Stop::NotBuilt("a binding over an index with a day or span"));
                    }
                    return index_values(m, ix, ctx.cut).map(Values::Known);
                }
                if m.kinds
                    .get(id)
                    .is_some_and(|k| k.spec != crate::model::KindSpec::Event)
                {
                    return self.kind_values(m, bind, id, env, ctx);
                }
                let dom = if m.inputs.get(id).is_some_and(|i| i.observed) {
                    (crate::book::MvDom::Obs, id.as_str())
                } else if m.kinds.get(id).is_some() {
                    (crate::book::MvDom::Event, id.as_str())
                } else {
                    return Err(invariant("a binding over a name load didn't resolve"));
                };
                Ok(match self.movement_members(m, bind, dom, env, ctx)? {
                    Err(h) => Values::Unknown(h),
                    Ok(moves) => Values::Moves(moves),
                })
            }
            Over::List(lits) => {
                let fixed = m.list_keys.get(&bind.site.key());
                list_values(m, lits, fixed).map(Values::Known)
            }
            Over::Range(parts) => self.range_values(m, parts, env, ctx),
        }
    }

    /// The things of kind `kind` the cut sees, alive on the binding's day or over its span, by
    /// the text of `kind:key` (refcore's `bind_values` for a kind). L §5.7, §2.5.
    fn kind_values(
        &mut self,
        m: &Reality,
        bind: &Bind,
        kind: &str,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Values, Stop> {
        let reads_own = bind.on.as_ref().is_some_and(|e| reads_name(e, &bind.var))
            || bind.over.as_ref().is_some_and(|s| span_reads(s, &bind.var));
        if reads_own {
            return Err(Stop::NotBuilt(
                "a binding whose day or span reads its own variable",
            ));
        }
        let things: Vec<&Thing> = m
            .things
            .entries()
            .map(|(_, t)| t)
            .filter(|t| super::read::thing_seen(m, t, ctx.cut) && m.thing_in(&t.name, kind))
            .collect();
        let when = match (&bind.on, &bind.over) {
            (Some(on), _) => {
                let day = self.ev(m, on, env, ctx)?;
                if let Value::Hole(h) = day {
                    return empty_or_unknown(bind, &things, h);
                }
                When::Day(day_of_binding(&day)?)
            }
            (None, Some(over)) => match self.span_value(m, over, env, ctx)? {
                Err(h) => return empty_or_unknown(bind, &things, h),
                Ok(span) => When::Over(span),
            },
            (None, None) => {
                return Err(Stop::NotBuilt("a binding over a kind with no day or span"));
            }
        };
        let mut alive: Vec<&Thing> = Vec::with_capacity(things.len());
        for thing in things {
            if is_alive(m, thing, kind, &when, ctx.cut)? {
                alive.push(thing);
            }
        }
        alive.sort_by_key(|t| order_text(t));
        Ok(Values::Known(
            alive
                .into_iter()
                .map(|t| (Value::Key(KeyVal::Thing(t.name.clone())), None))
                .collect(),
        ))
    }

    /// A span as written, `a..b`, `a..`, `..`, or one period or day (refcore's `span_value`).
    /// L §5.7.
    pub(crate) fn span_value(
        &mut self,
        m: &Reality,
        sp: &SpanText,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Result<Span, Hole>, Stop> {
        let mut ends: Vec<Option<Value>> = Vec::with_capacity(2);
        for end in [&sp.from, &sp.to] {
            let Some(e) = end else {
                ends.push(None);
                continue;
            };
            let v = self.ev(m, e, env, ctx)?;
            if let Value::Hole(h) = v {
                return Ok(Err(h));
            }
            let year = span_year(&v).map(Value::Period);
            ends.push(Some(year.unwrap_or(v)));
        }
        let [from, to] = <[Option<Value>; 2]>::try_from(ends)
            .map_err(|_| invariant("a span with other than two ends"))?;
        let start = from.as_ref().map(|v| span_start(m, v)).transpose()?;
        if !sp.dots {
            let end = from.as_ref().map(|v| span_end(m, v)).transpose()?;
            return Ok(Ok(Span::between(start, end)));
        }
        let end = to.as_ref().map(|v| span_end(m, v)).transpose()?;
        Ok(Ok(Span::between(start, end)))
    }
}

/// When a binding over a kind tests its things: on a day, or over a span.
#[derive(Debug, Clone, PartialEq, Eq)]
enum When {
    /// `on d`.
    Day(Day),
    /// `over s`.
    Over(Span),
}

/// The domain a bound variable is a key of.
pub(super) fn bind_domain(m: &Reality, bind: &Bind) -> Option<Domain> {
    let Over::Name { id, qual: None } = &bind.domain else {
        return None;
    };
    if m.indexes.get(id).is_some() {
        return Some(Domain::Index(id.clone()));
    }
    m.kinds.get(id).map(|_| Domain::Kind(id.clone()))
}

/// The keys of an index the cut sees, in its order (refcore's `keys_at`). L §4.5, §2.5.
fn index_values(m: &Reality, ix: &IndexInfo, cut: Day) -> Result<Vec<(Value, Option<Hole>)>, Stop> {
    let mut out = Vec::with_capacity(ix.keys.len());
    for key in &ix.keys {
        if let Some(version) = ix.added_by(key) {
            let known = m
                .versions
                .get(version)
                .map(|v| v.known)
                .ok_or_else(|| invariant("a key added by a version load didn't build"))?;
            if known > cut {
                continue;
            }
        }
        out.push((Value::of_key(key), None));
    }
    Ok(out)
}

/// A written key list's values: the keys of the index its key positions fixed, else
/// integers, days and periods as such and other keys as literals their use resolves
/// (refcore's `bind_values` for a list, its `b._var.fixed`). L §2.5.
///
/// # Errors
///
/// `NotBuilt` for a key the index lacks, which load refuses first.
fn list_values(
    m: &Reality,
    lits: &[Lit],
    fixed: Option<&String>,
) -> Result<Vec<(Value, Option<Hole>)>, Stop> {
    let mut out = Vec::with_capacity(lits.len());
    for lit in lits {
        let value = match fixed {
            Some(ix) => {
                let key =
                    literal_key(m, lit, &Domain::Index(ix.clone()))?.ok_or_else(lacks_at_run)?;
                Value::of_key(&key)
            }
            None => list_value(lit),
        };
        out.push((value, None));
    }
    Ok(out)
}

/// One written key's value (refcore's `bind_values` for a list): an integer of any size, read
/// exactly. L §2.5, §7.1, §7.6.
fn list_value(lit: &Lit) -> Value {
    let word = Value::Word(lit.text());
    match lit {
        Lit::Num(n) if n.whole && n.unit.is_none() => Value::Number(Number {
            q: n.value.clone(),
            class: Class::Int,
        }),
        Lit::Day(d) => Value::Day(*d),
        Lit::Month(y, mo) => Period::month(*y, *mo).map_or(word, Value::Period),
        Lit::Quarter(y, q) => Period::quarter(*y, *q).map_or(word, Value::Period),
        Lit::Num(_)
        | Lit::MonthDay(..)
        | Lit::Key(_)
        | Lit::Name(_)
        | Lit::Cur(_)
        | Lit::Bool(_)
        | Lit::Nominal(_)
        | Lit::Inf { .. } => word,
    }
}

/// A binding's day: a day, or a day period's (refcore's `date_of`). L §5.7.
pub(super) fn day_of_binding(v: &Value) -> Result<Day, Stop> {
    if let Value::Day(d) = v {
        return a_day(*d);
    }
    if let Value::Period(p) = v
        && p.kind() == crate::time::PeriodKind::Day
    {
        return Ok(p.start());
    }
    if *v == Value::None {
        return Err(refuse(
            Run::None,
            "a value is none here: the day of a binding".to_owned(),
        ));
    }
    Err(not_typed())
}

/// A span's first day: a day, a period's start (refcore's `span_start`). L §5.7.
fn span_start(m: &Reality, v: &Value) -> Result<Day, Stop> {
    if let Value::Day(d) = v {
        return Ok(*d);
    }
    if let Value::Period(p) = v {
        return Ok(p.start());
    }
    Err(wrong_type(m, "a span end is", v))
}

/// The first day after a span's end: the day after a day, a period's end (refcore's
/// `span_end`); the end of time is no day to end on, `RUN-LIMIT`. L §5.7, §7.6 item 1.
fn span_end(m: &Reality, v: &Value) -> Result<Day, Stop> {
    if let Value::Day(d) = v {
        return a_day(*d)?
            .end_days(1)
            .ok_or_else(|| invariant("a day past the end of time"));
    }
    if let Value::Period(p) = v {
        return Ok(p.end());
    }
    Err(wrong_type(m, "a span end is", v))
}

/// A binding whose day or span is a hole: no members where no thing of the kind could be one,
/// else a domain that is that hole (refcore's `bind_day`, `kind_has_member`). L §5.7.
fn empty_or_unknown(bind: &Bind, things: &[&Thing], h: Hole) -> Result<Values, Stop> {
    if things.is_empty() {
        return Ok(Values::Known(Vec::new()));
    }
    if bind.filter.is_some() {
        return Err(Stop::NotBuilt(
            "a binding whose day is a hole and whose where may rule its members out",
        ));
    }
    Ok(Values::Unknown(h))
}

/// Whether a thing is of the kind on the day or over the span: one of its memberships holds
/// it (refcore's `_alive`). L §5.7.
fn is_alive(m: &Reality, thing: &Thing, kind: &str, when: &When, cut: Day) -> Result<bool, Stop> {
    let subtree = m.subtree(kind);
    let kinds: BTreeSet<&str> = thing
        .entries
        .iter()
        .filter(|e| subtree.contains(&e.kind))
        .map(|e| e.kind.as_str())
        .collect();
    for k in kinds {
        let Some(span) = m.span_in(thing, k, cut) else {
            continue;
        };
        let holds = match when {
            When::Day(d) => span.contains(*d),
            When::Over(s) => span.overlaps(s),
        };
        if !holds {
            continue;
        }
        if span.end_question().is_some() {
            return Err(Stop::NotBuilt(
                "a member whose lifespan ends at a stated hole",
            ));
        }
        return Ok(true);
    }
    Ok(false)
}

/// A thing's place in binding order: the text of its stable id `kind:key`, by code point
/// (refcore's `kind_order_key`). L §2.5.
pub(crate) fn order_text(t: &Thing) -> String {
    format!("{}:{}", t.base.as_deref().map_or("None", |b| b), t.name)
}

/// Whether expression `e` may read the name `name`: the parts an aggregate's day or span
/// writes are walked; a part S doesn't build counts as a read.
pub(crate) fn reads_name(e: &Expr, name: &str) -> bool {
    let mut work: Vec<&Expr> = vec![e];
    while let Some(x) = work.pop() {
        match x {
            Expr::Name(n) if n == name => return true,
            Expr::Lit(_)
            | Expr::Str(_)
            | Expr::Name(_)
            | Expr::None
            | Expr::Never
            | Expr::Cut
            | Expr::List(_) => {}
            Expr::If(parts) => work.extend(parts.iter()),
            Expr::Or(terms) | Expr::And(terms) => work.extend(terms.iter()),
            Expr::Not(y) | Expr::Neg(y) | Expr::Paren(y) => work.push(y),
            Expr::Cmp(c) => work.extend([&c.0, &c.2]),
            Expr::In(i) => {
                work.extend([&i.member, &i.set]);
                work.extend(i.on.iter());
            }
            Expr::Sum(first, rest) => {
                work.push(first);
                work.extend(rest.iter().map(|(_, y)| y));
            }
            Expr::Prod(first, rest) => {
                work.push(first);
                work.extend(rest.iter().map(|(_, y)| y));
            }
            Expr::Post(base, posts) => {
                work.push(base);
                work.extend(posts.iter().flat_map(post_exprs));
            }
            Expr::Call(call) => {
                work.extend(call.args.iter());
                if let Some(Key::Expr(k)) = &call.band_key {
                    work.push(k);
                }
            }
            Expr::Record(fields) => work.extend(fields.iter().filter_map(field_expr)),
            Expr::Agg(_)
            | Expr::LegOf(..)
            | Expr::To(..)
            | Expr::Last(_)
            | Expr::Bal(_)
            | Expr::Flows(_)
            | Expr::Covered(_)
            | Expr::Calendar(_)
            | Expr::Value(_)
            | Expr::Query(_)
            | Expr::Low(_)
            | Expr::Crps(_)
            | Expr::DistFn(_)
            | Expr::Dist(_)
            | Expr::Prorata(_)
            | Expr::AsIf(..) => return true,
        }
    }
    false
}

/// The key expressions of a postfix.
fn post_exprs(post: &Postfix) -> Vec<&Expr> {
    match post {
        Postfix::Keys(keys) => keys
            .iter()
            .filter_map(|k| if let Key::Expr(e) = k { Some(e) } else { None })
            .collect(),
        Postfix::Field(_) => Vec::new(),
    }
}

/// A record field's expression, if it has one.
fn field_expr(f: &Field) -> Option<&Expr> {
    if let Field::Named(_, FieldValue::Expr(e)) = f {
        return Some(e);
    }
    None
}

/// Whether a span as written may read the name `name`.
pub(super) fn span_reads(sp: &SpanText, name: &str) -> bool {
    sp.from.as_ref().is_some_and(|e| reads_name(e, name))
        || sp.to.as_ref().is_some_and(|e| reads_name(e, name))
}

#[cfg(test)]
mod tests {
    use super::{order_text, reads_name};
    use crate::model::Thing;
    use crate::syntax::Expr;

    #[test]
    fn things_bind_by_kind_and_key_text() {
        let mut t = Thing::new("anna");
        t.base = Some("person".to_owned());
        assert_eq!(order_text(&t), "person:anna", "kind:key");
    }

    #[test]
    fn a_span_reads_its_variable() {
        let e = Expr::Name("c".to_owned());
        assert!(reads_name(&e, "c"), "the variable");
        assert!(!reads_name(&e, "d"), "another name");
        assert!(!reads_name(&Expr::Cut, "c"), "cut");
    }
}
