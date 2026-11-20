//! Holes: a missing fact as the set of questions it carries, and the one rule every operator applies to them.
//! Spec: L §8.1 (holes: they propagate, carry the union of their operands' questions each once,
//! and print every question sorted by code point with the figures it was reached at).
//! Twin: engine/tools/refcore/values.py (`Hole`, `hole`, `merge_holes`, `fmt_hole`) and
//! evaluate.py (`merge`).
//! Invariants: a hole is built only here (R-116): its fields are private, so no other module
//! can make one, and an operator that meets holes calls [`spread`], the one rule. A hole is
//! never 0 and never a default (R-27). The bound a later milestone adds to a hole has its slot
//! here, empty at S (seam U). The unknown days a hole carries (L §4.4, which the register's
//! `unknown day:` lines name, §8.2) are kept by question, spread with them, go with a question
//! a hole loses, and are no part of its identity.

use std::collections::{BTreeMap, BTreeSet};

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

impl Asked {
    /// The question a read `read` asks where a window put it for a balance (`date within …
    /// for a balance`), or the question as it is (the plan's §14.1). L §4.4.
    fn for_read(&self, read: &str) -> Asked {
        match self.text.strip_suffix(" for a balance") {
            Some(head) if head.starts_with("date within ") => Asked {
                text: format!("{head} for {read}"),
                figure: self.figure.clone(),
            },
            _ => self.clone(),
        }
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
    /// The unknown days each question was made of, in the order the line's reads met them,
    /// each once: what the register's `unknown day:` lines name (L §4.4: "a hole made of
    /// either carries them too"; §8.2). Kept by question, as refcore's `CARRY` keys them, so a
    /// hole that loses a question loses the days only it carried. No part of the hole's
    /// identity.
    carry: BTreeMap<Asked, Vec<Unknown>>,
}

/// An unknown day a value or a hole carries: its identity (the read that gave it, or a
/// calendar's entry), the name the register's `unknown day:` line prints and its window's
/// own text, which its question names (refcore's `window_id`). L §4.4, §8.2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unknown {
    /// Its identity.
    pub id: String,
    /// Its name: the cell a caller pins as its read names it (`wd`, `held_from[b]`),
    /// `added <entry> of <calendar>`, its movement, or the window's own text (R21 on #241).
    pub label: String,
    /// Its window's own text (`window(2026-03-05, 2026-03-07)`).
    pub text: String,
}

/// `HOLE-IMPRECISE`'s hole: `date within <window> not precise enough for <what>`, asked of
/// `figure`, carrying the unknown days `carry`; where they are two or more, one question per
/// unknown day, each naming its own window, asked of `label` where one is given, else of the
/// day's own name, and carrying that day alone (refcore's `imprecise_hole`). L §8.1
/// (`HOLE-IMPRECISE`: "one question per unknown day", R21 on #241), §4.4.
#[must_use]
pub fn imprecise(
    (window, what): (&str, &str),
    figure: &str,
    (carry, label): (&[Unknown], Option<&str>),
) -> Hole {
    let mut out: Option<Hole> = None;
    if let [_, _, ..] = carry {
        for day in carry {
            let text = format!("date within {} not precise enough for {what}", day.text);
            let one = Hole::asked(&text, label.unwrap_or(&day.label))
                .carrying(core::slice::from_ref(day));
            out = spread(&[out.as_ref(), Some(&one)]);
        }
    }
    if let Some(per_day) = out {
        return per_day;
    }
    let text = format!("date within {window} not precise enough for {what}");
    Hole::asked(&text, figure).carrying(carry)
}

