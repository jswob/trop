//! Fence F6: a projection's walk read again through a later day is continued from where it stood, not walked again from its first day.
//! Spec: P §2.5 (timelines whose acts agree up to a day share the walk to it: the shared part
//! walked once, each timeline continued from the state before its first different day, which
//! comes from the plan pass's outputs; the unforked walk is its oracle; the counter "walks
//! shared"), §2.4 (a walk continued from the state it stood in), §3.5 (a projection's walk
//! freed when its scope ends); E §4.3 (a projection's walk from day t counts the movements
//! dated t through d as known on t: one walk of its window, steps 1–3), §4.1 (the plan pass
//! hands the walk its acts by day), §4.2 (the day walk).
//! Twin: the unforked walk, `Forks::Off`: projection.rs's `proj_walk` makes each read its own
//! walk and `end_projection` frees it, so every read walks its window from its first day
//! (refcore's walk.py `proj_walk`, `run_to`).
//! Invariants: a walk is continued only where every act the read's plan pass puts on the days
//! it walked is the act it took there, and every other plan output it walks by is the same; it
//! keeps what it walked and takes the read's plan for the days after; any other read takes a
//! new walk, as the twin does; at most `PARKED` walks are kept past their reads in a timeline,
//! the one kept first freed first, with the walks that continue it; a walk a read holds is
//! never parked, continued or freed; a kept walk outlives its read under its key, so nothing
//! else may learn from that key what an ended read did (F5's tree asks the cap instead).
//! PERF: P8-14 and the budget table's F6-c1 to F6-v4 and F6-X (X's repeated walks: a
//! projection read day by day, `min(bal(konto, d) from t … for d in t .. b step 1 day)`, walks
//! its window once, each read continued from the last, where a new walk per read walks t
//! through d for each d; at M2's G6 P8-14's check stepped 9 868 days in 605 walks; with F6,
//! 1 826);
//! plain twin: projection.rs (`Forks::Off`); the equivalence test:
//! tally-cli/tests/fork_oracle.rs, and this file's tests.
//!
//! The first different day of two plans of one projection, the walk through day d₁ and the
//! read through d₂ after it, is the first day on which the acts the plans put differ: an
//! instance the longer window keeps and the shorter leaves out has its first day and every leg
//! after d₁ (`agenda.rs` `outside_window`), so the acts agree on every day the shorter walk
//! walked, and the walk goes on from its last day. That is checked, act by act, never assumed:
//! the walk keeps the plan's acts of each day it walks, and where one differs from the read's
//! plan, or where anything else the plan hands the walk differs, the read walks anew.

use std::collections::{BTreeMap, VecDeque};

use super::agenda::{Act, Mv};
use super::point::At;
use super::state::{Progress, Walk};
use crate::eval::Engine;
use crate::syntax::Stop;
use crate::time::Day;

/// Whether a projection's walk read again through a later day is continued from where it
/// stood: the switch F6's equivalence test turns (P §2.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Forks {
    /// Continued, where its acts agree up to there: fence F6, as `run` walks.
    On,
    /// Walked again from its first day, each read its own walk: the plain twin.
    Off,
}

/// The most projections' walks a timeline keeps past their reads: past it the one kept first
/// is freed, so the memory held stays a few walks (P §3.5; pick P54).
const PARKED: usize = 4;

/// The projections' walks a timeline keeps past their reads, the one kept first first, and
/// the kept walks waiting for a read's own walk to be planned (P §2.5, §3.5).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Parked {
    /// The walks kept, by their place in the timeline's list.
    ids: VecDeque<usize>,
    /// Each kept walk taken out of the list to be continued, by the place of the read's own
    /// walk, whose plan pass decides whether it is.
    waiting: BTreeMap<usize, Walk>,
}

/// What a read of a projection does with the walk kept under its key (P §2.5).
#[derive(Debug)]
pub(crate) enum Held {
    /// Takes it as it is: it plans through the read's last day.
    Kept(usize),
    /// Takes a new walk, which continues this one once its plan pass agrees with it.
    Continue(Box<Walk>),
    /// Takes a new walk, walked from its first day.
    New,
}

