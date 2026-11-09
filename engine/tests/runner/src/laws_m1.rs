//! M1's laws (`PROPERTIES.md` § M1), each a check over the runs of one case's variants and,
//! for two of them, the case's text: the steps law's oracle is in its model's note, and the
//! scenario law reads whether the case plants a movement E §9 item 3 lists. The same checks
//! as `properties/laws_m1.py`'s, which proves them on refcore.

use crate::corpus::{Run, netstrings};
use crate::laws::Runs;
use crate::output::{ask_line, blocks, cents, is_hole, values};
use std::collections::BTreeMap;

/// The corpus `properties/gen_m1.py` writes: every record of M1's thirteen laws.
pub const M1_CORPUS: &[u8] = include_bytes!("../../properties/m1.corpus");

/// refcore's output and exit code for every record of [`M1_CORPUS`], from
/// `properties/laws_m1.py`.
pub const M1_REFCORE: &[u8] = include_bytes!("../../properties/m1.refcore");

/// The target `held.tsv` names for a case refcore breaks inside a filed issue.
pub const TARGET: &str = "m1_properties";

type Blocks = BTreeMap<String, Vec<String>>;

fn answered<'a>(runs: &'a Runs, variant: &str) -> Result<&'a Run, String> {
    let r = runs
        .get(variant)
        .ok_or_else(|| format!("no {variant} run"))?;
    if r.code == 0 {
        Ok(r)
    } else {
        let first = r
            .out
            .split_once('\n')
            .map_or(r.out.as_str(), |(line, _)| line);
        Err(format!("{variant}: exit {}, not answered: {first}", r.code))
    }
}

fn line_values(b: &Blocks, ask: &str) -> Result<Vec<String>, String> {
    Ok(values(ask_line(b, ask)?))
}

fn first_value(b: &Blocks, ask: &str) -> Result<String, String> {
    line_values(b, ask)?
        .into_iter()
        .next()
        .ok_or_else(|| format!("{ask}: no value"))
}

fn hole_or_refused(v: &str) -> bool {
    is_hole(v) || v.starts_with("refused")
}

/// The contents of a record's files, joined by newlines, in the file table's order.
pub fn texts(files: &[u8]) -> Result<String, String> {
    let mut out = Vec::new();
    // the table is path, content, path, content, …
    let mut is_content = false;
    for part in netstrings(files)? {
        if is_content {
            out.push(std::str::from_utf8(part).map_err(|_| "a file isn't UTF-8".to_owned())?);
        }
        is_content = !is_content;
    }
    Ok(out.join("\n"))
}

/// Determinism: the reality as one file and as the same sections over 2–3 files in another
/// order print the same bytes (L §9; R-47).
pub fn determinism(runs: &Runs) -> Result<String, String> {
    if answered(runs, "one")? == answered(runs, "split")? {
        Ok("one file and split equal".to_owned())
    } else {
        Err("one file and split differ".to_owned())
    }
}

/// Money conserved: every place's flows over the run, outside places included, sum to zero
/// (E §9.1), with prorata splits, plans over keys and decisions.
pub fn conservation(runs: &Runs) -> Result<String, String> {
    let b = blocks(&answered(runs, "one")?.out);
    let mut sum: i128 = 0;
    let mut nonzero = 0_usize;
    for v in line_values(&b, "flows_all")? {
        let c = cents(&v).ok_or_else(|| format!("not a sum of DUK: {v}"))?;
        sum = sum.checked_add(c).ok_or("overflow")?;
        if c != 0 {
            nonzero = nonzero.checked_add(1).ok_or("overflow")?;
        }
    }
    if sum == 0 && nonzero > 0 {
        Ok(format!("{nonzero} nonzero flows sum to 0"))
    } else {
        Err(format!(
            "flows sum to {sum} minor units ({nonzero} nonzero)"
        ))
    }
}

/// The worths `nw_0`, `nw_1`, … in their order.
fn worths(out: &str) -> Result<Vec<i128>, String> {
    let mut keyed: BTreeMap<u32, i128> = BTreeMap::new();
    for (name, lines) in &blocks(out) {
        let Some(k) = name.strip_prefix("nw_") else {
            continue;
        };
        let k: u32 = k.parse().map_err(|_| format!("the ask {name}"))?;
        let line = lines.first().ok_or("an empty block")?;
        let c = values(line)
            .first()
            .and_then(|v| cents(v))
            .ok_or_else(|| format!("not a sum of DUK: {line}"))?;
        keyed.insert(k, c);
    }
    Ok(keyed.into_values().collect())
}

