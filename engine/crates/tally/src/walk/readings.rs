//! A reading applied at its instant, and the sign rule's groups: the day's first batch, a decision's legs, the closing readings, and what each refuses.
//! Spec: E §4.2 step 0 (readings at the start of the day), step 3 (a reading `after m`), step 4
//! (readings at the end of the day), step 5 (the sign rule at its points; a reading's own place
//! tested at its point; an anchor books no leg); L §5.10 (the sign rule and its three points;
//! `RUN-PLACE`), §5.19 (`opening` always anchors), §4.6 (a reading's instant), §8.4; E §9
//! item 2 (an opening and a refused leg in the day's worth).
//! Twin: engine/tools/refcore/walk.py (`apply_reading`, its opening; `check_batch`,
//! `check_group`, `reading_group`, `sign_anchor`, `unbook`, `book_leg`'s groups).
//! Invariants: a leg booked in a group is tested with the group, at the balance the group
//! leaves; every place of a group is tested before any leg is taken back, so the order of
//! places changes nothing; a refused leg is taken back at both its ends and refuses its places
//! from the group's point; an anchored place with no leg in the group is refused with its
//! reading's message.

use std::collections::BTreeSet;

use super::agenda::Reading;
use super::day::ends;
use super::plan::{Amount, DayWindow, End, LegVal, Status, invariant, qty_value};
use super::point::At;
use super::state::{ALL, Change, Ent, Group, St, Taken, Walk};
use crate::base::{Code, Refusal, Run};
use crate::eval::Engine;
use crate::hole::Hole;
use crate::model::{GapRole, Reality};
use crate::num::Num;
use crate::syntax::{Sign, Stop};
use crate::time::Day;

/// The sub-step of a decision's point where the readings `after` it stand: after its legs
/// (0), the observations booked there (1) and its group's test (2) (refcore's `key[4] + 5`).
/// E §4.2 step 3.
const AFTER_SUB: u32 = 5;

/// The logs where decision instance `mid`'s window left a hole no reading answers, each with
/// the units its hole a reading ends goes to: the log's own, or for a place-wide log the
/// place's units in the walk, else its unit (refcore's `dec_win_end`, its logs). L §4.6,
/// E §4.2 Holes in the walk.
fn fallen_places(m: &Reality, walk: &Walk, mid: &str) -> Vec<(String, String, Vec<String>)> {
    let mut fallen = Vec::new();
    for ((place, unit), log) in &walk.logs {
        let held = log
            .ents
            .iter()
            .any(|e| matches!(e.change, Change::Perm(_)) && e.mv.as_deref() == Some(mid));
        if !held {
            continue;
        }
        let mut units: Vec<String> = Vec::new();
        if unit == ALL {
            let others = walk.logs.keys().filter(|(p, u)| p == place && u != ALL);
            units.extend(others.map(|(_, u)| u.clone()));
            if units.is_empty()
                && let Some(thing) = m.things.get(place)
            {
                units.extend(m.place_unit(thing));
            }
        } else {
            units.push(unit.clone());
        }
        fallen.push((place.clone(), unit.clone(), units));
    }
    fallen
}

/// Whether anything but decision instance `mid` touched reading `r`'s place inside `mid`'s
/// window `w`, in its unit or every unit, so the balance after it would differ by m's day
/// (pick P29 on #175; refcore's `dec_win_end`, its `others`). L §4.6, §4.4.
fn touched_inside(walk: &Walk, r: &Reading, (mid, w): (&str, &DayWindow)) -> bool {
    for unit in [r.unit.as_str(), ALL] {
        for e in walk.entries(&r.place, unit) {
            if (w.lo..=w.hi).contains(&e.at.day)
                && e.mv.as_deref() != Some(mid)
                && !matches!(e.change, Change::Unperm(_))
            {
                return true;
            }
        }
    }
    false
}

/// How a refusal of the sign rule names what broke it (refcore's `check_group` messages).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Broke {
    /// The day's first batch.
    Batch(Day),
    /// A decision's legs, by its instance.
    Decision(String),
    /// The closing readings.
    Closing,
}

