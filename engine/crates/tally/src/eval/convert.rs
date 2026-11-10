//! A row's value converted to its target's type: numbers in its unit, money in its currency, keys, days, lists, records and calendars.
//! Spec: L §3.1 (the types and their values: dates, months, quarters, month-days, `dates`,
//! `key K`, records, calendars), §3.2 (a number's unit, `%`, a keyed unit's `$v`), §3.3 (a
//! rate's code words), §2.1, §4.2 (keys as values), §4.4 (a window or a choice of days), §5.7
//! (an event kind's keys are movements), §5.8 (inputs and their values), §5.19 (a reading's
//! row), §7.1 (exact numbers), §8.3 (`LOAD-UNIT`), §8.4.
//! Twin: engine/tools/refcore/loader.py (`convert_value` and its arms, `key_value`).
//! Invariants: a converted value has its target's declared type; a stated hole stays its
//! text; money is whole in its minor unit; a hole is built in the hole module.

use super::keys::literal_key;
use super::read::keyed;
use super::state::{invariant, num_stop, refuse_load, unit_stop};
use crate::base::Load;
use crate::model::{
    BindInfo, Class, Code, CodeKind, Domain, NameKind, Number, Pat, PeriodWord, Reality, Ty, Unit,
    Value, Window, currency_id, num_money,
};
use crate::num::Num;
use crate::syntax::{
    Calendar as CalendarLit, CalendarDay, DaySet, Expr, Field, FieldValue, Lit, Stop,
    Value as Written,
};
use crate::time::{Added, Calendar, Day, FirstYear, Period, weekday_number};

/// A row's value converted to its target's type: a value, or a stated hole's text (refcore's
/// `convert_value`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Converted {
    /// A value.
    Value(Value),
    /// `?"…"`: the hole's question, built with its figure where it is read.
    Hole(String),
}

/// What a number of a row is converted with: the factor a `%` in its type's unit brings, and,
/// for a keyed unit, the key text its pattern states for each bound variable (refcore's
/// `convert_value`'s `factor`, and its `env` from `binds` and `pattern`).
#[derive(Debug, Clone)]
pub(crate) struct ConvertFor {
    /// The `%` the type's unit brings.
    pub(crate) factor: Num,
    /// `(variable, key text)` for each key the row states; left empty unless the type's
    /// unit holds a `$var`.
    pub(crate) keys: Vec<(String, String)>,
}

impl ConvertFor {
    /// A value with no `%` factor and no keys: a reading's, a record field's.
    pub(crate) fn plain() -> ConvertFor {
        ConvertFor {
            factor: Num::int(1),
            keys: Vec::new(),
        }
    }

    /// A row of a target of type `ty` whose unit brings `factor`, its keys stated by
    /// `pattern` at `binds` (refcore's `convert_value`'s `env`). L §3.2.
    pub(crate) fn row(factor: &Num, ty: &Ty, binds: &[BindInfo], pattern: &[Pat]) -> ConvertFor {
        let keys = match ty {
            Ty::Num(unit, _) if keyed(unit) => binds
                .iter()
                .zip(pattern)
                .filter_map(|(bind, pat)| match pat {
                    Pat::Key(key) => Some((bind.var.clone(), key.text())),
                    Pat::Star | Pat::Range(..) => None,
                })
                .collect(),
            Ty::Int
            | Ty::Num(..)
            | Ty::Money(_)
            | Ty::Rate
            | Ty::Period(_)
            | Ty::Bool
            | Ty::Text
            | Ty::Date
            | Ty::Dates
            | Ty::Dist(_)
            | Ty::Calendar
            | Ty::Code(_)
            | Ty::MonthDay
            | Ty::Key(_)
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
            | Ty::KindName => Vec::new(),
        };
        ConvertFor {
            factor: factor.clone(),
            keys,
        }
    }
}

