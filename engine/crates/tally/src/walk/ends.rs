//! A leg's ends in the walk: refused, a hole, or a window's, kept at each place the leg names.
//! Spec: E §4.2 step 5 (a refused leg refuses its places to each one's next reading; holes in
//! the walk), §4.1 step 2; L §8 (Refusals outrank holes, tally 2.1.8), §5.19, §4.4 (a leg
//! dated by a window is `HOLE-IMPRECISE` from the window's first day and booked at its last;
//! where its lifespan or sign test comes out one way on some of the window's days and another
//! on the rest, it is the window's hole from the first day; tally 2.1.8 item 77), §5.10,
//! §8.1.
//! Twin: engine/tools/refcore/walk.py (`leg_bad`, `pbad`, `act`'s `legq` and `perm`,
//! `imp_on`, `window_lifespan`, `imp_tests`, `imp_hole`).
//! Invariants: a refused leg is kept at its unit, else its place's, else every unit of a
//! place of unit `any`; one whose day is a hole or refused is ended by no reading; no day of a
//! window is walked to tell whether its lifespan test comes out both ways.

use std::collections::BTreeSet;

use super::day::{ends, signed};
use super::instance::{Amount, Dated, DayWindow, LegVal};
use super::plan::invariant;
use super::point::At;
use super::readings::{Three, imp_breaks};
use super::state::{ALL, Change, Ent, HoleLeg, Walk};
use crate::base::Refusal;
use crate::eval::Collected;
use crate::hole::{Hole, spread};
use crate::model::{MoveInfo, Reality, Target};
use crate::syntax::Stop;

impl Walk {
    /// A refused leg: its places refused from here to each one's next reading; a leg whose
    /// amount is refused has no unit of its own, so it is kept at its place's unit, where that
    /// reading ends it, or at every unit of a place of unit `any` (refcore's `leg_bad`). E §4.2
    /// step 5; L §8 (Refusals outrank holes, tally 2.1.8).
    pub(crate) fn leg_bad(
        &mut self,
        m: &Reality,
        (movement, lg, read): (&str, &LegVal, &Collected),
        err: &Refusal,
        at: At,
    ) -> Result<(), Stop> {
        self.leg_refused(
            m,
            (movement, lg, read),
            &Ent::new(at, Change::Bad(err.clone())),
        )
    }

    /// [`Walk::leg_bad`] of a plan's or an observation's own leg: its refusal reads no value
    /// under a window's hole (a plain movement makes no running reads, L §5.14.2), so every day
    /// gives it and it stands beside one (refcore's `leg_bad`, `own`). L §8 l.2217, §4.4 l.671;
    /// P31 (B4).
    pub(crate) fn leg_bad_own(
        &mut self,
        m: &Reality,
        (movement, lg, read): (&str, &LegVal, &Collected),
        err: &Refusal,
        at: At,
    ) -> Result<(), Stop> {
        let mut ent = Ent::new(at, Change::Bad(err.clone()));
        ent.own = true;
        self.leg_refused(m, (movement, lg, read), &ent)
    }

    /// Whether decision `mv`'s refusal at `at` may rest on a value under a window's hole: a
    /// place its `reads` names whose balance there holds a window's question, or a place it
    /// names by a key or an expression, taken as under it; movements it reads are no place
    /// (P31 said whole, P41: "a decision's, unless a place it names in `reads` is under the
    /// hole"; F6). L §8 l.2217, §4.4 l.671.
    pub(crate) fn reads_under_window(&self, m: &Reality, mv: &MoveInfo, at: At) -> bool {
        for target in &mv.read_targets {
            let place = match target {
                Target::Thing(p) => p,
                Target::PlaceSec(_) | Target::Unknown => return true,
                Target::Move(_) | Target::Kind(_) => continue,
            };
            let Some(thing) = m.things.get(place) else {
                return true;
            };
            let unit = m.place_unit(thing).unwrap_or_else(|| ALL.to_owned());
            let st = self.pstate(m, place, &unit, at);
            if st
                .questions(None)
                .as_ref()
                .and_then(Hole::of_windows)
                .is_some()
            {
                return true;
            }
        }
        false
    }

