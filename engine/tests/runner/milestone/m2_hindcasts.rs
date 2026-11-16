//! Milestone M2's hindcasts (DONE gate 4; issue #167; `MILESTONE-M2.md` row 3;
//! `engine/spec/verification/hindcast.md`): a 60-month synthetic ledger made by a known model
//! M\*, one cause planted per fixture (`m2/hindcast/gen.py`), run under the modeller's M, each
//! fixture's residual ledgers (E §6.3) at K = `on` (the replay) and `as known` month 48 (the
//! backtest) in `m2/vectors.refcore`, refcore's bytes, which `m2/hindcast/judge.py` holds to
//! the answer key. The engine is held to them byte for byte. One test per fixture, so a
//! fixture's whole ledger turns green at once; every failure of a fixture is collected
//! before it fails. `test = false` until M2's marker.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use tally_runner::engine::run_args;
    use tally_runner::held::is_held;
    use tally_runner::kernel::{difference, vectors};
    use tally_runner::m2::VECTORS;

    /// The fixtures this target tests, one `#[test]` each.
    const GROUPS: [&str; 8] = [
        "hindcasts-h0",
        "hindcasts-h1",
        "hindcasts-h2",
        "hindcasts-h3",
        "hindcasts-h4",
        "hindcasts-h5",
        "hindcasts-h8",
        "hindcasts-h9",
    ];

    /// Runs every vector of `group` on the engine and holds it to refcore's run.
    fn fixture_holds(group: &str) {
        let all = vectors(VECTORS).expect("m2/vectors.refcore reads");
        let mine: Vec<_> = all.iter().filter(|v| v.group == group).collect();
        assert!(!mine.is_empty(), "the fixture {group} has vectors");
        let failed: Vec<String> = mine
            .iter()
            .filter(|v| !is_held("m2_hindcasts", &v.id))
            .filter_map(|v| {
                difference(&v.refcore, &run_args(&v.args, &v.files))
                    .map(|d| format!("{}: {d}", v.id))
            })
            .collect();
        assert!(
            failed.is_empty(),
            "{} of {} vectors of {group} differ from refcore:\n{}",
            failed.len(),
            mine.len(),
            failed.join("\n")
        );
    }

    #[test]
    fn m2_hindcasts_every_fixture_is_tested() {
        let named: BTreeSet<String> = vectors(VECTORS)
            .expect("m2/vectors.refcore reads")
            .into_iter()
            .map(|v| v.group)
            .filter(|g| g.starts_with("hindcasts-"))
            .collect();
        let tested: BTreeSet<String> = GROUPS.iter().map(|g| (*g).to_owned()).collect();
        assert_eq!(
            named, tested,
            "each hindcasts- group of m2/vectors.tsv has its test here"
        );
    }

    /// H0, the base: no planted line in either ledger; its `as known` K reads (`mask_bal`,
    /// `mask_net`) are H8's byte for byte (masking at K).
    #[test]
    fn m2_hindcasts_h0() {
        fixture_holds("hindcasts-h0");
    }

    /// H1, A's gross +5 % from month 30: nothing in the replay (the law reads the payslips);
    /// matched pay lines +364.50 in the backtests, from K (month 48) and from month 30.
    #[test]
    fn m2_hindcasts_h1() {
        fixture_holds("hindcasts-h1");
    }

    /// H2, a car repair in month 41 that fulfils nothing: one unexplained line.
    #[test]
    fn m2_hindcasts_h2() {
        fixture_holds("hindcasts-h2");
    }

    /// H3, M's tax rounds down where M\* rounds half-up: each pay's matched line -1.00 in
    /// every month, to the grosz (the three-way split's one law line).
    #[test]
    fn m2_hindcasts_h3() {
        fixture_holds("hindcasts-h3");
    }

    /// H4, rent on the 3rd from month 20: matched rent lines, timing +2 days, amount 0.00.
    #[test]
    fn m2_hindcasts_h4() {
        fixture_holds("hindcasts-h4");
    }

    /// H5, the subscription cancelled after month 45: expected but missing from month 46.
    #[test]
    fn m2_hindcasts_h5() {
        fixture_holds("hindcasts-h5");
    }

    /// H8, B's gross +4 % from month 52, after K: in the backtest only, +215.00 from month
    /// 52; its `as known` K reads equal H0's.
    #[test]
    fn m2_hindcasts_h8() {
        fixture_holds("hindcasts-h8");
    }

    /// H9, H1 and H4 at once: each in its own lines, as when planted alone.
    #[test]
    fn m2_hindcasts_h9() {
        fixture_holds("hindcasts-h9");
    }
}