/// Net worth moves only at the boundary (E §9.2): an inside transfer changes the worth of the
/// places that aren't outside on no day; a movement across the boundary changes it by one
/// amount from its day on.
pub fn networth(runs: &Runs) -> Result<String, String> {
    let base = worths(&answered(runs, "base")?.out)?;
    let inside = worths(&answered(runs, "inside")?.out)?;
    let boundary = worths(&answered(runs, "boundary")?.out)?;
    if base.is_empty() || base.len() != boundary.len() {
        return Err("no worths, or not as many in each variant".to_owned());
    }
    if base != inside {
        return Err(format!(
            "an inside transfer moved the worth: {base:?} vs {inside:?}"
        ));
    }
    let d: Vec<i128> = base
        .iter()
        .zip(&boundary)
        .map(|(a, b)| b.checked_sub(*a).ok_or_else(|| "overflow".to_owned()))
        .collect::<Result<_, _>>()?;
    let j = d
        .iter()
        .position(|x| *x != 0)
        .ok_or("the boundary move changed nothing")?;
    let step = *d.get(j).ok_or("no step")?;
    if j == 0 || step <= 0 || d.iter().skip(j).any(|x| *x != step) {
        return Err(format!(
            "the boundary move changed the worth by {d:?}, not by one amount from its day on"
        ));
    }
    Ok(format!("inside: unchanged; boundary: +{step} from day {j}"))
}

/// A scenario changes nothing before its `from` that it doesn't list (E §9.3, L §6.4; P43's
/// `before from:` lines): a case that plants a movement before `from` read at a law date
/// after it (`amt_mv_l` in its text) lists it; any other prints every value before `from` as
/// the base does. After `from` the scenario changes something, so the law can fail.
pub fn scenario_before(runs: &Runs, text: &str) -> Result<String, String> {
    let b = blocks(&answered(runs, "one")?.out);
    let listed: Vec<&String> = b
        .get("pre_s")
        .ok_or("no pre_s block")?
        .iter()
        .filter(|l| l.starts_with("  before from: "))
        .collect();
    let (pre_m, pre_s) = (line_values(&b, "pre_m")?, line_values(&b, "pre_s")?);
    let after = |ask: &str| -> Result<String, String> {
        Ok(ask_line(&b, ask)?
            .split_once(": ")
            .map_or("", |(_, v)| v)
            .to_owned())
    };
    if after("post_m")? == after("post_s")? {
        return Err("the scenario changes nothing after its from: vacuous".to_owned());
    }
    if pre_m.iter().chain(&pre_s).any(|v| hole_or_refused(v)) {
        return Err("a value before from is a hole or a refusal".to_owned());
    }
    if listed.iter().any(|l| l.starts_with("  before from: mv_l ")) {
        return Ok(format!("listed: {}", listed.len()));
    }
    if text.contains("amt_mv_l") {
        return Err("mv_l reads the scenario before from, and no line lists it".to_owned());
    }
    if !listed.is_empty() {
        return Err(format!("listed with nothing to list: {listed:?}"));
    }
    if pre_m == pre_s {
        Ok("every value before from is the base's".to_owned())
    } else {
        Err(format!("before from: {pre_m:?} vs {pre_s:?}"))
    }
}

/// The projection asks' blocks (P6): `p_bal` and `p_flows_<place>`.
fn projection(out: &str) -> Blocks {
    blocks(out)
        .into_iter()
        .filter(|(k, _)| k.starts_with("p_"))
        .collect()
}

/// Asks move no money (E §9.4): the projection asks print the same alone and beside an
/// `as if`, a `with`, a `solve` and a difference; a planted move changes them.
pub fn asks_no_money(runs: &Runs) -> Result<String, String> {
    let alone = projection(&answered(runs, "alone")?.out);
    let beside = projection(&answered(runs, "beside")?.out);
    let control = projection(&answered(runs, "control")?.out);
    if alone.is_empty() {
        return Err("no projection asks".to_owned());
    }
    if alone != beside {
        return Err("the projection differs beside other asks".to_owned());
    }
    if alone == control {
        return Err("the planted move doesn't change the projection: vacuous".to_owned());
    }
    Ok("equal beside other asks; a planted move changes it".to_owned())
}

