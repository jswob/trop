//! Reads of inputs: a read's keys made keys of their positions, the row the read rule takes, and the row's value as a runtime value of the input's type.
//! Spec: L §4.6 (the read rule; a row's value, `assumed`, `held`), §2.5 (keys by position),
//! §4.2 (keys as values), §5.6 (`assumed` on an index), §5.8 (inputs and their values), §3.4
//! (a figure in Y prices), §8.1 (a read whose key is a hole carries the key's questions only),
//! §8.2 (the register entries a read makes), §8.4 (`RUN-KEY`, `RUN-NONE`).
//! Twin: engine/tools/refcore/evaluate.py (`key_values`, `to_key`, `check_key`, `read_input`,
//! `row_value`, `prices`, `index_assumed`) and loader.py (`convert_value`, `key_value`).
//! Invariants: a key a read takes is a key its domain holds and the cut sees; a row's value
//! has its input's declared type; a hole is built in the hole module.

use super::arith::{Op, arith, int_of};
use super::state::{Ctx, Engine, invariant, num_stop, refuse, refuse_load};
use crate::base::{Load, Run};
use crate::hole::Hole;
use crate::load::index_key_of;
use crate::model::{
    BindInfo, Class, Code, CodeKind, Domain, IndexInfo, InputInfo, KeyVal, Number, PeriodWord,
    Reality, Row, Ty, Unit, Value, currency_id, num_money,
};
use crate::num::Num;
use crate::read::{Asked, Clocks, Found, held, origin, read_input, time_point};
use crate::syntax::{
    Calendar as CalendarLit, CalendarDay, FieldValue, Lit, Stop, Value as Written,
};
use crate::time::{Calendar, Day, Period, PeriodKind, weekday_number};

/// A key a read is given: a value, or a key literal its position resolves (L §2.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Given {
    /// A value.
    Value(Value),
    /// A key literal, resolved in the position's domain.
    Literal(Lit),
}

impl Engine {
    /// The clocks of a read in context `ctx` (L §4.3, §4.5).
    pub(super) fn clocks(&self, ctx: Ctx) -> Clocks {
        Clocks {
            law: ctx.law,
            cut: ctx.cut,
            ask_cut: self.ask_cut,
        }
    }

    /// An index with `assumed` lists `assumed: <index>: <reason>` for every figure keyed by
    /// it (refcore's `index_assumed`). L §5.6, §8.2.
    pub(super) fn index_assumed(&mut self, m: &Reality, binds: &[BindInfo]) {
        for bind in binds {
            if let Domain::Index(id) = &bind.domain
                && let Some(ix) = m.indexes.get(id)
                && let Some(reason) = &ix.assumed
            {
                self.note("assumed", format!("{}: {reason}", ix.id), String::new());
            }
        }
    }

    /// A read of input `input` at keys already made keys (refcore's `read_input`): the row
    /// the read rule takes, as a value; `none` outside a lifespan; else the gap rule's hole.
    /// L §4.6.
    ///
    /// # Errors
    ///
    /// The refusals of the read rule and of the row's value.
    pub(super) fn read_input_at(
        &mut self,
        m: &Reality,
        input: &InputInfo,
        keys: &[KeyVal],
        ctx: Ctx,
        asked: Asked,
    ) -> Result<Value, Stop> {
        self.index_assumed(m, &input.binds);
        let clocks = self.clocks(ctx);
        match read_input(m, input, keys, clocks, asked)? {
            Found::Row(at) => {
                let row = input
                    .rows
                    .get(at)
                    .ok_or_else(|| invariant("the read rule took a row the input lacks"))?;
                let value = self.row_value(m, input, row, keys, ctx)?;
                self.priced(m, input, keys, ctx, value)
            }
            Found::None => Ok(Value::None),
            Found::Hole(q) => Ok(Value::Hole(Hole::asked(&q.text, &q.origin))),
        }
    }

