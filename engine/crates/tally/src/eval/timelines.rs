//! One timeline per (cut, world, pins): the walk's state kept apart per world and per pin set the walk reaches.
//! Spec: E §3.3 (a timeline per cut class, world and pin set the moves reach), §3.2 (a
//! timeline's contexts), §9 item 4 ("Moves inside `as if` are computed and never booked";
//! each timeline's walks keep their signatures until the ask ends or the scope that made them
//! is freed); L §5.11 ("`flows` inside sees the recomputed movements"), §6.4 (a scenario's
//! world: its moves, places and versions), §5.14.1 (a decision's running reads stay in its
//! own timeline); P §3.2 (a pin set's layer freed when its `as if` or `solve` step ends);
//! L §5.5 (a read outside its ask's fan-out that needs the walk reads each combination's of
//! the axes the walk reaches and its timeline sets none of, P48).
//! Twin: engine/tools/refcore/walk.py (`m_main_walk`, its walks keyed by `wkey(ctx)`),
//! moves.py (`wkey`: the cut, the pins, the world) and mreads.py (`leg_refused`).
//! Invariants: the engine's walk fields hold the timeline in force and every other one is
//! parked under its key; a key is a world and the pins of the reader's layer on items the
//! walk reaches, so a pin no move reaches shares its reader's timeline; a scope that changes
//! the key enters its timeline and leaving it brings the one before back; a decision's
//! running read never changes timeline; a parked timeline freed with its layer is checked
//! first (asks move no money).

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use super::context::Layer;
use super::state::{Ctx, Engine, invariant};
use crate::base::{Refusal, Warn};
use crate::book::{Adjustment, Coverage, Linked, Obs};
use crate::model::{Reality, World};
use crate::syntax::Stop;
use crate::time::Day;
use crate::walk::{Booked, Inst, ProjKey, Trees, Walk};

/// A timeline's key: the world its walks are walked in and the pins on items they reach
/// (E §3.3). It has no order of its own (R-47): timelines are found by equality.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TimelineKey {
    /// The world (L §6.4).
    pub(crate) world: World,
    /// The pins and wording pins on items the walk reaches.
    pub(crate) pins: Layer,
}

impl TimelineKey {
    /// The reality's own timeline: its whole world, unpinned.
    pub(crate) const BASE: TimelineKey = TimelineKey {
        world: World::Whole,
        pins: Layer::BASE,
    };
}

/// One timeline's walk state, as the engine's walk fields hold it while it is in force
/// (E §3.3, §4.1–§4.3, §5, §9 items 4 and 5).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Timeline {
    /// The walks, by their place in the list.
    walks: Vec<Walk>,
    /// The main walk of each cut.
    main_walks: BTreeMap<Day, usize>,
    /// The walks being walked.
    walking: BTreeSet<usize>,
    /// Each move's instances per cut.
    plans: BTreeMap<(String, Day), Rc<Vec<Inst>>>,
    /// The moves whose instances are being computed.
    plan_active: BTreeSet<(String, Day)>,
    /// Each plan move's adjustments per cut.
    adjustments: BTreeMap<(String, Day), Vec<Adjustment>>,
    /// The moves whose adjustments are being computed.
    adjust_active: BTreeSet<(String, Day)>,
    /// The projections' walks.
    proj_walks: BTreeMap<ProjKey, usize>,
    /// Fence F5's trees.
    trees: Trees,
    /// The observations each cut sees.
    observed: BTreeMap<Day, Vec<Obs>>,
    /// The versions each cut sees that cover each place.
    covered: BTreeMap<Day, Coverage>,
    /// Each instance's links per cut.
    links: BTreeMap<(Day, String), Linked>,
    /// The links being judged.
    link_active: BTreeSet<(Day, String)>,
    /// The refusals of observations' records per cut.
    obs_errs: BTreeMap<(Day, String), Refusal>,
    /// Each plan move's adjustments per cut, settled.
    settled: BTreeMap<(String, Day), Vec<Adjustment>>,
    /// Every adjustment per cut, unsettled.
    unsettled: BTreeMap<Day, Vec<Adjustment>>,
    /// The cuts whose adjustments are being measured.
    adjust_all_active: BTreeSet<Day>,
    /// Each booked plan instance's amount by (instance, booking cut).
    booked: BTreeMap<(String, Day), Booked>,
    /// Each main walk's signature once it is done.
    signatures: BTreeMap<usize, crate::invariant::Signature>,
    /// The walks up to a day.
    prefixes: super::state::Prefixes,
    /// The legs each combination's main walk of a cut refused, where the main walk read an
    /// axis the timeline sets no option of.
    each_refused: BTreeMap<Day, Vec<LegRefusals>>,
}

