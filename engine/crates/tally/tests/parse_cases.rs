//! The builder's parser tests: S's forms parse, and refusals print refcore's text.
//! Spec: L §2.3 (the grammar), §5.1 (the header's checks), §8.3 (split every file, then parse).
//! Twin: engine/tools/refcore/parser.py; each expected line is refcore's `run` on the same text.
//! Invariants: reads no test, vector or answer key (R-93); the texts are written here, small.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally::syntax::{AskBody, Expr, ItemForm, Section, Value, parse_reality};
    use tally::{Asks, Given, NotImplemented, Pin, Request, Source, parse, run};

    /// The smallest model header: lines 1 to 5.
    const HEADER: &str =
        "model m\n  tally 2\n  on 2026-01-01\n  start 2026-01-01\n  end 2026-12-31\n";

    /// A reality in the kernel corpus's forms; refcore answers both its asks.
    const FULL: &str = "model full
  tally 2
  on 2025-02-17
  start 2025-01-01
  end 2025-03-31

source card \"a card\"

unit DUK: currency minor 0.01

kind ext: place DUK signed outside

place out_0: ext lives 2025-01-01..

place acc_0: accounts DUK lives 2025-01-01..

place acc_1: accounts DUK lives 2025-01-01..

input amt_mv_0: money DUK question \"How much does mv_0 move?\"

input day_mv_0: date question \"When does mv_0 move?\"

input reach: date question \"How far does it look?\"

def low_room: money DUK
  rule base = min(bal(acc_0, d) from day_mv_0 for d in day_mv_0 .. reach step 1 day)

move mv_0: transfers
  on day_mv_0
  reads acc_0
  leg cash from acc_0 to acc_1 amount max(0, low_room - amt_mv_0)

version v1
  cite card
  governs always
  known 2025-01-01
  opening[acc_0] = 100.00 at 2025-01-01 start
  amt_mv_0 = 10.00
  day_mv_0 = 2025-02-01
  reach = 2025-03-01

version v2
  cite card
  governs always
  known 2025-02-11
  replace amt_mv_0 = 12.50

ask bals: bal(acc_0, 2025-03-31), bal(acc_1, 2025-03-31)

ask early: flows(out_0, 2025-01-01 .. 2025-03-31), bal(acc_0, 2025-03-31) as known 2025-01-20
";

    fn every() -> Request {
        Request {
            asks: Asks::Every,
            pin: Pin::Canonical,
        }
    }

    /// The run of one file `a.tally` holding `text`.
    fn run_one(text: &str) -> Result<Vec<String>, NotImplemented> {
        let given = vec![Given {
            path: "a.tally".to_owned(),
            bytes: Some(text.as_bytes().to_vec()),
        }];
        run(given, &every()).map(|answer| answer.lines().to_vec())
    }

    /// A section's kind, by name.
    fn name(section: &Section) -> &'static str {
        match section {
            Section::Source(_) => "source",
            Section::Version(_) => "version",
            Section::Unit(_) => "unit",
            Section::Kind(_) => "kind",
            Section::Index(_) => "index",
            Section::Place(_) => "place",
            Section::Input(_) => "input",
            Section::Band(_) => "band",
            Section::Def(_) => "def",
            Section::Move(_) => "move",
            Section::Ask(_) => "ask",
        }
    }

    #[test]
    fn the_forms_of_milestone_s_parse() {
        let source = Source {
            path: "full.tally".to_owned(),
            text: FULL.to_owned(),
        };
        let files = parse_reality(&[source]).expect("parses");
        let [file] = files.as_slice() else {
            panic!("one file");
        };
        let names: Vec<&str> = file.sections.iter().map(name).collect();
        assert_eq!(
            names,
            [
                "source", "unit", "kind", "place", "place", "place", "input", "input", "input",
                "def", "move", "version", "version", "ask", "ask"
            ],
            "every section, in order"
        );
        let Some(Section::Version(v1)) = file.sections.get(11) else {
            panic!("v1");
        };
        assert_eq!(v1.items.len(), 4, "v1's rows");
        let Some(ItemForm::Row(opening)) = v1.items.first().map(|item| &item.form) else {
            panic!("a row");
        };
        assert_eq!(opening.target, "opening", "the row's target");
        assert!(opening.instant.is_some(), "at 2025-01-01 start");
        let Value::Lit(_) = opening.value else {
            panic!("100.00 is a literal");
        };
        let Some(Section::Ask(early)) = file.sections.get(14) else {
            panic!("early");
        };
        assert!(early.known.is_some(), "as known");
        let AskBody::Exprs(exprs) = &early.body else {
            panic!("values");
        };
        let [Expr::Flows(_), Expr::Bal(_)] = exprs.as_slice() else {
            panic!("flows, then bal");
        };
    }

    #[test]
    fn a_value_reads_its_by_before_its_of() {
        let text = format!(
            "{HEADER}def worth: money PLN = value(acc, d) in PLN by last(px) + bal(cash, d)\n"
        );
        let files = parse(text.as_bytes()).expect("parses");
        let Some(Section::Def(def)) = files.first().and_then(|file| file.sections.first()) else {
            panic!("a def");
        };
        assert_eq!(def.rules.len(), 1, "a short def's one rule");
    }

    #[test]
    fn refusals_print_refcore_text() {
        let cases = [
            (
                "model m\n  on 2026-01-01\n".to_owned(),
                "refused: LOAD-HEADER missing tally version (a.tally:1)",
            ),
            (
                format!("{HEADER}source s \"t\" x\n"),
                "refused: LOAD-PARSE cannot parse: expected end of line, got x (a.tally:6)",
            ),
            (
                String::new(),
                "refused: LOAD-HEADER file has no header (a.tally)",
            ),
            (
                format!("{HEADER}foo x\n"),
                "refused: LOAD-PARSE cannot parse: expected a section, got foo (a.tally:6)",
            ),
            (
                format!("{HEADER}version v\n  cite s\n  x = 1\n"),
                "refused: LOAD-KNOWN version v has no known date (a.tally:6)",
            ),
            (
                format!("{HEADER}source if \"t\"\n"),
                "refused: LOAD-RESERVED if is a keyword (a.tally:6)",
            ),
            (
                format!("{HEADER}kind k: place PLN\n"),
                "refused: LOAD-PLACEKIND place kind k has no unit or sign rule (a.tally:6)",
            ),
        ];
        for (text, line) in cases {
            assert_eq!(run_one(&text), Ok(vec![line.to_owned()]), "{text}");
        }
    }

    #[test]
    fn a_construct_built_later_is_not_implemented() {
        let text = format!("{HEADER}ask x: explain y\n");
        assert_eq!(
            run_one(&text),
            Err(NotImplemented),
            "explain is built later"
        );
        assert_eq!(run_one(HEADER), Err(NotImplemented), "the checks are next");
    }
}
