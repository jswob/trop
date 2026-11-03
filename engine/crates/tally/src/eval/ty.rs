//! A sample value of an expression's class, made without reading, for the zero of an empty sum and a bare figure that is an `if`'s branch.
//! Spec: L §5.11 (a sum over no keys is 0 in its term's type), §3.2 (inline figures take the
//! type of what they meet; `if`'s branches unify), §3.1 (the types).
//! Twin: engine/tools/refcore/check.py (the static types `_t` the checker gives: `_arith`,
//! `t_if`, `c_round`) and evaluate.py (`zero_of`, `typed`).
//! Invariants: a sample is computed from declared types, literals and bound values only: no
//! read, no register entry, no memo cell; where the class can't be seen the run stops
//! `NotBuilt`, never guesses a zero.

use super::arith::{Op, arith, negate};
use super::builtin::{minor_of, round_values};
use super::expr::{literal, typed};
use super::state::{Ctx, Env, num_stop, refuse};
use crate::base::Run;
use crate::model::{
    Class, Code, IndexInfo, KeyVal, MvRef, MvSrc, Number, Reality, Ty, Unit, Value, currency_id,
    currency_unit, num_money,
};
use crate::num::{Money, Num};
use crate::syntax::{AddOp, Agg, AggFn, Bind, Call, Domain as Over, Expr, MulOp, Postfix, Stop};
use crate::time::{Day, Period};

/// The class of a term the run can't see stops here.
fn unseen() -> Stop {
    Stop::NotBuilt("the type of an expression the run can't see without reading")
}

/// 0 in the type of a sum's term, for a sum over no keys (refcore's `zero_of`). L §5.11.
///
/// # Errors
///
/// `RUN-DOMAIN` for a sum of rates; `NotBuilt` where the term's class can't be seen.
pub(super) fn zero_of(
    m: &Reality,
    term: &Expr,
    env: &Env,
    binds: &[Bind],
    ctx: Ctx,
) -> Result<Value, Stop> {
    let mut scope = env.clone();
    for bind in binds {
        let v = bind_sample(m, bind).ok_or_else(unseen)?;
        scope.bind(&bind.var, v, None);
    }
    let like = sample(m, term, &scope, ctx)?.ok_or_else(unseen)?;
    if let Value::Money(money) = &like {
        return Ok(Value::Money(Money::from_minor(0, money.currency())));
    }
    if let Value::Key(_) = like {
        return Ok(Value::int(0));
    }
    let Value::Number(Number { class, .. }) = like else {
        return Err(unseen());
    };
    if let Class::Rate(_) = class {
        return Err(refuse(
            Run::Domain,
            "the sum of rates over no keys (a rate carries its period)".to_owned(),
        ));
    }
    let Class::Num(unit, shown) = class else {
        return Ok(Value::int(0));
    };
    if unit
        .terms()
        .iter()
        .any(|(base, _)| m.kinds.get(base).is_some())
    {
        return Ok(Value::int(0));
    }
    Ok(Value::Number(Number {
        q: Num::zero(),
        class: Class::Num(unit, shown),
    }))
}

/// Bare figure `q` in the class of sample `like` (refcore's `typed` of the type `flex_to`
/// gives it); none where the class takes no figure. L §3.2.
///
/// # Errors
///
/// `LOAD-PRECISION` for money finer than its minor unit; a number error.
pub(super) fn figure_like(m: &Reality, q: &Num, like: &Value) -> Result<Option<Value>, Stop> {
    if let Value::Money(money) = like {
        let (name, _) = crate::model::currency(m, money.currency()).map_err(num_stop)?;
        return typed(m, q, &Ty::Money(name.to_owned()));
    }
    let Value::Number(Number { class, .. }) = like else {
        return Ok(None);
    };
    match class {
        Class::Int => typed(m, q, &Ty::Int),
        Class::Num(unit, shown) => typed(m, q, &Ty::Num(unit.clone(), *shown)),
        Class::Rate(_) | Class::Flex => Ok(None),
    }
}

