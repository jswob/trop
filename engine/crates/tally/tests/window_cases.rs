//! The builder's window tests: a test that only some of a window's days pass is the window's hole, as refcore prints it.
//! Spec: L §4.4 (a window's date not precise enough: a read is a hole only where its answer
//! depends on the day), E §4.1 steps 2–3 (a window-dated instance's class, X4), E §4.2 Holes in
//! the walk and L §5.19 (a hole movement reaches to its place's next reading, R11), the plan's
//! §14.1–§14.2 (item 77: the printed question doesn't change; "depends" reaches as far as the
//! refusal would), pick P30 (the answer is what prints).
//! Twin: engine/tools/refcore/moves.py (`window_class_depends`), walk.py (`imp_three`,
//! `imp_hole`, `words_differ`); each expected line is refcore's run of the same text
//! (`python3 -m refcore run`, spec/2.1.8 at 9f450393), each text one of the review of build
//! group 2's inputs.
//! Invariants: reads no test, vector or answer key (R-93); the texts are written here; every
//! `windows each` line equals its ask pinned to its sub-window's first day and its last.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally::time::Day;
    use tally::{Asks, Days, Given, Pin, Request, run};

    /// The lines a run of `text` prints for `asks`.
    fn lines_for(text: &str, asks: Asks) -> Vec<String> {
        let given = vec![Given {
            path: "w.tally".to_owned(),
            bytes: Some(text.as_bytes().to_vec()),
        }];
        let request = Request {
            asks,
            pin: Pin::Canonical,
            days: Days::Events,
        };
        let answer = run(given, &request).expect("an answer");
        answer.lines().to_vec()
    }

    /// The lines a run of `text` prints, every ask.
    fn lines_of(text: &str) -> Vec<String> {
        lines_for(text, Asks::Every)
    }

    /// Each ask's block: its head line and the lines under it.
    fn blocks(lines: &[String]) -> Vec<Vec<String>> {
        let mut out: Vec<Vec<String>> = Vec::new();
        for line in lines {
            match out.last_mut() {
                Some(block) if line.starts_with("  ") => block.push(line.clone()),
                _ => out.push(vec![line.clone()]),
            }
        }
        out
    }

    /// An unknown day's input and the days a line `windows each` doesn't split is pinned to.
    type Unsplit<'a> = Option<(&'a str, &'a [String])>;

    /// The days from `first` to `last`, each written `YYYY-MM-DD`.
    fn days_from(first: &str, last: &str) -> Vec<String> {
        let day = |text: &str| {
            let mut parts = text.split('-');
            let mut part = || parts.next().expect("a date's part");
            let year: u16 = part().parse().expect("a year");
            let month: u8 = part().parse().expect("a month");
            let day: u8 = part().parse().expect("a day");
            Day::from_ymd(year, month, day).expect("a day")
        };
        let (mut d, end) = (day(first), day(last));
        let mut out = vec![d.text()];
        while d < end {
            d = d.next().expect("the next day");
            out.push(d.text());
        }
        out
    }

    /// Every line `windows each` splits by a window's days equals its ask's plain line
    /// pinned to each day of its sub-window, and a line it doesn't split equals the plain line
    /// pinned to each of `unsplit`'s days of its unknown: the same answer and the same
    /// register, the pin's own `pinned:` line and the run's warnings aside (the lead's ruling
    /// on P30; the review of group 2's fix, N-h).
    fn pinned_twins(text: &str, unsplit: Unsplit<'_>) -> Vec<String> {
        let mut checked = Vec::new();
        let asks: Vec<(String, String)> = text
            .lines()
            .filter_map(|l| l.strip_prefix("ask "))
            .filter_map(|l| l.strip_suffix(" windows each"))
            .filter_map(|l| l.split_once(": "))
            .map(|(name, expr)| (name.to_owned(), expr.to_owned()))
            .collect();
        let all = blocks(&lines_of(text));
        for (name, expr) in &asks {
            let head = format!("{name} [");
            let plain = format!("{name}: ");
            for block in &all {
                let (var, days, answer) = if let Some(rest) = block[0].strip_prefix(&head) {
                    let (pins, answer) = rest.split_once("]: ").expect("a split line's head");
                    let (var, span) = pins.split_once(" = ").expect("one pin");
                    assert!(!span.contains(", "), "one unknown day: {}", block[0]);
                    let (first, last) = span.split_once(" .. ").unwrap_or((span, span));
                    (var.to_owned(), days_from(first, last), answer.to_owned())
                } else if let (Some(answer), Some((var, days))) =
                    (block[0].strip_prefix(&plain), unsplit)
                {
                    (var.to_owned(), days.to_vec(), answer.to_owned())
                } else {
                    continue;
                };
                for day in &days {
                    let twin = format!("{text}\nask zz_twin: {expr} with {var} = {day}\n");
                    let got = lines_for(&twin, Asks::Named(vec!["zz_twin".to_owned()]));
                    let mut want = vec![format!("zz_twin: {answer}")];
                    want.extend(block[1..].iter().cloned());
                    let got: Vec<String> = got
                        .into_iter()
                        .filter(|l| !l.starts_with("  pinned: ") && !l.starts_with("warning: "))
                        .collect();
                    assert_eq!(got, want, "{} pinned to {var} = {day}", block[0]);
                    checked.push(format!("{name} {var} = {day}"));
                }
            }
        }
        checked
    }

    /// The covered-day face (i77b's text): `pay`, dated by `window(03-05, 03-10)`, is booked
    /// on the days after the statement's span and modelled past on the days inside it. Reads
    /// before the window answer; a binding over the window is the window's hole, its
    /// question the read's; the review of group 2's B1.
    const COVER_REACH: &str = r#"model t
  tally 2
  on 2026-03-08
  start 2026-01-01
  end 2026-06-30

