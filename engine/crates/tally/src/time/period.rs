//! Periods: a day, a month, a quarter or a year, with the step its index advances by, and their arithmetic.
//! Spec: L §4.1 (periods and period indexes, `by n`, anchored ranges), §4.2 (`month_of`,
//! `quarter_of`, `year_of`, `age`, `age_months`, `overlap_days`, `start_of`, `end_of`).
//! Twin: engine/tools/refcore/dates.py (`Period`, `day_period` to `year_period`, `month_of` to
//! `year_of`, `age`, `age_months`, `overlap_days`).
//! Invariants: a period's end is the first day after it and lies after its start; shifting
//! a period keeps its kind and its step.

use super::day::{Day, is_leap};

/// What a period is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeriodKind {
    /// One day.
    Day,
    /// A month, or a month-long period of an anchored index.
    Month,
    /// Three months.
    Quarter,
    /// A year.
    Year,
}

/// What a period's index advances by: so many days, or so many months.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// `n` days.
    Days(u32),
    /// `n` months.
    Months(u32),
}

/// A period: its kind, its first day, the first day after it, and its index's step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Period {
    /// What it is.
    kind: PeriodKind,
    /// Its first day.
    start: Day,
    /// The first day after it.
    end: Day,
    /// What its index advances by.
    step: Step,
}

impl Period {
    /// A period of `kind` from `start` to the day before `end`, advancing by `step`, or
    /// `None` when `end` isn't after `start`. L §4.1.
    #[must_use]
    pub fn new(kind: PeriodKind, start: Day, end: Day, step: Step) -> Option<Period> {
        (end > start).then_some(Period {
            kind,
            start,
            end,
            step,
        })
    }

    /// The day `day` as a period (refcore's `day_period`). L §4.1.
    #[must_use]
    pub fn day(day: Day) -> Option<Period> {
        Period::new(PeriodKind::Day, day, day.next()?, Step::Days(1))
    }

    /// The month `year-month` (refcore's `month_period`). L §4.1.
    #[must_use]
    pub fn month(year: u16, month: u8) -> Option<Period> {
        let start = Day::from_ymd(year, month, 1)?;
        Period::new(
            PeriodKind::Month,
            start,
            start.add_months(1)?,
            Step::Months(1),
        )
    }

    /// The quarter `year-Qq` (refcore's `quarter_period`). L §4.1.
    #[must_use]
    pub fn quarter(year: u16, quarter: u8) -> Option<Period> {
        let month = quarter.checked_sub(1)?.checked_mul(3)?.checked_add(1)?;
        let start = Day::from_ymd(year, month, 1)?;
        Period::new(
            PeriodKind::Quarter,
            start,
            start.add_months(3)?,
            Step::Months(3),
        )
    }

    /// The year `year` (refcore's `year_period`). L §4.1.
    #[must_use]
    pub fn year(year: u16) -> Option<Period> {
        let start = Day::from_ymd(year, 1, 1)?;
        let end = Day::from_ymd(year.checked_add(1)?, 1, 1)?;
        Period::new(PeriodKind::Year, start, end, Step::Months(12))
    }

    /// The month that holds `day`. L §4.2 `month_of`.
    #[must_use]
    pub fn month_of(day: Day) -> Option<Period> {
        Period::month(day.year(), day.month())
    }

    /// The quarter that holds `day`. L §4.2 `quarter_of`.
    #[must_use]
    pub fn quarter_of(day: Day) -> Option<Period> {
        Period::quarter(day.year(), quarter_number(day.month()))
    }

    /// The year that holds `day`. L §4.2 `year_of`.
    #[must_use]
    pub fn year_of(day: Day) -> Option<Period> {
        Period::year(day.year())
    }

    /// What it is.
    #[must_use]
    pub fn kind(self) -> PeriodKind {
        self.kind
    }

    /// Its first day. L §4.2 `start_of`.
    #[must_use]
    pub fn start(self) -> Day {
        self.start
    }

    /// The first day after it. L §4.2 `end_of`.
    #[must_use]
    pub fn end(self) -> Day {
        self.end
    }

    /// What its index advances by.
    #[must_use]
    pub fn step(self) -> Step {
        self.step
    }

