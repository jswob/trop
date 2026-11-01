//! Key values and spans: what a key position holds, what a row's pattern states, and the days a span covers.
//! Spec: L §2.5 (keys by position), §4.1 (indexes and their keys), §4.6 step 1 (a row's keys,
//! `*`, `a..b`), §5.7 (things), §4.5 (spans: `always`, `a..b`, a lifespan's hole end).
//! Twin: engine/tools/refcore/values.py (`Key`, `Thing`), model.py (`_kv`, `Span`, `ALWAYS`,
//! `Row.keys_text`, `key_text`).
//! Invariants: two key values are equal when refcore's `_kv` is; a span's end is the first day
//! after it; `always` holds every day.

use crate::time::{Day, Period};

/// A key value (refcore's `_kv` cases).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyVal {
    /// A key of a list or open index, or an axis option: its index and its text.
    Key {
        /// The index.
        index: String,
        /// The key's text.
        name: String,
    },
    /// A thing of a kind, by its name.
    Thing(String),
    /// A day of a day index.
    Day(Day),
    /// A period of a period index.
    Period(Period),
    /// A key of an integer index.
    Int(i64),
}

impl KeyVal {
    /// The key as it prints (refcore's `key_text`). L §2.4.
    #[must_use]
    pub fn text(&self) -> String {
        match self {
            KeyVal::Key { name, .. } | KeyVal::Thing(name) => name.clone(),
            KeyVal::Day(day) => day.text(),
            KeyVal::Period(period) => period.label(),
            KeyVal::Int(n) => n.to_string(),
        }
    }

    /// The day a day key or a period key starts on.
    #[must_use]
    pub fn start(&self) -> Option<Day> {
        match self {
            KeyVal::Day(day) => Some(*day),
            KeyVal::Period(period) => Some(period.start()),
            KeyVal::Key { .. } | KeyVal::Thing(_) | KeyVal::Int(_) => None,
        }
    }
}

/// One position of a row's pattern (L §4.6 step 1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pat {
    /// A stated key.
    Key(KeyVal),
    /// `*`, or a position the row leaves out.
    Star,
    /// `a..b`, in index order.
    Range(KeyVal, KeyVal),
}

/// A row's pattern as it prints, trailing stars dropped: `[me, 2026-03]` (refcore's
/// `Row.keys_text`). L §8.2.
#[must_use]
pub fn pattern_text(pattern: &[Pat]) -> String {
    let mut parts: Vec<String> = pattern
        .iter()
        .map(|p| match p {
            Pat::Key(key) => key.text(),
            Pat::Star => "*".to_owned(),
            Pat::Range(a, b) => format!("{}..{}", a.text(), b.text()),
        })
        .collect();
    while parts.last().is_some_and(|p| p == "*") {
        parts.pop();
    }
    if parts.is_empty() {
        return String::new();
    }
    format!("[{}]", parts.join(", "))
}

/// The days a governs span, a lifespan or a version's span covers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// The first day, or none: from the start of time.
    start: Option<Day>,
    /// The first day after it, or none: open.
    end: Option<Day>,
    /// `always`.
    always: bool,
    /// A lifespan whose end is a stated hole: its question (its end is then open).
    end_question: Option<String>,
}

impl Span {
    /// `always`.
    #[must_use]
    pub fn always() -> Span {
        Span {
            start: None,
            end: None,
            always: true,
            end_question: None,
        }
    }

    /// The days from `start` to the day before `end`, either open.
    #[must_use]
    pub fn between(start: Option<Day>, end: Option<Day>) -> Span {
        Span {
            start,
            end,
            always: false,
            end_question: None,
        }
    }

    /// From `start` to an end not yet known, a stated hole with its question (L §5.7).
    #[must_use]
    pub fn open_ended(start: Option<Day>, question: String) -> Span {
        Span {
            start,
            end: None,
            always: false,
            end_question: Some(question),
        }
    }

    /// The first day, if it has one.
    #[must_use]
    pub fn start(&self) -> Option<Day> {
        self.start
    }

    /// The first day after it, if it ends.
    #[must_use]
    pub fn end(&self) -> Option<Day> {
        self.end
    }

    /// Whether it is `always`.
    #[must_use]
    pub fn is_always(&self) -> bool {
        self.always
    }

    /// The question of a lifespan's hole end.
    #[must_use]
    pub fn end_question(&self) -> Option<&str> {
        self.end_question.as_deref()
    }

    /// Whether it holds `day` (refcore's `Span.contains`).
    #[must_use]
    pub fn contains(&self, day: Day) -> bool {
        self.always || (self.start.is_none_or(|s| s <= day) && self.end.is_none_or(|e| day < e))
    }

    /// Whether it shares a day with `other` (refcore's `Span.overlaps`).
    #[must_use]
    pub fn overlaps(&self, other: &Span) -> bool {
        if self.always || other.always {
            return true;
        }
        let start = match (self.start, other.start) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (a, b) => a.or(b),
        };
        let end = match (self.end, other.end) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        match (start, end) {
            (Some(a), Some(b)) => a < b,
            _ => true,
        }
    }

    /// The span as it prints: `always`, `2026-01-01..2026-12-31`, `2026-01-01..` (refcore's
    /// `Span.text`). L §8.2.
    #[must_use]
    pub fn text(&self) -> String {
        if self.always {
            return "always".to_owned();
        }
        let start = self.start.map_or_else(String::new, Day::text);
        let end = match (&self.end_question, self.end.and_then(Day::previous)) {
            (Some(question), _) => format!("?\"{question}\""),
            (None, Some(last)) => last.text(),
            (None, None) => String::new(),
        };
        format!("{start}..{end}")
    }
}

#[cfg(test)]
mod tests {
    use super::{KeyVal, Pat, Span, pattern_text};
    use crate::time::Day;

    fn day(y: u16, m: u8, d: u8) -> Day {
        Day::from_ymd(y, m, d).expect("a real day")
    }

    #[test]
    fn spans_hold_and_print() {
        let year = Span::between(Some(day(2026, 1, 1)), Some(day(2027, 1, 1)));
        assert!(
            year.contains(day(2026, 12, 31)) && !year.contains(day(2027, 1, 1)),
            "end excluded"
        );
        assert_eq!(
            year.text(),
            "2026-01-01..2026-12-31",
            "printed with its last day"
        );
        let from = Span::between(Some(day(2026, 6, 1)), None);
        assert!(year.overlaps(&from), "overlap");
        assert!(
            !Span::between(None, Some(day(2026, 1, 1))).overlaps(&year),
            "touching only"
        );
        assert_eq!(Span::always().text(), "always", "always");
    }

    #[test]
    fn patterns_drop_trailing_stars() {
        let me = KeyVal::Thing("me".to_owned());
        assert_eq!(
            pattern_text(&[Pat::Key(me.clone()), Pat::Star]),
            "[me]",
            "trailing star"
        );
        assert_eq!(
            pattern_text(&[Pat::Star, Pat::Key(me)]),
            "[*, me]",
            "leading star"
        );
        assert_eq!(pattern_text(&[Pat::Star]), "", "all stars");
    }
}
