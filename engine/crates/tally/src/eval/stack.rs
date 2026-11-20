//! The outermost reads: a read whose chain of def cells passes the counted depth is replayed once the work stack has computed the cells it waited on.
//! Spec: E §3.2 (evaluation is demand-driven; a recurrence needs no deep stack); L §5.11 (a
//! chain that returns to a cell still being evaluated is refused at that key, `RUN-CYCLE`);
//! E §4.1 step 2 (a decision dated by a window counts in `flows` when the window lies inside
//! the span: the line read again per day of it, L §4.4 l.668–671).
//! Twin: engine/tools/refcore/evaluate.py (`def_cell`); refcore recurses on a 512 MB thread
//! (cli.py `big_stack`), where the engine counts its depth and keeps a work stack (BUILDER §4);
//! ask.py (`one`, `window_days`).
//! Invariants: the depth is counted, never measured (R-16); a cell on the work stack is
//! computed with the cells that unwound above it still in progress, so a cycle through it
//! meets the `RUN-CYCLE` plain recursion meets; every task the replay finishes adds its cell
//! to the memo, so the replay ends; a read inside another read never replays, the outermost
//! does, and ends with no cell in progress; a line read per day of a window reads another
//! window per day under each day, to a counted depth, and no cell keeps the stop that
//! started it.

use super::aggregate::{Brings, Member};
use super::context::WINDOW_DAYS;
use super::keys::Given;
use super::memo::{CellId, CellKey};
use super::pins::Pinning;
use super::rules::key_env;
use super::state::{Collected, Ctx, Engine, Env, invariant};
use super::windows::{imprecise, window_days};
use crate::base::{Code, Refusal};
use crate::book::Ledger;
use crate::hole::{Hole, spread};
use crate::model::{DefInfo, KeyVal, Reality, Value, Window};
use crate::read::origin;
use crate::syntax::{Bind, Expr, Pin, Residuals, Stop};
use crate::time::Day;

/// One value of an ask's line: a value, or the run-time refusal it met, which the line prints
/// in its place (refcore's `one`). L §5.18.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    /// A value, a hole included.
    Value(Value),
    /// A refusal of L §8.4.
    Refused(Refusal),
}

/// What a read found when it began: the collectors open, and whether it is the outermost
/// read. A replay sets the collectors back before the read starts again.
struct Mark {
    /// The collectors open.
    collectors: usize,
    /// Begun with no def cell open, none in progress and no task waiting: it replays when it
    /// unwinds past the counted depth; an inner read unwinds on (R-16).
    outermost: bool,
    /// Under a trace, the node open and its children then: a replay starts them again
    /// (E §10).
    trace: Option<(usize, usize)>,
}

impl Engine {
    /// The value of expression `e` read from outside the evaluator, as an ask's line reads it:
    /// a chain of def cells past the counted depth is computed from the work stack, then the
    /// read starts again, its earlier cells now in the memo; a read that met a decision dated
    /// by a window, read again per day of it ([`Engine::line_per_day`]). E §3.2, §4.1 step 2.
    ///
    /// # Errors
    ///
    /// The refusals of L §8.4 the expression meets; `NotBuilt` for a node S doesn't build.
    pub fn value(&mut self, m: &Reality, e: &Expr, env: &Env, ctx: Ctx) -> Result<Value, Stop> {
        if self.window_reads != WindowReads::Off || !self.outermost() {
            return self.value_read(m, e, env, ctx);
        }
        self.window_reads = WindowReads::Line;
        let read = self.value_read(m, e, env, ctx);
        self.window_reads = WindowReads::Off;
        match read {
            Err(Stop::NotBuilt(why)) if why == WINDOW_DAYS => {
                self.line_per_day(m, e, (env, ctx), 1)
            }
            other => other,
        }
    }

