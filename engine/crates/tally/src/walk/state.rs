//! A walk's place states: each place's log of entries in walk order, the state after each, and the register a balance read gathers.
//! Spec: E §4.2 (the day walk; balances are step functions, a change point per entry; holes in
//! the walk; a refused leg refuses its place from that point), §4.3 step 1 (a projection's walk
//! starts from the walk it continues, at a point), §6.1 (readings anchor a place); L §5.10 (a
//! place's balance: zero from its lifespan's start, else `HOLE-ANCHOR` until its first
//! reading), §8.1 (`HOLE-ANCHOR`'s questions), §8.2 (the register: what a balance rests on),
//! §6.4 (a walk is walked in one world: the places and lifespans it sees);
//! P §2.4 (monthly checkpoints: every place's state, a state that didn't change shared, not
//! copied; D2-6; tally 2.2 books no leg ahead, so a checkpoint holds none).
//! Twin: engine/tools/refcore/walk.py (`St` with its `pbad`, `Ent`, `Log`, `apply`, `Walk.log`,
//! `default`, `add`, `state`, `pstate`, `cum`, `unbook`, `st_value`, `anchor_question`).
//! Invariants: a log's entries are in walk order and each holds the state after it; a state's
//! value is known, or none before the place's first reading; a refusal, once met at a place,
//! holds from that point to the place's next reading, and past every reading where the
//! movement's day is a hole or refused (L §8, tally 2.1.8); the register of a balance is every entry's since the last reset; a
//! projection's walk starts each place as the walk it continues stood at its point; a main
//! walk's checkpoint at the first of a month holds each place's state before that day, the one
//! of the checkpoint before where it is the same.
//! PERF: P8-14 (footprint: a checkpoint shares each place state that didn't change since the
//! one before, D2-6; the forks and the walk restart that start from checkpoints, P §2.5 and
//! §4.2, come with later rows, so at S U-6 is measured on its cost side only); plain twin:
//! state.rs (the log itself: a checkpoint's state is the log's state just before its day,
//! `Walk::state_before`); the equivalence test: this file's tests, under both day sources.

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use super::agenda::Act;
use super::instance::{Amount, Dated, DayWindow, HoleSrc, Inst, LegVal, Order};
use super::order::{CycleItem, Run};
use super::plan::invariant;
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

/// A leg of a modelled-past decision's own observation booked before its day, with the ends
/// its row booked: of two rows stated `same`, each books the end its statement covers, so an
/// end another version's row books isn't taken back out (refcore's `own_obs`). L §5.19
/// Booking an observation, E §4.2 step 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OwnLeg {
    /// The leg.
    pub(crate) leg: LegVal,
    /// Whether its row booked its `from` end.
    pub(crate) from: bool,
    /// Whether its row booked its `to` end.
    pub(crate) to: bool,
}

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
    /// A decision dated by a window has fallen at the window's last day: its hole no reading
    /// answered is taken back, as each unit's hole a reading ends takes its place (L §5.19 Holes,
    /// §4.4; pick P29 on #175; refcore's `unperm`).
    Unperm(Hole),
    /// A refusal that holds from here to the place's next reading: a dated refused movement's,
    /// or the sign rule's (E §4.2 step 5; L §8, tally 2.1.8).
    Bad(Refusal),
    /// A refusal no reading ends: a refused movement whose day is a hole or refused, which may
    /// fall after any reading (L §8, §5.19; tally 2.1.8; refcore's `pbad`).
    PermBad(Refusal),
    /// A movement dated by a window may fall from here: the balance is `HOLE-IMPRECISE`
    /// until its leg is booked at the window's last day; the window is the entry's (L §4.4;
    /// refcore's `imp_on`).
    ImpOn(Hole),
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
    /// The refusal that holds from a refused leg on, to the place's next reading.
    pub bad: Option<Refusal>,
    /// The refusal no reading ends (refcore's `pbad`): kept as `bad` at a reset.
    pub perm_bad: Option<Refusal>,
    /// The movements dated by a window that may still fall (L §4.4; refcore's `imp`).
    pub imp: Vec<Pending>,
    /// The position of the last reset in its log.
    pub rst: Option<usize>,
    /// After a reading a window runs past, the windows' questions it keeps open from its next
    /// instant: at the reading's own instant the stated balance stands without them (refcore's
    /// `reopen`; L §5.19 l.1726, E §4.1 step 2; P32, N1 and N8). None after any other entry.
    pub reopen: Option<Hole>,
    /// The movements dated by a window a reading met before they fell, not yet booked: each
    /// reading after it reopens them too, every reading inside a window an edge (P32 said
    /// whole, P41, F2). L §5.19 l.1726.
    pub held: Vec<Pending>,
}

