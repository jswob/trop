//! Milestone M2's goldens (issue #167, #114; `MILESTONE-M2.md` row 2, `GOLDENS.md`): the
//! prototype's figures that read its bank history, restated over `m2/bank/` (the ledger as
//! 34 monthly record parts and the bank half of R7), with refcore's output in
//! `m2/vectors.refcore`, the engine's held to it byte for byte. One test per builder group
//! (the `for` column of `m2/vectors.tsv`), so a group's test turns green with its group;
//! every failure of a group is collected before it fails.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use tally_runner::engine::run_args;
    use tally_runner::held::is_held;
    use tally_runner::kernel::{difference, vectors};
    use tally_runner::m2::VECTORS;

    /// The groups this target tests, one `#[test]` each.
    const GROUPS: [&str; 2] = ["goldens-records", "goldens-ledger"];

    /// Runs every vector of `group` on the engine and holds it to refcore's run.
    fn group_holds(group: &str) {
        let all = vectors(VECTORS).expect("m2/vectors.refcore reads");
        let mine: Vec<_> = all.iter().filter(|v| v.group == group).collect();
        assert!(!mine.is_empty(), "the group {group} has vectors");
        let failed: Vec<String> = mine
            .iter()
            .filter(|v| !is_held("m2_goldens", &v.id))
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
    fn m2_goldens_every_group_is_tested() {
        let named: BTreeSet<String> = vectors(VECTORS)
            .expect("m2/vectors.refcore reads")
            .into_iter()
            .map(|v| v.group)
            .filter(|g| g.starts_with("goldens-"))
            .collect();
        let tested: BTreeSet<String> = GROUPS.iter().map(|g| (*g).to_owned()).collect();
        assert_eq!(
            named, tested,
            "each goldens- group of m2/vectors.tsv has its test here"
        );
    }

    /// G4 `records`: G2, konto 11 685.67 on 2026-10-16 (and 12 000.00 the day before) over
    /// 34 months of observed rows anchored at each month's end; G5, the learned daily
    /// spending, 234.22, the last 365 days' `codzienne` rows times a flat `prices` (L §5.19
    /// reading history, `covered`); the pay law's nets the ledger's pay rows fulfil.
    #[test]
    fn m2_goldens_records() {
        group_holds("goldens-records");
    }

    /// G5 `ledger`: G4, the residual ledger (E §6.3): the pay of 2025-12, model 13 611.54
    /// against bank 16 611.54, one matched line +3 000.00, every other pay and rent month
    /// 0.00, by employer, by landlord and by category.
    #[test]
    fn m2_goldens_ledger() {
        group_holds("goldens-ledger");
    }
}
