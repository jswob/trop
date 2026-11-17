//! Reads of inputs: the row the read rule takes for keys already made, and the row's value with its register entries.
//! Spec: L §4.6 (the read rule; a row's value, `assumed`, `held`), §5.6 (`assumed` on an
//! index), §5.8 (inputs and their values), §3.4 (a figure in Y prices), §8.1 (a read whose
//! key is a hole carries the key's questions only), §8.2 (the register entries a read makes),
//! §8.4 (`RUN-KEY`, `RUN-NONE`), `last(x, d)` and `since(x, d)` (§4.6 Past the last row,
//! Readings; §5.3); §4.4 (a window read is that read's value, its holes asked of its row). A
//! read's keys made keys of their positions are `keys.rs`'s (§2.5); a row's value converted
//! to its input's type is `convert.rs`'s; a scenario's expression row evaluated, `srows.rs`'s
//! (§6.4).
//! Twin: engine/tools/refcore/evaluate.py (`read_input`, `row_value`, `e_lastx`,
//! `e_call_since`, `prices`, `index_assumed`).
//! Invariants: a row's value has its input's declared type; a hole is built in the hole
//! module.

use std::collections::{BTreeMap, BTreeSet};

use super::arith::{Op, arith, same_value};
use super::builtin::read_parts;
use super::convert::{ConvertFor, Converted, convert_read};
use super::keys::Given;
use super::memo::{LawClass, Read, may_hold, rows_class};
use super::state::{Ctx, Engine, Env, invariant, num_stop, refuse, refuse_load};
use super::windows::{as_read, imprecise, known_hole, one_day};
use crate::base::{Load, Run};
use crate::book::{AfterDay, after_missing, and_list, reading_place};
use crate::hole::{Hole, spread};
use crate::model::{
    BindInfo, Class, Domain, InputInfo, KeyVal, Number, Pat, Reality, Row, RowInstant, TimePos,
    Unit, Value, World,
};
use crate::read::{
    Asked, Clocks, Found, KnownInside, Test, held, known_inside, last_began, need_law, origin,
    read_input, read_last, row_span, standing_rows, time_point,
};
use crate::syntax::{Call, Expr, Stop, Value as Written};
use crate::time::{Day, PeriodKind};
use crate::walk::Status;

/// One side of two rows stated `same`: the value of the row its read takes with the row's
/// id, or what the read finds where it takes none; and the read's day (L §6.1).
type SameSide = (Result<(Value, String), Found>, Option<Day>);

/// The other side of a `same` line of rows: its input and keys (L §6.1).
pub(super) type SameRow = (String, Vec<KeyVal>);

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

/// What one side of `last(x[k], d)` or `since(x[k], d)` takes (L §4.6, §6.1).
#[derive(Debug, Clone, PartialEq, Eq)]
enum Side {
    /// The row it reads.
    Row(Latest),
    /// A known window's hole, which is the read's whatever the other sides hold (L §4.5).
    Signal(Value),
    /// No row: `none`, the gap rule's hole or `HOLE-NOREADING`, the read's own where no side
    /// has a row.
    Missing(Value),
}

