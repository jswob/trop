//! Milestone S's household vectors on the engine (DONE gate 3; `MILESTONE-S.md` row 4): X's
//! base run in process as each of milestone S's three checks, and its output held to HH4's
//! hand-checked vectors restated in tally 2.1.1's output form (`hh4/vectors.tsv`, each with
//! its HH4 lines and the rule it checks). Red on the stub (exit 69, "not implemented"), so
//! this target has `test = false`; it gates once S is tagged (`engine/tools/milestones.toml`).

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use tally_runner::engine::run_args;
    use tally_runner::household::{VECTORS, broken, s_checks, vectors};

    /// Runs check `id` on the engine and holds its output to every vector of that check.
    /// Collects every broken vector before failing, so one run shows them all.
    fn holds(id: &str) {
        let vs = vectors(VECTORS).expect("hh4/vectors.tsv reads");
        let check = s_checks()
            .into_iter()
            .find(|c| c.id == id)
            .expect("a check of milestone S");
        let run = run_args(&check.args, &check.files);
        // refcore answers each of the three checks (exit 0)
        assert_eq!(run.code, 0, "{id}: not answered: {}", run.out);
        let failed = broken(id, &run.out, &vs);
        let mine = vs
            .iter()
            .filter(|v| v.checks.iter().any(|c| c == id))
            .count();
        assert!(
            failed.is_empty(),
            "{id}: {} of {mine} vectors fail:\n{}",
            failed.len(),
            failed.join("\n")
        );
    }

    #[test]
    fn s_hh4_konto_balance_at_b_r1() {
        holds("I:X:konto_balance@b_r1");
    }

    #[test]
    fn s_hh4_replay_september_at_b_r1() {
        holds("I:X:replay_september@b_r1");
    }

    #[test]
    fn s_hh4_replay_september_at_b_r1s() {
        holds("I:X:replay_september@b_r1s");
    }
}
