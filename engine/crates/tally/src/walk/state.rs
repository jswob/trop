//! A walk's place states: each place's log of entries in walk order, the state after each, and the register a balance read gathers.
//! Spec: E §4.2 (the day walk; balances are step functions, a change point per entry; holes in
//! the walk; a refused leg refuses its place from that point), §4.3 step 1 (a projection's walk
//! starts from the walk it continues, at a point), §6.1 (readings anchor a place); L §5.10 (a
//! place's balance: zero from its lifespan's start, else `HOLE-ANCHOR` until its first
//! reading), §8.1 (`HOLE-ANCHOR`'s questions), §8.2 (the register: what a balance rests on),
//! §6.4 (a walk is walked in one world: the places and lifespans it sees);
//! P §2.4 (monthly checkpoints: every place's state and the legs booked ahead, a state that
//! didn't change shared, not copied; D2-6).
//! Twin: engine/tools/refcore/walk.py (`St`, `Ent`, `Log`, `apply`, `Walk.log`, `default`,
//! `add`, `state`, `pstate`, `cum`, `unbook`, `st_value`, `anchor_question`).
//! Invariants: a log's entries are in walk order and each holds the state after it; a state's
//! value is known, or none before the place's first reading; a refusal, once met at a place,
//! holds from that point; the register of a balance is every entry's since the last reset; a
//! projection's walk starts each place as the walk it continues stood at its point; a main
//! walk's checkpoint at the first of a month holds each place's state before that day, the one
//! of the checkpoint before where it is the same, and the decision legs dated that day or later.
//! PERF: P8-14 (footprint: a checkpoint shares each place state that didn't change since the
//! one before, D2-6; the forks and the walk restart that start from checkpoints, P §2.5 and
//! §4.2, come with later rows, so at S U-6 is measured on its cost side only); plain twin:
//! state.rs (the log itself: a checkpoint's state is the log's state just before its day,
//! `Walk::state_before`); the equivalence test: this file's tests, under both day sources.

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use super::agenda::{Act, Mv};
use super::order::Run;
use super::plan::{Inst, LegVal, Order, invariant};
use super::point::{At, Step};
use crate::base::{Refusal, Warn};
use crate::eval::{Collected, Entry};
use crate::hole::{Hole, spread};
use crate::model::{Reality, World};
use crate::num::Num;
use crate::syntax::Stop;
use crate::time::Day;

/// The name a log takes for "every place" and "every unit" (refcore's `'*'`).
pub const ALL: &str = "*";

/// What an entry does to its place's state (refcore's entry kinds).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// An opening or an anchor: the balance is set to `val`, the holes since the last reset
    /// answered, `qs` the hole of a reading that is one (E §6.1).
    Reset {
        /// The balance stated.
        val: Num,
        /// The hole of a stated hole.
        qs: Option<Hole>,
    },
    /// A booked end of a leg: the balance moves by the quantity (E §4.2 step 2).
    Delta(Num),
    /// A hole movement's end: the balance is a hole from here to the next reading (E §4.2
    /// Holes in the walk).
    Hole(Hole),
    /// A hole no reading answers: a hole day, `HOLE-CHANGED` (E §4.2, E §5 step 5).
    Perm(Hole),
    /// A refusal that holds from here (E §4.2 step 5).
    Bad(Refusal),
    /// A register entry and nothing else: a modelled-past instance nothing fulfils (L §5.19).
    Mark,
}

/// A place's state after an entry (refcore's `St`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct St {
    /// The balance; none before the place's first reading (`HOLE-ANCHOR`).
    pub val: Option<Num>,
    /// The questions of the hole movements since the last reset.
    pub hole: Option<Hole>,
    /// The questions no reading answers.
    pub perm: Option<Hole>,
    /// The refusal that holds from a refused leg on.
    pub bad: Option<Refusal>,
    /// The position of the last reset in its log.
    pub rst: Option<usize>,
}

impl St {
    /// A state of balance `val` and nothing else. E §4.2.
    fn of(val: Option<Num>) -> St {
        St {
            val,
            hole: None,
            perm: None,
            bad: None,
            rst: None,
        }
    }

    /// Whether the state's balance is known: a value, no hole, no refusal (E §4.2 step 5:
    /// "Each warning is tested only where the balance is known").
    #[must_use]
    pub fn known(&self) -> Option<&Num> {
        if self.hole.is_some() || self.perm.is_some() || self.bad.is_some() {
            return None;
        }
        self.val.as_ref()
    }
}

/// One entry of a place's log (refcore's `Ent`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ent {
    /// Its point.
    pub at: At,
    /// What it does.
    pub change: Change,
    /// The movement it books, by id.
    pub mv: Option<String>,
    /// The leg it books, by name.
    pub leg: Option<String>,
    /// The register entries it makes (`unfulfilled:`).
    pub reg: Vec<Entry>,
    /// What its movement's evaluation met.
    pub coll: Option<Collected>,
    /// The state after it.
    pub st: St,
}

impl Ent {
    /// An entry at `at` doing `change`, nothing else known yet. E §4.2.
    #[must_use]
    pub fn new(at: At, change: Change) -> Ent {
        Ent {
            at,
            change,
            mv: None,
            leg: None,
            reg: Vec::new(),
            coll: None,
            st: St::of(None),
        }
    }
}

