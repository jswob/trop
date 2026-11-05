//! The builder's load tests: a small reality loads to refcore's indexes, things, rows and observations, and load's refusals print refcore's text.
//! Spec: E §3.1 steps 1–4; L §4.1 (indexes), §4.6 (rows and their stable ids), §5.3
//! (versions), §5.7 (kinds, things, lifespans), §5.10 (places), §5.19 (observations), §8.3,
//! §8.5 (WARN-UNREAD for a source and a place); §5.10 (priced places, `bal`, `value`), §5.14
//! (moves and legs, `flows`), §5.19 (`observed`, `gap`, `covered`) for the checks of moves,
//! prices and reads; §5.3 (known windows, `moves`), §4.6 (overlap and `replace`, a period
//! across keys), §5.9 (band tables), §5.19 (`covers`) for the checks of versions and rows;
//! §3.6, §5.7 (the parts clauses, the arrival and leaving rules), §5.19 (stated parts),
//! §5.14.1 (an arrival rule's reads) for the checks of parts; §5.11 (recurrence reads and
//! their guards), §4.3 (law dates), §5.14.1 (running reads, a decision's balance of a day
//! load sees), §4.6 (readings `after` a move), §4.2 (schedules) for the item graph's checks;
//! §5.5 (axes, their keys, a mode, `LOAD-FORKS`), §5.12 (fills and `.taken`), §5.11 (`as
//! if`), §5.18 (pins and the ask forms), §6.4 (an ask naming an unloaded world) for G8's;
//! §5.16 (distributions, queries, `crps`, `cdf`, `pdf`, `pmf`, the random cone), §7.3 and
//! §3.3 (`to u`, the transcendental calls), §5.18 (`solve`), §5.13 (`prorata`), §6.4 (an
//! expression row outside a scenario) for G8's second step.
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

    /// Two place kinds that keep parts, one by owner and one by batch, an arrival rule and
    /// both leaving rules, and an opening stated by parts.
    const PARTS: &str = "model g
  tally 2
  on 2026-06-01
  start 2026-01-01
  end 2026-12-31

source card \"a card\"

unit PLN: currency minor 0.01

kind persons: parties

kind gifts: transfers

kind pots: place PLN min 0
  owned by share
  step minor(PLN)
  takes k for w order (if k.owner == ola then 0 else 1) prorata ties by (if k.owner == ola then 0 else 1)

kind lots: place PLN min 0
  batched
  step minor(PLN)
  takes k for w prorata

place pot: pots lives 2026-01-01..

place lot: lots lives 2026-01-01..

place konto: accounts PLN lives 2026-01-01..

place world: outside lives 1900-01-01..

input amt: money PLN

input day: date

def share[m in events, p in parties]: num at m.day
  rule base = (if p == ola then 1 else 0)
  rule gift beats base = (if p == jan then 1 else 0) when m in gifts

move dep: transfers
  on day
  leg cash from world to pot amount amt

move buy: transfers
  on day
  leg cash from world to lot amount amt

ask q: bal(pot, 2026-12-31) of p for p in parties on 2026-12-31

version cast
  cite card
  known 2026-01-01
  persons + [ola] lives 1980-01-01..
  persons + [jan] lives 1982-01-01..

version card_2026
  cite card
  governs always
  known 2026-01-01
  amt = 100.00 PLN \"the card\"
  day = 2026-02-10 \"the card\"
  opening[pot] = 30.00 PLN at 2026-01-01 start \"the card\"
  opening[pot] of ola = 10.00 PLN at 2026-01-01 start \"the card\"
  opening[pot] of jan = 20.00 PLN at 2026-01-01 start \"the card\"
  opening[konto] = 5.00 PLN at 2026-01-01 start \"the card\"
  opening[lot] = 0 PLN at 2026-01-01 start \"the card\"