/// The acts of one day, as a plan pass put them and the walk takes them: in walk order, a
/// decision's legs booked ahead left out, since the walk makes those (E §4.2 step 3).
type DayActs = Vec<(At, Act)>;

impl Walk {
    /// Keeps the plan's acts of day `day` as the walk takes them (`acts`, in walk order), for
    /// a later read that continues it to compare its own plan with (P §2.5). E §4.2.
    pub(crate) fn keep_walked(&mut self, day: Day, acts: &[(At, Act)]) {
        let planned: DayActs = acts
            .iter()
            .filter(|(_, act)| !booked_ahead(act))
            .cloned()
            .collect();
        if !planned.is_empty() {
            self.walked.insert(day, planned);
        }
    }
}

/// Whether an act is a decision's leg booked ahead onto a later day, which the walk puts on
/// its agenda as it runs the decision, not the plan pass (E §4.2 step 3, N-6).
fn booked_ahead(act: &Act) -> bool {
    matches!(act, Act::Leg(Mv::Run(_), _))
}

impl Engine {
    /// Whether walk `id` is a projection's walk kept past its reads (P §2.5).
    pub(crate) fn is_parked(&self, id: usize) -> bool {
        self.parked.ids.contains(&id)
    }

    /// A projection's walk whose last read ended: freed with its key as its scope ends, the
    /// twin's; with forks on, kept for the next read of the same projection, and the one kept
    /// first freed once more than `PARKED` are kept. P §2.5, §3.5; E §4.3.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a walk the engine lacks.
    pub(crate) fn park_projection(&mut self, id: usize) -> Result<(), Stop> {
        self.parked.waiting.remove(&id);
        if self.forks == Forks::Off {
            return self.free_projection(id);
        }
        if !self.is_parked(id) {
            self.parked.ids.push_back(id);
        }
        while self.parked.ids.len() > PARKED {
            let Some(oldest) = self.parked.ids.pop_front() else {
                break;
            };
            self.free_projection(oldest)?;
        }
        Ok(())
    }

    /// A projection's walk freed, its key with it, and every kept walk that continues it, whose
    /// key no read can name again (P §3.5, D1-6; E §4.3 step 1).
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a walk the engine lacks.
    pub(crate) fn free_projection(&mut self, id: usize) -> Result<(), Stop> {
        let mut gone = vec![id];
        while let Some(next) = gone.pop() {
            self.parked.ids.retain(|kept| *kept != next);
            self.parked.waiting.remove(&next);
            let walk = self.walk_mut(next)?;
            *walk = walk.freed();
            self.proj_walks.retain(|_, held| *held != next);
            for kept in &self.parked.ids {
                let base = self.walk(*kept)?.proj.as_ref().map(|p| p.base);
                if base == Some(next) {
                    gone.push(*kept);
                }
            }
        }
        Ok(())
    }

    /// Kept walk `id`, met by a read of its projection through day `through`: taken as it is
    /// where it plans through that day or further, as an open walk is (a read through a day
    /// it walked past reads its log, one through a later day walks on); taken out of the list
    /// to be continued where it plans less far and stands between two days; freed otherwise,
    /// the read walking anew as the twin does (P §2.5, §3.5; pick P73). E §4.3.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a walk the engine lacks.
    pub(crate) fn meet_parked(&mut self, id: usize, through: Day) -> Result<Held, Stop> {
        self.parked.ids.retain(|kept| *kept != id);
        let kept = self.walk(id)?;
        if kept.plans_through() >= through {
            // the twin walks this read's days again; here the days walked are shared
            if kept.done.is_some_and(|walked| walked < through) {
                self.count_shared();
            }
            return Ok(Held::Kept(id));
        }
        if !paused(kept) {
            self.free_projection(id)?;
            return Ok(Held::New);
        }
        let stub = kept.freed();
        let walk = core::mem::replace(self.walk_mut(id)?, stub);
        self.free_projection(id)?;
        Ok(Held::Continue(Box::new(walk)))
    }

