//! Milestone M1's property tests on the engine (`engine/tests/PROPERTIES.md` § M1;
//! `MILESTONE-M1.md` row 4; DONE gate 6). Each record of `properties/m1.corpus` runs on the
//! engine in process; each case is held to its law, the same law `tally-runner`'s own tests
//! prove on refcore's runs. A case refcore breaks inside a filed issue (`held.tsv`, target
//! `m1_properties`) is held to its law here too: the engine follows the spec. One test per
//! law, so a failure names it. `test = false` until M1's marker (pick P4).

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use tally_runner::corpus::{records, refcore_runs};
    use tally_runner::engine::run_record;
    use tally_runner::laws::Runs;
    use tally_runner::laws_m1::{M1_CORPUS, M1_REFCORE, check, texts};

    /// Runs every record of `law` on the engine and checks each case against its law,
    /// collecting every failure before failing.
    fn law_holds(law: &str) {
        let mut cases: BTreeMap<String, (Runs, String)> = BTreeMap::new();
        for r in records(M1_CORPUS)
            .expect("the corpus reads")
            .iter()
            .filter(|r| r.law == law)
        {
            let entry = cases
                .entry(r.case.clone())
                .or_insert_with(|| (BTreeMap::new(), String::new()));
            entry.0.insert(r.variant.clone(), run_record(r));
            entry.1.push_str(&texts(&r.files).expect("the files read"));
        }
        assert!(!cases.is_empty(), "the corpus has cases of {law}");
        let failed: Vec<String> = cases
            .iter()
            .filter_map(|(case, (runs, text))| {
                check(law, runs, text).err().map(|e| format!("{case}: {e}"))
            })
            .collect();
        assert!(
            failed.is_empty(),
            "{} of {} cases fail:\n{}",
            failed.len(),
            cases.len(),
            failed.join("\n")
        );
    }

    /// Every record run twice prints the same bytes, and the engine answers every record
    /// refcore answers (no vacuous pass).
    #[test]
    fn m1_determinism_across_runs() {
        let refcore = refcore_runs(M1_REFCORE).expect("refcore's runs read");
        let mut bad = Vec::new();
        for r in records(M1_CORPUS).expect("the corpus reads") {
            let first = run_record(&r);
            if first != run_record(&r) {
                bad.push(format!("{}/{}: two runs differ", r.case, r.variant));
            }
            let answered = refcore
                .get(&(r.case.clone(), r.variant.clone()))
                .is_some_and(|x| x.code == 0);
            if answered && first.code != 0 {
                let line = first.out.lines().next().unwrap_or("");
                bad.push(format!(
                    "{}/{}: exit {}: {line}",
                    r.case, r.variant, first.code
                ));
            }
        }
        assert!(bad.is_empty(), "{} records:\n{}", bad.len(), bad.join("\n"));
    }

    #[test]
    fn m1_determinism_files_and_sections_in_any_order() {
        law_holds("determinism");
    }

    #[test]
    fn m1_money_conserved() {
        law_holds("conservation");
    }

    #[test]
    fn m1_net_worth_moves_only_at_the_boundary() {
        law_holds("networth");
    }

    #[test]
    fn m1_a_scenario_changes_nothing_before_its_from_it_doesnt_list() {
        law_holds("scenario-before");
    }

    #[test]
    fn m1_asks_move_no_money() {
        law_holds("asks-no-money");
    }

    /// Report-only until `explain` is built (G11): the engine stops "not implemented".
    #[test]
    fn m1_asks_move_no_money_beside_explain() {
        law_holds("asks-no-money-explain");
    }

    #[test]
    fn m1_things_hold_nothing_outside_their_lifespans() {
        law_holds("lifespans");
    }

    #[test]
    fn m1_no_foresight_in_a_keyed_decision() {
        law_holds("foresight-keyed");
    }

    /// Report-only until `low` is built (G8).
    #[test]
    fn m1_no_foresight_through_low() {
        law_holds("foresight-low");
    }

    #[test]
    fn m1_a_hole_never_becomes_zero() {
        law_holds("hole");
    }

    /// Report-only until `explain` is built (G11).
    #[test]
    fn m1_a_hole_never_becomes_zero_through_explain() {
        law_holds("hole-explain");
    }

    #[test]
    fn m1_a_key_moved_by_whole_steps_lands_on_a_key() {
        law_holds("steps");
    }

    /// Report-only until `low` is built (G8).
    #[test]
    fn m1_low_is_the_minimum_of_the_daily_balances() {
        law_holds("low-min");
    }
}
