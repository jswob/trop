//! Milestone M0's row 3 (`BUILD-ORDER.md` § M0, Green: "`tools/loadcheck.py` (`--selftest`
//! first) agrees with the loader over the 79 models, the household and the changes";
//! `MILESTONE-M0.md` row 3): the engine's load probe (`run <files> --ask zz_m0_probe`) on each
//! of the spec's 79 models, its household and X's 7 changes and 3 variants, held to refcore
//! 2.1.3's whole output (`m0/loadcheck.refcore`). `m0/loadcheck.py` checked, before it wrote
//! them, that the spec's scan `engine/spec/tools/loadcheck.py` agrees with refcore on each
//! (both clean, or a refusal of a code the scan doesn't check: MP-10's `LOAD-BAND`), and on
//! three fixtures of codes it does check; so an engine that prints refcore's output agrees with
//! the scan. Red until M0's loader is built, so this target has `test = false`; it gates once
//! M0 is tagged (`engine/tools/milestones.toml`).

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally_runner::m0::loadcheck_group_failures;

    /// Holds the load probe of every reality of `group` to refcore's.
    fn loads_as_refcore(group: &str) {
        let (failed, of) = loadcheck_group_failures(group).expect("m0/loadcheck.refcore reads");
        assert!(
            failed.is_empty(),
            "{} of {of} realities of {group} fail:\n{}",
            failed.len(),
            failed.join("\n")
        );
    }

    /// The spec's 79 models (`engine/spec/models/`), each a reality of one file; LH-15 and
    /// LH-16 at full size; MP-10 refused (`LOAD-BAND`).
    #[test]
    fn m0_loadcheck_models() {
        loads_as_refcore("models");
    }

    /// The household (`engine/spec/amendments/household/`), a folder's 13 files.
    #[test]
    fn m0_loadcheck_household() {
        loads_as_refcore("household");
    }

    /// X's changes `c1`..`c7` and variants `v3`..`v5`
    /// (`integration/amendments/real11/`), each the household with the change made.
    #[test]
    fn m0_loadcheck_changes() {
        loads_as_refcore("changes");
    }
}
