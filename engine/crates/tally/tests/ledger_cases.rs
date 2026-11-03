//! The builder's residual-ledger tests: a small reality's `residuals` asks print as refcore prints them, under both day sources.
//! Spec: E §6.3 (the lines: matched, expected but missing, unexplained, partly fulfilled,
//! balance anchored and padded; an adjustment settled by an observation, against its change on
//! V.known; the `?` group of an instance whose day is a hole, and one whose day an observation
//! answers; an observation that fulfils an instance outside the span, which is that
//! instance's, not unexplained; zero lines; signs; totals; holes first, then size, day, group),
//! §6.2 (a decision on a covered day as the walk ran it), L §5.19 (coverage; `as known`),
//! §5.18 (`residuals x span`); P §2.2 (the oracle).
//! Twin: engine/tools/refcore/resid.py (`m_residuals`) and ask.py (`answer_residuals`); the
//! expected lines are refcore's run of the same text (`python3 -m refcore run`).
//! Invariants: reads no test, vector or answer key (R-93); the text is written here, small.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally::{Asks, Days, Given, Pin, Request, run};

    /// Two tenants' rent, one paid; an uncertain grant paid in part; a fee whose day is a
    /// hole; a tip whose day an observation answers; an amount of 0; a January payment
    /// observed in February; a January tax a later version raises, its change settled by an
    /// observation; a March payment observed on an uncovered day; a sweep decision on a
    /// covered day that moves 0; two observations nothing fulfils; February's statement
    /// anchors, March's pads.
    const REALITY: &str = r#"model lg
  tally 2
  on 2025-03-10
  start 2025-01-01
  end 2025-03-31

source card "a card"

source stmt "a statement"

unit DUK: currency minor 0.01

kind ext: place DUK signed outside

kind promises: transfers uncertain

place out: ext lives 2025-01-01..

place bank: accounts DUK lives 2025-01-01..

place save: accounts DUK lives 2025-01-01..

index tenant = [a, b]

input rows: observed

input pay[t in tenant]: money DUK

input rent_day: date

input grant_amt: money DUK

input tax_amt: money DUK

input fee_amt: money DUK

input tip_amt: money DUK

input nil_amt: money DUK

input jan_amt: money DUK

input late_amt: money DUK

input later_amt: money DUK

input floor: money DUK

input grant_day: date

input nil_day: date

input jan_pay_day: date

input tax_day: date

input late_day: date

input later_day: date

input sweep_day: date

input fee_day: date question "When is the fee charged?"

input tip_day: date question "When does the tip come?"

move rent[t in tenant]: transfers
  on rent_day
  leg cash from bank to out amount pay[t]

move grant: promises
  on grant_day
  leg cash from out to bank amount grant_amt

move fee: transfers
  on fee_day
  leg cash from save to out amount fee_amt

move tip: transfers
  on tip_day
  leg cash from out to bank amount tip_amt

move nil: transfers
  on nil_day
  leg cash from bank to out amount nil_amt

move jan_pay: transfers
  on jan_pay_day
  leg cash from bank to out amount jan_amt

move tax: transfers
  on tax_day
  leg cash from bank to out amount tax_amt

move late: transfers
  on late_day
  leg cash from bank to out amount late_amt

move later: transfers
  on later_day
  leg cash from bank to out amount later_amt

move sweep: transfers
  on sweep_day
  reads bank
  leg cash from bank to save amount max(0, bal(bank) - floor)

version facts
  cite card
  governs always
  known 2025-01-01
  pay[a] = 300.00
  pay[b] = 200.00
  rent_day = 2025-02-03
  grant_amt = 300.00
  tax_amt = 100.00
  fee_amt = 15.00
  tip_amt = 40.00
  nil_amt = 0.00
  jan_amt = 70.00
  late_amt = 25.00
  later_amt = 35.00
  floor = 900.00
  grant_day = 2025-02-12
  nil_day = 2025-02-16
  jan_pay_day = 2025-01-28
  tax_day = 2025-01-20
  late_day = 2025-03-03
  later_day = 2025-03-04
  sweep_day = 2025-02-20
  opening[bank] = 1000.00 at 2025-01-01 start
  opening[save] = 0.00 at 2025-01-01 start

version fix
  cite card
  governs always
  known 2025-02-18
  replace tax_amt = 130.00