impl Engine {
    /// A reading at its point, once per unit it stands for: an `opening` anchors, a `stated`
    /// one pads or anchors by its version's gap role (refcore's `apply_reading`). E §4.2 steps
    /// 0 and 4, E §6.1, L §5.19.
    pub(super) fn apply_reading(
        &mut self,
        m: &Reality,
        id: usize,
        at: At,
        reading: &Reading,
    ) -> Result<(), Stop> {
        // a hole that states no unit stands for each unit its place holds here (pick P22, #173)
        for each in crate::book::per_unit(self.walk(id)?, reading, at) {
            if each.input == "opening" {
                self.walk_mut(id)?.apply_opening(m, at, &each)?;
            } else {
                self.apply_stated(m, id, at, &each)?;
            }
        }
        Ok(())
    }

    /// The readings `after m` that wait on decision run `ix`, applied right after it at its
    /// point `key`, after its group is tested, each tested as it is booked: where m is booked
    /// before the cut, and where m is modelled past and observations booked at its point stand
    /// in for it; where m is modelled past and nothing fulfils it, the reading's place is
    /// refused from there (`RUN-RECORD`); none where m doesn't exist, is a hole, uncertain or
    /// refused (refcore's `after_mv`, `run_one`'s `_after`). E §4.2 steps 3 and 5, L §4.6
    /// Readings, §5.10 point 2, §5.19.
    pub(super) fn readings_after(
        &mut self,
        m: &Reality,
        id: usize,
        ix: usize,
        key: At,
    ) -> Result<(), Stop> {
        let walk = self.walk(id)?;
        if walk.after.is_empty() {
            return Ok(());
        }
        let run = walk
            .runs
            .get(ix)
            .ok_or_else(|| invariant("a run just made"))?;
        let (mid, status, linked) = (
            run.inst.id.clone(),
            run.inst.status,
            !run.inst.obs.is_empty(),
        );
        let Some(waiting) = self.walk_mut(id)?.after.remove(&mid) else {
            return Ok(());
        };
        let at = key.with_sub(AFTER_SUB);
        for r in waiting {
            match status {
                Status::Past => self.apply_after(m, id, at, &r)?,
                Status::Modelled if linked => self.apply_after(m, id, at, &r)?,
                Status::Modelled => {
                    let why = "is modelled past and nothing fulfils it";
                    self.after_missing(m, id, at, (&r, &mid, why))?;
                }
                Status::None => self.after_missing(m, id, at, (&r, &mid, "doesn't exist"))?,
                Status::Open
                | Status::LeftOut
                | Status::Refused
                | Status::Hole
                | Status::Decision
                | Status::Uncertain
                | Status::Plan => {}
            }
        }
        Ok(())
    }

    /// The end of the last day of decision `mid`'s window `w`, at point `at`: m has fallen,
    /// so the hole `h` it left its places becomes each unit's, which a reading ends; a reading
    /// `after` m before the cut then stands where nothing else touched its place inside the
    /// window, the balance being the reading plus what is booked since, the same for every
    /// day of the window; else the hole stays up to the place's next reading (refcore's
    /// `dec_win_end`; pick P29 on #175). L §4.6 Readings, §4.4, E §4.2 Holes in the walk.
    pub(super) fn window_end(
        &mut self,
        m: &Reality,
        id: usize,
        at: At,
        (mid, w, h): (&str, &DayWindow, &Hole),
    ) -> Result<(), Stop> {
        let fallen = fallen_places(m, self.walk(id)?, mid);
        let walk = self.walk_mut(id)?;
        for (place, unit, units) in fallen {
            let mut back = Ent::new(at, Change::Unperm(h.clone()));
            back.mv = Some(mid.to_owned());
            walk.add(m, &place, &unit, back)?;
            for each in units {
                let mut ent = Ent::new(at, Change::Hole(h.clone()));
                ent.mv = Some(mid.to_owned());
                ent.flow = Some(Amount::Hole(h.clone()));
                walk.add(m, &place, &each, ent)?;
            }
        }
        if w.hi >= walk.cut {
            return Ok(());
        }
        let Some(waiting) = walk.after.remove(mid) else {
            return Ok(());
        };
        let mut kept = Vec::new();
        for r in waiting {
            if touched_inside(self.walk(id)?, &r, (mid, w)) {
                kept.push(r);
            } else {
                self.apply_after(m, id, at.with_sub(1), &r)?;
            }
        }
        if !kept.is_empty() {
            self.walk_mut(id)?.after.insert(mid.to_owned(), kept);
        }
        Ok(())
    }

