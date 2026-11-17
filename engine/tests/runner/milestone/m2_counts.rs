//! Milestone M2's fence F6 counted (issue #167; gate 8 run 43749994847 on 205157a2, the
//! builder's sort of the fence's survivors, pick P75's addendum; `MILESTONE-M2.md` row 5). The
//! forks print the same lines on and off by design (`m2_forks`), so a mutant that only shares
//! fewer walks, or walks again what a kept walk had walked, is told apart by the work counters
//! alone (R-51, BUILDER §13.2): the counters build's `Answer::counters`, the line `--stats`
//! prints. Each ask of the fixed inputs below runs through `tally_cli::tally::run_with_forks`
//! with `Forks::On` (as `run` walks) and `Forks::Off` (each read its own walk), and its counts
//! are these exactly: days stepped, days with an act, walks, walks shared, legs booked, def
//! hits and misses (P §1.1's units). The inputs: `m2/counts/days_c.tally` (G7's `days.tally`
//! with the builder's R1, a projection read day by day across a month's end; R2, one read twice
//! through a day; R3, five projections and four across a month's end; R6, a decision on walked
//! days), `m2/counts/r4.tally` (R4, a decision due twice reading one projection from a fixed
//! day, a place first booked between the runs; R7, a hole and a refused leg on walked days) and
//! M1's `m1/lows/` (T5, the reads F5's tree answers). A count the engine changes on purpose (a
//! faster walk) is re-pinned by T, never by the builder. `test = false` and
//! `required-features = ["counters"]`, as `m1_invariant` takes the fault: no other build of the
//! runner has the counters.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally_cli::tally::walk::Forks;
    use tally_cli::tally::{Asks, Days, Given, NotImplemented, Pin, Request, run_with_forks};

    /// R1, R2, R3, R6 (the builder's names): G7's days with the counts' asks.
    const DAYS: [(&str, &str); 1] =
        [("days_c.tally", include_str!("../../m2/counts/days_c.tally"))];

    /// R4 and R7.
    const R4: [(&str, &str); 1] = [("r4.tally", include_str!("../../m2/counts/r4.tally"))];

    /// T5: M1's lows (G9's `lows.tally` with its scenario), the reads F5's tree answers.
    const LOWS: [(&str, &str); 2] = [
        ("lows.tally", include_str!("../../m1/lows/lows.tally")),
        ("richer.tally", include_str!("../../m1/lows/richer.tally")),
    ];

    /// G7's `b1_cons` (T's copy, `m2/fixes/g7/`): places that start near the cap, where F5's tree
    /// declines by the start's bits (`walk/lows.rs` `start_bits`).
    const CAP: [(&str, &str); 1] = [(
        "b1_cons.tally",
        include_str!("../../m2/fixes/g7/b1_cons.tally"),
    )];

    /// A decision run in the next month, a projection read across the month's end
    /// (`m2/fixes/gate8/g8c/b2.tally`; until tally 2.2 the decision ran on the 18th and its leg
    /// stated its own day in the next month, `walk/fork.rs` `booked_ahead`).
    const AHEAD: [(&str, &str); 1] = [(
        "b2.tally",
        include_str!("../../m2/fixes/gate8/g8c/b2.tally"),
    )];

    /// The counters a line holds, in the `--stats` line's words.
    const KEPT: [&str; 7] = [
        "days_stepped",
        "nonempty_days",
        "walks",
        "walks_shared",
        "legs_booked",
        "def_hits",
        "def_misses",
    ];

    /// The counts of one run of `files` asking `asks` with `forks`, as `key=value` words in
    /// `KEPT`'s order; `stopped` where the run stops not implemented.
    fn counts(files: &[(&str, &str)], asks: Asks, forks: Forks) -> String {
        let given = files
            .iter()
            .map(|(path, text)| Given {
                path: (*path).to_owned(),
                bytes: Some(text.as_bytes().to_vec()),
            })
            .collect();
        let request = Request {
            asks,
            pin: Pin::Canonical,
            days: Days::Events,
        };
        let line = match run_with_forks(given, &request, forks) {
            Ok(answer) => answer.counters().line(),
            Err(NotImplemented) => return "stopped".to_owned(),
        };
        let mut out = Vec::new();
        for key in KEPT {
            let word = line
                .split_whitespace()
                .find(|w| w.split_once('=').is_some_and(|(k, _)| k == key))
                .unwrap_or("missing");
            out.push(word.to_owned());
        }
        out.join(" ")
    }

    /// One line per ask of `asks` (and one for every ask, where `every`): the ask, `on` and its
    /// counts, then `off` and its counts.
    fn table(files: &[(&str, &str)], asks: &[&str], every: bool) -> String {
        let mut runs: Vec<(String, Asks)> = asks
            .iter()
            .map(|a| ((*a).to_owned(), Asks::Named(vec![(*a).to_owned()])))
            .collect();
        if every {
            runs.push(("every".to_owned(), Asks::Every));
        }
        let mut out = String::new();
        for (name, asks) in runs {
            let on = counts(files, asks.clone(), Forks::On);
            let off = counts(files, asks, Forks::Off);
            out.push_str(&format!("{name}: on {on}; off {off}\n"));
        }
        out
    }

    /// R1 (`r1`, `r1b`), R2 (`r2`), R3 (`r3` five, `r3f` four), R6 (`rm`, the room def the
    /// sweep decision reads), at 2026-11-15 (fc79eff9); re-pinned at tally 2.2's G1 (T1,
    /// 2026-11-18, the engine of 8ece2484): the sweep decision is dated on the day its leg stated
    /// (a leg has no day of its own, S-6), so no walk carries a leg booked ahead (class S-6/a).
    #[test]
    fn m2_counts_days() {
        let got = table(&DAYS, &["r1", "r1b", "r2", "r3", "r3f", "rm"], false);
        assert_eq!(got, DAYS_COUNTS, "the counts of m2/counts/days_c.tally");
    }

    const DAYS_COUNTS: &str = concat!(
        "r1: on days_stepped=88 nonempty_days=88 walks=6 walks_shared=32 legs_booked=88 def_hits=114 def_misses=38; off days_stepped=222 nonempty_days=222 walks=38 walks_shared=0 legs_booked=222 def_hits=114 def_misses=70\n",
        "r1b: on days_stepped=88 nonempty_days=88 walks=6 walks_shared=32 legs_booked=88 def_hits=114 def_misses=38; off days_stepped=222 nonempty_days=222 walks=38 walks_shared=0 legs_booked=222 def_hits=114 def_misses=70\n",
        "r2: on days_stepped=84 nonempty_days=82 walks=4 walks_shared=2 legs_booked=81 def_hits=136 def_misses=40; off days_stepped=94 nonempty_days=90 walks=6 walks_shared=0 legs_booked=89 def_hits=136 def_misses=42\n",
        "r3: on days_stepped=572 nonempty_days=468 walks=132 walks_shared=0 legs_booked=468 def_hits=202 def_misses=172; off days_stepped=572 nonempty_days=468 walks=132 walks_shared=0 legs_booked=468 def_hits=202 def_misses=172\n",
        "r3f: on days_stepped=136 nonempty_days=124 walks=18 walks_shared=88 legs_booked=124 def_hits=180 def_misses=56; off days_stepped=472 nonempty_days=394 walks=106 walks_shared=0 legs_booked=394 def_hits=180 def_misses=144\n",
        "rm: on days_stepped=72 nonempty_days=72 walks=2 walks_shared=0 legs_booked=72 def_hits=114 def_misses=40; off days_stepped=72 nonempty_days=72 walks=2 walks_shared=0 legs_booked=72 def_hits=114 def_misses=40\n",
    );

    /// R4 (`r4`, `r4j`) and R7 (`r7`), at 2026-11-15 (fc79eff9).
    #[test]
    fn m2_counts_r4() {
        let got = table(&R4, &["r4", "r4j", "r7"], false);
        assert_eq!(got, R4_COUNTS, "the counts of m2/counts/r4.tally");
    }

    const R4_COUNTS: &str = concat!(
        "r4: on days_stepped=45 nonempty_days=43 walks=3 walks_shared=0 legs_booked=47 def_hits=0 def_misses=0; off days_stepped=45 nonempty_days=43 walks=3 walks_shared=0 legs_booked=47 def_hits=0 def_misses=0\n",
        "r4j: on days_stepped=45 nonempty_days=43 walks=3 walks_shared=0 legs_booked=47 def_hits=0 def_misses=0; off days_stepped=45 nonempty_days=43 walks=3 walks_shared=0 legs_booked=47 def_hits=0 def_misses=0\n",
        "r7: on days_stepped=65 nonempty_days=61 walks=7 walks_shared=41 legs_booked=71 def_hits=0 def_misses=0; off days_stepped=330 nonempty_days=299 walks=75 walks_shared=0 legs_booked=404 def_hits=0 def_misses=0\n",
    );

    /// T5: every ask of M1's lows at once, and each, at 2026-11-15 (fc79eff9).
    #[test]
    fn m2_counts_lows() {
        let got = table(&LOWS, &[], true);
        assert_eq!(got, LOWS_COUNTS, "the counts of m1/lows/lows.tally");
    }

    const LOWS_COUNTS: &str = concat!(
        "every: on days_stepped=880 nonempty_days=878 walks=28 walks_shared=0 legs_booked=878 def_hits=710 def_misses=494; off days_stepped=880 nonempty_days=878 walks=28 walks_shared=0 legs_booked=878 def_hits=710 def_misses=494\n",
    );

    /// G7's `b1_cons`, each ask, at 2026-11-15 (fc79eff9).
    #[test]
    fn m2_counts_cap() {
        let got = table(&CAP, &["b_pair", "b_pair_rev", "b_big", "b_konto"], false);
        assert_eq!(got, CAP_COUNTS, "the counts of m2/fixes/g7/b1_cons.tally");
    }

    const CAP_COUNTS: &str = concat!(
        "b_pair: on days_stepped=14 nonempty_days=13 walks=2 walks_shared=1 legs_booked=124 def_hits=300 def_misses=24; off days_stepped=16 nonempty_days=14 walks=3 walks_shared=0 legs_booked=236 def_hits=300 def_misses=24\n",
        "b_pair_rev: on days_stepped=14 nonempty_days=13 walks=2 walks_shared=1 legs_booked=124 def_hits=300 def_misses=24; off days_stepped=16 nonempty_days=14 walks=3 walks_shared=0 legs_booked=236 def_hits=300 def_misses=24\n",
        "b_big: on days_stepped=14 nonempty_days=13 walks=2 walks_shared=0 legs_booked=124 def_hits=300 def_misses=24; off days_stepped=14 nonempty_days=13 walks=2 walks_shared=0 legs_booked=124 def_hits=300 def_misses=24\n",
        "b_konto: on days_stepped=14 nonempty_days=13 walks=2 walks_shared=0 legs_booked=124 def_hits=300 def_misses=24; off days_stepped=14 nonempty_days=13 walks=2 walks_shared=0 legs_booked=124 def_hits=300 def_misses=24\n",
    );

    /// `b2`'s asks, at 2026-11-17 on tally 2.1.8's engine (91421810): the walks #183 forces
    /// (R16, #200) take walks 6 to 28 (4 to 26 for `b2_one`); at 2026-11-15 (fc79eff9) before.
    /// Re-pinned at tally 2.2's G1 (T1, 2026-11-18, the engine of 8ece2484): the sweep decision
    /// is dated on the day its leg stated (S-6), so the walks #183 forced are gone (4 with forks
    /// on; class S-6/a, the plan's §4).
    #[test]
    fn m2_counts_ahead() {
        let got = table(&AHEAD, &["b2_min", "b2_all", "b2_one"], false);
        assert_eq!(
            got, AHEAD_COUNTS,
            "the counts of m2/fixes/gate8/g8c/b2.tally"
        );
    }

    const AHEAD_COUNTS: &str = concat!(
        "b2_min: on days_stepped=78 nonempty_days=78 walks=4 walks_shared=14 legs_booked=78 def_hits=136 def_misses=36; off days_stepped=120 nonempty_days=120 walks=18 walks_shared=0 legs_booked=120 def_hits=136 def_misses=50\n",
        "b2_all: on days_stepped=76 nonempty_days=76 walks=4 walks_shared=6 legs_booked=76 def_hits=136 def_misses=36; off days_stepped=94 nonempty_days=94 walks=10 walks_shared=0 legs_booked=94 def_hits=136 def_misses=42\n",
        "b2_one: on days_stepped=76 nonempty_days=76 walks=4 walks_shared=0 legs_booked=76 def_hits=136 def_misses=36; off days_stepped=76 nonempty_days=76 walks=4 walks_shared=0 legs_booked=76 def_hits=136 def_misses=36\n",
    );
}
