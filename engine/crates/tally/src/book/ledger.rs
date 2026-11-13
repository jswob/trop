//! The residual ledger: a place over a span, the model at the ask's cut against the observations, line by line, grouped by month and move.
//! Spec: E §6.3 (the lines: matched, expected but missing, unexplained, partly fulfilled,
//! balance; grouping by month and move id, the `?`, `unexplained` and `balance` groups; zero
//! lines; signs; totals; order), §6.2 (the model: each instance as booked or modelled at its
//! booking cut, the settlements dated in the span, each decision as the walk at K ran it),
//! §6.1 (a reading's gap: the reading less the booked balance at its instant); L §5.19 (a day
//! no statement covers holds no line, but an instance an observation fulfils; observations
//! stand in for their instance), §5.18 (`residuals x span`), §8.1 (holes sort first, by their
//! first question).
//! Twin: engine/tools/refcore/resid.py (`m_residuals`, `m_flow_into`, `m_touches`,
//! `m_resid_held`, `sk`, `m_rtext`, `_add`, `_sub`).
//! Invariants: every amount is signed as a flow into the place; a hole on either side makes a
//! difference a hole, never 0; the ledger books nothing: it reads the plan pass's instances,
//! the main walk at the cut and the observations; groups and lines are ordered by explicit
//! keys (size, day, group, movement order), ties in the order they were made.

use core::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use super::adjust::Adjustment;
use super::observe::{Obs, inst_id};
use super::reading::{Seen, booked_amount, reading_rank, readings};
use crate::base::{Load, Refusal};
use crate::eval::{Ctx, Engine, Env, Rows, num_stop};
use crate::hole::{Hole, spread};
use crate::model::{Fulfils, GapRole, Reality, Span};
use crate::num::Num;
use crate::syntax::{Certainty, Residuals, Stop};
use crate::time::{Day, Period};
use crate::walk::{
    Amount, Dated, Exists, Inst, LegVal, Status, Term, invariant, kind_certainty, qty_value,
    reading_point,
};

/// A figure of the ledger: a quantity of the place's unit, or a hole (refcore's `Fraction` or
/// `Hole`). E §6.3 Signs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fig {
    /// A quantity, signed as a flow into the place.
    Qty(Num),
    /// A hole.
    Hole(Hole),
}

impl Fig {
    /// The hole, when the figure is one. L §8.1.
    fn hole(&self) -> Option<&Hole> {
        match self {
            Fig::Qty(_) => None,
            Fig::Hole(h) => Some(h),
        }
    }

    /// Whether the figure is the quantity zero. E §6.3 Zero lines.
    fn is_zero(&self) -> bool {
        match self {
            Fig::Qty(q) => q.is_zero(),
            Fig::Hole(_) => false,
        }
    }
}

/// A group's key: its month, none for the `?` group, and its name (`move rent_pay`,
/// `unexplained`, `balance`, `?`). E §6.3 Grouping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupKey {
    /// The month; none for the group of instances whose day is a hole.
    pub month: Option<Period>,
    /// The name.
    pub name: String,
}

/// One line of the ledger (refcore's `RLine`). E §6.3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    /// Its day; none for a line of the `?` group.
    day: Option<Day>,
    /// Its group.
    group: GroupKey,
    /// The model's amount; none for an unexplained line.
    model: Option<Fig>,
    /// The observed amount; none for an expected-but-missing line.
    observed: Option<Fig>,
    /// The difference.
    diff: Fig,
    /// Its text.
    pub text: String,
    /// Its movement's order, typed: the last tie's key (E §6.3 Order, tally 2.1.2).
    order: Term,
}

/// A group of the ledger: its lines and its totals. E §6.3 Totals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    /// Its key.
    pub key: GroupKey,
    /// The model's total (the booked balances, for a balance group).
    pub model: Fig,
    /// The observations' total (the stated balances, for a balance group).
    pub observed: Fig,
    /// The difference, observed less model.
    pub diff: Fig,
    /// Its lines, in order.
    pub lines: Vec<Line>,
}

/// What a `residuals` ask reads: the hole its place or span is, or the ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ledger {
    /// The place or the span is a hole.
    Hole(Hole),
    /// The ledger of a place.
    Groups {
        /// The place.
        place: String,
        /// Its unit.
        unit: String,
        /// The groups, in order.
        groups: Vec<Group>,
    },
}

/// The observations that fulfil each instance, by instance id; those that fulfil each
/// adjustment, by (instance id, leg, version) (refcore's `by_target`, `by_adj`). E §6.3.
#[derive(Default)]
struct Fulfilling {
    /// By instance id.
    by_target: BTreeMap<String, Vec<Obs>>,
    /// By the adjustment's instance id, leg and version.
    by_adj: BTreeMap<(String, String, String), Vec<Obs>>,
}

impl Fulfilling {
    /// The observations that fulfil the instance `id`, none when no observation names it.
    /// E §6.3.
    fn of(&self, id: &str) -> &[Obs] {
        match self.by_target.get(id) {
            Some(os) => os,
            None => &[],
        }
    }
}