/// A sample of expression `e`'s class: literals and bound values as they are, a read as a
/// value of its declared type (check.py's `tc`, where S needs it). L §3.1, §3.2.
///
/// # Errors
///
/// `NotBuilt` where an operation on the samples fails: the class can't be seen.
pub(super) fn sample(m: &Reality, e: &Expr, env: &Env, ctx: Ctx) -> Result<Option<Value>, Stop> {
    match e {
        Expr::Lit(lit) => literal(m, lit, env.figures).map(Some).map_err(|_| unseen()),
        Expr::Str(_) => Ok(Some(Value::Text(String::new()))),
        Expr::None => Ok(Some(Value::None)),
        Expr::Never => Ok(Some(Value::Never)),
        Expr::Cut => Ok(Some(Value::Day(ctx.cut))),
        Expr::Name(name) => name_sample(m, name, env),
        Expr::If(parts) => {
            let [_, yes, no] = parts.as_ref();
            let a = sample(m, yes, env, ctx)?;
            if a.as_ref().is_some_and(|v| !is_flex(v)) {
                return Ok(a);
            }
            let b = sample(m, no, env, ctx)?;
            Ok(if b.is_some() { b } else { a })
        }
        Expr::Or(_) | Expr::And(_) | Expr::Not(_) | Expr::Cmp(_) | Expr::In(_) => {
            Ok(Some(Value::Bool(true)))
        }
        Expr::Sum(first, rest) => {
            let ops: Vec<(Op, &Expr)> = rest.iter().map(|(op, x)| (add_op(*op), x)).collect();
            fold(m, first, &ops, env, ctx)
        }
        Expr::Prod(first, rest) => {
            let ops: Vec<(Op, &Expr)> = rest.iter().map(|(op, x)| (mul_op(*op), x)).collect();
            fold(m, first, &ops, env, ctx)
        }
        Expr::Neg(x) => match sample(m, x, env, ctx)? {
            None => Ok(None),
            Some(v) => negate(&v).map_err(|_| unseen()),
        },
        Expr::Paren(x) => sample(m, x, env, ctx),
        Expr::Post(base, posts) => post_sample(m, base, posts, env, ctx),
        Expr::Call(call) => call_sample(m, call, env, ctx),
        Expr::Agg(agg) => agg_sample(m, agg, env, ctx),
        Expr::Record(_)
        | Expr::List(_)
        | Expr::LegOf(..)
        | Expr::To(..)
        | Expr::Last(_)
        | Expr::Bal(_)
        | Expr::Flows(_)
        | Expr::Covered(_)
        | Expr::Calendar(_)
        | Expr::Value(_) => Ok(None),
    }
}

/// Whether a sample is a bare figure.
fn is_flex(v: &Value) -> bool {
    matches!(
        v,
        Value::Number(Number {
            class: Class::Flex,
            ..
        })
    )
}

/// The arithmetic operator of `+` and `-`.
fn add_op(op: AddOp) -> Op {
    match op {
        AddOp::Add => Op::Add,
        AddOp::Sub => Op::Sub,
    }
}

/// The arithmetic operator of `*` and `/`.
fn mul_op(op: MulOp) -> Op {
    match op {
        MulOp::Mul => Op::Mul,
        MulOp::Div => Op::Div,
    }
}

/// The samples of a chain of operations, left to right.
fn fold(
    m: &Reality,
    first: &Expr,
    rest: &[(Op, &Expr)],
    env: &Env,
    ctx: Ctx,
) -> Result<Option<Value>, Stop> {
    let Some(mut acc) = sample(m, first, env, ctx)? else {
        return Ok(None);
    };
    for (op, x) in rest {
        let Some(b) = sample(m, x, env, ctx)? else {
            return Ok(None);
        };
        acc = arith(m, *op, &acc, &b).map_err(|_| unseen())?;
    }
    Ok(Some(acc))
}