source s "s"
unit PLN: currency minor 0.01
place konto: accounts PLN lives 2025-01-01..
place world: outside lives 1900-01-01..
input bank: observed
input amt: money PLN
input wd: date

move pay: transfers
  on wd
  leg cash from world to konto amount amt

version v
  cite s
  governs always
  known 2025-12-01
  amt = 100.00 PLN "s"
  wd = window(2026-03-05, 2026-03-10) "s"
  opening[konto] = 0.00 PLN at 2026-01-01 start "s"

version st
  cite s
  governs 2026-03-01..2026-03-07
  known 2026-03-08
  covers konto

ask a1: bal(konto, 2026-02-28)
ask a2: bal(konto, 2026-03-04)
ask a3: count(t in transfers over 2026-03-01 .. 2026-03-31)
ask a4: count(t in transfers over 2026-03-01 .. 2026-03-31) windows each
ask a5: flows(pay, 2026-02)
"#;

    const COVER_REACH_LINES: &str = r#"a1: 0.00 PLN
  cut 2026-03-08
a2: 0.00 PLN
  cut 2026-03-08
a3: ?"date within window(2026-03-05, 2026-03-10) not precise enough for a binding over transfers" (pay)
  cut 2026-03-08
a4 [wd = 2026-03-05 .. 2026-03-07]: 0
  cut 2026-03-08
a4 [wd = 2026-03-08 .. 2026-03-10]: 1
  cut 2026-03-08
a5: 0.00 PLN
  cut 2026-03-08"#;

    /// The same text a month later, with an anchor of 500.00 at 04-01 after the window: the
    /// hole reaches from the window's first day to that reading, which ends it.
    const ANCHOR_AFTER: &str = r#"model t
  tally 2
  on 2026-04-08
  start 2026-01-01
  end 2026-06-30

source s "s"
unit PLN: currency minor 0.01
place konto: accounts PLN lives 2025-01-01..
place world: outside lives 1900-01-01..
input bank: observed
input amt: money PLN
input wd: date

move pay: transfers
  on wd
  leg cash from world to konto amount amt

version v
  cite s
  governs always
  known 2025-12-01
  anchor konto
  amt = 100.00 PLN "s"
  wd = window(2026-03-05, 2026-03-10) "s"
  opening[konto] = 0.00 PLN at 2026-01-01 start "s"
  stated[konto] = 500.00 PLN at 2026-04-01 end "s"