    /// A chosen row's value with the register entries it makes: `assumed` (its own and its
    /// version's), `held` (a row with a start and no end read for a law date after the ask's
    /// cut) (refcore's `row_value`). L §4.6, §8.2.
    fn row_value(
        &mut self,
        m: &Reality,
        input: &InputInfo,
        row: &Row,
        keys: &[KeyVal],
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let figure = origin(&input.id, keys);
        self.note_version(&row.version);
        if let Some(reason) = &row.assumed {
            self.note("assumed", reason.clone(), figure.clone());
        }
        let version = m
            .versions
            .get(&row.version)
            .ok_or_else(|| invariant("a row names a version load didn't build"))?;
        if let Some(reason) = &version.assumed {
            self.note("assumed", reason.clone(), figure.clone());
        }
        self.index_assumed(m, &input.binds);
        let ty = input
            .ty
            .as_ref()
            .ok_or(Stop::NotBuilt("the value of an observed row"))?;
        let value = match convert(m, &row.value, ty, &input.unit_factor)? {
            Converted::Hole(text) => Value::Hole(Hole::asked(&text, &figure)),
            Converted::Value(v) => v,
        };
        let point = time_point(input, keys, ctx.law)?;
        if held(m, row, point.day, self.clocks(ctx))? {
            let shown = shown(m, &value)?;
            let read_for = match (&point.period, point.day) {
                (Some(period), _) => Value::of_key(period).text(m).map_err(num_stop)?,
                (None, Some(day)) => day.text(),
                (None, None) => String::new(),
            };
            self.note(
                "held",
                format!("{figure} = {shown} ({}), read for {read_for}", row.version),
                String::new(),
            );
        }
        Ok(value)
    }

    /// A figure in Y prices moved to its read's day by the index it names: `x × idx[d] /
    /// idx[Y]`, a `num CUR` (refcore's `prices`). L §3.4.
    fn priced(
        &mut self,
        m: &Reality,
        input: &InputInfo,
        keys: &[KeyVal],
        ctx: Ctx,
        value: Value,
    ) -> Result<Value, Stop> {
        let Some(year) = input.prices else {
            return Ok(value);
        };
        if matches!(value, Value::Hole(_) | Value::None) {
            return Ok(value);
        }
        let point = time_point(input, keys, ctx.law)?;
        let day = point
            .day
            .ok_or_else(|| invariant("a priced figure read with no day"))?;
        let Some(by) = &input.prices_by else {
            if i64::from(day.year()) == year {
                return Ok(value);
            }
            let text = format!(
                "{} is in {year} prices; no index moves it to {}",
                input.id,
                day.text()
            );
            return Ok(Value::Hole(Hole::asked(&text, &origin(&input.id, keys))));
        };
        let (index_at_day, index_at_year) = self.price_index(m, by, year, day, ctx)?;
        let operands = [
            value.as_hole(),
            index_at_day.as_hole(),
            index_at_year.as_hole(),
        ];
        if let Some(hole) = crate::hole::spread(&operands) {
            return Ok(Value::Hole(hole));
        }
        let plain = as_plain(m, &value)?;
        let scaled = arith(m, Op::Mul, &plain, &index_at_day)?;
        let moved = arith(m, Op::Div, &scaled, &index_at_year)?;
        let Value::Number(n) = moved else {
            return Err(invariant("a price index moved a figure to no number"));
        };
        Ok(Value::Number(Number::plain(n.q, plain_unit(&plain))))
    }