/// A place's log in one unit: its entries in walk order, and the register gathered through
/// each, filled as balances are read (refcore's `Log`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Log {
    /// The entries.
    pub ents: Vec<Ent>,
    /// The register since the last reset, through each entry.
    cum: Vec<Collected>,
}

/// Where the walk stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Progress {
    /// The plan pass hasn't finished.
    Planning,
    /// The days are being walked.
    Walking,
    /// Every day is walked, its close still to run (its warnings, its signature, E §9 item
    /// 3), so a read cut short past the counted depth runs it again on replay (R-16).
    Walked,
    /// Every day is walked and closed.
    Done,
    /// A refusal the walk met: every read of it is refused (L §8.4).
    Failed(Stop),
}

/// The legs a check of the sign rule tests together: per (place, unit), each movement's leg
/// (L §5.10 The sign rule; E §4.2 step 5).
pub type Group = BTreeMap<(String, String), Vec<(String, LegVal)>>;

/// The day being walked, kept so a walk that unwinds past the counted depth resumes where it
/// stopped (R-16).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Today {
    /// The day.
    pub day: Day,
    /// Its acts, in walk order.
    pub acts: Vec<(At, Act)>,
    /// The next act to take.
    pub next: usize,
    /// The next instance of a decision act to run.
    pub next_inst: usize,
    /// The runs of the decision act being taken, by their place in the walk's list.
    pub runs: Vec<usize>,
    /// The legs of the day's first batch, while it is open.
    pub batch: Option<Group>,
    /// The closing readings' legs, once the day's end is reached.
    pub closing: Option<Group>,
    /// The instant of each anchored reading (an opening included) that joined a group of the
    /// sign rule with no leg, by place and unit (L §5.10, tally 2.1.2).
    pub anchors: BTreeMap<(String, String), String>,
    /// The places the day touched.
    pub touched: BTreeSet<String>,
}

/// One walk of a timeline: the main walk from S to E (refcore's `Walk`). E §4.2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Walk {
    /// Its cut K.
    pub cut: Day,
    /// Its world: the moves, places and versions of the scenarios it sees (L §6.4).
    pub world: World,
    /// Its first day.
    pub start: Day,
    /// Its last day.
    pub end: Day,
    /// Each (place, unit)'s log.
    pub(crate) logs: BTreeMap<(String, String), Log>,
    /// The acts of each day not yet walked.
    pub(crate) agenda: BTreeMap<Day, Vec<(At, Act)>>,
    /// The last day walked.
    pub(crate) done: Option<Day>,
    /// Where it stands.
    pub(crate) progress: Progress,
    /// The day being walked.
    pub(crate) today: Option<Today>,
    /// The plan instances it books, by their place in its list.
    pub(crate) insts: Vec<Inst>,
    /// The decision instances due each (move, day).
    pub(crate) due: BTreeMap<(String, Day), Vec<Inst>>,
    /// The decision instances it ran.
    pub(crate) runs: Vec<Run>,
    /// A run's place in `runs`, by instance id.
    pub(crate) run_ids: BTreeMap<String, usize>,
    /// The running decision's live reads: the places it reads at its point (L §5.14.2).
    pub(crate) live: Option<BTreeSet<String>>,
    /// The instances left out, per move: how many, and the first day (L §8.2 `left out:`).
    pub(crate) left_out: BTreeMap<String, (u32, Option<Day>)>,
    /// The moves left out that touch each place.
    pub(crate) left_out_at: BTreeMap<String, BTreeSet<String>>,
    /// The adjustments with a remainder, per place, from their day (L §8.2 `adjustment:`).
    pub(crate) adj_regs: BTreeMap<String, Vec<(Day, Entry)>>,
    /// The legs the walk refused, by (movement, leg) (E §4.2 step 5).
    pub(crate) bad_legs: BTreeMap<(String, String), Refusal>,
    /// The warnings it met (L §8.5).
    pub(crate) warnings: Vec<(Warn, String)>,
    /// Whether each (place, unit) ended its last day below zero (`WARN-NEGATIVE`).
    pub(crate) negative: BTreeMap<(String, String), bool>,
    /// The day's booked flows per unit, summed over every place (E §9 item 1).
    pub(crate) conserved: BTreeMap<String, Num>,
    /// The day's expected change in the worth of the places that aren't outside, per unit:
    /// the legs across the boundary and those booked by their ends, the pads from outside,
    /// the openings and the anchors' gaps; none where a hole makes it unknown that day
    /// (E §9 item 2).
    pub(crate) worth: BTreeMap<String, Option<Num>>,
    /// Whether each place counts in the household's worth, once judged (E §9 item 2).
    pub(crate) inside: BTreeMap<String, bool>,
    /// The uncertain instances no observation fulfils (E §4.1 step 3).
    pub(crate) uncertain: Vec<Inst>,
    /// Each place's first reading day, for `HOLE-ANCHOR` (L §8.1).
    pub(crate) first_reading: BTreeMap<String, Day>,
    /// The places whose lifespan ends on each day, by its last day (E §9 item 6).
    pub(crate) endings: BTreeMap<Day, Vec<String>>,
    /// The running decision's legs, one group of the sign rule (L §5.10 point 2).
    pub(crate) group: Option<Group>,
    /// The movement orders of the walk's plan and decision instances, each once, sorted in
    /// the typed movement order: a movement's rank among the day's plans (refcore's
    /// `plan_key`; tally 2.1.6).
    pub(crate) orders: Vec<Order>,
    /// A projection's walk: what it continues and what it leaves out; none for a main walk.
    pub(crate) proj: Option<Box<Projected>>,
    /// The observations a main walk books, by their place in the list (E §4.2 step 1).
    pub(crate) obs: Vec<WalkObs>,
    /// The observed legs' texts, `<id> <leg>`, sorted by code point: a leg's rank among the
    /// day's observations (refcore's walk key).
    pub(crate) obs_texts: Vec<String>,
    /// The modelled-past instances, kept for the ledger (E §4.1 step 3, §6.2).
    pub(crate) modelled: Vec<Inst>,
    /// While a modelled-past decision runs, the legs of its own observations booked on their
    /// days before its own, which its running reads take back out (E §4.2 step 3).
    pub(crate) less_obs: Option<Vec<LegVal>>,
    /// A main walk's checkpoints, one at the start of each month it walked into, in day
    /// order (P §2.4).
    pub(crate) checkpoints: Vec<Checkpoint>,
    /// A scenario's main walk's movements before its `from` that read the scenario's facts,
    /// each a `before from:` text its reads list (E §9 item 3, ruling P43).
    pub(crate) listed: Vec<String>,
}

