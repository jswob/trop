//! A band's call `b(x)` or `b[k](x)`: the table the read rule takes, the row whose interval holds x, and that row's value with x bound.
//! Spec: L §5.9 (bands: rows by interval, every bare literal in a row a figure, a keyed band's
//! key), §4.6 (the read rule over tables), §8.1 (`HOLE-UNGOVERNED`: no table, no row), §8.2
//! (`assumed` and `held` of a band row).
//! Twin: engine/tools/refcore/evaluate.py (`call_band`, `band_key`, `band_value`, `band_x`,
//! `interval_text`) and model.py (`BandRow.holds`).
//! Invariants: the row is the one whose interval holds the argument's value in the
//! argument's unit; a row's value of a money band is money, a bare figure takes the band's
//! type; a hole argument or key makes the call that hole.

use core::cmp::Ordering;

use super::expr::{hole_of, typed};
use super::read::{Given, key_value, make_keys};
use super::state::{Ctx, Engine, Env, invariant, num_stop, refuse, refuse_load, wrong_type};
use crate::base::{Load, Run};
use crate::hole::Hole;
use crate::model::{
    BandEnd, BandInfo, BandRow, BandTable, Class, KeyVal, Number, Reality, Ty, Value, currency_id,
    money_num,
};
use crate::num::{Money, Num};
use crate::read::{BandAt, band_no_row, band_no_table, band_origin, band_rows, table_held};
use crate::syntax::{Call, Expr, Key, Lit, RuleValue, Stop};
use crate::time::Day;

impl Engine {
    /// `b(x)`, `b[k](x)`: the band's value at x (refcore's `call_band`). L §5.9.
    pub(super) fn band_call(
        &mut self,
        m: &Reality,
        call: &Call,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let band = m
            .bands
            .get(&call.name)
            .ok_or_else(|| invariant("a band call of a band load didn't build"))?;
        let key = match self.band_key(m, band, call, env, ctx)? {
            Err(hole) => return Ok(Value::Hole(hole)),
            Ok(key) => key,
        };
        let [arg] = call.args.as_slice() else {
            return Err(invariant("a band called with other than one argument"));
        };
        let x = self.ev(m, arg, env, ctx)?;
        if let Value::Hole(_) = x {
            return hole_of(&[x.as_hole()]);
        }
        if x == Value::None {
            return Err(refuse(
                Run::None,
                "a value is none here: a band argument".to_owned(),
            ));
        }
        let figure = band_origin(&band.id, key.as_ref());
        let Some(rows) = band_rows(m, band, key.as_ref(), self.clocks(ctx))? else {
            let q = band_no_table(&band.id, key.as_ref(), ctx.law);
            return Ok(Value::Hole(Hole::asked(&q.text, &figure)));
        };
        let point = band_x(m, &x)?;
        for at in rows {
            let (_, row) = band_row(band, at)?;
            if holds(row, &point)? {
                return self.band_value(m, band, &figure, at, x, ctx);
            }
        }
        let q = band_no_row(
            &band.id,
            key.as_ref(),
            &x.text(m).map_err(num_stop)?,
            ctx.law,
        );
        Ok(Value::Hole(Hole::asked(&q.text, &q.origin)))
    }

