//! The builder's projection tests: a small reality's written projections print as refcore prints them, under both day sources.
//! Spec: L §5.14.1 (The projection is written: from the main walk at the start of t, or from a
//! decision's walk at its point or at the start of an earlier t; the movements counted as known
//! on t; those that depend on the holder left out and listed; `with uncertain`; `HOLE-PRERUN`;
//! `RUN-FUTURE`), E §4.3 (decisions in the window run in the projection's own walk; nesting),
//! L §8.2 (`left out:`), §8.5 (a projection's walk prints no warning); P §2.2 (the oracle).
//! Twin: engine/tools/refcore/mreads.py (`proj_bal`) and walk.py (`proj_walk`); the expected
//! lines are refcore's run of the same text (`python3 -m refcore run`).
//! Invariants: reads no test, vector or answer key (R-93); the text is written here, small.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally::{Asks, Days, Given, Pin, Request, run};

    /// A sweep decision reading a def's projection, a decision reading its own projection, one
    /// reading a projection from an earlier day, one from a later day, a row that changes a
    /// wage after the sweep's day, a fee that drives a projection below zero, an uncertain
    /// bonus.
    const REALITY: &str = r#"model pj
  tally 2
  on 2025-02-10
  start 2025-01-01
  end 2025-03-31

source card "a card"

unit DUK: currency minor 0.01

kind ext: place DUK signed outside

kind promises: transfers uncertain

place out: ext lives 2025-01-01..

place bank: accounts DUK lives 2025-01-01..

place save: accounts DUK lives 2025-01-01..

place spare: accounts DUK lives 2025-01-01..

input pay: money DUK

input wage_amt: money DUK

input bonus_amt: money DUK

input target: money DUK

input floor: money DUK

input rent_day: date

input wage_day: date

input bonus_day: date

input sweep_day: date

input top_day: date

input late_day: date

input skim_day: date

input fee_day: date

input fee_amt: money DUK

input reach: date

input top_end: date

input late_end: date

input gift_day: date question "When does the gift come?"

move rent: transfers
  on rent_day
  leg cash from bank to out amount pay

move wage: transfers
  on wage_day
  leg cash from out to bank amount wage_amt

move bonus: promises
  on bonus_day
  leg cash from out to bank amount bonus_amt

def room: money DUK at sweep_day
  rule base = min(bal(bank, d) from sweep_day for d in sweep_day .. reach step 1 day)

move sweep: transfers
  on sweep_day
  reads bank
  leg cash from bank to save amount max(0, room - floor)

move top: transfers
  on top_day
  reads save
  leg cash from out to save amount max(0, target - bal(save, top_end) from top_day)

move skim: transfers
  on skim_day
  reads bank
  leg cash from bank to save amount max(0, bal(bank, skim_day) from rent_day - floor)

move fee: transfers
  on fee_day
  leg cash from bank to out amount fee_amt

move late: transfers
  on late_day
  reads spare
  leg cash from spare to out amount bal(spare, late_end) from add_days(late_day, 1)

version facts
  cite card
  governs always
  known 2025-01-01
  pay = 400.00
  wage_amt = 600.00
  bonus_amt = 250.00
  target = 500.00
  floor = 100.00
  rent_day = 2025-01-25
  wage_day = 2025-02-20
  bonus_day = 2025-03-05
  sweep_day = 2025-01-20
  top_day = 2025-03-10
  late_day = 2025-03-15
  skim_day = 2025-02-25
  fee_day = 2025-03-20
  fee_amt = 2000.00
  reach = 2025-03-01
  top_end = 2025-03-20
  late_end = 2025-03-31
  opening[bank] = 1000.00 at 2025-01-01 start
  opening[save] = 0.00 at 2025-01-01 start
  opening[spare] = 50.00 at 2025-01-01 start

version raise
  cite card
  governs always
  known 2025-01-22
  replace wage_amt = 900.00

version cheaper
  cite card
  governs always
  known 2025-02-01
  replace fee_amt = 10.00

