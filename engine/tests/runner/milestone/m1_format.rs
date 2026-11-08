//! Milestone M1's format law on the engine (issue #108, #109; `MILESTONE-M1.md` row 7): a
//! canonical text reprints as itself (L §2, "one canonical form"), on the texts the format
//! fuzz target found where it doesn't. The expected value is the law, not refcore's bytes:
//! refcore breaks it on the same text (#111, M2's round). The coordinator's pick P3 orders
//! row keys of different literal kinds by kind first, then by value.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally_runner::m1::{KEY_KINDS, engine_fixed_point};

    /// #109: rows of an input keyed by days, keyed by two days, a day range and the integer
    /// `20240`. Its first reprint puts the integer's row third; reprinting that output moved
    /// it first (engine and refcore alike, 2026-11-08), so the canonical form wasn't a fixed
    /// point. For G1.
    #[test]
    fn m1_format_keys_of_two_kinds_reprint_as_themselves() {
        let (path, text) = KEY_KINDS;
        let result = engine_fixed_point(path, text);
        assert!(
            result.is_ok(),
            "{}",
            result.as_ref().err().map_or("", String::as_str)
        );
    }
}