/// The same beside `explain` (L §5.18, E §10).
pub fn asks_no_money_explain(runs: &Runs) -> Result<String, String> {
    let alone = projection(&answered(runs, "alone")?.out);
    let explained = answered(runs, "explained")?;
    ask_line(&blocks(&explained.out), "k_explain")?;
    if !alone.is_empty() && alone == projection(&explained.out) {
        Ok("equal beside explain".to_owned())
    } else {
        Err("the projection differs beside explain".to_owned())
    }
}

/// Things hold nothing outside their lifespans (E §9.6, L §5.7): inside, the place holds 0
/// before its lifespan, the amount inside it and 0 after it; a leg before or after it is
/// refused (`RUN-LIFESPAN`) wherever the place's balance is read.
pub fn lifespans(runs: &Runs) -> Result<String, String> {
    let inside = line_values(&blocks(&answered(runs, "inside")?.out), "life")?;
    let c: Vec<Option<i128>> = inside.iter().map(|v| cents(v)).collect();
    if c.first() != Some(&Some(0))
        || c.get(2) != Some(&Some(0))
        || matches!(c.get(1), None | Some(None | Some(0)))
    {
        return Err(format!(
            "inside: {inside:?}, not 0, the amount, 0 before, during, after"
        ));
    }
    for v in ["early", "late"] {
        let out = &answered(runs, v)?.out;
        if !out.contains("RUN-LIFESPAN") {
            return Err(format!("{v}: a leg outside the lifespan isn't refused"));
        }
        // before the lifespan and after it; the middle day is inside it
        let life = line_values(&blocks(out), "life")?;
        for x in [life.first(), life.get(2)].into_iter().flatten() {
            if !(x.starts_with("refused: RUN-LIFESPAN") || cents(x) == Some(0)) {
                return Err(format!("{v}: the place holds {x} outside its lifespan"));
            }
        }
    }
    Ok("0 outside the lifespan, RUN-LIFESPAN for a leg outside it".to_owned())
}

/// No foresight in a decision (D-5, D-6, L §5.14.1): a keyed decision's instances up to j, or a
/// decision reading `low` through its reach, are unchanged by a movement dated after them; the
/// control, a movement before, changes them.
pub fn foresight(runs: &Runs) -> Result<String, String> {
    let line = |v: &str| -> Result<String, String> {
        Ok(ask_line(&blocks(&answered(runs, v)?.out), "swept")?.to_owned())
    };
    let (base, later, control) = (line("base")?, line("later")?, line("control")?);
    if values(&base).iter().any(|v| hole_or_refused(v)) {
        return Err(format!("the decision is a hole or refused: {base}"));
    }
    if base != later {
        return Err(format!("a later movement changed it: {base} vs {later}"));
    }
    if base == control {
        return Err("the control doesn't change the decision: vacuous".to_owned());
    }
    Ok(format!("{base}; unchanged by a later movement"))
}

/// The asks of the hole law, in `gen_m1.py`'s `hole_asks`.
const HOLE_ASKS: [&str; 8] = [
    "flows_all",
    "bals",
    "v_share",
    "v_fee",
    "v_agg",
    "v_cmp",
    "v_if",
    "zero",
];

