//! Aggregates `sum`, `count`, `min`, `max`, `any`, `all` over bindings: the binding tuples in binding order, the unknown members a hole `where` makes, and each aggregate's rule for holes.
//! Spec: L §5.11 (aggregates; a key whose `where` is a hole is an unknown member; `sum` over
//! no keys is 0 in the term's type; `min`/`max` over no keys is `RUN-EMPTY`), §2.5 (binding
//! order: an index's order, a kind's things by the text of `kind:key`), §5.7 (a binding over
//! a kind `on d` or `over s`), §4.5 (the keys and things a cut sees), §8.1 (holes spread).
//! Twin: engine/tools/refcore/evaluate.py (`members`, `bind_values`, `range_values`,
//! `span_value`, `zero_of`, `e_agg`, `agg_minmax`, `agg_anyall`) and model.py (`alive_on`,
//! `alive_over`, `kind_order_key`).
//! Invariants: tuples come in binding order, depth first as refcore's generator; an unknown
//! member's term counts only as its questions, except a 0 in a sum; a domain that is a hole
//! makes the aggregate that hole; a binding over movements or an observed input takes its
//! members from the book (`book/members.rs`), and a member's register reaches the reader only
//! when the member passes the `where`.

use core::cmp::Ordering;
use std::collections::BTreeSet;

use super::arith::{Op, Test, arith, compare, int_of};
use super::expr::hole_of;
use super::state::{Ctx, Engine, Env, invariant, refuse, refuse_load, wrong_type};
use super::ty::zero_of;
use crate::base::{Load, Run};
use crate::hole::{Hole, spread};
use crate::model::{Domain, IndexInfo, KeyVal, Reality, Span, Thing, Value};
use crate::syntax::{
    Agg, AggFn, Bind, Domain as Over, Expr, Field, FieldValue, Key, Lit, Postfix, Span as SpanText,
    Stop,
};
use crate::time::{Day, Period};

/// The most keys a stepped range binds: past it the run stops `NotBuilt` (refcore's size guard;
/// L §5.6 sets no cap).
const RANGE_LIMIT: usize = 1_000_000;

/// A binding tuple: its scope, and the questions that make it an unknown member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    /// The scope with the tuple's variables bound.
    pub env: Env,
    /// None for a member; the hole of a `where` (or a membership) that is one.
    pub unknown: Option<Hole>,
}

/// One binding's values, each with its own unknown-member hole, or the hole of a domain that
/// is unknown.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Values {
    /// The values, in binding order.
    Known(Vec<(Value, Option<Hole>)>),
    /// The domain is a hole.
    Unknown(Hole),
    /// Movements, each with what it brings to a reader that keeps it.
    Moves(Vec<crate::book::MvMember>),
}

/// The frame of a binding over movements: each member's value and hole, and what it
/// brings, kept by position (refcore's `members`, `_colls`). L §5.19 Bindings.
fn moves_frame(
    level: usize,
    env: Env,
    unknown: Option<Hole>,
    moves: Vec<crate::book::MvMember>,
) -> Frame {
    let mut values = Vec::with_capacity(moves.len());
    let mut colls = Vec::with_capacity(moves.len());
    for mv in moves {
        values.push((mv.value, mv.unknown));
        colls.push(mv.coll);
    }
    Frame {
        level,
        env,
        unknown,
        values,
        next: 0,
        colls,
    }
}

/// A frame of the walk over the bindings: the binding's values and the next one to take.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Frame {
    /// The binding's position.
    level: usize,
    /// The scope before this binding.
    env: Env,
    /// The unknown-member hole so far.
    unknown: Option<Hole>,
    /// The values.
    values: Vec<(Value, Option<Hole>)>,
    /// The next value's position.
    next: usize,
    /// What each value brings to a reader that keeps it: a movement's register.
    colls: Vec<crate::eval::Collected>,
}

