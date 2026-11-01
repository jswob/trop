//! Milestone M0's code fixtures on the engine (DONE gate 5; `BUILD-ORDER.md` § M0: each
//! `LOAD-*` code has its fixture): each fixture of `codes/fixtures.tsv` run in process, its
//! output and exit code refcore's, byte for byte. Red on the stub (exit 69, "not
//! implemented"), so this target has `test = false`; it gates once M0 is tagged
//! (`engine/tools/milestones.toml`).

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally_runner::codes::{FIXTURES, fixtures};
    use tally_runner::engine::run_args;

    /// Runs every fixture of `code` on the engine against refcore's output and exit code.
    fn refuses_as_refcore(code: &str) {
        let fs = fixtures(FIXTURES).expect("codes/fixtures.tsv reads");
        let mine: Vec<_> = fs.iter().filter(|f| f.code == code).collect();
        assert!(!mine.is_empty(), "{code} has fixtures");
        let failed: Vec<String> = mine
            .iter()
            .filter_map(|f| {
                let run = run_args(&f.args, &f.files);
                (run.out != f.out || run.code != f.exit_code).then(|| {
                    format!(
                        "{}: exit {} printing {:?}; refcore: exit {} printing {:?}",
                        f.id, run.code, run.out, f.exit_code, f.out
                    )
                })
            })
            .collect();
        assert!(
            failed.is_empty(),
            "{} of {} fixtures fail:\n{}",
            failed.len(),
            mine.len(),
            failed.join("\n")
        );
    }

    #[test]
    fn m0_load_path_a_path_that_names_no_file() {
        refuses_as_refcore("LOAD-PATH");
    }

    #[test]
    fn m0_load_parse_the_load_path_fixtures_control() {
        refuses_as_refcore("LOAD-PARSE");
    }
}