    /// A reading `after m` where m doesn't exist, or is modelled past with nothing standing in
    /// for it: `RUN-RECORD` at its place from m's point `at` on, and at the other place of a
    /// pad, whose pad is a refused movement there (refcore's `after_missing`). L §4.6
    /// Readings, §8.4.
    fn after_missing(
        &mut self,
        m: &Reality,
        id: usize,
        at: At,
        (r, mid, why): (&Reading, &str, &str),
    ) -> Result<(), Stop> {
        let err = Refusal::new(
            Code::Run(Run::Record),
            format!(
                "reading {}[{}] {}: {mid} {why}",
                r.input, r.place, r.instant
            ),
        );
        let mut places = vec![r.place.clone()];
        if let Some(version) = m.versions.get(&r.version)
            && let Some((_, GapRole::Pad(other))) =
                version.gap_roles.iter().find(|(p, _)| *p == r.place)
        {
            places.push(other.clone());
        }
        let walk = self.walk_mut(id)?;
        for place in places {
            walk.add(m, &place, &r.unit, Ent::new(at, Change::Bad(err.clone())))?;
        }
        Ok(())
    }

    /// A reading `after m` at point `at`, its point kept for its balance line (refcore's
    /// `after_keys`). E §6.3 Grouping, L §4.6.
    fn apply_after(&mut self, m: &Reality, id: usize, at: At, r: &Reading) -> Result<(), Stop> {
        let key = (r.place.clone(), r.instant.clone(), r.version.clone());
        self.walk_mut(id)?.after_at.insert(key, at);
        self.apply_reading(m, id, at, r)
    }

    /// The sign rule after the day's first batch, its legs taken together (refcore's
    /// `check_batch`). L §5.10, E §4.2 step 5.
    pub(super) fn check_batch(&mut self, m: &Reality, id: usize, day: Day) -> Result<(), Stop> {
        let Some(batch) = self.today_mut(id)?.batch.take() else {
            return Ok(());
        };
        self.check_group(
            m,
            id,
            &batch,
            At::batch_end(day),
            Taken::Batch(day),
            &Broke::Batch(day),
        )
    }

    /// The sign rule at one of its points: each place tested against the balance the group
    /// leaves, and a place that breaks its kind's rule refused from there with the group's
    /// legs at it, which are taken back; an anchored place with no leg there is refused with
    /// its reading's message (refcore's `check_group`). L §5.10, §8.4, E §4.2 step 5.
    pub(super) fn check_group(
        &mut self,
        m: &Reality,
        id: usize,
        group: &Group,
        at: At,
        taken: Taken,
        broke: &Broke,
    ) -> Result<(), Stop> {
        let mut bad = Vec::new();
        let mut depends = Vec::new();
        for ((place, unit), legs) in group {
            let st = self.walk(id)?.pstate(m, place, unit, at);
            // legs dated by a window may still fall here: the test asked of each balance they
            // allow, a refusal whose words differ by the day the window's hole (E §4.1 step 2;
            // tally 2.1.8 item 77, pick P30)
            let nv = match st.known() {
                Some(nv) => Some(nv.clone()),
                None => match imp_breaks(m, place, &st, None, Words::Printed)? {
                    Three::Depends => {
                        depends.push((place.clone(), unit.clone(), legs.clone()));
                        None
                    }
                    Three::Every => st.val.clone(),
                    Three::None => None,
                },
            };
            if let Some(nv) = nv
                && breaks(m, place, &nv)
            {
                bad.push((place.clone(), unit.clone(), nv, legs.clone()));
            }
        }
        for (place, unit, legs) in depends {
            self.walk_mut(id)?.imp_hole(m, at, (&place, &unit), &legs)?;
        }
        for (place, unit, nv, legs) in bad {
            let shown = qty_value(m, &nv, &unit)?
                .text(m)
                .map_err(crate::eval::num_stop)?;
            if legs.is_empty() {
                // an anchored place with no leg of the group at it: refused from here, with
                // its reading's message (L §8.4; tally 2.1.2)
                let instant = self
                    .today(id)?
                    .anchors
                    .get(&(place.clone(), unit.clone()))
                    .cloned()
                    .ok_or_else(|| invariant("a place in a group with no leg and no anchor"))?;
                let err = Refusal::new(
                    Code::Run(Run::Place),
                    format!("{place} is {shown} after its reading at {instant}"),
                );
                self.walk_mut(id)?
                    .add(m, &place, &unit, Ent::new(at, Change::Bad(err)))?;
                continue;
            }
            let err = Refusal::new(
                Code::Run(Run::Place),
                broke_text(m, broke, &place, &shown, &legs),
            );
            for (mv, leg) in &legs {
                let walk = self.walk_mut(id)?;
                let key = (mv.clone(), leg.name.clone());
                if walk.bad_legs.contains_key(&key) {
                    continue;
                }
                walk.bad_legs.insert(key, err.clone());
                for p in ends(leg) {
                    walk.take_back(m, p, &unit, (mv, leg), taken)?;
                }
                for p in ends(leg) {
                    let mut ent = Ent::new(at, Change::Bad(err.clone()));
                    ent.mv = Some(mv.clone());
                    walk.add(m, p, &unit, ent)?;
                }
            }
        }
        Ok(())
    }