/// A reading's row value as a runtime value of type `ty`: the value, or a stated hole's text
/// (refcore's `convert_value`, for the walk's readings). L §5.19, §5.8.
///
/// Load's place-value check has converted every reading row before the walk reads one, with
/// its input named, so no refusal of a message that names the target is met here.
///
/// # Errors
///
/// The unit refusals load makes; `NotBuilt` for a window.
pub(crate) fn convert_row(
    m: &Reality,
    value: &Written,
    ty: &Ty,
) -> Result<Result<Value, String>, Stop> {
    match convert(m, value, ty, "a reading", &ConvertFor::plain())? {
        Converted::Value(v) => Ok(Ok(v)),
        Converted::Hole(text) => Ok(Err(text)),
    }
}

/// A row's value as a runtime value of type `ty`; `target` is the item a message names
/// (refcore's `convert_value`, its checks in its order). L §5.8, §3.1, §3.2, §2.1.
///
/// # Errors
///
/// `LOAD-UNIT`, `LOAD-PRECISION`, `LOAD-FIELD`, `LOAD-DOMAIN` and `LOAD-PARSE` as refcore
/// refuses a row's value; `NotBuilt` for a scenario's expression row, a distribution and a
/// movement named as a value.
pub(crate) fn convert(
    m: &Reality,
    value: &Written,
    ty: &Ty,
    target: &str,
    of: &ConvertFor,
) -> Result<Converted, Stop> {
    let v = match value {
        Written::Hole(text) => return Ok(Converted::Hole(text.clone())),
        Written::None => Value::None,
        Written::Never if *ty == Ty::Date => Value::Never,
        Written::Never => {
            return Err(unit_refusal(format!(
                "never is a date; {target} is {}",
                ty.show()
            )));
        }
        Written::Lit(lit) => convert_lit(m, lit, ty, target, of)?,
        Written::Str(s) if *ty == Ty::Text => Value::Text(s.clone()),
        Written::Str(_) => return Err(wrong_for("text", ty)),
        Written::Days(days) => return days_value(days, ty, target),
        Written::List(items) => list_value(items, ty, target)?,
        Written::Record(fields) => convert_record(m, fields, ty, target)?,
        Written::Calendar(cal) if *ty == Ty::Calendar => calendar_value(cal)?,
        Written::Calendar(_) => {
            return Err(unit_refusal(format!("a calendar for {}", ty.show())));
        }
        Written::Dist(_) if matches!(ty, Ty::Dist(_)) => {
            return Err(Stop::NotBuilt("distributions"));
        }
        Written::Dist(_) => {
            return Err(unit_refusal(format!("a distribution for {}", ty.show())));
        }
        Written::Expr(_) => return Err(Stop::NotBuilt("a scenario's expression row")),
    };
    Ok(Converted::Value(v))
}

/// `LOAD-UNIT` with `message`.
fn unit_refusal(message: String) -> Stop {
    refuse_load(Load::Unit, message)
}

/// `LOAD-UNIT a value of the wrong type: <what> for <type>`.
fn wrong_for(what: &str, ty: &Ty) -> Stop {
    unit_refusal(format!(
        "a value of the wrong type: {what} for {}",
        ty.show()
    ))
}

/// A window or a choice of days as a row's value: a date, its window in order (refcore's
/// `convert_value`, its `window` and `oneof` arms); the read that makes it names its origin.
/// L §4.4, §8.4.
fn days_value(days: &DaySet, ty: &Ty, target: &str) -> Result<Converted, Stop> {
    if *ty != Ty::Date {
        return Err(unit_refusal(format!(
            "a window is a date; {target} is {}",
            ty.show()
        )));
    }
    let window = match days {
        DaySet::Window(a, b) => {
            if a > b {
                return Err(refuse_load(
                    Load::Domain,
                    format!("window({}, {}) with a after b (L §8.4)", a.text(), b.text()),
                ));
            }
            Window {
                lo: Value::Day(*a),
                hi: Value::Day(*b),
                days: None,
                src: Some(format!("window({}, {})", a.text(), b.text())),
                figure: None,
                read: None,
            }
        }
        DaySet::OneOf(written) => one_of(written)?,
    };
    Ok(Converted::Value(Value::Window(Box::new(window))))
}

