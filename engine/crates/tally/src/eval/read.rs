//! Reads of inputs: a read's keys made keys of their positions, the row the read rule takes, and the row's value with its register entries.
//! Spec: L §4.6 (the read rule; a row's value, `assumed`, `held`), §2.5 (keys by position),
//! §4.2 (keys as values), §5.6 (`assumed` on an index), §5.8 (inputs and their values), §3.4
//! (a figure in Y prices), §8.1 (a read whose key is a hole carries the key's questions only),
//! §8.2 (the register entries a read makes), §8.4 (`RUN-KEY`, `RUN-NONE`), §7.6 items 1 and 3
//! (the end of time is no key; a key offset's 18 digits), `last(x, d)` and `since(x, d)`
//! (§4.6 Past the last row, Readings; §5.3); §4.4 (a window read is that read's value, its
//! holes asked of its row). A row's value converted to its input's type is `convert.rs`'s.
//! Twin: engine/tools/refcore/evaluate.py (`key_values`, `to_key`, `check_key`, `read_input`,
//! `row_value`, `e_lastx`, `e_call_since`, `prices`, `index_assumed`).
//! Invariants: a key a read takes is a key its domain holds and the cut sees; a row's value
//! has its input's declared type; a hole is built in the hole module.

use super::arith::{Op, arith, int_of};
use super::builtin::read_parts;
use super::convert::{ConvertFor, Converted, convert};
use super::memo::{LawClass, Read, may_hold, rows_class};
use super::state::{Ctx, Engine, Env, a_day, invariant, num_stop, refuse, refuse_load};
use super::windows::{as_read, one_day};
use crate::base::{Load, Run};
use crate::hole::Hole;
use crate::load::index_key_of;
use crate::model::{
    BindInfo, Domain, IndexInfo, InputInfo, KeyVal, Number, Reality, Row, RowInstant, TimePos,
    Unit, Value,
};
use crate::read::{Asked, Clocks, Found, held, origin, read_input, read_last, time_point};
use crate::syntax::{Call, Expr, Lit, Stop};
use crate::time::{Day, PeriodKind};

/// Which read past a row's span: `last(x, d)` or `since(x, d)` (L §4.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Past {
    /// `last(x, d)`.
    Last,
    /// `since(x, d)`.
    Since,
}

impl Past {
    /// Its word, as a message names it.
    fn word(self) -> &'static str {
        match self {
            Past::Last => "last",
            Past::Since => "since",
        }
    }
}

/// A key a read is given: a value, or a key literal its position resolves (L §2.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Given {
    /// A value.
    Value(Value),
    /// A key literal, resolved in the position's domain.
    Literal(Lit),
}

/// The row `last(x[k], d)` or `since(x[k], d)` reads: its input, its keys, its position
/// among the input's rows and d (L §4.6).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Latest {
    /// The input x.
    input: String,
    /// Its keys, made.
    keys: Vec<KeyVal>,
    /// The row's position among the input's rows.
    at: usize,
    /// The day d.
    day: Day,
}

/// The row at position `at` of `input`, which the read rule took (L §4.6).
fn latest_of(input: &InputInfo, at: usize) -> Result<&Row, Stop> {
    input
        .rows
        .get(at)
        .ok_or_else(|| invariant("the read rule took a row the input lacks"))
}

