//! Milestone M4's checks on the engine (issue #252; `MILESTONE-M4.md` row 1): 73 of the 77
//! checks of BUILD-ORDER § M4's "Green (77)" (the other 4 gate in `s_kernel` since S), each
//! refcore's whole output in `m4/vectors.refcore`,
//! the engine's held to it byte for byte. One test per builder group (the `for` column of
//! `m4/vectors.tsv`: `checks-pins` for an `as if` pin at bound keys, `checks-accrue` for
//! `accrue`), so a group's test turns green with its group, and one for the checks the engine
//! answered before M4 (`checks-green`); every failure of a group is collected before it
//! fails. `I:D:d11_not_follows_fund` is held to M7 by name (`held.tsv`).

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use tally_runner::engine::run_args;
    use tally_runner::held::is_held;
    use tally_runner::kernel::{difference, vectors};
    use tally_runner::m4::VECTORS;

    /// The groups this target tests, one `#[test]` each.
    const GROUPS: [&str; 3] = ["checks-green", "checks-pins", "checks-accrue"];

    /// Runs every vector of `group` on the engine and holds it to refcore's run.
    fn group_holds(group: &str) {
        let all = vectors(VECTORS).expect("m4/vectors.refcore reads");
        let mine: Vec<_> = all.iter().filter(|v| v.group == group).collect();
        assert!(!mine.is_empty(), "the group {group} has vectors");
        let failed: Vec<String> = mine
            .iter()
            .filter(|v| !is_held("m4_checks", &v.id))
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
    fn m4_checks_every_group_is_tested() {
        let named: BTreeSet<String> = vectors(VECTORS)
            .expect("m4/vectors.refcore reads")
            .into_iter()
            .map(|v| v.group)
            .filter(|g| g.starts_with("checks-"))
            .collect();
        let tested: BTreeSet<String> = GROUPS.iter().map(|g| (*g).to_owned()).collect();
        assert_eq!(
            named, tested,
            "each checks- group of m4/vectors.tsv has its test here"
        );
    }

    /// The 69 checks of this store the engine already answers byte for byte (`U:cur/CUR-03`,
    /// LAW-02, -07, -10, -11; 32 of the 34 cases; X's change 5 and its variant; four of
    /// household C's five; 22 of D's 23), and D-11's path ask, held to M7. Green from the
    /// start; it keeps them so. (`U:cur/CUR-01`, `CUR-02`, `U:law/LAW-12` and `I:M:RC-07`
    /// gate in `s_kernel` since S.)
    #[test]
    fn m4_checks_green() {
        group_holds("checks-green");
    }

    /// `I:M:UL-14` and `I:C:C-04~c04`: an `as if` pin whose keys or value are bound variables,
    /// a def's keys or an ask's `for` (L §5.11 `as if`, "A pin's keys"; C-04's inverse, "the
    /// least x such that c"), stopped "not implemented" today.
    #[test]
    fn m4_checks_pins() {
        group_holds("checks-pins");
    }

    /// `I:E2:E2-01~e2c`: C's loan law accrues each day's interest,
    /// `accrue(p, r, d, d + 1, act365, simple)` (L §3.3), stopped "not implemented" today.
    #[test]
    fn m4_checks_accrue() {
        group_holds("checks-accrue");
    }
}
