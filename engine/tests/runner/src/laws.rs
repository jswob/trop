//! The kernel's six laws (`PROPERTIES.md`), each a check over the runs of one case's
//! variants. The same checks as `properties/laws.py`'s, which proves them on refcore.

use crate::corpus::Run;
use crate::output::{ask_line, blocks, cents, is_hole, values};
use std::collections::BTreeMap;

/// The runs of one case, by variant.
pub type Runs = BTreeMap<String, Run>;

fn run<'a>(runs: &'a Runs, variant: &str) -> Result<&'a Run, String> {
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

/// Determinism: the reality as one file and as the same sections spread over 2–3 files in
/// another order print the same bytes (L §9 "the order of files, and of sections in a file,
/// changes nothing"; R-47). Across runs: the runner runs `one` twice (`DONE` gate 6).
pub fn determinism(runs: &Runs) -> Result<String, String> {
    let (one, split) = (run(runs, "one")?, run(runs, "split")?);
    if one == split {
        Ok("one file and split equal".to_owned())
    } else {
        Err(format!(
            "one file and split differ:\n{}\n---\n{}",
            one.out, split.out
        ))
    }
}

/// Money conserved: every place's flows over the run, outside places included, sum to zero
/// (E §9.1: the two ends of each leg booked whole sum to zero).
pub fn conservation(runs: &Runs) -> Result<String, String> {
    let one = run(runs, "one")?;
    let line = ask_line(&blocks(&one.out), "flows_all")?.to_owned();
    let mut sum: i128 = 0;
    let mut nonzero = 0_usize;
    for v in values(&line) {
        let c = cents(&v).ok_or_else(|| format!("not a sum of DUK: {v} in {line}"))?;
        sum = sum.checked_add(c).ok_or("overflow")?;
        if c != 0 {
            nonzero = nonzero.checked_add(1).ok_or("overflow")?;
        }
    }
    if sum == 0 && nonzero > 0 {
        Ok(format!("{nonzero} nonzero flows sum to 0"))
    } else {
        Err(format!(
            "flows sum to {sum} minor units ({nonzero} nonzero): {line}"
        ))
    }
}

/// No foresight, the cut: an ask `as known D` prints the same with and without a version
/// known after D (L §4.5, the ask's cut K; `DONE` gate 6). Its control, the same ask at the
/// model's `on`, sees the version and differs, so the law can fail.
pub fn foresight_known(runs: &Runs) -> Result<String, String> {
    let (base, later) = (
        blocks(&run(runs, "base")?.out),
        blocks(&run(runs, "later")?.out),
    );
    if base.get("early") != later.get("early") {
        let show = |b: &BTreeMap<String, Vec<String>>| {
            b.get("early").map_or_else(String::new, |l| l.join("\n"))
        };
        return Err(format!(
            "as known D differs:\n{}\n---\n{}",
            show(&base),
            show(&later)
        ));
    }
    if base.get("at_on") == later.get("at_on") {
        return Err(
            "the control at `on` doesn't see the later version: the law is vacuous here".to_owned(),
        );
    }
    Ok("as known D equal; at on, the control, differs".to_owned())
}

/// No foresight, the decision: a movement dated after a decision's day changes nothing it
/// moved (D-5; L §5.14.1: a decision reads the walk as it stands on its day). Its control, the
/// same movement before the decision's day, changes it.
pub fn foresight_decision(runs: &Runs) -> Result<String, String> {
    let line = |v: &str| -> Result<String, String> {
        Ok(ask_line(&blocks(&run(runs, v)?.out), "swept")?.to_owned())
    };
    let (base, later, control) = (line("base")?, line("later")?, line("control")?);
    if values(&base).iter().any(|v| is_hole(v)) {
        return Err(format!("the decision is a hole: {base}"));
    }
    if base != later {
        return Err(format!(
            "a later movement changed the decision: {base} vs {later}"
        ));
    }
    if base == control {
        return Err(
            "the control, an earlier movement, doesn't change the decision: vacuous".to_owned(),
        );
    }
    Ok(format!(
        "{base}; unchanged by a later movement, changed by an earlier one"
    ))
}

