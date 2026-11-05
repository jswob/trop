//! M0's laws (`PROPERTIES.md` § M0), each a check over the runs of one case's variants, and
//! the runs a corpus record asks for: the same checks as `properties/laws_m0.py`'s, which
//! proves them on refcore's outputs (`properties/m0.refcore`). A record runs as the load
//! probe (its own argument words), as `reprint` of its files and, when that reprint exits 0,
//! as `reprint` of the canonical sections it printed (`again`); its runs are keyed
//! `<variant>:probe`, `<variant>:reprint` and `<variant>:again`. `m0.refcore` holds no
//! `again` run: `laws_m0.py` checks that each is its reprint's bytes.

use crate::corpus::{Record, Run, pack, paths};
use crate::laws::Runs;
use crate::m0::printed;

/// The corpus `properties/gen_m0.py` writes: every record of M0's four laws.
pub const M0_CORPUS: &[u8] = include_bytes!("../../properties/m0.corpus");

/// refcore's runs of every record of [`M0_CORPUS`], from `properties/laws_m0.py`.
pub const M0_REFCORE: &[u8] = include_bytes!("../../properties/m0.refcore");

/// What the load probe prints after a clean load (E §11: no ask of that name).
pub const PROBED: &str = "refused: REQ-ASK no ask zz_m0_probe\n";

/// A record's reprint: `reprint` and its files, in the table's order.
pub fn reprint_args(record: &Record) -> Result<Vec<String>, String> {
    let mut args = vec!["reprint".to_owned()];
    args.extend(paths(&record.files)?);
    Ok(args)
}

/// The run that reprints a reprint's canonical sections, each under the path it printed:
/// its argument words and its file table; `None` when the reprint was refused.
pub fn again_of(reprint: &Run) -> Option<(Vec<String>, Vec<u8>)> {
    if reprint.code != 0 {
        return None;
    }
    let sections = printed(&reprint.out).sections;
    let mut args = vec!["reprint".to_owned()];
    args.extend(sections.iter().map(|(path, _)| path.clone()));
    let files: Vec<(&str, &str)> = sections
        .iter()
        .map(|(path, text)| (path.as_str(), text.as_str()))
        .collect();
    Some((args, pack(&files)))
}

/// The code of an output's last line, `refused: <code> …`: upper-case letters, a `-`, then
/// upper-case letters, digits and `-`.
pub fn code(out: &str) -> Option<&str> {
    let last = out.trim_end_matches('\n').rsplit('\n').next()?;
    let word = last.strip_prefix("refused: ")?.split(' ').next()?;
    let (head, tail) = word.split_once('-')?;
    let upper = |c: char| c.is_ascii_uppercase();
    let ok = !head.is_empty()
        && head.chars().all(upper)
        && !tail.is_empty()
        && tail
            .chars()
            .all(|c| upper(c) || c.is_ascii_digit() || c == '-');
    ok.then_some(word)
}

fn get<'a>(runs: &'a Runs, key: &str) -> Result<&'a Run, String> {
    runs.get(key).ok_or_else(|| format!("no {key} run"))
}

/// The variant's probe printed the REQ-ASK line alone: it loads clean.
fn clean(runs: &Runs, variant: &str) -> bool {
    runs.get(&format!("{variant}:probe"))
        .is_some_and(|r| r.code == 1 && r.out == PROBED)
}

fn first_line(out: &str) -> &str {
    out.split_once('\n').map_or(out, |(line, _)| line)
}

/// Spelling: a text and its rewrites that the grammar allows and L §2.4 makes equal print
/// the same canonical text (L §2.4 "equal models print identically", N-1902); the control,
/// a figure changed, prints another.
pub fn spelling(runs: &Runs) -> Result<String, String> {
    let base = get(runs, "base:reprint")?;
    if base.code != 0 || !clean(runs, "base") {
        return Err(format!(
            "the base does not load clean: {}",
            first_line(&base.out)
        ));
    }
    let mut count = 0_usize;
    let mut differ = Vec::new();
    for (key, run) in runs {
        let Some(variant) = key.strip_suffix(":reprint") else {
            continue;
        };
        if variant == "base" || variant == "control" {
            continue;
        }
        count = count.checked_add(1).ok_or("overflow")?;
        if run != base || !clean(runs, variant) {
            differ.push(variant);
        }
    }
    if count == 0 || !differ.is_empty() {
        return Err(format!(
            "{count} spellings, these print otherwise than the base: {}",
            differ.join(", ")
        ));
    }
    let control = get(runs, "control:reprint")?;
    if control == base || !clean(runs, "control") {
        return Err("the control prints as the base: vacuous".to_owned());
    }
    Ok(format!(
        "{count} spellings print as the base; control differs"
    ))
}