/// Whether the row at position `at` of `input` is a removal, `replace … = none` (L §5.19,
/// §4.6 Readings).
fn removed_row(input: &InputInfo, at: usize) -> bool {
    matches!(input.rows.get(at), Some(row) if row.value == Written::None)
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

/// The rows of `rows` but those at the positions `hidden`, which the cut doesn't see under
/// the line's choice (L §4.5).
pub(super) fn kept_rows(rows: &[Row], hidden: &[usize]) -> Vec<Row> {
    let mut out = Vec::new();
    for (at, row) in rows.iter().enumerate() {
        if !hidden.contains(&at) {
            out.push(row.clone());
        }
    }
    out
}

/// Whether `row` is a reading `after m` of `place`. L §4.6 Readings, step 1.
fn after_at(row: &Row, place: Option<&str>) -> bool {
    matches!(row.instant, Some(RowInstant::After(_))) && reading_place(row) == place
}

/// A reading row's instant `after m` as text, `after save` (refcore's `instant_text`). L §4.6
/// Readings.
fn after_text(row: &Row) -> String {
    match &row.instant {
        Some(RowInstant::After(node)) => format!("after {}", crate::load::expr_text(node)),
        Some(RowInstant::At(..)) | None => String::new(),
    }
}

/// Whether `last(x, d)` reads its row past it, for day `day`: a reading past its instant's
/// day, a state row past its span's end; a row that governs `always` is timeless (L §4.6
/// Past the last row, Readings; §8.2 `held`; refcore's `last_value`).
fn read_past(
    m: &Reality,
    row: &Row,
    (day, seen): (Day, (Day, World)),
    after: Option<Day>,
) -> Result<bool, Stop> {
    match &row.instant {
        Some(RowInstant::At(at, _)) => Ok(day > *at),
        // a reading after m stands on the day the walk applies it (L §4.6 Readings)
        Some(RowInstant::After(_)) => Ok(after.is_some_and(|at| day > at)),
        None => {
            let Some(span) = row_span(m, row, seen)? else {
                return Ok(false);
            };
            Ok(!span.is_always() && span.end().is_some_and(|end| day >= end))
        }
    }
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
            return self.pin_chosen(value, &origin(&input.id, keys), ctx);
        }
        // rows stated `same` are read together, the whole class, a reading's at its instant
        // (L §6.1 l.1806, §5.19; tally 2.1.8 §4)
        if let Some(class) = self.same_class(m, (&input.id, keys), ctx)? {
            return self.read_same_rows(m, (input, keys), &class, ctx, asked);
        }
        let clocks = self.clocks(ctx, &input.id)?;
        #[cfg(feature = "counters")]
        self.counts
            .read(format!("{}|{}", origin(&input.id, keys), ctx.key()));
        self.read(Read::Input {
            id: input.id.clone(),
            keys: keys.to_vec(),
        });
        // every row's span, the ones a known window hides included, decides the read's class
        let by_law = self.read_class(m, input, ctx, asked)?;
        // a row known in a window that holds the cut: the line's `windows each` decides it,
        // else the read is `HOLE-IMPRECISE` (L §4.5)
        let view = match self.input_view(m, input, keys, (None, clocks), ctx)? {
            Err(h) => return Ok(Value::Hole(h)),
            Ok(view) => view,
        };
        let input = match &view {
            Some(seen) => seen,
            None => input,
        };
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
                self.row_window(value, (input, keys), row, ctx)
            }
            Found::None => Ok(Value::None),
            Found::Hole(q) => Ok(Value::Hole(Hole::asked(&q.text, &q.origin))),
        }
    }

    /// Row `row`'s value `value` as the read of input `input` at `keys` gives it: a window is
    /// named by that read, its row and figure, and under `windows each` takes its day; any
    /// other value is itself (L §4.4 "One unknown day, named by its read").
    ///
    /// # Errors
    ///
    /// As [`Engine::chosen`].
    fn row_window(
        &mut self,
        value: Value,
        (input, keys): (&InputInfo, &[KeyVal]),
        row: &Row,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        // only a window keeps its read, and only a window takes a day (L §4.4)
        if !matches!(value, Value::Window(_)) {
            return Ok(value);
        }
        let value = as_read(value, (&input.id, keys), row_cell(row));
        let figure = origin(&input.id, keys);
        self.chosen(value, (format!("row {} {figure}", row.sid), &figure), ctx)
    }

    /// The rows stated `same` with input `id`'s at `keys`, that row among them, each as its
    /// input and keys, in the order of their texts compared by code point; none where no
    /// line the cut of `ctx` sees names it. `same` is an equivalence: lines that share a side
    /// make one class, each line seen as [`Engine::same_line_seen`] decides, and only the
    /// lines that reach the row are asked (refcore's `same_class`). L §6.1, §2.5, §4.5 (tally
    /// 2.1.8 R12, S6).
    ///
    /// # Errors
    ///
    /// As [`Engine::same_line_seen`].
    pub(super) fn same_class(
        &mut self,
        m: &Reality,
        (id, keys): (&str, &[KeyVal]),
        ctx: Ctx,
    ) -> Result<Option<Vec<SameRow>>, Stop> {
        let start: SameRow = (id.to_owned(), keys.to_vec());
        let mut class = vec![start.clone()];
        let mut todo = vec![start];
        while let Some((x, xk)) = todo.pop() {
            for pair in &m.sames {
                let Some((other, other_keys, version)) = pair.other_row(&x, &xk) else {
                    continue;
                };
                if class
                    .iter()
                    .any(|(y, yk)| y == other && yk.as_slice() == other_keys)
                {
                    continue;
                }
                let sides = [origin(&x, &xk), origin(other, other_keys)];
                let [ours, theirs] = &sides;
                if self.same_line_seen(m, version, ctx, [ours, theirs])? {
                    let next: SameRow = (other.to_owned(), other_keys.to_vec());
                    class.push(next.clone());
                    todo.push(next);
                }
            }
        }
        if class.len() < 2 {
            return Ok(None);
        }
        class.sort_by_cached_key(|(y, yk)| origin(y, yk));
        Ok(Some(class))
    }

    /// A read of input `input` at `keys`, whose row is stated `same` with the rows of
    /// `class` (itself among them): every side read by the read rule, a price at its own
    /// prices; where only one side has a row its value is the fact, a hole on any side is the
    /// hole, values that agree are the value, and values that differ are `HOLE-SAME` naming
    /// each row that has one, in id order; where no side has one, `input`'s read finds none
    /// (refcore's `read_same_row`; a joint model's own row that settles them is M5c's).
    /// L §6.1 What `same` does, §5.19, §8.1 (tally 2.1.8 S6, §4).
    ///
    /// # Errors
    ///
    /// The refusals of every side's read rule and row value; `ENGINE-INVARIANT` for an input
    /// load didn't build.
    fn read_same_rows(
        &mut self,
        m: &Reality,
        (input, keys): (&InputInfo, &[KeyVal]),
        class: &[SameRow],
        ctx: Ctx,
        asked: Asked,
    ) -> Result<Value, Stop> {
        // each side with a row: its text, its value and the row's id, in the class's order
        let mut got: Vec<(String, Value, String)> = Vec::new();
        let (mut own_found, mut own_day, mut first_day) = (Found::None, None, None);
        for (id, ks) in class {
            let info = m
                .inputs
                .get(id)
                .ok_or_else(|| invariant("a row stated same of an input load didn't build"))?;
            let (side, day) = match self.same_side(m, (info, ks), ctx, asked)? {
                Err(h) => return Ok(Value::Hole(h)),
                Ok(side) => side,
            };
            let own = *id == input.id && ks.as_slice() == keys;
            if own {
                own_day = day;
            }
            first_day = first_day.or(day);
            match side {
                Ok((v, sid)) => got.push((origin(id, ks), v, sid)),
                Err(found) if own => own_found = found,
                Err(_) => {}
            }
        }
        let holes: Vec<&Hole> = got
            .iter()
            .filter_map(|(_, v, _)| {
                if let Value::Hole(h) = v {
                    Some(h)
                } else {
                    None
                }
            })
            .collect();
        match holes.as_slice() {
            [] => {}
            [one] => return Ok(Value::Hole((*one).clone())),
            many => {
                let all: Vec<Option<&Hole>> = many.iter().copied().map(Some).collect();
                let spread_out =
                    spread(&all).ok_or_else(|| invariant("two holes spread to none"))?;
                return Ok(Value::Hole(spread_out));
            }
        }
        let Some(((_, first, _), rest)) = got.split_first() else {
            return Ok(match own_found {
                Found::Hole(q) => Value::Hole(Hole::asked(&q.text, &q.origin)),
                Found::None | Found::Row(_) => Value::None,
            });
        };
        let mut agree = true;
        for (_, v, _) in rest {
            agree = agree && same_value(m, first, v)?;
        }
        if agree {
            return Ok(first.clone());
        }
        let day = own_day
            .or(first_day)
            .map_or_else(|| "every day".to_owned(), Day::text);
        let names: Vec<String> = got.iter().map(|(t, _, _)| t.clone()).collect();
        let mut parts = Vec::with_capacity(got.len());
        for (_, v, sid) in &got {
            parts.push(format!("{} ({sid})", v.text(m).map_err(num_stop)?));
        }
        let text = format!(
            "{} are one, but the value differs on {day}: {}",
            and_list(&names),
            and_list(&parts)
        );
        Ok(Value::Hole(Hole::asked(&text, &names.join(", "))))
    }

    /// One side of two rows stated `same`: the value of the row the read rule takes for
    /// `input` at `keys`, with the row's id, or what the read finds where it takes none; and
    /// the read's day (refcore's `read_same_row`, its `side`). The hole where no `windows
    /// each` decides a row known in a window. L §6.1, §4.6, §4.5.
    ///
    /// # Errors
    ///
    /// The refusals of the read rule and of the row's value.
    fn same_side(
        &mut self,
        m: &Reality,
        (input, keys): (&InputInfo, &[KeyVal]),
        ctx: Ctx,
        asked: Asked,
    ) -> Result<Result<SameSide, Hole>, Stop> {
        let clocks = self.clocks(ctx, &input.id)?;
        self.read(Read::Input {
            id: input.id.clone(),
            keys: keys.to_vec(),
        });
        let by_law = self.read_class(m, input, ctx, asked)?;
        let view = match self.input_view(m, input, keys, (None, clocks), ctx)? {
            Err(h) => return Ok(Err(h)),
            Ok(view) => view,
        };
        let input = match &view {
            Some(seen) => seen,
            None => input,
        };
        let day = time_point(input, keys, ctx.law)?.day;
        let found = read_input(m, input, keys, clocks, asked)?;
        let Found::Row(at) = found else {
            if by_law {
                self.narrow_to_day(ctx.law);
            }
            return Ok(Ok((Err(found), day)));
        };
        let row = input
            .rows
            .get(at)
            .ok_or_else(|| invariant("the read rule took a row the input lacks"))?;
        self.read(Read::Row {
            version: row.version.clone(),
            sid: row.sid.clone(),
        });
        let value = self.row_value(m, input, row, keys, ctx)?;
        // each side at its own prices (L §3.4, §6.1: every read of either, a price's included)
        let value = self.priced(m, input, keys, ctx, value)?;
        // a window read through a `same` line is its row's read, as a plain read's (L §4.4)
        let value = self.row_window(value, (input, keys), row, ctx)?;
        Ok(Ok((Ok((value, row.sid.clone())), day)))
    }

    /// A pinned value `value` of figure `figure` as a `windows each` in `ctx` takes it: a
    /// pinned window is a window the line meets, so the line splits by it; one that came from
    /// a read is that read's window, taken with its own name once, and a literal is named by
    /// the figure pinned (L §4.4; picks P30, P32).
    ///
    /// # Errors
    ///
    /// As [`Engine::chosen`].
    pub(super) fn pin_chosen(
        &mut self,
        value: Value,
        figure: &str,
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let Value::Window(mut w) = value else {
            return Ok(value);
        };
        // a pinned window's holes are asked of the item it pins, as a row's of its row
        // (L §8.1, §5.11)
        if w.figure.is_none() {
            w.figure = Some(figure.to_owned());
        }
        let (id, label) = match &w.read {
            Some(read) => {
                let from = origin(&read.item, &read.keys);
                (format!("{} {from}", read.cell), from)
            }
            None => (format!("pin {figure}"), figure.to_owned()),
        };
        self.chosen(Value::Window(w), (id, &label), ctx)
    }

    /// Input `input` as a read at `keys` in `ctx` under `clocks` sees its rows, a read at its
    /// law date or, for `last(x, d)` and `since`, on or before day `past`: a row whose `known`
    /// window holds the cut and that applies at the read's time is left out where the line's
    /// choice puts its day after the cut; none where every row stands, at once for an input
    /// with no row known in a window and none `after m`; the hole where no `windows each`
    /// decides one (refcore's `row_visible`, `known_vis`). L §4.5, §4.4, §4.6 step 1.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a freed layer or a row whose version is missing; as
    /// [`read_input`] for a dated read with no law date.
    pub(super) fn input_view(
        &mut self,
        m: &Reality,
        input: &InputInfo,
        keys: &[KeyVal],
        (past, clocks): (Option<Day>, Clocks),
        ctx: Ctx,
    ) -> Result<Result<Option<InputInfo>, Hole>, Stop> {
        if !input.holds.known_window && !input.holds.after && !input.holds.unitless {
            return Ok(Ok(None));
        }
        let test = match past {
            Some(day) => Test::Last(day),
            // a reading stands from its instant on (L §4.6 Readings)
            None if input.reading => match clocks.law {
                Some(day) => Test::Last(day),
                None => Test::At(None),
            },
            None => {
                let point = time_point(input, keys, clocks.law)?;
                need_law(m, input, point.day)?;
                Test::At(point.day)
            }
        };
        let inside = known_inside(m, &input.rows, (&input.binds, keys), test, clocks)?;
        let hidden = match self.hidden_rows(&inside, ctx)? {
            Err(h) => return Ok(Err(h)),
            Ok(hidden) => hidden,
        };
        // a read takes its keys' rows only (L §4.6 step 1): a reading after m at another
        // place plans nothing here (L §5.14.2)
        let place = if let Some(KeyVal::Thing(p)) = keys.first() {
            Some(p.as_str())
        } else {
            None
        };
        // only `stated` and `opening` hold readings `after m`
        let after = input.holds.after && input.rows.iter().any(|row| after_at(row, place));
        let held = match place {
            Some(p) if input.holds.unitless => self.unitless_held(m, &input.rows, p, ctx)?,
            Some(_) | None => BTreeMap::new(),
        };
        if hidden.is_empty() && !after && held.is_empty() {
            return Ok(Ok(None));
        }
        let mut rows = kept_rows(&input.rows, &hidden);
        if after
            && let Err(h) = self.after_on_days(m, (&input.id, &mut rows), (place, test), ctx)?
        {
            return Ok(Err(h));
        }
        Ok(Ok(Some(InputInfo {
            rows,
            held,
            ..input.clone()
        })))
    }

    /// For each instant where a reading row of `place` among `rows` states no unit: the units
    /// the walk of the read's cut up to the instant's day applied a reading in there, its gaps
    /// at that instant, alike for every reader at that cut, inside a walk or not (L §5.10;
    /// tally 2.1.8 R14, B4 of group 3's review, pick P46; refcore's `units_at_reading`).
    fn unitless_held(
        &mut self,
        m: &Reality,
        rows: &[Row],
        place: &str,
        ctx: Ctx,
    ) -> Result<BTreeMap<String, BTreeSet<String>>, Stop> {
        let mut out = BTreeMap::new();
        let mut instants = Vec::new();
        for row in rows {
            if let Some(RowInstant::At(day, Some(edge))) = row.instant
                && reading_place(row) == Some(place)
                && m.reading_ty(place, &row.value).map_err(num_stop)?.is_none()
            {
                instants.push((day, crate::book::instant_text(day, edge)));
            }
        }
        for (day, text) in instants {
            // the cut's walk up to the day, which lists no plan after it; one being walked
            // already holds its gaps of the days it passed (refcore's `prefix_walk`, its
            // running walk); stated units alone where it is refused at run
            let Some(id) = self.unitless_walk(m, ctx.cut, day)? else {
                out.insert(text, BTreeSet::new());
                continue;
            };
            let walk = self.walk(id)?;
            let mut units = BTreeSet::new();
            for g in walk
                .gaps
                .get(&(place.to_owned(), day))
                .into_iter()
                .flatten()
            {
                if g.instant == text {
                    units.insert(g.unit.clone());
                }
            }
            out.insert(text, units);
        }
        Ok(out)
    }

    /// The walk of cut `cut` up to day `day` whose gaps give a unitless row its units: one
    /// kept to that day or a later one, if it is being walked, else the prefix walk; none
    /// where it is refused at run (refcore's `prefix_walk`). L §5.10, E §3.2, §4.1.
    fn unitless_walk(&mut self, m: &Reality, cut: Day, day: Day) -> Result<Option<usize>, Stop> {
        let kept = self
            .prefixes
            .walks
            .range((cut, day)..)
            .next()
            .filter(|((c, _), id)| *c == cut && self.walking.contains(id))
            .map(|(_, id)| *id);
        if kept.is_some() {
            return Ok(kept);
        }
        self.prefix_walk(m, cut, Some(day))
    }

    /// Each reading `after m` of `place` among the rows of input `input` read on the day the
    /// walk at the read's cut applies it ([`Engine::row_after_day`]), so `last`, `since` and a
    /// plain read find it there: m's window's last day once the read's day `test` is past it,
    /// none before its first, the window's hole inside it (or at an instant in it); a hole
    /// where m's day or whether it exists is one; refused where m doesn't exist; one with no
    /// day keeps no instant a read finds; a reading of another place is left as it is (L §4.6
    /// Readings, §4.4; E §4.2 step 3; tally 2.1.8 R16, pick P29 on #175; refcore's
    /// `rows_for`, its readings after m).
    fn after_on_days(
        &mut self,
        m: &Reality,
        (input, rows): (&str, &mut [Row]),
        (place, test): (Option<&str>, Test),
        ctx: Ctx,
    ) -> Result<Result<(), Hole>, Stop> {
        for row in rows {
            if !after_at(row, place) {
                continue;
            }
            let day = match self.row_after_day(m, input, row, ctx)? {
                AfterDay::Day(day) => day,
                AfterDay::Window(w) => {
                    let (probe, exact) = match test {
                        Test::Last(day) => (Some(day), false),
                        Test::Instant(day) | Test::At(Some(day)) => (Some(day), true),
                        Test::At(None) => (None, false),
                    };
                    let Some(probe) = probe.filter(|d| *d >= w.lo && !(exact && *d > w.hi)) else {
                        continue;
                    };
                    if exact || probe < w.hi {
                        let what = format!("the day of {}", after_text(row));
                        return Ok(Err(imprecise(m, &w.value, &what)?));
                    }
                    w.hi
                }
                AfterDay::Hole(h) => return Ok(Err(h)),
                AfterDay::Refused(r) => return Err(Stop::Refused(r)),
                AfterDay::Unplaced => continue,
            };
            row.instant = Some(RowInstant::At(day, None));
        }
        Ok(Ok(()))
    }

    /// What m gives the reading row `after m` of input `input` read in context `ctx`, as the
    /// walk applies it ([`Engine::reading_day`]), and as m's run there judged it: `RUN-RECORD`
    /// where m's `when` was false at its point, or m is modelled past and nothing fulfils it;
    /// unplaced for any other row (L §4.6 Readings, §8.4; E §4.2 step 3; refcore's
    /// `after_day`).
    fn row_after_day(
        &mut self,
        m: &Reality,
        input: &str,
        row: &Row,
        ctx: Ctx,
    ) -> Result<AfterDay, Stop> {
        let (Some(RowInstant::After(node)), Some(place)) = (&row.instant, reading_place(row))
        else {
            return Ok(AfterDay::Unplaced);
        };
        let judged = self.reading_day(m, node, (input, place), ctx.cut)?;
        if !matches!(judged, AfterDay::Day(_)) {
            return Ok(judged);
        }
        let mid = crate::load::expr_text(node);
        let why = match self.run_status(m, &mid, ctx)? {
            Some((Status::None, _)) => "doesn't exist",
            Some((Status::Modelled, false)) => "is modelled past and nothing fulfils it",
            Some(_) | None => return Ok(judged),
        };
        Ok(AfterDay::Refused(after_missing((input, place, &mid), why)))
    }

    /// The positions of the rows `inside`, each known in a window that holds the cut, that
    /// the line's choice leaves out; the hole of the first where no `windows each` decides
    /// it (refcore's `known_vis`). L §4.5, §4.4.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a freed layer.
    pub(super) fn hidden_rows(
        &mut self,
        inside: &[KnownInside],
        ctx: Ctx,
    ) -> Result<Result<Vec<usize>, Hole>, Stop> {
        let mut out = Vec::new();
        for row in inside {
            match self.known_seen(ctx, row.window, &row.sid)? {
                Some(true) => {}
                Some(false) => out.push(row.at),
                None => return Ok(Err(known_hole(row.window, ctx.cut, &row.sid)?)),
            }
        }
        Ok(Ok(out))
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
        // a window's reading stands from its last day (pick P29 on #175)
        let after = match self.row_after_day(m, &input.id, row, ctx)? {
            AfterDay::Day(day) => Some(day),
            AfterDay::Window(w) => Some(w.hi),
            AfterDay::Hole(_) | AfterDay::Refused(_) | AfterDay::Unplaced => None,
        };
        if read_past(m, row, (latest.day, (ctx.cut, ctx.world)), after)? {
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
        self.chosen(value, (format!("row {} {figure}", row.sid), &figure), ctx)
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
        let at = match &row.instant {
            Some(RowInstant::At(at, _)) => *at,
            // a reading after m counts from the day the walk applies it; since reads the day
            // itself, so a window's is imprecise (L §4.6 Readings, §4.4)
            Some(RowInstant::After(_)) => match self.row_after_day(m, &input.id, row, ctx)? {
                AfterDay::Day(on) => on,
                AfterDay::Window(w) => {
                    let what = format!("the day of {}", after_text(row));
                    return Ok(Value::Hole(imprecise(m, &w.value, &what)?));
                }
                AfterDay::Hole(h) => return Ok(Value::Hole(h)),
                AfterDay::Refused(r) => return Err(Stop::Refused(r)),
                AfterDay::Unplaced => {
                    return Err(invariant("since took a reading after an m with no day"));
                }
            },
            None => return Err(invariant("since read a reading with no day")),
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
            return Ok(Err(self.pin_chosen(
                value,
                &origin(&input.id, &keys),
                ctx,
            )?));
        }
        // rows stated `same` carry together, the whole class (L §6.1; tally 2.1.8 §4)
        if let Some(class) = self.same_class(m, (&input.id, &keys), ctx)? {
            return self.latest_in_class(m, (input, &keys), &class, (day, past), ctx);
        }
        Ok(match self.latest_side(m, (input, keys), day, past, ctx)? {
            Side::Row(latest) => Ok(latest),
            Side::Signal(value) | Side::Missing(value) => Err(value),
        })
    }

    /// The row the class `class` of rows stated `same` with input `input`'s at `keys` carries
    /// for `last` or `since` on day `day`: of the sides' latest rows, those that began latest,
    /// the first side's; where two such disagree, `last`'s `HOLE-SAME` naming them on that day;
    /// with no side's row, what the read of `input` itself finds; a hole a side's known window
    /// gives is the read's (refcore's `carry_sides`, `carry_latest`, `last_value`). L §6.1
    /// l.1806, §4.6, §8.1 (tally 2.1.8 §4, items 40 and 41).
    fn latest_in_class(
        &mut self,
        m: &Reality,
        (input, keys): (&InputInfo, &[KeyVal]),
        class: &[SameRow],
        (day, past): (Day, Past),
        ctx: Ctx,
    ) -> Result<Result<Latest, Value>, Stop> {
        let mut rows: Vec<(Option<Day>, Latest)> = Vec::new();
        let mut own = None;
        for (id, ks) in class {
            let info = m
                .inputs
                .get(id)
                .ok_or_else(|| invariant("a row stated same of an input load didn't build"))?;
            let side = match self.latest_side(m, (info, ks.clone()), day, past, ctx)? {
                // a side whose readings are all removed (`replace … = none`) is no row: the
                // class's latest kept reading carries (L §6.1 l.1806, §4.6 Readings; refcore's
                // `carry_latest`, B1 of group 4's review); with no kept reading on any side the
                // read is none, as refcore's `last`
                Side::Row(latest) if info.reading && removed_row(info, latest.at) => {
                    Side::Missing(Value::None)
                }
                side @ (Side::Row(_) | Side::Signal(_) | Side::Missing(_)) => side,
            };
            match side {
                Side::Signal(value) => return Ok(Err(value)),
                Side::Missing(value) => {
                    if *id == input.id && ks.as_slice() == keys {
                        own = Some(value);
                    }
                }
                Side::Row(latest) => {
                    let began = self.latest_began(m, info, &latest, ctx)?;
                    rows.push((began, latest));
                }
            }
        }
        let Some(top) = rows.iter().map(|(began, _)| *began).max() else {
            return own
                .map(Err)
                .ok_or_else(|| invariant("a class with no row and no read of its own"));
        };
        let latest: Vec<Latest> = rows
            .into_iter()
            .filter(|(began, _)| *began == top)
            .map(|(_, l)| l)
            .collect();
        let Some(first) = latest.first() else {
            return Err(invariant("the latest rows of a class are none"));
        };
        // rows that govern always began on no day: they disagree on the day the read reads
        // them for, as a plain read there prints (L §8.1 HOLE-SAME's `differs on <d>`; refcore's
        // `e_call_last`, P42, group 4's review N3)
        if past != Past::Since
            && let Some(hole) = self.carry_disagrees(m, &latest, top.unwrap_or(day), ctx)?
        {
            return Ok(Err(Value::Hole(hole)));
        }
        Ok(Ok(first.clone()))
    }

    /// The day row `latest` of `input` began: a reading's instant, or the day a reading after
    /// m stands from (a window's last day); a state's first day, none for a span from the
    /// start of time (refcore's `carry_began`, `reading_on`). L §4.6 Readings, Past the last
    /// row.
    fn latest_began(
        &mut self,
        m: &Reality,
        input: &InputInfo,
        latest: &Latest,
        ctx: Ctx,
    ) -> Result<Option<Day>, Stop> {
        let row = latest_of(input, latest.at)?;
        match &row.instant {
            Some(RowInstant::At(at, _)) => Ok(Some(*at)),
            Some(RowInstant::After(_)) => Ok(match self.row_after_day(m, &input.id, row, ctx)? {
                AfterDay::Day(on) => Some(on),
                AfterDay::Window(w) => Some(w.hi),
                AfterDay::Hole(_) | AfterDay::Refused(_) | AfterDay::Unplaced => None,
            }),
            None => Ok(row_span(m, row, (ctx.cut, ctx.world))?.and_then(|span| span.start())),
        }
    }

    /// Where the rows `latest` of a class, which began on one day `began`, give `last`
    /// different values: their holes where any is one, else `HOLE-SAME` naming each with its
    /// value on that day; none where they agree (refcore's `same_value`). L §6.1, §8.1.
    fn carry_disagrees(
        &mut self,
        m: &Reality,
        latest: &[Latest],
        began: Day,
        ctx: Ctx,
    ) -> Result<Option<Hole>, Stop> {
        let mut got: Vec<(String, Value, String)> = Vec::with_capacity(latest.len());
        for l in latest {
            let input = m
                .inputs
                .get(&l.input)
                .ok_or_else(|| invariant("an input of last load didn't build"))?;
            let row = latest_of(input, l.at)?;
            let figure = origin(&l.input, &l.keys);
            let at = Ctx {
                law: Some(l.day),
                ..ctx
            };
            let value = self.row_converted(m, (input, row, &l.keys), &figure, at)?;
            got.push((figure, value, row.sid.clone()));
        }
        let [(_, first, _), rest @ ..] = got.as_slice() else {
            return Ok(None);
        };
        let mut agree = true;
        for (_, v, _) in rest {
            agree = agree && same_value(m, first, v)?;
        }
        if agree {
            return Ok(None);
        }
        let holes: Vec<Option<&Hole>> = got
            .iter()
            .map(|(_, v, _)| {
                if let Value::Hole(h) = v {
                    Some(h)
                } else {
                    None
                }
            })
            .collect();
        if let Some(hole) = spread(&holes) {
            return Ok(Some(hole));
        }
        let names: Vec<String> = got.iter().map(|(t, _, _)| t.clone()).collect();
        let mut parts = Vec::with_capacity(got.len());
        for (_, v, sid) in &got {
            parts.push(format!("{} ({sid})", v.text(m).map_err(num_stop)?));
        }
        let day = began.text();
        let text = format!(
            "{} are one, but the value differs on {day}: {}",
            and_list(&names),
            and_list(&parts)
        );
        Ok(Some(Hole::asked(&text, &names.join(", "))))
    }

    /// One side's row for `last` or `since` of `input` at `keys` on day `day`, recorded as
    /// read; or what it is instead: a known window's hole (`Signal`), or `none`, the gap
    /// rule's hole, `HOLE-NOREADING` (`Missing`) (refcore's `last_of`). L §4.6, §4.5.
    fn latest_side(
        &mut self,
        m: &Reality,
        (input, keys): (&InputInfo, Vec<KeyVal>),
        day: Day,
        past: Past,
        ctx: Ctx,
    ) -> Result<Side, Stop> {
        self.read(Read::Input {
            id: input.id.clone(),
            keys: keys.clone(),
        });
        // a row known in a window that holds the cut: the line's `windows each` decides it,
        // else the read is `HOLE-IMPRECISE` (L §4.5)
        let clocks = self.clocks(ctx, &input.id)?;
        let view = match self.input_view(m, input, &keys, (Some(day), clocks), ctx)? {
            Err(h) => return Ok(Side::Signal(Value::Hole(h))),
            Ok(view) => view,
        };
        let all = input;
        let input = match &view {
            Some(seen) => seen,
            None => input,
        };
        let found = match past {
            // `since` reads only the day the latest rows began, which they share: one reading
            // per unit at a place of unit `any` is no tie for it (L §4.6 Readings); a reading
            // `replace … = none` removed is none (L §5.19), so with only such it is
            // `HOLE-NOREADING`
            Past::Since => match last_began(m, input, &keys, day, clocks)? {
                Ok(kept) => {
                    let stand = standing_rows(m, &input.rows, &kept)?;
                    let read = stand.into_iter().find(
                        |at| matches!(input.rows.get(*at), Some(row) if row.value != Written::None),
                    );
                    let Some(at) = read else {
                        let figure = origin(&input.id, &keys);
                        return Ok(Side::Missing(Value::Hole(no_reading(&figure, day))));
                    };
                    Found::Row(at)
                }
                Err(found) => found,
            },
            Past::Last | Past::Price => read_last(m, input, &keys, day, clocks)?,
        };
        let at = match found {
            Found::Row(at) => at,
            Found::None => return Ok(Side::Missing(Value::None)),
            // a price with no row on or before d: no price, which `value` names (refcore's
            // `last_of`)
            Found::Hole(_) if past == Past::Price => return Ok(Side::Missing(Value::None)),
            Found::Hole(q) => {
                return Ok(Side::Missing(Value::Hole(Hole::asked(&q.text, &q.origin))));
            }
        };
        let row = latest_of(input, at)?;
        self.read(Read::Row {
            version: row.version.clone(),
            sid: row.sid.clone(),
        });
        // its place among all the input's rows, the ones the cut doesn't see included
        let among = all
            .rows
            .iter()
            .position(|r| r.sid == row.sid)
            .ok_or_else(|| invariant("a row of a read past its input's rows"))?;
        Ok(Side::Row(Latest {
            input: input.id.clone(),
            keys,
            at: among,
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
    let reading;
    let ty = match (&input.ty, input.reading, row.pattern.first()) {
        (Some(ty), ..) => ty,
        // a balance reading's value is a quantity in its place's unit, or the unit its row
        // states at a place of unit `any` (L §4.6 Readings, §5.19)
        (None, true, Some(Pat::Key(KeyVal::Thing(place)))) => {
            reading = m.reading_ty(place, &row.value).map_err(num_stop)?;
            match (&reading, &row.value) {
                (Some((_, ty)), _) => ty,
                // a row that states no unit, a hole or `none`, stands for every unit (pick
                // P22, #173)
                (None, Written::Hole(text)) => return Ok(Value::Hole(Hole::asked(text, figure))),
                (None, Written::None) => return Ok(Value::None),
                (None, _) => return Err(Stop::NotBuilt("the value of a reading of no unit")),
            }
        }
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
