//! A refusal: a code of the catalogue, its message, and where in the reality it was met.
//! Spec: L §8.3 (each refusal names its file, line and item; which refusal prints), §8.4;
//! INTERFACE §2 (a joint model's member refused: `member <alias>: <the refusal>`).
//! Twin: engine/tools/refcore/errors.py (`TallyError`, its `text`); joint.py (`MemberRefused`).
//! Invariants: a refusal always carries its catalogue code (R-76); its line is
//! `<CODE> <message>`, then ` (<where>)` when it was met in a file, as refcore prints it, after
//! `member <a>: ` when a joint model's member was refused; what a rule acts on (an unloaded
//! world, the id a refusal names) is its `Reach`, set where it is made, never read from its
//! message.

use super::code::{Code, Load};

/// What a refusal names, for the load rules that act on it (L §6.4): set where the refusal is
/// made, beside the message that says it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reach {
    /// An ask naming a world the run doesn't load: it refuses that ask alone (`LOAD-SCENARIO
    /// ask <q> names <w>, neither the model nor a loaded scenario`).
    UnloadedWorld,
    /// The id an `unknown id` or `ambiguous key` refusal names: a scenario's id, read outside
    /// it, is renamed `LOAD-SCENARIO` (refcore's `run_worlds`).
    Id(String),
}

/// Where a refusal was met: a file as the run names it, and a line of it when one is known.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Place {
    /// The whole file, as for bytes that aren't UTF-8.
    File(String),
    /// A line of a file, counted from 1.
    Line(String, u32),
}

/// A refusal of the run, at load or for a value, with its code (R-58: never a bare text).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    /// The catalogue's code.
    code: Code,
    /// The message, in the words of the code's row.
    message: String,
    /// Where it was met, when in a file.
    place: Option<Place>,
    /// The member of a joint model that was refused, its aliases from the run's model down
    /// (`f.b`), or `None` for the run's own refusal.
    member: Option<String>,
    /// What it names for the rules that act on it, if anything.
    reach: Option<Reach>,
}

impl Refusal {
    /// A refusal with no place in a file. L §8.3, §8.4.
    #[must_use]
    pub fn new(code: Code, message: String) -> Refusal {
        Refusal {
            code,
            message,
            place: None,
            member: None,
            reach: None,
        }
    }

    /// A refusal met at `place`. L §8.3 ("each refusal names its file, line and item").
    #[must_use]
    pub fn at(code: Code, message: String, place: Place) -> Refusal {
        Refusal {
            code,
            message,
            place: Some(place),
            member: None,
            reach: None,
        }
    }

    /// This refusal, naming `reach` for the rules that act on it. L §6.4.
    #[must_use]
    pub fn reaching(self, reach: Reach) -> Refusal {
        Refusal {
            reach: Some(reach),
            ..self
        }
    }

    /// What this refusal names for the rules that act on it. L §6.4.
    #[must_use]
    pub fn reach(&self) -> Option<&Reach> {
        self.reach.as_ref()
    }

    /// `LOAD-PATH`: a path the run is given names no file or folder, named as given. L §8.3.
    #[must_use]
    pub fn no_path(path: &str) -> Refusal {
        Refusal::new(Code::Load(Load::Path), format!("{path} doesn't exist"))
    }

    /// `LOAD-PARSE` for a file whose bytes aren't UTF-8, named as given. L §8.3; R-49.
    #[must_use]
    pub fn not_utf8(path: &str) -> Refusal {
        Refusal::at(
            Code::Load(Load::Parse),
            "cannot parse: file is not UTF-8".to_owned(),
            Place::File(path.to_owned()),
        )
    }

    /// The refusal's code. L §8.
    #[must_use]
    pub fn code(&self) -> Code {
        self.code
    }

    /// The message, without the code or the place. L §8.3.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Another code and message at this refusal's place: a load refusal of the base that a
    /// later rule names better (refcore's `run_worlds`: an id of a scenario read outside it,
    /// L §6.4). L §8.3.
    #[must_use]
    pub fn renamed(&self, code: Code, message: String) -> Refusal {
        Refusal {
            code,
            message,
            place: self.place.clone(),
            member: self.member.clone(),
            reach: None,
        }
    }

