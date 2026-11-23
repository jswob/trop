//! Fence F7: a `flows` read at a past booking cut evaluates the instances it counts, not its move's whole list at that cut.
//! Spec: E §5 step 1 (an instance dated t before the cut is booked at min(t, K), so every day
//! before K is a cut its readers read the move at); E §4.1 (the plan pass: the binding tuples,
//! each instance's day at the cut, its body at its booking cut, its status); L §5.14.3
//! (`flows(m, span)` counts the instances dated in the span; one outside it brings the day
//! that put it there, P20 and P22), §5.19 Links, §8.2 One read set; E §3.2 (a shared
//! structure's reads are none of its reader's).
//! Twin: `Plans::Whole`: plan.rs's `move_instances`, the move's whole list at the cut, then
//! every instance at the keys (`move_flows.rs` `flows_move`, refcore's `flows_move`).
//! Invariants: a narrow read makes the move's binding tuples as the list does and each
//! instance it takes by the list's own step (`tuple_instance`), with the move's plan at that
//! cut in progress, so what it gives an instance is what the list gives it; a stop the list
//! could meet in an instance it skips is either ruled out by the move's cone (it can't reach
//! the move's own plan, a walk or an observation's link) or declined: a refused tuple whose
//! later keys are unbound, an instance that is a hole booking, a decision, an uncertain kind,
//! an `explain` or an `inputs-of`, a read at the ask's cut K or later, and a move whose whole
//! list at that cut is kept are all read whole; the links of every instance an observation at
//! the cut names are judged once per (move, cut), their reads absorbed by the first reader,
//! as the list's status loop absorbs them, and by no later one; and since which reader is a
//! list's first is the whole evaluation's order, a move is read narrowly only at cuts before
//! the first day a link to another move in its cone is known, where no instance it skips can
//! be a first reader that takes a link's reads (group 6c's review, B1). `o.fulfils` takes no
//! list in either mode: it reads the instance's existence alone (`tuples.rs`
//! `linked_instance`, P22). The BUILD-LOG's G6c entries list every stop and its ruling.
//! PERF: P8-L10 (the stage runs: `spend[d]` is daily, so every day before K is a cut its
//! reads take the moves at, and each whole list ran to the horizon: at G6b's profile
//! `move_instances` held 67 % of b1's time at 2030, `flows_move` 17 %), the count row
//! F7-c1r2 (`c1_r2`'s def misses, set from G6c's measure), and P8-14, whose def misses it
//! must not raise past its budget;
//! plain twin: plan.rs (`move_instances` under `Plans::Whole`, then the list's instances,
//! refcore's `flows_move`); the equivalence tests: tally-runner's `m4_narrow`, tally-cli's
//! `narrow_oracle.rs`, and this file's tests.

use super::instance::{Dated, Inst, Status, inst_order};
use super::plan::{date_of, invariant, kind_certainty};
use super::tuples::Tuple;
use crate::base::{Code, Refusal, Run};
use crate::eval::{Collected, Engine};
use crate::model::{Fulfils, KeyVal, MoveInfo, Node, NodeKind, Reality, Rel, Span, Value};
use crate::syntax::{Certainty, Stop};
use crate::time::Day;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

/// Whether a read at a past booking cut evaluates only the instances it names: the switch
/// F7's equivalence tests turn (P8-L10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plans {
    /// Only the instances a read names: fence F7, as `run` reads.
    Narrow,
    /// The move's whole list at the cut, then the instances a read names: the plain twin.
    Whole,
}

/// One plan move's binding tuples at one past booking cut, as narrow reads have taken them:
/// each tuple's instance once evaluated and each one's day once read, by its place among the
/// tuples (P8-L10).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Narrowed {
    /// The tuples, in binding order, as the list makes them.
    tuples: Vec<Tuple>,
    /// Each tuple's place by its instance's id.
    by_id: BTreeMap<String, Vec<usize>>,
    /// Each tuple's instance once evaluated, with its status: none where its day is `never`
    /// or `none`.
    made: BTreeMap<usize, Option<Inst>>,
    /// Each tuple's day once read.
    days: BTreeMap<usize, Probe>,
    /// Whether a tuple is refused with its later keys unbound, which stops the list: every
    /// read of the move at the cut is then read whole.
    whole: bool,
    /// Whether the links of every instance an observation at the cut names are judged, their
    /// reads absorbed by the first reader.
    judged: bool,
}

/// At which cuts a read of a move may be narrow, by its cone (P8-L10; [`narrows`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Narrows {
    /// At none: its cone reaches its own plan, a walk or an observation.
    No,
    /// At every cut before the day: the first a link to an instance of another move in its
    /// cone is known on.
    Before(Day),
    /// At every cut: no link to another move in its cone.
    Always,
}

/// What a move's kept tuples at a cut give for some keys (P8-L10; [`Engine::kept_tuple`]).
#[derive(Debug)]
pub(super) enum KeptTuple {
    /// None kept: `Plans::Whole`, or a decision; the reader makes its own.
    Own,
    /// The tuple at the keys, or none, and whether a tuple stops the move's list.
    Kept(Option<Box<Tuple>>, bool),
}

/// What reading a tuple's day at the cut gave, before its body (L §5.14, E §4.1 step 2).
#[derive(Debug, Clone, PartialEq, Eq)]
enum Probe {
    /// `never` or `none`: the tuple has no instance.
    Gone,
    /// A day, with what reading it met.
    Day(Day, Collected),
    /// A hole, a window, a refusal: the instance is taken whole.
    Taken,
}