/// A hole never becomes 0: with an input's row left out, every value that read it prints a
/// hole, never a number, and `x * 0`, `x - x` and `x + 0` stay holes (L §8.1).
pub fn hole(runs: &Runs) -> Result<String, String> {
    let (base, dropped) = (
        blocks(&run(runs, "base")?.out),
        blocks(&run(runs, "dropped")?.out),
    );
    let mut changed = 0_usize;
    for ask in ["flows_all", "bals", "zero"] {
        let (b, d) = (
            values(ask_line(&base, ask)?),
            values(ask_line(&dropped, ask)?),
        );
        if let Some(v) = b.iter().find(|v| is_hole(v)) {
            return Err(format!("{ask}: a hole with every row present: {v}"));
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
    if let Some(z) = values(ask_line(&dropped, "zero")?)
        .iter()
        .find(|z| !is_hole(z))
    {
        return Err(format!("zero: {z} is not a hole"));
    }
    if changed == 0 {
        return Err("no value read the row: vacuous".to_owned());
    }
    Ok(format!("{changed} values became holes, none a number"))
}

/// The law a corpus record names.
pub fn check(law: &str, runs: &Runs) -> Result<String, String> {
    match law {
        "determinism" => determinism(runs),
        "conservation" => conservation(runs),
        "foresight-known" => foresight_known(runs),
        "foresight-decision" | "foresight-projection" => foresight_decision(runs),
        "hole" => hole(runs),
        other => Err(format!("no law {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::{Runs, check};
    use crate::corpus::{KERNEL_CORPUS, KERNEL_REFCORE, Run, records, refcore_runs};
    use crate::output::values;
    use std::collections::BTreeMap;

    /// Every case's runs on refcore, by (law, case).
    fn refcore_cases() -> BTreeMap<(String, String), Runs> {
        let recs = records(KERNEL_CORPUS).expect("the corpus reads");
        let runs = refcore_runs(KERNEL_REFCORE).expect("refcore's runs read");
        let mut out: BTreeMap<(String, String), Runs> = BTreeMap::new();
        for r in recs {
            let got = runs
                .get(&(r.case.clone(), r.variant.clone()))
                .cloned()
                .expect("refcore ran every record");
            out.entry((r.law, r.case))
                .or_insert_with(BTreeMap::new)
                .insert(r.variant, got);
        }
        out
    }

    #[test]
    fn every_law_holds_on_refcore() {
        let cases = refcore_cases();
        assert_eq!(cases.len(), 144, "six laws, 24 cases each");
        for ((law, case), runs) in &cases {
            assert!(check(law, runs).is_ok(), "{case}: {:?}", check(law, runs));
        }
    }

    /// Changes one refcore output the way a broken engine would, and expects the law to fail.
    fn planted(law: &str, variant: &str, edit: fn(&str) -> String) -> bool {
        let cases = refcore_cases();
        let (_, runs) = cases
            .iter()
            .find(|((l, _), _)| l == law)
            .expect("a case of the law");
        let mut runs = runs.clone();
        let r = runs.get(variant).cloned().expect("the variant");
        runs.insert(
            variant.to_owned(),
            Run {
                out: edit(&r.out),
                code: r.code,
            },
        );
        check(law, &runs).is_err()
    }

    /// `out` with the first value of ask `ask`'s line replaced by `new`.
    fn first_value(out: &str, ask: &str, new: &str) -> String {
        let prefix = format!("{ask}: ");
        let lines: Vec<String> = out
            .split('\n')
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
            .collect();
        lines.join("\n")
    }

    #[test]
    fn each_law_fails_on_a_planted_fault() {
        assert!(
            planted("determinism", "split", |o| o.replacen("DUK", "DUK ", 1)),
            "a changed byte"
        );
        assert!(
            planted("conservation", "one", |o| first_value(
                o,
                "flows_all",
                "-999 999.99 DUK"
            )),
            "a lost leg end"
        );
        assert!(
            planted("foresight-known", "later", |o| o.replacen(
                "early: (",
                "early: (?\"x\" (y), ",
                1
            )),
            "foresight"
        );
        assert!(
            planted("foresight-decision", "later", |o| first_value(
                o, "swept", "0.01 DUK"
            )),
            "a decision that saw ahead"
        );
        assert!(
            planted("hole", "dropped", |o| first_value(o, "zero", "0.00 DUK")),
            "a hole printed as 0"
        );
        assert!(
            planted("hole", "dropped", |o| first_value(
                o,
                "flows_all",
                "0.00 DUK"
            )),
            "a hole's flow as 0"
        );
        let exit = |code: u8| Run {
            out: "q: 1\n".to_owned(),
            code,
        };
        let runs: Runs = [("one".to_owned(), exit(69)), ("split".to_owned(), exit(69))]
            .into_iter()
            .collect();
        assert!(
            check("determinism", &runs).is_err(),
            "the stub's exit 69 never passes"
        );
    }
}
