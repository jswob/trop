//! T's fixtures for the catalogue's codes that no check prints (DONE gate 5), read from
//! `codes/fixtures.tsv`, which `codes/codes.py` writes from refcore's runs: each a command
//! line, run from a folder under `codes/`, and refcore's output and exit code. First
//! `LOAD-PATH` (tally 2.1.1, L §8.3); then the order of load's checks (tally 2.1.2, E §3.1,
//! `order/`) and `LOAD-DEPTH`'s `ties by` term (tally 2.1.2, L §2.3, `LOAD-DEPTH/`), which
//! only M0's load catalogue reaches. Milestone M0's target `m0_codes` runs them on the engine.
//!
//! A fixture's file table holds every file under `codes/` that its command line names and
//! that exists (`codes.py` runs refcore on the disk, where the others don't): a path the
//! table doesn't hold is a path that doesn't exist (`INTERFACE.md` §4).
//!
//! Then [`M0`], `codes/m0.refcore`, which `codes/m0.py` writes: a fixture for each `LOAD-*`
//! code no other output at S or M0 prints, each one edit of a family's base text (under
//! `codes/m0/`), stored with its whole file table, and each family's base, which loads clean.

use crate::corpus::{Run, groups, netstrings, pack, text};

/// `codes/fixtures.tsv`.
pub const FIXTURES: &str = include_str!("../../codes/fixtures.tsv");

/// `codes/m0.refcore`: M0's fixtures of the `LOAD-*` codes no other output prints, seven
/// netstrings per vector (`codes/m0.py`): each family's base, then each fixture.
pub const M0: &[u8] = include_bytes!("../../codes/m0.refcore");

/// One vector of `codes/m0.refcore`: a family's base text, which loads clean, or a fixture
/// written off it by one edit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct M0Fixture {
    /// `<family>/base`, or the fixture's id (`basic/reserved`).
    pub id: String,
    /// The code it prints: `REQ-ASK` for a base (the load probe after a clean load).
    pub code: String,
    /// Its family (`basic`, `joint`, …).
    pub family: String,
    /// The command line's words after the program name.
    pub args: Vec<String>,
    /// The file table (`INTERFACE.md` §4), packed: each text as the fixture's edit left it.
    pub files: Vec<u8>,
    /// refcore's run: its whole output and exit code.
    pub run: Run,
}

impl M0Fixture {
    /// Whether it is a family's base, the stated twin of the family's fixtures.
    pub fn is_base(&self) -> bool {
        self.id.ends_with("/base")
    }
}

/// The vectors of `bytes`, in `codes/m0.py`'s order.
pub fn m0_fixtures(bytes: &[u8]) -> Result<Vec<M0Fixture>, String> {
    let mut out = Vec::new();
    for seven in groups(&netstrings(bytes)?, 7)? {
        let [id, code, family, args, files, rc, whole] = seven.as_slice() else {
            return Err("an M0 code fixture is seven netstrings".to_owned());
        };
        let id = text(id)?;
        out.push(M0Fixture {
            code: text(code)?,
            family: text(family)?,
            args: text(args)?.split(' ').map(str::to_owned).collect(),
            files: files.to_vec(),
            run: Run {
                out: text(whole)?,
                code: text(rc)?
                    .parse()
                    .map_err(|_| format!("{id}: an exit code"))?,
            },
            id,
        });
    }
    Ok(out)
}

/// The fixtures' files, by their path from the fixtures' `cwd` (`engine/tests/codes`).
const FILES: [(&str, &str); 8] = [
    (
        "LOAD-PATH/a.tally",
        include_str!("../../codes/LOAD-PATH/a.tally"),
    ),
    (
        "LOAD-PATH/m.tally",
        include_str!("../../codes/LOAD-PATH/m.tally"),
    ),
    (
        "order/passes.tally",
        include_str!("../../codes/order/passes.tally"),
    ),
    (
        "order/passes_control.tally",
        include_str!("../../codes/order/passes_control.tally"),
    ),
    (
        "order/items.tally",
        include_str!("../../codes/order/items.tally"),
    ),
    (
        "order/items_control.tally",
        include_str!("../../codes/order/items_control.tally"),
    ),
    (
        "LOAD-DEPTH/ties.tally",
        include_str!("../../codes/LOAD-DEPTH/ties.tally"),
    ),
    (
        "LOAD-DEPTH/ties_control.tally",
        include_str!("../../codes/LOAD-DEPTH/ties_control.tally"),
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
    use super::{FIXTURES, M0, fixtures, m0_fixtures};
    use crate::corpus::paths;
    use std::collections::BTreeSet;

    /// Thirteen families' bases print the load probe's line alone; every fixture prints its
    /// code and no warning; 201 fixtures of 52 codes (tally 2.1.6 adds 58: #71's, #75's and
    /// #88's held rows, #89's as LOAD-LIMIT, the round's new rows, and after T's re-run P29's
    /// family `scenwhen` and P30's twin; G4's 49 of 43, G7's 64
    /// message forms, the gate's 13 of G11's forms: path counts and `lifespan` at load, 14 of G12's: a
    /// built-in through a scenario row's base prefix, a `same` side keyed by a keyed place,
    /// an adjustment's fields and a field read through `.adjusts` or an aggregate, `lifespan`
    /// of `dyn`, and 4 of G13's: `lifespan` of a bound name's field, axis combinations past
    /// 15 digits and past 64 bits).
    #[test]
    fn the_m0_fixtures_read_and_print_their_codes() {
        let vs = m0_fixtures(M0).expect("codes/m0.refcore reads");
        let mut ids = BTreeSet::new();
        for v in &vs {
            assert!(ids.insert(v.id.clone()), "{}: twice", v.id);
            assert_eq!(v.args.first().map(String::as_str), Some("run"), "{}", v.id);
            assert!(
                paths(&v.files).is_ok_and(|p| !p.is_empty()),
                "{}: a file table",
                v.id
            );
            assert!(!v.run.out.contains("warning: "), "{}: no warning", v.id);
            if v.is_base() {
                assert_eq!(
                    (v.run.out.as_str(), v.run.code),
                    ("refused: REQ-ASK no ask zz_m0_probe\n", 1),
                    "{}: a base loads clean",
                    v.id
                );
            } else {
                assert!(
                    v.run.out.contains(&format!("refused: {} ", v.code)),
                    "{}: prints {}",
                    v.id,
                    v.code
                );
                assert!(
                    vs.iter().any(|b| b.is_base() && b.family == v.family),
                    "{}: its family's base is a vector",
                    v.id
                );
            }
        }
        assert_eq!(
            vs.iter().filter(|v| v.is_base()).count(),
            13,
            "thirteen families"
        );
        let codes: BTreeSet<&str> = vs
            .iter()
            .filter(|v| !v.is_base())
            .map(|v| v.code.as_str())
            .collect();
        assert_eq!(
            (vs.len(), codes.len()),
            (214, 52),
            "201 fixtures of 52 codes"
        );
    }

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
    fn every_fixture_is_run_by_a_test_of_m0_codes() {
        let fs = fixtures(FIXTURES).expect("codes/fixtures.tsv reads");
        for f in &fs {
            assert!(
                ["LOAD-PATH", "LOAD-PARSE"].contains(&f.code.as_str())
                    || f.id.starts_with("order/")
                    || f.id.starts_with("LOAD-DEPTH/"),
                "{}: m0_codes runs fixtures by the codes LOAD-PATH and LOAD-PARSE and by the \
                 folders order/ and LOAD-DEPTH/",
                f.id
            );
        }
        assert_eq!(fs.len(), 9, "codes.py writes 9 fixtures");
    }

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