";

    #[test]
    fn parts_clauses_and_stated_parts_are_checked_at_load_as_refcore_checks_them() {
        assert_eq!(refusal(PARTS), "loaded", "the base");
        let cases: Vec<(&str, String, &str)> = vec![
            // base
            ("  amt = 100.00 PLN \"the card\"", "  amt = 100.00 PLN \"the card\"".to_owned(), "loaded"),
            // not-place
            ("kind persons: parties", "kind persons: parties\n  owned by share\n  step minor(PLN)\n  takes k for w prorata".to_owned(), "LOAD-PARTS kind persons keeps parts and is not a place kind (a.tally:12)"),
            // under
            ("place pot: pots lives 2026-01-01..", "kind sub_pots: pots\n  owned by share\n  step minor(PLN)\n  takes k for w prorata\n\nplace pot: sub_pots lives 2026-01-01..".to_owned(), "LOAD-PARTS sub_pots states parts clauses under pots, which keeps parts (a.tally:26)"),
            // sub-keeps
            ("place pot: pots lives 2026-01-01..", "kind small_pots: pots\n\nplace pot: small_pots lives 2026-01-01..".to_owned(), "loaded"),
            // sub-of-plain-place
            ("kind pots: place PLN min 0", "kind savings: holdings\n  owned by share\n  step minor(PLN)\n  takes k for w prorata\n\nkind pots: place PLN min 0".to_owned(), "loaded"),
            // twice
            ("  batched\n", "  batched\n  batched\n".to_owned(), "LOAD-PARTS kind lots: batched stated twice (a.tally:22)"),
            // order
            ("  batched\n  step minor(PLN)", "  step minor(PLN)\n  batched".to_owned(), "LOAD-PARTS kind lots: batched after step (the parts clauses go owned by, batched, step, takes) (a.tally:22)"),
            // step-missing
            ("  owned by share\n  step minor(PLN)\n", "  owned by share\n".to_owned(), "LOAD-PARTS kind pots: step missing (a.tally:16)"),
            // takes-missing
            ("  batched\n  step minor(PLN)\n  takes k for w prorata", "  batched\n  step minor(PLN)".to_owned(), "LOAD-PARTS kind lots: takes missing (a.tally:21)"),
            // owned-missing
            ("  batched\n  step minor(PLN)\n  takes k for w prorata", "  step minor(PLN)\n  takes k for w prorata".to_owned(), "LOAD-PARTS kind lots: owned by or batched missing (a.tally:21)"),
            // signed
            ("kind pots: place PLN min 0", "kind pots: place PLN signed".to_owned(), "LOAD-PARTS kind pots keeps parts and is signed (a.tally:16)"),
            // max0
            ("kind pots: place PLN min 0", "kind pots: place PLN max 0".to_owned(), "LOAD-PARTS kind pots keeps parts and is max 0 (a.tally:16)"),
            // asset
            ("kind pots: place PLN min 0", "kind homes: assets\n  owned by share\n  step minor(PLN)\n  takes k for w prorata\n\nkind pots: place PLN min 0".to_owned(), "LOAD-PARTS kind homes keeps parts and is an asset kind (a.tally:16)"),
            // accounts-sub
            ("place konto: accounts PLN lives 2026-01-01..", "kind sav2: accounts\n  owned by share\n  step minor(PLN)\n  takes k for w prorata\n\nplace konto: sav2 PLN lives 2026-01-01..".to_owned(), "LOAD-PARTS kind sav2 keeps parts and is signed (a.tally:30)"),
            // both
            ("  batched\n  step minor(PLN)\n  takes k for w prorata", "  owned by share\n  batched\n  step minor(PLN)\n  takes k for w prorata".to_owned(), "loaded"),
            // rule-unknown
            ("  owned by share", "  owned by nope".to_owned(), "LOAD-NAME unknown id nope (a.tally:16)"),
            // rule-not-def
            ("  owned by share", "  owned by amt".to_owned(), "LOAD-PARTS arrival rule amt is not a num keyed by a movement and a party at its movement's day (a.tally:16)"),
            // rule-keys
            ("def share[m in events, p in parties]: num at m.day", "def share[p in parties, m in events]: num at m.day".to_owned(), "LOAD-PARTS arrival rule share is not a num keyed by a movement and a party at its movement's day (a.tally:16)"),
            // rule-one-key
            ("def share[m in events, p in parties]: num at m.day\n  rule base = (if p == ola then 1 else 0)\n  rule gift beats base = (if p == jan then 1 else 0) when m in gifts", "def share[p in parties]: num\n  rule base = (if p == ola then 1 else 0)".to_owned(), "LOAD-PARTS arrival rule share is not a num keyed by a movement and a party at its movement's day (a.tally:16)"),
            // rule-money
            ("def share[m in events, p in parties]: num at m.day", "def share[m in events, p in parties]: money PLN at m.day".to_owned(), "LOAD-PARTS arrival rule share is not a num keyed by a movement and a party at its movement's day (a.tally:16)"),
            // rule-no-at
            ("def share[m in events, p in parties]: num at m.day", "def share[m in events, p in parties]: num".to_owned(), "LOAD-PARTS arrival rule share is not a num keyed by a movement and a party at its movement's day (a.tally:16)"),
            // rule-at-paren
            ("def share[m in events, p in parties]: num at m.day", "def share[m in events, p in parties]: num at (m.day)".to_owned(), "loaded"),
            // rule-at-other
            ("def share[m in events, p in parties]: num at m.day", "def share[m in events, p in parties]: num at day".to_owned(), "LOAD-PARTS arrival rule share is not a num keyed by a movement and a party at its movement's day (a.tally:16)"),
            // rule-subkinds
            ("def share[m in events, p in parties]: num at m.day", "def share[m in gifts, p in persons]: num at m.day".to_owned(), "loaded"),
            // rule-key-index
            ("def share[m in events, p in parties]: num at m.day", "index ks = [a1, a2]\n\ndef share[m in ks, p in parties]: num at m.day".to_owned(), "LOAD-PARTS arrival rule share is not a num keyed by a movement and a party at its movement's day (a.tally:16)"),
            // rule-ties
            ("  owned by share", "  owned by share ties by (if p == ola then 0 else 1)".to_owned(), "loaded"),
            // rule-ties-unknown
            ("  owned by share", "  owned by share ties by zz".to_owned(), "loaded"),
            // takes-dup
            ("  takes k for w order", "  takes w for w order".to_owned(), "LOAD-DUP w is declared twice (takes w for w) (a.tally:18)"),
            // takes-v-unknown
            ("  takes k for w order", "  takes k for w v nope order".to_owned(), "LOAD-NAME takes of pots: v nope names no version (a.tally:18)"),
            // takes-v
            ("  takes k for w order", "  takes k for w v card_2026 order".to_owned(), "loaded"),
            // takes-shadow
            ("  takes k for w order", "  takes amt for w order".to_owned(), "LOAD-SHADOW amt shadows a top-level id (a.tally:18)"),
            // takes-shadow-w
            ("  takes k for w order", "  takes k for day order".to_owned(), "LOAD-SHADOW day shadows a top-level id (a.tally:18)"),
            // takes-term-unknown
            ("  takes k for w order (if k.owner == ola then 0 else 1) prorata ties by (if k.owner == ola then 0 else 1)", "  takes k for w order zz".to_owned(), "loaded"),
            // takes-batch-day
            ("  takes k for w order (if k.owner == ola then 0 else 1) prorata ties by (if k.owner == ola then 0 else 1)", "  takes k for w order k.batch.day".to_owned(), "loaded"),
            // takes-field
            ("  takes k for w order (if k.owner == ola then 0 else 1) prorata ties by (if k.owner == ola then 0 else 1)", "  takes k for w order k.nope".to_owned(), "LOAD-FIELD nope is not a field of (owner key parties, batch key events) (a.tally:18)"),
            // takes-tie-w
            ("  takes k for w order (if k.owner == ola then 0 else 1) prorata ties by (if k.owner == ola then 0 else 1)", "  takes k for w order (if k.owner == ola then 0 else 1) prorata ties by w".to_owned(), "loaded"),
            // step-eur
            ("  owned by share\n  step minor(PLN)\n", "  owned by share\n  step minor(EUR)\n".to_owned(), "LOAD-UNIT money in an undeclared currency EUR (a.tally:17)"),
            // step-number
            ("  owned by share\n  step minor(PLN)\n", "  owned by share\n  step 0.01 PLN\n".to_owned(), "loaded"),
            // step-unknown
            ("  owned by share\n  step minor(PLN)\n", "  owned by share\n  step zz\n".to_owned(), "loaded"),
            // owns
            ("input day: date", "input day: date\n\ninput owns[p in parties, x in places]: num".to_owned(), "LOAD-PARTS owns[ola, pot]: pot keeps parts (a.tally:65)"),
            // of-amt
            ("  amt = 100.00 PLN \"the card\"", "  amt of ola = 100.00 PLN \"the card\"".to_owned(), "LOAD-PARTS of on a row of amt (a.tally:61)"),
            // of-konto
            ("  opening[konto] = 5.00 PLN at 2026-01-01 start \"the card\"", "  opening[konto] = 5.00 PLN at 2026-01-01 start \"the card\"\n  opening[konto] of ola = 5.00 PLN at 2026-01-01 start \"the card\"".to_owned(), "LOAD-PARTS of ola at konto, which keeps no labels of its sort (a.tally:67)"),
            // of-batch-at-pot
            ("  opening[pot] of jan = 20.00 PLN at 2026-01-01 start \"the card\"", "  opening[pot] of dep = 20.00 PLN at 2026-01-01 start \"the card\"".to_owned(), "LOAD-PARTS of dep at pot, which keeps no labels of its sort (a.tally:65)"),
            // of-pair-at-pot
            ("  opening[pot] of jan = 20.00 PLN at 2026-01-01 start \"the card\"", "  opening[pot] of (jan, dep) = 20.00 PLN at 2026-01-01 start \"the card\"".to_owned(), "LOAD-PARTS of (jan, dep) at pot, which keeps no labels of its sort (a.tally:65)"),
            // of-batch-at-lot
            ("  opening[lot] = 0 PLN at 2026-01-01 start \"the card\"", "  opening[lot] = 10.00 PLN at 2026-01-01 start \"the card\"\n  opening[lot] of dep = 10.00 PLN at 2026-01-01 start \"the card\"".to_owned(), "loaded"),
            // of-party-at-lot
            ("  opening[lot] = 0 PLN at 2026-01-01 start \"the card\"", "  opening[lot] = 10.00 PLN at 2026-01-01 start \"the card\"\n  opening[lot] of ola = 10.00 PLN at 2026-01-01 start \"the card\"".to_owned(), "LOAD-PARTS of ola at lot, which keeps no labels of its sort (a.tally:68)"),
            // no total row at all: refcore loads it (it skips an input with no total row);
            // the spec wins (#75), L §5.19 and §8.3's form
            ("  opening[pot] = 30.00 PLN at 2026-01-01 start \"the card\"\n  opening[pot] of ola = 10.00 PLN at 2026-01-01 start \"the card\"\n  opening[pot] of jan = 20.00 PLN at 2026-01-01 start \"the card\"\n  opening[konto] = 5.00 PLN at 2026-01-01 start \"the card\"\n  opening[lot] = 0 PLN at 2026-01-01 start \"the card\"", "  opening[pot] of ola = 10.00 PLN at 2026-01-01 start \"the card\"\n  opening[pot] of jan = 20.00 PLN at 2026-01-01 start \"the card\"".to_owned(), "LOAD-PARTS the parts of pot at 2026-01-01 start have no total in card_2026 (a.tally:63)"),
            // no-total-other-instant
            ("  opening[pot] of jan = 20.00 PLN at 2026-01-01 start \"the card\"", "  opening[pot] of jan = 20.00 PLN at 2026-01-01 end \"the card\"".to_owned(), "LOAD-PARTS the parts of pot at 2026-01-01 end have no total in card_2026 (a.tally:65)"),
            // total-none
            ("  opening[pot] = 30.00 PLN at 2026-01-01 start \"the card\"", "  opening[pot] = none at 2026-01-01 start \"the card\"".to_owned(), "LOAD-PARTS the parts of pot at 2026-01-01 start have no total in card_2026 (a.tally:64)"),
            // below
            ("  opening[pot] of jan = 20.00 PLN at 2026-01-01 start \"the card\"", "  opening[pot] of jan = -20.00 PLN at 2026-01-01 start \"the card\"".to_owned(), "LOAD-PARTS the stated part of jan at pot is below 0 (a.tally:65)"),
            // overlap
            ("  opening[pot] of jan = 20.00 PLN at 2026-01-01 start \"the card\"", "  opening[pot] of ola = 20.00 PLN at 2026-01-01 start \"the card\"".to_owned(), "LOAD-PARTS the stated parts ola and ola of pot overlap (a.tally:65)"),
            // over
            ("  opening[pot] of jan = 20.00 PLN at 2026-01-01 start \"the card\"", "  opening[pot] of jan = 25.00 PLN at 2026-01-01 start \"the card\"".to_owned(), "LOAD-PARTS the stated parts of pot sum to 35.00 PLN, over its total 30.00 PLN (a.tally:65)"),
            // total-hole
            ("  opening[pot] = 30.00 PLN at 2026-01-01 start \"the card\"", "  opening[pot] = ?\"how much\" at 2026-01-01 start \"the card\"".to_owned(), "loaded"),
            // part-hole
            ("  opening[pot] of jan = 20.00 PLN at 2026-01-01 start \"the card\"", "  opening[pot] of jan = ?\"how much\" at 2026-01-01 start \"the card\"".to_owned(), "loaded"),
            // label-unknown
            ("  opening[pot] of jan = 20.00 PLN at 2026-01-01 start \"the card\"", "  opening[pot] of zz = 20.00 PLN at 2026-01-01 start \"the card\"".to_owned(), "loaded"),
            // label-unknown-below
            ("  opening[pot] of jan = 20.00 PLN at 2026-01-01 start \"the card\"", "  opening[pot] of zz = -20.00 PLN at 2026-01-01 start \"the card\"".to_owned(), "LOAD-PARTS the stated part of zz at pot is below 0 (a.tally:65)"),
            // lot-overlap
            ("  opening[lot] = 0 PLN at 2026-01-01 start \"the card\"", "  opening[lot] = 10.00 PLN at 2026-01-01 start \"the card\"\n  opening[lot] of dep = 5.00 PLN at 2026-01-01 start \"the card\"\n  opening[lot] of dep = 5.00 PLN at 2026-01-01 start \"the card\"".to_owned(), "LOAD-PARTS the stated parts dep and dep of lot overlap (a.tally:69)"),
            // read-of-type
            ("ask q: bal(pot, 2026-12-31) of p for p in parties on 2026-12-31", "ask q: bal(pot, 2026-12-31) of p for p in parties on 2026-12-31\n\nask q2: bal(x, 2026-12-31) of ola for x in lots on 2026-12-31".to_owned(), "LOAD-PARTS of ola at lots, which keeps no labels of its sort (a.tally:51)"),
            // read-of-type-ok
            ("ask q: bal(pot, 2026-12-31) of p for p in parties on 2026-12-31", "ask q: bal(pot, 2026-12-31) of p for p in parties on 2026-12-31\n\nask q2: bal(x, 2026-12-31) of dep for x in lots on 2026-12-31".to_owned(), "loaded"),
            // read-of-plain-type
            ("ask q: bal(pot, 2026-12-31) of p for p in parties on 2026-12-31", "ask q: bal(pot, 2026-12-31) of p for p in parties on 2026-12-31\n\nask q2: bal(x, 2026-12-31) of ola for x in accounts on 2026-12-31".to_owned(), "loaded"),
            // read-of-literal
            ("ask q: bal(pot, 2026-12-31) of p for p in parties on 2026-12-31", "ask q: bal(pot, 2026-12-31) of p for p in parties on 2026-12-31\n\nask q2: bal(lot, 2026-12-31) of ola".to_owned(), "LOAD-PARTS of ola at lot, which keeps no labels of its sort (a.tally:51)"),
            // arrival-end
            ("  rule base = (if p == ola then 1 else 0)", "  rule base = (if p == ola then 1 else 0) when bal(konto, m.day) > 0".to_owned(), "LOAD-CYCLE arrival rule share reads bal(konto, m.day): the end of its arrival's day (a.tally:38)"),
            // arrival-start
            ("  rule base = (if p == ola then 1 else 0)", "  rule base = (if p == ola then 1 else 0) when bal(konto, m.day start) > 0".to_owned(), "loaded"),
            // arrival-later
            ("  rule base = (if p == ola then 1 else 0)", "  rule base = (if p == ola then 1 else 0) when bal(konto, m.day + 1 day start) > 0".to_owned(), "LOAD-CYCLE arrival rule share reads bal(konto, m.day + 1 day start): a day after its arrival's (a.tally:38)"),
            // arrival-earlier
            ("  rule base = (if p == ola then 1 else 0)", "  rule base = (if p == ola then 1 else 0) when bal(konto, m.day - 1 day) > 0".to_owned(), "loaded"),
        ];
        for (old, new, expected) in cases {
            assert!(PARTS.contains(old), "{old} is in the base");
            let mut text = PARTS.replacen(old, &new, 1);
            if new.contains("input owns") {
                let day = "  day = 2026-02-10 \"the card\"";
                text = text.replacen(day, &format!("{day}\n  owns[ola, pot] = 1 \"the card\""), 1);
            }
            assert_eq!(refusal(&text), expected, "{new}");
        }
    }

    #[test]
    fn a_def_a_takes_order_reads_is_read_and_an_ask_over_parts_is_not_built_yet() {
        let order = "order (if k.owner == ola then 0 else 1) prorata";
        let day = "input day: date";
        assert!(
            PARTS.contains(order) && PARTS.contains(day),
            "the base's lines"
        );
        let text = PARTS.replacen(order, "order prio[k.owner] prorata", 1).replacen(
            day,
            "input day: date\n\ndef prio[p in parties]: num = (if p == ola then 0 else 1)\n\ndef idle: num = 1",
            1,
        );
        let m = load_one(&text).expect("it loads");
        let warned: Vec<&str> = m.warnings.iter().map(|(_, w)| w.as_str()).collect();
        assert_eq!(
            warned,
            [
                "def idle is read by nothing",
                "place konto is read by nothing"
            ],
            "refcore's WARN-UNREAD lines: prio is read by the takes order, share by the owned clause"
        );
        assert_eq!(first_line(PARTS), None, "M5b walks the parts");
    }

    /// A reality with a recurrence guarded by its rules' conditions, defs that state their law
    /// dates, a band, a plan, a schedule and a decision: each case one or more edits of it.
    const GRAPH: &str = "model g
  tally 2
  on 2026-03-01
  start 2026-01-01
  end 2026-12-31