/// The lines made so far, the place and unit they are of, and the observations they hold.
struct Lines {
    /// The place.
    place: String,
    /// Its unit.
    unit: String,
    /// The lines, in the order made.
    out: Vec<Line>,
    /// The observations a line holds, by id.
    seen: BTreeSet<String>,
}

impl Engine {
    /// A `residuals` ask's ledger: its place and span read, then the ledger of that place
    /// over that span at the context's cut (refcore's `answer_residuals`, its reads). E §6.3,
    /// L §5.18.
    ///
    /// # Errors
    ///
    /// The refusals reading the place, the span or the ledger meets; `NotBuilt` for `by f`
    /// and for a place of unit `any`.
    pub(crate) fn ledger_of(
        &mut self,
        m: &Reality,
        r: &Residuals,
        ctx: Ctx,
    ) -> Result<Ledger, Stop> {
        if r.by.is_some() {
            return Err(Stop::NotBuilt(
                "residuals by a field (the ledger in full, M2)",
            ));
        }
        let env = Env::empty();
        let place = match self.place_arg(m, &r.place, &env, ctx)? {
            Err(h) => return Ok(Ledger::Hole(h)),
            Ok(p) => p,
        };
        let span = match self.span_value(m, &r.span, &env, ctx)? {
            Err(h) => return Ok(Ledger::Hole(h)),
            Ok(s) => s,
        };
        let unit = m
            .things
            .get(&place)
            .and_then(|t| m.place_unit(t))
            .ok_or_else(|| invariant("a place with no unit"))?;
        if unit == "any" {
            return Err(Stop::NotBuilt("the ledgers of a place of unit any"));
        }
        let mut lines = Lines {
            place: place.clone(),
            unit: unit.clone(),
            out: Vec::new(),
            seen: BTreeSet::new(),
        };
        self.ledger(m, &mut lines, &span, ctx.cut)?;
        let groups = grouped(lines.out)?;
        Ok(Ledger::Groups {
            place,
            unit,
            groups,
        })
    }

    /// The ledger's lines: the model at cut `cut` against the observations as known on the
    /// model's `on` date, in the order refcore makes them (its `m_residuals`). E §6.3.
    fn ledger(
        &mut self,
        m: &Reality,
        lines: &mut Lines,
        span: &Span,
        cut: Day,
    ) -> Result<(), Stop> {
        let obs = self.ledger_obs(m, &lines.place, span)?;
        let (model, undated) = self.ledger_model(m, &lines.place, span, cut, &obs)?;
        let fulfilling = fulfilling(&obs);
        for inst in &model {
            instance_line(m, lines, inst, &fulfilling)?;
        }
        if !fulfilling.by_adj.is_empty() {
            let adjustments = self.all_adjustments(m, cut)?;
            for (key, os) in &fulfilling.by_adj {
                adjustment_line(m, lines, &adjustments, key, os)?;
            }
        }
        for inst in &undated {
            undated_line(m, lines, inst, &fulfilling)?;
        }
        for o in &obs {
            if lines.seen.contains(&o.id) {
                continue;
            }
            if let Some(f) = &o.fulfils
                && self.resid_held(m, f, cut)?
            {
                continue;
            }
            unexplained_line(m, lines, o)?;
        }
        self.balance_lines(m, lines, span, cut)?;
        after_run_line(m, lines, span)
    }

    /// The observations as known on the model's `on` date that touch the place, dated in the
    /// span (refcore's `obs`). E §6.3.
    fn ledger_obs(&mut self, m: &Reality, place: &str, span: &Span) -> Result<Vec<Obs>, Stop> {
        let all = self.observations(m, m.on)?;
        Ok(all
            .iter()
            .filter(|o| touches(&o.legs, place) && span.contains(o.day))
            .cloned()
            .collect())
    }

    /// The model: each plan instance at cut `cut` that touches the place, dated in the span
    /// on a day a statement covers it or fulfilled by an observation there, and each decision
    /// as the main walk at the cut ran it on a covered day, in the movement order; and apart,
    /// the instances whose day is a hole (refcore's `model`, `undated`). E §6.3, §6.2.
    fn ledger_model(
        &mut self,
        m: &Reality,
        place: &str,
        span: &Span,
        cut: Day,
        obs: &[Obs],
    ) -> Result<(Vec<Inst>, Vec<Inst>), Stop> {
        let fulfilled: BTreeSet<String> = obs.iter().filter_map(Obs::target_id).collect();
        let mut model = Vec::new();
        let mut undated = Vec::new();
        // the plan moves the timeline's world holds (L §6.4)
        let mut moves = m.moves_in(self.timeline_world());
        moves.retain(|mv| !mv.decision);
        let mut insts = Vec::new();
        for mv in moves {
            insts.extend(self.move_instances(m, mv, cut)?.iter().cloned());
        }
        for inst in insts {
            let day = match model_day(&inst)? {
                Modelled::Skip => continue,
                Modelled::Undated => None,
                Modelled::On(day) => Some(day),
            };
            let refused = inst.status == Status::Refused;
            let touching = if refused {
                may_touch(m, &inst, place)
            } else {
                touches(&inst.legs, place)
            };
            if !touching {
                continue;
            }
            if let Some(day) = day {
                let shown =
                    !self.covering(m, place, day, m.on).is_empty() || fulfilled.contains(&inst.id);
                if !(span.contains(day) && shown) {
                    continue;
                }
            }
            if refused {
                return Err(refusal_of(&inst));
            }
            match day {
                None => undated.push(inst),
                Some(_) => model.push(inst),
            }
        }
        model.extend(self.ledger_runs(m, place, span, cut)?);
        model.sort_by(|a, b| a.order.cmp(&b.order));
        Ok((model, undated))
    }

