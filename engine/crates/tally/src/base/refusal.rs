//! A refusal: a code of the catalogue, its message, and where in the reality it was met.
//! Spec: L §8.3 (each refusal names its file, line and item; which refusal prints), §8.4.
//! Twin: engine/tools/refcore/errors.py (`TallyError`, its `text`).
//! Invariants: a refusal always carries its catalogue code (R-76); its line is
//! `<CODE> <message>`, then ` (<where>)` when it was met in a file, as refcore prints it.

use super::code::{Code, Load};

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
}

impl Refusal {
    /// A refusal with no place in a file. L §8.3, §8.4.
    #[must_use]
    pub fn new(code: Code, message: String) -> Refusal {
        Refusal {
            code,
            message,
            place: None,
        }
    }

    /// A refusal met at `place`. L §8.3 ("each refusal names its file, line and item").
    #[must_use]
    pub fn at(code: Code, message: String, place: Place) -> Refusal {
        Refusal {
            code,
            message,
            place: Some(place),
        }
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

    /// The refusal as printed after `refused: `: the code, the message, then the place in
    /// parentheses (`LOAD-PARSE cannot parse: expected ":", got int (a.tally:4)`). L §8.3.
    #[must_use]
    pub fn text(&self) -> String {
        let code = self.code.as_str();
        match &self.place {
            None => format!("{code} {}", self.message),
            Some(Place::File(file)) => format!("{code} {} ({file})", self.message),
            Some(Place::Line(file, line)) => format!("{code} {} ({file}:{line})", self.message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Code, Load, Place, Refusal};

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
