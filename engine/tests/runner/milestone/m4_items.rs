//! Milestone M4's in-scope backlog on the engine (issue #252; `MILESTONE-M4.md` row 3): small
//! invented texts in `m4/items/` for #238's standing items, #245's "Built in part" and #250's
//! "The engine alone", each ask refcore's whole output in `m4/vectors.refcore`, the engine's
//! held to it byte for byte. One test per builder group that builds them (`items-<group>` in
//! `m4/vectors.tsv`, the builder's plan: G2 `paper`, G3 `walked`, G4 `backlog`); a vector
//! where the engine follows the text and refcore differs is held "for #n" in `held.tsv`, its
//! spec answer filled after the group. Every failure of a group is collected before it fails.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use tally_runner::engine::run_args;
    use tally_runner::held::is_held;
    use tally_runner::kernel::{difference, vectors};
    use tally_runner::m4::VECTORS;

    /// The groups this target tests, one `#[test]` each.
    const GROUPS: [&str; 3] = ["items-paper", "items-walked", "items-backlog"];

    /// Runs every vector of `group` on the engine and holds it to refcore's run.
    fn group_holds(group: &str) {
        let all = vectors(VECTORS).expect("m4/vectors.refcore reads");
        let mine: Vec<_> = all.iter().filter(|v| v.group == group).collect();
        assert!(!mine.is_empty(), "the group {group} has vectors");
        let failed: Vec<String> = mine
            .iter()
            .filter(|v| !is_held("m4_items", &v.id))
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
    fn m4_items_every_group_is_tested() {
        let named: BTreeSet<String> = vectors(VECTORS)
            .expect("m4/vectors.refcore reads")
            .into_iter()
            .map(|v| v.group)
            .filter(|g| g.starts_with("items-"))
            .collect();
        let tested: BTreeSet<String> = GROUPS.iter().map(|g| (*g).to_owned()).collect();
        assert_eq!(
            named, tested,
            "each items- group of m4/vectors.tsv has its test here"
        );
    }

    /// G2 `paper`: `m4/items/months/months.tally`, #250's "The engine alone": a plain `check`
    /// of two months, two quarters and two years that differ, and of one month with itself
    /// (L §5.18 `check`, §4.2 "Differences").
    #[test]
    fn m4_items_paper() {
        group_holds("items-paper");
    }

    /// G3 `walked`: `m4/items/explain/`, #244's row 5 (#245): `explain` of a def that reads a
    /// decision's `flows` or `count`, on a plain day (`plain.tally`) and on a window's hole
    /// day (`hole.tally`); and `m4/items/g3/g3.tally`, #238's gate review 3: a move's key
    /// whose `where` waits for the tuple's `at` beside a key whose `where` is refused, and the
    /// same with every waiting `where` false (E §10, L §5.18; L §5.14 Keys, §4.3).
    #[test]
    fn m4_items_walked() {
        group_holds("items-walked");
    }

    /// G4 `backlog`: `m4/items/none/none.tally`, #245's `RUN-NONE` builders (`sum`, `min`,
    /// `abs`, `round` held for #245, pick P3; `if`, `and`, `not` and membership on refcore's
    /// line); `m4/items/age/age.tally`, #250's `age_months` in the calendar's last month (held
    /// for #250) and the month before; and #238's A-4 (`m4/items/a4/`, a walk-bound def read by
    /// a decision's `when`, false, and in a cycle), gate review 5 (`m4/items/g5/`, a member's
    /// texts holding `·` in a joint run, held for #238) and gate review 7 (`m4/items/g7/`, a
    /// member with a refused scenario and asks refused alone, run through its joint model)
    /// (L §8.4, §4.2, §6.1).
    #[test]
    fn m4_items_backlog() {
        group_holds("items-backlog");
    }
}
