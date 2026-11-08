//! The extremes over bindings: `min` and `max`, `argmin` and `argmax`, `first` and `last`, and the `ties by` that breaks a tie among them.
//! Spec: L §5.11 (aggregates: `min`/`max`/`argmin`/`argmax` over no keys are `RUN-EMPTY`,
//! a hole if any term is; `first`/`last` stop at the first key whose `where` is true, a hole
//! if a hole comes before it, `HOLE-NOTYET` over a binding empty at the cut whose span reaches
//! past it, else `RUN-EMPTY`; ties are stated: `ties by e`, `RUN-TIE` without one, and over
//! things, movements or an open index's keys the `ties by` term is the order `first` and
//! `last` run in), §4.4 (windows: `min`/`max` give the interval of the ends' extremes,
//! `argmin`/`argmax` the key decided below or above every other, else `HOLE-IMPRECISE`),
//! §5.6, §5.7 (things, movements and an open index's keys have no order a rule can read),
//! §3.1, §8.1 (holes spread), §8.4 (`RUN-TIE`, `RUN-EMPTY`, `RUN-NONE`).
//! Twin: engine/tools/refcore/evaluate.py (`agg_minmax`, `extreme`, `agg_arg`, `tie_break`,
//! `agg_first`, `tie_groups`, `notyet_bind`, `unordered_what`).
//! Invariants: a tie is broken only by the written `ties by`, never by index order or binding
//! order; a hole in a `ties by` term makes the result that hole (L §8.1; refcore answers the
//! key, STOP-REPORT 2026-11-08); `first` and `last` bring the register of the movements they
//! met up to the key they return, no further.

use super::aggregate::{Brings, Member};
use super::arith::{Test, compare};
use super::bindings::{day_of_binding, reads_name, span_reads};
use super::expr::hole_of;
use super::state::{Ctx, Engine, Env, a_day, invariant, num_stop, refuse};
use super::windows::{End, as_window, imprecise, window_end};
use crate::base::Run;
use crate::hole::Hole;
use crate::model::{IndexForm, KindSpec, Reality, Value, Window};
use crate::syntax::{Agg, AggFn, Bind, Domain as Over, Stop};
use core::cmp::Ordering;

/// A binding with no order a rule can read, by what it ranges over (refcore's
/// `unordered_what`). L §5.7, §5.6.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Unordered {
    /// A kind's things.
    Things,
    /// An event kind's or an observed input's movements.
    Movements,
    /// An open index's keys.
    OpenKeys,
}

impl Unordered {
    /// The words `RUN-TIE` names it by (L §8.4).
    fn word(self) -> &'static str {
        match self {
            Unordered::Things => "things",
            Unordered::Movements => "movements",
            Unordered::OpenKeys => "open-index keys",
        }
    }
}

/// What a binding ranges over when it has no order a rule can read, else none (refcore's
/// `unordered_what`). L §5.7, §5.6.
fn unordered(m: &Reality, bind: &Bind) -> Option<Unordered> {
    let Over::Name { id, qual: None } = &bind.domain else {
        return None;
    };
    if let Some(ix) = m.indexes.get(id) {
        return (ix.form == IndexForm::Open).then_some(Unordered::OpenKeys);
    }
    if let Some(kind) = m.kinds.get(id) {
        if kind.spec == KindSpec::Event {
            return Some(Unordered::Movements);
        }
        return Some(Unordered::Things);
    }
    m.inputs
        .get(id)
        .is_some_and(|i| i.observed)
        .then_some(Unordered::Movements)
}

