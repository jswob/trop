//! The builder's load tests: a small reality loads to refcore's indexes, things, rows and observations, and load's refusals print refcore's text.
//! Spec: E §3.1 steps 1–4; L §4.1 (indexes), §4.6 (rows and their stable ids), §5.3
//! (versions), §5.7 (kinds, things, lifespans), §5.10 (places), §5.19 (observations), §8.3,
//! §8.5 (WARN-UNREAD for a source and a place); §5.10 (priced places, `bal`, `value`), §5.14
//! (moves and legs, `flows`), §5.19 (`observed`, `gap`, `covered`) for the checks of moves,
//! prices and reads.
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
}
