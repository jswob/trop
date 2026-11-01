//! A calendar day of the proleptic Gregorian calendar, checked when it is made, with its day number.
//! Spec: L §2.1 (dates are ISO), §4.1 (days, months, years), §4.2 (`add_days`, `add_months`,
//! `days_between`, `weekday`).
//! Twin: engine/tools/refcore/dates.py (`add_days`, `add_months`, `month_add`, `last_day`), on
//! Python's `datetime.date` (years 1 to 9999, `toordinal`, `isoweekday`).
//! Invariants: a `Day` is always a real day, year 1 to 9999; its number is Python's ordinal
//! (0001-01-01 is 1), and its year, month and day agree with that number; days are ordered by it.

use core::cmp::Ordering;

/// Days in 400 Gregorian years.
const DAYS_IN_400_YEARS: u32 = 146_097;
/// Days in 100 years whose last is not a leap year.
const DAYS_IN_100_YEARS: u32 = 36_524;
/// Days in 4 years, one of them a leap year.
const DAYS_IN_4_YEARS: u32 = 1_461;
/// The number of 9999-12-31, the last day there is.
const LAST: u32 = 3_652_059;

/// A day, `2026-07-14`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Day {
    /// The day's number: 1 is 0001-01-01 (Python's `toordinal`).
    number: u32,
    /// The year, 1 to 9999.
    year: u16,
    /// The month, 1 to 12.
    month: u8,
    /// The day of the month, from 1.
    dom: u8,
}

impl Ord for Day {
    fn cmp(&self, other: &Day) -> Ordering {
        self.number.cmp(&other.number)
    }
}

