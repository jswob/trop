//! The builder's tests of group 4's review: each review input, the lines refcore prints for the asks it names.
//! Spec: L §6.1 (`same`: one value, the class's latest kept reading, #177; B1 and N3 of the
//! review), L §5.19 and §4.6 Readings (`replace … = none` removes a reading).
//! Twin: engine/tools/refcore (spec/2.1.8 at 61f8630b, the eighth turn); each expected line is
//! its run of the same text (`python3 -m refcore run`), each text an input of the review of
//! build group 4 (`tmp/spec-2.1.8-build/review-g4/`).
//! Invariants: reads no test, vector or answer key (R-93); the texts are written here.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally::{Asks, Days, Given, Pin, Request, run};

    /// The lines a run of `text`, read from `path`, prints, every ask.
    fn lines_of(path: &str, text: &str) -> Vec<String> {
        let given = vec![Given {
            path: path.to_owned(),
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

    /// B1 (`agent/same_removed.tally`): a side of a `same` class whose one reading is removed is
    /// no row; `last` of the other side takes the class's latest kept reading.
    #[test]
    fn a_same_side_whose_readings_are_removed_is_no_row() {
        let text = r#"model t
  tally 2
  on 2026-12-31
  start 2026-01-01
  end 2026-12-31

source s "s"
unit PLN: currency minor 0.01
unit EUR: currency minor 0.01

input px[u in currencies]: num PLN/u reading
input qx[u in currencies]: num PLN/u reading

version r
  cite s
  known 2026-03-09
  px[EUR] = 4.3 PLN/EUR at 2026-03-02
  qx[EUR] = 4.4 PLN/EUR at 2026-03-08

version r2
  cite s
  known 2026-04-01
  replace qx[EUR] = none at 2026-03-08

version j
  cite s
  governs always
  known 2026-01-01
  same px[EUR] = qx[EUR]

ask e: last(px[EUR], 2026-03-10), since(px[EUR], 2026-03-10)
ask q: last(qx[EUR], 2026-03-10), since(qx[EUR], 2026-03-10)
ask v: px[EUR] law 2026-03-08
"#;
        let lines = lines_of("same_removed.tally", text);
        assert_eq!(
            lines,
            [
                "e: (4.3 PLN/EUR, 8)",
                "  held: px[EUR] = 4.3 PLN/EUR (r), read for 2026-03-10",
                "  cut 2026-12-31",
                "q: (4.3 PLN/EUR, 8)",
                "  held: px[EUR] = 4.3 PLN/EUR (r), read for 2026-03-10",
                "  cut 2026-12-31",
                "v: ?\"px[EUR] has no reading at 2026-03-08\" (px[EUR])",
                "  law date 2026-03-08, cut 2026-12-31",
            ],
            "every line of the run"
        );
    }

    /// B1 (`agent/same3.tally`): two removed sides of three; the third's kept reading carries,
    /// and a class whose kept values differ is still the hole.
    #[test]
    fn the_class_latest_kept_reading_carries_over_two_removed_sides() {
        let text = r#"model t
  tally 2
  on 2026-12-31
  start 2026-01-01
  end 2026-12-31

source s "s"
unit PLN: currency minor 0.01
unit EUR: currency minor 0.01

input px[u in currencies]: num PLN/u reading
input qx[u in currencies]: num PLN/u reading
input zx[u in currencies]: num PLN/u reading
input a: money PLN
input b: money PLN
input c: money PLN

version r
  cite s
  known 2026-03-09
  px[EUR] = 4.3 PLN/EUR at 2026-03-02
  qx[EUR] = 4.4 PLN/EUR at 2026-03-08
  zx[EUR] = 4.5 PLN/EUR at 2026-03-08

version r2
  cite s
  known 2026-04-01
  replace qx[EUR] = none at 2026-03-08
  replace zx[EUR] = none at 2026-03-08

version st
  cite s
  governs always
  known 2026-01-01
  a = 1.00 PLN
  b = 2.00 PLN
  c = 3.00 PLN

version j
  cite s
  governs always
  known 2026-01-01
  same zx[EUR] = qx[EUR]
  same c = b

version j2
  cite s
  governs always
  known 2026-01-01
  same px[EUR] = qx[EUR]
  same a = b

ask e: last(px[EUR], 2026-03-10), since(px[EUR], 2026-03-10) as known 2026-03-20
ask e2: last(px[EUR], 2026-03-10), since(px[EUR], 2026-03-10)
ask p8: px[EUR] law 2026-03-08 as known 2026-03-20
ask p8b: zx[EUR] law 2026-03-08
ask sa: a law 2026-05-01
ask sl: last(c, 2026-05-01)
"#;
        let lines = lines_of("same3.tally", text);
        // N3: rows that govern always disagree on the day the read reads them for
        holds(
            &lines,
            &[
                "sl: ?\"a, b and c are one, but the value differs on 2026-05-01: 1.00 PLN (st:a), 2.00 PLN (st:b) and 3.00 PLN (st:c)\" (a, b, c)",
            ],
        );
        holds(
            &lines,
            &[
                "e: (?\"qx[EUR] and zx[EUR] are one, but the value differs on 2026-03-08: 4.4 PLN/EUR (r:qx[EUR]) and 4.5 PLN/EUR (r:zx[EUR])\" (qx[EUR], zx[EUR]), 2)",
                "e2: (4.3 PLN/EUR, 8)",
                "  held: px[EUR] = 4.3 PLN/EUR (r), read for 2026-03-10",
                "p8b: ?\"zx[EUR] has no reading at 2026-03-08\" (zx[EUR])",
            ],
        );
    }

    /// B2 (`s1_two_b.tally` and January's count): rows stated same whose `fulfils` name two
    /// instances refuse every read resting on the class or on either instance, a count, a sum
    /// and flows of the move as `bal` and `observed` (S1, P42); a month neither is in answers.
    #[test]
    fn s1_refuses_every_read_resting_on_the_class() {
        let text = r#"model p
  tally 2
  on 2026-06-30
  start 2026-01-01
  end 2026-12-31

source s "s"
unit PLN: currency minor 0.01
kind pays: event certain
  leg cash: PLN
place konto: accounts PLN lives 2026-01-01..
place work: outside lives 1900-01-01..
input amt[m in months]: money PLN
input bank: observed
input card: observed
def tot[m in months]: money PLN = sum(p.cash.amount for p in pays over m)

move pay[m in months]: pays
  on start_of(m)
  leg cash from work to konto amount amt[m]

ask b3: bal(konto, 2026-03-31)
ask t2: tot[2026-02]
ask t3: tot[2026-03]
ask c3: count(p in pays over 2026-03)

version v
  cite s
  governs always
  known 2026-01-01
  amt[2026-01..2026-06] = 10.00 PLN "plan"
  opening[konto] = 0.00 PLN at 2026-01-01 start "own"

version b
  cite s
  governs 2026-03
  known 2026-03-10
  bank[o1] = (on: 2026-03-01, kind: pays, leg cash from work to konto amount 11.00 PLN, fulfils: pay[2026-03]) "b1"
  card[o1] = (on: 2026-03-01, kind: pays, leg cash from work to konto amount 11.00 PLN, fulfils: pay[2026-02]) "c1"

version sm
  cite s
  known 2026-03-11
  same bank[o1] = card[o1]
ask fp: flows(pay, 2026-03)
ask ob: observed(pay[2026-03])
ask c2: count(p in pays over 2026-02)
ask c1: count(p in pays over 2026-01)
"#;
        let refused = "refused: RUN-RECORD bank[o1] and card[o1] are stated same and fulfil pay[2026-03] and pay[2026-02]";
        let lines = lines_of("s1_two_b.tally", text);
        let mut want = vec![
            format!("b3: {refused}"),
            "  unfulfilled: pay[2026-01]".to_owned(),
            "  unfulfilled: pay[2026-02]".to_owned(),
            "  unfulfilled: pay[2026-03]".to_owned(),
            "  cut 2026-06-30".to_owned(),
            "c1: 1".to_owned(),
            "  unfulfilled: pay[2026-01]".to_owned(),
            "  cut 2026-06-30".to_owned(),
        ];
        for id in ["c2", "c3", "fp", "ob", "t2", "t3"] {
            want.push(format!("{id}: {refused}"));
            want.push("  cut 2026-06-30".to_owned());
        }
        assert_eq!(lines, want, "every line of the run");
    }

    /// B3 (`agent/law_via_at.tally`): an ask's law chain stops at a def with its own written
    /// `at`, so an `at reader` def that one reads is no `LOAD-LAWDATE` (L §4.3, P42).
    #[test]
    fn a_law_chain_stops_at_a_def_with_its_own_at() {
        let text = r#"model t
  tally 2
  on 2026-06-10
  start 2026-01-01
  end 2026-06-30

source s "s"
unit PLN: currency minor 0.01

input cutoff: date
input fixday: date

def cutoff_r: date at reader
  rule base = cutoff

def fixed: date at fixday
  rule base = cutoff_r

def amount: num at reader
  rule base = 1

ask q: amount law fixed
ask q2: fixed

version v0
  cite s
  governs always
  known 2026-01-01
  fixday = 2026-03-01 "s"

version v1
  cite s
  governs 2026-01-01..
  known 2026-01-01
  cutoff = 2026-04-01 "s"
"#;
        let lines = lines_of("law_via_at.tally", text);
        assert_eq!(
            lines,
            [
                "q: 1",
                "  law date 2026-04-01, cut 2026-06-10",
                "q2: 2026-04-01",
                "  cut 2026-06-10",
            ],
            "every line of the run"
        );
    }

    /// N2 (`agent/s6_sec_a.tally`, `s6_sec_b.tally`): the member that stands for a `same`
    /// class in a binding is its first in id order, whichever section states the class's
    /// first line (L §6.1: section order changes nothing; pick P42).
    #[test]
    fn a_same_class_stands_in_a_binding_by_its_first_id() {
        let jj = "version jj
  cite s
  governs 2026-03
  known 2026-04-03
  same bank[c1] = bank[b1]
";
        let rest = r#"version st_sav
  cite s
  governs 2026-03
  known 2026-04-03
  covers sav
  bank[c1] = (on: 2026-03-10, kind: transfers, leg cash from konto to sav amount 100.00 PLN) "c"

version st_konto
  cite s
  governs 2026-03
  known 2026-04-02
  covers konto
  bank[b1] = (on: 2026-03-10, kind: transfers, leg cash from konto to sav amount 100.00 PLN, fulfils: pay[2026-03]) "a"

version st_card
  cite s
  governs 2026-03
  known 2026-04-03
  covers card
  bank[a1] = (on: 2026-03-10, kind: transfers, leg cash from konto to sav amount 100.00 PLN) "b"
  same bank[a1] = bank[c1]

ask l: x.cash.amount for x in transfers over 2026-03
"#;
        let head = r#"model t
  tally 2
  on 2026-04-10
  start 2026-03-01
  end 2026-04-30

source s "s"
unit PLN: currency minor 0.01
place konto: accounts PLN lives 2026-01-01..
place sav: accounts PLN lives 2026-01-01..
place card: accounts PLN lives 2026-01-01..
input bank: observed
input amt: money PLN
input d_p: int

move pay[m in months]: transfers
  on day_of(m, d_p)
  leg cash from konto to sav amount amt

version v
  cite s
  governs always
  known 2026-01-01
  amt = 100.00 PLN "s"
  d_p = 10 "s"
  opening[konto] = 1000.00 PLN at 2026-03-01 start "s"
  opening[sav] = 0.00 PLN at 2026-03-01 start "s"
  opening[card] = 0.00 PLN at 2026-03-01 start "s"

"#;
        let want = [
            "l (x = bank[a1]): 100.00 PLN",
            "  cut 2026-04-10",
            "warning: WARN-UNREAD place card is read by nothing",
        ];
        let first = format!("{head}{jj}\n{rest}");
        let last = format!("{head}{rest}\n{jj}");
        assert_eq!(lines_of("s6_sec_a.tally", &first), want, "jj first");
        assert_eq!(lines_of("s6_sec_b.tally", &last), want, "jj last");
    }

    /// N4 (`agent/s6_reads.tally`): `explain` of an observed row's field in a `same` class
    /// whose rows disagree prints the class's `HOLE-SAME`, as the plain read does; judging the
    /// link the class's `fulfils` makes is no walk read (L §6.1, §8.1, E §10).
    #[test]
    fn explain_of_a_field_of_a_same_class_prints_its_hole() {
        let text = r#"model t
  tally 2
  on 2026-04-10
  start 2026-03-01
  end 2026-04-30

source s "s"
unit PLN: currency minor 0.01
place konto: accounts PLN lives 2026-01-01..
place sav: accounts PLN lives 2026-01-01..
place card: accounts PLN lives 2026-01-01..
input bank: observed
input amt: money PLN
input d_p: int

move pay[m in months]: transfers
  on day_of(m, d_p)
  leg cash from konto to sav amount amt

version v
  cite s
  governs always
  known 2026-01-01
  amt = 100.00 PLN "s"
  d_p = 10 "s"
  opening[konto] = 1000.00 PLN at 2026-03-01 start "s"
  opening[sav] = 0.00 PLN at 2026-03-01 start "s"
  opening[card] = 0.00 PLN at 2026-03-01 start "s"

version st_sav
  cite s
  governs 2026-03
  known 2026-04-03
  covers sav
  bank[c1] = (on: 2026-03-10, kind: transfers, leg cash from konto to sav amount 110.00 PLN) "c"

version st_konto
  cite s
  governs 2026-03
  known 2026-04-02
  covers konto
  bank[b1] = (on: 2026-03-10, kind: transfers, leg cash from konto to sav amount 100.00 PLN, fulfils: pay[2026-03]) "a"

version st_card
  cite s
  governs 2026-03
  known 2026-04-03
  covers card
  bank[a1] = (on: 2026-03-10, kind: transfers, leg cash from konto to sav amount 100.00 PLN) "b"
  same bank[c1] = bank[b1]
  same bank[a1] = bank[c1]

ask s4: explain bank[a1].cash.amount
ask s5: bank[a1].cash.amount
"#;
        let hole = "?\"bank[a1], bank[b1] and bank[c1] are one, but the amount of cash differs on 2026-03-10: 100.00 PLN (bank[a1]), 100.00 PLN (bank[b1]) and 110.00 PLN (bank[c1])\" (bank[a1], bank[b1], bank[c1])";
        let lines = lines_of("s6_reads.tally", text);
        let want = [
            format!("s4: {hole}"),
            "  cut 2026-04-10".to_owned(),
            format!("s5: {hole}"),
            "  cut 2026-04-10".to_owned(),
            "warning: WARN-UNREAD place card is read by nothing".to_owned(),
        ];
        assert_eq!(lines, want, "every line of the run");
    }
}