/// A main walk's state at the start of a month, kept for a walk to start from later: each
/// (place, unit)'s state there, shared with the checkpoint before where it didn't change, and
/// the decision legs booked ahead onto that day or later (refcore keeps none). P §2.4, D2-6;
/// E §4.2 step 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkpoint {
    /// Its day: the first of a month, the state before any of the day's acts.
    pub day: Day,
    /// The state of every (place, unit) with an entry before the day; one with none stands as
    /// the walk opened it.
    pub states: BTreeMap<(String, String), Rc<St>>,
    /// The legs decisions booked onto the day or later (N-6), in agenda order.
    pub ahead: Vec<(At, Act)>,
}

/// An observation a main walk books, judged at its cut: its record's refusal, and the day of
/// the decision instance it fulfils, at whose point it is booked that day (refcore's
/// `ObsMv.err`, `.target`). E §4.2 steps 1 and 3, L §5.19.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalkObs {
    /// The observation.
    pub obs: crate::book::Obs,
    /// Its record's refusal.
    pub err: Option<Refusal>,
    /// The day of the decision instance it fulfils.
    pub decision_day: Option<Day>,
}

/// What a projection's walk continues and what it leaves out (refcore's `Walk.base`,
/// `point`, `leftout`, `counted_leftout`). E §4.3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Projected {
    /// The walk it continues, by its place in the engine's list.
    pub base: usize,
    /// The point of that walk it starts after.
    pub point: At,
    /// The moves it leaves out: those that depend on the projection's holder (L §5.14.1).
    pub left_out: BTreeSet<String>,
    /// The days of the instances it leaves out, per move (L §8.2 `left out:`).
    pub counted: BTreeMap<String, Vec<Day>>,
    /// Each walk of the chain it continues, the nearest first: the state at the point the
    /// next walk starts from of every (place, unit) that walk logged.
    pub levels: Vec<BTreeMap<(String, String), St>>,
    /// The cut and the first day of the main walk at the chain's end.
    pub root: (Day, Day),
    /// The world the chain's walks are walked in (L §6.4).
    pub world: World,
}

impl Projected {
    /// A place's state at level `level` of the chain, with what holds for all its units and
    /// for the whole walk there (refcore's `base.pstate`, raw). The recursion is counted by
    /// the level, which grows to the chain's length: a chain names each holder at most once
    /// (E §4.3 Nesting is finite).
    fn state_at(&self, m: &Reality, place: &str, unit: &str, level: usize) -> St {
        let st = self.raw_at(m, place, unit, level);
        let any = self.raw_at(m, place, ALL, level);
        let all = self.raw_at(m, ALL, ALL, level);
        fold(st, any, all)
    }

    /// A place's own state at level `level`: the one that walk logged, or else the one it
    /// started from (refcore's `state`, then `default`). E §4.3 step 1.
    fn raw_at(&self, m: &Reality, place: &str, unit: &str, level: usize) -> St {
        let key = (place.to_owned(), unit.to_owned());
        if let Some(st) = self.levels.get(level).and_then(|states| states.get(&key)) {
            return st.clone();
        }
        match level.checked_add(1) {
            Some(next) if next < self.levels.len() => {
                let mut st = self.state_at(m, place, unit, next);
                st.rst = None;
                st
            }
            Some(_) | None => opening_state(m, place, unit, (self.root, self.world)),
        }
    }
}

