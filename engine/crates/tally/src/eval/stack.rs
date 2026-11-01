//! The outermost reads: a read whose chain of def cells passes the counted depth is replayed once the work stack has computed the cells it waited on.
//! Spec: E §3.2 (evaluation is demand-driven; a recurrence needs no deep stack); L §5.11 (a
//! chain that returns to a cell still being evaluated is refused at that key, `RUN-CYCLE`).
//! Twin: engine/tools/refcore/evaluate.py (`def_cell`); refcore recurses on a 512 MB thread
//! (cli.py `big_stack`), where the engine counts its depth and keeps a work stack (BUILDER §4).
//! Invariants: the depth is counted, never measured (R-16); a cell on the work stack is
//! computed with the cells that unwound above it still in progress, so a cycle through it
//! meets the `RUN-CYCLE` plain recursion meets; every task the replay finishes adds its cell
//! to the memo, so the replay ends; a read inside another read never replays, the outermost
//! does, and ends with no cell in progress.

use super::aggregate::Member;
use super::rules::{cell_key, key_env};
use super::state::{Collected, Ctx, Engine, Env, invariant};
use crate::base::{Code, Refusal};
use crate::hole::Hole;
use crate::model::{DefInfo, KeyVal, Reality, Value};
use crate::read::origin;
use crate::syntax::{Bind, Expr, Stop};

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
}

impl Engine {
    /// The value of expression `e` read from outside the evaluator, as an ask's line reads it:
    /// a chain of def cells past the counted depth is computed from the work stack, then the
    /// read starts again, its earlier cells now in the memo. E §3.2.
    ///
    /// # Errors
    ///
    /// The refusals of L §8.4 the expression meets; `NotBuilt` for a node S doesn't build.
    pub fn value(&mut self, m: &Reality, e: &Expr, env: &Env, ctx: Ctx) -> Result<Value, Stop> {
        let mark = self.mark();
        loop {
            match self.ev(m, e, env, ctx) {
                Err(Stop::Deep) if mark.outermost => self.replay(m, &mark)?,
                result => return result,
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
                result => return result,
            }
        }
    }

    /// The binding tuples of an ask's `for`, read from outside the evaluator: each tuple's
    /// scope with the hole of a `where` that is one, or the hole of the first domain that is
    /// one (refcore's `members` as `_answer_in` reads it). L §5.18, §5.11.
    ///
    /// # Errors
    ///
    /// The refusals the domains and `where`s meet.
    pub fn tuples(
        &mut self,
        m: &Reality,
        binds: &[Bind],
        ctx: Ctx,
    ) -> Result<Result<Vec<Member>, Hole>, Stop> {
        let mark = self.mark();
        loop {
            match self.members(m, binds, &Env::empty(), ctx) {
                Err(Stop::Deep) if mark.outermost => self.replay(m, &mark)?,
                result => return result,
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

    /// What the read finds as it begins.
    fn mark(&self) -> Mark {
        Mark {
            collectors: self.stack.len(),
            outermost: self.depth == 0 && self.active.is_empty() && self.work.is_empty(),
        }
    }

    /// Keeps cell `key`, which unwound past the counted depth, in progress under the task
    /// that its read waits on (R-16).
    pub(super) fn suspend(&mut self, key: String) -> Result<(), Stop> {
        let task = self
            .work
            .last_mut()
            .ok_or_else(|| invariant("a cell unwound with no task on the work stack"))?;
        task.unwound.push(key);
        Ok(())
    }

    /// Computes the cells on the work stack, the latest first, each from depth 0 while the
    /// cells that unwound above it stay in progress; a task that unwinds again waits for the
    /// one it pushed. A task done, the cells that waited on it leave progress, to be read
    /// again by the read below it (R-16, BUILDER §4).
    fn replay(&mut self, m: &Reality, mark: &Mark) -> Result<(), Stop> {
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
            let result = self.def_cell(m, def, &keys, &env, ctx);
            self.stack.truncate(mark.collectors);
            if result == Err(Stop::Deep) {
                if self.work.len() <= waiting {
                    return Err(invariant("a task unwound with no cell on the work stack"));
                }
                continue;
            }
            if !self.memo.contains_key(&cell_key(&origin(&id, &keys), ctx)) {
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
