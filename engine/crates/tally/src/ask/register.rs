//! The register printed under an ask's line, its entries grouped and sorted by word, the law date and cut last; then the run's warnings.
//! Spec: L §8.2 (the register: its entries and their order, `held:` once per row and keys
//! read, `law date <d>, cut <K>`), §8.5 (warnings print sorted by code, then by text, by code
//! point).
//! Twin: engine/tools/refcore/ask.py (`Asker.register`, `merge_held`, `warning_lines`).
//! Invariants: every entry prints under its word in refcore's order; an entry of a word the
//! register doesn't print is `ENGINE-INVARIANT`, never dropped; lines sort by code point.

use std::collections::{BTreeMap, BTreeSet};

use crate::base::{Code, Refusal, Warn};
use crate::eval::{Collected, Ctx, Entries, Entry};
use crate::hole::Unknown;
use crate::syntax::Stop;
use crate::time::Day;

/// How an entry of a word prints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Style {
    /// `<word>: <text>`.
    Plain,
    /// `missing record counted by <text>`.
    Missing,
}

/// The words printed after `assumed:`, in the register's order, each with its style
/// (refcore's `register`: given, missing, pinned, unknown day, world, reading, then the walk's
/// words, then held; `before from` after `world` and `reading`, by ruling P43, refcore having
/// none). L §8.2, §5.5, E §9 item 3.
const WORDS: [(&str, Style); 15] = [
    ("given", Style::Plain),
    ("missing", Style::Missing),
    ("pinned", Style::Plain),
    ("unknown day", Style::Plain),
    ("world", Style::Plain),
    ("reading", Style::Plain),
    ("before from", Style::Plain),
    ("anchor", Style::Plain),
    ("pad", Style::Plain),
    ("adjustment", Style::Plain),
    ("unfulfilled", Style::Plain),
    ("open", Style::Plain),
    ("left out", Style::Plain),
    ("uncertain", Style::Plain),
    ("held", Style::Plain),
];

/// Adds to `met` an `unknown day:` entry for each unknown day of `unknown`, by its name: one
/// line per name, sorted with the other lines of its word (L §8.2 `unknown day:`, §4.4;
/// refcore's `register`, its `unknown`).
///
/// # Errors
///
/// `ENGINE-INVARIANT` past 2^31 entries in one run.
pub(super) fn name_unknown_days(
    (met, entries): (&mut Collected, &mut Entries),
    unknown: &[Unknown],
) -> Result<(), Stop> {
    for day in unknown {
        met.note(entries.intern(Entry {
            word: "unknown day",
            text: day.label.clone(),
            figure: String::new(),
        })?);
    }
    Ok(())
}

/// The register's lines for what a line met, in context `ctx`, without their indent: the
/// `assumed` entries merged by reason, the other words in their order, each sorted, then the
/// law date and the cut (refcore's `register`). L §8.2.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for an entry of a word the register doesn't print, or one the run's
/// table `entries` lacks.
pub fn register_lines(
    (met, entries): (&Collected, &Entries),
    ctx: Ctx,
) -> Result<Vec<String>, Stop> {
    let mut out = entry_lines(met, entries)?;
    out.push(match ctx.law {
        Some(law) => format!("law date {}, cut {}", law.text(), ctx.cut.text()),
        None => format!("cut {}", ctx.cut.text()),
    });
    Ok(out)
}

/// The register's lines under a `check` whose sides are read at different law dates: the
/// entries, then each side's law date (refcore's `register`, its `laws`). L §5.18, §8.2.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for an entry of a word the register doesn't print, or one the run's
/// table `entries` lacks.
pub(crate) fn check_register_lines(
    (met, entries): (&Collected, &Entries),
    ctx: Ctx,
    (left, right): (Option<Day>, Option<Day>),
) -> Result<Vec<String>, Stop> {
    let mut out = entry_lines(met, entries)?;
    out.push(format!(
        "law date {} (left), {} (right), cut {}",
        side_law_text(left),
        side_law_text(right),
        ctx.cut.text()
    ));
    Ok(out)
}

/// A check's side's law date as its register line prints it: the day, or `none` (refcore's
/// `laws[i] or 'none'`). L §5.18.
fn side_law_text(day: Option<Day>) -> String {
    match day {
        Some(day) => day.text(),
        None => "none".to_owned(),
    }
}

