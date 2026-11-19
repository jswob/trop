//! Milestone M3's run function through the CLI (`ENGINE.md` §1; `OPERATIONS.md` §2.1;
//! `MILESTONE-M3.md` row 5): an id asked twice, the request's order, `--ask` with no id, two
//! knowledge cuts in one run, a file in the table that no argument names, a file under a
//! directory that isn't `.tally`, a member's ask beside the joint model's, an ask in a refused
//! scenario's world, and two usage errors (exit 2, the words the engine's). Each held to
//! refcore's run (`m3/runs.vectors`, group `run`), byte for byte. `test = false` until M3 is
//! tagged (`engine/tools/milestones.toml`).

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally_runner::m3::{CHECK, RUNS, group_failures, library_failures};

    /// Holds every vector of `group` to refcore's run.
    fn runs_as_stored(group: &str) {
        let (failed, of) =
            group_failures((RUNS, "m3_run"), group, (0, 1)).expect("m3/runs.vectors reads");
        assert!(
            failed.is_empty(),
            "{} of {of} run vectors of {group} fail:\n{}",
            failed.len(),
            failed.join("\n")
        );
    }

    #[test]
    fn m3_run_requests() {
        runs_as_stored("run");
    }

    /// OPERATIONS §3 and §7's runs on the small model `mini`, among them SCN-1's and §7's
    /// printed outputs, which refcore prints byte for byte.
    #[test]
    fn m3_run_operations() {
        runs_as_stored("operations");
    }

    /// The run function is the library's one entry and the CLI's (E §1; M3 step b): every
    /// `run` vector of M3 and every `check` fixture, through `run_with_how` with every fence's
    /// form (what `run` is) or `check` on the files INTERFACE §4 resolves, prints the CLI's
    /// lines and exit code, and stops where the CLI stops.
    #[test]
    fn m3_run_the_library_prints_what_the_cli_prints() {
        let (mut failed, runs) =
            library_failures(RUNS, &["changes", "run", "operations", "last"]).expect("runs");
        let (more, checks) = library_failures(
            CHECK,
            &[
                "ok",
                "base",
                "scenario",
                "alone",
                "sections",
                "notclean",
                "paths",
                "changes",
                "operations",
                "reviews",
            ],
        )
        .expect("checks");
        failed.extend(more);
        assert!(runs > 30 && checks > 60, "{runs} runs, {checks} checks");
        assert!(
            failed.is_empty(),
            "{} of {} vectors differ:\n{}",
            failed.len(),
            runs.checked_add(checks).expect("a count"),
            failed.join("\n")
        );
    }
}
