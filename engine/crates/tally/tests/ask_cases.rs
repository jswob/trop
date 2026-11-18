//! The builder's ask tests: a small reality's asks print as refcore prints them, line for line, with the register under each.
//! Spec: L §5.18 (asks: `law`, `as known`, `in`, `for` with `where`, one line per binding tuple,
//! a tuple whose `where` is a hole printed as that hole, a line's values), §5.11 (an ask's
//! `law` that is a hole makes its line that hole), §5.7 (`x in K on d`: a hole `d` is that
//! hole; §8.4 `RUN-NONE` for `none` and `never`), §5.19 Bindings (movements by day, a hole
//! day last), §8.2 (the register: `assumed`, `held`, `world`, the law date and the cut; a
//! tuple's `unfulfilled:` on its own line), §8.4 (a run-time refusal stands as its value,
//! with the register met before it; L §5.14.3 a refused `flows`, L §5.14 `with uncertain`);
//! tally 2.1.8 R13.1 (a diff's own `for`, one block per tuple, on one stack and on two), R11
//! (L §4.2 `add_business` past the day range, a hole on the way read day by day).
//! Twin: engine/tools/refcore/ask.py (`Asker`); the expected lines are refcore's run of the
//! same text (`python3 -m refcore run`), but where refcore stops or misses: there the lines
//! are the spec's, and the case's comment says so.
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

    /// A payment into a place whose lifespan has ended is refused on 2025-02-05; decisions
    /// read its flows, so their amounts are refused from then on: `match` on 2025-02-20, and
    /// `split[b]` on 2025-02-25 after `split[a]` on 2025-01-20 moved 0.
    const REFUSED_FLOWS: &str = r#"model rf
  tally 2
  on 2025-03-10
  start 2025-01-01
  end 2025-03-31

source card "a card"

unit DUK: currency minor 0.01

kind ext: place DUK signed outside

place out: ext lives 2025-01-01..

place bank: accounts DUK lives 2025-01-01..

place save: accounts DUK lives 2025-01-01..

place old: accounts DUK lives 2025-01-01..2025-01-31

input pay_amt: money DUK

input pay_day: date

input match_day: date

index who = [a, b]

input split_day[k in who]: date

input from_day: date

input feb_start: date

input feb_end: date

move pay: transfers
  on pay_day
  leg cash from bank to old amount pay_amt

move match: transfers
  on match_day
  reads bank
  leg cash from bank to save amount min(bal(bank), flows(pay, feb_start .. feb_end))

move split[k in who]: transfers
  on split_day[k]
  reads bank
  leg cash from bank to save amount min(bal(bank), flows(pay, from_day .. split_day[k]))

version facts
  cite card
  governs always
  known 2025-01-01
  pay_amt = 50.00
  pay_day = 2025-02-05
  match_day = 2025-02-20
  split_day[a] = 2025-01-20
  split_day[b] = 2025-02-25
  from_day = 2025-01-01
  feb_start = 2025-02-01
  feb_end = 2025-02-28
  opening[bank] = 1000.00 at 2025-01-01 start
  opening[save] = 0.00 at 2025-01-01 start
  opening[old] = 0.00 at 2025-01-01 start

ask matched: flows(match, 2025-02-01 .. 2025-03-31)

ask splits: flows(split, 2025-01-01 .. 2025-03-31)

ask early: flows(split, 2025-01-01 .. 2025-01-31)
"#;

    /// refcore's lines for [`REFUSED_FLOWS`]: a refused `flows` of a move keeps the register it
    /// met before the refusal, the refused run's `unfulfilled:` and every earlier one's.
    const REFUSED_FLOWS_LINES: &str = r"early: 0.00 DUK
  unfulfilled: split[a]
  cut 2025-03-10
matched: refused: RUN-LIFESPAN pay.cash moves into or out of old outside its lifespan
  unfulfilled: match
  cut 2025-03-10
splits: refused: RUN-LIFESPAN pay.cash moves into or out of old outside its lifespan
  unfulfilled: split[a]
  unfulfilled: split[b]
  cut 2025-03-10