version stmt_02
  cite stmt
  governs 2025-02
  known 2025-03-05
  covers bank
  anchor bank
  rows[r1] = (on: 2025-02-05, kind: transfers, leg cash from bank to out amount 300.00 DUK, fulfils: rent[a])
  rows[r2] = (on: 2025-02-13, kind: transfers, leg cash from out to bank amount 200.00 DUK, fulfils: grant)
  rows[r3] = (on: 2025-02-14, kind: transfers, leg cash from out to bank amount 40.00 DUK, fulfils: tip)
  rows[r4] = (on: 2025-02-10, kind: transfers, leg cash from bank to out amount 70.00 DUK, fulfils: jan_pay)
  rows[r5] = (on: 2025-02-21, kind: transfers, leg cash from bank to out amount 30.00 DUK, fulfils: adjustments[tax.cash, fix])
  rows[r6] = (on: 2025-02-25, kind: transfers, leg cash from bank to out amount 12.50 DUK)
  rows[r7] = (on: 2025-02-26, kind: transfers, leg cash from out to bank amount 12.50 DUK)
  stated[bank] = 600.00 at 2025-02-14 end
  stated[bank] = 500.00 at 2025-02-28 end

version stmt_03
  cite stmt
  governs 2025-03
  known 2025-03-08
  pad bank from out
  rows[r8] = (on: 2025-03-03, kind: transfers, leg cash from bank to out amount 20.00 DUK, fulfils: late)
  stated[bank] = 450.00 at 2025-03-06 end

ask ledger: residuals bank 2025-02 .. 2025-03

ask feb: residuals bank 2025-02

ask early: residuals bank 2025-02 as known 2025-02-20

ask saving: residuals save 2025-01 .. 2025-03
"#;

    /// refcore's lines for the reality, every ask, in id order.
    const LINES: &str = r#"early: residuals bank 2025-02
  2025-02 balance: model ?"When does the tip come?" (tip_day) ?"tax on 2025-01-20 changed after it was paid (version fix); is back pay due, and when?" (tax), observed 1 100.00 DUK, difference ?"When does the tip come?" (tip_day) ?"tax on 2025-01-20 changed after it was paid (version fix); is back pay due, and when?" (tax)
    balance bank 2025-02-14 end (anchored): booked ?"When does the tip come?" (tip_day), stated 600.00 DUK, gap ?"When does the tip come?" (tip_day)
    balance bank 2025-02-28 end (anchored): booked ?"When does the tip come?" (tip_day) ?"tax on 2025-01-20 changed after it was paid (version fix); is back pay due, and when?" (tax), stated 500.00 DUK, gap ?"When does the tip come?" (tip_day) ?"tax on 2025-01-20 changed after it was paid (version fix); is back pay due, and when?" (tax)
  2025-02 move sweep: model ?"When does the tip come?" (tip_day) ?"tax on 2025-01-20 changed after it was paid (version fix); is back pay due, and when?" (tax), observed 0.00 DUK, difference ?"When does the tip come?" (tip_day) ?"tax on 2025-01-20 changed after it was paid (version fix); is back pay due, and when?" (tax)
    expected but missing sweep on 2025-02-20: model ?"When does the tip come?" (tip_day) ?"tax on 2025-01-20 changed after it was paid (version fix); is back pay due, and when?" (tax)
  2025-02 move rent: model -500.00 DUK, observed -300.00 DUK, difference 200.00 DUK
    expected but missing rent[b] on 2025-02-03: model -200.00 DUK
    matched rent[a] on 2025-02-03: model -300.00 DUK, observed -300.00 DUK, difference 0.00 DUK, timing +2 days
  2025-02 move grant: model 300.00 DUK, observed 200.00 DUK, difference -100.00 DUK
    partly fulfilled grant on 2025-02-12: model 300.00 DUK, observed 200.00 DUK, difference -100.00 DUK, remainder 100.00 DUK: when does the rest arrive, or is it written off?
  2025-02 move tax: model -30.00 DUK, observed -30.00 DUK, difference 0.00 DUK
    matched adjustments[tax.cash, fix] on 2025-02-18: model -30.00 DUK, observed -30.00 DUK, difference 0.00 DUK, timing +3 days
  2025-02 move tip: model 40.00 DUK, observed 40.00 DUK, difference 0.00 DUK
    matched tip on 2025-02-14: model 40.00 DUK, observed 40.00 DUK, difference 0.00 DUK, timing +0 days
  2025-02 unexplained: model 0.00 DUK, observed 0.00 DUK, difference 0.00 DUK
    unexplained rows[r6] on 2025-02-25: observed -12.50 DUK
    unexplained rows[r7] on 2025-02-26: observed 12.50 DUK
  cut 2025-02-20
