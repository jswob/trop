//! Milestone M3's vectors (issue #226; `MILESTONE-M3.md`), written by `m3/vectors.py`.
//!
//! `check` (`OPERATIONS.md` §2.3; rows 2 and 3): `tally check <path>…` on M0's 315 text sets
//! and on the fixtures of `m3/check/`, one per line kind the sets don't reach, stored in
//! `m3/check.vectors`. Each vector's exit code and whole output were predicted from refcore's
//! `run` (the load probe, a probe ask refused alone that prints load's lines and warnings, and
//! the asks refused alone: pick P8), until refcore's own `check` confirmed them byte for byte
//! (chunk C4, 2026-11-19: each `source` now reads `check`). A usage error's words are the
//! engine's (`INTERFACE.md` §1): its vectors (`source` `rule`) hold the exit code alone.
//!
//! `run` (rows 5 and 6): the seven changes' realities before and after (`m3/changes/stages.tsv`,
//! group `changes`, `milestone/m3_changes.rs`) and the run function's vectors (`m3/run/`, group
//! `run`, `milestone/m3_run.rs`), refcore's run of each, stored in `m3/runs.vectors` in the same
//! eight netstrings, the argument words as given.
//!
//! - [`CHECK`] and [`RUNS`]: the stored vectors, read by [`check_vectors`];
//! - [`texts`]: M0's embedded texts and the fixtures' ([`TEXTS`]);
//! - [`check_failure`]: how the engine's run of a vector fails, if it does;
//! - [`check_group_failures`], [`group_failures`]: every failure of one group of a store.

use crate::corpus::{Run, groups, netstrings, text};
use crate::engine::run_args;
use crate::held::is_held;
use crate::kernel::difference;
use crate::m0::{self, TableFile};
use crate::m3_texts::TEXTS;
use std::collections::{BTreeMap, BTreeSet};
use tally_cli::tally::{
    Asks, Files, Given, How, NotImplemented, Pin, Request, check, run_with_how,
};

/// `m3/check.vectors`: eight netstrings per vector (`m3/vectors.py`).
pub const CHECK: &[u8] = include_bytes!("../../m3/check.vectors");

/// `m3/runs.vectors`: the `run` vectors, in the same eight netstrings (`m3/vectors.py`).
pub const RUNS: &[u8] = include_bytes!("../../m3/runs.vectors");

/// The exit code of a usage error (`INTERFACE.md` §1, pick P3).
const USAGE: u8 = 2;

/// One `check` vector: the command line, its file table and the expected run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckVector {
    /// The vector's id: a set's (`I:X:…`) or a fixture's (`fixture/<kind>/<name>`).
    pub id: String,
    /// The test that runs it: an M0 corpus group, or a fixture's kind.
    pub group: String,
    /// Where its bytes come from: `check` (refcore's `check`, since chunk C4), `run` (refcore's
    /// run: a `run` vector, or a `check` vector's prediction before C4) or `rule` (a usage error).
    pub source: String,
    /// The words after the program name: `check` or `run` first, or a usage error's (none at all).
    pub args: Vec<String>,
    /// The files the run may read.
    pub table: Vec<TableFile>,
    /// The expected exit code and whole output.
    pub run: Run,
}

/// The vectors of `bytes`, in `vectors.py`'s order.
///
/// # Errors
///
/// A store that isn't eight netstrings per vector, or a field that doesn't read.
pub fn check_vectors(bytes: &[u8]) -> Result<Vec<CheckVector>, String> {
    let mut out = Vec::new();
    for eight in groups(&netstrings(bytes)?, 8)? {
        let [id, group, source, _cwd, args, table, code, whole] = eight.as_slice() else {
            return Err("a check vector is eight netstrings".to_owned());
        };
        let id = text(id)?;
        let table = text(table)?
            .lines()
            .map(|line| match line.split_once('\t') {
                Some((key, path)) => Ok(TableFile {
                    key: key.to_owned(),
                    path: path.to_owned(),
                }),
                None => Err(format!("{id}: a table line without its tab")),
            })
            .collect::<Result<Vec<_>, String>>()?;
        out.push(CheckVector {
            group: text(group)?,
            source: text(source)?,
            args: text(args)?
                .split(' ')
                .filter(|w| !w.is_empty())
                .map(str::to_owned)
                .collect(),
            table,
            run: Run {
                out: text(whole)?,
                code: text(code)?
                    .parse()
                    .map_err(|_| format!("{id}: an exit code"))?,
            },
            id,
        });
    }
    Ok(out)
}

