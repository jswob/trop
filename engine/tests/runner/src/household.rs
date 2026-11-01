//! Household X's base as milestone S runs it (`BUILD-ORDER.md` § S), and the hand-checked
//! vectors of `spec/smoke/HH4-a.md` and `HH4-b.md` restated on it in tally 2.1.1's output
//! form (DONE gate 3; `MILESTONE-S.md` row 4). The vectors are `hh4/vectors.tsv`, which
//! `hh4/hh4.py` writes from refcore's runs and checks, figure by figure, against HH4's lines.
//!
//! - [`s_checks`]: milestone S's three checks on X's base, each as its command line and its
//!   file table (`INTERFACE.md` §4), the paths as `run.py` gives them from the check's `cwd`;
//! - [`vectors`]: the vectors, read from [`VECTORS`];
//! - [`broken`]: the vectors a run's output breaks;
//! - [`REFCORE`]: refcore's output for each check (`hh4/refcore/`), on which this crate's own
//!   tests prove the vectors, so the milestone target can't pass or fail vacuously.

use crate::corpus::pack;
use std::collections::BTreeMap;

/// The `cwd` of X's checks in `checks.tsv`; the paths of the file tables are relative to it.
pub const CWD: &str = "engine/tests/integration/amendments";

/// X's base, `cut11/b_r1/`, its files in `checks.tsv`'s order.
const B_R1: [(&str, &str); 13] = [
    (
        "cut11/b_r1/asks/cut.tally",
        include_str!("../../integration/amendments/cut11/b_r1/asks/cut.tally"),
    ),
    (
        "cut11/b_r1/facts/2026.tally",
        include_str!("../../integration/amendments/cut11/b_r1/facts/2026.tally"),
    ),
    (
        "cut11/b_r1/law/pl/benefits.tally",
        include_str!("../../integration/amendments/cut11/b_r1/law/pl/benefits.tally"),
    ),
    (
        "cut11/b_r1/law/pl/business.tally",
        include_str!("../../integration/amendments/cut11/b_r1/law/pl/business.tally"),
    ),
    (
        "cut11/b_r1/law/pl/common.tally",
        include_str!("../../integration/amendments/cut11/b_r1/law/pl/common.tally"),
    ),
    (
        "cut11/b_r1/law/pl/employment.tally",
        include_str!("../../integration/amendments/cut11/b_r1/law/pl/employment.tally"),
    ),
    (
        "cut11/b_r1/law/pl/wrappers.tally",
        include_str!("../../integration/amendments/cut11/b_r1/law/pl/wrappers.tally"),
    ),
    (
        "cut11/b_r1/loans.tally",
        include_str!("../../integration/amendments/cut11/b_r1/loans.tally"),
    ),
    (
        "cut11/b_r1/pays.tally",
        include_str!("../../integration/amendments/cut11/b_r1/pays.tally"),
    ),
    (
        "cut11/b_r1/people.tally",
        include_str!("../../integration/amendments/cut11/b_r1/people.tally"),
    ),
    (
        "cut11/b_r1/places.tally",
        include_str!("../../integration/amendments/cut11/b_r1/places.tally"),
    ),
    (
        "cut11/b_r1/reality.tally",
        include_str!("../../integration/amendments/cut11/b_r1/reality.tally"),
    ),
    (
        "cut11/b_r1/records/mbank/2026-09.tally",
        include_str!("../../integration/amendments/cut11/b_r1/records/mbank/2026-09.tally"),
    ),
];

