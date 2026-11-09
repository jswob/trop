//! The engine's entry: the files a run is given and its request go in, the printed answer comes out.
//! Spec: E §1 (the run function); E §3.1 step 1 (files by full path); L §8.3 (which refusal
//! prints: every file is read before any is split, and split before any is parsed); L §6.4
//! (a reality with scenarios runs as worlds); L §5.18 (the asks answered once the reality
//! loads); E §11 (`REQ-ASK`); L §6.1 (a joint model's members read from the files given, then
//! from the file table; its lines printed with each member's prefix and its dot).
//! Twin: engine/tools/refcore/cli.py (`read_files`, `run_texts`, `_run_texts`, `run_one`,
//! `by_path`, `joint`'s `shown`).
//! Invariants: reads no clock, environment or file (R-38, R-39); takes the files in the order
//! of their full paths by code point, whatever order they come in (R-40); never decodes lossily
//! (R-49).

use crate::NotImplemented;
use crate::ask::{Answered, answer, answer_worlds, report_lines};
use crate::base::Refusal;
use crate::eval::Memo;
use crate::load::{Loaded, load_run, split_files};
use crate::oracle::Days;
use crate::syntax::Stop;
use crate::walk::Lows;
use std::collections::{BTreeMap, BTreeSet};

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
    /// Which days the walk steps: the event walk, or the oracle's every day (P §2.2).
    pub days: Days,
}

/// The printed answer of a run: its lines, without their line ends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    /// The lines, in the order printed.
    lines: Vec<String>,
    /// What the run did, in P §1.1's units: the `counters` build only (BUILDER §13.2).
    #[cfg(feature = "counters")]
    counters: crate::Counters,
}

impl Answer {
    /// The one line of a refused run, `refused: <code> <message>`. L §8.3.
    #[must_use]
    pub fn refused(refusal: &Refusal) -> Answer {
        Answer {
            lines: vec![format!("refused: {}", refusal.text())],
            #[cfg(feature = "counters")]
            counters: crate::Counters::default(),
        }
    }