/// The legs a walk refused, by (move, leg).
pub(crate) type LegRefusals = BTreeMap<(String, String), Refusal>;

/// Every combination of positions in lists of `counts` options, the first list slowest
/// (refcore's `combos`); `None` past `cap` combinations, counted as they are made, so a
/// product past it is never built. L §5.5.
pub(crate) fn combinations(counts: &[usize], cap: usize) -> Option<Vec<Vec<usize>>> {
    let mut combos: Vec<Vec<usize>> = vec![Vec::new()];
    for count in counts {
        if combos.len().checked_mul(*count)? > cap {
            return None;
        }
        let mut next = Vec::with_capacity(combos.len().checked_mul(*count)?);
        for combo in &combos {
            for at in 0..*count {
                let mut longer = combo.clone();
                longer.push(at);
                next.push(longer);
            }
        }
        combos = next;
    }
    Some(combos)
}

/// The refusal of leg `wanted` in every walk of `each`, which must be the same (P48).
///
/// # Errors
///
/// `NotBuilt` where two walks differ on it, the line being then one per combination.
fn agreed(each: &[LegRefusals], wanted: &(String, String)) -> Result<Option<Refusal>, Stop> {
    let mut seen: Option<Option<&Refusal>> = None;
    for legs in each {
        let got = legs.get(wanted);
        if seen.is_some_and(|first| first != got) {
            return Err(Stop::NotBuilt(
                "a leg's refusal that differs by an axis its ask doesn't fan out over",
            ));
        }
        seen = Some(got);
    }
    Ok(seen.flatten().cloned())
}

impl Timeline {
    /// A timeline no scope has entered yet: no walk walked, nothing planned or booked.
    fn fresh() -> Timeline {
        Timeline::default()
    }
}

/// The timeline in force, the keys of those the open scopes left, and the parked ones.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Timelines {
    /// The key of the timeline in force.
    now: TimelineKey,
    /// The keys in force before each open scope that entered a timeline, the innermost last.
    before: Vec<TimelineKey>,
    /// The timelines not in force, each with its key.
    parked: Vec<(TimelineKey, Timeline)>,
}

impl Default for Timelines {
    fn default() -> Timelines {
        Timelines {
            now: TimelineKey::BASE,
            before: Vec::new(),
            parked: Vec::new(),
        }
    }
}

impl Engine {
    /// A context of the timeline in force: law date `law`, cut `cut`, its world and the pins
    /// its walks reach (E §3.2, §3.3; L §6.4).
    #[must_use]
    pub(crate) fn timeline_ctx(&self, law: Option<Day>, cut: Day) -> Ctx {
        let key = self.timelines.now;
        Ctx {
            layer: key.pins,
            world: key.world,
            ..Ctx::new(law, cut)
        }
    }

    /// The world of the timeline in force (L §6.4).
    #[must_use]
    pub(crate) fn timeline_world(&self) -> World {
        self.timelines.now.world
    }

    /// The key of the timeline in force: its world and pins (E §3.3).
    #[must_use]
    pub(crate) fn timeline_key(&self) -> TimelineKey {
        self.timelines.now
    }

    /// Enters the timeline of context `inside` for a scope: its world and the pins of its
    /// layer the walk reaches; the key in force before is kept for `leave_timeline`. A
    /// decision's running read stays in its own timeline (L §5.14.1), so a scope there that
    /// would change it stops `NotBuilt`. E §3.3.
    ///
    /// # Errors
    ///
    /// `NotBuilt` for a decision's running read whose scope changes the timeline;
    /// `ENGINE-INVARIANT` for a freed layer.
    pub(crate) fn enter_timeline(&mut self, m: &Reality, inside: Ctx) -> Result<(), Stop> {
        let key = TimelineKey {
            world: inside.world,
            pins: self.walk_pins(m, inside.layer)?,
        };
        if inside.at.is_some() && key != self.timelines.now {
            return Err(Stop::NotBuilt(
                "a decision's running read in another world or under a pin the moves reach",
            ));
        }
        self.timelines.before.push(self.timelines.now);
        self.use_timeline(key);
        Ok(())
    }

    /// The timeline of a task replayed from the work stack: its context's, whatever scope it
    /// unwound through (R-16). E §3.3.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a freed layer.
    pub(crate) fn replay_timeline(&mut self, m: &Reality, ctx: Ctx) -> Result<(), Stop> {
        let key = TimelineKey {
            world: ctx.world,
            pins: self.walk_pins(m, ctx.layer)?,
        };
        self.timelines.before.push(self.timelines.now);
        self.use_timeline(key);
        Ok(())
    }

    /// Leaves the timeline the innermost scope entered: the one in force before it is back.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a timeline left that no scope entered.
    pub(crate) fn leave_timeline(&mut self) -> Result<(), Stop> {
        let key = self
            .timelines
            .before
            .pop()
            .ok_or_else(|| invariant("a timeline left that no scope entered"))?;
        self.use_timeline(key);
        Ok(())
    }