    /// [`Engine::value`]'s read, once: the work stack's replay past the counted depth. E §3.2.
    fn value_read(&mut self, m: &Reality, e: &Expr, env: &Env, ctx: Ctx) -> Result<Value, Stop> {
        let mark = self.mark();
        loop {
            match self.ev(m, e, env, ctx) {
                Err(Stop::Deep) if mark.outermost => self.replay(m, &mark)?,
                result => {
                    self.read_ended(&mark);
                    return result;
                }
            }
        }
    }

    /// A read of def `def` at keys already made keys, from outside the evaluator: as
    /// [`Engine::value`], for one cell. L §5.11, E §3.2.
    ///
    /// # Errors
    ///
    /// The refusals of the def's evaluation.
    pub fn read_def(
        &mut self,
        m: &Reality,
        def: &DefInfo,
        keys: &[KeyVal],
        ctx: Ctx,
    ) -> Result<Value, Stop> {
        let mark = self.mark();
        loop {
            match self.read_def_at(m, def, keys, ctx) {
                Err(Stop::Deep) if mark.outermost => self.replay(m, &mark)?,
                result => {
                    self.read_ended(&mark);
                    return result;
                }
            }
        }
    }

    /// A key as written, read from outside the evaluator: as [`Engine::value`], for a key
    /// (L §2.5).
    ///
    /// # Errors
    ///
    /// The refusals the key's expression meets.
    pub(crate) fn key_given(
        &mut self,
        m: &Reality,
        e: &Expr,
        env: &Env,
        ctx: Ctx,
    ) -> Result<Given, Stop> {
        let mark = self.mark();
        loop {
            match self.given_key(m, e, env, ctx) {
                Err(Stop::Deep) if mark.outermost => self.replay(m, &mark)?,
                result => {
                    self.read_ended(&mark);
                    return result;
                }
            }
        }
    }

    /// The context inside an ask's `with` pins, read from outside the evaluator: as
    /// [`Engine::value`], for the pins' dates, keys and values; or the hole of a pin whose key
    /// or date is one (L §5.11, §5.18).
    ///
    /// # Errors
    ///
    /// The refusals the pins meet; `NotBuilt` for a pin form not built yet.
    pub fn with_pins(
        &mut self,
        m: &Reality,
        pins: &[Pin],
        ctx: Ctx,
    ) -> Result<Result<Ctx, Hole>, Stop> {
        let mark = self.mark();
        let env = self.ask_env.clone();
        loop {
            match self.pinned(m, pins, &env, ctx, Pinning::Ask) {
                Err(Stop::Deep) if mark.outermost => self.replay(m, &mark)?,
                result => {
                    self.read_ended(&mark);
                    return result;
                }
            }
        }
    }

    /// The binding tuples of an ask's `for`, or of a diff's own binding inside a tuple's scope
    /// `env`, read from outside the evaluator: each tuple's scope with the hole of a `where`
    /// that is one and the registers of the movements it binds, kept for its own line; or the
    /// hole of the first domain that is one (refcore's `members` with `regs`, as `_answer_in`
    /// reads it). L §5.18, §5.11, §8.2.
    ///
    /// # Errors
    ///
    /// The refusals the domains and `where`s meet.
    pub fn tuples(
        &mut self,
        m: &Reality,
        binds: &[Bind],
        (env, ctx): (&Env, Ctx),
    ) -> Result<Result<Vec<Member>, Hole>, Stop> {
        let mark = self.mark();
        loop {
            match self.members(m, binds, env, ctx, Brings::ToTuple) {
                Err(Stop::Deep) if mark.outermost => self.replay(m, &mark)?,
                result => {
                    self.read_ended(&mark);
                    return result;
                }
            }
        }
    }

    /// A `residuals` ask's ledger in scope `env`, read from outside the evaluator: as
    /// [`Engine::value`], for its place, its span and the ledger's lines (refcore's
    /// `answer_residuals`, its reads). E §6.3, §3.2.
    ///
    /// # Errors
    ///
    /// The refusals reading them meets; `NotBuilt` past what S builds.
    pub fn residuals(
        &mut self,
        m: &Reality,
        r: &Residuals,
        (env, ctx): (&Env, Ctx),
    ) -> Result<Ledger, Stop> {
        let mark = self.mark();
        loop {
            match self.ledger_of(m, r, (env, ctx)) {
                Err(Stop::Deep) if mark.outermost => self.replay(m, &mark)?,
                result => {
                    self.read_ended(&mark);
                    return result;
                }
            }
        }
    }

