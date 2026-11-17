//! Partial answers: under a `partial` ask line a hole carries its known part and the named holes it is linear in.
//! Spec: L §5.18 Partial answers (a line marked `partial` prints, when its value is a hole, its
//! known part and the named holes it is linear in, each with its coefficient: a hole stays
//! linear only through reads of numbers or money, sums, differences, products and quotients
//! by known numbers, `sum` and a balance; any other operation it passes through makes the line
//! a plain hole naming every hole; every named hole is printed; defs are unchanged), §5.19 and
//! §5.10 (a balance is its opening plus what is booked since, a sum).
//! Twin: engine/tools/refcore/evaluate.py (`plain_node`, `lin_of`, `lin_add`, `lin_scale`,
//! `lin_bin`, `e_neg`, `e_agg`'s `sum` under `ctx.partial`), ask.py (`fmt_partial`) and
//! mreads.py (`m_partial_bal`).
//! Invariants: only a `partial` line's context computes a linear part, and a cell read under it
//! is a cell of its own (the memo's key holds the clause), so no other line's value changes;
//! a hole whose linear part can't be told is plain, never a guessed one.

use super::arith::{Op, arith, negate};
use super::state::{Ctx, Env, invariant, num_stop};
use super::ty::sample;
use crate::hole::{Hole, Linear, atom_text, spread};
use crate::model::{Reality, Value};
use crate::syntax::{AggFn, Expr, Stop};

/// Node `e`'s value `v` as a `partial` line reads it: a hole stays linear in its holes only
/// through the operations L §5.18 names (sums, differences, products and quotients by known
/// numbers, `sum`, a balance) and a read of a number or money; any other node a hole passes
/// through (a call, a comparison, a key, a period, a field of a hole) makes it plain.
/// L §5.18 Partial answers.
pub(super) fn partial_value(m: &Reality, e: &Expr, env: &Env, ctx: Ctx, v: Value) -> Value {
    let Value::Hole(h) = &v else {
        return v;
    };
    // a walked sum or a mean builds its own linear form; met any other way the hole is plain
    // (tally 2.1.8 R15)
    if builds_linear(e) {
        return if h.printed_linear().is_some() {
            v
        } else {
            Value::Hole(h.plain())
        };
    }
    if keeps_linear(e) && reads_a_number(m, e, env, ctx) {
        return v;
    }
    Value::Hole(h.plain())
}

/// Whether node `e` gives its own linear form under a `partial` line: `flows`, a sum of its
/// movements; `value`, a sum of each position's units times its price; a query (`mean` over
/// a known count, its sum scaled; `share` and `quantile` none). L §5.18 (tally 2.1.8 R15).
fn builds_linear(e: &Expr) -> bool {
    matches!(e, Expr::Flows(_) | Expr::Query(_) | Expr::Value(_))
}

/// Whether node `e` passes a hole's linear part on: a read, a sum or difference, a product
/// or quotient, a minus, `sum`, `first` or `last`, a balance, a choice, `last(x, d)`, an
/// `as if`; L §5.18's first sentence, every other node plain.
fn keeps_linear(e: &Expr) -> bool {
    match e {
        Expr::Name(_)
        | Expr::Post(..)
        | Expr::Paren(_)
        | Expr::Sum(..)
        | Expr::Prod(..)
        | Expr::Neg(_)
        | Expr::Bal(_)
        | Expr::If(_)
        | Expr::Last(_)
        | Expr::AsIf(..) => true,
        Expr::Agg(agg) => matches!(agg.func, AggFn::Sum | AggFn::First | AggFn::Last),
        Expr::Lit(_)
        | Expr::Str(_)
        | Expr::None
        | Expr::Never
        | Expr::Cut
        | Expr::Or(_)
        | Expr::And(_)
        | Expr::Not(_)
        | Expr::Cmp(_)
        | Expr::In(_)
        | Expr::Call(_)
        | Expr::LegOf(..)
        | Expr::To(..)
        | Expr::Record(_)
        | Expr::List(_)
        | Expr::Flows(_)
        | Expr::Covered(_)
        | Expr::Calendar(_)
        | Expr::Value(_)
        | Expr::Query(_)
        | Expr::Low(_)
        | Expr::Crps(_)
        | Expr::DistFn(_)
        | Expr::Dist(_)
        | Expr::Prorata(_) => false,
    }
}

