//! The walk oracle on the kernel corpus: every record answered by the event walk and by the step-every-day walk, the same bytes.
//! Spec: P §2.2 (the oracle: a step-every-day walk beside the event-driven one, built from the
//! same step functions, prints the same bytes; a property test compares them on every case);
//! E §4.2 (the day walk), §4.3 (a projection's walk).
//! Twin: engine/tools/refcore/walk.py (`run_to`, which steps every day).
//! Invariants: reads the corpus's realities only, never its answer key (R-93); every record
//! answers, and both walks print each alike (SQ-3).

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally::{Asks, Days, Given, NotImplemented, Pin, Request, run};

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

    /// A record's answer with the walk stepping `days`; none past what the engine builds.
    fn answer(record: &Record, days: Days) -> Option<Vec<String>> {
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
        match run(given, &request) {
            Ok(out) => Some(out.lines().to_vec()),
            Err(NotImplemented) => None,
        }
    }

    #[test]
    fn the_event_walk_prints_what_the_every_day_walk_prints() {
        let all = records(CORPUS);
        assert_eq!(all.len(), 312, "the corpus's records");
        for record in &all {
            let events = answer(record, Days::Events);
            let every = answer(record, Days::Every);
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
}