/// The `min` or `max` of known values: of windows, the window from the extreme of their
/// lowest ends to the extreme of their highest, one value where those are equal (refcore's
/// `agg_minmax`, `extreme`). L §5.11, §4.4.
///
/// # Errors
///
/// The refusals of comparing the values; an invariant for no value, which every caller
/// refuses first (`RUN-EMPTY`, the call's arity).
pub(super) fn min_max(m: &Reality, vals: Vec<Value>, pick: End) -> Result<Value, Stop> {
    let Some(w) = vals.iter().find_map(as_window) else {
        return extreme_of(m, vals, pick);
    };
    let src = w.src.clone();
    let los: Vec<Value> = vals.iter().map(|v| window_end(v, End::Low)).collect();
    let his: Vec<Value> = vals.iter().map(|v| window_end(v, End::High)).collect();
    let (lo, hi) = (extreme_of(m, los, pick)?, extreme_of(m, his, pick)?);
    if compare(m, &lo, &hi, Test::Order)? == Ordering::Equal {
        return Ok(lo);
    }
    Ok(Value::Window(Box::new(Window {
        lo,
        hi,
        days: None,
        src,
        origin: None,
    })))
}

/// The least (`End::Low`) or greatest of values none of which is a window, the first of
/// equals (refcore's `extreme`). L §5.11.
fn extreme_of(m: &Reality, vals: Vec<Value>, pick: End) -> Result<Value, Stop> {
    let mut best: Option<Value> = None;
    for v in vals {
        best = Some(match best {
            None => v,
            Some(held) => {
                let order = compare(m, &v, &held, Test::Order)?;
                let better = match pick {
                    End::Low => order == Ordering::Less,
                    End::High => order == Ordering::Greater,
                };
                if better { v } else { held }
            }
        });
    }
    best.ok_or_else(|| invariant("an extreme of no values"))
}

/// A bare binding's value in scope `env`: its one variable's (refcore's `bare_key`). L §5.11.
pub(super) fn bare_key(agg: &Agg, env: &Env) -> Result<Value, Stop> {
    let [bind] = agg.binds.as_slice() else {
        return Err(Stop::NotBuilt("a bare aggregate over a tuple of keys"));
    };
    env.get(&bind.var)
        .map(|v| v.value.clone())
        .ok_or_else(|| invariant("a bound variable the scope lacks"))
}

/// The keys of tied candidates as `RUN-TIE` lists them, `b, c` (refcore's `fmt` of each
/// `bare_key`). L §8.4.
fn tied_keys(m: &Reality, agg: &Agg, envs: &[&Env]) -> Result<String, Stop> {
    let mut out = Vec::with_capacity(envs.len());
    for env in envs {
        out.push(bare_key(agg, env)?.text(m).map_err(num_stop)?);
    }
    Ok(out.join(", "))
}

/// Two `ties by` scores compared term by term, the first that differs deciding (refcore's
/// `cmp` in `tie_groups` and `tie_break`). L §5.11.
fn score_order(m: &Reality, a: &[Value], b: &[Value]) -> Result<Ordering, Stop> {
    for (x, y) in a.iter().zip(b) {
        let order = compare(m, x, y, Test::Order)?;
        if order != Ordering::Equal {
            return Ok(order);
        }
    }
    Ok(Ordering::Equal)
}

/// What a `ties by` gives a candidate: its score, or the hole one of its terms is.
enum Score {
    /// The terms' values, in the order written.
    Known(Vec<Value>),
    /// A term is a hole, with the questions of every term that is.
    Unknown(Hole),
}

impl Engine {
    /// The `ties by` terms of `agg` in a candidate's scope (refcore's `tie_break`,
    /// `tie_groups`): a term that is `none` is refused. L §5.11, §8.4.
    fn score(&mut self, m: &Reality, agg: &Agg, env: &Env, ctx: Ctx) -> Result<Score, Stop> {
        let Some(ties) = &agg.ties else {
            return Err(invariant("a ties by with no term"));
        };
        let mut vals = Vec::with_capacity(ties.len());
        for t in ties {
            vals.push(self.ev(m, t, env, ctx)?);
        }
        let holes: Vec<Option<&Hole>> = vals.iter().map(Value::as_hole).collect();
        if let Some(h) = crate::hole::spread(&holes) {
            return Ok(Score::Unknown(h));
        }
        if vals.contains(&Value::None) {
            return Err(refuse(
                Run::None,
                format!("the ties by of {} is none here", agg.func.word()),
            ));
        }
        Ok(Score::Known(vals))
    }

