//! An instance's body: its existence, legs, attributes, period and what it fulfils, read at
//! its law date and, under a window, at each law date the window can take.
//! Spec: L §4.3 (a window `at`: one body where every law date agrees, else `HOLE-IMPRECISE`),
//! §4.4, §5.14 (moves: `on`, `at e`, `when`, `for`, legs, attributes), §5.14.1, §5.14.3
//! (`fulfils`), §8.1; E §5 step 1 (the body at its booking cut).
//! Twin: engine/tools/refcore/moves.py (`eval_inst`, its `lds`; `body_over_law`, `inst_body`,
//! `body_key`).
//! Invariants: a law date its class holds gives what the class gave, so each class is read
//! once; two bodies differ only where their existence, a leg's name, places, amount or day,
//! an attribute, `for` or `fulfils` differ; a body's move holds the projections it meets.

use super::instance::{Amount, Dated, Exists, Inst, LegVal, join};
use super::plan::{refuse, refuse_load, shown};
use super::tuples::imprecise_law;
use crate::base::{Load, Run};
use crate::eval::{Ctx, Engine, LawClass};
use crate::hole::Hole;
use crate::model::{MoveInfo, Node, NodeKind, Reality, Value};
use crate::syntax::Stop;
use crate::time::Day;

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
}

impl Engine {
    /// An instance's day: its `on` read at its law date, and under a window `at` at each law
    /// date the window can take, `HOLE-IMPRECISE` where they differ (refcore's `eval_inst`,
    /// its `lds`). L §4.3, §5.14 `at e`.
    pub(super) fn day_over_law(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &Inst,
        tctx: Ctx,
    ) -> Result<Value, Stop> {
        let (Some(window), Some(day)) = (inst.law.window(), inst.law.first()) else {
            return self.ev(m, &mv.decl.on, &inst.env, tctx.at_law(inst.law.first()));
        };
        // once per law-date class: a day its class holds gives what it gave (L §4.4's cost)
        let (first, mut class) = self.on_at(m, mv, inst, (tctx, day))?;
        for at in inst.law.rest() {
            if class.holds(*at) {
                continue;
            }
            let (other, next) = self.on_at(m, mv, inst, (tctx, *at))?;
            if other != first {
                return Ok(Value::Hole(imprecise_law(window, &inst.id)));
            }
            class = next;
        }
        Ok(first)
    }

    /// An instance's `on` read at law date `day`, with the law dates that give the same
    /// (E §3.2). L §4.3, §5.14 `at e`.
    fn on_at(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &Inst,
        (tctx, day): (Ctx, Day),
    ) -> Result<(Value, LawClass), Stop> {
        let mark = self.open_law_class(day);
        let value = self.ev(m, &mv.decl.on, &inst.env, tctx.at_law(Some(day)));
        let class = self.close_law_class(mark, day);
        Ok((value?, class?))
    }

    /// An instance's body at its law date in context `ctx`, its cut set, from its tuple's
    /// existence; under a window `at`, at each law date the window can take: one body where
    /// all agree, else each leg's amount `HOLE-IMPRECISE`, and its existence where that
    /// differs too (refcore's `body_over_law`). L §4.3, §5.14 `at e`, E §5 step 1.
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
        let (Some(window), Some(day)) = (inst.law.window(), inst.law.first()) else {
            return self.inst_body(m, mv, inst, ctx.at_law(inst.law.first()), exists);
        };
        // once per law-date class: a day its class holds gives what it gave (L §4.4's cost)
        let (first, mut class) = self.body_at(m, mv, inst, (ctx, day))?;
        for at in inst.law.rest() {
            if class.holds(*at) {
                continue;
            }
            let (other, next) = self.body_at(m, mv, inst, (ctx, *at))?;
            if !same_body(&other, &first) {
                return Ok(imprecise_body(
                    first,
                    &other,
                    &imprecise_law(window, &inst.id),
                ));
            }
            class = next;
        }
        Ok(first)
    }

    /// An instance's body at law date `day`, with the law dates that give the same (E §3.2).
    /// L §4.3, §5.14 `at e`.
    fn body_at(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        inst: &Inst,
        (ctx, day): (Ctx, Day),
    ) -> Result<(Body, LawClass), Stop> {
        let mark = self.open_law_class(day);
        let body = self.inst_body(m, mv, inst, ctx.at_law(Some(day)), inst.exists.clone());
        let class = self.close_law_class(mark, day);
        Ok((body?, class?))
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
        for leg in &mv.decl.legs {
            let unit = kind_legs
                .iter()
                .find(|(name, _, _)| *name == leg.name)
                .map(|(_, unit, _)| unit.clone())
                .ok_or(Stop::NotBuilt(
                    "a leg its kind doesn't declare (M0's check)",
                ))?;
            legs.push(self.eval_leg(m, inst, leg, &unit, ctx)?);
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
        })
    }
}