/// Every text a vector may name: M0's ([`m0::texts`]) and the fixtures' ([`TEXTS`]).
#[must_use]
pub fn texts() -> BTreeMap<&'static str, &'static str> {
    let mut all = m0::texts();
    all.extend(TEXTS.iter().copied());
    all
}

/// How the engine's `check` of `v` fails, or `None`: it must exit with the vector's code and
/// print its whole output; a usage error (`source` `rule`) must exit 2, whatever it prints.
#[must_use]
pub fn check_failure(v: &CheckVector, texts: &BTreeMap<&str, &str>) -> Option<String> {
    let files = match m0::table(&v.id, &v.table, texts) {
        Ok(t) => t,
        Err(e) => return Some(e),
    };
    let got = run_args(&v.args, &files);
    if v.source == "rule" {
        return (got.code != USAGE).then(|| {
            format!(
                "{}: exit {} (a usage error is {USAGE}), output {:?}",
                v.id, got.code, got.out
            )
        });
    }
    difference(&v.run, &got).map(|d| format!("{}: {d}", v.id))
}

/// Every vector of `group` in `store` whose place in the group is `part` modulo `parts` that
/// the engine fails, each with why (a vector `held.tsv` holds for `target` left out), and how
/// many vectors that part holds: a slow group runs as several tests, together every vector
/// (pick P70).
///
/// # Errors
///
/// A store that doesn't read, or a part with no vector.
pub fn group_failures(
    (store, target): (&[u8], &str),
    group: &str,
    (part, parts): (usize, usize),
) -> Result<(Vec<String>, usize), String> {
    let vs = check_vectors(store)?;
    let texts = texts();
    let mine: Vec<&CheckVector> = vs
        .iter()
        .filter(|v| v.group == group)
        .enumerate()
        .filter(|(i, _)| i.checked_rem(parts) == Some(part))
        .map(|(_, v)| v)
        .collect();
    if mine.is_empty() {
        return Err(format!("{target}'s store holds no vector of {group}"));
    }
    let failed = mine
        .iter()
        .filter(|v| !is_held(target, &v.id))
        .filter_map(|v| check_failure(v, &texts))
        .collect();
    Ok((failed, mine.len()))
}

/// [`group_failures`] of `m3/check.vectors`, for `milestone/m3_check.rs`.
///
/// # Errors
///
/// A store that doesn't read, or a part with no vector.
pub fn check_group_failures(
    group: &str,
    part: (usize, usize),
) -> Result<(Vec<String>, usize), String> {
    group_failures((CHECK, "m3_check"), group, part)
}

/// The exit code the CLI gives a command past the part of the engine built so far.
const NOT_IMPLEMENTED: u8 = 69;

/// A `run` or `check` command line's words after the command, as INTERFACE §1 reads them: its
/// paths (the words before `--ask` that aren't `--pin=`), its asks (every ask when none is
/// named) and its pin.
fn request_words(rest: &[String]) -> (Vec<String>, Asks, Pin) {
    let (mut paths, mut asks, mut pin, mut after_ask) =
        (Vec::new(), Vec::new(), Pin::Canonical, false);
    for word in rest {
        if word == "--ask" {
            after_ask = true;
        } else if let Some(form) = word.strip_prefix("--pin=") {
            pin = if form == "bytes" {
                Pin::Bytes
            } else {
                Pin::Canonical
            };
        } else if after_ask {
            asks.push(word.clone());
        } else {
            paths.push(word.clone());
        }
    }
    let asks = if asks.is_empty() {
        Asks::Every
    } else {
        Asks::Named(asks)
    };
    (paths, asks, pin)
}

/// The library's files for `paths` against the file table `table`, INTERFACE §4's reading,
/// written here apart from the CLI's: a key is that file; a directory is every key under it
/// that ends `.tally`, in path order; anything else names nothing (`LOAD-PATH`).
fn library_files(paths: &[String], table: BTreeMap<String, Vec<u8>>, pin: Pin) -> Files {
    let mut given = Vec::new();
    for path in paths {
        if let Some(bytes) = table.get(path) {
            given.push(Given {
                path: path.clone(),
                bytes: Some(bytes.clone()),
            });
            continue;
        }
        let dir = if path.ends_with('/') {
            path.clone()
        } else {
            format!("{path}/")
        };
        let under: Vec<Given> = table
            .iter()
            .filter(|(key, _)| {
                key.starts_with(dir.as_str())
                    && key.rsplit_once('.').is_some_and(|(_, ext)| ext == "tally")
            })
            .map(|(key, bytes)| Given {
                path: key.clone(),
                bytes: Some(bytes.clone()),
            })
            .collect();
        if under.is_empty() {
            given.push(Given {
                path: path.clone(),
                bytes: None,
            });
        }
        given.extend(under);
    }
    Files { given, table, pin }
}

