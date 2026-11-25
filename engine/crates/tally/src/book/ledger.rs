//! The residual ledger: a place over a span, the model at the ask's cut against the observations, line by line, each line in its group (`ledger_order.rs` totals and orders them).
//! Spec: E §6.3 (the lines: matched, expected but missing, unexplained, partly fulfilled,
//! balance; each line's group: by month and move id, or with `by f` by the observations'
//! field, the `?`, `unexplained` and `balance` groups; a place of unit `any` gives one ledger
//! per unit; zero lines; signs), §6.2 (the model: each instance as booked or modelled at its
//! booking cut, the settlements dated in the span, each decision as the walk at K ran it),
//! §6.1 (a reading's gap: the reading less the booked balance at its instant); L §5.19 (a day
//! no statement covers holds no line, but an instance an observation fulfils; observations
//! stand in for their instance), §5.18 (`residuals x span [by f]`).
//! Twin: engine/tools/refcore/resid.py (`m_residuals`, `m_residual_units`, `m_flow_into`,
//! `m_touches`, `m_resid_held`, `obs_field`, `m_rtext`) and ask.py (`answer_residuals`).
//! Invariants: every amount is signed as a flow into the place; a hole on either side makes a
//! difference a hole, never 0; the ledger books nothing: it reads the plan pass's instances,
//! the main walk at the cut and the observations.

use core::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use super::adjust::Adjustment;
use super::ledger_order::{Group, GroupKey, abs, fig_add, fig_sub, grouped, month_group};
use super::members::cell_of;
use super::observe::{Obs, inst_id};
use super::reading::{Seen, booked_amount, reading_rank, readings};
use crate::base::{Load, Refusal};
use crate::eval::{Ctx, Engine, Env, Rows, num_stop};
use crate::hole::{Hole, Unknown, carry_union, spread};
use crate::model::{Fulfils, GapRole, Reality, Span};
use crate::num::Num;
use crate::syntax::{Certainty, Residuals, Stop};
use crate::time::Day;
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
    /// The unknown days the figure carries, a hole's, added to `into` in order. L §4.4, §8.2.
    pub fn hole_days_into(&self, into: &mut Vec<Unknown>) {
        if let Fig::Hole(h) = self {
            carry_union(into, &h.carried());
        }
    }

    /// The hole, when the figure is one. L §8.1.
    pub(super) fn hole(&self) -> Option<&Hole> {
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

/// One line of the ledger (refcore's `RLine`). E §6.3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    /// Its day; none for a line of the `?` group.
    pub(super) day: Option<Day>,
    /// Its group.
    pub(super) group: GroupKey,
    /// The model's amount; none for an unexplained line.
    pub(super) model: Option<Fig>,
    /// The observed amount; none for an expected-but-missing line.
    pub(super) observed: Option<Fig>,
    /// The difference.
    pub(super) diff: Fig,
    /// Its text.
    pub text: String,
    /// Its movement's order, typed: the last tie's key (E §6.3 Order, tally 2.1.2).
    pub(super) order: Term,
    /// The unknown days the day it prints carries where that day is a hole: an expected
    /// instance's (L §4.4; refcore's `carry_day`, which `hole_days` reads only as a hole).
    pub(super) carry: Vec<Unknown>,
}

impl Line {
    /// The unknown days its holes carry, its figures' and its day's, added to `into` in order
    /// (refcore's `answer_residuals`, its `vals`, and `hole_days`). L §4.4, §8.2.
    pub fn hole_days_into(&self, into: &mut Vec<Unknown>) {
        for fig in [
            self.model.as_ref(),
            self.observed.as_ref(),
            Some(&self.diff),
        ]
        .into_iter()
        .flatten()
        {
            fig.hole_days_into(into);
        }
        carry_union(into, &self.carry);
    }
}

/// What a `residuals` ask reads: the hole its place or span is, or the ledgers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ledger {
    /// The place or the span is a hole.
    Hole(Hole),
    /// The ledgers of a place: its unit's, or for a place of unit `any` one per unit.
    Groups {
        /// The place.
        place: String,
        /// Whether the place is of unit `any`, so each ledger is one unit's.
        per_unit: bool,
        /// The ledgers, in the order of their units' ids.
        ledgers: Vec<UnitLedger>,
    },
}

