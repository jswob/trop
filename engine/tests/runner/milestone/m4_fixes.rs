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
    const GROUPS: [&str; 10] = [
        "fixes-g1",
        "fixes-g2",
        "fixes-g2fix",
        "fixes-g3",
        "fixes-p12",
        "spec-g1",
        "spec-g2",
        "spec-g2fix",
        "spec-g3",
        "spec-p12",
    ];

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

    /// G2 `paper`'s review (`tmp/m4/review-g2.md`), refcore's bytes for each probe's asks:
    /// B1, a bare key pinned on a key input or def, whose keyed check prints its difference
    /// and whose key arithmetic answers (`b1_pinkey`, `b1b`, `b15_pinkey2`, `a8_more`; L §4.2,
    /// §5.18, §3.2 Inline figures); B3 across indexes, two `by 3` month keys differing by
    /// months (`g3_min`, `a10_steps` `c1`); B4, a four-digit figure pinned on a `year` is the
    /// year, on a date or month `LOAD-UNIT` (`b11_yearmin`, `b4_date`, `b13_ratety`); the
    /// crossings both engines answer alike, a bare figure on `num %` among them; and, held for
    /// #255, B3's step counts within one index, B4's `LOAD-KEY` on a year-index key, B5's
    /// open-index difference and B6's `num %/year` pin.
    #[test]
    fn m4_fixes_g2() {
        group_holds(VECTORS, "fixes-g2");
    }

    /// G2's fix review (`tmp/m4/review-g2fix.md`) under picks P13 to P16, refcore's bytes:
    /// a four-digit pin on a key target outside its index, or of another period, is refused
    /// `LOAD-KEY` as the same row is (P16 F2: `p7_yk_out`, `p5_b4`, `r3_y4_mk`, `r3_y4_dk`;
    /// the rows `p6_row_*`); keys of a plain day index subtract (`r2_dayidx`); `b + (a - b)`
    /// is `a` on a `by 3` index (`r1_by3`, P15); `n = 0` with a known day refuses `RUN-ZERO`
    /// (`q5_zero`); a known `never` alone in `add_business` and `is_business`; held for #255,
    /// refcore's hole beside a `never` day or `n = 0`, its `RUN-DOMAIN` for `never`, its
    /// `with mk = 2026-08` on a `by 3` key, and its `LOAD-KEY` on `p4_yk`'s in-index pins.
    #[test]
    fn m4_fixes_g2fix() {
        group_holds(VECTORS, "fixes-g2fix");
    }

    /// G3 `walked`'s review (`tmp/m4/review-g3.md`, pick P18), refcore's bytes for `explain`
    /// of a decision's `flows` or `count`: B2, a hole running read prints its question
    /// (`f6_holerun` `e_wb`, `f11_window` `e_cu`); B3, under a `with` axis pin (`f7_axpin`);
    /// B4, two decisions refused `RUN-ORDER` print no body (`f12_order`, `f11_runorder`); B5
    /// and non-blocking 1, `count` and `sum` over transfers with a decision member and a
    /// `where` (`f13c_count_where`, `f13_bind`, `f13b_bind_plan`); the `inputs-of` asks;
    /// held for #255, B1's explains, where refcore lists runs outside the cell's key or span
    /// or another move's.
    #[test]
    fn m4_fixes_g3() {
        group_holds(VECTORS, "fixes-g3");
    }

    /// P12, the one statement for built-ins (`tmp/m4/auto-picks.md`; G1's fix review F1,
    /// `tmp/m4/review-g1fix.md`): refcore's bytes for each cell of `m4/fixes/p12/`'s
    /// `accrue`, `periodic`, `factor`, `round`, `dcf`, `add_business`, `last_business`,
    /// `schedule` and `is_business` (a hole in each argument, alone, with another hole, or
    /// beside each other argument's refusal or a `never` day; two refusals) where refcore
    /// gives the text's answer, a `none` calendar read at a day the line doesn't know among
    /// them (P13: the hole); F1, a hole compounding code in `periodic` is the hole (`F1_min`,
    /// `F1_if`, `c1_codehole`, `periodic` `h_c`); held for #255, the cells where refcore is
    /// hole-first.
    #[test]
    fn m4_fixes_p12() {
        group_holds(VECTORS, "fixes-p12");
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

    /// G2's review, the spec's answers: B2, a `none` side of a keyed check prints no
    /// difference (L §5.18, §3.1; written, where refcore crashes); and for #255, B3, keys of a
    /// range `by n` differ by its periods (L §4.2, from twins without `by n`); B4, the key
    /// 2024 of a year index (a twin pinning it through an input); B5, an open index's keys
    /// print no difference (L §5.6, written); B6, `with g = 5` on `num %/year` is
    /// `0.05 1/year` (L §3.2, a twin pinning `5 %/year`). The written ones were checked by hand.
    #[test]
    fn m4_fixes_spec_g2() {
        group_holds(SPEC, "spec-g2");
    }

    /// P12's answers (#255): a built-in's known argument that alone forces a refusal (`none`,
    /// a monthly rate, `discount`, `nominal 0`, a step not positive, `n = 0`, a `never` day)
    /// refuses it before any hole (L §8; from twins stating the hole; `RUN-DIV`, a `none`
    /// roll, `RUN-NONE never in …` (P16) and `RUN-ZERO` with the hole printed (P14) written);
    /// of two refusals, the first argument in written order prints (#255 item 20; twins
    /// stating the second argument good).
    #[test]
    fn m4_fixes_spec_p12() {
        group_holds(SPEC, "spec-p12");
    }

    /// G2's fix review, the spec's answers (#255): a known `never` day refuses `accrue`,
    /// `dcf` and `add_business` beside a hole, `RUN-NONE never in …` (P16 F1, L §3.1);
    /// `add_business(d, 0, cal)` with d a hole refuses `RUN-ZERO`, the hole printed (P14);
    /// `with mk = 2026-08` on a `by 3` key is refused `LOAD-KEY` as its row is (P16 F2);
    /// `pos(yk)` and the key 2024 on a year index (a twin pinning keys through inputs). The
    /// written ones were checked by hand.
    #[test]
    fn m4_fixes_spec_g2fix() {
        group_holds(SPEC, "spec-g2fix");
    }

    /// G3's review, the spec's answers (#255, P18): B1, `explain` of a decision's `flows` or
    /// `count` lists L §8.2's one read set, the days that decided a movement doesn't count but
    /// no run outside the cell's key or span and no other move's runs (`f4_min`, `f3_span`,
    /// `f6_holerun`, `f11_window`); with B2, the hole running read's question. Written by
    /// hand from refcore's and G3's build's lines.
    #[test]
    fn m4_fixes_spec_g3() {
        group_holds(SPEC, "spec-g3");
    }
}