/// Whether a read `e` (a name, a postfix chain) may be of a number or money; other nodes
/// pass. A read whose declared type is a key, a period, a day, a flag or a text is no
/// coefficient's operand. A read whose class the run's samples can't see (a rate, L §3.3,
/// whose input's sample falls to a bare word; a run-time movement's leg; an opening or
/// `stated` read) keeps its linear part: `lin_of` still takes only a number or money as a
/// known part, and a sample that fails to tell the class (`sample`'s `NotBuilt`) says
/// nothing about the value, so it is not the line's answer. L §5.18, §3.1, §3.3.
fn reads_a_number(m: &Reality, e: &Expr, env: &Env, ctx: Ctx) -> bool {
    if !matches!(e, Expr::Name(_) | Expr::Post(..)) {
        return true;
    }
    match sample(m, e, env, ctx) {
        Ok(Some(Value::Number(_) | Value::Money(_) | Value::Word(_)) | None) | Err(_) => true,
        Ok(Some(
            Value::Hole(_)
            | Value::Bool(_)
            | Value::Day(_)
            | Value::Period(_)
            | Value::MonthDay(..)
            | Value::Key(_)
            | Value::Text(_)
            | Value::Code(_)
            | Value::Record(_)
            | Value::List(_)
            | Value::Calendar(_)
            | Value::None
            | Value::Never
            | Value::OutKey { .. }
            | Value::Mv(_)
            | Value::Window(_),
        )) => false,
    }
}

/// What value `v` is linear in: a known number or money is its own known part, any other
/// known value none; a hole, what it says (refcore's `lin_of`). L §5.18.
fn lin_of(v: &Value) -> Option<Linear> {
    match v {
        Value::Hole(h) => h.linear_part(),
        Value::Number(_) | Value::Money(_) => Some(Linear {
            known: Some(v.clone()),
            terms: Vec::new(),
        }),
        Value::Bool(_)
        | Value::Day(_)
        | Value::Period(_)
        | Value::MonthDay(..)
        | Value::Key(_)
        | Value::Text(_)
        | Value::Code(_)
        | Value::Record(_)
        | Value::List(_)
        | Value::Calendar(_)
        | Value::None
        | Value::Never
        | Value::Word(_)
        | Value::OutKey { .. }
        | Value::Mv(_)
        | Value::Window(_) => None,
    }
}

/// `a + b`, or `a - b`, of two linear parts: the known parts and each hole's coefficients
/// added (refcore's `lin_add`). L §5.18.
fn lin_add(m: &Reality, a: Linear, b: Linear, op: Op) -> Result<Linear, Stop> {
    let known = match (a.known, b.known) {
        (None, None) => None,
        (Some(ka), None) => Some(ka),
        (None, Some(kb)) => Some(signed(kb, op)?),
        (Some(ka), Some(kb)) => Some(arith(m, op, &ka, &kb)?),
    };
    let mut terms = a.terms;
    for (atom, c) in b.terms {
        let c = signed(c, op)?;
        match terms.iter_mut().find(|(at, _)| *at == atom) {
            Some((_, have)) => *have = arith(m, Op::Add, have, &c)?,
            None => terms.push((atom, c)),
        }
    }
    terms.sort_by(|x, y| x.0.cmp(&y.0));
    Ok(Linear { known, terms })
}

/// `v`, or `-v` under a subtraction.
fn signed(v: Value, op: Op) -> Result<Value, Stop> {
    if op != Op::Sub {
        return Ok(v);
    }
    negate(&v)?.ok_or_else(|| invariant("a known part or a coefficient with no negation"))
}

