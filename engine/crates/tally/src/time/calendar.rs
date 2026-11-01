//! A business calendar of fixed days: its weekends, its yearly days off, its added days off, and the business-day count.
//! Spec: L §4.2 (`calendar`, `is_business`, `add_business`, `last_business`).
//! Twin: engine/tools/refcore/evaluate.py (`business`, `c_add_business`, `c_last_business`),
//! dates.py (`WEEKDAYS`); values.py (`Cal`).
//! Invariants: a day is a business day unless its weekday is a weekend, it is a yearly day
//! from the yearly list's first year, or it is added; counting never stops on a non-business day.
//!
//! An added day that is a window, a `one_of` or a hole needs the evaluator's windows and
//! holes; the evaluator decides those before it asks this calendar (milestone S, G5).

use super::day::Day;
use super::period::Period;

/// The weekday number, 1 Monday to 7 Sunday, of a weekday's word (refcore's `WEEKDAYS`).
/// L §4.2.
#[must_use]
pub fn weekday_number(word: &str) -> Option<u8> {
    match word {
        "mon" => Some(1),
        "tue" => Some(2),
        "wed" => Some(3),
        "thu" => Some(4),
        "fri" => Some(5),
        "sat" => Some(6),
        "sun" => Some(7),
        _ => None,
    }
}

/// A business calendar whose days off are all fixed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Calendar {
    /// The weekend days, as weekday numbers 1 to 7.
    weekends: Vec<u8>,
    /// The days off every year, as (month, day).
    yearly: Vec<(u8, u8)>,
    /// The first year the yearly days are off, or every year.
    yearly_from: Option<u16>,
    /// The added days off.
    added: Vec<Day>,
}

impl Calendar {
    /// A calendar of these weekends, yearly days (from `yearly_from` on, if given) and added
    /// days. L §4.2 `calendar`.
    #[must_use]
    pub fn new(
        weekends: Vec<u8>,
        yearly: Vec<(u8, u8)>,
        yearly_from: Option<u16>,
        added: Vec<Day>,
    ) -> Calendar {
        Calendar {
            weekends,
            yearly,
            yearly_from,
            added,
        }
    }

    /// Whether `day` is a business day (refcore's `business`, its fixed days). L §4.2
    /// `is_business`.
    #[must_use]
    pub fn is_business(&self, day: Day) -> bool {
        if self.weekends.contains(&day.weekday()) {
            return false;
        }
        let yearly = self.yearly.contains(&(day.month(), day.day()))
            && self.yearly_from.is_none_or(|from| day.year() >= from);
        !yearly && !self.added.contains(&day)
    }

    /// The `n`-th business day after `day` (before it for a negative `n`), never `day`
    /// itself; `None` for `n = 0` (`RUN-ZERO`) or past the calendar's years (refcore's
    /// `c_add_business`). L §4.2 `add_business`.
    #[must_use]
    pub fn add_business(&self, day: Day, n: i64) -> Option<Day> {
        let step: i64 = match n.signum() {
            1 => 1,
            -1 => -1,
            _ => return None,
        };
        let mut left = n.checked_abs()?;
        let mut current = day;
        while left > 0 {
            current = current.add_days(step)?;
            if self.is_business(current) {
                left = left.checked_sub(1)?;
            }
        }
        Some(current)
    }

    /// The `n`-th last business day of `period`, counting back from its last day; `None` when
    /// it has fewer (refcore's `c_last_business`; `n` below 1 is the evaluator's `RUN-DOMAIN`).
    /// L §4.2 `last_business`.
    #[must_use]
    pub fn last_business(&self, period: Period, n: i64) -> Option<Day> {
        let mut left = n;
        let mut current = period.end().previous()?;
        while period.contains(current) {
            if self.is_business(current) {
                left = left.checked_sub(1)?;
                if left == 0 {
                    return Some(current);
                }
            }
            current = current.previous()?;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{Calendar, weekday_number};
    use crate::time::{Day, Period};

    fn day(y: u16, m: u8, d: u8) -> Day {
        Day::from_ymd(y, m, d).expect("a real day")
    }

    fn polish() -> Calendar {
        let weekends = ["sat", "sun"]
            .iter()
            .filter_map(|w| weekday_number(w))
            .collect();
        Calendar::new(
            weekends,
            Vec::new(),
            None,
            vec![day(2026, 1, 1), day(2026, 1, 6)],
        )
    }

    #[test]
    fn weekends_and_added_days_are_off() {
        let cal = polish();
        assert!(!cal.is_business(day(2026, 1, 1)), "added");
        assert!(cal.is_business(day(2026, 1, 2)), "a Friday");
        assert!(!cal.is_business(day(2026, 1, 3)), "a Saturday");
        let yearly = Calendar::new(Vec::new(), vec![(12, 25)], Some(2027), Vec::new());
        assert!(
            yearly.is_business(day(2026, 12, 25)),
            "before its first year"
        );
        assert!(!yearly.is_business(day(2027, 12, 25)), "from it");
    }

    #[test]
    fn counting_business_days() {
        let cal = polish();
        assert_eq!(
            cal.add_business(day(2026, 1, 2), 1),
            Some(day(2026, 1, 5)),
            "over the weekend"
        );
        assert_eq!(
            cal.add_business(day(2026, 1, 5), 1),
            Some(day(2026, 1, 7)),
            "over the 6th"
        );
        assert_eq!(
            cal.add_business(day(2026, 1, 5), -1),
            Some(day(2026, 1, 2)),
            "back"
        );
        assert_eq!(cal.add_business(day(2026, 1, 5), 0), None, "zero");
        let jan = Period::month(2026, 1).expect("January");
        assert_eq!(
            cal.last_business(jan, 1),
            Some(day(2026, 1, 30)),
            "the 31st is a Saturday"
        );
    }
}
