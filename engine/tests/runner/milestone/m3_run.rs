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
    use tally_runner::m3::{RUNS, group_failures};

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
}