/// A linear part times, or divided by, known number `s`: its known part and each coefficient
/// (refcore's `lin_scale`; `RUN-DIV` by zero). L §5.18.
fn lin_scale(m: &Reality, l: Linear, s: &Value, op: Op) -> Result<Linear, Stop> {
    let known = match l.known {
        Some(k) => Some(arith(m, op, &k, s)?),
        None => None,
    };
    let mut terms = Vec::with_capacity(l.terms.len());
    for (atom, c) in l.terms {
        terms.push((atom, arith(m, op, &c, s)?));
    }
    Ok(Linear { known, terms })
}

/// `a op b` under a `partial` line where a hole meets it: linear in the holes for a sum, a
/// difference, a product by a known number and a quotient by one; plain otherwise (refcore's
/// `lin_bin`). L §5.18.
pub(super) fn lin_bin(m: &Reality, op: Op, a: &Value, b: &Value) -> Result<Value, Stop> {
    let qs = spread(&[a.as_hole(), b.as_hole()])
        .ok_or_else(|| invariant("a partial operation meeting no hole"))?;
    Ok(Value::Hole(match linear_op(m, op, a, b)? {
        Some(lin) => qs.linear(lin),
        None => qs.plain(),
    }))
}

/// [`lin_bin`]'s linear part, none where the operation isn't linear in its holes: a product
/// of two holes, a quotient by one, an operand that is no number. L §5.18.
fn linear_op(m: &Reality, op: Op, a: &Value, b: &Value) -> Result<Option<Linear>, Stop> {
    let (Some(la), Some(lb)) = (lin_of(a), lin_of(b)) else {
        return Ok(None);
    };
    let known = |v: &Value| matches!(v, Value::Number(_) | Value::Money(_));
    let (hole_a, hole_b) = (a.as_hole().is_some(), b.as_hole().is_some());
    match op {
        Op::Add | Op::Sub => lin_add(m, la, lb, op).map(Some),
        Op::Mul if hole_a && !hole_b && known(b) => lin_scale(m, la, b, op).map(Some),
        Op::Mul if hole_b && !hole_a && known(a) => lin_scale(m, lb, a, op).map(Some),
        Op::Div if !hole_b && known(b) => lin_scale(m, la, b, op).map(Some),
        Op::Mul | Op::Div => Ok(None),
    }
}

/// `-h` of a hole under a `partial` line: its linear part times -1, else the hole (refcore's
/// `e_neg`). L §5.18.
pub(super) fn lin_neg(m: &Reality, h: &Hole) -> Result<Value, Stop> {
    let qs = spread(&[Some(h)]).ok_or_else(|| invariant("a hole spread over itself"))?;
    match h.linear_part() {
        Some(lin) => Ok(Value::Hole(qs.linear(lin_scale(
            m,
            lin,
            &Value::int(-1),
            Op::Mul,
        )?))),
        None => Ok(Value::Hole(qs)),
    }
}

/// `sum` under a `partial` line whose terms include holes, every member known: the known
/// terms' sum `known` plus each hole term, linear where every hole term is, else plain
/// (refcore's `e_agg`, its `sum`). L §5.18, §5.11.
pub(super) fn lin_sum(m: &Reality, known: Value, holes: &[Hole]) -> Result<Value, Stop> {
    let all: Vec<Option<&Hole>> = holes.iter().map(Some).collect();
    let qs = spread(&all).ok_or_else(|| invariant("a partial sum with no hole term"))?;
    let mut acc = Linear {
        known: Some(known),
        terms: Vec::new(),
    };
    for h in holes {
        let Some(lin) = h.linear_part() else {
            return Ok(Value::Hole(qs.plain()));
        };
        acc = lin_add(m, acc, lin, Op::Add)?;
    }
    Ok(Value::Hole(qs.linear(acc)))
}