source card \"an invented card\"

unit PLN: currency minor 0.01

kind wages: event certain
  leg cash: PLN

index qs = [q1, q2]

index gm = month 2026-02..2026-03

input amt: money PLN

input day: date

input rate[k in qs]: num

input cal: calendar

band tax(x: money PLN): num

place konto: accounts PLN lives 2026-01-01..

place safe: accounts PLN lives 2026-01-01..

place world: outside lives 1900-01-01..

def acc[m in months]: num
  rule base = rate[q1]
  rule step beats base = acc[m - 1] + rate[q2] when pos(m) > 1

def fee: money PLN = amt at day

def levy: num = tax(amt) at day

move pay[m in gm]: wages
  on start_of(m)
  leg cash from world to konto amount amt

move rent[m in gm]: transfers
  on schedule(m, 1, unadjusted, cal)
  leg cash from konto to world amount amt

move sweep[m in gm]: transfers
  on end_of(m)
  reads konto
  leg cash from konto to safe amount bal(konto, end_of(m) - 1 day)

ask q: fee, levy, acc[2026-02], bal(konto, 2026-12-31)

version card_2026
  cite card
  governs always
  known 2026-01-01
  amt = 100.00 PLN \"the card\"
  day = 2026-02-10 \"the card\"
  rate[q1] = 1 \"the card\"
  rate[q2] = 2 \"the card\"
  cal = calendar(weekends [sat, sun]) \"the card\"
  tax: [0 PLN, inf) = 0 \"the card\"
  opening[konto] = 500.00 PLN at 2026-01-01 start \"the card\"
  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"