    /// The refusal of leg `wanted` in the walks of cut `cut` of each combination of the axes
    /// the walk reaches and the timeline in force sets no option of, once they are walked for
    /// it (`refused_in_each`); `None` before. E §3.3, §4.2 step 5; L §5.5.
    pub(crate) fn refused_by_each(
        &self,
        wanted: &(String, String),
        cut: Day,
    ) -> Option<Result<Option<Refusal>, Stop>> {
        Some(agreed(self.each_refused.get(&cut)?, wanted))
    }

    /// The refusal of leg `wanted` read outside its ask's fan-out, where the main walk of cut
    /// `cut` in the timeline in force stopped at `stop`, an axis it sets no option of: the
    /// main walk of each combination of the axes the walk reaches and it sets none of, each
    /// walked once for every leg read here, their refused legs kept with the timeline. They
    /// must agree, so the line reads no option (P48); a walk that refuses refuses none of its
    /// legs to this read, as at the ask's own timeline (refcore's `leg_refused`). These walks
    /// are a check, not the line's: a warning every combination's walk gives reads none of
    /// the axes and is the run's, as refcore's walk to the leg's day meets it before its
    /// guard (its `guarded_walk`, `prefix_walk`); one that only some give is dropped, as
    /// refcore warns nothing past its guard. Their timelines are freed with their layers.
    /// E §3.3, §4.2 step 5; L §5.5, §8.5.
    ///
    /// # Errors
    ///
    /// `stop` where the walk reaches no such axis; `NotBuilt` past 64 combinations
    /// (`LOAD-FORKS`'s) and where two combinations' walks differ on the leg, the line being
    /// then one per combination; the stops of each walk but a refusal.
    pub(crate) fn refused_in_each(
        &mut self,
        m: &Reality,
        wanted: &(String, String),
        cut: Day,
        stop: Stop,
    ) -> Result<Option<Refusal>, Stop> {
        let unset = self.unset_walk_axes(m, self.timelines.now.pins)?;
        if unset.is_empty() {
            return Err(stop);
        }
        let mut options: Vec<Vec<String>> = Vec::with_capacity(unset.len());
        for id in &unset {
            let axis = m
                .axes
                .get(id)
                .ok_or_else(|| invariant("an axis load didn't build"))?;
            options.push(axis.options.iter().map(|(name, _)| name.clone()).collect());
        }
        let counts: Vec<usize> = options.iter().map(Vec::len).collect();
        let combos = combinations(&counts, 64).ok_or(Stop::NotBuilt(
            "a leg's refusal over more combinations of the walk's axes than an ask's 64",
        ))?;
        let here = self.timeline_ctx(None, cut);
        let warned = self.walk_warnings.len();
        let mut common: Option<Vec<(Warn, String)>> = None;
        let mut each = Vec::with_capacity(combos.len());
        for combo in &combos {
            let mut named = Vec::with_capacity(combo.len());
            for ((id, names), at) in unset.iter().zip(&options).zip(combo) {
                let name: &String = names
                    .get(*at)
                    .ok_or_else(|| invariant("a combination's option its axis lacks"))?;
                named.push((id.clone(), name.clone()));
            }
            let inside = self.with_options(here, &named)?;
            self.open_layer(m, inside)?;
            let outer = self.guarded.replace(self.active.clone());
            let built = self.main_walk(m, cut);
            self.guarded = outer;
            // a main walk continues no other walk: its own refused legs are its chain's
            let legs = match built {
                Ok(id) => self.walk(id).map(|w| w.bad_legs.clone()),
                Err(Stop::Refused(_)) => Ok(BTreeMap::new()),
                Err(Stop::NotBuilt(why)) if why == super::GUARDED => Err(Stop::NotBuilt(
                    "a leg's refusal over the walk's axes whose walk needs a cell still being \
                     computed",
                )),
                Err(other) => Err(other),
            };
            self.end_layer(inside.layer, legs.as_ref().err())?;
            let met: Vec<(Warn, String)> =
                self.walk_warnings.iter().skip(warned).cloned().collect();
            self.walk_warnings.truncate(warned);
            common = Some(match common {
                None => met,
                Some(before) => before.into_iter().filter(|w| met.contains(w)).collect(),
            });
            each.push(legs?);
        }
        for w in common.into_iter().flatten() {
            if !self.walk_warnings.contains(&w) {
                self.walk_warnings.push(w);
            }
        }
        let read = agreed(&each, wanted);
        self.each_refused.insert(cut, each);
        read
    }