impl Engine {
    /// The instances of plan move `mv` at keys starting `want` that a `flows` over span `sp`
    /// at cut `cut` counts, in the list's order: each one's as the list gives it, but one
    /// whose day at the cut is a day outside the span, which brings only that day's reads
    /// (`uncounted_reads`), so it is handed over with its day and day's reads alone (refcore's
    /// `flows_move`, its list). L §5.14.3, §8.2 One read set (P20, P22); E §4.1.
    ///
    /// # Errors
    ///
    /// As [`Engine::move_instances`].
    pub(crate) fn flows_instances(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        (want, sp): (&[KeyVal], &Span),
        cut: Day,
    ) -> Result<Rc<Vec<Inst>>, Stop> {
        if let Some(insts) = self.narrow_flows(m, mv, (want, sp), cut)? {
            return Ok(Rc::new(insts));
        }
        self.move_instances(m, mv, cut)
    }

    /// The instances of plan move `mv` at cut `cut` the measure of its adjustments takes, in
    /// the list's order: at a past cut only those dated before it or whose day is a hole, a
    /// window or a refusal, each as the list gives it, the others' bodies never evaluated,
    /// since the measure skips every instance dated on the cut or later whatever its body
    /// gives; else, and for a walk up to a day, `plan_list`'s (the twin). E §5 steps 1–4,
    /// E §4.1; P8-L10.
    ///
    /// # Errors
    ///
    /// As [`Engine::plan_list`].
    pub(crate) fn measured_list(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        cut: Day,
    ) -> Result<Rc<Vec<Inst>>, Stop> {
        if self.list_upto(mv, cut).is_none()
            && let Some(insts) = self.narrow_measured(m, mv, cut)?
        {
            return Ok(Rc::new(insts));
        }
        self.plan_list(m, mv, cut)
    }

    /// The instances a measure takes read narrowly (see [`Engine::measured_list`]), or none
    /// where the read is declined: also where one it takes is a hole booking before the cut,
    /// which a later version answers over the whole list. E §5 steps 1–4, §4.1.
    fn narrow_measured(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        cut: Day,
    ) -> Result<Option<Vec<Inst>>, Stop> {
        if !self.may_narrow(m, mv, cut)? || !self.narrowed(m, mv, cut)? {
            return Ok(None);
        }
        let groups = self.same_groups(m, cut)?;
        let mut out = Vec::new();
        for at in self.tuples_from(mv, cut, &[]) {
            let Some((plain, id)) = self.tuple_plain(mv, cut, at) else {
                return Err(invariant("a narrow read's tuple past its tuples"));
            };
            let day = if plain && !groups.contains_key(&id) {
                self.narrow_day(m, mv, cut, at)?
            } else {
                Probe::Taken
            };
            match day {
                Probe::Gone => continue,
                Probe::Day(d, _) if d >= cut => continue,
                Probe::Day(..) | Probe::Taken => {}
            }
            let Some(inst) = self.narrow_inst(m, mv, cut, at)? else {
                continue;
            };
            if crate::book::hole_booked_before(&inst, cut) {
                return Ok(None);
            }
            out.push(inst);
        }
        out.sort_by(|a, b| a.order.cmp(&b.order));
        Ok(Some(out))
    }

    /// Whether a read of move `mv` at cut `cut` may be narrow: F7 on, a plan, a cut before
    /// the ask's, no ask traced or listing its inputs, no whole list kept at that cut, a cone
    /// that can't reach the move's own plan, and a cut before the first day a link to an
    /// instance of another move in that cone is known ([`narrows`]). E §4.1, §5 step 1, §10;
    /// L §5.19 Links, §8.2 One read set; P8-L10.
    ///
    /// # Errors
    ///
    /// As [`narrows`].
    fn may_narrow(&mut self, m: &Reality, mv: &MoveInfo, cut: Day) -> Result<bool, Stop> {
        if self.plan_reads == Plans::Whole
            || mv.decision
            || cut >= self.ask_cut
            || self.explaining()
            || self.plans.contains_key(&(mv.id.clone(), cut))
        {
            return Ok(false);
        }
        let reach = if let Some(known) = self.prefixes.narrows.get(&mv.id) {
            *known
        } else {
            let found = narrows(m, mv)?;
            self.prefixes.narrows.insert(mv.id.clone(), found);
            found
        };
        Ok(match reach {
            Narrows::No => false,
            Narrows::Before(day) => cut < day,
            Narrows::Always => true,
        })
    }

    /// The instances a `flows` counts read narrowly (see [`Engine::flows_instances`]), or
    /// none where the read is declined: also for a move of an uncertain kind, whose instance
    /// outside the span the read skips or brings by what its body gives (L §5.7).
    fn narrow_flows(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        (want, sp): (&[KeyVal], &Span),
        cut: Day,
    ) -> Result<Option<Vec<Inst>>, Stop> {
        if !self.may_narrow(m, mv, cut)?
            || kind_certainty(m, &mv.kind) == Some(Certainty::Uncertain)
            || !self.narrowed(m, mv, cut)?
        {
            return Ok(None);
        }
        let groups = self.same_groups(m, cut)?;
        let mut out = Vec::new();
        for at in self.tuples_from(mv, cut, want) {
            let Some((plain, id)) = self.tuple_plain(mv, cut, at) else {
                return Err(invariant("a narrow read's tuple past its tuples"));
            };
            let day = if plain && !groups.contains_key(&id) {
                self.narrow_day(m, mv, cut, at)?
            } else {
                Probe::Taken
            };
            match day {
                Probe::Gone => {}
                Probe::Day(d, day_coll) if !sp.contains(d) => {
                    out.push(self.outside_span(m, mv, (cut, at), (d, day_coll))?);
                }
                Probe::Day(..) | Probe::Taken => {
                    let Some(inst) = self.narrow_inst(m, mv, cut, at)? else {
                        continue;
                    };
                    if crate::book::hole_booked_before(&inst, cut) {
                        return Ok(None);
                    }
                    out.push(inst);
                }
            }
        }
        out.sort_by(|a, b| a.order.cmp(&b.order));
        Ok(Some(out))
    }