";

    #[test]
    fn the_item_graph_s_checks_refuse_as_refcore_refuses() {
        let cases: &[(&[(&str, &str)], &str)] = &[
            // base
            (&[], "loaded"),
            // guard
            (
                &[(
                    "  rule step beats base = acc[m - 1] + rate[q2] when pos(m) > 1",
                    "  rule step beats base = acc[m - 1] + rate[q2]",
                )],
                "LOAD-GUARD acc[m - n] is a recurrence read with no condition on m or pos(m) guarding it (a.tally:36)",
            ),
            // guard-later
            (
                &[(
                    "  rule step beats base = acc[m - 1] + rate[q2] when pos(m) > 1",
                    "  rule step beats base = acc[m + 1] + rate[q2]",
                )],
                "LOAD-GUARD acc[m + n] is a recurrence read with no condition on m or pos(m) guarding it (a.tally:36)",
            ),
            // guard-by-beater
            (
                &[("  rule base = rate[q1]", "  rule base = acc[m - 1]")],
                "loaded",
            ),
            // guard-if
            (
                &[(
                    "  rule step beats base = acc[m - 1] + rate[q2] when pos(m) > 1",
                    "  rule step beats base = (if pos(m) > 1 then acc[m - 1] else rate[q1]) + rate[q2]",
                )],
                "loaded",
            ),
            // guard-and
            (
                &[(
                    "  rule step beats base = acc[m - 1] + rate[q2] when pos(m) > 1",
                    "  rule step beats base = rate[q2] when pos(m) > 1 and acc[m - 1] > 0",
                )],
                "loaded",
            ),
            // guard-and-left
            (
                &[(
                    "  rule step beats base = acc[m - 1] + rate[q2] when pos(m) > 1",
                    "  rule step beats base = rate[q2] when acc[m - 1] > 0 and pos(m) > 1",
                )],
                "loaded",
            ),
            // both
            (
                &[(
                    "  rule step beats base = acc[m - 1] + rate[q2] when pos(m) > 1",
                    "  rule step beats base = acc[m - 1] + acc[m + 1] when pos(m) > 1",
                )],
                "LOAD-CYCLE acc reads itself both earlier and later on m",
            ),
            // not-recurrence
            (
                &[(
                    "def fee: money PLN = amt at day",
                    "def fee: money PLN = amt at day\n\ndef lag[m in months]: num = acc[m - 1]",
                )],
                "loaded",
            ),
            // mutual
            (
                &[
                    (
                        "def fee: money PLN = amt at day",
                        "def fee: money PLN = amt at day\n\ndef lag[m in months]: num = acc[m - 1]",
                    ),
                    (
                        "  rule step beats base = acc[m - 1] + rate[q2] when pos(m) > 1",
                        "  rule step beats base = lag[m] + rate[q2] when pos(m) > 1",
                    ),
                ],
                "LOAD-GUARD acc[m - n] is a recurrence read with no condition on m or pos(m) guarding it (a.tally:40)",
            ),
            // dated-move
            (
                &[(
                    "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"",
                    "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"\n\nversion card_b\n  cite card\n  governs 2026-06..\n  known 2026-02-01\n  replace amt = 200.00 PLN \"a raise\"",
                )],
                "LOAD-LAWDATE pay reads law-dated facts and states no at (a.tally:42)",
            ),
            // dated-move-at
            (
                &[
                    (
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"",
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"\n\nversion card_b\n  cite card\n  governs 2026-06..\n  known 2026-02-01\n  replace amt = 200.00 PLN \"a raise\"",
                    ),
                    ("  on start_of(m)", "  on start_of(m)\n  at start_of(m)"),
                    (
                        "  on schedule(m, 1, unadjusted, cal)",
                        "  on schedule(m, 1, unadjusted, cal)\n  at start_of(m)",
                    ),
                ],
                "loaded",
            ),
            // dated-def
            (
                &[
                    (
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"",
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"\n\nversion card_b\n  cite card\n  governs 2026-06..\n  known 2026-02-01\n  replace amt = 200.00 PLN \"a raise\"",
                    ),
                    (
                        "def fee: money PLN = amt at day",
                        "def fee: money PLN = amt",
                    ),
                    (
                        "  leg cash from world to konto amount amt",
                        "  leg cash from world to konto amount fee",
                    ),
                    (
                        "  leg cash from konto to world amount amt",
                        "  leg cash from konto to world amount fee",
                    ),
                ],
                "LOAD-LAWDATE fee reads law-dated facts and states no at (a.tally:38)",
            ),
            // reader
            (
                &[(
                    "def fee: money PLN = amt at day",
                    "def fee: money PLN = amt at reader",
                )],
                "LOAD-LAWDATE ask q reaches an at reader def without law (a.tally:55)",
            ),
            // reader-law
            (
                &[
                    (
                        "def fee: money PLN = amt at day",
                        "def fee: money PLN = amt at reader",
                    ),
                    (
                        "ask q: fee, levy, acc[2026-02], bal(konto, 2026-12-31)",
                        "ask q: fee, levy, acc[2026-02], bal(konto, 2026-12-31) law 2026-02-01",
                    ),
                ],
                "loaded",
            ),
            // at-reader
            (
                &[(
                    "def fee: money PLN = amt at day",
                    "def fee: money PLN = amt at stamp\n\ndef stamp: date = day at reader",
                )],
                "LOAD-LAWDATE ask q reaches fee, whose at reads law-dated facts at its reader's law date, without law (L §4.3) (a.tally:57)",
            ),
            // at-reader-law
            (
                &[
                    (
                        "def fee: money PLN = amt at day",
                        "def fee: money PLN = amt at stamp\n\ndef stamp: date = day at reader",
                    ),
                    (
                        "ask q: fee, levy, acc[2026-02], bal(konto, 2026-12-31)",
                        "ask q: fee, levy, acc[2026-02], bal(konto, 2026-12-31) law 2026-02-01",
                    ),
                ],
                "loaded",
            ),
            // direct-input
            (
                &[
                    (
                        "input cal: calendar",
                        "input cal: calendar\n\ninput spare: num",
                    ),
                    (
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"",
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"\n  spare = 1 governs 2026-01..2026-06 \"the card\"\n  spare = -1 governs 2026-07..2026-12 \"the card\"",
                    ),
                    (
                        "ask q: fee, levy, acc[2026-02], bal(konto, 2026-12-31)",
                        "ask q: fee, levy, acc[2026-02], bal(konto, 2026-12-31), spare",
                    ),
                ],
                "LOAD-LAWDATE ask q reads law-dated spare directly, without law (a.tally:57)",
            ),
            // direct-input-law
            (
                &[
                    (
                        "input cal: calendar",
                        "input cal: calendar\n\ninput spare: num",
                    ),
                    (
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"",
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"\n  spare = 1 governs 2026-01..2026-06 \"the card\"\n  spare = -1 governs 2026-07..2026-12 \"the card\"",
                    ),
                    (
                        "ask q: fee, levy, acc[2026-02], bal(konto, 2026-12-31)",
                        "ask q: fee, levy, acc[2026-02], bal(konto, 2026-12-31), spare law 2026-02-01",
                    ),
                ],
                "loaded",
            ),
            // direct-band
            (
                &[
                    ("  tax: [0 PLN, inf) = 0 \"the card\"", ""),
                    (
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"",
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"\n\nversion bands\n  cite card\n  governs 2026-01..2026-12\n  known 2026-01-01\n  tax: [0 PLN, inf) = 0 \"the card\"",
                    ),
                    (
                        "ask q: fee, levy, acc[2026-02], bal(konto, 2026-12-31)",
                        "ask q: fee, levy, acc[2026-02], bal(konto, 2026-12-31), tax(amt)",
                    ),
                ],
                "LOAD-LAWDATE ask q reads law-dated tax directly, without law (a.tally:55)",
            ),
            // band-def
            (
                &[
                    ("  tax: [0 PLN, inf) = 0 \"the card\"", ""),
                    (
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"",
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"\n\nversion bands\n  cite card\n  governs 2026-01..2026-12\n  known 2026-01-01\n  tax: [0 PLN, inf) = 0 \"the card\"",
                    ),
                    (
                        "def levy: num = tax(amt) at day",
                        "def levy: num = tax(amt)",
                    ),
                ],
                "LOAD-LAWDATE levy reads law-dated facts and states no at (a.tally:40)",
            ),
            // def-no-at
            (
                &[
                    (
                        "input cal: calendar",
                        "input cal: calendar\n\ninput spare: num",
                    ),
                    (
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"",
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"\n  spare = 1 governs 2026-01..2026-06 \"the card\"\n  spare = -1 governs 2026-07..2026-12 \"the card\"",
                    ),
                    (
                        "def fee: money PLN = amt at day",
                        "def fee: money PLN = amt at day\n\ndef lawd: num = spare",
                    ),
                ],
                "LOAD-LAWDATE lawd reads law-dated facts and states no at (a.tally:42)",
            ),
            // move-at-reader
            (
                &[
                    ("  on start_of(m)", "  on start_of(m)\n  at stamp"),
                    (
                        "def fee: money PLN = amt at day",
                        "def fee: money PLN = amt at day\n\ndef stamp: date = day at reader",
                    ),
                ],
                "LOAD-LAWDATE pay: its at reads stamp, which takes its law date from its reader (a.tally:44)",
            ),
            // move-at-input
            (
                &[
                    (
                        "input cal: calendar",
                        "input cal: calendar\n\ninput spare: num",
                    ),
                    (
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"",
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"\n  spare = 1 governs 2026-01..2026-06 \"the card\"\n  spare = -1 governs 2026-07..2026-12 \"the card\"",
                    ),
                    (
                        "  on start_of(m)",
                        "  on start_of(m)\n  at if spare > 0 then start_of(m) else end_of(m)",
                    ),
                ],
                "LOAD-LAWDATE move pay's at reads law-dated spare (a.tally:44)",
            ),
            // move-at-band
            (
                &[
                    ("  tax: [0 PLN, inf) = 0 \"the card\"", ""),
                    (
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"",
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"\n\nversion bands\n  cite card\n  governs 2026-01..2026-12\n  known 2026-01-01\n  tax: [0 PLN, inf) = 0 \"the card\"",
                    ),
                    (
                        "  on start_of(m)",
                        "  on start_of(m)\n  at if tax(amt) > 0 then start_of(m) else end_of(m)",
                    ),
                ],
                "LOAD-LAWDATE move pay's at reads law-dated tax (a.tally:42)",
            ),
            // plan-reads
            (
                &[(
                    "  leg cash from world to konto amount amt",
                    "  leg cash from world to konto amount bal(konto, start_of(m))",
                )],
                "LOAD-READS pay reaches a running read or projection of konto without reads konto (a.tally:42)",
            ),
            // plan-reads-def
            (
                &[
                    (
                        "def fee: money PLN = amt at day",
                        "def fee: money PLN = amt at day\n\ndef seen: money PLN = bal(safe, day)",
                    ),
                    (
                        "  leg cash from world to konto amount amt",
                        "  leg cash from world to konto amount seen",
                    ),
                ],
                "LOAD-READS pay reaches a running read or projection of safe without reads safe (a.tally:44)",
            ),
            // day-reads
            (
                &[(
                    "  on end_of(m)",
                    "  on if bal(konto, day) > 0 PLN then end_of(m) else start_of(m)",
                )],
                "LOAD-READS sweep: its day reads running state (konto); a decision's day reads no running state (a.tally:50)",
            ),
            // undeclared
            (
                &[(
                    "  leg cash from konto to safe amount bal(konto, end_of(m) - 1 day)",
                    "  leg cash from konto to safe amount bal(safe, end_of(m) - 1 day)",
                )],
                "LOAD-READS sweep reaches a running read or projection of safe without reads safe (a.tally:50)",
            ),
            // never-read
            (
                &[(
                    "  leg cash from konto to safe amount bal(konto, end_of(m) - 1 day)",
                    "  leg cash from konto to safe amount amt",
                )],
                "LOAD-READS sweep declares reads konto and never reads it (a.tally:50)",
            ),
            // balnow-def
            (
                &[
                    (
                        "def fee: money PLN = amt at day",
                        "def fee: money PLN = amt at day\n\ndef now: money PLN = bal(konto)",
                    ),
                    (
                        "ask q: fee, levy, acc[2026-02], bal(konto, 2026-12-31)",
                        "ask q: fee, levy, acc[2026-02], bal(konto, 2026-12-31), now",
                    ),
                ],
                "LOAD-READS now holds bal(x) with no day: a running read outside a decision and its defs",
            ),
            // balnow-decision
            (
                &[
                    (
                        "def fee: money PLN = amt at day",
                        "def fee: money PLN = amt at day\n\ndef now: money PLN = bal(konto)",
                    ),
                    (
                        "  leg cash from konto to safe amount bal(konto, end_of(m) - 1 day)",
                        "  leg cash from konto to safe amount now",
                    ),
                ],
                "loaded",
            ),
            // after-plan
            (
                &[
                    (
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"",
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"\n  stated[konto] = 400.00 PLN after pay[2026-02] \"the card\"",
                    ),
                    ("  known 2026-01-01", "  known 2026-01-01\n  anchor konto"),
                ],
                "LOAD-ORDER stated[konto] after pay[2026-02]: pay[2026-02] is a plan or an observation (a.tally:70)",
            ),
            // after-decision
            (
                &[
                    (
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"",
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"\n  stated[konto] = 400.00 PLN after sweep[2026-02] \"the card\"",
                    ),
                    ("  known 2026-01-01", "  known 2026-01-01\n  anchor konto"),
                ],
                "loaded",
            ),
            // after-obs
            (
                &[
                    (
                        "input cal: calendar",
                        "input cal: calendar\n\ninput bank: observed",
                    ),
                    (
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"",
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"\n  opening[safe] = 0 PLN after bank \"the card\"",
                    ),
                ],
                "LOAD-ORDER opening[safe] after bank: bank is a plan or an observation (a.tally:71)",
            ),
            // seen-own
            (
                &[(
                    "  leg cash from konto to safe amount bal(konto, end_of(m) - 1 day)",
                    "  leg cash from konto to safe amount bal(konto, add_days(end_of(m), 0))",
                )],
                "LOAD-CYCLE sweep[2026-02] on 2026-03-01 reads bal(konto, add_days(end_of(m), 0)) = 2026-03-01, its own day: its own result or its future (write the projection) (a.tally:50)",
            ),
            // seen-later
            (
                &[(
                    "  leg cash from konto to safe amount bal(konto, end_of(m) - 1 day)",
                    "  leg cash from konto to safe amount bal(konto, add_days(end_of(m), 1))",
                )],
                "LOAD-CYCLE sweep[2026-02] on 2026-03-01 reads bal(konto, add_days(end_of(m), 1)) = 2026-03-02, a later day: its own result or its future (write the projection) (a.tally:50)",
            ),
            // seen-start
            (
                &[(
                    "  leg cash from konto to safe amount bal(konto, end_of(m) - 1 day)",
                    "  leg cash from konto to safe amount bal(konto, add_days(end_of(m), 0) start)",
                )],
                "loaded",
            ),
            // seen-earlier
            (
                &[(
                    "  leg cash from konto to safe amount bal(konto, end_of(m) - 1 day)",
                    "  leg cash from konto to safe amount bal(konto, add_days(end_of(m), -1))",
                )],
                "loaded",
            ),
            // unseen
            (
                &[(
                    "  leg cash from konto to safe amount bal(konto, end_of(m) - 1 day)",
                    "  leg cash from konto to safe amount bal(konto, day)",
                )],
                "loaded",
            ),
            // schedule-none
            (
                &[(
                    "  on schedule(m, 1, unadjusted, cal)",
                    "  on schedule(m, 0, unadjusted, cal)",
                )],
                "LOAD-SCHEDULE schedule schedule(m, 0, unadjusted, cal) (the day of rent) gives no date in the run (a.tally:46)",
            ),
            // schedule-paren
            (
                &[(
                    "  on schedule(m, 1, unadjusted, cal)",
                    "  on (schedule(m, 0, unadjusted, cal))",
                )],
                "LOAD-SCHEDULE schedule schedule(m, 0, unadjusted, cal) (the day of rent) gives no date in the run (a.tally:46)",
            ),
            // schedule-rolled
            (
                &[(
                    "  on schedule(m, 1, unadjusted, cal)",
                    "  on schedule(m, 0, following, cal)",
                )],
                "LOAD-SCHEDULE schedule schedule(m, 0, following, cal) (the day of rent) gives no date in the run (a.tally:46)",
            ),
            // schedule-some
            (
                &[
                    (
                        "index gm = month 2026-02..2026-03",
                        "index gm = month 2026-02..2026-03\n\nindex dd = int 30..31",
                    ),
                    (
                        "move rent[m in gm]: transfers",
                        "move rent[m in gm, d in dd]: transfers",
                    ),
                    (
                        "  on schedule(m, 1, unadjusted, cal)",
                        "  on schedule(m, d, unadjusted, cal)",
                    ),
                ],
                "loaded",
            ),
            // schedule-every-none
            (
                &[
                    (
                        "index gm = month 2026-02..2026-03",
                        "index gm = month 2026-02..2026-03\n\nindex dd = int 30..31\n\nindex fm = month 2026-02..2026-02",
                    ),
                    (
                        "move rent[m in gm]: transfers",
                        "move rent[m in fm, d in dd]: transfers",
                    ),
                    (
                        "  on schedule(m, 1, unadjusted, cal)",
                        "  on schedule(m, d, unadjusted, cal)",
                    ),
                ],
                "LOAD-SCHEDULE schedule schedule(m, d, unadjusted, cal) (the day of rent) gives no date in the run (a.tally:50)",
            ),
            // schedule-unseen
            (
                &[
                    (
                        "input cal: calendar",
                        "input cal: calendar\n\ninput rd: int",
                    ),
                    (
                        "  on schedule(m, 1, unadjusted, cal)",
                        "  on schedule(m, rd, unadjusted, cal)",
                    ),
                    (
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"",
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"\n  rd = 0 \"the card\"",
                    ),
                ],
                "loaded",
            ),
            // schedule-anchored
            (
                &[
                    (
                        "index gm = month 2026-02..2026-03",
                        "index gm = month 2026-02..2026-03\n\nindex am = month 2026-02..2026-03 from 15",
                    ),
                    (
                        "move rent[m in gm]: transfers",
                        "move rent[m in am]: transfers",
                    ),
                ],
                "LOAD-DOMAIN schedule of a key of the anchored index am has no meaning (a.tally:49)",
            ),
            // schedule-year
            (
                &[
                    (
                        "index gm = month 2026-02..2026-03",
                        "index gm = month 2026-02..2026-03\n\nindex ys = year 2026..2026",
                    ),
                    (
                        "move rent[m in gm]: transfers",
                        "move rent[m in ys]: transfers",
                    ),
                ],
                "LOAD-UNIT schedule takes a month, not key ys (a.tally:49)",
            ),
            // schedule-roll
            (
                &[(
                    "  on schedule(m, 1, unadjusted, cal)",
                    "  on schedule(m, 1, half-up, cal)",
                )],
                "LOAD-UNIT half-up is not a roll code (a.tally:47)",
            ),
            // schedule-calendar
            (
                &[(
                    "  on schedule(m, 1, unadjusted, cal)",
                    "  on schedule(m, 1, unadjusted, day)",
                )],
                "LOAD-UNIT schedule: the calendar is date (a.tally:47)",
            ),
            // schedule-arity
            (
                &[(
                    "  on schedule(m, 1, unadjusted, cal)",
                    "  on schedule(m, 1, unadjusted)",
                )],
                "LOAD-PARSE cannot parse: schedule takes 4 arguments (a.tally:47)",
            ),
            // day-of-anchored
            (
                &[
                    (
                        "index gm = month 2026-02..2026-03",
                        "index gm = month 2026-02..2026-03\n\nindex am = month 2026-02..2026-03 from 15",
                    ),
                    ("move pay[m in gm]: wages", "move pay[m in am]: wages"),
                    ("  on start_of(m)", "  on day_of(m, 1)"),
                ],
                "LOAD-DOMAIN day_of of a key of the anchored index am has no meaning (a.tally:45)",
            ),
            // order-guard-lawdate
            (
                &[
                    (
                        "  rule step beats base = acc[m - 1] + rate[q2] when pos(m) > 1",
                        "  rule step beats base = acc[m - 1] + rate[q2]",
                    ),
                    (
                        "input cal: calendar",
                        "input cal: calendar\n\ninput spare: num",
                    ),
                    (
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"",
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"\n  spare = 1 governs 2026-01..2026-06 \"the card\"\n  spare = -1 governs 2026-07..2026-12 \"the card\"",
                    ),
                    (
                        "def fee: money PLN = amt at day",
                        "def fee: money PLN = amt at day\n\ndef lawd: num = spare",
                    ),
                ],
                "LOAD-GUARD acc[m - n] is a recurrence read with no condition on m or pos(m) guarding it (a.tally:38)",
            ),
            // order-reads-lawdate
            (
                &[
                    (
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"",
                        "  opening[safe] = 0 PLN at 2026-01-01 start \"the card\"\n\nversion card_b\n  cite card\n  governs 2026-06..\n  known 2026-02-01\n  replace amt = 200.00 PLN \"a raise\"",
                    ),
                    (
                        "  leg cash from world to konto amount amt",
                        "  leg cash from world to konto amount bal(konto, start_of(m))",
                    ),
                ],
                "LOAD-READS pay reaches a running read or projection of konto without reads konto (a.tally:42)",
            ),
        ];
        for &(edits, expected) in cases {
            let mut text = GRAPH.to_owned();
            for &(old, new) in edits {
                assert!(text.contains(old), "{old} is in the text");
                text = text.replacen(old, new, 1);
            }
            assert_eq!(refusal(&text), expected, "{edits:?}");
        }
    }

    /// Two axes, one a rounding mode; a fill and a def reading its one output; a check, an
    /// explain and a readers-of ask; an ask pinning an axis option, an input, and an `as if`.
    const ASKS: &str = "model m
  tally 2
  on 2026-03-01
  start 2026-01-01
  end 2026-12-31

