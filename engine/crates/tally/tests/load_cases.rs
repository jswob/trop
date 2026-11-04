//! The builder's load tests: a small reality loads to refcore's indexes, things, rows and observations, and load's refusals print refcore's text.
//! Spec: E §3.1 steps 1–4; L §4.1 (indexes), §4.6 (rows and their stable ids), §5.3
//! (versions), §5.7 (kinds, things, lifespans), §5.10 (places), §5.19 (observations), §8.3,
//! §8.5 (WARN-UNREAD for a source and a place); §5.10 (priced places, `bal`, `value`), §5.14
//! (moves and legs, `flows`), §5.19 (`observed`, `gap`, `covered`) for the checks of moves,
//! prices and reads; §5.3 (known windows, `moves`), §4.6 (overlap and `replace`, a period
//! across keys), §5.9 (band tables), §5.19 (`covers`) for the checks of versions and rows.
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
            Err(Stop::Deep) => "deep".to_owned(),
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
            "warning WARN-UNREAD place wallet is read by nothing",
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

    /// A reality whose rows, kinds, `prices … by` and bindings G4a checks at load; it loads
    /// clean, as in refcore.
    const TYPED: &str = "model g
  tally 2
  on 2026-03-01
  start 2026-01-01
  end 2026-12-31

source card \"an invented card\"

unit PLN: currency minor 0.01

index qs = [q1, q2]

index ys = year 2025..2026

input amt: money PLN

input day: date

input n: int

input rate[k in qs]: num

input cpi[y in ys]: num

input real: money PLN prices 2026 by cpi

def total: money PLN = sum(round(amt * rate[k], minor(PLN), half-up) for k in qs)

place konto: accounts PLN lives 2026-01-01..

place world: outside lives 1900-01-01..

move pay: transfers
  on day
  leg cash from konto to world amount amt

ask q: total, bal(konto, 2026-12-31)

version card_2026
  cite card
  governs always
  known 2026-01-01
  amt = 100.00 PLN \"the card\"
  day = 2026-02-10 \"the card\"
  n = 2 \"the card\"
  rate[q1] = 0.5 \"the card\"
  rate[q2] = 0 \"the card\"
  cpi[2025] = 1 \"the card\"
  cpi[2026] = 1.1 \"the card\"
  real = 10.00 PLN \"the card\"
  opening[konto] = 500.00 PLN at 2026-01-01 start \"the card\"
";

    #[test]
    fn rows_kinds_prices_and_bindings_are_checked_at_load_as_refcore_checks_them() {
        assert_eq!(refusal(TYPED), "loaded", "the base");
        let ask = "ask q: total, bal(konto, 2026-12-31)";
        let each = "rate[k], minor(PLN), half-up) for k in qs)";
        let kind = "place konto:";
        let cases = [
            (
                "  amt = 100.00 PLN",
                "  amt = 100.005 PLN",
                "LOAD-PRECISION 100.005 is finer than the minor unit (a.tally:43)",
            ),
            (
                "  rate[q1] = 0.5",
                "  rate[q1] = 1/2",
                "LOAD-PRECISION 1/2 terminates; write it as a decimal (a.tally:46)",
            ),
            ("  rate[q1] = 0.5", "  rate[q1] = 1/3", "loaded"),
            (
                "  n = 2",
                "  n = 1.5",
                "LOAD-UNIT 1.5 is not an int (a.tally:45)",
            ),
            (
                "  day = 2026-02-10",
                "  day = 2026-02",
                "LOAD-UNIT a value of the wrong type: a month for date (a.tally:44)",
            ),
            (
                "  opening[konto] = 500.00 PLN at",
                "  opening[konto] = \"x\" at",
                "LOAD-UNIT a value of the wrong type for opening[konto] (a.tally:51)",
            ),
            (
                kind,
                "kind wages: event certain\n  leg cash: PLM\n\nplace konto:",
                "LOAD-UNIT money in an undeclared currency PLM (a.tally:29)",
            ),
            (
                kind,
                "kind wages: event certain\n  leg cash: zloty\n\nplace konto:",
                "LOAD-NAME unknown id zloty (the unit of leg cash of wages) (a.tally:29)",
            ),
            (
                kind,
                "kind wages: event certain\n  leg cash: PLN\n  attr grade: key levels\n\nplace konto:",
                "LOAD-NAME unknown id levels (a.tally:29)",
            ),
            (
                "prices 2026 by cpi",
                "prices 2026 by cpx",
                "LOAD-NAME unknown id cpx (a.tally:25)",
            ),
            (
                "prices 2026 by cpi",
                "prices 2026 by rate",
                "LOAD-KEY prices by rate: not keyed by a time index (a.tally:25)",
            ),
            (
                "prices 2026 by cpi",
                "prices 2027 by cpi",
                "LOAD-KEY prices 2027 by cpi needs an index with a key for the year 2027 (a.tally:25)",
            ),
            (
                ask,
                "ask q: total, sum(sum(rate[k] for k in qs) for k in qs)",
                "LOAD-SHADOW bound variable k has the name of a bound variable in scope (a.tally:37)",
            ),
            (
                each,
                "rate[amt], minor(PLN), half-up) for amt in qs)",
                "LOAD-SHADOW bound variable amt has the name of a top-level id (a.tally:27)",
            ),
            (
                each,
                "rate[q1], minor(PLN), half-up) for q1 in qs)",
                "LOAD-SHADOW bound variable q1 has the spelling of a key of qs (a.tally:27)",
            ),
            (
                ask,
                "ask q: total, count(a in [a, b])",
                "LOAD-SHADOW bound variable a has the spelling of a key of its list (a.tally:37)",
            ),
            (
                ask,
                "ask q: total, count(e in event over 2026-01-01..2026-12-31)",
                "LOAD-NAME unknown id event (a.tally:37)",
            ),
            (
                ask,
                "ask q: total, count(x in amt)",
                "LOAD-KEY amt is not an index, a kind, a key list or a range (a.tally:37)",
            ),
            (
                ask,
                "ask q: total, count(p in accounts)",
                "LOAD-ALIVE binding over accounts needs on or over (a.tally:37)",
            ),
            (
                ask,
                "ask q: total, count(p in accounts on 2026-01-01)",
                "loaded",
            ),
            (
                ask,
                "ask q: total, count(k in qs on 2026-01-01)",
                "LOAD-DOMAIN on on a binding that is not over a kind (a.tally:37)",
            ),
            (
                ask,
                "ask q: total, count(k in qs over 2026-01-01..2026-12-31)",
                "LOAD-DOMAIN over on a binding that is not over a kind (a.tally:37)",
            ),
            (
                "amount amt",
                "amount fees[q1]",
                "LOAD-NAME unknown id fees (a.tally:35)",
            ),
            (
                "  on day\n",
                "  on day\n  when ratex[q1] > 0\n",
                "LOAD-NAME unknown id ratex (a.tally:35)",
            ),
        ];
        for (old, new, expected) in cases {
            assert!(TYPED.contains(old), "{old} is in the base");
            let text = TYPED.replacen(old, new, 1);
            assert_eq!(refusal(&text), expected, "{new}");
        }
    }

    /// A reality with priced places, a quantity place, a move of a kind and a decision that
    /// reads a balance through a projection; defs and an ask on `value`, `flows` and `bal`.
    const MOVES: &str = "model g
  tally 2
  on 2026-03-01
  start 2026-01-01
  end 2026-12-31

