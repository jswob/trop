//! Milestone M2's review fixes and refcore 2.1.7's misses (issue #167; `MILESTONE-M2.md`
//! row 8). Two stores: `m2/vectors.refcore`'s `fixes-…` groups, refcore's bytes for the
//! builders' review probes where refcore answers otherwise than the spec (each held "for
//! #169" or "for #173" in `held.tsv`, so it runs once refcore 2.1.8 is stored), and
//! `m2/spec.vectors`, the spec's answers T wrote (`m2/spec.py`) where refcore crashes or
//! answers otherwise, each row's section named in `m2/spec.tsv`. One test per group; every
//! failure of a group is collected before it fails. The probes' asks where refcore and the
//! engine agree gate in `m1_fixes`' `fixes-gate8` (`fixes/gate8/m2_*`).

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use tally_runner::engine::run_args;
    use tally_runner::held::is_held;
    use tally_runner::kernel::{difference, vectors};
    use tally_runner::m2::{SPEC, VECTORS};

    /// The held groups of `m2/vectors.refcore` this target tests, one `#[test]` each.
    const GROUPS: [&str; 6] = [
        "fixes-forms",
        "fixes-explain",
        "fixes-windows",
        "fixes-sweep",
        "fixes-records",
        "fixes-ledger",
    ];

    /// The groups of `m2/spec.vectors`, one `#[test]` each.
    const SPEC_GROUPS: [&str; 4] = ["spec-forms", "spec-explain", "spec-windows", "spec-records"];

    /// Runs every vector of `group` in `store` on the engine, less the held ones, and holds it
    /// to the stored run.
    fn group_holds(store: &[u8], group: &str) {
        let all = vectors(store).expect("the store reads");
        let mine: Vec<_> = all.iter().filter(|v| v.group == group).collect();
        assert!(!mine.is_empty(), "the group {group} has vectors");
        let failed: Vec<String> = mine
            .iter()
            .filter(|v| !is_held("m2_fixes", &v.id))
            .filter_map(|v| {
                difference(&v.refcore, &run_args(&v.args, &v.files))
                    .map(|d| format!("{}: {d}", v.id))
            })
            .collect();
        assert!(
            failed.is_empty(),
            "{} of {} vectors of {group} differ from the stored answer:\n{}",
            failed.len(),
            mine.len(),
            failed.join("\n")
        );
    }

    #[test]
    fn m2_fixes_every_group_is_tested() {
        let named: BTreeSet<String> = vectors(VECTORS)
            .expect("m2/vectors.refcore reads")
            .into_iter()
            .map(|v| v.group)
            .filter(|g| g.starts_with("fixes-"))
            .chain(
                vectors(SPEC)
                    .expect("m2/spec.vectors reads")
                    .into_iter()
                    .map(|v| v.group),
            )
            .collect();
        let tested: BTreeSet<String> = GROUPS
            .iter()
            .chain(SPEC_GROUPS.iter())
            .map(|g| (*g).to_owned())
            .collect();
        assert_eq!(
            named, tested,
            "each fixes- and spec- group has its test here"
        );
    }

    /// G1's review: a cross-world `diff` past 64 combinations (P17 B-2), held for #169.
    #[test]
    fn m2_fixes_forms() {
        group_holds(VECTORS, "fixes-forms");
    }

    /// G2's reviews: the walk-bound explain of a decision less its own earlier observation
    /// (P21 B-3), an observation in a sum of leg amounts (P24 F-2), readings removed by
    /// `replace … = none` (P21 NB-3, P24 SQ-1), `last` of a reading `after m` (P11, G4's), all
    /// held for #169; a unitless reading row at a place of unit `any` (P22), held for #173.
    #[test]
    fn m2_fixes_explain() {
        group_holds(VECTORS, "fixes-explain");
    }

    /// G3's review: `low … windows each` without `from` (P11), flows over a window crossing an
    /// end (B1), a binding under `windows each` (B2), a known window holding the cut (B4), a
    /// read's own clause and the register (B5), a pinned window (P30), a windowed instance's
    /// body and keys (NB-6, NB-7), all held for #169.
    #[test]
    fn m2_fixes_windows() {
        group_holds(VECTORS, "fixes-windows");
    }

    /// The sweep group's: `add_business` past the day range with a hole on the way (P5, P15)
    /// and a four-digit literal where money is expected (P16), held for #169.
    #[test]
    fn m2_fixes_sweep() {
        group_holds(VECTORS, "fixes-sweep");
    }

    /// G4's review probes where refcore drops a reading after a modelled-past m (P43 B1, B2,
    /// B4; `a_*`, `c_since`), held for #169: their asks both engines answer alike gate as
    /// m1's `fixes/gate8/m2_g4_*`.
    #[test]
    fn m2_fixes_records() {
        group_holds(VECTORS, "fixes-records");
    }

    /// The ledger group's: `HOLE-SAME` names its rows in the `same` line's written order
    /// (P51, #177; `adj_amt`'s settlement and its adjustment's remainder).
    #[test]
    fn m2_fixes_ledger() {
        group_holds(VECTORS, "fixes-ledger");
    }

    /// The spec's answers for G4's held probes (P43) and the `nb3_*` texts refcore can't run:
    /// a link's `when` that reads a reading after its own instance is `RUN-CYCLE` (L §5.14.2).
    #[test]
    fn m2_fixes_spec_records() {
        group_holds(SPEC, "spec-records");
    }

    /// The spec's answers for G1's forms: a quantile's `none` level (P17 B-1), `residuals`
    /// fanned out over an axis (P14), a cross-world `diff` past 64 combinations (P17 B-2).
    #[test]
    fn m2_fixes_spec_forms() {
        group_holds(SPEC, "spec-forms");
    }

    /// The spec's answers for G2's fixes (P21 B-3, NB-3; P24 F-2, SQ-1).
    #[test]
    fn m2_fixes_spec_explain() {
        group_holds(SPEC, "spec-explain");
    }

    /// The spec's answers for G3's fixes (P11, P27 B1, B2, B4, B5, NB-6, NB-7, P30) and the
    /// interim cap past 20 000 day combinations (P28, #174).
    #[test]
    fn m2_fixes_spec_windows() {
        group_holds(SPEC, "spec-windows");
    }
}
