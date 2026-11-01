//! The builder's evaluator tests: the defs of a small reality read as refcore reads them, holes kept as holes.
//! Spec: L §5.11 (rules, `beats`, `when`, `missing`, aggregates, `if`, Kleene logic), §5.9
//! (bands), §7.2 (`round` and its type), §3.3 (`periodic`, `factor`), §8.1 (holes spread and
//! never become 0), §8.2 (the register's `missing`), E §3.2 (the memo).
//! Twin: engine/tools/refcore/evaluate.py (`read_def`, `e_agg`, `call_band`, `c_round`,
//! `c_missing`); each expected line is refcore's read of the same text at law 2026-02-01.
//! Invariants: reads no test, vector or answer key (R-93); the text is written here, small.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally::Source;
    use tally::eval::{Ctx, Engine};
    use tally::load::load;
    use tally::model::{Domain, KeyVal, Reality, Value};
    use tally::syntax::{Stop, parse_reality};
    use tally::time::Day;

    /// Rules with and without a law date, a gap, sums with an unknown term and over no keys,
    /// a band with closed and open ends, `round` to a num, an int and money, Kleene logic,
    /// a bare figure beside money, a rate's period and factor, `missing` in a cited rule's
    /// `when`, `beats` with a `when` that is a hole, an `if` on a hole whose branches agree.
    const REALITY: &str = r#"model m
  tally 2
  on 2026-03-01
  start 2026-01-01
  end 2026-12-31

source card "a card"

unit PLN: currency minor 0.01

index month = month 2026-01..2026-03

input pay[m in month]: money PLN

input bonus[m in month]: money PLN

input fee: money PLN

input small: money PLN

input big: money PLN

input limit: money PLN

input x: num

input half: num

input step: num

input price: num PLN

input cent: money PLN

input apr: rate

input term: int

band scale(v: money PLN): money PLN

def gap: money PLN at reader
  rule base = fee + small

def total: money PLN
  rule base = sum(pay[m] for m in month)

def none_sum: money PLN at reader
  rule base = sum(pay[m] for m in month where pay[m] > big)

def none_min: money PLN at reader
  rule base = min(pay[m] for m in month where pay[m] > big)

def holed_sum: money PLN at reader
  rule base = sum(pay[m] + fee for m in month)

def low: money PLN at reader
  rule base = scale(small)

def high: money PLN at reader
  rule base = scale(big)

def up: num at reader
  rule base = round(x, step, half-up)

def even: num at reader
  rule base = round(x, step, half-even)

def whole: int at reader
  rule base = round(half, 1, half-even)

def in_cash: money PLN at reader
  rule base = round(price, cent, half-up)

def all_false: bool at reader
  rule base = small < limit and fee > limit

def any_true: bool at reader
  rule base = small > limit or fee > limit

def still_open: bool at reader
  rule base = small > limit and fee > limit

def floor[m in month]: money PLN at reader
  rule base = max(0, bonus[m] - limit)

def per_month: num at reader
  rule base = periodic(apr, month, nominal 12)

def grown: num at reader
  rule base = factor(apr, term, month, nominal 12)

def asked: int at reader
  rule base = 1
  rule told beats base v v1 = -1 when missing(fee)

def chosen[m in month]: int
  rule base = 1
  rule over beats base = -1 when bonus[m] > 0

def either: money PLN at reader
  rule base = if fee > limit then 0 else 0

version v1
  cite card
  governs always
  known 2026-01-01
  pay[2026-01] = 100.00
  pay[2026-02] = 50.00
  pay[2026-03] = 0.00
  bonus[2026-01] = 100.00
  bonus[2026-02] = 50.00
  small = 100.00
  big = 150.00
  limit = 80.00
  x = 2.345
  half = 2.5
  step = 0.01
  price = 2.345
  cent = 0.01
  apr = 0.12 1/year
  term = 12
  scale: [0, 100] = round(v * 0.1, 0.01, half-up)
  scale: (100, inf) = 10 + round((v - 100) * 0.2, 0.01, half-up)"#;

    /// refcore's reads: `def[keys] = value` or `def ! refusal`, then the register.
    const READS: &str = r#"all_false = false
any_true = true
asked = -1
  + missing|v1|