/// `one_of([…])` of these days: their window, its days sorted and each once (refcore's
/// `convert_value`, its `oneof` arm, and `c_one_of`). L §4.4.
pub(super) fn one_of(written: &[Day]) -> Result<Window, Stop> {
    let mut days = written.to_vec();
    days.sort();
    days.dedup();
    let (Some(lo), Some(hi)) = (days.first().copied(), days.last().copied()) else {
        return Err(Stop::NotBuilt("one_of of no days, met at run"));
    };
    let texts: Vec<String> = days.iter().map(|d| d.text()).collect();
    Ok(Window {
        lo: Value::Day(lo),
        hi: Value::Day(hi),
        src: Some(format!("one_of([{}])", texts.join(", "))),
        days: Some(days),
        figure: None,
        read: None,
    })
}

/// A list of days, the value of `dates` (refcore's `convert_value`, its `list` arm). L §3.1.
fn list_value(items: &[Lit], ty: &Ty, target: &str) -> Result<Value, Stop> {
    if *ty != Ty::Dates {
        return Err(unit_refusal(format!(
            "a list is of type dates; {target} is {}",
            ty.show()
        )));
    }
    let mut days = Vec::with_capacity(items.len());
    for item in items {
        let Lit::Day(d) = item else {
            return Err(unit_refusal("dates holds days".to_owned()));
        };
        days.push(Value::Day(*d));
    }
    Ok(Value::List(days))
}

/// A record's fields, each named once and each a field of its type, converted, then in the
/// type's order; a field the record leaves out is refused (refcore's `convert_value`, its
/// `record` arm). L §3.1.
fn convert_record(m: &Reality, fields: &[Field], ty: &Ty, target: &str) -> Result<Value, Stop> {
    let Ty::Record(types) = ty else {
        return Err(unit_refusal(format!("a record for {}", ty.show())));
    };
    let mut values: Vec<(&str, Value)> = Vec::with_capacity(fields.len());
    for field in fields {
        let Field::Named(name, written) = field else {
            return Err(refuse_load(
                Load::Field,
                format!("a leg is not a field of {}", ty.show()),
            ));
        };
        let Some((_, field_ty)) = types.iter().find(|(n, _)| n == name) else {
            return Err(refuse_load(
                Load::Field,
                format!("{name} is not a field of {}", ty.show()),
            ));
        };
        if values.iter().any(|(n, _)| n == name) {
            return Err(refuse_load(Load::Field, format!("field {name} twice")));
        }
        values.push((name, field_value(m, written, field_ty, target)?));
    }
    let mut out = Vec::with_capacity(types.len());
    for (name, _) in types {
        let Some((_, value)) = values.iter().find(|(n, _)| n == name) else {
            return Err(refuse_load(
                Load::Field,
                format!("{target} lacks field {name}"),
            ));
        };
        out.push((name.clone(), value.clone()));
    }
    Ok(Value::Record(out))
}

/// A record field's value as a value of its field's type; a hole or an expression stops "not
/// built" (refcore's `convert_value` on the field). L §3.1.
fn field_value(m: &Reality, written: &FieldValue, ty: &Ty, target: &str) -> Result<Value, Stop> {
    if let FieldValue::None | FieldValue::Expr(Expr::None) = written {
        return Ok(Value::None);
    }
    let lit = field_literal(written).ok_or(Stop::NotBuilt(
        "a record field that is a hole or an expression",
    ))?;
    convert_lit(m, &lit, ty, target, &ConvertFor::plain())
}

/// The literal a record field states, a word read as a key included; none for `none`, a hole
/// or an expression (refcore's `convert_value`, a record's fields). L §3.1.
pub(crate) fn field_literal(value: &FieldValue) -> Option<Lit> {
    match value {
        FieldValue::Lit(lit) | FieldValue::Expr(Expr::Lit(lit)) => Some(lit.clone()),
        FieldValue::Expr(Expr::Name(word)) => Some(Lit::Name(word.clone())),
        FieldValue::None | FieldValue::Hole(_) | FieldValue::Expr(_) => None,
    }
}

