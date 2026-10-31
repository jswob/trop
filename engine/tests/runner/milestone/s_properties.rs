//! Milestone S's property tests on the engine (`engine/tests/PROPERTIES.md`, the kernel's
//! part; DONE gate 6). Each corpus case is run on the engine in process and held to its law,
//! the same law `tally-runner`'s own tests prove on refcore's outputs. Red on the stub (exit
//! 69, "not implemented"), so this target has `test = false` until milestones are gated.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use tally_runner::corpus::{KERNEL_CORPUS, records};
    use tally_runner::engine::run_record;
    use tally_runner::laws::{Runs, check};

    /// Runs every record on the engine and checks each case against its law. Collects every
    /// failure before failing, so one run shows them all.
    fn law_holds(law: &str) {
        let mut cases: BTreeMap<String, Runs> = BTreeMap::new();
        for r in records(KERNEL_CORPUS)
            .expect("the corpus reads")
            .iter()
            .filter(|r| r.law == law)
        {
            cases
                .entry(r.case.clone())
                .or_insert_with(BTreeMap::new)
                .insert(r.variant.clone(), run_record(r));
        }
        assert!(!cases.is_empty(), "the corpus has cases of {law}");
        let failed: Vec<String> = cases
            .iter()
            .filter_map(|(case, runs)| check(law, runs).err().map(|e| format!("{case}: {e}")))
            .collect();
        assert!(
            failed.is_empty(),
            "{} of {} cases fail:\n{}",
            failed.len(),
            cases.len(),
            failed.join("\n")
        );
    }

    #[test]
    fn s_determinism_files_and_sections_in_any_order() {
        law_holds("determinism");
    }

    #[test]
    fn s_determinism_across_runs() {
        for r in records(KERNEL_CORPUS).expect("the corpus reads") {
            let first = run_record(&r);
            // refcore answers every record (exit 0), so an unanswered run can't pass vacuously
            assert_eq!(
                first.code, 0,
                "{}/{}: not answered: {}",
                r.case, r.variant, first.out
            );
            assert_eq!(
                first,
                run_record(&r),
                "{}/{}: two runs differ",
                r.case,
                r.variant
            );
        }
    }

    #[test]
    fn s_money_conserved() {
        law_holds("conservation");
    }

    #[test]
    fn s_no_foresight_past_the_cut() {
        law_holds("foresight-known");
    }

    #[test]
    fn s_no_foresight_in_a_decision() {
        law_holds("foresight-decision");
    }

    #[test]
    fn s_no_foresight_through_the_written_projection() {
        law_holds("foresight-projection");
    }

    #[test]
    fn s_a_hole_never_becomes_zero() {
        law_holds("hole");
    }
}
