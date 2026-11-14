//! Milestone M2's checks on the engine (issue #167; `MILESTONE-M2.md` row 1): the checks of
//! BUILD-ORDER § M2's "Green (26)" that S's `s_kernel` doesn't already hold (3 of the 26
//! gate there: `I:M:LH-05`, `U:rec/REC-03`, `U:surf/SURF-06`), each refcore's whole output
//! in `m2/vectors.refcore`, the engine's held to it byte for byte. One test per builder
//! group (the `for` column of `m2/vectors.tsv`; `checks-plan` holds the ones whose group
//! the builder's plan hasn't named yet), so a group's test turns green with its group, and
//! one for the checks the engine answered before M2 (`checks-green`); every failure of a
//! group is collected before it fails.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use tally_runner::engine::run_args;
    use tally_runner::held::is_held;
    use tally_runner::kernel::{difference, vectors};
    use tally_runner::m2::VECTORS;

    /// The groups this target tests, one `#[test]` each.
    const GROUPS: [&str; 2] = ["checks-green", "checks-plan"];

    /// Runs every vector of `group` on the engine and holds it to refcore's run.
    fn group_holds(group: &str) {
        let all = vectors(VECTORS).expect("m2/vectors.refcore reads");
        let mine: Vec<_> = all.iter().filter(|v| v.group == group).collect();
        assert!(!mine.is_empty(), "the group {group} has vectors");
        let failed: Vec<String> = mine
            .iter()
            .filter(|v| !is_held("m2_checks", &v.id))
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
    fn m2_checks_every_group_is_tested() {
        let named: BTreeSet<String> = vectors(VECTORS)
            .expect("m2/vectors.refcore reads")
            .into_iter()
            .map(|v| v.group)
            .filter(|g| g.starts_with("checks-"))
            .collect();
        let tested: BTreeSet<String> = GROUPS.iter().map(|g| (*g).to_owned()).collect();
        assert_eq!(
            named, tested,
            "each checks- group of m2/vectors.tsv has its test here"
        );
    }

    /// The 20 checks the engine already answers byte for byte but that no gating target held
    /// before M2: X's `wife_net`, `social_and_health` and `pit37` on change 3 and its variant
    /// (the retroactive law, L §5.14.3, E §5), `I:M:LH-14`, `RC-08`, `UL-06` (#42's case),
    /// `UL-09`, `CH-04`, and the probes `LAW-04`, `REC-01`, `-02`, `-05`, `-07`, `RULE-06`,
    /// `SCEN-02` and its set, `VAL-05`. Green from the start; it keeps them so.
    #[test]
    fn m2_checks_green() {
        group_holds("checks-green");
    }

    /// The checks the engine stops "not implemented" today, until the builder's plan names
    /// their groups: `U:rule/RULE-07` (`bal … modelled` and `residuals`, L §5.19, E §6.2,
    /// §6.3), `U:rule/RULE-08` (a parameter estimated from history up to the decision's
    /// date, L §5.11, E §2) and `I:M:UL-15` (PIT 2022, two employments, the return as amended
    /// mid-year).
    #[test]
    fn m2_checks_plan() {
        group_holds("checks-plan");
    }
}
