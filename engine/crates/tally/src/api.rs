//! The engine's entry: the files a run is given and its request go in, the printed answer comes out.
//! Spec: E §1 (the run function); E §3.1 step 1 (files by full path); L §8.3 (which refusal
//! prints: every file is read before any is split or parsed).
//! Twin: engine/tools/refcore/cli.py (`read_files`, `run_texts`, `by_path`).
//! Invariants: reads no clock, environment or file (R-38, R-39); takes the files in the order
//! of their full paths by code point, whatever order they come in (R-40); never decodes lossily
//! (R-49).

use crate::NotImplemented;
use crate::base::Refusal;

/// One file the run is given: its path as the run names it, and its bytes, or none when the
/// path names no file or folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Given {
    /// The path, as the run names it (INTERFACE §4).
    pub path: String,
    /// The file's bytes; `None` for a path that names nothing (`LOAD-PATH`).
    pub bytes: Option<Vec<u8>>,
}

/// One file of the reality, read: its path and its text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// The path, as the run names it.
    pub path: String,
    /// The file's text, valid UTF-8.
    pub text: String,
}

/// Which asks a run answers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Asks {
    /// Every ask of the reality: the command line names none.
    Every,
    /// The asks named, in the order named.
    Named(Vec<String>),
}

/// How a `use` line's pin is taken (E §3.1 step 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pin {
    /// The sha256 of the member's canonical text.
    Canonical,
    /// The sha256 of the member's file bytes (`--pin=bytes`).
    Bytes,
}

/// What the run is asked: the asks to answer and the pin's form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// The asks to answer.
    pub asks: Asks,
    /// How a `use` line's pin is taken.
    pub pin: Pin,
}

/// The printed answer of a run: its lines, without their line ends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    /// The lines, in the order printed.
    lines: Vec<String>,
}

impl Answer {
    /// The one line of a refused run, `refused: <code> <message>`. L §8.3.
    #[must_use]
    pub fn refused(refusal: &Refusal) -> Answer {
        Answer {
            lines: vec![format!("refused: {}", refusal.text())],
        }
    }

    /// Whether the run was refused: its first line starts `refused: ` (INTERFACE §3).
    #[must_use]
    pub fn is_refused(&self) -> bool {
        self.lines
            .first()
            .is_some_and(|line| line.starts_with("refused: "))
    }

    /// The lines, in the order printed.
    #[must_use]
    pub fn lines(&self) -> &[String] {
        &self.lines
    }
}

/// Runs a reality's files on a request and answers with the printed lines. So far it reads
/// the files (L §8.3); the parser lands next (milestone S, group G3).
///
/// # Errors
///
/// [`NotImplemented`] once the files are read, until the parser and the rest land.
pub fn run(given: Vec<Given>, _request: &Request) -> Result<Answer, NotImplemented> {
    match read(given) {
        Err(refusal) => Ok(Answer::refused(&refusal)),
        Ok(_sources) => Err(NotImplemented),
    }
}

/// The read step: every file in the order of its full path, by code point; the first path
/// that names nothing is `LOAD-PATH`, the first file that isn't UTF-8 is `LOAD-PARSE`, each
/// named as the run gives it, whichever comes first in that order (L §8.3, tally 2.1.1).
fn read(mut given: Vec<Given>) -> Result<Vec<Source>, Refusal> {
    given.sort_by(|a, b| a.path.cmp(&b.path));
    let mut sources = Vec::with_capacity(given.len());
    for file in given {
        let Some(bytes) = file.bytes else {
            return Err(Refusal::no_path(&file.path));
        };
        let text = String::from_utf8(bytes).map_err(|_| Refusal::not_utf8(&file.path))?;
        sources.push(Source {
            path: file.path,
            text,
        });
    }
    Ok(sources)
}

#[cfg(test)]
mod tests {
    use super::{Answer, Asks, Given, Pin, Request, read, run};
    use crate::NotImplemented;

    fn every() -> Request {
        Request {
            asks: Asks::Every,
            pin: Pin::Canonical,
        }
    }

    fn given(files: &[(&str, Option<&[u8]>)]) -> Vec<Given> {
        files
            .iter()
            .map(|(path, bytes)| Given {
                path: (*path).to_owned(),
                bytes: bytes.map(<[u8]>::to_vec),
            })
            .collect()
    }

    fn line(answer: Result<Answer, NotImplemented>) -> String {
        let answer = answer.expect("an answer");
        assert!(answer.is_refused(), "refused");
        answer.lines().join("\n")
    }

    #[test]
    fn files_are_read_in_full_path_order() {
        let sources = read(given(&[
            ("a/b.tally", Some(b"b")),
            ("a.x.tally", Some(b"x")),
        ]))
        .expect("both read");
        let paths: Vec<&str> = sources.iter().map(|s| s.path.as_str()).collect();
        assert_eq!(paths, ["a.x.tally", "a/b.tally"], "'.' sorts before '/'");
    }

    #[test]
    fn the_first_refusal_by_full_path_prints() {
        let missing = given(&[
            ("m.tally", Some(b"model m")),
            ("z.tally", None),
            ("b.tally", None),
        ]);
        assert_eq!(
            line(run(missing, &every())),
            "refused: LOAD-PATH b.tally doesn't exist",
            "the first missing path by full path"
        );
        let bad = given(&[("z.tally", None), ("a.tally", Some(b"\xff"))]);
        assert_eq!(
            line(run(bad, &every())),
            "refused: LOAD-PARSE cannot parse: file is not UTF-8 (a.tally)",
            "bytes that aren't UTF-8 come first by path"
        );
    }

    #[test]
    fn a_reality_that_reads_goes_on_to_the_parser() {
        let files = given(&[("m.tally", Some(b"model m"))]);
        assert_eq!(run(files, &every()), Err(NotImplemented), "G3 next");
    }
}
