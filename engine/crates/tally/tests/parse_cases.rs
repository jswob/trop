//! The builder's parser tests: S's forms parse, and refusals print refcore's text.
//! Spec: L §2.3 (the grammar), §5.1 (the header's checks), §8.3 (split every file, then parse).
//! Twin: engine/tools/refcore/parser.py; each expected line is refcore's `run` on the same text.
//! Invariants: reads no test, vector or answer key (R-93); the texts are written here, small.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally::syntax::{Ask, AskBody, Expr, ItemForm, RuleValue, Section, Value, parse_reality};
    use tally::{Asks, Days, Given, NotImplemented, Pin, Request, Source, parse, run};

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
            days: Days::Events,
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
            Section::Axis(_) => "axis",
            Section::Fill(_) => "fill",
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

    /// The first section of `text` under the header, in a word or two: its expression's node,
    /// its ask's form and clauses, its options, parts, moves or `of` rows.
    fn shape(text: &str) -> String {
        let files = parse(format!("{HEADER}{text}\n").as_bytes()).expect(text);
        let Some(section) = files.first().and_then(|file| file.sections.first()) else {
            panic!("a section: {text}");
        };
        if let Section::Def(def) = section {
            let Some(RuleValue::Expr(e)) = def.rules.first().map(|rule| &rule.value) else {
                return "a rule".to_owned();
            };
            return expr_shape(e).to_owned();
        }
        if let Section::Ask(ask) = section {
            return ask_shape(ask);
        }
        if let Section::Axis(axis) = section {
            return format!("axis {}", axis.options.len());
        }
        if let Section::Kind(kind) = section {
            return format!("parts {}", kind.parts.len());
        }
        if let Section::Version(version) = section {
            let of = version
                .items
                .iter()
                .filter(|item| matches!(&item.form, ItemForm::Row(row) if row.of.is_some()));
            return format!("moves {} of {}", version.moves.len(), of.count());
        }
        name(section).to_owned()
    }

    /// An expression's node, by name: the first of these it is, else "other".
    fn expr_shape(e: &Expr) -> &'static str {
        let names = [
            (matches!(e, Expr::Query(_)), "query"),
            (matches!(e, Expr::Low(_)), "low"),
            (matches!(e, Expr::Crps(_)), "crps"),
            (matches!(e, Expr::DistFn(_)), "distfn"),
            (matches!(e, Expr::Dist(_)), "dist"),
            (matches!(e, Expr::Prorata(_)), "prorata"),
            (
                matches!(e, Expr::AsIf(_, pins) if pins.len() == 1),
                "as if, one pin",
            ),
            (
                matches!(e, Expr::Bal(bal) if bal.proj.random.is_some()),
                "bal, random at",
            ),
        ];
        names
            .iter()
            .find(|(is, _)| *is)
            .map_or("other", |(_, name)| name)
    }

    /// An ask's form, and its pins and clauses.
    fn ask_shape(ask: &Ask) -> String {
        let form = match &ask.body {
            AskBody::Exprs(_) => "values",
            AskBody::Residuals(_) => "residuals",
            AskBody::Explain(_) => "explain",
            AskBody::InputsOf(_) => "inputs-of",
            AskBody::ReadersOf(_) => "readers-of",
            AskBody::Check(..) => "check",
            AskBody::Solve(_) => "solve",
            AskBody::Diff(_) => "diff",
        };
        let clauses = [
            (!ask.pins.is_empty(), " pins"),
            (ask.random.is_some(), " random"),
            (ask.paths.is_some(), " paths"),
            (ask.orient.is_some(), " orient"),
        ];
        let mut shape = form.to_owned();
        for (written, word) in clauses {
            if written {
                shape.push_str(word);
            }
        }
        shape
    }

    /// Each form S's parser left out, as M0's parser reads it; refcore parses each of these
    /// lines (its answers are load refusals, the `diff` naming an unloaded world refused
    /// alone; the axis loads and prints nothing, as the engine does since G8). L §2.3. An
    /// ask's `with` pins run since M1's G5, so that form's run isn't checked here: its line
    /// `x`, a bare word, is refcore's `LOAD-KEY ambiguous key x` at run, which the engine
    /// prints as the word with or without the pin (a load gap, G5's BUILD-LOG entry).
    #[test]
    fn the_forms_of_milestone_m0_parse() {
        let cases = [
            ("def a: date = mean(x for i in k)", "query"),
            ("def a: date = quantile(x for i in k, 0.5)", "query"),
            ("def a: date = low(acc, 2026-01-01 .. 2026-02-01)", "low"),
            ("def a: date = crps(x, y)", "crps"),
            ("def a: date = cdf(x, 1)", "distfn"),
            ("def a: date = normal(0, 1)", "dist"),
            ("def a: date = discrete(x for i in k)", "dist"),
            (
                "def a: date = prorata(t, w for i in k, i, 0.01 PLN, rest largest)",
                "prorata",
            ),
            ("def a: date = x as if {law 2026-01-01}", "as if, one pin"),
            (
                "def a: date = bal(acc, d) from 2026-01-01 random at mean",
                "bal, random at",
            ),
            ("axis rates\n  low \"Low\"\n  high \"High\"", "axis 2"),
            ("kind lot: place PLN signed\n  batched", "parts 1"),
            ("ask q: explain x", "explain"),
            ("ask q: check x == y", "check"),
            ("ask q: x with law 2026-01-01", "values pins"),
            ("ask q: x random at quantile(0.5)", "values random"),
            ("ask q: x paths 10 seed 1", "values paths"),
            ("ask q: x orient lower", "values orient"),
            ("ask q: diff(x) in s against b", "diff"),
            ("ask q: solve x == y for z in [0, 10] to 1 PLN", "solve"),
            (
                "version v\n  cite s\n  known 2026-01-01\n  moves m to 2026-01-01..",
                "moves 1 of 0",
            ),
            (
                "version v\n  cite s\n  known 2026-01-01\n  r of acc = 1",
                "moves 0 of 1",
            ),
        ];
        for (text, want) in cases {
            assert_eq!(shape(text), want, "{text}");
            if want == "values pins" {
                continue;
            }
            let ran = run_one(&format!("{HEADER}{text}\n"));
            let stopped = match &ran {
                Err(NotImplemented) => true,
                Ok(lines) => match lines.as_slice() {
                    [] => want == "axis 2",
                    [line] => line.starts_with("refused: ") || line.starts_with("q: refused: "),
                    _ => false,
                },
            };
            assert!(stopped, "{text}: answers a refusal, or nothing: {ran:?}");
        }
    }

    /// `as if` nested five deep, through parentheses or through pins, is refused at its
    /// section's line once the file has parsed; four deep parses (L §2.3, refcore's
    /// `asif_depth`).
    #[test]
    fn as_if_deeper_than_four_is_refused() {
        let law = "{law 2026-01-01}";
        let four =
            format!("def d: date = (((x as if {law}) as if {law}) as if {law}) as if {law}\n");
        let five = format!(
            "def d: date = ((((x as if {law}) as if {law}) as if {law}) as if {law}) as if {law}\n"
        );
        let pins = format!(
            "def d: date = x as if {{law y as if {{law z as if {{law w as if {{law v as if {law}}}}}}}}}\n"
        );
        let refused = "refused: LOAD-ASIF as if deeper than 4 (a.tally:6)".to_owned();
        assert!(
            parse(format!("{HEADER}{four}").as_bytes()).is_ok(),
            "four deep"
        );
        assert_eq!(
            run_one(&format!("{HEADER}{five}")),
            Ok(vec![refused.clone()]),
            "five deep, through parentheses"
        );
        assert_eq!(
            run_one(&format!("{HEADER}{pins}")),
            Ok(vec![refused]),
            "five deep, through pins"
        );
    }

    #[test]
    fn a_construct_built_later_is_not_implemented() {
        let text = format!(
            "{HEADER}source s \"s\"\ninput r: num\nask x: pow(r, r) to 0.01\n\
             version v\n  cite s\n  governs always\n  known 2026-01-01\n  r = 0.5\n"
        );
        assert_eq!(
            run_one(&text),
            Err(NotImplemented),
            "a transcendental value is built later (M6)"
        );
        assert_eq!(run_one(HEADER), Ok(Vec::new()), "no ask, nothing printed");
    }
}