    /// Whether the narrow reads of move `mv` at cut `cut` may go on: its binding tuples made,
    /// once, as the list makes them, with the move's plan at that cut in progress; false where
    /// one is refused with its later keys unbound, which stops the list (refcore's
    /// `insts_of_tuples` raises); and the links of every instance an observation at the cut
    /// names judged, once, their reads absorbed by this reader, as the list's status loop
    /// absorbs them into the list's first reader. E §4.1 steps 1–3, L §5.19 Links.
    ///
    /// # Errors
    ///
    /// `RUN-CYCLE` for a move that reads its own instances; the stops of the tuples, as the
    /// list meets them; as [`Engine::obs_links_for`].
    fn narrowed(&mut self, m: &Reality, mv: &MoveInfo, cut: Day) -> Result<bool, Stop> {
        self.plan_open(mv, cut)?;
        let key = (mv.id.clone(), cut);
        self.keep_tuples(m, mv, cut)?;
        let Some(entry) = self.prefixes.narrow.get(&key) else {
            return Err(invariant("a narrow read's tuples just kept"));
        };
        if entry.whole {
            return Ok(false);
        }
        if !entry.judged {
            self.judge_named(m, mv, cut)?;
        }
        Ok(true)
    }

    /// The tuple of plan move `mv` at keys `keys` under cut `cut`, as `move_tuples` gives it
    /// there, from the move's tuples at that cut made once and kept, with whether one of them
    /// is refused with its later keys unbound, which stops the move's list; a decision's, and
    /// any under `Plans::Whole`, its reader makes (the twin: `link_tuple`'s and
    /// `linked_instance`'s own). E §4.1 step 1, L §5.19 Links; P8-L10.
    ///
    /// # Errors
    ///
    /// The stops of the tuples, as `move_tuples` meets them.
    pub(super) fn kept_tuple(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        keys: &[KeyVal],
        cut: Day,
    ) -> Result<KeptTuple, Stop> {
        if self.plan_reads == Plans::Whole || mv.decision {
            return Ok(KeptTuple::Own);
        }
        self.keep_tuples(m, mv, cut)?;
        let Some(entry) = self.prefixes.narrow.get(&(mv.id.clone(), cut)) else {
            return Err(invariant("a narrow read's tuples just kept"));
        };
        let places: &[usize] = entry
            .by_id
            .get(&crate::book::inst_id(&mv.id, keys))
            .map_or(&[], Vec::as_slice);
        let mut found = None;
        for at in places {
            if let Some(tuple) = entry.tuples.get(*at)
                && tuple.keys == keys
            {
                found = Some(Box::new(tuple.clone()));
                break;
            }
        }
        Ok(KeptTuple::Kept(found, entry.whole))
    }

    /// Move `mv`'s binding tuples at cut `cut` made once and kept: under the move, their reads
    /// the plan's, with no plan guard, since a link judged while the move's list is made takes
    /// them too (a narrow read checks the guard as its twin's list does; an existence read
    /// takes none, P30). E §4.1 step 1, §3.2.
    ///
    /// # Errors
    ///
    /// The stops of the tuples, as `move_tuples` meets them; none is kept then.
    fn keep_tuples(&mut self, m: &Reality, mv: &MoveInfo, cut: Day) -> Result<(), Stop> {
        let key = (mv.id.clone(), cut);
        if self.prefixes.narrow.contains_key(&key) {
            return Ok(());
        }
        let outer = self.holding(Node::new(NodeKind::Move, &mv.id));
        let mark = self.reads_apart();
        let tctx = self.timeline_ctx(None, cut);
        let made = self.move_tuples(m, mv, tctx);
        self.reads_back(mark);
        self.holder = outer;
        let tuples = made?;
        let mut by_id: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (at, tuple) in tuples.iter().enumerate() {
            let id = crate::book::inst_id(&mv.id, &tuple.keys);
            by_id.entry(id).or_insert_with(Vec::new).push(at);
        }
        let whole = tuples
            .iter()
            .any(|t| t.refused.is_some() && t.keys.len() < mv.binds.len());
        let entry = Narrowed {
            tuples,
            by_id,
            whole,
            ..Narrowed::default()
        };
        self.prefixes.narrow.insert(key, entry);
        Ok(())
    }

    /// The links of every instance of move `mv` an observation at cut `cut` names, each
    /// instance evaluated and its links judged, their reads absorbed by this reader, in the
    /// list's order: what the list's status loop absorbs into its first reader (refcore's
    /// `move_instances`, its statuses). E §4.1 step 3, L §5.19 Links.
    ///
    /// # Errors
    ///
    /// As [`Engine::narrow_inst`] and [`Engine::obs_links_for`].
    fn judge_named(&mut self, m: &Reality, mv: &MoveInfo, cut: Day) -> Result<(), Stop> {
        let key = (mv.id.clone(), cut);
        let targets: Vec<String> = self
            .observations(m, cut)?
            .iter()
            .filter_map(crate::book::Obs::target_id)
            .collect();
        let mut named = BTreeSet::new();
        if let Some(entry) = self.prefixes.narrow.get(&key) {
            for tid in &targets {
                if let Some(places) = entry.by_id.get(tid) {
                    named.extend(places.iter().copied());
                }
            }
        }
        // the union of what the links read is the same in any order (`Collected::merge`)
        for at in named {
            if let Some(inst) = self.narrow_inst(m, mv, cut, at)? {
                self.obs_links_for(m, &inst.id, cut)?;
            }
        }
        if let Some(entry) = self.prefixes.narrow.get_mut(&key) {
            entry.judged = true;
        }
        Ok(())
    }

