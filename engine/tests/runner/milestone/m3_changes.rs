//! Milestone M3's seven changes through `OPERATIONS.md` §6.1 (`BUILD-ORDER.md` § M3; #224's
//! fifth item; `MILESTONE-M3.md` row 6): household X before and after each change
//! (`m3/changes/stages.tsv`: a `b` stage is the household with only `on` moved, a `c` stage the
//! household with the change), each run in process on its law and def asks and held to
//! refcore's run (`m3/runs.vectors`, group `changes`), byte for byte. The walk asks are load
//! row L:10's, held to M4 (P8-L10); change 5's after side waits for M4's currencies. Two tests
//! (the even and the odd stages), so a slow debug run stays inside the test budget. `test =
//! false` until M3 is tagged (`engine/tools/milestones.toml`).

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally_runner::m3::{RUNS, group_failures};

    /// Holds every stage at `part` modulo 2 to refcore's run.
    fn stages_as_stored(part: usize) {
        let (failed, of) = group_failures((RUNS, "m3_changes"), "changes", (part, 2))
            .expect("m3/runs.vectors reads");
        assert!(
            failed.is_empty(),
            "{} of {of} change runs fail:\n{}",
            failed.len(),
            failed.join("\n")
        );
    }

    /// The stages b1, b2, b3, b4a, b4b, b5, c6a, c6b, c7 (`stages.tsv`'s order, the first of each
    /// two).
    #[test]
    fn m3_changes_part_1() {
        stages_as_stored(0);
    }

    /// The other stages.
    #[test]
    fn m3_changes_part_2() {
        stages_as_stored(1);
    }
}