impl Engine {
    /// An aggregate (refcore's `e_agg`). L §5.11.
    pub(super) fn aggregate(
        &mut self,
        m: &Reality,
        agg: &Agg,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        match agg.func {
            AggFn::First | AggFn::Last | AggFn::Argmin | AggFn::Argmax => {
                return Err(Stop::NotBuilt("first, last, argmin and argmax"));
            }
            AggFn::Sum | AggFn::Count | AggFn::Min | AggFn::Max | AggFn::Any | AggFn::All => {}
        }
        let members = match self.members(m, &agg.binds, env, ctx)? {
            Err(hole) => return Ok(Value::Hole(hole)),
            Ok(members) => members,
        };
        if let (AggFn::Count, None) = (agg.func, &agg.term) {
            return count_members(&members);
        }
        if let AggFn::Any | AggFn::All = agg.func {
            let term = agg
                .term
                .as_ref()
                .ok_or_else(|| invariant("any or all with no term"))?;
            return self.any_all(m, agg.func, term, &members, ctx);
        }
        self.terms_aggregate(m, agg, &members, env, ctx)
    }

    /// `count`, `sum`, `min`, `max` over their terms (refcore's `e_agg`). L §5.11.
    fn terms_aggregate(
        &mut self,
        m: &Reality,
        agg: &Agg,
        members: &[Member],
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let word = agg_word(agg.func);
        let mut terms: Vec<Value> = Vec::with_capacity(members.len());
        let mut holes: Vec<Hole> = Vec::new();
        for member in members {
            let t = match &agg.term {
                None => bare_key(agg, &member.env)?,
                Some(term) => self.ev(m, term, &member.env, ctx)?,
            };
            if let Some(unknown) = &member.unknown {
                if agg.func == AggFn::Sum && is_zero(&t) {
                    continue;
                }
                holes.push(unknown.clone());
                if let Value::Hole(h) = t {
                    holes.push(h);
                }
                continue;
            }
            if let Value::Hole(h) = t {
                holes.push(h);
                continue;
            }
            if t == Value::None {
                return Err(refuse(Run::None, format!("a term of {word} is none here")));
            }
            terms.push(t);
        }
        if !holes.is_empty() {
            let operands: Vec<Option<&Hole>> = holes.iter().map(Some).collect();
            return hole_of(&operands);
        }
        if agg.func == AggFn::Count {
            let n = terms.iter().filter(|t| **t == Value::Bool(true)).count();
            return Ok(Value::int(
                i64::try_from(n).map_err(|_| invariant("a count past the integers"))?,
            ));
        }
        if agg.func == AggFn::Sum {
            return sum_terms(m, agg, terms, env, ctx);
        }
        extreme_term(m, agg.func, terms)
    }

    /// `any(c for …)`, `all(c for …)` under Kleene logic (refcore's `agg_anyall`). L §5.11.
    fn any_all(
        &mut self,
        m: &Reality,
        func: AggFn,
        term: &Expr,
        members: &[Member],
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let word = agg_word(func);
        let decides = func == AggFn::Any;
        let mut holes: Vec<Hole> = Vec::new();
        for member in members {
            let t = self.ev(m, term, &member.env, ctx)?;
            if t == Value::None {
                return Err(refuse(Run::None, format!("a term of {word} is none here")));
            }
            if let Some(unknown) = &member.unknown {
                holes.push(unknown.clone());
                if let Value::Hole(h) = t {
                    holes.push(h);
                }
                continue;
            }
            if let Value::Hole(h) = t {
                holes.push(h);
                continue;
            }
            let Value::Bool(holds) = t else {
                return Err(wrong_type(m, &format!("{word} of"), &t));
            };
            if holds == decides {
                return Ok(Value::Bool(decides));
            }
        }
        if !holes.is_empty() {
            let operands: Vec<Option<&Hole>> = holes.iter().map(Some).collect();
            return hole_of(&operands);
        }
        Ok(Value::Bool(!decides))
    }