    /// `argmin(t for b)`, `argmax(t for b)` over the known terms, each with its scope: the
    /// key whose term is the extreme, a tie broken by `ties by`; of windows, the key decided
    /// below (above) every other, else `HOLE-IMPRECISE` (refcore's `agg_arg`). L §5.11, §4.4.
    pub(super) fn arg_extreme(
        &mut self,
        m: &Reality,
        agg: &Agg,
        terms: Vec<(&Env, Value)>,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let word = agg.func.word();
        if terms.is_empty() {
            return Err(refuse(Run::Empty, format!("{word} over no keys")));
        }
        let wanted = if agg.func == AggFn::Argmin {
            Ordering::Less
        } else {
            Ordering::Greater
        };
        if let Some(w) = terms.iter().find_map(|(_, t)| as_window(t)) {
            let w = w.clone();
            return decided_key(m, agg, &terms, wanted, &w);
        }
        let mut best: Vec<(&Env, Value)> = Vec::new();
        for (env, t) in terms {
            let Some((_, held)) = best.first() else {
                best.push((env, t));
                continue;
            };
            let order = compare(m, &t, held, Test::Order)?;
            if order == wanted {
                best = vec![(env, t)];
            } else if order == Ordering::Equal {
                best.push((env, t));
            }
        }
        let envs: Vec<&Env> = best.into_iter().map(|(env, _)| env).collect();
        match self.tie_break(m, agg, &envs, ctx)? {
            Ok(at) => bare_key(
                agg,
                envs.get(at)
                    .ok_or_else(|| invariant("a tie broken past its candidates"))?,
            ),
            Err(hole) => Ok(Value::Hole(hole)),
        }
    }

    /// The position in `tied` of the one candidate whose `ties by` score is least; with one
    /// candidate, it (refcore's `tie_break`). L §5.11, §8.4.
    ///
    /// # Errors
    ///
    /// `RUN-TIE` for a tie with no `ties by`, or one it doesn't break.
    fn tie_break(
        &mut self,
        m: &Reality,
        agg: &Agg,
        tied: &[&Env],
        ctx: Ctx,
    ) -> Result<Result<usize, Hole>, Stop> {
        let word = agg.func.word();
        if tied.len() == 1 {
            return Ok(Ok(0));
        }
        if agg.ties.is_none() {
            return Err(refuse(
                Run::Tie,
                format!(
                    "{word} has a tie at {} that changes the result, and no ties by",
                    tied_keys(m, agg, tied)?
                ),
            ));
        }
        let mut scored: Vec<Vec<Value>> = Vec::with_capacity(tied.len());
        for env in tied {
            match self.score(m, agg, env, ctx)? {
                Score::Unknown(h) => return Ok(Err(h)),
                Score::Known(vals) => scored.push(vals),
            }
        }
        let mut best: Vec<usize> = Vec::new();
        for (at, vals) in scored.iter().enumerate() {
            let order = match best.first().and_then(|b| scored.get(*b)) {
                None => Ordering::Less,
                Some(held) => score_order(m, vals, held)?,
            };
            if order == Ordering::Less {
                best = vec![at];
            } else if order == Ordering::Equal {
                best.push(at);
            }
        }
        let [one] = best.as_slice() else {
            let envs: Vec<&Env> = best
                .iter()
                .filter_map(|at| tied.get(*at).copied())
                .collect();
            return Err(refuse(
                Run::Tie,
                format!(
                    "{word} has a tie at {} that changes the result, and its ties by doesn't break it",
                    tied_keys(m, agg, &envs)?
                ),
            ));
        };
        Ok(Ok(*one))
    }