source card \"the card\"

unit PLN: currency minor 0.01

index qs = [q1, q2]

index ys = year 2026..2027

axis path_ax
  low \"a low path\"
  high \"a high path\"

axis rnd
  half-up \"half up\"
  down \"down\"

input amt: money PLN

input rate[q in qs]: num

input grow[a in path_ax]: num

input cap[q in qs]: money PLN

input need[q in qs]: money PLN

place konto: accounts PLN lives 2026-01-01..

fill share[y in ys]
  supply cap[s] for s in qs
  demand need[d] for d in qs
  arc s == d
  order pos(s), pos(d)

def total: money PLN
  rule base = amt

def scaled: num = rate[q1] * grow[path_ax]

def rounded: money PLN = round(amt, 1 PLN, rnd)

def taken[y in ys]: money PLN = share[y].taken[q1, q2]

version v1
  cite card
  governs always
  known 2026-01-01
  amt = 100.00
  rate[q1] = 2
  rate[q2] = 3
  grow[low] = 1
  grow[high] = 2
  cap[q1] = 10.00
  cap[q2] = 20.00
  need[q1] = 5.00
  need[q2] = 25.00
  opening[konto] = 0 PLN at 2026-01-01 start

ask qv: total, scaled with path_ax = low

ask qc: check total == amt

ask qe: explain total

ask qr: readers-of amt

ask qp: total with amt = 50

ask qa: total as if {amt = 20} law 2026-02-01

