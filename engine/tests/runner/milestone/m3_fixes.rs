//! Milestone M3's backlog vectors for the builder's last group (`MILESTONE-M3.md` row 9; pick
//! P7, T's plan Q9): household X's `net_worth` binding on a small model (`m3/fixes/`), the
//! places over every key, not outside, of four kinds, in PLN by `unit(x)`, and `unit()` of a
//! quantity unit's place (`RUN-DOMAIN`); the G3 review's fixes as runs (`m3/fixes/g3/`); and
//! #192's standing items (`m3/fixes/b192/`). Each held to refcore's run (`m3/runs.vectors`,
//! groups `last`, `fixes` and `b192`), byte for byte. The M3 gate's fixes (chunk C7,
//! `m3/fixes/lawdate/`, `m3/fixes/alone/`): a move's law-dated reads (#235) where refcore
//! answers (group `lawdate`), and the spec's answers where refcore prints its guard or, for a
//! scenario file's ask refused alone, nothing (`m3/spec.vectors`, groups `spec-lawdate` and
//! `spec-alone`; refcore's bytes beside each row in `m3/spec.tsv`).
//! Gating since M3's marker (`engine/tools/milestones.toml`).

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally_runner::m3::{RUNS, SPEC, group_failures};

    /// Every vector of `group` in `store` but the held ones prints its stored bytes.
    fn group_holds(store: &[u8], group: &str) {
        let (failed, of) =
            group_failures((store, "m3_fixes"), group, (0, 1)).expect("the store reads");
        assert!(
            failed.is_empty(),
            "{} of {of} vectors of {group} fail:\n{}",
            failed.len(),
            failed.join("\n")
        );
    }

    /// A move's law-dated reads where refcore answers (the M3 gate's B-1, B-2, the fix review's
    /// probes): a calendar read by a business-day function and a def with its own `at` need no
    /// law date, with or without the move's `at`; a law-dated read in a move with no `at` is
    /// `LOAD-LAWDATE … states no at` (L §4.2, §4.3).
    #[test]
    fn m3_fixes_lawdate() {
        group_holds(RUNS, "lawdate");
    }

    /// The spec's answers for #235 (refcore prints its guard): a law-dated read in a key's `on`
    /// before the `at` is `LOAD-LAWDATE` at load (B-3, G-1 B, P28); a key's `where` waiting for its
    /// tuple's `at` answers as its law-free twin (P30, P31, B-3, G-1 C and D); a calendar's
    /// law-dated key in a move with no `at` is `LOAD-LAWDATE` (G-1 A). The gate review's items 1
    /// and 15 are held for #235.
    #[test]
    fn m3_fixes_spec_lawdate() {
        group_holds(SPEC, "spec-lawdate");
    }

    /// The spec's answer for #229 (B-4): `run` prints a scenario file's ask refused alone in its
    /// place, as `check` does (OPERATIONS §2.1, §2.3); refcore's run prints nothing for it.
    #[test]
    fn m3_fixes_spec_alone() {
        group_holds(SPEC, "spec-alone");
    }

    #[test]
    fn m3_fixes_last() {
        let (failed, of) =
            group_failures((RUNS, "m3_fixes"), "last", (0, 1)).expect("m3/runs.vectors reads");
        assert!(
            failed.is_empty(),
            "{} of {of} vectors of last fail:\n{}",
            failed.len(),
            failed.join("\n")
        );
    }

    /// The G3 review's blocking findings as runs (`m3/fixes/g3/`, group `fixes`): a scenario
    /// over a refused one (B-1, held for #232 but its control), a member's ask (B-2, B-3) and
    /// `REQ-CLOCK` (B-4, B-5), held for #233 while refcore prints the old engine's lines; and
    /// G1 review A-4's probe (`m3/fixes/a4/`): a walk-bound def read by an unordered cycle's
    /// `when`s, kept at the act's point before each instance's own.
    #[test]
    fn m3_fixes_review() {
        let (failed, of) =
            group_failures((RUNS, "m3_fixes"), "fixes", (0, 1)).expect("m3/runs.vectors reads");
        assert!(
            failed.is_empty(),
            "{} of {of} vectors of fixes fail:\n{}",
            failed.len(),
            failed.join("\n")
        );
    }

    /// #192's standing items for T (`m3/fixes/b192/`, group `b192`; T's plan §5): G1's fix
    /// N-c, `solve` over a rate and over a num with a unit; the review of P73's fix, one
    /// projection read on falling days within a month across a refused leg and a hole; and
    /// the builder's plan's rows 22, 40, 50, 51 and 52 (chunk C7): a calendar's hole, pads
    /// across a continued walk, a settlement listed inside a walk up to a leg's day, `explain`
    /// stopping at an answer, `HOLE-SAME` over two legs at one place (its residual line held
    /// for #192).
    #[test]
    fn m3_fixes_backlog() {
        let (failed, of) =
            group_failures((RUNS, "m3_fixes"), "b192", (0, 1)).expect("m3/runs.vectors reads");
        assert!(
            failed.is_empty(),
            "{} of {of} vectors of b192 fail:\n{}",
            failed.len(),
            failed.join("\n")
        );
    }

    /// tally 2.2.1's guards (#241, T's plan §1): answers the round leaves as they are, which
    /// its window, register, naming and run-line changes must not move (S-208.3, 208.4, 208.6,
    /// S-216, 228.Q12, S-231.Q4, S-window-key's day key, the sign rule, 235.8, R18's cell 1,
    /// 231.Q6, S-230.Q5b, S-233.5, 229.2, 221.10, S-233.8's twin, 233.c's load words).
    #[test]
    fn m3_fixes_t221_guards() {
        group_holds(RUNS, "t221-gd");
    }

    /// tally 2.2.1's builder group G1 (#241): the window group's vectors, refcore's bytes;
    /// 221.C3 names both windows since the second pin (R21, the names: G3).
    #[test]
    fn m3_fixes_t221_g1() {
        group_holds(RUNS, "t221-g1");
    }

    /// tally 2.2.1's builder group G2 (#241): the register, `same` and a projection (206.1,
    /// 208.2, 208.5, 208.10, 219.1, 219.2, 220.2, 221.1, 221.5, 231.Q1b, 231.R2, 237.1), the
    /// "both" rows the writer's W6 mended, refcore's bytes.
    #[test]
    fn m3_fixes_t221_g2() {
        group_holds(RUNS, "t221-g2");
    }

    /// tally 2.2.1's refcore-alone vectors (#241): refcore's mends, which the engine prints on
    /// arrival; 221.10's `mean` and `quantile` over a window term take it as the hole (P27).
    #[test]
    fn m3_fixes_t221_refcore() {
        group_holds(RUNS, "t221-rc");
    }
}