    /// Whether `day` lies in it.
    #[must_use]
    pub fn contains(self, day: Day) -> bool {
        self.start <= day && day < self.end
    }

    /// The period `k` steps later (earlier for a negative `k`): key arithmetic on its index
    /// (refcore's `Period.shift`). L §4.1.
    #[must_use]
    pub fn shift(self, k: i64) -> Option<Period> {
        match self.step {
            Step::Days(n) => {
                let start = self.start.add_days(k.checked_mul(i64::from(n))?)?;
                let end = start.add_days(i64::from(n))?;
                Period::new(self.kind, start, end, self.step)
            }
            Step::Months(n) => {
                let n = i64::from(n);
                let start = self.start.add_months(k.checked_mul(n)?)?;
                let end = self.start.add_months(k.checked_add(1)?.checked_mul(n)?)?;
                Period::new(self.kind, start, end, self.step)
            }
        }
    }

    /// How many steps from `self` to `other` (`other - self`), or `None` when `other` isn't on
    /// `self`'s grid (refcore's `Period.steps_to`). L §4.1.
    #[must_use]
    pub fn steps_to(self, other: Period) -> Option<i64> {
        match self.step {
            Step::Days(n) => {
                let days = self.start.days_to(other.start)?;
                let n = i64::from(n);
                (days.checked_rem_euclid(n)? == 0).then_some(days.checked_div_euclid(n)?)
            }
            Step::Months(n) => {
                let years =
                    i64::from(other.start.year()).checked_sub(i64::from(self.start.year()))?;
                let months =
                    i64::from(other.start.month()).checked_sub(i64::from(self.start.month()))?;
                let total = years.checked_mul(12)?.checked_add(months)?;
                let n = i64::from(n);
                let on_grid =
                    total.checked_rem_euclid(n)? == 0 && other.start.day() == self.start.day();
                on_grid.then_some(total.checked_div_euclid(n)?)
            }
        }
    }

    /// The period as a key prints: `2026-07-14`, `2026-07`, `2026-Q3`, `2026` (refcore's
    /// `Period.label`). L §2.1, §2.4.
    #[must_use]
    pub fn label(self) -> String {
        let start = self.start;
        match self.kind {
            PeriodKind::Day => start.text(),
            PeriodKind::Month => format!("{:04}-{:02}", start.year(), start.month()),
            PeriodKind::Year => format!("{:04}", start.year()),
            PeriodKind::Quarter => {
                format!("{:04}-Q{}", start.year(), quarter_number(start.month()))
            }
        }
    }
}

/// Completed years from `birth` to `day`: the anniversary of 29 February in a common year is
/// 1 March; `None` before `birth` (refcore's `age`). L §4.2 `age`.
#[must_use]
pub fn age(birth: Day, day: Day) -> Option<i64> {
    if day < birth {
        return None;
    }
    let years = i64::from(day.year()).checked_sub(i64::from(birth.year()))?;
    let anniversary = if birth.month() == 2 && birth.day() == 29 && !is_leap(day.year()) {
        Day::from_ymd(day.year(), 3, 1)?
    } else {
        Day::from_ymd(day.year(), birth.month(), birth.day())?
    };
    if day < anniversary {
        return years.checked_sub(1);
    }
    Some(years)
}

/// The largest `n` with `add_months(birth, n) <= day`; `None` before `birth` (refcore's
/// `age_months`). L §4.2 `age_months`.
#[must_use]
pub fn age_months(birth: Day, day: Day) -> Option<i64> {
    if day < birth {
        return None;
    }
    let years = i64::from(day.year()).checked_sub(i64::from(birth.year()))?;
    let months = i64::from(day.month()).checked_sub(i64::from(birth.month()))?;
    let mut n = years.checked_mul(12)?.checked_add(months)?;
    while n > 0 && birth.add_months(n)? > day {
        n = n.checked_sub(1)?;
    }
    while birth.add_months(n.checked_add(1)?)? <= day {
        n = n.checked_add(1)?;
    }
    Some(n)
}