    /// The price index `by` read at the key holding `day` and at the year `year` (L §3.4).
    fn price_index(
        &mut self,
        m: &Reality,
        by: &str,
        year: i64,
        day: Day,
        ctx: Ctx,
    ) -> Result<(Value, Value), Stop> {
        let not_built = || Stop::NotBuilt("prices by an index that isn't a def keyed by years");
        let def = m.defs.get(by).ok_or_else(not_built)?;
        let [bind] = def.binds.as_slice() else {
            return Err(not_built());
        };
        let Domain::Index(ix_id) = &bind.domain else {
            return Err(not_built());
        };
        let ix = m.indexes.get(ix_id).ok_or_else(not_built)?;
        let at_year = ix
            .keys
            .iter()
            .find(|k| {
                matches!(k, KeyVal::Period(p)
                    if p.kind() == PeriodKind::Year && i64::from(p.start().year()) == year)
            })
            .cloned()
            .ok_or_else(|| {
                refuse_load(
                    Load::Key,
                    format!("prices {year} by {by} needs an index with a key for the year {year}"),
                )
            })?;
        let at_day = ix
            .holding(day)
            .cloned()
            .ok_or_else(|| refuse(Run::Key, format!("{} is outside {}", day.text(), ix.id)))?;
        let a = self.read_def_at(m, def, &[at_day], ctx)?;
        let b = self.read_def_at(m, def, &[at_year], ctx)?;
        for v in [&a, &b] {
            if *v == Value::None {
                return Err(refuse(
                    Run::None,
                    "a value is none here: arithmetic".to_owned(),
                ));
            }
        }
        Ok((a, b))
    }
}

/// The keys of a read: each given key made a key of its position's domain, checked
/// against the domain and the cut; a hole key makes the read that hole, carrying the
/// keys' questions only (refcore's `key_values`). L §2.5, §8.1.
///
/// # Errors
///
/// `RUN-KEY`, `RUN-NONE` and the conversions' refusals.
pub(crate) fn make_keys(
    m: &Reality,
    item: &str,
    binds: &[BindInfo],
    given: Vec<Given>,
    cut: Day,
) -> Result<Result<Vec<KeyVal>, Hole>, Stop> {
    if given.len() != binds.len() {
        return Err(Stop::NotBuilt("a read with fewer or more keys than binds"));
    }
    let mut keys = Vec::with_capacity(binds.len());
    let mut holes: Vec<Hole> = Vec::new();
    for (bind, key) in binds.iter().zip(given) {
        let key = match key {
            Given::Value(Value::Hole(h)) => {
                holes.push(h);
                continue;
            }
            Given::Value(v) => to_key(m, item, &v, &bind.domain)?,
            Given::Literal(lit) => literal_key(m, &lit, &bind.domain)?,
        };
        check_key(m, &key, &bind.domain, cut)?;
        keys.push(key);
    }
    let operands: Vec<Option<&Hole>> = holes.iter().map(Some).collect();
    if let Some(hole) = crate::hole::spread(&operands) {
        return Ok(Err(hole));
    }
    Ok(Ok(keys))
}

/// A money value as a `num CUR`, anything else as it is (L §3.2).
fn as_plain(m: &Reality, v: &Value) -> Result<Value, Stop> {
    if let Value::Money(money) = v {
        let q = crate::model::money_num(m, money).map_err(num_stop)?;
        let unit = crate::model::currency_unit(m, money.currency()).map_err(num_stop)?;
        return Ok(Value::Number(Number::plain(q, unit)));
    }
    Ok(v.clone())
}

/// A number's unit, or 1.
fn plain_unit(v: &Value) -> Unit {
    if let Value::Number(n) = v {
        return n.unit();
    }
    Unit::one()
}

/// A value made a key of `domain` (refcore's `to_key`). L §2.5, §4.2.
///
/// # Errors
///
/// `RUN-NONE` for `none`, `RUN-KEY` for a value the domain doesn't hold.
pub(crate) fn to_key(m: &Reality, item: &str, v: &Value, domain: &Domain) -> Result<KeyVal, Stop> {
    if *v == Value::None {
        return Err(refuse(Run::None, "a key is none here: a key".to_owned()));
    }
    if let Value::OutKey { index, text } = v {
        return Err(refuse(Run::Key, format!("{text} is outside {index}")));
    }
    match domain {
        Domain::Index(id) => {
            let ix = m
                .indexes
                .get(id)
                .ok_or_else(|| invariant("a bind over an index load didn't build"))?;
            index_key(m, item, ix, v)
        }
        Domain::Kind(kind) => {
            let Value::Key(KeyVal::Thing(name)) = v else {
                return Err(refuse(
                    Run::Key,
                    format!("{} is not a key of {kind}", v.text(m).map_err(num_stop)?),
                ));
            };
            if !m.thing_in(name, kind) {
                return Err(refuse(Run::Key, format!("{name} is not a key of {kind}")));
            }
            Ok(KeyVal::Thing(name.clone()))
        }
        Domain::List(_) | Domain::Range | Domain::Axis(_) => {
            Err(Stop::NotBuilt("a key of a key list, a range or an axis"))
        }
    }
}

