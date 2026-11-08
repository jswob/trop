//! Milestone M1's carried items on the engine (issue #108; `MILESTONE-M1.md` row 6; the M1
//! brief §1 "In scope" 1–9): the pre-M1 run's items left for M1, each a fixture of its own
//! under `m1/carried/`, refcore's whole output in `m1/vectors.refcore` (groups `carried-<item>`),
//! the engine's held to it byte for byte. Each vector's builder group is its `for` column of
//! `m1/vectors.tsv`. One test per item, so a failure names it. `test = false` until M1's
//! marker (the coordinator's pick P4).

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use tally_runner::engine::run_args;
    use tally_runner::held::is_held;
    use tally_runner::kernel::{difference, vectors};
    use tally_runner::m1::VECTORS;

    /// The groups this target tests, one `#[test]` each.
    const GROUPS: [&str; 7] = [
        "carried-1",
        "carried-2",
        "carried-3",
        "carried-4",
        "carried-5",
        "carried-6",
        "carried-7",
    ];

    /// Runs every vector of `group` on the engine and holds it to refcore's run.
    fn group_holds(group: &str) {
        let all = vectors(VECTORS).expect("m1/vectors.refcore reads");
        let mine: Vec<_> = all.iter().filter(|v| v.group == group).collect();
        assert!(!mine.is_empty(), "the group {group} has vectors");
        let failed: Vec<String> = mine
            .iter()
            .filter(|v| !is_held("m1_carried", &v.id))
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
    fn m1_carried_every_group_is_tested() {
        let named: BTreeSet<String> = vectors(VECTORS)
            .expect("m1/vectors.refcore reads")
            .into_iter()
            .map(|v| v.group)
            .filter(|g| g.starts_with("carried-"))
            .collect();
        let tested: BTreeSet<String> = GROUPS.iter().map(|g| (*g).to_owned()).collect();
        assert_eq!(
            named, tested,
            "each carried- group of m1/vectors.tsv has its test here"
        );
    }

    /// Item 1, scenarios as layers (D1-3, L §6.4): an ask in the scenario's world beside the
    /// base, a scenario file's ask in the model, `as if {scenario s}`, a difference by key and
    /// one of two worlds (`m1/carried/layers/`). For G6 (two of six green before M1).
    #[test]
    fn m1_carried_1_scenarios_as_layers() {
        group_holds("carried-1");
    }

    /// Item 2, reads at run (L §2.5, §5.6): a kind-keyed input at a key a row gives, a stepped
    /// range's input at a key no row states (a hole), a row for every key, money and fraction
    /// ranges (`m1/carried/reads/`). For G13.
    #[test]
    fn m1_carried_2_reads_at_run() {
        group_holds("carried-2");
    }

    /// Item 3, ranges an ask steps through (L §5.6, §7.6): a step of 0.001, an end computed
    /// from rows, and the 1 000 000-key limit on each side (`m1/carried/ranges/`). For G13
    /// (the limit's twins green before M1).
    #[test]
    fn m1_carried_3_ranges_an_ask_steps_through() {
        group_holds("carried-3");
    }

    /// Item 4, the 26 run sites G4a's fix left "not implemented": one probe per cause class,
    /// each refused at load first by refcore (bin i: the site is unreachable, so the builder
    /// makes it `ENGINE-INVARIANT` with its reason; `m1/carried/sites/`). 11 of 12 green before
    /// M1; red: a missing field, which the engine names by the move, not its kind. For G1.
    #[test]
    fn m1_carried_4_the_26_sites() {
        group_holds("carried-4");
    }

    /// Item 5, the coordinator's picks P6 and P12: a parenthesised literal is that literal
    /// (L §2.1), where refcore reads it so: compared to a year, a month key, a day
    /// (`m1/carried/years/`, a file per form beside the base). The forms refcore refuses or
    /// misreads are in `m1/carried/held.tsv` (GAPS.md). For G1 (the month and the day green).
    #[test]
    fn m1_carried_5_a_parenthesised_literal() {
        group_holds("carried-5");
    }

    /// Item 6, #97's "Later (M1)" list: N1, the written year forms (`2026 + 1 year`, a list
    /// `[2026, 2027]`); N4, `pos(k + 1)` inside, past the end and under a guard; N7, `weekday`,
    /// `last_business`, `add_business` over a calendar with an `added` window, `explain` of a
    /// balance a movement moves; B1, a sum over a 100 000-key list (`m1/carried/years/`,
    /// `steps/`, `big/`). For G1 (N1), G2 (N7's calendar), G3 (N4), G12 (`explain`).
    #[test]
    fn m1_carried_6_the_later_list() {
        group_holds("carried-6");
    }

    /// Item 7, the G2 review's walk notes: a plan whose day reads a hole prints the hole and
    /// books nothing at the run's end (R-27, `m1/carried/walk/`). Green before M1; for G10's
    /// rework of `plan.rs`.
    #[test]
    fn m1_carried_7_the_walk_notes() {
        group_holds("carried-7");
    }
}
