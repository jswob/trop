//! Milestone S's kernel tests on the engine (issue #41; `MILESTONE-S.md` row 5, DONE gate 8):
//! every vector of `kernel/vectors.refcore` and every record of the kernel corpus run in
//! process, the engine's whole output and exit code held to refcore's, byte for byte. One test
//! per group of T's plan, so a failure names its group; every failure of a group is collected
//! before it fails, so one run shows them all.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use tally_runner::corpus::{KERNEL_CORPUS, KERNEL_REFCORE, records, refcore_runs};
    use tally_runner::engine::{run_args, run_record};
    use tally_runner::kernel::{VECTORS, difference, vectors};

    /// The groups this target tests, one `#[test]` each.
    const GROUPS: [&str; 10] = [
        "checks", "g1", "g2", "g3", "g4", "g5", "g6", "g7", "g8", "g9",
    ];

    /// Runs every vector of `group` on the engine and holds it to refcore's run.
    fn group_holds(group: &str) {
        let all = vectors(VECTORS).expect("kernel/vectors.refcore reads");
        let mine: Vec<_> = all.iter().filter(|v| v.group == group).collect();
        assert!(!mine.is_empty(), "the group {group} has vectors");
        let failed: Vec<String> = mine
            .iter()
            .filter_map(|v| {
                difference(&v.refcore, &run_args(&v.args, &v.files))
                    .map(|d| format!("{}: {d}", v.id))
            })
            .collect();
        assert!(
            failed.is_empty(),
            "{} of {} vectors of {group} differ from refcore:\n{}",
            failed.len(),
            mine.len(),
            failed.join("\n")
        );
    }

    #[test]
    fn s_kernel_every_group_is_tested() {
        let named: BTreeSet<String> = vectors(VECTORS)
            .expect("kernel/vectors.refcore reads")
            .into_iter()
            .map(|v| v.group)
            .collect();
        let tested: BTreeSet<String> = GROUPS.iter().map(|g| (*g).to_owned()).collect();
        assert_eq!(
            named, tested,
            "each group of kernel/vectors.tsv has its test here"
        );
    }

    /// The kernel corpus's records (`properties/gen.py`), each the engine's whole output
    /// against refcore's (`properties/laws.py`).
    #[test]
    fn s_kernel_corpus_whole_output() {
        let runs = refcore_runs(KERNEL_REFCORE).expect("refcore's runs read");
        let recs = records(KERNEL_CORPUS).expect("the corpus reads");
        let failed: Vec<String> = recs
            .iter()
            .filter_map(|r| {
                let key = (r.case.clone(), r.variant.clone());
                match runs.get(&key) {
                    None => Some(format!("{}/{}: no refcore run", r.case, r.variant)),
                    Some(expected) => difference(expected, &run_record(r))
                        .map(|d| format!("{}/{}: {d}", r.case, r.variant)),
                }
            })
            .collect();
        assert!(
            failed.is_empty(),
            "{} of {} corpus records differ from refcore:\n{}",
            failed.len(),
            recs.len(),
            failed.join("\n")
        );
    }

    /// The checks of `checks.tsv` the engine at S answers: 33 unit probes, 5 models and 5 of
    /// household X's checks, each compared whole (a household check's warnings included).
    #[test]
    fn s_kernel_checks_whole_output() {
        group_holds("checks");
    }

    /// G1, exact numbers, their text and rounding (L §7.1, §7.2, §5.18): every rounding mode
    /// at ties and off them, both signs, in the big tier too; a/b and the reading aid; money
    /// past the machine word; the 4 096-digit cap and a step that isn't positive, each refused
    /// beside an answered twin (`kernel/g1/`).
    #[test]
    fn s_kernel_g1_numbers() {
        group_holds("g1");
    }

    /// G2, the read rule (L §4.6, §4.5, §5.3, §5.11): rows by literal, range, `*` and a
    /// left-out key; known dates under `as known` cuts; spans; `replace`; precedence; keys and
    /// things a later version adds; dated rules; `held` past the cut (`kernel/g2/`).
    #[test]
    fn s_kernel_g2_read_rule() {
        group_holds("g2");
    }

    /// G3, bands (L §5.9): open, closed and infinite ends, a hole row, patches chained by
    /// known date, an argument no row holds, a band governing always (`kernel/g3/`).
    #[test]
    fn s_kernel_g3_bands() {
        group_holds("g3");
    }

    /// G4, holes in the answer (L §8.1, §4.6 step 5): one question at two figures, two
    /// questions sorted, a stated hole, the no-value texts, keys in brackets, a thing outside
    /// its lifespan read as none (`kernel/g4/`).
    #[test]
    fn s_kernel_g4_holes() {
        group_holds("g4");
    }

    /// G5, the day walk's agenda and the order within a day (E §4.2 steps 0–4, L §5.10,
    /// §5.14.2, §5.19): openings before the start and on it; readings at a day's start and end
    /// around the cut; observations before, inside and after the run, two on a day; a covered
    /// month; plans outside the run; decisions in strata, one due twice with `ties by` a
    /// position and one by a money value; a decision computed less its own earlier
    /// observation; decisions on the cut's day; a walk-bound def read at two points; ties with
    /// no `ties by` (RUN-TIE) beside an untied twin; a decision refused in its `when`; a
    /// projection read by a decision on its own day and by an ask, from a day with a reading
    /// at its start; lifespans that end inside the run; a cut after the run's end
    /// (`kernel/g5/`).
    #[test]
    fn s_kernel_g5_agenda() {
        group_holds("g5");
    }

    /// G6, the plan pass (E §4.1, L §5.14): a daily move's ledger lines in a covered month; a
    /// `when` that holds, fails, is a hole and is refused; an attribute refused; an uncertain
    /// kind with and without `with uncertain`; legs in money and units, of a kind of unit
    /// `any`, and of 0 (`kernel/g6/`).
    #[test]
    fn s_kernel_g6_plan_pass() {
        group_holds("g6");
    }

    /// G7, reads inside the walk and at the ask (L §5.10, §5.14, §5.14.1, §5.14.3, §8.2): a
    /// decision's read of its own day (RUN-CYCLE), of a later one (RUN-FUTURE), of its day's
    /// start and of the day before; flows of a place and of a decision move through a later
    /// day, the reader's own and the day before; flows read that tie two instances (RUN-TIE)
    /// beside a twin that doesn't; flows of a place across a refusal of every place, a hole
    /// amount, a hole day and the run's start; flows of a move by leg and keys, and over a
    /// span with no movement; `bal(x, u, d)`; `with uncertain` around a reading; flows of a
    /// move whose leg the walk refuses, asked first while the walk it needs reads a def deeper
    /// than the evaluation's depth limit, in a decision and in a plan (`kernel/g7/`).
    #[test]
    fn s_kernel_g7_reads() {
        group_holds("g7");
    }

    /// G8, projections (L §5.14.1, E §4.3, D-6): `bal(x, d) from t` from before the run's
    /// start, from it, around the cut; the read's own `with uncertain` and the ask line's;
    /// a place first stated after t; a decision's projection from a later day (RUN-FUTURE)
    /// and an earlier one; #35, a decision reading a projection, asked with and without the
    /// line's `with uncertain`; #37 T1 and N2; a decision reading its own move's flows,
    /// projected from before its day; a projection from a decision's own point, alone, nested
    /// in another, and with a later decision that day refused in its `when` (`kernel/g8/`).
    #[test]
    fn s_kernel_g8_projections() {
        group_holds("g8");
    }

    /// G9, booking a leg and the sign rule (L §5.10, §5.14, E §4.2 steps 4–5): claims, debts
    /// and a holding broken by a first-batch plan and by a decision (RUN-PLACE), each beside a
    /// twin that stays at 0 or nets in its batch; a claim stated below zero at a close; legs
    /// out of and into places outside their lifespans (RUN-LIFESPAN) beside a leg of 0;
    /// `WARN-NEGATIVE` twice; a hole amount and a hole day (`kernel/g9/`).
    #[test]
    fn s_kernel_g9_sign_rule() {
        group_holds("g9");
    }
}