    /// The binding tuples in binding order, depth first; the first domain that is a hole
    /// makes the aggregate that hole (refcore's `members`). L §5.11.
    pub(crate) fn members(
        &mut self,
        m: &Reality,
        binds: &[Bind],
        env: &Env,
        ctx: Ctx,
    ) -> Result<Result<Vec<Member>, Hole>, Stop> {
        let Some(first) = binds.first() else {
            return Err(invariant("an aggregate with no binding"));
        };
        let mut out: Vec<Member> = Vec::new();
        let mut domain_hole: Option<Hole> = None;
        let mut frames: Vec<Frame> = Vec::new();
        match self.bind_values(m, first, env, ctx)? {
            Values::Unknown(h) => return Ok(Err(h)),
            Values::Known(values) => frames.push(Frame {
                level: 0,
                env: env.clone(),
                unknown: None,
                values,
                next: 0,
                colls: Vec::new(),
            }),
            Values::Moves(moves) => frames.push(moves_frame(0, env.clone(), None, moves)),
        }
        while let Some(frame) = frames.last_mut() {
            let Some((value, own)) = frame.values.get(frame.next).cloned() else {
                frames.pop();
                continue;
            };
            let brings = frame.colls.get(frame.next).cloned();
            frame.next = frame
                .next
                .checked_add(1)
                .ok_or_else(|| invariant("a binding's position"))?;
            let (level, scope, unknown) = (frame.level, frame.env.clone(), frame.unknown.clone());
            let bind = binds
                .get(level)
                .ok_or_else(|| invariant("a binding past the list"))?;
            let Some((inner, unknown)) =
                self.take_value(m, bind, &scope, value, joined(unknown, own), ctx)?
            else {
                continue;
            };
            if let Some(c) = &brings {
                self.absorb(c);
            }
            let next = level
                .checked_add(1)
                .ok_or_else(|| invariant("a binding's position"))?;
            let Some(bind_next) = binds.get(next) else {
                out.push(Member {
                    env: inner,
                    unknown,
                });
                continue;
            };
            match self.bind_values(m, bind_next, &inner, ctx)? {
                Values::Unknown(h) => {
                    if domain_hole.is_none() {
                        domain_hole = Some(h);
                    }
                }
                Values::Known(values) => frames.push(Frame {
                    level: next,
                    env: inner,
                    unknown,
                    values,
                    next: 0,
                    colls: Vec::new(),
                }),
                Values::Moves(moves) => frames.push(moves_frame(next, inner, unknown, moves)),
            }
        }
        Ok(domain_hole.map_or(Ok(out), Err))
    }

    /// A value of binding `bind` bound, its `where` tested: none when the `where` is false
    /// (refcore's `members`, one value). L §5.11.
    fn take_value(
        &mut self,
        m: &Reality,
        bind: &Bind,
        scope: &Env,
        value: Value,
        unknown: Option<Hole>,
        ctx: Ctx,
    ) -> Result<Option<(Env, Option<Hole>)>, Stop> {
        let inner = scope.with(&bind.var, value, bind_domain(m, bind));
        let Some(filter) = &bind.filter else {
            return Ok(Some((inner, unknown)));
        };
        let tested = self.ev(m, filter, &inner, ctx)?;
        if let Value::Hole(h) = tested {
            return Ok(Some((inner, joined(unknown, Some(h)))));
        }
        if tested == Value::None {
            return Err(refuse(Run::None, "a where is none here".to_owned()));
        }
        let Value::Bool(holds) = tested else {
            return Err(wrong_type(m, "a where of", &tested));
        };
        Ok(holds.then_some((inner, unknown)))
    }

