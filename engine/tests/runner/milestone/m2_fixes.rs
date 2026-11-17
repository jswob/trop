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
    const GROUPS: [&str; 16] = [
        "fixes-forms",
        "fixes-explain",
        "fixes-windows",
        "fixes-sweep",
        "fixes-records",
        "fixes-ledger",
        "fixes-r11",
        "fixes-r12",
        "fixes-r13",
        "fixes-r14",
        "fixes-r15",
        "fixes-r16",
        "fixes-s",
        "fixes-d",
        "fixes-cut",
        "fixes-rv",
    ];

    /// The groups of `m2/spec.vectors`, one `#[test]` each.
    const SPEC_GROUPS: [&str; 3] = ["spec-forms", "spec-records", "spec-sweep"];

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
    /// and a four-digit literal where money is expected (P16), held for #169; G6's partial
    /// answers refcore prints linear (`pa1` q6, q7) and an observed row's `none` field (`obsrow`
    /// o5), held for #169; G6's last part: B1 and B2 (`same` rows under `windows each`, for the
    /// sweep's fix), B3 and B4, P29 and P30 held for #169, a set split by sub-window for #186.
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
    /// (P51, #177; `adj_amt`'s settlement and its adjustment's remainder); G5's review probes
    /// (`m2/fixes/g5/`): `by f`, unit `any`, the made-exist leg, the answered hole, bindings
    /// under `with uncertain` and over hole days, the prefix walk's pair (B2).
    #[test]
    fn m2_fixes_ledger() {
        group_holds(VECTORS, "fixes-ledger");
    }

    /// tally 2.1.8's R11 (#195): a refused movement's refusal stops at a later reading of its
    /// place, as a hole does, and a refusal met on the way to a hole wins (`m2/fixes/r11/`,
    /// plan §2; each vector named by its case, refcore's bytes).
    #[test]
    fn m2_fixes_tally_218_r11() {
        group_holds(VECTORS, "fixes-r11");
    }

    /// tally 2.1.8's R12 (#196): what a version states besides its rows (`covers`, gap-role,
    /// `moves` and `same` lines, a removal, a link) has its version's cut and world
    /// (`m2/fixes/r12/`).
    #[test]
    fn m2_fixes_tally_218_r12() {
        group_holds(VECTORS, "fixes-r12");
    }

    /// tally 2.1.8's R13 (#197): the ask clauses (`for`, the axes, `windows each`, `in`, `as
    /// known`, `partial`) reach every form; `windows each` splits only the windows the answer
    /// reads (`m2/fixes/r13/`).
    #[test]
    fn m2_fixes_tally_218_r13() {
        group_holds(VECTORS, "fixes-r13");
    }

    /// tally 2.1.8's R14 (#198): at a place of unit `any` a reading, its gap, pad and ledger
    /// are the unit's (`m2/fixes/r14/`).
    #[test]
    fn m2_fixes_tally_218_r14() {
        group_holds(VECTORS, "fixes-r14");
    }

    /// tally 2.1.8's R15 (#199): under `partial` a hole stays a linear form only through what
    /// adds and scales (`m2/fixes/r15/`).
    #[test]
    fn m2_fixes_tally_218_r15() {
        group_holds(VECTORS, "fixes-r15");
    }

    /// tally 2.1.8's R16 (#200): `modelled`, a reading `after m`, the keyed `quantile` and a
    /// projection's legs as their definitions say (`m2/fixes/r16/`).
    #[test]
    fn m2_fixes_tally_218_r16() {
        group_holds(VECTORS, "fixes-r16");
    }

    /// tally 2.1.8's S1 to S9, the singletons (`m2/fixes/s<n>/`): S1's reach over `count` and
    /// `sum`, S7's seen and unseen limit among them.
    #[test]
    fn m2_fixes_tally_218_s() {
        group_holds(VECTORS, "fixes-s");
    }

    /// tally 2.1.8's follow-up items (`m2/fixes/d<n>/`), item 77's faces among them.
    #[test]
    fn m2_fixes_tally_218_d() {
        group_holds(VECTORS, "fixes-d");
    }

    /// tally 2.1.8's cut (plan §13): a version's `same` of a row and an instance is
    /// `LOAD-SAME`, two rows or two instances still load (`m2/fixes/cut/`).
    #[test]
    fn m2_fixes_tally_218_cut() {
        group_holds(VECTORS, "fixes-cut");
    }

    /// One vector per blocking finding of the round's reviews, from its input, and the rulings
    /// no finding's input holds, from refcore's own test of the ruling (`m2/fixes/rv/`).
    #[test]
    fn m2_fixes_tally_218_reviews() {
        group_holds(VECTORS, "fixes-rv");
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

    /// The spec's answers for G6's held probes (L §5.18 partial answers, L §8.4 an observed
    /// row's `none` field) and for P60's census forms refcore can't run: an ask's `law`,
    /// `as known` and `with` formed per its own `for`'s binding tuple (L §5.18, §5.11); G6's
    /// last part: a removal known in a window holding the cut (B3), a `same` line before its
    /// version is known (B4), P29's and P30's lines.
    #[test]
    fn m2_fixes_spec_sweep() {
        group_holds(SPEC, "spec-sweep");
    }
}