    /// Puts a leg's booked ends in the group of the sign rule it waits for: the day's first
    /// batch, the running decision's legs, or the closing readings; false when none is open
    /// (refcore's `book_leg`, its groups). L §5.10, E §4.2 step 5.
    pub(super) fn group_legs(
        &mut self,
        id: usize,
        at: At,
        movement: &str,
        lg: &LegVal,
        on: &[(String, i64)],
        unit: &str,
    ) -> Result<bool, Stop> {
        let Some(group) = self.walk_mut(id)?.open_group(at) else {
            return Ok(false);
        };
        for (p, _) in on {
            group
                .entry((p.clone(), unit.to_owned()))
                .or_insert_with(Vec::new)
                .push((movement.to_owned(), lg.clone()));
        }
        Ok(true)
    }
}

impl Walk {
    /// The group of the sign rule a booking at `at` waits for: the day's first batch, the
    /// running decision's legs, or the closing readings; none when none is open (refcore's
    /// `reading_group`, `book_leg`'s groups). L §5.10, E §4.2 step 5.
    pub(crate) fn open_group(&mut self, at: At) -> Option<&mut Group> {
        let in_batch =
            at.in_first_batch() && self.today.as_ref().is_some_and(|t| t.batch.is_some());
        if in_batch {
            return self.today.as_mut().and_then(|t| t.batch.as_mut());
        }
        if self.group.is_some() {
            return self.group.as_mut();
        }
        if at.at_or_after_end() {
            return self.today.as_mut().and_then(|t| t.closing.as_mut());
        }
        None
    }

    /// A leg refused as it is booked: kept among the refused legs, and its places refused
    /// from `at` by it (refcore's `bad_legs`, `book_leg`'s refusal). E §4.2 step 5.
    ///
    /// # Errors
    ///
    /// As [`Walk::add`].
    pub(crate) fn refuse_leg(
        &mut self,
        m: &Reality,
        (movement, lg): (&str, &LegVal),
        err: &Refusal,
        at: At,
    ) -> Result<(), Stop> {
        self.bad_legs
            .insert((movement.to_owned(), lg.name.clone()), err.clone());
        let unit = lg.unit.clone().unwrap_or_else(|| ALL.to_owned());
        for p in ends(lg) {
            let mut ent = Ent::new(at, Change::Bad(err.clone()));
            ent.mv = Some(movement.to_owned());
            self.add(m, p, &unit, ent)?;
        }
        Ok(())
    }

    /// A leg booked outside every group of the sign rule, tested as it is booked: each end
    /// `on` names, with its sign, against its balance at `at` with the leg; the first that
    /// breaks its kind's rule refuses the leg (`RUN-PLACE <movement> takes <place> …`); an end
    /// whose balance isn't known isn't tested (refcore's `book_leg`, no group). L §5.10 point
    /// 2, E §4.2 step 5.
    ///
    /// # Errors
    ///
    /// A number error.
    pub(super) fn tested_alone(
        &self,
        m: &Reality,
        (movement, q): (&str, &Num),
        on: &[(String, i64)],
        unit: &str,
        at: At,
    ) -> Result<Alone, Stop> {
        for (p, sign) in on {
            let st = self.pstate(m, p, unit, at);
            let flow = super::day::signed(q, *sign)?;
            let Some(v) = st.known() else {
                // legs dated by a window may still fall here: the test asked of each balance
                // they allow; its refusal names no balance, so its words are every day's
                // (tally 2.1.8 item 77)
                match imp_breaks(m, p, &st, Some(&flow), Words::Same)? {
                    Three::Depends => return Ok(Alone::Depends(p.clone())),
                    Three::Every => {
                        return Ok(Alone::Refused(Refusal::new(
                            Code::Run(Run::Place),
                            takes_text(m, movement, p),
                        )));
                    }
                    Three::None => continue,
                }
            };
            let moved = v.checked_add(&flow).map_err(crate::eval::num_stop)?;
            if breaks(m, p, &moved) {
                return Ok(Alone::Refused(Refusal::new(
                    Code::Run(Run::Place),
                    takes_text(m, movement, p),
                )));
            }
        }
        Ok(Alone::Passes)
    }