chosen[2026-01] = -1
chosen[2026-02] = -1
chosen[2026-03] = ?"bonus at 2026-03: no value for 2026-03" (bonus[2026-03])
either = 0.00 PLN
even = 2.34
floor[2026-01] = 20.00 PLN
floor[2026-02] = 0.00 PLN
floor[2026-03] = ?"bonus at 2026-03: no value for 2026-03" (bonus[2026-03])
gap = ?"fee: no value at 2026-02-01" (fee)
grown = 1.126825030131969720661201 (1.12682503013197)
high = 20.00 PLN
holed_sum = ?"fee: no value at 2026-02-01" (fee)
in_cash = 2.35 PLN
low = 10.00 PLN
none_min ! RUN-EMPTY min over no keys
none_sum = 0.00 PLN
per_month = 0.01
still_open = ?"fee: no value at 2026-02-01" (fee)
total = 150.00 PLN
up = 2.35
whole = 2
"#;

    /// The reality, loaded.
    fn reality() -> Reality {
        let files = parse_reality(&[Source {
            path: "r.tally".to_owned(),
            text: REALITY.to_owned(),
        }])
        .expect("the reality parses");
        load(files).expect("the reality loads")
    }

    /// The read of `id` at `keys` in a fresh engine, as refcore's lines print it.
    fn read(m: &Reality, id: &str, keys: &[KeyVal]) -> Vec<String> {
        let def = m.defs.get(id).expect("a def");
        let law = Day::from_ymd(2026, 2, 1).expect("a day");
        let mut engine = Engine::new(m.on);
        let ctx = Ctx {
            law: Some(law),
            cut: m.on,
        };
        let label = if keys.is_empty() {
            id.to_owned()
        } else {
            let texts: Vec<String> = keys.iter().map(KeyVal::text).collect();
            format!("{id}[{}]", texts.join(", "))
        };
        let shown = match engine.read_def(m, def, keys, ctx) {
            Ok(v @ Value::Hole(_)) => format!("= {}", v.text(m).expect("a text")),
            Ok(v) => format!("= {}", v.typed_text(m).expect("a text")),
            Err(Stop::Refused(r)) => format!("! {}", r.text()),
            Err(Stop::NotBuilt(what)) => format!("~ {what}"),
            Err(Stop::Deep) => "~ deep".to_owned(),
        };
        let mut out = vec![format!("{label} {shown}")];
        let mut entries: Vec<String> = engine
            .collected()
            .entries
            .iter()
            .map(|e| format!("  + {}|{}|{}", e.word, e.text, e.figure))
            .collect();
        entries.sort();
        out.extend(entries);
        out
    }

    #[test]
    fn defs_read_as_refcore_reads_them() {
        let m = reality();
        let mut ids = m.defs.names();
        ids.sort();
        let mut got: Vec<String> = Vec::new();
        for id in &ids {
            let def = m.defs.get(id).expect("a def");
            let keysets: Vec<Vec<KeyVal>> = match def.binds.first().map(|b| &b.domain) {
                None => vec![Vec::new()],
                Some(Domain::Index(ix)) => {
                    let index = m.indexes.get(ix).expect("an index");
                    index.keys.iter().map(|k| vec![k.clone()]).collect()
                }
                Some(_) => panic!("a def keyed by an index"),
            };
            for keys in keysets {
                got.extend(read(&m, id, &keys));
            }
        }
        let want: Vec<&str> = READS.lines().collect();
        assert_eq!(got, want, "every read as refcore's");
        let reads = want.iter().filter(|l| !l.starts_with("  + ")).count();
        assert_eq!(reads, 24, "every read");
    }

    #[test]
    fn a_hole_never_becomes_zero() {
        let m = reality();
        for id in ["gap", "holed_sum", "still_open"] {
            let line = read(&m, id, &[]);
            let first = line.first().expect("a line");
            assert!(first.contains("= ?"), "{id} stays a hole: {first}");
        }
    }

    #[test]
    fn a_cell_is_computed_once_and_keeps_what_it_met() {
        let m = reality();
        let law = Day::from_ymd(2026, 2, 1).expect("a day");
        let ctx = Ctx {
            law: Some(law),
            cut: m.on,
        };
        let def = m.defs.get("asked").expect("a def");
        let mut engine = Engine::new(m.on);
        let first = engine.read_def(&m, def, &[], ctx);
        let met = engine.collected();
        let second = engine.read_def(&m, def, &[], ctx);
        assert_eq!(first, second, "the memo gives the same result");
        assert_eq!(engine.collected(), met, "the register is the same set");
        assert_eq!(met.entries.len(), 1, "one missing entry");
    }
}
