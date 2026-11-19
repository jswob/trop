//! Milestone M3's backlog vectors for the builder's last group (`MILESTONE-M3.md` row 9; pick
//! P7, T's plan Q9): household X's `net_worth` binding on a small model (`m3/fixes/`), the
//! places over every key, not outside, of four kinds, in PLN by `unit(x)`, and `unit()` of a
//! quantity unit's place (`RUN-DOMAIN`); the G3 review's fixes as runs (`m3/fixes/g3/`); and
//! #192's standing items (`m3/fixes/b192/`). Each held to refcore's run (`m3/runs.vectors`,
//! groups `last`, `fixes` and `b192`), byte for byte.
//! `test = false` until M3 is tagged (`engine/tools/milestones.toml`).

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally_runner::m3::{RUNS, group_failures};

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
    /// projection read on falling days within a month across a refused leg and a hole.
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
}