/// The register's entries' lines: `assumed` merged by reason, then the other words in their
/// order, each sorted, a `pinned:` date pin's days merged into spans, a `held:` row's days or
/// periods into runs (refcore's `register`). L §8.2.
fn entry_lines(met: &Collected, entries: &Entries) -> Result<Vec<String>, Stop> {
    let mut assumed: BTreeSet<(&str, &str)> = BTreeSet::new();
    let mut by_word: BTreeSet<(usize, String)> = BTreeSet::new();
    let mut pins: Vec<&str> = Vec::new();
    let mut held: Vec<&str> = Vec::new();
    for entry in entries.sorted(met)? {
        // an ask pin a read met under another claim: in the read set, printed by no view
        // (L §8.2 One read set; P45); an ask's pin of a cell, which says which pins a read
        // must take to be listed, likewise (tally 2.1.8 §4, #161)
        if entry.word == "pin met" || entry.word == "pin cell" {
            continue;
        }
        if entry.word == "assumed" {
            assumed.insert((entry.text.as_str(), entry.figure.as_str()));
            continue;
        }
        if entry.word == "pinned" {
            pins.push(&entry.text);
            continue;
        }
        if entry.word == "held" {
            held.push(&entry.text);
            continue;
        }
        let Some((at, (_, style))) = WORDS
            .iter()
            .enumerate()
            .find(|(_, (word, _))| *word == entry.word)
        else {
            return Err(Stop::Refused(Refusal::new(
                Code::EngineInvariant,
                format!(
                    "a register entry of a word the register doesn't print ({})",
                    entry.word
                ),
            )));
        };
        let line = match style {
            Style::Plain => format!("{}: {}", entry.word, entry.text),
            Style::Missing => format!("missing record counted by {}", entry.text),
        };
        by_word.insert((at, line));
    }
    for line in merged_pins(&pins) {
        by_word.insert((word_at("pinned")?, format!("pinned: {line}")));
    }
    for line in merged_held(&held) {
        by_word.insert((word_at("held")?, format!("held: {line}")));
    }
    let mut out = assumed_lines(&assumed);
    out.extend(by_word.into_iter().map(|(_, line)| line));
    Ok(out)
}

/// The place of `word` among the register's words. L §8.2.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for a word the register doesn't print.
fn word_at(word: &str) -> Result<usize, Stop> {
    WORDS.iter().position(|(w, _)| *w == word).ok_or_else(|| {
        Stop::Refused(Refusal::new(
            Code::EngineInvariant,
            format!("the register's words lack {word}"),
        ))
    })
}

/// The `held:` texts of one row at one keys (alike up to `, read for`) merged into one, the
/// days or periods it was read for in order, consecutive ones a run `a .. b`; keys are never
/// merged, and a text read for another form stays as it is (refcore's `merge_held`). L §8.2
/// `held:` ("It prints once per row and keys read, the periods it was read for merged into
/// maximal runs"; tally 2.3.1 R4, #245, #253 item 5).
fn merged_held(texts: &[&str]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rows: BTreeMap<&str, BTreeMap<(u8, i64), &str>> = BTreeMap::new();
    for text in texts {
        let read = text
            .rsplit_once(", read for ")
            .and_then(|(head, at)| Some((head, held_point(at)?, at)));
        let Some((head, point, at)) = read else {
            out.insert((*text).to_owned());
            continue;
        };
        match rows.get_mut(head) {
            Some(points) => {
                points.insert(point, at);
            }
            None => {
                rows.insert(head, BTreeMap::from([(point, at)]));
            }
        }
    }
    for (head, points) in rows {
        let mut runs: Vec<(&str, (u8, i64), &str)> = Vec::new();
        for (point, at) in points {
            match runs.last_mut() {
                Some((_, (kind, last), last_at))
                    if *kind == point.0 && last.checked_add(1) == Some(point.1) =>
                {
                    *last = point.1;
                    *last_at = at;
                }
                _ => runs.push((at, point, at)),
            }
        }
        let read_for: Vec<String> = runs
            .iter()
            .map(|(first, _, last)| {
                if first == last {
                    (*first).to_owned()
                } else {
                    format!("{first} .. {last}")
                }
            })
            .collect();
        out.insert(format!("{head}, read for {}", read_for.join(", ")));
    }
    out
}

/// A `read for` day or period as its kind's rank (a day, a month, a quarter, a year, refcore's
/// order of their names) and its place on its kind's line, consecutive ones a step apart;
/// none for another text (refcore's `_held_point`). L §8.2 `held:`.
fn held_point(text: &str) -> Option<(u8, i64)> {
    let mut parts = text.split('-');
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some(y), None, None, None) => Some((3, i64::from(digits(y, 4)?))),
        (Some(y), Some(q), None, None) if q.starts_with('Q') => {
            let quarter = i64::from(digits(q.get(1..)?, 1)?);
            if !(1..=4).contains(&quarter) {
                return None;
            }
            let at = i64::from(digits(y, 4)?)
                .checked_mul(4)?
                .checked_add(quarter)?;
            Some((2, at))
        }
        (Some(y), Some(mo), None, None) => {
            let at = i64::from(digits(y, 4)?)
                .checked_mul(12)?
                .checked_add(i64::from(digits(mo, 2)?))?;
            Some((1, at))
        }
        (Some(_), Some(_), Some(_), None) => {
            let day = iso_day(text)?;
            Some((0, i64::from(day.number())))
        }
        _ => None,
    }
}