    /// Whether narrow reads judged the links of move `id`'s instances at cut `cut`, their
    /// reads absorbed by the first reader, so the list made later at that cut absorbs none, as
    /// the twin's cached list does. E §4.1 step 3, §3.2; P8-L10.
    pub(crate) fn links_judged(&self, id: &str, cut: Day) -> bool {
        self.prefixes
            .narrow
            .get(&(id.to_owned(), cut))
            .is_some_and(|e| e.judged)
    }

    /// The instance of tuple `at` of move `mv` at cut `cut`, once: made by the list's own step
    /// with the move's plan at that cut in progress, its order and status set as the list sets
    /// them, the status's link reads absorbed by none (the list's first reader took them,
    /// [`Engine::judge_named`]). E §4.1 steps 1–3, §5 step 1.
    ///
    /// # Errors
    ///
    /// As [`Engine::tuple_instance`] and [`Engine::status`]; `RUN-CYCLE` for a move that reads
    /// its own instances.
    fn narrow_inst(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        cut: Day,
        at: usize,
    ) -> Result<Option<Inst>, Stop> {
        let key = (mv.id.clone(), cut);
        let kept = self.prefixes.narrow.get(&key);
        if let Some(made) = kept.and_then(|e| e.made.get(&at)) {
            return Ok(made.clone());
        }
        let Some(tuple) = kept.and_then(|e| e.tuples.get(at)).cloned() else {
            return Err(invariant("a narrow read's tuple past its tuples"));
        };
        self.enter_plan(mv, cut)?;
        let outer = self.holding(Node::new(NodeKind::Move, &mv.id));
        let mark = self.reads_apart();
        let tctx = self.timeline_ctx(None, cut);
        let made = self.tuple_instance(m, mv, (tuple, cut, None), tctx, None);
        self.reads_back(mark);
        self.holder = outer;
        self.plan_active.remove(&key);
        let mut made = made?;
        if let Some(inst) = &mut made {
            inst.order = inst_order(m, mv, inst);
            self.push();
            let status = self.status(m, mv, inst, cut);
            self.pop_alone();
            inst.status = status?;
        }
        if let Some(entry) = self.prefixes.narrow.get_mut(&key) {
            entry.made.insert(at, made.clone());
        }
        Ok(made)
    }

    /// The day of tuple `at` of move `mv` read at cut `cut`, once, as the list reads it before
    /// the body (its law date, then its `on`; refcore's `eval_inst`), with what reading it met.
    /// E §4.1 step 2, §5 step 0; L §5.14.
    ///
    /// # Errors
    ///
    /// A load refusal met at run, `NotBuilt`, `Deep`, as the list meets them reading that day;
    /// `RUN-CYCLE` for a move that reads its own instances.
    fn narrow_day(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        cut: Day,
        at: usize,
    ) -> Result<Probe, Stop> {
        let key = (mv.id.clone(), cut);
        let kept = self.prefixes.narrow.get(&key);
        if let Some(day) = kept.and_then(|e| e.days.get(&at)) {
            return Ok(day.clone());
        }
        let Some(tuple) = kept.and_then(|e| e.tuples.get(at)).cloned() else {
            return Err(invariant("a narrow read's tuple past its tuples"));
        };
        self.enter_plan(mv, cut)?;
        let outer = self.holding(Node::new(NodeKind::Move, &mv.id));
        let mark = self.reads_apart();
        let tctx = self.timeline_ctx(None, cut);
        let mut inst = Inst::new(mv, tuple.keys, tuple.env);
        self.push();
        let read = self.law_and_day(m, mv, &mut inst, tctx);
        let day_coll = self.pop_alone();
        self.reads_back(mark);
        self.holder = outer;
        self.plan_active.remove(&key);
        // a day as `inst_parts` takes it
        let day = match read {
            Ok(Ok(Value::Never | Value::None)) => Probe::Gone,
            Ok(Ok(v @ (Value::Day(_) | Value::Period(_)))) => match date_of(&v) {
                Ok(d) => Probe::Day(d, day_coll),
                Err(stop) => taken(stop)?,
            },
            // a law date or a day that is a hole, or a window: its hole day, read whole
            Ok(_) => Probe::Taken,
            Err(stop) => taken(stop)?,
        };
        if let Some(entry) = self.prefixes.narrow.get_mut(&key) {
            entry.days.insert(at, day.clone());
        }
        Ok(day)
    }

    /// Instance of tuple `at` of move `mv` at cut `cut` whose day `d` there is outside a
    /// `flows` read's span, as `count_move` takes it: its keys, id, day, order and the day's
    /// reads `day_coll` alone, since such an instance brings that day whatever its body gives,
    /// whether it exists, is refused, modelled, past or planned (`uncounted_reads`; L §8.2 One
    /// read set, P20). L §5.14.3.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a tuple past the move's tuples.
    fn outside_span(
        &mut self,
        m: &Reality,
        mv: &MoveInfo,
        (cut, at): (Day, usize),
        (d, day_coll): (Day, Collected),
    ) -> Result<Inst, Stop> {
        let Some(tuple) = self
            .prefixes
            .narrow
            .get(&(mv.id.clone(), cut))
            .and_then(|e| e.tuples.get(at))
            .cloned()
        else {
            return Err(invariant("a narrow read's tuple past its tuples"));
        };
        let mut inst = Inst::new(mv, tuple.keys, tuple.env);
        inst.day = Some(Dated::Day(d));
        inst.day_coll = day_coll;
        inst.status = if d < cut { Status::Past } else { Status::Plan };
        inst.order = inst_order(m, mv, &inst);
        Ok(inst)
    }