    /// An ask line's values in scope `env`, each read from outside the evaluator, with the
    /// register entries the line met; a run-time refusal stands as its value (refcore's
    /// `evaluate` and `one`). L §5.18, §8.2.
    ///
    /// # Errors
    ///
    /// A load refusal met at run, which refuses the run; `NotBuilt` for a node S doesn't build.
    pub fn line(
        &mut self,
        m: &Reality,
        exprs: &[Expr],
        env: &Env,
        ctx: Ctx,
    ) -> Result<(Vec<Item>, Collected), Stop> {
        self.push();
        let mut items = Vec::with_capacity(exprs.len());
        let mut stopped = None;
        for e in exprs {
            match self.value(m, e, env, ctx) {
                Ok(v) => items.push(Item::Value(v)),
                Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => {
                    items.push(Item::Refused(r));
                }
                Err(stop) => {
                    stopped = Some(stop);
                    break;
                }
            }
        }
        let met = self.pop_alone();
        match stopped {
            Some(stop) => Err(stop),
            None => Ok((items, met)),
        }
    }

    /// Ends a read: the outermost lets go of the layers its unwinding kept, which its replay
    /// and its read again needed (R-16, P §3.5).
    fn read_ended(&mut self, mark: &Mark) {
        if mark.outermost {
            self.release_kept();
        }
    }

    /// What the read finds as it begins.
    fn mark(&self) -> Mark {
        Mark {
            collectors: self.stack.len(),
            outermost: self.outermost(),
            trace: self.trace_mark(),
        }
    }

    /// Whether a read begun now is the outermost: no def cell open, none in progress and no
    /// task waiting (R-16).
    fn outermost(&self) -> bool {
        self.depth == 0 && self.active.is_empty() && self.work.is_empty()
    }

    /// Keeps cell `key`, which unwound past the counted depth, in progress under the task
    /// that its read waits on (R-16).
    pub(super) fn suspend(&mut self, key: CellId) -> Result<(), Stop> {
        let task = self
            .work
            .last_mut()
            .ok_or_else(|| invariant("a cell unwound with no task on the work stack"))?;
        task.unwound.push(key);
        Ok(())
    }

    /// The replay of an outermost read; one that fails ends the read, which lets go of the
    /// layers its unwinding kept (R-16, P §3.5).
    fn replay(&mut self, m: &Reality, mark: &Mark) -> Result<(), Stop> {
        // under a trace, the read's children start again, and what the work stack computes
        // hangs apart until the read references it (E §10)
        self.trace_apart();
        let replayed = self.replay_tasks(m, mark);
        self.trace_join();
        self.trace_restart(mark.trace);
        if replayed.is_err() {
            self.read_ended(mark);
        }
        replayed
    }

    /// Computes the cells on the work stack, the latest first, each from depth 0 while the
    /// cells that unwound above it stay in progress; a task that unwinds again waits for the
    /// one it pushed. A task done, the cells that waited on it leave progress, to be read
    /// again by the read below it (R-16, BUILDER §4).
    fn replay_tasks(&mut self, m: &Reality, mark: &Mark) -> Result<(), Stop> {
        if self.work.is_empty() {
            return Err(invariant("a read unwound with no cell on the work stack"));
        }
        while let Some(task) = self.work.last() {
            let (id, keys, ctx) = (task.def.clone(), task.keys.clone(), task.ctx);
            let def = m
                .defs
                .get(&id)
                .ok_or_else(|| invariant("a task of a def load didn't build"))?;
            let env = key_env(def, &keys);
            let waiting = self.work.len();
            self.depth = 0;
            self.push();
            // the task's own timeline, whatever scope it unwound through (E §3.3)
            self.replay_timeline(m, ctx)?;
            let result = self.def_cell(m, def, &keys, &env, ctx);
            self.leave_timeline()?;
            self.stack.truncate(mark.collectors);
            if result == Err(Stop::Deep) {
                if self.work.len() <= waiting {
                    return Err(invariant("a task unwound with no cell on the work stack"));
                }
                continue;
            }
            let kept = CellKey::new(&id, &keys, ctx);
            if self.cells.get(&kept, ctx.law).is_none() {
                return Err(invariant("a task finished with no cell in the memo"));
            }
            let done = self
                .work
                .pop()
                .ok_or_else(|| invariant("a task left the work stack twice"))?;
            for key in &done.unwound {
                self.active.remove(key);
            }
        }
        if !self.active.is_empty() {
            return Err(invariant("a cell left in progress after the replay"));
        }
        Ok(())
    }
}

