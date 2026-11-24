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
    const GROUPS: [&str; 40] = [
        "fixes-g1",
        "fixes-g2",
        "fixes-g2fix",
        "fixes-g3",
        "fixes-g3fix",
        "fixes-g4",
        "fixes-g6",
        "fixes-g6c",
        "fixes-g6cfix",
        "fixes-gate",
        "fixes-gate8",
        "fixes-gatefix",
        "fixes-gatelead",
        "fixes-narrow",
        "fixes-p12",
        "fixes-p21",
        "fixes-p22",
        "fixes-p23",
        "fixes-p30",
        "fixes-p32",
        "fixes-p40",
        "fixes-p40fix",
        "fixes-p43",
        "fixes-t231g1",
        "fixes-t231g2",
        "fixes-t231rv1",
        "fixes-traced",
        "spec-g1",
        "spec-g2",
        "spec-g2fix",
        "spec-g3",
        "spec-g3fix",
        "spec-gate",
        "spec-p12",
        "spec-p21",
        "spec-p22",
        "spec-p26",
        "spec-p31",
        "spec-p40fix",
        "spec-p43",
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

    /// G4 `backlog`'s review X1, pick P28 (`tmp/m4/review-g4.md`), refcore's bytes: a `given`
    /// row in a version that governs from 2026 on, read at a law date after the cut, prints its
    /// `held:` line as an `input` there does (`m4/fixes/g4/h1_held` `q_late`, and `q_x`, the
    /// input; L §8.2's `held:` row).
    #[test]
    fn m4_fixes_g4() {
        group_holds(VECTORS, "fixes-g4");
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

    /// The widening's fix review (`tmp/m4/review-g6cfix.md`), X1's corners where refcore answers
    /// and reads no link at the ask's cut that names the reader's own instance: an amount that
    /// refuses (`RUN-NEG`) or is a hole, a day that is a hole, a `when` false (`RUN-RECORD`) or a
    /// hole (`x1a_amounts`, `x1b_when` `k`, `f`, `l`, `n`); the control reading the rows by their
    /// place (`x1e_nofulfils`); `loose` alone at the cut and at months no link names
    /// (`x1d_selfk_min` `l`, `x1g_order` `l1`, `l3`).
    #[test]
    fn m4_fixes_g6cfix() {
        group_holds(VECTORS, "fixes-g6cfix");
    }

    /// P30's refcore bytes, every one held for #255 (so the group holds none today): `o.fulfils`
    /// naming the reading move's own instance at the ask's cut, where refcore builds the move's
    /// plan and stops `ENGINE-INVARIANT` (`RUN-CYCLE` for `x1d_selfk_min` `f`), alone and in both
    /// ask orders (`x1h_order_min`, `x1g_order`, `x1d_selfk_min`); the statement's answers are
    /// `spec-p30`'s.
    #[test]
    fn m4_fixes_p30() {
        group_holds(VECTORS, "fixes-p30");
    }

    /// P32, the coordinator's probes on P30 (`m4/fixes/p32/`), refcore's bytes: an observation
    /// naming its own reader, whose day reads the reading cell (`e_selfcycle`, held for #255:
    /// refcore answers `loose[2026-04]` 0.00 PLN where L §5.11 refuses the cycle); a `when`
    /// reading the cell (`b_whencycle`: `l4`, and `l24`, `f` held, where refcore refuses with its
    /// old list guard); a `when` false on the named instance (`c_nowhen`, `RUN-RECORD`); a pin of
    /// the day by `with` and `as if` (`d_pin` `fw`, `fa`). The text's lines are `spec-p32`'s.
    #[test]
    fn m4_fixes_p32() {
        group_holds(VECTORS, "fixes-p32");
    }

    /// The gate review's probes (`tmp/m4/gate-review.md`, `m4/fixes/gate/`), refcore's bytes:
    /// B1, a rate literal pinned on a `rate` by `with` and `as if` (`c02_ratepin`, `p_c1` `p16`,
    /// `s01_solve` `s8`); B2, an untaken or empty term holding `round` with a read mode,
    /// `accrue` or a per-tuple pin (`B2_empty_sum_min`, `o03_bare`, `p_s1` `q5`, `q6`, `q12`);
    /// X1, `solve`'s centre to a minor unit (`s02_solve_minor` `a2`); X3, `explain` above back
    /// pay (`p_X3_explain_backpay_min` `z2` to `z5`), and `p_g5x`'s three stops, held for #255
    /// (P31: refcore lists swap's whole list; the text's lines are `spec-gate`'s).
    #[test]
    fn m4_fixes_gate() {
        group_holds(VECTORS, "fixes-gate");
    }

    /// The gate fix's review (`tmp/m4/review-gatefix.md`, `m4/fixes/gatefix/`), refcore's bytes:
    /// G1, `solve` through a money def at step 3's centre (`x1_min`, `x1_grossup2`, `r4` `h5`,
    /// `s3` `i3`; P36); G2 and G3, an empty `sum` or a bare branch beside a leg's `flows`, a keyed
    /// place's `bal`, a band read, `band_lo`, a place of `any` unit, `pow`, `accrue` of a band
    /// (`b_legflows*`, `b_keyedbal*`, `b_roundband`, `b_bandif`, `t1_*`, `t2_*`), each 0 in its
    /// term's load type (P36); `b_bindbal`, a place bound by `for p in accounts` (a bare 0).
    #[test]
    fn m4_fixes_gatefix() {
        group_holds(VECTORS, "fixes-gatefix");
    }

    /// The coordinator's probes on the gate's fixes (`m4/fixes/gatelead/`), refcore's bytes:
    /// P36's bare 0s and empty terms (`p36_t1`, every ask; `a7` held for #255, where refcore
    /// prints `0` for money PLN; `a12`, a rate's bare 0, prints `0` on both), and P37's settled
    /// back pay (`x3_settled` `z3`, `z4`; `z5`, `e5` held for #255, where refcore's `explain`
    /// leaves out the settling movement). The text's lines are `spec-gate`'s.
    #[test]
    fn m4_fixes_gatelead() {
        group_holds(VECTORS, "fixes-gatelead");
    }

    /// Gate 8's survivor sort (`tmp/m4/gate8/survivor-sort.md`, its K rows; `m4/fixes/gate8/`),
    /// refcore's bytes, each an ask that tells a narrow read's code from a surviving mutant:
    /// a list that stops at a refused `where` (`k_stop`), a link's `assumed` `when` absorbed by
    /// the move's first reader (`k_pin`), a hole domain (`k_hole`, and `k_r35` with a version
    /// known before the cut), an uncertain kind (`k_unc`), a refused day (`k_day`, and
    /// `k_dayref` through an existence read), B1's shape at the link's own day (`k_b1`), a
    /// refused tuple's instance named by `o.fulfils` read by its id or in a binding (`k_r60`,
    /// `k_r65`), and `x1r`, every spend tuple refused, whose `l24` refcore refuses with its
    /// list guard (held for #255; the text's lines are `spec-gate8`'s).
    #[test]
    fn m4_fixes_gate8() {
        group_holds(VECTORS, "fixes-gate8");
    }

    /// P40's two cases, refcore's bytes: B1, an existence read takes only what decides that
    /// the instance exists, so no other instance's link's `assumed:` line (`k_link` `qz`, after
    /// `qa` and before it, and `qa` alone; refcore's bytes are the answer, green since P40's fix,
    /// 8f6f4338); B2, a list that stops gives that stop to every read of it (`k_stop3`,
    /// `k_stop2`, each ask alone and in both orders), where refcore prints `ENGINE-INVARIANT`:
    /// held for #255, the text's lines are `spec-p40`'s.
    #[test]
    fn m4_fixes_p40() {
        group_holds(VECTORS, "fixes-p40");
    }

    /// P40's fix and P42's (8f6f4338, b9969e50), the builder's crossings and the coordinator's
    /// probes (`m4/fixes/p40/`), refcore's bytes: a list that stops read nine ways, alone and in
    /// orders (`c1_stop`); existence reads beside a link whose `when` reads an `assumed` row,
    /// open, closed, of a refused tuple and of a decision's instance, and their `explain`, alone
    /// and in both full orders (`c2_link`, `c3_dec`, `c4_when` with pins and a decision's running
    /// `when`, `c5_wref` with a refused `when`); P40's probes (`p40a` a stopping list, `p40b` a
    /// whole one; `e4` left out). Where refcore prints its B2 fault after another ask, its list
    /// guard, P22's whole-list reads or an answer load refuses (P43, `c4_when` `qo`), held for
    /// #255 with the text's lines in `spec-p40fix`; `c4_when` `efo` isn't stored (#256). Since
    /// P43's fix (ca2f2ea2) and P44, eleven `explain` asks of `c2_link`, `c4_when` and `c5_wref`
    /// list the named instance's own `where` row first: held for #255, `spec-p43`'s lines.
    #[test]
    fn m4_fixes_p40fix() {
        group_holds(VECTORS, "fixes-p40fix");
    }

    /// P43's fix (ca2f2ea2): `explain` lists the `where` rows an existence read rests on.
    /// Refcore's bytes for `p40b` `e4`, alone and beside `f`, and the builder's `c6_own` (a
    /// def reading its own move's flows through an existence read); held for #255 where refcore
    /// adds the whole list's rows or refuses with its list guard (`RUN-CYCLE`), the text's lines
    /// in `spec-p43`; `c6_own` `l4`, `c4`, `f3` gate.
    #[test]
    fn m4_fixes_p43() {
        group_holds(VECTORS, "fixes-p43");
    }

    /// Picks P23 and P24, G4 `backlog`'s: refcore's bytes for `explain` under an axis `*`
    /// (`fixes/g3/f5_cross` `e_pr`, `RUN-DIV`, held for #255 since P31: the text's lines are
    /// `spec-p31`'s) and under a hole axis (`fixes/g3fix/h2_plan_hole_axis` `e_cw`, `e_sm`), and
    /// a `given` row marked `assumed` printing its `assumed:` line (`fixes/g6b/g1_given_assumed`
    /// `q`; L §8.2).
    #[test]
    fn m4_fixes_p23() {
        group_holds(VECTORS, "fixes-p23");
    }

    /// F7's traced reads, apart so `fixes-narrow` gated green while the engine stopped them "not
    /// implemented": `explain` of a `flows` whose plan reads `o.fulfils` at its day's cut
    /// (`n1_fulfils` `e_feb`) and of a hole booking's skipped read (`p06_holebook_io` `e1`);
    /// refcore's bytes, which list the named move's whole list, both held for #255 since P31
    /// (the text's lines are `spec-p31`'s).
    #[test]
    fn m4_fixes_traced() {
        group_holds(VECTORS, "fixes-traced");
    }

    /// tally 2.3.1's group 1 (#259; the plan's §2, §8.1, §12, §16 row 1), refcore's bytes at
    /// the first pin's texts (`m4/fixes/t231g1/`, the writer's W1 tests and the critic's
    /// probes): what an operation decides (R1) — `any` and `all` by a deciding term in every
    /// order (`anyall_first`, `anyall`), a check not made on a hole (`chk`, `hd_sign_q`), a
    /// window comparison naming both days (`win2`), step 5's hole beside two candidates
    /// (`a03_conflict_hole`), `same`'s two days as bounds (`same2`), a hole-day decision's legs
    /// the hole by every path (`holeday`, `holeday_b`), an uncertain hole-day movement left out
    /// (`b1e`), a lifespan's end against a hole balance printing the balance's hole (#248 face
    /// (b): `life_amt`, `life_day`, `j3q`); `never` the latest date in ordering (S9, `never`);
    /// `age_months` in the calendar's last month (S23, `age_last`).
    #[test]
    fn m4_fixes_t231g1() {
        group_holds(VECTORS, "fixes-t231g1");
    }

    /// tally 2.3.1's group 2 (#259; the plan's §3, §7, §10, §16 row 2), refcore's bytes at
    /// the second pin's texts (`m4/fixes/t231g2/`): an instance's existence is one cell (R6) —
    /// a refused `where` stands for its tuple alone (`where2`, `k_stop3_nodiv`), the order of
    /// its reads (`w1_day` to `w5_whenhole`, `hdfalse`), a field read of a missing instance
    /// `RUN-NONE` (`x_none`), a tuple reading another tuple of its move (`selfread`), a
    /// decision's link judged at its walk point (`dec_link`, `exist_mark`), a row fulfilling an
    /// instance that doesn't exist (`k_refw`, `RUN-RECORD`); a link's reads only for the readers
    /// that rest on it, a refused link refusing them (item 6: `first`, `l_false`).
    #[test]
    fn m4_fixes_t231g2() {
        group_holds(VECTORS, "fixes-t231g2");
    }

    /// tally 2.3.1 G1's review (`tmp/spec-2.3.1/inputs/review-g1.md`), refcore's bytes where
    /// the rulings give refcore's line (`m4/fixes/t231rv1/`): B1, a refused `where` under
    /// `any`/`all` (`p21_where_min`, `p02_where_refused`); B2 and B3, step 5's hole and its
    /// `explain` beside two candidates (`p20_step5_min`, `p04_step5`); B4, a scenario turning
    /// a `same` pair into equal amounts on two days (`p16`); B5, a decision side of a two-day
    /// `same` (`p18_same_dec`); B6, `explain` of the last day where a reading comes between
    /// (`p09_same_two`); N1, every side of a three-sided `same` (`p10_same_three`). Left out,
    /// for the writer: `p09` `f_a`, `f_b`, `p10` `fb`, `fy` (refcore's `flows` over the two
    /// days) and N5's `p03c` (refcore still refuses where a member decides).
    #[test]
    fn m4_fixes_t231rv1() {
        group_holds(VECTORS, "fixes-t231rv1");
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

    /// P26's answers (#245, pick P3): `RUN-NONE` in L §8.4's form names the read, `<x>[k] is
    /// none here: <arithmetic, ordering, aggregate, …>`, the catalogue's words bare, in the seven
    /// vectors G4's wording changed (`fixes/g1/a12_nonehole/q_arith2`, `fixes/p12/round`'s
    /// `n_x__z_step`, `n_x__g_step`, and P12's answers `spec/p12/round/h_x__n_step`,
    /// `h_step__n_x`, `h_mode__n_x`, `h_mode__n_step`, all held) and C3's four `items/none` asks;
    /// written from build/m4's run (702bdcd3), which differs only in that line.
    #[test]
    fn m4_fixes_spec_p26() {
        group_holds(SPEC, "spec-p26");
    }

    /// P31's answers (#255; P20, P22, P30): `explain` lists one read set (L §8.2), a refused
    /// projection a balance node for its refusal with the main walk's runs that name its place
    /// through the refused one (`fixes/g3/f5_cross` `e_pr`), a binding the `when` it reads
    /// judging a link and an existence read its day's rows, no other instance's
    /// (`fixes/g6c/p06_holebook_io` `e1`, `fixes/narrow/n1_fulfils` `e_feb`). Written from
    /// build/m4's run at 15d512ec (G3's P31 turn), checked by hand.
    #[test]
    fn m4_fixes_spec_p31() {
        group_holds(SPEC, "spec-p31");
    }

    /// The gate's answers (#255, #250), written from build/m4's run at 4b450ec8: `p_g5x`'s
    /// three `explain` stops in P31's read set (their answer lines refcore's); P36, a bare 0
    /// beside money is money (`p36_t1` `a7`, 0.00 PLN); P37, `explain` of a span lists the
    /// movement that settles its adjustment (`x3_settled` `z5`, `e5`); L §4.2, `age_months` in
    /// the calendar's last month is 0 (`items/age` `a_last`, held for #250 since C3).
    #[test]
    fn m4_fixes_spec_gate() {
        group_holds(SPEC, "spec-gate");
    }

    /// The text's lines for P40's crossings (#255), written from build/m4's run at b9969e50:
    /// `RUN-DIV` for every read of a stopping list in every order and `p40b`'s 270.00 PLN, each
    /// ask's block checked equal to that ask run alone (P40); `c4_when` `qo` refused at load
    /// (P43, `LOAD-READS`); `c5_wref` `ew`, `ed`, an existence read's `explain` without the
    /// other instances' reads (P22, P30).
    #[test]
    fn m4_fixes_spec_p40fix() {
        group_holds(SPEC, "spec-p40fix");
    }

    /// P43's and P44's answers (#255), written from build/m4's run at ca2f2ea2: an existence
    /// read's `explain` lists the `where` rows it rests on, the named instance's own first (the
    /// eleven of `fixes-p40fix`); `p40b` `e4` lists `ten`, `dz[2026-01]`, `d25` and r0406, alone
    /// and in either order with `f`; `c6_own`'s 270.00 PLN (P30). Each multi-ask answer's
    /// blocks checked equal to their asks run alone.
    #[test]
    fn m4_fixes_spec_p43() {
        group_holds(SPEC, "spec-p43");
    }
}