/// X's base with `replay_september` asked alone, `cut11/b_r1s/`, in `checks.tsv`'s order.
const B_R1S: [(&str, &str); 13] = [
    (
        "cut11/b_r1s/asks/cut.tally",
        include_str!("../../integration/amendments/cut11/b_r1s/asks/cut.tally"),
    ),
    (
        "cut11/b_r1s/facts/2026.tally",
        include_str!("../../integration/amendments/cut11/b_r1s/facts/2026.tally"),
    ),
    (
        "cut11/b_r1s/law/pl/benefits.tally",
        include_str!("../../integration/amendments/cut11/b_r1s/law/pl/benefits.tally"),
    ),
    (
        "cut11/b_r1s/law/pl/business.tally",
        include_str!("../../integration/amendments/cut11/b_r1s/law/pl/business.tally"),
    ),
    (
        "cut11/b_r1s/law/pl/common.tally",
        include_str!("../../integration/amendments/cut11/b_r1s/law/pl/common.tally"),
    ),
    (
        "cut11/b_r1s/law/pl/employment.tally",
        include_str!("../../integration/amendments/cut11/b_r1s/law/pl/employment.tally"),
    ),
    (
        "cut11/b_r1s/law/pl/wrappers.tally",
        include_str!("../../integration/amendments/cut11/b_r1s/law/pl/wrappers.tally"),
    ),
    (
        "cut11/b_r1s/loans.tally",
        include_str!("../../integration/amendments/cut11/b_r1s/loans.tally"),
    ),
    (
        "cut11/b_r1s/pays.tally",
        include_str!("../../integration/amendments/cut11/b_r1s/pays.tally"),
    ),
    (
        "cut11/b_r1s/people.tally",
        include_str!("../../integration/amendments/cut11/b_r1s/people.tally"),
    ),
    (
        "cut11/b_r1s/places.tally",
        include_str!("../../integration/amendments/cut11/b_r1s/places.tally"),
    ),
    (
        "cut11/b_r1s/reality.tally",
        include_str!("../../integration/amendments/cut11/b_r1s/reality.tally"),
    ),
    (
        "cut11/b_r1s/records/mbank/2026-09.tally",
        include_str!("../../integration/amendments/cut11/b_r1s/records/mbank/2026-09.tally"),
    ),
];

/// HH4's vectors restated for milestone S: `hh4/vectors.tsv`, written by `hh4/hh4.py`.
pub const VECTORS: &str = include_str!("../../hh4/vectors.tsv");

/// refcore's output for each check of [`s_checks`] (`hh4/refcore/`, `hh4.py --store`).
pub const REFCORE: [(&str, &str); 3] = [
    (
        "I:X:konto_balance@b_r1",
        include_str!("../../hh4/refcore/I_X_konto_balance@b_r1.out"),
    ),
    (
        "I:X:replay_september@b_r1",
        include_str!("../../hh4/refcore/I_X_replay_september@b_r1.out"),
    ),
    (
        "I:X:replay_september@b_r1s",
        include_str!("../../hh4/refcore/I_X_replay_september@b_r1s.out"),
    ),
];

/// One of milestone S's checks on X's base.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    /// Its id in `checks.tsv`.
    pub id: &'static str,
    /// The command line's words after the program name, as `run.py` sends them (`argv_of`).
    pub args: Vec<String>,
    /// The file table (`INTERFACE.md` §4): every file the run reads.
    pub files: Vec<u8>,
}

fn check(id: &'static str, files: &[(&str, &str)], ask: &str) -> Check {
    let mut args = vec!["run".to_owned()];
    args.extend(files.iter().map(|(path, _)| (*path).to_owned()));
    args.push("--ask".to_owned());
    args.push(ask.to_owned());
    Check {
        id,
        args,
        files: pack(files),
    }
}

/// The three checks milestone S runs on X's base (`BUILD-ORDER.md` § S, Green), in
/// `checks.tsv`'s order.
pub fn s_checks() -> Vec<Check> {
    vec![
        check("I:X:konto_balance@b_r1", &B_R1, "konto_balance"),
        check("I:X:replay_september@b_r1", &B_R1, "replay_september"),
        check("I:X:replay_september@b_r1s", &B_R1S, "replay_september"),
    ]
}

/// What a vector says of its block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The line is printed in the block, after the line of the vector before it of the same
    /// check and block.
    Line,
    /// No line of the block holds the text.
    Absent,
}

/// One row of `hh4/vectors.tsv`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vector {
    /// Its id (`K-1`, `R-2.30`, `W-1`).
    pub id: String,
    /// The checks whose output holds it.
    pub checks: Vec<String>,
    /// The ask whose block holds it, or `warning` for the run's warning lines.
    pub block: String,
    /// What it says of the block.
    pub kind: Kind,
    /// The whole line, or the text that is absent.
    pub text: String,
    /// HH4's figures on it, each with the HH4 lines it is written on.
    pub figures: String,
}

