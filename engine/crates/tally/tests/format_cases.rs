//! The builder's formatter tests: a small reality reprints as refcore reprints it, and its
//! canonical text reprints as itself.
//! Spec: L §2.4 (the canonical format); E §3.1 step 1 (the hash).
//! Twin: engine/tools/refcore/canon.py; each expected text is refcore's `reprint` of the same
//! text.
//! Invariants: reads no test, vector or answer key (R-93); the texts are written here, small.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally::syntax::{Stop, printed_hash};
    use tally::{Given, format_file, reprint};

    /// Written off its canonical form: sections out of order, extra parentheses, an unsorted
    /// `beats`, rules and version items out of order, numbers and a fraction not reduced.
    const WRITTEN: &str = "model m
  tally 2
  on 2026-01-01
  start 2026-01-01
  end 2026-12-31

input z: num

version v
  cite s
  governs always
  known 2026-01-01
  z = 3/6
  x = 1000.50

input x: num

source s \"s\"

def b: num = ((x + z)) * (x) + (x * z)

def a: num
  rule base = x - (z - 1000.50)

def c: bool
  rule q beats r, p = (x > z) or (z > x)
  rule p = x>z
  rule r = true
";

    /// refcore's reprint of [`WRITTEN`].
    const CANONICAL: &str = "model m
  tally 2
  on 2026-01-01
  start 2026-01-01
  end 2026-12-31

source s \"s\"

input x: num

input z: num

def a: num = x - (z - 1 000.5)

def b: num = (x + z) * x + x * z

def c: bool
  rule p = x > z
  rule q beats p, r = x > z or z > x
  rule r = true

version v
  cite s
  governs always
  known 2026-01-01
  x = 1 000.5
  z = 1/2