    /// Kept walk `kept` waits for new walk `id`'s plan pass, which decides whether `id`
    /// continues it (P §2.5).
    pub(crate) fn wait_to_continue(&mut self, id: usize, kept: Walk) {
        self.parked.waiting.insert(id, kept);
    }

    /// Walk `id`'s plan pass done: the lengths of its plan's uncertain and modelled lists kept,
    /// the plan's part of each; where a kept walk waits for it and agrees with its plan on
    /// every day it walked, walk `id` becomes that walk, continued with this plan from the day
    /// after its last walked day; otherwise it walks from its first day, as the twin's
    /// (P §2.5). Walk `id` has walked no day yet: its plan pass has just ended. E §4.1, §4.3.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a walk the engine lacks, or one with no day walked or no
    /// projection.
    pub(crate) fn continue_planned(&mut self, id: usize) -> Result<(), Stop> {
        let walk = self.walk_mut(id)?;
        walk.plan_lists = (walk.uncertain.len(), walk.modelled.len());
        let Some(mut kept) = self.parked.waiting.remove(&id) else {
            return Ok(());
        };
        let planned = self.walk(id)?;
        if !agrees(&kept, planned) {
            return Ok(());
        }
        let stub = planned.freed();
        let fresh = core::mem::replace(self.walk_mut(id)?, stub);
        continue_with(&mut kept, fresh)?;
        *self.walk_mut(id)? = kept;
        self.count_shared();
        Ok(())
    }

    /// One more walk shared: a read that took a kept walk's days where the twin walks them
    /// again (P §1.1, P §2.5, the counter "walks shared").
    fn count_shared(&mut self) {
        #[cfg(feature = "counters")]
        self.counts.add(crate::Unit::WalkShared);
    }
}

/// Whether a kept walk can be continued: walking, between two days, not refused, with a day
/// walked. E §4.2.
fn paused(walk: &Walk) -> bool {
    walk.progress == Progress::Walking && walk.today.is_none() && walk.done.is_some()
}

/// Whether kept walk `kept` and the read's plan `fresh` agree on every day `kept` walked, and
/// on everything else the plan hands the walk: its days, cut and world, the movement orders,
/// each place's last day and first reading, the observed `same` ids, the uncertain and the
/// modelled-past instances (the plan's part of each the whole of the read's plan's; the walk
/// then adds to them), and what the projection continues
/// and leaves out (P §2.5, the first different day from the plan pass's outputs). E §4.1,
/// §4.3.
fn agrees(kept: &Walk, fresh: &Walk) -> bool {
    let (Some(a), Some(b)) = (&kept.proj, &fresh.proj) else {
        return false;
    };
    let same_projection = a.base == b.base
        && a.point == b.point
        && a.left_out == b.left_out
        && a.counted == b.counted
        && a.levels == b.levels
        && a.root == b.root
        && a.world == b.world
        && a.through < b.through;
    let same_plan = kept.cut == fresh.cut
        && kept.world == fresh.world
        && (kept.start, kept.end) == (fresh.start, fresh.end)
        && kept.orders == fresh.orders
        && kept.endings == fresh.endings
        && kept.first_reading == fresh.first_reading
        && kept.plan_lists == fresh.plan_lists
        && kept.uncertain.starts_with(&fresh.uncertain)
        && kept.modelled.starts_with(&fresh.modelled);
    same_projection && same_plan && same_days(kept, fresh)
}

