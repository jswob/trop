//! Milestone S's kernel vectors (issue #41; `MILESTONE-S.md` row 5): realities run on refcore
//! by `kernel/vectors.py`, each with its command line, its file table (`INTERFACE.md` §4) and
//! refcore's whole output and exit code, stored in `kernel/vectors.refcore`. The milestone
//! target `milestone/s_kernel.rs` holds the engine's output to them byte for byte, and the
//! kernel corpus's (`properties/kernel.corpus`) to refcore's runs of it
//! (`properties/kernel.refcore`), which `s_properties` holds only to the laws.
//!
//! - [`VECTORS`]: the stored vectors, read by [`vectors`];
//! - [`difference`]: how a run differs from refcore's, by its first differing line.

use crate::corpus::{Run, groups, netstrings, text};

/// The vectors `kernel/vectors.py` writes from refcore's runs: six netstrings each, the id,
/// the group, the argument words joined by single spaces, the file table, the exit code and
/// the output.
pub const VECTORS: &[u8] = include_bytes!("../../kernel/vectors.refcore");

/// One vector: a command line, the files it reads and refcore's run of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vector {
    /// Its id in `kernel/vectors.tsv` (a check's id when it is one of `checks.tsv`).
    pub id: String,
    /// The plan group it belongs to (`tmp/t-close/t-plan.md` §2), or `checks` for a check
    /// of `checks.tsv`.
    pub group: String,
    /// The command line's words after the program name, as `run.py` sends them.
    pub args: Vec<String>,
    /// The file table, packed as `INTERFACE.md` §4 says.
    pub files: Vec<u8>,
    /// refcore's output and exit code.
    pub refcore: Run,
}

/// The vectors of `bytes`, in `kernel/vectors.tsv`'s order.
pub fn vectors(bytes: &[u8]) -> Result<Vec<Vector>, String> {
    let mut out = Vec::new();
    for six in groups(&netstrings(bytes)?, 6)? {
        let [id, group, args, files, code, printed] = six.as_slice() else {
            return Err("a vector is six netstrings".to_owned());
        };
        let code: u8 = text(code)?
            .parse()
            .map_err(|_| "a vector's exit code".to_owned())?;
        out.push(Vector {
            id: text(id)?,
            group: text(group)?,
            args: text(args)?.split(' ').map(str::to_owned).collect(),
            files: files.to_vec(),
            refcore: Run {
                out: text(printed)?,
                code,
            },
        });
    }
    Ok(out)
}

/// How `got` differs from refcore's run `expected`: the exit codes and the first line that
/// differs (as `run.py`'s `first_diff`), or the line counts; `None` when they are equal.
pub fn difference(expected: &Run, got: &Run) -> Option<String> {
    if expected == got {
        return None;
    }
    let first = expected
        .out
        .split('\n')
        .zip(got.out.split('\n'))
        .zip(1_usize..)
        .find(|((a, b), _)| a != b);
    let at = match first {
        Some(((a, b), line)) => format!("line {line}: refcore {a:?}, the engine {b:?}"),
        None => format!(
            "refcore prints {} lines, the engine {}",
            expected.out.split('\n').count(),
            got.out.split('\n').count()
        ),
    };
    Some(format!(
        "exit {} (refcore {}); {at}",
        got.code, expected.code
    ))
}

#[cfg(test)]
mod tests {
    use super::{VECTORS, difference, vectors};
    use crate::corpus::{KERNEL_CORPUS, KERNEL_REFCORE, Run, paths, records, refcore_runs};
    use std::collections::BTreeSet;

    #[test]
    fn the_vectors_read_and_their_tables_hold_every_path_they_name() {
        let vs = vectors(VECTORS).expect("kernel/vectors.refcore reads");
        assert!(!vs.is_empty(), "kernel/vectors.refcore holds vectors");
        let mut ids = BTreeSet::new();
        for v in &vs {
            assert!(ids.insert(v.id.clone()), "{}: twice", v.id);
            assert!(!v.group.is_empty(), "{}: no group", v.id);
            assert_eq!(v.args.first().map(String::as_str), Some("run"), "{}", v.id);
            let table = paths(&v.files).expect("the file table reads");
            let named = v
                .args
                .iter()
                .skip(1)
                .take_while(|a| a.as_str() != "--ask")
                .filter(|a| !a.starts_with("--"));
            for path in named {
                assert!(table.contains(path), "{}: {path} isn't in its table", v.id);
            }
            assert!(
                v.refcore.code == 0 || v.refcore.out.starts_with("refused: "),
                "{}: refcore's exit {} after an answer",
                v.id,
                v.refcore.code
            );
        }
    }

    #[test]
    fn every_corpus_record_has_refcores_run() {
        let runs = refcore_runs(KERNEL_REFCORE).expect("refcore's runs read");
        let recs = records(KERNEL_CORPUS).expect("the corpus reads");
        assert_eq!(runs.len(), recs.len(), "one run per record, keys unique");
        for r in &recs {
            assert!(
                runs.contains_key(&(r.case.clone(), r.variant.clone())),
                "{}/{}: no refcore run",
                r.case,
                r.variant
            );
        }
    }

    #[test]
    fn a_difference_names_the_exit_and_the_first_line() {
        let run = |out: &str, code: u8| Run {
            out: out.to_owned(),
            code,
        };
        assert_eq!(
            difference(&run("a\nb\n", 0), &run("a\nb\n", 0)),
            None,
            "equal runs"
        );
        assert_eq!(
            difference(&run("a\nb\n", 0), &run("a\nc\n", 0)),
            Some("exit 0 (refcore 0); line 2: refcore \"b\", the engine \"c\"".to_owned()),
            "a changed line"
        );
        assert_eq!(
            difference(&run("a\n", 0), &run("a\n", 1)),
            Some("exit 1 (refcore 0); refcore prints 2 lines, the engine 2".to_owned()),
            "the same lines with another exit"
        );
        assert_eq!(
            difference(&run("a\n", 0), &run("a\nb\n", 0)),
            Some("exit 0 (refcore 0); line 2: refcore \"\", the engine \"b\"".to_owned()),
            "a line more"
        );
    }
}
