//! Milestone M3's backlog vectors for the builder's last group (`MILESTONE-M3.md` row 9; pick
//! P7, T's plan Q9): household X's `net_worth` binding on a small model (`m3/fixes/`), the
//! places over every key, not outside, of four kinds, in PLN by `unit(x)`, and `unit()` of a
//! quantity unit's place (`RUN-DOMAIN`). Each held to refcore's run (`m3/runs.vectors`, group
//! `last`), byte for byte. `test = false` until M3 is tagged (`engine/tools/milestones.toml`).

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
}