ask deep: bal(bank, 2025-03-31) from 2025-01-20

ask skimmed: flows(skim, 2025-01-01 .. 2025-03-31)

ask bank_end: bal(bank, 2025-03-31)

ask seen: bal(bank, 2025-03-01) from 2025-01-20

ask room_now: room

ask swept: flows(sweep, 2025-01-01 .. 2025-03-31)

ask topped: flows(top, 2025-01-01 .. 2025-03-31)

ask saved: bal(save, 2025-03-31)

ask backwards: bal(bank, 2025-01-03) from 2025-01-10

ask prerun: bal(bank, 2025-01-03) from 2024-12-31

ask hole_from: bal(bank, 2025-03-31) from gift_day

ask unc: bal(bank, 2025-03-31) from 2025-02-01 with uncertain

ask plain_unc: bal(bank, 2025-03-31) from 2025-02-01

ask before_spare: bal(spare, 2025-03-10) from 2025-03-01

ask after_spare: bal(spare, 2025-03-20) from 2025-03-01

ask spare_end: bal(spare, 2025-03-31)
"#;

    /// refcore's lines for the reality, every ask, in id order.
    const LINES: &str = r#"after_spare: refused: RUN-FUTURE a decision on 2025-03-15 reads a projection from 2025-03-16, after its day
  cut 2025-02-10
backwards: refused: RUN-DOMAIN bal(bank, 2025-01-03) from 2025-01-10: the day is before the projection's start
  cut 2025-02-10
bank_end: 90.00 DUK
  unfulfilled: rent
  left out: skim, 1 instances, first 2025-02-25
  cut 2025-02-10
before_spare: 50.00 DUK
  cut 2025-02-10
deep: -1 900.00 DUK
  left out: skim, 1 instances, first 2025-02-25
  cut 2025-02-10
hole_from: ?"When does the gift come?" (gift_day)
  cut 2025-02-10
plain_unc: 490.00 DUK
  unfulfilled: rent
  left out: skim, 1 instances, first 2025-02-25
  cut 2025-02-10
prerun: ?"before the run's start" (bank)
  cut 2025-02-10
room_now: 100.00 DUK
  left out: skim, 1 instances, first 2025-02-25
  left out: sweep, 1 instances, first 2025-01-20
  cut 2025-02-10
saved: 1 400.00 DUK
  left out: skim, 1 instances, first 2025-02-25
  cut 2025-02-10
seen: 100.00 DUK
  left out: skim, 1 instances, first 2025-02-25
  cut 2025-02-10
skimmed: 1 400.00 DUK
  left out: skim, 1 instances, first 2025-02-25
  cut 2025-02-10
spare_end: refused: RUN-FUTURE a decision on 2025-03-15 reads a projection from 2025-03-16, after its day
  cut 2025-02-10
swept: 0.00 DUK
  unfulfilled: sweep
  left out: skim, 1 instances, first 2025-02-25
  left out: sweep, 1 instances, first 2025-01-20
  cut 2025-02-10
topped: 0.00 DUK
  left out: skim, 1 instances, first 2025-02-25
  left out: top, 1 instances, first 2025-03-10
  cut 2025-02-10
unc: 740.00 DUK
  unfulfilled: rent
  left out: skim, 1 instances, first 2025-02-25
  uncertain: bonus
  cut 2025-02-10"#;

    /// The lines a run of the reality prints, the walk stepping `days`.
    fn lines(days: Days) -> Vec<String> {
        let given = vec![Given {
            path: "r.tally".to_owned(),
            bytes: Some(REALITY.as_bytes().to_vec()),
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
    fn every_projection_prints_as_refcore_prints_it() {
        let want: Vec<&str> = LINES.lines().collect();
        assert_eq!(lines(Days::Events), want, "refcore's lines");
    }

    #[test]
    fn the_every_day_walk_prints_the_same_projections() {
        let want: Vec<&str> = LINES.lines().collect();
        assert_eq!(lines(Days::Every), want, "the oracle's lines");
    }
}