    /// An anchored reading's place, an opening's included, tested at the reading's point by
    /// its kind's sign rule: a number at a `min 0` or `max 0` place joins the point's group
    /// with no leg, and the group's test refuses it there when it breaks the rule; a reading
    /// after a decision, where no group is open, is tested as it is booked (refcore's
    /// `sign_anchor`; tally 2.1.2). L §5.10 (point 2), §5.19; E §4.2 step 5.
    ///
    /// # Errors
    ///
    /// A number error.
    pub(crate) fn sign_anchor(&mut self, m: &Reality, at: At, r: &Reading) -> Result<(), Stop> {
        let ruled = matches!(place_sign(m, &r.place), Some(Sign::Min0 | Sign::Max0));
        let Ok(value) = &r.value else {
            return Ok(());
        };
        if !ruled {
            return Ok(());
        }
        let key = (r.place.clone(), r.unit.clone());
        let Some(group) = self.open_group(at) else {
            if !breaks(m, &r.place, value) {
                return Ok(());
            }
            let shown = qty_value(m, value, &r.unit)?
                .text(m)
                .map_err(crate::eval::num_stop)?;
            let err = Refusal::new(
                Code::Run(Run::Place),
                format!("{} is {shown} after its reading at {}", r.place, r.instant),
            );
            return self.add(m, &r.place, &r.unit, Ent::new(at, Change::Bad(err)));
        };
        group.entry(key.clone()).or_insert_with(Vec::new);
        self.today
            .as_mut()
            .ok_or_else(|| invariant("a reading with no day open"))?
            .anchors
            .insert(key, r.instant.clone());
        Ok(())
    }

    /// An opening reading: its place's balance set to it, the holes since answered, a stated
    /// hole's questions kept; it always anchors, so its place is tested at its point (refcore's
    /// `apply_reading`, an opening). L §5.19, §5.10, E §4.2 step 0.
    fn apply_opening(&mut self, m: &Reality, at: At, reading: &Reading) -> Result<(), Stop> {
        if self.inside(m, &reading.place) {
            // E §9 item 2: an opening states the place's worth; its change counts as stated
            let expected = match &reading.value {
                Ok(v) => {
                    let prev = self.pstate(m, &reading.place, &reading.unit, at);
                    match (&prev.val, prev.known()) {
                        (None, _) => Some(v.clone()),
                        (Some(_), Some(p)) => {
                            Some(v.checked_sub(p).map_err(crate::eval::num_stop)?)
                        }
                        (Some(_), None) => None,
                    }
                }
                Err(_) => None,
            };
            self.worth_add(&reading.unit, expected.as_ref())?;
        }
        let change = match &reading.value {
            Ok(v) => Change::Reset {
                val: v.clone(),
                qs: None,
            },
            Err(h) => Change::Reset {
                val: Num::zero(),
                qs: Some(h.clone()),
            },
        };
        let mut ent = Ent::new(at, change);
        ent.coll = Some(reading.coll.clone());
        self.add(m, &reading.place, &reading.unit, ent)?;
        self.sign_anchor(m, at, reading)
    }

    /// Takes back a refused leg's entries at its end `p` (refcore's `unbook`); what came back
    /// across the boundary leaves the day's worth (E §9 item 2). E §4.2 step 5.
    fn take_back(
        &mut self,
        m: &Reality,
        p: &str,
        unit: &str,
        (movement, leg): (&str, &LegVal),
        taken: Taken,
    ) -> Result<(), Stop> {
        let back = self.unbook(m, p, unit, (movement, &leg.name), taken)?;
        let other = if leg.from.place() == Some(p) {
            &leg.to
        } else {
            &leg.from
        };
        let within = matches!(other, End::Place(o) if self.inside(m, o));
        if self.inside(m, p) && !within {
            let undone = back.checked_neg().map_err(crate::eval::num_stop)?;
            self.worth_add(unit, Some(&undone))?;
        }
        Ok(())
    }
}