    /// The decisions as the main walk at cut `cut` ran them, by instance id, each dated in
    /// the span on a day a statement covers the place and touching it (refcore's `w.decs`).
    /// E §6.3, §6.2.
    fn ledger_runs(
        &mut self,
        m: &Reality,
        place: &str,
        span: &Span,
        cut: Day,
    ) -> Result<Vec<Inst>, Stop> {
        let id = self.main_walk(m, cut)?;
        let walk = self.walk(id)?;
        let runs: Vec<Inst> = walk
            .run_ids
            .values()
            .filter_map(|ix| walk.runs.get(*ix))
            .map(|run| run.inst.clone())
            .collect();
        let mut out = Vec::new();
        for inst in runs {
            let Some(Dated::Day(day)) = inst.day else {
                continue;
            };
            if !span.contains(day) || self.covering(m, place, day, m.on).is_empty() {
                continue;
            }
            if let Some(err) = &inst.err {
                if may_touch(m, &inst, place) {
                    return Err(Stop::Refused(err.clone()));
                }
                continue;
            }
            if touches(&inst.legs, place) {
                out.push(inst);
            }
        }
        Ok(out)
    }

    /// Whether the model at cut `cut` holds what an observation's `fulfils` names: an
    /// instance that exists there, or an adjustment some version made by then; such an
    /// observation outside the ledger's lines is that instance's, not unexplained (refcore's
    /// `m_resid_held`). E §6.3 unexplained.
    fn resid_held(&mut self, m: &Reality, f: &Fulfils, cut: Day) -> Result<bool, Stop> {
        match f {
            Fulfils::Instance(move_id, keys) => {
                let Some(mv) = m.moves.get(move_id) else {
                    return Ok(false);
                };
                let insts = match self.move_instances(m, mv, cut) {
                    Ok(insts) => insts,
                    Err(Stop::Refused(_)) => return Ok(true),
                    Err(stop) => return Err(stop),
                };
                let id = inst_id(move_id, keys);
                Ok(insts
                    .iter()
                    .find(|i| i.id == id)
                    .is_some_and(|i| i.exists != Exists::No))
            }
            Fulfils::Adjustment(move_id, keys, leg, vid) => {
                let id = inst_id(move_id, keys);
                let adjustments = self.all_adjustments(m, cut)?;
                Ok(adjustments
                    .iter()
                    .any(|a| a.inst_id == id && a.leg.name == *leg && a.vids.contains(vid)))
            }
        }
    }

    /// A balance line for every `stated` reading of the place in the span, as known on the
    /// model's `on` date, at its instant: booked is the main walk's balance at cut `cut` just
    /// before it, the gap stated less booked, marked `anchored` or `padded` by its version
    /// (refcore's balance lines). E §6.3 balance, §6.1.
    fn balance_lines(
        &mut self,
        m: &Reality,
        lines: &mut Lines,
        span: &Span,
        cut: Day,
    ) -> Result<(), Stop> {
        let id = self.main_walk(m, cut)?;
        let world = self.timeline_world();
        let walked = readings(m, (cut, world), Rows::Unkept)?;
        for seen in readings(m, (m.on, world), Rows::Unkept)? {
            if seen.input != "stated" || seen.place != lines.place || !span.contains(seen.day) {
                continue;
            }
            let Some(reading) = &seen.reading else {
                continue;
            };
            let at = reading_point(seen.day, seen.edge, reading_rank(&walked, &seen)?);
            let walk = self.walk(id)?;
            let st = walk.pstate_before(m, &reading.place, &reading.unit, at);
            let booked = match booked_amount(&st, &walk.anchor_question(&reading.place)) {
                Amount::Qty(q) => Fig::Qty(q),
                Amount::Hole(h) => Fig::Hole(h),
                Amount::Refused(r) => return Err(Stop::Refused(r)),
            };
            balance_line(m, lines, &seen, &reading.version, booked)?;
        }
        Ok(())
    }
}

/// Where an instance of the model stands: left out of the ledger, a day that is a hole, or a
/// day (refcore's statuses `none` and `leftout`, a refused one raised, `isinstance(d, Hole)`).
/// E §6.3.
enum Modelled {
    /// It doesn't exist, or is dated outside the run.
    Skip,
    /// Its day is a hole.
    Undated,
    /// Its day.
    On(Day),
}

/// Whether a refused instance refuses the ledger of `place`: exactly where a hole one would be
/// a line of it, a place its legs give or any place where a leg's place couldn't be read; with
/// no legs, wherever its move has some (refcore's `may_touch`; tally 2.1.7 R1, R2, SR-1).
/// E §6.3, L §8.4.
fn may_touch(m: &Reality, inst: &Inst, place: &str) -> bool {
    if inst.legs.is_empty() {
        return m
            .moves
            .get(&inst.move_id)
            .is_some_and(|mv| !mv.decl.legs.is_empty());
    }
    touches(&inst.legs, place)
        || inst
            .legs
            .iter()
            .any(|lg| lg.to.place().is_none() || lg.from.place().is_none())
}

