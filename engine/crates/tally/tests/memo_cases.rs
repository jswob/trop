//! The builder's memo tests: cells shared by law date class answer as refcore does where spans, holes, `held` and rules turn on the day.
//! Spec: E §3.2 (the law date is a class; every law date inside one gives the same value);
//! P §3.3 (a trimming changes no printed byte; its check runs a case with it on and off);
//! L §4.6 (the read rule's spans; the gap's question names the day), §8.2 (`held` names the
//! day read for), §5.11 (`RUN-NO-RULE` names the law date; a rule version in force by its
//! version's span), §5.9 (a band's tables by their spans).
//! Twin: engine/tools/refcore/evaluate.py (`def_cell`, its memo keyed by the exact law date);
//! each expected line is refcore's run of the same text (`python3 -m refcore run`).
//! Invariants: reads no test, vector or answer key (R-93); the text is written here; the same
//! lines under `Memo::Classes` and `Memo::Exact`.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally::eval::Memo;
    use tally::{Answer, Asks, Days, Given, Pin, Request, run_with_memo};

    /// Defs read `at reader` from six law dates: two before any row (a hole naming its day),
    /// two in 2026 (one class), two in 2027, past the ask's cut (`held` naming the day, a rule
    /// version and a band table of 2027, `RUN-NO-RULE` naming the day).
    const REALITY: &str = r#"model memo
  tally 2
  on 2026-06-30
  start 2026-01-01
  end 2027-12-31

source card "a card"

unit PLN: currency minor 0.01

index k = int 1..6

input fee: money PLN

input mode: int

input when_of[i in k]: date

band tax(v: money PLN): money PLN

def base: money PLN at reader
  rule base = fee

def chain[i in k]: money PLN at reader
  rule first = base when i == 1
  rule later = chain[i - 1] + base when i > 1

def flag: int at reader
  rule on = 1 when mode == 1

def bonus: money PLN at reader
  rule plain = 0
  rule extra beats plain v v27 = fee

def taxed: money PLN = tax(chain[3]) at reader

def seen[i in k]: money PLN = chain[3] + bonus at when_of[i]

def flagged[i in k]: int = flag at when_of[i]

def taxed_at[i in k]: money PLN = taxed at when_of[i]

version v26
  cite card
  governs 2026
  known 2026-01-01
  fee = 10.00
  mode = 1
  tax: [0, inf) = round(v * 0.1, 0.01, half-up)

version v27
  cite card
  governs 2027..
  known 2026-01-01
  fee = 20.00
  mode = 2
  tax: [0, inf) = round(v * 0.2, 0.01, half-up)

version dates
  cite card
  governs always
  known 2026-01-01
  when_of[1] = 2025-06-01
  when_of[2] = 2025-07-01
  when_of[3] = 2026-03-01
  when_of[4] = 2026-09-01
  when_of[5] = 2027-03-01
  when_of[6] = 2027-06-01

ask s: seen[i] for i in [1, 2, 3, 4, 5, 6]

ask f: flagged[i] for i in [1, 2, 3, 4, 5, 6]

ask t: taxed_at[i] for i in [1, 2, 3, 4, 5, 6]
"#;

    /// refcore's lines for the text.
    const LINES: [&str; 44] = [
        "f (i = 1): ?\"mode: no value at 2025-06-01\" (mode)",
        "  cut 2026-06-30",
        "f (i = 2): ?\"mode: no value at 2025-07-01\" (mode)",
        "  cut 2026-06-30",
        "f (i = 3): 1",
        "  cut 2026-06-30",
        "f (i = 4): 1",
        "  cut 2026-06-30",
        "f (i = 5): refused: RUN-NO-RULE no rule applies to flag on 2027-03-01",
        "  held: mode = 2 (v27), read for 2027-03-01",
        "  cut 2026-06-30",
        "f (i = 6): refused: RUN-NO-RULE no rule applies to flag on 2027-06-01",
        "  held: mode = 2 (v27), read for 2027-06-01",
        "  cut 2026-06-30",
        "s (i = 1): ?\"fee: no value at 2025-06-01\" (fee)",
        "  cut 2026-06-30",
        "s (i = 2): ?\"fee: no value at 2025-07-01\" (fee)",
        "  cut 2026-06-30",
        "s (i = 3): 30.00 PLN",
        "  cut 2026-06-30",
        "s (i = 4): 30.00 PLN",
        "  cut 2026-06-30",
        "s (i = 5): 80.00 PLN",
        "  held: fee = 20.00 PLN (v27), read for 2027-03-01",
        "  cut 2026-06-30",
        "s (i = 6): 80.00 PLN",
        "  held: fee = 20.00 PLN (v27), read for 2027-06-01",
        "  cut 2026-06-30",
        "t (i = 1): ?\"fee: no value at 2025-06-01\" (fee)",
        "  cut 2026-06-30",
        "t (i = 2): ?\"fee: no value at 2025-07-01\" (fee)",
        "  cut 2026-06-30",
        "t (i = 3): 3.00 PLN",
        "  cut 2026-06-30",
        "t (i = 4): 3.00 PLN",
        "  cut 2026-06-30",
        "t (i = 5): 12.00 PLN",
        "  held: fee = 20.00 PLN (v27), read for 2027-03-01",
        "  held: tax row [0, inf) (v27), read for 2027-03-01",
        "  cut 2026-06-30",
        "t (i = 6): 12.00 PLN",
        "  held: fee = 20.00 PLN (v27), read for 2027-06-01",
        "  held: tax row [0, inf) (v27), read for 2027-06-01",
        "  cut 2026-06-30",
    ];

    /// The run's answer with the memo keeping cells as `memo` says.
    fn answer(memo: Memo) -> Answer {
        let given = vec![Given {
            path: "r.tally".to_owned(),
            bytes: Some(REALITY.as_bytes().to_vec()),
        }];
        let request = Request {
            asks: Asks::Every,
            pin: Pin::Canonical,
            days: Days::Events,
        };
        run_with_memo(given, &request, memo).expect("an answer")
    }

    #[test]
    fn cells_shared_by_law_date_class_answer_as_refcore() {
        assert_eq!(answer(Memo::Classes).lines(), LINES, "refcore's lines");
    }

    #[test]
    fn cells_kept_at_each_law_date_answer_as_refcore() {
        assert_eq!(answer(Memo::Exact).lines(), LINES, "the twin's lines");
    }

    /// The `def_misses` count of a counted run's statistics line.
    #[cfg(feature = "counters")]
    fn misses(answer: &Answer) -> u64 {
        let line = answer.counters().line();
        let count = line
            .split(' ')
            .find_map(|part| part.strip_prefix("def_misses="))
            .expect("a def_misses count");
        count.parse().expect("a number")
    }

    #[cfg(feature = "counters")]
    #[test]
    fn a_class_computes_its_cells_once() {
        let shared = misses(&answer(Memo::Classes));
        let exact = misses(&answer(Memo::Exact));
        assert!(
            shared < exact,
            "{shared} misses shared, {exact} at each law date"
        );
    }
}
