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
#[derive(Debug, Clone, Eq)]
pub struct Hole {
    /// The questions, sorted by their text, then the figure (code point order).
    questions: BTreeSet<Asked>,
    /// A bound on the unknown, which a later milestone states; empty at S (L §8.1 "a bound on
    /// the unknown can be added later", seam U).
    bound: Bound,
    /// What a `partial` ask line knows of it (L §5.18 Partial answers).
    lin: Lin,
}

/// What a `partial` ask line knows of a hole (L §5.18 Partial answers; refcore's `Hole.lin`):
/// nothing said, so one question is linear in itself and more are not; plain, as a rounding,
/// `min`, `max`, a band or a comparison made it; or its known part and its named holes.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Lin {
    /// Nothing said (refcore's `None`).
    Unsaid,
    /// Not linear in its holes (refcore's `False`).
    Plain,
    /// Its known part and the holes it is linear in.
    Linear(Box<Linear>),
}

/// A value linear in named holes: its known part and each hole with its coefficient, sorted
/// by the question's text, then its figure (L §5.18 Partial answers).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Linear {
    /// The known part; none where no known term met it.
    pub known: Option<crate::model::Value>,
    /// Each hole as (its question's text, the figure it was asked of), with its coefficient.
    pub terms: Vec<((String, String), crate::model::Value)>,
}

/// The slot for a hole's bound: none is ever stated at S (seam U).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Bound {
    /// No bound is known.
    Unbounded,
}

/// Two holes are the same hole when they carry the same questions and bound: what a `partial`
/// line knows of them is how one line prints it, not the hole (L §8.1, §5.18).
impl PartialEq for Hole {
    /// The same questions and bound, whatever a `partial` line knows of them. L §8.1, §5.18.
    fn eq(&self, other: &Hole) -> bool {
        self.questions == other.questions && self.bound == other.bound
    }
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
            lin: Lin::Unsaid,
        }
    }

    /// The hole of `self`'s questions that is linear in its named holes as `lin` says, for a
    /// `partial` ask line (L §5.18 Partial answers).
    #[must_use]
    pub fn linear(&self, lin: Linear) -> Hole {
        Hole {
            lin: Lin::Linear(Box::new(lin)),
            ..self.clone()
        }
    }

    /// The hole of `self`'s questions, not linear in them: a rounding, `min`, `max`, a band,
    /// a comparison or a product of two holes met it (L §5.18 Partial answers).
    #[must_use]
    pub fn plain(&self) -> Hole {
        Hole {
            lin: Lin::Plain,
            ..self.clone()
        }
    }

    /// What it is linear in, where a `partial` line can say: its own known part and holes,
    /// or one question alone, which is 1 × that hole; none for a plain hole or several
    /// questions nothing said of (refcore's `lin_of`). L §5.18 Partial answers.
    #[must_use]
    pub fn linear_part(&self) -> Option<Linear> {
        match &self.lin {
            Lin::Plain => None,
            Lin::Linear(lin) => Some(Linear::clone(lin)),
            Lin::Unsaid => {
                let mut all = self.questions.iter();
                let (Some(one), None) = (all.next(), all.next()) else {
                    return None;
                };
                Some(Linear {
                    known: None,
                    terms: vec![(
                        (one.text.clone(), one.figure.clone()),
                        crate::model::Value::int(1),
                    )],
                })
            }
        }
    }

    /// Its known part and holes as a `partial` line prints them, where an operation computed
    /// them; none for a hole nothing said of or a plain one (refcore's `fmt_value`, its
    /// `partial and v.lin`). L §5.18 Partial answers.
    #[must_use]
    pub fn printed_linear(&self) -> Option<&Linear> {
        match &self.lin {
            Lin::Linear(lin) => Some(lin),
            Lin::Unsaid | Lin::Plain => None,
        }
    }

    /// Whether the two holes carry the same questions. L §8.1.
    #[must_use]
    pub fn same_questions(&self, other: &Hole) -> bool {
        self.questions == other.questions
    }

    /// `HOLE-PRERUN`'s hole, asked of `figure`: a walked value of a day before the run's
    /// start S, which nothing walks (L §8.1; E §2).
    #[must_use]
    pub fn before_run(figure: &str) -> Hole {
        Hole::asked("before the run's start", figure)
    }

    /// `HOLE-POSTRUN`'s hole, asked of `figure`: a walked value of a day after the run's end
    /// E, which nothing walks (L §8.1; E §2, §6.3).
    #[must_use]
    pub fn after_run(figure: &str) -> Hole {
        Hole::asked("after the run's end", figure)
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

    /// The questions with the day each gap-rule question names (`…: no value at <d>`) written
    /// `D`, sorted, each once; and whether any named one. Two days of one law-date class give
    /// holes alike but for that day (L §4.6 step 5; E §3.2; refcore's `def_window`, `norm`).
    #[must_use]
    pub fn undated(&self) -> (Vec<(String, String)>, bool) {
        let mut out = Vec::with_capacity(self.questions.len());
        let mut dated = false;
        for asked in &self.questions {
            let text = match gap_stem(&asked.text) {
                Some(stem) => {
                    dated = true;
                    format!("{stem}: no value at D")
                }
                None => asked.text.clone(),
            };
            out.push((text, asked.figure.clone()));
        }
        out.sort();
        out.dedup();
        (out, dated)
    }

    /// The question texts, sorted, for a register line or a test (L §8.2).
    #[must_use]
    pub fn question_texts(&self) -> Vec<String> {
        let mut out: Vec<String> = self.questions.iter().map(|a| a.text.clone()).collect();
        out.dedup();
        out
    }
}

/// The text before a gap-rule question's `: no value at <yyyy-mm-dd>` at its end, or none
/// for another question (L §4.6 step 5).
fn gap_stem(text: &str) -> Option<&str> {
    let (stem, day) = text.rsplit_once(": no value at ")?;
    let shape = day.len() == 10
        && day.char_indices().all(|(i, c)| match i {
            4 | 7 => c == '-',
            _ => c.is_ascii_digit(),
        });
    shape.then_some(stem)
}

/// One named hole of a partial answer, `?"text" (figure)` (L §5.18 Partial answers, §8.1).
#[must_use]
pub fn atom_text(text: &str, figure: &str) -> String {
    if figure.is_empty() {
        return one_question(text, &[]);
    }
    one_question(text, &[figure])
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
            None => {
                // an operation that merely spreads them can't tell what its hole is linear in:
                // it is plain (refcore's `merge`, whose `lin` is `False`; L §5.18)
                out = Some(Hole {
                    lin: Lin::Plain,
                    ..Hole::clone(hole)
                });
            }
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
    }

    #[test]
    fn a_question_with_no_figure_prints_alone_escaped() {
        let h = Hole::asked("say \"why\"", "");
        assert_eq!(h.text(), "?\"say \\\"why\\\"\"", "escaped, no brackets");
        assert_eq!(h, Hole::asked("say \"why\"", ""), "same");
        assert_eq!(h.question_texts(), vec!["say \"why\"".to_owned()], "texts");
    }
}