impl Engine {
    /// The clocks of a read of `item` in context `ctx`: the law date, the cut, the ask's cut,
    /// and the day a `wording` pin names the item with (L §4.3, §4.5).
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a context whose layer was freed.
    pub(super) fn clocks(&self, ctx: Ctx, item: &str) -> Result<Clocks, Stop> {
        Ok(Clocks {
            law: ctx.law,
            cut: ctx.cut,
            ask_cut: self.ask_cut,
            wording: self.wording_of(item, ctx)?,
        })
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
        // a `with` or `as if` pin replaces every read of the input at its keys (L §5.11)
        if let Some(pinned) = self.pin_of(&input.id, keys, ctx)? {
            return Ok(pinned);
        }
        let clocks = self.clocks(ctx, &input.id)?;
        #[cfg(feature = "counters")]
        self.counts
            .read(format!("{}|{}", origin(&input.id, keys), ctx.key()));
        self.read(Read::Input {
            id: input.id.clone(),
            keys: keys.to_vec(),
        });
        let by_law = self.read_class(m, input, ctx, asked)?;
        let found = read_input(m, input, keys, clocks, asked)?;
        if by_law && !matches!(found, Found::Row(_)) {
            // a thing's lifespan or the question's text: the day itself (L §4.6 step 5)
            self.narrow_to_day(ctx.law);
        }
        let value = match found {
            Found::Row(at) => {
                let row = input
                    .rows
                    .get(at)
                    .ok_or_else(|| invariant("the read rule took a row the input lacks"))?;
                self.read(Read::Row {
                    version: row.version.clone(),
                    sid: row.sid.clone(),
                });
                let value = self.row_value(m, input, row, keys, ctx)?;
                self.priced(m, input, keys, ctx, value)?
            }
            Found::None => Value::None,
            Found::Hole(q) => Value::Hole(Hole::asked(&q.text, &q.origin)),
        };
        Ok(as_read(value, (&input.id, keys), ctx, None))
    }

