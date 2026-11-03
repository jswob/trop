//! The walk's day source: the event walk's days with an act, or the oracle's every day from S to E through the same steps, a test oracle never shipped.
//! Spec: P §2.2 (the event-driven walk steps only the days that hold work; the oracle, a
//! step-every-day walk built from the same step functions, must print the same bytes);
//! E §4.2 (the day walk).
//! Twin: engine/tools/refcore/walk.py (`run_to`: every day from S to the last one asked).
//! Invariants: both sources step the days in order from S; the oracle steps every day to E;
//! the CLI's runs use the event walk, only the oracle's test the every-day one.

use crate::time::Day;

/// Which days the walk steps (P §2.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Days {
    /// The event walk: the run's first day, each day with an act, each lifespan's last.
    Events,
    /// The oracle: every day from S to E.
    Every,
}

/// The oracle's next day after `done`: the day after it, or S for a walk not started; none
/// past E (refcore's `run_to`). P §2.2.
#[must_use]
pub(crate) fn every_day(done: Option<Day>, start: Day, end: Day) -> Option<Day> {
    let next = match done {
        None => start,
        Some(d) => d.next()?,
    };
    (next <= end).then_some(next)
}

#[cfg(test)]
mod tests {
    use super::every_day;
    use crate::time::Day;

    #[test]
    fn the_oracle_steps_every_day_from_s_to_e() {
        let s = Day::from_ymd(2025, 1, 30).expect("a day");
        let e = Day::from_ymd(2025, 2, 1).expect("a day");
        let mut days = Vec::new();
        let mut done = None;
        while let Some(d) = every_day(done, s, e) {
            days.push(d.text());
            done = Some(d);
        }
        assert_eq!(
            days,
            ["2025-01-30", "2025-01-31", "2025-02-01"],
            "every day"
        );
    }
}
