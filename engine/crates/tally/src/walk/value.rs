//! The walk's `value` read: a place's positions at the end of a day, each at its price in a currency.
//! Spec: L §5.10 (`value(x, d) in CUR [by p]`: a position in CUR at its amount, one in another
//! unit at its price, `by p` an input or def of type `num CUR/u` keyed by unit or an expression
//! of that type written with its keys, `by last(px)` through its carry; A priced place, priced
//! per unit or whole; `HOLE-PRICE`, `LOAD-PRICE`), §3.2 (per-unit prices of a unit kind), §3.4
//! (a price's year), §4.6 (a price read through its carry), §8.1 (`HOLE-PRICE`'s question).
//! Twin: engine/tools/refcore/mreads.py (`m_e_value`, `m_priced_read`, `m_price_read`,
//! `m_last_of`).
//! Invariants: `value` multiplies exactly and never rounds; a position with no price is a hole
//! naming it and its price, never left in its own unit; a price of another unit than CUR per
//! the position's unit (a place priced whole: per its currency) is refused, never multiplied.

use super::plan::invariant;
use super::point::At;
use super::reads::refuse;
use crate::base::{Code, Load, Refusal, Run};
use crate::eval::{Ctx, Engine, Env};
use crate::hole::{Hole, spread};
use crate::model::{Domain, KeyVal, Priced, Reality, Value};
use crate::num::Num;
use crate::syntax::{Expr, Price, Stop, Valuation};
use crate::time::Day;

impl Engine {
    /// `value(x, d) in CUR [by p]`: the place's positions at the end of day d at their prices
    /// in CUR, multiplied exactly: a position in CUR at its amount, a place priced whole at its
    /// price input, every other position at its unit's price; a hole naming a position and
    /// its price where it has none (`HOLE-PRICE`), never left in its own unit (refcore's
    /// `m_e_value`). L §5.10 (Value, A priced place), §8.1.
    ///
    /// # Errors
    ///
    /// `LOAD-PRICE`, `LOAD-UNIT`, `LOAD-KEY` for a price load lets by and the run meets;
    /// `RUN-DOMAIN` for a price in another unit, `RUN-NONE` for a written price that is none;
    /// the refusals of the walk; `NotBuilt` for `of q` and a price clause with keys.
    pub(crate) fn valuation(
        &mut self,
        m: &Reality,
        read: &Valuation,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let place = match self.place_arg(m, &read.place, env, ctx)? {
            Err(h) => return Ok(Value::Hole(h)),
            Ok(p) => p,
        };
        let thing = m
            .things
            .get(&place)
            .ok_or_else(|| invariant("a place that is no thing"))?;
        let d = self.ev(m, &read.day, env, ctx)?;
        if let Value::Hole(_) = d {
            return Ok(d);
        }
        let day = match crate::eval::one_day(m, &d, "the day of value")? {
            Ok(day) => day,
            Err(h) => return Ok(Value::Hole(h)),
        };
        let valued = Valued {
            place: place.clone(),
            day,
            priced: m.price_of(thing),
            by: read.by.as_ref().map(|by| by_form(m, env, by)),
        };
        if read.of.is_some() {
            return Err(Stop::NotBuilt("value of a party or a batch (M5a)"));
        }
        if let Some(price) = &valued.priced
            && let Some((id, Priced::Whole(whole))) = m.clause_mode(price)
        {
            return self.value_whole(m, (read, &valued), (price, &id, &whole), (env, ctx));
        }
        // a place of unit any: the units its log holds, in the walk the read sees
        let units = if let Some(u) = m.place_unit(thing).filter(|u| u != "any") {
            vec![u]
        } else {
            let id = match ctx.at {
                Some(at) => at.walk,
                None => self.main_walk(m, ctx.cut)?,
            };
            self.walk(id)?.units_of(&place)
        };
        let cur = read.currency.as_str();
        let mut total = Num::zero();
        let mut holes = Vec::new();
        for u in units {
            let q = self.bal_day(m, (&place, &u), day, At::end(day), ctx)?;
            if let Value::Hole(h) = q {
                holes.push(h);
                continue;
            }
            let (q, _) = super::plan::quantity(m, &q)?;
            if u == cur {
                total = total.checked_add(&q).map_err(crate::eval::num_stop)?;
                continue;
            }
            if q.is_zero() {
                continue;
            }
            match self.position_price(m, (read, &valued), &u, env, ctx)? {
                Err(h) => holes.push(h),
                Ok(p) => {
                    let worth = q.checked_mul(&p).map_err(crate::eval::num_stop)?;
                    total = total.checked_add(&worth).map_err(crate::eval::num_stop)?;
                }
            }
        }
        if let Some(h) = spread(&holes.iter().map(Some).collect::<Vec<_>>()) {
            return Ok(Value::Hole(h));
        }
        in_currency(cur, total)
    }

