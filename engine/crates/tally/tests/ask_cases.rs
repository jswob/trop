//! The builder's ask tests: a small reality's asks print as refcore prints them, line for line, with the register under each.
//! Spec: L §5.18 (asks: `law`, `as known`, `in`, `for` with `where`, one line per binding tuple,
//! a tuple whose `where` is a hole printed as that hole, a line's values), §5.11 (an ask's
//! `law` that is a hole makes its line that hole), §5.7 (`x in K on d`: a hole `d` is that
//! hole; §8.4 `RUN-NONE` for `none` and `never`), §5.19 Bindings (movements by day, a hole
//! day last), §8.2 (the register: `assumed`, `held`, `world`, the law date and the cut; a
//! tuple's `unfulfilled:` on its own line), §8.4 (a run-time refusal stands as its value).
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

    /// Three movements over an event kind: a tip in January, rent in February, and a fee
    /// whose day is a hole; the hole day's id comes first in text order.
    const MOVES: &str = r#"model mb
  tally 2
  on 2025-03-10
  start 2025-01-01
  end 2025-03-31

source card "a card"

unit DUK: currency minor 0.01

kind ext: place DUK signed outside

place out: ext lives 2025-01-01..

place bank: accounts DUK lives 2025-01-01..

input fee_day: date question "When is the fee charged?"

input rent_day: date

input tip_day: date

input fee_amt: money DUK

input rent_amt: money DUK

input tip_amt: money DUK

input limit: money DUK

move fee: transfers
  on fee_day
  leg cash from bank to out amount fee_amt

move rent: transfers
  on rent_day
  leg cash from bank to out amount rent_amt

move tip: transfers
  on tip_day
  leg cash from out to bank amount tip_amt

version facts
  cite card
  governs always
  known 2025-01-01
  rent_day = 2025-02-03
  fee_amt = 15.00
  rent_amt = 300.00
  tip_amt = 40.00
  limit = 100.00
  tip_day = 2025-01-14
  opening[bank] = 1000.00 at 2025-01-01 start

ask members: m.cash.amount for m in transfers over 2025-01 .. 2025-03

ask total: count(m in transfers over 2025-01 .. 2025-03)

ask pairs: m.cash.amount + n.cash.amount for m in transfers over 2025-01, n in transfers over 2025-02 .. 2025-03

ask big: m.cash.amount for m in transfers over 2025-01 .. 2025-03 where m.cash.amount > limit
"#;

    /// refcore's lines for `MOVES`: a member whose day is a hole comes after every dated one;
    /// each tuple's line has the register of the movements it binds, and only theirs.
    const MOVES_LINES: &str = r#"big (m = rent): 300.00 DUK
  unfulfilled: rent
  cut 2025-03-10
members (m = tip): 40.00 DUK
  unfulfilled: tip
  cut 2025-03-10
members (m = rent): 300.00 DUK
  unfulfilled: rent
  cut 2025-03-10
members (m = fee): ?"When is the fee charged?" (fee_day)
pairs (m = tip, n = rent): 340.00 DUK
  unfulfilled: rent
  unfulfilled: tip
  cut 2025-03-10
pairs (m = tip, n = fee): ?"When is the fee charged?" (fee_day)
pairs (m = fee, n = rent): ?"When is the fee charged?" (fee_day)
pairs (m = fee, n = fee): ?"When is the fee charged?" (fee_day)
total: ?"When is the fee charged?" (fee_day)
  unfulfilled: rent
  unfulfilled: tip
  cut 2025-03-10"#;

    /// The lines a run of `text` prints, every ask, the walk stepping `days`.
    fn lines_of(text: &str, days: Days) -> Vec<String> {
        let given = vec![Given {
            path: "r.tally".to_owned(),
            bytes: Some(text.as_bytes().to_vec()),
        }];
        let request = Request {
            asks: Asks::Every,
            pin: Pin::Canonical,
            days,
        };
        let answer = run(given, &request).expect("an answer");
        answer.lines().to_vec()
    }

    #[test]
    fn movement_members_come_by_day_a_hole_day_last_each_line_with_its_own_register() {
        let want: Vec<&str> = MOVES_LINES.lines().collect();
        assert_eq!(lines_of(MOVES, Days::Events), want, "refcore's lines");
        assert_eq!(
            lines_of(MOVES, Days::Every),
            want,
            "the every-day walk's lines"
        );
    }

    /// Dated memberships of a kind tested `on` a day: a day, a hole, `never`, and `none` (a
    /// date input read at a key outside its thing's lifespan, with no row).
    const MEMBERSHIP: &str = r#"model mt
  tally 2
  on 2026-03-01
  start 2026-01-01
  end 2026-12-31

source card "a card"

kind person

kind adult: person

input joined[p in adult]: date

input check_day: date question "When is the check?"

input stop_day: date

version v1
  cite card
  governs always
  known 2026-01-01
  adult + [ana, bob] lives 2026-01-01..
  adult + [cid, dan] lives 2026-02-01..
  joined[ana] = 2026-01-10
  joined[cid] = 2026-02-05
  stop_day = never

ask known_day: ana in adult on joined[ana]

ask late_member: cid in adult on joined[cid]

ask none_day: cid in adult on joined[dan] law 2026-01-15

ask hole_day: bob in adult on check_day

ask never_day: ana in adult on stop_day

ask early_member: cid in adult on joined[ana]
"#;

    /// refcore's lines for [`MEMBERSHIP`]: a hole day makes the test that hole; `none` and
    /// `never` are refused `RUN-NONE`, never printed as the test's value.
    const MEMBERSHIP_LINES: &str = r#"early_member: false
  cut 2026-03-01
hole_day: ?"When is the check?" (check_day)
  cut 2026-03-01
known_day: true
  cut 2026-03-01
late_member: true
  cut 2026-03-01
never_day: refused: RUN-NONE never in the day of a membership test
  cut 2026-03-01
none_day: refused: RUN-NONE a value is none here: the day of a membership test
  law date 2026-01-15, cut 2026-03-01"#;

    #[test]
    fn a_membership_tested_on_a_value_that_is_no_day_is_refused() {
        let want: Vec<&str> = MEMBERSHIP_LINES.lines().collect();
        assert_eq!(lines_of(MEMBERSHIP, Days::Events), want, "refcore's lines");
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
