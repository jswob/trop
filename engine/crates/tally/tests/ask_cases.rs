//! The builder's ask tests: a small reality's asks print as refcore prints them, line for line, with the register under each.
//! Spec: L §5.18 (asks: `law`, `as known`, `in`, `for` with `where`, one line per binding tuple,
//! a tuple whose `where` is a hole printed as that hole, a line's values), §5.11 (an ask's
//! `law` that is a hole makes its line that hole), §8.2 (the register: `assumed`, `held`,
//! `world`, the law date and the cut), §8.4 (a run-time refusal stands as its value).
//! Twin: engine/tools/refcore/ask.py (`Asker`); the expected lines are refcore's run of the
//! same text (`python3 -m refcore run`).
//! Invariants: reads no test, vector or answer key (R-93); the text is written here, small.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally::{Asks, Days, Given, Pin, Request, run};

    /// Inputs with rows from a version with an `assumed` reason, a row governing from a later
    /// day, a row known after an `as known` cut, holes, a division by zero.
    const REALITY: &str = r#"model m
  tally 2
  on 2026-03-01
  start 2026-01-01
  end 2026-12-31

source card "a card"

unit PLN: currency minor 0.01

index month = month 2026-01..2026-06

input pay[m in month]: money PLN

input fee: money PLN

input rate: num

input lawday: date

def doubled[m in month]: money PLN
  rule base = pay[m] + pay[m]

version v1
  cite card
  governs always
  known 2026-01-01
  assumed "a guess until the statement comes"
  pay[2026-01] = 100.00
  pay[2026-02] = 50.00
  rate = 0

version v2
  cite card
  governs 2026-05-01..
  known 2026-01-01
  pay[2026-05] = 70.00

version v3
  cite card
  governs always
  known 2026-02-01
  pay[2026-03] = 30.00

ask plain: pay[2026-01], fee

ask each: doubled[m] for m in month where pay[m] > fee

ask some: doubled[m] for m in month where pay[m] > 0

ask later: pay[2026-05] law 2026-05-01

ask past: pay[2026-03] as known 2026-01-15

ask nolaw: pay[2026-01] law lawday

ask div: pay[2026-01], 1 / rate

ask listed: pay[m] for m in [2026-02, 2026-01]

ask base: fee in m
"#;

    /// refcore's lines for the reality, every ask, in id order.
    const LINES: &str = r#"base: ?"fee: no value" (fee)
  world: m
  cut 2026-03-01
div: (100.00 PLN, refused: RUN-DIV division by zero)
  assumed: a guess until the statement comes (pay[2026-01], rate)
  cut 2026-03-01
each (m = 2026-01): ?"fee: no value" (fee)
each (m = 2026-02): ?"fee: no value" (fee)
each (m = 2026-03): ?"fee: no value" (fee)
each (m = 2026-04): ?"fee: no value" (fee) ?"pay at 2026-04: no value for 2026-04" (pay[2026-04])
each (m = 2026-05): ?"fee: no value" (fee)
each (m = 2026-06): ?"fee: no value" (fee) ?"pay at 2026-06: no value for 2026-06" (pay[2026-06])
later: 70.00 PLN
  held: pay[2026-05] = 70.00 PLN (v2), read for 2026-05
  law date 2026-05-01, cut 2026-03-01
listed (m = 2026-02): 50.00 PLN
  assumed: a guess until the statement comes (pay[2026-02])
  cut 2026-03-01
listed (m = 2026-01): 100.00 PLN
  assumed: a guess until the statement comes (pay[2026-01])
  cut 2026-03-01
nolaw: ?"lawday: no value" (lawday)
past: ?"pay at 2026-03: no value for 2026-03" (pay[2026-03])
  cut 2026-01-15
plain: (100.00 PLN, ?"fee: no value" (fee))
  assumed: a guess until the statement comes (pay[2026-01])
  cut 2026-03-01
some (m = 2026-01): 200.00 PLN
  assumed: a guess until the statement comes (pay[2026-01])
  cut 2026-03-01
some (m = 2026-02): 100.00 PLN
  assumed: a guess until the statement comes (pay[2026-02])
  cut 2026-03-01
some (m = 2026-03): 60.00 PLN
  cut 2026-03-01
some (m = 2026-04): ?"pay at 2026-04: no value for 2026-04" (pay[2026-04])
some (m = 2026-05): 140.00 PLN
  held: pay[2026-05] = 70.00 PLN (v2), read for 2026-05
  cut 2026-03-01
some (m = 2026-06): ?"pay at 2026-06: no value for 2026-06" (pay[2026-06])"#;

    /// The lines a run of the reality prints for `asks`.
    fn lines(asks: Asks) -> Vec<String> {
        let given = vec![Given {
            path: "r.tally".to_owned(),
            bytes: Some(REALITY.as_bytes().to_vec()),
        }];
        let request = Request {
            asks,
            pin: Pin::Canonical,
            days: Days::Events,
        };
        let answer = run(given, &request).expect("an answer");
        answer.lines().to_vec()
    }

    #[test]
    fn every_ask_prints_as_refcore_prints_it() {
        let want: Vec<&str> = LINES.lines().collect();
        assert_eq!(lines(Asks::Every), want, "refcore's lines");
    }

    #[test]
    fn the_asks_named_print_in_the_order_named() {
        let named = Asks::Named(vec!["nolaw".to_owned(), "later".to_owned()]);
        assert_eq!(
            lines(named),
            [
                "nolaw: ?\"lawday: no value\" (lawday)",
                "later: 70.00 PLN",
                "  held: pay[2026-05] = 70.00 PLN (v2), read for 2026-05",
                "  law date 2026-05-01, cut 2026-03-01",
            ],
            "refcore's lines for `--ask nolaw later`"
        );
    }
}