    /// A place priced whole, `priced p` with p a value in a currency: p at the value's day,
    /// in CUR through `by p` when p is in another, priced as one unit of a position in p's
    /// currency is, in every form `by` takes and with its unit checked (refcore's `m_e_value`,
    /// its `whole` arm; pick P53, #126). L §5.10 (Value, A priced place).
    fn value_whole(
        &mut self,
        m: &Reality,
        (read, valued): (&Valuation, &Valued),
        (price, id, whole): (&Price, &str, &str),
        (env, ctx): (&Env, Ctx),
    ) -> Result<Value, Stop> {
        let amount = match self.priced_read(m, (read, valued), (price, id), None, ctx)? {
            Err(h) => return Ok(Value::Hole(h)),
            Ok(q) => q,
        };
        let cur = read.currency.as_str();
        if whole == cur {
            return in_currency(cur, amount);
        }
        if valued.by.is_none() {
            return Err(Stop::Refused(Refusal::new(
                Code::Load(Load::Price),
                format!(
                    "value({}) in {cur} holds units in {whole} and states no by",
                    valued.place
                ),
            )));
        }
        match self.position_price(m, (read, valued), whole, env, ctx)? {
            Err(h) => Ok(Value::Hole(h)),
            Ok(p) => {
                let worth = amount.checked_mul(&p).map_err(crate::eval::num_stop)?;
                in_currency(cur, worth)
            }
        }
    }