/// A refused instance's refusal. L §8.4.
fn refusal_of(inst: &Inst) -> Stop {
    match &inst.err {
        Some(err) => Stop::Refused(err.clone()),
        None => invariant("a refused instance with no refusal"),
    }
}

/// Where an instance of the model stands (refcore's `m_residuals`, its status test): one
/// that doesn't exist is no line; a refused one stands where its day is, and refuses the
/// ledger where it may touch the place. E §6.3, L §8.4.
fn model_day(inst: &Inst) -> Result<Modelled, Stop> {
    match inst.status {
        Status::None | Status::LeftOut => return Ok(Modelled::Skip),
        Status::Refused
        | Status::Open
        | Status::Hole
        | Status::Decision
        | Status::Uncertain
        | Status::Modelled
        | Status::Past
        | Status::Plan => {}
    }
    match &inst.day {
        Some(Dated::Day(d)) => Ok(Modelled::On(*d)),
        Some(Dated::Hole(_)) => Ok(Modelled::Undated),
        None => Err(invariant("an instance with no day, not refused")),
    }
}

/// The observations that fulfil each instance and each adjustment, in the observations'
/// order (refcore's `by_target`, `by_adj`). E §6.3.
fn fulfilling(obs: &[Obs]) -> Fulfilling {
    let mut out = Fulfilling::default();
    for o in obs {
        match &o.fulfils {
            None => {}
            Some(Fulfils::Instance(move_id, keys)) => out
                .by_target
                .entry(inst_id(move_id, keys))
                .or_insert_with(Vec::new)
                .push(o.clone()),
            Some(Fulfils::Adjustment(move_id, keys, leg, vid)) => out
                .by_adj
                .entry((inst_id(move_id, keys), leg.clone(), vid.clone()))
                .or_insert_with(Vec::new)
                .push(o.clone()),
        }
    }
    out
}

/// An instance's line: expected but missing when no observation fulfils it (none for an
/// amount of 0), partly fulfilled when its kind is uncertain and its observations sum to
/// less, else matched (refcore's model lines). E §6.3.
fn instance_line(
    m: &Reality,
    lines: &mut Lines,
    inst: &Inst,
    fulfilling: &Fulfilling,
) -> Result<(), Stop> {
    let day = match &inst.day {
        Some(Dated::Day(d)) => *d,
        Some(Dated::Hole(_)) | None => return Err(invariant("a ledger instance with no day")),
    };
    let mdl = flow_into(&inst.legs, &lines.place, &lines.unit)?;
    let os = fulfilling.of(&inst.id);
    lines.seen.extend(os.iter().map(|o| o.id.clone()));
    let group = month_group(day, format!("move {}", inst.move_id))?;
    if os.is_empty() {
        if mdl.is_zero() {
            return Ok(());
        }
        let diff = fig_sub(Some(&Fig::Qty(Num::zero())), Some(&mdl))?;
        let text = format!(
            "expected but missing {} on {}: model {}",
            inst.id,
            day.text(),
            fig_text(m, &mdl, &lines.unit)?
        );
        lines.out.push(Line {
            day: Some(day),
            group,
            model: Some(mdl),
            observed: None,
            diff,
            text,
            order: inst.order.term()?,
        });
        return Ok(());
    }
    let ob = observed_sum(os, &lines.place, &lines.unit)?;
    let diff = fig_sub(Some(&ob), Some(&mdl))?;
    let uncertain = m
        .moves
        .get(&inst.move_id)
        .and_then(|mv| kind_certainty(m, &mv.kind))
        == Some(Certainty::Uncertain);
    let mut partly = false;
    if let (Fig::Qty(o), Fig::Qty(d), true) = (&ob, &mdl, uncertain) {
        partly = abs(o)?.compare(&abs(d)?).map_err(num_stop)? == Ordering::Less;
    }
    let text = if partly {
        partly_text(m, lines, inst, day, (&mdl, &ob, &diff))?
    } else {
        matched_text(m, &lines.unit, &inst.id, day, (&mdl, &ob, &diff), os)?
    };
    lines.out.push(Line {
        day: Some(day),
        group,
        model: Some(mdl),
        observed: Some(ob),
        diff,
        text,
        order: inst.order.term()?,
    });
    Ok(())
}

/// A partly fulfilled line's text: the model, the observed, the difference, the remainder and
/// its question (refcore's partly fulfilled `RLine`). E §6.3, N-1206.
fn partly_text(
    m: &Reality,
    lines: &Lines,
    inst: &Inst,
    day: Day,
    (mdl, ob, diff): (&Fig, &Fig, &Fig),
) -> Result<String, Stop> {
    let rem = fig_sub(Some(mdl), Some(ob))?;
    let unit = &lines.unit;
    Ok(format!(
        "partly fulfilled {} on {}: model {}, observed {}, difference {}, remainder {}: when \
         does the rest arrive, or is it written off?",
        inst.id,
        day.text(),
        fig_text(m, mdl, unit)?,
        fig_text(m, ob, unit)?,
        fig_text(m, diff, unit)?,
        fig_text(m, &rem, unit)?
    ))
}

