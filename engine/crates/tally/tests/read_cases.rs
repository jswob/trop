//! The builder's read-rule tests: reads of a small reality take refcore's rows, `none`s and hole questions.
//! Spec: L §4.6 (candidates, replacement, precedence, no candidate, outside a lifespan,
//! readings), §4.3 (a read with no law date), §5.9 (a band's table, rows replaced), §5.11
//! step 2 (a rule's version in force), §8.2 (`held`).
//! Twin: engine/tools/refcore/evaluate.py (`candidates`, `choose`, `no_candidate`,
//! `rule_in_force`, `band_table`); each expected line is refcore's read of the same text,
//! printed in the same form.
//! Invariants: reads no test, vector or answer key (R-93); the text is written here, small.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally::Source;
    use tally::load::load;
    use tally::model::{BandEnd, BandRow, Domain, KeyVal, Reality};
    use tally::read::{
        Asked, Clocks, Found, band_rows, held, read_input, rule_in_force, time_point,
    };
    use tally::syntax::{Stop, parse_reality};
    use tally::time::Day;

    /// A reality with a subkind and a late member, a replaced row, a `*` row and a narrower
    /// one, a row known before its key was added, open, closed and `always` spans, a
    /// `question`, readings, a replaced rule and a patched band.
    const REALITY: &str = "model m
  tally 2
  on 2026-03-01
  start 2026-01-01
  end 2026-12-31

source card \"a card\"

unit PLN: currency minor 0.01

kind person

kind adult: person

index month = month 2026-01..2026-06

index colour = [red, green]

place konto: accounts PLN lives 2026-01-01..

input pay[p in adult, m in month]: money PLN

input born[p in adult]: date question \"when was this person born?\"

input paint[c in colour]: int

input fee: num

input tariff: num

input code[c in colour]: int

band scale(x: num PLN): num PLN

def tax[m in month]: num at start_of(m)
  rule base v v1 = 1

version v1
  cite card
  governs always
  known 2026-01-01
  adult + [ana, bob] lives 2026-01-01..
  adult + [cid] lives 2026-02-01..
  pay[ana, 2026-01] = 100.00
  pay[*, 2026-02] = 50.00
  pay[ana, 2026-02] = 60.00
  born[ana] = 1990-05-01
  paint[*] = 1
  code[red] = 1
  scale: [0, 100] = x * 0.1
  scale: (100, inf) = 10 + (x - 100) * 0.2
  opening[konto] = 10.00 at 2026-01-01 start

version v2
  cite card
  governs always
  known 2026-02-10
  colour + [blue]
  replace pay[ana, 2026-01] = 120.00
  replace scale: [0, 100] = x * 0.15

version v3
  cite card
  governs 2026-01-01..
  known 2026-01-05
  fee = 5

version v4
  cite card
  governs 2026-01-01..2026-01-31
  known 2026-01-05
  tariff = 3

version v5
  cite card
  governs always
  known 2026-01-05
  tariff = 2

version v6
  cite card
  governs 2026-03-01..
  known 2026-02-20
  replace rule tax.base v v6 = 1 + 1

version v7
  cite card
  governs 2026-02-01..
  known 2026-02-25
  anchor konto
  stated[konto] = 5.00 at 2026-02-28 end

