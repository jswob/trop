//! A business calendar of fixed days: its weekends, its yearly days off, its added days off, and the business-day count.
//! Spec: L §4.2 (`calendar`, `is_business`, `add_business`, `last_business`), §7.6 item 5 (a
//! business day searched for within 400 years).
//! Twin: engine/tools/refcore/evaluate.py (`business`, `c_add_business`, `c_last_business`),
//! dates.py (`WEEKDAYS`); values.py (`Cal`).
//! Invariants: a day is a business day unless its weekday is a weekend, it is a yearly day
//! from the yearly list's first year, or it is added; counting never stops on a non-business day.
//!
//! An added day that is a window, a `one_of` or a hole needs the evaluator's windows and
//! holes; the evaluator decides those before it asks this calendar (milestone S, G5).

use std::collections::BTreeSet;

use super::day::Day;
use super::period::Period;
use crate::num::Num;

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

/// The word of weekday number `n`, 1 Monday to 7 Sunday (refcore's `WEEKDAYS`). L §4.2.
fn weekday_word(n: u8) -> &'static str {
    match n {
        1 => "mon",
        2 => "tue",
        3 => "wed",
        4 => "thu",
        5 => "fri",
        6 => "sat",
        _ => "sun",
    }
}

/// The first year a calendar's yearly days are off: the year its rule compares, and the
/// figure as written, which it prints, both made from one figure. L §4.2 `calendar`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirstYear {
    /// The year compared: any year past 9999 is 10000, after every day there is (L §2.1).
    year: u16,
    /// The figure as written.
    text: String,
}

impl FirstYear {
    /// The first year a figure names: its year, any year past 9999 as 10000, and the figure
    /// as written; none for a figure that is no whole number of zero or more. L §4.2
    /// `calendar`, §2.1.
    #[must_use]
    pub fn of(n: &Num) -> Option<FirstYear> {
        let text = n.integer_text()?;
        let year = match n.to_i64().and_then(|y| u16::try_from(y).ok()) {
            Some(y) if y <= 9_999 => y,
            Some(_) | None if !n.is_negative() => 10_000,
            Some(_) | None => return None,
        };
        Some(FirstYear { year, text })
    }

    /// The year compared. L §4.2 `calendar`.
    #[must_use]
    pub fn year(&self) -> u16 {
        self.year
    }

    /// The figure as written. L §2.4.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
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
    yearly_from: Option<FirstYear>,
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
        yearly_from: Option<FirstYear>,
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
            && self
                .yearly_from
                .as_ref()
                .is_none_or(|from| day.year() >= from.year());
        !yearly && !self.added.contains(&day)
    }

    /// Whether a search for a business day can find one, as of `day`: not when the weekends
    /// name all seven days, or the yearly days all 366 month-days from their first year on
    /// (refcore's `business`, its `seek`). L §4.2, §7.6 item 5.
    #[must_use]
    pub fn has_business_day(&self, day: Day) -> bool {
        let weekends: BTreeSet<u8> = self.weekends.iter().copied().collect();
        let yearly: BTreeSet<(u8, u8)> = self.yearly.iter().copied().collect();
        let all_yearly = yearly.len() == 366
            && self
                .yearly_from
                .as_ref()
                .is_none_or(|from| day.year() >= from.year());
        weekends.len() < 7 && !all_yearly
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

    /// The calendar as it prints, `calendar(weekends [sat, sun], yearly [12-25] from 2027)`
    /// (refcore's `fmt` of a `Cal`). L §5.18.
    #[must_use]
    pub fn text(&self) -> String {
        let words: Vec<&str> = self.weekends.iter().map(|n| weekday_word(*n)).collect();
        let mut parts = vec![format!("weekends [{}]", words.join(", "))];
        if !self.yearly.is_empty() {
            let days: Vec<String> = self
                .yearly
                .iter()
                .map(|(m, d)| format!("{m:02}-{d:02}"))
                .collect();
            let from = self
                .yearly_from
                .as_ref()
                .map_or_else(String::new, |from| format!(" from {}", from.text()));
            parts.push(format!("yearly [{}]{from}", days.join(", ")));
        }
        format!("calendar({})", parts.join(", "))
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
    use super::{Calendar, FirstYear, Num, weekday_number};
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
        let from = FirstYear::of(&Num::int(2027)).expect("a year");
        let yearly = Calendar::new(Vec::new(), vec![(12, 25)], Some(from), Vec::new());
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