/// A hole never becomes 0 (L §8.1): with one row left out, every value that changed is a hole,
/// through rules, a band, prorata, round, aggregates, `if` and comparisons, and `x * 0`,
/// `x - x`, `x + 0` stay holes; with the row, every value is a number.
pub fn hole(runs: &Runs) -> Result<String, String> {
    let base = blocks(&answered(runs, "base")?.out);
    let dropped = blocks(&answered(runs, "dropped")?.out);
    let mut changed = 0_usize;
    for ask in HOLE_ASKS {
        let (b, d) = (line_values(&base, ask)?, line_values(&dropped, ask)?);
        if let Some(v) = b.iter().find(|v| hole_or_refused(v)) {
            return Err(format!("{ask}: {v} with every row present"));
        }
        if b.len() != d.len() {
            return Err(format!("{ask}: {} values, then {}", b.len(), d.len()));
        }
        for (x, y) in b.iter().zip(&d) {
            if x != y {
                changed = changed.checked_add(1).ok_or("overflow")?;
                if !is_hole(y) {
                    return Err(format!("{ask}: {x} became {y}, not a hole"));
                }
            }
        }
    }
    if let Some(z) = line_values(&dropped, "zero")?.iter().find(|z| !is_hole(z)) {
        return Err(format!("zero: {z} is not a hole"));
    }
    if changed == 0 {
        return Err("no value read the row: vacuous".to_owned());
    }
    Ok(format!("{changed} values became holes, none a number"))
}

/// The same through `explain`: the explained balance prints the hole.
pub fn hole_explain(runs: &Runs) -> Result<String, String> {
    let base = first_value(&blocks(&answered(runs, "base")?.out), "ex")?;
    let dropped = first_value(&blocks(&answered(runs, "dropped")?.out), "ex")?;
    if !is_hole(&base) && is_hole(&dropped) {
        Ok(format!("{base}, then a hole"))
    } else {
        Err(format!("explained: {base}, then {dropped}"))
    }
}

/// The steps law's oracle, written by `gen_m1.py` into the model's note: (ask, what it prints,
/// or the code it is refused with).
fn oracle(text: &str) -> Vec<(String, String)> {
    text.split_once("note \"steps oracle: ")
        .and_then(|(_, rest)| rest.split_once('"'))
        .map_or_else(Vec::new, |(o, _)| {
            o.split("; ")
                .filter_map(|kv| kv.split_once('='))
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect()
        })
}

/// A key moved by whole steps lands on a key (#98's class; L §4.1, §4.2, §5.6): each ask
/// prints what the generator's datetime oracle says, or is refused with its code.
pub fn steps(runs: &Runs, text: &str) -> Result<String, String> {
    let b = blocks(&answered(runs, "one")?.out);
    let want = oracle(text);
    if want.is_empty() {
        return Err("no oracle in the text".to_owned());
    }
    let mut wrong = Vec::new();
    for (ask, w) in &want {
        let got = first_value(&b, ask)?;
        let good = if w == "RUN-KEY" || w == "RUN-DOMAIN" {
            got.starts_with(&format!("refused: {w} "))
        } else {
            got == *w
        };
        if !good {
            wrong.push(format!("{ask}: {got}, the oracle {w}"));
        }
    }
    if wrong.is_empty() {
        Ok(format!("all {} asks as the oracle", want.len()))
    } else {
        Err(wrong.join("; "))
    }
}

/// `low(x, a .. b) from t` is the minimum over d in a..b of `bal(x, d) from t` (L §5.14.1).
pub fn low_min(runs: &Runs) -> Result<String, String> {
    let b = blocks(&answered(runs, "one")?.out);
    let (lo, mn) = (first_value(&b, "lo")?, first_value(&b, "mn")?);
    if lo == mn && cents(&lo).is_some() {
        Ok(format!("low {lo}, the minimum over the days"))
    } else {
        Err(format!("low {lo}, the minimum over the days {mn}"))
    }
}