/// The vectors of `tsv`, in its order.
pub fn vectors(tsv: &str) -> Result<Vec<Vector>, String> {
    let mut lines = tsv.lines();
    if lines.next() != Some("id\tchecks\tblock\tkind\ttext\tfigures\trule") {
        return Err(
            "hh4/vectors.tsv's columns are id, checks, block, kind, text, figures, rule".to_owned(),
        );
    }
    let mut out = Vec::new();
    for line in lines {
        let cols: Vec<&str> = line.split('\t').collect();
        let [id, checks, block, kind, text, figures, _rule] = cols.as_slice() else {
            return Err(format!("a vector is 7 columns: {line}"));
        };
        let kind = match *kind {
            "line" => Kind::Line,
            "absent" => Kind::Absent,
            other => return Err(format!("vector {id}: kind {other}, not line or absent")),
        };
        out.push(Vector {
            id: (*id).to_owned(),
            checks: checks.split(' ').map(str::to_owned).collect(),
            block: (*block).to_owned(),
            kind,
            text: (*text).to_owned(),
            figures: (*figures).to_owned(),
        });
    }
    Ok(out)
}

/// The name a line of the output starts with: an ask's id, as `run.py`'s `owner` reads it.
fn owner(line: &str) -> &str {
    line.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .next()
        .unwrap_or(line)
}

/// The lines of block `name` in a run's output: an ask's (its unindented line and the
/// indented lines after it), or for `warning` every `warning:` line.
pub fn block<'a>(out: &'a str, name: &str) -> Vec<&'a str> {
    let lines = out.split('\n').filter(|l| !l.is_empty());
    if name == "warning" {
        return lines.filter(|l| l.starts_with("warning: ")).collect();
    }
    let mut res = Vec::new();
    let mut inside = false;
    for line in lines {
        if !line.starts_with(' ') {
            inside = !line.starts_with("warning: ") && owner(line) == name;
        }
        if inside {
            res.push(line);
        }
    }
    res
}

/// Every vector of check `id` that `out` breaks, each as a message; empty when `out` holds
/// them all. A line vector must be printed in its block after the line of the one before it.
pub fn broken(id: &str, out: &str, vectors: &[Vector]) -> Vec<String> {
    let mut failures = Vec::new();
    // per block, where the next line vector's search starts
    let mut next: BTreeMap<String, usize> = BTreeMap::new();
    for v in vectors.iter().filter(|v| v.checks.iter().any(|c| c == id)) {
        let lines = block(out, &v.block);
        match v.kind {
            Kind::Absent => {
                if let Some(line) = lines.iter().find(|l| l.contains(v.text.as_str())) {
                    failures.push(format!(
                        "{}: prints '{line}', which HH4 says prints nothing ({})",
                        v.id, v.figures
                    ));
                }
            }
            Kind::Line => {
                let from = *next.entry(v.block.clone()).or_insert(0);
                let found = lines
                    .iter()
                    .enumerate()
                    .skip(from)
                    .find(|(_, l)| **l == v.text)
                    .and_then(|(i, _)| i.checked_add(1));
                match found {
                    Some(after) => {
                        next.insert(v.block.clone(), after);
                    }
                    None if lines.contains(&v.text.as_str()) => failures.push(format!(
                        "{}: '{}' is printed, but not after the vector before it",
                        v.id, v.text
                    )),
                    None => failures.push(format!(
                        "{}: '{}' is not printed in {} (HH4: {})",
                        v.id, v.text, v.block, v.figures
                    )),
                }
            }
        }
    }
    failures
}

#[cfg(test)]
mod tests {
    use super::{CWD, Kind, REFCORE, VECTORS, block, broken, s_checks, vectors};
    use crate::corpus::paths;

