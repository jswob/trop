//! T's fixtures for the catalogue's codes that no check prints (DONE gate 5), read from
//! `codes/fixtures.tsv`, which `codes/codes.py` writes from refcore's runs: each a command
//! line, run from a folder under `codes/`, and refcore's output and exit code. First
//! `LOAD-PATH` (tally 2.1.1, L §8.3). Milestone M0's target `m0_codes` runs them on the engine.
//!
//! A fixture's file table holds every file under `codes/` that its command line names and
//! that exists (`codes.py` runs refcore on the disk, where the others don't): a path the
//! table doesn't hold is a path that doesn't exist (`INTERFACE.md` §4).

use crate::corpus::pack;

/// `codes/fixtures.tsv`.
pub const FIXTURES: &str = include_str!("../../codes/fixtures.tsv");

/// The fixtures' files, by their path from the fixtures' `cwd` (`engine/tests/codes`).
const FILES: [(&str, &str); 2] = [
    (
        "LOAD-PATH/a.tally",
        include_str!("../../codes/LOAD-PATH/a.tally"),
    ),
    (
        "LOAD-PATH/m.tally",
        include_str!("../../codes/LOAD-PATH/m.tally"),
    ),
];

/// One row of `codes/fixtures.tsv`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fixture {
    /// Its id (`LOAD-PATH/alone`).
    pub id: String,
    /// The code its output prints.
    pub code: String,
    /// The command line's words after the program name.
    pub args: Vec<String>,
    /// The file table: every fixture text under `codes/` that `args` names.
    pub files: Vec<u8>,
    /// refcore's exit code.
    pub exit_code: u8,
    /// refcore's output, standard output then standard error.
    pub out: String,
}

/// The fixtures of `tsv`, in its order.
pub fn fixtures(tsv: &str) -> Result<Vec<Fixture>, String> {
    let mut lines = tsv.lines();
    if lines.next() != Some("id\tcode\tcwd\targs\trc\tout\trule") {
        return Err(
            "codes/fixtures.tsv's columns are id, code, cwd, args, rc, out, rule".to_owned(),
        );
    }
    let mut res = Vec::new();
    for line in lines {
        let cols: Vec<&str> = line.split('\t').collect();
        let [id, code, cwd, args, rc, out, _rule] = cols.as_slice() else {
            return Err(format!("a fixture is 7 columns: {line}"));
        };
        if *cwd != "engine/tests/codes" {
            return Err(format!("fixture {id}: cwd {cwd}, not engine/tests/codes"));
        }
        let args: Vec<String> = args.split(' ').map(str::to_owned).collect();
        let named: Vec<(&str, &str)> = FILES
            .iter()
            .filter(|(path, _)| args.iter().any(|a| a == path))
            .copied()
            .collect();
        res.push(Fixture {
            id: (*id).to_owned(),
            code: (*code).to_owned(),
            files: pack(&named),
            args,
            exit_code: rc
                .parse()
                .map_err(|_| format!("fixture {id}: exit code {rc}"))?,
            out: out.replace("\\n", "\n"),
        });
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::{FIXTURES, fixtures};
    use crate::corpus::paths;

    #[test]
    fn every_fixture_reads_and_refuses_with_its_code() {
        let fs = fixtures(FIXTURES).expect("codes/fixtures.tsv reads");
        assert!(
            fs.iter().filter(|f| f.code == "LOAD-PATH").count() >= 2,
            "LOAD-PATH has its fixtures"
        );
        for f in &fs {
            assert!(
                f.out.starts_with(&format!("refused: {} ", f.code)) && f.out.ends_with('\n'),
                "{}: refcore's output prints its code: {}",
                f.id,
                f.out
            );
            assert_eq!(
                f.exit_code, 1,
                "{}: a refusal exits 1 (INTERFACE.md section 3)",
                f.id
            );
        }
    }

    /// The table holds the files the command line names that exist, and never the missing
    /// path the fixture is about.
    #[test]
    fn a_fixtures_table_leaves_out_the_missing_path() {
        let fs = fixtures(FIXTURES).expect("codes/fixtures.tsv reads");
        let paths = |id: &str| -> Vec<String> {
            let f = fs.iter().find(|f| f.id == id).expect("the fixture");
            paths(&f.files).expect("the table reads")
        };
        assert!(paths("LOAD-PATH/alone").is_empty(), "nope.tally is no file");
        assert_eq!(
            paths("LOAD-PATH/before-parse"),
            vec!["LOAD-PATH/a.tally", "LOAD-PATH/m.tally"],
            "z.tally is no file; the others are"
        );
        assert_eq!(
            paths("LOAD-PATH/control"),
            vec!["LOAD-PATH/a.tally", "LOAD-PATH/m.tally"],
            "the control names both files"
        );
    }
}