/// A movement dated by a window that may still fall at a place (refcore's `imp`, one entry;
/// L §4.4; tally 2.1.8 item 77).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pending {
    /// Its window's id, `<movement>.<leg>`.
    pub wid: String,
    /// The question it puts.
    pub hole: Hole,
    /// Its leg's amount signed as a flow into the place, where known.
    pub signed: Option<Num>,
}

/// What a read of a leg the walk took out meets: its refusal, or the hole a test some of a
/// window's days pass leaves it (refcore's `bad_legs`, `hole_legs`, `hole_flows`; E §4.2
/// step 5; tally 2.1.8 item 77).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LegOut {
    /// The leg is refused.
    Refused(Refusal),
    /// The leg is the window's hole.
    Hole(Hole),
}

impl St {
    /// Every question the state's balance rests on: its holes, those no reading answers,
    /// the windows that may still fall, and `missing`, a balance not known yet (refcore's
    /// `st_value`). L §8.1, §4.4.
    #[must_use]
    pub fn questions(&self, missing: Option<&Hole>) -> Option<Hole> {
        let mut all = vec![self.hole.as_ref(), self.perm.as_ref()];
        all.extend(self.imp.iter().map(|p| Some(&p.hole)));
        all.push(missing);
        spread(&all)
    }

    /// A state of balance `val` and nothing else. E §4.2.
    fn of(val: Option<Num>) -> St {
        St {
            val,
            hole: None,
            perm: None,
            bad: None,
            perm_bad: None,
            imp: Vec::new(),
            rst: None,
            reopen: None,
            held: Vec::new(),
        }
    }

    /// Whether the state's balance is known: a value, no hole, no refusal (E §4.2 step 5:
    /// "Each warning is tested only where the balance is known").
    #[must_use]
    pub fn known(&self) -> Option<&Num> {
        if self.hole.is_some() || self.perm.is_some() || self.bad.is_some() || !self.imp.is_empty()
        {
            return None;
        }
        self.val.as_ref()
    }

    /// The least and the greatest balance the legs dated by a window that may still fall here
    /// allow, each booked or not; none where the balance isn't known but for them, or one's
    /// amount isn't (E §4.1 step 2, §4.2 step 5; tally 2.1.8 item 77).
    ///
    /// # Errors
    ///
    /// A number past the cap.
    pub fn imp_range(&self) -> Result<Option<(Num, Num)>, Stop> {
        if self.imp.is_empty() || self.hole.is_some() || self.perm.is_some() || self.bad.is_some() {
            return Ok(None);
        }
        let Some(val) = &self.val else {
            return Ok(None);
        };
        let (mut lo, mut hi) = (val.clone(), val.clone());
        for pending in &self.imp {
            let Some(a) = &pending.signed else {
                return Ok(None);
            };
            if a.is_negative() {
                lo = lo.checked_add(a).map_err(crate::eval::num_stop)?;
            } else {
                hi = hi.checked_add(a).map_err(crate::eval::num_stop)?;
            }
        }
        Ok(Some((lo, hi)))
    }

    /// The hole of the windows that may still fall here: what their days leave open
    /// (L §4.4; tally 2.1.8 item 77).
    #[must_use]
    pub fn imp_hole(&self) -> Option<Hole> {
        let all: Vec<Option<&Hole>> = self.imp.iter().map(|p| Some(&p.hole)).collect();
        spread(&all)
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
    /// For a leg dated by a window, the window's id (`<movement>.<leg>`) and the window: an
    /// `ImpOn` opens its imprecision, the leg's booking ends it (refcore's `wid`, `win`).
    pub window: Option<(String, DayWindow)>,
    /// What its movement's evaluation met.
    pub coll: Option<Collected>,
    /// A pad's flow at its reading's own place, where the reset books it: the gap, a number
    /// or a hole, which `flows` of the place counts (E §6.1; L §5.19).
    pub flow: Option<Amount>,
    /// A hole leg's direction at the place and where its amount was read, which a `partial`
    /// line reads again (refcore's `sgn`, `leg`; L §5.18).
    pub hole_leg: Option<HoleLeg>,
    /// An `ImpOn`'s leg amount signed as a flow into the place, where known (tally 2.1.8
    /// item 77).
    pub signed: Option<Num>,
    /// A refusal a plan's or an observation's own evaluation gives: it reads no value under a
    /// window's hole, so every day gives it and it outranks one (refcore's `note == 'own'`;
    /// L §8 l.2217, §4.4 l.671; P31, B4).
    pub own: bool,
    /// The state after it.
    pub st: St,
}

/// A hole leg as a `partial` line reads a balance through it: into the place or out of it, and
/// where its amount was read, none where nothing read it (refcore's `Ent.sgn`, `Leg.src`).
/// L §5.18 Partial answers, §5.19.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoleLeg {
    /// Whether it enters the place (`+`) or leaves it (`-`).
    pub into: bool,
    /// Where its amount was read.
    pub src: Option<Box<HoleSrc>>,
}