version st
  cite s
  governs 2026-03-01..2026-03-07
  known 2026-03-08
  covers konto

ask a1: bal(konto, 2026-02-28)
ask b1: bal(konto, 2026-03-20)
ask b2: bal(konto, 2026-04-02)
ask b4: flows(pay, 2026-03)
ask b5: flows(pay, 2026-03) windows each
"#;

    const ANCHOR_AFTER_LINES: &str = r#"a1: 0.00 PLN
  cut 2026-04-08
b1: ?"date within window(2026-03-05, 2026-03-10) not precise enough for a balance" (pay)
  cut 2026-04-08
b2: 500.00 PLN
  anchor: konto 2026-04-01 end, stated 500.00 PLN, difference ?"date within window(2026-03-05, 2026-03-10) not precise enough for a balance" (pay)
  cut 2026-04-08
b4: ?"date within window(2026-03-05, 2026-03-10) not precise enough for flows of pay" (pay)
  cut 2026-04-08
b5 [wd = 2026-03-05 .. 2026-03-07]: 0.00 PLN
  open: pay
  cut 2026-04-08
b5 [wd = 2026-03-08 .. 2026-03-10]: 100.00 PLN
  unfulfilled: pay
  cut 2026-04-08"#;

    /// The run-start face (i77c's text): `early`, dated by `window(12-30, 01-03)` across the
    /// run's start, is a member of a span that holds its whole window.
    const RUN_START: &str = r#"model t
  tally 2
  on 2025-12-01
  start 2026-01-01
  end 2026-06-30

source s "s"
unit PLN: currency minor 0.01
place konto: accounts PLN lives 2025-01-01..
place world: outside lives 1900-01-01..
input amt: money PLN
input ws: date
input we: date

move early: transfers
  on ws
  leg cash from world to konto amount amt

move late: transfers
  on we
  leg cash from world to konto amount amt

version v
  cite s
  governs always
  known 2025-11-01
  amt = 100.00 PLN "s"
  ws = window(2025-12-30, 2026-01-03) "s"
  we = window(2026-06-28, 2026-07-03) "s"
  opening[konto] = 0.00 PLN at 2026-01-01 start "s"

ask s1: bal(konto, 2026-01-10)
ask s4: count(t in transfers over 2025-12-01 .. 2026-01-31)
ask s4w: count(t in transfers over 2025-12-01 .. 2026-01-31) windows each
"#;

    const RUN_START_LINES: &str = r#"s1: ?"date within window(2025-12-30, 2026-01-03) not precise enough for a balance" (early)
  cut 2025-12-01
s4: 1
  cut 2025-12-01
s4w: 1
  cut 2025-12-01"#;

    /// The covered-day face with a gap (the review of group 2's B4): `pay`, dated by
    /// `window(03-04, 03-10)`, is modelled past on the days two statements cover (03-04 ..
    /// 03-05, 03-09 .. 03-10) and booked on the days between, though its first day and its
    /// last are both covered.
    const COVER_GAP: &str = r#"model t
  tally 2
  on 2026-04-30
  start 2026-01-01
  end 2026-06-30

source s "s"
unit PLN: currency minor 0.01
place konto: accounts PLN lives 2025-01-01..
place world: outside lives 1900-01-01..
input bank: observed
input amt: money PLN
input wd: date

move pay: transfers
  on wd
  leg cash from world to konto amount amt

version v
  cite s
  governs always
  known 2025-12-01
  amt = 100.00 PLN "s"
  wd = window(2026-03-04, 2026-03-10) "s"
  opening[konto] = 0.00 PLN at 2026-01-01 start "s"

version s1
  cite s
  governs 2026-03-01..2026-03-05
  known 2026-03-08
  covers konto

version s2
  cite s
  governs 2026-03-09..2026-03-12
  known 2026-03-08
  covers konto