/// The law a corpus record names, over its case's runs and text.
pub fn check(law: &str, runs: &Runs, text: &str) -> Result<String, String> {
    match law {
        "determinism" => determinism(runs),
        "conservation" => conservation(runs),
        "networth" => networth(runs),
        "scenario-before" => scenario_before(runs, text),
        "asks-no-money" => asks_no_money(runs),
        "asks-no-money-explain" => asks_no_money_explain(runs),
        "lifespans" => lifespans(runs),
        "foresight-keyed" | "foresight-low" => foresight(runs),
        "hole" => hole(runs),
        "hole-explain" => hole_explain(runs),
        "steps" => steps(runs, text),
        "low-min" => low_min(runs),
        other => Err(format!("no law {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::{M1_CORPUS, M1_REFCORE, TARGET, check, texts};
    use crate::corpus::{Run, records, refcore_runs};
    use crate::held::is_held;
    use crate::laws::Runs;
    use crate::output::values;
    use std::collections::BTreeMap;

    /// One case: its law, its runs on refcore and its text.
    type Case = (String, Runs, String);

    fn refcore_cases() -> BTreeMap<String, Case> {
        let runs = refcore_runs(M1_REFCORE).expect("refcore's runs read");
        let mut out: BTreeMap<String, Case> = BTreeMap::new();
        for r in records(M1_CORPUS).expect("the corpus reads") {
            let got = runs
                .get(&(r.case.clone(), r.variant.clone()))
                .cloned()
                .expect("refcore ran every record");
            let text = texts(&r.files).expect("the files read");
            let entry = out
                .entry(r.case.clone())
                .or_insert_with(|| (r.law.clone(), BTreeMap::new(), String::new()));
            entry.1.insert(r.variant, got);
            entry.2.push_str(&text);
        }
        out
    }

    #[test]
    fn every_m1_law_holds_on_refcore() {
        let cases = refcore_cases();
        assert_eq!(cases.len(), 312, "thirteen laws, 24 cases each");
        for (case, (law, runs, text)) in &cases {
            let got = check(law, runs, text);
            if is_held(TARGET, case) {
                assert!(
                    got.is_err(),
                    "{case} holds on refcore: drop its held.tsv row"
                );
            } else {
                assert!(got.is_ok(), "{case}: {got:?}");
            }
        }
    }

    /// `out` with the first value of ask `ask`'s line replaced by `new`.
    fn edit_first(out: &str, ask: &str, new: &str) -> String {
        let prefix = format!("{ask}: ");
        out.split('\n')
            .map(|l| {
                if !l.starts_with(&prefix) {
                    return l.to_owned();
                }
                let mut v = values(l);
                if let Some(x) = v.first_mut() {
                    new.clone_into(x);
                }
                if v.len() == 1 {
                    format!("{prefix}{new}")
                } else {
                    format!("{prefix}({})", v.join(", "))
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Changes one variant's output of the first case of `law` that holds, as a broken engine
    /// would, and reports whether the law then fails.
    fn planted(law: &str, variant: &str, ask: &str, new: &str) -> bool {
        let cases = refcore_cases();
        let (_, (_, runs, text)) = cases
            .iter()
            .find(|(case, (l, runs, text))| {
                l == law && !is_held(TARGET, case) && check(l, runs, text).is_ok()
            })
            .expect("a case of the law that holds");
        let mut runs = runs.clone();
        let r = runs.get(variant).cloned().expect("the variant");
        runs.insert(
            variant.to_owned(),
            Run {
                out: edit_first(&r.out, ask, new),
                code: r.code,
            },
        );
        check(law, &runs, text).is_err()
    }

    #[test]
    fn each_m1_law_fails_on_a_planted_fault() {
        let faults = [
            ("determinism", "split", "flows_all", "1.00 DUK"),
            ("conservation", "one", "flows_all", "-999 999.99 DUK"),
            ("networth", "inside", "nw_0", "0.01 DUK"),
            ("scenario-before", "one", "pre_s", "0.01 DUK"),
            ("asks-no-money", "beside", "p_bal", "0.01 DUK"),
            ("asks-no-money-explain", "explained", "p_bal", "0.01 DUK"),
            ("lifespans", "early", "life", "5.00 DUK"),
            ("foresight-keyed", "later", "swept", "0.01 DUK"),
            ("foresight-low", "later", "swept", "0.01 DUK"),
            ("hole", "dropped", "zero", "0.00 DUK"),
            ("hole-explain", "dropped", "ex", "0.00 DUK"),
            ("steps", "one", "a_key", "999"),
            ("low-min", "one", "lo", "-0.01 DUK"),
        ];
        for (law, variant, ask, new) in faults {
            assert!(
                planted(law, variant, ask, new),
                "{law}: a planted fault passes"
            );
        }
        let stub = Run {
            out: "tally run: not implemented\n".to_owned(),
            code: 69,
        };
        let runs: Runs = [("one".to_owned(), stub.clone()), ("split".to_owned(), stub)]
            .into_iter()
            .collect();
        assert!(
            check("determinism", &runs, "").is_err(),
            "an unanswered run never passes"
        );
    }
}