    /// The places among move `mv`'s tuples at cut `cut` whose keys start with `want`, in
    /// binding order: the instances a `flows` at those keys counts. L §5.14.3, E §4.1 step 1.
    fn tuples_from(&self, mv: &MoveInfo, cut: Day, want: &[KeyVal]) -> Vec<usize> {
        let Some(entry) = self.prefixes.narrow.get(&(mv.id.clone(), cut)) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for (at, tuple) in entry.tuples.iter().enumerate() {
            if tuple.keys.starts_with(want) {
                out.push(at);
            }
        }
        out
    }

    /// Whether tuple `at` of move `mv` at cut `cut` is a plain one, neither refused nor a
    /// hole, whose day a read may take alone; with its instance's id. L §5.14 Keys, §8.4.
    fn tuple_plain(&self, mv: &MoveInfo, cut: Day, at: usize) -> Option<(bool, String)> {
        let tuple = self
            .prefixes
            .narrow
            .get(&(mv.id.clone(), cut))?
            .tuples
            .get(at)?;
        let plain = tuple.refused.is_none() && tuple.domain_hole.is_none();
        Some((plain, crate::book::inst_id(&mv.id, &tuple.keys)))
    }

    /// Move `mv`'s plan at cut `cut` put in progress, as the list's is while it is made, so a
    /// read of the move's own instances from inside it is the list's `RUN-CYCLE` (refcore's
    /// `move_instances`, its guard). L §5.11, E §4.1.
    ///
    /// # Errors
    ///
    /// `RUN-CYCLE` where it already is.
    pub(super) fn enter_plan(&mut self, mv: &MoveInfo, cut: Day) -> Result<(), Stop> {
        self.plan_open(mv, cut)?;
        self.plan_active.insert((mv.id.clone(), cut));
        Ok(())
    }

    /// Move `mv`'s plan at cut `cut` not in progress, as a read of its list asks (refcore's
    /// `move_instances`, its guard). L §5.11, E §4.1.
    ///
    /// # Errors
    ///
    /// `RUN-CYCLE` where it is: a move that reads its own instances.
    pub(super) fn plan_open(&mut self, mv: &MoveInfo, cut: Day) -> Result<(), Stop> {
        if self.plan_active.contains(&(mv.id.clone(), cut)) {
            self.narrow_open_to_days();
            return Err(Stop::Refused(Refusal::new(
                Code::Run(Run::Cycle),
                format!("{} reads its own instances", mv.id),
            )));
        }
        Ok(())
    }
}

/// Whether move `mv`'s cone can't reach its own plan, so an instance of it a narrow read skips
/// can't meet the move's list in progress, a walk or an observation's link: from its own
/// nodes, along the item graph's edges, no read of the move's movements (its `flows`, a field
/// of an instance, a binding over its kind) other than its body's of its own day, no balance
/// or value read, no binding over observed rows or `observed`, no running read or projection
/// (`marks`). A `flows` of another move is followed into that move's cone; its edge to the
/// balances, the walk that refuses a guarded leg, is never walked inside a plan pass
/// (`leg_refused` reads only a walk already kept there). E §3.1 steps 6–8, L §5.11.
///
/// And from which cut on it is read whole: the first day a link to an instance of another
/// move in that cone is known. The list of such a move at a cut where its observations name
/// instances gives the links' reads to its first reader, which a narrow read of this move
/// there could change by skipping the instance that is that reader (L §5.19 Links, §8.2 One
/// read set; group 6c's review, B1). Before that day no list in the cone has a link to give.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for an observation of a version load didn't build.
fn narrows(m: &Reality, mv: &MoveInfo) -> Result<Narrows, Stop> {
    let body = Node::new(NodeKind::Move, &mv.id);
    let day = Node::new(NodeKind::MoveOn, &mv.id);
    let mut seen = BTreeSet::new();
    let mut reached = BTreeSet::new();
    let mut stack = vec![body.clone(), day.clone()];
    while let Some(next) = stack.pop() {
        if !seen.insert(next.clone()) {
            continue;
        }
        if m.graph
            .marks
            .get(&next)
            .is_some_and(|marks| !marks.is_empty())
            || matches!(
                next.kind,
                NodeKind::Balances | NodeKind::Price | NodeKind::Parts | NodeKind::Fill
            )
        {
            return Ok(Narrows::No);
        }
        if matches!(next.kind, NodeKind::Move | NodeKind::MoveOn) && next.id != mv.id {
            reached.insert(next.id.clone());
        }
        let Some(reads) = m.graph.reads(&next) else {
            continue;
        };
        for (target, rels) in reads {
            let own_day = next == body && *target == day;
            if !own_day && (*target == body || *target == day) {
                return Ok(Narrows::No);
            }
            if rels
                .iter()
                .any(|r| matches!(r, Rel::Bind | Rel::Named | Rel::Observed | Rel::Bal))
            {
                return Ok(Narrows::No);
            }
            if target.kind == NodeKind::Balances && rels.iter().all(|r| *r == Rel::Flows) {
                continue;
            }
            stack.push(target.clone());
        }
    }
    Ok(first_link(m, &reached)?.map_or(Narrows::Always, Narrows::Before))
}