    /// What the run did, in P §1.1's units: the `counters` build only (BUILDER §13.2).
    #[cfg(feature = "counters")]
    #[must_use]
    pub fn counters(&self) -> &crate::Counters {
        &self.counters
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

/// Runs a reality's files on a request and answers with the printed lines: it reads the
/// files, splits them all and reads their scenario headers (L §8.3), loads the reality, or its
/// worlds, the base and each scenario's (L §6.4), then answers the asks (L §5.18).
///
/// # Errors
///
/// [`NotImplemented`] for a construct the engine doesn't build yet.
pub fn run(given: Vec<Given>, request: &Request) -> Result<Answer, NotImplemented> {
    run_with_memo(given, request, Memo::Classes)
}

/// [`run`] with the file table's files, `disk`, where a joint model's members not among the
/// files given are read (E §3.1 step 1; INTERFACE §2, §4): the CLI's entry.
///
/// # Errors
///
/// As [`run`].
pub fn run_in(
    given: Vec<Given>,
    disk: &BTreeMap<&str, &[u8]>,
    request: &Request,
) -> Result<Answer, NotImplemented> {
    answered(given, disk, request, (Memo::Classes, Lows::Tree))
}

/// Loads a reality's files and answers with what load prints when no ask is answered: its one
/// refusal, or each refused scenario's line and the warnings (L §8, §8.5). The fuzz target
/// `load` calls it.
///
/// # Errors
///
/// [`NotImplemented`] for a construct the engine doesn't build yet.
pub fn load_report(given: Vec<Given>) -> Result<Answer, NotImplemented> {
    let report = loaded(given, &BTreeMap::new(), Pin::Canonical).map(|loaded| {
        let lines = report_lines(&loaded);
        Answered::printed(if is_joint(&loaded) {
            joint_lines(lines)
        } else {
            lines
        })
    });
    finish(report)
}

/// [`run`] with each ask's memo keeping its cells as `memo` says: shared by a law date class,
/// as [`run`] keeps them, or at each exact law date, the plain twin the equivalence test
/// compares it with (E §3.2, P §3.3).
///
/// # Errors
///
/// As [`run`].
pub fn run_with_memo(
    given: Vec<Given>,
    request: &Request,
    memo: Memo,
) -> Result<Answer, NotImplemented> {
    answered(given, &BTreeMap::new(), request, (memo, Lows::Tree))
}

/// [`run`] with each ask's engine reading a written projection's common case as `lows` says:
/// from fence F5's running sums, as [`run`] reads it (`Lows::Tree`), or through the
/// projection's own walk (`Lows::Walk`), the plain twin the equivalence test compares it with
/// (E §4.3, P §2.3). Both print the same bytes for every reality and request: a difference is
/// a fault of the fence (R-130), which is how a test of the fence uses it.
///
/// # Errors
///
/// As [`run`].
pub fn run_with_lows(
    given: Vec<Given>,
    request: &Request,
    lows: Lows,
) -> Result<Answer, NotImplemented> {
    answered(given, &BTreeMap::new(), request, (Memo::Classes, lows))
}

/// The run's answer, its members read from the files given, then from `disk`; a joint model's
/// lines printed with each member's prefix and its dot (refcore's `_run_texts` and `joint`).
fn answered(
    given: Vec<Given>,
    disk: &BTreeMap<&str, &[u8]>,
    request: &Request,
    how: (Memo, Lows),
) -> Result<Answer, NotImplemented> {
    let (asks, days) = (&request.asks, request.days);
    finish(loaded(given, disk, request.pin).and_then(|loaded| {
        let mut out = match &loaded {
            Loaded::One(m) => answer(m, asks, days, how),
            Loaded::Worlds(w) => answer_worlds(w, asks, days, how),
        }?;
        if is_joint(&loaded) {
            out.lines = joint_lines(out.lines);
        }
        Ok(out)
    }))
}

/// Whether the run's reality is a joint model's.
fn is_joint(loaded: &Loaded) -> bool {
    match loaded {
        Loaded::One(m) => m.joint.is_some(),
        Loaded::Worlds(w) => w.base.joint.is_some(),
    }
}

/// A joint model's printed lines (refcore's `joint`, its `shown`): each member's prefix printed
/// with its dot; a warning that a member's item is unread left out, since each member warns
/// alone. L §6.1, §8.5.
fn joint_lines(lines: Vec<String>) -> Vec<String> {
    lines
        .into_iter()
        .filter(|line| !(line.starts_with("warning: WARN-UNREAD") && line.contains('\u{b7}')))
        .map(|line| line.replace('\u{b7}', "."))
        .collect()
}

/// The run's files read, split with their scenario headers, and loaded: one reality, or its
/// worlds, a joint model's members read from the files given, then from `disk` (refcore's
/// `_run_texts`). E §3.1 step 1; L §8.3, §6.1, §6.4.
fn loaded(given: Vec<Given>, disk: &BTreeMap<&str, &[u8]>, pin: Pin) -> Result<Loaded, Stop> {
    let sources = read(given).map_err(Stop::Refused)?;
    let split = split_files(sources).map_err(Stop::Refused)?;
    load_run(&split, disk, pin)
}

/// The printed answer: the lines, or a refusal's one line (L §8.3); [`NotImplemented`] for a
/// construct the engine doesn't build yet.
fn finish(outcome: Result<Answered, Stop>) -> Result<Answer, NotImplemented> {
    match outcome {
        Ok(answered) => Ok(Answer {
            lines: answered.lines,
            #[cfg(feature = "counters")]
            counters: answered.counts,
        }),
        Err(Stop::Refused(refusal)) => Ok(Answer::refused(&refusal)),
        Err(Stop::NotBuilt(_) | Stop::Deep) => Err(NotImplemented),
    }
}

/// The canonical text of each file `reprint` is given, then the reality's hash (L §2.4, E §3.1
/// step 1; refcore's `reprint_paths`, `reprint_lines`): each file as `(given, name)`, its full
/// path and the name it prints under. The files are read, printed and hashed in the order of
/// their full paths, by code point; each prints as `--- <name>` and its lines, then one line
/// `sha256 <hash>`. A read, parse or order refusal prints alone, as `run`'s does.
///
/// # Errors
///
/// [`NotImplemented`] for a form whose order the engine doesn't build yet (a member's keys).
pub fn reprint(files: Vec<(Given, String)>) -> Result<Answer, NotImplemented> {
    reprint_in(files, &BTreeMap::new())
}

/// [`reprint`] with the file table's files, `disk`: a joint model's members are found among
/// the files given, by their full paths, then on `disk` where it holds the model's file, and
/// pinned as `run` pins them, before anything prints; each member's files given are a reality
/// of their own, printed under the names given (E §3.1 step 1; INTERFACE §1; tally 2.1.6,
/// #74, #91). The CLI's entry.
///
/// # Errors
///
/// As [`reprint`].
pub fn reprint_in(
    mut files: Vec<(Given, String)>,
    disk: &BTreeMap<&str, &[u8]>,
) -> Result<Answer, NotImplemented> {
    files.sort_by(|(a, x), (b, y)| (&a.path, x).cmp(&(&b.path, y)));
    let mut given = Vec::with_capacity(files.len());
    let mut sources = Vec::with_capacity(files.len());
    for (file, name) in files {
        let Some(bytes) = file.bytes else {
            return Ok(Answer::refused(&Refusal::no_path(&file.path)));
        };
        let Ok(text) = String::from_utf8(bytes) else {
            return Ok(Answer::refused(&Refusal::not_utf8(&file.path)));
        };
        given.push(Source {
            path: file.path,
            text: text.clone(),
        });
        sources.push(Source { path: name, text });
    }
    let printed = match crate::load::reprint_members(&given, disk).and_then(|members| {
        crate::syntax::reprint_among(&sources, &named(&members, &given, &sources))
    }) {
        Ok(printed) => printed,
        Err(Stop::Refused(refusal)) => return Ok(Answer::refused(&refusal)),
        Err(Stop::NotBuilt(_) | Stop::Deep) => return Err(NotImplemented),
    };
    let mut lines = Vec::new();
    for (name, text) in &printed {
        lines.push(format!("--- {name}"));
        lines.extend(text.trim_end_matches('\n').split('\n').map(str::to_owned));
    }
    lines.push(format!("sha256 {}", crate::syntax::printed_hash(&printed)));
    Ok(Answer {
        lines,
        #[cfg(feature = "counters")]
        counters: crate::Counters::default(),
    })
}

/// Each member's files given, by the names they print under (`given` and `sources` are the
/// same files, by their paths as the run keys them and by those names); a member's file found
/// on the disk is none of them.
fn named(
    members: &[BTreeSet<String>],
    given: &[Source],
    sources: &[Source],
) -> Vec<BTreeSet<String>> {
    let names: BTreeMap<&str, &str> = given
        .iter()
        .zip(sources)
        .map(|(file, source)| (file.path.as_str(), source.path.as_str()))
        .collect();
    let mut out = Vec::with_capacity(members.len());
    for paths in members {
        let mut group = BTreeSet::new();
        for path in paths {
            if let Some(name) = names.get(path.as_str()) {
                group.insert((*name).to_owned());
            }
        }
        out.push(group);
    }
    out
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
            days: crate::Days::Events,
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
    fn every_file_is_split_before_any_is_parsed() {
        let files = given(&[
            ("a.tally", Some(b"model m\n")),
            ("b.tally", Some(b"model b\n\ttally 2\n")),
        ]);
        assert_eq!(
            line(run(files, &every())),
            "refused: LOAD-PARSE cannot parse: tab (b.tally:2)",
            "b's tab is met when it is split, before a's header is parsed"
        );
    }

    const MODEL: &str =
        "model m\n  tally 2\n  on 2026-01-01\n  start 2026-01-01\n  end 2026-12-31\n";

    fn lines(files: Vec<Given>, request: &Request) -> Vec<String> {
        let answer = run(files, request).expect("an answer");
        answer.lines().to_vec()
    }

    #[test]
    fn a_reality_with_no_asks_prints_nothing() {
        let files = given(&[("m.tally", Some(MODEL.as_bytes()))]);
        assert!(lines(files, &every()).is_empty(), "no ask, no warning");
    }

    #[test]
    fn asks_print_in_id_order_or_as_named() {
        let text = format!(
            "{MODEL}\nsource card \"a card\"\n\nunit PLN: currency minor 0.01\n\ninput fee: money PLN\n\nversion v\n  cite card\n  governs always\n  known 2026-01-01\n  fee = 12.50\n\nask b: fee, fee + fee\n\nask a: fee + fee law 2026-03-01\n"
        );
        let files = || given(&[("m.tally", Some(text.as_bytes()))]);
        assert_eq!(
            lines(files(), &every()),
            [
                "a: 25.00 PLN",
                "  law date 2026-03-01, cut 2026-01-01",
                "b: (12.50 PLN, 25.00 PLN)",
                "  cut 2026-01-01",
            ],
            "by id"
        );
        let named = Request {
            asks: Asks::Named(vec!["b".to_owned()]),
            pin: Pin::Canonical,
            days: crate::Days::Events,
        };
        assert_eq!(
            lines(files(), &named),
            ["b: (12.50 PLN, 25.00 PLN)", "  cut 2026-01-01"],
            "named"
        );
        let missing = Request {
            asks: Asks::Named(vec!["b".to_owned(), "c".to_owned()]),
            pin: Pin::Canonical,
            days: crate::Days::Events,
        };
        assert_eq!(
            lines(files(), &missing),
            ["refused: REQ-ASK no ask c"],
            "the run refused, before b is answered"
        );
    }

    #[test]
    fn a_load_refusal_prints() {
        let text = format!("{MODEL}\ninput x: num\n\ninput x: num\n");
        let files = given(&[("m.tally", Some(text.as_bytes()))]);
        assert_eq!(
            line(run(files, &every())),
            "refused: LOAD-DUP x is declared twice (m.tally:9)",
            "the second x"
        );
    }

    fn probe() -> Request {
        Request {
            asks: Asks::Named(vec!["zz".to_owned()]),
            pin: Pin::Canonical,
            days: crate::Days::Events,
        }
    }

    #[test]
    fn a_reality_with_scenarios_runs_as_worlds() {
        let base = format!(
            "{MODEL}\nsource card \"a card\"\n\nunit PLN: currency minor 0.01\n\ninput fee: money PLN\n\nversion v\n  cite card\n  governs always\n  known 2026-01-01\n  fee = 12.50\n\nask a: fee\n"
        );
        let scenario = "scenario s over m\n  tally 2\n  from 2026-06-01\n\nask b: fee in s\n";
        let alone = given(&[("s.tally", Some(scenario.as_bytes()))]);
        assert_eq!(
            line(run(alone, &every())),
            "refused: LOAD-HEADER 0 files have a model header; exactly one does",
            "a scenario alone: its base has no model"
        );
        let both = || {
            given(&[
                ("m.tally", Some(base.as_bytes())),
                ("s.tally", Some(scenario.as_bytes())),
            ])
        };
        assert_eq!(
            lines(both(), &probe()),
            ["refused: REQ-ASK no ask zz"],
            "after both worlds load"
        );
        let base_ask = Request {
            asks: Asks::Named(vec!["a".to_owned()]),
            pin: Pin::Canonical,
            days: crate::Days::Events,
        };
        assert_eq!(
            lines(both(), &base_ask),
            ["a: 12.50 PLN", "  cut 2026-01-01"],
            "a base ask in the base"
        );
        assert_eq!(
            lines(both(), &every()),
            [
                "a: 12.50 PLN",
                "  cut 2026-01-01",
                "b: 12.50 PLN",
                "  world: s",
                "  cut 2026-01-01"
            ],
            "b is asked in s's world, on s's reality"
        );
        let under = "scenario s over m\n  tally 2\n  from 2026-06-01\n\nask b: fee in m\n";
        let hidden = given(&[
            ("m.tally", Some(base.as_bytes())),
            ("s.tally", Some(under.as_bytes())),
        ]);
        assert_eq!(
            lines(hidden, &every()),
            [
                "a: 12.50 PLN",
                "  cut 2026-01-01",
                "b: 12.50 PLN",
                "  world: m",
                "  cut 2026-01-01"
            ],
            "s's ask in the model's world, on s's reality, s's layer hidden (refcore's bytes)"
        );
        let fromless = "scenario s over m\n  tally 2\n\nask b: fee in s\n";
        let refused = given(&[
            ("m.tally", Some(base.as_bytes())),
            ("s.tally", Some(fromless.as_bytes())),
        ]);
        assert_eq!(
            lines(refused, &every()),
            [
                "scenario s: refused: LOAD-HEADER missing from date (s.tally:1)",
                "a: 12.50 PLN",
                "  cut 2026-01-01"
            ],
            "s refused alone, its ask with it, the base answered"
        );
    }
}