ask c1: bal(konto, 2026-03-31)
ask c2: bal(konto, 2026-03-31) windows each
ask c3: bal(konto, 2026-03-31) with wd = 2026-03-07
ask c4: bal(konto, 2026-03-31) with wd = 2026-03-04
"#;

    const COVER_GAP_LINES: &str = r#"c1: ?"date within window(2026-03-04, 2026-03-10) not precise enough for a balance" (pay)
  cut 2026-04-30
c2 [wd = 2026-03-04 .. 2026-03-05]: 0.00 PLN
  open: pay
  cut 2026-04-30
c2 [wd = 2026-03-06 .. 2026-03-08]: 100.00 PLN
  unfulfilled: pay
  cut 2026-04-30
c2 [wd = 2026-03-09 .. 2026-03-10]: 0.00 PLN
  open: pay
  cut 2026-04-30
c3: 100.00 PLN
  pinned: wd = 2026-03-07
  unfulfilled: pay
  cut 2026-04-30
c4: 0.00 PLN
  pinned: wd = 2026-03-04
  open: pay
  cut 2026-04-30"#;

    /// The sign rule's face, its other ends (the review of group 2's B2): `oa` takes 100.00
    /// from `pot` (`min 0`, 50.00) to `jar` on a day of `window(04-05, 04-10)`; `ob` brings
    /// 100.00 into `pot` on 04-07. Before 04-07 the rule refuses `oa`, after it it doesn't:
    /// `pot`, `jar` and a read of `oa`'s leg are the window's hole.
    const SIGN_ENDS: &str = r#"model t
  tally 2
  on 2026-01-01
  start 2026-01-01
  end 2026-06-30

source s "s"
unit PLN: currency minor 0.01
kind savings: place PLN min 0
place pot: savings lives 2025-01-01..
place jar: accounts PLN lives 2025-01-01..
place world: outside lives 1900-01-01..
input wa: date
input db: date
input amt: money PLN

move oa: transfers
  on wa
  leg cash from pot to jar amount amt

move ob: transfers
  on db
  leg cash from world to pot amount amt

version v
  cite s
  governs always
  known 2025-12-01
  amt = 100.00 PLN "s"
  wa = window(2026-04-05, 2026-04-10) "s"
  db = 2026-04-07 "s"
  opening[pot] = 50.00 PLN at 2026-01-01 start "s"
  opening[jar] = 0.00 PLN at 2026-01-01 start "s"

ask j1: bal(jar, 2026-04-30)
ask j2: bal(jar, 2026-04-30) windows each
ask f1: flows(oa, 2026-04)
ask f2: flows(oa, 2026-04) windows each
ask p1: bal(pot, 2026-04-30)
ask p2: bal(pot, 2026-04-30) windows each
ask j3: bal(jar, 2026-04-30) with wa = 2026-04-05
ask f3: flows(oa, 2026-04) with wa = 2026-04-05
"#;

    const SIGN_ENDS_LINES: &str = r#"f1: ?"date within window(2026-04-05, 2026-04-10) not precise enough for flows of oa" (oa)
  cut 2026-01-01
f2 [wa = 2026-04-05]: refused: RUN-PLACE pot is -50.00 PLN after the day's observations and plans of 2026-04-05
  cut 2026-01-01
f2 [wa = 2026-04-06]: refused: RUN-PLACE pot is -50.00 PLN after the day's observations and plans of 2026-04-06
  cut 2026-01-01
f2 [wa = 2026-04-07 .. 2026-04-10]: 100.00 PLN
  cut 2026-01-01
f3: refused: RUN-PLACE pot is -50.00 PLN after the day's observations and plans of 2026-04-05
  pinned: wa = 2026-04-05
  cut 2026-01-01
j1: ?"date within window(2026-04-05, 2026-04-10) not precise enough for a balance" (oa)
  cut 2026-01-01
j2 [wa = 2026-04-05]: refused: RUN-PLACE pot is -50.00 PLN after the day's observations and plans of 2026-04-05
  cut 2026-01-01
j2 [wa = 2026-04-06]: refused: RUN-PLACE pot is -50.00 PLN after the day's observations and plans of 2026-04-06
  cut 2026-01-01
j2 [wa = 2026-04-07 .. 2026-04-10]: 100.00 PLN
  cut 2026-01-01