/// The first day a link to an instance of one of `moves` is known: the earliest `known` of a
/// row that states a `fulfils` naming one, in any version and any world, or none (L §5.19
/// Links, §6.4). A row stated `same` with one takes that row's `fulfils`, known by then.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for an observation of a version load didn't build.
fn first_link(m: &Reality, moves: &BTreeSet<String>) -> Result<Option<Day>, Stop> {
    let mut first: Option<Day> = None;
    for (_, input) in m.inputs.entries() {
        for o in &input.observations {
            let Some(Fulfils::Instance(named, _)) = &o.fulfils else {
                continue;
            };
            if !moves.contains(named) {
                continue;
            }
            let version = m
                .versions
                .get(&o.version)
                .ok_or_else(|| invariant("an observation of a version load didn't build"))?;
            let known = o.own_known.unwrap_or(version.known);
            first = Some(first.map_or(known, |d| d.min(known)));
        }
    }
    Ok(first)
}

/// A stop met reading an instance's day: a run-time refusal is the instance's own, its `err`,
/// so the list's step takes it whole; any other stop is the list's (`eval_inst`; R-66).
/// E §4.1 step 2, L §8.4.
///
/// # Errors
///
/// The stop, where it isn't a run-time refusal.
fn taken(stop: Stop) -> Result<Probe, Stop> {
    match stop {
        Stop::Refused(r) if !matches!(r.code(), Code::Load(_)) => Ok(Probe::Taken),
        stop @ (Stop::Refused(_) | Stop::NotBuilt(_) | Stop::Deep) => Err(stop),
    }
}

#[cfg(test)]
mod tests {
    use super::{Narrows, Plans, narrows};
    use crate::api::{Asks, Files, Given, Request, Source, run_with_how};
    use crate::eval::How;
    use crate::load::load;
    use crate::syntax::parse_reality;
    use crate::time::Day;

    /// A month's spending, booked on its 25th, reads the loose card spending of the month: the
    /// observations whose `fulfils` is none. The rent's observations name its instances: one
    /// whole, one refused (a negative amount), one whose amount is a hole, one whose day is a
    /// hole, one whose day is a window.
    const LINKS: &str = r#"model links
  tally 2
  on 2026-06-20
  start 2026-01-01
  end 2026-07-31

source s "invented values"

unit PLN: currency minor 0.01

index month = month 2026-01..2026-07

place konto: accounts PLN lives 2026-01-01..

place card: accounts PLN lives 2026-01-01..

place landlord: outside lives 1900-01-01..

place shop: outside lives 1900-01-01..

input bank: observed

input rent_amt[m in month]: money PLN

input rent_day[m in month]: date

input base: money PLN

input spend_dom: int

move rent[m in month]: transfers
  on rent_day[m]
  at start_of(m)
  leg cash from konto to landlord amount rent_amt[m]

def loose[m in month]: money PLN = sum(o.cash.amount for o in bank over m where o.fulfils == none) at start_of(m)

move spend[m in month]: transfers
  on day_of(m, spend_dom)
  at start_of(m)
  leg cash from konto to shop amount max(0, base - loose[m])

ask k: bal(konto, 2026-06-19)

ask f: flows(spend, m) for m in [2026-01, 2026-02, 2026-03, 2026-04, 2026-05, 2026-06]

version v
  cite s
  governs always
  known 2026-01-01
  base = 300.00 PLN "s"
  spend_dom = 25 "s"
  rent_amt[2026-01] = 800.00 PLN "s"
  rent_amt[2026-02] = -5.00 PLN "refused: a negative amount"
  rent_amt[2026-04] = 800.00 PLN "s"
  rent_amt[2026-05] = 800.00 PLN "s"
  rent_amt[2026-06] = 800.00 PLN "s"
  rent_amt[2026-07] = 800.00 PLN "s"
  rent_day[2026-01] = 2026-01-05 "s"
  rent_day[2026-02] = 2026-02-05 "s"
  rent_day[2026-03] = 2026-03-05 "s"
  rent_day[2026-05] = window(2026-05-03, 2026-05-08) "a window"
  rent_day[2026-06] = 2026-06-05 "s"
  rent_day[2026-07] = 2026-07-05 "s"
  opening[konto] = 9000.00 PLN at 2026-01-01 start "s"
  opening[card] = 1000.00 PLN at 2026-01-01 start "s"

version jan
  cite s
  governs 2026-01
  known 2026-01-21
  covers card
  bank[a1] = (on: 2026-01-02, kind: transfers, leg cash from card to shop amount 40.00 PLN) "loose"
  bank[a2] = (on: 2026-01-05, kind: transfers, leg cash from konto to landlord amount 800.00 PLN, fulfils: rent[2026-01]) "whole"

version feb
  cite s
  governs 2026-02
  known 2026-02-21
  covers card
  bank[b1] = (on: 2026-02-03, kind: transfers, leg cash from card to shop amount 50.00 PLN) "loose"
  bank[b2] = (on: 2026-02-05, kind: transfers, leg cash from konto to landlord amount 800.00 PLN, fulfils: rent[2026-02]) "the refused one"

version mar
  cite s
  governs 2026-03
  known 2026-03-21
  covers card
  bank[c1] = (on: 2026-03-04, kind: transfers, leg cash from card to shop amount 60.00 PLN) "loose"
  bank[c2] = (on: 2026-03-05, kind: transfers, leg cash from konto to landlord amount 800.00 PLN, fulfils: rent[2026-03]) "its amount a hole"

version apr
  cite s
  governs 2026-04
  known 2026-04-21
  covers card
  bank[d1] = (on: 2026-04-06, kind: transfers, leg cash from konto to landlord amount 800.00 PLN, fulfils: rent[2026-04]) "its day a hole"
  bank[d2] = (on: 2026-04-10, kind: transfers, leg cash from card to shop amount 70.00 PLN) "loose"

version may
  cite s
  governs 2026-05
  known 2026-05-21
  covers card
  bank[e1] = (on: 2026-05-06, kind: transfers, leg cash from konto to landlord amount 800.00 PLN, fulfils: rent[2026-05]) "its day a window"
  bank[e2] = (on: 2026-05-11, kind: transfers, leg cash from card to shop amount 80.00 PLN) "loose"
"#;

