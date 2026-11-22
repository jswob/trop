//! Milestone M4's group reviews on the engine (issue #252; `MILESTONE-M4.md` row 5): each
//! review's findings as vectors in `m4/fixes/<group>/`, refcore's whole output in
//! `m4/vectors.refcore` (groups `fixes-…`), and the spec's answer where refcore crashes or
//! answers otherwise in `m4/spec.vectors` (groups `spec-…`, `m4/spec.py`); refcore's bytes
//! beside a spec answer are held "for #n" in `held.tsv`. One test per group, so a review's
//! blocking fix turns its own groups green; every failure of a group is collected before it
//! fails.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use tally_runner::engine::run_args;
    use tally_runner::held::is_held;
    use tally_runner::kernel::{difference, vectors};
    use tally_runner::m4::{SPEC, VECTORS};

    /// The groups this target tests, one `#[test]` each: refcore's (`fixes-…`) and the
    /// spec's (`spec-…`).
    const GROUPS: [&str; 2] = ["fixes-g1", "spec-g1"];

    /// Runs every vector of `group` in `store` on the engine and holds it to the stored run.
    fn group_holds(store: &[u8], group: &str) {
        let all = vectors(store).expect("the store reads");
        let mine: Vec<_> = all.iter().filter(|v| v.group == group).collect();
        assert!(!mine.is_empty(), "the group {group} has vectors");
        let failed: Vec<String> = mine
            .iter()
            .filter(|v| !is_held("m4_fixes", &v.id))
            .filter_map(|v| {
                difference(&v.refcore, &run_args(&v.args, &v.files))
                    .map(|d| format!("{}: {d}", v.id))
            })
            .collect();
        assert!(
            failed.is_empty(),
            "{} of {} vectors of {group} differ from the stored run:\n{}",
            failed.len(),
            mine.len(),
            failed.join("\n")
        );
    }

    #[test]
    fn m4_fixes_every_group_is_tested() {
        let named: BTreeSet<String> = vectors(VECTORS)
            .expect("m4/vectors.refcore reads")
            .into_iter()
            .chain(vectors(SPEC).expect("m4/spec.vectors reads"))
            .map(|v| v.group)
            .filter(|g| g.starts_with("fixes-") || g.starts_with("spec-"))
            .collect();
        let tested: BTreeSet<String> = GROUPS.iter().map(|g| (*g).to_owned()).collect();
        assert_eq!(
            named, tested,
            "each fixes- and spec- group of M4's stores has its test here"
        );
    }

    /// G1 `pins-accrue`'s review (`tmp/m4/review-g1.md`), refcore's bytes for each probe's
    /// asks: B1, a per-tuple pin whose value names two bindings inside a walk's read (`last`,
    /// `bal`, `flows`, `value`, a `count` over a span; `B1_min`, `p2_unordered`, `p14_bal`,
    /// `b1_reads`), which stops the run "not implemented" (L §5.11 A pin's keys, §8.2); the
    /// crossings both engines answer alike; and, held for #255, B2's holes and the
    /// non-blocking rows where refcore is wrong.
    #[test]
    fn m4_fixes_g1() {
        group_holds(VECTORS, "fixes-g1");
    }

    /// G1's review, the spec's answers (#255): B2, a refusal outranks a hole in `accrue`,
    /// `periodic`, `factor` and `round` (L §8; each from refcore's run of a twin that states
    /// the hole); a second pin on one hole keeps its `pinned:` line (L §8.2); `explain`'s
    /// `see [n]` in one world (E §10); ranges and inverses over rates (L §3.3, §5.11);
    /// `nominal 0` (`RUN-DIV`); `with` pins naming the ask's `for` (L §5.18), where refcore
    /// crashes. The written ones are G1's build's answers, checked by hand.
    #[test]
    fn m4_fixes_spec_g1() {
        group_holds(SPEC, "spec-g1");
    }
}
