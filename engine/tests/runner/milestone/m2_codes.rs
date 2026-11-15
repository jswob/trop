//! Milestone M2's code fixtures on the engine (DONE gate 5; `BUILD-ORDER.md` § M2;
//! `MILESTONE-M2.md` row 4; issue #167): each fixture of `codes/m2.refcore` run in process,
//! its output and exit code refcore's, byte for byte. A fixture is one edit of the family
//! `records`' base text (`codes/m2/records/`: two statements, a transfer on both stated
//! `same`, a reading after a decision), run on the asks it names; beside it run its stated
//! twins, the family's base (the load probe) and the answers of its asks on the base (each
//! answered clean). `tools/xref_check.py --gate M2` counts them. One test per code, for the
//! builder group G4 `records`; the reading-after form is held for #169 (`codes/m2/held.tsv`:
//! refcore 2.1.7 prints no refusal there). `test = false` until M2's marker.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use tally_runner::codes::{M0Fixture, M2, m0_fixtures};
    use tally_runner::engine::run_args;
    use tally_runner::kernel::difference;

    /// The codes this target tests, one `#[test]` each.
    const CODES: [&str; 2] = ["HOLE-SAME", "RUN-RECORD"];

    /// Whether `v` is a family's twin: its base or one of its answers.
    fn is_twin(v: &M0Fixture) -> bool {
        v.is_base() || v.id.contains("/answers/")
    }

    /// The ids of a fixture's stated twins: its family's base, and the answers of each ask
    /// it runs (`<family>/answers/<ask>`).
    fn twin_ids(v: &M0Fixture) -> Vec<String> {
        let asks = v.args.iter().skip_while(|a| *a != "--ask").skip(1);
        std::iter::once(format!("{}/base", v.family))
            .chain(asks.map(|a| format!("{}/answers/{a}", v.family)))
            .collect()
    }

    /// Runs every fixture of `codes/m2.refcore` that prints `code`, and the stated twins of
    /// each, on the engine against refcore's whole output and exit code.
    fn code_as_refcore(code: &str) {
        let vs = m0_fixtures(M2).expect("codes/m2.refcore reads");
        let mine: Vec<&M0Fixture> = vs
            .iter()
            .filter(|v| !is_twin(v) && v.code == code)
            .collect();
        assert!(!mine.is_empty(), "{code} has fixtures");
        let wanted: BTreeSet<String> = mine.iter().flat_map(|v| twin_ids(v)).collect();
        let twins = vs.iter().filter(|v| wanted.contains(&v.id));
        let failed: Vec<String> = twins
            .chain(mine.iter().copied())
            .filter_map(|v| {
                difference(&v.run, &run_args(&v.args, &v.files)).map(|d| format!("{}: {d}", v.id))
            })
            .collect();
        assert!(
            failed.is_empty(),
            "{} of the vectors of {code} and their twins fail:\n{}",
            failed.len(),
            failed.join("\n")
        );
    }

    #[test]
    fn m2_codes_every_code_is_tested() {
        let named: BTreeSet<String> = m0_fixtures(M2)
            .expect("codes/m2.refcore reads")
            .into_iter()
            .filter(|v| !is_twin(v))
            .map(|v| v.code)
            .collect();
        let tested: BTreeSet<String> = CODES.iter().map(|c| (*c).to_owned()).collect();
        assert_eq!(
            named, tested,
            "each code of codes/m2.refcore has its test here"
        );
    }

    /// `HOLE-SAME`'s `<p> and <q> are one, but <x> differs on <d>: …`: `records/same-differs`,
    /// rows stated `same` that disagree on the amount, read by `flows` of their move away
    /// from its places (L §5.19, E §6, P35), with its `WARN-LEGS` line. For G4.
    #[test]
    fn m2_code_hole_same() {
        code_as_refcore("HOLE-SAME");
    }

    /// `RUN-RECORD`'s M2 forms: `records/same-missing` (`same <p> = <q>: <p> doesn't exist`),
    /// `records/unjoined` (two rows of one transfer, not stated `same`, both fulfil one
    /// instance, P35) and `records/adjustment-unmade` (a row fulfils an adjustment no version
    /// made by the cut). For G4.
    #[test]
    fn m2_code_run_record() {
        code_as_refcore("RUN-RECORD");
    }
}
