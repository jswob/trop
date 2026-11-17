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
}