/// Order: the text as one file and split over two load alike, and the two files in either
/// order reprint the same (L §9 "the order of files … changes nothing"); the control, a
/// figure changed in the second file, reprints otherwise.
pub fn order(runs: &Runs) -> Result<String, String> {
    for variant in ["whole", "split", "reversed"] {
        if !clean(runs, variant) {
            return Err(format!("{variant} does not load clean"));
        }
    }
    let split = get(runs, "split:reprint")?;
    let reversed = get(runs, "reversed:reprint")?;
    if split.code != 0 || split != reversed {
        return Err(format!(
            "split and reversed reprint otherwise:\n{}\n---\n{}",
            split.out, reversed.out
        ));
    }
    let control = get(runs, "control:reprint")?;
    if control == split || !clean(runs, "control") {
        return Err("the control reprints as the split: vacuous".to_owned());
    }
    Ok("whole, split and reversed load alike; split and reversed reprint the same".to_owned())
}

/// One run of a mutant is total's: exit 1 with one refusal line after any warnings (a
/// `LOAD-*` code, or the probe's REQ-ASK line), or, for a reprint, exit 0 with sections and
/// a `sha256` line. `None` when it is; else what's wrong. `key` is the run's.
fn total_one(key: &str, run: &Run) -> Option<String> {
    let probe = key.ends_with(":probe");
    let Some(body) = run.out.strip_suffix('\n') else {
        return Some(format!("exit {}, no newline at the end", run.code));
    };
    let shown = run.out.chars().take(160).collect::<String>();
    match run.code {
        1 => {
            let lines: Vec<&str> = body.split('\n').collect();
            let Some((_, before)) = lines.split_last() else {
                return Some("exit 1 with no line".to_owned());
            };
            let warned = before.iter().all(|l| l.starts_with("warning: "));
            let refusal = code(&run.out)
                .is_some_and(|c| c.starts_with("LOAD-") || (probe && run.out.ends_with(PROBED)));
            (!(warned && refusal))
                .then(|| format!("exit 1 but not one LOAD-* refusal line: {shown:?}"))
        }
        0 if !probe => {
            let p = printed(&run.out);
            let ok = !p.sections.is_empty()
                && p.tail.starts_with("sha256 ")
                && p.tail.matches('\n').count() == 1;
            (!ok).then(|| format!("exit 0 but not sections and a sha256 line: {shown:?}"))
        }
        other => Some(format!("exit {other}: {shown:?}")),
    }
}

/// Total: a text one operator broke loads clean or is refused by one `LOAD-*` line, never
/// a crash, a hang or a half answer (L §8 "a load refusal stops the model before anything
/// runs"); a refused reprint, E §3.1's step 1, is the probe's refusal too.
pub fn total(runs: &Runs) -> Result<String, String> {
    let mut bad = Vec::new();
    for (key, run) in runs {
        if key.ends_with(":again") {
            continue;
        }
        if let Some(e) = total_one(key, run) {
            bad.push(format!("{key}: {e}"));
        }
    }
    let probe = get(runs, "mutant:probe")?;
    let reprint = get(runs, "mutant:reprint")?;
    if reprint.code != 0 && code(&reprint.out) != code(&probe.out) {
        bad.push(format!(
            "reprint is refused {:?}, the probe {:?}",
            code(&reprint.out),
            code(&probe.out)
        ));
    }
    if bad.is_empty() {
        Ok(format!(
            "probe {:?}, reprint exit {}",
            code(&probe.out),
            reprint.code
        ))
    } else {
        Err(bad.join("; "))
    }
}

