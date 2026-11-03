//! Milestone M0's code fixtures on the engine (DONE gate 5; `BUILD-ORDER.md` § M0: each
//! `LOAD-*` code has its fixture): each fixture of `codes/fixtures.tsv` run in process, its
//! output and exit code refcore's, byte for byte. Red on the stub (exit 69, "not
//! implemented"), so this target has `test = false`; it gates once M0 is tagged
//! (`engine/tools/milestones.toml`).

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally_runner::codes::{FIXTURES, Fixture, fixtures};
    use tally_runner::engine::run_args;

    /// Runs every fixture of `code` on the engine against refcore's output and exit code.
    fn refuses_as_refcore(code: &str) {
        let fs = fixtures(FIXTURES).expect("codes/fixtures.tsv reads");
        let mine: Vec<&Fixture> = fs.iter().filter(|f| f.code == code).collect();
        assert!(!mine.is_empty(), "{code} has fixtures");
        all_as_refcore(&mine);
    }

    /// Runs every fixture under the folder `folder` of `codes/` (its id `<folder>/…`), each
    /// refusal with its control, on the engine against refcore's output and exit code.
    fn folder_as_refcore(folder: &str) {
        let fs = fixtures(FIXTURES).expect("codes/fixtures.tsv reads");
        let prefix = format!("{folder}/");
        let mine: Vec<&Fixture> = fs.iter().filter(|f| f.id.starts_with(&prefix)).collect();
        assert!(!mine.is_empty(), "{folder} has fixtures");
        all_as_refcore(&mine);
    }

    /// Holds each fixture of `mine` to refcore's run, collecting every failure.
    fn all_as_refcore(mine: &[&Fixture]) {
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

    /// E §3.1 The order of the checks (tally 2.1.2, issue #23): the passes run in order (a
    /// second declaration refuses before a def's key, whatever the lines), and the items' own
    /// checks take moves by full path and line, not by id; each beside its control.
    #[test]
    fn m0_load_order_the_checks_past_parsing() {
        folder_as_refcore("order");
    }

    /// L §2.3 `LOAD-DEPTH` (tally 2.1.2, issue #29): a `ties by` term counts one level, so 9
    /// argmins chained by `ties by` are refused at the parse, before the declarations pass;
    /// its control, 8 deep, parses and is refused by that pass.
    #[test]
    fn m0_load_depth_a_ties_by_term_counts_one_level() {
        folder_as_refcore("LOAD-DEPTH");
    }
}
