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
    const GROUPS: [&str; 21] = [
        "fixes-g1",
        "fixes-g2",
        "fixes-g2fix",
        "fixes-g3",
        "fixes-g3fix",
        "fixes-g6",
        "fixes-g6c",
        "fixes-narrow",
        "fixes-p12",
        "fixes-p21",
        "fixes-p22",
        "fixes-p23",
        "fixes-traced",
        "spec-g1",
        "spec-g2",
        "spec-g2fix",
        "spec-g3",
        "spec-g3fix",
        "spec-p12",
        "spec-p21",
        "spec-p22",
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

    /// G3's fix review (`tmp/m4/review-g3fix.md`) under pick P20, the read set of a decision's
    /// `flows` or `count`, refcore's bytes: a `count` lists a false-`when` member's reads (F2:
    /// `c6_count_false_when`, `c5_dec_false`, a plan's in `c7_count_plan_false`); a refused
    /// order lists each tested instance's `when` and amount (F1: `RUN-ORDER` in
    /// `o1_order_inputs`, `c1_false_when`, `RUN-TIE` in `t1_tie`, `t2_tie_inputs`); a plan whose
    /// `when` is false, through `explain flows`, answers (`c2_plan_false`, `c3`, `c4`); held for
    /// #255, the `inputs-of` lists that leave those reads out and the explains that list
    /// another move's runs.
    #[test]
    fn m4_fixes_g3fix() {
        group_holds(VECTORS, "fixes-g3fix");
    }

    /// Pick P21 (G3's last fix's STOP-REPORT): M3's whole run `t221-gd/216-o0` is held for #255
    /// for its `i2` (a refused member's reads, P20), so its other asks gate here one by one,
    /// refcore's bytes (`m3/fixes/t221/o216/o0.tally`: `f`, `ra`, `r1` to `r4`, `i1`).
    #[test]
    fn m4_fixes_p21() {
        group_holds(VECTORS, "fixes-p21");
    }

    /// The performance groups' guard (U1, `tmp/m4/perf-plan.md` §5 items 1 and 4), refcore's
    /// bytes, which no G6 group may move: a register that grows month by month through a
    /// decision's nested balance reads, printed in code-point order where a key seen later
    /// sorts first (`z10[…]` before `z1[…]` before `z_1[…]`, `aa[…]` first; `held:` past the
    /// cut; `r1_register`); `with uncertain` across a projection's window, an uncertain
    /// movement on its first and last days, the days beside them, one fulfilled, one with a
    /// hole day (`u1_uncertain`).
    #[test]
    fn m4_fixes_g6() {
        group_holds(VECTORS, "fixes-g6");
    }

    /// Fence F7's own realities (`m4/narrow/`, perf plan §5 items 2 and 3; the CLI's
    /// `narrow_oracle` runs them under `Plans::Narrow` and `Plans::Whole`), refcore's bytes:
    /// `o.fulfils` read at a past booking cut while another instance of the named move refuses,
    /// is a hole or has a hole day, and where the named instance itself does (`n1_fulfils`);
    /// `flows(mv, span)` read so, on a span's first and last days, a window day inside a span
    /// and a missing day that may fall in any (`n2_flows`); projections whose window holds or
    /// excludes a hole-day instance and instances on its first and last days (`n3_window`).
    #[test]
    fn m4_fixes_narrow() {
        group_holds(VECTORS, "fixes-narrow");
    }

    /// G6c's review B1 (`tmp/m4/review-g6c.md`), refcore's bytes: the links an observation names
    /// at a past cut give their reads to the reader that reads them, whichever instance first
    /// enters the narrow read there (`p09b_pin` `zp`: no `pinned:` line; `p09_first_min` `i`: no
    /// `v:ok[2026-01]`; L §8.2 One read set, R-130).
    #[test]
    fn m4_fixes_g6c() {
        group_holds(VECTORS, "fixes-g6c");
    }

    /// P22's refcore bytes, every one held for #255 (so the group holds none today): `n4_own`'s
    /// asks, where refcore's `o.fulfils` builds the named move's whole list with its amounts, and
    /// `n2_flows`' `e_jan` and `i_may`, a plan's `flows` read set; the text's answers are
    /// `spec-p22`'s.
    #[test]
    fn m4_fixes_p22() {
        group_holds(VECTORS, "fixes-p22");
    }

    /// Picks P23 and P24, G4 `backlog`'s: refcore's bytes for `explain` under an axis `*`
    /// (`fixes/g3/f5_cross` `e_pr`, `RUN-DIV`) and under a hole axis (`fixes/g3fix/
    /// h2_plan_hole_axis` `e_cw`, `e_sm`), which stop "not implemented", and a `given` row marked
    /// `assumed` printing its `assumed:` line (`fixes/g6b/g1_given_assumed` `q`; L §8.2).
    #[test]
    fn m4_fixes_p23() {
        group_holds(VECTORS, "fixes-p23");
    }

    /// F7's traced read the engine doesn't answer yet, apart so `fixes-narrow` gates green:
    /// `explain` of a `flows` whose plan reads `o.fulfils` at its day's cut, every instance's
    /// reads listed (`n1_fulfils` `e_feb`; the narrow read declines under a trace), which stops
    /// "not implemented" on master's and build/m4's engines; refcore's bytes.
    #[test]
    fn m4_fixes_traced() {
        group_holds(VECTORS, "fixes-traced");
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

    /// P20's answers (#255): `explain` and `inputs-of` of a decision's `flows` or `count` list
    /// one set, each due instance at the cell's key and in its span with every `when` it
    /// tested and amount it read, a refused order's tested instances, nothing at another key
    /// or of another move (`c1_false_when`, `c5_dec_false`, `c6_count_false_when`,
    /// `c7_count_plan_false`, `o1_order_inputs`, `t1_tie`, `t2_tie_inputs`). Written by hand
    /// from refcore's and G3's fix's lines; `f2_keyed`'s and `f7_axpin`'s are in `spec-g3`.
    #[test]
    fn m4_fixes_spec_g3fix() {
        group_holds(SPEC, "spec-g3fix");
    }

    /// P21's answer (#255): M1's `fixes/g12/r_p_mr3_z2`, held, lists the false-`when` run's
    /// balance line `unfulfilled: rent[2026-01]` beside refcore's `rent[2026-02]` (P20; L §8.2,
    /// E §10), written by hand.
    #[test]
    fn m4_fixes_spec_p21() {
        group_holds(SPEC, "spec-p21");
    }

    /// P22's answers (#255): a link reads only its instance's existence (L §5.19), so a plan
    /// reading `o.fulfils` of rows naming its own instances reads none of their amounts
    /// (`n4_own` `k`, `f`, `l`, from refcore's run of a twin reading the rows by their place);
    /// a plan's `flows` read set is P20's (`n2_flows` `e_jan`, `i_may`, written from build/m4's
    /// lines at 00143f5a and checked by hand).
    #[test]
    fn m4_fixes_spec_p22() {
        group_holds(SPEC, "spec-p22");
    }
}