/// Depth: an expression nested `dNN` deep is refused `LOAD-DEPTH` past 8 (`LOAD-ASIF` past
/// 4 for `as if`) and loads clean short of it; a chain of `nNN` terms at one level always
/// loads (L §2.3). `form` is the case's (`parens`, `call`, `if`, `asif`, `chain`).
pub fn depth(form: &str, runs: &Runs) -> Result<String, String> {
    let mut bad = Vec::new();
    let (mut refused, mut loaded) = (0_usize, 0_usize);
    for (key, run) in runs {
        let Some(variant) = key.strip_suffix(":probe") else {
            continue;
        };
        let n: u32 = variant
            .get(1..)
            .and_then(|d| d.parse().ok())
            .ok_or_else(|| format!("a depth variant {variant}"))?;
        let want = match form {
            "asif" if n > 4 => "LOAD-ASIF",
            "parens" | "call" | "if" if n > 8 => "LOAD-DEPTH",
            "chain" | "asif" | "parens" | "call" | "if" => "REQ-ASK",
            other => return Err(format!("no depth form {other}")),
        };
        let ok = code(&run.out) == Some(want) && (want != "REQ-ASK" || run.out == PROBED);
        if !ok {
            bad.push(format!("{variant}: {}, wants {want}", first_line(&run.out)));
        }
        let tally = if want == "REQ-ASK" {
            &mut loaded
        } else {
            &mut refused
        };
        *tally = tally.checked_add(1).ok_or("overflow")?;
    }
    if !bad.is_empty() {
        return Err(bad.join("; "));
    }
    if loaded == 0 || (form != "chain" && refused == 0) {
        return Err(format!("{loaded} loaded, {refused} refused: vacuous"));
    }
    Ok(format!("{loaded} load, {refused} refused past the limit"))
}

/// Idempotence: every canonical text reprints as itself (L §2.4, "one canonical form").
pub fn again(runs: &Runs) -> Result<String, String> {
    let mut count = 0_usize;
    for (key, run) in runs {
        let Some(variant) = key.strip_suffix(":again") else {
            continue;
        };
        if Some(run) != runs.get(&format!("{variant}:reprint")) {
            return Err(format!(
                "{variant}: its canonical text reprints otherwise:\n{}",
                run.out
            ));
        }
        count = count.checked_add(1).ok_or("overflow")?;
    }
    Ok(format!("{count} canonical texts reprint as themselves"))
}

/// The law a corpus record names, for its case `case` (`depth/<form>` for depth).
pub fn check(law: &str, case: &str, runs: &Runs) -> Result<String, String> {
    match law {
        "spelling" => spelling(runs),
        "order" => order(runs),
        "total" => total(runs),
        "depth" => depth(case.split_once('/').map_or(case, |(_, form)| form), runs),
        other => Err(format!("no law {other}")),
    }
}

/// Whether the engine's run of a record agrees with refcore's: the same bytes, but for a
/// mutant's refusal, which agrees by its code and exit (`PROPERTIES.md` § M0: which check
/// of the five passes meets a broken text first is the spec's; the words of a refusal on a
/// random break are row 1's and row 2's, not this law's).
pub fn agrees(law: &str, engine: &Run, refcore: &Run) -> bool {
    if law == "total" && refcore.code != 0 {
        engine.code == refcore.code && code(&engine.out) == code(&refcore.out)
    } else {
        engine == refcore
    }
}

#[cfg(test)]
mod tests {
    use super::{M0_CORPUS, M0_REFCORE, PROBED, again, agrees, check, code};
    use crate::corpus::{Run, records, refcore_runs};
    use crate::laws::Runs;
    use std::collections::BTreeMap;

    /// Every case's runs on refcore, by (law, case).
    fn refcore_cases() -> BTreeMap<(String, String), Runs> {
        let recs = records(M0_CORPUS).expect("the corpus reads");
        let runs = refcore_runs(M0_REFCORE).expect("refcore's runs read");
        let mut out: BTreeMap<(String, String), Runs> = BTreeMap::new();
        for r in recs {
            let entry = out
                .entry((r.law.clone(), r.case.clone()))
                .or_insert_with(BTreeMap::new);
            for kind in ["probe", "reprint", "again"] {
                let key = format!("{}:{kind}", r.variant);
                if let Some(got) = runs.get(&(r.case.clone(), key.clone())) {
                    entry.insert(key, got.clone());
                }
            }
            assert!(
                entry.contains_key(&format!("{}:reprint", r.variant)),
                "refcore ran {}/{}",
                r.case,
                r.variant
            );
        }
        out
    }

