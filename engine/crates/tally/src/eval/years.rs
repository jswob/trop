//! The written year forms: a four-digit figure the checker read as a year, met at run as the year it names.
//! Spec: L §2.1 (a four-digit literal is a year where a period is expected, an int elsewhere;
//! a parenthesised literal is that literal; an `if`'s arms take the type of the place the `if`
//! stands in, tally 2.1.6 and the ruling on #113), §2.5 (a key literal its position resolves),
//! §4.1 (periods), §4.2 (period arguments).
//! Twin: engine/tools/refcore/evaluate.py (`e_num`'s `_year` and period readings) and check.py
//! (`ambig`, `compare`, `span`, `tdate_or_period`, which set the marks).
//! Invariants: load lets a whole number reach a place that reads a year only as a year, so the
//! run reads one as a year exactly where its written form is a year form and the place takes
//! a period, or where it ends a span; a value that names no calendar year is left as it is,
//! for the place to refuse.

use super::state::invariant;
use crate::model::{Class, Number, Value};
use crate::syntax::{Expr, Lit, Number as Written, Stop};
use crate::time::{Period, PeriodKind, Step};

/// Whether `e` is written in a year form: a four-digit figure, or an `if` with one among its
/// arms, at any depth of `if`s and brackets (L §2.1).
pub(crate) fn year_form(e: &Expr) -> bool {
    let mut work = vec![e];
    while let Some(node) = work.pop() {
        if let Expr::Lit(Lit::Num(n)) = node
            && four_digit(n)
        {
            return true;
        }
        if let Expr::If(parts) = node {
            let [_, yes, no] = parts.as_ref();
            work.push(yes);
            work.push(no);
        }
        if let Expr::Paren(inner) = node {
            work.push(inner);
        }
    }
    false
}

/// The year that value `v` of expression `e` names where a period is read: `Some` when `e`
/// is written in a year form and `v` is an int that is a calendar year (L §2.1).
pub(super) fn as_year(e: Option<&Expr>, v: &Value) -> Option<Period> {
    if !year_form(e?) {
        return None;
    }
    int_year(v)
}

/// Calendar year `year` as the key it labels on the grid of `like`, a key of a year index:
/// the period of `like`'s index whose first day falls in that year, an anchored year
/// included (`index cy = year 2025..2026 from 02-01`: 2026 is 2026-02-01 to 2027-01-31); the
/// calendar year itself beside any other period, or where no key of that grid starts in it
/// (refcore's `resolve_key_literal` of a year literal beside a key of a year index). L §2.1,
/// §2.5, §4.1.
pub(super) fn year_like(like: Period, year: Period) -> Period {
    if like.kind() != PeriodKind::Year {
        return year;
    }
    let Step::Months(n) = like.step() else {
        return year;
    };
    let years = i64::from(year.start().year()).checked_sub(i64::from(like.start().year()));
    let months = years.and_then(|y| y.checked_mul(12));
    let steps = months.and_then(|k| {
        let n = i64::from(n);
        (k.checked_rem_euclid(n)? == 0).then_some(k.checked_div_euclid(n)?)
    });
    steps.and_then(|k| like.shift(k)).unwrap_or(year)
}

/// The year an int names at a span's end. Load lets an int stand there only as a year: a
/// figure in a year form, or a key-list variable of four-digit keys (refcore's `span_value`,
/// and the `_year` marks of `span` and `kl_value`). L §2.1, §2.5, §4.1.
pub(super) fn span_year(v: &Value) -> Option<Period> {
    int_year(v)
}

/// The year that a whole-number value names, or `None` for any other value: an int past the
/// calendar's years names none, and the place that reads it refuses it. L §2.1, §4.1.
fn int_year(v: &Value) -> Option<Period> {
    let Value::Number(Number {
        q,
        class: Class::Int,
    }) = v
    else {
        return None;
    };
    let Ok(year) = u16::try_from(q.to_i64()?) else {
        return None;
    };
    Period::year(year)
}

/// An int value `v` of an expression written in a year form, as the key literal its digits
/// write: the literal the key's position resolves, as it resolves the bare figure (L §2.1,
/// §2.5). `None` for an expression in no year form, or a value that is no int.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for an int whose digits don't print.
pub(super) fn year_literal(e: &Expr, v: &Value) -> Result<Option<Lit>, Stop> {
    if !year_form(e) {
        return Ok(None);
    }
    let Value::Number(Number {
        q,
        class: Class::Int,
    }) = v
    else {
        return Ok(None);
    };
    let text = q
        .integer_text()
        .ok_or_else(|| invariant("an int whose digits don't print"))?;
    let four = text.len() == 4 && text.bytes().all(|b| b.is_ascii_digit());
    Ok(Some(Lit::Num(Written {
        value: q.clone(),
        whole: true,
        unit: None,
        text,
        four,
        fraction: false,
    })))
}

/// Whether a written number is a four-digit figure with no unit (L §2.1).
fn four_digit(n: &Written) -> bool {
    n.whole && n.four && n.unit.is_none()
}
