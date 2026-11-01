//! Holes: a missing fact as the set of questions it carries, and the one rule every operator applies to them.
//! Spec: L §8.1 (holes: they propagate, carry the union of their operands' questions each once,
//! and print every question sorted by code point with the figures it was reached at).
//! Twin: engine/tools/refcore/values.py (`Hole`, `hole`, `merge_holes`, `fmt_hole`) and
//! evaluate.py (`merge`).
//! Invariants: a hole is built only here (R-116): its fields are private, so no other module
//! can make one, and an operator that meets holes calls [`spread`], the one rule. A hole is
//! never 0 and never a default (R-27). The bound a later milestone adds to a hole has its slot
//! here, empty at S (seam U).

use std::collections::BTreeSet;

/// One question of a hole and the figure it was asked of (`pay[ana, 2026-01]`), or the empty
/// text for a hole no figure names (a record's field written `?"…"`). L §8.1.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Asked {
    /// The question.
    text: String,
    /// The figure it was asked of.
    figure: String,
}

impl Ord for Asked {
    /// The print order: the question's text by code point, then the figure (L §8.1).
    fn cmp(&self, other: &Asked) -> core::cmp::Ordering {
        self.text
            .cmp(&other.text)
            .then_with(|| self.figure.cmp(&other.figure))
    }
}

impl PartialOrd for Asked {
    /// As [`Ord`]: the print order (L §8.1).
    fn partial_cmp(&self, other: &Asked) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// A missing fact: the questions it carries, each once (L §8.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hole {
    /// The questions, sorted by their text, then the figure (code point order).
    questions: BTreeSet<Asked>,
    /// A bound on the unknown, which a later milestone states; empty at S (L §8.1 "a bound on
    /// the unknown can be added later", seam U).
    bound: Bound,
}

/// The slot for a hole's bound: none is ever stated at S (seam U).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Bound {
    /// No bound is known.
    Unbounded,
}

impl Hole {
    /// The hole of one question, asked of `figure`: a stated `?"…"`, the gap rule's question,
    /// a reading's, a band's (L §8.1 `HOLE-STATED`, `HOLE-UNGOVERNED`, `HOLE-NOREADING`).
    #[must_use]
    pub fn asked(text: &str, figure: &str) -> Hole {
        let mut questions = BTreeSet::new();
        questions.insert(Asked {
            text: text.to_owned(),
            figure: figure.to_owned(),
        });
        Hole {
            questions,
            bound: Bound::Unbounded,
        }
    }

    /// The questions as L §8.1 prints them: each text once, sorted by code point, with the
    /// figures it was reached at in brackets; a text no figure names prints alone.
    #[must_use]
    pub fn text(&self) -> String {
        let mut out: Vec<String> = Vec::new();
        let mut current: Option<(&str, Vec<&str>)> = None;
        for asked in &self.questions {
            match &mut current {
                Some((text, figures)) if *text == asked.text => {
                    if !asked.figure.is_empty() {
                        figures.push(&asked.figure);
                    }
                }
                _ => {
                    if let Some((text, figures)) = current.take() {
                        out.push(one_question(text, &figures));
                    }
                    let figures = if asked.figure.is_empty() {
                        Vec::new()
                    } else {
                        vec![asked.figure.as_str()]
                    };
                    current = Some((&asked.text, figures));
                }
            }
        }
        if let Some((text, figures)) = current {
            out.push(one_question(text, &figures));
        }
        if out.is_empty() {
            return "?".to_owned();
        }
        out.join(" ")
    }

    /// Whether two holes carry the same questions (L §8.1; an `if` whose branches are equal
    /// holes, an `at` window's classes, E §3.2).
    #[must_use]
    pub fn same_questions(&self, other: &Hole) -> bool {
        self.questions == other.questions
    }

    /// The question texts, sorted, for a register line or a test (L §8.2).
    #[must_use]
    pub fn question_texts(&self) -> Vec<String> {
        let mut out: Vec<String> = self.questions.iter().map(|a| a.text.clone()).collect();
        out.dedup();
        out
    }

    /// Whether a bound is known: never at S (seam U, L §8.1).
    #[must_use]
    pub fn is_bounded(&self) -> bool {
        match self.bound {
            Bound::Unbounded => false,
        }
    }
}

/// `?"text" (figures)`, the text's `\` and `"` escaped (L §8.1, §2.4).
fn one_question(text: &str, figures: &[&str]) -> String {
    let escaped = text.replace('\\', "\\\\").replace('"', "\\\"");
    if figures.is_empty() {
        return format!("?\"{escaped}\"");
    }
    format!("?\"{escaped}\" ({})", figures.join(", "))
}

/// The one rule of every operator that meets holes: the result is the hole of every
/// operand's questions, the union, each once; `None` when no operand is a hole (L §8.1 "it
/// carries the questions of every operand, `where` and term it evaluated").
#[must_use]
pub fn spread(operands: &[Option<&Hole>]) -> Option<Hole> {
    let mut out: Option<Hole> = None;
    for hole in operands.iter().flatten() {
        match &mut out {
            None => out = Some(Hole::clone(hole)),
            Some(acc) => acc.questions.extend(hole.questions.iter().cloned()),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{Hole, spread};

    #[test]
    fn a_hole_prints_its_questions_sorted_each_once() {
        let a = Hole::asked("Ile wynosi czynsz?", "rent");
        let b = Hole::asked("Ile wynosi czynsz?", "rent");
        let c = Hole::asked("b: no value", "b[x]");
        let d = Hole::asked("Ile wynosi czynsz?", "fee[2026-01]");
        let all = spread(&[Some(&a), None, Some(&b), Some(&c), Some(&d)]).expect("a hole");
        assert_eq!(
            all.text(),
            "?\"Ile wynosi czynsz?\" (fee[2026-01], rent) ?\"b: no value\" (b[x])",
            "sorted by code point, figures listed once"
        );
        assert!(spread(&[None, None]).is_none(), "no operand is a hole");
        assert!(!all.is_bounded(), "no bound at S");
    }

    #[test]
    fn a_question_with_no_figure_prints_alone_escaped() {
        let h = Hole::asked("say \"why\"", "");
        assert_eq!(h.text(), "?\"say \\\"why\\\"\"", "escaped, no brackets");
        assert!(h.same_questions(&Hole::asked("say \"why\"", "")), "same");
        assert_eq!(h.question_texts(), vec!["say \"why\"".to_owned()], "texts");
    }
}
