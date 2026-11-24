//! A leg's ends in the walk: refused or a hole, kept at each place the leg names.
//! Spec: E §4.2 step 5 (a refused leg refuses its places to each one's next reading; holes in
//! the walk); L §8 (Refusals outrank holes, tally 2.1.8), §5.19, §8.1.
//! Twin: engine/tools/refcore/walk.py (`leg_bad`, `pbad`, `act`'s `legq` and `perm`).
//! Invariants: a refused leg is kept at its unit, else its place's, else every unit of a
//! place of unit `any`; one whose day is a hole or refused is ended by no reading.

use super::day::ends;
use super::instance::LegVal;
use super::point::At;
use super::state::{ALL, Change, Ent, Walk};
use crate::base::Refusal;
use crate::eval::Collected;
use crate::model::Reality;
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
        let pins = refusal_read(read);
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
}

/// What a refused leg's movement rests on, `read` what its evaluation met: a refusal rests on
/// the reads that refused it, so its places' entry lists every entry they gave like a value's,
/// its pins, its assumptions and its register's `unfulfilled:` lines, and a pin nothing it
/// read isn't (refcore's `add`, a `bad` entry's `mv.coll`); none where it took none. L §8.2
/// One read set (208.2, N2 and P35 on #241).
pub(super) fn refusal_read(read: &Collected) -> Option<Collected> {
    read.has_entries().then(|| read.clone())
}