/// Whether `fresh`'s plan puts on each day `kept` walked the acts `kept` took there, and
/// nothing on a day `kept` stepped over (P §2.5). E §4.2.
fn same_days(kept: &Walk, fresh: &Walk) -> bool {
    let Some(done) = kept.done else {
        return false;
    };
    // a day the plan left no act on (a projection's first, before its point) holds none
    let planned = fresh
        .agenda
        .range(..=done)
        .filter(|(_, acts)| !acts.is_empty());
    if planned.clone().count() != kept.walked.len() {
        return false;
    }
    for ((day, acts), (walked_day, walked)) in planned.zip(&kept.walked) {
        if day != walked_day || acts.len() != walked.len() {
            return false;
        }
        let mut acts = acts.clone();
        acts.sort_by_key(|a| a.0);
        let all = acts
            .iter()
            .zip(walked)
            .all(|(plan, took)| plan.0 == took.0 && same_act((fresh, &plan.1), (kept, &took.1)));
        if !all {
            return false;
        }
    }
    true
}

/// Whether two acts are one: the same kind and terms, an instance by what it is (its body,
/// as the plan pass made it) and not by its place in its walk's list, a day's decisions by
/// the instances due; any other act by its terms, an observation's among them, which only a
/// main walk books, never a projection's (E §4.1, §4.2 step 1).
fn same_act((one, act): (&Walk, &Act), (other, like): (&Walk, &Act)) -> bool {
    match (act, like) {
        (Act::Leg(mv, leg), Act::Leg(mv2, leg2))
        | (Act::ImpOn(mv, leg), Act::ImpOn(mv2, leg2))
        | (Act::LegHole(mv, leg, _), Act::LegHole(mv2, leg2, _))
        | (Act::LegBad(mv, leg, _), Act::LegBad(mv2, leg2, _))
        | (Act::Perm(mv, leg, _), Act::Perm(mv2, leg2, _)) => {
            leg == leg2 && same_mv((one, *mv), (other, *mv2)) && same_terms(act, like)
        }
        (Act::Decide(id, day), Act::Decide(id2, day2)) => {
            let key = (id.clone(), *day);
            id == id2 && day == day2 && one.due.get(&key) == other.due.get(&key)
        }
        _ => act == like,
    }
}

/// Whether two movements are one: plan instances by their bodies; a decision's run never comes
/// from a plan pass, so it is no plan's (E §4.1, §4.2 step 3).
fn same_mv((one, mv): (&Walk, Mv), (other, mv2): (&Walk, Mv)) -> bool {
    match (mv, mv2) {
        (Mv::Plan(at), Mv::Plan(at2)) => {
            one.insts.get(at).is_some() && one.insts.get(at) == other.insts.get(at2)
        }
        (Mv::Plan(_) | Mv::Run(_), _) => false,
    }
}

/// Whether two leg acts of one movement carry the same hole or refusal. E §4.2.
fn same_terms(act: &Act, like: &Act) -> bool {
    match (act, like) {
        (Act::LegHole(_, _, hole), Act::LegHole(_, _, hole2)) => hole == hole2,
        (Act::LegBad(_, _, refusal), Act::LegBad(_, _, refusal2)) => refusal == refusal2,
        (Act::Perm(_, _, hole), Act::Perm(_, _, hole2)) => hole == hole2,
        _ => true,
    }
}