/// A literal of a row as a value of type `ty` (refcore's `convert_value`, its arms by the
/// literal's kind). L §5.8, §3.1.
fn convert_lit(
    m: &Reality,
    lit: &Lit,
    ty: &Ty,
    target: &str,
    of: &ConvertFor,
) -> Result<Value, Stop> {
    match lit {
        Lit::Num(n) => convert_number(m, n, ty, of),
        Lit::Bool(b) if *ty == Ty::Bool => Ok(Value::Bool(*b)),
        Lit::Bool(b) => Err(wrong_for(if *b { "True" } else { "False" }, ty)),
        Lit::Day(d) => {
            let own = (*ty == Ty::Date).then_some(Value::Day(*d));
            dated(m, lit, ty, own, "a date")
        }
        Lit::Month(y, mo) => {
            let own = (*ty == Ty::Period(PeriodWord::Month))
                .then(|| Period::month(*y, *mo).map(Value::Period))
                .flatten();
            dated(m, lit, ty, own, "a month")
        }
        Lit::Quarter(y, q) => {
            let own = (*ty == Ty::Period(PeriodWord::Quarter))
                .then(|| Period::quarter(*y, *q).map(Value::Period))
                .flatten();
            dated(m, lit, ty, own, "a quarter")
        }
        Lit::MonthDay(mo, d) => {
            if *ty == Ty::MonthDay && Day::from_ymd(2001, *mo, *d).is_none() {
                return Err(refuse_load(
                    Load::Parse,
                    format!(
                        "cannot parse: {} is not a month-day every year has",
                        lit.text()
                    ),
                ));
            }
            let own = (*ty == Ty::MonthDay).then_some(Value::MonthDay(*mo, *d));
            dated(m, lit, ty, own, "a month-day")
        }
        Lit::Name(text)
            if is_event_key(m, ty)
                && matches!(
                    m.names.kind(text.as_str()),
                    Some(NameKind::Move | NameKind::Input)
                ) =>
        {
            Err(Stop::NotBuilt("a movement named as a row's value"))
        }
        Lit::Name(text) | Lit::Key(text) | Lit::Cur(text) => word_value(m, lit, text, ty),
        Lit::Nominal(times) => {
            if *ty != Ty::Code(CodeKind::Compounding) {
                let figure = times
                    .integer_text()
                    .ok_or_else(|| invariant("a nominal the parser read"))?;
                return Err(unit_refusal(format!(
                    "nominal {figure} is a compounding code"
                )));
            }
            // as `eval/expr.rs`'s literal: refcore takes any size, the engine 64 bits
            times
                .to_i64()
                .map(|whole| Value::Code(Code::Nominal(whole)))
                .ok_or(Stop::NotBuilt("a nominal compounding past 64 bits"))
        }
        Lit::Inf { .. } => Err(unit_refusal(format!(
            "a value of the wrong type for {target}"
        ))),
    }
}

/// A date, a month, a quarter or a month-day of a row: its own type's value, a key of
/// `key ix`, or `LOAD-UNIT` naming `what` (refcore's `convert_value`, its dated arms). L §4.2.
fn dated(m: &Reality, lit: &Lit, ty: &Ty, own: Option<Value>, what: &str) -> Result<Value, Stop> {
    if let Some(value) = own {
        return Ok(value);
    }
    if let Ty::Key(ix) = ty {
        return key_value(m, lit, ix);
    }
    Err(wrong_for(what, ty))
}

/// A word of a row: a key of `key ix`, or a code word of its code type (refcore's
/// `convert_value`, its `name`, `key` and `cur` arm). L §3.1, §3.3.
fn word_value(m: &Reality, lit: &Lit, text: &str, ty: &Ty) -> Result<Value, Stop> {
    match ty {
        Ty::Key(ix) => key_value(m, lit, ix),
        Ty::Code(kind) if CodeKind::of_word(text) == Some(*kind) && text != "nominal" => {
            Ok(Value::Code(Code::Word(text.to_owned())))
        }
        Ty::Code(kind) => Err(unit_refusal(format!(
            "{text} is not a {} code",
            kind.name()
        ))),
        Ty::Int
        | Ty::Num(..)
        | Ty::Money(_)
        | Ty::Rate
        | Ty::Period(_)
        | Ty::Bool
        | Ty::Text
        | Ty::Date
        | Ty::Dates
        | Ty::Dist(_)
        | Ty::Calendar
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
        | Ty::KindName => Err(wrong_for(text, ty)),
    }
}