/// A name's sample: a bound variable's value, a read's declared type, a thing, a key literal.
fn name_sample(m: &Reality, name: &str, env: &Env) -> Result<Option<Value>, Stop> {
    if let Some(var) = env.get(name) {
        if let Value::Hole(_) = var.value {
            return Ok(None);
        }
        return Ok(Some(var.value.clone()));
    }
    if let Some(v) = item_sample(m, name)? {
        return Ok(Some(v));
    }
    if m.things.get(name).is_some() {
        return Ok(Some(Value::Key(KeyVal::Thing(name.to_owned()))));
    }
    Ok(Some(Value::Word(name.to_owned())))
}

/// A sample of an input's or a def's declared type; a figure in Y prices is a `num CUR`.
fn item_sample(m: &Reality, id: &str) -> Result<Option<Value>, Stop> {
    if let Some(input) = m.inputs.get(id) {
        let Some(ty) = &input.ty else {
            return Ok(None);
        };
        let v = ty_sample(m, ty)?;
        if input.prices.is_some()
            && let Some(Value::Money(money)) = &v
        {
            let unit = currency_unit(m, money.currency()).map_err(num_stop)?;
            return Ok(Some(Value::Number(Number::plain(Num::int(1), unit))));
        }
        return Ok(v);
    }
    match m.defs.get(id) {
        Some(def) => ty_sample(m, &def.ty),
        None => Ok(None),
    }
}

/// A read's sample with its fields taken, or a value's fields.
fn post_sample(
    m: &Reality,
    base: &Expr,
    posts: &[Postfix],
    env: &Env,
    ctx: Ctx,
) -> Result<Option<Value>, Stop> {
    let (mut value, rest) = match (base, posts) {
        (Expr::Name(name), [Postfix::Keys(_), rest @ ..]) if env.get(name).is_none() => {
            (item_sample(m, name)?, rest)
        }
        _ => (sample(m, base, env, ctx)?, posts),
    };
    for post in rest {
        if let (Some(Value::Mv(mv)), Postfix::Field(name)) = (&value, post)
            && let MvSrc::Kind(kind) = &mv.src
        {
            value = mv_field_sample(m, kind, name)?;
            continue;
        }
        let (Some(Value::Record(fields)), Postfix::Field(name)) = (&value, post) else {
            return Ok(None);
        };
        value = fields
            .iter()
            .find(|(f, _)| f == name)
            .map(|(_, v)| v.clone());
    }
    Ok(value)
}

/// A call's sample: a band's declared type, a built-in's result class.
fn call_sample(m: &Reality, call: &Call, env: &Env, ctx: Ctx) -> Result<Option<Value>, Stop> {
    if let Some(band) = m.bands.get(&call.name) {
        return ty_sample(m, &band.ty);
    }
    let first = match call.args.first() {
        Some(a) => sample(m, a, env, ctx)?,
        None => None,
    };
    let day = Value::Day(ctx.cut);
    match call.name.as_str() {
        "start_of" | "end_of" | "day_of" | "add_days" | "add_months" | "add_business" => {
            Ok(Some(day))
        }
        "year_of" => Ok(Period::year_of(ctx.cut).map(Value::Period)),
        "month_of" => Ok(Period::month_of(ctx.cut).map(Value::Period)),
        "quarter_of" => Ok(Period::quarter_of(ctx.cut).map(Value::Period)),
        "days_between" | "age" | "pos" => Ok(Some(Value::int(1))),
        "is_business" | "missing" => Ok(Some(Value::Bool(true))),
        "periodic" | "factor" => Ok(Some(Value::Number(Number::plain(Num::int(1), Unit::one())))),
        "minor" => minor_of(m, call).map(Some),
        "abs" => Ok(first),
        "min" | "max" => {
            let mut all = Vec::with_capacity(call.args.len());
            for a in &call.args {
                all.push(sample(m, a, env, ctx)?);
            }
            let with_unit = all.into_iter().flatten().find(|v| !is_flex(v));
            Ok(with_unit.or(first))
        }
        "round" => {
            let [x, u, mode] = call.args.as_slice() else {
                return Ok(None);
            };
            let (Some(x), Some(u), Some(mode)) = (
                sample(m, x, env, ctx)?,
                sample(m, u, env, ctx)?,
                sample(m, mode, env, ctx)?,
            ) else {
                return Ok(None);
            };
            round_values(m, &x, &u, &mode)
                .map(Some)
                .map_err(|_| unseen())
        }
        _ => Ok(None),
    }
}