    #[test]
    fn every_m0_law_holds_on_refcore() {
        let cases = refcore_cases();
        for ((law, case), runs) in &cases {
            assert!(
                check(law, case, runs).is_ok(),
                "{case}: {:?}",
                check(law, case, runs)
            );
            assert!(again(runs).is_ok(), "{case}: {:?}", again(runs));
        }
        let counts: Vec<String> = ["depth", "order", "spelling", "total"]
            .iter()
            .map(|l| format!("{l} {}", cases.keys().filter(|(law, _)| law == l).count()))
            .collect();
        assert_eq!(
            counts.join(", "),
            "depth 5, order 10, spelling 16, total 119",
            "the corpus's cases by law"
        );
    }

    /// Changes one refcore run the way a broken engine would, and expects the law (or
    /// idempotence) to fail on the first case of `law` that has `key`.
    fn planted(law: &str, key: &str, edit: fn(&Run) -> Run) -> bool {
        let cases = refcore_cases();
        let ((_, case), runs) = cases
            .iter()
            .find(|((l, _), r)| l == law && r.contains_key(key))
            .expect("a case of the law with the run");
        let mut runs = runs.clone();
        let r = runs.get(key).cloned().expect("the run");
        runs.insert(key.to_owned(), edit(&r));
        check(law, case, &runs).is_err() || again(&runs).is_err()
    }

    #[test]
    fn each_m0_law_fails_on_a_planted_fault() {
        let byte = |r: &Run| Run {
            out: r.out.replacen('\n', " \n", 1),
            code: r.code,
        };
        assert!(planted("spelling", "sections:reprint", byte), "a spelling");
        assert!(planted("spelling", "blank:probe", byte), "a probe");
        assert!(planted("order", "reversed:reprint", byte), "file order");
        assert!(planted("order", "split:probe", byte), "a split load");
        let crash = |_: &Run| Run {
            out: "thread 'main' panicked\n".to_owned(),
            code: 101,
        };
        assert!(planted("total", "mutant:probe", crash), "a crash");
        assert!(planted("total", "mutant:reprint", crash), "a crash");
        let clean = |_: &Run| Run {
            out: PROBED.to_owned(),
            code: 1,
        };
        assert!(planted("depth", "d09:probe", clean), "past the limit");
        let stub = |_: &Run| Run {
            out: "not implemented\n".to_owned(),
            code: 69,
        };
        assert!(planted("depth", "d01:probe", stub), "the stub");
        assert!(planted("spelling", "base:probe", stub), "the stub");
    }

    #[test]
    fn idempotence_fails_on_a_planted_fault() {
        let cases = refcore_cases();
        let mut runs = cases
            .iter()
            .find(|((law, _), _)| law == "spelling")
            .map(|(_, r)| r.clone())
            .expect("a case of spelling");
        let reprint = runs.get("base:reprint").cloned().expect("a reprint");
        runs.insert("base:again".to_owned(), reprint.clone());
        assert!(again(&runs).is_ok(), "a reprint reprints as itself");
        runs.insert(
            "base:again".to_owned(),
            Run {
                out: reprint.out.replacen('\n', " \n", 1),
                code: reprint.code,
            },
        );
        assert!(again(&runs).is_err(), "a changed byte");
    }

    #[test]
    fn a_mutant_agrees_by_its_code() {
        let run = |out: &str, c: u8| Run {
            out: out.to_owned(),
            code: c,
        };
        let refused = run("refused: LOAD-INDENT an indent of 3 (m.tally:4)\n", 1);
        assert_eq!(code(&refused.out), Some("LOAD-INDENT"), "a code");
        assert_eq!(code("refused: load"), None, "no code");
        assert!(
            agrees(
                "total",
                &run("refused: LOAD-INDENT (m.tally:5)\n", 1),
                &refused
            ),
            "same code, other words"
        );
        assert!(
            !agrees(
                "total",
                &run("refused: LOAD-TAB (m.tally:4)\n", 1),
                &refused
            ),
            "another code"
        );
        assert!(
            !agrees("depth", &run(PROBED, 1), &run("refused: LOAD-DEPTH x\n", 1)),
            "depth agrees by its bytes"
        );
    }
}