    /// Narrows the reading cell's law date class by what a read of `input` tests its law
    /// date against: every row's span, or the day itself for a reading, a calendar's days and
    /// a priced figure; true when the read's day is the law date, not a time key's (E §3.2,
    /// L §4.6 step 1).
    fn read_class(
        &mut self,
        m: &Reality,
        input: &InputInfo,
        ctx: Ctx,
        asked: Asked,
    ) -> Result<bool, Stop> {
        let Some(day) = ctx.law else {
            return Ok(false);
        };
        if input.reading || asked == Asked::Calendar || input.prices.is_some() {
            self.narrow_to_day(ctx.law);
            return Ok(true);
        }
        if matches!(input.time_pos, TimePos::At(_)) {
            return Ok(false);
        }
        let class = rows_class(m, &input.rows, ctx.cut, day)?;
        self.narrow(ctx.law, class);
        Ok(true)
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
        let value = self.row_converted(m, input, row, &figure)?;
        let point = time_point(input, keys, ctx.law)?;
        let is_held = held(m, row, point.day, self.clocks(ctx, &input.id)?)?;
        if let (Some(day), false) = (ctx.law, matches!(input.time_pos, TimePos::At(_))) {
            if is_held {
                // the `held` entry names the day (L §8.2)
                self.narrow_to_day(ctx.law);
            } else {
                self.unheld_class(m, row, ctx, day)?;
            }
        }
        if is_held {
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

    /// The keys of a read that are movements, each checked to be a movement of its position's
    /// event kind or a subkind (refcore's `check_key`, a movement). L §5.7, §5.11.
    ///
    /// # Errors
    ///
    /// `RUN-KEY` for a movement of another kind.
    pub(super) fn movement_keys(
        &mut self,
        m: &Reality,
        binds: &[BindInfo],
        keys: &[KeyVal],
        ctx: Ctx,
    ) -> Result<(), Stop> {
        for (bind, key) in binds.iter().zip(keys) {
            let (Domain::Kind(kind), KeyVal::Mv(mv)) = (&bind.domain, key) else {
                continue;
            };
            if !self.mv_kind_in(m, mv, kind, ctx)? {
                return Err(refuse(
                    Run::Key,
                    format!("{} is not a movement of {kind}", mv.id),
                ));
            }
        }
        Ok(())
    }

    /// `last(x[k], d)`: the row of x whose span began latest on or before d, past its end
    /// too, its value marked `held` for d in the register, except a row that governs `always`,
    /// which is never `held`; a reading's latest on or before d (refcore's `e_lastx`). A key
    /// or a day that is a hole makes it that hole. L §4.6 (Past the last row, Readings), §5.3,
    /// §8.2.
    pub(super) fn last_read(
        &mut self,
        m: &Reality,
        (read, day): (&Expr, &Expr),
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let latest = match self.latest_row(m, (read, day), Past::Last, env, ctx)? {
            Err(value) => return Ok(value),
            Ok(latest) => latest,
        };
        let input = m
            .inputs
            .get(&latest.input)
            .ok_or_else(|| invariant("an input of last load didn't build"))?;
        let row = latest_of(input, latest.at)?;
        let figure = origin(&input.id, &latest.keys);
        let value = self.row_converted(m, input, row, &figure)?;
        let always = row.instant.is_none()
            && crate::read::row_span(m, row, ctx.cut)?.is_some_and(|span| span.is_always());
        if !always {
            let text = format!(
                "{figure} = {} ({}), read for {}",
                shown(m, &value)?,
                row.version,
                latest.day.text()
            );
            self.note("held", text, String::new());
        }
        // a read of x at its keys, its day d part of the read (L §4.4, §4.6)
        Ok(as_read(
            value,
            (&input.id, &latest.keys),
            ctx,
            Some(latest.day),
        ))
    }

    /// `since(x[k], d)`: the days from the last reading of x on or before d to d, a hole
    /// where `last(x[k], d)` is one (refcore's `e_call_since`). L §4.6 Readings.
    pub(super) fn since_read(
        &mut self,
        m: &Reality,
        call: &Call,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let [read, day] = call.args.as_slice() else {
            return Err(invariant("since's two arguments, which load counts"));
        };
        let latest = match self.latest_row(m, (read, day), Past::Since, env, ctx)? {
            Err(value) => return Ok(value),
            Ok(latest) => latest,
        };
        let input = m
            .inputs
            .get(&latest.input)
            .ok_or_else(|| invariant("an input of since load didn't build"))?;
        let Some(RowInstant::At(at, _)) = &latest_of(input, latest.at)?.instant else {
            return Err(invariant("since read a reading with no day"));
        };
        let days = at
            .days_to(latest.day)
            .ok_or_else(|| invariant("the days from a reading"))?;
        Ok(Value::int(days))
    }

    /// The row `last(x[k], d)` and `since(x[k], d)` read, recorded as read, with its keys and
    /// d; or the value they are instead: the hole of a key or of d, a window d's
    /// `HOLE-IMPRECISE`, a pin of x at its keys (`last`'s, as refcore's `e_lastx`), `none`,
    /// the gap rule's hole. A movement of another kind as a key is refused (`RUN-KEY`, as
    /// every read). L §4.6 (Past the last row, Readings), §4.4, §5.7, §5.11.
    fn latest_row(
        &mut self,
        m: &Reality,
        (read, day): (&Expr, &Expr),
        past: Past,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Result<Latest, Value>, Stop> {
        let what = past.word();
        let (name, written) =
            read_parts(read).ok_or_else(|| invariant("a read of last load didn't check"))?;
        let input = m
            .inputs
            .get(name)
            .ok_or_else(|| invariant("an input of last load didn't build"))?;
        let given = self.given_keys(m, written, env, ctx)?;
        let keys = match make_keys(m, name, &input.binds, given, ctx.cut)? {
            Err(hole) => return Ok(Err(Value::Hole(hole))),
            Ok(keys) => keys,
        };
        self.movement_keys(m, &input.binds, &keys, ctx)?;
        let day = self.ev(m, day, env, ctx)?;
        if let Value::Hole(_) = day {
            return Ok(Err(day));
        }
        let day = match one_day(m, &day, &format!("the day of {what}"))? {
            Ok(day) => day,
            Err(hole) => return Ok(Err(Value::Hole(hole))),
        };
        if past == Past::Last
            && let Some(pinned) = self.pin_of(&input.id, &keys, ctx)?
        {
            return Ok(Err(pinned));
        }
        self.read(Read::Input {
            id: input.id.clone(),
            keys: keys.clone(),
        });
        let at = match read_last(m, input, &keys, day, self.clocks(ctx, &input.id)?)? {
            Found::Row(at) => at,
            Found::None => return Ok(Err(Value::None)),
            Found::Hole(q) => return Ok(Err(Value::Hole(Hole::asked(&q.text, &q.origin)))),
        };
        let row = latest_of(input, at)?;
        self.read(Read::Row {
            version: row.version.clone(),
            sid: row.sid.clone(),
        });
        Ok(Ok(Latest {
            input: name.to_owned(),
            keys,
            at,
            day,
        }))
    }

    /// A chosen row's value with the register entries every read of it makes: `assumed`, its
    /// own and its version's, and its keys' indexes' (refcore's `row_value`, before `held`).
    /// L §4.6, §8.2.
    fn row_converted(
        &mut self,
        m: &Reality,
        input: &InputInfo,
        row: &Row,
        figure: &str,
    ) -> Result<Value, Stop> {
        let figure = figure.to_owned();
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
        let of = ConvertFor::row(&input.unit_factor, ty, &input.binds, &row.pattern);
        let value = match convert(m, &row.value, ty, &input.id, &of)? {
            Converted::Hole(text) => Value::Hole(Hole::asked(&text, &figure)),
            Converted::Value(Value::Window(mut w)) => {
                // its holes are asked of this row (L §8.1); the read that made it is stamped
                // by `read_input_at`
                w.figure = Some(figure.clone());
                Value::Window(w)
            }
            Converted::Value(v) => v,
        };
        Ok(value)
    }

    /// Narrows the reading cell's law date class, for a row read for law date `day` and not
    /// `held`, to the day's side of the ask's cut where a later day would hold it (L §8.2,
    /// E §3.2).
    fn unheld_class(&mut self, m: &Reality, row: &Row, ctx: Ctx, day: Day) -> Result<(), Stop> {
        if row.instant.is_none()
            && let Some(span) = crate::read::row_span(m, row, ctx.cut)?
            && may_hold(&span)
        {
            let class = LawClass::all().by_ask_cut(day, self.ask_cut);
            self.narrow(ctx.law, class);
        }
        Ok(())
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
        let keys = ix.keys();
        let at_year = keys
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
            Given::Literal(lit) => literal_key(m, &lit, &bind.domain)?.ok_or_else(lacks_at_run)?,
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
            if let Value::Mv(mv) = v {
                // a key of an event kind is a movement; its kind is checked where the walk
                // can find it (`movement_keys`, L §5.7, §5.11)
                return Ok(KeyVal::Mv(mv.clone()));
            }
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
/// period of a period index, a key of a list or open index as it is (refcore's `to_key`);
/// an integer past 64 bits is no key of an integer index, whose keys have at most 18 digits
/// (L §7.6 item 3), `RUN-KEY`.
fn index_key(m: &Reality, item: &str, ix: &IndexInfo, v: &Value) -> Result<KeyVal, Stop> {
    if ix.is_int {
        if let Value::Number(n) = v
            && n.q.to_i64().is_none()
            && let Some(text) = n.q.integer_text()
        {
            return Err(refuse(Run::Key, format!("{text} is outside {}", ix.id)));
        }
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
/// key equal to the period or holding it (refcore's `to_key`, `holding` and `period`); the
/// end of time is no day a key holds, `RUN-LIMIT`. L §4.6, §7.6 item 1.
fn period_key(m: &Reality, item: &str, ix: &IndexInfo, v: &Value) -> Result<KeyVal, Stop> {
    let outside = |text: String| refuse(Run::Key, format!("{text} is outside {}", ix.id));
    if let Value::Day(day) = v {
        let day = a_day(*day)?;
        return ix.holding(day).cloned().ok_or_else(|| {
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
        for key in ix.keys() {
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
/// `resolve_key_literal`); `None` for a literal the domain doesn't hold, which its caller
/// refuses. L §2.5.
///
/// # Errors
///
/// An invariant for an index load didn't build; `NotBuilt` for a key list, a range or an
/// axis.
pub(super) fn literal_key(m: &Reality, lit: &Lit, domain: &Domain) -> Result<Option<KeyVal>, Stop> {
    let text = lit.text();
    match domain {
        Domain::Index(id) => {
            let ix = m
                .indexes
                .get(id)
                .ok_or_else(|| invariant("a bind over an index load didn't build"))?;
            Ok(index_key_of(ix, lit))
        }
        Domain::Kind(kind) => Ok(m.thing_in(&text, kind).then_some(KeyVal::Thing(text))),
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
            if let KeyVal::Mv(_) = key {
                return Ok(());
            }
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

/// A literal key its index or kind lacks, met at run: load refuses it with its place
/// (`key_value`), refcore's run with `RUN-KEY`, so the run stops "not implemented".
pub(super) fn lacks_at_run() -> Stop {
    Stop::NotBuilt("a literal key its index or kind lacks, met at run")
}

/// Whether a unit has a term `$v`, a bound variable's key (L §3.2).
pub(super) fn keyed(unit: &Unit) -> bool {
    unit.terms().iter().any(|(base, _)| base.starts_with('$'))
}
