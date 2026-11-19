//! Fence F6 against its twin on T's corpora, X's P8-14 check and its own realities: a projection's walk continued, against a new walk per read.
//! Spec: P §2.5 (timelines whose acts agree up to a day share the walk to it; the unforked walk
//! is the oracle: the same bytes, fewer walks), §2.3 (a projection's walk, and F5's tree in its
//! common case), E §4.3 (a projection's walk from day t, the movements dated t through d as
//! known on t), R-130 (a fence and its plain twin print the same bytes).
//! Twin: engine/tools/refcore/walk.py (`proj_walk`, `run_to`: each read its own walk), which
//! prints the lines pinned here for F6's own reality.
//! Invariants: reads the corpora's realities, X's files and its own realities only, never an
//! answer key or a baseline (R-93); each runs with forks on and off, under F5's tree and under
//! the projection's own walk, and prints alike; X's check and F6's realities are answered, and
//! F6's realities print refcore's lines.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally::eval::Memo;
    use tally::walk::{Forks, Lows};
    use tally::{Asks, Files, Given, How, NotImplemented, Request, run_with_how};

    /// T's generated realities: netstring records of law, case, variant, arguments, files.
    const KERNEL_CORPUS: &[u8] = include_bytes!("../../../tests/properties/kernel.corpus");

    /// T's M0 realities, in the same form.
    const M0_CORPUS: &[u8] = include_bytes!("../../../tests/properties/m0.corpus");

    /// T's M1 realities, in the same form.
    const M1_CORPUS: &[u8] = include_bytes!("../../../tests/properties/m1.corpus");

    /// T's M2 realities, in the same form.
    const M2_CORPUS: &[u8] = include_bytes!("../../../tests/properties/m2.corpus");

    /// F6's own reality: a pay that differs by an axis's option, a monthly decision whose
    /// amount reads a projection day by day from its month's first day, a rent a later
    /// version raises; projections read day by day, back and forth, five
    /// at a time (past the walks a timeline keeps), and with `with uncertain`.
    const DAYS: &str = include_str!("forks/days.tally");

    /// F6's review's reality B1: 132 places each given a number under the cap, whose sum at
    /// the outside place they come from passes it, so the projection's walk from 2026-01-20
    /// refuses every read through a day from 2026-02-01 on, at any place; konto read alone,
    /// after such a read and before it.
    const CAP: &str = include_str!("forks/b1_cons.tally");

    /// F5's realities, each read through projections: under axes and in a scenario's world,
    /// across cuts, after a walk refused on a decision's day, past a covered month, and with a
    /// rent's day moving between cuts.
    const LOWS: [(&str, &str); 7] = [
        ("lows.tally", include_str!("lows/lows.tally")),
        ("richer.tally", include_str!("lows/richer.tally")),
        ("cuts.tally", include_str!("lows/cuts.tally")),
        ("failed.tally", include_str!("lows/failed.tally")),
        ("cov_x.tally", include_str!("lows/cov_x.tally")),
        ("cov_nocov.tally", include_str!("lows/cov_nocov.tally")),
        ("inv_min.tally", include_str!("lows/inv_min.tally")),
    ];

    /// X's files at cut 11, change b, read 1: P8-14's check (`I:X:replay_september@b_r1`),
    /// whose projections are read day by day.
    const X_B_R1: [(&str, &str); 13] = [
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

    /// One netstring, its length, a colon, its bytes and a comma, at the head of `s`, and the
    /// rest after it.
    fn take(s: &[u8]) -> (&[u8], &[u8]) {
        let colon = s.iter().position(|b| *b == b':').expect("a length");
        let digits = std::str::from_utf8(s.get(..colon).expect("digits")).expect("digits");
        let n: usize = digits.parse().expect("a length");
        let rest = s
            .get(colon.checked_add(1).expect("a colon")..)
            .expect("a body");
        let body = rest.get(..n).expect("a netstring's bytes");
        let after = rest.get(n..).expect("a netstring's end");
        let after = after.strip_prefix(b",").expect("a netstring's comma");
        (body, after)
    }

    /// One record: its name (case, variant, law) and its files, as bytes: a load case's file
    /// may be no UTF-8 at all.
    struct Record {
        name: String,
        files: Vec<(String, Vec<u8>)>,
    }

    /// Every record of the corpus.
    fn records(corpus: &[u8]) -> Vec<Record> {
        let text = |b: &[u8]| String::from_utf8(b.to_vec()).expect("a name in UTF-8");
        let mut out = Vec::new();
        let mut rest = corpus;
        loop {
            while let Some(after) = rest.strip_prefix(b"\n") {
                rest = after;
            }
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
                files.push((text(name), body.to_vec()));
                t = after_body;
            }
            out.push(Record {
                name: format!("{}/{} ({})", text(case), text(variant), text(law)),
                files,
            });
        }
    }

    /// Files' lines for `asks`, each ask's engine reading a projection's common case as `lows`
    /// says and continuing a projection's walk as `forks` says; none past what the engine
    /// builds.
    fn answer(
        files: &[(&str, &[u8])],
        asks: &Asks,
        (lows, forks): (Lows, Forks),
    ) -> Option<Vec<String>> {
        let given = files
            .iter()
            .map(|(path, body)| Given {
                path: (*path).to_owned(),
                bytes: Some(body.to_vec()),
            })
            .collect();
        let request = Request::new(asks.clone());
        let how = How::new(Memo::Classes, lows, forks);
        match run_with_how(Files::new(given), &request, how) {
            Ok(out) => Some(out.lines().to_vec()),
            Err(NotImplemented) => None,
        }
    }

    /// Files' lines for `asks` with forks on, after checking they are the lines with forks
    /// off, under `lows`: the fence and its twin alike (R-130).
    fn alike(name: &str, files: &[(&str, &[u8])], asks: &Asks, lows: Lows) -> Option<Vec<String>> {
        let on = answer(files, asks, (lows, Forks::On));
        let off = answer(files, asks, (lows, Forks::Off));
        assert_eq!(on, off, "{name} under {lows:?}: forks on and off differ");
        on
    }

    /// Text files as the run is given them.
    fn bytes<'a>(files: &[(&'a str, &'a str)]) -> Vec<(&'a str, &'a [u8])> {
        files
            .iter()
            .map(|(path, text)| (*path, text.as_bytes()))
            .collect()
    }

    /// Every record of `corpus` (`count` of them) alike with forks on and off, under F5's tree
    /// and under the projection's own walk, where a projection's walk is read most.
    fn corpus_alike(corpus: &[u8], count: usize) {
        let all = records(corpus);
        assert_eq!(all.len(), count, "the corpus's records");
        for record in &all {
            let files: Vec<(&str, &[u8])> = record
                .files
                .iter()
                .map(|(path, body)| (path.as_str(), body.as_slice()))
                .collect();
            for lows in [Lows::Tree, Lows::Walk] {
                alike(&record.name, &files, &Asks::Every, lows);
            }
        }
    }

    #[test]
    fn forks_on_prints_forks_off_on_the_kernel_and_m0_corpora() {
        corpus_alike(KERNEL_CORPUS, 312);
        corpus_alike(M0_CORPUS, 285);
    }

    #[test]
    fn forks_on_prints_forks_off_on_the_m1_corpus() {
        corpus_alike(M1_CORPUS, 648);
    }

    #[test]
    fn forks_on_prints_forks_off_on_the_m2_corpus() {
        corpus_alike(M2_CORPUS, 168);
    }

    #[test]
    fn forks_on_prints_forks_off_on_p8_14s_check() {
        for ask in ["replay_september", "konto_balance"] {
            let asks = Asks::Named(vec![ask.to_owned()]);
            let lines = alike(ask, &bytes(&X_B_R1), &asks, Lows::Tree);
            assert!(lines.is_some(), "{ask}@b_r1 is answered");
        }
    }

    #[test]
    fn forks_on_prints_forks_off_and_refcore_on_its_own_reality() {
        let files = bytes(&[("days.tally", DAYS)]);
        for lows in [Lows::Tree, Lows::Walk] {
            let lines = alike("days.tally", &files, &Asks::Every, lows).expect("answered");
            // refcore's lines (tally 2.1.8, the sweep's leg on its day as tally 2.2 books it),
            // each under both options of the axis the pay reads
            for line in [
                "d_min [pay_reading = low]: 350.00 DUK",
                "d_min [pay_reading = high]: 575.00 DUK",
                "d_back (d = 2026-03-10) [pay_reading = low]: -50.00 DUK",
                "d_back (d = 2026-02-20) [pay_reading = high]: -75.00 DUK",
                "d_back (d = 2026-02-03) [pay_reading = low]: 1 400.00 DUK",
                "d_many [pay_reading = high]: 275.00 DUK",
                "d_axis [pay_reading = low]: 325.00 DUK",
                "d_uncertain [pay_reading = low]: -218.75 DUK",
                "d_room (m = 2026-07) [pay_reading = high]: 910.94 DUK",
                "  held: rent = 950.00 DUK (later), read for 2026-07-01",
            ] {
                assert!(
                    lines.iter().any(|l| l == line),
                    "{lows:?}: {line} is printed"
                );
            }
        }
    }

    #[test]
    fn a_walk_refused_at_another_place_refuses_the_read_on_and_off() {
        let files = bytes(&[("b1_cons.tally", CAP)]);
        let refused = "refused: RUN-DIGITS a value passes 4 096 digits; round where the law rounds";
        // refcore's lines (tally 2.1.7), byte for byte
        let expected = [
            format!("b_big: {refused}"),
            "  cut 2026-04-15".to_owned(),
            format!("b_konto: {refused}"),
            "  cut 2026-04-15".to_owned(),
            format!("b_pair: ({refused}, {refused})"),
            "  cut 2026-04-15".to_owned(),
            format!("b_pair_rev: ({refused}, {refused})"),
            "  cut 2026-04-15".to_owned(),
        ];
        for lows in [Lows::Tree, Lows::Walk] {
            let lines = alike("b1_cons.tally", &files, &Asks::Every, lows).expect("answered");
            assert_eq!(lines, expected, "{lows:?}: refcore's lines");
        }
    }

    #[test]
    fn forks_on_prints_forks_off_on_f5s_realities() {
        for lows in [Lows::Tree, Lows::Walk] {
            for (name, text) in LOWS {
                if name == "richer.tally" {
                    continue;
                }
                let lines = alike(name, &bytes(&[(name, text)]), &Asks::Every, lows);
                assert!(lines.is_some(), "{name} is answered");
            }
            let world = bytes(LOWS.get(..2).expect("the model and its scenario"));
            let lines = alike("lows.tally with richer.tally", &world, &Asks::Every, lows);
            assert!(
                lines.is_some(),
                "F5's reality and its scenario are answered"
            );
        }
    }
}