impl Walk {
    /// A projection's walk of cut `cut` from `start` to `end`, continuing `projected` in its
    /// world, each place's first reading as `first_reading` gives it; nothing planned yet.
    /// E §4.3.
    #[must_use]
    pub fn projection(
        cut: Day,
        (start, end): (Day, Day),
        projected: Projected,
        first_reading: BTreeMap<String, Day>,
    ) -> Walk {
        let world = projected.world;
        Walk {
            proj: Some(Box::new(projected)),
            first_reading,
            ..Walk::new(cut, (start, end), world)
        }
    }

    /// A walk of cut `cut` from `start` to `end` in world `world`, nothing planned yet.
    /// E §4.2; L §6.4.
    #[must_use]
    pub fn new(cut: Day, (start, end): (Day, Day), world: World) -> Walk {
        Walk {
            cut,
            world,
            start,
            end,
            logs: BTreeMap::new(),
            agenda: BTreeMap::new(),
            done: None,
            progress: Progress::Planning,
            today: None,
            insts: Vec::new(),
            due: BTreeMap::new(),
            runs: Vec::new(),
            run_ids: BTreeMap::new(),
            live: None,
            left_out: BTreeMap::new(),
            left_out_at: BTreeMap::new(),
            adj_regs: BTreeMap::new(),
            bad_legs: BTreeMap::new(),
            warnings: Vec::new(),
            negative: BTreeMap::new(),
            conserved: BTreeMap::new(),
            worth: BTreeMap::new(),
            inside: BTreeMap::new(),
            uncertain: Vec::new(),
            first_reading: BTreeMap::new(),
            endings: BTreeMap::new(),
            group: None,
            orders: Vec::new(),
            proj: None,
            obs: Vec::new(),
            obs_texts: Vec::new(),
            modelled: Vec::new(),
            less_obs: None,
            checkpoints: Vec::new(),
            listed: Vec::new(),
        }
    }

    /// The rank of an observed leg's text among the walk's (refcore's walk key compares the
    /// text). E §4.2 step 1.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` past four billion observed legs.
    pub fn obs_rank(&self, text: &str) -> Result<u32, Stop> {
        let at = self.obs_texts.partition_point(|t| t.as_str() < text);
        u32::try_from(at).map_err(|_| invariant("an observed leg's rank"))
    }

    /// The rank of movement order `order` among the walk's, each term typed (refcore's walk
    /// key compares `plan_key`): twice its place plus one for one of them, twice the place it
    /// would take otherwise. Of several refusals at one place on one day, the first in this
    /// order is the one its reads print (tally 2.1.6). E §4.1, §4.2.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` past two billion movements.
    pub fn plan_rank(&self, order: &Order) -> Result<u32, Stop> {
        let at = self.orders.partition_point(|o| o < order);
        let found = self.orders.get(at).is_some_and(|o| o == order);
        let twice = u32::try_from(at)
            .map_err(|_| invariant("a movement's rank"))?
            .checked_mul(2)
            .ok_or_else(|| invariant("a movement's rank"))?;
        if !found {
            return Ok(twice);
        }
        twice
            .checked_add(1)
            .ok_or_else(|| invariant("a movement's rank"))
    }

    /// A place's state before its first entry: in a main walk, zero from its lifespan's start
    /// when that is on or after the walk's start, else unknown until a reading; in a
    /// projection's walk, the state of the walk it continues at the point it starts from
    /// (refcore's `default`). L §5.10, E §4.3 step 1.
    fn default_state(&self, m: &Reality, place: &str, unit: &str) -> St {
        match &self.proj {
            None => opening_state(m, place, unit, ((self.cut, self.start), self.world)),
            Some(p) => {
                let mut st = p.state_at(m, place, unit, 0);
                st.rst = None;
                st
            }
        }
    }

    /// Adds an entry to a place's log, its state the previous one's with its change applied
    /// (refcore's `add`, `apply`). E §4.2.
    ///
    /// # Errors
    ///
    /// A number error moving the balance.
    pub fn add(&mut self, m: &Reality, place: &str, unit: &str, mut ent: Ent) -> Result<(), Stop> {
        let key = (place.to_owned(), unit.to_owned());
        let prev = match self.logs.get(&key).and_then(|lg| lg.ents.last()) {
            Some(last) => last.st.clone(),
            None => self.default_state(m, place, unit),
        };
        let log = self.logs.entry(key).or_insert_with(Log::default);
        ent.st = apply(&prev, &ent.change, log.ents.len())?;
        log.ents.push(ent);
        if place != ALL
            && let Some(today) = &mut self.today
        {
            today.touched.insert(place.to_owned());
        }
        Ok(())
    }

    /// A place's state in one unit at point `at`, and the position of its last entry there
    /// (refcore's `state`). E §4.2 Running reads.
    #[must_use]
    pub fn state(&self, m: &Reality, place: &str, unit: &str, at: At) -> (St, Option<usize>) {
        let Some(log) = self.logs.get(&(place.to_owned(), unit.to_owned())) else {
            return (self.default_state(m, place, unit), None);
        };
        let n = log.ents.partition_point(|e| e.at <= at);
        let Some(i) = n.checked_sub(1) else {
            return (self.default_state(m, place, unit), None);
        };
        match log.ents.get(i) {
            Some(e) => (e.st.clone(), Some(i)),
            None => (self.default_state(m, place, unit), None),
        }
    }