    /// This refusal at `place`, unless it names a place of its own: a refusal met reading a
    /// value, put at the line that states it. L §8.3.
    #[must_use]
    pub fn placed(self, place: Place) -> Refusal {
        Refusal {
            place: self.place.or(Some(place)),
            ..self
        }
    }

    /// This refusal as a joint model's member's, `chain` its aliases from the run's model down
    /// (refcore's `MemberRefused`). L §6.1; INTERFACE §2.
    #[must_use]
    pub fn of_member(self, chain: String) -> Refusal {
        Refusal {
            member: Some(chain),
            ..self
        }
    }

    /// Whether a joint model's member was refused.
    #[must_use]
    pub fn is_member(&self) -> bool {
        self.member.is_some()
    }

    /// The run's own refusal of a joint model with each member's prefix printed with its dot,
    /// `a.x` (refcore's `text().replace(SEP, '.')`); a member's refusal prints as the member
    /// gives it. L §6.1.
    #[must_use]
    pub fn shown(self) -> Refusal {
        if self.member.is_some() {
            return self;
        }
        Refusal {
            message: dotted(&self.message),
            ..self
        }
    }

    /// The refusal as printed after `refused: `: the code, the message, then the place in
    /// parentheses (`LOAD-PARSE cannot parse: expected ":", got int (a.tally:4)`), after
    /// `member <a>: ` for a joint model's member. L §8.3; INTERFACE §2.
    #[must_use]
    pub fn text(&self) -> String {
        let code = self.code.as_str();
        let line = match &self.place {
            None => format!("{code} {}", self.message),
            Some(Place::File(file)) => format!("{code} {} ({file})", self.message),
            Some(Place::Line(file, line)) => format!("{code} {} ({file}:{line})", self.message),
        };
        match &self.member {
            None => line,
            Some(chain) => format!("member {chain}: {line}"),
        }
    }
}

/// A joint model's line as it prints: each member's prefix, joined inside by `·`, printed with
/// its dot (`a.x`), outside the line's quoted texts only, so a `·` a text writes prints as
/// written (refcore's `replace(SEP, '.')`, which names never hold in a text). L §6.1, §2.4
/// (tally 2.3.1 Q2.34).
#[must_use]
pub(crate) fn dotted(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let (mut quoted, mut escaped) = (false, false);
    for c in line.chars() {
        if escaped {
            escaped = false;
        } else if c == '\\' && quoted {
            escaped = true;
        } else if c == '"' {
            quoted = !quoted;
        } else if c == '\u{b7}' && !quoted {
            out.push('.');
            continue;
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{Code, Load, Place, Refusal, dotted};

    #[test]
    fn a_written_middle_dot_prints_as_written() {
        // L §6.1, §2.4 (tally 2.3.1 Q2.34): a member's prefix prints with its dot, a `·`
        // inside a quoted text as written, an escaped quote kept inside it
        assert_eq!(
            dotted("a\u{b7}q_dot: ?\"the fee \u{b7} as \\\"it\\\" \u{b7}\" (a\u{b7}fee)"),
            "a.q_dot: ?\"the fee \u{b7} as \\\"it\\\" \u{b7}\" (a.fee)",
            "outside the quotes only"
        );
    }

    #[test]
    fn a_refusal_prints_as_refcore_does() {
        assert_eq!(
            Refusal::no_path("LOAD-PATH/nope.tally").text(),
            "LOAD-PATH LOAD-PATH/nope.tally doesn't exist",
            "no place"
        );
        assert_eq!(
            Refusal::not_utf8("a.tally").text(),
            "LOAD-PARSE cannot parse: file is not UTF-8 (a.tally)",
            "a file"
        );
        let at_line = Refusal::at(
            Code::Load(Load::Parse),
            "cannot parse: expected \":\", got int".to_owned(),
            Place::Line("LOAD-PATH/a.tally".to_owned(), 4),
        );
        assert_eq!(
            at_line.text(),
            "LOAD-PARSE cannot parse: expected \":\", got int (LOAD-PATH/a.tally:4)",
            "a line"
        );
        assert_eq!(at_line.code(), Code::Load(Load::Parse), "its code");
    }
}