    /// A refused leg's entry `like`, a dated refusal or one no reading ends, at each place it
    /// names, in its unit or its place's, with the pins its movement's evaluation `read` took:
    /// a refusal rests on the reads that refused it, so a pin nothing it read isn't listed
    /// under it (refcore's `leg_bad`, its kind, and `add`; L §8.2 One read set, 208.2 on
    /// #241). E §4.2 step 5.
    fn leg_refused(
        &mut self,
        m: &Reality,
        (movement, lg, read): (&str, &LegVal, &Collected),
        like: &Ent,
    ) -> Result<(), Stop> {
        let pins = pins_read(read);
        for p in ends(lg) {
            let own = m.things.get(p).and_then(|t| m.place_unit(t));
            let unit = lg
                .unit
                .clone()
                .or(own)
                .filter(|u| u != "any")
                .unwrap_or_else(|| ALL.to_owned());
            let mut ent = like.clone();
            ent.mv = Some(movement.to_owned());
            ent.coll.clone_from(&pins);
            self.add(m, p, &unit, ent)?;
        }
        Ok(())
    }

    /// A refused movement's leg whose day is a hole or refused: it may fall after any reading,
    /// so no reading ends its places' refusal, as a hole day's hole (refcore's `leg_bad`,
    /// `pbad`). L §8, §5.19; E §4.2 Holes in the walk.
    pub(super) fn leg_bad_undated(
        &mut self,
        m: &Reality,
        (movement, lg, read): (&str, &LegVal, &Collected),
        err: &Refusal,
        at: At,
    ) -> Result<(), Stop> {
        self.leg_refused(
            m,
            (movement, lg, read),
            &Ent::new(at, Change::PermBad(err.clone())),
        )
    }

    /// A hole movement's leg: a hole at both its places (refcore's `act`, `legq` and
    /// `perm`). E §4.2 Holes in the walk.
    pub(super) fn hole_ends(
        &mut self,
        m: &Reality,
        (movement, lg): (&str, &LegVal),
        at: At,
        change: &Change,
        coll: &Collected,
    ) -> Result<(), Stop> {
        let unit = lg.unit.clone().unwrap_or_else(|| ALL.to_owned());
        for p in ends(lg) {
            let mut ent = Ent::new(at, change.clone());
            ent.mv = Some(movement.to_owned());
            ent.coll = Some(coll.clone());
            self.add(m, p, &unit, ent)?;
        }
        Ok(())
    }

    /// The window's hole of leg `lg` of `movement`, dated by a window some of whose days are
    /// inside the lifespan of every place it moves into or out of and some aren't, each day
    /// of the window asked; none where every day is inside, or none is, which are the
    /// lifespan's yes and no, or where a lifespan's end is a hole (refcore's
    /// `window_lifespan`). The lifespans are intervals, so every day is inside where the
    /// window's first and last are, and one is where their common first day from the window's
    /// first lies in the window: no day is walked (E §4.1 step 2, §4.2 step 5; L §4.4; tally
    /// 2.1.8 item 77).
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a leg's place load didn't build.
    pub(super) fn lifespan_depends(
        &self,
        m: &Reality,
        (movement, lg): (&str, &LegVal),
    ) -> Result<Option<Hole>, Stop> {
        let Dated::Window(w) = &lg.day else {
            return Ok(None);
        };
        let mut spans = Vec::new();
        for p in ends(lg) {
            let thing = m
                .things
                .get(p)
                .ok_or_else(|| invariant("a leg's place load didn't build"))?;
            // no lifespan: no day is inside, the plain refusal
            let Some(sp) = m.lifespan(thing, (self.cut, self.world)) else {
                return Ok(None);
            };
            if sp.end_question().is_some() {
                return Ok(None);
            }
            spans.push(sp);
        }
        if spans
            .iter()
            .all(|sp| sp.contains(w.lo) && sp.contains(w.hi))
        {
            return Ok(None);
        }
        // the first day from the window's first that every lifespan may hold: the latest of
        // their first days, which holds a day of all of them where any does
        let mut first = Some(w.lo);
        for sp in &spans {
            first = first.and_then(|d| sp.first_from(d));
        }
        let Some(first) = first.filter(|d| *d <= w.hi) else {
            return Ok(None);
        };
        if !spans.iter().all(|sp| sp.contains(first)) {
            return Ok(None);
        }
        let text = format!(
            "date within {} not precise enough for a balance",
            w.question
        );
        Ok(Some(Hole::asked(&text, movement).carrying(&w.value.carry)))
    }

