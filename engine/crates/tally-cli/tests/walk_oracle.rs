//! Twins on the kernel corpus and on X's three S checks: the event walk against the every-day walk, cells shared by law date class against cells at each law date.
//! Spec: P §2.2 (the oracle: a step-every-day walk beside the event-driven one, built from the
//! same step functions, prints the same bytes; a property test compares them on every case);
//! E §4.2 (the day walk), §4.3 (a projection's walk); E §3.2 (the law date is a class), P §3.3
//! (the trimming's check: every case run with it on and off, the bytes compared);
//! BUILD-ORDER § S, Green ("the event walk equals the step-every-day walk on every S check").
//! Twin: engine/tools/refcore/walk.py (`run_to`, which steps every day), evaluate.py
//! (`def_cell`, its memo keyed by the exact law date).
//! Invariants: reads the corpus's realities and X's files only, never an answer key or a
//! baseline (R-93); every record and every check answers, both walks print each alike (SQ-3),
//! and so do both memos on the corpus.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally::eval::Memo;
    use tally::{Asks, Days, Given, NotImplemented, Pin, Request, run_with_memo};

    /// T's generated realities: netstring records of law, case, variant, arguments, files.
    const CORPUS: &str = include_str!("../../../tests/properties/kernel.corpus");

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
}
