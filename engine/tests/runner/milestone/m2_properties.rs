//! Milestone M2's property tests on the engine (`engine/tests/PROPERTIES.md` § M2;
//! `MILESTONE-M2.md` row 5; DONE gate 6). Each record of `properties/m2.corpus` runs on the
//! engine in process; each case is held to its law, the same law `tally-runner`'s own tests
//! prove on refcore's runs (`laws_m2`). A case refcore breaks inside a filed issue
//! (`held.tsv`, target `m2_properties`) is held to its law here too: the engine follows the
//! spec. One test per law, so a failure names it, and one across runs. `test = false` until
//! M2's marker.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use tally_runner::corpus::{records, refcore_runs};
    use tally_runner::engine::run_record;
    use tally_runner::laws::Runs;
    use tally_runner::laws_m2::{M2_CORPUS, M2_REFCORE, check};

    /// Runs every record of `law` on the engine and checks each case against its law,
    /// collecting every failure before failing.
    fn law_holds(law: &str) {
        let mut cases: BTreeMap<String, Runs> = BTreeMap::new();
        for r in records(M2_CORPUS)
            .expect("the corpus reads")
            .iter()
            .filter(|r| r.law == law)
        {
            cases
                .entry(r.case.clone())
                .or_insert_with(BTreeMap::new)
                .insert(r.variant.clone(), run_record(r));
        }
        assert!(!cases.is_empty(), "the corpus has cases of {law}");
        let failed: Vec<String> = cases
            .iter()
            .filter_map(|(case, runs)| check(law, runs).err().map(|e| format!("{case}: {e}")))
            .collect();
        assert!(
            failed.is_empty(),
            "{} of {} cases fail:\n{}",
            failed.len(),
            cases.len(),
            failed.join("\n")
        );
    }

    /// Masking at K (E §2): an ask `as known K` prints the same with and without a row known
    /// after K (an observed row, a plan's version, a payslip). For G6 `sweep`.
    #[test]
    fn m2_masking_at_k() {
        law_holds("masking");
    }

    /// The three-way split's replay (E §6.2, §6.3): the matched pay lines' differences are
    /// 0.00 where the law reads the payslips. For G5 `ledger`.
    #[test]
    fn m2_the_replay_shows_no_law_difference() {
        law_holds("split");
    }

    /// The fixed past (L §5.14.3, E §5): a version known on D moves nothing before D.
    #[test]
    fn m2_the_past_is_fixed() {
        law_holds("fixed-past");
    }

    /// Every record run twice prints the same bytes, and the engine answers every record
    /// refcore answers.
    #[test]
    fn m2_determinism_across_runs() {
        let refcore = refcore_runs(M2_REFCORE).expect("refcore's runs read");
        let mut bad = Vec::new();
        for r in records(M2_CORPUS).expect("the corpus reads") {
            let (a, b) = (run_record(&r), run_record(&r));
            if a != b {
                bad.push(format!("{}/{}: two runs differ", r.case, r.variant));
            }
            let answered = refcore
                .get(&(r.case.clone(), r.variant.clone()))
                .is_some_and(|x| x.code == 0);
            if answered && a.code != 0 {
                bad.push(format!(
                    "{}/{}: refcore answers, the engine exits {}",
                    r.case, r.variant, a.code
                ));
            }
        }
        assert!(bad.is_empty(), "{}", bad.join("\n"));
    }
}