/// Whether `ty` is a key of an event kind, whose keys are movements (L §3.1 `key K`, §5.7).
fn is_event_key(m: &Reality, ty: &Ty) -> bool {
    let Ty::Key(ix) = ty else {
        return false;
    };
    m.kinds.contains(ix) && m.subtree("events").contains(ix)
}

/// A key literal of `key ix` as a value: an index's key, or a thing of a kind; a key of an
/// integer index is an int (refcore's `key_value`). L §4.2.
pub(super) fn key_value(m: &Reality, lit: &Lit, ix: &str) -> Result<Value, Stop> {
    let domain = if m.indexes.get(ix).is_some() {
        Domain::Index(ix.to_owned())
    } else if m.kinds.get(ix).is_some() {
        Domain::Kind(ix.to_owned())
    } else {
        return Err(refuse_load(Load::Name, format!("unknown id {ix}")).naming(ix));
    };
    let key = literal_key(m, lit, &domain)?
        .ok_or_else(|| refuse_load(Load::Key, format!("{} is not a key of {ix}", lit.text())))?;
    Ok(Value::of_key(&key))
}

/// A number literal of a row as a value of type `ty` (refcore's `convert_value`, its `num`
/// arm in its order: a terminating fraction, then the type's own rule). L §2.1, §3.2, §7.1.
fn convert_number(
    m: &Reality,
    n: &crate::syntax::Number,
    ty: &Ty,
    of: &ConvertFor,
) -> Result<Value, Stop> {
    if n.fraction && n.value.decimal_places().map_err(num_stop)?.is_some() {
        return Err(refuse_load(
            Load::Precision,
            format!("{} terminates; write it as a decimal", n.text),
        ));
    }
    let written = match &n.unit {
        None => None,
        Some(unit) => {
            let terms = unit
                .terms
                .iter()
                .map(|t| (t.base.clone(), t.power.clone()))
                .collect();
            Some(Unit::from_syntax(terms).map_err(unit_stop)?)
        }
    };
    match ty {
        Ty::Money(cur) => money_value(m, n, cur, written.as_ref(), of, ty),
        Ty::Num(unit, shown) => {
            let q = number_in(n, unit, written.as_ref(), of, ty)?;
            Ok(Value::Number(Number {
                q,
                class: Class::Num(unit.clone(), *shown),
            }))
        }
        Ty::Int => {
            if written.is_some() || !n.whole || n.fraction {
                return Err(unit_refusal(format!("{} is not an int", n.text)));
            }
            Ok(Value::Number(Number {
                q: n.value.clone(),
                class: Class::Int,
            }))
        }
        Ty::Rate => {
            let Some((unit, f)) = written.filter(|(u, _)| u.period_of().is_some()) else {
                return Err(unit_refusal(format!(
                    "a rate states its period: {}",
                    n.text
                )));
            };
            Ok(Value::Number(Number {
                q: n.value.checked_mul(&f).map_err(num_stop)?,
                class: Class::Rate(unit),
            }))
        }
        Ty::Period(PeriodWord::Year) if written.is_none() && n.whole && n.four => {
            let year = n.value.to_i64().and_then(|y| u16::try_from(y).ok());
            year.and_then(Period::year)
                .map(Value::Period)
                .ok_or_else(|| wrong_number(n, ty))
        }
        Ty::Key(ix) => key_value(m, &Lit::Num(n.clone()), ix),
        Ty::Dist(_) => Err(Stop::NotBuilt("distributions")),
        Ty::Period(_)
        | Ty::Bool
        | Ty::Text
        | Ty::Date
        | Ty::Dates
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
        | Ty::KindName => Err(wrong_number(n, ty)),
    }
}