/// The days `x` with `from <= x <= to` that lie in `period` (refcore's `overlap_days`).
/// L §4.2 `overlap_days`.
#[must_use]
pub fn overlap_days(from: Day, to: Day, period: Period) -> Option<i64> {
    let lo = from.max(period.start());
    let hi = to.min(period.end().previous()?);
    if hi < lo {
        return Some(0);
    }
    lo.days_to(hi)?.checked_add(1)
}

/// The quarter, 1 to 4, that holds month `month`.
fn quarter_number(month: u8) -> u8 {
    match month {
        1..=3 => 1,
        4..=6 => 2,
        7..=9 => 3,
        _ => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::{Period, PeriodKind, Step, age, age_months, overlap_days};
    use crate::time::Day;

    fn day(y: u16, m: u8, d: u8) -> Day {
        Day::from_ymd(y, m, d).expect("a real day")
    }

    #[test]
    fn periods_and_their_labels() {
        let march = Period::month(2026, 3).expect("March");
        assert_eq!(march.end(), day(2026, 4, 1), "first day after");
        assert_eq!(march.label(), "2026-03", "label");
        let q4 = Period::quarter_of(day(2026, 11, 2)).expect("Q4");
        assert_eq!(
            (q4.start(), q4.label()),
            (day(2026, 10, 1), "2026-Q4".to_owned()),
            "Q4"
        );
        assert_eq!(
            Period::year_of(day(2026, 5, 5)).map(Period::label),
            Some("2026".to_owned()),
            "year"
        );
        assert_eq!(
            Period::day(day(2026, 1, 9)).map(Period::label),
            Some("2026-01-09".to_owned()),
            "day"
        );
        assert!(
            march.contains(day(2026, 3, 31)) && !march.contains(day(2026, 4, 1)),
            "contains"
        );
    }

    #[test]
    fn shifts_and_steps() {
        let jan = Period::month(2026, 1).expect("Jan");
        let nov = jan.shift(10).expect("Nov");
        assert_eq!(nov, Period::month(2026, 11).expect("Nov"), "ten months on");
        assert_eq!(jan.steps_to(nov), Some(10), "and back");
        assert_eq!(nov.shift(-11), Period::month(2025, 12), "into last year");
        let start = day(2026, 1, 15);
        let anchored = Period::new(
            PeriodKind::Month,
            start,
            start.add_months(2).expect("+2"),
            Step::Months(2),
        )
        .expect("two months");
        assert_eq!(
            anchored.shift(1).map(Period::start),
            Some(day(2026, 3, 15)),
            "by 2"
        );
        assert_eq!(anchored.steps_to(jan), None, "off the grid");
        let d = Period::day(day(2026, 2, 27)).expect("a day");
        assert_eq!(d.shift(2).map(Period::start), Some(day(2026, 3, 1)), "days");
    }

    #[test]
    fn ages() {
        assert_eq!(
            age(day(1991, 2, 14), day(2026, 2, 13)),
            Some(34),
            "the day before"
        );
        assert_eq!(
            age(day(1991, 2, 14), day(2026, 2, 14)),
            Some(35),
            "the birthday"
        );
        assert_eq!(
            age(day(2000, 2, 29), day(2025, 2, 28)),
            Some(24),
            "leapling, Feb 28"
        );
        assert_eq!(
            age(day(2000, 2, 29), day(2025, 3, 1)),
            Some(25),
            "leapling, Mar 1"
        );
        assert_eq!(age(day(2000, 1, 1), day(1999, 1, 1)), None, "before birth");
        assert_eq!(
            age_months(day(2026, 1, 31), day(2026, 2, 28)),
            Some(1),
            "clamped month"
        );
        assert_eq!(
            age_months(day(2026, 1, 31), day(2026, 2, 27)),
            Some(0),
            "not yet"
        );
    }

    #[test]
    fn overlaps() {
        let march = Period::month(2026, 3).expect("March");
        assert_eq!(
            overlap_days(day(2026, 2, 20), day(2026, 3, 10), march),
            Some(10),
            "ten days"
        );
        assert_eq!(
            overlap_days(day(2026, 4, 2), day(2026, 4, 10), march),
            Some(0),
            "none"
        );
    }
}