/// Where a read that meets a decision dated by a window stands (E §4.1 step 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WindowReads {
    /// Outside an ask line's own read: the read gives the window's hole.
    Off,
    /// An ask line's outermost read: it stops (`WINDOW_DAYS`), and the line is read per day.
    Line,
    /// The line read on one of a window's days, `n` windows deep: another window met is read
    /// per day of its own under that day, up to [`WINDOW_NEST`] windows, beyond them its hole.
    PerDay(u32),
}

/// The windows a line is read per day of, one inside another, before the next one is its
/// hole: the counted depth of the nested reads (R-16; P41, F7).
const WINDOW_NEST: u32 = 8;

/// A decision's window an ask line's read met, with what met it: `flows of <move>` or `a
/// binding over <move>` (refcore's `WinDay`). E §4.1 step 2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WindowDay {
    /// The window, as a read of the decision's day gives it.
    window: Box<Window>,
    /// What met it, which its hole names.
    what: String,
}

impl Engine {
    /// A decision's window `w`, which no walk books on one day, met by `what`, a `flows`, a
    /// binding over a span it may reach or a field read: at an ask line's own read, a `check`
    /// side or a def the line reaches, outside the walks and the work stack's replay, the
    /// stop that reads the line per day, the window kept, a window met under another's day
    /// included; anywhere else (inside a walk, a `for` domain, an `explain`) its hole
    /// (E §4.1 step 2; refcore's `WinDay`; P41, F5, F7).
    ///
    /// # Errors
    ///
    /// `WINDOW_DAYS`, the line's read to start again per day.
    pub(crate) fn window_met(
        &mut self,
        m: &Reality,
        w: &Window,
        what: String,
    ) -> Result<Hole, Stop> {
        let per_day = match self.window_reads {
            WindowReads::Line => true,
            WindowReads::PerDay(n) => n < WINDOW_NEST,
            WindowReads::Off => false,
        };
        if per_day && self.walking.is_empty() && self.work.is_empty() {
            self.window_day = Some(WindowDay {
                window: Box::new(w.clone()),
                what,
            });
            return Err(Stop::NotBuilt(WINDOW_DAYS));
        }
        imprecise(m, w, &what)
    }