feb: residuals bank 2025-02
  2025-02 balance: model 1 340.00 DUK, observed 1 100.00 DUK, difference -240.00 DUK
    balance bank 2025-02-14 end (anchored): booked 770.00 DUK, stated 600.00 DUK, gap -170.00 DUK
    balance bank 2025-02-28 end (anchored): booked 570.00 DUK, stated 500.00 DUK, gap -70.00 DUK
  2025-02 move rent: model -500.00 DUK, observed -300.00 DUK, difference 200.00 DUK
    expected but missing rent[b] on 2025-02-03: model -200.00 DUK
    matched rent[a] on 2025-02-03: model -300.00 DUK, observed -300.00 DUK, difference 0.00 DUK, timing +2 days
  2025-02 move grant: model 300.00 DUK, observed 200.00 DUK, difference -100.00 DUK
    partly fulfilled grant on 2025-02-12: model 300.00 DUK, observed 200.00 DUK, difference -100.00 DUK, remainder 100.00 DUK: when does the rest arrive, or is it written off?
  2025-02 move tax: model -30.00 DUK, observed -30.00 DUK, difference 0.00 DUK
    matched adjustments[tax.cash, fix] on 2025-02-18: model -30.00 DUK, observed -30.00 DUK, difference 0.00 DUK, timing +3 days
  2025-02 move tip: model 40.00 DUK, observed 40.00 DUK, difference 0.00 DUK
    matched tip on 2025-02-14: model 40.00 DUK, observed 40.00 DUK, difference 0.00 DUK, timing +0 days
  2025-02 unexplained: model 0.00 DUK, observed 0.00 DUK, difference 0.00 DUK
    unexplained rows[r6] on 2025-02-25: observed -12.50 DUK
    unexplained rows[r7] on 2025-02-26: observed 12.50 DUK
  cut 2025-03-10
ledger: residuals bank 2025-02 .. 2025-03
  2025-02 balance: model 1 340.00 DUK, observed 1 100.00 DUK, difference -240.00 DUK
    balance bank 2025-02-14 end (anchored): booked 770.00 DUK, stated 600.00 DUK, gap -170.00 DUK
    balance bank 2025-02-28 end (anchored): booked 570.00 DUK, stated 500.00 DUK, gap -70.00 DUK
  2025-02 move rent: model -500.00 DUK, observed -300.00 DUK, difference 200.00 DUK
    expected but missing rent[b] on 2025-02-03: model -200.00 DUK
    matched rent[a] on 2025-02-03: model -300.00 DUK, observed -300.00 DUK, difference 0.00 DUK, timing +2 days
  2025-02 move grant: model 300.00 DUK, observed 200.00 DUK, difference -100.00 DUK
    partly fulfilled grant on 2025-02-12: model 300.00 DUK, observed 200.00 DUK, difference -100.00 DUK, remainder 100.00 DUK: when does the rest arrive, or is it written off?
  2025-03 balance: model 445.00 DUK, observed 450.00 DUK, difference 5.00 DUK
    balance bank 2025-03-06 end (padded): booked 445.00 DUK, stated 450.00 DUK, gap 5.00 DUK
  2025-03 move late: model -25.00 DUK, observed -20.00 DUK, difference 5.00 DUK
    matched late on 2025-03-03: model -25.00 DUK, observed -20.00 DUK, difference 5.00 DUK, timing +0 days
  2025-02 move tax: model -30.00 DUK, observed -30.00 DUK, difference 0.00 DUK
    matched adjustments[tax.cash, fix] on 2025-02-18: model -30.00 DUK, observed -30.00 DUK, difference 0.00 DUK, timing +3 days
  2025-02 move tip: model 40.00 DUK, observed 40.00 DUK, difference 0.00 DUK
    matched tip on 2025-02-14: model 40.00 DUK, observed 40.00 DUK, difference 0.00 DUK, timing +0 days
  2025-02 unexplained: model 0.00 DUK, observed 0.00 DUK, difference 0.00 DUK
    unexplained rows[r6] on 2025-02-25: observed -12.50 DUK
    unexplained rows[r7] on 2025-02-26: observed 12.50 DUK
  cut 2025-03-10
saving: residuals save 2025-01 .. 2025-03
  ?: model -15.00 DUK, observed 0.00 DUK, difference 15.00 DUK
    expected but missing fee on ?"When is the fee charged?" (fee_day): model -15.00 DUK
  cut 2025-03-10"#;

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
    fn every_ledger_prints_as_refcore_prints_it() {
        let want: Vec<&str> = LINES.lines().collect();
        assert_eq!(lines(Days::Events), want, "refcore's lines");
    }

    #[test]
    fn the_every_day_walk_prints_the_same_ledgers() {
        assert_eq!(
            lines(Days::Every),
            lines(Days::Events),
            "the oracle's walk and the event walk"
        );
    }
}