    /// A place's state at `at` with what holds for all its units and for the whole walk: their
    /// holes and refusals (refcore's `pstate`, raw). E §4.2.
    #[must_use]
    pub fn pstate(&self, m: &Reality, place: &str, unit: &str, at: At) -> St {
        let (st, _) = self.state(m, place, unit, at);
        let (any, _) = self.state(m, place, ALL, at);
        let (all, _) = self.state(m, ALL, ALL, at);
        fold(st, any, all)
    }

    /// A place's state just before point `at`: every entry before it, none at it, with what
    /// holds for all its units and for the whole walk (refcore's `pstate` at the key less
    /// one, raw), the booked balance a reading at `at` is compared with. E §6.1, §6.3.
    #[must_use]
    pub fn pstate_before(&self, m: &Reality, place: &str, unit: &str, at: At) -> St {
        let st = self.state_before(m, place, unit, at);
        let any = self.state_before(m, place, ALL, at);
        let all = self.state_before(m, ALL, ALL, at);
        fold(st, any, all)
    }

    /// A place's state in one unit just before point `at` (refcore's `state` at the key less
    /// one). E §6.3.
    fn state_before(&self, m: &Reality, place: &str, unit: &str, at: At) -> St {
        let Some(log) = self.logs.get(&(place.to_owned(), unit.to_owned())) else {
            return self.default_state(m, place, unit);
        };
        let n = log.ents.partition_point(|e| e.at < at);
        match n.checked_sub(1).and_then(|i| log.ents.get(i)) {
            Some(e) => e.st.clone(),
            None => self.default_state(m, place, unit),
        }
    }

    /// The register of a place's log through entry `i`: every entry's since the last reset,
    /// the movement's evaluation and the entry's own (refcore's `cum`). L §8.2.
    pub fn cum(&mut self, place: &str, unit: &str, i: usize) -> Collected {
        let Some(log) = self.logs.get_mut(&(place.to_owned(), unit.to_owned())) else {
            return Collected::default();
        };
        while log.cum.len() <= i {
            let j = log.cum.len();
            let Some(e) = log.ents.get(j) else {
                break;
            };
            let mut c = Collected::default();
            let reset = matches!(e.change, Change::Reset { .. });
            if !reset
                && let Some(prev) = j.checked_sub(1).and_then(|p| log.cum.get(p))
                && e.st.rst.is_none_or(|r| r < j)
            {
                c.merge(prev);
            }
            if let Some(own) = &e.coll {
                c.merge(own);
            }
            c.entries.extend(e.reg.iter().cloned());
            log.cum.push(c);
        }
        match log.cum.get(i) {
            Some(c) => c.clone(),
            None => Collected::default(),
        }
    }

    /// Takes back a leg the sign rule refuses: its entries at `place` that `holds` matches are
    /// dropped, and the states after them made again (refcore's `unbook`); the flows dropped,
    /// summed. E §4.2 step 5.
    ///
    /// # Errors
    ///
    /// A number error making the states again.
    pub fn unbook(
        &mut self,
        m: &Reality,
        place: &str,
        unit: &str,
        movement: (&str, &str),
        holds: Taken,
    ) -> Result<Num, Stop> {
        let key = (place.to_owned(), unit.to_owned());
        let start = self.default_state(m, place, unit);
        let Some(log) = self.logs.get_mut(&key) else {
            return Ok(Num::zero());
        };
        let drop = |e: &Ent| {
            matches!(e.change, Change::Delta(_))
                && e.mv.as_deref() == Some(movement.0)
                && e.leg.as_deref() == Some(movement.1)
                && holds.holds(e.at)
        };
        let Some(first) = log.ents.iter().position(drop) else {
            return Ok(Num::zero());
        };
        let mut dropped = Num::zero();
        for e in log.ents.iter().filter(|e| drop(e)) {
            if let Change::Delta(f) = &e.change {
                dropped = dropped.checked_add(f).map_err(crate::eval::num_stop)?;
            }
        }
        let all = core::mem::take(&mut log.ents);
        log.ents = all.into_iter().filter(|e| !drop(e)).collect();
        log.cum.truncate(first);
        let mut prev = match first.checked_sub(1).and_then(|p| log.ents.get(p)) {
            Some(e) => e.st.clone(),
            None => start,
        };
        for (j, e) in log.ents.iter_mut().enumerate().skip(first) {
            e.st = apply(&prev, &e.change, j)?;
            prev = e.st.clone();
        }
        Ok(dropped)
    }

    /// Whether `place` counts in the household's worth: a thing whose base kind isn't
    /// outside (refcore's `inside`). E §9 item 2.
    pub fn inside(&mut self, m: &Reality, place: &str) -> bool {
        if let Some(hit) = self.inside.get(place) {
            return *hit;
        }
        let hit = match m.things.get(place) {
            None => false,
            Some(thing) => match &thing.base {
                None => true,
                Some(base) => !m.subtree("outside").contains(base),
            },
        };
        self.inside.insert(place.to_owned(), hit);
        hit
    }