impl PartialOrd for Day {
    fn partial_cmp(&self, other: &Day) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Day {
    /// The day `year-month-day`, or `None` when no such day exists (as `datetime.date`
    /// refuses it). L §2.1.
    #[must_use]
    pub fn from_ymd(year: u16, month: u8, day: u8) -> Option<Day> {
        if !(1..=9999).contains(&year) || day == 0 || day > month_length(year, month)? {
            return None;
        }
        let number = days_before_year(year)?
            .checked_add(days_before_month(year, month)?)?
            .checked_add(u32::from(day))?;
        Some(Day {
            number,
            year,
            month,
            dom: day,
        })
    }

    /// The day with number `number` (Python's `fromordinal`), or `None` outside years 1 to
    /// 9999. L §4.1.
    #[must_use]
    pub fn from_number(number: u32) -> Option<Day> {
        if !(1..=LAST).contains(&number) {
            return None;
        }
        let n = number.checked_sub(1)?;
        let (n400, n) = div_rem(n, DAYS_IN_400_YEARS)?;
        let (n100, n) = div_rem(n, DAYS_IN_100_YEARS)?;
        let (n4, n) = div_rem(n, DAYS_IN_4_YEARS)?;
        let (n1, n) = div_rem(n, 365)?;
        let centuries = n400.checked_mul(400)?.checked_add(n100.checked_mul(100)?)?;
        let years = n4.checked_mul(4)?.checked_add(n1)?;
        let year = centuries.checked_add(years)?.checked_add(1)?;
        if n1 == 4 || n100 == 4 {
            // the last day of a leap year: Python's `_ord2ymd` turns back one year
            let year = u16::try_from(year.checked_sub(1)?).ok()?;
            return Day::from_ymd(year, 12, 31);
        }
        let year = u16::try_from(year).ok()?;
        let mut left = n;
        for month in 1..=12_u8 {
            let length = u32::from(month_length(year, month)?);
            if left < length {
                let dom = u8::try_from(left.checked_add(1)?).ok()?;
                return Day::from_ymd(year, month, dom);
            }
            left = left.checked_sub(length)?;
        }
        None
    }

    /// The day's number: 0001-01-01 is 1. L §4.1.
    #[must_use]
    pub fn number(self) -> u32 {
        self.number
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

    /// The day of the week, 1 Monday to 7 Sunday (Python's `isoweekday`). L §4.2 `weekday`.
    #[must_use]
    pub fn weekday(self) -> u8 {
        // Python: `toordinal() % 7 or 7`; 0001-01-01, number 1, was a Monday
        match self
            .number
            .checked_rem_euclid(7)
            .and_then(|r| u8::try_from(r).ok())
        {
            Some(0) | None => 7,
            Some(r) => r,
        }
    }

    /// The day `n` days later (earlier for a negative `n`), or `None` outside years 1 to
    /// 9999. L §4.2 `add_days`.
    #[must_use]
    pub fn add_days(self, n: i64) -> Option<Day> {
        let number = i64::from(self.number).checked_add(n)?;
        Day::from_number(u32::try_from(number).ok()?)
    }

    /// The next day, or `None` after 9999-12-31.
    #[must_use]
    pub fn next(self) -> Option<Day> {
        Day::from_number(self.number.checked_add(1)?)
    }

    /// The day before, or `None` before 0001-01-01.
    #[must_use]
    pub fn previous(self) -> Option<Day> {
        Day::from_number(self.number.checked_sub(1)?)
    }

    /// The days from `self` to `other`: `other - self` (Python's `(other - self).days`); always
    /// `Some`, since day numbers fit an `i64` many times over. L §4.2 `days_between`.
    #[must_use]
    pub fn days_to(self, other: Day) -> Option<i64> {
        i64::from(other.number).checked_sub(i64::from(self.number))
    }

    /// The same day of the month `n` months later, clamped to the target month's last day,
    /// or `None` outside years 1 to 9999. L §4.2 `add_months`.
    #[must_use]
    pub fn add_months(self, n: i64) -> Option<Day> {
        let (year, month) = month_add(self.year, self.month, n)?;
        let dom = self.dom.min(month_length(year, month)?);
        Day::from_ymd(year, month, dom)
    }

    /// The day as ISO text, `2026-07-14` (Python's `isoformat`). L §2.1.
    #[must_use]
    pub fn text(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.dom)
    }
}

/// `n` divided by `d`, and the remainder (Python's `divmod` on non-negative numbers).
fn div_rem(n: u32, d: u32) -> Option<(u32, u32)> {
    Some((n.checked_div_euclid(d)?, n.checked_rem_euclid(d)?))
}

/// The days before 1 January of `year` (Python's `_days_before_year`).
fn days_before_year(year: u16) -> Option<u32> {
    let y = u32::from(year).checked_sub(1)?;
    y.checked_mul(365)?
        .checked_add(y.checked_div_euclid(4)?)?
        .checked_sub(y.checked_div_euclid(100)?)?
        .checked_add(y.checked_div_euclid(400)?)
}

/// The days of `year` before the first of `month` (Python's `_days_before_month`).
fn days_before_month(year: u16, month: u8) -> Option<u32> {
    let mut days = 0_u32;
    for earlier in 1..month {
        days = days.checked_add(u32::from(month_length(year, earlier)?))?;
    }
    Some(days)
}

/// The year and month `n` months after `year-month` (refcore's `month_add`: `k // 12`,
/// `k % 12 + 1`), or `None` outside years 1 to 9999.
pub(super) fn month_add(year: u16, month: u8, n: i64) -> Option<(u16, u8)> {
    let months_before = i64::from(year)
        .checked_mul(12)?
        .checked_add(i64::from(month).checked_sub(1)?)?;
    let k = months_before.checked_add(n)?;
    let new_year = u16::try_from(k.checked_div_euclid(12)?).ok()?;
    let new_month = u8::try_from(k.checked_rem_euclid(12)?.checked_add(1)?).ok()?;
    if !(1..=9999).contains(&new_year) {
        return None;
    }
    Some((new_year, new_month))
}

/// Whether `year` is a leap year of the Gregorian calendar. L §4.1.
pub(super) fn is_leap(year: u16) -> bool {
    let by = |n: u16| year.checked_rem(n) == Some(0);
    by(4) && (!by(100) || by(400))
}

/// The number of days in `month` of `year`, or `None` for a month that isn't 1 to 12
/// (refcore's `last_day`). L §4.1.
pub(super) fn month_length(year: u16, month: u8) -> Option<u8> {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Some(31),
        4 | 6 | 9 | 11 => Some(30),
        2 if is_leap(year) => Some(29),
        2 => Some(28),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::Day;

    fn day(y: u16, m: u8, d: u8) -> Day {
        Day::from_ymd(y, m, d).expect("a real day")
    }

    #[test]
    fn real_days_only() {
        assert!(Day::from_ymd(2024, 2, 29).is_some(), "a leap day");
        assert!(Day::from_ymd(2023, 2, 29).is_none(), "not a leap year");
        assert!(Day::from_ymd(1900, 2, 29).is_none(), "1900 isn't leap");
        assert!(Day::from_ymd(2000, 2, 29).is_some(), "2000 is");
        assert!(Day::from_ymd(2026, 4, 31).is_none(), "April has 30");
        assert!(Day::from_ymd(2026, 13, 1).is_none(), "no month 13");
        assert!(Day::from_ymd(0, 1, 1).is_none(), "no year 0");
        assert!(Day::from_ymd(2026, 1, 0).is_none(), "no day 0");
    }

    #[test]
    fn numbers_are_python_ordinals() {
        let cases = [
            ((1, 1, 1), 1, 1),
            ((2026, 1, 1), 739_617, 4),
            ((9999, 12, 31), 3_652_059, 5),
            ((2024, 2, 29), 738_945, 4),
            ((2000, 3, 1), 730_180, 3),
        ];
        for ((y, m, d), number, weekday) in cases {
            let it = day(y, m, d);
            assert_eq!(it.number(), number, "{}", it.text());
            assert_eq!(it.weekday(), weekday, "{}", it.text());
            assert_eq!(Day::from_number(number), Some(it), "back from {number}");
        }
        assert_eq!(Day::from_number(0), None, "before the first day");
        assert_eq!(Day::from_number(3_652_060), None, "after the last");
    }

    #[test]
    fn every_day_of_four_centuries_goes_and_comes_back() {
        let mut it = day(1899, 12, 31);
        let end = day(2301, 1, 1);
        while it < end {
            let next = it.next().expect("a next day");
            assert_eq!(it.days_to(next), Some(1), "{}", it.text());
            let (y, m, d) = (next.year(), next.month(), next.day());
            assert_eq!(Day::from_ymd(y, m, d), Some(next), "{}", next.text());
            it = next;
        }
    }

    #[test]
    fn months_clamp_to_the_last_day() {
        assert_eq!(
            day(2026, 1, 31).add_months(1),
            Some(day(2026, 2, 28)),
            "Feb"
        );
        assert_eq!(
            day(2024, 1, 31).add_months(1),
            Some(day(2024, 2, 29)),
            "leap"
        );
        assert_eq!(
            day(2026, 3, 15).add_months(-3),
            Some(day(2025, 12, 15)),
            "back"
        );
        assert_eq!(
            day(2026, 1, 1).add_months(-12),
            Some(day(2025, 1, 1)),
            "a year"
        );
        assert_eq!(day(1, 1, 1).add_months(-1), None, "before year 1");
        assert_eq!(
            day(2026, 1, 1).add_days(-1),
            Some(day(2025, 12, 31)),
            "a day back"
        );
        assert_eq!(day(2026, 1, 1).text(), "2026-01-01", "ISO");
    }
}
