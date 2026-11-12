//! Milestone M1's checks on the engine (issue #108; `MILESTONE-M1.md` row 1): the checks of
//! BUILD-ORDER § M1's "Green (90)" that S's `s_kernel` doesn't already hold (the coordinator's
//! pick P5: 26 of the 90 gate there), and `inputs_of_balance` (#36, moved from `later/`),
//! each refcore's whole output in `m1/vectors.refcore`, the engine's held to it byte for
//! byte. One test per builder group of `tmp/m1/build-plan.md` §3 (the `for` column of
//! `m1/vectors.tsv`), so a group's test turns green with its group, and one for the checks
//! the engine answered before M1 (`checks-green`); every failure of a group is collected
//! before it fails.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use tally_runner::engine::run_args;
    use tally_runner::held::is_held;
    use tally_runner::kernel::{difference, vectors};
    use tally_runner::m1::VECTORS;

    /// The groups this target tests, one `#[test]` each.
    const GROUPS: [&str; 11] = [
        "checks-green",
        "checks-g1",
        "checks-g2",
        "checks-g3",
        "checks-g4",
        "checks-g5",
        "checks-g6",
        "checks-g7",
        "checks-g8",
        "checks-g10",
        "checks-g11",
    ];

    /// Runs every vector of `group` on the engine and holds it to refcore's run.
    fn group_holds(group: &str) {
        let all = vectors(VECTORS).expect("m1/vectors.refcore reads");
        let mine: Vec<_> = all.iter().filter(|v| v.group == group).collect();
        assert!(!mine.is_empty(), "the group {group} has vectors");
        let failed: Vec<String> = mine
            .iter()
            .filter(|v| !is_held("m1_checks", &v.id))
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
    fn m1_checks_every_group_is_tested() {
        let named: BTreeSet<String> = vectors(VECTORS)
            .expect("m1/vectors.refcore reads")
            .into_iter()
            .map(|v| v.group)
            .filter(|g| g.starts_with("checks-"))
            .collect();
        let tested: BTreeSet<String> = GROUPS.iter().map(|g| (*g).to_owned()).collect();
        assert_eq!(
            named, tested,
            "each checks- group of m1/vectors.tsv has its test here"
        );
    }

    /// The 17 checks the engine already answers byte for byte but that no gating target
    /// held before M1: `U:law/LAW-13/LAW-13`, `U:scen/SCEN-04/SCEN-04`,
    /// `U:time/TIME-08-before`, X's `wife_net` and `social_and_health` on changes 1, 2, 4, 6,
    /// 7 and variant 4, and `n`, `n2` on `min_m2b`. Green from the start; it keeps them so.
    #[test]
    fn m1_checks_green() {
        group_holds("checks-green");
    }

    /// G1, the written year forms and a scenario's `replace` on its own known day (L §2.1,
    /// §5.6, §6.4): X's `pit37` on changes 1, 2, 4, 6, 7 and variant 4 (`y in [2026, 2027,
    /// 2028]` over a year index), `U:life/LIFE-01` (`year_of(m) == 2025`), `n`, `n2` on
    /// `min_m2`.
    #[test]
    fn m1_checks_g1() {
        group_holds("checks-g1");
    }

    /// G2, the date, calendar and number built-ins (L §3.1–§3.3, §4.1–§4.4): `TIME-01`,
    /// `-02`, `-04`, `-05`, `NUM-01`, `-02`, `LIFE-03`.
    #[test]
    fn m1_checks_g2() {
        group_holds("checks-g2");
    }

    /// G3, aggregates, rules, bands and keys (L §5.5, §5.6, §5.8, §5.9, §5.11): `ASK-01`,
    /// `LAW-05`, `LAW-06`, `PTY-01`, `TIME-06`, `VAL-01`.
    #[test]
    fn m1_checks_g3() {
        group_holds("checks-g3");
    }

    /// G4, `prorata` and its largest-remainder split (L §5.13): `NUM-05`.
    #[test]
    fn m1_checks_g4() {
        group_holds("checks-g4");
    }

    /// G5, pins as a layer and the asks that pin (L §4.5, §5.11, §5.18): `ASK-04`, `LAW-03`,
    /// `SURF-04`, `VAL-04`.
    #[test]
    fn m1_checks_g5() {
        group_holds("checks-g5");
    }

    /// G6, worlds as a layer (L §6.1–§6.4): the probe sets of `LAW-13`, `SCEN-01`, `SCEN-04`
    /// and `SURF-04`.
    #[test]
    fn m1_checks_g6() {
        group_holds("checks-g6");
    }

    /// G7, axes at run (L §5.5): `NUM-04`, `I:M:UL-01`, `-02`, `-05`.
    #[test]
    fn m1_checks_g7() {
        group_holds("checks-g7");
    }

    /// G8, the written projections and the walk's reads (L §3.6, §5.14.1, §5.19, E §6.1):
    /// `OWN-02`, `RULE-02`, `FLOW-01`, `FLOW-02`.
    #[test]
    fn m1_checks_g8() {
        group_holds("checks-g8");
    }

    /// G10, decisions and bindings over movements (L §5.7, §5.14.1, §5.14.2): `RULE-05`, X's
    /// minimal texts `F4g`, `F4h`, `m10l`.
    #[test]
    fn m1_checks_g10() {
        group_holds("checks-g10");
    }

    /// G11, `explain`, `inputs-of` and `readers-of` (L §5.18, E §10): `ASK-05`, `VAL-03`,
    /// `SCEN-03` and its set, and #36's `inputs_of_balance` (a cell's read set holds its walk
    /// reads; tally 2.1.6, P1).
    #[test]
    fn m1_checks_g11() {
        group_holds("checks-g11");
    }
}