    /// A leg dated by a window, from the window's first day: each place it names is
    /// `HOLE-IMPRECISE` for a balance, its amount's questions too where that is a hole, until
    /// the leg books at the window's last day (refcore's `imp_on`). L §4.4, §8.1; E §4.2.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a leg not dated by a window.
    pub(super) fn imprecise_ends(
        &mut self,
        m: &Reality,
        (movement, lg): (&str, &LegVal),
        at: At,
        coll: &Collected,
    ) -> Result<(), Stop> {
        let Dated::Window(w) = &lg.day else {
            return Err(invariant("an imprecise leg not dated by a window"));
        };
        let text = format!(
            "date within {} not precise enough for a balance",
            w.question
        );
        let own = Hole::asked(&text, movement).carrying(&w.value.carry);
        let amount = match &lg.amount {
            Amount::Hole(h) => Some(h),
            Amount::Qty(_) | Amount::Refused(_) => None,
        };
        let q = match spread(&[Some(&own), amount]) {
            Some(both) => both,
            None => own,
        };
        let unit = lg.unit.clone().unwrap_or_else(|| ALL.to_owned());
        for p in ends(lg) {
            let into = lg.to.place() == Some(p);
            let mut ent = Ent::new(at, Change::ImpOn(q.clone()));
            ent.mv = Some(movement.to_owned());
            ent.window = leg_window(movement, lg);
            ent.coll = Some(coll.clone());
            // the leg's amount, which `flows` reads to tell a zero leg (L §4.4)
            ent.flow = Some(lg.amount.clone());
            ent.signed = match &lg.amount {
                Amount::Qty(a) => Some(signed(a, if into { 1 } else { -1 })?),
                Amount::Hole(_) | Amount::Refused(_) => None,
            };
            self.add(m, p, &unit, ent)?;
        }
        if let Amount::Qty(a) = &lg.amount
            && !a.is_zero()
            && let Some(h) = self.lifespan_depends(m, (movement, lg))?
        {
            // its lifespan, asked of each of the window's days, comes out both ways: the leg
            // is the window's hole at both ends from here, never booked (refcore's
            // `imp_tests`, `window_lifespan`; E §4.1 step 2; tally 2.1.8 item 77)
            return self.window_hole_ends(m, (movement, lg), at, (&h, coll));
        }
        for p in ends(lg) {
            self.imp_test(m, (p, &unit), at, (movement, lg))?;
        }
        Ok(())
    }

    /// Leg `lg` of `movement` as the window's hole `h` at both its ends from point `at`, where
    /// its lifespan test comes out one way on some of the window's days and another on the
    /// rest: a read of the leg meets the hole, and a refusal reaching a place after it doesn't
    /// outrank it (refcore's `window_lifespan`). E §4.1 step 2, L §4.4; tally 2.1.8 item 77.
    ///
    /// # Errors
    ///
    /// A number error.
    fn window_hole_ends(
        &mut self,
        m: &Reality,
        (movement, lg): (&str, &LegVal),
        at: At,
        (h, coll): (&Hole, &Collected),
    ) -> Result<(), Stop> {
        self.open_tests = spread(&[self.open_tests.as_ref(), Some(h)]);
        self.hole_legs
            .entry((movement.to_owned(), lg.name.clone()))
            .or_insert_with(|| h.clone());
        let unit = lg.unit.clone().unwrap_or_else(|| ALL.to_owned());
        self.worth_add(&unit, None)?;
        for (end, into) in [(&lg.from, false), (&lg.to, true)] {
            let Some(p) = end.place() else {
                continue;
            };
            let mut ent = Ent::new(at, Change::Hole(h.clone()));
            ent.mv = Some(movement.to_owned());
            ent.leg = Some(lg.name.clone());
            ent.hole_leg = Some(HoleLeg {
                into,
                src: lg.src.clone(),
            });
            ent.window = leg_window(movement, lg);
            ent.coll = Some(coll.clone());
            self.add(m, p, &unit, ent)?;
        }
        Ok(())
    }

