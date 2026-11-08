//! Milestone M1's goldens on the engine (issue #108; `GOLDENS.md`, `MILESTONE-M1.md` row 2,
//! DONE gate 2): the prototype's goldens and pins restated in tally 2, each refcore's whole
//! output in `m1/vectors.refcore` (groups `goldens-…`), the engine's held to it byte for
//! byte. `GOLDENS.md` records each figure against the prototype's. One test per golden, so
//! a failure names it.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use tally_runner::engine::run_args;
    use tally_runner::held::is_held;
    use tally_runner::kernel::{difference, vectors};
    use tally_runner::m1::VECTORS;

    /// The groups this target tests, one `#[test]` each.
    const GROUPS: [&str; 4] = ["goldens-pins", "goldens-g6", "goldens-g7", "goldens-g10"];

    /// Runs every vector of `group` on the engine and holds it to refcore's run.
    fn group_holds(group: &str) {
        let all = vectors(VECTORS).expect("m1/vectors.refcore reads");
        let mine: Vec<_> = all.iter().filter(|v| v.group == group).collect();
        assert!(!mine.is_empty(), "the group {group} has vectors");
        let failed: Vec<String> = mine
            .iter()
            .filter(|v| !is_held("m1_goldens", &v.id))
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
    fn m1_goldens_every_group_is_tested() {
        let named: BTreeSet<String> = vectors(VECTORS)
            .expect("m1/vectors.refcore reads")
            .into_iter()
            .map(|v| v.group)
            .filter(|g| g.starts_with("goldens-"))
            .collect();
        let tested: BTreeSet<String> = GROUPS.iter().map(|g| (*g).to_owned()).collect();
        assert_eq!(
            named, tested,
            "each goldens- group of m1/vectors.tsv has its test here"
        );
    }

    /// The pins `my_tax/net[2026-10]` 8 509.87, `[2027-03]` 8 837.78 and
    /// `wife_tax/net[2026-10]` 5 101.67 on household X's employment law with the prototype's
    /// `my_job`, and the separate PIT-37 returns that replace `pit_joint` (L-26). For G1 (the
    /// returns' year list).
    #[test]
    fn m1_goldens_pins_employment() {
        group_holds("goldens-pins");
    }

    /// G6: my net in 2027-03 with a 5 % raise from March, on top of indexation, 9 262.77, an
    /// ask in the scenario `raise`. Green before M1.
    #[test]
    fn m1_goldens_g6_raise() {
        group_holds("goldens-g6");
    }

    /// G7: a rent stated with its price year books 2 500.00 in 2028; the same figure in 2026
    /// money, 2 652.25. Green before M1.
    #[test]
    fn m1_goldens_g7_price_year() {
        group_holds("goldens-g7");
    }

    /// G10: the rent of 2030, 3 601.63, and 3 889.62 in the scenario `infl5`, whose row is
    /// known on its base row's day (L §6.4 replaces it whatever the two `known` dates). For
    /// G1.
    #[test]
    fn m1_goldens_g10_rent() {
        group_holds("goldens-g10");
    }
}