/// The day an ISO date `YYYY-MM-DD` names; none for any other text (refcore's `_PIN_DAY`).
fn iso_day(text: &str) -> Option<Day> {
    let mut parts = text.split('-');
    let (Some(y), Some(mo), Some(d), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return None;
    };
    let (Ok(year), Ok(month), Ok(dom)) = (
        u16::try_from(digits(y, 4)?),
        u8::try_from(digits(mo, 2)?),
        u8::try_from(digits(d, 2)?),
    ) else {
        return None;
    };
    Day::from_ymd(year, month, dom)
}

/// The `pinned:` texts with the days of one `known` or `law` pin given by an expression
/// (`known <e> = <d>`, one per key it was formed at) merged into maximal spans, `= <a> .. <b>,
/// <c>` (refcore's `merge_pin_days`). L §8.2.
fn merged_pins(pins: &[&str]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut days: BTreeMap<&str, BTreeSet<Day>> = BTreeMap::new();
    for text in pins {
        let Some((head, day)) = pin_day(text) else {
            out.insert((*text).to_owned());
            continue;
        };
        match days.get_mut(head) {
            Some(set) => {
                set.insert(day);
            }
            None => {
                days.insert(head, BTreeSet::from([day]));
            }
        }
    }
    for (head, set) in days {
        let mut spans: Vec<(Day, Day)> = Vec::new();
        for day in set {
            match spans.last_mut() {
                Some((_, end)) if end.next() == Some(day) => *end = day,
                _ => spans.push((day, day)),
            }
        }
        let texts: Vec<String> = spans
            .iter()
            .map(|(a, b)| {
                if a == b {
                    a.text()
                } else {
                    format!("{} .. {}", a.text(), b.text())
                }
            })
            .collect();
        out.insert(format!("{head} = {}", texts.join(", ")));
    }
    out
}

/// A `known` or `law` pin's text split into its head and its day, `known e = 2026-01-05`
/// (refcore's `_PIN_DAY`); none for any other pin.
fn pin_day(text: &str) -> Option<(&str, Day)> {
    if !text.starts_with("known ") && !text.starts_with("law ") {
        return None;
    }
    let (head, day) = text.rsplit_once(" = ")?;
    Some((head, iso_day(day)?))
}

/// The number `len` decimal digits write; none for any other text.
fn digits(text: &str, len: usize) -> Option<u32> {
    if text.len() != len {
        return None;
    }
    text.chars().try_fold(0_u32, |n, c| {
        n.checked_mul(10)?.checked_add(c.to_digit(10)?)
    })
}

/// The `assumed` lines: one per reason, the figures that met it listed after it, sorted
/// (refcore's `register`, its `assumed` map). L §8.2.
fn assumed_lines(entries: &BTreeSet<(&str, &str)>) -> Vec<String> {
    let mut out = Vec::new();
    let mut reason: Option<&str> = None;
    let mut figures: Vec<&str> = Vec::new();
    for (text, figure) in entries {
        if reason != Some(*text) {
            if let Some(done) = reason {
                out.push(assumed_line(done, &figures));
            }
            reason = Some(*text);
            figures.clear();
        }
        if !figure.is_empty() {
            figures.push(figure);
        }
    }
    if let Some(done) = reason {
        out.push(assumed_line(done, &figures));
    }
    out
}

/// `assumed: <reason>`, with `(<figure>, …)` when figures met it.
fn assumed_line(reason: &str, figures: &[&str]) -> String {
    if figures.is_empty() {
        return format!("assumed: {reason}");
    }
    format!("assumed: {reason} ({})", figures.join(", "))
}