/// The body of an instance at a booking cut that doesn't see its keys: it doesn't exist, and
/// moves, states and fulfils nothing (refcore's `(False, [], {}, None, None)`). E §5 steps
/// 1–3.
pub(super) fn no_body() -> Body {
    Body {
        exists: Exists::No,
        legs: Vec::new(),
        attrs: Vec::new(),
        for_period: None,
        fulfils: None,
    }
}

/// Whether two bodies are the same, read at two law dates or two cuts: their existence, each
/// leg's name, places, amount and day, their attributes, `for` and `fulfils` (refcore's
/// `body_key`, its legs). L §4.3, E §5 step 1.
pub(super) fn same_body(a: &Body, b: &Body) -> bool {
    a.exists == b.exists
        && same_legs(&a.legs, &b.legs)
        && a.attrs == b.attrs
        && a.for_period == b.for_period
        && a.fulfils == b.fulfils
}

/// Whether two bodies' legs are the same: each leg's name, places, amount and day (refcore's
/// `body_key`). L §4.3.
fn same_legs(a: &[LegVal], b: &[LegVal]) -> bool {
    a.len() == b.len()
        && a.iter().zip(b).all(|(x, y)| {
            x.name == y.name
                && x.from == y.from
                && x.to == y.to
                && x.amount == y.amount
                && x.day == y.day
        })
}

/// The body of a window `at` whose law dates give two bodies: the first's, each leg's amount
/// the hole `hole` (the second's legs where the first has none) unless only attributes, `for`
/// or `fulfils` differ, its existence the hole too where the two differ, and each attribute,
/// `for` and `fulfils` the two give differently the hole (refcore's `body_over_law`). L §4.3,
/// §8.1, E §5 step 1.
pub(super) fn imprecise_body(first: Body, other: &Body, hole: &Hole) -> Body {
    let legs = if first.exists == other.exists && same_legs(&first.legs, &other.legs) {
        first.legs
    } else {
        let src = if first.legs.is_empty() {
            &other.legs
        } else {
            &first.legs
        };
        src.iter()
            .map(|lg| LegVal {
                amount: Amount::Hole(hole.clone()),
                src: None,
                ..lg.clone()
            })
            .collect()
    };
    let exists = if first.exists == other.exists {
        first.exists
    } else {
        Exists::Unknown(hole.clone())
    };
    let mut attrs = first.attrs;
    for (name, value) in &mut attrs {
        let theirs = other.attrs.iter().find(|(n, _)| n == name).map(|(_, v)| v);
        if theirs != Some(&*value) {
            *value = Value::Hole(hole.clone());
        }
    }
    Body {
        exists,
        legs,
        attrs,
        for_period: imprecise_part(first.for_period, other.for_period.as_ref(), hole),
        fulfils: imprecise_part(first.fulfils, other.fulfils.as_ref(), hole),
    }
}

/// A body's `for` or `fulfils` read two ways: the first where both agree, else the hole
/// `hole` (L §4.3, §8.1).
fn imprecise_part(first: Option<Value>, other: Option<&Value>, hole: &Hole) -> Option<Value> {
    if first.as_ref() == other {
        first
    } else {
        Some(Value::Hole(hole.clone()))
    }
}