/// `into` with each unknown day of `more` it lacks added at its end, in order (refcore's
/// `carry_union`). L §4.4.
pub fn carry_union(into: &mut Vec<Unknown>, more: &[Unknown]) {
    for u in more {
        if !into.iter().any(|have| have.id == u.id) {
            into.push(u.clone());
        }
    }
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
            carry: BTreeMap::new(),
        }
    }

    /// The hole, each of its questions carrying the unknown days `unknowns` too, after its own
    /// (refcore's `carry_hole`). L §4.4; tally 2.1.8 R13.
    #[must_use]
    pub fn carrying(mut self, unknowns: &[Unknown]) -> Hole {
        if unknowns.is_empty() {
            return self;
        }
        for q in &self.questions {
            let days = self.carry.entry(q.clone()).or_insert_with(Vec::new);
            carry_union(days, unknowns);
        }
        self
    }

    /// The hole a read `read` meets where a window's test reaches it: each question a window
    /// put for a balance asked for `read` instead, carrying the days it did, the rest as they
    /// are (the plan's §14.1: a hole's printed question is the read's; tally 2.1.8 item 77;
    /// refcore's `_reask`). L §4.4.
    #[must_use]
    pub fn for_read(&self, read: &str) -> Hole {
        let mut out = self.clone();
        out.questions = self.questions.iter().map(|q| q.for_read(read)).collect();
        out.carry = BTreeMap::new();
        for (q, days) in &self.carry {
            carry_union(
                out.carry.entry(q.for_read(read)).or_insert_with(Vec::new),
                days,
            );
        }
        out
    }

    /// The unknown days it carries, each once: its questions' in their order, each
    /// question's in the order met (refcore's `carried` of a hole). L §4.4.
    #[must_use]
    pub fn carried(&self) -> Vec<Unknown> {
        let mut out = Vec::new();
        for days in self.carry.values() {
            carry_union(&mut out, days);
        }
        out
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

    /// Whether the two holes carry a question in common. L §8.1.
    #[must_use]
    pub fn shares(&self, other: &Hole) -> bool {
        !self.questions.is_disjoint(&other.questions)
    }

    /// Whether the two holes carry the same questions. L §8.1.
    #[must_use]
    pub fn same_questions(&self, other: &Hole) -> bool {
        self.questions == other.questions
    }

    /// The hole of the questions this one carries and `other` doesn't, with the unknown days
    /// those questions carry; none where `other` carries every one (refcore's `perm - e.qs`,
    /// each question's days its `CARRY[q]`). L §8.1, §4.4.
    #[must_use]
    pub fn without(&self, other: &Hole) -> Option<Hole> {
        let questions: BTreeSet<Asked> = self
            .questions
            .difference(&other.questions)
            .cloned()
            .collect();
        if questions.is_empty() {
            return None;
        }
        let carry = self
            .carry
            .iter()
            .filter(|(q, _)| questions.contains(*q))
            .map(|(q, days)| (q.clone(), days.clone()))
            .collect();
        Some(Hole {
            questions,
            bound: self.bound,
            lin: Lin::Plain,
            carry,
        })
    }

    /// The hole of the questions this one carries that a window's day leaves open, `date
    /// within …`, with the unknown days they carry; none where it carries none (refcore's
    /// `q[0].startswith('date within ')`). L §4.4, §5.19 l.1726; P32.
    #[must_use]
    pub fn of_windows(&self) -> Option<Hole> {
        let questions: BTreeSet<Asked> = self
            .questions
            .iter()
            .filter(|q| q.text.starts_with("date within "))
            .cloned()
            .collect();
        if questions.is_empty() {
            return None;
        }
        let carry = self
            .carry
            .iter()
            .filter(|(q, _)| questions.contains(*q))
            .map(|(q, days)| (q.clone(), days.clone()))
            .collect();
        Some(Hole {
            questions,
            bound: self.bound,
            lin: Lin::Plain,
            carry,
        })
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
/// operand's questions, the union, each once, and their unknown days, in order; `None` when
/// no operand is a hole (L §8.1 "it carries the questions of every operand, `where` and term
/// it evaluated"; L §4.4).
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
            Some(acc) => {
                acc.questions.extend(hole.questions.iter().cloned());
                for (q, days) in &hole.carry {
                    carry_union(acc.carry.entry(q.clone()).or_insert_with(Vec::new), days);
                }
            }
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