j3: refused: RUN-PLACE pot is -50.00 PLN after the day's observations and plans of 2026-04-05
  pinned: wa = 2026-04-05
  cut 2026-01-01
p1: ?"date within window(2026-04-05, 2026-04-10) not precise enough for a balance" (oa)
  cut 2026-01-01
p2 [wa = 2026-04-05]: refused: RUN-PLACE pot is -50.00 PLN after the day's observations and plans of 2026-04-05
  cut 2026-01-01
p2 [wa = 2026-04-06]: refused: RUN-PLACE pot is -50.00 PLN after the day's observations and plans of 2026-04-06
  cut 2026-01-01
p2 [wa = 2026-04-07 .. 2026-04-10]: 50.00 PLN
  cut 2026-01-01"#;

    /// Every day refuses, in words that differ by the day (the review of group 2's B3, pick
    /// P30): `oa` takes 10.00 from `pot` (`min 0`, 50.00) on a day of `window(04-05,
    /// 04-10)`, `ob` 200.00 on 04-07; `pot` is -160.00 where `oa` has fallen by 04-07,
    /// -150.00 after. The plain line is the window's hole; `windows each` prints each day's
    /// own refusal.
    const BOTH_BREAK: &str = r#"model t
  tally 2
  on 2026-01-01
  start 2026-01-01
  end 2026-06-30

source s "s"
unit PLN: currency minor 0.01
kind savings: place PLN min 0
place pot: savings lives 2025-01-01..
place jar: accounts PLN lives 2025-01-01..
place world: outside lives 1900-01-01..
input wa: date
input db: date
input amt: money PLN
input small: money PLN
input amt2: money PLN

move oa: transfers
  on wa
  leg cash from pot to jar amount small

move ob: transfers
  on db
  leg cash from pot to world amount amt2

version v
  cite s
  governs always
  known 2025-12-01
  amt = 100.00 PLN "s"
  small = 10.00 PLN "s"
  amt2 = 200.00 PLN "s"
  wa = window(2026-04-05, 2026-04-10) "s"
  db = 2026-04-07 "s"
  opening[pot] = 50.00 PLN at 2026-01-01 start "s"
  opening[jar] = 0.00 PLN at 2026-01-01 start "s"

ask g1: flows(ob, 2026-04)
ask g2: flows(ob, 2026-04) windows each
ask g3: bal(pot, 2026-04-30)
ask g4: bal(pot, 2026-04-30) windows each
ask g5: bal(pot, 2026-04-30) with wa = 2026-04-09
ask g6: bal(pot, 2026-04-30) with wa = 2026-04-05
ask g7: flows(ob, 2026-04) with wa = 2026-04-05
"#;

    const BOTH_BREAK_LINES: &str = r#"g1: ?"date within window(2026-04-05, 2026-04-10) not precise enough for a balance" (oa)
  cut 2026-01-01
g2 [wa = 2026-04-05 .. 2026-04-07]: refused: RUN-PLACE pot is -160.00 PLN after the day's observations and plans of 2026-04-07
  cut 2026-01-01
g2 [wa = 2026-04-08 .. 2026-04-10]: refused: RUN-PLACE pot is -150.00 PLN after the day's observations and plans of 2026-04-07
  cut 2026-01-01
g3: ?"date within window(2026-04-05, 2026-04-10) not precise enough for a balance" (oa)
  cut 2026-01-01
g4 [wa = 2026-04-05 .. 2026-04-07]: refused: RUN-PLACE pot is -160.00 PLN after the day's observations and plans of 2026-04-07
  cut 2026-01-01
g4 [wa = 2026-04-08 .. 2026-04-10]: refused: RUN-PLACE pot is -150.00 PLN after the day's observations and plans of 2026-04-07
  cut 2026-01-01
g5: refused: RUN-PLACE pot is -150.00 PLN after the day's observations and plans of 2026-04-07
  pinned: wa = 2026-04-09
  cut 2026-01-01
g6: refused: RUN-PLACE pot is -160.00 PLN after the day's observations and plans of 2026-04-07
  pinned: wa = 2026-04-05
  cut 2026-01-01