/// A place's kind's sign rule (L §5.10).
pub(super) fn place_sign(m: &Reality, place: &str) -> Option<Sign> {
    let base = m.things.get(place)?.base.as_deref()?;
    m.kind_sign(base)
}

/// What a leg tested alone meets (refcore's `book_leg`, its test outside a group). L §5.10,
/// E §4.2 step 5.
pub(super) enum Alone {
    /// The leg is booked.
    Passes,
    /// The leg is refused.
    Refused(Refusal),
    /// Some of a pending window's days refuse it and some don't: the place named is the
    /// window's hole (tally 2.1.8 item 77).
    Depends(String),
}

/// A three-valued test (E §4.1 step 2; refcore's `three`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Three {
    /// No balance the pending legs allow breaks the rule.
    None,
    /// Every one does, in the same words.
    Every,
    /// Some do and some don't, or all do in words that differ by the day.
    Depends,
}

/// Whether a refusal prints the balance it found, so its words differ where the balances do
/// (pick P30: the answer is what prints).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Words {
    /// It prints the balance.
    Printed,
    /// It doesn't.
    Same,
}

/// The sign rule at `place` asked of each balance the window-dated legs pending in `st` allow,
/// a leg's `flow` added where one is being booked: the least and the greatest, the rule being
/// monotone in the balance; none where the balance isn't known but for them (refcore's
/// `imp_three`, `words_differ`). E §4.1 step 2, §4.2 step 5; L §4.4; tally 2.1.8 item 77.
///
/// # Errors
///
/// A number past the cap.
pub(super) fn imp_breaks(
    m: &Reality,
    place: &str,
    st: &St,
    flow: Option<&Num>,
    words: Words,
) -> Result<Three, Stop> {
    let Some((mut lo, mut hi)) = st.imp_range()? else {
        return Ok(Three::None);
    };
    if let Some(f) = flow {
        lo = lo.checked_add(f).map_err(crate::eval::num_stop)?;
        hi = hi.checked_add(f).map_err(crate::eval::num_stop)?;
    }
    Ok(match (breaks(m, place, &lo), breaks(m, place, &hi)) {
        (false, false) => Three::None,
        (true, true) if words == Words::Same || lo == hi => Three::Every,
        _ => Three::Depends,
    })
}

/// Whether balance `v` breaks its place's sign rule (L §5.10).
pub(crate) fn breaks(m: &Reality, place: &str, v: &Num) -> bool {
    match place_sign(m, place) {
        Some(Sign::Min0) => v.is_negative(),
        Some(Sign::Max0) => !v.is_negative() && !v.is_zero(),
        Some(Sign::Signed) | None => false,
    }
}

/// ` units` for a holding: its balance is units, not money (refcore's message). L §5.10.
fn units_word(m: &Reality, place: &str) -> &'static str {
    let holds = m
        .things
        .get(place)
        .and_then(|t| t.base.as_ref())
        .is_some_and(|b| m.subtree("holdings").contains(b));
    if holds { " units" } else { "" }
}

/// `<mover> takes <place> below 0`, ` units` after it for a holding, or `above 0`: a leg
/// the sign rule refuses where it is tested alone or with a decision's (refcore's
/// `check_group`, `book_leg` messages). L §5.10.
pub(crate) fn takes_text(m: &Reality, mover: &str, place: &str) -> String {
    let side = match place_sign(m, place) {
        Some(Sign::Max0) => "above 0".to_owned(),
        Some(Sign::Min0 | Sign::Signed) | None => format!("below 0{}", units_word(m, place)),
    };
    format!("{mover} takes {place} {side}")
}

/// The message of a refusal of the sign rule (refcore's `check_group` messages). L §5.10.
fn broke_text(
    m: &Reality,
    broke: &Broke,
    place: &str,
    shown: &str,
    legs: &[(String, LegVal)],
) -> String {
    match broke {
        Broke::Batch(day) => format!(
            "{place} is {shown} after the day's observations and plans of {}",
            day.text()
        ),
        Broke::Decision(run) => takes_text(m, run, place),
        Broke::Closing => {
            let ids: BTreeSet<&str> = legs.iter().map(|(mv, _)| mv.as_str()).collect();
            let all: Vec<&str> = ids.into_iter().collect();
            let plain = match place_sign(m, place) {
                Some(Sign::Max0) => "above 0",
                Some(Sign::Min0 | Sign::Signed) | None => "below 0",
            };
            format!("{} takes {place} {plain}", all.join(", "))
        }
    }
}
