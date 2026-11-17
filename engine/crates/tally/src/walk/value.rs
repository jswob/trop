//! The walk's `value` read: a place's positions at the end of a day, each at its price in a currency.
//! Spec: L §5.10 (`value(x, d) in CUR [by p]`: a position in CUR at its amount, one in another
//! unit at its price, `by p` an input or def of type `num CUR/u` keyed by unit or an expression
//! of that type written with its keys, `by last(px)` through its carry; A priced place, priced
//! per unit or whole; `HOLE-PRICE`, `LOAD-PRICE`), §2.3 (a price `ref` with its keys and a
//! record's fields), §3.2 (per-unit prices of a unit kind), §3.4 (a price's year), §4.6 (a
//! price read through its carry), §8.1 (`HOLE-PRICE`'s question).
//! Twin: engine/tools/refcore/mreads.py (`m_e_value`, `m_priced_read`, `m_price_read`,
//! `m_last_of`).
//! Invariants: `value` multiplies exactly and never rounds; a position with no price is a hole
//! naming it and its price, never left in its own unit; a price of another unit than CUR per
//! the position's unit (a place priced whole: per its currency) is refused, never multiplied.

use super::plan::invariant;
use super::point::At;
use super::reads::refuse;
use crate::base::{Load, Refusal, Run};
use crate::eval::{Ctx, Engine, Env, Given};
use crate::hole::{Hole, Linear, spread};
use crate::model::{Domain, KeyVal, Priced, Reality, Value, price_fields};
use crate::num::Num;
use crate::syntax::{Expr, Key, Postfix, Price, Stop, Valuation};
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
    /// the refusals of the walk; `NotBuilt` for `of q`.
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
        // under `partial`, each hole position's term as a linear form (tally 2.1.8 R15)
        let mut lins: Vec<Option<Linear>> = Vec::new();
        for u in units {
            let held = self.bal_day(m, (&place, &u), day, At::end(day), ctx)?;
            if let Value::Hole(h) = held {
                holes.push(h.clone());
                if ctx.partial {
                    let lin =
                        self.hole_position(m, (read, &valued), (&u, &h), (env, ctx), &mut holes)?;
                    lins.push(lin);
                }
                continue;
            }
            let (q, _) = super::plan::quantity(m, &held)?;
            if u == cur {
                total = total.checked_add(&q).map_err(crate::eval::num_stop)?;
                continue;
            }
            if q.is_zero() {
                continue;
            }
            match self.position_price(m, (read, &valued), &u, env, ctx)? {
                Err(h) => {
                    if ctx.partial {
                        lins.push(scaled(m, h.linear_part(), &held)?);
                    }
                    holes.push(h);
                }
                Ok(p) => {
                    let worth = q.checked_mul(&p).map_err(crate::eval::num_stop)?;
                    total = total.checked_add(&worth).map_err(crate::eval::num_stop)?;
                }
            }
        }
        if let Some(h) = spread(&holes.iter().map(Some).collect::<Vec<_>>()) {
            // a sum of each position's units times its price: linear where every hole
            // position's term is (L §5.18; tally 2.1.8 R15)
            if ctx.partial && !lins.is_empty() && lins.iter().all(Option::is_some) {
                let parts = lins.into_iter().flatten().collect();
                return crate::eval::lin_fold(m, &h, in_currency(cur, total)?, parts);
            }
            return Ok(Value::Hole(h));
        }
        in_currency(cur, total)
    }

    /// A `partial` line's term of a position whose units are hole `h`: its linear part times
    /// the price of one unit `u` in CUR, 1 in CUR itself; none where the price is a hole too,
    /// which joins `holes` (refcore's `m_e_value`, its partial arm). L §5.18 (tally 2.1.8
    /// R15), §5.10.
    fn hole_position(
        &mut self,
        m: &Reality,
        (read, valued): (&Valuation, &Valued),
        (u, h): (&str, &Hole),
        (env, ctx): (&Env, Ctx),
        holes: &mut Vec<Hole>,
    ) -> Result<Option<Linear>, Stop> {
        let cur = read.currency.as_str();
        let price = if u == cur {
            Num::int(1)
        } else {
            match self.position_price(m, (read, valued), u, env, ctx)? {
                Err(p) => {
                    holes.push(p);
                    return Ok(None);
                }
                Ok(p) => p,
            }
        };
        let per = crate::model::Unit::make(vec![(cur.to_owned(), 1), (u.to_owned(), -1)])
            .map_err(crate::eval::num_stop)?;
        let price = Value::Number(crate::model::Number::plain(price, per));
        scaled(m, h.printed_linear().cloned(), &price)
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
        let amount = match self.priced_read(m, (read, valued), (price, id), None, (env, ctx))? {
            Err(h) => return Ok(Value::Hole(h)),
            Ok(q) => q,
        };
        let cur = read.currency.as_str();
        if whole == cur {
            return in_currency(cur, amount);
        }
        if valued.by.is_none() {
            return Err(Stop::Refused(Refusal::new(
                Load::Price.at_run(),
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
                self.priced_position(m, (read, valued), u, (env, ctx))
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
        (env, ctx): (&Env, Ctx),
    ) -> Result<Result<Num, Hole>, Stop> {
        let (cur, place) = (read.currency.as_str(), valued.place.as_str());
        let Some(price) = &valued.priced else {
            return Err(Stop::Refused(Refusal::new(
                Load::Price.at_run(),
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
        self.priced_read(m, (read, valued), (price, &id), Some(u), (env, ctx))
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
                vec![Given::Value(Value::Key(KeyVal::Thing(u.to_owned())))],
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

    /// A place's own price clause read on the value's day: at the keys it is written with,
    /// the place section's key variables bound to the place's keys; else at the place's
    /// thing, or at its position's unit `u`, when the price input is keyed by a kind holding
    /// one; then the record fields it names, `priced p.f`; a hole naming the place or `u`
    /// where it is `none` (refcore's `m_priced_read`). L §5.10, §2.3, §4.6, §8.1.
    fn priced_read(
        &mut self,
        m: &Reality,
        (read, valued): (&Valuation, &Valued),
        (price, pid): (&Price, &str),
        u: Option<&str>,
        (env, ctx): (&Env, Ctx),
    ) -> Result<Result<Num, Hole>, Stop> {
        let Some((written, fields)) = price_fields(&price.input) else {
            return Err(invariant("a price clause load didn't read"));
        };
        let place = valued.place.as_str();
        let given = if let Expr::Post(_, posts) = &written {
            let [Postfix::Keys(keys)] = posts.as_slice() else {
                return Err(invariant("a price clause load didn't read"));
            };
            self.clause_keys(m, (price, keys), place, (env, ctx), valued.day)?
        } else {
            thing_key(m, pid, place, u)?
        };
        let keys = match self.price_keys(m, (pid, given), valued.day, ctx)? {
            Err(h) => return Ok(Err(h)),
            Ok(keys) => keys,
        };
        let texts: Vec<String> = keys.iter().map(KeyVal::text).collect();
        let figure = format!("{pid}[{}]", texts.join(", "));
        let mut p = self.price_read_at(m, (pid, &keys), (valued.day, price.last), ctx)?;
        // `priced p.f`: the field of the record read (L §2.3 ref); a record that is none
        // stays the price that is none
        for field in fields {
            p = price_field(p, field)?;
        }
        let who = match u {
            Some(u) => u,
            None => place,
        };
        priced_value(m, (read, valued), p, &figure, who)
    }

    /// The keys a price clause is written with, `priced px[f]`, each read at the value's
    /// day with the place section's key variables bound to the place's keys (refcore's
    /// `m_priced_read`, its `_expr` arm). L §5.10, §2.3.
    fn clause_keys(
        &mut self,
        m: &Reality,
        (price, keys): (&Price, &[Key]),
        place: &str,
        (env, ctx): (&Env, Ctx),
        day: Day,
    ) -> Result<Vec<Given>, Stop> {
        let mut bound = env.clone();
        let section = m
            .things
            .get(place)
            .and_then(|thing| thing.place.as_ref())
            .and_then(|own| Some((own, m.places.get(&own.place)?)))
            .filter(|(_, info)| info.priced.as_ref() == Some(price));
        if let Some((own, info)) = section {
            for (bind, key) in info.binds.iter().zip(&own.keys) {
                let value = Value::Key(key.clone());
                bound = bound.with(&bind.var, value, Some(bind.domain.clone()));
            }
        }
        let at = Ctx {
            law: Some(day),
            ..ctx
        };
        let mut given = Vec::with_capacity(keys.len());
        for key in keys {
            let Key::Expr(e) = key else {
                return Err(Stop::NotBuilt("a price clause read at *"));
            };
            given.push(self.given_key(m, e, &bound, at)?);
        }
        Ok(given)
    }
}

/// The key a price clause with no keys written is read at: none for an unkeyed price; the
/// place's thing, or its position's unit `u`, for a price keyed by a kind holding it
/// (refcore's `m_priced_read`, its `info.binds` arm). L §5.10.
///
/// # Errors
///
/// `LOAD-KEY` for a price keyed by a domain that holds neither.
fn thing_key(m: &Reality, pid: &str, place: &str, u: Option<&str>) -> Result<Vec<Given>, Stop> {
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
                Load::Key.at_run(),
                format!(
                    "{place} is priced by {pid}, keyed by {over}, which holds neither {place} nor its unit"
                ),
            )));
        }
    };
    Ok(key
        .map(|k| Given::Value(Value::Key(KeyVal::Thing(k.to_owned()))))
        .into_iter()
        .collect())
}

/// A record's field `name`, read of a price: a hole or `none` stays itself (refcore's
/// `m_priced_read`, its fields). L §2.3, §5.10.
fn price_field(v: Value, name: &str) -> Result<Value, Stop> {
    if let Value::Hole(_) | Value::None = v {
        return Ok(v);
    }
    let Value::Record(fields) = v else {
        return Err(invariant(
            "a price's field of a value that is no record: load refuses it (LOAD-FIELD)",
        ));
    };
    fields
        .into_iter()
        .find(|(f, _)| f == name)
        .map(|(_, value)| value)
        .ok_or_else(|| invariant("a price's field its record lacks: load refuses it (LOAD-FIELD)"))
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

/// A position's linear part `lin` scaled by `factor`, a price or a number of units; none
/// where it has none (L §5.18; tally 2.1.8 R15).
fn scaled(m: &Reality, lin: Option<Linear>, factor: &Value) -> Result<Option<Linear>, Stop> {
    match lin {
        Some(lin) => crate::eval::lin_times(m, lin, factor).map(Some),
        None => Ok(None),
    }
}