ask qt: taken[2026], rounded
";

    /// The first line a run of `text` prints for the ask its expected line `line` names, or
    /// `None` where the engine doesn't build what the run needs.
    fn ask_line(text: &str, line: &str) -> Option<String> {
        let (ask, _) = line.split_once(':')?;
        let given = vec![tally::Given {
            path: "a.tally".to_owned(),
            bytes: Some(text.as_bytes().to_vec()),
        }];
        let request = tally::Request {
            asks: tally::Asks::Named(vec![ask.to_owned()]),
            pin: tally::Pin::Canonical,
            days: tally::Days::Events,
        };
        match tally::run(given, &request) {
            Ok(answer) => answer.lines().first().cloned(),
            Err(_) => None,
        }
    }

    /// The base `base` with each edit applied once.
    fn edited(base: &str, edits: &[(&str, &str)]) -> String {
        let mut text = base.to_owned();
        for &(old, new) in edits {
            assert!(text.contains(old), "{old} is in the text");
            text = text.replacen(old, new, 1);
        }
        text
    }

    #[test]
    fn axes_fills_pins_and_ask_forms_are_checked_at_load_as_refcore_checks_them() {
        // The load probe's refusal ("loaded" when it loads clean), refcore's text.
        let probes: &[(&[(&str, &str)], &str)] = &[
            // base
            (&[], "loaded"),
            // axis-dup
            (
                &[(
                    "axis path_ax\n  low \"a low path\"\n  high \"a high path\"",
                    "axis path_ax\n  low \"a low path\"\n  low \"again\"",
                )],
                "LOAD-DUP a key entered twice in axis path_ax (a.tally:15)",
            ),
            // axis-key-id
            (
                &[(
                    "axis path_ax\n  low \"a low path\"\n  high \"a high path\"",
                    "axis path_ax\n  low \"a low path\"\n  high \"a high path\"\n\ninput low: num",
                )],
                "LOAD-KEY a key is also an id: low (an option of path_ax)",
            ),
            // axis-key
            (
                &[(
                    "def scaled: num = rate[q1] * grow[path_ax]",
                    "def scaled: num = rate[q1] * grow[high]",
                )],
                "loaded",
            ),
            // axis-key-unknown
            (
                &[(
                    "def scaled: num = rate[q1] * grow[path_ax]",
                    "def scaled: num = rate[q1] * grow[middle]",
                )],
                "LOAD-KEY middle is not a key of path_ax (a.tally:44)",
            ),
            // axis-other
            (
                &[(
                    "def scaled: num = rate[q1] * grow[path_ax]",
                    "def scaled: num = rate[q1] * grow[rnd]",
                )],
                "LOAD-KEY rnd is used as a key of path_ax (a.tally:44)",
            ),
            // axis-value
            (
                &[(
                    "def scaled: num = rate[q1] * grow[path_ax]",
                    "def scaled: bool = path_ax == low",
                )],
                "loaded",
            ),
            // axis-mode
            (
                &[
                    (
                        "axis path_ax\n  low \"a low path\"\n  high \"a high path\"",
                        "axis path_ax\n  low \"a low path\"\n  high \"a high path\"\n\naxis odd\n  half-up \"h\"\n  exact \"e\"",
                    ),
                    (
                        "def rounded: money PLN = round(amt, 1 PLN, rnd)",
                        "def rounded: money PLN = round(amt, 1 PLN, odd)",
                    ),
                ],
                "LOAD-UNIT axis odd is used as a mode, but exact is not a mode name (a.tally:50)",
            ),
            // axis-bind
            (
                &[(
                    "ask qv: total, scaled with path_ax = low",
                    "ask qv: total, scaled with path_ax = low\n\nask qb: sum(rate[q1] for a in path_ax)",
                )],
                "LOAD-DOMAIN a binding over the axis path_ax: a bind is over an index, a kind, a key list or a stepped range (L §5.6) (a.tally:67)",
            ),
            // forks
            (
                &[
                    (
                        "ask qv: total, scaled with path_ax = low",
                        "ask qv: total, scaled",
                    ),
                    (
                        "axis path_ax\n  low \"a low path\"\n  high \"a high path\"",
                        "axis path_ax\n  low \"a low path\"\n  high \"a high path\"\n\naxis ax0\n  a0 \"a\"\n  b0 \"b\"\n\naxis ax1\n  a1 \"a\"\n  b1 \"b\"\n\naxis ax2\n  a2 \"a\"\n  b2 \"b\"\n\naxis ax3\n  a3 \"a\"\n  b3 \"b\"\n\naxis ax4\n  a4 \"a\"\n  b4 \"b\"\n\naxis ax5\n  a5 \"a\"\n  b5 \"b\"",
                    ),
                    (
                        "def scaled: num = rate[q1] * grow[path_ax]",
                        "def scaled: num = rate[q1] * grow[path_ax] + (if ax0 == a0 then 0 else 0) + (if ax1 == a1 then 0 else 0) + (if ax2 == a2 then 0 else 0) + (if ax3 == a3 then 0 else 0) + (if ax4 == a4 then 0 else 0) + (if ax5 == a5 then 0 else 0)",
                    ),
                ],
                "LOAD-FORKS ask qv reaches 128 axis combinations; the cap is 64 (a.tally:89)",
            ),
            // forks-pinned
            (
                &[
                    (
                        "ask qv: total, scaled with path_ax = low",
                        "ask qv: total, scaled with ax0 = a0",
                    ),
                    (
                        "axis path_ax\n  low \"a low path\"\n  high \"a high path\"",
                        "axis path_ax\n  low \"a low path\"\n  high \"a high path\"\n\naxis ax0\n  a0 \"a\"\n  b0 \"b\"\n\naxis ax1\n  a1 \"a\"\n  b1 \"b\"\n\naxis ax2\n  a2 \"a\"\n  b2 \"b\"\n\naxis ax3\n  a3 \"a\"\n  b3 \"b\"\n\naxis ax4\n  a4 \"a\"\n  b4 \"b\"\n\naxis ax5\n  a5 \"a\"\n  b5 \"b\"",
                    ),
                    (
                        "def scaled: num = rate[q1] * grow[path_ax]",
                        "def scaled: num = rate[q1] * grow[path_ax] + (if ax0 == a0 then 0 else 0) + (if ax1 == a1 then 0 else 0) + (if ax2 == a2 then 0 else 0) + (if ax3 == a3 then 0 else 0) + (if ax4 == a4 then 0 else 0) + (if ax5 == a5 then 0 else 0)",
                    ),
                ],
                "loaded",
            ),
            // pin-unknown
            (
                &[(
                    "ask qp: total with amt = 50",
                    "ask qp: total with nope = 50",
                )],
                "LOAD-ASIF pin on unknown id nope (a.tally:73)",
            ),
            // pin-option
            (
                &[(
                    "ask qv: total, scaled with path_ax = low",
                    "ask qv: total, scaled with path_ax = middle",
                )],
                "LOAD-KEY middle is not a key of path_ax (a.tally:65)",
            ),
            // pin-axis-keys
            (
                &[(
                    "ask qv: total, scaled with path_ax = low",
                    "ask qv: total, scaled with path_ax[q1] = low",
                )],
                "LOAD-ASIF an axis pin names its option (a.tally:65)",
            ),
            // pin-place
            (
                &[(
                    "ask qp: total with amt = 50",
                    "ask qp: total with konto = 50",
                )],
                "LOAD-ASIF pin on konto, which is not an input, a def or an axis (a.tally:73)",
            ),
            // pin-keys
            (
                &[(
                    "ask qp: total with amt = 50",
                    "ask qp: total with rate = 50",
                )],
                "LOAD-KEY wrong number of keys for rate: write [*] to pin every key (a.tally:73)",
            ),
            // pin-keys-count
            (
                &[(
                    "ask qp: total with amt = 50",
                    "ask qp: total with rate[q1, q2] = 50",
                )],
                "LOAD-KEY wrong number of keys for rate (a.tally:73)",
            ),
            // pin-star
            (
                &[(
                    "ask qp: total with amt = 50",
                    "ask qp: total with rate[*] = 50",
                )],
                "loaded",
            ),
            // pin-field
            (
                &[(
                    "ask qp: total with amt = 50",
                    "ask qp: total with amt.x = 50",
                )],
                "LOAD-FIELD x is not a field of money PLN (a.tally:73)",
            ),
            // pin-precision
            (
                &[(
                    "ask qp: total with amt = 50",
                    "ask qp: total with amt = 50.001",
                )],
                "LOAD-PRECISION 50.001 is finer than the minor unit (a.tally:73)",
            ),
            // pin-fraction
            (
                &[(
                    "ask qp: total with amt = 50",
                    "ask qp: total with amt = 1/2",
                )],
                "LOAD-PARSE cannot parse: a fraction is allowed only in rows (a.tally:73)",
            ),
            // pin-law
            (
                &[("ask qp: total with amt = 50", "ask qp: total with law 5")],
                "LOAD-UNIT pin law takes a date, not int (a.tally:73)",
            ),
            // pin-wording
            (
                &[(
                    "ask qp: total with amt = 50",
                    "ask qp: total with wording 2026-02-01 for [nope]",
                )],
                "LOAD-ASIF pin on unknown id nope (a.tally:73)",
            ),
            // pin-literal-rule
            (
                &[(
                    "ask qa: total as if {amt = 20} law 2026-02-01",
                    "ask qa: total as if {amt = 20} law 2026-02-01\n\ndef twice: money PLN = total as if {amt = 20}",
                )],
                "LOAD-LITERAL figure 20 in a rule; make it a row (a.tally:77)",
            ),
            // asif-unknown
            (
                &[(
                    "ask qa: total as if {amt = 20} law 2026-02-01",
                    "ask qa: total as if {nope = 20} law 2026-02-01",
                )],
                "LOAD-ASIF pin on unknown id nope (a.tally:75)",
            ),
            // asif-scenario
            (
                &[(
                    "ask qa: total as if {amt = 20} law 2026-02-01",
                    "ask qa: total as if {amt = 20} law 2026-02-01\n\ndef twice: money PLN = total as if {scenario lowered}",
                )],
                "LOAD-ASIF pin on unknown id lowered (a.tally:77)",
            ),
            // check-unit
            (
                &[(
                    "ask qc: check total == amt",
                    "ask qc: check total == rate[q1]",
                )],
                "LOAD-UNIT money PLN and num have different units (a.tally:67)",
            ),
            // explain-ref
            (
                &[("ask qe: explain total", "ask qe: explain total + amt")],
                "LOAD-PARSE cannot parse: expected end of line, got + (a.tally:69)",
            ),
            // inputs-of-bal
            (
                &[(
                    "ask qe: explain total",
                    "ask qe: inputs-of bal(konto, 2026-02-01)",
                )],
                "LOAD-PARSE cannot parse: expected end of line, got ( (a.tally:69)",
            ),
            // readers-of-unknown
            (
                &[("ask qr: readers-of amt", "ask qr: readers-of nope")],
                "LOAD-NAME unknown id nope (a.tally:71)",
            ),
            // readers-of-key
            (
                &[("ask qr: readers-of amt", "ask qr: readers-of rate[q1]")],
                "LOAD-PARSE cannot parse: readers-of takes an id (a.tally:71)",
            ),
            // quantile-level
            (
                &[(
                    "ask qr: readers-of amt",
                    "ask qr: total random at quantile(0.9)",
                )],
                "loaded",
            ),
            // fill-name
            (
                &[(
                    "def taken[y in ys]: money PLN = share[y].taken[q1, q2]",
                    "def taken[y in ys]: money PLN = share",
                )],
                "LOAD-PARSE cannot parse: fill share is read as share.taken[...] (a.tally:48)",
            ),
            // fill-keys
            (
                &[(
                    "def taken[y in ys]: money PLN = share[y].taken[q1, q2]",
                    "def taken[y in ys]: money PLN = share[y]",
                )],
                "LOAD-PARSE cannot parse: fill share is not read with keys (a.tally:48)",
            ),
            // fill-count
            (
                &[(
                    "def taken[y in ys]: money PLN = share[y].taken[q1, q2]",
                    "def taken[y in ys]: money PLN = share[y].taken[q1]",
                )],
                "LOAD-KEY wrong number of keys for share.taken: 1, not 2 (supply keys, then demand keys) (a.tally:48)",
            ),
            // fill-own-keys
            (
                &[(
                    "def taken[y in ys]: money PLN = share[y].taken[q1, q2]",
                    "def taken[y in ys]: money PLN = share.taken[q1, q2]",
                )],
                "LOAD-KEY wrong number of keys for fill share (a.tally:48)",
            ),
            // fill-field
            (
                &[(
                    "def taken[y in ys]: money PLN = share[y].taken[q1, q2]",
                    "def taken[y in ys]: money PLN = share[y].taken[q1, q2].x",
                )],
                "LOAD-FIELD x is not a field of money PLN (a.tally:48)",
            ),
            // fill-units
            (
                &[(
                    "  supply cap[s] for s in qs\n  demand need[d] for d in qs\n  arc s == d\n  order pos(s), pos(d)",
                    "  supply cap[s] for s in qs\n  demand rate[d] for d in qs\n  arc s == d\n  order pos(s), pos(d)",
                )],
                "LOAD-UNIT fill share: supply (money PLN) and demand (num) must share a unit (L §5.12) (a.tally:35)",
            ),
            // fill-supply
            (
                &[(
                    "  supply cap[s] for s in qs\n  demand need[d] for d in qs\n  arc s == d\n  order pos(s), pos(d)",
                    "  supply s == q1 for s in qs\n  demand need[d] for d in qs\n  arc s == d\n  order pos(s), pos(d)",
                )],
                "LOAD-UNIT fill share: the supply is bool (a.tally:35)",
            ),
            // fill-both
            (
                &[(
                    "  supply cap[s] for s in qs\n  demand need[d] for d in qs\n  arc s == d\n  order pos(s), pos(d)",
                    "  supply cap[s] for s in qs\n  demand need[d] for s in qs\n  arc s == d\n  order pos(s), pos(d)",
                )],
                "LOAD-KEY d is not a key of qs (a.tally:37)",
            ),
            // fill-arc
            (
                &[(
                    "  supply cap[s] for s in qs\n  demand need[d] for d in qs\n  arc s == d\n  order pos(s), pos(d)",
                    "  supply cap[s] for s in qs\n  demand need[d] for d in qs\n  arc cap[s]\n  order pos(s), pos(d)",
                )],
                "LOAD-UNIT fill share: arc is money PLN, not bool (a.tally:35)",
            ),
            // fill-key-bind
            (
                &[
                    (
                        "  supply cap[s] for s in qs\n  demand need[d] for d in qs\n  arc s == d\n  order pos(s), pos(d)",
                        "  supply cap[s] for s in qs\n  demand need[d] for d in qs\n  arc s == d\n  order pos(s), pos(d)",
                    ),
                    ("fill share[y in ys]", "fill share[k in konto]"),
                ],
                "LOAD-KEY konto is not an index, a kind, a key list or a range (a.tally:35)",
            ),
        ];
        for &(edits, expected) in probes {
            assert_eq!(refusal(&edited(ASKS, edits)), expected, "{edits:?}");
        }
        // An ask refused alone: the run's first line for the ask it names, refcore's text.
        let runs: &[(&[(&str, &str)], &str)] = &[
            // pin-scenario
            (
                &[(
                    "ask qp: total with amt = 50",
                    "ask qp: total with scenario lowered",
                )],
                "qp: refused: LOAD-SCENARIO ask qp names lowered, neither the model nor a loaded scenario (a.tally:73)",
            ),
            // ask-world
            (
                &[("ask qc: check total == amt", "ask qc: total in lowered")],
                "qc: refused: LOAD-SCENARIO ask qc names lowered, neither the model nor a loaded scenario (a.tally:67)",
            ),
            // diff-world
            (
                &[(
                    "ask qr: readers-of amt",
                    "ask qr: diff(total) in lowered against m",
                )],
                "qr: refused: LOAD-SCENARIO ask qr names lowered, neither the model nor a loaded scenario (a.tally:71)",
            ),
        ];
        for &(edits, expected) in runs {
            let line = ask_line(&edited(ASKS, edits), expected);
            assert_eq!(line.as_deref(), Some(expected), "{edits:?}");
        }
    }

    /// Random facts of each kind (a continuous `dist PLN`, a counting `dist`), a def lifting
    /// one, a `discrete`, the transcendental calls under `to u`, a `prorata`, a `solve` in a
    /// rule with `v`, and an ask of each query form: closed forms, paths with a seed, `orient`.
    const RANDOM: &str = "model r
  tally 2
  on 2026-03-01
  start 2026-01-01
  end 2026-12-31

