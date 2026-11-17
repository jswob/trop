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
}