    /// Drops each parked timeline whose pins are a layer freed (numbered in `freed`), its
    /// walks checked first: asks move no money (E §9 item 4; P §3.2).
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT asks move no money: …` for a walk of one whose entries changed.
    pub(crate) fn free_timelines(&mut self, freed: &BTreeSet<u32>) -> Result<(), Stop> {
        let mut kept = Vec::with_capacity(self.timelines.parked.len());
        for (key, timeline) in core::mem::take(&mut self.timelines.parked) {
            if freed.contains(&key.pins.number()) {
                crate::invariant::unmoved(&timeline.walks, &timeline.signatures)?;
            } else {
                kept.push((key, timeline));
            }
        }
        self.timelines.parked = kept;
        Ok(())
    }

    /// Asks move no money in every timeline the engine holds: the one in force and each
    /// parked (E §9 item 4).
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT asks move no money: …` for the first walk whose entries changed.
    pub(crate) fn asks_moved_no_money(&self) -> Result<(), Stop> {
        crate::invariant::unmoved(&self.walks, &self.signatures)?;
        for (_, timeline) in &self.timelines.parked {
            crate::invariant::unmoved(&timeline.walks, &timeline.signatures)?;
        }
        Ok(())
    }

    /// The booked plan amounts of the reality's own timeline, handed on to the run's next ask
    /// (E §9 item 5): the timelines of another world or of pins are its ask's alone.
    pub fn take_booked(&mut self) -> BTreeMap<(String, Day), Booked> {
        if self.timelines.now == TimelineKey::BASE {
            return core::mem::take(&mut self.booked);
        }
        let own = self
            .timelines
            .parked
            .iter_mut()
            .find(|(key, _)| *key == TimelineKey::BASE);
        match own {
            Some((_, timeline)) => core::mem::take(&mut timeline.booked),
            // the reality's own timeline was never walked by this ask: nothing it booked
            None => BTreeMap::new(),
        }
    }

    /// Puts the timeline of `key` in force, the one in force parked under its own key.
    fn use_timeline(&mut self, key: TimelineKey) {
        if key == self.timelines.now {
            return;
        }
        let out = self.park();
        let held = self.timelines.now;
        let mut next = None;
        let mut kept = Vec::with_capacity(self.timelines.parked.len());
        for (k, t) in core::mem::take(&mut self.timelines.parked) {
            if next.is_none() && k == key {
                next = Some(t);
            } else {
                kept.push((k, t));
            }
        }
        kept.push((held, out));
        self.timelines.parked = kept;
        self.unpark(next.unwrap_or_else(Timeline::fresh));
        self.timelines.now = key;
    }

    /// The walk fields' state as a timeline, the fields left empty.
    fn park(&mut self) -> Timeline {
        Timeline {
            each_refused: core::mem::take(&mut self.each_refused),
            walks: core::mem::take(&mut self.walks),
            main_walks: core::mem::take(&mut self.main_walks),
            walking: core::mem::take(&mut self.walking),
            plans: core::mem::take(&mut self.plans),
            plan_active: core::mem::take(&mut self.plan_active),
            adjustments: core::mem::take(&mut self.adjustments),
            adjust_active: core::mem::take(&mut self.adjust_active),
            proj_walks: core::mem::take(&mut self.proj_walks),
            trees: core::mem::take(&mut self.trees),
            observed: core::mem::take(&mut self.observed),
            covered: core::mem::take(&mut self.covered),
            links: core::mem::take(&mut self.links),
            link_active: core::mem::take(&mut self.link_active),
            obs_errs: core::mem::take(&mut self.obs_errs),
            settled: core::mem::take(&mut self.settled),
            unsettled: core::mem::take(&mut self.unsettled),
            adjust_all_active: core::mem::take(&mut self.adjust_all_active),
            booked: core::mem::take(&mut self.booked),
            signatures: core::mem::take(&mut self.signatures),
            prefixes: core::mem::take(&mut self.prefixes),
        }
    }

    /// The walk fields set to timeline `t`'s state.
    fn unpark(&mut self, t: Timeline) {
        self.each_refused = t.each_refused;
        self.walks = t.walks;
        self.main_walks = t.main_walks;
        self.walking = t.walking;
        self.plans = t.plans;
        self.plan_active = t.plan_active;
        self.adjustments = t.adjustments;
        self.adjust_active = t.adjust_active;
        self.proj_walks = t.proj_walks;
        self.trees = t.trees;
        self.observed = t.observed;
        self.covered = t.covered;
        self.links = t.links;
        self.link_active = t.link_active;
        self.obs_errs = t.obs_errs;
        self.settled = t.settled;
        self.unsettled = t.unsettled;
        self.adjust_all_active = t.adjust_all_active;
        self.booked = t.booked;
        self.signatures = t.signatures;
        self.prefixes = t.prefixes;
    }
}
