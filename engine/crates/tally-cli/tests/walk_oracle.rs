//! Twins on the corpora and on X's three S checks: the event walk against the every-day walk, cells by law date class against exact, F5's tree against the projection's walk.
//! Spec: P §2.2 (the oracle: a step-every-day walk beside the event-driven one, built from the
//! same step functions, prints the same bytes; a property test compares them on every case);
//! E §4.2 (the day walk), §4.3 (a projection's walk; its common case from a running sum,
//! fence F5); E §3.2 (the law date is a class), P §3.3 (the trimming's check: every case run
//! with it on and off, the bytes compared), P §2.3 (F5's tree);
//! BUILD-ORDER § S, Green ("the event walk equals the step-every-day walk on every S check").
//! Twin: engine/tools/refcore/walk.py (`run_to`, which steps every day), evaluate.py
//! (`def_cell`, its memo keyed by the exact law date), mreads.py (`proj_bal`, `m_e_low`).
//! Invariants: reads the corpora's realities, X's files and its own reality only, never an
//! answer key or a baseline (R-93); every record and every check answers, both walks print
//! each alike (SQ-3), and so do both memos on the kernel corpus, and F5's tree and the
//! projection's own walk on the kernel and M1 corpora and on a projection under an axis and in
//! a scenario's world.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally::eval::Memo;
    use tally::walk::Lows;
    use tally::{Asks, Days, Given, NotImplemented, Pin, Request, run_with_lows, run_with_memo};

    /// T's generated realities: netstring records of law, case, variant, arguments, files.
    const CORPUS: &str = include_str!("../../../tests/properties/kernel.corpus");

    /// T's M1 realities (chunk C6), in the kernel corpus's form: each M1 law's records, the
    /// `low` laws' among them (`low-min`, `foresight-low`).
    const M1_CORPUS: &str = include_str!("../../../tests/properties/m1.corpus");

    /// F5's own reality: a pay that differs by an axis's option, a rent that a later version
    /// raises, a monthly look-ahead decision reading `low` from its own day, and projections
    /// from days before and after the cut, one with the axis pinned.
    const LOWS_MODEL: &str = include_str!("lows/lows.tally");

    /// Its scenario `richer`, from June: a higher pay, and two projections read in its world.
    const LOWS_SCENARIO: &str = include_str!("lows/richer.tally");

    /// Projections read from many days before the cut: across a version that raises the rent
    /// (the same days, new sums) and one that moves a bonus (new days), and back again.
    const LOWS_CUTS: &str = include_str!("lows/cuts.tally");

    /// G9's review's `fail_cut`: a projection's walk refused on a decision's day, read through
    /// a window that reaches it and one that ends before it, in both orders (pick P54, #127).
    const LOWS_FAILED: &str = include_str!("lows/failed.tally");

    /// G9's fix review's `cov_x` (B1): a sweep whose amount reads `flows` of a rent a version
    /// covers, read from two cuts with no new fact between them, so the rent turns past and
    /// modelled with the cut alone.
    const LOWS_COVERED: &str = include_str!("lows/cov_x.tally");

    /// Its `cov_nocov`: the same without `covers`, so the past rent stays unfulfilled, and the
    /// register says so.
    const LOWS_UNCOVERED: &str = include_str!("lows/cov_nocov.tally");

    /// Its `inv_min` (B2): a rent's day read at the ask's cut moves between the cut and the
    /// projection's while its booking cut stays, so its A₀ must not count the day.
    const LOWS_BOOKED: &str = include_str!("lows/inv_min.tally");

    /// One netstring `N:text,` at the head of `s`, and the rest after it.
    fn take(s: &str) -> (&str, &str) {
        let (len, rest) = s.split_once(':').expect("a netstring's length");
        let n: usize = len.parse().expect("a length");
        let body = rest.get(..n).expect("a netstring's text");
        let after = rest.get(n..).expect("a netstring's end");
        let after = after.strip_prefix(',').expect("a netstring's comma");
        (body, after)
    }

    /// One record: its law, case and variant, its files.
    struct Record {
        law: String,
        case: String,
        variant: String,
        files: Vec<(String, String)>,
    }

    /// Every record of the corpus.
    fn records(corpus: &str) -> Vec<Record> {
        let mut out = Vec::new();
        let mut rest = corpus;
        loop {
            rest = rest.trim_start_matches('\n');
            if rest.is_empty() {
                return out;
            }
            let (law, after_law) = take(rest);
            let (case, after_case) = take(after_law);
            let (variant, after_variant) = take(after_case);
            let (_args, after_args) = take(after_variant);
            let (table, after_table) = take(after_args);
            rest = after_table;
            let mut files = Vec::new();
            let mut t = table;
            while !t.is_empty() {
                let (name, after_name) = take(t);
                let (body, after_body) = take(after_name);
                files.push((name.to_owned(), body.to_owned()));
                t = after_body;
            }
            out.push(Record {
                law: law.to_owned(),
                case: case.to_owned(),
                variant: variant.to_owned(),
                files,
            });
        }
    }

    /// A record's answer with the walk stepping `days` and the memo keeping cells as `memo`
    /// says; none past what the engine builds.
    fn answer(record: &Record, days: Days, memo: Memo) -> Option<Vec<String>> {
        let given = record
            .files
            .iter()
            .map(|(path, body)| Given {
                path: path.clone(),
                bytes: Some(body.as_bytes().to_vec()),
            })
            .collect();
        let request = Request {
            asks: Asks::Every,
            pin: Pin::Canonical,
            days,
        };
        match run_with_memo(given, &request, memo) {
            Ok(out) => Some(out.lines().to_vec()),
            Err(NotImplemented) => None,
        }
    }

    #[test]
    fn the_event_walk_prints_what_the_every_day_walk_prints() {
        let all = records(CORPUS);
        assert_eq!(all.len(), 312, "the corpus's records");
        for record in &all {
            let events = answer(record, Days::Events, Memo::Classes);
            let every = answer(record, Days::Every, Memo::Classes);
            assert_eq!(events, every, "{} {}", record.case, record.variant);
            assert!(
                events.is_some(),
                "{} {} ({}) is answered",
                record.case,
                record.variant,
                record.law
            );
        }
    }

    /// X's files: each path, relative to the amendments folder, with its text.
    type XFiles = [(&'static str, &'static str); 13];

    /// X's base files at cut 11 of revision 1, as the S checks' recipes name them, relative to
    /// the amendments folder (`checks.tsv`'s cwd), with their text.
    const X_B_R1: XFiles = [
        (
            "cut11/b_r1/asks/cut.tally",
            include_str!("../../../tests/integration/amendments/cut11/b_r1/asks/cut.tally"),
        ),
        (
            "cut11/b_r1/facts/2026.tally",
            include_str!("../../../tests/integration/amendments/cut11/b_r1/facts/2026.tally"),
        ),
        (
            "cut11/b_r1/law/pl/benefits.tally",
            include_str!("../../../tests/integration/amendments/cut11/b_r1/law/pl/benefits.tally"),
        ),
        (
            "cut11/b_r1/law/pl/business.tally",
            include_str!("../../../tests/integration/amendments/cut11/b_r1/law/pl/business.tally"),
        ),
        (
            "cut11/b_r1/law/pl/common.tally",
            include_str!("../../../tests/integration/amendments/cut11/b_r1/law/pl/common.tally"),
        ),
        (
            "cut11/b_r1/law/pl/employment.tally",
            include_str!(
                "../../../tests/integration/amendments/cut11/b_r1/law/pl/employment.tally"
            ),
        ),
        (
            "cut11/b_r1/law/pl/wrappers.tally",
            include_str!("../../../tests/integration/amendments/cut11/b_r1/law/pl/wrappers.tally"),
        ),
        (
            "cut11/b_r1/loans.tally",
            include_str!("../../../tests/integration/amendments/cut11/b_r1/loans.tally"),
        ),
        (
            "cut11/b_r1/pays.tally",
            include_str!("../../../tests/integration/amendments/cut11/b_r1/pays.tally"),
        ),
        (
            "cut11/b_r1/people.tally",
            include_str!("../../../tests/integration/amendments/cut11/b_r1/people.tally"),
        ),
        (
            "cut11/b_r1/places.tally",
            include_str!("../../../tests/integration/amendments/cut11/b_r1/places.tally"),
        ),
        (
            "cut11/b_r1/reality.tally",
            include_str!("../../../tests/integration/amendments/cut11/b_r1/reality.tally"),
        ),
        (
            "cut11/b_r1/records/mbank/2026-09.tally",
            include_str!(
                "../../../tests/integration/amendments/cut11/b_r1/records/mbank/2026-09.tally"
            ),
        ),
    ];

    /// The same files of the revision `b_r1s`.
    const X_B_R1S: XFiles = [
        (
            "cut11/b_r1s/asks/cut.tally",
            include_str!("../../../tests/integration/amendments/cut11/b_r1s/asks/cut.tally"),
        ),
        (
            "cut11/b_r1s/facts/2026.tally",
            include_str!("../../../tests/integration/amendments/cut11/b_r1s/facts/2026.tally"),
        ),
        (
            "cut11/b_r1s/law/pl/benefits.tally",
            include_str!("../../../tests/integration/amendments/cut11/b_r1s/law/pl/benefits.tally"),
        ),
        (
            "cut11/b_r1s/law/pl/business.tally",
            include_str!("../../../tests/integration/amendments/cut11/b_r1s/law/pl/business.tally"),
        ),
        (
            "cut11/b_r1s/law/pl/common.tally",
            include_str!("../../../tests/integration/amendments/cut11/b_r1s/law/pl/common.tally"),
        ),
        (
            "cut11/b_r1s/law/pl/employment.tally",
            include_str!(
                "../../../tests/integration/amendments/cut11/b_r1s/law/pl/employment.tally"
            ),
        ),
        (
            "cut11/b_r1s/law/pl/wrappers.tally",
            include_str!("../../../tests/integration/amendments/cut11/b_r1s/law/pl/wrappers.tally"),
        ),
        (
            "cut11/b_r1s/loans.tally",
            include_str!("../../../tests/integration/amendments/cut11/b_r1s/loans.tally"),
        ),
        (
            "cut11/b_r1s/pays.tally",
            include_str!("../../../tests/integration/amendments/cut11/b_r1s/pays.tally"),
        ),
        (
            "cut11/b_r1s/people.tally",
            include_str!("../../../tests/integration/amendments/cut11/b_r1s/people.tally"),
        ),
        (
            "cut11/b_r1s/places.tally",
            include_str!("../../../tests/integration/amendments/cut11/b_r1s/places.tally"),
        ),
        (
            "cut11/b_r1s/reality.tally",
            include_str!("../../../tests/integration/amendments/cut11/b_r1s/reality.tally"),
        ),
        (
            "cut11/b_r1s/records/mbank/2026-09.tally",
            include_str!(
                "../../../tests/integration/amendments/cut11/b_r1s/records/mbank/2026-09.tally"
            ),
        ),
    ];

    /// One of X's checks answered with the walk stepping `days`: the files as the check's
    /// recipe names them, the one ask.
    fn x_answer(files: &[(&str, &str)], ask: &str, days: Days) -> Option<Vec<String>> {
        let given = files
            .iter()
            .map(|(path, text)| Given {
                path: (*path).to_owned(),
                bytes: Some(text.as_bytes().to_vec()),
            })
            .collect();
        let request = Request {
            asks: Asks::Named(vec![ask.to_owned()]),
            pin: Pin::Canonical,
            days,
        };
        match run_with_memo(given, &request, Memo::Classes) {
            Ok(out) => Some(out.lines().to_vec()),
            Err(NotImplemented) => None,
        }
    }

    #[test]
    fn the_event_walk_prints_what_the_every_day_walk_prints_on_xs_checks() {
        // C1–C3 (BUILD-ORDER § S, Green): konto_balance and replay_september at b_r1,
        // replay_september at b_r1s.
        let checks: [(&XFiles, &str, &str); 3] = [
            (&X_B_R1, "b_r1", "konto_balance"),
            (&X_B_R1, "b_r1", "replay_september"),
            (&X_B_R1S, "b_r1s", "replay_september"),
        ];
        for (files, cut, ask) in checks {
            let events = x_answer(files, ask, Days::Events);
            let every = x_answer(files, ask, Days::Every);
            assert!(events.is_some(), "{ask}@{cut} is answered");
            assert_eq!(events, every, "{ask}@{cut}");
        }
    }

    #[test]
    fn cells_shared_by_law_date_class_print_what_cells_at_each_law_date_print() {
        let all = records(CORPUS);
        assert_eq!(all.len(), 312, "the corpus's records");
        for record in &all {
            let shared = answer(record, Days::Events, Memo::Classes);
            let exact = answer(record, Days::Events, Memo::Exact);
            assert_eq!(shared, exact, "{} {}", record.case, record.variant);
            assert!(
                shared.is_some(),
                "{} {} ({}) is answered",
                record.case,
                record.variant,
                record.law
            );
        }
    }

    /// Files' lines, every ask, with a written projection's common case read as `lows` says;
    /// none past what the engine builds.
    fn lows_answer(files: &[(String, String)], lows: Lows) -> Option<Vec<String>> {
        let given = files
            .iter()
            .map(|(path, body)| Given {
                path: path.clone(),
                bytes: Some(body.as_bytes().to_vec()),
            })
            .collect();
        let request = Request {
            asks: Asks::Every,
            pin: Pin::Canonical,
            days: Days::Events,
        };
        match run_with_lows(given, &request, lows) {
            Ok(out) => Some(out.lines().to_vec()),
            Err(NotImplemented) => None,
        }
    }

    #[test]
    fn the_tree_prints_what_the_projections_walk_prints_on_both_corpora() {
        for (corpus, count) in [(CORPUS, 312), (M1_CORPUS, 648)] {
            let all = records(corpus);
            assert_eq!(all.len(), count, "the corpus's records");
            for record in &all {
                let tree = lows_answer(&record.files, Lows::Tree);
                let walk = lows_answer(&record.files, Lows::Walk);
                assert_eq!(
                    tree, walk,
                    "{} {} ({})",
                    record.case, record.variant, record.law
                );
            }
        }
    }

    #[test]
    fn the_tree_prints_what_the_projections_walk_prints_under_axes_and_in_a_scenario() {
        let files = [
            ("lows.tally".to_owned(), LOWS_MODEL.to_owned()),
            ("richer.tally".to_owned(), LOWS_SCENARIO.to_owned()),
        ];
        let tree = lows_answer(&files, Lows::Tree);
        assert_eq!(tree, lows_answer(&files, Lows::Walk), "F5's reality");
        let lines = tree.expect("F5's reality is answered");
        // refcore's lines (tally 2.1.7): one per option of the axis, so the two aren't the same
        // refusal; and in the scenario, whose last days rest on a walked value past the run's
        // end E, the hole `after the run's end` (L §8.1 `HOLE-POSTRUN`)
        for line in [
            "q_low_late [pay_reading = low]: 250.49 DUK",
            "q_low_late [pay_reading = high]: 550.35 DUK",
            "s_low [pay_reading = *]: ?\"after the run's end\" (konto)",
        ] {
            assert!(lines.iter().any(|l| l == line), "{line} is printed");
        }
    }

    #[test]
    fn the_tree_prints_what_the_projections_walk_prints_across_cuts_and_after_a_failure() {
        for (name, text, pinned) in [
            (
                "cuts.tally",
                LOWS_CUTS,
                // refcore's, byte for byte
                &[
                    "c_low: 600.00 DUK",
                    "c_sum: 222 150.00 DUK",
                    "c_back: (1 550.00 DUK, 1 950.00 DUK, 800.00 DUK, 1 500.00 DUK)",
                ][..],
            ),
            (
                "failed.tally",
                LOWS_FAILED,
                // P54: a read ending before the failed day is answered, in either order
                &[
                    "f_long_short: (refused: RUN-TIE pick: its ties by is a hole, 800.00 DUK)",
                    "f_short_long: (800.00 DUK, refused: RUN-TIE pick: its ties by is a hole)",
                ][..],
            ),
            (
                "cov_x.tally",
                LOWS_COVERED,
                // refcore's: from 03-15 the March rent is modelled, so the sweep is 0.00
                &[
                    "cov_two: (100.00 DUK, 800.00 DUK)",
                    "cov_two_rev: (800.00 DUK, 100.00 DUK)",
                ][..],
            ),
            (
                "cov_nocov.tally",
                LOWS_UNCOVERED,
                // refcore's: the register follows the line
                &[
                    "cov_two: (900.00 DUK, 900.00 DUK)",
                    "  unfulfilled: rent_pay[2026-03]",
                ][..],
            ),
            (
                "inv_min.tally",
                LOWS_BOOKED,
                // refcore's: no invariant broken
                &["z: 1 950.00 DUK"][..],
            ),
        ] {
            let files = [(name.to_owned(), text.to_owned())];
            let tree = lows_answer(&files, Lows::Tree);
            assert_eq!(tree, lows_answer(&files, Lows::Walk), "{name}");
            let lines = tree.expect("answered");
            for line in pinned {
                assert!(lines.iter().any(|l| l == line), "{name}: {line} is printed");
            }
        }
        // the register of cov_two's second read: the tree's own, not its first read's cut
        let files = [("cov_nocov.tally".to_owned(), LOWS_UNCOVERED.to_owned())];
        let lines = lows_answer(&files, Lows::Tree).expect("answered");
        let at = lines
            .iter()
            .position(|l| l == "cov_two: (900.00 DUK, 900.00 DUK)")
            .expect("cov_two");
        assert_eq!(
            lines
                .get(at.checked_add(1).expect("a line"))
                .map(String::as_str),
            Some("  unfulfilled: rent_pay[2026-03]"),
            "cov_two's register"
        );
    }
}