/// A matched line's text: the model, the observed, the difference and the timing, each
/// observation's day less `day` (refcore's matched `RLine`). E §6.3.
fn matched_text(
    m: &Reality,
    unit: &str,
    id: &str,
    day: Day,
    (mdl, ob, diff): (&Fig, &Fig, &Fig),
    os: &[Obs],
) -> Result<String, Stop> {
    let mut timing: BTreeSet<String> = BTreeSet::new();
    for o in os {
        let n = day
            .days_to(o.day)
            .ok_or_else(|| invariant("a timing past the calendar"))?;
        timing.insert(format!("{n:+} days"));
    }
    let timing: Vec<String> = timing.into_iter().collect();
    Ok(format!(
        "matched {id} on {}: model {}, observed {}, difference {}, timing {}",
        day.text(),
        fig_text(m, mdl, unit)?,
        fig_text(m, ob, unit)?,
        fig_text(m, diff, unit)?,
        timing.join(", ")
    ))
}

/// An adjustment's matched line, against its change δ on its day, signed by the way its leg
/// runs at the place; none when the model at the cut holds no such adjustment (refcore's
/// `by_adj` lines, the last adjustment of a name kept). E §6.3 matched.
fn adjustment_line(
    m: &Reality,
    lines: &mut Lines,
    adjustments: &[Adjustment],
    (iid, leg, vid): &(String, String, String),
    os: &[Obs],
) -> Result<(), Stop> {
    let Some(a) = adjustments
        .iter()
        .rev()
        .find(|a| a.inst_id == *iid && a.leg.name == *leg && a.vids.contains(vid))
    else {
        return Ok(());
    };
    lines.seen.extend(os.iter().map(|o| o.id.clone()));
    let mdl = match &a.delta {
        Amount::Refused(r) => return Err(Stop::Refused(r.clone())),
        Amount::Hole(h) => Fig::Hole(h.clone()),
        Amount::Qty(q) => Fig::Qty(signed(q, way_at(&a.leg, &lines.place))?),
    };
    let ob = observed_sum(os, &lines.place, &lines.unit)?;
    let diff = fig_sub(Some(&ob), Some(&mdl))?;
    let text = matched_text(m, &lines.unit, &a.id, a.day, (&mdl, &ob, &diff), os)?;
    lines.out.push(Line {
        day: Some(a.day),
        group: month_group(a.day, format!("move {}", a.move_id))?,
        model: Some(mdl),
        observed: Some(ob),
        diff,
        text,
        order: a.order(),
    });
    Ok(())
}

/// An instance whose day is a hole: matched on the day of the last observation that fulfils
/// it; with none, expected but missing in the `?` group with its day's questions (none for an
/// amount of 0) (refcore's `undated` lines). E §6.3 Grouping.
fn undated_line(
    m: &Reality,
    lines: &mut Lines,
    inst: &Inst,
    fulfilling: &Fulfilling,
) -> Result<(), Stop> {
    let mdl = flow_into(&inst.legs, &lines.place, &lines.unit)?;
    let os = fulfilling.of(&inst.id);
    if let Some(day) = os.iter().map(|o| o.day).max() {
        lines.seen.extend(os.iter().map(|o| o.id.clone()));
        let ob = observed_sum(os, &lines.place, &lines.unit)?;
        let diff = fig_sub(Some(&ob), Some(&mdl))?;
        let text = matched_text(m, &lines.unit, &inst.id, day, (&mdl, &ob, &diff), os)?;
        lines.out.push(Line {
            day: Some(day),
            group: month_group(day, format!("move {}", inst.move_id))?,
            model: Some(mdl),
            observed: Some(ob),
            diff,
            text,
            order: inst.order.term()?,
        });
        return Ok(());
    }
    if mdl.is_zero() {
        return Ok(());
    }
    let Some(Dated::Hole(day_hole)) = &inst.day else {
        return Err(invariant("an undated instance whose day isn't a hole"));
    };
    let diff = fig_sub(Some(&Fig::Qty(Num::zero())), Some(&mdl))?;
    let text = format!(
        "expected but missing {} on {}: model {}",
        inst.id,
        day_hole.text(),
        fig_text(m, &mdl, &lines.unit)?
    );
    lines.out.push(Line {
        day: None,
        group: GroupKey {
            month: None,
            name: "?".to_owned(),
        },
        model: Some(mdl),
        observed: None,
        diff,
        text,
        order: inst.order.term()?,
    });
    Ok(())
}