/// One ledger of a place: its unit and its groups. E §6.3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitLedger {
    /// The unit.
    pub unit: String,
    /// The groups, in order.
    pub groups: Vec<Group>,
}

/// The place a ledger is of and its unit; for one unit's ledger of a place of unit `any`
/// (`only`), a leg in another unit is another ledger's (refcore's `only`). E §6.3.
#[derive(Debug, Clone)]
struct Of {
    /// The place.
    place: String,
    /// The ledger's unit.
    unit: String,
    /// Whether it is one unit's ledger of a place of unit `any`.
    only: bool,
}

impl Of {
    /// Whether this ledger counts leg `lg`: every leg, or in one unit's ledger a leg in that
    /// unit or stating none (refcore's `m_touches` with a unit, `m_flow_into` with `only`).
    /// E §6.3.
    fn counts(&self, lg: &LegVal) -> bool {
        !self.only || lg.unit.as_ref().is_none_or(|u| *u == self.unit)
    }

    /// Whether a movement's legs that this ledger counts touch its place (refcore's
    /// `m_touches`). E §6.3.
    fn touches(&self, legs: &[LegVal]) -> bool {
        legs.iter()
            .any(|lg| self.counts(lg) && at_place(lg, &self.place))
    }
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

/// The lines made so far, the ledger they are of, the field they go by, and the observations
/// they hold.
struct Lines {
    /// The place, the unit, and whether the ledger is one unit's.
    of: Of,
    /// `by f`: the field of the observed input the lines go by.
    by: Option<String>,
    /// The lines, in the order made.
    out: Vec<Line>,
    /// The observations a line holds, by id.
    seen: BTreeSet<String>,
}

impl Engine {
    /// A `residuals` ask's ledgers: its place and span read in scope `env`, the ask's binding
    /// tuple's, then the ledger of that place over that span at the context's cut, grouped by
    /// month and move or by field `by`; a place of unit `any` gives one ledger per unit, in
    /// the order of the units' ids (refcore's `answer_residuals`, its reads). E §6.3, L §5.18
    /// (tally 2.1.8 R13: a ledger per binding tuple).
    ///
    /// # Errors
    ///
    /// The refusals reading the place, the span or the ledgers meets.
    pub(crate) fn ledger_of(
        &mut self,
        m: &Reality,
        r: &Residuals,
        (env, ctx): (&Env, Ctx),
    ) -> Result<Ledger, Stop> {
        let place = match self.place_arg(m, &r.place, env, ctx)? {
            Err(h) => return Ok(Ledger::Hole(h)),
            Ok(p) => p,
        };
        let span = match self.span_value(m, &r.span, env, ctx)? {
            Err(h) => return Ok(Ledger::Hole(h)),
            Ok(s) => s,
        };
        let place_unit = m
            .things
            .get(&place)
            .and_then(|t| m.place_unit(t))
            .ok_or_else(|| invariant("a place with no unit"))?;
        let per_unit = place_unit == "any";
        let units = if per_unit {
            self.ledger_units(m, &place, ctx.cut)?
        } else {
            vec![place_unit]
        };
        let mut ledgers = Vec::with_capacity(units.len());
        for unit in units {
            let mut lines = Lines {
                of: Of {
                    place: place.clone(),
                    unit: unit.clone(),
                    only: per_unit,
                },
                by: r.by.clone(),
                out: Vec::new(),
                seen: BTreeSet::new(),
            };
            self.ledger(m, &mut lines, &span, ctx.cut)?;
            let groups = grouped(lines.out)?;
            ledgers.push(UnitLedger { unit, groups });
        }
        Ok(Ledger::Groups {
            place,
            per_unit,
            ledgers,
        })
    }