g7: refused: RUN-PLACE pot is -160.00 PLN after the day's observations and plans of 2026-04-07
  pinned: wa = 2026-04-05
  cut 2026-01-01
warning: WARN-UNREAD input amt is read by nothing"#;

    /// The close test on two balances (i77a with `opening[tmp] = 50.00`; the review's N6):
    /// `tmp` ends on 03-07 holding -50.00 where `empty` has fallen, 50.00 where it hasn't;
    /// every day refuses, in words that differ, so the plain line is the hole.
    const CLOSE_TWO: &str = r#"model t
  tally 2
  on 2026-01-01
  start 2026-01-01
  end 2026-06-30

source s "s"
unit PLN: currency minor 0.01
kind accts: place PLN signed
kind pots: place PLN min 0
place jar: accts lives 2025-01-01..
place tmp: accts lives 2025-01-01..2026-03-07
place pot: pots lives 2025-01-01..
place world: outside lives 1900-01-01..
input amt: money PLN
input wd: date
input wa: date
input wb: date

move empty: transfers
  on wd
  leg cash from tmp to jar amount amt

move oa: transfers
  on wa
  leg cash from pot to world amount amt

move ob: transfers
  on wb
  leg cash from world to pot amount amt

version v
  cite s
  governs always
  known 2025-12-01
  amt = 100.00 PLN "s"
  wd = window(2026-03-05, 2026-03-10) "s"
  wa = window(2026-04-05, 2026-04-10) "s"
  wb = window(2026-04-07, 2026-04-12) "s"
  opening[jar] = 0.00 PLN at 2026-01-01 start "s"
  opening[tmp] = 50.00 PLN at 2026-01-01 start "s"
  opening[pot] = 50.00 PLN at 2026-01-01 start "s"

ask e1: bal(jar, 2026-03-31)
ask e3: bal(tmp, 2026-03-07)
ask e4: bal(tmp, 2026-03-07) windows each
"#;

    const CLOSE_TWO_LINES: &str = r#"e1: ?"date within window(2026-03-05, 2026-03-10) not precise enough for a balance" (empty)
  cut 2026-01-01
e3: ?"date within window(2026-03-05, 2026-03-10) not precise enough for a balance" (empty)
  cut 2026-01-01
e4 [wd = 2026-03-05 .. 2026-03-07]: refused: RUN-LIFESPAN tmp ends on 2026-03-07 holding -50.00 PLN
  cut 2026-01-01
e4 [wd = 2026-03-08 .. 2026-03-10]: refused: RUN-LIFESPAN tmp ends on 2026-03-07 holding 50.00 PLN
  cut 2026-01-01"#;

    /// A built-in over two windows (the review of group 2's N3): `days_between(a, b)` is the
    /// interval over every corner of the windows' ends, a value a comparison reads (`q6`) and
    /// a print can't (L §4.4, "for a print"; tally 2.1.7 R3).
    const TWO_WINDOWS: &str = r#"model t
  tally 2
  on 2026-01-01
  start 2026-01-01
  end 2026-06-30

source s "s"
input a: date
input b: date
input lim: int

def n1: int
  rule base = days_between(a, b)

def small: bool
  rule base = n1 < lim

def n2: int
  rule base = days_between(a, a)

version v
  cite s
  governs always
  known 2025-12-01
  a = window(2026-03-01, 2026-03-03) "s"
  b = window(2026-03-02, 2026-03-04) "s"
  lim = 10 "s"

ask q4: n1
ask q6: small
ask q7: n2
"#;

    const TWO_WINDOWS_LINES: &str = r#"q4: ?"date within window(2026-03-01, 2026-03-03) not precise enough for a print" (a)
  cut 2026-01-01
q6: true
  cut 2026-01-01