source card \"the card\"

unit PLN: currency minor 0.01

index qs = [q1, q2]

input amt: money PLN

input rate: rate

input nyears: int

input mu: num

input u4: dist PLN

input k4: dist

input step_in: money PLN

def lifted: num PLN = amt + u4

def picked: dist = discrete(1 for q in qs)

def grown: num = exp(mu) to 0.0001

def compounded: num = factor(rate, nyears, year, effective)

def share_of[q in qs]: money PLN = prorata(amt, 1 for p in qs, q, 1 PLN, half-up, rest last)

def target: money PLN
  rule base v card_2026 = solve amt == lifted for step_in in [0 PLN, 1000 PLN] to 1 PLN

version card_2026
  cite card
  governs always
  known 2026-01-01
  amt = 100.00 PLN \"the card\"
  rate = 5 %/year \"the card\"
  nyears = 3 \"the card\"
  mu = 1 \"the card\"
  u4 = normal(10 PLN, 2 PLN) to 0.01 PLN \"the card\"
  k4 = poisson(2) \"the card\"
  step_in = 1.00 PLN \"the card\"

ask qm: mean(mu * u4 + amt)

ask qsh: share(u4 > amt)

ask qq: quantile(u4, 0.9)

ask qc: cdf(normal(0, 1), 1) to 0.0001

ask qk: pmf(k4, nyears)

ask qo: crps(u4, amt) paths 100 seed 1 orient lower

ask qp: mean(u4 * u4) paths 100 seed 1

