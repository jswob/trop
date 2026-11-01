//! A walk's place states: each place's log of entries in walk order, the state after each, and the register a balance read gathers.
//! Spec: E §4.2 (the day walk; balances are step functions, a change point per entry; holes in
//! the walk; a refused leg refuses its place from that point), §4.3 step 1 (a projection's walk
//! starts from the walk it continues, at a point), §6.1 (readings anchor a place); L §5.10 (a
//! place's balance: zero from its lifespan's start, else `HOLE-ANCHOR` until its first
//! reading), §8.1 (`HOLE-ANCHOR`'s questions), §8.2 (the register: what a balance rests on).
//! Twin: engine/tools/refcore/walk.py (`St`, `Ent`, `Log`, `apply`, `Walk.log`, `default`,
//! `add`, `state`, `pstate`, `cum`, `unbook`, `st_value`, `anchor_question`).
//! Invariants: a log's entries are in walk order and each holds the state after it; a state's
//! value is known, or none before the place's first reading; a refusal, once met at a place,
//! holds from that point; the register of a balance is every entry's since the last reset; a
//! projection's walk starts each place as the walk it continues stood at its point.

use std::collections::{BTreeMap, BTreeSet};

use super::agenda::Act;
use super::order::Run;
use super::plan::{Inst, LegVal, invariant};
use super::point::{At, Step};
use crate::base::{Refusal, Warn};
use crate::eval::{Collected, Entry};
use crate::hole::{Hole, spread};
use crate::model::Reality;
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
    /// Every day is walked.
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
    /// The places the day touched.
    pub touched: BTreeSet<String>,
}

/// One walk of a timeline: the main walk from S to E (refcore's `Walk`). E §4.2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Walk {
    /// Its cut K.
    pub cut: Day,
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
    /// The uncertain instances no observation fulfils (E §4.1 step 3).
    pub(crate) uncertain: Vec<Inst>,
    /// Each place's first reading day, for `HOLE-ANCHOR` (L §8.1).
    pub(crate) first_reading: BTreeMap<String, Day>,
    /// The places whose lifespan ends on each day, by its last day (E §9 item 6).
    pub(crate) endings: BTreeMap<Day, Vec<String>>,
    /// The running decision's legs, one group of the sign rule (L §5.10 point 2).
    pub(crate) group: Option<Group>,
    /// The plan movements' order texts, `<move>|<keys>`, sorted by code point: a movement's
    /// rank among the day's plans (refcore's `osort`).
    pub(crate) order_texts: Vec<String>,
    /// A projection's walk: what it continues and what it leaves out; none for a main walk.
    pub(crate) proj: Option<Box<Projected>>,
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
            Some(_) | None => opening_state(m, place, unit, self.root),
        }
    }
}

impl Walk {
    /// A projection's walk of cut `cut` from `start` to `end`, continuing `projected`, each
    /// place's first reading as `first_reading` gives it; nothing planned yet. E §4.3.
    #[must_use]
    pub fn projection(
        cut: Day,
        (start, end): (Day, Day),
        projected: Projected,
        first_reading: BTreeMap<String, Day>,
    ) -> Walk {
        Walk {
            proj: Some(Box::new(projected)),
            first_reading,
            ..Walk::new(cut, start, end)
        }
    }

    /// A walk of cut `cut` from `start` to `end`, nothing planned yet. E §4.2.
    #[must_use]
    pub fn new(cut: Day, start: Day, end: Day) -> Walk {
        Walk {
            cut,
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
            uncertain: Vec::new(),
            first_reading: BTreeMap::new(),
            endings: BTreeMap::new(),
            group: None,
            order_texts: Vec::new(),
            proj: None,
        }
    }

    /// The rank of order text `text` among the plans' (refcore's walk keys compare the text):
    /// twice its place plus one for one of them, twice the place it would take otherwise. E §4.2.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` past two billion movements.
    pub fn text_rank(&self, text: &str) -> Result<u32, Stop> {
        let at = self.order_texts.partition_point(|t| t.as_str() < text);
        let found = self.order_texts.get(at).is_some_and(|t| t == text);
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
            None => opening_state(m, place, unit, (self.cut, self.start)),
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
    /// dropped, and the states after them made again (refcore's `unbook`). E §4.2 step 5.
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
    ) -> Result<(), Stop> {
        let key = (place.to_owned(), unit.to_owned());
        let start = self.default_state(m, place, unit);
        let Some(log) = self.logs.get_mut(&key) else {
            return Ok(());
        };
        let drop = |e: &Ent| {
            matches!(e.change, Change::Delta(_))
                && e.mv.as_deref() == Some(movement.0)
                && e.leg.as_deref() == Some(movement.1)
                && holds.holds(e.at)
        };
        let Some(first) = log.ents.iter().position(drop) else {
            return Ok(());
        };
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

/// A place's state where a main walk of cut and first day `root` starts: every place's
/// total zero, a unit's position unknown until booked, a place zero from its lifespan's start
/// when that is on or after the walk's first day, else unknown until a reading (refcore's
/// `default`). L §5.10.
fn opening_state(m: &Reality, place: &str, unit: &str, (cut, start): (Day, Day)) -> St {
    if place == ALL {
        return St::of(Some(Num::zero()));
    }
    if unit == ALL {
        return St::of(None);
    }
    let span = m.things.get(place).and_then(|th| m.lifespan(th, cut));
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
    }
    Ok(st)
}

#[cfg(test)]
mod tests {
    use super::{Change, St, apply};
    use crate::base::{Code, Refusal, Run};
    use crate::hole::Hole;
    use crate::num::Num;

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