    /// A month's top-up, booked on its 20th, reads the flows of the rent and of a fee in its
    /// month: a refused rent, a missing one, fee days on a span's first and last days, one a
    /// window.
    const SPANS: &str = r#"model spans
  tally 2
  on 2026-06-20
  start 2026-01-01
  end 2026-07-31

source s "invented values"

unit PLN: currency minor 0.01

index month = month 2026-01..2026-07

place konto: accounts PLN lives 2026-01-01..

place pot: accounts PLN lives 2026-01-01..

place landlord: outside lives 1900-01-01..

place shop: outside lives 1900-01-01..

input rent_amt[m in month]: money PLN

input fee_day[m in month]: date

input fee_amt: money PLN

input share: num

move rent[m in month]: transfers
  on start_of(m)
  at start_of(m)
  leg cash from pot to landlord amount rent_amt[m]

move fee[m in month]: transfers
  on fee_day[m]
  at start_of(m)
  leg cash from pot to landlord amount fee_amt

move top[m in month]: transfers
  on day_of(m, 20)
  at start_of(m)
  leg cash from konto to shop amount round(flows(rent, m) * share, minor(PLN), half-up)

move top_fee[m in month]: transfers
  on day_of(m, 20)
  at start_of(m)
  leg cash from konto to shop amount flows(fee, m .. (m + 1))

ask r: flows(top, m) for m in [2026-01, 2026-02, 2026-03, 2026-04, 2026-05, 2026-06, 2026-07]

ask t: flows(top_fee, m) for m in [2026-01, 2026-03, 2026-04, 2026-05, 2026-06]

ask k: bal(konto, 2026-06-19)

version v
  cite s
  governs always
  known 2026-01-01
  fee_amt = 7.00 PLN "s"
  share = 0.1 "s"
  rent_amt[2026-01] = 900.00 PLN "s"
  rent_amt[2026-02] = -5.00 PLN "refused: a negative amount"
  rent_amt[2026-04] = 910.00 PLN "s"
  rent_amt[2026-05] = 920.00 PLN "s"
  rent_amt[2026-06] = 930.00 PLN "s"
  rent_amt[2026-07] = 940.00 PLN "s"
  fee_day[2026-01] = 2026-01-01 "a span's first day"
  fee_day[2026-02] = 2026-02-28 "a span's last day"
  fee_day[2026-03] = 2026-03-15 "s"
  fee_day[2026-04] = 2026-04-30 "s"
  fee_day[2026-05] = window(2026-05-03, 2026-05-08) "a window"
  fee_day[2026-06] = 2026-06-01 "s"
  fee_day[2026-07] = 2026-07-31 "s"
  opening[konto] = 5000.00 PLN at 2026-01-01 start "s"
  opening[pot] = 9000.00 PLN at 2026-01-01 start "s"
"#;

    /// The lines of asks `asks` of `text`, read as `plans` says.
    fn lines(text: &str, asks: &[&str], plans: Plans) -> Vec<String> {
        let given = vec![Given {
            path: "narrow.tally".to_owned(),
            bytes: Some(text.as_bytes().to_vec()),
        }];
        let request = Request::new(Asks::Named(asks.iter().map(|a| (*a).to_owned()).collect()));
        let how = How {
            plans,
            ..How::FENCES
        };
        run_with_how(Files::new(given), &request, how)
            .expect("an answer")
            .lines()
            .to_vec()
    }

    #[test]
    fn a_link_read_at_a_past_cut_gives_what_the_whole_list_gives() {
        // L §5.19 Links, E §5 step 1: each spend[m] is booked on its 25th, before the cut for
        // January to May, where its loose[m] reads `o.fulfils` of the rent's observations; the
        // named instances are whole, refused, a hole booking, a hole day and a window
        let asks = ["k", "f"];
        let narrow = lines(LINKS, &asks, Plans::Narrow);
        assert_eq!(
            narrow,
            lines(LINKS, &asks, Plans::Whole),
            "the twin's lines"
        );
        assert!(
            narrow.iter().all(|l| !l.contains("not implemented")),
            "every ask answered: {narrow:?}"
        );
    }

    #[test]
    fn a_flows_read_at_a_past_cut_counts_what_the_whole_list_counts() {
        // L §5.14.3, §8.2 (P20): each top[m] is booked on its 20th, before the cut for
        // January to May, where it reads the flows of the rent and of the fee in its month; the
        // instances outside the span bring their day's reads, a window's day is taken whole
        let asks = ["r", "t", "k"];
        let narrow = lines(SPANS, &asks, Plans::Narrow);
        assert_eq!(
            narrow,
            lines(SPANS, &asks, Plans::Whole),
            "the twin's lines"
        );
        assert!(
            narrow.iter().all(|l| !l.contains("not implemented")),
            "every ask answered: {narrow:?}"
        );
    }