    /// Adds `expected` to the day's expected change in worth in `unit`; none makes it unknown
    /// that day (refcore's `inv2_add`). E §9 item 2.
    ///
    /// # Errors
    ///
    /// A number error adding it.
    pub fn worth_add(&mut self, unit: &str, expected: Option<&Num>) -> Result<(), Stop> {
        let slot = self
            .worth
            .entry(unit.to_owned())
            .or_insert_with(|| Some(Num::zero()));
        *slot = match (slot.take(), expected) {
            (Some(sum), Some(e)) => Some(sum.checked_add(e).map_err(crate::eval::num_stop)?),
            (None, _) | (_, None) => None,
        };
        Ok(())
    }

    /// The units a place has logs in, but `*` (refcore's `units` of a place's logs). E §4.2.
    #[must_use]
    pub fn units_of(&self, place: &str) -> Vec<String> {
        self.logs
            .keys()
            .filter(|(p, u)| p == place && u != ALL)
            .map(|(_, u)| u.clone())
            .collect()
    }

    /// The entries of a place's log in one unit. E §4.2.
    #[must_use]
    pub fn entries(&self, place: &str, unit: &str) -> &[Ent] {
        self.logs
            .get(&(place.to_owned(), unit.to_owned()))
            .map_or(&[], |lg| lg.ents.as_slice())
    }

    /// `HOLE-ANCHOR`'s question for a place read before its first reading (refcore's
    /// `anchor_question`). L §8.1.
    #[must_use]
    pub fn anchor_question(&self, place: &str) -> Hole {
        match self.first_reading.get(place) {
            Some(d) => Hole::asked(
                &format!("{place} has no balance before {}", d.text()),
                place,
            ),
            None => Hole::asked(&format!("{place} has no opening or stated reading"), place),
        }
    }

    /// Keeps a main walk's checkpoint at the start of each month it walks into before it
    /// opens day `next`, or, for none, before it ends: every first of a month after the last
    /// day walked, through `next` or the walk's last day. Days the walk skips have no act, so
    /// a skipped month's checkpoint is the state the next day opens on. A projection's walk
    /// keeps none. P §2.4, §2.2; D2-6.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` past the calendar's last day.
    pub(crate) fn keep_checkpoints(&mut self, next: Option<Day>) -> Result<(), Stop> {
        if self.proj.is_some() {
            return Ok(());
        }
        let through = next.unwrap_or(self.end);
        let mut first = match self.done {
            Some(d) => month_after(d)?,
            None if self.start.day() == 1 => self.start,
            None => month_after(self.start)?,
        };
        while first <= through {
            let kept = self.checkpoint(first);
            self.checkpoints.push(kept);
            first = month_after(first)?;
        }
        Ok(())
    }

    /// The checkpoint at the start of day `day`, made before any act of it: each logged
    /// (place, unit)'s last state, the previous checkpoint's shared where it is the same, and
    /// the decision legs on the agenda from `day` on. P §2.4, D2-6; E §4.2 step 3.
    fn checkpoint(&self, day: Day) -> Checkpoint {
        let before = self.checkpoints.last();
        let mut states = BTreeMap::new();
        for (key, log) in &self.logs {
            let Some(last) = log.ents.last() else {
                continue;
            };
            let shared = before
                .and_then(|c| c.states.get(key))
                .filter(|st| ***st == last.st);
            let st = match shared {
                Some(st) => Rc::clone(st),
                None => Rc::new(last.st.clone()),
            };
            states.insert(key.clone(), st);
        }
        let mut ahead = Vec::new();
        for (_, acts) in self.agenda.range(day..) {
            for (at, act) in acts {
                if let Act::Leg(Mv::Run(_), _) = act {
                    ahead.push((*at, act.clone()));
                }
            }
        }
        Checkpoint { day, states, ahead }
    }

    /// A place's state in one unit at checkpoint `kept`: the one it holds, else the state the
    /// walk opened the place in. P §2.4.
    #[must_use]
    pub fn checkpoint_state(&self, m: &Reality, kept: &Checkpoint, place: &str, unit: &str) -> St {
        match kept.states.get(&(place.to_owned(), unit.to_owned())) {
            Some(st) => St::clone(st),
            None => self.default_state(m, place, unit),
        }
    }
}

/// The first day of the month after `day`'s; after 9999-12 the end of time, past every day
/// the walk reaches (L §7.6 item 1). P §2.4.
fn month_after(day: Day) -> Result<Day, Stop> {
    Day::from_ymd(day.year(), day.month(), 1)
        .and_then(|first| first.end_months(1))
        .ok_or_else(|| invariant("a checkpoint past the calendar"))
}

/// Which of a group's entries a refusal of the sign rule takes back (refcore's `match`).
/// E §4.2 step 5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Taken {
    /// The day's first batch: its readings at the start, observations and plans.
    Batch(Day),
    /// One decision's legs, at its point.
    Act(At),
    /// The closing readings of the day.
    Closing(Day),
}

impl Taken {
    /// Whether an entry at `at` belongs to the group (E §4.2 step 5).
    #[must_use]
    pub fn holds(self, at: At) -> bool {
        match self {
            Taken::Batch(d) => at.day == d && at.in_first_batch(),
            Taken::Act(point) => point.same_act(at),
            Taken::Closing(d) => at.day == d && at.step == Step::End,
        }
    }
}

