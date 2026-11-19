//! The builder's tests of group 3's review: each review input, the lines refcore prints for the asks it names.
//! Spec: L §5.18 (R15: `partial` follows what is linear), L §4.4 (a decision's window ends at
//! its last day, pick P29 on #175), L §8 (refusals outrank holes).
//! Twin: engine/tools/refcore (spec/2.1.8 at 75f27ca1, the sixth turn; B5 and N-f restated at
//! spec/2.2 300cda08, tally 2.2's S-3; R14's B4, N1 and G1 dropped with it, tally 2.2's S-1);
//! each expected line is its run of the same text
//! (`python3 -m refcore run`), each text an input of the review of build group 3
//! (`tmp/spec-2.1.8-build/review-g3/`).
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

    /// B5 (`t12.tally`): `flows` over spans after a window-day decision's window has ended: nothing moves there. Its reading stood `after save`, cut in tally 2.2 (S-3): it is stated at the month's end, refcore's lines at spec/2.2 300cda08.
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
  stated[konto] = 90.00 PLN at 2026-03-31 end "saldo"

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

    /// N-f (`k9k10.tally`): two decisions' windows ending on one day, `save[k9]` and `save[k10]` of `ks = [k9, k10]`: their ends list k9 first, keys in index order, never as text (E §4.1). Its readings stood `after save[k9]` and `after save[k10]`, cut in tally 2.2 (S-3): the window ends are read through `explain`, refcore's lines at spec/2.2 300cda08.
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

ask b: bal(konto, 2026-03-25), bal(konto, 2026-03-31), bal(sav, 2026-03-31)
ask e: explain bal(konto, 2026-03-31)
"#;
        all_of(
            &lines_of("k9k10.tally", text),
            &[
                "b: (?\"date within window(2026-03-16, 2026-03-25) not precise enough for the day of save[k10]\" (save[k10]) ?\"date within window(2026-03-16, 2026-03-25) not precise enough for the day of save[k9]\" (save[k9]), ?\"date within window(2026-03-16, 2026-03-25) not precise enough for the day of save[k10]\" (save[k10]) ?\"date within window(2026-03-16, 2026-03-25) not precise enough for the day of save[k9]\" (save[k9]), ?\"date within window(2026-03-16, 2026-03-25) not precise enough for the day of save[k10]\" (save[k10]) ?\"date within window(2026-03-16, 2026-03-25) not precise enough for the day of save[k9]\" (save[k9]))",
                "  unknown day: d_save",
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
                "  unknown day: d_save",
                "  cut 2026-06-10",
                "warning: WARN-UNREAD place world is read by nothing",
            ],
        );
    }
}
