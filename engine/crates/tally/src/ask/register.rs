//! The register printed under an ask's line, its entries grouped and sorted by word, the law date and cut last; then the run's warnings.
//! Spec: L §8.2 (the register: its entries and their order, `law date <d>, cut <K>`), §8.5
//! (warnings print sorted by code, then by text, by code point).
//! Twin: engine/tools/refcore/ask.py (`Asker.register`, `warning_lines`).
//! Invariants: every entry prints under its word in refcore's order; an entry of a word the
//! register doesn't print is `ENGINE-INVARIANT`, never dropped; lines sort by code point.

use std::collections::BTreeSet;

use crate::base::{Code, Refusal, Warn};
use crate::eval::{Collected, Ctx};
use crate::syntax::Stop;

/// How an entry of a word prints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Style {
    /// `<word>: <text>`.
    Plain,
    /// `missing record counted by <text>`.
    Missing,
}

/// The words printed after `assumed:`, in the register's order, each with its style
/// (refcore's `register`: given, missing, world, then the walk's words, then held). L §8.2.
const WORDS: [(&str, Style); 11] = [
    ("given", Style::Plain),
    ("missing", Style::Missing),
    ("world", Style::Plain),
    ("anchor", Style::Plain),
    ("pad", Style::Plain),
    ("adjustment", Style::Plain),
    ("unfulfilled", Style::Plain),
    ("open", Style::Plain),
    ("left out", Style::Plain),
    ("uncertain", Style::Plain),
    ("held", Style::Plain),
];

/// The register's lines for what a line met, in context `ctx`, without their indent: the
/// `assumed` entries merged by reason, the other words in their order, each sorted, then the
/// law date and the cut (refcore's `register`). L §8.2.
///
/// # Errors
///
/// `ENGINE-INVARIANT` for an entry of a word the register doesn't print.
pub fn register_lines(met: &Collected, ctx: Ctx) -> Result<Vec<String>, Stop> {
    let mut assumed: BTreeSet<(&str, &str)> = BTreeSet::new();
    let mut by_word: BTreeSet<(usize, String)> = BTreeSet::new();
    for entry in &met.entries {
        if entry.word == "assumed" {
            assumed.insert((entry.text.as_str(), entry.figure.as_str()));
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
    let mut out = assumed_lines(&assumed);
    out.extend(by_word.into_iter().map(|(_, line)| line));
    out.push(match ctx.law {
        Some(law) => format!("law date {}, cut {}", law.text(), ctx.cut.text()),
        None => format!("cut {}", ctx.cut.text()),
    });
    Ok(out)
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
    use crate::eval::{Collected, Ctx, Entry};
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
        let mut met = Collected::default();
        met.entries
            .insert(entry("held", "x = 1 (v), read for 2027-01-01", ""));
        met.entries.insert(entry("assumed", "a guess", "b[2]"));
        met.entries.insert(entry("assumed", "a guess", "a[1]"));
        met.entries.insert(entry("assumed", "ix: open keys", ""));
        met.entries.insert(entry("missing", "v1", ""));
        met.entries.insert(entry("given", "g = 2 (v)", ""));
        let day = Day::from_ymd(2026, 3, 1).expect("a day");
        let law = Day::from_ymd(2026, 2, 1).expect("a day");
        let lines = register_lines(
            &met,
            Ctx {
                law: Some(law),
                cut: day,
            },
        )
        .expect("lines");
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
        let bare = register_lines(
            &Collected::default(),
            Ctx {
                law: None,
                cut: day,
            },
        );
        assert_eq!(bare.expect("lines"), ["cut 2026-03-01"], "no law date");
    }

    #[test]
    fn a_word_the_register_does_not_print_is_an_invariant() {
        let mut met = Collected::default();
        met.entries.insert(entry("sample", "x", ""));
        let day = Day::from_ymd(2026, 3, 1).expect("a day");
        assert!(
            register_lines(
                &met,
                Ctx {
                    law: None,
                    cut: day
                }
            )
            .is_err(),
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