/// A place's state in one unit with the holes and refusals of all its units and of the
/// whole walk folded in, its own refusal first (refcore's `pstate`, raw). E §4.2.
fn fold(mut st: St, any: St, all: St) -> St {
    for other in [any, all] {
        st.hole = spread(&[st.hole.as_ref(), other.hole.as_ref()]);
        st.perm = spread(&[st.perm.as_ref(), other.perm.as_ref()]);
        if st.bad.is_none() {
            st.bad = other.bad;
        }
    }
    st
}

/// A place's state where a main walk of cut and first day `root` starts in `world`: every
/// place's total zero, a unit's position unknown until booked, a place zero from its
/// lifespan's start when that is on or after the walk's first day, else unknown until a
/// reading (refcore's `default`). L §5.10, §6.4.
fn opening_state(
    m: &Reality,
    place: &str,
    unit: &str,
    ((cut, start), world): ((Day, Day), World),
) -> St {
    if place == ALL {
        return St::of(Some(Num::zero()));
    }
    if unit == ALL {
        return St::of(None);
    }
    let span = m
        .things
        .get(place)
        .and_then(|th| m.lifespan(th, (cut, world)));
    match span.and_then(|sp| sp.start()) {
        Some(s) if s >= start => St::of(Some(Num::zero())),
        Some(_) | None => St::of(None),
    }
}

