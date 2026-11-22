//! Milestone M4's constructs on the engine (issue #252; `MILESTONE-M4.md` row 2): small
//! invented texts that pin the corners of each construct BUILD-ORDER § M4 builds, beyond the
//! 77 checks, each ask refcore's whole output in `m4/vectors.refcore`, the engine's held to
//! it byte for byte. One test per construct (`constructs-<construct>` in `m4/vectors.tsv`),
//! so a construct's corners turn green with the group that builds it; every failure of a
//! group is collected before it fails.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use tally_runner::engine::run_args;
    use tally_runner::held::is_held;
    use tally_runner::kernel::{difference, vectors};
    use tally_runner::m4::VECTORS;

    /// The groups this target tests, one `#[test]` each.
    const GROUPS: [&str; 2] = ["constructs-pins", "constructs-accrue"];

    /// Runs every vector of `group` on the engine and holds it to refcore's run.
    fn group_holds(group: &str) {
        let all = vectors(VECTORS).expect("m4/vectors.refcore reads");
        let mine: Vec<_> = all.iter().filter(|v| v.group == group).collect();
        assert!(!mine.is_empty(), "the group {group} has vectors");
        let failed: Vec<String> = mine
            .iter()
            .filter(|v| !is_held("m4_constructs", &v.id))
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
    fn m4_constructs_every_group_is_tested() {
        let named: BTreeSet<String> = vectors(VECTORS)
            .expect("m4/vectors.refcore reads")
            .into_iter()
            .map(|v| v.group)
            .filter(|g| g.starts_with("constructs-"))
            .collect();
        let tested: BTreeSet<String> = GROUPS.iter().map(|g| (*g).to_owned()).collect();
        assert_eq!(
            named, tested,
            "each constructs- group of m4/vectors.tsv has its test here"
        );
    }

    /// `m4/constructs/pins/pins.tally`: an `as if` pin keyed by a def's own key, by an ask's
    /// `for` variable and by an aggregate's bound variable; a pin at another key than the one
    /// read; the least t such that c with the pin's value the bound t, and its empty key
    /// (`RUN-EMPTY`); one def under two pin sets in one line; a pin key that is a hole; a `with`
    /// pin over a def's own pin; `explain` under the pin; and in a def of two keys, a pin at
    /// both keys or one, its value fixed or read at the def's keys, the form UL-14 and C-04
    /// stop on today (`b_def`, `b_lit`, `b_mix`) (L §5.11 `as if`, §8.2 `pinned:`).
    #[test]
    fn m4_constructs_pins() {
        group_holds("constructs-pins");
    }

    /// `m4/constructs/accrue/accrue.tally`: `accrue` under act365 for a day and a month,
    /// `simple`, `effective` over a whole year, `nominal 365` over a day; b before a; a rate
    /// that is a hole; `dcf` both ways; a rate read at its day across a change, and their sum
    /// (L §3.3).
    #[test]
    fn m4_constructs_accrue() {
        group_holds("constructs-accrue");
    }
}