    /// Line value `e`, whose read met a decision dated by a window: read again once per day of
    /// the window, the cell its day is read from pinned to that day and listed nowhere, `depth`
    /// windows deep. One value where every day gives it; the refusal where every day refuses;
    /// the days' holes where one gives a hole, a window met under a day naming itself where
    /// its own days differ; else the window's hole, as for a window no such cell names
    /// (E §4.1 step 2: it counts in `flows(x, span)` when the window lies inside the span;
    /// L §4.4 l.668–671; tally 2.2.1 N3, P39, P41; refcore's `window_days`).
    fn line_per_day(
        &mut self,
        m: &Reality,
        e: &Expr,
        (env, ctx): (&Env, Ctx),
        depth: u32,
    ) -> Result<Value, Stop> {
        let met = self
            .window_day
            .take()
            .ok_or_else(|| invariant("a line read per day with no window met"))?;
        let Some(target) = self.pin_target(&met.window, ctx)? else {
            return Ok(Value::Hole(imprecise(m, &met.window, &met.what)?));
        };
        let days = window_days(&met.window)?;
        let before = self.window_reads;
        self.window_reads = WindowReads::PerDay(depth);
        let outs = self.each_day(m, (e, env, ctx), (&target.0, &target.1), (&days, depth));
        self.window_reads = before;
        let outs = outs?;
        let refused: Vec<&Refusal> = outs.iter().filter_map(|o| o.as_ref().err()).collect();
        if let Some(first) = refused.first() {
            if refused.len() == outs.len() {
                return Err(Stop::Refused((*first).clone()));
            }
            return Ok(Value::Hole(imprecise(m, &met.window, &met.what)?));
        }
        let values: Vec<&Value> = outs.iter().filter_map(|o| o.as_ref().ok()).collect();
        let holes: Vec<Option<&Hole>> = values.iter().map(|v| v.as_hole()).collect();
        if let Some(hole) = spread(&holes) {
            return Ok(Value::Hole(hole));
        }
        match values.split_first() {
            Some((first, rest)) if rest.iter().all(|v| v == first) => Ok((*first).clone()),
            Some(_) | None => Ok(Value::Hole(imprecise(m, &met.window, &met.what)?)),
        }
    }

    /// Line value `e` read on each of `days`, `item` at `keys` pinned to the day by a pin no
    /// register lists, in a layer of its own freed after the read, a window `depth` deep; a
    /// window the read meets under the day read per day of its own inside that layer (refcore's
    /// `window_days`, its `PinVal('solve', …)` and `one`); a run-time refusal stands as that
    /// day's value. L §5.18; P41 (F7).
    fn each_day(
        &mut self,
        m: &Reality,
        (e, env, ctx): (&Expr, &Env, Ctx),
        (item, keys): (&str, &[KeyVal]),
        (days, depth): (&[Day], u32),
    ) -> Result<Vec<Result<Value, Refusal>>, Stop> {
        let mut outs = Vec::with_capacity(days.len());
        let inner = depth
            .checked_add(1)
            .ok_or_else(|| invariant("a window depth past the integers"))?;
        for day in days {
            let step = self.solve_step((item, keys), Value::Day(*day), ctx)?;
            self.open_layer(m, step)?;
            let mut read = self.value_read(m, e, env, step);
            if matches!(&read, Err(Stop::NotBuilt(why)) if *why == WINDOW_DAYS) {
                read = self.line_per_day(m, e, (env, step), inner);
            }
            self.end_layer(step.layer, read.as_ref().err())?;
            match read {
                Ok(v) => outs.push(Ok(v)),
                Err(Stop::Refused(r)) if !matches!(r.code(), Code::Load(_)) => outs.push(Err(r)),
                Err(stop) => return Err(stop),
            }
        }
        Ok(outs)
    }

    /// The cell window `w` is read from, which a pin can set day by day: a row read of an item
    /// at no keys, or a pin of a cell; none for a window no one such cell names, computed or
    /// written (refcore's `window_days`, its `('row', …)` and `('pin', …)` names). L §4.4.
    fn pin_target(&self, w: &Window, ctx: Ctx) -> Result<Option<(String, Vec<KeyVal>)>, Stop> {
        let [named] = w.carry.as_slice() else {
            return Ok(None);
        };
        if named.id.starts_with("row ") {
            return Ok(w
                .read
                .as_ref()
                .filter(|r| r.keys.is_empty() && r.item == named.label)
                .map(|r| (r.item.clone(), Vec::new())));
        }
        if !named.id.starts_with("pin ") {
            return Ok(None);
        }
        let data = self.layer_data(ctx.layer)?;
        for p in data.pins.iter().filter(|p| !p.solve && p.fields.is_empty()) {
            let Some(keys) = p.keys.iter().cloned().collect::<Option<Vec<KeyVal>>>() else {
                continue;
            };
            if origin(&p.item, &keys) == named.label {
                return Ok(Some((p.item.clone(), keys)));
            }
        }
        Ok(None)
    }
}
