//! The values a binding ranges over: an index's keys the cut sees, a key list's, a kind's things alive on a day or over a span.
//! Spec: L §2.5 (binding order: an index's order, a kind's things by the text of `kind:key`;
//! a key list's literals as keys of the index its key positions fix), §5.7 (a binding over a
//! kind `on d` or `over s`, alive by its memberships), §5.6 (an index's keys), §4.5 (the keys
//! and things a cut sees), §6.4 (and the world: a scenario's keys and things only in its
//! world and those over it), §7.6 item 1 (the end of time is no day to end a span on). The
//! aggregates over these values are `aggregate.rs`'s; a stepped range's keys `ranges.rs`'s;
//! movements `book/members.rs`'s.
//! Twin: engine/tools/refcore/evaluate.py (`bind_values`, `span_value`, `span_start`,
//! `span_end`, `bind_day`, `kind_has_member`) and model.py (`alive_on`, `alive_over`,
//! `kind_order_key`, `keys_at`).
//! Invariants: values come in binding order; a day or span that is a hole makes the domain
//! that hole, unless no thing of the kind could be a member; a key a version added after the
//! cut is no value.

use super::keys::{lacks_at_run, literal_key};
use super::state::{Ctx, Engine, Env, a_day, invariant, wrong_type};
use super::windows::{imprecise, one_day};
use super::years::span_year;
use crate::hole::Hole;
use crate::model::{Class, Domain, IndexInfo, KeyVal, Number, Reality, Span, Thing, Value, World};
use crate::syntax::{
    Bind, Domain as Over, Expr, Field, FieldValue, Key, Lit, OfQ, Pin, PinForm, Postfix, Prorata,
    RChoice, Span as SpanText, Step, Stop,
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
                    return index_values(m, ix, (ctx.cut, ctx.world)).map(Values::Known);
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
            .filter(|t| {
                super::keys::thing_seen(m, t, (ctx.cut, ctx.world)) && m.thing_in(&t.name, kind)
            })
            .collect();
        let when = match (&bind.on, &bind.over) {
            (Some(on), _) => {
                let day = self.ev(m, on, env, ctx)?;
                if let Value::Hole(h) = day {
                    return self.empty_or_unknown(m, (bind, env), &things, h, ctx);
                }
                match one_day(m, &day, "the day of a binding")? {
                    Ok(day_on) => When::Day(day_on),
                    Err(h) => return self.empty_or_unknown(m, (bind, env), &things, h, ctx),
                }
            }
            (None, Some(over)) => match self.span_value(m, over, env, ctx)? {
                Err(h) => return self.empty_or_unknown(m, (bind, env), &things, h, ctx),
                Ok(span) => When::Over(span),
            },
            (None, None) => {
                return Err(Stop::NotBuilt("a binding over a kind with no day or span"));
            }
        };
        let mut alive: Vec<&Thing> = Vec::with_capacity(things.len());
        for thing in things {
            self.note_thing(m, thing, ctx.world, super::state::ThingRead::Member);
            if is_alive(m, thing, kind, &when, (ctx.cut, ctx.world))? {
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
        // one day or period alone is the span from its start to its end
        let last = if sp.dots { to.as_ref() } else { from.as_ref() };
        let start = match from
            .as_ref()
            .map(|v| span_start(m, v))
            .transpose()?
            .transpose()
        {
            Err(h) => return Ok(Err(h)),
            Ok(start) => start,
        };
        let end = match last.map(|v| span_end(m, v)).transpose()?.transpose() {
            Err(h) => return Ok(Err(h)),
            Ok(end) => end,
        };
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

/// The keys of an index the cut and the world of `seen` see, in its order (refcore's
/// `keys_at`, `IndexInfo.visible`). L §4.5, §2.5, §6.4.
fn index_values(
    m: &Reality,
    ix: &IndexInfo,
    (cut, world): (Day, World),
) -> Result<Vec<(Value, Option<Hole>)>, Stop> {
    let mut out = Vec::with_capacity(ix.keys().len());
    for key in ix.keys() {
        if let Some(version) = ix.added_by(key) {
            let known = m
                .versions
                .get(version)
                .map(|v| v.known)
                .ok_or_else(|| invariant("a key added by a version load didn't build"))?;
            if known > cut || !m.scenarios.sees(world, Some(version)) {
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
/// `ENGINE-INVARIANT` for a key the index lacks, which load refuses first (`LOAD-KEY`).
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

/// A span's first day: a day, a period's start; a window is its `HOLE-IMPRECISE` for a span
/// end (refcore's `span_start`). L §5.7, §4.4.
fn span_start(m: &Reality, v: &Value) -> Result<Result<Day, Hole>, Stop> {
    if let Value::Day(d) = v {
        return Ok(Ok(*d));
    }
    if let Value::Period(p) = v {
        return Ok(Ok(p.start()));
    }
    if let Value::Window(w) = v {
        return Ok(Err(imprecise(w, "a span end")));
    }
    Err(wrong_type(m, "a span end is", v))
}

/// The first day after a span's end: the day after a day, a period's end; a window is its
/// `HOLE-IMPRECISE` for a span end (refcore's `span_end`); the end of time is no day to end on,
/// `RUN-LIMIT`. L §5.7, §4.4, §7.6 item 1.
fn span_end(m: &Reality, v: &Value) -> Result<Result<Day, Hole>, Stop> {
    if let Value::Day(d) = v {
        return a_day(*d)?
            .end_days(1)
            .map(Ok)
            .ok_or_else(|| invariant("a day past the end of time"));
    }
    if let Value::Period(p) = v {
        return Ok(Ok(p.end()));
    }
    if let Value::Window(w) = v {
        return Ok(Err(imprecise(w, "a span end")));
    }
    Err(wrong_type(m, "a span end is", v))
}

impl Engine {
    /// A binding over a kind whose day or span is a hole: no members where no thing of the
    /// kind could be one, else a domain that is that hole (refcore's `bind_day`,
    /// `kind_has_member`). L §5.7, §8.1.
    fn empty_or_unknown(
        &mut self,
        m: &Reality,
        (bind, env): (&Bind, &Env),
        things: &[&Thing],
        h: Hole,
        ctx: Ctx,
    ) -> Result<Values, Stop> {
        let cands = things
            .iter()
            .map(|t| Value::Key(KeyVal::Thing(t.name.clone())))
            .collect();
        Ok(if self.may_hold(m, (bind, env), cands, ctx)? {
            Values::Unknown(h)
        } else {
            Values::Known(Vec::new())
        })
    }

    /// Whether binding `bind`, its day or span a hole, could hold one of the candidates on some
    /// day: one its `where` doesn't rule out. A member whose `where` is false is outside the
    /// binding whatever its day, so the domain is decided without the day; a `where` that is a
    /// hole or none leaves the candidate in. What the `where` reads reaches no reader
    /// (refcore's `kind_has_member`). L §5.7 Bindings, §8.1 ("It stops propagating").
    ///
    /// # Errors
    ///
    /// A refusal a `where` meets: it is read on the hole day too, so its refusal reaches the
    /// binding and outranks the day's hole (L §8, R1 and Refusals outrank holes; tally 2.1.8,
    /// #180); any other stop it meets.
    pub(crate) fn may_hold(
        &mut self,
        m: &Reality,
        (bind, env): (&Bind, &Env),
        cands: Vec<Value>,
        ctx: Ctx,
    ) -> Result<bool, Stop> {
        let Some(filter) = &bind.filter else {
            return Ok(!cands.is_empty());
        };
        let written = self.bind_text(&bind.text);
        for value in cands {
            let inner = env.with_bind((&bind.var, &written), value, bind_domain(m, bind));
            self.push();
            let tested = self.ev(m, filter, &inner, ctx);
            self.pop_alone();
            match tested {
                Ok(Value::Bool(false)) => {}
                Ok(_) => return Ok(true),
                Err(stop) => return Err(stop),
            }
        }
        Ok(false)
    }
}

/// Whether a thing is of the kind on the day or over the span, in the world of `seen`: one
/// of its memberships holds it (refcore's `_alive`). L §5.7, §6.4.
fn is_alive(
    m: &Reality,
    thing: &Thing,
    kind: &str,
    when: &When,
    seen: (Day, World),
) -> Result<bool, Stop> {
    let subtree = m.subtree(kind);
    let kinds: BTreeSet<&str> = thing
        .entries
        .iter()
        .filter(|e| subtree.contains(&e.kind))
        .map(|e| e.kind.as_str())
        .collect();
    for k in kinds {
        let Some(span) = m.span_in(thing, k, seen) else {
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
            | Expr::Crps(_)
            | Expr::DistFn(_)
            | Expr::Dist(_)
            | Expr::Prorata(_)
            | Expr::AsIf(..) => return true,
        }
    }
    false
}

/// Expression `e` as written, each variable a binding of `env` gave printed as its value, so a
/// message names a thing by its keys' values, never by a bound variable: `rent[2026-02]`, not
/// `rent[m]` (refcore's `env_txt`). The tree's name nodes are swapped, never its text: a
/// period word, a field, a call's name and a string are no name node. L §8.1 ("with the item
/// and keys each came from"), §8.4 (tally 2.2.1's 235.6, 233.7; P48 B1).
///
/// # Errors
///
/// A number error printing a value.
pub(crate) fn written_bound(m: &Reality, e: &Expr, env: &Env) -> Result<String, Stop> {
    Ok(crate::load::bound_text(e, &env.bound_texts(m)?))
}

/// The names expression `e` reads, its parts walked as [`reads_name`] walks them; none for a
/// form whose names aren't walked here (an aggregate, a walk's read, an `as if`), where
/// `reads_name` counts a read. L §8.2 (a pin formed per binding tuple).
pub(crate) fn names_in(e: &Expr) -> Option<Vec<&str>> {
    let mut out: Vec<&str> = Vec::new();
    let mut work: Vec<&Expr> = vec![e];
    while let Some(x) = work.pop() {
        match x {
            Expr::Name(n) => out.push(n.as_str()),
            Expr::Lit(_) | Expr::Str(_) | Expr::None | Expr::Never | Expr::Cut | Expr::List(_) => {}
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
            | Expr::Crps(_)
            | Expr::DistFn(_)
            | Expr::Dist(_)
            | Expr::Prorata(_)
            | Expr::AsIf(..) => return None,
        }
    }
    Some(out)
}

/// The names expression `e` holds, in the order refcore's walk meets them (`node.walk`: a
/// node before its parts, its fields last-written first, a list's items in order, so the
/// walk of `x[a] + y[b]` meets `b` before `a`). L §8.2 (a pin formed per binding tuple
/// prints with the bindings its value and keys name).
pub(crate) fn names_walked(e: &Expr) -> Vec<&str> {
    let mut names: Vec<&str> = Vec::new();
    let mut work: Vec<&Expr> = vec![e];
    while let Some(x) = work.pop() {
        if let Expr::Name(n) = x {
            names.push(n.as_str());
        }
        work.extend(walk_order(x).into_iter().rev());
    }
    names
}

/// The parts of expression `e` in the order refcore's walk visits them, from its syntax
/// tree's fields as `parser.py` writes them, the last field first: an operator chain is
/// left-nested, so its last term first; a read's keys before its base; a call's band key
/// before its arguments; an aggregate's `ties by`, its bindings, then its term; an `as if`'s
/// pins (each its value, then its keys) before its expression; a walk's read its clauses
/// before what it reads (`bal`: its `of`, its days, its place, never its projection, which
/// refcore keeps out of the walk). L §8.2.
fn walk_order(e: &Expr) -> Vec<&Expr> {
    match e {
        Expr::Lit(_)
        | Expr::Str(_)
        | Expr::Name(_)
        | Expr::None
        | Expr::Never
        | Expr::Cut
        | Expr::List(_)
        | Expr::Calendar(_) => Vec::new(),
        Expr::If(parts) => parts.iter().rev().collect(),
        Expr::Or(terms) | Expr::And(terms) => terms.iter().rev().collect(),
        Expr::Not(y) | Expr::Neg(y) | Expr::Paren(y) => vec![y],
        Expr::Cmp(c) => vec![&c.2, &c.0],
        Expr::In(i) => i.on.iter().chain([&i.set, &i.member]).collect(),
        // left-nested chains: the last term first
        Expr::Sum(first, rest) => {
            let mut out: Vec<&Expr> = rest.iter().rev().map(|(_, y)| y).collect();
            out.push(first);
            out
        }
        Expr::Prod(first, rest) => {
            let mut out: Vec<&Expr> = rest.iter().rev().map(|(_, y)| y).collect();
            out.push(first);
            out
        }
        Expr::Post(base, posts) => {
            // a read's keys, its last postfix first, before its base
            let mut out: Vec<&Expr> = posts.iter().rev().flat_map(post_exprs).collect();
            out.push(base);
            out
        }
        Expr::Call(call) => key_expr(call.band_key.as_ref())
            .into_iter()
            .chain(call.args.iter())
            .collect(),
        Expr::To(base, step) => step_expr(step)
            .into_iter()
            .chain(core::iter::once(&**base))
            .collect(),
        Expr::Record(fields) => fields.iter().flat_map(field_order).collect(),
        Expr::Agg(agg) => {
            let mut out: Vec<&Expr> = agg.ties.iter().flatten().collect();
            out.extend(agg.binds.iter().flat_map(bind_order));
            out.extend(agg.term.iter());
            out
        }
        Expr::AsIf(inner, pins) => pins
            .iter()
            .flat_map(pin_order)
            .chain(core::iter::once(&**inner))
            .collect(),
        Expr::LegOf(base, of) => {
            let mut out = of_order(of);
            out.push(base);
            out
        }
        Expr::Last(pair) => vec![&pair.1, &pair.0],
        Expr::Bal(bal) => {
            let mut out: Vec<&Expr> = bal.of.iter().flat_map(of_order).collect();
            out.extend(bal.args.iter());
            out.push(&bal.place);
            out
        }
        Expr::Flows(flows) => {
            let mut out: Vec<&Expr> = flows.for_key.iter().collect();
            out.extend(span_order(&flows.span));
            out.push(&flows.target);
            out
        }
        Expr::Covered(pair) => {
            let mut out: Vec<&Expr> = vec![&pair.0];
            out.extend(span_order(&pair.1));
            out
        }
        Expr::Value(v) => {
            let mut out: Vec<&Expr> = v.of.iter().flat_map(of_order).collect();
            out.extend(v.by.iter().map(|price| &price.input));
            out.extend([&v.day, &v.place]);
            out
        }
        Expr::Query(q) => {
            let mut out: Vec<&Expr> = q.level.iter().chain(q.filter.iter()).collect();
            out.extend(q.binds.iter().flatten().flat_map(bind_order));
            out.push(&q.arg);
            out
        }
        Expr::Crps(c) => {
            let mut out: Vec<&Expr> = vec![&c.outcome];
            out.extend(c.bind.iter().flat_map(bind_order));
            out.extend(c.weight.iter());
            out.push(&c.forecast);
            out
        }
        Expr::DistFn(f) => vec![&f.value, &f.at],
        Expr::Dist(d) => {
            let mut out: Vec<&Expr> = d.bind.iter().flat_map(bind_order).collect();
            out.extend(d.term.iter());
            out.extend(d.args.iter());
            out
        }
        Expr::Prorata(p) => prorata_order(p),
    }
}

/// `prorata(total, w for binds, key, u, mode, rest, tie)`'s parts as refcore's walk visits
/// them: its `ties by`, its mode, its step, its key, its bindings, its weight, its total.
fn prorata_order(p: &Prorata) -> Vec<&Expr> {
    let mut out: Vec<&Expr> = p.ties.iter().flatten().collect();
    out.extend(p.mode.iter());
    out.extend(step_expr(&p.unit));
    out.push(&p.key);
    out.extend(p.binds.iter().flat_map(bind_order));
    out.extend([&p.weight, &p.total]);
    out
}

/// A binding's parts as refcore's walk visits them (`bind(var, dom, on, over, where)`): its
/// `where`, its span's ends (the last first), its `on`, its range's step and ends (the last
/// first).
fn bind_order(bind: &Bind) -> Vec<&Expr> {
    let mut out: Vec<&Expr> = bind.filter.iter().collect();
    out.extend(bind.over.iter().flat_map(span_order));
    out.extend(bind.on.iter());
    if let Over::Range(range) = &bind.domain {
        out.extend(range.2.iter());
        out.extend([&range.1, &range.0]);
    }
    out
}

/// A span's ends as refcore's walk visits them (`span(a, b)`): the last first.
fn span_order(span: &SpanText) -> Vec<&Expr> {
    span.to.iter().chain(span.from.iter()).collect()
}

/// An `of`'s parts as refcore's walk visits them: one read, or a pair's last first
/// (`ofpair(a, b)`).
fn of_order(of: &OfQ) -> Vec<&Expr> {
    match of {
        OfQ::Ref(e) => vec![e],
        OfQ::Pair(a, b) => vec![b, a],
    }
}

/// A record field's parts as refcore's walk visits them: a named field's value; a leg's
/// amount, its `to`, its `from` (`leg(name, frm, to, amount)`).
fn field_order(f: &Field) -> Vec<&Expr> {
    match f {
        Field::Named(_, FieldValue::Expr(e)) => vec![e],
        Field::Named(_, FieldValue::Lit(_) | FieldValue::Hole(_) | FieldValue::None) => Vec::new(),
        Field::Leg(leg) => vec![&leg.amount, &leg.to, &leg.from],
    }
}

/// A pin's parts as refcore's walk visits them (`pin(kind, target, value, sample)`): its
/// value, then its target's keys in order; a date pin's date.
fn pin_order(pin: &Pin) -> Vec<&Expr> {
    match &pin.form {
        PinForm::Law(e) | PinForm::Known(e) | PinForm::Wording(e, _) => vec![e],
        PinForm::Scenario(_) => Vec::new(),
        PinForm::Set { target, value, .. } => core::iter::once(value)
            .chain(
                target
                    .keys
                    .iter()
                    .flatten()
                    .filter_map(|k| key_expr(Some(k))),
            )
            .collect(),
    }
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

/// Whether expression `e` reads the variable `var`, every part of every form walked: an
/// aggregate's term, bindings and `ties by`, a pin's, a span's, a projection's. No binding
/// takes a name in scope (L §2.5), so a name `var` anywhere in `e` is that variable. Load's
/// `LOAD-LAWDATE` on a move's key and the walk's `move_law` both decide by it, so a move whose
/// `at` reads no key has its law date before its keys are bound. L §4.3 (a move's `at` dates
/// what it reads, its keys' among them), §5.14 Keys.
pub(crate) fn reads_var(e: &Expr, var: &str) -> bool {
    let mut work: Vec<&Expr> = vec![e];
    while let Some(x) = work.pop() {
        if matches!(x, Expr::Name(n) if n == var) {
            return true;
        }
        work.extend(expr_parts(x));
    }
    false
}

/// Every expression `e` holds directly, by an exhaustive match, so that no form's part is
/// left unwalked. L §2.3 (the expressions), §5.6 (bindings), §5.11 (pins).
fn expr_parts(e: &Expr) -> Vec<&Expr> {
    let mut out: Vec<&Expr> = Vec::new();
    match e {
        Expr::Lit(_)
        | Expr::Str(_)
        | Expr::Name(_)
        | Expr::None
        | Expr::Never
        | Expr::Cut
        | Expr::List(_)
        | Expr::Calendar(_) => {}
        Expr::If(parts) => out.extend(parts.iter()),
        Expr::Or(terms) | Expr::And(terms) => out.extend(terms.iter()),
        Expr::Not(y) | Expr::Neg(y) | Expr::Paren(y) => out.push(y),
        Expr::Cmp(c) => out.extend([&c.0, &c.2]),
        Expr::In(i) => {
            out.extend([&i.member, &i.set]);
            out.extend(i.on.iter());
        }
        Expr::Sum(first, rest) => {
            out.push(first);
            out.extend(rest.iter().map(|(_, y)| y));
        }
        Expr::Prod(first, rest) => {
            out.push(first);
            out.extend(rest.iter().map(|(_, y)| y));
        }
        Expr::Post(base, posts) => {
            out.push(base);
            out.extend(posts.iter().flat_map(post_exprs));
        }
        Expr::Call(call) => {
            out.extend(call.args.iter());
            out.extend(key_expr(call.band_key.as_ref()));
        }
        Expr::LegOf(base, of) => {
            out.push(base);
            out.extend(of_exprs(of));
        }
        Expr::To(base, step) => {
            out.push(base);
            out.extend(step_expr(step));
        }
        Expr::Record(fields) => out.extend(fields.iter().flat_map(field_exprs)),
        Expr::Agg(agg) => {
            out.extend(agg.term.iter());
            out.extend(agg.binds.iter().flat_map(bind_parts));
            out.extend(agg.ties.iter().flatten());
        }
        Expr::Last(pair) => out.extend([&pair.0, &pair.1]),
        Expr::Bal(bal) => {
            out.push(&bal.place);
            out.extend(bal.args.iter());
            out.extend(bal.proj.from.iter());
            if let Some(RChoice::Quantile(q)) = &bal.proj.random {
                out.push(q);
            }
            out.extend(bal.of.iter().flat_map(of_exprs));
        }
        Expr::Flows(flows) => {
            out.push(&flows.target);
            out.extend(span_parts(&flows.span));
            out.extend(flows.for_key.iter());
        }
        Expr::Covered(pair) => {
            out.push(&pair.0);
            out.extend(span_parts(&pair.1));
        }
        Expr::Value(v) => {
            out.extend([&v.place, &v.day]);
            out.extend(v.by.iter().map(|price| &price.input));
            out.extend(v.of.iter().flat_map(of_exprs));
        }
        Expr::Query(q) => {
            out.push(&q.arg);
            out.extend(q.binds.iter().flatten().flat_map(bind_parts));
            out.extend(q.filter.iter());
            out.extend(q.level.iter());
        }
        Expr::Crps(c) => {
            out.extend([&c.forecast, &c.outcome]);
            out.extend(c.weight.iter());
            out.extend(c.bind.iter().flat_map(bind_parts));
        }
        Expr::DistFn(f) => out.extend([&f.value, &f.at]),
        Expr::Dist(d) => {
            out.extend(d.args.iter());
            out.extend(d.term.iter());
            out.extend(d.bind.iter().flat_map(bind_parts));
        }
        Expr::Prorata(p) => {
            out.extend([&p.total, &p.weight, &p.key]);
            out.extend(p.binds.iter().flat_map(bind_parts));
            out.extend(step_expr(&p.unit));
            out.extend(p.mode.iter());
            out.extend(p.ties.iter().flatten());
        }
        Expr::AsIf(inner, pins) => {
            out.push(inner);
            out.extend(pins.iter().flat_map(pin_exprs));
        }
    }
    out
}

/// A key position's expression, if it is one (not `*`).
fn key_expr(key: Option<&Key>) -> Option<&Expr> {
    if let Some(Key::Expr(e)) = key {
        return Some(e);
    }
    None
}

/// The expressions of an `of`: one read, or a pair.
fn of_exprs(of: &OfQ) -> Vec<&Expr> {
    match of {
        OfQ::Ref(e) => vec![e],
        OfQ::Pair(a, b) => vec![a, b],
    }
}

/// A step's expression, where it is read from one.
fn step_expr(step: &Step) -> Option<&Expr> {
    match step {
        Step::Ref(e) => Some(e),
        Step::Number(_) | Step::Minor(_) => None,
    }
}

/// A record field's expressions: a named field's value, a leg's places and amount.
fn field_exprs(f: &Field) -> Vec<&Expr> {
    match f {
        Field::Named(_, FieldValue::Expr(e)) => vec![e],
        Field::Named(_, FieldValue::Lit(_) | FieldValue::Hole(_) | FieldValue::None) => Vec::new(),
        Field::Leg(leg) => vec![&leg.from, &leg.to, &leg.amount],
    }
}

/// A span's ends.
fn span_parts(span: &SpanText) -> Vec<&Expr> {
    span.from.iter().chain(span.to.iter()).collect()
}

/// A binding's expressions: its range's ends and step, its `on`, its span and its `where`.
fn bind_parts(bind: &Bind) -> Vec<&Expr> {
    let mut out: Vec<&Expr> = Vec::new();
    if let Over::Range(range) = &bind.domain {
        out.extend([&range.0, &range.1]);
        out.extend(range.2.iter());
    }
    out.extend(bind.on.iter());
    out.extend(bind.over.iter().flat_map(span_parts));
    out.extend(bind.filter.iter());
    out
}

/// A pin's expressions: its law, knowledge or wording date, or its target's keys and value.
fn pin_exprs(pin: &Pin) -> Vec<&Expr> {
    match &pin.form {
        PinForm::Law(e) | PinForm::Known(e) | PinForm::Wording(e, _) => vec![e],
        PinForm::Scenario(_) => Vec::new(),
        PinForm::Set { target, value, .. } => {
            let mut out: Vec<&Expr> = target
                .keys
                .iter()
                .flatten()
                .filter_map(|k| key_expr(Some(k)))
                .collect();
            out.push(value);
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{names_walked, order_text, reads_name};
    use crate::model::Thing;
    use crate::syntax::{AddOp, AskBody, Expr, Key, Postfix, Section};

    /// `id[keys]`, each key a name.
    fn read(id: &str, keys: &[&str]) -> Expr {
        let keys = keys
            .iter()
            .map(|k| Key::Expr(Expr::Name((*k).to_owned())))
            .collect();
        Expr::Post(
            Box::new(Expr::Name(id.to_owned())),
            vec![Postfix::Keys(keys)],
        )
    }

    #[test]
    fn names_are_walked_as_refcore_walks_them() {
        // `b2[k, j] + c2[j, m]`: the last term first, a read's keys in order, then its base
        let e = Expr::Sum(
            Box::new(read("b2", &["k", "j"])),
            vec![(AddOp::Add, read("c2", &["j", "m"]))],
        );
        let flat: Vec<&str> = names_walked(&e);
        assert_eq!(flat, ["j", "m", "c2", "k", "j", "b2"], "refcore's order");
        // `if a then b else c`: the else first
        let parts = ["a", "b", "c"].map(|n| Expr::Name(n.to_owned()));
        let choice = Expr::If(Box::new(parts));
        let names: Vec<&str> = names_walked(&choice);
        assert_eq!(names, ["c", "b", "a"], "the if's parts, last first");
    }

    /// The expressions of the asks of model text `text`, parsed, not loaded.
    fn asked(text: &str) -> Vec<Expr> {
        let files = crate::syntax::parse_reality(&[crate::api::Source {
            path: "walk.tally".to_owned(),
            text: text.to_owned(),
        }])
        .expect("parses");
        let mut out = Vec::new();
        for section in files.into_iter().flat_map(|f| f.sections) {
            if let Section::Ask(ask) = section
                && let AskBody::Exprs(exprs) = ask.body
            {
                out.extend(exprs);
            }
        }
        out
    }

    const WALKED: &str = "model walk
  tally 2
  on 2026-10-25
  start 2026-01-01
  end 2026-12-31

ask q: sum(x[i] + y[j] for i in a .. b step c where z[k] > d), (u[k] as if {v[j] = w[i]})
ask r: last(q[j], d), bal(x[k], e), flows(f[k], a .. b, for j)
";

    #[test]
    fn an_aggregate_and_an_as_if_are_walked_as_refcore_walks_them() {
        let exprs = asked(WALKED);
        let walked: Vec<Vec<&str>> = exprs.iter().map(|e| names_walked(e)).collect();
        // the aggregate: its binding's `where` (the right side first), its step and ends,
        // the last first, then its term; the `as if`: the pin's value, its keys, then its
        // expression
        assert_eq!(
            walked,
            [
                vec!["d", "k", "z", "c", "b", "a", "j", "y", "i", "x"],
                vec!["i", "w", "j", "k", "u"],
                vec!["d", "j", "q"],
                vec!["e", "k", "x"],
                vec!["j", "b", "a", "k", "f"],
            ],
            "refcore's order"
        );
    }

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
