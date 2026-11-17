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
            &lines_of("t13.tally", text),
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
            &lines_of("t17.tally", text),
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
}