/// A value made a key of index `ix`: an int of an integer index, the key holding a day or a
/// period of a period index, a key of a list or open index as it is (refcore's `to_key`).
fn index_key(m: &Reality, item: &str, ix: &IndexInfo, v: &Value) -> Result<KeyVal, Stop> {
    if ix.is_int {
        return Ok(KeyVal::Int(int_of(m, v)?));
    }
    if ix.period.is_some() {
        return period_key(m, item, ix, v);
    }
    if let Value::Key(key @ KeyVal::Key { .. }) = v {
        return Ok(key.clone());
    }
    Err(refuse(
        Run::Key,
        format!("{} is outside {}", v.text(m).map_err(num_stop)?, ix.id),
    ))
}

/// The key of a period index a day or a period stands for: the key holding the day, the
/// key equal to the period or holding it (refcore's `to_key`, `holding` and `period`). L §4.6.
fn period_key(m: &Reality, item: &str, ix: &IndexInfo, v: &Value) -> Result<KeyVal, Stop> {
    let outside = |text: String| refuse(Run::Key, format!("{text} is outside {}", ix.id));
    if let Value::Day(day) = v {
        return ix.holding(*day).cloned().ok_or_else(|| {
            let def_days = m.defs.get(item).is_some() && ix.period == Some(PeriodKind::Day);
            if def_days {
                refuse(
                    Run::Key,
                    format!("{} is outside the days of {item}", day.text()),
                )
            } else {
                outside(day.text())
            }
        });
    }
    if let Value::Period(p) = v {
        for key in &ix.keys {
            match key {
                KeyVal::Day(d) if p.kind() == PeriodKind::Day && *d == p.start() => {
                    return Ok(key.clone());
                }
                KeyVal::Period(k) if k == p || (k.start() <= p.start() && p.end() <= k.end()) => {
                    return Ok(key.clone());
                }
                KeyVal::Day(_)
                | KeyVal::Period(_)
                | KeyVal::Key { .. }
                | KeyVal::Thing(_)
                | KeyVal::Int(_)
                | KeyVal::Mv(_) => {}
            }
        }
        return Err(outside(p.label()));
    }
    Err(outside(v.text(m).map_err(num_stop)?))
}

/// A value as a register entry shows it: a hole as `?` (L §8.2).
pub(super) fn shown(m: &Reality, v: &Value) -> Result<String, Stop> {
    if let Value::Hole(_) = v {
        return Ok("?".to_owned());
    }
    v.text(m).map_err(num_stop)
}

/// A key literal resolved in `domain`: a key of an index, an integer or a period by its
/// label or a key by its text, a thing of a kind by its name (refcore's
/// `resolve_key_literal`). L §2.5.
///
/// # Errors
///
/// `LOAD-KEY`, where load sees it, for a literal the domain doesn't hold.
pub(super) fn literal_key(m: &Reality, lit: &Lit, domain: &Domain) -> Result<KeyVal, Stop> {
    let text = lit.text();
    match domain {
        Domain::Index(id) => {
            let ix = m
                .indexes
                .get(id)
                .ok_or_else(|| invariant("a bind over an index load didn't build"))?;
            index_key_of(ix, lit)
                .ok_or_else(|| refuse_load(Load::Key, format!("{text} is not a key of {id}")))
        }
        Domain::Kind(kind) => {
            if !m.thing_in(&text, kind) {
                return Err(refuse_load(
                    Load::Key,
                    format!("{text} is not a thing of {kind}"),
                ));
            }
            Ok(KeyVal::Thing(text))
        }
        Domain::List(_) | Domain::Range | Domain::Axis(_) => Err(Stop::NotBuilt(
            "a key literal of a key list, a range or an axis",
        )),
    }
}

