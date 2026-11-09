//! A scenario's expression row at run: its expression evaluated at the row's keys and law date, the base's items it names by prefix read in the base.
//! Spec: L §6.4 (a scenario row may be an expression over the base's facts: the base's items
//! named with the base's id as a prefix, read in the base, the scenario's own bare; it reads
//! inputs, bands and defs, which load checks), §2.3 Calls (`household.b(x)`, the base's band,
//! called bare), §4.6 (the row the read rule takes; its value read for the read's law date),
//! §8.4 (`RUN-CYCLE`).
//! Twin: engine/tools/refcore/evaluate.py (`row_value`'s `sexpr` arm, `_sexpr_active`;
//! `read`'s `_world` arm) and check2.py (`m_check_scen_rows`, the rewrite of `base.x` and
//! `base.b(x)`).
//! Invariants: a prefixed name's keys are made where the row is read and the item is read in
//! the world under the row's scenario; a prefixed band call is the bare call, where the row
//! is read; the base and the rows being evaluated are set for the row's evaluation alone and
//! set back after it, a refusal or an unwind included; a row met again at the same figure and
//! law date while it is evaluated reads itself, `RUN-CYCLE`.

use super::context::world_named;
use super::expr::is_read;
use super::state::{Ctx, Engine, Env, invariant, refuse};
use crate::base::Run;
use crate::model::{Reality, Row, Value};
use crate::syntax::{Expr, Postfix, Stop};

impl Engine {
    /// The value of scenario row `row`, whose value is expression `e`, read as `figure` in
    /// `at`, at the row's law date: no variable in scope, the base's items named by its id read
    /// in the base (refcore's `row_value`, its `sexpr` arm). L §6.4, §4.6.
    ///
    /// # Errors
    ///
    /// `RUN-CYCLE` for a row that reads itself; the refusals its expression meets.
    pub(super) fn row_expr(
        &mut self,
        m: &Reality,
        (row, e): (&Row, &Expr),
        figure: &str,
        at: Ctx,
    ) -> Result<Value, Stop> {
        let scenario = m
            .scenarios
            .of(&row.version)
            .ok_or_else(|| invariant("an expression row outside a scenario"))?;
        let base = m
            .scenarios
            .over
            .get(scenario)
            .and_then(Option::as_deref)
            .unwrap_or(&m.header.id)
            .to_owned();
        let world = world_named(m, &base)?;
        let active = (row.sid.clone(), figure.to_owned(), at.law);
        if !self.rows_active.insert(active.clone()) {
            return Err(refuse(
                Run::Cycle,
                format!("{figure} reads itself through its scenario row {}", row.sid),
            ));
        }
        let outer = self.row_base.replace((base, world));
        let value = self.ev(m, e, &Env::empty(), at);
        self.row_base = outer;
        self.rows_active.remove(&active);
        value
    }

    /// Whether `prefix` is the base's id of the expression row being evaluated (refcore's
    /// `m_check_scen_rows`, its `base.x`). L §6.4.
    pub(super) fn reads_base(&self, prefix: &str) -> bool {
        self.row_base
            .as_ref()
            .is_some_and(|(base, _)| base == prefix)
    }

    /// `<base>.name` and the postfixes after it in an expression row: the item read in the
    /// base, at keys made where the row is read; the postfixes left (refcore's `read`, its
    /// `_world` arm). L §6.4.
    ///
    /// # Errors
    ///
    /// The refusals of the read.
    pub(super) fn base_read<'p>(
        &mut self,
        m: &Reality,
        name: &str,
        rest: &'p [Postfix],
        env: &Env,
        ctx: Ctx,
    ) -> Result<(Value, &'p [Postfix]), Stop> {
        let (_, world) = self
            .row_base
            .clone()
            .ok_or_else(|| invariant("a base read outside an expression row"))?;
        let read = ctx.in_world(world);
        match rest {
            [Postfix::Keys(keys), after @ ..] if is_read(m, name) => {
                let value = self.read_item_in(m, (name, keys), env, (ctx, read))?;
                Ok((value, after))
            }
            _ => Ok((self.name(m, name, env, read)?, rest)),
        }
    }
}