    /// The units of a place of unit `any`'s ledgers: those of the plan instances' legs at cut
    /// `cut` and of the observations' legs, as known on the model's `on` date, that touch it,
    /// in the order of the units' ids; it takes no reading (refcore's `m_residual_units`).
    /// E §6.3, L §5.10.
    fn ledger_units(&mut self, m: &Reality, place: &str, cut: Day) -> Result<Vec<String>, Stop> {
        let mut units: BTreeSet<String> = BTreeSet::new();
        let mut add = |legs: &[LegVal]| {
            for lg in legs {
                if let Some(u) = &lg.unit
                    && at_place(lg, place)
                {
                    units.insert(u.clone());
                }
            }
        };
        let mut moves = m.moves_in(self.timeline_world());
        moves.retain(|mv| !mv.decision);
        for mv in moves {
            for inst in self.move_instances(m, mv, cut)?.iter() {
                add(&inst.legs);
            }
        }
        for o in self.observations(m, m.on)? {
            add(&o.legs);
        }
        Ok(units.into_iter().collect())
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
        let obs = self.ledger_obs(m, &lines.of, span)?;
        let far = self.far_rows(m, &lines.of, span)?;
        let (model, undated) = self.ledger_model(m, &lines.of, span, cut, (&obs, &far))?;
        // rows stated same are one movement: listed once, naming both rows (L §6.1); the
        // `same` lines are the observations' side, so read as known on `on`, as the rows are
        let (obs, also) = self.same_once_at(m, &lines.of.place, obs, m.on)?;
        // a row stands in for the instance it names whatever places it touches: it is that
        // instance's in every ledger, its flow into this place none where it touches none
        // (E §6.3; tally 2.3.1 part 1's review B4)
        let fulfilling = fulfilling(&[obs.as_slice(), far.as_slice()].concat());
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
            let ids = also.get(&o.id).map_or(o.id.as_str(), String::as_str);
            unexplained_line(m, lines, o, ids)?;
        }
        self.balance_lines(m, lines, span, cut)?;
        after_run_line(m, lines, span)
    }

    /// The observations as known on the model's `on` date that touch the ledger's place,
    /// dated in the span (refcore's `obs`). E §6.3.
    fn ledger_obs(&mut self, m: &Reality, of: &Of, span: &Span) -> Result<Vec<Obs>, Stop> {
        let all = self.observations(m, m.on)?;
        Ok(all
            .iter()
            .filter(|o| of.touches(&o.legs) && span.contains(o.day))
            .cloned()
            .collect())
    }

    /// The observations as known on the model's `on` date, dated in the span, that fulfil an
    /// instance and touch no place of the ledger: each is compared with the instance it names
    /// all the same, its flow into the place none (E §6.3, an instance that an observation
    /// fulfils; tally 2.3.1 part 1's review B4).
    fn far_rows(&mut self, m: &Reality, of: &Of, span: &Span) -> Result<Vec<Obs>, Stop> {
        let all = self.observations(m, m.on)?;
        Ok(all
            .iter()
            .filter(|o| far_row(o, of, span))
            .cloned()
            .collect())
    }

    /// The last day of the observations as known on the model's `on` date that fulfil each
    /// instance, whatever places they touch, in any span: the day of a hole-day instance they
    /// fulfil, in every ledger (refcore's `ful_last`; tally 2.3.1 part 1's review B4). E §6.3
    /// Grouping, L §5.19.
    fn last_fulfilled(&mut self, m: &Reality) -> Result<BTreeMap<String, Day>, Stop> {
        let mut last: BTreeMap<String, Day> = BTreeMap::new();
        for o in self.observations(m, m.on)? {
            if let Some(id) = o.target_id() {
                let day = last.entry(id).or_insert(o.day);
                *day = (*day).max(o.day);
            }
        }
        Ok(last)
    }