source card \"an invented card\"

unit PLN: currency minor 0.01

unit EUR: currency minor 0.01

unit etf_u \"fund units\"

kind stocks: place one min 0

kind homes: place PLN min 0

kind wages: event certain
  leg cash: PLN

index qs = [q1, q2]

input amt: money PLN

input day: date

input y: year

input hv: money PLN

input px: num PLN/etf_u

input fx: num PLN/EUR

input rate[k in qs]: num

place konto: accounts PLN lives 2026-01-01..

place konto_eur: accounts EUR lives 2026-01-01..

place world: outside lives 1900-01-01..

place flat: homes priced hv lives 2026-01-01..

place st: stocks etf_u priced px lives 2026-01-01..

move pay: wages
  on day
  leg cash from world to konto amount amt

move sweep: transfers
  on day
  reads konto
  leg cash from konto to world amount bal(konto, day) from day

def worth: num PLN = value(flat, day) in PLN

def flowed: money PLN = flows(pay, y)

ask q: worth, flowed, bal(konto, 2026-12-31)

version card_2026
  cite card
  governs always
  known 2026-01-01
  amt = 100.00 PLN \"the card\"
  day = 2026-02-10 \"the card\"
  y = 2026 \"the card\"
  hv = 300 000.00 PLN \"the card\"
  opening[konto] = 500.00 PLN at 2026-01-01 start \"the card\"