/// What a `partial` line reads of a balance that is a hole: the known part booked since the
/// last reset, each hole leg's direction and source since, and whether those legs' questions
/// are all the balance's own (refcore's `m_partial_bal`, its walk side). L §5.18, §5.19.
pub(crate) struct PartialLegs {
    /// The known part.
    pub known: Num,
    /// Each hole leg's direction and where its amount was read.
    pub legs: Vec<(bool, HoleSrc)>,
    /// Whether the legs' questions, with the start's, are the balance's holes, every one.
    pub all_met: bool,
    /// A projection's start hole where no reading reset the place since: the walk it continues
    /// holds its terms at the point (E §4.3; B9 of group 3's review).
    pub start: Option<Hole>,
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
            window: None,
            coll: None,
            flow: None,
            hole_leg: None,
            signed: None,
            own: false,
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
    /// A refusal the walk met, with the day it was walking: every read of it that reaches
    /// that day is refused, and a read that ends before it is read as the walk stood there;
    /// none for a refusal of its plan pass or its close, which refuses every read (L §8.4,
    /// E §4.3; pick P54, #127).
    Failed(Stop, Option<Day>),
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
    /// The order of an unordered cycle's instances due that day, once judged, so a day cut
    /// short past the counted depth runs the rest of it as judged (L §5.14.2).
    pub cycle: Option<Vec<CycleItem>>,
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
    /// The running decision's reads: every place it reads, at its point or before it, which
    /// `explain` prints (E §10).
    pub(crate) reads: Option<BTreeSet<String>>,
    /// The moves of an unordered cycle whose instances due each day ran together, by (move,
    /// day): the cycle's other moves' acts that day run nothing more (L §5.14.2).
    pub(crate) cycles_run: BTreeSet<(String, Day)>,
    /// The instances left out, per move: how many, and the first day, a hole until a dated
    /// one comes (L §8.2 `left out:`, §2.4 Values).
    pub(crate) left_out: BTreeMap<String, (u32, Dated)>,
    /// The moves left out that touch each place.
    pub(crate) left_out_at: BTreeMap<String, BTreeSet<String>>,
    /// The adjustments with a remainder, per place, from their day (L §8.2 `adjustment:`).
    pub(crate) adj_regs: BTreeMap<String, Vec<(Day, Entry)>>,
    /// The movements that don't count where a read of a place reaches, by each place their
    /// legs name: the day a leg is on, none where no day of the walk holds it (left out, past
    /// a projection's plans, a false `when`), and what decided it ([`Collected::deciding`]).
    /// A read of the place over a span that leaves the day out lists them: a balance, `flows`
    /// (L §8.2 One read set, "a day outside the span"; P35 on #241); one that doesn't exist
    /// from its day, a read whose prefix reaches the day (S4, P43 on #241).
    pub(crate) uncounted: BTreeMap<String, Vec<(Noted, Collected)>>,
    /// The legs the walk refused, by (movement, leg) (E §4.2 step 5).
    pub(crate) bad_legs: BTreeMap<(String, String), Refusal>,
    /// The legs a test some of a window's days pass left the window's hole, by (movement,
    /// leg), which a read of the leg meets (refcore's `hole_legs`, `hole_flows`; tally 2.1.8
    /// item 77).
    pub(crate) hole_legs: BTreeMap<(String, String), Hole>,
    /// The window-dated legs whose first day's sign test waits for their point's group, by
    /// (place, unit): the legs that group's refusal would reach (refcore's `pend_at`; tally
    /// 2.1.8 item 77).
    pub(crate) pend_at: BTreeMap<(String, String), Vec<(String, LegVal)>>,
    /// The questions of the holes a test of a window-dated movement left where some of the
    /// window's days pass it and some don't: a refusal reaching a place after one doesn't
    /// outrank it, which comes first being what the window leaves open (refcore's
    /// `UNCERTAIN`; L §4.4; tally 2.1.8 item 77).
    pub(crate) open_tests: Option<Hole>,
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
    pub(crate) less_obs: Option<Vec<OwnLeg>>,
    /// A main walk's checkpoints, one at the start of each month it walked into, in day
    /// order (P §2.4).
    pub(crate) checkpoints: Vec<Checkpoint>,
    /// The gaps of the `stated` readings it applied, per (place, day), in the order applied
    /// (L §5.19 `gap(x, d)`, E §6.1).
    pub(crate) gaps: BTreeMap<(String, Day), Vec<GapAt>>,
    /// The ends stated `same` as a decision not yet run, by the decision's id: booked at its
    /// point (refcore's `same_waiting`; L §6.1).
    pub(crate) same_waiting: BTreeMap<String, Vec<crate::book::Waiting>>,
    /// The pads this walk booked or refused, each at its point, in the order it met them
    /// (refcore's `pads`; E §6.1): the members of a binding over pads.
    pub(crate) pads: Vec<(crate::book::Pad, At)>,
    /// A walk up to a day: that day, past which its plan pass listed no plan's instance
    /// (refcore's prefix walk; E §3.2, §4.1); none for a main walk or a projection's.
    pub(crate) upto: Option<Day>,
    /// With forks on, a projection's walk's plan acts of each day it walked, in walk order:
    /// what a later read's plan is compared with before it continues the walk (P §2.5,
    /// `fork.rs`); empty otherwise.
    pub(crate) walked: BTreeMap<Day, Vec<(At, Act)>>,
    /// The lengths of `uncertain` and `modelled` when its plan pass ended: the plan's part of
    /// each, which the walk then adds to (P §2.5, `fork.rs`).
    pub(crate) plan_lists: (usize, usize),
}