    /// A keyed band's key, made a key of its bind's domain; a hole key is the call's hole
    /// (refcore's `band_key`). L §5.9, §2.5.
    fn band_key(
        &mut self,
        m: &Reality,
        band: &BandInfo,
        call: &Call,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Result<Option<KeyVal>, Hole>, Stop> {
        if band.binds.is_empty() {
            return Ok(Ok(None));
        }
        let Some(Key::Expr(e)) = &call.band_key else {
            return Err(invariant("a keyed band called with no key"));
        };
        let given: Given = self.given_key(m, e, env, ctx)?;
        Ok(make_keys(m, &band.id, &band.binds, vec![given], ctx.cut)?
            .map(|keys| keys.into_iter().next()))
    }

    /// The row's value with the band's argument bound, and its register entries (refcore's
    /// `band_value`). L §5.9, §8.2.
    fn band_value(
        &mut self,
        m: &Reality,
        band: &BandInfo,
        figure: &str,
        at: BandAt,
        x: Value,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let (table, row) = band_row(band, at)?;
        let version = m
            .versions
            .get(&table.version)
            .ok_or_else(|| invariant("a band table of a version load didn't build"))?;
        if let Some(reason) = &version.assumed {
            self.note("assumed", reason.clone(), figure.to_owned());
        }
        if table_held(m, table, self.clocks(ctx))? {
            let law = ctx.law.map_or_else(String::new, Day::text);
            self.note(
                "held",
                format!(
                    "{figure} row {} ({}), read for {law}",
                    interval_text(row)?,
                    table.version
                ),
                String::new(),
            );
        }
        let expr = match &row.value {
            RuleValue::Hole(text) => return Ok(Value::Hole(Hole::asked(text, figure))),
            RuleValue::Expr(e) => e,
        };
        let v = self.ev(m, expr, &Env::band(&band.arg, x), ctx)?;
        band_typed(m, band, expr, v)
    }
}

/// The table and row a candidate names.
fn band_row(band: &BandInfo, at: BandAt) -> Result<(&BandTable, &BandRow), Stop> {
    let table = band
        .tables
        .get(at.table)
        .ok_or_else(|| invariant("a band row of a table the band lacks"))?;
    let row = table
        .rows
        .get(at.row)
        .ok_or_else(|| invariant("a band row the table lacks"))?;
    Ok((table, row))
}

/// The value a row's interval is tested on: a number's value in its unit, money's in its
/// currency's units (refcore's `band_x`). L §5.9.
fn band_x(m: &Reality, x: &Value) -> Result<Num, Stop> {
    if let Value::Number(n) = x {
        return Ok(n.q.clone());
    }
    if let Value::Money(money) = x {
        return money_num(m, money).map_err(num_stop);
    }
    if let Value::Day(_) | Value::Period(_) | Value::Key(_) = x {
        return Err(Stop::NotBuilt("a band over days, periods or keys"));
    }
    Err(wrong_type(m, "a band argument", x))
}

/// Whether the row's interval holds x (refcore's `BandRow.holds`). L §5.9.
fn holds(row: &BandRow, x: &Num) -> Result<bool, Stop> {
    if let Some(BandEnd::Num(lo)) = &row.lo {
        let order = x.compare(lo).map_err(num_stop)?;
        if order == Ordering::Less || (order == Ordering::Equal && !row.lo_closed) {
            return Ok(false);
        }
    }
    if let Some(BandEnd::Num(hi)) = &row.hi {
        let order = x.compare(hi).map_err(num_stop)?;
        if order == Ordering::Greater || (order == Ordering::Equal && !row.hi_closed) {
            return Ok(false);
        }
    }
    Ok(true)
}

/// A row's interval as the register writes it, `[0, 120 000)` (refcore's `interval_text`).
/// L §8.2.
fn interval_text(row: &BandRow) -> Result<String, Stop> {
    let end = |e: &Option<BandEnd>, inf: &str| -> Result<String, Stop> {
        match e {
            None => Ok(inf.to_owned()),
            Some(BandEnd::Num(q)) => q.to_text().map_err(num_stop),
        }
    };
    Ok(format!(
        "{}{}, {}{}",
        if row.lo_closed { "[" } else { "(" },
        end(&row.lo, "-inf")?,
        end(&row.hi, "inf")?,
        if row.hi_closed { "]" } else { ")" },
    ))
}

/// A row's value in the band's type: a bare figure takes it, a key literal is a key of a
/// `key` band, money's zero stands for a money band, a money value of a `num` band is its
/// number (refcore's `band_value`, check.py's fit of a row's value). L §5.9, §3.2.
fn band_typed(m: &Reality, band: &BandInfo, expr: &Expr, v: Value) -> Result<Value, Stop> {
    if let Value::Number(Number {
        q,
        class: Class::Flex,
    }) = &v
        && bare_figure(expr)
    {
        return Ok(typed(m, q, &band.ty)?.unwrap_or(v));
    }
    if let (Ty::Key(ix), Value::Word(word)) = (&band.ty, &v) {
        return key_value(m, word, ix);
    }
    if let (Ty::Money(cur), Value::Number(n)) = (&band.ty, &v) {
        if matches!(n.class, Class::Int | Class::Flex) && n.q.is_zero() {
            let id = currency_id(m, cur).map_err(num_stop)?;
            return Ok(Value::Money(Money::from_minor(0, id)));
        }
        return Err(refuse_load(
            Load::Unit,
            format!("a num stored as money in band {}", band.id),
        ));
    }
    if let (Ty::Num(..), Value::Money(money)) = (&band.ty, &v) {
        let q = money_num(m, money).map_err(num_stop)?;
        let unit = crate::model::currency_unit(m, money.currency()).map_err(num_stop)?;
        return Ok(Value::Number(Number::plain(q, unit)));
    }
    Ok(v)
}

/// Whether a row's value is a bare figure, through brackets and a minus (check.py's
/// `flex_to`). L §3.2.
fn bare_figure(e: &Expr) -> bool {
    let mut at = e;
    while let Expr::Paren(x) | Expr::Neg(x) = at {
        at = x;
    }
    if let Expr::Lit(Lit::Num(n)) = at {
        return n.unit.is_none();
    }
    false
}
