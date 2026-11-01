//! The builder's load tests: a small reality loads to refcore's indexes, things, rows and observations, and load's refusals print refcore's text.
//! Spec: E §3.1 steps 1–4; L §4.1 (indexes), §4.6 (rows and their stable ids), §5.3
//! (versions), §5.7 (kinds, things, lifespans), §5.10 (places), §5.19 (observations), §8.3,
//! §8.5 (WARN-UNREAD for a source).
//! Twin: engine/tools/refcore/loader.py; each expected line is refcore's load of the same
//! text, printed in the same normalized form.
//! Invariants: reads no test, vector or answer key (R-93); the text is written here, small.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally::Source;
    use tally::load::load;
    use tally::model::Reality;
    use tally::syntax::{Stop, parse_reality};

    /// A reality with a subkind, list, range and open indexes, a keyed place, a `replace`, an
    /// observation and balance readings.
    const REALITY: &str = "model m
  tally 2
  on 2026-03-01
  start 2026-01-01
  end 2026-12-31

source card \"a card\"

source unused \"never cited\"

unit PLN: currency minor 0.01

kind person

kind adult: person

index month = month 2026-01..2026-03

index colour = [red, green]

index acct: open

place konto: accounts PLN lives 2026-01-01..

place wallet[p in adult]: cash PLN lives 2026-02-01..

place shop: outside lives 2026-01-01..

input pay[p in adult, m in month]: money PLN

input code[a in acct]: int

input bank: observed

version v1
  cite card
  governs always
  known 2026-01-01
  adult + [ana, bob] lives 2026-01-01..
  colour + [blue]
  pay[ana, 2026-01] = 100.00
  pay[*, 2026-02] = 50.00
  code[x1] = 1
  opening[konto] = 10.00 at 2026-01-01 start

version v2
  cite card
  governs 2026-02
  known 2026-02-10
  covers konto
  anchor konto
  replace pay[ana, 2026-01] = 120.00
  bank[r1] = (on: 2026-02-03, kind: transfers, leg cash from konto to shop amount 5.00 PLN)
  stated[konto] = 5.00 at 2026-02-28 end

ask q1: pay[ana, 2026-01], code[x1]
";

    /// The load of one file `a.tally` holding `text`.
    fn load_one(text: &str) -> Result<Reality, Stop> {
        let files = parse_reality(&[Source {
            path: "a.tally".to_owned(),
            text: text.to_owned(),
        }])?;
        load(files)
    }

    /// The refusal `text` meets, as the run prints it.
    fn refusal(text: &str) -> String {
        match load_one(text) {
            Err(Stop::Refused(r)) => r.text(),
            Err(Stop::NotBuilt(what)) => format!("not built: {what}"),
            Ok(_) => "loaded".to_owned(),
        }
    }

    /// The reality's load facts, one per line, in refcore's order and form (`refx.py`).
    fn facts(m: &Reality) -> Vec<String> {
        let mut out = Vec::new();
        for (id, ix) in m.indexes.entries() {
            let first = ix
                .keys
                .first()
                .map_or_else(|| "-".to_owned(), tally::model::KeyVal::text);
            out.push(format!("index {id} {} {first}", ix.keys.len()));
        }
        for (id, p) in m.places.entries() {
            out.push(format!("place {id} {} {}", p.kind, p.things.len()));
        }
        for (name, t) in m.things.entries() {
            let span = m
                .lifespan(t, m.on)
                .map_or_else(|| "-".to_owned(), |s| s.text());
            let base = t.base.as_deref().unwrap_or("None");
            out.push(format!("thing {name} {base} {span}"));
        }
        for (_, input) in m.inputs.entries() {
            for row in &input.rows {
                out.push(format!("row {}", row.sid));
            }
            for o in &input.observations {
                let legs: Vec<String> = o
                    .legs
                    .iter()
                    .map(|l| format!("'{} {}->{} {}'", l.name, l.from, l.to, l.unit))
                    .collect();
                let day = o
                    .day
                    .map_or_else(|| "None".to_owned(), tally::time::Day::text);
                let kind = o.kind.as_deref().unwrap_or("None");
                let fulfils = if o.fulfils.is_some() { "True" } else { "False" };
                out.push(format!(
                    "obs {} {day} {kind} [{}] {fulfils}",
                    o.sid(),
                    legs.join(", ")
                ));
            }
        }
        for (code, text) in &m.warnings {
            out.push(format!("warning {} {text}", code.as_str()));
        }
        out
    }

    #[test]
    fn a_reality_loads_as_refcore_loads_it() {
        let m = load_one(REALITY).expect("the reality loads");
        let expected = [
            "index days 365 2026-01-01",
            "index months 12 2026-01",
            "index years 1 2026",
            "index bank_account 0 -",
            "index month 3 2026-01",
            "index colour 3 red",
            "index acct 1 x1",
            "place konto accounts 1",
            "place wallet cash 2",
            "place shop outside 1",
            "thing PLN currencies always",
            "thing ana adult 2026-01-01..",
            "thing bob adult 2026-01-01..",
            "thing konto accounts 2026-01-01..",
            "thing wallet[ana] cash 2026-02-01..",
            "thing wallet[bob] cash 2026-02-01..",
            "thing shop outside 2026-01-01..",
            "row v1:opening[konto]",
            "row v2:stated[konto]",
            "row v1:pay[ana, 2026-01]",
            "row v1:pay[*, 2026-02]",
            "row v2:pay[ana, 2026-01]",
            "row v1:code[x1]",
            "obs bank[r1] 2026-02-03 transfers ['cash konto->shop PLN'] False",
            "warning WARN-UNREAD source unused is read by nothing",
        ];
        assert_eq!(facts(&m), expected, "refcore's load facts");
        let adult = m.kind("adult").expect("adult");
        assert_eq!(
            adult.parent.as_deref(),
            Some("person"),
            "a subkind's parent"
        );
        assert_eq!(
            m.kind("person").map(|k| k.children.clone()),
            Some(vec!["adult".to_owned()]),
            "and its children"
        );
        let v2 = m.versions.get("v2").expect("v2");
        assert_eq!(v2.covers.len(), 1, "covers konto");
        assert_eq!(v2.gap_roles.len(), 1, "anchor konto");
    }

    #[test]
    fn load_refusals_print_refcore_text() {
        let cases = [
            (
                "ask q1:",
                "ask a:",
                "LOAD-SHADOW bound variable a has the name of a top-level id (a.tally:31)",
            ),
            (
                "  cite card\n  governs 2026-02",
                "  cite carde\n  governs 2026-02",
                "LOAD-NAME unknown id carde (version v2 cites no source) (a.tally:46)",
            ),
            (
                "  known 2026-02-10",
                "  known 2026-03-10",
                "LOAD-KNOWN version v2 is known after the model's on date (a.tally:46)",
            ),
            (
                "adult + [ana, bob]",
                "adult + [ana, ana]",
                "LOAD-DUP a key entered twice: ana in adult (a.tally:39)",
            ),
            (
                "index month = month 2026-01..2026-03",
                "index month = month 2026-03..2026-01",
                "LOAD-KEY range out of order (a.tally:17)",
            ),
            (
                "amount 5.00 PLN)",
                "amount 5.001 PLN)",
                "LOAD-PRECISION 5.001 is finer than the minor unit (a.tally:53)",
            ),
        ];
        for (old, new, expected) in cases {
            let text = REALITY.replacen(old, new, 1);
            assert_eq!(refusal(&text), expected, "{new}");
        }
    }
}