/// The run's warnings as printed after every ask: `warning: <code> <text>`, sorted by code,
/// then by text, by code point, a warning met twice printed twice (refcore's
/// `warning_lines`). L §8.5.
#[must_use]
pub fn warning_lines(warnings: &[(Warn, String)]) -> Vec<String> {
    let sorted: BTreeSet<(&str, &str, usize)> = warnings
        .iter()
        .enumerate()
        .map(|(at, (code, text))| (code.as_str(), text.as_str(), at))
        .collect();
    sorted
        .into_iter()
        .map(|(code, text, _)| format!("warning: {code} {text}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{register_lines, warning_lines};
    use crate::base::Warn;
    use crate::eval::{Collected, Ctx, Entries, Entry};
    use crate::time::Day;

    fn entry(word: &'static str, text: &str, figure: &str) -> Entry {
        Entry {
            word,
            text: text.to_owned(),
            figure: figure.to_owned(),
        }
    }

    #[test]
    fn the_register_prints_in_its_order() {
        let mut table = Entries::default();
        let mut met = Collected::default();
        for e in [
            entry("held", "x = 1 (v), read for 2027-01-01", ""),
            entry("assumed", "a guess", "b[2]"),
            entry("assumed", "a guess", "a[1]"),
            entry("assumed", "ix: open keys", ""),
            entry("missing", "v1", ""),
            entry("given", "g = 2 (v)", ""),
        ] {
            met.note(table.intern(e).expect("an id"));
        }
        let day = Day::from_ymd(2026, 3, 1).expect("a day");
        let law = Day::from_ymd(2026, 2, 1).expect("a day");
        let lines = register_lines((&met, &table), Ctx::new(Some(law), day)).expect("lines");
        assert_eq!(
            lines,
            [
                "assumed: a guess (a[1], b[2])",
                "assumed: ix: open keys",
                "given: g = 2 (v)",
                "missing record counted by v1",
                "held: x = 1 (v), read for 2027-01-01",
                "law date 2026-02-01, cut 2026-03-01",
            ],
            "refcore's order"
        );
        let bare = register_lines((&Collected::default(), &table), Ctx::new(None, day));
        assert_eq!(bare.expect("lines"), ["cut 2026-03-01"], "no law date");
    }

    #[test]
    fn held_prints_once_per_row_and_keys_its_reads_in_runs() {
        // L §8.2 `held:` (tally 2.3.1 R4): one line per row and keys read, the days or
        // periods in order, consecutive ones a run; keys never merged; another form as it is
        let mut table = Entries::default();
        let mut met = Collected::default();
        for text in [
            "x = 1 (v), read for 2027-01-04",
            "x = 1 (v), read for 2027-01-01",
            "x = 1 (v), read for 2027-01-02",
            "y[a] = 2 (v), read for 2027-03",
            "y[a] = 2 (v), read for 2027-02",
            "y[b] = 2 (v), read for 2027-03",
            "m = 1 (v), read for 2026-12",
            "m = 1 (v), read for 2027-01",
            "z = 3 (v), read for 2027-Q1",
            "z = 3 (v), read for 2026-Q4",
            "w = 1 (v), read for 2029",
            "w = 1 (v), read for 2027",
            "t row 2 (v), read for 2027-01-01",
            "q = 1 (v), read for soon",
        ] {
            met.note(table.intern(entry("held", text, "")).expect("an id"));
        }
        let day = Day::from_ymd(2026, 3, 1).expect("a day");
        let lines = register_lines((&met, &table), Ctx::new(None, day)).expect("lines");
        assert_eq!(
            lines,
            [
                "held: m = 1 (v), read for 2026-12 .. 2027-01",
                "held: q = 1 (v), read for soon",
                "held: t row 2 (v), read for 2027-01-01",
                "held: w = 1 (v), read for 2027, 2029",
                "held: x = 1 (v), read for 2027-01-01 .. 2027-01-02, 2027-01-04",
                "held: y[a] = 2 (v), read for 2027-02 .. 2027-03",
                "held: y[b] = 2 (v), read for 2027-03",
                "held: z = 3 (v), read for 2026-Q4 .. 2027-Q1",
                "cut 2026-03-01",
            ],
            "refcore's merge_held"
        );
    }

    #[test]
    fn a_word_the_register_does_not_print_is_an_invariant() {
        let mut table = Entries::default();
        let mut met = Collected::default();
        met.note(table.intern(entry("sample", "x", "")).expect("an id"));
        let day = Day::from_ymd(2026, 3, 1).expect("a day");
        assert!(
            register_lines((&met, &table), Ctx::new(None, day)).is_err(),
            "never dropped"
        );
    }

    #[test]
    fn warnings_sort_by_code_then_text() {
        let lines = warning_lines(&[
            (Warn::Unread, "b is never read".to_owned()),
            (Warn::Negative, "z".to_owned()),
            (Warn::Unread, "a is never read".to_owned()),
        ]);
        assert_eq!(
            lines,
            [
                "warning: WARN-NEGATIVE z",
                "warning: WARN-UNREAD a is never read",
                "warning: WARN-UNREAD b is never read",
            ],
            "by code, then text"
        );
    }
}