/// Whether the cut sees key `key` of `domain`: a key its index holds and no version known
/// after the cut added, a thing an entry the cut sees brings (refcore's `check_key`). L §4.5.
///
/// # Errors
///
/// `RUN-KEY`.
pub(super) fn check_key(m: &Reality, key: &KeyVal, domain: &Domain, cut: Day) -> Result<(), Stop> {
    match domain {
        Domain::Index(id) => {
            let ix = m
                .indexes
                .get(id)
                .ok_or_else(|| invariant("a bind over an index load didn't build"))?;
            if ix.pos(key).is_none() {
                return Err(refuse(Run::Key, format!("{} is outside {id}", key.text())));
            }
            if let Some(version) = ix.added_by(key) {
                let known = m
                    .versions
                    .get(version)
                    .map(|v| v.known)
                    .ok_or_else(|| invariant("a key added by a version load didn't build"))?;
                if known > cut {
                    return Err(refuse(
                        Run::Key,
                        format!("{} was added after the cut", key.text()),
                    ));
                }
            }
            Ok(())
        }
        Domain::Kind(kind) => {
            let KeyVal::Thing(name) = key else {
                return Err(refuse(
                    Run::Key,
                    format!("{} is not a key of {kind}", key.text()),
                ));
            };
            let thing = m
                .things
                .get(name)
                .ok_or_else(|| refuse(Run::Key, format!("{name} is not a key of {kind}")))?;
            if !thing_seen(m, thing, cut) {
                return Err(refuse(Run::Key, format!("{name} was added after the cut")));
            }
            Ok(())
        }
        Domain::List(_) | Domain::Range | Domain::Axis(_) => Ok(()),
    }
}

/// Whether the cut sees an entry of `thing`: one with no version, or whose version is known by
/// the cut (refcore's `thing_seen`). L §4.5, §5.7.
#[must_use]
pub(crate) fn thing_seen(m: &Reality, thing: &crate::model::Thing, cut: Day) -> bool {
    if let Some(keys) = &thing.place {
        return keys
            .key_things
            .iter()
            .filter_map(|name| m.things.get(name))
            .all(|t| thing_seen(m, t, cut));
    }
    for entry in &thing.entries {
        let Some(version) = &entry.version else {
            return true;
        };
        if m.versions.get(version).is_some_and(|v| v.known <= cut) {
            return true;
        }
    }
    false
}

/// A row's value converted to its target's type: a value, or a stated hole's text (refcore's
/// `convert_value`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Converted {
    /// A value.
    Value(Value),
    /// `?"…"`: the hole's question, built with its figure where it is read.
    Hole(String),
}

/// A reading's row value as a runtime value of type `ty`: the value, or a stated hole's text
/// (refcore's `convert_value`, for the walk's readings). L §5.19, §5.8.
///
/// # Errors
///
/// The unit refusals load makes; `NotBuilt` for a window.
pub(crate) fn convert_row(
    m: &Reality,
    value: &Written,
    ty: &Ty,
) -> Result<Result<Value, String>, Stop> {
    match convert(m, value, ty, &Num::int(1))? {
        Converted::Value(v) => Ok(Ok(v)),
        Converted::Hole(text) => Ok(Err(text)),
    }
}

/// A row's literal value as a runtime value of type `ty`; `factor` is the `%` the input's
/// unit brings (refcore's `convert_value`). L §5.8, §3.2.
///
/// # Errors
///
/// The unit refusals load makes (`LOAD-UNIT`, `LOAD-PRECISION`); `NotBuilt` for a window.
pub(super) fn convert(
    m: &Reality,
    value: &Written,
    ty: &Ty,
    factor: &Num,
) -> Result<Converted, Stop> {
    let v = match value {
        Written::Hole(text) => return Ok(Converted::Hole(text.clone())),
        Written::None => Value::None,
        Written::Never => Value::Never,
        Written::Str(s) => Value::Text(s.clone()),
        Written::Lit(lit) => convert_lit(m, lit, ty, factor)?,
        Written::Record(fields) => convert_record(m, fields, ty)?,
        Written::List(items) => {
            let mut days = Vec::with_capacity(items.len());
            for item in items {
                let Lit::Day(d) = item else {
                    return Err(refuse_load(Load::Unit, "dates holds days".to_owned()));
                };
                days.push(Value::Day(*d));
            }
            Value::List(days)
        }
        Written::Calendar(cal) => calendar_value(cal)?,
        Written::Days(_) => return Err(Stop::NotBuilt("a window as a row's value")),
        Written::Expr(_) => return Err(Stop::NotBuilt("a scenario's expression row")),
    };
    Ok(Converted::Value(v))
}