    /// The price in CUR of one unit `u` of a position on the value's day: by the `by` written,
    /// else by the place's price clause (refcore's `m_e_value`, its loop). L §5.10.
    fn position_price(
        &mut self,
        m: &Reality,
        (read, valued): (&Valuation, &Valued),
        u: &str,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Result<Num, Hole>, Stop> {
        let cur = read.currency.as_str();
        let (place, day) = (valued.place.as_str(), valued.day);
        match (&valued.by, &read.by) {
            (Some(By::Written), Some(by)) => {
                // an expression of type num CUR/u with its keys, read at the value's day
                let at = Ctx {
                    law: Some(day),
                    ..ctx
                };
                let p = if by.last {
                    self.last_read(m, (&by.input, &read.day), env, at)?
                } else {
                    self.ev(m, &by.input, env, at)?
                };
                if let Value::Hole(h) = p {
                    return Ok(Err(h));
                }
                if p == Value::None {
                    let text = format!("the price of {u} in {cur} on {} is none here", day.text());
                    return Err(refuse(Run::None, text));
                }
                per_unit_price(m, &p, (place, cur, u)).map(Ok)
            }
            (Some(By::UnkeyedLast(pid)), _) => {
                // one unit's price: the position's unit must be its unit
                let p = self.price_at(m, (pid, Vec::new()), (day, true), ctx)?;
                if matches!(p, Value::Hole(_) | Value::None) {
                    return priced_value(m, (read, valued), p, &format!("{pid}[]"), u);
                }
                per_unit_price(m, &p, (place, cur, u)).map(Ok)
            }
            (Some(By::Keyed(pid, last)), _) => {
                self.named_price(m, (read, valued), (pid, *last), u, ctx)
            }
            (Some(By::Written), None) | (None, _) => {
                self.priced_position(m, (read, valued), u, ctx)
            }
        }
    }

    /// A position in unit `u` of a place with no `by`: at the place's own price clause, which
    /// prices `u` in CUR (refcore's `m_e_value`, its `priced` arm). L §5.10.
    fn priced_position(
        &mut self,
        m: &Reality,
        (read, valued): (&Valuation, &Valued),
        u: &str,
        ctx: Ctx,
    ) -> Result<Result<Num, Hole>, Stop> {
        let (cur, place) = (read.currency.as_str(), valued.place.as_str());
        let Some(price) = &valued.priced else {
            return Err(Stop::Refused(Refusal::new(
                Code::Load(Load::Price),
                format!("value({place}) in {cur} holds units in {u} and states no by"),
            )));
        };
        let Some((id, Priced::PerUnit(pc, per))) = m.clause_mode(price) else {
            return Err(invariant("a place priced by no price load let by"));
        };
        // a clause keyed by a kind of units (`num PLN/u`, its unit `$u`) takes the unit of the
        // key it is read at, the position's, when that kind holds it (L §3.2; pick P61, #130)
        let keyed_by_unit = per.starts_with('$')
            && matches!(m.first_domain(&id), Some(Domain::Kind(kind)) if unit_in(m, Some(u), &kind));
        let per = if keyed_by_unit { u } else { per.as_str() };
        if pc != cur || per != u {
            return Err(Stop::Refused(Refusal::new(
                Load::Unit.at_run(),
                format!("value({place}) in {cur}: {place} is priced in {pc}/{per}, not {cur}/{u}"),
            )));
        }
        self.priced_read(m, (read, valued), (price, &id), Some(u), ctx)
    }

    /// A price named by `by p` or `by last(p)`, an input or def keyed by unit, read at unit
    /// (or currency) `u` on the value's day; a hole naming `u` where it is `none` (refcore's
    /// `m_price_read`). L §5.10, §4.6, §8.1.
    fn named_price(
        &mut self,
        m: &Reality,
        (read, valued): (&Valuation, &Valued),
        (pid, last): (&str, bool),
        u: &str,
        ctx: Ctx,
    ) -> Result<Result<Num, Hole>, Stop> {
        let (given, figure) = if key_count(m, pid).is_some_and(|n| n != 0) {
            (
                vec![Value::Key(KeyVal::Thing(u.to_owned()))],
                format!("{pid}[{u}]"),
            )
        } else {
            (Vec::new(), format!("{pid}[]"))
        };
        let p = self.price_at(m, (pid, given), (valued.day, last), ctx)?;
        if matches!(p, Value::Hole(_) | Value::None) {
            return priced_value(m, (read, valued), p, &figure, u);
        }
        // load saw the price's currency; the run sees its unit at the position's key
        // (L §3.2: a price read at another key than the quantity's is refused, RUN-DOMAIN)
        per_unit_price(m, &p, (valued.place.as_str(), read.currency.as_str(), u)).map(Ok)
    }

    /// A place's own price clause read on the value's day: at the place's thing, or at its
    /// position's unit `u`, when the price input is keyed by a kind holding one; a hole naming
    /// the place or `u` where it is `none` (refcore's `m_priced_read`). L §5.10, §4.6, §8.1.
    fn priced_read(
        &mut self,
        m: &Reality,
        (read, valued): (&Valuation, &Valued),
        (price, pid): (&Price, &str),
        u: Option<&str>,
        ctx: Ctx,
    ) -> Result<Result<Num, Hole>, Stop> {
        if !matches!(price.input, Expr::Name(_)) {
            return Err(Stop::NotBuilt("a price clause written with its keys"));
        }
        let place = valued.place.as_str();
        let key = match m.first_domain(pid) {
            None => None,
            Some(Domain::Kind(kind)) if m.thing_in(place, &kind) => Some(place),
            Some(Domain::Kind(kind)) if unit_in(m, u, &kind) => u,
            Some(domain) => {
                let over = match &domain {
                    Domain::Kind(id) | Domain::Index(id) | Domain::Axis(id) => id.as_str(),
                    Domain::List(_) => "a list of keys",
                    Domain::Range => "a stepped range",
                };
                return Err(Stop::Refused(Refusal::new(
                    Code::Load(Load::Key),
                    format!(
                        "{place} is priced by {pid}, keyed by {over}, which holds neither {place} nor its unit"
                    ),
                )));
            }
        };
        let (given, figure) = match key {
            Some(k) => (
                vec![Value::Key(KeyVal::Thing(k.to_owned()))],
                format!("{pid}[{k}]"),
            ),
            None => (Vec::new(), format!("{pid}[]")),
        };
        let p = self.price_at(m, (pid, given), (valued.day, price.last), ctx)?;
        let who = match u {
            Some(u) => u,
            None => place,
        };
        priced_value(m, (read, valued), p, &figure, who)
    }
}

/// What a `value` reads besides the read as written: its place, its day, the place's price
/// clause (on its entry or its kind, L §5.10) and the form of the read's `by`.
struct Valued {
    /// The place.
    place: String,
    /// The day.
    day: Day,
    /// The place's price clause.
    priced: Option<Price>,
    /// The form of the read's `by p`.
    by: Option<By>,
}

/// How a value's `by p` reads a price (refcore's `n._price`, `_price_kind`,
/// `_price_unkeyed`, `_price_expr`, which load's `m_t_value` sets). L §5.10.
enum By {
    /// `by last(px)`, px an unkeyed input: one unit's price, through its carry.
    UnkeyedLast(String),
    /// `by px` or `by last(px)`, px an input or def keyed by unit: read at the position's
    /// unit, through its carry with `last`.
    Keyed(String, bool),
    /// An expression of type `num CUR/u` written with its keys, `by px[month_of(d)]`, or
    /// `by last(px[k])`: read at the value's day, as the read writes it.
    Written,
}

/// The form of a `by p` (load's `Checker::by`, at run). L §5.10.
fn by_form(m: &Reality, env: &Env, by: &Price) -> By {
    if let Expr::Name(id) = &by.input
        && env.get(id).is_none()
    {
        match key_count(m, id) {
            Some(0) if by.last => return By::UnkeyedLast(id.clone()),
            Some(n) if n != 0 => return By::Keyed(id.clone(), by.last),
            Some(_) | None => {}
        }
    }
    By::Written
}

/// How many keys an input or a def takes; none for another id. L §5.8, §5.11.
fn key_count(m: &Reality, id: &str) -> Option<usize> {
    if let Some(input) = m.inputs.get(id) {
        return Some(input.binds.len());
    }
    m.defs.get(id).map(|def| def.binds.len())
}

/// Whether a position's unit `u` is a thing of `kind`, for a price keyed by that kind.
/// L §5.7, §5.10.
fn unit_in(m: &Reality, u: Option<&str>, kind: &str) -> bool {
    let Some(u) = u else {
        return false;
    };
    m.things.get(u).is_some() && m.thing_in(u, kind)
}

/// A price read's quantity; `none` a hole naming the position and the price at its keys,
/// `figure` (refcore's `no price` hole). L §5.10, §8.1 `HOLE-PRICE`.
fn priced_value(
    m: &Reality,
    (read, valued): (&Valuation, &Valued),
    p: Value,
    figure: &str,
    who: &str,
) -> Result<Result<Num, Hole>, Stop> {
    if let Value::Hole(h) = p {
        return Ok(Err(h));
    }
    if p == Value::None {
        let text = format!(
            "{who} has no price in {} on {}",
            read.currency,
            valued.day.text()
        );
        return Ok(Err(Hole::asked(&text, figure)));
    }
    Ok(Ok(super::plan::quantity(m, &p)?.0))
}

/// A price written as an expression, or read unkeyed, must be in CUR per the position's unit
/// (refcore's `m_e_value`, its unit test). L §5.10.
fn per_unit_price(
    m: &Reality,
    p: &Value,
    (place, cur, u): (&str, &str, &str),
) -> Result<Num, Stop> {
    let fits = if let Value::Number(n) = p {
        let mut terms = n.unit().terms().to_vec();
        terms.sort();
        let mut want = vec![(cur.to_owned(), 1), (u.to_owned(), -1)];
        want.sort();
        terms == want
    } else {
        false
    };
    if !fits {
        let shown = p.text(m).map_err(crate::eval::num_stop)?;
        return Err(refuse(
            Run::Domain,
            format!("value({place}) in {cur}: the price {shown} is not in {cur}/{u}"),
        ));
    }
    Ok(super::plan::quantity(m, p)?.0)
}

/// A value's result: a `num CUR`, never rounded (L §5.10: `value` multiplies exactly).
fn in_currency(cur: &str, total: Num) -> Result<Value, Stop> {
    let unit = crate::model::Unit::make(vec![(cur.to_owned(), 1)])
        .map_err(|_| invariant("a currency's unit"))?;
    Ok(Value::Number(crate::model::Number::plain(total, unit)))
}
