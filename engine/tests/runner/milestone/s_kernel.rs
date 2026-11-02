//! Milestone S's kernel tests on the engine (issue #41; `MILESTONE-S.md` row 5, DONE gate 8):
//! every vector of `kernel/vectors.refcore` and every record of the kernel corpus run in
//! process, the engine's whole output and exit code held to refcore's, byte for byte. One test
//! per group of T's plan, so a failure names its group; every failure of a group is collected
//! before it fails, so one run shows them all.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use tally_runner::corpus::{KERNEL_CORPUS, KERNEL_REFCORE, records, refcore_runs};
    use tally_runner::engine::{run_args, run_record};
    use tally_runner::kernel::{VECTORS, difference, vectors};

    /// The groups this target tests, one `#[test]` each.
    const GROUPS: [&str; 1] = ["checks"];

    /// Runs every vector of `group` on the engine and holds it to refcore's run.
    fn group_holds(group: &str) {
        let all = vectors(VECTORS).expect("kernel/vectors.refcore reads");
        let mine: Vec<_> = all.iter().filter(|v| v.group == group).collect();
        assert!(!mine.is_empty(), "the group {group} has vectors");
        let failed: Vec<String> = mine
            .iter()
            .filter_map(|v| {
                difference(&v.refcore, &run_args(&v.args, &v.files))
                    .map(|d| format!("{}: {d}", v.id))
            })
            .collect();
        assert!(
            failed.is_empty(),
            "{} of {} vectors of {group} differ from refcore:\n{}",
            failed.len(),
            mine.len(),
            failed.join("\n")
        );
    }

    #[test]
    fn s_kernel_every_group_is_tested() {
        let named: BTreeSet<String> = vectors(VECTORS)
            .expect("kernel/vectors.refcore reads")
            .into_iter()
            .map(|v| v.group)
            .collect();
        let tested: BTreeSet<String> = GROUPS.iter().map(|g| (*g).to_owned()).collect();
        assert_eq!(
            named, tested,
            "each group of kernel/vectors.tsv has its test here"
        );
    }

    /// The kernel corpus's records (`properties/gen.py`), each the engine's whole output
    /// against refcore's (`properties/laws.py`).
    #[test]
    fn s_kernel_corpus_whole_output() {
        let runs = refcore_runs(KERNEL_REFCORE).expect("refcore's runs read");
        let recs = records(KERNEL_CORPUS).expect("the corpus reads");
        let failed: Vec<String> = recs
            .iter()
            .filter_map(|r| {
                let key = (r.case.clone(), r.variant.clone());
                match runs.get(&key) {
                    None => Some(format!("{}/{}: no refcore run", r.case, r.variant)),
                    Some(expected) => difference(expected, &run_record(r))
                        .map(|d| format!("{}/{}: {d}", r.case, r.variant)),
                }
            })
            .collect();
        assert!(
            failed.is_empty(),
            "{} of {} corpus records differ from refcore:\n{}",
            failed.len(),
            recs.len(),
            failed.join("\n")
        );
    }

    /// The checks of `checks.tsv` the engine at S answers: 33 unit probes, 5 models and 5 of
    /// household X's checks, each compared whole (a household check's warnings included).
    #[test]
    fn s_kernel_checks_whole_output() {
        group_holds("checks");
    }
}