/// A period that E cuts holds its days through E; the days after E are one line, the hole
/// `after the run's end`, since the walk holds none of them (refcore's `m_residuals`).
/// E §6.3, L §8.1 `HOLE-POSTRUN`.
fn after_run_line(m: &Reality, lines: &mut Lines, span: &Span) -> Result<(), Stop> {
    let (Some(first), Some(end)) = (span.start(), span.end()) else {
        return Ok(());
    };
    let last = end
        .previous()
        .ok_or_else(|| invariant("a span ending before the calendar"))?;
    if last <= m.end {
        return Ok(());
    }
    lines.out.retain(|l| match l.day {
        Some(d) => d <= m.end,
        None => true,
    });
    let after = m
        .end
        .next()
        .ok_or_else(|| invariant("a run ending at the calendar's end"))?
        .max(first);
    let hole = Fig::Hole(Hole::after_run(&lines.place));
    let text = format!(
        "days {} .. {}: {}",
        after.text(),
        last.text(),
        fig_text(m, &hole, &lines.unit)?
    );
    lines.out.push(Line {
        day: Some(after),
        group: GroupKey {
            month: None,
            name: "after the run's end".to_owned(),
        },
        model: Some(hole.clone()),
        observed: Some(hole.clone()),
        diff: hole,
        text,
        order: Term::Tuple(Vec::new()),
    });
    Ok(())
}

/// An unexplained line: an observation that fulfils nothing the model at the cut holds, its
/// amount the difference (refcore's unexplained `RLine`). E §6.3 unexplained.
fn unexplained_line(m: &Reality, lines: &mut Lines, o: &Obs) -> Result<(), Stop> {
    let ob = flow_into(&o.legs, &lines.place, &lines.unit)?;
    let text = format!(
        "unexplained {} on {}: observed {}",
        o.id,
        o.day.text(),
        fig_text(m, &ob, &lines.unit)?
    );
    lines.out.push(Line {
        day: Some(o.day),
        group: month_group(o.day, "unexplained".to_owned())?,
        model: None,
        observed: Some(ob.clone()),
        diff: ob,
        text,
        order: o.order(),
    });
    Ok(())
}

/// A reading's balance line: booked, stated and the gap, stated less booked, marked by its
/// version's gap role (refcore's balance `RLine`). E §6.3 balance.
fn balance_line(
    m: &Reality,
    lines: &mut Lines,
    seen: &Seen,
    version: &str,
    booked: Fig,
) -> Result<(), Stop> {
    let Some(reading) = &seen.reading else {
        return Ok(());
    };
    let stated = match &reading.value {
        Ok(q) => Fig::Qty(q.clone()),
        Err(h) => Fig::Hole(h.clone()),
    };
    let gap = fig_sub(Some(&stated), Some(&booked))?;
    let roles = match m.versions.get(version) {
        Some(v) => v.gap_roles.as_slice(),
        None => &[],
    };
    let anchored = roles
        .iter()
        .any(|(p, role)| *p == lines.place && *role == GapRole::Anchor);
    let mark = if anchored { "anchored" } else { "padded" };
    let unit = &lines.unit;
    let text = format!(
        "balance {} {} ({mark}): booked {}, stated {}, gap {}",
        lines.place,
        reading.instant,
        fig_text(m, &booked, unit)?,
        fig_text(m, &stated, unit)?,
        fig_text(m, &gap, unit)?
    );
    lines.out.push(Line {
        day: Some(seen.day),
        group: month_group(seen.day, "balance".to_owned())?,
        model: Some(booked),
        observed: Some(stated),
        diff: gap,
        text,
        // a balance line's order is `(day, 9)`, after a day's movements (refcore's `RLine`)
        order: Term::Tuple(vec![Term::Day(seen.day), Term::Int(9)]),
    });
    Ok(())
}

/// The group of a line on `day` named `name`: its month's. E §6.3 Grouping.
fn month_group(day: Day, name: String) -> Result<GroupKey, Stop> {
    let month = Period::month_of(day).ok_or_else(|| invariant("a month past the calendar"))?;
    Ok(GroupKey {
        month: Some(month),
        name,
    })
}

/// The way a leg runs at a place: into it, out of it, or neither (both ends there, or none)
/// (refcore's `s`). E §6.3 Signs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Way {
    /// Into the place.
    In,
    /// Out of it.
    Out,
    /// Neither.
    Neither,
}

/// The way leg `lg` runs at `place`. E §6.3 Signs.
fn way_at(lg: &LegVal, place: &str) -> Way {
    let to = lg.to.place() == Some(place);
    let from = lg.from.place() == Some(place);
    match (to, from) {
        (true, false) => Way::In,
        (false, true) => Way::Out,
        (true, true) | (false, false) => Way::Neither,
    }
}

/// A quantity signed as a flow into the place: itself in, negated out, 0 neither. E §6.3
/// Signs.
fn signed(q: &Num, way: Way) -> Result<Num, Stop> {
    match way {
        Way::In => Ok(q.clone()),
        Way::Out => q.checked_neg().map_err(num_stop),
        Way::Neither => Ok(Num::zero()),
    }
}

/// Whether a movement's legs touch a place (refcore's `m_touches`). E §6.3.
fn touches(legs: &[LegVal], place: &str) -> bool {
    legs.iter()
        .any(|lg| lg.to.place() == Some(place) || lg.from.place() == Some(place))
}

