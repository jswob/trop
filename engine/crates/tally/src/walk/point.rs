//! A point in the day walk's order: the day, the step of the day, and the place of an act inside its step.
//! Spec: E §4.2 (the steps of a day: readings at its start, observations, plans, decisions by
//! stratum, readings at its end, the close); L §5.14.2 (the order of a day's decisions).
//! Twin: engine/tools/refcore/walk.py (the walk's keys: `K_start`, `K_end`, `K_before`, and
//! the tuples `prepare`, `run_decisions` and `book_leg` schedule).
//! Invariants: points compare by day, then step, then stratum, then the act's rank and its
//! instance, then the sub-step; a balance read at a point sees every entry at or before it.

use core::cmp::Ordering;

use crate::time::Day;

/// The step of a day an act belongs to (E §4.2 steps 0–5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Before the day's first act: a hole or a refusal that holds from the walk's start.
    Before,
    /// Step 0, the readings at the start of the day.
    Start,
    /// Step 1, the observations.
    Observations,
    /// Step 2, the plans.
    Plans,
    /// Step 3, the decisions.
    Decisions,
    /// Step 4, the readings at the end of the day.
    End,
    /// Step 5, the close.
    Close,
    /// After everything the day holds: the balance at the end of the day.
    After,
}

impl Step {
    /// The step's place in a day's order (E §4.2).
    fn rank(self) -> u8 {
        match self {
            Step::Before => 0,
            Step::Start => 1,
            Step::Observations => 2,
            Step::Plans => 3,
            Step::Decisions => 4,
            Step::End => 5,
            Step::Close => 6,
            Step::After => 7,
        }
    }
}

/// A point of the walk (refcore's walk key).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct At {
    /// The day.
    pub day: Day,
    /// The step of the day.
    pub step: Step,
    /// The stratum of a decision; a reading's group at the start of the day.
    pub stratum: u32,
    /// The act's rank inside its step: a move's place among the moves, a reading's among the
    /// readings.
    pub rank: u32,
    /// The instance's place among its move's instances that day.
    pub n: u32,
    /// The sub-step: an act's own entries at 0, what follows it at its point after.
    pub sub: u32,
}

impl Ord for At {
    /// The walk's order: the day, the step, the stratum, the rank, the instance, the
    /// sub-step (E §4.2, L §5.14.2).
    fn cmp(&self, other: &At) -> Ordering {
        let mine = (
            self.day,
            self.step.rank(),
            self.stratum,
            self.rank,
            self.n,
            self.sub,
        );
        let theirs = (
            other.day,
            other.step.rank(),
            other.stratum,
            other.rank,
            other.n,
            other.sub,
        );
        mine.cmp(&theirs)
    }
}

impl PartialOrd for At {
    /// As [`Ord`]: the walk's order (E §4.2).
    fn partial_cmp(&self, other: &At) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl At {
    /// A point of step `step` on `day`, at the step's first place (E §4.2).
    #[must_use]
    pub fn of(day: Day, step: Step) -> At {
        At {
            day,
            step,
            stratum: 0,
            rank: 0,
            n: 0,
            sub: 0,
        }
    }

    /// Before everything on `day` (refcore's `K_before`). E §4.2.
    #[must_use]
    pub fn before(day: Day) -> At {
        At::of(day, Step::Before)
    }

    /// The start of `day`, after its opening readings (refcore's `K_start`). E §4.2 step 0.
    #[must_use]
    pub fn start(day: Day) -> At {
        At {
            stratum: u32::MAX,
            ..At::of(day, Step::Start)
        }
    }

    /// The end of `day`, after everything it holds (refcore's `K_end`). E §4.2 step 4.
    #[must_use]
    pub fn end(day: Day) -> At {
        At::of(day, Step::After)
    }

    /// The close of `day` (E §4.2 step 5).
    #[must_use]
    pub fn close(day: Day) -> At {
        At::of(day, Step::Close)
    }

    /// The end of the day's first batch: after its readings at the start, observations and
    /// plans (L §5.10, E §4.2 step 5).
    #[must_use]
    pub fn batch_end(day: Day) -> At {
        At {
            stratum: u32::MAX,
            rank: u32::MAX,
            n: u32::MAX,
            sub: u32::MAX,
            ..At::of(day, Step::Plans)
        }
    }

    /// After the readings at the end of `day`, before its close: where they are tested
    /// together (L §5.10 point 3, E §4.2 step 5).
    #[must_use]
    pub fn closing_end(day: Day) -> At {
        At {
            stratum: u32::MAX,
            rank: u32::MAX,
            n: u32::MAX,
            sub: u32::MAX,
            ..At::of(day, Step::End)
        }
    }

    /// The point as a memo key's text: a running read's cells are kept per point (E §3.2).
    #[must_use]
    pub fn text(self) -> String {
        format!(
            "{}.{}.{}.{}.{}.{}",
            self.day.text(),
            self.step.rank(),
            self.stratum,
            self.rank,
            self.n,
            self.sub
        )
    }

    /// The same act's point with sub-step `sub` (E §4.2 step 3: what follows a decision at
    /// its point).
    #[must_use]
    pub fn with_sub(self, sub: u32) -> At {
        At { sub, ..self }
    }

    /// Whether `other` is the same act: the same point but for the sub-step (E §4.2 step 3).
    #[must_use]
    pub fn same_act(self, other: At) -> bool {
        At { sub: 0, ..self } == At { sub: 0, ..other }
    }

    /// Whether the point is in the day's first batch: its readings at the start, its
    /// observations and plans (L §5.10, E §4.2 step 5).
    #[must_use]
    pub fn in_first_batch(self) -> bool {
        self.step.rank() <= Step::Plans.rank()
    }

    /// Whether the point is at or after the day's decisions (E §4.2 step 3).
    #[must_use]
    pub fn at_or_after_decisions(self) -> bool {
        self.step.rank() >= Step::Decisions.rank()
    }

    /// Whether the point is at or after the day's readings at its end (E §4.2 step 4).
    #[must_use]
    pub fn at_or_after_end(self) -> bool {
        self.step.rank() >= Step::End.rank()
    }
}

#[cfg(test)]
mod tests {
    use super::{At, Step};
    use crate::time::Day;

    #[test]
    fn a_day_orders_its_steps_as_the_walk_takes_them() {
        let d = Day::from_ymd(2025, 1, 2).expect("a day");
        let prev = d.previous().expect("a day");
        let order = [
            At::end(prev),
            At::before(d),
            At::of(d, Step::Start),
            At::start(d),
            At::of(d, Step::Plans),
            At::batch_end(d),
            At::of(d, Step::Decisions),
            At::close(d),
            At::end(d),
        ];
        for (a, b) in order.iter().zip(order.iter().skip(1)) {
            assert!(a < b, "{a:?} before {b:?}");
        }
        let p = At {
            n: 2,
            ..At::of(d, Step::Decisions)
        };
        assert!(p.same_act(p.with_sub(2)), "the same act");
        assert!(!p.same_act(At { n: 3, ..p }), "another instance");
        assert!(At::batch_end(d).in_first_batch(), "the batch's end");
        assert!(!p.in_first_batch(), "a decision");
        assert!(p.at_or_after_decisions(), "a decision");
        assert!(At::close(d).at_or_after_end(), "the close");
    }
}