q7: ?"date within window(2026-03-01, 2026-03-03) not precise enough for a print" (a)
  cut 2026-01-01"#;

    /// A pending leg no day refuses (the review of group 2's fix, G1): `oa` moves 10.00 from
    /// `pot` (`min 0`, 50.00) to `jar` on one of 04-05 and 04-09; `ob` takes 200.00 out of
    /// `pot` on 04-07 and is refused on either day. `oa` is never refused, so `jar` and
    /// `flows(oa)` keep 10.00: the hole reaches only the legs the refusal would refuse.
    const PENDING_KEPT: &str = r#"model t
  tally 2
  on 2026-01-01
  start 2026-01-01
  end 2026-06-30

source s "s"
unit PLN: currency minor 0.01
kind savings: place PLN min 0
place pot: savings lives 2025-01-01..
place jar: accounts PLN lives 2025-01-01..
place world: outside lives 1900-01-01..
input wa: date
input db: date
input small: money PLN
input amt2: money PLN

move oa: transfers
  on wa
  leg cash from pot to jar amount small

move ob: transfers
  on db
  leg cash from pot to world amount amt2

version v
  cite s
  governs always
  known 2025-12-01
  small = 10.00 PLN "s"
  amt2 = 200.00 PLN "s"
  wa = one_of([2026-04-05, 2026-04-09]) "s"
  db = 2026-04-07 "s"
  opening[pot] = 50.00 PLN at 2026-01-01 start "s"
  opening[jar] = 0.00 PLN at 2026-01-01 start "s"

ask j1: bal(jar, 2026-04-30)
ask j2: bal(jar, 2026-04-30) windows each
ask j3: flows(oa, 2026-04)
ask j4: bal(jar, 2026-04-30) with wa = 2026-04-05
ask j5: bal(jar, 2026-04-30) with wa = 2026-04-09
ask j6: flows(oa, 2026-04) with wa = 2026-04-09
"#;

    const PENDING_KEPT_LINES: &str = r"j1: 10.00 PLN
  cut 2026-01-01
j2: 10.00 PLN
  cut 2026-01-01
j3: 10.00 PLN
  cut 2026-01-01
j4: 10.00 PLN
  pinned: wa = 2026-04-05
  cut 2026-01-01
j5: 10.00 PLN
  pinned: wa = 2026-04-09
  cut 2026-01-01
j6: 10.00 PLN
  pinned: wa = 2026-04-09
  cut 2026-01-01";

    #[test]
    fn a_window_whose_days_take_two_classes_keeps_its_window_and_the_reads_question() {
        for (text, want) in [
            (COVER_REACH, COVER_REACH_LINES),
            (ANCHOR_AFTER, ANCHOR_AFTER_LINES),
            (RUN_START, RUN_START_LINES),
            (COVER_GAP, COVER_GAP_LINES),
            (SIGN_ENDS, SIGN_ENDS_LINES),
            (BOTH_BREAK, BOTH_BREAK_LINES),
            (CLOSE_TWO, CLOSE_TWO_LINES),
            (TWO_WINDOWS, TWO_WINDOWS_LINES),
            (PENDING_KEPT, PENDING_KEPT_LINES),
        ] {
            let want: Vec<&str> = want.lines().collect();
            assert_eq!(lines_of(text), want, "refcore's lines");
        }
    }

    #[test]
    fn each_line_of_a_window_whose_days_take_two_classes_is_its_pinned_twin() {
        let one_of = ["2026-04-05".to_owned(), "2026-04-09".to_owned()];
        let start = days_from("2025-12-30", "2026-01-03");
        let texts: [(&str, Unsplit<'_>); 8] = [
            (COVER_REACH, None),
            (ANCHOR_AFTER, None),
            (RUN_START, Some(("ws", &start))),
            (COVER_GAP, None),
            (SIGN_ENDS, None),
            (BOTH_BREAK, None),
            (CLOSE_TWO, None),
            (PENDING_KEPT, Some(("wa", &one_of))),
        ];
        let checked: Vec<String> = texts
            .iter()
            .flat_map(|(text, unsplit)| pinned_twins(text, *unsplit))
            .collect();
        // a4 and b5 six days each, s4w five, c2 seven, f2, j2 and p2 six each, g2 and g4 six
        // each, e4 six, j2 of PENDING_KEPT two
        assert_eq!(checked.len(), 62, "every day of each window: {checked:?}");
    }
}
