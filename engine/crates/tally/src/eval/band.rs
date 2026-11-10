//! A band's call `b(x)`: the table the read rule takes, the row holding x and its value with x bound; `marginal(b, x)`, the rows below x by width.
//! Spec: L §5.9 (bands: rows by interval, every bare literal in a row a figure, a keyed band's
//! key; `marginal`), §4.6 (the read rule over tables), §8.1 (`HOLE-UNGOVERNED`: no table, no
//! row), §8.2 (`assumed` and `held` of a band row).
//! Twin: engine/tools/refcore/evaluate.py (`call_band`, `band_key`, `band_rows`,
//! `band_no_row`, `band_value`, `band_x`, `c_marginal`, `interval_text`) and model.py
//! (`BandRow.holds`).
//! Invariants: the row is the one whose interval holds the argument's value in the
//! argument's unit; a row's value of a money band is money, a bare figure takes the band's
//! type; a hole argument or key makes the call that hole; `marginal` reads every row whose
//! lower end is below x, each with x bound, and is a hole if any of their values is.

use core::cmp::Ordering;

use super::convert::key_value;
use super::expr::{hole_of, typed};
use super::keys::Given;
use super::memo::{LawClass, Read, may_hold, tables_class};
use super::state::{
    Ctx, Engine, Env, Rows, invariant, num_stop, refuse, refuse_load, unit_stop, wrong_type,
};
use crate::base::{Load, Run};
use crate::hole::Hole;
use crate::model::{
    BandEnd, BandInfo, BandRow, BandTable, Class, KeyVal, Node, NodeKind, Number, Reality, Ty,
    Unit, Value, currency_id, currency_unit, money_num,
};
use crate::num::{Money, Num};
use crate::read::{BandAt, band_no_row, band_no_table, band_origin, band_rows, table_held};
use crate::syntax::{Call, Expr, Key, Lit, Postfix, RuleValue, Stop};
use crate::time::Day;