warning: WARN-UNREAD place out is read by nothing";

    #[test]
    fn a_refused_flows_of_a_move_keeps_what_it_met_before() {
        let want: Vec<&str> = REFUSED_FLOWS_LINES.lines().collect();
        assert_eq!(
            lines_of(REFUSED_FLOWS, Days::Events),
            want,
            "refcore's lines"
        );
        assert_eq!(
            lines_of(REFUSED_FLOWS, Days::Every),
            want,
            "the every-day walk's lines"
        );
    }

    /// A place's flows over a span that holds a booked rent, then a payment refused for a
    /// lifespan; a balance `with uncertain` over two uncertain gifts, the second's amount a
    /// division by zero.
    const REFUSED_SUMS: &str = r#"model rp
  tally 2
  on 2025-03-10
  start 2025-01-01
  end 2025-03-31

source card "a card"

unit DUK: currency minor 0.01

kind ext: place DUK signed outside

kind promises: transfers uncertain

place out: ext lives 2025-01-01..

place bank: accounts DUK lives 2025-01-01..

place old: accounts DUK lives 2025-01-01..2025-01-31

input pay_amt: money DUK

input rent_amt: money DUK

input gift_amt: money DUK

input none_amt: int

input cent: money DUK

input pay_day: date

input rent_day: date

input gift_day: date

input late_day: date

move rent: transfers
  on rent_day
  leg cash from bank to out amount rent_amt

move pay: transfers
  on pay_day
  leg cash from bank to old amount pay_amt

move gift: promises
  on gift_day
  leg cash from out to bank amount gift_amt

move late: promises
  on late_day
  leg cash from out to bank amount round(gift_amt / none_amt, cent, half-up)

version facts
  cite card
  governs always
  known 2025-01-01
  pay_amt = 50.00
  rent_amt = 100.00
  gift_amt = 40.00
  none_amt = 0
  cent = 0.01
  rent_day = 2025-01-10
  pay_day = 2025-02-05
  gift_day = 2025-01-12
  late_day = 2025-01-14
  opening[bank] = 1000.00 at 2025-01-01 start
  opening[old] = 0.00 at 2025-01-01 start

ask bank_flows: flows(bank, 2025-01-01 .. 2025-03-31)

ask with_gifts: bal(bank, 2025-01-31) with uncertain
"#;

    /// refcore's lines for [`REFUSED_SUMS`]: each refused line keeps the register met before
    /// the refusal, the rent's `unfulfilled:` and both gifts' `uncertain:`.
    const REFUSED_SUMS_LINES: &str = r"bank_flows: refused: RUN-LIFESPAN pay.cash moves into or out of old outside its lifespan
  unfulfilled: rent
  cut 2025-03-10
