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

    /// The M0 gate's B3 and B4 (G11): `same` sides of every `sref` form stopped "not built",
    /// and a fraction past 64 bits was `ENGINE-INVARIANT`. The expected text and hash are
    /// refcore's `reprint` of the same text. A signed key is no `sref` key since tally 2.1.6
    /// (#88, `format/same_key_signed`), so `fee[-1]` left the text.
    #[test]
    fn same_sides_of_every_form_and_fractions_of_any_size_reprint_as_refcore() {
        let written = "model home
  tally 2
  on 2026-03-01
  start 2026-01-01
  end 2026-12-31
  same fee[a2].x = fee[a1]
  same fee[a1].x[a2] = fee[a2]

source card \"a card\"

input r: num

input s: num

input fee[f in flat]: money PLN

unit PLN: currency minor 0.01

index flat = [a1, a2]

version v1
  cite card
  governs always
  known 2026-01-01
  same fee[a2].x = fee[a1]
  same fee[a1].x[a2] = fee[a2]
  r = 100000000000000000000/30
  same fee.y = fee[a2]
  s = -200000000000000000000/40000000000000000000000
  same fee[a1].x = fee[a2]
  same z.b = fee[a2]
";
        let canonical = "model home
  tally 2
  on 2026-03-01
  start 2026-01-01
  end 2026-12-31
  same fee[a1].x[a2] = fee[a2]
  same fee[a2].x = fee[a1]

source card \"a card\"

unit PLN: currency minor 0.01

index flat = [a1, a2]

input fee[f in flat]: money PLN

input r: num

input s: num

version v1
  cite card
  governs always
  known 2026-01-01
  same fee.y = fee[a2]
  same fee[a1].x = fee[a2]
  same fee[a1].x[a2] = fee[a2]
  same fee[a2].x = fee[a1]
  r = 10000000000000000000/3
  s = -1/200
  same z.b = fee[a2]
";
        let printed = format_file(written.as_bytes()).expect("it reprints");
        assert_eq!(printed, canonical, "refcore's reprint");
        let again = format_file(canonical.as_bytes()).expect("it reprints");
        assert_eq!(again, canonical, "the canonical text is a fixed point");
        let files = [("b.tally".to_owned(), canonical.to_owned())];
        assert_eq!(
            printed_hash(&files),
            "058774a2696cd0ba8c629f91ed7c18836002cfcc65a77b03ae480a290e1b0965",
            "refcore's sha256 line"
        );
    }

    /// A `same` side's keys are literals (L §2.3 `sref`): any other key is refused at parse,
    /// where refcore's parser takes an expression and names its parse tree (#88, pick P12).
    /// A side with fields after its first two names is keyed by its written text (L §2.4: "as
    /// an item whose target and keys are its left side's"), where refcore keys it by its
    /// parse node's text, `field(obj=…)`: the two orders differ only against such a side.
    #[test]
    fn a_same_side_s_keys_are_literals_and_a_deeper_side_sorts_by_its_text() {
        let head =
            "part p\n  tally 2\n\nversion v1\n  cite card\n  governs always\n  known 2026-01-01\n";
        let refused = [
            ("same fee[1 + 1] = fee[a1]", "expected \"]\", got +"),
            ("same fee[a1] = fee[f(x)]", "expected \"]\", got ("),
            ("same fee[*] = fee[a1]", "expected a literal, got *"),
            ("same fee[(a1)] = fee[a1]", "expected a literal, got ("),
        ];
        for (line, why) in refused {
            let text = format!("{head}  {line}\n");
            let Err(Stop::Refused(refusal)) = format_file(text.as_bytes()) else {
                panic!("{line}: a LOAD-PARSE refusal");
            };
            assert_eq!(
                refusal.text(),
                format!("LOAD-PARSE cannot parse: {why} (input:8)"),
                "{line}"
            );
        }
        let text = format!(
            "{head}  same z.b.c = y\n  same b.x = y\n  same a.b.c[k2] = y\n  same a.b.c[k1] = y\n"
        );
        let printed = format_file(text.as_bytes()).expect("it reprints");
        assert_eq!(
            printed,
            format!(
                "{head}  same a.b.c[k1] = y\n  same a.b.c[k2] = y\n  same b.x = y\n  same z.b.c = y\n"
            ),
            "by the side's text, then its keys"
        );
    }

    /// The M0 gate's second review, N3 (G12): a `same` side keyed by a place keyed by things,
    /// `lim[konto[ana]]` (L §2.3 `rk`'s last form, one key by L §2.4), was refused at parse;
    /// refcore's `sref` takes it and sorts it as a row's place key. The expected text and hash
    /// are refcore's `reprint` (`tmp/m0/g12/p.py`, `n3-two-rep`). A computed key inside it
    /// stays refused at parse (#88, pick P12).
    #[test]
    fn a_same_side_keyed_by_a_keyed_place_reprints_as_refcore() {
        let head = "model home\n  tally 2\n  on 2026-03-01\n  start 2026-01-01\n  end 2026-12-31\n\nsource card \"a card\"\n\nunit PLN: currency minor 0.01\n\nkind persons\n\nplace konto[p in persons]: accounts PLN lives 2026-01-01..\n\nplace world: outside lives 2026-01-01..\n\n";
        let written = format!(
            "{head}input lim[k in accounts]: money PLN\n\ninput amt: money PLN\n\ndef twice: money PLN = amt + amt\n\nask q: twice\n\nversion v1\n  cite card\n  governs always\n  known 2026-01-01\n  persons + [ana] lives 2026-01-01..\n  persons + [ben] lives 2026-01-01..\n  amt = 1.00 PLN\n  opening[konto[ana]] = 1.00 PLN at 2026-01-01 start\n  opening[konto[ben]] = 2.00 PLN at 2026-01-01 start\n  same lim[konto[ben]] = lim[konto[ana]]\n  same lim[konto[ana]] = lim[konto[ben]]\n"
        );
        let canonical = format!(
            "{head}input amt: money PLN\n\ninput lim[k in accounts]: money PLN\n\ndef twice: money PLN = amt + amt\n\nask q: twice\n\nversion v1\n  cite card\n  governs always\n  known 2026-01-01\n  persons + [ana] lives 2026-01-01..\n  persons + [ben] lives 2026-01-01..\n  amt = 1 PLN\n  same lim[konto[ana]] = lim[konto[ben]]\n  same lim[konto[ben]] = lim[konto[ana]]\n  opening[konto[ana]] = 1 PLN at 2026-01-01 start\n  opening[konto[ben]] = 2 PLN at 2026-01-01 start\n"
        );
        let printed = format_file(written.as_bytes()).expect("it reprints");
        assert_eq!(printed, canonical, "refcore's reprint");
        let files = [("a.tally".to_owned(), canonical.clone())];
        assert_eq!(
            printed_hash(&files),
            "59d7a4bfdbc0f0bf22746e6da4d8a10b3d1c7a5c7395758daefb40aff13e7759",
            "refcore's sha256 line"
        );
        let part =
            "part p\n  tally 2\n\nversion v1\n  cite card\n  governs always\n  known 2026-01-01\n";
        let text = format!("{part}  same fee[konto[1 + 1]] = fee[a1]\n");
        let Err(Stop::Refused(refusal)) = format_file(text.as_bytes()) else {
            panic!("a LOAD-PARSE refusal");
        };
        assert_eq!(
            refusal.text(),
            "LOAD-PARSE cannot parse: expected \"]\", got + (input:8)",
            "a computed key inside a place key"
        );
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
