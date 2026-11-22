//! tally 2.3's pin oracle on the engine (#247, the plan's §9; `properties/oracle.py`). The
//! generator's texts (`properties/oracle.corpus`, written by `oracle.py --corpus`): one input
//! `w` holds a window (a readers' text of two windows also `v2`), and each line is asked unpinned
//! and under `with w = D` for every day D of it (every pair of days, for two). The unpinned line
//! must print what every pinned day prints that isn't counted apart (a balance check's refusal,
//! `RUN-PLACE`, `RUN-LIFESPAN`, a take's, and a movement that doesn't exist, `RUN-NONE`), or a
//! hole that names a window, or a refused day's refusal, or, for a date, the stretch
//! `window(a, b)` holding every pinned day's print. A reader's line is held harder (`list`,
//! `off`, `false`, as `oracle.py`'s `judge` says). An answer's register lines are left out and
//! its body is held line by line. No stored answer: the engine's own lines are compared with
//! each other. A line `oracle.py`'s `HELD` names (a fault found, mended at a later pin) is
//! reported, not failed; a text whose every line is a hole or a refusal fails, but a place
//! `APART` names (a rule kept, N2) is counted apart. Gating with M3 (`check commit`);
//! `m3_oracle_tenth` is the fixed tenth by seed a quick loop takes, `m3_oracle_rest` the rest.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use tally_runner::corpus::netstrings;
    use tally_runner::engine::run_args;

    const CORPUS: &[u8] = include_bytes!("../../properties/oracle.corpus");
    /// `oracle.py`'s default seed, 20261121, modulo 10: the tenth `--sample 10` runs.
    const TENTH: usize = 1;
    const REGISTER: [&str; 11] = [
        "cut",
        "unfulfilled",
        "pinned",
        "assumed",
        "adjustment",
        "pad",
        "unknown day",
        "anchor",
        "open",
        "left out",
        "world",
    ];
    /// The refusals whose days are counted apart: a balance check's (C-3) and `RUN-NONE` (ruling 15).
    const BALANCE_CHECKS: [&str; 5] = [
        "RUN-PLACE",
        "RUN-LIFESPAN",
        "RUN-TAKE",
        "RUN-PART",
        "RUN-NONE",
    ];

    /// One generated text: the corpus record's nine fields.
    struct Text {
        id: String,
        args: Vec<String>,
        files: Vec<u8>,
        days: Vec<String>,
        lines: usize,
        held: BTreeMap<usize, String>,
        apart: Option<String>,
        anchors: BTreeMap<usize, String>,
        expect: BTreeMap<usize, String>,
    }

    /// `YYYYMMDD` as printed, `YYYY-MM-DD`.
    fn dashed(d: &str) -> String {
        let (y, md) = d.split_at_checked(4).unwrap_or((d, ""));
        let (m, dd) = md.split_at_checked(2).unwrap_or((md, ""));
        format!("{y}-{m}-{dd}")
    }

    /// The text's window `w` as an answer prints it, `window(a, b)`: the first and last pinned
    /// keys' first day.
    fn window_of(t: &Text) -> String {
        let day = |k: &String| dashed(k.get(..8).unwrap_or(k));
        let first = t.days.first().map_or(String::new(), day);
        let last = t.days.last().map_or(String::new(), day);
        format!("window({first}, {last})")
    }

    /// What one text gave: lines judged, value lines, refused days, failures, held lines.
    #[derive(Default)]
    struct Tally {
        texts: usize,
        lines: usize,
        values: usize,
        refused: usize,
        fails: Vec<String>,
        held: Vec<String>,
        apart: usize,
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Kind {
        Value,
        Hole,
        Refusal,
        Stretch,
    }

    fn text_of(b: &[u8]) -> String {
        String::from_utf8_lossy(b).into_owned()
    }

    fn texts() -> Vec<Text> {
        let parts = netstrings(CORPUS).expect("properties/oracle.corpus reads");
        parts
            .chunks(9)
            .map(|r| {
                let pairs = |b: &[u8]| -> Vec<(String, String)> {
                    text_of(b)
                        .lines()
                        .filter_map(|l| l.split_once(' '))
                        .map(|(a, b)| (a.to_owned(), b.to_owned()))
                        .collect()
                };
                let held_rows = pairs(r[6]);
                let index = |i: &str| -> usize { i.parse().expect("a line's index") };
                Text {
                    id: text_of(r[0]),
                    args: text_of(r[2]).lines().map(str::to_owned).collect(),
                    files: r[3].to_vec(),
                    days: text_of(r[4]).split(' ').map(str::to_owned).collect(),
                    lines: text_of(r[5]).parse().expect("a text's line count"),
                    held: held_rows
                        .iter()
                        .filter(|(i, _)| i != "apart")
                        .map(|(i, code)| (index(i), code.clone()))
                        .collect(),
                    apart: held_rows
                        .iter()
                        .find(|(i, _)| i == "apart")
                        .map(|(_, code)| code.clone()),
                    anchors: pairs(r[7])
                        .into_iter()
                        .map(|(i, day)| (index(&i), day))
                        .collect(),
                    expect: pairs(r[8])
                        .into_iter()
                        .map(|(i, e)| (index(&i), e))
                        .collect(),
                }
            })
            .collect()
    }

    fn digits(s: &str, n: usize) -> bool {
        s.len() == n && s.bytes().all(|b| b.is_ascii_digit())
    }

    /// `lNN: answer` or `lNN_pKEY: answer`: the line's index, its pinned run's key (`YYYYMMDD`, or
    /// two such for two windows), its answer.
    fn head(line: &str) -> Option<(usize, Option<String>, String)> {
        let (name, answer) = line.strip_prefix('l')?.split_once(": ")?;
        let (num, day) = name
            .split_once("_p")
            .map_or((name, None), |(n, d)| (n, Some(d)));
        if !digits(num, 2) || day.is_some_and(|d| !digits(d, 8) && !digits(d, 16)) {
            return None;
        }
        Some((num.parse().ok()?, day.map(str::to_owned), answer.to_owned()))
    }

    fn is_register(line: &str) -> bool {
        line.strip_prefix("  ")
            .filter(|rest| !rest.starts_with(' '))
            .is_some_and(|rest| {
                REGISTER.iter().any(|w| {
                    rest.strip_prefix(w).is_some_and(|after| {
                        !after.starts_with(|c: char| c.is_alphanumeric() || c == '_')
                    })
                })
            })
    }

    /// Each ask's block: its head line's answer and the indented lines under it, the register
    /// left out.
    fn answers(out: &str) -> BTreeMap<(usize, Option<String>), Vec<String>> {
        let mut got: BTreeMap<(usize, Option<String>), Vec<String>> = BTreeMap::new();
        let mut cur: Option<(usize, Option<String>)> = None;
        for line in out.split('\n') {
            if let Some((i, day, answer)) = head(line) {
                got.insert((i, day.clone()), vec![answer]);
                cur = Some((i, day));
            } else if cur.is_some() && is_register(line) {
                continue;
            } else if let Some(block) = cur
                .as_ref()
                .filter(|_| line.starts_with(' '))
                .and_then(|k| got.get_mut(k))
            {
                block.push(line.to_owned());
            } else {
                cur = None;
            }
        }
        got
    }

    fn balance_check(answer: &str) -> bool {
        answer.strip_prefix("refused: ").is_some_and(|rest| {
            BALANCE_CHECKS.iter().any(|code| {
                rest.strip_prefix(code).is_some_and(|after| {
                    !after.starts_with(|c: char| c.is_alphanumeric() || c == '_')
                })
            })
        })
    }

    /// A label list `(…)` with no quote or parenthesis inside, followed by a space or the
    /// line's end, one of whose comma-separated names is a window's cell, `w` or `v2`
    /// (`oracle.py`'s `names_w`).
    fn names_w(u: &str) -> bool {
        u.match_indices('(').any(|(i, _)| {
            u.get(i..)
                .and_then(|s| s.strip_prefix('('))
                .and_then(|after| after.split_once(')'))
                .is_some_and(|(inner, rest)| {
                    !inner.contains('"')
                        && !inner.contains('(')
                        && inner.split(',').any(|x| matches!(x.trim(), "w" | "v2"))
                        && (rest.is_empty() || rest.starts_with(' '))
                })
        })
    }

    fn is_date(v: &str) -> bool {
        let p: Vec<&str> = v.split('-').collect();
        p.len() == 3
            && p.first().is_some_and(|y| digits(y, 4))
            && p.iter().skip(1).all(|x| digits(x, 2))
    }

    fn stretch(u: &str) -> Option<(&str, &str)> {
        let (a, b) = u
            .strip_prefix("window(")?
            .strip_suffix(')')?
            .split_once(", ")?;
        (is_date(a) && is_date(b)).then_some((a, b))
    }

    /// One line against its pinned days: `(why, kind)`, `why` empty where it holds. A hole names
    /// the day by a window among its labels or by citing the text's window (`cite`, C13's
    /// HOLE-PART). A reader's line (`expect`): `list` prints the value where every pinned day
    /// prints one, and a date's stretch from the first pinned print to the last; `off` prints the
    /// hole naming a window, or the refusal every day prints; `false` prints false.
    fn judge(
        u: &str,
        pinned: &[(String, String)],
        refused: &BTreeSet<String>,
        cite: &str,
        expect: &str,
    ) -> (String, Kind) {
        let rest: Vec<&str> = pinned
            .iter()
            .filter(|(d, _)| !refused.contains(d))
            .map(|(_, v)| v.as_str())
            .collect();
        let vals: BTreeSet<&str> = rest.iter().copied().collect();
        let kind = if u.starts_with('?') {
            Kind::Hole
        } else if u.starts_with("refused:") {
            Kind::Refusal
        } else {
            Kind::Value
        };
        // The one value every unrefused pinned day prints, if they agree.
        let only = vals.first().copied().filter(|_| vals.len() == 1);
        match (expect, only) {
            ("false", _) if u != "false" => {
                return (
                    format!("printed {u} where a test of what is known is false"),
                    kind,
                );
            }
            ("false", _) => return (String::new(), kind),
            ("off", _) if !(kind == Kind::Refusal && only == Some(u)) => {
                return if kind == Kind::Hole && names_w(u) {
                    (String::new(), kind)
                } else {
                    (
                        format!("a reader off the list printed {u}, not the hole"),
                        kind,
                    )
                };
            }
            ("list", Some(v)) if v != u && kind == Kind::Hole => {
                return (
                    format!("the hole where every pinned day prints {v}: {u}"),
                    kind,
                );
            }
            _ => {}
        }
        let (lo, hi) = (vals.first().copied(), vals.last().copied());
        match (expect, stretch(u), lo, hi) {
            ("list", Some((a, b)), Some(lo), Some(hi))
                if vals.iter().all(|v| is_date(v)) && (a, b) != (lo, hi) =>
            {
                return (
                    format!("the stretch {u} where the pinned days print {lo} to {hi}"),
                    Kind::Stretch,
                );
            }
            _ => {}
        }
        if only == Some(u) {
            return (String::new(), kind);
        }
        match (kind, stretch(u)) {
            (Kind::Hole, _) if names_w(u) || u.contains(&format!("date within {cite}")) => {
                (String::new(), kind)
            }
            (Kind::Hole, _) => (format!("a hole that names no day: {u}"), kind),
            (Kind::Refusal, _) if pinned.iter().any(|(d, v)| refused.contains(d) && v == u) => {
                (String::new(), kind)
            }
            (Kind::Refusal, _) => (format!("a refusal no pinned day prints: {u}"), kind),
            (_, Some((a, b)))
                if !rest.is_empty() && rest.iter().all(|v| is_date(v) && a <= *v && *v <= b) =>
            {
                (String::new(), Kind::Stretch)
            }
            (_, Some(_)) => (
                format!("a stretch that doesn't hold every pinned day: {u}"),
                Kind::Stretch,
            ),
            (Kind::Value | Kind::Stretch, None) => (
                format!(
                    "printed {u} where the pinned days print {}",
                    vals.iter().copied().collect::<Vec<_>>().join(" | ")
                ),
                Kind::Value,
            ),
        }
    }

    /// A block of several lines: line by line where every block has the same shape.
    fn judge_block(
        u: &[String],
        pinned: &[(String, Vec<String>)],
        cite: &str,
        expect: &str,
    ) -> (String, Kind) {
        let first = |b: &Vec<String>| b.first().cloned().unwrap_or_default();
        let refused: BTreeSet<String> = pinned
            .iter()
            .filter(|(_, b)| balance_check(&first(b)))
            .map(|(d, _)| d.clone())
            .collect();
        let line = |i: usize, trim: bool| -> Vec<(String, String)> {
            pinned
                .iter()
                .map(|(d, b)| {
                    let l = b.get(i).map_or("", String::as_str);
                    (d.clone(), if trim { l.trim() } else { l }.to_owned())
                })
                .collect()
        };
        let u0 = u.first().cloned().unwrap_or_default();
        if u.len() == 1 && pinned.iter().all(|(_, b)| b.len() == 1) {
            return judge(&u0, &line(0, false), &refused, cite, expect);
        }
        let rest: Vec<&Vec<String>> = pinned
            .iter()
            .filter(|(d, _)| !refused.contains(d))
            .map(|(_, b)| b)
            .collect();
        if !rest.is_empty() && rest.iter().all(|b| b.as_slice() == u) {
            return (String::new(), judge(&u0, &[], &refused, cite, "").1);
        }
        if pinned.iter().any(|(_, b)| b.len() != u.len()) {
            return (
                format!(
                    "a block of {} lines where the pinned days differ in shape",
                    u.len()
                ),
                Kind::Value,
            );
        }
        let mut kind = None;
        for (i, ul) in u.iter().enumerate() {
            let (why, k) = judge(ul.trim(), &line(i, true), &refused, cite, "");
            let k0 = *kind.get_or_insert(k);
            if !why.is_empty() {
                return (format!("line {} of the block: {why}", add(i, 1)), k0);
            }
        }
        (String::new(), kind.unwrap_or(Kind::Value))
    }

    /// A count plus `by`; past `usize` the test fails (R-23: no clamping).
    fn add(n: usize, by: usize) -> usize {
        n.checked_add(by)
            .expect("a count of the oracle's lines fits in usize")
    }

    fn judge_text(t: &Text, sum: &mut Tally) {
        let run = run_args(&t.args, &t.files);
        let got = answers(&run.out);
        sum.texts = add(sum.texts, 1);
        if got.is_empty() {
            let first: String = run.out.lines().take(1).collect();
            sum.fails.push(format!(
                "{}: refused at load or printed nothing: {first}",
                t.id
            ));
            return;
        }
        let (mut values, mut fails) = (0_usize, 0_usize);
        for i in 0..t.lines {
            let u = got.get(&(i, None));
            let pinned: Option<Vec<(String, Vec<String>)>> = t
                .days
                .iter()
                .map(|d| {
                    got.get(&(i, Some(d.clone())))
                        .map(|b| (d.clone(), b.clone()))
                })
                .collect();
            let (Some(u), Some(pinned)) = (u, pinned) else {
                sum.fails.push(format!("{} l{i:02}: no line printed", t.id));
                fails = add(fails, 1);
                continue;
            };
            sum.lines = add(sum.lines, 1);
            let nref = pinned
                .iter()
                .filter(|(_, b)| b.first().is_some_and(|l| balance_check(l)))
                .count();
            sum.refused = add(sum.refused, nref);
            // An observation answers the day: the line is held to the pin on its day alone.
            let pinned: Vec<(String, Vec<String>)> = match t.anchors.get(&i) {
                Some(day) => pinned.into_iter().filter(|(d, _)| d == day).collect(),
                None => pinned,
            };
            let expect = t.expect.get(&i).map_or("", String::as_str);
            let (why, kind) = judge_block(u, &pinned, &window_of(t), expect);
            if why.is_empty() && matches!(kind, Kind::Value | Kind::Stretch) {
                values = add(values, 1);
            }
            match (why.is_empty(), t.held.get(&i)) {
                (true, _) => {}
                (false, Some(code)) => sum.held.push(format!("{} l{i:02} ({code}): {why}", t.id)),
                (false, None) => {
                    sum.fails.push(format!("{} l{i:02}: {why}", t.id));
                    fails = add(fails, 1);
                }
            }
        }
        sum.values = add(sum.values, values);
        // A text whose every line is off the list or a test of what is known is held line by line.
        let teeth = (0..t.lines)
            .any(|i| !matches!(t.expect.get(&i).map(String::as_str), Some("off" | "false")));
        match (values == 0 && fails == 0 && teeth, &t.apart) {
            (false, _) => {}
            (true, Some(_)) => sum.apart = add(sum.apart, 1),
            (true, None) => sum
                .fails
                .push(format!("{}: empty, every line a hole or a refusal", t.id)),
        }
    }

    fn oracle_holds(tenth: bool) {
        let all = texts();
        let mut sum = Tally::default();
        for (_, t) in all
            .iter()
            .enumerate()
            .filter(|(i, _)| (i.checked_rem(10) == Some(TENTH)) == tenth)
        {
            judge_text(t, &mut sum);
        }
        assert!(sum.texts > 0, "the oracle's corpus has texts in this part");
        assert!(
            sum.fails.is_empty(),
            "{} of {} lines fail ({} value lines, {} refused days, {} texts apart, {} held by name):\n{}\nheld:\n{}",
            sum.fails.len(),
            sum.lines,
            sum.values,
            sum.refused,
            sum.apart,
            sum.held.len(),
            sum.fails.join("\n"),
            sum.held.join("\n")
        );
    }

    /// The fixed tenth by seed (`oracle.py --sample 10`): what a quick loop takes.
    #[test]
    fn m3_oracle_tenth() {
        oracle_holds(true);
    }

    /// The other nine tenths.
    #[test]
    fn m3_oracle_rest() {
        oracle_holds(false);
    }
}
