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
    const GROUPS: [&str; 8] = [
        "goldens-pins",
        "goldens-g6",
        "goldens-g7",
        "goldens-g10",
        "goldens-y22",
        "goldens-business",
        "goldens-g8",
        "goldens-g9",
    ];

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

    /// The 2022 pins (06-pit-2022): the advances of January and June, 266, and of July and
    /// December, 188, each month under its own law; the return under July's scale for the
    /// whole year, 2 253, its advances 2 724 and its due −471; as known in March 2022, the
    /// return under January's scale, 3 192 (L §4.5, `goldens.md`: changes form).
    #[test]
    fn m1_goldens_pins_2022() {
        group_holds("goldens-y22");
    }

    /// The side business's pins (05-side-business): health in 2026-10, 498.35, the ryczałt
    /// advance, 690, the year's health settlement, 0, and the VAT due, 1 288.
    #[test]
    fn m1_goldens_pins_business() {
        group_holds("goldens-business");
    }

    /// G8's first half: a job whose pay no law reads warns (`WARN-UNREAD`; the 381 885.38
    /// half is blocked, `GOLDENS.md`).
    #[test]
    fn m1_goldens_g8_job_without_law() {
        group_holds("goldens-g8");
    }

    /// G9: a question with a payment is refused (C-19: an ask's grammar has no pay); its base
    /// answers.
    #[test]
    fn m1_goldens_g9_payment_refused() {
        group_holds("goldens-g9");
    }
}
