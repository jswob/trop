//! Aggregates over keys written as queries: `mean`, `share` and `quantile` with `for`, the exact average, fraction and type-1 quantile of their terms.
//! Spec: L §5.11 (aggregates over keys: `mean(t for b)` the exact average, `share(c for b)` the
//! exact fraction of bindings where c holds, `quantile(t for b, q)` type 1, the smallest term
//! with at least q·n terms at or below it; over no keys `RUN-EMPTY`; a hole if any term or
//! `where` is; a term that is `none` refused, `RUN-NONE`), §4.4 (a term that is a
//! ranged date is `HOLE-IMPRECISE`), §8.1 (holes spread), §8.4. Without
//! `for` a query reads a random value (L §5.16, M7's).
//! Twin: engine/tools/refcore/evaluate.py (`e_query`, its arm with `for`).
//! Invariants: the terms are read in binding order, every one, as refcore reads them; a hole
//! where any term or `where` is, its questions in the order met; a `none` term refuses at its
//! member; `mean` adds the terms as `sum` does and divides by their count, exactly; `share`
//! counts the true terms over all of them; `quantile` sorts the terms by value, equals kept
//! in binding order, and takes the first whose rank reaches q·n, the last when none does.

use super::aggregate::{Brings, window_term_hole};
use super::arith::{Op, Test, arith, compare};
use super::expr::hole_of;
use super::state::{Ctx, Engine, Env, not_typed, num_stop, refuse};
use crate::base::Run;
use crate::hole::Hole;
use crate::model::{Class, Number, Reality, Value};
use crate::num::Num;
use crate::syntax::{Expr, Query, QueryFn, Stop};
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
            if let Value::Window(w) = &t {
                // a ranged date is no term's one value: the hole, as `sum`'s
                // (L §4.4: every other read is the hole)
                holes.push(window_term_hole(w, word));
                continue;
            }
            terms.push(t);
        }
        if !holes.is_empty() {
            // a `partial` line's mean over a known count: its sum scaled by the count, linear in
            // its terms (L §5.18; tally 2.1.8 R15)
            if ctx.partial
                && query.func == QueryFn::Mean
                && members.iter().all(|b| b.unknown.is_none())
            {
                let count = Value::int(i64::try_from(members.len()).map_err(|_| too_many())?);
                let mut known: Option<Value> = None;
                for t in terms {
                    known = Some(match known {
                        None => t,
                        Some(acc) => arith(m, Op::Add, &acc, &t)?,
                    });
                }
                return super::partial::lin_mean(m, known, &holes, &count);
            }
            // a refused level refuses before the hole terms: refusals outrank holes (L §8;
            // tally 2.1.8 R16, #171; B7 of group 3's review)
            if query.func == QueryFn::Quantile
                && let Some(level) = &query.level
            {
                let read = self.ev(m, level, env, ctx)?;
                if let Value::Hole(h) = &read {
                    holes.push(h.clone());
                } else if read == Value::None {
                    // a `none` level too (L §8, l.2217; 208.10 on #241)
                    return Err(none_level(m, level, env));
                } else if let Value::Number(Number { q, .. }) = &read {
                    level_in_range(m, &read, q)?;
                }
            }
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
    /// last when none does; a level that is a hole makes it that hole, one that is `none` is
    /// `RUN-NONE` (refcore's `e_query`). L §5.11, §3.1.
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
        if read == Value::None {
            return Err(none_level(m, level, env));
        }
        let Value::Number(Number { q, .. }) = &read else {
            return Err(not_typed());
        };
        level_in_range(m, &read, q)?;
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

/// A quantile's level `level` that is `none`: `none` is only tested, so the level's use is
/// refused, `RUN-NONE`, naming the level as written, each variable of `env` by its value,
/// `rate[k2] is none here: the level of quantile` (refcore's `level_none`). L §3.1, §8, §8.4
/// `RUN-NONE`'s row (tally 2.2.1's 208.10, 233.7, 235.6; P48 B2). A number error printing a
/// bound value is that error.
fn none_level(m: &Reality, level: &Expr, env: &Env) -> Stop {
    match super::bindings::written_bound(m, level, env) {
        Ok(shown) => refuse(
            Run::None,
            format!("{shown} is none here: the level of quantile"),
        ),
        Err(stop) => stop,
    }
}

/// A quantile's level `q` (read as `read`) outside (0, 1]: type 1 over its terms has no
/// smallest term there, `RUN-DOMAIN` (L §5.11, §5.16; tally 2.1.8 R16, #171).
fn level_in_range(m: &Reality, read: &Value, q: &Num) -> Result<(), Stop> {
    let above_zero = q.compare(&Num::zero()).map_err(num_stop)?;
    let past_one = q.compare(&Num::int(1)).map_err(num_stop)?;
    if above_zero != Ordering::Greater || past_one == Ordering::Greater {
        let shown = read.text(m).map_err(num_stop)?;
        let text = format!("quantile has no meaning for the level {shown}");
        return Err(refuse(Run::Domain, text));
    }
    Ok(())
}

/// `mean(t for b)`: the terms added as `sum` adds them, divided by their count, exactly; a
/// mean of rates is a `num` of their unit, a rate being a `num` with a period (refcore's
/// `e_query`, its `Num(acc.q / n, acc.u)`; tally 2.3.1 S17, N256.5). L §5.11, §3.3.
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
    if let Value::Number(Number {
        q,
        class: Class::Rate(unit),
    }) = acc
    {
        acc = Value::Number(Number::plain(q, unit));
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

#[cfg(test)]
mod tests {
    use crate::api::{Asks, Files, Given, Request, run};

    #[test]
    fn a_mean_of_rates_keeps_their_unit() {
        // L §5.11 (tally 2.3.1 S17): a rate is a num with a period
        let text = "model m\n  tally 2\n  on 2026-10-18\n  start 2026-01-01\n  end 2026-12-31\n\n\
                    source s \"s\"\nindex ks = [k1, k2, k3]\ninput r[k in ks]: rate\n\n\
                    ask m1: mean(r[k] for k in ks)\n\nversion ver\n  cite s\n  governs always\n  \
                    known 2026-01-01\n  r[k1] = 2 %/year \"s\"\n  r[k2] = 4 %/year \"s\"\n  \
                    r[k3] = 6 %/year \"s\"\n";
        let given = vec![Given {
            path: "m.tally".to_owned(),
            bytes: Some(text.as_bytes().to_vec()),
        }];
        let lines = run(Files::new(given), &Request::new(Asks::Every))
            .expect("an answer")
            .lines()
            .to_vec();
        assert_eq!(
            lines.first().map(String::as_str),
            Some("m1: 0.04 1/year"),
            "the mean in the rates' unit"
        );
    }
}