/// A record's fields, each converted to its field's type, in the type's order (L §3.1).
fn convert_record(m: &Reality, fields: &[crate::syntax::Field], ty: &Ty) -> Result<Value, Stop> {
    let Ty::Record(types) = ty else {
        return Err(refuse_load(
            Load::Unit,
            format!("a record for {}", ty.show()),
        ));
    };
    let mut out = Vec::with_capacity(types.len());
    for (name, field_ty) in types {
        let found = fields.iter().find_map(|f| match f {
            crate::syntax::Field::Named(n, v) if n == name => Some(v),
            crate::syntax::Field::Named(..) | crate::syntax::Field::Leg(_) => None,
        });
        let Some(written) = found else {
            return Err(refuse_load(
                Load::Field,
                format!("a record lacks field {name}"),
            ));
        };
        let one = Num::int(1);
        let value = match written {
            FieldValue::Lit(lit) | FieldValue::Expr(crate::syntax::Expr::Lit(lit)) => {
                convert_lit(m, lit, field_ty, &one)?
            }
            FieldValue::Expr(crate::syntax::Expr::Name(word)) => {
                convert_lit(m, &Lit::Name(word.clone()), field_ty, &one)?
            }
            FieldValue::None | FieldValue::Expr(crate::syntax::Expr::None) => Value::None,
            FieldValue::Hole(_) | FieldValue::Expr(_) => {
                return Err(Stop::NotBuilt(
                    "a record field that is a hole or an expression",
                ));
            }
        };
        out.push((name.clone(), value));
    }
    Ok(Value::Record(out))
}

/// A literal of a row as a value of type `ty` (refcore's `convert_value`). L §5.8.
fn convert_lit(m: &Reality, lit: &Lit, ty: &Ty, factor: &Num) -> Result<Value, Stop> {
    let wrong = || {
        refuse_load(
            Load::Unit,
            format!(
                "a value of the wrong type: {} for {}",
                lit.text(),
                ty.show()
            ),
        )
    };
    match lit {
        Lit::Num(n) => convert_number(m, n, ty, factor),
        Lit::Bool(b) if *ty == Ty::Bool => Ok(Value::Bool(*b)),
        Lit::Day(d) if *ty == Ty::Date => Ok(Value::Day(*d)),
        Lit::Month(y, mo) if *ty == Ty::Period(PeriodWord::Month) => {
            Period::month(*y, *mo).map(Value::Period).ok_or_else(wrong)
        }
        Lit::Quarter(y, q) if *ty == Ty::Period(PeriodWord::Quarter) => {
            Period::quarter(*y, *q).map(Value::Period).ok_or_else(wrong)
        }
        Lit::MonthDay(mo, d) if *ty == Ty::MonthDay => Ok(Value::MonthDay(*mo, *d)),
        Lit::Nominal(n) if *ty == Ty::Code(CodeKind::Compounding) => {
            let n = n.to_i64().ok_or_else(wrong)?;
            Ok(Value::Code(Code::Nominal(n)))
        }
        Lit::Name(text) | Lit::Key(text) | Lit::Cur(text) if matches!(ty, Ty::Code(_)) => {
            Ok(Value::Code(Code::Word(text.clone())))
        }
        Lit::Bool(_)
        | Lit::Day(_)
        | Lit::Month(..)
        | Lit::Quarter(..)
        | Lit::MonthDay(..)
        | Lit::Nominal(_)
        | Lit::Name(_)
        | Lit::Key(_)
        | Lit::Cur(_)
        | Lit::Inf { .. } => {
            let Ty::Key(ix) = ty else {
                return Err(wrong());
            };
            key_value(m, lit, ix)
        }
    }
}