/// Kept walk `walk` continued with the read's plan `fresh`: what it walked kept, the plan's
/// acts of the days after its last walked day taken, with the decisions' legs it booked ahead
/// onto them after the day's plan acts, as the twin's walk puts them there as it runs; the
/// plan's instances, decisions due and last planned day taken, and the reads that hold it
/// now. P §2.5; E §4.2 step 3, §4.3.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a walk with no day walked or no projection.
fn continue_with(walk: &mut Walk, fresh: Walk) -> Result<(), Stop> {
    let done = walk
        .done
        .ok_or_else(|| super::plan::invariant("a walk continued with no day walked"))?;
    let mut agenda = fresh.agenda;
    let mut later = match done.next() {
        Some(first) => agenda.split_off(&first),
        None => BTreeMap::new(),
    };
    // the plan's legs of decisions run before t (#183) are the kept walk's own too, carried
    // below with what it booked ahead: the plan's copies would book them twice
    for acts in later.values_mut() {
        acts.retain(|(_, act)| !booked_ahead(act));
    }
    for (day, acts) in core::mem::take(&mut walk.agenda) {
        for act in acts.into_iter().filter(|(_, act)| booked_ahead(act)) {
            later.entry(day).or_insert_with(Vec::new).push(act);
        }
    }
    walk.agenda = later;
    walk.insts = fresh.insts;
    walk.due = fresh.due;
    let (through, readers) = fresh
        .proj
        .as_ref()
        .map(|p| (p.through, p.readers))
        .ok_or_else(|| super::plan::invariant("a projection's walk with no projection"))?;
    let p = walk
        .proj
        .as_mut()
        .ok_or_else(|| super::plan::invariant("a projection's walk with no projection"))?;
    p.through = through;
    p.readers = readers;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{Forks, PARKED, agrees, continue_with};
    use crate::eval::{Collected, Engine};
    use crate::model::World;
    use crate::time::Day;
    use crate::walk::{Act, At, Mv, Progress, Projected, Step, Walk};

    fn day(d: u8) -> Day {
        Day::from_ymd(2026, 2, d).expect("a day")
    }

    /// A projection's walk from Feb 1st, continuing walk 0 from that day's start, planned
    /// through day `through` of February, nothing walked.
    fn projection(through: u8) -> Walk {
        let projected = Projected {
            base: 0,
            point: At::start(day(1)),
            left_out: BTreeSet::new(),
            counted: BTreeMap::new(),
            levels: Vec::new(),
            root: (day(1), day(1)),
            world: World::Whole,
            through: day(through),
            readers: 0,
        };
        let mut walk = Walk::projection(day(1), (day(1), day(28)), projected, BTreeMap::new());
        walk.progress = Progress::Walking;
        walk
    }

    /// A register mark at konto on day `d`: an act no index names.
    fn mark(d: u8, place: &str) -> (At, Act) {
        let act = Act::Mark(
            place.to_owned(),
            "DUK".to_owned(),
            None,
            Collected::default(),
        );
        (At::of(day(d), Step::Plans), act)
    }

    /// An act put on its day of `walk`'s agenda.
    fn put(walk: &mut Walk, (at, act): (At, Act)) {
        walk.at(at, act);
    }

    /// `kept` walked through day `done`, taking the acts its agenda holds there.
    fn walked_through(kept: &mut Walk, done: u8) {
        let days: Vec<Day> = kept.agenda.range(..=day(done)).map(|(d, _)| *d).collect();
        for d in days {
            let mut acts = kept.agenda.remove(&d).expect("the day's acts");
            acts.sort_by_key(|a| a.0);
            kept.keep_walked(d, &acts);
        }
        kept.done = Some(day(done));
    }

    #[test]
    fn a_walk_whose_plan_agrees_on_every_day_it_walked_is_continued() {
        let mut kept = projection(5);
        for item in [mark(2, "konto"), mark(4, "konto"), mark(9, "konto")] {
            put(&mut kept, item);
        }
        walked_through(&mut kept, 5);
        let mut fresh = projection(10);
        for item in [
            mark(2, "konto"),
            mark(4, "konto"),
            mark(9, "konto"),
            mark(7, "konto"),
        ] {
            put(&mut fresh, item);
        }
        assert!(agrees(&kept, &fresh), "the same acts on the 2nd and 4th");
        // an act the longer plan puts on a day the walk walked: the read walks anew
        let mut differs = fresh.clone();
        put(&mut differs, mark(3, "konto"));
        assert!(!agrees(&kept, &differs), "a new act on the 3rd");
        let mut moved = fresh.clone();
        moved.agenda.remove(&day(4));
        put(&mut moved, mark(4, "savings"));
        assert!(!agrees(&kept, &moved), "another act on the 4th");
        // a day the plan left empty holds no act
        let mut empty = fresh.clone();
        empty.agenda.insert(day(3), Vec::new());
        assert!(agrees(&kept, &empty), "an empty day is no act");
        // a plan that differs in what it hands the walk: anew
        let mut other = fresh;
        other.first_reading.insert("konto".to_owned(), day(20));
        assert!(!agrees(&kept, &other), "another first reading");
    }

    #[test]
    fn a_decisions_leg_booked_ahead_is_the_walks_not_the_plans() {
        let mut kept = projection(5);
        put(&mut kept, mark(2, "konto"));
        kept.at(At::of(day(2), Step::Plans), Act::Leg(Mv::Run(0), 0));
        kept.at(At::of(day(8), Step::Plans), Act::Leg(Mv::Run(0), 1));
        walked_through(&mut kept, 5);
        assert_eq!(
            kept.walked.get(&day(2)).map(Vec::len),
            Some(1),
            "the mark alone"
        );
        let mut fresh = projection(10);
        for item in [mark(2, "konto"), mark(9, "konto")] {
            put(&mut fresh, item);
        }
        fresh.proj.as_mut().expect("a projection").readers = 1;
        assert!(agrees(&kept, &fresh), "the run's legs left out");
        continue_with(&mut kept, fresh).expect("continued");
        let days: Vec<(Day, usize)> = kept.agenda.iter().map(|(d, a)| (*d, a.len())).collect();
        assert_eq!(
            days,
            [(day(8), 1), (day(9), 1)],
            "the days after the 5th only"
        );
        let p = kept.proj.as_ref().expect("a projection");
        assert_eq!(
            (p.through, p.readers),
            (day(10), 1),
            "the read's day and readers"
        );
        assert_eq!(kept.done, Some(day(5)), "walked as it was");
    }

    /// An engine holding walk 0 and `n` projections' walks continuing it, each under a key of
    /// its own, every read of them ended.
    fn engine_with(n: usize, forks: Forks) -> Engine {
        let mut engine = Engine::new(day(1)).with_how((
            crate::eval::Memo::Classes,
            crate::walk::Lows::Tree,
            forks,
        ));
        engine
            .walks
            .push(Walk::new(day(1), (day(1), day(28)), World::Whole));
        for k in 1..=n {
            engine.walks.push(projection(5));
            let left: BTreeSet<String> = [format!("m{k}")].into();
            engine
                .proj_walks
                .insert((0, At::start(day(1)), day(1), left), k);
        }
        engine
    }

    #[test]
    fn a_timeline_keeps_a_few_walks_past_their_reads_the_first_freed_first() {
        let mut engine = engine_with(PARKED + 2, Forks::On);
        for k in 1..=PARKED + 2 {
            engine.park_projection(k).expect("parked");
        }
        let kept: Vec<usize> = (1..=PARKED + 2).filter(|k| engine.is_parked(*k)).collect();
        assert_eq!(
            kept,
            (3..=PARKED + 2).collect::<Vec<_>>(),
            "the last ones kept"
        );
        assert_eq!(
            engine.proj_walks.len(),
            PARKED,
            "the freed walks' keys gone"
        );
        assert!(
            matches!(
                engine.walks.get(1).map(|w| &w.progress),
                Some(Progress::Failed(..))
            ),
            "the first freed"
        );
        let mut twin = engine_with(2, Forks::Off);
        twin.park_projection(1).expect("freed");
        assert!(
            !twin.is_parked(1) && twin.proj_walks.len() == 1,
            "the twin frees at once"
        );
    }

    #[test]
    fn a_walk_freed_frees_the_kept_walks_that_continue_it() {
        let mut engine = engine_with(3, Forks::On);
        if let Some(p) = engine.walks.get_mut(3).and_then(|w| w.proj.as_mut()) {
            p.base = 2;
        }
        for k in [2, 3, 1] {
            engine.park_projection(k).expect("parked");
        }
        engine.free_projection(2).expect("freed");
        assert!(
            !engine.is_parked(2) && !engine.is_parked(3),
            "2 and the walk continuing it"
        );
        assert!(engine.is_parked(1), "1 kept");
    }
}