/// How the library's entry (E §1: `run_with_how` with every fence's form, which `run` is, or
/// `check`) fails to print what the CLI prints for `v`'s command line, or `None`: the same
/// lines, and the exit code the CLI gives them (`check`: 0 only after `ok`; `run`: 1 for a
/// refused run), or both stopping not implemented. A usage error (`rule`) reaches no entry.
#[must_use]
pub fn library_failure(v: &CheckVector, texts: &BTreeMap<&str, &str>) -> Option<String> {
    let (Some((command, rest)), false) = (v.args.split_first(), v.source == "rule") else {
        return None;
    };
    let mut table = BTreeMap::new();
    for f in &v.table {
        let Some(text) = texts.get(f.path.as_str()) else {
            return Some(format!("{}: {} has no embedded text", v.id, f.path));
        };
        table.insert(f.key.clone(), text.as_bytes().to_vec());
    }
    let packed = match m0::table(&v.id, &v.table, texts) {
        Ok(t) => t,
        Err(e) => return Some(e),
    };
    let cli = run_args(&v.args, &packed);
    let (paths, asks, pin) = request_words(rest);
    let files = library_files(&paths, table, pin);
    let answer = match command.as_str() {
        "run" => run_with_how(files, &Request::new(asks), How::FENCES),
        "check" => check(files),
        _ => return Some(format!("{}: neither run nor check", v.id)),
    };
    let lib = match answer {
        Ok(a) => {
            let mut out = String::new();
            for line in a.lines() {
                out.push_str(line);
                out.push('\n');
            }
            let refused = if command == "check" {
                a.lines().first().map(String::as_str) != Some("ok")
            } else {
                a.is_refused()
            };
            Run {
                out,
                code: u8::from(refused),
            }
        }
        Err(NotImplemented) => Run {
            out: cli.out.clone(),
            code: NOT_IMPLEMENTED,
        },
    };
    difference(&cli, &lib).map(|d| format!("{}: the CLI against the library: {d}", v.id))
}

/// Every vector of `store` whose group is one of `groups` where the library's entry fails to
/// print the CLI's lines ([`library_failure`]), and how many it ran.
///
/// # Errors
///
/// A store that doesn't read.
pub fn library_failures(store: &[u8], groups: &[&str]) -> Result<(Vec<String>, usize), String> {
    let vs = check_vectors(store)?;
    let texts = texts();
    let mine: Vec<&CheckVector> = vs
        .iter()
        .filter(|v| groups.contains(&v.group.as_str()))
        .collect();
    let failed = mine
        .iter()
        .filter_map(|v| library_failure(v, &texts))
        .collect();
    Ok((failed, mine.len()))
}

/// The groups of `m3/check.vectors`.
///
/// # Errors
///
/// A store that doesn't read.
pub fn check_groups() -> Result<BTreeSet<String>, String> {
    Ok(check_vectors(CHECK)?.into_iter().map(|v| v.group).collect())
}

#[cfg(test)]
mod tests {
    use super::{CHECK, RUNS, check_vectors, texts};
    use std::collections::BTreeSet;

    #[test]
    fn the_store_reads_and_every_text_is_embedded() {
        let vs = check_vectors(CHECK).expect("m3/check.vectors reads");
        assert!(vs.len() > 315, "the 315 sets and the fixtures");
        let texts = texts();
        for v in &vs {
            assert_eq!(
                v.args.first().map(String::as_str),
                Some("check"),
                "{}",
                v.id
            );
            for f in &v.table {
                assert!(texts.contains_key(f.path.as_str()), "{}: {}", v.id, f.path);
            }
        }
    }

    /// The runs store: ids once, groups `changes` and `run`, `run` first but for a usage error,
    /// every text embedded.
    #[test]
    fn the_runs_store_reads_and_every_text_is_embedded() {
        let vs = check_vectors(RUNS).expect("m3/runs.vectors reads");
        let texts = texts();
        let mut ids = BTreeSet::new();
        for v in &vs {
            assert!(ids.insert(v.id.as_str()), "{}: twice", v.id);
            assert!(
                ["changes", "run", "operations", "last", "fixes"].contains(&v.group.as_str()),
                "{}",
                v.id
            );
            assert!(
                v.source == "rule" || v.args.first().map(String::as_str) == Some("run"),
                "{}",
                v.id
            );
            for f in &v.table {
                assert!(texts.contains_key(f.path.as_str()), "{}: {}", v.id, f.path);
            }
        }
    }
}