    /// The sign rule at `place` in `unit` from a window-dated leg's first day: tested in its
    /// point's group where one is open, the place joining it with no leg; else here, asked of
    /// each balance the pending legs allow, the place the windows' hole where some break the
    /// rule and some don't (refcore's `imp_tests`). E §4.1 step 2, §4.2 step 5; L §5.10, §4.4;
    /// tally 2.1.8 item 77.
    ///
    /// # Errors
    ///
    /// A number past the cap; `ENGINE-INVARIANT` for a place load didn't build.
    fn imp_test(
        &mut self,
        m: &Reality,
        (place, unit): (&str, &str),
        at: At,
        (movement, lg): (&str, &LegVal),
    ) -> Result<(), Stop> {
        let key = (place.to_owned(), unit.to_owned());
        let leg = (movement.to_owned(), lg.clone());
        if let Some(group) = self.open_group(at) {
            group.entry(key.clone()).or_insert_with(Vec::new);
            // tested with its group: where its own amount breaks the place, a leg the refusal
            // reaches (refcore's `pend_at`)
            self.pend_at.entry(key).or_insert_with(Vec::new).push(leg);
            return Ok(());
        }
        let st = self.pstate(m, place, unit, at);
        if imp_breaks(m, place, &st, None)? == Three::Depends {
            self.imp_hole(m, at, (place, unit), &[leg])?;
        }
        Ok(())
    }

    /// A test of E §4.2 step 5 that comes out one way on some days of the windows pending at
    /// `place` in `unit` and another on the rest: their `HOLE-IMPRECISE`, naming them, as far
    /// as the refusal would reach, each place to its next reading: the place and the other
    /// ends of the legs `legs` it would refuse with it (a window-dated leg tested on its first
    /// day among them); a read of each of those legs is the hole too. A pending leg the
    /// refusal wouldn't refuse keeps its value at its other end and in its flows (refcore's
    /// `imp_hole`). E §4.1 step 2, L §4.4; tally 2.1.8 item 77.
    ///
    /// # Errors
    ///
    /// A number past the cap; `ENGINE-INVARIANT` for a place load didn't build.
    pub(super) fn imp_hole(
        &mut self,
        m: &Reality,
        at: At,
        (place, unit): (&str, &str),
        legs: &[(String, LegVal)],
    ) -> Result<(), Stop> {
        let st = self.pstate(m, place, unit, at);
        let Some(h) = st.imp_hole() else {
            return Ok(());
        };
        self.open_tests = spread(&[self.open_tests.as_ref(), Some(&h)]);
        let mut reach: BTreeSet<String> = BTreeSet::new();
        reach.insert(place.to_owned());
        for (mv, lg) in legs {
            reach.extend(ends(lg).into_iter().map(str::to_owned));
            self.hole_legs
                .entry((mv.clone(), lg.name.clone()))
                .or_insert_with(|| h.clone());
        }
        for p in &reach {
            self.add(m, p, unit, Ent::new(at, Change::Hole(h.clone())))?;
        }
        Ok(())
    }
}

/// What decided a refused leg's movement, `read` what its evaluation met: a refusal rests on
/// the reads that refused it, so its places' entry lists them like a value's, its pins and
/// its assumptions, and a pin nothing it read isn't (refcore's `add`, a `bad` entry's
/// `mv.coll`); none where it took none. L §8.2 One read set (208.2, N2 and P35 on #241).
pub(super) fn pins_read(read: &Collected) -> Option<Collected> {
    let pins = read.deciding();
    (!pins.entries.is_empty()).then_some(pins)
}

/// A leg's window id, `<movement>.<leg>`, and its window, where its day is one (refcore's
/// `wid`, `win`). L §4.4.
pub(super) fn leg_window(movement: &str, lg: &LegVal) -> Option<(String, DayWindow)> {
    match &lg.day {
        Dated::Window(w) => Some((format!("{movement}.{}", lg.name), w.clone())),
        Dated::Day(_) | Dated::Hole(_) => None,
    }
}
