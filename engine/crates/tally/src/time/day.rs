//! A calendar day of the proleptic Gregorian calendar, checked when it is made.
//! Spec: L §2.1 (dates are ISO), §4.1 (days, months, years).
//! Twin: engine/tools/refcore/lexer.py (`datetime.date`, years 1 to 9999).
//! Invariants: a `Day` is always a real day: year 1 to 9999, month 1 to 12, day 1 to the
//! month's length, leap years by the Gregorian rule.

use core::cmp::Ordering;

/// A day, `2026-07-14`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Day {
    /// The year, 1 to 9999.
    year: u16,
    /// The month, 1 to 12.
    month: u8,
    /// The day of the month, from 1.
    dom: u8,
}

impl Day {
    /// The day `year-month-day`, or `None` when no such day exists (as `datetime.date`
    /// refuses it). L §2.1.
    #[must_use]
    pub fn from_ymd(year: u16, month: u8, day: u8) -> Option<Day> {
        if !(1..=9999).contains(&year) || day == 0 {
            return None;
        }
        if day > month_length(year, month)? {
            return None;
        }
        Some(Day {
            year,
            month,
            dom: day,
        })
    }

    /// The year. L §4.1.
    #[must_use]
    pub fn year(self) -> u16 {
        self.year
    }

    /// The month, 1 to 12. L §4.1.
    #[must_use]
    pub fn month(self) -> u8 {
        self.month
    }

    /// The day of the month. L §4.1.
    #[must_use]
    pub fn day(self) -> u8 {
        self.dom
    }

    /// The calendar order of two days: by year, then month, then day. L §4.1.
    #[must_use]
    pub fn order(self, other: Day) -> Ordering {
        self.year
            .cmp(&other.year)
            .then(self.month.cmp(&other.month))
            .then(self.dom.cmp(&other.dom))
    }
}

/// Whether `year` is a leap year of the Gregorian calendar. L §4.1.
fn is_leap(year: u16) -> bool {
    let by = |n: u16| year.checked_rem(n) == Some(0);
    by(4) && (!by(100) || by(400))
}

/// The number of days in a month, or `None` for a month outside 1 to 12. L §4.1.
fn month_length(year: u16, month: u8) -> Option<u8> {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Some(31),
        4 | 6 | 9 | 11 => Some(30),
        2 => Some(if is_leap(year) { 29 } else { 28 }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::Day;

    #[test]
    fn days_are_checked() {
        assert!(Day::from_ymd(2026, 2, 28).is_some(), "28 February");
        assert!(Day::from_ymd(2026, 2, 29).is_none(), "2026 is no leap year");
        assert!(Day::from_ymd(2024, 2, 29).is_some(), "2024 is");
        assert!(Day::from_ymd(1900, 2, 29).is_none(), "1900 isn't");
        assert!(Day::from_ymd(2000, 2, 29).is_some(), "2000 is");
        assert!(Day::from_ymd(2026, 4, 31).is_none(), "April has 30");
        assert!(Day::from_ymd(2026, 13, 1).is_none(), "month 13");
        assert!(
            Day::from_ymd(0, 1, 1).is_none(),
            "year 0, as datetime refuses it"
        );
        assert!(Day::from_ymd(2026, 1, 0).is_none(), "day 0");
        let d = Day::from_ymd(2026, 7, 14).expect("a day");
        assert_eq!((d.year(), d.month(), d.day()), (2026, 7, 14), "its parts");
    }
}