ask q1: pay[ana, 2026-01]
";

    /// The reads and refcore's outcome of each (`refread.py`), one per line: kind, item,
    /// keys, law date (`-` for none), cut, outcome.
    const READS: &str = "\
input|pay|ana, 2026-01|-|2026-03-01|row v2:pay[ana, 2026-01]
input|pay|ana, 2026-01|-|2026-02-01|row v1:pay[ana, 2026-01]
input|pay|bob, 2026-02|-|2026-03-01|row v1:pay[*, 2026-02]
input|pay|ana, 2026-02|-|2026-03-01|row v1:pay[ana, 2026-02]
input|pay|bob, 2026-03|-|2026-03-01|hole pay at bob, 2026-03: no value for 2026-03 | pay[bob, 2026-03]
input|pay|cid, 2026-01|-|2026-03-01|none
input|pay|cid, 2026-03|-|2026-03-01|hole pay at cid, 2026-03: no value for 2026-03 | pay[cid, 2026-03]
input|born|ana|-|2026-03-01|row v1:born[ana]
input|born|bob|-|2026-03-01|hole when was this person born? | born[bob]
input|paint|red|-|2026-03-01|row v1:paint
input|paint|blue|-|2026-03-01|hole paint at blue: no value | paint[blue]
input|fee||2026-06-01|2026-03-01|row v3:fee held
input|fee||2026-02-01|2026-03-01|row v3:fee
input|fee||2025-12-01|2026-03-01|hole fee: no value at 2025-12-01 | fee
input|fee||-|2026-03-01|lawdate
input|fee||2026-02-01|2026-01-02|hole fee: no value at 2026-02-01 | fee
input|tariff||2026-01-15|2026-03-01|row v4:tariff
input|tariff||2026-02-15|2026-03-01|row v5:tariff
input|code|green|-|2026-03-01|hole code at green: no value | code[green]
input|stated|konto|2026-02-28|2026-03-01|row v7:stated[konto]
input|stated|konto|2026-02-27|2026-03-01|hole stated[konto] has no reading at 2026-02-27 | stated[konto]
input|stated|konto|2026-02-28|2026-02-20|hole stated[konto] has no reading at 2026-02-28 | stated[konto]
input|opening|konto|2026-01-01|2026-03-01|row v1:opening[konto]
rule|tax|base|2026-02-15|2026-03-01|rule v v1
rule|tax|base|2026-04-01|2026-03-01|rule v6
rule|tax|base|2026-04-01|2026-02-15|rule v v1
band|scale||2026-02-15|2026-03-01|rows v2 [0, 100]; v1 (100, inf)
band|scale||2026-02-15|2026-02-01|rows v1 [0, 100]; v1 (100, inf)
band|scale||2025-06-01|2026-03-01|rows v2 [0, 100]; v1 (100, inf)
";

    /// The day `text`, `2026-03-01`.
    fn day(text: &str) -> Day {
        let parts: Vec<&str> = text.split('-').collect();
        let [y, m, d] = parts.as_slice() else {
            panic!("a day: {text}");
        };
        Day::from_ymd(
            y.parse().expect("a year"),
            m.parse().expect("a month"),
            d.parse().expect("a day"),
        )
        .expect("a day of the calendar")
    }

    /// The key `text` in the domain `domain`.
    fn key(m: &Reality, domain: &Domain, text: &str) -> KeyVal {
        match domain {
            Domain::Index(id) => m
                .indexes
                .get(id)
                .and_then(|ix| ix.keys.iter().find(|k| k.text() == text))
                .cloned()
                .expect("a key of the index"),
            Domain::Kind(_) => KeyVal::Thing(text.to_owned()),
            Domain::Axis(_) | Domain::List(_) | Domain::Range => panic!("a domain this test reads"),
        }
    }

    /// A band row's interval as refcore prints it, `[0, 100]`, `(100, inf)`.
    fn interval(row: &BandRow) -> String {
        let end = |e: &Option<BandEnd>, inf: &str| match e {
            Some(BandEnd::Num(n)) => n.to_text().expect("an end that prints"),
            None => inf.to_owned(),
        };
        format!(
            "{}{}, {}{}",
            if row.lo_closed { "[" } else { "(" },
            end(&row.lo, "-inf"),
            end(&row.hi, "inf"),
            if row.hi_closed { "]" } else { ")" }
        )
    }

    /// A read's outcome as `refread.py` prints it.
    fn outcome(m: &Reality, kind: &str, id: &str, texts: &[&str], clocks: Clocks) -> String {
        let got = match kind {
            "rule" => read_rule(m, id, texts, clocks),
            "band" => read_band(m, id, clocks),
            _ => read_plain(m, id, texts, clocks),
        };
        match got {
            Ok(line) => line,
            Err(Stop::NotBuilt(what)) if what.contains("LOAD-LAWDATE") => "lawdate".to_owned(),
            Err(Stop::NotBuilt(what)) => format!("not built: {what}"),
            Err(Stop::Deep) => "deep".to_owned(),
            Err(Stop::Refused(r)) => format!("refused {}", r.text()),
        }
    }

    /// A read of a def's rule version.
    fn read_rule(m: &Reality, id: &str, texts: &[&str], clocks: Clocks) -> Result<String, Stop> {
        let def = m.defs.get(id).expect("the def");
        let label = texts.first().expect("a label");
        let (_, versions) = def
            .rules
            .iter()
            .find(|(l, _)| l == label)
            .expect("the label");
        let Some(at) = rule_in_force(m, id, label, versions, clocks.law, clocks.cut)? else {
            return Ok("no rule".to_owned());
        };
        let rule = versions.get(at).expect("the version");
        Ok(match (&rule.version, &rule.v) {
            (Some(version), _) => format!("rule {version}"),
            (None, Some(v)) => format!("rule v {v}"),
            (None, None) => "rule ?".to_owned(),
        })
    }

    /// A read of a band's rows.
    fn read_band(m: &Reality, id: &str, clocks: Clocks) -> Result<String, Stop> {
        let band = m.bands.get(id).expect("the band");
        let Some(rows) = band_rows(m, band, None, clocks)? else {
            return Ok("no table".to_owned());
        };
        let mut parts = Vec::new();
        for at in rows {
            let table = band.tables.get(at.table).expect("the table");
            let row = table.rows.get(at.row).expect("the row");
            parts.push(format!("{} {}", table.version, interval(row)));
        }
        Ok(format!("rows {}", parts.join("; ")))
    }

    /// A read of an input.
    fn read_plain(m: &Reality, id: &str, texts: &[&str], clocks: Clocks) -> Result<String, Stop> {
        let input = m.inputs.get(id).expect("the input");
        let keys: Vec<KeyVal> = input
            .binds
            .iter()
            .zip(texts)
            .map(|(b, t)| key(m, &b.domain, t))
            .collect();
        Ok(match read_input(m, input, &keys, clocks, Asked::Plain)? {
            Found::Row(at) => {
                let row = input.rows.get(at).expect("the row");
                let point = time_point(input, &keys, clocks.law)?;
                let mark = if held(m, row, point.day, clocks)? {
                    " held"
                } else {
                    ""
                };
                format!("row {}{mark}", row.sid)
            }
            Found::None => "none".to_owned(),
            Found::Hole(q) => format!("hole {} | {}", q.text, q.origin),
        })
    }

    #[test]
    fn reads_take_refcore_rows_nones_and_questions() {
        let files = parse_reality(&[Source {
            path: "r.tally".to_owned(),
            text: REALITY.to_owned(),
        }])
        .expect("the reality parses");
        let m = load(files).expect("the reality loads");
        let mut count = 0;
        for line in READS.lines() {
            let fields: Vec<&str> = line.splitn(6, '|').collect();
            let [kind, id, keys, law, cut, expected] = fields.as_slice() else {
                panic!("a read line: {line}");
            };
            let texts: Vec<&str> = keys.split(", ").filter(|t| !t.is_empty()).collect();
            let clocks = Clocks {
                law: (*law != "-").then(|| day(law)),
                cut: day(cut),
                ask_cut: m.on,
            };
            let got = outcome(&m, kind, id, &texts, clocks);
            assert_eq!(got, *expected, "{line}");
            count += 1;
        }
        assert_eq!(count, 29, "every read");
    }

    /// The reality, loaded.
    fn reality() -> Reality {
        let files = parse_reality(&[Source {
            path: "r.tally".to_owned(),
            text: REALITY.to_owned(),
        }])
        .expect("the reality parses");
        load(files).expect("the reality loads")
    }

    /// Clocks at law date `law` and the model's cut.
    fn at(m: &Reality, law: &str) -> Clocks {
        Clocks {
            law: Some(day(law)),
            cut: m.on,
            ask_cut: m.on,
        }
    }

    /// The known date of version `id`.
    fn known(m: &Reality, id: &str) -> Day {
        m.versions.get(id).expect("the version").known
    }

    /// Tax's versions of rule `base`.
    fn base_rules(m: &mut Reality) -> &mut Vec<tally::model::RuleVer> {
        let def = m.defs.get_mut("tax").expect("tax");
        let (_, rules) = def.rules.first_mut().expect("rule base");
        rules
    }

    // The tie refusals: each is read on the reality changed as load (M0's checks) would
    // refuse it, so the read meets it; refcore's text from the same change (`refread.py`'s
    // variants).

    #[test]
    fn rows_still_tied_are_refused() {
        // v4's tariff row governs always, as v5's does, known the same day
        let mut m = reality();
        m.versions.get_mut("v4").expect("v4").governs = Some(tally::model::Span::always());
        let got = outcome(&m, "input", "tariff", &[], at(&m, "2026-02-15"));
        let text = "refused LOAD-OVERLAP rows r.tally:72 and r.tally:78 of tariff tie";
        assert_eq!(got, text, "refcore's tie");
    }

    #[test]
    fn replace_rules_known_the_same_day_are_refused() {
        // a second replace version of tax.base, from v7, known the day v6 is
        let mut m = reality();
        let v6 = known(&m, "v6");
        m.versions.get_mut("v7").expect("v7").known = v6;
        let rules = base_rules(&mut m);
        let mut copy = rules.last().expect("v6's").clone();
        copy.version = Some("v7".to_owned());
        rules.push(copy);
        let got = outcome(&m, "rule", "tax", &["base"], at(&m, "2026-04-01"));
        let text =
            "refused LOAD-OVERLAP rules tax.base of v6 and v7 overlap, neither replacing the other";
        assert_eq!(got, text, "refcore's overlap");
    }

    #[test]
    fn two_plain_versions_of_a_rule_are_refused() {
        // a second plain version of tax.base, beside v1's
        let mut m = reality();
        let rules = base_rules(&mut m);
        let copy = rules.first().expect("v1's").clone();
        rules.push(copy);
        let got = outcome(&m, "rule", "tax", &["base"], at(&m, "2026-02-15"));
        let text = "refused LOAD-LABEL rule base already exists in tax";
        assert_eq!(got, text, "refcore's label");
    }

    #[test]
    fn band_tables_known_the_same_day_are_refused() {
        // v2's band table known the day v1's is
        let mut m = reality();
        let v1 = known(&m, "v1");
        m.versions.get_mut("v2").expect("v2").known = v1;
        let got = outcome(&m, "band", "scale", &[], at(&m, "2026-02-15"));
        let text = "refused LOAD-OVERLAP band scale: two tables known the same day";
        assert_eq!(got, text, "refcore's same-day tables");
    }

    #[test]
    fn band_tables_overlapping_without_replace_are_refused() {
        // v2's band table without replace
        let mut m = reality();
        let band = m.bands.get_mut("scale").expect("scale");
        for table in band.tables.iter_mut().filter(|t| t.version == "v2") {
            for row in &mut table.rows {
                row.replace = false;
            }
        }
        let got = outcome(&m, "band", "scale", &[], at(&m, "2026-02-15"));
        let text = "refused LOAD-OVERLAP band scale: tables overlap without replace";
        assert_eq!(got, text, "refcore's plain tables");
    }
}
