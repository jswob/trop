//! The builder's tests of group 4's review: each review input, the lines refcore prints for the asks it names.
//! Spec: L §6.1 and §5.19 (a `same` class of movements: the member that stands for it, S1's
//! refusals, `explain` of its field), §4.3 (an ask's law chain), §4.4 (a window's key, a
//! lifespan test, a diff over two stacks), E §10 (an explanation's rows). The tests of a class
//! of rows of an input went with tally 2.2's S-8.
//! Twin: engine/tools/refcore (spec/2.1.8 at 61f8630b, the eighth turn, and at 00a00b22, the
//! tenth, for the fix review's F1); each expected line is its run of the same text
//! (`python3 -m refcore run`), each text an input of the review of build group 4 or of its fix
//! (`tmp/spec-2.1.8-build/review-g4/`, `review-g4fix/`).
//! Invariants: reads no test, vector or answer key (R-93); the texts are written here.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally::{Asks, Files, Given, Request, run};

    /// The lines a run of `text`, read from `path`, prints, every ask.
    fn lines_of(path: &str, text: &str) -> Vec<String> {
        let given = vec![Given {
            path: path.to_owned(),
            bytes: Some(text.as_bytes().to_vec()),
        }];
        let request = Request::new(Asks::Every);
        let answer = run(Files::new(given), &request).expect("an answer");
        answer.lines().to_vec()
    }

    /// The lines a run of `text`, read from `path`, prints for the asks `asks`.
    fn asked_lines_of(path: &str, text: &str, asks: &[&str]) -> Vec<String> {
        let given = vec![Given {
            path: path.to_owned(),
            bytes: Some(text.as_bytes().to_vec()),
        }];
        let request = Request::new(Asks::Named(asks.iter().map(|a| (*a).to_owned()).collect()));
        let answer = run(Files::new(given), &request).expect("an answer");
        answer.lines().to_vec()
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
        let hole = "?\"bank[a1], bank[b1] and bank[c1] are one, but leg cash differs on 2026-03-10: 100.00 PLN (bank[a1]), 100.00 PLN (bank[b1]) and 110.00 PLN (bank[c1])\" (bank[a1], bank[b1], bank[c1])";
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

    /// F1 of the fix's review (`n2_range_9.tally`, `n2_list.tally`): the member that stands
    /// for a `same` class is its first in index order, a range or list keeping its own order:
    /// `pay[9]` before `pay[10]` on `1..12`, `pay[zz]` on `[zz, aa, mm]` (L §2.5, pick P50).
    #[test]
    fn a_same_class_stands_by_its_first_id_in_index_order() {
        let text = |index: &str, line: &str| {
            format!(
                r#"model n2_list
  tally 2
  on 2026-04-10
  start 2026-03-01
  end 2026-04-30

source s "s"
unit PLN: currency minor 0.01
index who = {index}
place konto: accounts PLN lives 2026-01-01..
place world: outside lives 1900-01-01..
input dd: date
input amt: money PLN
kind wages: transfers

move pay[w in who]: wages
  on dd
  leg cash from world to konto amount amt

ask l: x.cash.amount for x in wages over 2026-03

version v
  cite s
  governs always
  known 2026-01-01
  opening[konto] = 1000.00 PLN at 2026-03-01 start "s"
  dd = 2026-03-10 "s"
  amt = 500.00 PLN "s"

version sm
  cite s
  known 2026-01-02
  {line}
"#
            )
        };
        let heads = |lines: Vec<String>| -> Vec<String> {
            lines.into_iter().filter(|l| l.starts_with("l (")).collect()
        };
        let range = heads(lines_of(
            "n2_range_9.tally",
            &text("1..12", "same pay[9] = pay[10]"),
        ));
        assert!(
            range.contains(&"l (x = pay[9]): 500.00 PLN".to_owned()),
            "pay[9] stands: {range:?}"
        );
        assert!(
            !range.iter().any(|l| l.starts_with("l (x = pay[10])")),
            "pay[10] is bound with it: {range:?}"
        );
        let list = heads(lines_of(
            "n2_list.tally",
            &text("[zz, aa, mm]", "same pay[aa] = pay[zz]"),
        ));
        assert_eq!(
            list,
            [
                "l (x = pay[zz]): 500.00 PLN".to_owned(),
                "l (x = pay[mm]): 500.00 PLN".to_owned()
            ],
            "pay[zz] stands, in the list's order"
        );
    }

    /// The final check's `r11/sum_first` (tally 2.1.8 R11): a sum whose first term is a booked
    /// leg refused at its amount builds the walk its hole twin's read builds, so the run warns
    /// as the twin's does: `b` goes below zero on the walk, though the sum reads no balance.
    #[test]
    fn a_refused_leg_read_warns_as_its_hole_twin() {
        let text = r#"model t
  tally 2
  on 2026-06-30
  start 2026-01-01
  end 2026-06-30

source src "s"
unit PLN: currency minor 0.01
input amt: money PLN
input dom: int
input dq: int
input div[m in months]: num
place a: accounts PLN lives 2020-01-01..
place b: accounts PLN lives 2020-01-01..
place world: outside lives 1900-01-01..

move pay[m in months]: transfers
  on day_of(m, dom)
  at start_of(m)
  leg cash from a to world amount round(amt * div[m] / div[m], minor(PLN), half-even)

move q[m in months]: transfers
  on day_of(m, dq)
  at start_of(m)
  leg cash from b to world amount amt

version p
  cite src
  governs 2026..
  known 2026-01-01
  amt = 400.00 PLN "p"
  dom = 4 "p"
  dq = 10 "p"
  div[2026-01] = 1 "ok"
  div[2026-02] = 0 "zero"
  div[2026-03..2026-06] = 1 "ok"
  opening[a] = 1 000.00 PLN at 2026-01-01 start "o"
  opening[b] = 1 000.00 PLN at 2026-01-01 start "o"

ask s: sum(t.cash.amount for t in transfers over 2026-02)
"#;
        let lines = asked_lines_of("r11_warns.tally", text, &["s"]);
        assert_eq!(
            lines,
            [
                "s: refused: RUN-DIV division by zero",
                "  unfulfilled: pay[2026-02]",
                "  unfulfilled: q[2026-02]",
                "  cut 2026-06-30",
                "warning: WARN-NEGATIVE b is below zero from 2026-03-10",
            ],
            "the refusal, and the warning the walk meets"
        );
    }

    /// The final check's `fixes/gate8/d512_hole_ans` (E §10, §5 step 1): an explanation lists
    /// the row a later version answers a hole booking with right after that booking's hole,
    /// as the instance is answered where it is made, before the next instance's rows.
    #[test]
    fn an_answer_row_is_listed_after_the_hole_it_answers() {
        let text = r#"model t
  tally 2
  on 2026-08-31
  start 2026-01-01
  end 2026-08-31

source src "s"
unit PLN: currency minor 0.01
input fee[m in months]: money PLN
input dd: int
place a: accounts PLN lives 2020-01-01..
place out: outside lives 1900-01-01..

move fee_out[m in months]: transfers
  on day_of(m, dd)
  at start_of(m)
  leg cash from a to out amount fee[m]

version base
  cite src
  governs 2026..
  known 2026-01-01
  dd = 3 "s"
  fee[2026-01] = 10.00 PLN "s"
  fee[2026-05..2026-08] = 10.00 PLN "s"
  opening[a] = 500.00 PLN at 2026-01-01 start "s"

version open_q
  cite src
  governs 2026-02..2026-04
  known 2026-01-01
  fee[2026-02] = ?"What was the February fee?"
  fee[2026-03] = ?"What was the March fee?"
  fee[2026-04] = ?"What was the April fee?"

version later
  cite src
  governs 2026-03
  known 2026-06-01
  replace fee[2026-03] = 12.00 PLN "s"

ask e: explain fee_out[2026-03].cash.amount
"#;
        let rows: Vec<String> = lines_of("d512_answer_order.tally", text)
            .into_iter()
            .filter(|l| l.starts_with("  ["))
            .collect();
        assert_eq!(
            rows,
            [
                "  [1] row base:dd = 3",
                "  [2] row base:fee[2026-01] = 10.00 PLN",
                "  [3] row open_q:fee[2026-02] = ?\"What was the February fee?\" (fee[2026-02])",
                "  [4] row open_q:fee[2026-03] = ?\"What was the March fee?\" (fee[2026-03])",
                "  [5] row later:fee[2026-03] = 12.00 PLN",
                "  [6] row open_q:fee[2026-04] = ?\"What was the April fee?\" (fee[2026-04])",
                "  [7] row base:fee[2026-05..2026-08] = 10.00 PLN",
            ],
            "the answer's row right after the hole it answers"
        );
    }

    /// The final check's `d77/lifespan` (L §4.4 l.672, a refusal that only some of a
    /// window's days give; tally 2.1.8 item 77): a window leg whose lifespan test comes out
    /// both ways is the window's hole from the window's first day; a fixed-day leg's lifespan
    /// refusal the place meets later rests on no balance, so it is given on every day and
    /// stands beside that hole (L §8 l.2217; P31 said whole, P41, F6); a pin on either side
    /// of the split gives each side's refusal.
    #[test]
    fn a_lifespan_refusal_after_a_window_lifespan_test_stands() {
        let text = r#"model t
  tally 2
  on 2026-01-01
  start 2026-01-01
  end 2026-06-30

source src "s"
unit PLN: currency minor 0.01
place pot: accounts PLN lives 2025-01-01..
place fresh: accounts PLN lives 2026-04-08..
place shut: accounts PLN lives 2025-01-01..2026-04-06
input amt: money PLN
input dw: date
input df: date

move into_fresh: transfers
  on dw
  leg cash from pot to fresh amount amt

move into_shut: transfers
  on df
  leg cash from pot to shut amount amt

version v
  cite src
  governs always
  known 2025-12-01
  amt = 50.00 PLN "s"
  dw = window(2026-04-05, 2026-04-10) "s"
  df = 2026-04-08 "s"
  opening[pot] = 400.00 PLN at 2026-01-01 start "s"
  opening[shut] = 0.00 PLN at 2026-01-01 start "s"

ask b: bal(pot, 2026-04-30)
ask b_05: bal(pot, 2026-04-30) with dw = 2026-04-05
ask b_08: bal(pot, 2026-04-30) with dw = 2026-04-08
"#;
        let lines = lines_of("d77_lifespan_first.tally", text);
        assert_eq!(
            lines,
            [
                "b: refused: RUN-LIFESPAN into_shut.cash moves into or out of shut outside its \
                 lifespan",
                "  cut 2026-01-01",
                "b_05: refused: RUN-LIFESPAN into_fresh.cash moves into or out of fresh outside \
                 its lifespan",
                "  pinned: dw = 2026-04-05",
                "  cut 2026-01-01",
                "b_08: refused: RUN-LIFESPAN into_shut.cash moves into or out of shut outside its \
                 lifespan",
                "  pinned: dw = 2026-04-08",
                "  cut 2026-01-01",
            ],
            "the lifespan refusal every day gives, then one refusal per pinned side"
        );
    }

    /// The final check's `fixes/rv/rc_diff_two_stacks_each`, restated without `windows each`
    /// (L §4.4, §8.2; tally 2.2): a diff whose worlds sit on two stacks names in its register
    /// only the unknown days its holes carry: `db`, the day `main` is paid on, never `da`, a
    /// window its walk meets at another place.
    #[test]
    fn a_two_stack_diff_names_only_the_unknown_days_its_holes_carry() {
        let model = r#"model q
  tally 2
  on 2026-05-01
  start 2026-01-01
  end 2026-12-31

source s "s"
unit PLN: currency minor 0.01
place main: accounts PLN lives 2026-01-01..
place side: accounts PLN lives 2026-01-01..
place world: outside lives 1900-01-01..
input fee: money PLN
input da: date
input db: date
input d0: date

move to_side: transfers
  on da
  at d0
  leg cash from world to side amount fee

move to_main: transfers
  on db
  at d0
  leg cash from world to main amount fee

version v
  cite s
  governs always
  known 2026-01-01
  fee = 4.00 PLN "s"
  d0 = 2026-01-01 "s"
  da = window(2026-02-01, 2026-02-09) "s"
  db = window(2026-02-03, 2026-02-05) "s"
  opening[main] = 0.00 PLN at 2026-01-01 start "s"
  opening[side] = 0.00 PLN at 2026-01-01 start "s"

ask d: diff(bal(main, 2026-02-04)) in s2 against s1
"#;
        let scenario = |name: &str, fee: &str| {
            format!(
                "scenario {name} over q\n  tally 2\n  from 2026-01-01\n\nversion {name}_v\n  \
                 cite s\n  governs 2026-01-01..\n  known 2026-01-01\n  replace fee = {fee} PLN \
                 \"s\"\n"
            )
        };
        let given = [
            ("m.tally", model.to_owned()),
            ("s1.tally", scenario("s1", "5.00")),
            ("s2.tally", scenario("s2", "6.00")),
        ]
        .into_iter()
        .map(|(path, text)| Given {
            path: path.to_owned(),
            bytes: Some(text.into_bytes()),
        })
        .collect();
        let request = Request::new(Asks::Every);
        let answer = run(Files::new(given), &request).expect("an answer");
        assert_eq!(
            answer.lines(),
            [
                "d: diff in s2 against s1: 0 changed, 0 added, 0 removed, 1 unchanged",
                "  unknown day: db",
                "  world: s1",
                "  world: s2",
                "  cut 2026-05-01",
            ],
            "db alone among the unknown days"
        );
    }
}