";

    #[test]
    fn moves_legs_prices_and_reads_are_checked_at_load_as_refcore_checks_them() {
        assert_eq!(refusal(MOVES), "loaded", "the base");
        let priced = "place flat: homes priced hv";
        let leg = "leg cash from world to konto amount amt";
        let sweep = "  reads konto\n  leg cash from konto to world amount bal(konto, day) from day";
        let ask = "ask q: worth, flowed, bal(konto, 2026-12-31)";
        let worth = "def worth: num PLN = value(flat, day) in PLN";
        let flowed = "def flowed: money PLN = flows(pay, y)";
        let pay = "place st: stocks etf_u priced px lives 2026-01-01..\n\nmove pay: wages\n  on day\n  leg cash from world to konto amount amt";
        let cases = [
            (
                priced,
                "place flat: homes priced hz",
                "LOAD-NAME unknown id hz (a.tally:44)",
            ),
            (
                priced,
                "place flat: homes priced konto",
                "LOAD-UNIT place flat is priced by konto, a place, not an input or a def (a.tally:44)",
            ),
            (
                priced,
                "place flat: homes priced day",
                "LOAD-UNIT place flat is priced by day of type date, not num CUR (a value) or num CUR/u (a price per unit) (a.tally:44)",
            ),
            (
                priced,
                "place flat: homes priced rate[q1]",
                "LOAD-UNIT place flat is priced by rate of type num, not num CUR (a value) or num CUR/u (a price per unit) (a.tally:44)",
            ),
            (
                priced,
                "place flat: homes priced last(worth)",
                "LOAD-UNIT place flat is priced by last(worth): last reads an input (L §4.6) (a.tally:44)",
            ),
            (
                priced,
                "place flat: assets PLN lives 2026-01-01..\n\nplace flat2: homes priced hv",
                "LOAD-PRICE asset flat names no price input (a.tally:44)",
            ),
            (priced, "place flat: assets PLN priced hv", "loaded"),
            (
                pay,
                "place st: assets etf_u priced px lives 2026-01-01..\n\nmove pay: wages\n  on day\n  leg cash from world to st amount amt",
                "LOAD-KIND a leg to or from an asset (cash of pay) (a.tally:50)",
            ),
            (
                leg,
                "leg cash from world to amt amount amt",
                "LOAD-UNIT leg cash of pay: to is money PLN, not a place (a.tally:50)",
            ),
            (
                leg,
                "leg cash from konto_eur to konto amount amt",
                "LOAD-UNIT leg cash of pay runs between places of different units (EUR, PLN) (a.tally:50)",
            ),
            (
                leg,
                "leg cash from world to konto_eur amount amt",
                "LOAD-UNIT leg cash of pay is in PLN; its places hold EUR (a.tally:50)",
            ),
            (
                leg,
                "leg cash from world to konto amount day",
                "LOAD-UNIT the amount of leg cash of pay is date, not money PLN (a.tally:50)",
            ),
            (
                "  on day\n  leg cash from world",
                "  on day\n  fulfils konto\n  leg cash from world",
                "LOAD-UNIT fulfils of pay names key accounts, not a movement (a.tally:48)",
            ),
            (
                "  reads konto",
                "  reads amt",
                "LOAD-READS sweep declares reads amt, which is not a place or a move (a.tally:54)",
            ),
            (
                sweep,
                "  reads konto\n  leg cash from konto to world amount day",
                "LOAD-UNIT the amount of leg cash of sweep is date, not money PLN (a.tally:55)",
            ),
            (
                sweep,
                "  reads konto\n  leg cash from konto to world amount bal(konto, day)",
                "LOAD-CYCLE sweep reads bal(konto, day), its own day: its own result or its future (write the projection) (a.tally:52)",
            ),
            (
                sweep,
                "  reads konto\n  leg cash from konto to world amount bal(konto, day start)",
                "loaded",
            ),
            (
                sweep,
                "  reads konto\n  leg cash from konto to world amount bal(konto, day + 1 day)",
                "LOAD-CYCLE sweep reads bal(konto, day + 1 day), a later day: its own result or its future (write the projection) (a.tally:52)",
            ),
            (
                sweep,
                "  reads konto\n  leg cash from konto to world amount bal(konto, day) from amt",
                "LOAD-UNIT from of a projection is money PLN, not a date (a.tally:55)",
            ),
            (
                ask,
                "ask q: worth, flowed, bal(amt, 2026-12-31)",
                "LOAD-UNIT bal of money PLN, which is not a place (a.tally:61)",
            ),
            (
                ask,
                "ask q: worth, flowed, bal(konto, amt)",
                "LOAD-UNIT the day of bal is money PLN, not a date (a.tally:61)",
            ),
            (
                ask,
                "ask q: worth, flowed, bal(konto) modelled",
                "LOAD-PARSE cannot parse: bal(x) with no day takes no clause (a.tally:61)",
            ),
            (
                ask,
                "ask q: worth, flowed, bal(konto, PLN, 2026-12-31) + amt",
                "loaded",
            ),
            (
                ask,
                "ask q: worth, flowed, bal(konto, 2026-12-31) + day",
                "LOAD-UNIT money PLN + date (a.tally:61)",
            ),
            (
                ask,
                "ask q: worth, flowed, bal(konto, 2026-12-31) of konto",
                "LOAD-UNIT of konto is key accounts, not a party, a batch or the pair (a.tally:61)",
            ),
            (
                ask,
                "ask q: worth, flowed, pay.cash of konto",
                "LOAD-FIELD of after pay.cash: a read of parts follows a leg's amount (m.leg.amount of q) (a.tally:61)",
            ),
            (
                ask,
                "ask q: worth, flowed, low(konto, y)",
                "LOAD-DOMAIN low over a span with an open end (a.tally:61)",
            ),
            (
                ask,
                "ask q: worth, flowed, low(konto, 2026-01-01..2026-12-31) + amt",
                "loaded",
            ),
            (
                ask,
                "ask q: worth, flowed, gap(konto, amt)",
                "LOAD-UNIT gap(x, d): d is money PLN, not a date (a.tally:61)",
            ),
            (
                ask,
                "ask q: worth, flowed, gap(konto, day) + day",
                "LOAD-UNIT money PLN + date (a.tally:61)",
            ),
            (
                ask,
                "ask q: worth, flowed, covered(amt, y)",
                "LOAD-UNIT covered of money PLN, which is not a place (a.tally:61)",
            ),
            (
                ask,
                "ask q: worth, flowed, next(konto, day)",
                "LOAD-NAME next takes an event kind: konto (a.tally:61)",
            ),
            (
                ask,
                "ask q: worth, flowed, next(wages, amt)",
                "LOAD-UNIT next(K, d): d is money PLN, not a date (a.tally:61)",
            ),
            (
                ask,
                "ask q: worth, flowed, next(wages, day) + amt",
                "LOAD-UNIT key wages + money PLN (a.tally:61)",
            ),
            (
                ask,
                "ask q: worth, flowed, observed(amt)",
                "LOAD-PARSE cannot parse: observed takes a planned instance m[k] (a.tally:61)",
            ),
            (
                ask,
                "ask q: worth, flowed, observed(pay) + amt",
                "LOAD-UNIT (cash (from key places, to key places, amount money PLN, day date), day date) and money PLN in arithmetic (a.tally:61)",
            ),
            (
                ask,
                "ask q: worth, flowed, observed(pay).cash.amount + amt",
                "loaded",
            ),
            (
                ask,
                "ask q: worth, flowed, unit(world)",
                "LOAD-UNIT unit(world) of a place of unit any: it has no one unit (a.tally:61)",
            ),
            (
                ask,
                "ask q: worth, flowed, unit(st)",
                "LOAD-UNIT unit(st): its unit etf_u is a quantity unit, a key of no unit kind (a.tally:61)",
            ),
            (
                ask,
                "ask q: worth, flowed, unit(day)",
                "LOAD-UNIT unit of date, which is neither a place nor a quantity (a.tally:61)",
            ),
            (
                ask,
                "ask q: worth, flowed, unit(konto) + amt",
                "LOAD-UNIT key currencies + money PLN (a.tally:61)",
            ),
            (
                flowed,
                "def flowed: money PLN = flows(pay.tax, y)",
                "LOAD-LEG flows(pay.tax, span): wages names no leg tax (a.tally:59)",
            ),
            (
                flowed,
                "def flowed: money PLN = flows(pay, y) + day",
                "LOAD-UNIT money PLN + date (a.tally:59)",
            ),
            (
                flowed,
                "def flowed: money PLN = flows(world, y)",
                "LOAD-UNIT flows of world, a place of unit any: it sums no one unit; convert it (in CUR at fx mode) (a.tally:59)",
            ),
            (flowed, "def flowed: money PLN = flows(konto, y)", "loaded"),
            (
                flowed,
                "def flowed: money PLN = flows(pay[q1], y)",
                "LOAD-KEY wrong number of keys for pay (a.tally:59)",
            ),
            (
                flowed,
                "def flowed: money PLN = flows(world, y) in PLN at amt half-up",
                "LOAD-UNIT flows in PLN at amt: the rate is money PLN (a.tally:59)",
            ),
            (
                flowed,
                "def flowed: money PLN = flows(konto_eur, y) in PLN at fx half-up",
                "loaded",
            ),
            (
                worth,
                "def worth: num PLN = value(amt, day) in PLN",
                "LOAD-UNIT value of money PLN, which is not a place (a.tally:57)",
            ),
            (
                worth,
                "def worth: num PLN = value(flat, amt) in PLN",
                "LOAD-UNIT the day of value is money PLN, not a date (a.tally:57)",
            ),
            (
                worth,
                "def worth: num PLN = value(flat, day) in USD",
                "LOAD-UNIT money in an undeclared currency USD (a.tally:57)",
            ),
            (
                worth,
                "def worth: money PLN = value(flat, day) in PLN",
                "LOAD-UNIT rule base of worth gives a num PLN; a money value needs round or prorata (a num CUR stored as money) (a.tally:57)",
            ),
            (
                worth,
                "def worth: num EUR = value(flat, day) in EUR",
                "LOAD-PRICE value(flat) in EUR holds units in PLN and states no by (a.tally:57)",
            ),
            (
                worth,
                "def worth: num PLN = value(st, day) in PLN by amt",
                "LOAD-UNIT value by amt: the price is money PLN, not num PLN/u (a.tally:57)",
            ),
            (
                worth,
                "def worth: num PLN = value(st, day) in PLN by last(px)",
                "loaded",
            ),
            (
                worth,
                "def worth: num PLN = value(st, day) in PLN by last(hv)",
                "LOAD-UNIT value by last(hv): the price is money PLN, not num PLN/u (a.tally:57)",
            ),
            (
                worth,
                "def worth: num PLN = value(st, day) in PLN by last(zz)",
                "LOAD-NAME unknown price input zz (a.tally:57)",
            ),
            (
                worth,
                "def worth: num PLN = value(st, day) in PLN by rate",
                "loaded",
            ),
            (
                worth,
                "def worth: num PLN = value(st, day) in PLN by last(flowed)",
                "LOAD-UNIT value by flowed: a price is a num CUR/u keyed by unit, or an expression of that type with its keys (a.tally:57)",
            ),
        ];
        for (old, new, expected) in cases {
            assert!(MOVES.contains(old), "{old} is in the base");
            let text = MOVES.replacen(old, new, 1);
            assert_eq!(refusal(&text), expected, "{new}");
        }
    }

    /// A reality with two versions, keyed rows, a reading, an observation in a covered span,
    /// and band tables over ints, days and money: each case one edit of it.
    const VERSIONS: &str = "model g
  tally 2
  on 2026-06-01
  start 2026-01-01
  end 2026-12-31

