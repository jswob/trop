//! M2's laws (`PROPERTIES.md` § M2), each a check over the runs of one case's variants: the
//! masking law at K, the three-way split's replay and the fixed past. The same checks as
//! `properties/laws_m2.py`'s, which proves them on refcore.

use crate::corpus::Run;
use crate::laws::Runs;
use crate::output::blocks;
use std::collections::BTreeMap;

/// The corpus `properties/gen_m2.py` writes: every record of M2's three laws.
pub const M2_CORPUS: &[u8] = include_bytes!("../../properties/m2.corpus");

/// refcore's output and exit code for every record of [`M2_CORPUS`], from
/// `properties/laws_m2.py`.
pub const M2_REFCORE: &[u8] = include_bytes!("../../properties/m2.refcore");

/// The target `held.tsv` names for a case refcore breaks inside a filed issue.
pub const TARGET: &str = "m2_properties";

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

/// The asks of `a` whose name starts with `prefix`, sorted.
fn named(a: &Blocks, prefix: &str) -> Vec<String> {
    a.keys()
        .filter(|n| n.starts_with(prefix))
        .cloned()
        .collect()
}

/// The asks among `names` that print otherwise in `a` and `b`.
fn moved(a: &Blocks, b: &Blocks, names: &[String]) -> Vec<String> {
    names
        .iter()
        .filter(|n| a.get(*n) != b.get(*n))
        .cloned()
        .collect()
}

/// Masking at K (E §2): every `k_` ask (`as known K`) prints the same with and without the
/// row known after K; control: an `o_` ask (at `on`) sees it.
pub fn masking(runs: &Runs) -> Result<String, String> {
    let a = blocks(&answered(runs, "base")?.out);
    let b = blocks(&answered(runs, "late")?.out);
    let (k, o) = (named(&a, "k_"), named(&a, "o_"));
    if k.is_empty() || o.is_empty() {
        return Err("no as-known or control asks".to_owned());
    }
    let differ = moved(&a, &b, &k);
    if !differ.is_empty() {
        return Err(format!("as known K the late row moved {differ:?}"));
    }
    let seen = moved(&a, &b, &o);
    if seen.is_empty() {
        return Err("control: at on the late row moved nothing".to_owned());
    }
    Ok(format!(
        "{} reads as known K unchanged; at on {seen:?} moved",
        k.len()
    ))
}

/// The amount differences of the replay's matched pay lines (ask `rep`).
fn pay_differences(out: &str) -> Vec<String> {
    let b = blocks(out);
    let Some(lines) = b.get("rep") else {
        return Vec::new();
    };
    lines
        .iter()
        .filter(|l| l.starts_with("    matched pay["))
        .filter_map(|l| {
            let (_, rest) = l.split_once(", difference ")?;
            let (diff, _) = rest.split_once(", timing ")?;
            Some(diff.to_owned())
        })
        .collect()
}

/// The three-way split's replay (E §6.2, §6.3): every matched pay line, a law-computed
/// instance, has amount difference 0.00; control: the twin whose law rounds down shows one.
pub fn split(runs: &Runs) -> Result<String, String> {
    let model = pay_differences(&answered(runs, "model")?.out);
    let bug = pay_differences(&answered(runs, "bug")?.out);
    if model.len() != 6 || bug.len() != 6 {
        return Err(format!(
            "not six matched pay lines: {}, {}",
            model.len(),
            bug.len()
        ));
    }
    if model.iter().any(|d| d != "0.00 DUK") {
        return Err(format!("the replay shows a law difference: {model:?}"));
    }
    let shown = bug.iter().filter(|d| *d != "0.00 DUK").count();
    if shown == 0 {
        return Err("control: the rounding bug shows nothing".to_owned());
    }
    Ok(format!("six pay lines 0.00; the bug shows {shown}"))
}

