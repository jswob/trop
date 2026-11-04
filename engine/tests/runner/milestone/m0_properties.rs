//! Milestone M0's property tests on the engine (`engine/tests/PROPERTIES.md` § M0;
//! `MILESTONE-M0.md` row 7). Each record of `properties/m0.corpus` runs on the engine in
//! process as the load probe, as `reprint` and, when that exits 0, as `reprint` of its own
//! canonical sections; each case is held to its law, the same law `tally-runner`'s own tests
//! prove on refcore's runs, and each run to refcore's. Red on S's engine, so this target has
//! `test = false`; it gates once M0 is tagged (`engine/tools/milestones.toml`).

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use tally_runner::corpus::{Record, records, refcore_runs};
    use tally_runner::engine::{run_args, run_record};
    use tally_runner::laws::Runs;
    use tally_runner::laws_m0::{
        M0_CORPUS, M0_REFCORE, again, again_of, agrees, check, reprint_args,
    };

    /// A record's runs on the engine, keyed as `laws_m0` keys them.
    fn engine_runs(r: &Record, runs: &mut Runs) {
        runs.insert(format!("{}:probe", r.variant), run_record(r));
        let args = reprint_args(r).expect("the record's paths read");
        let reprint = run_args(&args, &r.files);
        if let Some((args, files)) = again_of(&reprint) {
            runs.insert(format!("{}:again", r.variant), run_args(&args, &files));
        }
        runs.insert(format!("{}:reprint", r.variant), reprint);
    }

    /// The engine's runs of every case of `law`, by case.
    fn cases(law: &str) -> BTreeMap<String, Runs> {
        let mut out: BTreeMap<String, Runs> = BTreeMap::new();
        for r in records(M0_CORPUS)
            .expect("the corpus reads")
            .iter()
            .filter(|r| r.law == law)
        {
            engine_runs(r, out.entry(r.case.clone()).or_insert_with(BTreeMap::new));
        }
        assert!(!out.is_empty(), "the corpus has cases of {law}");
        out
    }

    /// Checks every case of `law` on the engine, its law and refcore's runs, and collects
    /// every failure before failing, so one run shows them all.
    fn law_holds(law: &str) {
        let refcore = refcore_runs(M0_REFCORE).expect("refcore's runs read");
        let all = cases(law);
        let mut failed = Vec::new();
        for (case, runs) in &all {
            if let Err(e) = check(law, case, runs) {
                failed.push(format!("{case}: {e}"));
            }
            for (key, run) in runs {
                match refcore.get(&(case.clone(), key.clone())) {
                    Some(want) if agrees(law, run, want) => {}
                    Some(want) => failed.push(format!(
                        "{case} {key}: the engine prints\n{}refcore\n{}",
                        run.out, want.out
                    )),
                    None => failed.push(format!(
                        "{case} {key}: refcore's reprint was refused, the engine's wasn't"
                    )),
                }
            }
        }
        assert!(
            failed.is_empty(),
            "{} failures in {} cases:\n{}",
            failed.len(),
            all.len(),
            failed.join("\n")
        );
    }

    #[test]
    fn m0_format_equal_models_print_identically() {
        law_holds("spelling");
    }

    #[test]
    fn m0_load_file_order_changes_nothing() {
        law_holds("order");
    }

    #[test]
    fn m0_parser_total() {
        law_holds("total");
    }

    #[test]
    fn m0_depth_stops_at_8() {
        law_holds("depth");
    }

    /// Every canonical text reprints as itself; a record refcore reprints (exit 0) must be
    /// reprinted by the engine too, so a run that reprints nothing can't pass vacuously.
    #[test]
    fn m0_format_idempotent() {
        let refcore = refcore_runs(M0_REFCORE).expect("refcore's runs read");
        let mut failed = Vec::new();
        let mut checked = 0_usize;
        for law in ["spelling", "order", "total", "depth"] {
            for (case, runs) in cases(law) {
                if let Err(e) = again(&runs) {
                    failed.push(format!("{case}: {e}"));
                }
                for ((c, key), run) in &refcore {
                    let Some(variant) = key.strip_suffix(":reprint") else {
                        continue;
                    };
                    if c != &case || run.code != 0 {
                        continue;
                    }
                    checked = checked.checked_add(1).expect("a count");
                    if !runs.contains_key(&format!("{variant}:again")) {
                        failed.push(format!("{case} {variant}: the engine reprints nothing"));
                    }
                }
            }
        }
        assert!(checked > 0, "refcore reprints some records");
        assert!(
            failed.is_empty(),
            "{} failures:\n{}",
            failed.len(),
            failed.join("\n")
        );
    }
}
