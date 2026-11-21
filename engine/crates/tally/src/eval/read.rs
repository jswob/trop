//! Reads of inputs: the row the read rule takes for keys already made, and the row's value with its register entries.
//! Spec: L §4.6 (the read rule; a row's value, `assumed`, `held`), §5.6 (`assumed` on an
//! index), §5.8 (inputs and their values), §3.4 (a figure in Y prices), §8.1 (a read whose
//! key is a hole carries the key's questions only), §8.2 (the register entries a read makes),
//! §8.4 (`RUN-KEY`, `RUN-NONE`), `last(x, d)` and `since(x, d)` (§4.6 Past the last row,
//! Readings; §5.3); §4.4 (a window read is that read's value, its holes asked of its row).
//! A read's keys made keys of their positions are `keys.rs`'s (§2.5); a row's value converted
//! to its input's type is `convert.rs`'s; a scenario's expression row evaluated, `srows.rs`'s
//! (§6.4).
//! Twin: engine/tools/refcore/evaluate.py (`read_input`, `row_value`, `e_lastx`,
//! `e_call_since`, `prices`, `index_assumed`).
//! Invariants: a row's value has its input's declared type; a hole is built in the hole
//! module.

use super::arith::{Op, arith};
use super::builtin::read_parts;
use super::convert::{ConvertFor, Converted, convert_read};
use super::keys::Given;
use super::memo::{LawClass, Read, may_hold, rows_class};
use super::state::{Ctx, Engine, Env, invariant, num_stop, refuse, refuse_load};
use super::windows::{as_read, held_by, named_window, one_day};
use crate::base::{Load, Run};
use crate::hole::{Hole, spread};
use crate::model::{
    BindInfo, Class, Domain, InputInfo, KeyVal, Number, Pat, Reality, Row, TimePos, Unit, Value,
    World,
};
use crate::read::{
    Asked, Clocks, Found, held, last_began, origin, read_input, read_last, row_span, standing_rows,
    time_point,
};
use crate::syntax::{Call, Expr, Stop, Value as Written};
use crate::time::{Day, PeriodKind};

/// Which read past a row's span: `last(x, d)` or `since(x, d)` (L §4.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Past {
    /// `last(x, d)`.
    Last,
    /// `since(x, d)`.
    Since,
    /// `value(…) by last(p)` or `priced last(p)`: `last(p, d)` read as a price, where no row
    /// on or before d is no price, `none` (L §5.10).
    Price,
}

impl Past {
    /// Its word, as a message names it.
    fn word(self) -> &'static str {
        match self {
            Past::Last | Past::Price => "last",
            Past::Since => "since",
        }
    }
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

/// `HOLE-NOREADING` of `figure` on or before day `day`: "<x>[k] has no reading on or before
/// <d>" (L §4.6 Readings, §8.1; refcore's `e_lastx`).
fn no_reading(figure: &str, day: Day) -> Hole {
    Hole::asked(
        &format!("{figure} has no reading on or before {}", day.text()),
        figure,
    )
}

/// The cell a read of an input names its window by: the row its read rule took, which the
/// law date, cut and pins the read depends on decide (L §4.4; refcore's `('row', sid, …)`).
pub(super) fn row_cell(row: &Row) -> String {
    format!("row {}", row.sid)
}

/// Whether `last(x, d)` reads its row past it, for day `day`: a reading past its instant's
/// day, a state row past its span's end; a row that governs `always` is timeless (L §4.6
/// Past the last row, Readings; §8.2 `held`; refcore's `last_value`).
fn read_past(m: &Reality, row: &Row, (day, seen): (Day, (Day, World))) -> Result<bool, Stop> {
    if let Some(instant) = &row.instant {
        return Ok(day > instant.day);
    }
    let Some(span) = row_span(m, row, seen)? else {
        return Ok(false);
    };
    Ok(!span.is_always() && span.end().is_some_and(|end| day >= end))
}