source card \"an invented card\"

unit PLN: currency minor 0.01

kind wages: event certain
  leg cash: PLN

index qs = [q1, q2]

index tiers = [t1, t2, t3]

input amt: money PLN

input pay_day: date

input rate[k in qs]: num

input fee[k in qs, j in tiers]: num

input price: num PLN reading

input bank: observed

band scale(x: int): num

band dated(d: date): num

band tier(x: money PLN): num

place konto: accounts PLN lives 2026-01-01..

place world: outside lives 1900-01-01..

move pay: wages
  on pay_day
  leg cash from world to konto amount amt

ask q: amt + amt law 2026-05-01

version card_2026
  cite card
  governs always
  known 2026-01-01
  amt = 100.00 PLN \"the card\"
  pay_day = 2026-02-10 \"the card\"
  rate[q1] = 1 \"the card\"
  rate[q2] = 2 \"the card\"
  fee[q1, t1] = 1 \"the card\"
  price = 10 PLN at 2026-01-05 \"the card\"
  scale: [0, 10] = 1 \"the card\"
  scale: [11, 20) = 2 \"the card\"
  scale: [20, inf) = 3 \"the card\"
  dated: [2026-01-01, 2026-07-01) = 1 \"the card\"
  dated: [2026-07-01, inf) = 2 \"the card\"
  tier: [0, 100) = 1 \"the card\"
  tier: [100, inf) = 2 \"the card\"
  opening[konto] = 500.00 PLN at 2026-01-01 start \"the card\"