ask qv: solve amt == step_in for step_in in [0 PLN, 500 PLN] to 1 PLN
";

    #[test]
    fn random_values_exact_forms_and_scenario_rows_are_checked_at_load_as_refcore_checks_them() {
        // The load probe's refusal ("loaded" when it loads clean), refcore's text.
        let probes: &[(&[(&str, &str)], &str)] = &[
            // base
            (&[], "loaded"),
            // dist-arity
            (
                &[(
                    "  u4 = normal(10 PLN, 2 PLN) to 0.01 PLN \"the card\"",
                    "  u4 = normal(10 PLN) to 0.01 PLN \"the card\"",
                )],
                "LOAD-PARSE cannot parse: normal takes 2 parameters (a.tally:48)",
            ),
            // dist-units
            (
                &[(
                    "  u4 = normal(10 PLN, 2 PLN) to 0.01 PLN \"the card\"",
                    "  u4 = uniform(1 PLN, 2 PLN/year) to 0.01 PLN \"the card\"",
                )],
                "LOAD-UNIT uniform(money PLN, num PLN/year): the parameters have different units (a.tally:48)",
            ),
            // dist-whole
            (
                &[(
                    "  u4 = normal(10 PLN, 2 PLN) to 0.01 PLN \"the card\"",
                    "  u4 = normal(10 PLN, 2 PLN) \"the card\"",
                )],
                "LOAD-TO continuous distribution normal needs to u (a row of u4) (a.tally:48)",
            ),
            // dist-plain
            (
                &[(
                    "  k4 = poisson(2) \"the card\"",
                    "  k4 = poisson(2 PLN) \"the card\"",
                )],
                "LOAD-UNIT poisson: a parameter is money PLN, not a plain number (a.tally:49)",
            ),
            // dist-counts
            (
                &[(
                    "  u4 = normal(10 PLN, 2 PLN) to 0.01 PLN \"the card\"",
                    "  u4 = poisson(2) \"the card\"",
                )],
                "LOAD-UNIT poisson gives counts, not a dist PLN (a.tally:48)",
            ),
            // discrete-range
            (
                &[(
                    "def picked: dist = discrete(1 for q in qs)",
                    "def picked: dist = discrete(1 for q in 1 .. 3)",
                )],
                "LOAD-LITERAL figure 3 in a rule; make it a row (a.tally:29)",
            ),
            // discrete-weight
            (
                &[(
                    "def picked: dist = discrete(1 for q in qs)",
                    "def picked: dist = discrete(amt for q in qs)",
                )],
                "LOAD-UNIT discrete: the weight is money PLN, not a plain number (a.tally:29)",
            ),
            // dist-rule-whole
            (
                &[(
                    "def picked: dist = discrete(1 for q in qs)",
                    "def picked: dist = discrete(1 for q in qs)\n\ndef nd: dist PLN = normal(amt, amt)",
                )],
                "LOAD-TO continuous distribution normal needs to u (rule base of nd) (a.tally:31)",
            ),
            // dist-arith
            (
                &[(
                    "def picked: dist = discrete(1 for q in qs)",
                    "def picked: dist = discrete(1 for q in qs)\n\ndef nd: num = picked + 1",
                )],
                "LOAD-UNIT rule base of nd is key qs, not num (a.tally:31)",
            ),
            // cdf-to
            (
                &[(
                    "ask qc: cdf(normal(0, 1), 1) to 0.0001",
                    "ask qc: cdf(normal(0, 1), 1)",
                )],
                "LOAD-TO cdf of normal needs to u (a.tally:58)",
            ),
            // cdf-affine
            (
                &[(
                    "ask qc: cdf(normal(0, 1), 1) to 0.0001",
                    "ask qc: cdf(u4 * u4, amt) to 0.01",
                )],
                "LOAD-RANDOM cdf of a non-affine random value: it takes a distribution, a random fact read directly or an affine form of one; the question is a share over paths (a.tally:58)",
            ),
            // pmf-continuous
            (
                &[(
                    "ask qc: cdf(normal(0, 1), 1) to 0.0001",
                    "ask qc: pmf(normal(0, 1), 1) to 0.0001",
                )],
                "LOAD-UNIT pmf of the continuous normal (a.tally:58)",
            ),
            // pdf-orient
            (
                &[(
                    "ask qc: cdf(normal(0, 1), 1) to 0.0001",
                    "ask qc: pdf(normal(0, 1), 1) to 0.0001",
                )],
                "LOAD-ORIENT ask qc reaches crps or pdf without orient (a.tally:58)",
            ),
            // pdf-keys
            (
                &[(
                    "ask qk: pmf(k4, nyears)",
                    "ask qk: pdf(picked, q1) orient lower",
                )],
                "LOAD-UNIT pdf of a discrete distribution (a.tally:60)",
            ),
            // cdf-point
            (
                &[(
                    "ask qk: pmf(k4, nyears)",
                    "ask qk: cdf(u4, nyears) to 0.0001",
                )],
                "LOAD-UNIT cdf at int for a value of unit PLN (a.tally:60)",
            ),
            // query-flat
            (
                &[("ask qm: mean(mu * u4 + amt)", "ask qm: mean(amt)")],
                "LOAD-RANDOM mean of a value that is not random: a query is asked of a random value (L §5.16) (a.tally:52)",
            ),
            // query-askonly
            (
                &[(
                    "def picked: dist = discrete(1 for q in qs)",
                    "def picked: dist = discrete(1 for q in qs)\n\ndef sq: num PLN^2 = mean(u4 * u4)",
                )],
                "LOAD-ASKONLY mean(u4 * u4) has no closed form outside an ask (a.tally:31)",
            ),
            // query-where
            (
                &[(
                    "ask qsh: share(u4 > amt)",
                    "ask qsh: share(u4 > amt where amt > step_in)",
                )],
                "LOAD-RANDOM share(u4 > amt where amt > step_in) has no closed form (a query with where); state paths N seed S (a.tally:54)",
            ),
            // quantile-two
            (
                &[(
                    "ask qq: quantile(u4, 0.9)",
                    "ask qq: quantile(u4 + u4, 0.9)",
                )],
                "LOAD-RANDOM quantile(u4 + u4, 0.9) has no closed form (a quantile of more than one random fact); state paths N seed S (a.tally:56)",
            ),
            // share-both
            (
                &[("ask qsh: share(u4 > amt)", "ask qsh: share(u4 > u4)")],
                "LOAD-RANDOM share(u4 > u4) has no closed form (random values on both sides); state paths N seed S (a.tally:54)",
            ),
            // mean-dist
            (
                &[("ask qm: mean(mu * u4 + amt)", "ask qm: mean(normal(0, 1))")],
                "LOAD-RANDOM mean of a distribution value; ask it of a random fact (a.tally:52)",
            ),
            // quantile-to
            (
                &[(
                    "ask qq: quantile(u4, 0.9)",
                    "ask qq: quantile(normal(0, 1), 0.9)",
                )],
                "LOAD-TO quantile of normal needs to u (a.tally:56)",
            ),
            // line-random
            (
                &[("ask qm: mean(mu * u4 + amt)", "ask qm: u4")],
                "LOAD-RANDOM ask qm: u4 is random; apply a query (a.tally:52)",
            ),
            // line-random-def
            (
                &[("ask qm: mean(mu * u4 + amt)", "ask qm: lifted")],
                "LOAD-RANDOM ask qm: lifted is random; apply a query (a.tally:52)",
            ),
            // paths-zero
            (
                &[(
                    "ask qp: mean(u4 * u4) paths 100 seed 1",
                    "ask qp: mean(u4 * u4) paths 0 seed 1",
                )],
                "LOAD-DOMAIN paths 0: N is at least 1 (a.tally:64)",
            ),
            // path-out
            (
                &[(
                    "ask qp: mean(u4 * u4) paths 100 seed 1",
                    "ask qp: mean(u4 * u4) paths 100 seed 1 path 101",
                )],
                "LOAD-DOMAIN path 101 is outside 1..100 (a.tally:64)",
            ),
            // crps-def
            (
                &[(
                    "def picked: dist = discrete(1 for q in qs)",
                    "def picked: dist = discrete(1 for q in qs)\n\ndef sc: num PLN = crps(u4, amt)",
                )],
                "LOAD-RANDOM crps of a random value outside a paths ask (a.tally:31)",
            ),
            // crps-orient
            (
                &[(
                    "ask qo: crps(u4, amt) paths 100 seed 1 orient lower",
                    "ask qo: crps(u4, amt) paths 100 seed 1",
                )],
                "LOAD-ORIENT ask qo reaches crps or pdf without orient (a.tally:62)",
            ),
            // crps-normal
            (
                &[(
                    "ask qo: crps(u4, amt) paths 100 seed 1 orient lower",
                    "ask qo: crps(normal(amt, amt), amt) orient lower",
                )],
                "LOAD-TO crps of a normal forecast needs to u (a.tally:62)",
            ),
            // crps-outcome
            (
                &[(
                    "ask qo: crps(u4, amt) paths 100 seed 1 orient lower",
                    "ask qo: crps(u4, mu) paths 100 seed 1 orient lower",
                )],
                "LOAD-UNIT crps: the outcome num and the forecast (PLN) have different units (a.tally:62)",
            ),
            // to-plain
            (
                &[(
                    "def grown: num = exp(mu) to 0.0001",
                    "def grown: num = mu to 0.0001",
                )],
                "LOAD-UNIT to u on mu: `to` states the precision of a transcendental value (L §7.3) (a.tally:31)",
            ),
            // to-step-unit
            (
                &[(
                    "def grown: num = exp(mu) to 0.0001",
                    "def grown: num = exp(mu) to 1 PLN",
                )],
                "LOAD-UNIT to 1: the step and the value (num) have different units (a.tally:31)",
            ),
            // to-step-zero
            (
                &[(
                    "def grown: num = exp(mu) to 0.0001",
                    "def grown: num = exp(mu) to 0",
                )],
                "LOAD-DOMAIN to 0: a step that is not positive (a.tally:31)",
            ),
            // exp-bare
            (
                &[(
                    "def grown: num = exp(mu) to 0.0001",
                    "def grown: num = exp(mu)",
                )],
                "LOAD-TO exp needs to u (a.tally:31)",
            ),
            // exp-unit
            (
                &[(
                    "def grown: num = exp(mu) to 0.0001",
                    "def grown: num = exp(amt) to 0.0001",
                )],
                "LOAD-UNIT exp of money PLN: its argument is a plain number (a.tally:31)",
            ),
            // sqrt-odd
            (
                &[(
                    "def grown: num = exp(mu) to 0.0001",
                    "def grown: num = sqrt(amt) to 0.0001",
                )],
                "LOAD-UNIT sqrt of money PLN: its unit has an odd power (a.tally:31)",
            ),
            // pow-real
            (
                &[(
                    "def grown: num = exp(mu) to 0.0001",
                    "def grown: num = pow(amt, mu)",
                )],
                "LOAD-UNIT pow of money PLN to a power that may not be an integer: its unit would have a fractional power (a.tally:31)",
            ),
            // pow-literal
            (
                &[(
                    "def grown: num = exp(mu) to 0.0001",
                    "def grown: num = pow(amt, nyears)",
                )],
                "LOAD-UNIT pow of money PLN to a power that is not a literal (a.tally:31)",
            ),
            // factor-to
            (
                &[(
                    "def compounded: num = factor(rate, nyears, year, effective)",
                    "def compounded: num = factor(rate, mu, year, effective)",
                )],
                "LOAD-TO factor needs to u: load can't show its power is an integer (a.tally:33)",
            ),
            // factor-period
            (
                &[(
                    "def compounded: num = factor(rate, nyears, year, effective)",
                    "def compounded: num = factor(rate, nyears, week, effective)",
                )],
                "LOAD-UNIT factor: the period argument is a period word (day, month, quarter or year), not week (a.tally:33)",
            ),
            // factor-rate
            (
                &[(
                    "def compounded: num = factor(rate, nyears, year, effective)",
                    "def compounded: num = factor(mu, nyears, year, effective)",
                )],
                "LOAD-UNIT factor: mu is num, not a rate (a.tally:33)",
            ),
            // factor-code
            (
                &[(
                    "def compounded: num = factor(rate, nyears, year, effective)",
                    "def compounded: num = factor(rate, nyears, year, act365)",
                )],
                "LOAD-UNIT act365 is not a compounding code (a.tally:33)",
            ),
            // solve-askonly
            (
                &[(
                    "  rule base v card_2026 = solve amt == lifted for step_in in [0 PLN, 1000 PLN] to 1 PLN",
                    "  rule base = solve amt == lifted for step_in in [0 PLN, 1000 PLN] to 1 PLN",
                )],
                "LOAD-ASKONLY solve outside an ask (allowed in a rule with v) (a.tally:38)",
            ),
            // solve-grid
            (
                &[(
                    "ask qv: solve amt == step_in for step_in in [0 PLN, 500 PLN] to 1 PLN",
                    "ask qv: solve amt == step_in for step_in in [0 PLN, 500.50 PLN] to 1 PLN",
                )],
                "LOAD-DOMAIN solve: hi is not on the grid lo + i u (a.tally:66)",
            ),
            // solve-below
            (
                &[(
                    "ask qv: solve amt == step_in for step_in in [0 PLN, 500 PLN] to 1 PLN",
                    "ask qv: solve amt == step_in for step_in in [500 PLN, 0 PLN] to 1 PLN",
                )],
                "LOAD-DOMAIN solve: the bracket [lo, hi] has hi below lo (a.tally:66)",
            ),
            // solve-unknown
            (
                &[(
                    "ask qv: solve amt == step_in for step_in in [0 PLN, 500 PLN] to 1 PLN",
                    "ask qv: solve amt == step_in for nope in [0 PLN, 500 PLN] to 1 PLN",
                )],
                "LOAD-NAME unknown id nope (a.tally:66)",
            ),
            // solve-type
            (
                &[(
                    "ask qv: solve amt == step_in for step_in in [0 PLN, 500 PLN] to 1 PLN",
                    "ask qv: solve amt == step_in for k4 in [0, 5] to 1",
                )],
                "LOAD-UNIT solve for k4 of type dist: a root is searched over numbers (a.tally:66)",
            ),
            // solve-number
            (
                &[(
                    "ask qv: solve amt == step_in for step_in in [0 PLN, 500 PLN] to 1 PLN",
                    "ask qv: solve picked == picked for step_in in [0 PLN, 500 PLN] to 1 PLN",
                )],
                "LOAD-UNIT solve key qs == key qs: F = f - g is a number (a.tally:66)",
            ),
            // prorata-total
            (
                &[(
                    "def share_of[q in qs]: money PLN = prorata(amt, 1 for p in qs, q, 1 PLN, half-up, rest last)",
                    "def share_of[q in qs]: money PLN = prorata(mu, 1 for p in qs, q, 1 PLN, half-up, rest last)",
                )],
                "LOAD-UNIT prorata of num: the total is money or a num CUR (L §5.13) (a.tally:35)",
            ),
            // prorata-binds
            (
                &[(
                    "def share_of[q in qs]: money PLN = prorata(amt, 1 for p in qs, q, 1 PLN, half-up, rest last)",
                    "def share_of[q in qs]: money PLN = prorata(amt, 1 for p in qs, r in qs, q, 1 PLN, half-up, rest last)",
                )],
                "LOAD-PARSE cannot parse: prorata takes one binding (a.tally:35)",
            ),
            // prorata-step
            (
                &[(
                    "def share_of[q in qs]: money PLN = prorata(amt, 1 for p in qs, q, 1 PLN, half-up, rest last)",
                    "def share_of[q in qs]: money PLN = prorata(amt, 1 for p in qs, q, 0.01 PLN, half-up, rest last)",
                )],
                "LOAD-LITERAL figure 0.01 PLN in a rule; make it a row (a.tally:35)",
            ),
            // prorata-step-unit
            (
                &[(
                    "def share_of[q in qs]: money PLN = prorata(amt, 1 for p in qs, q, 1 PLN, half-up, rest last)",
                    "def share_of[q in qs]: money PLN = prorata(amt, 1 for p in qs, q, 1 %, half-up, rest last)",
                )],
                "LOAD-UNIT prorata: the step num % and the total money PLN have different units (a.tally:35)",
            ),
            // prorata-weight
            (
                &[(
                    "def share_of[q in qs]: money PLN = prorata(amt, 1 for p in qs, q, 1 PLN, half-up, rest last)",
                    "def share_of[q in qs]: money PLN = prorata(amt, true for p in qs, q, 1 PLN, half-up, rest last)",
                )],
                "LOAD-UNIT prorata: the weight is bool, not a number (a.tally:35)",
            ),
            // prorata-num
            (
                &[(
                    "def share_of[q in qs]: money PLN = prorata(amt, 1 for p in qs, q, 1 PLN, half-up, rest last)",
                    "def share_of[q in qs]: money PLN = prorata(amt, 1 for p in qs, q, step_in, half-up, rest last)",
                )],
                "loaded",
            ),
            // scenario-row
            (
                &[(
                    "  amt = 100.00 PLN \"the card\"",
                    "  amt = step_in + step_in \"the card\"",
                )],
                "LOAD-SCENARIO an expression row outside a scenario (a.tally:44)",
            ),
        ];
        for &(edits, expected) in probes {
            assert_eq!(refusal(&edited(RANDOM, edits)), expected, "{edits:?}");
        }
    }
}
