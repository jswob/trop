//! The builder's tests of group 3's review: each review input, the lines refcore prints for the asks it names.
//! Spec: L §5.14.1 and E §4.3 step 2 (a projection counts every leg dated t through d,
//! tally 2.1.8 R16, #183; pick P39 on B2), L §5.18 (R15: `partial` follows what is linear),
//! L §5.10 and §4.6 (R14: a place of unit `any` read unit by unit), L §4.6 Readings and §4.4
//! (#175: a reading `after m`, its key, its window day, pick P29), L §8 (refusals outrank
//! holes), L §5.19 (`bal … modelled`, pick P39 on B8).
//! Twin: engine/tools/refcore (spec/2.1.8 at 75f27ca1, the sixth turn); each expected line is
//! its run of the same text (`python3 -m refcore run`), each text an input of the review of
//! build group 3 (`tmp/spec-2.1.8-build/review-g3/`).
//! Invariants: reads no test, vector or answer key (R-93); the texts are written here.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally::{Asks, Days, Given, Pin, Request, run};

    /// The lines a run of `text` prints, every ask.
    fn lines_of(text: &str) -> Vec<String> {
        let given = vec![Given {
            path: "r.tally".to_owned(),
            bytes: Some(text.as_bytes().to_vec()),
        }];
        let request = Request {
            asks: Asks::Every,
            pin: Pin::Canonical,
            days: Days::Events,
        };
        let answer = run(given, &request).expect("an answer");
        answer.lines().to_vec()
    }

    /// Each of `expected` is the line of `lines` with its head (the text before `: `).
    fn holds(lines: &[String], expected: &[&str]) {
        for want in expected {
            let head = want.split(": ").next().expect("a head");
            let got = lines
                .iter()
                .find(|l| l.split(": ").next() == Some(head))
                .unwrap_or_else(|| panic!("no line {head} in {lines:?}"));
            assert_eq!(got, want, "the line {head}");
        }
    }

    /// B1 (`t13.tally`): a decision run before t whose leg is dated t, read at a point of t after it: the start holds it.
    #[test]
    fn a_base_leg_the_start_holds_is_counted_once() {
        let text = r#"model p
  tally 2
  on 2026-01-01
  start 2026-01-01
  end 2026-12-31
source card "probe"
unit DUK: currency minor 0.01
place konto: accounts DUK lives 2026-01-01..
place world: outside lives 2026-01-01..
input amt: money DUK
input floor: money DUK
input d_mv: date
input d_leg: date
input d_late: date
move early: transfers
  on d_mv
  reads konto
  leg cash from world to konto amount amt + bal(konto) on d_leg
move late: transfers
  on d_late
  reads konto
  leg cash from world to konto amount max(0 DUK, floor - bal(konto, d_late) from d_late)
ask a: bal(konto, 2026-03-31), bal(konto, 2026-03-01), bal(konto, 2026-02-28)
ask b: bal(konto, 2026-03-31) from 2026-02-01, bal(konto, 2026-03-31) from 2026-03-01
version v
  cite card
  governs always
  known 2026-01-01
  amt = 100.00 DUK "c"
  floor = 500.00 DUK "c"
  d_mv = 2026-01-15 "c"
  d_leg = 2026-03-01 "c"
  d_late = 2026-03-01 "c"
  opening[konto] = 0.00 DUK at 2026-01-01 start "c"
"#;
        holds(
            &lines_of(text),
            &[
                "a: (500.00 DUK, 500.00 DUK, 0.00 DUK)",
                "b: (500.00 DUK, 500.00 DUK)",
            ],
        );
    }

    /// B2 (`t17.tally`): a decision run earlier on day t: its leg dated after t is counted (pick P39).
    #[test]
    fn a_decision_run_earlier_on_t_counts_its_later_legs() {
        let text = r#"model p
  tally 2
  on 2026-01-01
  start 2026-01-01
  end 2026-12-31
source card "probe"
unit DUK: currency minor 0.01
place konto: accounts DUK lives 2026-01-01..
place world: outside lives 2026-01-01..
place sav: accounts DUK lives 2026-01-01..
input amt: money DUK
input floor: money DUK
input d_mv: date
input d_leg: date
input d_late: date
input d_to: date
move early: transfers
  on d_mv
  reads sav
  leg cash from world to konto amount amt + bal(sav) on d_leg
move late: transfers
  on d_late
  reads konto
  leg cash from world to konto amount max(0 DUK, floor - bal(konto, d_to) from d_late)
ask a: bal(konto, 2026-03-31), bal(konto, 2026-03-01), bal(konto, 2026-02-28)
ask b: bal(konto, 2026-03-31) from 2026-02-01, bal(konto, 2026-03-31) from 2026-03-01
version v
  cite card
  governs always
  known 2026-01-01
  amt = 100.00 DUK "c"
  floor = 500.00 DUK "c"
  d_mv = 2026-03-01 "c"
  d_leg = 2026-03-10 "c"
  d_late = 2026-03-01 "c"
  d_to = 2026-03-31 "c"
  opening[konto] = 0.00 DUK at 2026-01-01 start "c"
  opening[sav] = 0.00 DUK at 2026-01-01 start "c"
"#;
        holds(
            &lines_of(text),
            &[
                "a: (500.00 DUK, 400.00 DUK, 0.00 DUK)",
                "b: (500.00 DUK, 500.00 DUK)",
            ],
        );
    }

    /// B3 (`t8.tally`): `with uncertain partial` over a projection whose balance is a hole: the uncertain repayments stay in its known part.
    #[test]
    fn a_projection_with_uncertain_partial_keeps_the_uncertain_amounts() {
        let text = r#"model u
  tally 2
  on 2026-01-01
  start 2026-01-01
  end 2026-12-31

source own "own"
unit PLN: currency minor 0.01
kind promises: event uncertain
  leg cash: PLN
kind gifts: event certain
  leg cash: PLN
place konto: accounts PLN lives 2020-01-01..
place friend: outside lives 1900-01-01..
input owed[m in months]: money PLN
input repay_dom: int
input gift[m in months]: money PLN

move repay[m in months]: promises
  on day_of(m, repay_dom)
  at start_of(m)
  leg cash from friend to konto amount owed[m]

move g[m in months]: gifts
  on day_of(m, repay_dom)
  at start_of(m)
  leg cash from friend to konto amount gift[m]

version v
  cite own
  governs 2026..
  known 2026-01-01
  owed = 500.00 PLN "note"
  repay_dom = 1 "note"
  gift = 10.00 PLN "note"
  gift[2026-03] = ?"gift in march" "note"
  opening[konto] = 1 000.00 PLN at 2026-01-01 start "statement"

ask a: bal(konto, 2026-04-30) from 2026-02-15 with uncertain partial
ask b: bal(konto, 2026-04-30) with uncertain partial
ask c: bal(konto, 2026-04-30) from 2026-02-15 with uncertain
ask d: bal(konto, 2026-04-30) from 2026-02-15 partial
"#;
        holds(
            &lines_of(text),
            &["a: 2 030.00 PLN + 1 × ?\"gift in march\" (gift[2026-03])"],
        );
    }

    /// B4 (`t1_la.tally`): `last` at an instant where a unitless hole stands beside a EUR reading, the place holding EUR and PLN.
    #[test]
    fn last_beside_a_unitless_hole_and_a_units_reading_is_run_domain() {
        let text = r#"model t
  tally 2
  on 2026-06-10
  start 2026-01-01
  end 2026-06-30

source s "s"
unit PLN: currency minor 0.01
unit EUR: currency minor 0.01
kind wallets: place any signed
place broker: wallets lives 2025-01-01..

version v1
  cite s
  governs always
  known 2026-01-01
  anchor broker
  opening[broker] = 1.00 EUR at 2026-01-01 start "s"
  opening[broker] = 2.00 PLN at 2026-01-01 start "s"
  stated[broker] = 4.00 EUR at 2026-02-15 end "s"


version v2
  cite s
  governs always
  known 2026-03-01
  anchor broker
  stated[broker] = ?"not read" at 2026-02-15 end "s"

ask la: last(stated[broker], 2026-02-16)
"#;
        holds(
            &lines_of(text),
            &[
                "la: refused: RUN-DOMAIN last of broker: its readings at 2026-02-15 end are in EUR and PLN",
            ],
        );
    }

    /// B4 (`t1_pl.tally`): a plain read at the law date of that instant, as `last`.
    #[test]
    fn a_law_read_beside_a_unitless_hole_and_a_units_reading_is_run_domain() {
        let text = r#"model t
  tally 2
  on 2026-06-10
  start 2026-01-01
  end 2026-06-30

source s "s"
unit PLN: currency minor 0.01
unit EUR: currency minor 0.01
kind wallets: place any signed
place broker: wallets lives 2025-01-01..

version v1
  cite s
  governs always
  known 2026-01-01
  anchor broker
  opening[broker] = 1.00 EUR at 2026-01-01 start "s"
  opening[broker] = 2.00 PLN at 2026-01-01 start "s"
  stated[broker] = 4.00 EUR at 2026-02-15 end "s"


version v2
  cite s
  governs always
  known 2026-03-01
  anchor broker
  stated[broker] = ?"not read" at 2026-02-15 end "s"

ask pl: stated[broker] law 2026-02-15
"#;
        holds(
            &lines_of(text),
            &[
                "pl: refused: RUN-DOMAIN stated of broker: its readings at 2026-02-15 end are in EUR and PLN",
            ],
        );
    }
}