version bank_a
  cite card
  governs 2026-02
  known 2026-03-01
  covers konto
  anchor konto
  bank[r1] = (on: 2026-02-10, kind: wages, leg cash from world to konto amount 100.00 PLN, fulfils: pay) \"the statement\"
";

    #[test]
    fn versions_rows_observations_and_band_tables_are_checked_at_load_as_refcore_checks_them() {
        assert_eq!(refusal(VERSIONS), "loaded", "the base");
        let known = "  known 2026-01-01\n  amt = 100.00 PLN \"the card\"";
        let rate = "  rate[q2] = 2 \"the card\"";
        let fee = "  fee[q1, t1] = 1 \"the card\"";
        let price = "  price = 10 PLN at 2026-01-05 \"the card\"";
        let opening = "  opening[konto] = 500.00 PLN at 2026-01-01 start \"the card\"";
        let tier = "  tier: [100, inf) = 2 \"the card\"";
        let obs = "  bank[r1] = (on: 2026-02-10, kind: wages, leg cash from world to konto amount 100.00 PLN, fulfils: pay) \"the statement\"";
        let tiers = "index tiers = [t1, t2, t3]";
        let later = "\nversion card_b\n  cite card\n  governs 2026-04-01..\n  known 2026-04-01\n";
        let same = "\nversion card_c\n  cite card\n  governs 2026-04-01..\n  known 2026-01-01\n";
        let fine = "\n\nband fine(x: int): num\n\nversion f1\n  cite card\n  governs 2026-01..2026-03\n  known 2026-02-01\n  fine: [0, 10) = 1 \"a\"\n  fine: [10, 20) = 2 \"a\"\n  fine: [20, inf) = 3 \"a\"\n\nversion f2\n  cite card\n  governs 2026-03..\n  known 2026-04-01\n";
        let fy = "\n\nindex fy = year 2025..2026 from 04-06\n\ninput lim[y in fy]: num\n\n";
        let obs2 = "  bank[r1] = (on: 2026-02-11, kind: wages, leg cash from world to konto amount 100.00 PLN, fulfils: pay) \"again\"";
        let cases: Vec<(&str, String, &str)> = vec![
            // a version known in a window, and a row's own window
            (known, "  known window(2025-12-01, 2026-01-01)\n  amt = 100.00 PLN \"the card\"".to_owned(), "loaded"),
            (known, "  known window(2026-02-01, 2026-01-01)\n  amt = 100.00 PLN \"the card\"".to_owned(), "LOAD-DOMAIN window(a, b) with a after b (L §8.4) (a.tally:46)"),
            (known, "  known window(2026-01-01, 2026-07-01)\n  amt = 100.00 PLN \"the card\"".to_owned(), "LOAD-KNOWN version card_2026 is known after the model's on date (a.tally:46)"),
            (rate, "  rate[q2] = 2 known window(2026-01-02, 2026-02-01) \"the card\"".to_owned(), "loaded"),
            (rate, "  rate[q2] = 2 known window(2026-01-02, 2026-07-01) \"the card\"".to_owned(), "LOAD-KNOWN row rate[q2] is known after the model's on date (a.tally:53)"),
            (rate, "  rate[q2] = 2 known window(2025-12-01, 2026-02-01) \"the card\"".to_owned(), "LOAD-KNOWN row rate[q2] is known before its version card_2026 (a.tally:53)"),
            // a version's moves
            (obs, format!("{obs}{later}  moves bank_a to 2026-02-01..2026-03-31"), "loaded"),
            ("  covers konto", "  moves zz to 2026-03-01..\n  covers konto".to_owned(), "LOAD-MOVES moves zz: unknown, or not known before this version (a.tally:65)"),
            (known, "  known 2026-01-01\n  moves bank_a to 2026-03\n  amt = 100.00 PLN \"the card\"".to_owned(), "LOAD-MOVES moves bank_a: unknown, or not known before this version (a.tally:46)"),
            ("  covers konto", "  moves bank_a to 2026-03\n  covers konto".to_owned(), "LOAD-MOVES moves bank_a: unknown, or not known before this version (a.tally:65)"),
            (tier, format!("{tier}{later}  moves card_2026 to 2026-02-01..\n\nversion card_d\n  cite card\n  known 2026-04-01\n  moves card_2026 to 2026-05-01..\n"), "LOAD-MOVES two moves of card_2026 known the same day (a.tally:69)"),
            ("  covers konto", "  moves card_2026 to amt\n  covers konto".to_owned(), "LOAD-PARSE cannot parse: expected a date or a period, got amt (a.tally:65)"),
            (obs, format!("{obs}\n\nversion card_w\n  cite card\n  known window(2026-03-15, 2026-04-01)\n  moves bank_a to 2026-02-01..2026-03-31\n"), "loaded"),
            (obs, format!("{obs}\n\nversion card_w\n  cite card\n  known window(2026-04-01, 2026-04-10)\n  moves bank_a to 2026-02-01..2026-03-31\n{later}  moves bank_a to 2026-02-01..2026-04-30"), "LOAD-MOVES two moves of bank_a known the same day (a.tally:78)"),
            // a key that is also an id
            (tiers, "index tiers = [t1, t2, amt]".to_owned(), "LOAD-KEY a key is also an id: amt (a key of tiers) (a.tally:16)"),
            ("place world: outside lives 1900-01-01..", "place world: outside lives 1900-01-01..\n\nkind persons".to_owned(), "loaded"),
            ("kind wages: event certain", "kind persons\n\nkind wages: event certain".to_owned(), "loaded"),
            (obs, format!("{obs}\n\nkind persons\n\nversion people\n  cite card\n  known 2026-01-01\n  persons + [amt] lives 2000-01-01..\n"), "LOAD-KEY a key is also an id: amt (a.tally:78)"),
            (obs, format!("{obs}\n\nindex rows_ix: open\n\ninput tag[k in rows_ix]: num\n\nversion tags\n  cite card\n  governs always\n  known 2026-01-01\n  tag[amt] = 1 \"a\"\n"), "LOAD-KEY a key is also an id: amt (a key of rows_ix) (a.tally:73)"),
            // rows at the same keys: overlaps and replace
            (rate, format!("{rate}\n  rate[q2] = 3 \"again\""), "LOAD-OVERLAP rows a.tally:53 and a.tally:54 of rate overlap at [q2] in one version (a.tally:54)"),
            (rate, "  rate[q2] = 2 governs 2026-01..2026-03 \"the card\"\n  rate[q2] = 3 governs 2026-04.. \"again\"".to_owned(), "loaded"),
            (rate, "  rate[q2] = 2 governs 2026-01..2026-05 \"the card\"\n  rate[q2] = 3 governs 2026-04.. \"again\"".to_owned(), "LOAD-OVERLAP rows a.tally:53 and a.tally:54 of rate overlap at [q2] in one version (a.tally:54)"),
            (tier, format!("{tier}{later}  rate[q2] = 5 \"later\""), "LOAD-OVERLAP rows a.tally:53 and a.tally:67 of rate overlap at [q2], neither replacing the other (a.tally:67)"),
            (tier, format!("{tier}{later}  replace rate[q2] = 5 \"later\""), "loaded"),
            (rate, "  replace rate[q2] = 2 \"the card\"".to_owned(), "LOAD-REPLACE replace rate[q2]: no earlier row to replace (a.tally:53)"),
            (tier, format!("{tier}\nversion card_b\n  cite card\n  governs 2025\n  known 2026-04-01\n  replace rate[q2] = 5 \"later\""), "loaded"),
            (tier, format!("{tier}{same}  rate[q2] = 5 \"same day\""), "loaded"),
            (tier, format!("{tier}\nversion card_c\n  cite card\n  governs always\n  known 2026-01-01\n  rate[q2] = 5 \"same day\""), "LOAD-OVERLAP rows a.tally:53 and a.tally:67 of rate overlap at [q2], neither replacing the other (a.tally:67)"),
            (fee, format!("{fee}\n  fee[*, t1] = 2 \"all\""), "loaded"),
            (fee, format!("{fee}\n  fee[q1, *] = 2 \"all\"\n  fee[*, t1] = 3 \"all\""), "LOAD-OVERLAP rows a.tally:55 and a.tally:56 of fee overlap at [q1] and [*, t1] and tie (a.tally:56)"),
            (fee, "  fee[q1, t1..t2] = 2 \"a\"\n  fee[q1, t2..t3] = 3 \"b\"".to_owned(), "LOAD-OVERLAP rows a.tally:54 and a.tally:55 of fee overlap at [q1, t1..t2] and [q1, t2..t3] and tie (a.tally:55)"),
            (fee, "  fee[q1, t1..t3] = 2 \"a\"\n  fee[q1, t2..t3] = 3 \"b\"".to_owned(), "loaded"),
            (fee, format!("{fee}\n  fee[q1, t1..t3] = 3 \"b\""), "loaded"),
            // readings and balances: one instant each
            (price, format!("{price}\n  price = 11 PLN at 2026-01-05 \"again\""), "LOAD-OVERLAP rows a.tally:55 and a.tally:56 of price overlap at no keys in one version (a.tally:56)"),
            (price, format!("{price}\n  price = 11 PLN at 2026-01-06 \"again\""), "loaded"),
            (tier, format!("{tier}{later}  replace price = 12 PLN at 2026-01-05 \"later\""), "loaded"),
            (tier, format!("{tier}{later}  price = 12 PLN at 2026-01-05 \"later\""), "LOAD-OVERLAP rows a.tally:55 and a.tally:67 of price overlap at no keys, neither replacing the other (a.tally:67)"),
            (opening, format!("{opening}\n  opening[konto] = 600.00 PLN at 2026-01-01 start \"again\""), "LOAD-OVERLAP rows a.tally:63 and a.tally:64 of opening overlap at [konto] in one version (a.tally:64)"),
            (opening, format!("{opening}\n  opening[konto] = none at 2026-01-01 start \"again\""), "LOAD-OVERLAP rows a.tally:63 and a.tally:64 of opening overlap at [konto] in one version (a.tally:64)"),
            (opening, format!("{opening}\n  opening[konto] = 500.00 PLN at 2026-01-01 end \"again\""), "loaded"),
            // observations: one movement per id, on a covered day
            (obs, format!("  replace {}", obs.trim_start()), "LOAD-REPLACE replace bank[r1]: no earlier row to replace (a.tally:71)"),
            (obs, format!("{obs}\n{obs2}"), "LOAD-OVERLAP rows a.tally:71 and a.tally:72 of bank overlap at [r1] (a.tally:72)"),
            (obs, format!("{obs}\n\nversion bank_b\n  cite card\n  governs 2026-02\n  known 2026-04-01\n  anchor konto\n{obs2}"), "LOAD-OVERLAP rows a.tally:71 and a.tally:78 of bank overlap at [r1], neither replacing the other (a.tally:78)"),
            (obs, format!("{obs}\n\nversion bank_b\n  cite card\n  governs 2026-02\n  known 2026-04-01\n  anchor konto\n  replace {}", obs2.trim_start()), "loaded"),
            (obs, format!("{obs}\n  replace bank[r1] = (on: 2026-02-11, kind: wages, leg cash from world to konto amount 100.00 PLN, fulfils: pay) known 2026-03-02 \"again\""), "loaded"),
            (obs, obs.replace("2026-02-10", "2026-03-10"), "LOAD-COVERS observation bank[r1] of konto is dated outside the span bank_a covers (a.tally:71)"),
            ("  covers konto", "  covers konto 2026-02-01..2026-02-05".to_owned(), "LOAD-COVERS observation bank[r1] of konto is dated outside the span bank_a covers (a.tally:71)"),
            ("  covers konto", "  covers konto 2026-02-01..2026-02-28".to_owned(), "loaded"),
            // one band table: rows in order, whole, no gap
            ("  scale: [0, 10] = 1 \"the card\"", "  scale: [10, 0] = 1 \"the card\"".to_owned(), "LOAD-BAND band scale in card_2026: rows unordered, overlapping or with a gap at 10 (a.tally:56)"),
            ("  scale: [0, 10] = 1 \"the card\"", "  scale: [0, 0) = 1 \"the card\"".to_owned(), "LOAD-BAND band scale in card_2026: rows unordered, overlapping or with a gap at 0 (a.tally:56)"),
            ("  scale: [11, 20) = 2 \"the card\"", "  scale: [12, 20) = 2 \"the card\"".to_owned(), "LOAD-BAND band scale in card_2026: rows unordered, overlapping or with a gap at 12 (a.tally:57)"),
            ("  scale: [11, 20) = 2 \"the card\"", "  scale: (10, 20) = 2 \"the card\"".to_owned(), "loaded"),
            ("  scale: [11, 20) = 2 \"the card\"", "  scale: [10.5, 20) = 2 \"the card\"".to_owned(), "LOAD-UNIT 10.5 is not an int (a.tally:57)"),
            ("  scale: [11, 20) = 2 \"the card\"", "  scale: [9, 20) = 2 \"the card\"".to_owned(), "LOAD-BAND band scale in card_2026: rows unordered, overlapping or with a gap at 9 (a.tally:57)"),
            ("  scale: [20, inf) = 3 \"the card\"", "  scale: [20, inf) = 3 \"the card\"\n  scale: [30, 40) = 4 \"more\"".to_owned(), "LOAD-BAND band scale in card_2026: rows unordered, overlapping or with a gap at 30 (a.tally:59)"),
            ("  scale: [11, 20) = 2 \"the card\"", "  scale: (-inf, 20) = 2 \"the card\"".to_owned(), "LOAD-BAND band scale in card_2026: rows unordered, overlapping or with a gap at -inf (a.tally:57)"),
            (tier, "  tier: (100, inf) = 2 \"the card\"".to_owned(), "LOAD-BAND band tier in card_2026: rows unordered, overlapping or with a gap at 100 (a.tally:62)"),
            (tier, "  tier: [100.5, inf) = 2 \"the card\"".to_owned(), "LOAD-BAND band tier in card_2026: rows unordered, overlapping or with a gap at 201/2 (a.tally:62)"),
            ("  tier: [0, 100) = 1 \"the card\"", "  tier: [0, 100.5) = 1 \"the card\"".to_owned(), "LOAD-BAND band tier in card_2026: rows unordered, overlapping or with a gap at 100 (a.tally:62)"),
            (tier, "  tier: [100.001, inf) = 2 \"the card\"".to_owned(), "LOAD-PRECISION 100.001 is finer than the minor unit (a.tally:62)"),
            ("  dated: [2026-07-01, inf) = 2 \"the card\"", "  dated: [2026-07-02, inf) = 2 \"the card\"".to_owned(), "LOAD-BAND band dated in card_2026: rows unordered, overlapping or with a gap at 2026-07-02 (a.tally:60)"),
            (tier, "  tier: [abc, inf) = 2 \"the card\"".to_owned(), "LOAD-UNIT a value of the wrong type: abc for money PLN (a.tally:62)"),
            // tables of one key across versions: replace, whole or patching
            (tier, format!("{tier}{later}  tier: [0, inf) = 9 \"later\""), "LOAD-OVERLAP band tier: the table of card_b overlaps the table of card_2026 without replace (a.tally:67)"),
            (tier, format!("{tier}{later}  replace tier: [100, inf) = 9 \"later\""), "loaded"),
            (tier, format!("{tier}{later}  replace tier: [0, 50) = 8 \"later\"\n  replace tier: [50, inf) = 9 \"later\""), "loaded"),
            (tier, format!("{tier}{later}  replace tier: [0, 50) = 8 \"later\""), "loaded"),
            (tier, format!("{tier}{later}  replace tier: [100, inf) = 9 \"later\"\n  tier: [0, 100) = 8 \"later\""), "LOAD-BAND band tier in card_b: rows unordered, overlapping or with a gap at 0 (a.tally:68)"),
            (tier, format!("{tier}\n  replace scale: [0, 10] = 4 \"x\""), "LOAD-BAND band scale in card_2026: rows unordered, overlapping or with a gap at 0 (a.tally:63)"),
            (tier, format!("{tier}{same}  tier: [0, inf) = 9 \"same\""), "LOAD-OVERLAP band tier: tables of card_2026 and card_c overlap, known the same day (a.tally:61)"),
            (tier, format!("{tier}\nversion card_b\n  cite card\n  governs 2025-06-01..\n  known 2026-04-01\n  replace tier: [100, inf) = 9 \"later\""), "loaded"),
            (tier, format!("{tier}\nversion card_b\n  cite card\n  governs 2025-06-01..\n  known 2026-04-01\n  replace tier: [0, inf) = 9 \"later\""), "loaded"),
            (obs, format!("{obs}{fine}  replace fine: [0, 10) = 5 \"b\"\n  replace fine: [20, inf) = 6 \"b\""), "LOAD-REPLACE replace rows of fine in f2 patch no table on 2026-04-01.. (a.tally:87)"),
            (obs, format!("{obs}{}  replace fine: [0, 10) = 5 \"b\"\n  replace fine: [20, inf) = 6 \"b\"", fine.replace("governs 2026-03..", "governs 2026-02..2026-03")), "loaded"),
            (obs, format!("{obs}{fine}  replace fine: [0, 10) = 5 \"b\"\n  replace fine: [10, inf) = 6 \"b\""), "loaded"),
            (obs, format!("{obs}{fine}  replace fine: [0, 5) = 5 \"b\"\n  replace fine: [20, inf) = 6 \"b\""), "LOAD-BAND band fine in f2: the replace rows neither match the earlier table's intervals nor form a whole table (a.tally:87)"),
            // a period across two keys of a fiscal-year index
            (tiers, format!("{tiers}{fy}def l[m in months]: num = lim[m]"), "LOAD-KEY 2026-04 lies across keys 2025 and 2026 of fy (a.tally:22)"),
            (tiers, format!("{tiers}{fy}ask zq: lim[2026-04]"), "LOAD-KEY 2026-04 lies across keys 2025 and 2026 of fy (a.tally:22)"),
            (tiers, format!("{tiers}{fy}ask zq: lim[2026-05]"), "loaded"),
            (tiers, format!("{tiers}{fy}ask zq: lim[2026-04-05]"), "loaded"),
            (tiers, format!("{tiers}{fy}def l: num = sum(lim[k] for k in months)"), "loaded"),
            (tiers, format!("{tiers}{fy}def l[d in days]: num = lim[month_of(d)]"), "LOAD-KEY 2026-04 lies across keys 2025 and 2026 of fy (a.tally:22)"),
            (tiers, format!("{tiers}{fy}def l[m in months]: num = lim[(m)]"), "loaded"),
            (tiers, format!("{tiers}{fy}def l[y in years]: num = lim[y]"), "LOAD-KEY 2026 lies across keys 2025 and 2026 of fy (a.tally:22)"),
            (tiers, format!("{tiers}{fy}def l[k in qs, m in months]: num = lim[m]"), "LOAD-KEY 2026-04 lies across keys 2025 and 2026 of fy (a.tally:22)"),
            (tiers, format!("{tiers}{fy}index acct: open\n\ndef l[a in acct, m in months]: num = lim[m]"), "loaded"),
            (tiers, format!("{tiers}{fy}move tick[m in months]: wages\n  on start_of(m)\n  leg cash from world to konto amount 1.00 PLN\n  when lim[m] > 0"), "LOAD-KEY 2026-04 lies across keys 2025 and 2026 of fy (a.tally:25)"),
        ];
        for (old, new, expected) in cases {
            assert!(VERSIONS.contains(old), "{old} is in the base");
            let text = VERSIONS.replacen(old, &new, 1);
            assert_eq!(refusal(&text), expected, "{new}");
        }
    }

    /// The first line a run of `text` prints for ask `q`, or `None` where the engine doesn't
    /// build what the run needs.
    fn first_line(text: &str) -> Option<String> {
        let given = vec![tally::Given {
            path: "a.tally".to_owned(),
            bytes: Some(text.as_bytes().to_vec()),
        }];
        let request = tally::Request {
            asks: tally::Asks::Named(vec!["q".to_owned()]),
            pin: tally::Pin::Canonical,
            days: tally::Days::Events,
        };
        match tally::run(given, &request) {
            Ok(answer) => answer.lines().first().cloned(),
            Err(_) => None,
        }
    }

    #[test]
    fn an_ask_over_a_known_window_or_a_moves_line_is_not_built_yet() {
        assert_eq!(
            first_line(VERSIONS).as_deref(),
            Some("q: 200.00 PLN"),
            "refcore's first line"
        );
        let edits = [
            (
                "  known 2026-01-01\n  amt",
                "  known window(2025-12-01, 2026-01-01)\n  amt",
            ),
            (
                "  rate[q2] = 2 \"the card\"",
                "  rate[q2] = 2 known window(2026-01-02, 2026-02-01) \"the card\"",
            ),
            (
                "\"the statement\"\n",
                "\"the statement\"\n\nversion card_b\n  cite card\n  governs 2026-04-01..\n  known 2026-04-01\n  moves bank_a to 2026-02-01..2026-03-31\n",
            ),
        ];
        for (old, new) in edits {
            assert!(VERSIONS.contains(old), "{old} is in the base");
            let text = VERSIONS.replacen(old, new, 1);
            assert_eq!(refusal(&text), "loaded", "{new} loads");
            assert_eq!(first_line(&text), None, "{new}: M1 runs it");
        }
    }
}