/// A stated reading's gap, as the walk met it: the reading less the booked balance at its
/// instant, and what that rests on (refcore's `Walk.gaps`). L §5.19, E §6.1, L §8.2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GapAt {
    /// The reading's point.
    pub at: At,
    /// The gap: a number, a hole, or the refusal the booked balance is.
    pub gap: Amount,
    /// The reading's unit.
    pub unit: String,
    /// The reading's instant as text (refcore's `instant_text`).
    pub instant: String,
    /// The register of the booked balance and the reading's row.
    pub coll: Collected,
}

/// A main walk's state at the start of a month, kept for a walk to start from later: each
/// (place, unit)'s state there, shared with the checkpoint before where it didn't change
/// (refcore keeps none). P §2.4, D2-6.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkpoint {
    /// Its day: the first of a month, the state before any of the day's acts.
    pub day: Day,
    /// The state of every (place, unit) with an entry before the day; one with none stands as
    /// the walk opened it.
    pub states: BTreeMap<(String, String), Rc<St>>,
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
    /// The last day its read walks it through, d: it counts the movements dated t through d
    /// and plans none after (E §4.3 step 2; P §2.3, one walk of its window).
    pub through: Day,
    /// The reads that hold it now; the walk is freed when the last of them ends (P §3.5).
    pub readers: u32,
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
    /// The last day the walk plans a movement on: a projection's read's last day, else its
    /// end (E §4.3 step 2, §4.1).
    #[must_use]
    pub fn plans_through(&self) -> Day {
        self.proj
            .as_ref()
            .map_or(self.end, |p| p.through.min(self.end))
    }

    /// The walk freed when its scope ends, a projection's read: it keeps nothing, and a read
    /// of it is refused as the invariant it breaks (P §3.5, D1-6; E §4.3).
    #[must_use]
    pub fn freed(&self) -> Walk {
        Walk {
            progress: Progress::Failed(
                invariant("a projection's walk read after its read ended"),
                None,
            ),
            ..Walk::new(self.cut, (self.start, self.end), self.world)
        }
    }

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
            reads: None,
            cycles_run: BTreeSet::new(),
            left_out: BTreeMap::new(),
            left_out_at: BTreeMap::new(),
            adj_regs: BTreeMap::new(),
            uncounted: BTreeMap::new(),
            bad_legs: BTreeMap::new(),
            hole_legs: BTreeMap::new(),
            pend_at: BTreeMap::new(),
            open_tests: None,
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
            gaps: BTreeMap::new(),
            same_waiting: BTreeMap::new(),
            pads: Vec::new(),
            upto: None,
            walked: BTreeMap::new(),
            plan_lists: (0, 0),
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
    pub(crate) fn default_state(&self, m: &Reality, place: &str, unit: &str) -> St {
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
        ent.st = apply_ent(&prev, &ent, (log.ents.len(), self.open_tests.as_ref()))?;
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
            Some(e) => (at_reading(e, at), Some(i)),
            None => (self.default_state(m, place, unit), None),
        }
    }

    /// What a `partial` line reads of a place's balance at `at` in one unit: its known part
    /// and the hole legs booked since its last reset; none where the balance is a hole of
    /// another making (an anchor, a refusal, a window, a question no reading answers, a hole
    /// the place or the walk holds for every unit, as a decision's hole day books it, a hole
    /// with no leg's amount to read again): the balance read is the folded state `pstate`
    /// (refcore's `m_partial_bal`, its walk side). L §5.18 Partial answers, §5.19.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a log position past the integers.
    pub(crate) fn partial_legs(
        &self,
        m: &Reality,
        place: &str,
        unit: &str,
        at: At,
    ) -> Result<Option<PartialLegs>, Stop> {
        let Some(log) = self.logs.get(&(place.to_owned(), unit.to_owned())) else {
            return Ok(None);
        };
        let (st, Some(i)) = self.state(m, place, unit, at) else {
            return Ok(None);
        };
        let read = self.pstate(m, place, unit, at);
        if read.bad.is_some() || read.perm.is_some() || !read.imp.is_empty() {
            return Ok(None);
        }
        let Some(known) = st.val else {
            return Ok(None);
        };
        // a main walk's (its one reader is the plain read at the cut, `bal_day`): a place it
        // hasn't reset opens with no hole (`opening_state`), so its log holds every hole leg
        let from = match st.rst {
            Some(r) => next_entry(r)?,
            None => 0,
        };
        let mut legs = Vec::new();
        let mut seen: Vec<Option<&Hole>> = Vec::new();
        // a projection's walk starts from its base's balance at the point: its hole terms are
        // the base's there (E §4.3; L §5.18, R15; B9 of group 3's review)
        let start = if st.rst.is_none() && self.proj.is_some() {
            self.default_state(m, place, unit).hole
        } else {
            None
        };
        seen.push(start.as_ref());
        for e in log.ents.iter().take(next_entry(i)?).skip(from) {
            let Change::Hole(h) = &e.change else {
                continue;
            };
            let Some(HoleLeg {
                into,
                src: Some(src),
            }) = e.hole_leg.as_ref()
            else {
                return Ok(None);
            };
            legs.push((*into, HoleSrc::clone(src)));
            seen.push(Some(h));
        }
        let all_met = match (spread(&seen), &read.hole) {
            (Some(legs_holes), Some(own)) => legs_holes.same_questions(own),
            (None, None) => true,
            (Some(_), None) | (None, Some(_)) => false,
        };
        Ok(Some(PartialLegs {
            known,
            legs,
            all_met,
            start,
        }))
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

    /// What a pad's reset at `at` keeps beside its reading's reads: each open instance's
    /// since the place's last reset, in its unit and in every unit, with its `open:` line, for
    /// the gap a movement left unbooked is the pad's size (refcore's `open_kept`; L §8.2 One
    /// read set, the `open:` row; 220.2 on #241). E §6.1.
    pub(crate) fn open_kept(
        &self,
        m: &Reality,
        (place, unit): (&str, &str),
        at: At,
        into: &mut Collected,
    ) {
        for u in [unit, ALL] {
            let Some(log) = self.logs.get(&(place.to_owned(), u.to_owned())) else {
                continue;
            };
            let (st, _) = self.state(m, place, u, at);
            for (j, e) in log.ents.iter().enumerate().take_while(|(_, e)| e.at <= at) {
                let before_reset = st.rst.is_some_and(|r| j < r);
                let open = e.reg.iter().any(|r| r.word == "open");
                if before_reset || !matches!(e.change, Change::Mark) || !open {
                    continue;
                }
                if let Some(c) = &e.coll {
                    into.merge(c);
                }
                into.entries.extend(e.reg.iter().cloned());
            }
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

    /// The pins leg `movement` (its movement and leg names) took where it is booked at
    /// `places` in `unit`, from its entries' reads: what a refusal of it rests on (L §8.2 One
    /// read set; tally 2.2.1's 208.2); none where it took no pin.
    #[must_use]
    pub fn booked_pins(
        &self,
        places: Vec<&str>,
        unit: &str,
        movement: (&str, &str),
    ) -> Option<Collected> {
        let mut read = Collected::default();
        for place in places {
            let Some(log) = self.logs.get(&(place.to_owned(), unit.to_owned())) else {
                continue;
            };
            for e in &log.ents {
                let booked = matches!(e.change, Change::Delta(_))
                    && e.mv.as_deref() == Some(movement.0)
                    && e.leg.as_deref() == Some(movement.1);
                if booked && let Some(c) = &e.coll {
                    read.merge(c);
                }
            }
        }
        super::ends::pins_read(&read)
    }

    /// What the walk's refusals of movement `movement` rest on: its `bad` entries' reads, at
    /// every place (L §8.2 One read set, a refusal's like a value's; B1 on #241).
    #[must_use]
    pub(crate) fn refused_reads(&self, movement: &str) -> Collected {
        let mut out = Collected::default();
        for log in self.logs.values() {
            for e in &log.ents {
                let refused =
                    matches!(e.change, Change::Bad(_)) && e.mv.as_deref() == Some(movement);
                if refused && let Some(c) = &e.coll {
                    out.merge(c);
                }
            }
        }
        out
    }

    /// Notes a movement that doesn't count at `places` on `day` (none: on no day of the walk),
    /// with `read`'s deciding entries, none kept where it has none (P35 on #241). L §8.2.
    pub(crate) fn note_uncounted(&mut self, places: &[&str], day: Option<Day>, read: &Collected) {
        self.note(places, Noted::Out(day), read);
    }

    /// Notes a movement that doesn't exist (a false `when` or key `where`) at `places` from
    /// `day`, with `read`'s deciding entries: a read of day t reads its prefix through t, so a
    /// read whose last day is before `day` never met them (E §4.2; S4, P43 on #241). L §8.2.
    pub(crate) fn note_decided(&mut self, places: &[&str], day: Day, read: &Collected) {
        self.note(places, Noted::From(day), read);
    }

    /// Keeps `read`'s deciding entries at `places` as `noted` says they join a read, none kept
    /// where it has none. L §8.2.
    fn note(&mut self, places: &[&str], noted: Noted, read: &Collected) {
        let deciding = read.deciding();
        if deciding.entries.is_empty() {
            return;
        }
        for p in places {
            self.uncounted
                .entry((*p).to_owned())
                .or_insert_with(Vec::new)
                .push((noted, deciding.clone()));
        }
    }

    /// What decided that a movement at `place` doesn't count over the days `first` to `last`:
    /// each one noted on a day outside them or on none, and each that doesn't exist from a day
    /// on or before `last` (P35, S4 on #241). L §8.2 One read set, E §4.2.
    #[must_use]
    pub(crate) fn uncounted_in(&self, place: &str, (first, last): (Day, Day)) -> Collected {
        let mut out = Collected::default();
        for (noted, read) in self.uncounted.get(place).into_iter().flatten() {
            let met = match noted {
                Noted::Out(day) => day.is_none_or(|d| d < first || d > last),
                Noted::From(d) => *d <= last,
            };
            if met {
                out.merge(read);
            }
        }
        out
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
            e.st = apply_ent(&prev, e, (j, self.open_tests.as_ref()))?;
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
    /// (place, unit)'s last state, the previous checkpoint's shared where it is the same.
    /// P §2.4, D2-6.
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
        Checkpoint { day, states }
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

/// The state after entry `e` as a read at point `at` sees it: at a reading's own instant, a
/// read at its step or, for a closing reading, after it that day, the stated balance without
/// the windows' questions the reading keeps open from its next instant, every day of a window
/// that runs past it agreeing (refcore's `state`, its `reopen`; P32, N8). L §5.19 l.1726.
fn at_reading(e: &Ent, at: At) -> St {
    let Some(reopen) = &e.st.reopen else {
        return e.st.clone();
    };
    let own_instant = at.day == e.at.day
        && if e.at.step == Step::End {
            matches!(at.step, Step::End | Step::Close | Step::After)
        } else {
            at.step == e.at.step
        };
    if !own_instant {
        return e.st.clone();
    }
    let mut st = e.st.clone();
    st.hole = st.hole.as_ref().and_then(|h| h.without(reopen));
    st.perm = st.perm.as_ref().and_then(|h| h.without(reopen));
    st
}

/// The position after entry `n` of a log. E §4.2.
fn next_entry(n: usize) -> Result<usize, Stop> {
    n.checked_add(1)
        .ok_or_else(|| invariant("a log past the integers"))
}

/// A place's state in one unit with the holes and refusals of all its units and of the
/// whole walk folded in, its own refusal first (refcore's `pstate`, raw). E §4.2.
fn fold(mut st: St, any: St, all: St) -> St {
    for other in [any, all] {
        st.hole = spread(&[st.hole.as_ref(), other.hole.as_ref()]);
        st.perm = spread(&[st.perm.as_ref(), other.perm.as_ref()]);
        st.imp.extend(other.imp);
        if st.bad.is_none() {
            st.bad = other.bad;
        }
        if st.perm_bad.is_none() {
            st.perm_bad = other.perm_bad;
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

/// The state after `change`: a reading ends the holes and the dated refusals since the last
/// one, and keeps those no reading ends; a dated refusal leaves a hole whose question is one
/// of `open`, the window tests' (refcore's `apply`). E §4.2, §6.1; L §8 (Refusals outrank
/// holes, tally 2.1.8), §4.4 (a refusal that only some of a window's days give).
fn apply(prev: &St, change: &Change, (at, open): (usize, Option<&Hole>)) -> Result<St, Stop> {
    let mut st = prev.clone();
    st.reopen = None;
    match change {
        Change::Reset { val, qs } => {
            // a window that runs past the reading: the reading states the balance at its
            // instant, and from the next instant the movement may still fall, so the balance
            // is its hole up to the next reading (L §5.19 l.1726, E §4.1 step 2; P32, N1, N8);
            // `Walk::state` gives the reading's own instant without it. A window an earlier
            // reading met and not yet booked is reopened again: every reading inside it is an
            // edge (P41, F2)
            let pending: Vec<Option<&Hole>> = st
                .imp
                .iter()
                .chain(&st.held)
                .map(|p| Some(&p.hole))
                .collect();
            let imp = spread(&pending);
            let wins = st.perm.as_ref().and_then(Hole::of_windows);
            st.reopen = spread(&[imp.as_ref(), wins.as_ref()]);
            st.val = Some(val.clone());
            st.hole = spread(&[qs.as_ref(), imp.as_ref()]);
            let met = std::mem::take(&mut st.imp);
            st.held.extend(met);
            st.bad.clone_from(&st.perm_bad);
            st.rst = Some(at);
        }
        Change::Delta(q) => {
            if let Some(v) = &st.val {
                st.val = Some(v.checked_add(q).map_err(crate::eval::num_stop)?);
            }
        }
        Change::Hole(h) => st.hole = spread(&[st.hole.as_ref(), Some(h)]),
        Change::Perm(h) => st.perm = spread(&[st.perm.as_ref(), Some(h)]),
        Change::Unperm(h) => st.perm = st.perm.as_ref().and_then(|p| p.without(h)),
        Change::Bad(r) => {
            // after a test some of a window's days pass, which comes first is what the window
            // leaves open: the refusal doesn't outrank that hole
            let after_test = matches!((&st.hole, open), (Some(h), Some(o)) if h.shares(o));
            if st.bad.is_none() && !after_test {
                st.bad = Some(r.clone());
            }
        }
        Change::PermBad(r) => {
            if st.bad.is_none() {
                st.bad = Some(r.clone());
            }
            if st.perm_bad.is_none() {
                st.perm_bad = Some(r.clone());
            }
        }
        // its window's id is the entry's (`apply_ent`)
        Change::ImpOn(_) | Change::Mark => {}
    }
    Ok(st)
}

/// The state after entry `ent`: its change applied, the window tests' questions `open`
/// ([`apply`]); a movement dated by a window may fall from its `ImpOn` on, until its leg's
/// booking at the window's last day (refcore's `apply`, its `wid`); a movement's own refusal
/// meets no window test, so it outranks their hole (P31). E §4.2, L §4.4.
fn apply_ent(prev: &St, ent: &Ent, (at, open): (usize, Option<&Hole>)) -> Result<St, Stop> {
    let open = if ent.own { None } else { open };
    let mut st = apply(prev, &ent.change, (at, open))?;
    if let Some((wid, _)) = &ent.window {
        match &ent.change {
            Change::ImpOn(h) => st.imp.push(Pending {
                wid: wid.clone(),
                hole: h.clone(),
                signed: ent.signed.clone(),
            }),
            Change::Delta(_) | Change::Hole(_) => {
                st.imp.retain(|p| &p.wid != wid);
                st.held.retain(|p| &p.wid != wid);
            }
            Change::Reset { .. }
            | Change::Perm(_)
            | Change::Unperm(_)
            | Change::Bad(_)
            | Change::PermBad(_)
            | Change::Mark => {}
        }
    }
    Ok(st)
}

/// How a movement that doesn't count joins a read of its place (L §8.2 One read set, E §4.2):
/// by its day, which a pin can move out of the read's days, or from the day it doesn't exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Noted {
    /// Its leg's day: a read whose days leave it out lists it; none: every read.
    Out(Option<Day>),
    /// It doesn't exist from this day: a read whose prefix reaches it lists it (S4 on #241).
    From(Day),
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
    use crate::walk::At;

    /// A walk from mid-January to July: rent in January, a sweep decision on 10 February,
    /// nothing in March or April, a wage in May, a place that never moves after its opening.
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

move rent: transfers
  on rent_day
  leg cash from bank to out amount pay

move wage: transfers
  on wage_day
  leg cash from out to bank amount pay

move sweep: transfers
  on sweep_day
  reads bank
  leg cash from bank to save amount min(pay, bal(bank))

version facts
  cite card
  governs always
  known 2025-01-01
  pay = 100.00
  rent_day = 2025-01-20
  wage_day = 2025-05-05
  sweep_day = 2025-02-10
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
    fn a_place_that_did_not_move_is_shared() {
        let m = reality();
        let walk = walked(&m, Days::Events);
        let (feb, mar, may, jun) = (
            at(&walk, "2025-02-01"),
            at(&walk, "2025-03-01"),
            at(&walk, "2025-05-01"),
            at(&walk, "2025-06-01"),
        );
        assert!(
            shared(feb, jun, "idle"),
            "idle never moves after its opening"
        );
        assert!(
            !shared(feb, mar, "bank"),
            "the sweep's leg moves bank on 10 February"
        );
        assert!(
            shared(mar, may, "bank"),
            "bank doesn't move from March to May"
        );
        assert!(!shared(may, jun, "bank"), "the wage moves bank in May");
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
        let moved = apply(&start, &Change::Delta(Num::int(-7)), (0, None)).expect("moves");
        assert_eq!(moved.known(), Some(&Num::int(-2)), "5 - 7");
        let holed = apply(&moved, &Change::Hole(Hole::asked("q", "x")), (1, None)).expect("a hole");
        assert_eq!(holed.known(), None, "a hole");
        let reset = apply(
            &holed,
            &Change::Reset {
                val: Num::int(3),
                qs: None,
            },
            (2, None),
        )
        .expect("a reset");
        assert_eq!(reset.known(), Some(&Num::int(3)), "answered");
        assert_eq!(reset.rst, Some(2), "the reset's place");
        let perm = apply(&reset, &Change::Perm(Hole::asked("p", "x")), (3, None)).expect("a hole");
        let after = apply(
            &perm,
            &Change::Reset {
                val: Num::int(1),
                qs: None,
            },
            (4, None),
        )
        .expect("a reset");
        assert_eq!(after.known(), None, "a hole no reading answers");
        let bad = Refusal::new(Code::Run(Run::Place), "x".to_owned());
        let refused = apply(&after, &Change::Bad(bad.clone()), (5, None)).expect("refused");
        let other = Refusal::new(Code::Run(Run::Lifespan), "y".to_owned());
        let again = apply(&refused, &Change::Bad(other), (6, None)).expect("refused");
        assert_eq!(again.bad, Some(bad), "the first refusal holds");
        let read = Change::Reset {
            val: Num::int(2),
            qs: None,
        };
        let ended = apply(&again, &read, (7, None)).expect("a reset");
        assert_eq!(ended.bad, None, "a reading ends a dated refusal (R11)");
        let lasting = Refusal::new(Code::Run(Run::Div), "z".to_owned());
        let kept = apply(&ended, &Change::PermBad(lasting.clone()), (8, None)).expect("refused");
        let still = apply(&kept, &read, (9, None)).expect("a reset");
        assert_eq!(
            still.bad,
            Some(lasting),
            "no reading ends a hole day's refusal"
        );
    }

    #[test]
    fn a_refusal_after_a_window_test_leaves_its_hole() {
        let open = Hole::asked("date within w not precise enough for a balance", "put");
        let start = St::of(Some(Num::int(5)));
        let tested = apply(&start, &Change::Hole(open.clone()), (0, Some(&open))).expect("a hole");
        let bad = Refusal::new(Code::Run(Run::Lifespan), "x".to_owned());
        let after = apply(&tested, &Change::Bad(bad.clone()), (1, Some(&open))).expect("kept");
        assert_eq!(after.bad, None, "which comes first is the window's to say");
        assert_eq!(after.hole, Some(open), "the window's hole stands");
        let plain = apply(&tested, &Change::Bad(bad.clone()), (1, None)).expect("refused");
        assert_eq!(
            plain.bad,
            Some(bad),
            "after a plain hole the refusal outranks it"
        );
    }
}
