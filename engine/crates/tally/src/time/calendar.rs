//! A business calendar: its weekends, its yearly days off from their first year, and its added days, fixed or not yet.
//! Spec: L §4.2 (`calendar`, its `weekends`, `yearly … from Y` and `added`; a day not yet fixed
//! in `added`), §7.6 item 5 (a calendar with no business day).
//! Twin: engine/tools/refcore/evaluate.py (`business`), dates.py (`WEEKDAYS`); values.py
//! (`Cal`, its `fmt`); loader.py (`convert_value`, its `calval` arm).
//! Invariants: a day is off by the rules when its weekday is a weekend or it is a yearly day
//! from the yearly list's first year; an added day that is a hole or a window is kept as
//! written, and the evaluator decides it per day and per pass (`eval/business.rs`).

use std::collections::BTreeSet;

use super::day::Day;
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

/// A day a calendar adds as a day off (L §4.2 `added`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Added {
    /// A day.
    Day(Day),
    /// A day not known yet, `?"…"`: its question.
    Hole(String),
    /// `window(a, b)`: one day from a to b, not yet fixed.
    Window(Day, Day),
    /// `one_of([…])`: one of these days, sorted, each once.
    OneOf(Vec<Day>),
}

impl Added {
    /// Whether `day` is one the added entry may be, for a window or a choice of days.
    #[must_use]
    pub fn may_be(&self, day: Day) -> bool {
        match self {
            Added::Window(a, b) => *a <= day && day <= *b,
            Added::OneOf(days) => days.contains(&day),
            Added::Day(_) | Added::Hole(_) => false,
        }
    }

    /// The days a window or a choice of days can take, in order; none for another entry. A
    /// window's days stop at its last day before a step, so one that ends on the last day of
    /// time, 9999-12-31, has its days too.
    #[must_use]
    pub fn days(&self) -> Option<Vec<Day>> {
        match self {
            Added::Window(a, b) => {
                let mut out = Vec::new();
                let mut day = *a;
                while day < *b {
                    out.push(day);
                    day = day.next()?;
                }
                if day == *b {
                    out.push(day);
                }
                Some(out)
            }
            Added::OneOf(days) => Some(days.clone()),
            Added::Day(_) | Added::Hole(_) => None,
        }
    }

    /// A window or a choice of days as written, `window(a, b)` or `one_of([a, b])`, which a
    /// question names (refcore's `business`, its `wt`). L §4.2, §4.4.
    #[must_use]
    pub fn text(&self) -> String {
        match self {
            Added::Window(a, b) => format!("window({}, {})", a.text(), b.text()),
            Added::OneOf(days) => {
                let texts: Vec<String> = days.iter().map(|d| d.text()).collect();
                format!("one_of([{}])", texts.join(", "))
            }
            Added::Day(d) => d.text(),
            Added::Hole(q) => format!("?\"{q}\""),
        }
    }
}

/// A business calendar (L §4.2 `calendar`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Calendar {
    /// The weekend days, as weekday numbers 1 to 7.
    weekends: Vec<u8>,
    /// The days off every year, as (month, day).
    yearly: Vec<(u8, u8)>,
    /// The first year the yearly days are off, or every year.
    yearly_from: Option<FirstYear>,
    /// The added days off, as written.
    added: Vec<Added>,
}

impl Calendar {
    /// A calendar of these weekends, yearly days (from `yearly_from` on, if given) and added
    /// days. L §4.2 `calendar`.
    #[must_use]
    pub fn new(
        weekends: Vec<u8>,
        yearly: Vec<(u8, u8)>,
        yearly_from: Option<FirstYear>,
        added: Vec<Added>,
    ) -> Calendar {
        Calendar {
            weekends,
            yearly,
            yearly_from,
            added,
        }
    }

    /// Whether `day` is off by the calendar's rules: a weekend, or a yearly day from the
    /// yearly list's first year (refcore's `business`, before its added days). L §4.2.
    #[must_use]
    pub fn off_by_rule(&self, day: Day) -> bool {
        if self.weekends.contains(&day.weekday()) {
            return true;
        }
        self.yearly.contains(&(day.month(), day.day()))
            && self
                .yearly_from
                .as_ref()
                .is_none_or(|from| day.year() >= from.year())
    }

    /// The added days, as written, in their order. L §4.2 `added`.
    #[must_use]
    pub fn added(&self) -> &[Added] {
        &self.added
    }

    /// Whether a search for a business day can find one: not when the weekends name all
    /// seven days. Yearly days never empty a year, since 02-29 is no month-day (L §3.1)
    /// (refcore's `business`, its `seek`). L §4.2, §7.6 item 5.
    #[must_use]
    pub fn has_business_day(&self) -> bool {
        let weekends: BTreeSet<u8> = self.weekends.iter().copied().collect();
        weekends.len() < 7
    }

    /// The calendar as it prints, `calendar(weekends [sat, sun], yearly [12-25] from 2027)`;
    /// its added days don't print (refcore's `fmt` of a `Cal`). L §5.18.
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
}

#[cfg(test)]
mod tests {
    use super::{Added, Calendar, FirstYear, Num, weekday_number};
    use crate::time::Day;

    fn day(y: u16, m: u8, d: u8) -> Day {
        Day::from_ymd(y, m, d).expect("a real day")
    }

    #[test]
    fn weekends_and_yearly_days_are_off_by_rule() {
        let weekends = ["sat", "sun"]
            .iter()
            .filter_map(|w| weekday_number(w))
            .collect();
        let cal = Calendar::new(
            weekends,
            Vec::new(),
            None,
            vec![Added::Day(day(2026, 1, 1))],
        );
        assert!(
            !cal.off_by_rule(day(2026, 1, 1)),
            "an added day isn't a rule's"
        );
        assert!(!cal.off_by_rule(day(2026, 1, 2)), "a Friday");
        assert!(cal.off_by_rule(day(2026, 1, 3)), "a Saturday");
        let from = FirstYear::of(&Num::int(2027)).expect("a year");
        let yearly = Calendar::new(Vec::new(), vec![(12, 25)], Some(from), Vec::new());
        assert!(
            !yearly.off_by_rule(day(2026, 12, 25)),
            "before its first year"
        );
        assert!(yearly.off_by_rule(day(2027, 12, 25)), "from it");
    }

    #[test]
    fn an_added_window_names_its_days() {
        let w = Added::Window(day(2026, 6, 22), day(2026, 6, 24));
        assert_eq!(
            w.days(),
            Some(vec![day(2026, 6, 22), day(2026, 6, 23), day(2026, 6, 24)]),
            "every day from a to b"
        );
        assert!(w.may_be(day(2026, 6, 23)), "inside");
        assert!(!w.may_be(day(2026, 6, 25)), "after");
        assert_eq!(w.text(), "window(2026-06-22, 2026-06-24)", "as written");
        let o = Added::OneOf(vec![day(2026, 6, 22), day(2026, 6, 26)]);
        assert!(!o.may_be(day(2026, 6, 23)), "between, not one of them");
        assert_eq!(o.text(), "one_of([2026-06-22, 2026-06-26])", "as written");
        assert_eq!(Added::Day(day(2026, 1, 1)).days(), None, "a fixed day");
    }
}