/// `LOAD-UNIT <n> and <type> have different units (a value of the wrong type)`.
fn wrong_number(n: &crate::syntax::Number, ty: &Ty) -> Stop {
    unit_refusal(format!(
        "{} and {} have different units (a value of the wrong type)",
        n.text,
        ty.show()
    ))
}

/// `LOAD-UNIT <n> and <type> have different units`.
fn different_units(n: &crate::syntax::Number, ty: &Ty) -> Stop {
    unit_refusal(format!("{} and {} have different units", n.text, ty.show()))
}

/// A money literal in its currency, whole in its minor unit (refcore's `convert_value`, its
/// money arm). L §3.2, §7.1.
fn money_value(
    m: &Reality,
    n: &crate::syntax::Number,
    cur: &str,
    written: Option<&(Unit, Num)>,
    of: &ConvertFor,
    ty: &Ty,
) -> Result<Value, Stop> {
    let q = match written {
        None => n.value.checked_mul(&of.factor).map_err(num_stop)?,
        Some((unit, f)) => {
            if unit.terms() != [(cur.to_owned(), 1)] {
                return Err(different_units(n, ty));
            }
            n.value.checked_mul(f).map_err(num_stop)?
        }
    };
    let id = currency_id(m, cur).map_err(num_stop)?;
    num_money(m, &q, id).map(Value::Money).map_err(|_| {
        refuse_load(
            Load::Precision,
            format!("{} is finer than the minor unit", n.text),
        )
    })
}

/// A number literal's quantity in the type's unit, a keyed unit taking the keys the row
/// states (refcore's `convert_value`, its num arm). L §3.2.
fn number_in(
    n: &crate::syntax::Number,
    unit: &Unit,
    written: Option<&(Unit, Num)>,
    of: &ConvertFor,
    ty: &Ty,
) -> Result<Num, Stop> {
    let Some((from, f)) = written else {
        return n.value.checked_mul(&of.factor).map_err(num_stop);
    };
    let to = if keyed(unit) {
        unit.subst(&of.keys).map_err(unit_stop)?
    } else {
        unit.clone()
    };
    let Some(c) = from.conv(&to).map_err(num_stop)? else {
        if keyed(&to) {
            return Err(unit_refusal(format!(
                "{} gives one unit for keys of several units ({})",
                n.text,
                ty.show()
            )));
        }
        return Err(different_units(n, ty));
    };
    n.value
        .checked_mul(f)
        .and_then(|q| q.checked_mul(&c))
        .map_err(num_stop)
}

/// A calendar literal of a row as a calendar, its added days as written (refcore's `Cal`).
/// L §4.2.
fn calendar_value(cal: &CalendarLit) -> Result<Value, Stop> {
    let mut weekends = Vec::with_capacity(cal.weekends.len());
    for word in &cal.weekends {
        let n = weekday_number(word).ok_or_else(|| {
            refuse_load(
                Load::Parse,
                format!("{word} is not a weekday (mon tue wed thu fri sat sun)"),
            )
        })?;
        weekends.push(n);
    }
    let mut added = Vec::with_capacity(cal.added.len());
    for day in &cal.added {
        added.push(match day {
            CalendarDay::Day(d) => Added::Day(*d),
            CalendarDay::Hole(q) => Added::Hole(q.clone()),
            CalendarDay::Set(DaySet::Window(a, b)) => Added::Window(*a, *b),
            CalendarDay::Set(DaySet::OneOf(days)) => {
                let mut days = days.clone();
                days.sort();
                days.dedup();
                Added::OneOf(days)
            }
        });
    }
    let from = match &cal.yearly_from {
        None => None,
        // a first year past 9999, of any size, comes after every day there is (L §2.1)
        Some(n) => Some(
            FirstYear::of(n)
                .ok_or_else(|| refuse_load(Load::Unit, "a calendar's first year".to_owned()))?,
        ),
    };
    Ok(Value::Calendar(Box::new(Calendar::new(
        weekends,
        cal.yearly.clone(),
        from,
        added,
    ))))
}