    /// The values one binding ranges over in scope `env` (refcore's `bind_values`). L §5.11,
    /// §2.5, §5.7.
    fn bind_values(
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
            Over::List(lits) => Ok(Values::Known(list_values(lits))),
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
                When::Day(day_of_binding(m, &day)?)
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
            ends.push(Some(v));
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

    /// The values of a range `a .. b [step s]`: days stepped, or integers (refcore's
    /// `range_values`). L §5.11.
    fn range_values(
        &mut self,
        m: &Reality,
        parts: &(Expr, Expr, Option<Expr>),
        env: &Env,
        ctx: Ctx,
    ) -> Result<Values, Stop> {
        let (from, to, step) = parts;
        let first = self.ev(m, from, env, ctx)?;
        let last = self.ev(m, to, env, ctx)?;
        let by = match step {
            None => None,
            Some(e) => Some(self.ev(m, e, env, ctx)?),
        };
        let holes = [
            first.as_hole(),
            last.as_hole(),
            by.as_ref().and_then(Value::as_hole),
        ];
        if let Some(h) = spread(&holes) {
            return Ok(Values::Unknown(h));
        }
        if let Value::Day(start) = first {
            let Some(by) = by else {
                return Err(Stop::NotBuilt("a range of days with no step"));
            };
            return day_range(m, start, &last, &by).map(Values::Known);
        }
        for v in [&first, &last] {
            if *v == Value::None {
                return Err(refuse(
                    Run::None,
                    "a value is none here: a range end".to_owned(),
                ));
            }
        }
        if by.is_some() {
            return Err(Stop::NotBuilt("a stepped range of numbers"));
        }
        let (lo, hi) = (int_of(m, &first)?, int_of(m, &last)?);
        let mut out = Vec::new();
        let mut k = lo;
        while k <= hi {
            if out.len() >= RANGE_LIMIT {
                return Err(Stop::NotBuilt("a range of more than a million keys"));
            }
            out.push((Value::int(k), None));
            k = k
                .checked_add(1)
                .ok_or_else(|| invariant("a range past the integers"))?;
        }
        Ok(Values::Known(out))
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

/// The word of an aggregate, as messages name it.
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

/// `count(k in ks)`: the tuples, unless one is an unknown member (L §5.11).
fn count_members(members: &[Member]) -> Result<Value, Stop> {
    let holes: Vec<Option<&Hole>> = members.iter().map(|mm| mm.unknown.as_ref()).collect();
    if let Some(h) = spread(&holes) {
        return Ok(Value::Hole(h));
    }
    let n = i64::try_from(members.len()).map_err(|_| invariant("a count past the integers"))?;
    Ok(Value::int(n))
}

/// A bare binding's term: its one variable's value (refcore's `bare_key`).
fn bare_key(agg: &Agg, env: &Env) -> Result<Value, Stop> {
    let [bind] = agg.binds.as_slice() else {
        return Err(Stop::NotBuilt("a bare aggregate over a tuple of keys"));
    };
    env.get(&bind.var)
        .map(|v| v.value.clone())
        .ok_or_else(|| invariant("a bound variable the scope lacks"))
}

/// Whether a term is zero: an unknown member's 0 adds nothing to a sum (L §5.11).
fn is_zero(v: &Value) -> bool {
    if let Value::Number(n) = v {
        return n.q.is_zero();
    }
    if let Value::Money(money) = v {
        return money.is_zero();
    }
    false
}

/// The sum of known terms, left to right; 0 in the term's type over none (refcore's
/// `zero_of`). L §5.11.
fn sum_terms(
    m: &Reality,
    agg: &Agg,
    terms: Vec<Value>,
    env: &Env,
    ctx: Ctx,
) -> Result<Value, Stop> {
    let mut it = terms.into_iter();
    let Some(first) = it.next() else {
        let term = agg
            .term
            .as_ref()
            .ok_or_else(|| invariant("a sum with no term"))?;
        return zero_of(m, term, env, &agg.binds, ctx);
    };
    let mut acc = first;
    for t in it {
        acc = arith(m, Op::Add, &acc, &t)?;
    }
    Ok(acc)
}

/// `min` or `max` of known terms; `RUN-EMPTY` over none (refcore's `agg_minmax`). L §5.11.
fn extreme_term(m: &Reality, func: AggFn, terms: Vec<Value>) -> Result<Value, Stop> {
    let mut it = terms.into_iter();
    let Some(mut best) = it.next() else {
        return Err(refuse(
            Run::Empty,
            format!("{} over no keys", agg_word(func)),
        ));
    };
    let wanted = if func == AggFn::Min {
        Ordering::Less
    } else {
        Ordering::Greater
    };
    for t in it {
        if compare(m, &t, &best, Test::Order)? == wanted {
            best = t;
        }
    }
    Ok(best)
}

/// Two unknown-member holes as one.
fn joined(a: Option<Hole>, b: Option<Hole>) -> Option<Hole> {
    match (a, b) {
        (Some(x), Some(y)) => spread(&[Some(&x), Some(&y)]),
        (x, None) => x,
        (None, y) => y,
    }
}

/// The domain a bound variable is a key of.
fn bind_domain(m: &Reality, bind: &Bind) -> Option<Domain> {
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

/// A written key list's values: integers, days and periods as such, other keys as literals
/// their use resolves (refcore's `bind_values` for a list). L §2.5.
fn list_values(lits: &[Lit]) -> Vec<(Value, Option<Hole>)> {
    lits.iter().map(|lit| (list_value(lit), None)).collect()
}

/// One written key's value (refcore's `bind_values` for a list). L §2.5.
fn list_value(lit: &Lit) -> Value {
    let word = Value::Word(lit.text());
    match lit {
        Lit::Num(n) if n.whole && n.unit.is_none() => n.value.to_i64().map_or(word, Value::int),
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

/// The days from `start` to `end`, stepped (refcore's `range_values` for days). L §5.11.
fn day_range(
    m: &Reality,
    start: Day,
    end: &Value,
    step: &Value,
) -> Result<Vec<(Value, Option<Hole>)>, Stop> {
    let last = day_of_binding(m, end)?;
    let mut out = Vec::new();
    let mut current = start;
    while current <= last {
        if out.len() >= RANGE_LIMIT {
            return Err(Stop::NotBuilt("a range of more than a million keys"));
        }
        out.push((Value::Day(current), None));
        let Value::Day(next) = arith(m, Op::Add, &Value::Day(current), step)? else {
            return Err(refuse_load(
                Load::Unit,
                "a range of days stepped by a value that isn't a duration".to_owned(),
            ));
        };
        if next <= current {
            return Err(refuse(
                Run::Domain,
                "a stepped range whose step is not positive".to_owned(),
            ));
        }
        current = next;
    }
    Ok(out)
}

/// A binding's day: a day, or a day period's (refcore's `date_of`). L §5.7.
fn day_of_binding(m: &Reality, v: &Value) -> Result<Day, Stop> {
    if let Value::Day(d) = v {
        return Ok(*d);
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
    Err(wrong_type(m, "the day of a binding is", v))
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
/// `span_end`). L §5.7.
fn span_end(m: &Reality, v: &Value) -> Result<Day, Stop> {
    if let Value::Day(d) = v {
        return d
            .next()
            .ok_or_else(|| invariant("a day past the calendar's range"));
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
            | Expr::Value(_) => return true,
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
fn span_reads(sp: &SpanText, name: &str) -> bool {
    sp.from.as_ref().is_some_and(|e| reads_name(e, name))
        || sp.to.as_ref().is_some_and(|e| reads_name(e, name))
}

#[cfg(test)]
mod tests {
    use super::{joined, order_text, reads_name};
    use crate::hole::Hole;
    use crate::model::Thing;
    use crate::syntax::Expr;

    #[test]
    fn unknown_members_join_their_questions() {
        let a = Hole::asked("a?", "x");
        let b = Hole::asked("b?", "y");
        let both = joined(Some(a.clone()), Some(b.clone())).expect("a hole");
        let other_way = joined(Some(b), Some(a.clone())).expect("a hole");
        assert!(both.same_questions(&other_way), "joined in either order");
        assert_eq!(
            both.question_texts(),
            vec!["a?".to_owned(), "b?".to_owned()],
            "both questions"
        );
        assert_eq!(joined(Some(a.clone()), None), Some(a), "one side");
        assert_eq!(joined(None, None), None, "a member");
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