    /// `first(b where c)` and `last(…)`: the first (last) binding tuple whose `where` is
    /// true, in binding order, or over things, movements or an open index's keys in the order
    /// of the `ties by`; a hole met before it makes the result that hole; none, `HOLE-NOTYET`
    /// where a binding of things or movements reaches past the cut, else `RUN-EMPTY`
    /// (refcore's `agg_first`). L §5.11, §3.1, §8.1, §8.4.
    pub(super) fn first_last(
        &mut self,
        m: &Reality,
        agg: &Agg,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let word = agg.func.word();
        let un = agg.binds.iter().find_map(|b| unordered(m, b));
        if let (Some(what), None) = (un, &agg.ties) {
            return Err(refuse(
                Run::Tie,
                format!("{word} over {} has no ties by", what.word()),
            ));
        }
        let members = match self.members(m, &agg.binds, env, ctx, Brings::ToTuple)? {
            Err(hole) => return Ok(Value::Hole(hole)),
            Ok(members) => members,
        };
        let mut groups: Vec<Vec<Member>> = if un.is_some() {
            match self.tie_groups(m, agg, members, ctx)? {
                Err(hole) => return Ok(Value::Hole(hole)),
                Ok(groups) => groups,
            }
        } else {
            members.into_iter().map(|mm| vec![mm]).collect()
        };
        if agg.func == AggFn::Last {
            groups.reverse();
        }
        let mut holes: Vec<Hole> = Vec::new();
        for group in &groups {
            for member in group {
                self.absorb(&member.brought);
            }
            holes.extend(group.iter().filter_map(|mm| mm.unknown.clone()));
            let trues: Vec<&Env> = group
                .iter()
                .filter(|mm| mm.unknown.is_none())
                .map(|mm| &mm.env)
                .collect();
            let Some(first) = trues.first() else {
                continue;
            };
            if !holes.is_empty() {
                let operands: Vec<Option<&Hole>> = holes.iter().map(Some).collect();
                return hole_of(&operands);
            }
            if trues.len() > 1 {
                return Err(refuse(
                    Run::Tie,
                    format!(
                        "{word} has a tie at {} that changes the result, and its ties by doesn't break it",
                        tied_keys(m, agg, &trues)?
                    ),
                ));
            }
            return bare_key(agg, first);
        }
        if !holes.is_empty() {
            let operands: Vec<Option<&Hole>> = holes.iter().map(Some).collect();
            return hole_of(&operands);
        }
        if let Some((var, dom)) = self.notyet_bind(m, &agg.binds, env, ctx)? {
            let text = format!("{var}: no {dom} yet, and its span reaches past the cut");
            return Ok(Value::Hole(Hole::asked(&text, &dom)));
        }
        Err(refuse(Run::Empty, format!("{word} over no keys")))
    }

    /// The members in the order of their `ties by` scores, those with equal scores in one
    /// group, binding order kept within it; the hole of the first member whose score is one,
    /// with its own unknown-member hole (refcore's `tie_groups`). L §5.11.
    fn tie_groups(
        &mut self,
        m: &Reality,
        agg: &Agg,
        members: Vec<Member>,
        ctx: Ctx,
    ) -> Result<Result<Vec<Vec<Member>>, Hole>, Stop> {
        let mut scored: Vec<(Vec<Value>, Member)> = Vec::with_capacity(members.len());
        for member in members {
            match self.score(m, agg, &member.env, ctx)? {
                Score::Unknown(h) => {
                    let joined = crate::hole::spread(&[Some(&h), member.unknown.as_ref()]);
                    return Ok(Err(joined.unwrap_or(h)));
                }
                Score::Known(vals) => scored.push((vals, member)),
            }
        }
        // a stable sort, as refcore's: members with equal scores keep their binding order; the
        // first comparison refused refuses the aggregate
        let mut refused: Option<Stop> = None;
        scored.sort_by(|(a, _), (b, _)| match score_order(m, a, b) {
            Ok(order) => order,
            Err(stop) => {
                refused.get_or_insert(stop);
                Ordering::Equal
            }
        });
        if let Some(stop) = refused {
            return Err(stop);
        }
        let mut groups: Vec<(Vec<Value>, Vec<Member>)> = Vec::new();
        for (vals, member) in scored {
            if let Some((held, group)) = groups.last_mut()
                && score_order(m, &vals, held)? == Ordering::Equal
            {
                group.push(member);
                continue;
            }
            groups.push((vals, vec![member]));
        }
        Ok(Ok(groups.into_iter().map(|(_, g)| g).collect()))
    }