/// A movement's flow into the place, signed: in +, out −; the holes of the amounts that
/// touch it; a leg's refusal refuses it (refcore's `m_flow_into`). E §6.3 Signs.
fn flow_into(legs: &[LegVal], place: &str, unit: &str) -> Result<Fig, Stop> {
    let mut total = Num::zero();
    let mut holes: Vec<&Hole> = Vec::new();
    for lg in legs {
        let way = way_at(lg, place);
        if way == Way::Neither {
            continue;
        }
        let q = match &lg.amount {
            Amount::Refused(r) => return Err(Stop::Refused(r.clone())),
            Amount::Hole(h) => {
                holes.push(h);
                continue;
            }
            Amount::Qty(q) => q,
        };
        if let Some(u) = &lg.unit
            && u != unit
        {
            return Err(Stop::Refused(Refusal::new(
                Load::Unit.at_run(),
                format!("a leg in {u} touches {place}, a place in {unit}"),
            )));
        }
        total = total.checked_add(&signed(q, way)?).map_err(num_stop)?;
    }
    let operands: Vec<Option<&Hole>> = holes.into_iter().map(Some).collect();
    Ok(spread(&operands).map_or(Fig::Qty(total), Fig::Hole))
}

/// The observations' flow into the place, summed (refcore's `ob`, by `_add`). E §6.3.
fn observed_sum(os: &[Obs], place: &str, unit: &str) -> Result<Fig, Stop> {
    let mut sum: Option<Fig> = None;
    for o in os {
        sum = Some(fig_add(sum, &flow_into(&o.legs, place, unit)?)?);
    }
    sum.ok_or_else(|| invariant("a sum of no observations"))
}

/// `a + b`, where `a` may be no figure yet: holes spread (refcore's `_add`). E §6.3 Totals.
fn fig_add(a: Option<Fig>, b: &Fig) -> Result<Fig, Stop> {
    let Some(a) = a else {
        return Ok(b.clone());
    };
    if let (Fig::Qty(x), Fig::Qty(y)) = (&a, b) {
        return Ok(Fig::Qty(x.checked_add(y).map_err(num_stop)?));
    }
    spread(&[a.hole(), b.hole()])
        .map(Fig::Hole)
        .ok_or_else(|| invariant("a sum of holes with no hole"))
}

/// `a − b`, where no figure is 0: a hole on either side makes the difference a hole
/// (refcore's `_sub`). E §6.3 Signs.
fn fig_sub(a: Option<&Fig>, b: Option<&Fig>) -> Result<Fig, Stop> {
    let holes = [a.and_then(Fig::hole), b.and_then(Fig::hole)];
    if let Some(h) = spread(&holes) {
        return Ok(Fig::Hole(h));
    }
    let x = qty_or_zero(a);
    let y = qty_or_zero(b);
    Ok(Fig::Qty(x.checked_sub(&y).map_err(num_stop)?))
}

/// A figure's quantity, where no figure is 0 (refcore's `(a or 0)`); a hole is never one
/// here, `fig_sub` having taken it. E §6.3.
fn qty_or_zero(f: Option<&Fig>) -> Num {
    match f {
        Some(Fig::Qty(q)) => q.clone(),
        Some(Fig::Hole(_)) | None => Num::zero(),
    }
}

/// `|q|`. E §6.3 Order.
fn abs(q: &Num) -> Result<Num, Stop> {
    if q.is_negative() {
        return q.checked_neg().map_err(num_stop);
    }
    Ok(q.clone())
}

/// A figure as a line prints it: the quantity in the place's unit, or the hole (refcore's
/// `m_rtext`). E §6.3, L §8.1.
///
/// # Errors
///
/// A number error printing it.
pub fn fig_text(m: &Reality, fig: &Fig, unit: &str) -> Result<String, Stop> {
    match fig {
        Fig::Hole(h) => Ok(h.text()),
        Fig::Qty(q) => qty_value(m, q, unit)?.text(m).map_err(num_stop),
    }
}

/// What a line or a group sorts by (refcore's sort keys). E §6.3 Order.
struct SortKey {
    /// A group's: whether it has a month, so the `?` group comes first; false for a line.
    dated: bool,
    /// The difference, by size: holes first, then the biggest.
    size: Fig,
    /// A line's day, a group's month's first day; none sorts first.
    day: Option<Day>,
    /// The group's name.
    name: String,
    /// A line's movement order; the empty tuple for a group, whose order no tie reaches.
    order: Term,
}

/// Two figures by size: holes first, among themselves by their first question, then by
/// absolute value, the biggest first (refcore's `sk`). E §6.3 Order, L §8.1.
fn size_cmp(a: &Fig, b: &Fig) -> Result<Ordering, Stop> {
    match (a, b) {
        (Fig::Hole(x), Fig::Hole(y)) => Ok(first_question(x).cmp(&first_question(y))),
        (Fig::Hole(_), Fig::Qty(_)) => Ok(Ordering::Less),
        (Fig::Qty(_), Fig::Hole(_)) => Ok(Ordering::Greater),
        (Fig::Qty(x), Fig::Qty(y)) => abs(y)?.compare(&abs(x)?).map_err(num_stop),
    }
}

/// A hole's first question, by code point; the empty text for one with none. L §8.1.
fn first_question(h: &Hole) -> String {
    match h.question_texts().into_iter().next() {
        Some(q) => q,
        None => String::new(),
    }
}