    /// The model: each plan instance at cut `cut` that touches the place or that an
    /// observation there `obs` fulfils, dated in the span on a day a statement covers the
    /// place or fulfilled by an observation, `obs` or `far`, and each decision as the main walk
    /// at the cut ran it the same way, in the movement order; and apart, the instances whose
    /// day is a hole, but one an observation dated outside the span fulfils (refcore's
    /// `model`, `undated`). E §6.3, §6.2 (tally 2.3.1 part 1's review B4: whatever places the
    /// row touches).
    fn ledger_model(
        &mut self,
        m: &Reality,
        of: &Of,
        span: &Span,
        cut: Day,
        (obs, far): (&[Obs], &[Obs]),
    ) -> Result<(Vec<Inst>, Vec<Inst>), Stop> {
        let here: BTreeSet<String> = obs.iter().filter_map(Obs::target_id).collect();
        let mut fulfilled = here.clone();
        fulfilled.extend(far.iter().filter_map(Obs::target_id));
        let last = self.last_fulfilled(m)?;
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
                may_touch(m, &inst, of)
            } else {
                of.touches(&inst.legs)
            };
            if !touching && !here.contains(&inst.id) {
                continue;
            }
            if let Some(day) = day {
                let shown = !self.covering(m, &of.place, day, m.on).is_empty()
                    || fulfilled.contains(&inst.id);
                if !(span.contains(day) && shown) {
                    continue;
                }
            }
            // a hole day an observation fulfils is that observation's, in every ledger: a span
            // that leaves it out holds no line of it, never one under `?` (E §6.3 Grouping)
            if fulfilled_day(&inst, &last).is_some_and(|d| !span.contains(d)) {
                continue;
            }
            if refused {
                return Err(refusal_of(&inst));
            }
            match day {
                None => undated.push(inst),
                Some(_) => model.push(inst),
            }
        }
        model.extend(self.ledger_runs(m, of, (span, cut), (&here, &fulfilled))?);
        model.sort_by(|a, b| a.order.cmp(&b.order));
        Ok((model, undated))
    }

    /// The decisions as the main walk at cut `cut` ran them, by instance id, each dated in
    /// the span on a day a statement covers the place or fulfilled by an observation, its id
    /// in `fulfilled`, and touching the place or fulfilled by an observation there, in `here`
    /// (refcore's `w.decs`). E §6.3, §6.2 (tally 2.3.1 part 1's review B4).
    fn ledger_runs(
        &mut self,
        m: &Reality,
        of: &Of,
        (span, cut): (&Span, Day),
        (here, fulfilled): (&BTreeSet<String>, &BTreeSet<String>),
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
            let Some(day) = inst.day.as_ref().and_then(Dated::day) else {
                continue;
            };
            let shown =
                fulfilled.contains(&inst.id) || !self.covering(m, &of.place, day, m.on).is_empty();
            if !span.contains(day) || !shown {
                continue;
            }
            if let Some(err) = &inst.err {
                if may_touch(m, &inst, of) {
                    return Err(Stop::Refused(err.clone()));
                }
                continue;
            }
            if of.touches(&inst.legs) || here.contains(&inst.id) {
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
        let walked = readings(m, (cut, world), (Rows::Unkept, &mut self.entries))?;
        for seen in readings(m, (m.on, world), (Rows::Unkept, &mut self.entries))? {
            if seen.input != "stated" || seen.place != lines.of.place {
                continue;
            }
            let Some(reading) = seen.reading.clone() else {
                continue;
            };
            let at = reading_point(seen.day, seen.edge, reading_rank(&walked, &seen)?);
            if !span.contains(seen.day) {
                continue;
            }
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

/// Whether a refused instance refuses ledger `of`: exactly where a hole one would be a line of
/// it, a place its legs give or any place where a leg's place couldn't be read; with no legs,
/// wherever its move has some (refcore's `may_touch`; tally 2.1.7 R1, R2, SR-1). E §6.3,
/// L §8.4.
fn may_touch(m: &Reality, inst: &Inst, of: &Of) -> bool {
    if inst.legs.is_empty() {
        return m
            .moves
            .get(&inst.move_id)
            .is_some_and(|mv| !mv.decl.legs.is_empty());
    }
    of.touches(&inst.legs)
        || inst
            .legs
            .iter()
            .any(|lg| lg.to.place().is_none() || lg.from.place().is_none())
}

/// Whether observation `o` is dated in the span, fulfils an instance and touches no place of
/// ledger `of` (E §6.3; tally 2.3.1 part 1's review B4).
fn far_row(o: &Obs, of: &Of, span: &Span) -> bool {
    let names = matches!(o.fulfils, Some(Fulfils::Instance(..)));
    names && span.contains(o.day) && !of.touches(&o.legs)
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
/// ledger where it may touch the place, and one whose day is refused stands undated, so it
/// refuses every ledger it may touch. E §6.3 Grouping, L §8.4.
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
        // a refused day may be any day: the instance refuses every ledger of a place it may
        // touch, whatever its span (L §8.4: "every day, for a day that is a hole or refused")
        None if inst.status == Status::Refused => Ok(Modelled::Undated),
        None => Err(invariant("an instance with no day, not refused")),
    }
}

/// The day of a hole-day instance that observations fulfil: the last of their days, in
/// `last` (refcore's `ful_last`); none for a refused instance, which may fall on any day.
/// E §6.3 Grouping, L §5.19.
fn fulfilled_day(inst: &Inst, last: &BTreeMap<String, Day>) -> Option<Day> {
    if inst.status == Status::Refused || !matches!(inst.day, Some(Dated::Hole(_))) {
        return None;
    }
    last.get(&inst.id).copied()
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
    let Some(day) = inst.day.as_ref().and_then(Dated::day) else {
        return Err(invariant("a ledger instance with no day"));
    };
    let mdl = model_of(inst, &lines.of)?;
    let os = fulfilling.of(&inst.id);
    lines.seen.extend(os.iter().map(|o| o.id.clone()));
    let group = move_group(m, lines, day, &inst.move_id, os)?;
    if os.is_empty() {
        if mdl.is_zero() {
            return Ok(());
        }
        let diff = fig_sub(Some(&Fig::Qty(Num::zero())), Some(&mdl))?;
        let text = format!(
            "expected but missing {} on {}: model {}",
            inst.id,
            day.text(),
            fig_text(m, &mdl, &lines.of.unit)?
        );
        lines.out.push(Line {
            day: Some(day),
            group,
            model: Some(mdl),
            observed: None,
            diff,
            text,
            order: inst.order.term()?,
            carry: Vec::new(),
        });
        return Ok(());
    }
    let ob = observed_sum(os, &lines.of)?;
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
        matched_text(m, &lines.of.unit, &inst.id, day, (&mdl, &ob, &diff), os)?
    };
    lines.out.push(Line {
        day: Some(day),
        group,
        model: Some(mdl),
        observed: Some(ob),
        diff,
        text,
        order: inst.order.term()?,
        carry: Vec::new(),
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
    let unit = &lines.of.unit;
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
        Amount::Qty(q) => Fig::Qty(signed(q, way_at(&a.leg, &lines.of.place))?),
    };
    let ob = observed_sum(os, &lines.of)?;
    let diff = fig_sub(Some(&ob), Some(&mdl))?;
    let text = matched_text(m, &lines.of.unit, &a.id, a.day, (&mdl, &ob, &diff), os)?;
    lines.out.push(Line {
        day: Some(a.day),
        group: move_group(m, lines, a.day, &a.move_id, os)?,
        model: Some(mdl),
        observed: Some(ob),
        diff,
        text,
        order: a.order(),
        carry: Vec::new(),
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
    let mdl = model_of(inst, &lines.of)?;
    let os = fulfilling.of(&inst.id);
    if let Some(day) = os.iter().map(|o| o.day).max() {
        lines.seen.extend(os.iter().map(|o| o.id.clone()));
        let ob = observed_sum(os, &lines.of)?;
        let diff = fig_sub(Some(&ob), Some(&mdl))?;
        let text = matched_text(m, &lines.of.unit, &inst.id, day, (&mdl, &ob, &diff), os)?;
        lines.out.push(Line {
            day: Some(day),
            group: move_group(m, lines, day, &inst.move_id, os)?,
            model: Some(mdl),
            observed: Some(ob),
            diff,
            text,
            order: inst.order.term()?,
            carry: Vec::new(),
        });
        return Ok(());
    }
    if mdl.is_zero() {
        return Ok(());
    }
    // a hole as the hole, carrying its unknown days (L §8.2 `unknown day:`)
    let (day_text, carry) = match &inst.day {
        Some(Dated::Hole(h)) => (h.text(), h.carried()),
        Some(Dated::Day(_)) | None => {
            return Err(invariant("an undated instance whose day isn't a hole"));
        }
    };
    let diff = fig_sub(Some(&Fig::Qty(Num::zero())), Some(&mdl))?;
    let text = format!(
        "expected but missing {} on {}: model {}",
        inst.id,
        day_text,
        fig_text(m, &mdl, &lines.of.unit)?
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
        carry,
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
    let hole = Fig::Hole(Hole::after_run(&lines.of.place));
    let text = format!(
        "days {} .. {}: {}",
        after.text(),
        last.text(),
        fig_text(m, &hole, &lines.of.unit)?
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
        carry: Vec::new(),
    });
    Ok(())
}

/// An unexplained line: an observation that fulfils nothing the model at the cut holds, its
/// amount the difference, named by `ids` (its id, or its `same` group's); in its month's
/// `unexplained` group, or with `by f` in the group `<f> <value>` where it has a value of f
/// (refcore's unexplained `RLine`). E §6.3 unexplained, Grouping.
fn unexplained_line(m: &Reality, lines: &mut Lines, o: &Obs, ids: &str) -> Result<(), Stop> {
    let ob = flow_into(&o.legs, &lines.of)?;
    let text = format!(
        "unexplained {} on {}: observed {}",
        ids,
        o.day.text(),
        fig_text(m, &ob, &lines.of.unit)?
    );
    // a value group prints `<f> <value>`, never to be taken for `unexplained`, `balance` or a
    // move (tally 2.1.8 S3)
    let field = match &lines.by {
        Some(f) => obs_field(m, o, f)?.map(|v| format!("{f} {v}")),
        None => None,
    };
    lines.out.push(Line {
        day: Some(o.day),
        group: month_group(o.day, field.unwrap_or_else(|| "unexplained".to_owned()))?,
        model: None,
        observed: Some(ob.clone()),
        diff: ob,
        text,
        order: o.order(),
        carry: Vec::new(),
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
        .any(|(p, role)| *p == lines.of.place && *role == GapRole::Anchor);
    let mark = if anchored { "anchored" } else { "padded" };
    let unit = &lines.of.unit;
    let text = format!(
        "balance {} {} ({mark}): booked {}, stated {}, gap {}",
        lines.of.place,
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
        carry: Vec::new(),
    });
    Ok(())
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

/// Whether leg `lg` touches `place`, from it or to it (refcore's `m_touches`, one leg).
/// E §6.3.
fn at_place(lg: &LegVal, place: &str) -> bool {
    lg.to.place() == Some(place) || lg.from.place() == Some(place)
}

/// Instance `inst`'s model in the ledger: where it touches the place and its existence is
/// no plain value, that cell, the hole or its refusal, which refuses the ledger whole; else
/// its flow into the place (E §6.3; L §8.4; tally 2.3.1 rulings 23 (a), P33).
fn model_of(inst: &Inst, of: &Of) -> Result<Fig, Stop> {
    if of.touches(&inst.legs)
        && let Some(h) = cell_of(inst)?
    {
        return Ok(Fig::Hole(h));
    }
    flow_into(&inst.legs, of)
}

/// A movement's flow into the ledger's place, signed: in +, out −; the holes of the amounts
/// that touch it; a leg's refusal refuses it; in one unit's ledger, a leg in another unit is
/// another ledger's (refcore's `m_flow_into`). E §6.3 Signs.
fn flow_into(legs: &[LegVal], of: &Of) -> Result<Fig, Stop> {
    let (place, unit) = (of.place.as_str(), of.unit.as_str());
    let mut total = Num::zero();
    let mut holes: Vec<&Hole> = Vec::new();
    for lg in legs {
        let way = way_at(lg, place);
        if way == Way::Neither || !of.counts(lg) {
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

/// The observations' flow into the ledger's place, summed (refcore's `ob`, by `_add`).
/// E §6.3.
fn observed_sum(os: &[Obs], of: &Of) -> Result<Fig, Stop> {
    let mut sum: Option<Fig> = None;
    for o in os {
        sum = Some(fig_add(sum, &flow_into(&o.legs, of)?)?);
    }
    sum.ok_or_else(|| invariant("a sum of no observations"))
}

/// The group of a line on `day` of move `move_id` whose observations are `os`: with `by f`,
/// `<f> <value>`, the value of f the observations agree on; else, or where they disagree, one
/// lacks it or there are none, `move <id>` (refcore's `field_group`). E §6.3 Grouping (tally
/// 2.1.8 S3).
fn move_group(
    m: &Reality,
    lines: &Lines,
    day: Day,
    move_id: &str,
    os: &[Obs],
) -> Result<GroupKey, Stop> {
    let mut agreed = None;
    if let Some(f) = &lines.by {
        let mut values: BTreeSet<Option<String>> = BTreeSet::new();
        for o in os {
            values.insert(obs_field(m, o, f)?);
        }
        if let [Some(v)] = values.into_iter().collect::<Vec<_>>().as_slice() {
            agreed = Some(format!("{f} {v}"));
        }
    }
    month_group(day, agreed.unwrap_or_else(|| format!("move {move_id}")))
}

/// The text of field `f` of an observation: its kind, or its attribute `f` as it prints; none
/// where it has no such field (refcore's `obs_field`). E §6.3 Grouping; L §5.19.
fn obs_field(m: &Reality, o: &Obs, f: &str) -> Result<Option<String>, Stop> {
    if f == "kind" {
        return Ok(Some(o.kind.clone()));
    }
    match o.values.iter().find(|(name, _)| name == f) {
        Some((_, v)) => Ok(Some(v.text(m).map_err(num_stop)?)),
        None => Ok(None),
    }
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

#[cfg(test)]
mod tests {
    use crate::api::{Asks, Files, Given, Request, run};

    /// A pay on day `w`, written in for `DAY`, that an observation dated 30 March fulfils
    /// (T's `t23-g1/ledger-fulfilled`, its twin).
    const FULFILLED: &str = r#"model fulfilled
  tally 2
  on 2026-06-10
  start 2026-01-01
  end 2026-12-31

source fixture "invented values"

unit PLN: currency minor 0.01

place konto: accounts PLN lives 2025-01-01..

place unexplained: outside lives 1900-01-01..

place world: outside lives 1900-01-01..

input amt: money PLN

input w: date

input bank: observed

move pw: transfers
  on w
  leg cash from world to konto amount amt

version v
  cite fixture
  governs always
  known 2025-12-01
  opening[konto] = 500.00 PLN at 2026-01-01 start "own"
  amt = 100.00 PLN "its amount"
  w = DAY "its day"

version st
  cite fixture
  governs 2026-03
  known 2026-04-10
  covers konto
  pad konto from unexplained
  stated[konto] = 600.00 PLN at 2026-03-31 end "the statement"
  bank[r1] = (on: 2026-03-30, kind: transfers, leg cash from world to konto amount 100.00 PLN, fulfils: pw) "r1"

ask r_mar: residuals konto 2026-03

ask r_apr: residuals konto 2026-04
"#;

    /// The lines of ask `ask` over [`FULFILLED`] with its day `day`.
    fn lines(day: &str, ask: &str) -> Vec<String> {
        let given = vec![Given {
            path: "fulfilled.tally".to_owned(),
            bytes: Some(FULFILLED.replace("DAY", day).into_bytes()),
        }];
        let request = Request::new(Asks::Named(vec![ask.to_owned()]));
        run(Files::new(given), &request)
            .expect("an answer")
            .lines()
            .to_vec()
    }

    #[test]
    fn a_hole_day_an_observation_fulfils_stands_on_its_day_in_every_ledger() {
        // L §5.19, E §6.3: an instance observations fulfil is matched once, on the last of
        // their days; a ledger whose span leaves that day out holds no line of it, never an
        // expected-but-missing one under `?` (T's t23-g1/ledger-fulfilled; a window and a `?`)
        for day in ["window(2026-03-25, 2026-04-05)", "?\"which day?\""] {
            assert_eq!(
                lines(day, "r_apr"),
                ["r_apr: residuals konto 2026-04", "  cut 2026-06-10"],
                "{day}: April"
            );
            assert_eq!(
                lines(day, "r_mar"),
                [
                    "r_mar: residuals konto 2026-03",
                    "  2026-03 balance: model 600.00 PLN, observed 600.00 PLN, difference 0.00 PLN",
                    "    balance konto 2026-03-31 end (padded): booked 600.00 PLN, stated 600.00 PLN, gap 0.00 PLN",
                    "  2026-03 move pw: model 100.00 PLN, observed 100.00 PLN, difference 0.00 PLN",
                    "    matched pw on 2026-03-30: model 100.00 PLN, observed 100.00 PLN, difference 0.00 PLN, timing +0 days",
                    "  cut 2026-06-10",
                ],
                "{day}: March"
            );
        }
    }

    /// The lines of ask `ask` over [`FULFILLED`] with its day `day`, its row running from the
    /// world to place `save`, the version covering `covers`, and no reading.
    fn far_lines(day: &str, covers: &str, ask: &str) -> Vec<String> {
        let text = FULFILLED
            .replace("DAY", day)
            .replace(
                "place unexplained: outside lives 1900-01-01..\n\nplace world:",
                "place save: accounts PLN lives 2025-01-01..\n\nplace world:",
            )
            .replace("covers konto", &format!("covers {covers}"))
            .replace(
                "  pad konto from unexplained\n  stated[konto] = 600.00 PLN at 2026-03-31 end \
                 \"the statement\"\n",
                "",
            )
            .replace(
                "leg cash from world to konto amount 100.00 PLN, fulfils",
                "leg cash from world to save amount 110.00 PLN, fulfils",
            )
            .replace(
                "ask r_apr:",
                "ask s_mar: residuals save 2026-03\n\nask r_apr:",
            );
        let given = vec![Given {
            path: "far.tally".to_owned(),
            bytes: Some(text.into_bytes()),
        }];
        let request = Request::new(Asks::Named(vec![ask.to_owned()]));
        run(Files::new(given), &request)
            .expect("an answer")
            .lines()
            .to_vec()
    }

    #[test]
    fn an_instance_a_row_fulfils_is_matched_whatever_places_the_row_touches() {
        // E §6.3 (tally 2.3.1 part 1's review B4, the critic's P1-C5 repair): the row at save
        // stands in for pw at konto; each ledger holds pw's matched line on pw's day, its
        // difference the whole amount at that place, covered there or not; a hole day is the
        // row's, never the `?` group's
        let konto = |day: &str, timing: &str| {
            vec![
                "r_mar: residuals konto 2026-03".to_owned(),
                "  2026-03 move pw: model 100.00 PLN, observed 0.00 PLN, difference -100.00 PLN"
                    .to_owned(),
                format!(
                    "    matched pw on {day}: model 100.00 PLN, observed 0.00 PLN, difference \
                     -100.00 PLN, timing {timing}"
                ),
                "  cut 2026-06-10".to_owned(),
            ]
        };
        let save = |day: &str, timing: &str| {
            vec![
                "s_mar: residuals save 2026-03".to_owned(),
                "  2026-03 move pw: model 0.00 PLN, observed 110.00 PLN, difference 110.00 PLN"
                    .to_owned(),
                format!(
                    "    matched pw on {day}: model 0.00 PLN, observed 110.00 PLN, difference \
                     110.00 PLN, timing {timing}"
                ),
                "  cut 2026-06-10".to_owned(),
            ]
        };
        for covers in ["konto, save", "save"] {
            assert_eq!(
                far_lines("2026-03-05", covers, "r_mar"),
                konto("2026-03-05", "+25 days"),
                "konto, covers {covers}"
            );
            assert_eq!(
                far_lines("2026-03-05", covers, "s_mar"),
                save("2026-03-05", "+25 days"),
                "save, covers {covers}"
            );
        }
        let hole = "?\"which day?\"";
        assert_eq!(
            far_lines(hole, "save", "r_mar"),
            konto("2026-03-30", "+0 days"),
            "a hole day, konto"
        );
        assert_eq!(
            far_lines(hole, "save", "s_mar"),
            save("2026-03-30", "+0 days"),
            "a hole day, save"
        );
    }

    #[test]
    fn the_ledger_carries_an_instances_existence_cell_as_it_stands() {
        // E §6.3, L §5.14 An instance's existence is a cell (tally 2.3.1 rulings 23 (a)): the
        // ledger reads the instance; pw's existence is a hole, so its line's model is that
        // hole, while the balance reads the row by the name alone
        let text = FULFILLED
            .replace("DAY", "2026-03-05")
            .replace("  on w\n", "  on w\n  when ok\n")
            .replace("input w: date", "input w: date\n\ninput ok: bool")
            .replace(
                "  w = 2026-03-05 \"its day\"",
                "  w = 2026-03-05 \"its day\"\n  ok = ?\"ok?\" \"s\"",
            );
        let given = vec![Given {
            path: "cell.tally".to_owned(),
            bytes: Some(text.into_bytes()),
        }];
        let request = Request::new(Asks::Named(vec!["r_mar".to_owned()]));
        let lines = run(Files::new(given), &request)
            .expect("an answer")
            .lines()
            .to_vec();
        let matched = "    matched pw on 2026-03-05: model ?\"ok?\" (ok), observed 100.00 PLN, \
                       difference ?\"ok?\" (ok), timing +25 days";
        assert!(
            lines.iter().any(|l| l == matched),
            "the model is the existence's hole: {lines:?}"
        );
        let balance = "    balance konto 2026-03-31 end (padded): booked 600.00 PLN, stated \
                       600.00 PLN, gap 0.00 PLN";
        assert!(
            lines.iter().any(|l| l == balance),
            "the balance reads the row: {lines:?}"
        );
    }
}
