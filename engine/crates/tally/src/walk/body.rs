//! An instance's body: its existence, legs, attributes, period and what it fulfils, read at
//! its law date.
//! Spec: L §4.3, §5.14 (moves: `on`, `at e`, `when`, `for`, legs, attributes), §5.14.1,
//! §5.14.3 (`fulfils`), §8.1; E §5 step 1 (the body at its booking cut).
//! Twin: engine/tools/refcore/moves.py (`eval_inst`, `inst_body`).
//! Invariants: a body's move holds the projections it meets.

use super::instance::{Dated, Exists, Inst, LegVal, join};
use super::plan::{refuse, refuse_load, shown};
use super::tuples::Law;
use crate::base::{Load, Run};
use crate::eval::{Collected, Ctx, Engine};
use crate::hole::Hole;
use crate::model::{MoveInfo, Node, NodeKind, Reality, Value};
use crate::syntax::Stop;
use std::collections::BTreeMap;

/// What an instance's body gives: whether it exists, its legs, attributes and period.
pub(super) struct Body {
    /// Whether it exists.
    pub(super) exists: Exists,
    /// Its legs.
    pub(super) legs: Vec<LegVal>,
    /// Its attributes.
    pub(super) attrs: Vec<(String, Value)>,
    /// The period it is for.
    pub(super) for_period: Option<Value>,
    /// What its `fulfils` names.
    pub(super) fulfils: Option<Value>,
    /// What reading each leg met, by the leg's name: a read of one leg of a movement stated
    /// `same` holds the other side's reads of that leg only (S3, P43 on #241).
    pub(super) leg_reads: BTreeMap<String, Collected>,
}

impl Engine {
    /// An instance's law date by its move's `at`, kept on it, and its day, its `on` read at
    /// that date; a law date that is a hole is its day's hole (refcore's `eval_inst`). L §4.3,
    /// §5.14 `at e`.
    ///
    /// # Errors
    ///
    /// The refusals of the `at` and the `on`.
    pub(super) fn law_and_day(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &mut Inst,
        tctx: Ctx,
    ) -> Result<Result<Value, Hole>, Stop> {
        inst.law = self.move_law(m, mv, &inst.env, tctx)?;
        if let Law::Hole(h) = &inst.law {
            return Ok(Err(h.clone()));
        }
        let day = self.ev(m, &mv.decl.on, &inst.env, tctx.at_law(inst.law.first()))?;
        Ok(Ok(day))
    }

    /// An instance's body at its law date in context `ctx`, its cut set, from its tuple's
    /// existence (refcore's `body_over_law`). L §4.3, §5.14 `at e`, E §5 step 1.
    ///
    /// # Errors
    ///
    /// The refusals of its parts.
    pub(super) fn body_over_law(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &Inst,
        ctx: Ctx,
    ) -> Result<Body, Stop> {
        let exists = inst.exists.clone();
        self.inst_body(m, mv, inst, ctx.at_law(inst.law.first()), exists)
    }

    /// An instance's body in context `ctx`, its law date and cut set: its `when`, period,
    /// legs and attributes, with its move the holder of the projections met (refcore's
    /// `body_over_law`, `inst_body`). L §5.14, §5.14.1, E §5 step 1.
    ///
    /// # Errors
    ///
    /// `RUN-NONE` for a `when` that is none; the refusals of its parts.
    pub(super) fn inst_body(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &Inst,
        ctx: Ctx,
        exists: Exists,
    ) -> Result<Body, Stop> {
        let outer = self.holding(Node::new(NodeKind::Move, &mv.id));
        let body = self.move_body(m, mv, inst, ctx, exists);
        self.holder = outer;
        body
    }

    /// An instance's body: its `when`, period, legs and attributes (refcore's
    /// `body_over_law`, `inst_body`). L §5.14, E §5 step 1.
    fn move_body(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &Inst,
        ctx: Ctx,
        exists: Exists,
    ) -> Result<Body, Stop> {
        let mut ex = exists;
        if mv.decl.fulfils.is_some() {
            crate::book::settlement_move(m, mv)?;
        }
        if let Some(when) = &mv.decl.when {
            let w = self.ev(m, when, &inst.env, ctx)?;
            if w == Value::None {
                return Err(refuse(
                    Run::None,
                    format!("the when of {} is none here", inst.id),
                ));
            }
            if w == Value::Bool(false) {
                let Some(Dated::Hole(h)) = &inst.day else {
                    return Ok(Body {
                        exists: Exists::No,
                        legs: Vec::new(),
                        attrs: Vec::new(),
                        for_period: None,
                        fulfils: None,
                        leg_reads: BTreeMap::new(),
                    });
                };
                ex = join(ex, h);
            } else if let Value::Hole(h) = &w {
                ex = join(ex, h);
            } else if w != Value::Bool(true) {
                return Err(refuse_load(
                    Load::Unit,
                    format!("the when of {} is {}", inst.id, shown(m, &w)?),
                ));
            }
        }
        let for_period = match &mv.decl.for_period {
            Some(e) => Some(self.ev(m, e, &inst.env, ctx)?),
            None => None,
        };
        let kind_legs = m.kind_legs(&mv.kind);
        let mut legs = Vec::with_capacity(mv.decl.legs.len());
        let mut leg_reads = BTreeMap::new();
        for leg in &mv.decl.legs {
            let unit = kind_legs
                .iter()
                .find(|(name, _, _)| *name == leg.name)
                .map(|(_, unit, _)| unit.clone())
                .ok_or(Stop::NotBuilt(
                    "a leg its kind doesn't declare (M0's check)",
                ))?;
            self.push();
            let read = self.eval_leg(m, inst, leg, &unit, ctx);
            let reads = self.pop_alone();
            self.absorb(&reads);
            legs.push(read?);
            leg_reads.insert(leg.name.clone(), reads);
        }
        let mut attrs = Vec::with_capacity(mv.decl.attrs.len());
        for (name, e) in &mv.decl.attrs {
            attrs.push((name.clone(), self.ev(m, e, &inst.env, ctx)?));
        }
        let fulfils = match &mv.decl.fulfils {
            Some(e) => Some(self.ev(m, e, &inst.env, ctx)?),
            None => None,
        };
        Ok(Body {
            exists: ex,
            legs,
            attrs,
            for_period,
            fulfils,
            leg_reads,
        })
    }
}