/// The fixed past (L §5.14.3, E §5): a version known on D leaves every read before D as
/// without it; control: known before the first booking, it moves one.
pub fn fixed_past(runs: &Runs) -> Result<String, String> {
    let a = blocks(&answered(runs, "base")?.out);
    let b = blocks(&answered(runs, "late")?.out);
    let c = blocks(&answered(runs, "early")?.out);
    let names: Vec<String> = a.keys().cloned().collect();
    if names.is_empty() {
        return Err("no asks".to_owned());
    }
    let differ = moved(&a, &b, &names);
    if !differ.is_empty() {
        return Err(format!(
            "the version known later moved {differ:?} before its known day"
        ));
    }
    let seen = moved(&a, &c, &names);
    if seen.is_empty() {
        return Err("control: the version known before the bookings moved nothing".to_owned());
    }
    Ok(format!(
        "{} reads before D unchanged; known early, {seen:?} moved",
        names.len()
    ))
}

/// The law a corpus record names, over its case's runs.
pub fn check(law: &str, runs: &Runs) -> Result<String, String> {
    match law {
        "masking" => masking(runs),
        "split" => split(runs),
        "fixed-past" => fixed_past(runs),
        other => Err(format!("no law {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::{M2_CORPUS, M2_REFCORE, TARGET, check};
    use crate::corpus::{Run, records, refcore_runs};
    use crate::held::is_held;
    use crate::laws::Runs;
    use std::collections::BTreeMap;

    fn refcore_cases() -> BTreeMap<String, (String, Runs)> {
        let runs = refcore_runs(M2_REFCORE).expect("refcore's runs read");
        let mut out: BTreeMap<String, (String, Runs)> = BTreeMap::new();
        for r in records(M2_CORPUS).expect("the corpus reads") {
            let got = runs
                .get(&(r.case.clone(), r.variant.clone()))
                .cloned()
                .expect("refcore ran every record");
            out.entry(r.case.clone())
                .or_insert_with(|| (r.law.clone(), BTreeMap::new()))
                .1
                .insert(r.variant, got);
        }
        out
    }

    #[test]
    fn every_m2_law_holds_on_refcore() {
        let cases = refcore_cases();
        assert_eq!(cases.len(), 72, "three laws, 24 cases each");
        for (case, (law, runs)) in &cases {
            let got = check(law, runs);
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

    /// Replaces `from` by `to` in the first line holding `marker` of one variant's output of
    /// the first case of `law` that holds, as a broken engine would, and reports whether the
    /// law then fails.
    fn planted(law: &str, variant: &str, (marker, from, to): (&str, &str, &str)) -> bool {
        let cases = refcore_cases();
        let (_, (_, runs)) = cases
            .iter()
            .find(|(case, (l, runs))| l == law && !is_held(TARGET, case) && check(l, runs).is_ok())
            .expect("a case of the law that holds");
        let mut runs = runs.clone();
        let r = runs.get(variant).cloned().expect("the variant");
        let mut done = false;
        let out = r
            .out
            .split('\n')
            .map(|l| {
                if done || !l.contains(marker) {
                    return l.to_owned();
                }
                done = true;
                l.replacen(from, to, 1)
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert_ne!(out, r.out, "{law}: the fault is planted");
        runs.insert(variant.to_owned(), Run { out, code: r.code });
        check(law, &runs).is_err()
    }

    #[test]
    fn each_m2_law_fails_on_a_planted_fault() {
        let faults = [
            ("masking", "late", ("k_net: ", "k_net: ", "k_net: 1")),
            (
                "split",
                "model",
                ("    matched pay[", "difference 0.00", "difference 1.00"),
            ),
            ("fixed-past", "late", ("f_pre: ", "f_pre: ", "f_pre: 1")),
        ];
        for (law, variant, fault) in faults {
            assert!(
                planted(law, variant, fault),
                "{law}: a planted fault passes"
            );
        }
        let stub = Run {
            out: "tally run: not implemented\n".to_owned(),
            code: 69,
        };
        let runs: Runs = [("base".to_owned(), stub.clone()), ("late".to_owned(), stub)]
            .into_iter()
            .collect();
        assert!(
            check("masking", &runs).is_err(),
            "an unanswered run never passes"
        );
    }
}