    #[test]
    fn a_link_in_another_moves_list_goes_to_that_lists_first_reader() {
        // L §5.19 Links, §8.2 (group 6c's review, B1): zed[2026-02], booked on its 25th, reads
        // the flows of top in April, each top[m] those of rent in its month; an observation
        // known on 2026-01-21 names rent[2026-01], whose `when` reads the pinned ok[2026-01].
        // The whole list of top there gives that read to top[2026-03], its first reader of
        // rent at that cut, which the flows of April don't count: no `pinned:` line
        let text = r#"model pin
  tally 2
  on 2026-05-20
  start 2026-01-01
  end 2026-06-30

source s "invented values"

unit PLN: currency minor 0.01

index month = month 2026-01..2026-06

place konto: accounts PLN lives 2026-01-01..

place pot: accounts PLN lives 2026-01-01..

place landlord: outside lives 1900-01-01..

place shop: outside lives 1900-01-01..

input bank: observed

input ok[m in month]: bool

move rent[m in month]: transfers
  on day_of(m, 5)
  at start_of(m)
  when ok[m]
  leg cash from pot to landlord amount 100.00 PLN

move top[m in month]: transfers
  on day_of(m, 20)
  at start_of(m)
  leg cash from konto to shop amount flows(rent, m)

move zed[m in month]: transfers
  on day_of(m, 25)
  at start_of(m)
  leg cash from konto to shop amount flows(top, (m + 2) .. (m + 2))

def zz: money PLN = flows(zed, 2026-02 .. 2026-02)

ask zp: zz with ok[2026-01] = true

ask zi: inputs-of zz

version v
  cite s
  governs always
  known 2026-01-01
  ok[2026-01] = true "ok jan"
  ok[2026-02] = true "ok feb"
  ok[2026-03] = true "ok mar"
  ok[2026-04] = true "ok apr"
  ok[2026-05] = true "ok may"
  ok[2026-06] = true "ok jun"
  opening[konto] = 5000.00 PLN at 2026-01-01 start "s"
  opening[pot] = 9000.00 PLN at 2026-01-01 start "s"

version jan
  cite s
  governs 2026-01
  known 2026-01-21
  covers pot
  bank[a1] = (on: 2026-01-05, kind: transfers, leg cash from pot to landlord amount 100.00 PLN, fulfils: rent[2026-01]) "names jan"
"#;
        let asks = ["zp", "zi"];
        let narrow = lines(text, &asks, Plans::Narrow);
        assert_eq!(narrow, lines(text, &asks, Plans::Whole), "the twin's lines");
        assert!(
            narrow.iter().all(|l| !l.contains("pinned")),
            "no pin read: {narrow:?}"
        );
    }

    #[test]
    fn a_move_whose_cone_reaches_its_own_plan_is_read_whole() {
        // E §3.1 (the item graph): a plain move narrows at every cut, its own instance named
        // by an observation; one reading another plain move's instance narrows before the
        // first day a link to that move is known (L §5.19, §8.2; B1); one reading the
        // observations, and one reading another move that reads it back, don't (a plan can't
        // read a balance: load refuses it, LOAD-READS)
        let text = r#"model cone
  tally 2
  on 2026-03-20
  start 2026-01-01
  end 2026-03-31

source s "invented values"

unit PLN: currency minor 0.01

index month = month 2026-01..2026-03

place konto: accounts PLN lives 2026-01-01..

place world: outside lives 1900-01-01..

input bank: observed

input amt: money PLN

input from_m: month

move plain[m in month]: transfers
  on start_of(m)
  leg cash from world to konto amount amt

move by_field[m in month]: transfers
  on start_of(m)
  leg cash from world to konto amount plain[m].cash.amount

move by_obs[m in month]: transfers
  on start_of(m)
  leg cash from world to konto amount sum(o.cash.amount for o in bank over m)

move ping[m in month]: transfers
  on end_of(m)
  leg cash from world to konto amount flows(pong, from_m .. m)

move pong[m in month]: transfers
  on end_of(m)
  leg cash from world to konto amount flows(ping, from_m .. m)

ask q: bal(konto, 2026-03-31)

version v
  cite s
  governs always
  known 2026-01-01
  amt = 1.00 PLN "s"
  from_m = 2026-01 "s"
  opening[konto] = 10.00 PLN at 2026-01-01 start "s"

version feb
  cite s
  governs 2026-02
  known 2026-02-10
  bank[b1] = (on: 2026-02-01, kind: transfers, leg cash from world to konto amount 1.00 PLN, fulfils: plain[2026-02]) "s"
"#;
        let files = parse_reality(&[Source {
            path: "cone.tally".to_owned(),
            text: text.to_owned(),
        }])
        .expect("parses");
        let m = load(files).expect("loads");
        let narrows_of =
            |id: &str| narrows(&m, m.moves.get(id).expect("the move")).expect("judged");
        let linked = Day::from_ymd(2026, 2, 10).expect("a day");
        assert_eq!(narrows_of("plain"), Narrows::Always, "plain");
        assert_eq!(
            narrows_of("by_field"),
            Narrows::Before(linked),
            "another plain move's instance"
        );
        assert_eq!(
            narrows_of("by_obs"),
            Narrows::No,
            "a binding over observed rows"
        );
        assert_eq!(
            narrows_of("ping"),
            Narrows::No,
            "ping reads pong, which reads ping"
        );
        assert_eq!(
            narrows_of("pong"),
            Narrows::No,
            "pong reads ping, which reads pong"
        );
    }
}