/// What a band read at x finds: its key, x, and the rows the read rule takes, in order.
struct BandRead {
    /// The key of a keyed band.
    key: Option<KeyVal>,
    /// The argument.
    x: Value,
    /// The rows.
    rows: Vec<BandAt>,
}

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
        let [arg] = call.args.as_slice() else {
            return Err(invariant("a band called with other than one argument"));
        };
        let BandRead { key, x, rows } =
            match self.band_rows_at(m, band, call.band_key.as_ref(), arg, env, ctx)? {
                Err(hole) => return Ok(Value::Hole(hole)),
                Ok(found) => found,
            };
        let figure = band_origin(&band.id, key.as_ref());
        let point = band_x(m, &x)?;
        for at in rows {
            let (_, row) = band_row(band, at)?;
            if holds(row, &point)? {
                self.read(Read::BandRow {
                    band: band.id.clone(),
                    table: at.table,
                    row: at.row,
                });
                return self.band_value(m, band, &figure, at, x, ctx);
            }
        }
        Ok(Value::Hole(self.no_row(m, band, key.as_ref(), &x, ctx)?))
    }

    /// `marginal(b, x)`: the sum over the rows whose lower end is below x of the row's value
    /// times the part of its interval below x, `(min(x, hi) − lo)`, a `num` in the unit of x
    /// times the rows'; 0 in that unit where no row starts below x (refcore's `c_marginal`).
    /// L §5.9.
    ///
    /// # Errors
    ///
    /// `LOAD-BAND` for a first row from `-inf`; `LOAD-UNIT` for a row whose value is no
    /// number; `RUN-NONE` for a row that is `none`.
    pub(super) fn marginal(
        &mut self,
        m: &Reality,
        call: &Call,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let [written, arg] = call.args.as_slice() else {
            return Err(invariant("marginal with other than two arguments"));
        };
        let (id, key_written) = band_written(written)?;
        let band = m
            .bands
            .get(id)
            .ok_or_else(|| invariant("marginal of a band load didn't build"))?;
        let BandRead { key, x, rows } =
            match self.band_rows_at(m, band, key_written, arg, env, ctx)? {
                Err(hole) => return Ok(Value::Hole(hole)),
                Ok(found) => found,
            };
        let figure = band_origin(&band.id, key.as_ref());
        let first_open = if let Some(at) = rows.first() {
            band_row(band, *at)?.1.lo.is_none()
        } else {
            false
        };
        if first_open {
            return Err(refuse_load(
                Load::Band,
                format!("marginal({}, x) over a first row from -inf", band.id),
            ));
        }
        let point = band_x(m, &x)?;
        let mut held_somewhere = false;
        for at in &rows {
            held_somewhere |= holds(band_row(band, *at)?.1, &point)?;
        }
        if !held_somewhere {
            return Ok(Value::Hole(self.no_row(m, band, key.as_ref(), &x, ctx)?));
        }
        let x_unit = quantity(m, &x)?.1;
        let mut acc: Option<(Num, Unit)> = None;
        let mut holes: Vec<Hole> = Vec::new();
        for at in rows {
            let (_, row) = band_row(band, at)?;
            let Some(width) = width_below(row, &point)? else {
                continue;
            };
            self.read(Read::BandRow {
                band: band.id.clone(),
                table: at.table,
                row: at.row,
            });
            let v = self.band_value(m, band, &figure, at, x.clone(), ctx)?;
            if let Value::Hole(h) = v {
                holes.push(h);
                continue;
            }
            if v == Value::None {
                return Err(refuse(
                    Run::None,
                    "a value is none here: marginal".to_owned(),
                ));
            }
            let (q, unit) = quantity(m, &v)?;
            let (unit, factor) = x_unit.mul(&unit).map_err(unit_stop)?;
            let term = q
                .checked_mul(&width)
                .and_then(|t| t.checked_mul(&factor))
                .map_err(num_stop)?;
            let sum = match acc {
                None => term,
                Some((held, _)) => held.checked_add(&term).map_err(num_stop)?,
            };
            acc = Some((sum, unit));
        }
        if !holes.is_empty() {
            let operands: Vec<Option<&Hole>> = holes.iter().map(Some).collect();
            return hole_of(&operands);
        }
        let Some((q, unit)) = acc else {
            let (unit, _) = x_unit.mul(&ty_unit(m, &band.ty)?).map_err(unit_stop)?;
            return Ok(Value::Number(Number::plain(Num::zero(), unit)));
        };
        Ok(Value::Number(Number::plain(q, unit)))
    }

    /// The key, the argument and the rows the read rule takes of a band read at `arg`; the
    /// hole of a key or an argument that is one, or of no table (refcore's `band_ref_key`,
    /// `band_rows`). L §5.9, §4.6 step 5.
    fn band_rows_at(
        &mut self,
        m: &Reality,
        band: &BandInfo,
        key: Option<&Key>,
        arg: &Expr,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Result<BandRead, Hole>, Stop> {
        let key = match self.band_key(m, band, key, env, ctx)? {
            Err(hole) => return Ok(Err(hole)),
            Ok(key) => key,
        };
        let x = self.ev(m, arg, env, ctx)?;
        if let Value::Hole(h) = x {
            return Ok(Err(h));
        }
        if x == Value::None {
            return Err(refuse(
                Run::None,
                "a value is none here: a band argument".to_owned(),
            ));
        }
        self.read(Read::Band {
            id: band.id.clone(),
            key: key.clone(),
        });
        if let Some(day) = ctx.law {
            let class = tables_class(m, band, ctx.cut, day)?;
            self.narrow(ctx.law, class);
        }
        let Some(rows) = band_rows(m, band, key.as_ref(), self.clocks(ctx, &band.id)?)? else {
            self.narrow_to_day(ctx.law);
            let q = band_no_table(&band.id, key.as_ref(), ctx.law);
            let figure = band_origin(&band.id, key.as_ref());
            return Ok(Err(Hole::asked(&q.text, &figure)));
        };
        Ok(Ok(BandRead { key, x, rows }))
    }

    /// `HOLE-UNGOVERNED` for a band whose table holds no row for x (refcore's `band_no_row`).
    /// L §5.9, §8.1.
    fn no_row(
        &mut self,
        m: &Reality,
        band: &BandInfo,
        key: Option<&KeyVal>,
        x: &Value,
        ctx: Ctx,
    ) -> Result<Hole, Stop> {
        self.narrow_to_day(ctx.law);
        let q = band_no_row(&band.id, key, &x.text(m).map_err(num_stop)?, ctx.law);
        Ok(Hole::asked(&q.text, &q.origin))
    }

    /// A keyed band's key, made a key of its bind's domain; a hole key is the call's hole
    /// (refcore's `band_key`). L §5.9, §2.5.
    fn band_key(
        &mut self,
        m: &Reality,
        band: &BandInfo,
        key: Option<&Key>,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Result<Option<KeyVal>, Hole>, Stop> {
        if band.binds.is_empty() {
            return Ok(Ok(None));
        }
        let Some(Key::Expr(e)) = key else {
            return Err(invariant("a keyed band called with no key"));
        };
        let given: Given = self.given_key(m, e, env, ctx)?;
        Ok(self
            .make_keys(m, &band.id, &band.binds, vec![given], ctx)?
            .map(|keys| keys.into_iter().next()))
    }

    /// Narrows the reading cell's law date class, for a band table not `held`, to the law
    /// date's side of the ask's cut where a later one would hold it (L §8.2, E §3.2).
    fn unheld_table_class(&mut self, m: &Reality, table: &BandTable, ctx: Ctx) -> Result<(), Stop> {
        let Some(day) = ctx.law else {
            return Ok(());
        };
        let span = m
            .versions
            .get(&table.version)
            .ok_or_else(|| invariant("a band table of a version load didn't build"))?
            .span_at(ctx.cut);
        if span.is_some_and(may_hold) {
            let class = LawClass::all().by_ask_cut(day, self.ask_cut);
            self.narrow(ctx.law, class);
        }
        Ok(())
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
        if self.keep_rows == Rows::Kept || self.tracing() {
            // the band row as `inputs-of` lists it and `explain` prints it (refcore's
            // `BandRowRef`, `band_value`'s trace node). E §10.
            let interval = interval_text(row)?;
            let sid = format!("{}:{figure} {interval}", table.version);
            self.note_row(&sid, &table.version, row.locator.as_ref());
            self.trace_band_row(figure, &interval, (&table.version, row.locator.as_ref()));
        }
        if table_held(m, table, self.clocks(ctx, &band.id)?)? {
            // the `held` entry names the law date (L §8.2)
            self.narrow_to_day(ctx.law);
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
        } else {
            self.unheld_table_class(m, table, ctx)?;
        }
        let expr = match &row.value {
            RuleValue::Hole(text) => return Ok(Value::Hole(Hole::asked(text, figure))),
            RuleValue::Expr(e) => e,
            RuleValue::Solve(_) => return Err(Stop::NotBuilt("`solve`")),
        };
        let outer = self.holding(Node::new(NodeKind::Band, &band.id));
        let v = self.ev(m, expr, &Env::band(&band.arg, x), ctx);
        self.holder = outer;
        band_typed(m, band, expr, v?)
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

/// The band `marginal` names, `b` or `b[k]`, and its key as written (load's `band_ref`).
/// L §5.9.
fn band_written(e: &Expr) -> Result<(&str, Option<&Key>), Stop> {
    if let Expr::Name(id) = e {
        return Ok((id, None));
    }
    if let Expr::Post(base, posts) = e
        && let (Expr::Name(id), [Postfix::Keys(keys)]) = (base.as_ref(), posts.as_slice())
        && let [key] = keys.as_slice()
    {
        return Ok((id, Some(key)));
    }
    Err(invariant("marginal of a band load didn't read"))
}

/// A number's value and unit; money's in its currency's units (refcore's `v.q`, `v.u`).
/// L §3.2, §5.9.
fn quantity(m: &Reality, v: &Value) -> Result<(Num, Unit), Stop> {
    if let Value::Number(n) = v {
        return Ok((n.q.clone(), n.unit()));
    }
    if let Value::Money(money) = v {
        let q = money_num(m, money).map_err(num_stop)?;
        let unit = currency_unit(m, money.currency()).map_err(num_stop)?;
        return Ok((q, unit));
    }
    Err(refuse_load(
        Load::Unit,
        "marginal over rows that are not numbers".to_owned(),
    ))
}

/// The part of a row's interval below x, `min(x, hi) − lo`; none for a row whose lower end
/// isn't below x (refcore's `c_marginal`). L §5.9.
fn width_below(row: &BandRow, x: &Num) -> Result<Option<Num>, Stop> {
    let Some(BandEnd::Num(lo)) = &row.lo else {
        return Ok(None);
    };
    if lo.compare(x).map_err(num_stop)? != Ordering::Less {
        return Ok(None);
    }
    let hi = match &row.hi {
        Some(BandEnd::Num(hi)) if x.compare(hi).map_err(num_stop)? != Ordering::Less => hi,
        Some(BandEnd::Num(_)) | None => x,
        Some(BandEnd::Day(_) | BandEnd::Period(_) | BandEnd::Key(..)) => {
            return Err(Stop::NotBuilt("a band over days, periods or keys"));
        }
    };
    hi.checked_sub(lo).map(Some).map_err(num_stop)
}

/// The unit of a band's `num` or `money` type; 1 for another type (refcore's `info.type.u`).
/// L §5.9.
fn ty_unit(m: &Reality, ty: &Ty) -> Result<Unit, Stop> {
    if let Ty::Num(unit, _) = ty {
        return Ok(unit.clone());
    }
    if let Ty::Money(cur) = ty {
        let id = currency_id(m, cur).map_err(num_stop)?;
        return currency_unit(m, id).map_err(num_stop);
    }
    Ok(Unit::one())
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
            Some(BandEnd::Day(_) | BandEnd::Period(_) | BandEnd::Key(..)) => {
                Err(Stop::NotBuilt("a band over days, periods or keys"))
            }
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
        return key_value(m, &Lit::Name(word.clone()), ix);
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
        let unit = currency_unit(m, money.currency()).map_err(num_stop)?;
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