";

    #[test]
    fn a_reality_reprints_as_refcore_and_again_as_itself() {
        let canonical = format_file(WRITTEN.as_bytes()).expect("it reprints");
        assert_eq!(canonical, CANONICAL, "refcore's reprint");
        let again = format_file(CANONICAL.as_bytes()).expect("it reprints");
        assert_eq!(again, CANONICAL, "the canonical text is a fixed point");
        let printed = [("m.tally".to_owned(), CANONICAL.to_owned())];
        assert_eq!(
            printed_hash(&printed),
            "5f77af543e6f528885de7eed0709f1f613afebf24486750ab0cf8ac1e17c8cff",
            "refcore's sha256 line"
        );
    }

    /// The `format` fuzz target's finding (the rules step, 2026-11-04): a row governing from a
    /// key, ordered against a row governing from a year. refcore's reprint can't compare the
    /// two (REFCORE-NOT-YET); its loader refuses the key as no date or period.
    #[test]
    fn a_span_start_that_is_no_date_is_refused_as_load_refuses_it() {
        let text = "part f0\n  tally 2\n\nversion v\n  cite s\n  known 2027-01-20\n  x[c] = 0 governs 2026\n  x[c] = 1 governs ab\n";
        let Err(Stop::Refused(refusal)) = format_file(text.as_bytes()) else {
            panic!("a LOAD-* refusal");
        };
        assert_eq!(
            refusal.text(),
            "LOAD-PARSE cannot parse: expected a date or a period, got ab (input:8)",
            "load's refusal of the row, at its line"
        );
        let one = "part f0\n  tally 2\n\nversion v\n  cite s\n  known 2027-01-20\n  x[c] = 1 governs ab\n";
        assert!(
            format_file(one.as_bytes()).is_ok(),
            "alone, the row needs no order and reprints, as in refcore"
        );
    }

    /// The `format` fuzz target's finding at G5 (#80, tally 2.1.4): a prefixed call printed
    /// `year_.f (k)`, which no parser reads back. Each call's `(` touches its name, a band's
    /// key and a member's or the household's prefix included (L §2.4 Expressions).
    #[test]
    fn a_prefixed_call_prints_touching_its_paren_as_refcore_2_1_4() {
        let written = "model g
  tally 2
  on 2026-06-01
  start 2026-01-01
  end 2026-12-31

def e: num = n[q1]+year_.f(2)

index qs = [q1, q2]

def c: num = sum(n[k] for k in qs where (year_.f(k)) == 1)

input n[k in qs]: num

def g: num = a.b[q1](2) * (household.b(n[q2]))
";
        let canonical = "model g
  tally 2
  on 2026-06-01
  start 2026-01-01
  end 2026-12-31

index qs = [q1, q2]

input n[k in qs]: num

def c: num = sum(n[k] for k in qs where year_.f(k) == 1)

def e: num = n[q1] + year_.f(2)

def g: num = a.b[q1](2) * household.b(n[q2])
";
        let printed = format_file(written.as_bytes()).expect("it reprints");
        assert_eq!(printed, canonical, "refcore 2.1.4's reprint");
        let again = format_file(canonical.as_bytes()).expect("it reprints");
        assert_eq!(again, canonical, "the canonical text is a fixed point");
    }

    /// The `format` fuzz target's #84 finding: a prefixed call whose name is a keyword word
    /// (`mi.or(DUK)`, a field after `.` is never a keyword, L §2.3 rule 3) printed `mi.or (DUK)`,
    /// which no parser reads back. It prints touching its `(`, as every call (L §2.4
    /// Expressions, 2.1.4-1). The expected text is written from L §2.4 (the engine goes first,
    /// pick P8; refcore follows in tally 2.1.5).
    #[test]
    fn a_keyword_named_prefixed_call_prints_touching_its_paren() {
        let written = "model m
  tally 2
  on 2026-03-01
  start 2026-01-01
  end 2026-12-31

def other: money DUK = round(gross,mi.and(DUK),half-up)

unit DUK: currency minor 0.01

input gross: money DUK

def contrib: money DUK = round(gross, mi.or(DUK), half-up)

def inner: money DUK = gross+mi.in(gross)
";
        let canonical = "model m
  tally 2
  on 2026-03-01
  start 2026-01-01
  end 2026-12-31

unit DUK: currency minor 0.01

input gross: money DUK

def contrib: money DUK = round(gross, mi.or(DUK), half-up)

def inner: money DUK = gross + mi.in(gross)

def other: money DUK = round(gross, mi.and(DUK), half-up)
";
        let printed = format_file(written.as_bytes()).expect("it reprints");
        assert_eq!(printed, canonical, "L §2.4's form");
        let again = format_file(canonical.as_bytes()).expect("it reprints");
        assert_eq!(again, canonical, "the canonical text is a fixed point");
    }

    #[test]
    fn reprint_names_files_as_given_and_orders_them_by_full_path() {
        let file = |path: &str, text: &str| Given {
            path: path.to_owned(),
            bytes: Some(text.as_bytes().to_vec()),
        };
        let header = "model m\n  tally 2\n  on 2026-01-01\n  start 2026-01-01\n  end 2026-12-31\n";
        let part = "part p\n  tally 2\n";
        let answer = reprint(vec![
            (file("d/z.tally", part), "z.tally".to_owned()),
            (file("d/a/m.tally", header), "a/m.tally".to_owned()),
        ])
        .expect("both reprint");
        let heads: Vec<&str> = answer
            .lines()
            .iter()
            .filter(|line| line.starts_with("--- "))
            .map(String::as_str)
            .collect();
        assert_eq!(heads, ["--- a/m.tally", "--- z.tally"], "by full path");
        assert!(
            answer
                .lines()
                .last()
                .is_some_and(|line| line.starts_with("sha256 ")),
            "the hash last"
        );
        let missing = reprint(vec![(
            Given {
                path: "d/x.tally".to_owned(),
                bytes: None,
            },
            "x.tally".to_owned(),
        )])
        .expect("an answer");
        assert_eq!(
            missing.lines(),
            ["refused: LOAD-PATH d/x.tally doesn't exist"],
            "a missing file by its full path"
        );
    }
}