with_gifts: refused: RUN-DIV division by zero
  unfulfilled: rent
  uncertain: gift
  uncertain: late
  cut 2025-03-10";

    #[test]
    fn a_refused_flows_of_a_place_or_with_uncertain_keeps_what_it_met_before() {
        let want: Vec<&str> = REFUSED_SUMS_LINES.lines().collect();
        assert_eq!(
            lines_of(REFUSED_SUMS, Days::Events),
            want,
            "refcore's lines"
        );
        assert_eq!(
            lines_of(REFUSED_SUMS, Days::Every),
            want,
            "the every-day walk's lines"
        );
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

    /// The lines a run of the files `files`, each a path and its text, prints, every ask.
    fn lines_of_files(files: &[(&str, &str)]) -> Vec<String> {
        let given = files
            .iter()
            .map(|(path, text)| Given {
                path: (*path).to_owned(),
                bytes: Some(text.as_bytes().to_vec()),
            })
            .collect();
        let request = Request {
            asks: Asks::Every,
            pin: Pin::Canonical,
            days: Days::Events,
        };
        let answer = run(given, &request).expect("an answer");
        answer.lines().to_vec()
    }

    /// A diff's own `for`: two keys, a scenario that moves one (`sd`) and one that moves the
    /// other (`sc`), the two on two stacks (the review of group 1's B1).
    const DIFF_FOR: &str = "model pw
  tally 2
  on 2026-04-25
  start 2026-01-01
  end 2026-12-31

source s \"s\"
index ks = [a, b]
input dk[k in ks]: date

version v
  cite s
  governs always
  known 2026-01-01
  dk[a] = 2026-03-05 \"s\"
  dk[b] = 2026-03-05 \"s\"

ask h1: diff(dk[k]) in sd against pw for k in ks law 2026-03-01
ask h2: diff(dk[j] for j in ks) in sd against pw for k in [b] law 2026-03-01
ask h3: diff(dk[k]) in sd against sc for k in ks law 2026-03-01
";

    /// Scenario `sd`: dk[b] moves.
    const DIFF_FOR_SD: &str = "scenario sd over pw
  tally 2
  from 2026-01-01

version sd_v
  cite s
  governs 2026-01-01..
  known 2026-01-01
  replace dk[b] = 2026-03-07 \"s\"
";

    /// Scenario `sc`: dk[a] moves.
    const DIFF_FOR_SC: &str = "scenario sc over pw
  tally 2
  from 2026-01-01

version sc_v
  cite s
  governs 2026-01-01..
  known 2026-01-01
  replace dk[a] = 2026-03-06 \"s\"
";

    /// The spec's lines, not refcore's: refcore stops `REFCORE-NOT-YET (KeyError: 'k')` on h1
    /// and h3 and prints h2 as one block with no binding part (L §5.18, tally 2.1.8 R13.1:
    /// every clause stands on every form, one block per binding tuple).
    const DIFF_FOR_LINES: &str =
        "h1 (k = a): diff in sd against pw: 0 changed, 0 added, 0 removed, 1 unchanged
  world: pw
  world: sd
  law date 2026-03-01, cut 2026-04-25
h1 (k = b): diff in sd against pw: 1 changed, 0 added, 0 removed, 0 unchanged
  changed dk[k]: 2026-03-07 against 2026-03-05
  world: pw
  world: sd
  law date 2026-03-01, cut 2026-04-25
h2 (k = b): diff in sd against pw: 1 changed, 0 added, 0 removed, 1 unchanged
  changed (j = b): 2026-03-07 against 2026-03-05
  world: pw
  world: sd
  law date 2026-03-01, cut 2026-04-25
h3 (k = a): diff in sd against sc: 1 changed, 0 added, 0 removed, 0 unchanged
  changed dk[k]: 2026-03-05 against 2026-03-06
  world: sc
  world: sd
  law date 2026-03-01, cut 2026-04-25
h3 (k = b): diff in sd against sc: 1 changed, 0 added, 0 removed, 0 unchanged
  changed dk[k]: 2026-03-07 against 2026-03-05
  world: sc
  world: sd
  law date 2026-03-01, cut 2026-04-25";

    #[test]
    fn a_diff_reads_its_own_for_one_block_per_tuple_on_one_stack_and_two() {
        let files = [
            ("pw.tally", DIFF_FOR),
            ("sc.tally", DIFF_FOR_SC),
            ("sd.tally", DIFF_FOR_SD),
        ];
        let want: Vec<&str> = DIFF_FOR_LINES.lines().collect();
        assert_eq!(lines_of_files(&files), want, "one block per tuple");
    }

    /// A diff over two stacks whose law date is its tuple's (the review of group 1's fix, F4:
    /// `d3` h3), beside the same diff at one law date (h4): one block per tuple in both.
    #[test]
    fn a_two_stack_diff_with_its_law_date_per_tuple_prints_one_block_per_tuple() {
        let (base, _) = DIFF_FOR.split_once("\nask ").expect("DIFF_FOR has asks");
        let text = base
            .replace("input dk[k in ks]: date\n", "input dk[k in ks]: date\ninput lk[k in ks]: date\n")
            .replace(
                "  dk[b] = 2026-03-05 \"s\"\n",
                "  dk[b] = 2026-03-05 \"s\"\n  lk[a] = 2026-03-01 \"s\"\n  lk[b] = 2026-03-01 \"s\"\n",
            )
            + "\nask h3: diff(dk[k]) in sd against sc for k in ks law lk[k]\n\
               ask h4: diff(dk[k]) in sd against sc for k in ks law 2026-03-01\n";
        let lines = lines_of_files(&[
            ("pw.tally", &text),
            ("sc.tally", DIFF_FOR_SC),
            ("sd.tally", DIFF_FOR_SD),
        ]);
        let block = |h: &str| {
            [
                format!(
                    "{h} (k = a): diff in sd against sc: 1 changed, 0 added, 0 removed, 0 unchanged"
                ),
                "  changed dk[k]: 2026-03-05 against 2026-03-06".to_owned(),
                "  world: sc".to_owned(),
                "  world: sd".to_owned(),
                "  law date 2026-03-01, cut 2026-04-25".to_owned(),
                format!(
                    "{h} (k = b): diff in sd against sc: 1 changed, 0 added, 0 removed, 0 unchanged"
                ),
                "  changed dk[k]: 2026-03-07 against 2026-03-05".to_owned(),
                "  world: sc".to_owned(),
                "  world: sd".to_owned(),
                "  law date 2026-03-01, cut 2026-04-25".to_owned(),
            ]
        };
        let want: Vec<String> = block("h3").into_iter().chain(block("h4")).collect();
        assert_eq!(lines, want, "h3's law date per tuple prints as h4's");
    }

    /// `add_business` past the day range over a calendar read day by day: a key that is a
    /// hole, and a scenario's expression row governing 2026 only, the base's row before 2026
    /// a calendar with no business day (the review of group 1's B2).
    const SHIFT_HOLE_KEY: &str = "model hk
  tally 2
  on 2026-12-31
  start 2026-01-01
  end 2026-12-31

source plan \"plan\"
index cs = [pl, de]
input who: key cs
input calk[c in cs]: calendar
input d: date
input nf: int

version v
  cite plan
  governs always
  known 2026-01-01
  d = 2026-06-01 \"a day\"
  nf = 2912292 \"far\"
  calk[pl] = calendar(weekends [sat, sun]) \"s\"
  calk[de] = calendar(weekends [sat, sun]) \"s\"

ask k: add_business(d, nf, calk[who])
ask kp: add_business(d, nf, calk[pl])
";

    /// The base of the expression row's case.
    const SHIFT_ROWS: &str = "model a6
  tally 2
  on 2026-12-31
  start 2026-01-01
  end 2026-12-31

source plan \"plan\"
input d: date
input cr: calendar
input cq: calendar
input huge: int
input n3: int
input n300: int

version v
  cite plan
  governs always
  known 2026-01-01
  d = 2026-06-01 \"a day\"
  huge = 100000000000000000000 \"10^20\"
  n3 = 3 \"three\"
  n300 = 300 \"300\"

version v26
  cite plan
  governs 2026
  known 2026-01-01
  cr = calendar(weekends [sat, sun]) \"s\"
  cq = calendar(weekends [sat, sun]) \"s\"

version v25
  cite plan
  governs ..2025
  known 2026-01-01
  cq = calendar(weekends [mon, tue, wed, thu, fri, sat, sun]) \"none\"

ask q_fwd: add_business(d, huge, cq)
ask x_fwd: add_business(d, huge, cq) in sx
ask x_back: add_business(d, -huge, cq) in sx
ask x_in: add_business(d, n300, cq) in sx
ask x_3: add_business(d, n3, cq) in sx
";

    /// The scenario whose row is an expression.
    const SHIFT_ROWS_SX: &str = "scenario sx over a6
  tally 2
  from 2026-01-01

version sx_v
  cite plan
  governs 2026
  known 2026-01-01
  replace cq = a6.cr \"s\"
";

    /// refcore's lines but `x_back`'s: the ruled order (P27), the calendar's `RUN-DOMAIN` met
    /// on the way back before the edge, where refcore refuses `RUN-LIMIT` before any read.
    const SHIFT_ROWS_LINES: &str = "q_fwd: refused: RUN-LIMIT a day passes 9999-12-31
  cut 2026-12-31
x_3: 2026-06-04
  world: sx
  cut 2026-12-31
x_back: refused: RUN-DOMAIN calendar cq has no business day
  world: sx
  cut 2026-12-31
x_fwd: refused: RUN-LIMIT a day passes 9999-12-31
  world: sx
  cut 2026-12-31
x_in: ?\"calendar cq doesn't cover 2027\" (cq)
  world: sx
  cut 2026-12-31";

    #[test]
    fn a_shift_past_the_day_range_passes_a_hole_read_day_by_day() {
        assert_eq!(
            lines_of_files(&[("hk.tally", SHIFT_HOLE_KEY)]),
            [
                "k: refused: RUN-LIMIT a day passes 9999-12-31",
                "  cut 2026-12-31",
                "kp: refused: RUN-LIMIT a day passes 9999-12-31",
                "  cut 2026-12-31",
            ],
            "a key that is a hole reads no calendar"
        );
        let files = [("a6.tally", SHIFT_ROWS), ("sx.tally", SHIFT_ROWS_SX)];
        let want: Vec<&str> = SHIFT_ROWS_LINES.lines().collect();
        assert_eq!(lines_of_files(&files), want, "an expression row's days");
    }

    /// A shift of 10^20 business days on a calendar a scenario computes from another that has
    /// no row past 2026 (the review of group 1's fix, F2: `b2_min`).
    const SHIFT_COMPUTED: &str = "model bm
  tally 2
  on 2026-12-31
  start 2026-01-01
  end 2026-12-31

source plan \"plan\"
input d: date
input cr: calendar
input cq: calendar
input huge: int

version v
  cite plan
  governs always
  known 2026-01-01
  d = 2026-06-01 \"a day\"
  huge = 100000000000000000000 \"10^20\"

version v26
  cite plan
  governs 2026
  known 2026-01-01
  cr = calendar(weekends [sat, sun]) \"s\"
  cq = calendar(weekends [sat, sun]) \"s\"

ask f: add_business(d, huge, cq) in sx
";

    const SHIFT_COMPUTED_SX: &str = "scenario sx over bm
  tally 2
  from 2026-01-01

version sx_v
  cite plan
  governs 2026-01-01..
  known 2026-01-01
  replace cq = bm.cr \"s\"
";

    #[test]
    fn a_shift_past_the_edge_reads_the_rows_edges_and_holds_a_computed_hole_as_a_mark() {
        assert_eq!(
            lines_of_files(&[
                ("bm.tally", SHIFT_COMPUTED),
                ("sx.tally", SHIFT_COMPUTED_SX)
            ]),
            [
                "f: refused: RUN-LIMIT a day passes 9999-12-31",
                "  world: sx",
                "  held: cq = ? (sx_v), read for 2027-01-01",
                "  cut 2026-12-31",
            ],
            "the first hole met is the edge's; one held line, a computed hole as ? (P32)"
        );
    }

    /// A calendar stated `same` as another, whose row from 2028-03-01 has no business day,
    /// a day the first calendar's own rows don't bound (the review of group 1's fix, N3).
    const SHIFT_SAME_ROW: &str = "model n3
  tally 2
  on 2026-12-31
  start 2026-01-01
  end 2026-12-31

source plan \"plan\"
input d: date
input cr: calendar
input cq: calendar
input huge: int

version v
  cite plan
  governs always
  known 2026-01-01
  d = 2026-06-01 \"a day\"
  huge = 100000000000000000000 \"10^20\"

version v26
  cite plan
  governs 2026
  known 2026-01-01
  cq = calendar(weekends [sat, sun]) \"s\"

version vr
  cite plan
  governs 2026-01-01..2028-02-29
  known 2026-01-01
  cr = calendar(weekends [sat, sun]) \"s\"

version vz
  cite plan
  governs 2028-03-01..
  known 2026-01-01
  cr = calendar(weekends [mon, tue, wed, thu, fri, sat, sun]) \"s\"

version link
  cite plan
  governs always
  known 2026-01-01
  same cq = cr

ask f: add_business(d, huge, cq)
ask g: add_business(d, huge, cr)
";

    #[test]
    fn a_shift_past_the_edge_reads_the_rows_a_same_row_follows() {
        let held = "  held: cr = calendar(weekends [mon, tue, wed, thu, fri, sat, sun]) (vz), read for 2028-03-01";
        assert_eq!(
            lines_of_files(&[("n3.tally", SHIFT_SAME_ROW)]),
            [
                "f: refused: RUN-DOMAIN calendar cq has no business day",
                held,
                "  cut 2026-12-31",
                "g: refused: RUN-DOMAIN calendar cr has no business day",
                held,
                "  cut 2026-12-31",
            ],
            "the refusal cq meets through its same row isn't hidden by the edge's RUN-LIMIT"
        );
    }

    /// A window of one day read as a key, beside the plain day (the review of group 1's N6).
    const ONE_DAY_KEY: &str = "model w
  tally 2
  on 2026-04-25
  start 2026-01-01
  end 2026-12-31

source s \"s\"
input d: date
input e: date
input x[m in months]: int
input y[t in days]: int

version v
  cite s
  governs always
  known 2026-01-01
  d = window(2026-03-05, 2026-03-05) \"s\"
  e = 2026-03-05 \"s\"
  x[2026-03] = 7 \"s\"
  y[2026-03-05] = 9 \"s\"

ask c: x[d]
ask ce: x[e]
ask f: y[d]
ask fe: y[e]
";

    #[test]
    fn a_window_of_one_day_as_a_key_is_that_day() {
        assert_eq!(
            lines_of_files(&[("w.tally", ONE_DAY_KEY)]),
            [
                "c: 7",
                "  cut 2026-04-25",
                "ce: 7",
                "  cut 2026-04-25",
                "f: 9",
                "  cut 2026-04-25",
                "fe: 9",
                "  cut 2026-04-25",
            ],
            "L §4.4: a window of one day is that day, so not imprecise for a key"
        );
    }

    /// A ledger per binding tuple and per option of an axis, one of each dividing by zero or
    /// reading a hole key (the review of group 1's B4 and N8).
    const LEDGER_REFUSED: &str = "model pw
  tally 2
  on 2026-04-25
  start 2026-01-01
  end 2026-12-31

source s \"s\"
unit PLN: currency minor 0.01
index ks = [a, b]
axis ax
  lo \"low\"
  hi \"high\"

place jar[k in ks]: accounts PLN lives 2026-01-01..
place world: outside lives 1900-01-01..
input bank: observed
input amt: money PLN
input dv[k in ks]: int
input dk[k in ks]: date
input d0: date

def kz: key ks at reader
  rule base = a
  rule hi beats base = b when ax == hi

def kh: key ks at reader
  rule base = a
  rule hi beats base = ?\"which jar\" when ax == hi

move one[k in ks]: transfers
  on dk[k]
  at d0
  leg cash from world to jar[k] amount amt / dv[k]

version v
  cite s
  governs always
  known 2026-01-01
  amt = 10.00 PLN \"s\"
  dv[a] = 1 \"s\"
  dv[b] = 0 \"s\"
  dk[a] = window(2026-03-01, 2026-03-02) \"s\"
  dk[b] = 2026-03-05 \"s\"
  d0 = 2026-01-01 \"s\"
  opening[jar[a]] = 0.00 PLN at 2026-01-01 start \"s\"
  opening[jar[b]] = 0.00 PLN at 2026-01-01 start \"s\"

version st
  cite s
  governs 2026-03
  known 2026-04-01
  covers jar[a], jar[b]
  bank[r3] = (on: 2026-03-06, kind: transfers, leg cash from world to jar[a] amount 10.00 PLN) \"r3\"

ask q: residuals jar[k] 2026-03 for k in ks
ask r: residuals jar[kz] 2026-03 law 2026-03-01
ask r2: residuals jar[kh] 2026-03 law 2026-03-01
";

    #[test]
    fn a_refused_ledger_prints_alone_on_its_tuples_or_combinations_line() {
        let lines = lines_of_files(&[("pw.tally", LEDGER_REFUSED)]);
        let heads: Vec<&str> = lines
            .iter()
            .map(String::as_str)
            .filter(|l| !l.starts_with(' '))
            .collect();
        assert_eq!(
            heads,
            [
                "q (k = a): residuals jar[a] 2026-03",
                "q (k = b): refused: RUN-DIV division by zero",
                "r [ax = lo]: residuals jar[a] 2026-03",
                "r [ax = hi]: refused: RUN-DIV division by zero",
                "r2 [ax = lo]: residuals jar[a] 2026-03",
                "r2 [ax = hi]: ?\"which jar\" (kh)",
            ],
            "each tuple's and combination's line, the answers first"
        );
        // a refused or hole ledger's line has nothing under it: no register (refcore's `one`)
        for (line, next) in lines.iter().zip(lines.iter().skip(1)) {
            if line.contains("refused") || line.contains("?\"") {
                assert!(!next.starts_with(' '), "{line} stands alone, not {next}");
            }
        }
        assert!(
            lines.last().is_some_and(|l| l.starts_with("r2 [ax = hi]")),
            "the last line is the hole alone"
        );
    }
}