    /// The first binding of things or movements whose day is after the cut, or whose span
    /// reaches past it, its day or span read in the aggregate's scope; a binding whose day or
    /// span reads an earlier binding's variable, or is a hole, is passed over: its variable
    /// and what it ranges over (refcore's `notyet_bind`). L §3.1, §5.11.
    fn notyet_bind(
        &mut self,
        m: &Reality,
        binds: &[Bind],
        env: &Env,
        ctx: Ctx,
    ) -> Result<Option<(String, String)>, Stop> {
        let mut earlier: Vec<&str> = Vec::with_capacity(binds.len());
        for bind in binds {
            let reads_earlier = reads_any(bind, &earlier);
            earlier.push(&bind.var);
            let what = unordered(m, bind);
            if reads_earlier || !matches!(what, Some(Unordered::Things | Unordered::Movements)) {
                continue;
            }
            let Over::Name { id, .. } = &bind.domain else {
                continue;
            };
            if self.reaches_past(m, bind, env, ctx)? {
                return Ok(Some((bind.var.clone(), id.clone())));
            }
        }
        Ok(None)
    }

    /// Whether a binding's day is after the cut, or its span reaches past the day after it;
    /// a day or span that is a hole or `none` doesn't (refcore's `notyet_bind`, its tests).
    /// L §3.1.
    fn reaches_past(
        &mut self,
        m: &Reality,
        bind: &Bind,
        env: &Env,
        ctx: Ctx,
    ) -> Result<bool, Stop> {
        if let Some(on) = &bind.on {
            let day = self.ev(m, on, env, ctx)?;
            if matches!(day, Value::Hole(_) | Value::None) {
                return Ok(false);
            }
            return Ok(day_of_binding(&day)? > ctx.cut);
        }
        let Some(over) = &bind.over else {
            return Ok(false);
        };
        let Ok(span) = self.span_value(m, over, env, ctx)? else {
            return Ok(false);
        };
        let after = a_day(ctx.cut)?
            .next()
            .ok_or_else(|| invariant("the day after the cut"))?;
        Ok(span.end().is_none_or(|end| end > after))
    }
}

/// Whether a binding's day or span may read one of the variables `vars` (refcore's
/// `reads_var` over the earlier bindings). L §5.11.
fn reads_any(bind: &Bind, vars: &[&str]) -> bool {
    for var in vars {
        let on = bind.on.as_ref().is_some_and(|e| reads_name(e, var));
        let over = bind.over.as_ref().is_some_and(|s| span_reads(s, var));
        if on || over {
            return true;
        }
    }
    false
}

/// `argmin` (`wanted` Less) or `argmax` over terms of which one is a window: the key whose
/// term is decided below (above) every other's, its highest (lowest) end beside the other's
/// lowest (highest); otherwise `HOLE-IMPRECISE` of the first window (refcore's `agg_arg`).
/// L §5.11, §4.4.
fn decided_key(
    m: &Reality,
    agg: &Agg,
    terms: &[(&Env, Value)],
    wanted: Ordering,
    first: &Window,
) -> Result<Value, Stop> {
    let (own, other) = if wanted == Ordering::Less {
        (End::High, End::Low)
    } else {
        (End::Low, End::High)
    };
    let mut wins: Vec<&Env> = Vec::new();
    for (i, (env, t)) in terms.iter().enumerate() {
        let mut decided = true;
        for (j, (_, u)) in terms.iter().enumerate() {
            if i == j {
                continue;
            }
            if compare(m, &window_end(t, own), &window_end(u, other), Test::Order)? != wanted {
                decided = false;
                break;
            }
        }
        if decided {
            wins.push(env);
        }
    }
    if let [one] = wins.as_slice() {
        return bare_key(agg, one);
    }
    Ok(Value::Hole(imprecise(m, first, agg.func.word())?))
}
