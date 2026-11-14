//! Aggregates over keys written as queries: `mean`, `share` and `quantile` with `for`, the exact average, fraction and type-1 quantile of their terms.
//! Spec: L §5.11 (aggregates over keys: `mean(t for b)` the exact average, `share(c for b)` the
//! exact fraction of bindings where c holds, `quantile(t for b, q)` type 1, the smallest term
//! with at least q·n terms at or below it; over no keys `RUN-EMPTY`; a hole if any term or
//! `where` is; a term that is `none` refused, `RUN-NONE`), §8.1 (holes spread), §8.4. Without
//! `for` a query reads a random value (L §5.16, M7's).
//! Twin: engine/tools/refcore/evaluate.py (`e_query`, its arm with `for`).
//! Invariants: the terms are read in binding order, every one, as refcore reads them; a hole
//! where any term or `where` is, its questions in the order met; a `none` term refuses at its
//! member; `mean` adds the terms as `sum` does and divides by their count, exactly; `share`
//! counts the true terms over all of them; `quantile` sorts the terms by value, equals kept
//! in binding order, and takes the first whose rank reaches q·n, the last when none does.

use super::aggregate::Brings;
use super::arith::{Op, Test, arith, compare};
use super::expr::hole_of;
use super::state::{Ctx, Engine, Env, not_typed, num_stop, refuse};
use crate::base::Run;
use crate::hole::Hole;
use crate::model::{Number, Reality, Value};
use crate::num::Num;
use crate::syntax::{Query, QueryFn, Stop};
use core::cmp::Ordering;

impl Engine {
    /// `mean`, `share` or `quantile`: with `for`, an aggregate over its binding tuples
    /// (refcore's `e_query`). L §5.11.
    ///
    /// # Errors
    ///
    /// `RUN-EMPTY` over no keys, `RUN-NONE` for a term that is `none`, the refusals its terms
    /// meet; `NotBuilt` for a query over a random value (M7).
    pub(super) fn key_query(
        &mut self,
        m: &Reality,
        query: &Query,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let Some(binds) = &query.binds else {
            return Err(Stop::NotBuilt("a query over a random value"));
        };
        let members = match self.members(m, binds, env, ctx, Brings::ToReader)? {
            Err(hole) => return Ok(Value::Hole(hole)),
            Ok(members) => members,
        };
        let word = word(query.func);
        let mut holes: Vec<Hole> = Vec::new();
        let mut terms: Vec<Value> = Vec::with_capacity(members.len());
        for member in &members {
            let t = self.ev(m, &query.arg, &member.env, ctx)?;
            if let Some(unknown) = &member.unknown {
                holes.push(unknown.clone());
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
        if terms.is_empty() {
            return Err(refuse(Run::Empty, format!("{word} over no keys")));
        }
        match query.func {
            QueryFn::Mean => mean(m, terms),
            QueryFn::Share => share(&terms),
            QueryFn::Quantile => self.quantile(m, query, terms, (env, ctx)),
        }
    }

    /// `quantile(t for b, q)`: the terms sorted by value, the first whose rank reaches q·n, the
    /// last when none does; a level that is a hole makes it that hole (refcore's `e_query`).
    /// L §5.11.
    fn quantile(
        &mut self,
        m: &Reality,
        query: &Query,
        mut terms: Vec<Value>,
        (env, ctx): (&Env, Ctx),
    ) -> Result<Value, Stop> {
        let level = query
            .level
            .as_ref()
            .ok_or(Stop::NotBuilt("a quantile with no level"))?;
        let read = self.ev(m, level, env, ctx)?;
        if let Value::Hole(h) = read {
            return Ok(Value::Hole(h));
        }
        let Value::Number(Number { q, .. }) = read else {
            return Err(not_typed());
        };
        let mut failed: Option<Stop> = None;
        terms.sort_by(|a, b| match compare(m, a, b, Test::Order) {
            Ok(order) => order,
            Err(stop) => {
                failed.get_or_insert(stop);
                Ordering::Equal
            }
        });
        if let Some(stop) = failed {
            return Err(stop);
        }
        let count = Num::int(i64::try_from(terms.len()).map_err(|_| too_many())?);
        let need = q.checked_mul(&count).map_err(num_stop)?;
        let mut rank = Num::int(0);
        let mut last = None;
        for t in terms {
            rank = rank.checked_add(&Num::int(1)).map_err(num_stop)?;
            if rank.compare(&need).map_err(num_stop)? != Ordering::Less {
                return Ok(t);
            }
            last = Some(t);
        }
        last.ok_or_else(|| refuse(Run::Empty, "quantile over no keys".to_owned()))
    }
}

/// `mean(t for b)`: the terms added as `sum` adds them, divided by their count, exactly.
/// L §5.11.
fn mean(m: &Reality, terms: Vec<Value>) -> Result<Value, Stop> {
    let count = i64::try_from(terms.len()).map_err(|_| too_many())?;
    let mut it = terms.into_iter();
    let Some(first) = it.next() else {
        return Err(refuse(Run::Empty, "mean over no keys".to_owned()));
    };
    let mut acc = first;
    for t in it {
        acc = arith(m, Op::Add, &acc, &t)?;
    }
    arith(m, Op::Div, &acc, &Value::int(count))
}

/// `share(c for b)`: the fraction of the terms that are true, exactly. L §5.11.
fn share(terms: &[Value]) -> Result<Value, Stop> {
    let mut held: i64 = 0;
    for t in terms {
        let Value::Bool(holds) = t else {
            return Err(not_typed());
        };
        if *holds {
            held = held.checked_add(1).ok_or_else(too_many)?;
        }
    }
    let count = i64::try_from(terms.len()).map_err(|_| too_many())?;
    let q = Num::ratio(held, count).map_err(num_stop)?;
    Ok(Value::Number(Number::plain(q, crate::model::Unit::one())))
}

/// A query's name, as its refusals print it.
fn word(func: QueryFn) -> &'static str {
    match func {
        QueryFn::Mean => "mean",
        QueryFn::Share => "share",
        QueryFn::Quantile => "quantile",
    }
}

/// A count of terms past the integers.
fn too_many() -> Stop {
    super::state::invariant("a count of a query's terms past the integers")
}