/// A key literal of `key ix` as a value: an index's key, or a thing of a kind; a key of an
/// integer index is an int (refcore's `key_value`). L §4.2.
pub(super) fn key_value(m: &Reality, lit: &Lit, ix: &str) -> Result<Value, Stop> {
    let domain = if m.indexes.get(ix).is_some() {
        Domain::Index(ix.to_owned())
    } else if m.kinds.get(ix).is_some() {
        Domain::Kind(ix.to_owned())
    } else {
        return Err(refuse_load(Load::Name, format!("unknown id {ix}")));
    };
    Ok(Value::of_key(&literal_key(m, lit, &domain)?))
}

/// A number literal of a row as a value of type `ty` (refcore's `convert_value`). L §3.2.
fn convert_number(
    m: &Reality,
    n: &crate::syntax::Number,
    ty: &Ty,
    factor: &Num,
) -> Result<Value, Stop> {
    let written = match &n.unit {
        None => None,
        Some(unit) => {
            let terms = unit
                .terms
                .iter()
                .map(|t| (t.base.clone(), t.power))
                .collect();
            Some(Unit::from_syntax(terms).map_err(num_stop)?)
        }
    };
    let different = || {
        refuse_load(
            Load::Unit,
            format!("{} and {} have different units", n.text, ty.show()),
        )
    };
    match ty {
        Ty::Money(cur) => {
            let q = match &written {
                None => n.value.checked_mul(factor).map_err(num_stop)?,
                Some((unit, f)) => {
                    if unit.terms() != [(cur.clone(), 1)] {
                        return Err(different());
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
        Ty::Num(unit, shown) => {
            let q = match &written {
                None => n.value.checked_mul(factor).map_err(num_stop)?,
                Some((from, f)) => {
                    let c = from.conv(unit).map_err(num_stop)?.ok_or_else(different)?;
                    n.value
                        .checked_mul(f)
                        .and_then(|q| q.checked_mul(&c))
                        .map_err(num_stop)?
                }
            };
            Ok(Value::Number(Number {
                q,
                class: Class::Num(unit.clone(), *shown),
            }))
        }
        Ty::Int if written.is_none() && n.whole && !n.fraction => Ok(Value::Number(Number {
            q: n.value.clone(),
            class: Class::Int,
        })),
        Ty::Rate => {
            let Some((unit, f)) = written.filter(|(u, _)| u.period_of().is_some()) else {
                return Err(refuse_load(
                    Load::Unit,
                    format!("a rate states its period: {}", n.text),
                ));
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
                .ok_or_else(different)
        }
        Ty::Key(ix) => key_value(m, &Lit::Num(n.clone()), ix),
        Ty::Int
        | Ty::Period(_)
        | Ty::Bool
        | Ty::Text
        | Ty::Date
        | Ty::Dates
        | Ty::Dist(_)
        | Ty::Calendar
        | Ty::Code(_)
        | Ty::MonthDay
        | Ty::Record(_)
        | Ty::List(_)
        | Ty::Tuple(_)
        | Ty::Flex(_)
        | Ty::Year4
        | Ty::KeyLit
        | Ty::None
        | Ty::Hole
        | Ty::Never
        | Ty::KindName => Err(refuse_load(
            Load::Unit,
            format!(
                "{} and {} have different units (a value of the wrong type)",
                n.text,
                ty.show()
            ),
        )),
    }
}

/// A calendar literal of a row as a calendar (refcore's `Cal`); an added day that is a hole
/// or a window needs the windows S doesn't build. L §4.2.
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
        match day {
            CalendarDay::Day(d) => added.push(*d),
            CalendarDay::Hole(_) | CalendarDay::Set(_) => {
                return Err(Stop::NotBuilt(
                    "a calendar's added day that is a hole or a window",
                ));
            }
        }
    }
    let from = match &cal.yearly_from {
        None => None,
        Some(n) => Some(
            n.to_i64()
                .and_then(|y| u16::try_from(y).ok())
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
