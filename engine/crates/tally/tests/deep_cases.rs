//! The builder's work-stack tests: chains of def cells far past the counted depth answer as refcore answers them, on a 2 MiB thread.
//! Spec: E §3.2 (a recurrence needs no deep stack); L §5.11 (recurrences; a chain that returns
//! to a cell still being evaluated is `RUN-CYCLE` at that key); L §5.18 (the printed asks).
//! Twin: engine/tools/refcore/evaluate.py (`def_cell`); each expected line is refcore's run of
//! the same text (`python3 -m refcore run`), which recurses on a 512 MB stack.
//! Invariants: reads no test, vector or answer key (R-93); the texts are written here; each run
//! is on its test's thread, which libtest starts with 2 MiB as it does T's runner's (no
//! `RUST_MIN_STACK` is set), so a chain that recursed would overflow it.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally::{Asks, Days, Given, Pin, Request, run};

    const HEAD: &str = "model deep
  tally 2
  on 2026-01-01
  start 2026-01-01
  end 2026-12-31

source card \"a card\"
";

    /// A recurrence 3 000 keys deep.
    const CHAIN: &str = "
index n = int 1..3000

def acc[k in n]: int
  rule first = 1 when k == 1
  rule next = acc[k - 1] + 1 when k > 1

ask top: acc[3000], acc[1500]
";

    /// A loan's schedule: three defs reading each other at the same and the earlier key, 600
    /// keys deep, through `round`, `min` and `max`.
    const LOAN: &str = "
unit PLN: currency minor 0.01

index inst = int 1..600

input principal: money PLN

input rate: num

input instalment: money PLN

def debt[k in inst]: money PLN
  rule first = principal when k == 1
  rule later = debt[k - 1] - paid[k - 1] when k > 1

def interest[k in inst]: money PLN = round(debt[k] * rate, minor(PLN), half-up)

def paid[k in inst]: money PLN = max(0, min(debt[k], instalment - interest[k]))

version v
  cite card
  governs always
  known 2026-01-01
  principal = 100000.00
  rate = 0.002
  instalment = 300.00

ask end: debt[600], paid[600], interest[600]

ask mid: debt[k] for k in [1, 250, 599]
";

    /// A ring of 150 computed keys: every read returns to the cell it started from.
    fn cycle() -> String {
        let mut rows = Vec::new();
        for (k, next) in (1..=150).zip((2..=150).chain([1])) {
            rows.push(format!("  nxt[{k}] = {next}"));
        }
        format!(
            "
index n = int 1..150

input nxt[k in n]: key n

def hop[k in n]: int = hop[nxt[k]] + 1

version v
  cite card
  governs always
  known 2026-01-01
{}

ask loop: hop[1]

ask other: hop[75]
",
            rows.join("\n")
        )
    }

    /// The lines a run of `body` prints, on the test's own thread: libtest's, 2 MiB.
    fn lines(body: &str) -> Vec<String> {
        let given = vec![Given {
            path: "r.tally".to_owned(),
            bytes: Some(format!("{HEAD}{body}").into_bytes()),
        }];
        let request = Request {
            asks: Asks::Every,
            pin: Pin::Canonical,
            days: Days::Events,
        };
        let answer = run(given, &request).expect("an answer");
        answer.lines().to_vec()
    }

    #[test]
    fn a_recurrence_far_past_the_depth_answers() {
        assert_eq!(
            lines(CHAIN),
            [
                "top: (3 000, 1 500)",
                "  cut 2026-01-01",
                "warning: WARN-UNREAD source card is read by nothing",
            ],
            "refcore's lines"
        );
    }

    #[test]
    fn defs_reading_each_other_down_a_schedule_answer() {
        assert_eq!(
            lines(LOAN),
            [
                "end: (0.00 PLN, 0.00 PLN, 0.00 PLN)",
                "  cut 2026-01-01",
                "mid (k = 1): 100 000.00 PLN",
                "  cut 2026-01-01",
                "mid (k = 250): 67 769.52 PLN",
                "  cut 2026-01-01",
                "mid (k = 599): 0.00 PLN",
                "  cut 2026-01-01",
            ],
            "refcore's lines"
        );
    }

    #[test]
    fn a_cycle_longer_than_the_depth_is_refused_at_the_cell_read() {
        assert_eq!(
            lines(&cycle()),
            [
                "loop: refused: RUN-CYCLE hop[1] reads itself through a computed key",
                "  cut 2026-01-01",
                "other: refused: RUN-CYCLE hop[75] reads itself through a computed key",
                "  cut 2026-01-01",
            ],
            "refcore's lines"
        );
    }
}