/// Two sort keys: a group's month first, then the size, the day, the group's name and the
/// movement order (refcore's `sort` keys). E §6.3 Order.
fn key_cmp(a: &SortKey, b: &SortKey) -> Result<Ordering, Stop> {
    let first = a.dated.cmp(&b.dated).then(size_cmp(&a.size, &b.size)?);
    let rest = a.day.cmp(&b.day).then_with(|| a.name.cmp(&b.name));
    Ok(first.then(rest).then_with(|| a.order.cmp(&b.order)))
}

/// The positions of `keys` in sorted order, ties in their order: a merge sort from runs of
/// one, since comparing two sizes can refuse at the digits' cap (refcore's stable `sort`).
/// E §6.3 Order.
fn sorted_positions(keys: &[SortKey]) -> Result<Vec<usize>, Stop> {
    let mut runs: Vec<Vec<usize>> = (0..keys.len()).map(|i| vec![i]).collect();
    while runs.len() > 1 {
        let mut next = Vec::new();
        let mut pairs = runs.into_iter();
        while let Some(left) = pairs.next() {
            match pairs.next() {
                Some(right) => next.push(merged(keys, left, right)?),
                None => next.push(left),
            }
        }
        runs = next;
    }
    match runs.into_iter().next() {
        Some(order) => Ok(order),
        None => Ok(Vec::new()),
    }
}

/// Two sorted runs merged, the left one's first on a tie, so the sort is stable. E §6.3
/// Order.
fn merged(keys: &[SortKey], left: Vec<usize>, right: Vec<usize>) -> Result<Vec<usize>, Stop> {
    let mut out = Vec::new();
    let mut lefts = left.into_iter().peekable();
    let mut rights = right.into_iter().peekable();
    loop {
        let take_right = match (lefts.peek(), rights.peek()) {
            (Some(a), Some(b)) => {
                key_cmp(key_at(keys, *a)?, key_at(keys, *b)?)? == Ordering::Greater
            }
            (Some(_), None) => false,
            (None, Some(_)) => true,
            (None, None) => return Ok(out),
        };
        out.extend(if take_right {
            rights.next()
        } else {
            lefts.next()
        });
    }
}

/// The sort key at position `i`. E §6.3 Order.
fn key_at(keys: &[SortKey], i: usize) -> Result<&SortKey, Stop> {
    keys.get(i).ok_or_else(|| invariant("a ledger's sort"))
}

/// The lines in groups, each group's lines and totals, the groups in order (refcore's
/// `groups`, `out`). E §6.3 Grouping, Totals, Order.
fn grouped(lines: Vec<Line>) -> Result<Vec<Group>, Stop> {
    let mut made: Vec<(GroupKey, Vec<Line>)> = Vec::new();
    for line in lines {
        match made.iter_mut().find(|(k, _)| *k == line.group) {
            Some((_, ls)) => ls.push(line),
            None => made.push((line.group.clone(), vec![line])),
        }
    }
    let mut groups = Vec::with_capacity(made.len());
    for (key, ls) in made {
        groups.push(group_of(key, ls)?);
    }
    let keys: Vec<SortKey> = groups
        .iter()
        .map(|g| SortKey {
            dated: g.key.month.is_some(),
            size: g.diff.clone(),
            day: g.key.month.map(Period::start),
            name: g.key.name.clone(),
            order: Term::Tuple(Vec::new()),
        })
        .collect();
    let order = sorted_positions(&keys)?;
    let mut slots: Vec<Option<Group>> = groups.into_iter().map(Some).collect();
    let mut sorted = Vec::with_capacity(order.len());
    for i in &order {
        let group = slots.get_mut(*i).and_then(Option::take);
        sorted.push(group.ok_or_else(|| invariant("a ledger's group sorted twice"))?);
    }
    Ok(sorted)
}

/// One group: its totals, the model's and the observations' (none counting 0), the
/// difference, its lines in order. E §6.3 Totals, Order.
fn group_of(key: GroupKey, lines: Vec<Line>) -> Result<Group, Stop> {
    let mut model: Option<Fig> = None;
    let mut observed: Option<Fig> = None;
    for line in &lines {
        if let Some(f) = &line.model {
            model = Some(fig_add(model, f)?);
        }
        if let Some(f) = &line.observed {
            observed = Some(fig_add(observed, f)?);
        }
    }
    let model = model.unwrap_or_else(|| Fig::Qty(Num::zero()));
    let observed = observed.unwrap_or_else(|| Fig::Qty(Num::zero()));
    let diff = fig_sub(Some(&observed), Some(&model))?;
    let keys: Vec<SortKey> = lines
        .iter()
        .map(|l| SortKey {
            dated: false,
            size: l.diff.clone(),
            day: l.day,
            name: l.group.name.clone(),
            order: l.order.clone(),
        })
        .collect();
    let order = sorted_positions(&keys)?;
    let mut slots: Vec<Option<Line>> = lines.into_iter().map(Some).collect();
    let mut sorted = Vec::with_capacity(order.len());
    for i in &order {
        let line = slots.get_mut(*i).and_then(Option::take);
        sorted.push(line.ok_or_else(|| invariant("a ledger's line sorted twice"))?);
    }
    Ok(Group {
        key,
        model,
        observed,
        diff,
        lines: sorted,
    })
}