/// An aggregate's sample: its term's, with its variables bound to samples of their domains.
fn agg_sample(m: &Reality, agg: &Agg, env: &Env, ctx: Ctx) -> Result<Option<Value>, Stop> {
    match agg.func {
        AggFn::Count => return Ok(Some(Value::int(1))),
        AggFn::Any | AggFn::All => return Ok(Some(Value::Bool(true))),
        AggFn::Sum
        | AggFn::Min
        | AggFn::Max
        | AggFn::First
        | AggFn::Last
        | AggFn::Argmin
        | AggFn::Argmax => {}
    }
    let mut scope = env.clone();
    for bind in &agg.binds {
        let Some(v) = bind_sample(m, bind) else {
            return Ok(None);
        };
        scope.bind(&bind.var, v, None);
    }
    match (&agg.term, agg.func) {
        (Some(term), AggFn::Sum | AggFn::Min | AggFn::Max | AggFn::First | AggFn::Last) => {
            sample(m, term, &scope, ctx)
        }
        _ => Ok(None),
    }
}

/// A sample of a binding's values: an index's first key, a thing, a list's first item.
fn bind_sample(m: &Reality, bind: &Bind) -> Option<Value> {
    match &bind.domain {
        Over::Name { id, qual: None } => {
            if let Some(ix) = m.indexes.get(id) {
                return first_key(ix);
            }
            if let Some(k) = m.kinds.get(id) {
                if k.spec == crate::model::KindSpec::Event {
                    return Some(mv_sample(id));
                }
                return Some(Value::Key(KeyVal::Thing(String::new())));
            }
            m.inputs
                .get(id)
                .filter(|i| i.observed)
                .map(|_| mv_sample("events"))
        }
        Over::Name { qual: Some(_), .. } | Over::List(_) | Over::Range(_) => None,
    }
}

/// A movement of event kind `kind` as a sample.
fn mv_sample(kind: &str) -> Value {
    Value::Mv(Box::new(MvRef {
        id: String::new(),
        day: None,
        src: MvSrc::Kind(kind.to_owned()),
    }))
}

/// A sample of field `name` of a movement of event kind `kind`: a leg as a record with an
/// amount of its unit, `day`, `fulfils`; a field of a movement whose kind isn't known statically
/// (`events`) read by name, its amount's unit known only at run (check3.py's
/// `movement_field`). L §5.14, §2.2.
fn mv_field_sample(m: &Reality, kind: &str, name: &str) -> Result<Option<Value>, Stop> {
    for (leg, unit, _) in m.kind_legs(kind) {
        if leg == name {
            return Ok(Some(leg_sample(unit_sample(m, &unit)?)));
        }
    }
    if m.kind_attrs(kind).iter().any(|(a, _)| a == name) {
        return Ok(None);
    }
    match name {
        "day" => {
            return Ok(Some(Value::Day(
                Day::from_ymd(2000, 1, 1).ok_or_else(unseen)?,
            )));
        }
        "fulfils" => return Ok(Some(mv_sample("events"))),
        _ => {}
    }
    if kind != "events" {
        return Ok(None);
    }
    for (id, k) in m.kinds.entries() {
        if k.spec == crate::model::KindSpec::Event && has_leg(m, id, name) {
            return Ok(Some(leg_sample(dyn_sample())));
        }
    }
    Ok(None)
}