impl Engine {
    /// The clocks of a read of `item` in context `ctx`: the law date, the cut, the ask's cut,
    /// the day a `wording` pin names the item with, and the world (L §4.3, §4.5, §6.4).
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
            world: ctx.world,
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
            let value = pinned_at(pinned, &input.binds, keys)?;
            return Ok(Self::pinned_window(value, &origin(&input.id, keys)));
        }
        let clocks = self.clocks(ctx, &input.id)?;
        #[cfg(feature = "counters")]
        self.counts
            .read(format!("{}|{}", origin(&input.id, keys), ctx.key()));
        self.read(Read::Input {
            id: input.id.clone(),
            keys: keys.to_vec(),
        });
        // every row's span decides the read's class
        let by_law = self.read_class(m, input, ctx, asked)?;
        let found = read_input(m, input, keys, clocks, asked)?;
        if by_law && !matches!(found, Found::Row(_)) {
            // a thing's lifespan or the question's text: the day itself (L §4.6 step 5)
            self.narrow_to_day(ctx.law);
        }
        match found {
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
                let value = self.priced(m, input, keys, ctx, value)?;
                Ok(Self::row_window(value, (input, keys), row))
            }
            Found::None => Ok(Value::None),
            Found::Hole(q) => Ok(Value::Hole(Hole::asked(&q.text, &q.origin))),
        }
    }

    /// Row `row`'s value `value` as the read of input `input` at `keys` gives it: a window is
    /// named by that read, its row and figure; any other value is itself (L §4.4 "One unknown
    /// day, named by its read").
    fn row_window(value: Value, (input, keys): (&InputInfo, &[KeyVal]), row: &Row) -> Value {
        // only a window keeps its read (L §4.4)
        if !matches!(value, Value::Window(_)) {
            return value;
        }
        let value = as_read(value, (&input.id, keys), row_cell(row));
        let figure = origin(&input.id, keys);
        named_window(value, (format!("row {} {figure}", row.sid), &figure))
    }

    /// A pinned value `value` of figure `figure` named as an unknown day: a window written
    /// there, which carries no name of its own, is the unknown day of the figure pinned, its
    /// holes asked of it; one that came from a read, or was computed from one or from a
    /// calendar's entry, keeps the names and the days it carries (refcore's `pinned_read`: a
    /// pin whose `Iv` has no origin; L §4.4 "a copy keeps the name", "a computed value …
    /// carries the names it was computed from"; picks P30, P32, P26).
    pub(super) fn pinned_window(value: Value, figure: &str) -> Value {
        // a pinned window's holes are asked of the item it pins, as a row's of its row
        // (L §8.1, §5.11)
        held_by(value, (format!("pin {figure}"), figure))
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
        let class = rows_class(m, &input.rows, (ctx.cut, ctx.world), day)?;
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
        let point = time_point(input, keys, ctx.law)?;
        let at = Ctx {
            law: point.day.or(ctx.law),
            ..ctx
        };
        let value = self.row_converted(m, (input, row, keys), &figure, at)?;
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
            let shown = held_text(m, row, &value)?;
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

    /// `last(x[k], d)`: the row of x whose span began latest on or before d, past its end
    /// too, its value marked `held` for d in the register where d is past the row; a
    /// reading's latest on or before d (refcore's `e_lastx`). A key
    /// or a day that is a hole makes it that hole. L §4.6 (Past the last row, Readings), §5.3,
    /// §8.2.
    pub(crate) fn last_read(
        &mut self,
        m: &Reality,
        (read, day): (&Expr, &Expr),
        env: &Env,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        match self.latest_row(m, (read, day), Past::Last, env, ctx)? {
            Err(value) => Ok(value),
            Ok(latest) => self.last_value(m, &latest, ctx),
        }
    }

    /// The value of the row `last(x[k], d)` took: as read for d, `held` for d in the register
    /// where d is past the row (`read_past`): the one carry `last`, `by last(p)` and `priced
    /// last(p)` read (refcore's `last_value`; tally 2.1.7 R6). L §4.6, §8.2.
    fn last_value(&mut self, m: &Reality, latest: &Latest, ctx: Ctx) -> Result<Value, Stop> {
        let input = m
            .inputs
            .get(&latest.input)
            .ok_or_else(|| invariant("an input of last load didn't build"))?;
        let row = latest_of(input, latest.at)?;
        let figure = origin(&input.id, &latest.keys);
        let at = Ctx {
            law: Some(latest.day),
            ..ctx
        };
        let value = self.row_converted(m, (input, row, &latest.keys), &figure, at)?;
        // a reading `replace … = none` removed is no reading (L §5.19: "it removes the
        // reading it replaces"; L §4.6 Readings, `HOLE-NOREADING`; refcore's `e_lastx`): nothing
        // is read, so nothing is held (N1 of group 3's review)
        if input.reading && value == Value::None {
            return Ok(Value::Hole(no_reading(&figure, latest.day)));
        }
        if read_past(m, row, (latest.day, (ctx.cut, ctx.world)))? {
            let text = format!(
                "{figure} = {} ({}), read for {}",
                held_text(m, row, &value)?,
                row.version,
                latest.day.text()
            );
            self.note("held", text, String::new());
        }
        // a read of x at its keys, named by the row it took, whatever day d took it (L §4.4)
        let value = as_read(value, (&input.id, &latest.keys), row_cell(row));
        Ok(named_window(
            value,
            (format!("row {} {figure}", row.sid), &figure),
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
        let row = latest_of(input, latest.at)?;
        let instant = row
            .instant
            .ok_or_else(|| invariant("since read a reading with no day"))?;
        let days = instant
            .day
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
        let keys = match self.make_keys(m, name, &input.binds, given, ctx)? {
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
        self.latest_at(m, (input, keys), day, past, ctx)
    }

    /// The row of `input` at `keys` that `last` or `since` takes for day `day`, recorded as
    /// read; or the value it is instead: a pin of the input at its keys (`last`'s), `none`,
    /// the gap rule's hole (refcore's `last_of`). L §4.6, §5.11.
    fn latest_at(
        &mut self,
        m: &Reality,
        (input, keys): (&InputInfo, Vec<KeyVal>),
        day: Day,
        past: Past,
        ctx: Ctx,
    ) -> Result<Result<Latest, Value>, Stop> {
        if past != Past::Since
            && let Some(pinned) = self.pin_of(&input.id, &keys, ctx)?
        {
            let value = pinned_at(pinned, &input.binds, &keys)?;
            return Ok(Err(Self::pinned_window(value, &origin(&input.id, &keys))));
        }
        self.latest_read(m, (input, keys), day, past, ctx)
    }

    /// The row `last` or `since` of `input` at `keys` takes on day `day`, recorded as read; or
    /// the value it is instead: `none`, the gap rule's hole, `HOLE-NOREADING` (refcore's
    /// `last_of`). L §4.6.
    fn latest_read(
        &mut self,
        m: &Reality,
        (input, keys): (&InputInfo, Vec<KeyVal>),
        day: Day,
        past: Past,
        ctx: Ctx,
    ) -> Result<Result<Latest, Value>, Stop> {
        self.read(Read::Input {
            id: input.id.clone(),
            keys: keys.clone(),
        });
        let clocks = self.clocks(ctx, &input.id)?;
        let found = match past {
            // `since` reads only the day the latest rows began, which they share (L §4.6
            // Readings); a reading `replace … = none` removed is none (L §5.19), so with only
            // such it is `HOLE-NOREADING`
            Past::Since => match last_began(m, input, &keys, day, clocks)? {
                Ok(kept) => {
                    let stand = standing_rows(m, &input.rows, &kept)?;
                    let read = stand.into_iter().find(
                        |at| matches!(input.rows.get(*at), Some(row) if row.value != Written::None),
                    );
                    let Some(at) = read else {
                        let figure = origin(&input.id, &keys);
                        return Ok(Err(Value::Hole(no_reading(&figure, day))));
                    };
                    Found::Row(at)
                }
                Err(found) => found,
            },
            Past::Last | Past::Price => read_last(m, input, &keys, day, clocks)?,
        };
        let at = match found {
            Found::Row(at) => at,
            Found::None => return Ok(Err(Value::None)),
            // a price with no row on or before d: no price, which `value` names (refcore's
            // `last_of`)
            Found::Hole(_) if past == Past::Price => return Ok(Err(Value::None)),
            Found::Hole(q) => {
                return Ok(Err(Value::Hole(Hole::asked(&q.text, &q.origin))));
            }
        };
        let row = latest_of(input, at)?;
        self.read(Read::Row {
            version: row.version.clone(),
            sid: row.sid.clone(),
        });
        Ok(Ok(Latest {
            input: input.id.clone(),
            keys,
            at,
            day,
        }))
    }

    /// A price's read on day `day`: input or def `id` at the keys `given` (a thing, or a
    /// position's unit), at law date `day`, or the input through its carry, `last(id[keys],
    /// day)` (refcore's `m_price_read`, `m_priced_read`, their reads). L §5.10, §4.6.
    ///
    /// # Errors
    ///
    /// The refusals of the keys, the read rule and the row's value; `ENGINE-INVARIANT` for
    /// an id that load checked is an input or a def.
    pub(crate) fn price_at(
        &mut self,
        m: &Reality,
        (id, given): (&str, Vec<Given>),
        (day, last): (Day, bool),
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        match self.price_keys(m, (id, given), day, ctx)? {
            Err(hole) => Ok(Value::Hole(hole)),
            Ok(keys) => self.price_read_at(m, (id, &keys), (day, last), ctx),
        }
    }

    /// The keys a price is read at on day `day`: each given key made a key of its input's or
    /// def's binds; a hole key that hole (refcore's `key_values`, at the price's day). L §5.10,
    /// §2.5.
    ///
    /// # Errors
    ///
    /// The refusals of the keys; `ENGINE-INVARIANT` for an id that load checked is an input
    /// or a def.
    pub(crate) fn price_keys(
        &mut self,
        m: &Reality,
        (id, given): (&str, Vec<Given>),
        day: Day,
        ctx: Ctx,
    ) -> Result<Result<Vec<KeyVal>, Hole>, Stop> {
        let at = Ctx {
            law: Some(day),
            ..ctx
        };
        let binds = if let Some(input) = m.inputs.get(id) {
            &input.binds
        } else {
            &m.defs
                .get(id)
                .ok_or_else(|| invariant("a price that is neither an input nor a def"))?
                .binds
        };
        self.make_keys(m, id, binds, given, at)
    }

    /// A price's read on day `day` at keys already made: input or def `id` at law date
    /// `day`, or the input through its carry, `last(id[keys], day)` (refcore's
    /// `m_priced_read`, its reads). L §5.10, §4.6.
    ///
    /// # Errors
    ///
    /// The refusals of the read rule and the row's value; `ENGINE-INVARIANT` for an id that
    /// load checked is an input or a def.
    pub(crate) fn price_read_at(
        &mut self,
        m: &Reality,
        (id, keys): (&str, &[KeyVal]),
        (day, last): (Day, bool),
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let at = Ctx {
            law: Some(day),
            ..ctx
        };
        if let Some(input) = m.inputs.get(id) {
            if !last {
                return self.read_input_at(m, input, keys, at, Asked::Plain);
            }
            return match self.latest_at(m, (input, keys.to_vec()), day, Past::Price, at)? {
                Err(value) => Ok(value),
                Ok(latest) => self.last_value(m, &latest, at),
            };
        }
        let def = m
            .defs
            .get(id)
            .ok_or_else(|| invariant("a price that is neither an input nor a def"))?;
        self.read_def_at(m, def, keys, at)
    }

    /// A chosen row's value with the register entries every read of it makes: `assumed`, its
    /// own and its version's, and its keys' indexes' (refcore's `row_value`, before `held`);
    /// a scenario's expression row evaluated in `at`, at the row's law date. L §4.6, §8.2,
    /// §6.4.
    fn row_converted(
        &mut self,
        m: &Reality,
        (input, row, keys): (&InputInfo, &Row, &[KeyVal]),
        figure: &str,
        at: Ctx,
    ) -> Result<Value, Stop> {
        let figure = figure.to_owned();
        self.note_version(&row.version);
        self.note_row(&row.sid, &row.version, row.locator.as_ref());
        if m.scenarios.of(&row.version).is_some() {
            self.note_listed(&row.version, &input.id);
        }
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
        // the row's node under a trace, its value once known (E §10)
        let traced = self.trace_row(m, row);
        let value = if let Written::Expr(e) = &row.value {
            self.row_expr(m, (row, e), &figure, at)
        } else {
            row_value(m, (input, row, keys), &figure)
        };
        self.trace_value(traced, &value);
        value
    }

    /// Narrows the reading cell's law date class, for a row read for law date `day` and not
    /// `held`, to the day's side of the ask's cut where a later day would hold it (L §8.2,
    /// E §3.2).
    fn unheld_class(&mut self, m: &Reality, row: &Row, ctx: Ctx, day: Day) -> Result<(), Stop> {
        if row.instant.is_none()
            && let Some(span) = row_span(m, row, (ctx.cut, ctx.world))?
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
        if let Some(hole) = spread(&operands) {
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

/// Row `row`'s value `v` as its `held:` entry shows it: a row that states a hole, that hole
/// in its own form, `?"…" (x[k])`, as L §8.1 prints it (L §8.2: a hole lists them like a
/// value; tally 2.1.8 R7, R9, #179); a row computed whose value comes out a hole, `?`, since
/// that hole names the day read and the line prints once per row (L §8.2; pick P32).
fn held_text(m: &Reality, row: &Row, v: &Value) -> Result<String, Stop> {
    match (v, &row.value) {
        (Value::Hole(_), Written::Expr(_)) => Ok("?".to_owned()),
        (Value::Hole(h), _) => Ok(h.text()),
        _ => v.text(m).map_err(num_stop),
    }
}

/// A value as a register entry shows it: a hole as `?` (L §8.2).
pub(super) fn shown(m: &Reality, v: &Value) -> Result<String, Stop> {
    if let Value::Hole(_) = v {
        return Ok("?".to_owned());
    }
    v.text(m).map_err(num_stop)
}

/// Whether a unit has a term `$v`, a bound variable's key (L §3.2).
pub(super) fn keyed(unit: &Unit) -> bool {
    unit.terms().iter().any(|(base, _)| base.starts_with('$'))
}

/// A written row's value read at `keys` as `figure`, in its input's type: a stated hole asked
/// of it, a window's holes asked of it, a number at its keys (refcore's `row_value`). L §4.6,
/// §8.1.
fn row_value(
    m: &Reality,
    (input, row, keys): (&InputInfo, &Row, &[KeyVal]),
    figure: &str,
) -> Result<Value, Stop> {
    let ty = match (&input.ty, input.reading, row.pattern.first()) {
        (Some(ty), ..) => ty,
        // a balance reading's value is a quantity in its place's unit, typed at load (L §4.6
        // Readings, §5.19)
        (None, true, Some(Pat::Key(KeyVal::Thing(place)))) => m
            .balance_readings
            .get(place)
            .map(|of_place| &of_place.ty)
            .ok_or_else(|| invariant("a balance reading load didn't type"))?,
        (None, ..) => return Err(Stop::NotBuilt("the value of an observed row")),
    };
    let of = ConvertFor::row(&input.unit_factor, ty, &input.binds, &row.pattern);
    let value = match convert_read(m, &row.value, ty, &input.id, &of)? {
        Converted::Hole(text) => Value::Hole(Hole::asked(&text, figure)),
        Converted::Value(Value::Window(mut w)) => {
            // its holes are asked of this row (L §8.1); the read that made it is stamped by
            // `read_input_at`
            w.figure = Some(figure.to_owned());
            Value::Window(w)
        }
        Converted::Value(Value::Number(n)) => Value::Number(at_keys(n, &input.binds, keys)?),
        Converted::Value(v) => v,
    };
    Ok(value)
}

/// A pin's value as a read at `keys` takes it: a per-unit price pinned at `*` is in CUR per
/// the read key's unit, as a row is (L §3.2 Per-unit prices of a unit kind; §5.11).
pub(super) fn pinned_at(v: Value, binds: &[BindInfo], keys: &[KeyVal]) -> Result<Value, Stop> {
    if let Value::Number(n) = &v {
        return Ok(Value::Number(at_keys(n.clone(), binds, keys)?));
    }
    Ok(v)
}

/// A row's number, or a def's figure in its declared type, read at `keys`: a per-unit price
/// `num CUR/u` takes the unit of the key it is read at, the type's `$u` replaced (refcore's
/// `row_value`, its end). L §3.2 (Per-unit prices of a unit kind; Inline figures).
pub(super) fn at_keys(n: Number, binds: &[BindInfo], keys: &[KeyVal]) -> Result<Number, Stop> {
    let Class::Num(unit, shown) = &n.class else {
        return Ok(n);
    };
    if !keyed(unit) {
        return Ok(n);
    }
    let env: Vec<(String, String)> = binds
        .iter()
        .zip(keys)
        .map(|(bind, key)| (bind.var.clone(), key.text()))
        .collect();
    let unit = unit.subst(&env).map_err(super::state::unit_stop)?;
    Ok(Number {
        q: n.q,
        class: Class::Num(unit, *shown),
    })
}
