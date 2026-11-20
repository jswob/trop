//! A reading applied at its instant, and the sign rule's groups: the day's first batch, a decision's legs, the closing readings, and what each refuses.
//! Spec: E §4.2 step 0 (readings at the start of the day), step 4 (readings at the end of the
//! day), step 5 (the sign rule at its points; a reading's own place tested at its point; an
//! anchor books no leg); L §5.10 (the sign rule and its three points; `RUN-PLACE`), §5.19
//! (`opening` always anchors), §4.6 (a reading's instant), §4.4 (a decision dated by a
//! window falls at its last day), §8.4; E §9 item 2 (an opening and a refused leg in the
//! day's worth).
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
use super::instance::{Amount, End, LegVal};
use super::plan::{invariant, qty_value};
use super::point::At;
use super::state::{ALL, Change, Ent, Group, St, Taken, Walk};
use crate::base::{Code, Refusal, Run};
use crate::eval::{Collected, Engine};
use crate::hole::Hole;
use crate::model::Reality;
use crate::num::Num;
use crate::syntax::{Sign, Stop};
use crate::time::Day;

/// The logs where decision instance `mid`'s window left a hole no reading answers, each with
/// the units its hole a reading ends goes to: the log's own, or for a place-wide log the
/// place's units in the walk, else its unit (refcore's `dec_win_end`, its logs). L §5.19 Holes,
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
    /// A reading at its point: an `opening` anchors, a `stated` one pads or anchors by its
    /// version's gap role (refcore's `apply_reading`). E §4.2 steps 0 and 4, E §6.1, L §5.19.
    pub(super) fn apply_reading(
        &mut self,
        m: &Reality,
        id: usize,
        at: At,
        reading: &Reading,
    ) -> Result<(), Stop> {
        if reading.input == "opening" {
            self.walk_mut(id)?.apply_opening(m, at, reading)
        } else {
            self.apply_stated(m, id, at, reading)
        }
    }

    /// The end of the last day of decision `mid`'s window, at point `at`: m has fallen, so the
    /// hole `h` it left its places becomes each unit's, which a reading ends, and stays up to
    /// the place's next reading (refcore's `dec_win_end`; pick P29 on #175). L §5.19
    /// Holes, §4.4, E §4.2 Holes in the walk.
    pub(super) fn window_end(
        &mut self,
        m: &Reality,
        id: usize,
        at: At,
        (mid, h): (&str, &Hole),
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
        Ok(())
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
            // allow (E §4.1 step 2; tally 2.1.8 item 77)
            let nv = match st.known() {
                Some(nv) => Some(nv.clone()),
                None => match self.group_three(m, id, broke, (place, unit), legs, &st)? {
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
        let pending_walk = self.walk_mut(id)?;
        for (place, unit, mut legs) in depends {
            if let Some(pending) = pending_walk.pend_at.get(&(place.clone(), unit.clone())) {
                legs.extend(pending.iter().cloned());
            }
            pending_walk.imp_hole(m, at, (&place, &unit), &legs)?;
        }
        for key in group.keys() {
            pending_walk.pend_at.remove(key);
        }
        for (place, unit, nv, legs) in bad {
            let words = self.refusal_words(m, id, broke, (&place, &unit), &legs, &nv)?;
            let err = Refusal::new(Code::Run(Run::Place), words);
            // the test read the place's balance: what decided it is the refusal's read set,
            // like a value's (L §8.2 One read set; B1 on #241)
            let tested = match self.walk(id)?.state(m, &place, &unit, at).1 {
                Some(i) => self.walk_mut(id)?.cum(&place, &unit, i).deciding(),
                None => Collected::default(),
            };
            if legs.is_empty() {
                // an anchored place with no leg of the group at it: refused from here, with
                // its reading's message (L §8.4; tally 2.1.2)
                let mut ent = Ent::new(at, Change::Bad(err));
                ent.coll = (!tested.entries.is_empty()).then_some(tested);
                self.walk_mut(id)?.add(m, &place, &unit, ent)?;
                continue;
            }
            for (mv, leg) in &legs {
                let walk = self.walk_mut(id)?;
                let key = (mv.clone(), leg.name.clone());
                if walk.bad_legs.contains_key(&key) {
                    continue;
                }
                walk.bad_legs.insert(key, err.clone());
                // the refusal rests on what booked the leg: the pins its movement took stay
                // listed under it (L §8.2 One read set, l.2248; 208.2)
                let mut read = tested.clone();
                if let Some(own) = walk.booked_pins(ends(leg), &unit, (mv, &leg.name)) {
                    read.merge(&own);
                }
                let pins = (!read.entries.is_empty()).then_some(read);
                for p in ends(leg) {
                    walk.take_back(m, p, &unit, (mv, leg), taken)?;
                }
                for p in ends(leg) {
                    let mut ent = Ent::new(at, Change::Bad(err.clone()));
                    ent.mv = Some(mv.clone());
                    ent.coll.clone_from(&pins);
                    walk.add(m, p, &unit, ent)?;
                }
            }
        }
        Ok(())
    }

    /// The sign rule at a group's `place` in `unit`, whose state `st` has window-dated legs
    /// pending, asked of each balance they allow: where every one breaks it, the refusal is the
    /// windows' hole where the bytes it prints differ by the day, whatever its message's form
    /// (`ob takes pot below 0` names no balance; refcore's `words_differ`, its `words`). E §4.1
    /// step 2, §4.2 step 5; L §4.4; tally 2.1.8 item 77, pick P30.
    ///
    /// # Errors
    ///
    /// A number past the cap; `ENGINE-INVARIANT` for an anchored place with no instant.
    fn group_three(
        &self,
        m: &Reality,
        id: usize,
        broke: &Broke,
        (place, unit): (&str, &str),
        legs: &[(String, LegVal)],
        st: &St,
    ) -> Result<Three, Stop> {
        let three = imp_breaks(m, place, st, None)?;
        let (Three::Every, Some((lo, hi))) = (three, st.imp_range()?) else {
            return Ok(three);
        };
        let low = self.refusal_words(m, id, broke, (place, unit), legs, &lo)?;
        let high = self.refusal_words(m, id, broke, (place, unit), legs, &hi)?;
        Ok(if low == high {
            Three::Every
        } else {
            Three::Depends
        })
    }

    /// The bytes the sign rule's refusal at a group's `place` in `unit` prints for balance
    /// `nv`: the group's message for its legs `legs`, or an anchored place's with no leg there
    /// (refcore's `check_group`, its `words`). L §5.10, §8.4; E §4.2 step 5.
    ///
    /// # Errors
    ///
    /// A number past the cap; `ENGINE-INVARIANT` for an anchored place with no instant.
    fn refusal_words(
        &self,
        m: &Reality,
        id: usize,
        broke: &Broke,
        (place, unit): (&str, &str),
        legs: &[(String, LegVal)],
        nv: &Num,
    ) -> Result<String, Stop> {
        let shown = qty_value(m, nv, unit)?
            .text(m)
            .map_err(crate::eval::num_stop)?;
        if !legs.is_empty() {
            return Ok(broke_text(m, broke, place, &shown, legs));
        }
        let instant = self
            .today(id)?
            .anchors
            .get(&(place.to_owned(), unit.to_owned()))
            .cloned()
            .ok_or_else(|| invariant("a place in a group with no leg and no anchor"))?;
        Ok(format!("{place} is {shown} after its reading at {instant}"))
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
    /// from `at` by it, with the pins its movement's evaluation `read` took (refcore's
    /// `bad_legs`, `book_leg`'s refusal, `add`; L §8.2 One read set, 208.2). E §4.2 step 5.
    ///
    /// # Errors
    ///
    /// As [`Walk::add`].
    pub(crate) fn refuse_leg(
        &mut self,
        m: &Reality,
        (movement, lg, read): (&str, &LegVal, &Collected),
        err: &Refusal,
        at: At,
    ) -> Result<(), Stop> {
        self.bad_legs
            .insert((movement.to_owned(), lg.name.clone()), err.clone());
        let unit = lg.unit.clone().unwrap_or_else(|| ALL.to_owned());
        let pins = super::ends::pins_read(read);
        for p in ends(lg) {
            let mut ent = Ent::new(at, Change::Bad(err.clone()));
            ent.mv = Some(movement.to_owned());
            ent.coll.clone_from(&pins);
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
                match imp_breaks(m, p, &st, Some(&flow))? {
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
    /// with no leg, and the group's test refuses it there when it breaks the rule (refcore's
    /// `sign_anchor`; tally 2.1.2). L §5.10 (points 1 and 3), §5.19; E §4.2 step 5.
    ///
    /// # Errors
    ///
    /// `ENGINE-INVARIANT` for a reading outside every group of the sign rule: a reading stands
    /// at its day's start, in the first batch, or at its end, among the closing readings.
    pub(crate) fn sign_anchor(&mut self, m: &Reality, at: At, r: &Reading) -> Result<(), Stop> {
        let ruled = matches!(place_sign(m, &r.place), Some(Sign::Min0 | Sign::Max0));
        if r.value.is_err() || !ruled {
            return Ok(());
        }
        let key = (r.place.clone(), r.unit.clone());
        let group = self
            .open_group(at)
            .ok_or_else(|| invariant("a reading outside every group of the sign rule"))?;
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
    /// Every one does.
    Every,
    /// Some do and some don't, or, at a group, all do in bytes that differ by the day.
    Depends,
}

/// The sign rule at `place` asked of each balance the window-dated legs pending in `st` allow,
/// a leg's `flow` added where one is being booked: the least and the greatest, the rule being
/// monotone in the balance; none where the balance isn't known but for them (refcore's
/// `imp_three`). E §4.1 step 2, §4.2 step 5; L §4.4; tally 2.1.8 item 77.
///
/// # Errors
///
/// A number past the cap.
pub(super) fn imp_breaks(
    m: &Reality,
    place: &str,
    st: &St,
    flow: Option<&Num>,
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
        (true, true) => Three::Every,
        _ => Three::Depends,
    })
}

/// Whether balance `v` breaks its place's sign rule (L §5.10).
fn breaks(m: &Reality, place: &str, v: &Num) -> bool {
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
fn takes_text(m: &Reality, mover: &str, place: &str) -> String {
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
