//! The builder's tests of tally 2.2's `same` lines: one between two observed rows has no day of its own, one with a planned side keeps its version's.
//! Spec: L §5.19 One movement on two statements (tally 2.2, S-5: a line between two observed
//! rows holds, in the worlds its file is in, at every cut that sees both rows, whichever
//! version states it), §4.5 (R12: any other `same` line of a version is seen under a cut only
//! when known by it), §6.1 (rows stated `same` are one movement, counted once).
//! Twin: engine/tools/refcore (spec/2.2 at b9eb505c, G3's snapshot): `moves.py` `same_lines`,
//! `obs_row_seen`; each expected line is its run of the same text (`python3 -m refcore run`),
//! the texts the review's `s3`, a variant with the line in a third version, and the critic's
//! `s5mm` (`tmp/spec-2.2/inputs/review/probes/`, `tmp/spec-2.2/critic-scratch-2.2/`).
//! Invariants: reads no test, vector or answer key (R-93); the texts are written here.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally::{Asks, Days, Given, Pin, Request, run};

    /// The lines a run of `text`, read from `path`, prints for the ask `ask`.
    fn asked(path: &str, text: &str, ask: &str) -> Vec<String> {
        let given = vec![Given {
            path: path.to_owned(),
            bytes: Some(text.as_bytes().to_vec()),
        }];
        let request = Request {
            asks: Asks::Named(vec![ask.to_owned()]),
            pin: Pin::Canonical,
            days: Days::Events,
        };
        let answer = run(given, &request).expect("an answer");
        answer.lines().to_vec()
    }

    /// One transfer from konto to sav on two statements: konto's known 2026-04-02, sav's
    /// 2026-04-05. `{konto}` is a line konto's statement adds; `{link}`, a version after both.
    const TWO_ROWS: &str = r#"model s3
  tally 2
  on 2026-04-10
  start 2026-03-01
  end 2026-04-30

source s "s"
unit PLN: currency minor 0.01
place konto: accounts PLN lives 2026-01-01..
place sav: accounts PLN lives 2026-01-01..
place world: outside lives 1900-01-01..
input bank: observed
input sbank: observed

version v
  cite s
  governs always
  known 2026-01-01
  opening[konto] = 2000.00 PLN at 2026-03-01 start "s"
  opening[sav] = 0.00 PLN at 2026-03-01 start "s"

version st_konto
  cite s
  governs 2026-03
  known 2026-04-02
  covers konto
  bank[a2] = (on: 2026-03-18, kind: transfers, leg cash from konto to sav amount 950.00 PLN) "konto line 2"
{konto}
version st_sav
  cite s
  governs 2026-03
  known 2026-04-05
  covers sav
  sbank[b1] = (on: 2026-03-18, kind: transfers, leg cash from konto to sav amount 950.00 PLN) "sav line 1"
{link}
ask between: bal(konto, 2026-03-31), bal(sav, 2026-03-31), count(m in transfers over 2026-03), flows(konto, 2026-03) as known {cut}
"#;

    /// The review's `s3`: konto's statement states the line, asked as known 2026-04-03, when
    /// sav's row isn't known yet. The line isn't in force there, so the one row is read alone,
    /// where 2.1.8 refused it (`RUN-RECORD same bank[a2] = sbank[b1]: sbank[b1] doesn't
    /// exist`).
    #[test]
    fn a_line_seen_with_one_of_its_rows_is_not_in_force() {
        let text = TWO_ROWS
            .replace("{konto}", "  same bank[a2] = sbank[b1]\n")
            .replace("{link}", "")
            .replace("{cut}", "2026-04-03");
        assert_eq!(
            asked("s3.tally", &text, "between"),
            [
                "between: (1 050.00 PLN, 950.00 PLN, 1, -950.00 PLN)",
                "  cut 2026-04-03",
                "warning: WARN-UNREAD place world is read by nothing"
            ],
            "the line waits for sav's row: konto's row read alone"
        );
    }

    /// The line in a third version known 2026-04-08, asked as known 2026-04-06, when both rows
    /// are known: the line holds, so the transfer is counted once, where 2.1.8 read the rows
    /// apart (a count of 2).
    #[test]
    fn a_line_between_two_observed_rows_holds_before_its_version_is_known() {
        let link = "\nversion link\n  cite s\n  governs 2026-03\n  known 2026-04-08\n  same bank[a2] = sbank[b1]\n";
        let text = TWO_ROWS
            .replace("{konto}", "")
            .replace("{link}", link)
            .replace("{cut}", "2026-04-06");
        assert_eq!(
            asked("late.tally", &text, "between"),
            [
                "between: (1 050.00 PLN, 950.00 PLN, 1, -950.00 PLN)",
                "  cut 2026-04-06",
                "warning: WARN-UNREAD place world is read by nothing"
            ],
            "both rows seen: the line holds before its version's known"
        );
    }

    /// The critic's `s5mm`: two planned moves of 100.00 PLN on one day, stated `same` by a
    /// version known 2026-05-01. A line with a planned instance on a side keeps its version's
    /// `known`: as known 2026-04-01 the two count apart, at the cut once.
    const PLANNED: &str = r#"model s5mm
  tally 2
  on 2026-06-10
  start 2026-01-01
  end 2026-12-31

source s "s"
unit PLN: currency minor 0.01
place wallet: accounts PLN lives 2025-01-01..
place world: outside lives 1900-01-01..
input d1: date
input a1: money PLN

move in1: transfers
  on d1
  leg cash from world to wallet amount a1
move in2: transfers
  on d1
  leg cash from world to wallet amount a1

version v
  cite s
  governs always
  known 2025-12-01
  d1 = 2026-03-05 "s"
  a1 = 100.00 PLN "s"
  opening[wallet] = 0.00 PLN at 2026-01-01 start "s"

version later
  cite s
  governs always
  known 2026-05-01
  same in1 = in2

ask today: bal(wallet, 2026-03-31)
ask before: bal(wallet, 2026-03-31) as known 2026-04-01
"#;

    #[test]
    fn a_line_with_a_planned_side_is_seen_by_its_versions_known() {
        assert_eq!(
            asked("s5mm.tally", PLANNED, "before"),
            [
                "before: 200.00 PLN",
                "  unfulfilled: in1",
                "  unfulfilled: in2",
                "  cut 2026-04-01"
            ],
            "the line isn't known yet: two movements"
        );
        assert_eq!(
            asked("s5mm.tally", PLANNED, "today"),
            [
                "today: 100.00 PLN",
                "  unfulfilled: in2",
                "  cut 2026-06-10"
            ],
            "the line is known: one movement"
        );
    }
}