    fn refcore(id: &str) -> &'static str {
        REFCORE
            .iter()
            .find(|(c, _)| *c == id)
            .map(|(_, out)| *out)
            .expect("refcore's output for every check")
    }

    #[test]
    fn the_vectors_read_and_name_only_s_checks() {
        let vs = vectors(VECTORS).expect("hh4/vectors.tsv reads");
        assert_eq!(vs.len(), 75, "hh4.py writes 75 vectors");
        let ids: Vec<&str> = s_checks().iter().map(|c| c.id).collect();
        for v in &vs {
            assert!(
                v.checks.iter().all(|c| ids.contains(&c.as_str())),
                "{} names a check outside S: {:?}",
                v.id,
                v.checks
            );
        }
        assert!(
            vs.iter().any(|v| v.kind == Kind::Absent),
            "HH4's lines that print nothing are vectors too"
        );
    }

    #[test]
    fn every_vector_holds_on_refcores_output() {
        let vs = vectors(VECTORS).expect("hh4/vectors.tsv reads");
        for c in s_checks() {
            let mine = vs
                .iter()
                .filter(|v| v.checks.iter().any(|x| x == c.id))
                .count();
            assert!(
                mine >= 5,
                "{}: {mine} vectors, so it can't pass vacuously",
                c.id
            );
            let failed = broken(c.id, refcore(c.id), &vs);
            assert!(failed.is_empty(), "{}: {}", c.id, failed.join("\n"));
        }
    }

    #[test]
    fn a_changed_figure_order_or_line_is_caught() {
        let vs = vectors(VECTORS).expect("hh4/vectors.tsv reads");
        let konto = refcore("I:X:konto_balance@b_r1");
        let figure = konto.replacen("6 064.17 PLN", "6 064.18 PLN", 1);
        assert!(
            broken("I:X:konto_balance@b_r1", &figure, &vs)
                .iter()
                .any(|f| f.starts_with("K-1:")),
            "a changed value breaks K-1"
        );
        let replay = refcore("I:X:replay_september@b_r1");
        let balance = "  2026-09 balance: model 9 289.04 PLN, observed 41 386.27 PLN, difference 32 097.23 PLN\n    balance konto 2026-09-30 end (anchored): booked 9 289.04 PLN, stated 41 386.27 PLN, gap 32 097.23 PLN\n";
        assert!(
            replay.contains(balance),
            "the balance group, as refcore prints it"
        );
        let moved = replay.replacen(balance, "", 1).replacen(
            "  law date",
            &format!("{balance}  law date"),
            1,
        );
        assert!(
            broken("I:X:replay_september@b_r1", &moved, &vs)
                .iter()
                .any(|f| f.contains("not after")),
            "the balance group moved last breaks the order"
        );
        let extra = replay.replacen(
            "  law date",
            "    expected but missing loan_interest[laptop, 4] on 2026-09-05: model 0.00 PLN\n  law date",
            1,
        );
        assert!(
            broken("I:X:replay_september@b_r1", &extra, &vs)
                .iter()
                .any(|f| f.starts_with("R-15:")),
            "a loan_interest line breaks R-15"
        );
        let quiet: String = replay
            .split_inclusive('\n')
            .filter(|l| !l.starts_with("warning: "))
            .collect();
        assert!(
            broken("I:X:replay_september@b_r1", &quiet, &vs)
                .iter()
                .any(|f| f.starts_with("W-1:")),
            "the warnings are vectors too"
        );
    }

    #[test]
    fn a_block_is_its_asks_lines_and_warnings_are_their_own() {
        let out = "a (d = 1): 1\n  x\nb: 2\n  y\nwarning: W one\nwarning: W two\n";
        assert_eq!(
            block(out, "a"),
            vec!["a (d = 1): 1", "  x"],
            "ask a's block"
        );
        assert_eq!(
            block(out, "b"),
            vec!["b: 2", "  y"],
            "ask b's block, warnings not in it"
        );
        assert_eq!(
            block(out, "warning"),
            vec!["warning: W one", "warning: W two"],
            "every warning"
        );
    }

    /// The command lines and file tables are `checks.tsv`'s rows: the same cwd, files in the
    /// same order and the same ask.
    #[test]
    fn the_checks_are_checks_tsvs_rows() {
        let tsv = include_str!("../../checks.tsv");
        for c in s_checks() {
            let row: Vec<&str> = tsv
                .lines()
                .find(|l| l.split('\t').next() == Some(c.id))
                .expect("the check is a row of checks.tsv")
                .split('\t')
                .collect();
            let (cwd, files, asks) = (row[2], row[3], row[4]);
            assert_eq!(cwd, CWD, "{}: cwd", c.id);
            let mut want = vec!["run".to_owned()];
            want.extend(files.split(' ').map(|f| {
                f.strip_prefix(&format!("{CWD}/"))
                    .expect("under cwd")
                    .to_owned()
            }));
            want.extend(["--ask".to_owned(), asks.to_owned()]);
            assert_eq!(c.args, want, "{}: run.py's argv_of", c.id);
            assert_eq!(
                paths(&c.files).expect("the file table reads"),
                want[1..14].to_vec(),
                "{}: every file given, once",
                c.id
            );
        }
    }
}