/// The state after `change` (refcore's `apply`). E §4.2, §6.1.
fn apply(prev: &St, change: &Change, at: usize) -> Result<St, Stop> {
    let mut st = prev.clone();
    match change {
        Change::Reset { val, qs } => {
            st.val = Some(val.clone());
            st.hole.clone_from(qs);
            st.rst = Some(at);
        }
        Change::Delta(q) => {
            if let Some(v) = &st.val {
                st.val = Some(v.checked_add(q).map_err(crate::eval::num_stop)?);
            }
        }
        Change::Hole(h) => st.hole = spread(&[st.hole.as_ref(), Some(h)]),
        Change::Perm(h) => st.perm = spread(&[st.perm.as_ref(), Some(h)]),
        Change::Bad(r) => {
            if st.bad.is_none() {
                st.bad = Some(r.clone());
            }
        }
        Change::Mark => {}
    }
    Ok(st)
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::{Change, Checkpoint, St, Walk, apply};
    use crate::Days;
    use crate::api::Source;
    use crate::base::{Code, Refusal, Run};
    use crate::eval::Engine;
    use crate::hole::Hole;
    use crate::load::load;
    use crate::model::Reality;
    use crate::num::Num;
    use crate::syntax::parse_reality;
    use crate::time::Day;
    use crate::walk::{Act, At, Mv};

    /// A walk from mid-January to July: rent in January, a sweep decision on 10 February whose
    /// leg is dated 15 April, nothing in March, a wage in May, a place that never moves after
    /// its opening.
    const REALITY: &str = r#"model cp
  tally 2
  on 2025-06-01
  start 2025-01-15
  end 2025-07-10

source card "a card"

unit DUK: currency minor 0.01

kind ext: place DUK signed outside

place out: ext lives 2025-01-01..

place bank: accounts DUK lives 2025-01-01..

place save: accounts DUK lives 2025-01-01..

place idle: accounts DUK lives 2025-01-01..

input pay: money DUK

input rent_day: date

input wage_day: date

input sweep_day: date

input late_day: date

move rent: transfers
  on rent_day
  leg cash from bank to out amount pay

move wage: transfers
  on wage_day
  leg cash from out to bank amount pay

move sweep: transfers
  on sweep_day
  reads bank
  leg cash from bank to save amount min(pay, bal(bank)) on late_day

version facts
  cite card
  governs always
  known 2025-01-01
  pay = 100.00
  rent_day = 2025-01-20
  wage_day = 2025-05-05
  sweep_day = 2025-02-10
  late_day = 2025-04-15
  opening[bank] = 1000.00 at 2025-01-15 start
  opening[save] = 0.00 at 2025-01-15 start
  opening[idle] = 5.00 at 2025-01-15 start

ask bank_end: bal(bank, 2025-07-10)
"#;

    fn day(text: &str) -> Day {
        let (y, rest) = text.split_once('-').expect("a year");
        let (m, d) = rest.split_once('-').expect("a month");
        Day::from_ymd(
            y.parse().expect("a year"),
            m.parse().expect("a month"),
            d.parse().expect("a day"),
        )
        .expect("a day")
    }

    fn reality() -> Reality {
        let files = parse_reality(&[Source {
            path: "cp.tally".to_owned(),
            text: REALITY.to_owned(),
        }])
        .expect("parses");
        load(files).expect("loads")
    }

    /// The main walk at the model's cut, its days from `days`.
    fn walked(m: &Reality, days: Days) -> Walk {
        let mut engine = Engine::new(m.on).with_days(days);
        let id = engine.main_walk(m, m.on).expect("walks");
        engine.walk(id).expect("the walk").clone()
    }

    fn at<'a>(walk: &'a Walk, first: &str) -> &'a Checkpoint {
        walk.checkpoints
            .iter()
            .find(|c| c.day == day(first))
            .expect("a checkpoint")
    }

    fn shared(a: &Checkpoint, b: &Checkpoint, place: &str) -> bool {
        let key = (place.to_owned(), "DUK".to_owned());
        Rc::ptr_eq(
            a.states.get(&key).expect("a state"),
            b.states.get(&key).expect("a state"),
        )
    }

    #[test]
    fn a_checkpoint_is_kept_at_the_start_of_each_month_whatever_days_the_walk_steps() {
        let m = reality();
        let events = walked(&m, Days::Events);
        let every = walked(&m, Days::Every);
        let days: Vec<String> = events.checkpoints.iter().map(|c| c.day.text()).collect();
        assert_eq!(
            days,
            [
                "2025-02-01",
                "2025-03-01",
                "2025-04-01",
                "2025-05-01",
                "2025-06-01",
                "2025-07-01"
            ],
            "every first of a month after S through E (P §2.4)"
        );
        assert_eq!(
            events.checkpoints, every.checkpoints,
            "the event walk keeps the every-day walk's checkpoints (P §2.2)"
        );
    }

    #[test]
    fn a_checkpoint_holds_the_logs_state_before_its_day() {
        let m = reality();
        let walk = walked(&m, Days::Events);
        for kept in &walk.checkpoints {
            for (place, unit) in walk.logs.keys() {
                assert_eq!(
                    walk.checkpoint_state(&m, kept, place, unit),
                    walk.state_before(&m, place, unit, At::before(kept.day)),
                    "{place} {unit} at {} (P §2.4: the plain twin is the log)",
                    kept.day.text()
                );
            }
        }
    }

    #[test]
    fn a_place_that_did_not_move_is_shared_and_the_legs_booked_ahead_are_kept() {
        let m = reality();
        let walk = walked(&m, Days::Events);
        let (feb, mar, apr, may, jun) = (
            at(&walk, "2025-02-01"),
            at(&walk, "2025-03-01"),
            at(&walk, "2025-04-01"),
            at(&walk, "2025-05-01"),
            at(&walk, "2025-06-01"),
        );
        assert!(
            shared(feb, jun, "idle"),
            "idle never moves after its opening"
        );
        assert!(
            shared(feb, apr, "bank"),
            "bank doesn't move from February to April"
        );
        assert!(
            !shared(apr, may, "bank"),
            "the sweep's leg moves bank on 15 April"
        );
        assert!(!shared(may, jun, "bank"), "the wage moves bank in May");
        let ahead = |c: &Checkpoint| -> Vec<String> {
            c.ahead
                .iter()
                .map(|(point, act)| {
                    assert!(matches!(act, Act::Leg(Mv::Run(_), _)), "a decision's leg");
                    point.day.text()
                })
                .collect()
        };
        assert!(ahead(feb).is_empty(), "the sweep hasn't run on 1 February");
        assert_eq!(
            ahead(mar),
            ["2025-04-15"],
            "the sweep's leg, booked ahead (N-6)"
        );
        assert_eq!(ahead(apr), ["2025-04-15"], "still ahead on 1 April");
        assert!(ahead(may).is_empty(), "booked on 15 April");
    }

    #[test]
    fn a_movement_ranks_twice_its_place_in_the_typed_order_plus_one() {
        // E §4.1 (tally 2.1.6): rent (20 January), the sweep decision (10 February) and the
        // wage (5 May), each at twice its place among the walk's movement orders, plus one
        let m = reality();
        let walk = walked(&m, Days::Events);
        let ranks: Vec<u32> = walk
            .orders
            .iter()
            .map(|order| walk.plan_rank(order).expect("a rank"))
            .collect();
        assert_eq!(ranks, [1, 3, 5], "the plans' and the decision's ranks");
    }

    #[test]
    fn a_state_moves_by_its_entries_and_a_reset_answers_the_holes() {
        let start = St::of(Some(Num::int(5)));
        let moved = apply(&start, &Change::Delta(Num::int(-7)), 0).expect("moves");
        assert_eq!(moved.known(), Some(&Num::int(-2)), "5 - 7");
        let holed = apply(&moved, &Change::Hole(Hole::asked("q", "x")), 1).expect("a hole");
        assert_eq!(holed.known(), None, "a hole");
        let reset = apply(
            &holed,
            &Change::Reset {
                val: Num::int(3),
                qs: None,
            },
            2,
        )
        .expect("a reset");
        assert_eq!(reset.known(), Some(&Num::int(3)), "answered");
        assert_eq!(reset.rst, Some(2), "the reset's place");
        let perm = apply(&reset, &Change::Perm(Hole::asked("p", "x")), 3).expect("a hole");
        let after = apply(
            &perm,
            &Change::Reset {
                val: Num::int(1),
                qs: None,
            },
            4,
        )
        .expect("a reset");
        assert_eq!(after.known(), None, "a hole no reading answers");
        let bad = Refusal::new(Code::Run(Run::Place), "x".to_owned());
        let refused = apply(&after, &Change::Bad(bad.clone()), 5).expect("refused");
        let other = Refusal::new(Code::Run(Run::Lifespan), "y".to_owned());
        let again = apply(&refused, &Change::Bad(other), 6).expect("refused");
        assert_eq!(again.bad, Some(bad), "the first refusal holds");
    }
}
