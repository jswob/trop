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

    /// `lines` are `expected`, every one, in order.
    fn all_of(lines: &[String], expected: &[&str]) {
        assert_eq!(lines, expected, "every line of the run");
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
            &lines_of("t8.tally", text),
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
            &lines_of("t1_la.tally", text),
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
            &lines_of("t1_pl.tally", text),
            &[
                "pl: refused: RUN-DOMAIN stated of broker: its readings at 2026-02-15 end are in EUR and PLN",
            ],
        );
    }

    /// B7 (`t7.tally`): a level of 0 from a def, two hole terms: the refusal outranks the holes.
    #[test]
    fn a_refused_quantile_level_refuses_before_its_hole_terms() {
        let text = r#"model hh
  tally 2
  on 2026-10-20
  start 2026-01-01
  end 2026-12-31

source plan "plan"
unit PLN: currency minor 0.01
index ks = [a, b, c]
input z[k in ks]: money PLN
input w[k in ks]: int
input x: money PLN
input lv: num
version zz
  cite plan
  governs always
  known 2026-01-01
  z[a] = 10.00 PLN "s"
  z[b] = ?"z of b" "s"
  z[c] = ?"z of c" "s"
  w[a] = 1 "s"
  w[b] = 1 "s"
  w[c] = 0 "s"
  x = ?"what is x" "s"
  lv = 0 "s"
ask q1: quantile(z[k] for k in ks, lv)
ask q2: quantile(z[k] for k in ks where w[k] > 1, lv)
ask q3: quantile(z[k] for k in [a], lv)
"#;
        holds(
            &lines_of("t7.tally", text),
            &["q1: refused: RUN-DOMAIN quantile has no meaning for the level 0"],
        );
    }

    /// B6 (`t14.tally`): a reading `after save[kz]`, `kz` a bare name that is no key of the list index `ks`.
    #[test]
    fn a_reading_after_a_key_its_index_lacks_is_load_key() {
        let text = r#"model t
  tally 2
  on 2026-06-10
  start 2026-03-01
  end 2026-06-30

source s "s"
unit PLN: currency minor 0.01
place konto: accounts PLN lives 2026-01-01..
place sav: accounts PLN lives 2026-01-01..
place world: outside lives 1900-01-01..
input bank: observed
input keep: money PLN
input d_save: int
input d_sday: date
input d_rent: int
input rentamt: money PLN

index ks = [ka]
move save[k in ks]: transfers
  on d_sday
  reads konto
  leg cash from konto to sav amount max(0 PLN, bal(konto) - keep)

move rent[m in months]: transfers
  on day_of(m, d_rent)
  leg cash from konto to world amount rentamt

version v
  cite s
  governs always
  known 2026-01-01
  keep = 100.00 PLN "s"
  d_save = 15 "s"
  d_sday = 2026-03-15 "s"
  d_rent = 13 "s"
  rentamt = 30.00 PLN "s"
  opening[konto] = 1050.00 PLN at 2026-03-01 start "s"
  opening[sav] = 0.00 PLN at 2026-03-01 start "s"

version st
  cite s
  governs 2026-03
  known 2026-04-02
  covers konto
  anchor konto
  bank[o1] = (on: 2026-03-12, kind: transfers, leg cash from konto to sav amount 500.00 PLN, fulfils: save[ka]) "o1"
  bank[o2] = (on: 2026-03-13, kind: transfers, leg cash from konto to world amount 30.00 PLN, fulfils: rent[2026-03]) "o2"
  stated[konto] = 550.00 PLN after save[kz] "saldo"

ask b: bal(konto, 2026-03-31)
ask md: bal(konto, 2026-03-14) modelled, bal(konto, 2026-03-31) modelled
ask g: last(stated[konto], 2026-05-31), since(stated[konto], 2026-05-31)
"#;
        holds(
            &lines_of("t14.tally", text),
            &[
                "refused: LOAD-KEY stated[konto] after save[kz]: kz is not a key of ks (t14.tally:49)",
            ],
        );
    }

    /// B8 (`t3.tally`): `bal … modelled` starting from a plain reading later than a reading `after m`: m's leg stays out (pick P39).
    #[test]
    fn a_modelled_read_never_counts_m_after_a_reading_after_m() {
        let text = r#"model t
  tally 2
  on 2026-06-10
  start 2026-03-01
  end 2026-06-30

source s "s"
unit PLN: currency minor 0.01
place konto: accounts PLN lives 2026-01-01..
place sav: accounts PLN lives 2026-01-01..
place world: outside lives 1900-01-01..
input bank: observed
input keep: money PLN
input d_save: int
input d_rent: int
input rentamt: money PLN

move save[m in months]: transfers
  on day_of(m, d_save)
  reads konto
  leg cash from konto to sav amount max(0 PLN, bal(konto) - keep)

move rent[m in months]: transfers
  on day_of(m, d_rent)
  leg cash from konto to world amount rentamt

version v
  cite s
  governs always
  known 2026-01-01
  keep = 100.00 PLN "s"
  d_save = 15 "s"
  d_rent = 13 "s"
  rentamt = 30.00 PLN "s"
  opening[konto] = 1050.00 PLN at 2026-03-01 start "s"
  opening[sav] = 0.00 PLN at 2026-03-01 start "s"

version st
  cite s
  governs 2026-03
  known 2026-04-02
  covers konto
  anchor konto
  bank[o1] = (on: 2026-03-12, kind: transfers, leg cash from konto to sav amount 500.00 PLN, fulfils: save[2026-03]) "o1"
  bank[o2] = (on: 2026-03-13, kind: transfers, leg cash from konto to world amount 30.00 PLN, fulfils: rent[2026-03]) "o2"
  stated[konto] = 550.00 PLN after save[2026-03] "saldo"
  stated[konto] = 520.00 PLN at 2026-03-14 end "saldo2"

ask b: bal(konto, 2026-03-31)
ask md: bal(konto, 2026-03-14) modelled, bal(konto, 2026-03-31) modelled, bal(konto, 2026-03-13) modelled
"#;
        holds(
            &lines_of("t3.tally", text),
            &["md: (520.00 PLN, 520.00 PLN, 520.00 PLN)"],
        );
    }

    /// B5 (`t12.tally`): `flows` over spans after a window-day decision's window has ended: nothing moves there.
    #[test]
    fn flows_after_a_window_days_end_is_decided() {
        let text = r#"model t
  tally 2
  on 2026-06-10
  start 2026-03-01
  end 2026-06-30

source s "s"
unit PLN: currency minor 0.01
place konto: accounts PLN lives 2026-01-01..
place sav: accounts PLN lives 2026-01-01..
place world: outside lives 1900-01-01..
input keep: money PLN
input d_save: date
input go: bool

move save: transfers
  on d_save
  reads konto
  when go
  leg cash from konto to sav amount max(0 PLN, bal(konto) - keep)

move mm[m in months]: transfers
  on end_of(m) - 1 day
  reads konto
  leg cash from konto to sav amount max(0 PLN, bal(konto) - keep)

version v
  cite s
  governs always
  known 2026-01-01
  keep = 100.00 PLN "s"
  d_save = window(2026-03-16, 2026-03-25) "s"
  go = true "s"
  opening[konto] = 1050.00 PLN at 2026-03-01 start "s"
  opening[sav] = 0.00 PLN at 2026-03-01 start "s"

version st
  cite s
  governs 2026-03..2026-05
  known 2026-01-02
  anchor konto
  stated[konto] = 90.00 PLN after save "saldo"

ask b: bal(konto, 2026-03-31), bal(konto, 2026-04-30), flows(konto, 2026-04), flows(konto, 2026-04-01 .. 2026-04-10)
"#;
        holds(
            &lines_of("t12.tally", text),
            &["b: (90.00 PLN, 90.00 PLN, 0.00 PLN, 0.00 PLN)"],
        );
    }

    /// B9 (`r15_b3.tally`): `partial` projections whose start balance holds a hole: the start's terms are the base's at the point.
    #[test]
    fn a_projection_from_a_start_hole_keeps_the_linear_form() {
        let text = r#"model hh
  tally 2
  on 2026-10-20
  start 2026-01-01
  end 2026-12-31

source plan "plan"
source bk "bank"
unit PLN: currency minor 0.01
input g_amt: money PLN
input dom: int
input divisor[m in months]: num
input hamt[m in months]: money PLN
place konto: accounts PLN lives 2019-05-01..
place k2: accounts PLN lives 2019-05-01..
place world: outside lives 1900-01-01..
kind maybes: transfers uncertain
input uamt: money PLN
move gift: maybes
  on dgift
  at dgift
  leg cash from world to k2 amount uamt
input dgift: date

move groc[m in months]: transfers
  on day_of(m, dom)
  at start_of(m)
  leg cash from konto to world amount round(g_amt * divisor[m] / divisor[m], minor(PLN), half-even)

move hm[m in months]: transfers
  on day_of(m, dom)
  at start_of(m)
  leg cash from k2 to world amount hamt[m]

version plan_2026
  cite plan
  governs 2026..
  known 2026-01-01
  g_amt = 500.00 PLN "plan"
  dom = 5 "plan"
  divisor[2026-01] = 1 "ok"
  divisor[2026-02] = 0 "zero"
  divisor[2026-03..2026-12] = 1 "ok"
  hamt[2026-01] = 1.00 PLN "ok"
  hamt[2026-02] = ?"how much in Feb" "hole"
  hamt[2026-03] = ?"how much in Mar" "hole"
  hamt[2026-04..2026-12] = 1.00 PLN "ok"
  opening[konto] = 10 000.00 PLN at 2026-01-01 start "w"
  opening[k2] = 10 000.00 PLN at 2026-01-01 start "w"

version st
  cite bk
  governs 2026-03
  known 2026-04-02
  anchor konto
  anchor k2
  stated[konto] = 200.00 PLN at 2026-03-10 end "saldo"
  stated[k2] = 200.00 PLN at 2026-03-10 end "saldo"

index ks = [a, b]
input z[k in ks]: money PLN
input two: num
version zz
  cite plan
  governs always
  known 2026-01-01
  z[a] = 10.00 PLN "s"
  two = 2 "s"
  uamt = ?"gift size" "hole"
  dgift = 2026-03-20 "s"
  z[b] = ?"z of b" "s"

ask b3: bal(k2, 2026-03-31) from 2026-03-05 partial
ask b4: bal(k2, 2026-03-31) from 2026-03-05
ask b5: bal(k2, 2026-03-04) partial
ask b6: bal(k2, 2026-03-31) from 2026-03-06 partial
ask b7: bal(k2, 2026-03-04) from 2026-03-04 partial
ask b8: explain bal(k2, 2026-03-31) from 2026-03-05
"#;
        holds(
            &lines_of("r15_b3.tally", text),
            &[
                "b3: 9 999.00 PLN + -1 × ?\"how much in Feb\" (hamt[2026-02]) + -1 × ?\"how much in Mar\" (hamt[2026-03])",
                "b6: 9 999.00 PLN + -1 × ?\"how much in Feb\" (hamt[2026-02]) + -1 × ?\"how much in Mar\" (hamt[2026-03])",
                "b7: 9 999.00 PLN + -1 × ?\"how much in Feb\" (hamt[2026-02])",
            ],
        );
    }

    /// N1 (`r14_none.tally`): `last` after a unitless `replace … = none`: nothing is read, so no `held:` line (every line checked).
    #[test]
    fn last_of_a_reading_replaced_by_none_holds_nothing() {
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
place world: outside lives 1900-01-01..

input dbuy: date
input amt: money EUR
move buy: transfers
  on dbuy
  leg cash from world to broker amount amt

version v1
  cite s
  governs always
  known 2026-01-01
  anchor broker
  dbuy = 2026-02-10 "s"
  amt = 3.00 EUR "s"
  opening[broker] = 1.00 EUR at 2026-01-01 start "s"
  opening[broker] = 2.00 PLN at 2026-01-01 start "s"
  stated[broker] = 6.00 EUR at 2026-02-15 end "s"
  stated[broker] = 10.00 PLN at 2026-02-15 end "s"

version v2
  cite s
  governs always
  known 2026-03-01
  anchor broker
  replace stated[broker] = none at 2026-02-15 end "s"

ask a: bal(broker, EUR, 2026-02-20), bal(broker, PLN, 2026-02-20)
ask g: gap(broker, 2026-02-15)
ask l: last(stated[broker], 2026-02-20)
ask s1: since(stated[broker], 2026-02-20)
"#;
        all_of(
            &lines_of("r14_none.tally", text),
            &[
                "a: (4.00 EUR, 2.00 PLN)",
                "  unfulfilled: buy",
                "  cut 2026-06-10",
                "g: ?\"stated[broker] has no reading at 2026-02-15\" (stated[broker])",
                "  cut 2026-06-10",
                "l: ?\"stated[broker] has no reading on or before 2026-02-20\" (stated[broker])",
                "  cut 2026-06-10",
                "s1: ?\"stated[broker] has no reading on or before 2026-02-20\" (stated[broker])",
                "  cut 2026-06-10",
            ],
        );
    }

    /// G1 (`p5_min.tally`): a plan booked before K whose amount reads `last` of a unitless hole: the hole, the same at its booking cut and at K (pick P46).
    #[test]
    fn a_plan_reading_a_lone_unitless_hole_reads_it_alike_at_every_cut() {
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
place konto: accounts PLN lives 2025-01-01..
place world: outside lives 1900-01-01..
input d_p: date
input x1: money PLN
input x2: money PLN
input d_q: date
def lb: money EUR = last(stated[broker], d_q)
def amt: money PLN = if lb > 0 EUR then x1 else x2
move pay: transfers
  on d_p
  leg cash from world to konto amount amt

version v1
  cite s
  governs always
  known 2026-01-01
  anchor broker
  d_p = 2026-03-05 "s"
  x1 = 1.00 PLN "s"
  x2 = 2.00 PLN "s"
  d_q = 2026-02-16 "s"
  opening[konto] = 0.00 PLN at 2026-01-01 start "s"
  opening[broker] = 1.00 EUR at 2026-01-01 start "s"



version v2
  cite s
  governs always
  known 2026-03-01
  anchor broker
  stated[broker] = ?"not read" at 2026-02-15 end "s"

ask k: bal(konto, 2026-03-31)
"#;
        all_of(
            &lines_of("p5_min.tally", text),
            &[
                "k: ?\"not read\" (stated[broker])",
                "  unfulfilled: pay",
                "  held: stated[broker] = ?\"not read\" (stated[broker]) (v2), read for 2026-02-16",
                "  cut 2026-06-10",
            ],
        );
    }

    /// G1 (`p5_plan.tally`): the same plan with a EUR reading beside the hole: `RUN-DOMAIN … in EUR and PLN` wherever it is read (pick P46).
    #[test]
    fn a_plan_reading_a_unitless_hole_beside_a_unit_is_run_domain_at_every_cut() {
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
place konto: accounts PLN lives 2025-01-01..
place world: outside lives 1900-01-01..
input d_p: date
input x1: money PLN
input x2: money PLN
input d_q: date
def lb: money EUR = last(stated[broker], d_q)
def amt: money PLN = if lb > 0 EUR then x1 else x2
move pay: transfers
  on d_p
  leg cash from world to konto amount amt

version v1
  cite s
  governs always
  known 2026-01-01
  anchor broker
  d_p = 2026-03-05 "s"
  x1 = 1.00 PLN "s"
  x2 = 2.00 PLN "s"
  d_q = 2026-02-16 "s"
  opening[konto] = 0.00 PLN at 2026-01-01 start "s"
  opening[broker] = 1.00 EUR at 2026-01-01 start "s"
  opening[broker] = 2.00 PLN at 2026-01-01 start "s"
  stated[broker] = 4.00 EUR at 2026-02-15 end "s"


version v2
  cite s
  governs always
  known 2026-03-01
  anchor broker
  stated[broker] = ?"not read" at 2026-02-15 end "s"

ask k: bal(konto, 2026-03-31)
"#;
        all_of(
            &lines_of("p5_plan.tally", text),
            &[
                "k: refused: RUN-DOMAIN last of broker: its readings at 2026-02-15 end are in EUR and PLN",
                "  cut 2026-06-10",
            ],
        );
    }

    /// G1 (`p4_planlast.tally`): a plan's leg reading `last` of the place: every ask answers as refcore, none `ENGINE-INVARIANT` (pick P46).
    #[test]
    fn reads_of_a_unitless_hole_inside_a_plans_leg_load_and_answer() {
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
place konto: accounts PLN lives 2025-01-01..
place world: outside lives 1900-01-01..
input d_p: date
input x1: money PLN
input x2: money PLN
input x5: money PLN
input x6: money PLN
input d_q: date
def lb: money EUR = last(stated[broker], d_q)
def amt: money PLN = if lb > 0 EUR then x1 else x2
move pay: transfers
  on d_p
  leg cash from world to konto amount amt
move dec: transfers
  on d_p
  reads konto
  leg cash from world to konto amount if last(stated[broker], d_q) > 0 EUR and bal(konto) > 0 PLN then x5 else x6

version v1
  cite s
  governs always
  known 2026-01-01
  anchor broker
  d_p = 2026-03-05 "s"
  x1 = 1.00 PLN "s"
  x2 = 2.00 PLN "s"
  x5 = 5.00 PLN "s"
  x6 = 6.00 PLN "s"
  d_q = 2026-02-16 "s"
  opening[konto] = 0.00 PLN at 2026-01-01 start "s"
  opening[broker] = 1.00 EUR at 2026-01-01 start "s"
  opening[broker] = 2.00 PLN at 2026-01-01 start "s"
  stated[broker] = 4.00 EUR at 2026-02-15 end "s"


version v2
  cite s
  governs always
  known 2026-03-01
  anchor broker
  stated[broker] = ?"not read" at 2026-02-15 end "s"

ask a: bal(broker, EUR, 2026-02-20), bal(broker, PLN, 2026-02-20)
ask la: last(stated[broker], 2026-02-16)
ask ga: gap(broker, 2026-02-15)
ask si: since(stated[broker], 2026-02-16)
ask pl: stated[broker] law 2026-02-15
ask k: bal(konto, 2026-03-31)
ask lbx: lb
"#;
        all_of(
            &lines_of("p4_planlast.tally", text),
            &[
                "a: (4.00 EUR, ?\"not read\" (stated[broker]))",
                "  anchor: broker 2026-02-15 end, stated 4.00 EUR, difference 3.00 EUR",
                "  anchor: broker 2026-02-15 end, stated ?\"not read\" (stated[broker]), difference ?\"not read\" (stated[broker])",
                "  cut 2026-06-10",
                "ga: refused: RUN-DOMAIN gap of broker: its readings at 2026-02-15 end are in EUR and PLN",
                "  cut 2026-06-10",
                "k: refused: RUN-DOMAIN last of broker: its readings at 2026-02-15 end are in EUR and PLN",
                "  cut 2026-06-10",
                "la: refused: RUN-DOMAIN last of broker: its readings at 2026-02-15 end are in EUR and PLN",
                "  cut 2026-06-10",
                "lbx: refused: RUN-DOMAIN last of broker: its readings at 2026-02-15 end are in EUR and PLN",
                "  cut 2026-06-10",
                "pl: refused: RUN-DOMAIN stated of broker: its readings at 2026-02-15 end are in EUR and PLN",
                "  law date 2026-02-15, cut 2026-06-10",
                "si: 1",
                "  cut 2026-06-10",
            ],
        );
    }

    /// N-a (`b6_def_key.tally`): a reading `after save[pick]`, `pick` a def's name and no key of `ks`: a reading's key is a literal (L §2.5).
    #[test]
    fn a_reading_after_a_key_that_names_a_def_is_load_key() {
        let text = r#"model t
  tally 2
  on 2026-06-10
  start 2026-03-01
  end 2026-06-30

source s "s"
unit PLN: currency minor 0.01
place konto: accounts PLN lives 2026-01-01..
place sav: accounts PLN lives 2026-01-01..
place world: outside lives 1900-01-01..
input bank: observed
input keep: money PLN
def pick: key ks = ka
input d_save: int
input d_sday: date
input d_rent: int
input rentamt: money PLN

index ks = [ka]
move save[k in ks]: transfers
  on d_sday
  reads konto
  leg cash from konto to sav amount max(0 PLN, bal(konto) - keep)

move rent[m in months]: transfers
  on day_of(m, d_rent)
  leg cash from konto to world amount rentamt

version v
  cite s
  governs always
  known 2026-01-01
  keep = 100.00 PLN "s"
  d_save = 15 "s"
  d_sday = 2026-03-15 "s"
  d_rent = 13 "s"
  rentamt = 30.00 PLN "s"
  opening[konto] = 1050.00 PLN at 2026-03-01 start "s"
  opening[sav] = 0.00 PLN at 2026-03-01 start "s"

version st
  cite s
  governs 2026-03
  known 2026-04-02
  covers konto
  anchor konto
  bank[o1] = (on: 2026-03-12, kind: transfers, leg cash from konto to sav amount 500.00 PLN, fulfils: save[ka]) "o1"
  bank[o2] = (on: 2026-03-13, kind: transfers, leg cash from konto to world amount 30.00 PLN, fulfils: rent[2026-03]) "o2"
  stated[konto] = 550.00 PLN after save[pick] "saldo"

ask b: bal(konto, 2026-03-31)
ask md: bal(konto, 2026-03-14) modelled, bal(konto, 2026-03-31) modelled
ask g: last(stated[konto], 2026-05-31), since(stated[konto], 2026-05-31)
"#;
        holds(
            &lines_of("b6_def_key.tally", text),
            &[
                "refused: LOAD-KEY stated[konto] after save[pick]: pick is not a key of ks (b6_def_key.tally:50)",
            ],
        );
    }

    /// N-a (`p9_defkey.tally`): the same with reads of the reading: load refuses it, no read meets `RUN-RECORD`.
    #[test]
    fn a_reading_after_a_def_named_key_refuses_at_load_not_at_its_reads() {
        let text = r#"model t
  tally 2
  on 2026-06-10
  start 2026-03-01
  end 2026-06-30

source s "s"
unit PLN: currency minor 0.01
place konto: accounts PLN lives 2026-01-01..
place sav: accounts PLN lives 2026-01-01..
place world: outside lives 1900-01-01..
input bank: observed
input keep: money PLN
input d_save: int
input d_sday: date
input d_rent: int
input rentamt: money PLN

index ks = [ka]
def pick: key ks = ka
move save[k in ks]: transfers
  on d_sday
  reads konto
  leg cash from konto to sav amount max(0 PLN, bal(konto) - keep)

move rent[m in months]: transfers
  on day_of(m, d_rent)
  leg cash from konto to world amount rentamt

version v
  cite s
  governs always
  known 2026-01-01
  keep = 100.00 PLN "s"
  d_save = 15 "s"
  d_sday = 2026-03-15 "s"
  d_rent = 13 "s"
  rentamt = 30.00 PLN "s"
  opening[konto] = 1050.00 PLN at 2026-03-01 start "s"
  opening[sav] = 0.00 PLN at 2026-03-01 start "s"

version st
  cite s
  governs 2026-03
  known 2026-04-02
  covers konto
  anchor konto
  bank[o1] = (on: 2026-03-12, kind: transfers, leg cash from konto to sav amount 500.00 PLN, fulfils: save[ka]) "o1"
  bank[o2] = (on: 2026-03-13, kind: transfers, leg cash from konto to world amount 30.00 PLN, fulfils: rent[2026-03]) "o2"
  stated[konto] = 550.00 PLN after save[pick] "saldo"

ask b: bal(konto, 2026-03-31)
ask md: bal(konto, 2026-03-14) modelled, bal(konto, 2026-03-31) modelled
ask g: last(stated[konto], 2026-05-31), since(stated[konto], 2026-05-31)
"#;
        holds(
            &lines_of("p9_defkey.tally", text),
            &[
                "refused: LOAD-KEY stated[konto] after save[pick]: pick is not a key of ks (p9_defkey.tally:50)",
            ],
        );
    }

    /// N-b (`b6_wrongaxis.tally`): `after rent[ka]`, rent a plan keyed by months: the key is checked before the order (E §3.1).
    #[test]
    fn a_reading_after_a_plan_at_a_key_of_another_index_is_load_key() {
        let text = r#"model t
  tally 2
  on 2026-06-10
  start 2026-03-01
  end 2026-06-30

source s "s"
unit PLN: currency minor 0.01
place konto: accounts PLN lives 2026-01-01..
place sav: accounts PLN lives 2026-01-01..
place world: outside lives 1900-01-01..
input bank: observed
input keep: money PLN
input d_save: int
input d_sday: date
input d_rent: int
input rentamt: money PLN

index ks = [ka]
move save[k in ks]: transfers
  on d_sday
  reads konto
  leg cash from konto to sav amount max(0 PLN, bal(konto) - keep)

move rent[m in months]: transfers
  on day_of(m, d_rent)
  leg cash from konto to world amount rentamt

version v
  cite s
  governs always
  known 2026-01-01
  keep = 100.00 PLN "s"
  d_save = 15 "s"
  d_sday = 2026-03-15 "s"
  d_rent = 13 "s"
  rentamt = 30.00 PLN "s"
  opening[konto] = 1050.00 PLN at 2026-03-01 start "s"
  opening[sav] = 0.00 PLN at 2026-03-01 start "s"

version st
  cite s
  governs 2026-03
  known 2026-04-02
  covers konto
  anchor konto
  bank[o1] = (on: 2026-03-12, kind: transfers, leg cash from konto to sav amount 500.00 PLN, fulfils: save[ka]) "o1"
  bank[o2] = (on: 2026-03-13, kind: transfers, leg cash from konto to world amount 30.00 PLN, fulfils: rent[2026-03]) "o2"
  stated[konto] = 550.00 PLN after rent[ka] "saldo"

ask b: bal(konto, 2026-03-31)
ask md: bal(konto, 2026-03-14) modelled, bal(konto, 2026-03-31) modelled
ask g: last(stated[konto], 2026-05-31), since(stated[konto], 2026-05-31)
"#;
        holds(
            &lines_of("b6_wrongaxis.tally", text),
            &[
                "refused: LOAD-KEY stated[konto] after rent[ka]: ka is not a key of months (b6_wrongaxis.tally:49)",
            ],
        );
    }

    /// N-b (`b6_month_out.tally`): `after rent[2027-02]`, a month outside rent's index: `LOAD-KEY` before `LOAD-ORDER` (E §3.1).
    #[test]
    fn a_reading_after_a_plan_at_a_month_outside_its_index_is_load_key() {
        let text = r#"model t
  tally 2
  on 2026-06-10
  start 2026-03-01
  end 2026-06-30

source s "s"
unit PLN: currency minor 0.01
place konto: accounts PLN lives 2026-01-01..
place sav: accounts PLN lives 2026-01-01..
place world: outside lives 1900-01-01..
input bank: observed
input keep: money PLN
input d_save: int
input d_sday: date
input d_rent: int
input rentamt: money PLN

index ks = [ka]
move save[k in ks]: transfers
  on d_sday
  reads konto
  leg cash from konto to sav amount max(0 PLN, bal(konto) - keep)

move rent[m in months]: transfers
  on day_of(m, d_rent)
  leg cash from konto to world amount rentamt

version v
  cite s
  governs always
  known 2026-01-01
  keep = 100.00 PLN "s"
  d_save = 15 "s"
  d_sday = 2026-03-15 "s"
  d_rent = 13 "s"
  rentamt = 30.00 PLN "s"
  opening[konto] = 1050.00 PLN at 2026-03-01 start "s"
  opening[sav] = 0.00 PLN at 2026-03-01 start "s"

version st
  cite s
  governs 2026-03
  known 2026-04-02
  covers konto
  anchor konto
  bank[o1] = (on: 2026-03-12, kind: transfers, leg cash from konto to sav amount 500.00 PLN, fulfils: save[ka]) "o1"
  bank[o2] = (on: 2026-03-13, kind: transfers, leg cash from konto to world amount 30.00 PLN, fulfils: rent[2026-03]) "o2"
  stated[konto] = 550.00 PLN after rent[2027-02] "saldo"

ask b: bal(konto, 2026-03-31)
ask md: bal(konto, 2026-03-14) modelled, bal(konto, 2026-03-31) modelled
ask g: last(stated[konto], 2026-05-31), since(stated[konto], 2026-05-31)
"#;
        holds(
            &lines_of("b6_month_out.tally", text),
            &[
                "refused: LOAD-KEY stated[konto] after rent[2027-02]: 2027-02 is not a key of months (b6_month_out.tally:49)",
            ],
        );
    }

    /// N-f (`k9k10.tally`): two decisions' windows ending on one day, `save[k9]` and `save[k10]` of `ks = [k9, k10]`: their ends list k9 first, keys in index order, never as text (E §4.1); refcore's lines, which list them as text.
    #[test]
    fn window_ends_at_one_point_apply_in_index_order() {
        let text = r#"model t
  tally 2
  on 2026-06-10
  start 2026-03-01
  end 2026-06-30

source s "s"
unit PLN: currency minor 0.01
index ks = [k9, k10]
place konto: accounts PLN lives 2026-01-01..
place sav: accounts PLN lives 2026-01-01..
place world: outside lives 1900-01-01..
input d_save: date
input amt: money PLN

move save[k in ks]: transfers
  on d_save
  reads konto
  leg cash from konto to sav amount min(amt, bal(konto))

version v
  cite s
  governs always
  known 2026-01-01
  amt = 10.00 PLN "s"
  d_save = window(2026-03-16, 2026-03-25) "s"
  opening[konto] = 1000.00 PLN at 2026-03-01 start "s"
  opening[sav] = 0.00 PLN at 2026-03-01 start "s"

version r
  cite s
  governs 2026-03
  known 2026-04-02
  covers konto, sav
  anchor konto
  anchor sav
  stated[konto] = 700.00 PLN after save[k9] "after k9"
  stated[sav] = 25.00 PLN after save[k10] "after k10"

ask b: bal(konto, 2026-03-25), bal(konto, 2026-03-31), bal(sav, 2026-03-31)
ask e: explain bal(konto, 2026-03-31)
ask l: last(stated[konto], 2026-03-31)
"#;
        all_of(
            &lines_of("k9k10.tally", text),
            &[
                "b: (?\"date within window(2026-03-16, 2026-03-25) not precise enough for the day of save[k10]\" (save[k10]) ?\"date within window(2026-03-16, 2026-03-25) not precise enough for the day of save[k9]\" (save[k9]), ?\"date within window(2026-03-16, 2026-03-25) not precise enough for the day of save[k10]\" (save[k10]) ?\"date within window(2026-03-16, 2026-03-25) not precise enough for the day of save[k9]\" (save[k9]), ?\"date within window(2026-03-16, 2026-03-25) not precise enough for the day of save[k10]\" (save[k10]) ?\"date within window(2026-03-16, 2026-03-25) not precise enough for the day of save[k9]\" (save[k9]))",
                "  cut 2026-06-10",
                "e: ?\"date within window(2026-03-16, 2026-03-25) not precise enough for the day of save[k10]\" (save[k10]) ?\"date within window(2026-03-16, 2026-03-25) not precise enough for the day of save[k9]\" (save[k9])",
                "  balance of konto (PLN) at the end of 2026-03-31",
                "  from: opening v:opening[konto] = 1 000.00 PLN, 2026-03-01 start (version v, source s, locator \"s\")",
                "  2026-03-16:",
                "    save[k9]: hole: ?\"date within window(2026-03-16, 2026-03-25) not precise enough for the day of save[k9]\" (save[k9])",
                "    save[k10]: hole: ?\"date within window(2026-03-16, 2026-03-25) not precise enough for the day of save[k10]\" (save[k10])",
                "  2026-03-25:",
                "    save[k9]: hole: ?\"date within window(2026-03-16, 2026-03-25) not precise enough for the day of save[k9]\" (save[k9])",
                "    save[k10]: hole: ?\"date within window(2026-03-16, 2026-03-25) not precise enough for the day of save[k10]\" (save[k10])",
                "  cut 2026-06-10",
                "l: 700.00 PLN",
                "  held: stated[konto] = 700.00 PLN (r), read for 2026-03-31",
                "  cut 2026-06-10",
                "warning: WARN-UNREAD place world is read by nothing",
            ],
        );
    }
}