/// Whether event kind `kind` has a leg named `name`. L §5.14.
fn has_leg(m: &Reality, kind: &str, name: &str) -> bool {
    m.kind_legs(kind).iter().any(|(leg, _, _)| leg == name)
}

/// A leg's record as a sample, its amount `amount`. L §5.14.
fn leg_sample(amount: Value) -> Value {
    let place = Value::Key(KeyVal::Thing(String::new()));
    Value::Record(vec![
        ("from".to_owned(), place.clone()),
        ("to".to_owned(), place),
        ("amount".to_owned(), amount),
        ("day".to_owned(), Value::None),
    ])
}

/// A sample of a quantity of `unit`: money of a currency, a number of a unit; one whose unit
/// is known only at run is a bare figure, which takes the unit of what it meets.
fn unit_sample(m: &Reality, unit: &str) -> Result<Value, Stop> {
    if m.currencies.contains_key(unit) {
        let id = currency_id(m, unit).map_err(num_stop)?;
        return Ok(Value::Money(
            num_money(m, &Num::int(1), id).map_err(num_stop)?,
        ));
    }
    if m.units.contains(unit) {
        let u = Unit::make(vec![(unit.to_owned(), 1)]).map_err(num_stop)?;
        return Ok(Value::Number(Number::plain(Num::int(1), u)));
    }
    Ok(dyn_sample())
}

/// A value whose unit is known only at run: a bare figure (check.py's `DYN`).
fn dyn_sample() -> Value {
    Value::Number(Number {
        q: Num::int(1),
        class: Class::Flex,
    })
}

/// An index's first key as a value.
fn first_key(ix: &IndexInfo) -> Option<Value> {
    ix.keys.first().map(Value::of_key)
}

/// A sample value of declared type `ty`; none for a type S's samples don't take.
fn ty_sample(m: &Reality, ty: &Ty) -> Result<Option<Value>, Stop> {
    let one = Num::int(1);
    let day = Day::from_ymd(2000, 1, 1);
    Ok(match ty {
        Ty::Num(unit, shown) => Some(Value::Number(Number {
            q: one,
            class: Class::Num(unit.clone(), *shown),
        })),
        Ty::Money(cur) => {
            let id = currency_id(m, cur).map_err(num_stop)?;
            Some(Value::Money(num_money(m, &one, id).map_err(num_stop)?))
        }
        Ty::Int | Ty::Year4 => Some(Value::int(1)),
        Ty::Flex(n) => Some(Value::Number(Number {
            q: Num::int(*n),
            class: Class::Flex,
        })),
        Ty::Bool => Some(Value::Bool(true)),
        Ty::Text => Some(Value::Text(String::new())),
        Ty::Date => day.map(Value::Day),
        Ty::Period(word) => day
            .and_then(|d| match word {
                crate::model::PeriodWord::Month => Period::month_of(d),
                crate::model::PeriodWord::Quarter => Period::quarter_of(d),
                crate::model::PeriodWord::Year => Period::year_of(d),
            })
            .map(Value::Period),
        Ty::Key(ix) => match m.indexes.get(ix) {
            Some(index) => first_key(index),
            None => m
                .kinds
                .get(ix)
                .map(|_| Value::Key(KeyVal::Thing(String::new()))),
        },
        Ty::MonthDay => Some(Value::MonthDay(1, 1)),
        Ty::Code(_) => Some(Value::Code(Code::Word(String::new()))),
        Ty::Record(fields) => {
            let mut out = Vec::with_capacity(fields.len());
            for (name, t) in fields {
                let Some(v) = ty_sample(m, t)? else {
                    return Ok(None);
                };
                out.push((name.clone(), v));
            }
            Some(Value::Record(out))
        }
        Ty::Rate
        | Ty::Dates
        | Ty::Dist(_)
        | Ty::Calendar
        | Ty::List(_)
        | Ty::Tuple(_)
        | Ty::KeyLit
        | Ty::None
        | Ty::Hole
        | Ty::Never
        | Ty::KindName => None,
    })
}