/// `mean` under a `partial` line over a known count `count` whose terms include holes: the
/// known terms' sum `known` plus each hole term, scaled by the count; plain where a hole term
/// isn't linear (refcore's `e_query`, its `mean`). L §5.18 (tally 2.1.8 R15), §5.11.
pub(super) fn lin_mean(
    m: &Reality,
    known: Option<Value>,
    holes: &[Hole],
    count: &Value,
) -> Result<Value, Stop> {
    let all: Vec<Option<&Hole>> = holes.iter().map(Some).collect();
    let qs = spread(&all).ok_or_else(|| invariant("a partial mean with no hole term"))?;
    let mut acc = Linear {
        known,
        terms: Vec::new(),
    };
    for h in holes {
        let Some(lin) = h.linear_part() else {
            return Ok(Value::Hole(qs.plain()));
        };
        acc = lin_add(m, acc, lin, Op::Add)?;
    }
    Ok(Value::Hole(qs.linear(lin_scale(m, acc, count, Op::Div)?)))
}

/// A linear part scaled by known `factor`: a position's units by its price, or its price by
/// its units. L §5.18 (tally 2.1.8 R15).
///
/// # Errors
///
/// A number's error.
pub(crate) fn lin_times(m: &Reality, lin: Linear, factor: &Value) -> Result<Linear, Stop> {
    lin_scale(m, lin, factor, Op::Mul)
}

/// A linear part times -1: a hole leg's where it leaves the place it is read at. L §5.18,
/// §5.10.
///
/// # Errors
///
/// A number's error.
pub(crate) fn lin_negated(m: &Reality, lin: Linear) -> Result<Linear, Stop> {
    lin_scale(m, lin, &Value::int(-1), Op::Mul)
}

/// A walked sum read by a `partial` line (a place's or a move's `flows`, a `with uncertain`
/// total): its known part `known` plus each hole operand's linear part `parts`, over the hole
/// `qs` of every hole met (refcore's `lin_fold`). L §5.18 (tally 2.1.8 R15).
///
/// # Errors
///
/// A number's error.
pub(crate) fn lin_fold(
    m: &Reality,
    qs: &Hole,
    known: Value,
    parts: Vec<Linear>,
) -> Result<Value, Stop> {
    let mut acc = Linear {
        known: Some(known),
        terms: Vec::new(),
    };
    for lin in parts {
        acc = lin_add(m, acc, lin, Op::Add)?;
    }
    Ok(Value::Hole(qs.linear(acc)))
}

/// A balance read by a `partial` line, `known` booked since the last reading plus each hole
/// leg's amount, read again, signed by its direction at the place (`into`); the plain hole
/// where a leg's amount isn't linear in its holes (refcore's `m_partial_bal`). L §5.19, §5.10,
/// §5.18.
pub(crate) fn lin_balance(
    m: &Reality,
    (balance, known): (&Hole, Value),
    legs: &[(&Hole, bool)],
) -> Result<Value, Stop> {
    let mut acc = Linear {
        known: Some(known),
        terms: Vec::new(),
    };
    for (h, into) in legs {
        let Some(lin) = h.linear_part() else {
            return Ok(Value::Hole(balance.clone()));
        };
        acc = lin_add(m, acc, lin, if *into { Op::Add } else { Op::Sub })?;
    }
    if acc.terms.is_empty() {
        return Ok(Value::Hole(balance.clone()));
    }
    Ok(Value::Hole(balance.linear(acc)))
}

/// A hole as a `partial` line prints it: its known part and each named hole with its
/// coefficient, `412 300.00 PLN + 1 × ?"…" (firm_share)`; none for a hole whose linear part
/// no operation computed (refcore's `fmt_partial`). L §5.18 Partial answers.
///
/// # Errors
///
/// A number's printing error.
pub(crate) fn partial_text(m: &Reality, h: &Hole) -> Result<Option<String>, Stop> {
    let Some(lin) = h.printed_linear() else {
        return Ok(None);
    };
    let mut parts = Vec::new();
    if let Some(k) = &lin.known {
        parts.push(k.text(m).map_err(num_stop)?);
    }
    for ((text, figure), c) in &lin.terms {
        parts.push(format!(
            "{} × {}",
            c.text(m).map_err(num_stop)?,
            atom_text(text, figure)
        ));
    }
    Ok(Some(parts.join(" + ")))
}
