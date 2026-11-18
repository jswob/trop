//! Milestone M3's vectors (issue #226; `MILESTONE-M3.md`), written by `m3/vectors.py`.
//!
//! `check` (`OPERATIONS.md` §2.3; rows 2 and 3): `tally check <path>…` on M0's 315 text sets
//! and on the fixtures of `m3/check/`, one per line kind the sets don't reach, stored in
//! `m3/check.vectors`. Each vector's exit code and whole output were predicted from refcore's
//! `run` (the load probe, a probe ask refused alone that prints load's lines and warnings, and
//! the asks refused alone: pick P8), until refcore's own `check` confirms them byte for byte
//! (chunk C4; the vector's `source` then reads `check`). A usage error's words are the
//! engine's (`INTERFACE.md` §1): its vectors (`source` `rule`) hold the exit code alone.
//!
//! - [`CHECK`]: the stored vectors, read by [`check_vectors`];
//! - [`texts`]: M0's embedded texts and the fixtures' ([`TEXTS`]);
//! - [`check_failure`]: how the engine's `check` of a vector fails, if it does;
//! - [`check_group_failures`]: every failure of one group, for `milestone/m3_check.rs`.

use crate::corpus::{Run, groups, netstrings, text};
use crate::engine::run_args;
use crate::held::is_held;
use crate::kernel::difference;
use crate::m0::{self, TableFile};
use crate::m3_texts::TEXTS;
use std::collections::{BTreeMap, BTreeSet};

/// `m3/check.vectors`: eight netstrings per vector (`m3/vectors.py`).
pub const CHECK: &[u8] = include_bytes!("../../m3/check.vectors");

/// The exit code of a usage error (`INTERFACE.md` §1, pick P3).
const USAGE: u8 = 2;

/// One `check` vector: the command line, its file table and the expected run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckVector {
    /// The vector's id: a set's (`I:X:…`) or a fixture's (`fixture/<kind>/<name>`).
    pub id: String,
    /// The test that runs it: an M0 corpus group, or a fixture's kind.
    pub group: String,
    /// Where its bytes come from: `run` (predicted), `rule` (a usage error) or `check`.
    pub source: String,
    /// The words after the program name, `check` first.
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
            args: text(args)?.split(' ').map(str::to_owned).collect(),
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

/// Every vector of `group` whose place in it is `part` modulo `parts` that the engine fails,
/// each with why (a vector `held.tsv` holds for `m3_check` left out), and how many vectors
/// that part holds: a slow group runs as several tests, together every vector (pick P70).
///
/// # Errors
///
/// A store that doesn't read, or a part with no vector.
pub fn check_group_failures(
    group: &str,
    (part, parts): (usize, usize),
) -> Result<(Vec<String>, usize), String> {
    let vs = check_vectors(CHECK)?;
    let texts = texts();
    let mine: Vec<&CheckVector> = vs
        .iter()
        .filter(|v| v.group == group)
        .enumerate()
        .filter(|(i, _)| i.checked_rem(parts) == Some(part))
        .map(|(_, v)| v)
        .collect();
    if mine.is_empty() {
        return Err(format!("m3/check.vectors holds no vector of {group}"));
    }
    let failed = mine
        .iter()
        .filter(|v| !is_held("m3_check", &v.id))
        .filter_map(|v| check_failure(v, &texts))
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
    use super::{CHECK, check_vectors, texts};

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
}
