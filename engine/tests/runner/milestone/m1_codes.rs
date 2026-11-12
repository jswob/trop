//! Milestone M1's code fixtures on the engine (DONE gate 5; `BUILD-ORDER.md` § M1;
//! `MILESTONE-M1.md` row 3; issue #108): each fixture of `codes/m1.refcore` run in process,
//! its output and exit code refcore's, byte for byte. A fixture is one edit of a family's base
//! text (under `codes/m1/`), run on the asks it names; beside it run its stated twins, its
//! family's base (the load probe) and the answers of its asks on the base (each answered
//! clean), so a code's test needs only the constructs its asks read. `tools/xref_check.py
//! --gate M1` counts them. One test per code, its fixtures and the builder group that turns
//! it green named (8 of the 16 are green before M1); the `ENGINE-INVARIANT` fixture runs
//! through `run_faulted` in the rules step's `m1_invariant` target (the coordinator's pick
//! P8), not here. `test = false` until M1's marker (pick P4).

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use tally_runner::codes::{M0Fixture, M1, m0_fixtures};
    use tally_runner::engine::run_args;
    use tally_runner::held::is_held;
    use tally_runner::kernel::difference;

    /// The codes this target tests, one `#[test]` each.
    const CODES: [&str; 16] = [
        "HOLE-ANCHOR",
        "HOLE-NOTYET",
        "HOLE-PRICE",
        "HOLE-STATED",
        "LOAD-FIELD",
        "RUN-CONFLICT",
        "RUN-EMPTY",
        "RUN-FUTURE",
        "RUN-KEY",
        "RUN-NONE",
        "RUN-ORDER",
        "RUN-PLACE",
        "RUN-PRORATA",
        "RUN-SOLVE",
        "RUN-WEIGHTS",
        "RUN-ZERO",
    ];

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

    /// Runs every fixture of `codes/m1.refcore` that prints `code`, and the stated twins of
    /// each, on the engine against refcore's whole output and exit code.
    fn code_as_refcore(code: &str) {
        let vs = m0_fixtures(M1).expect("codes/m1.refcore reads");
        let mine: Vec<&M0Fixture> = vs
            .iter()
            .filter(|v| !is_twin(v) && v.code == code)
            .collect();
        assert!(!mine.is_empty(), "{code} has fixtures");
        let wanted: BTreeSet<String> = mine.iter().flat_map(|v| twin_ids(v)).collect();
        let twins = vs.iter().filter(|v| wanted.contains(&v.id));
        let failed: Vec<String> = twins
            .chain(mine.iter().copied())
            .filter(|v| !is_held("m1_codes", &v.id))
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
    fn m1_codes_every_code_is_tested() {
        let named: BTreeSet<String> = m0_fixtures(M1)
            .expect("codes/m1.refcore reads")
            .into_iter()
            .filter(|v| !is_twin(v))
            .map(|v| v.code)
            .collect();
        let tested: BTreeSet<String> = CODES.iter().map(|c| (*c).to_owned()).collect();
        assert_eq!(
            named, tested,
            "each code of codes/m1.refcore has its test here"
        );
    }

    /// `HOLE-ANCHOR`'s `<place> has no opening or stated reading`: `rules/no-anchor`. Green before M1.
    #[test]
    fn m1_code_hole_anchor() {
        code_as_refcore("HOLE-ANCHOR");
    }

    /// `HOLE-NOTYET`: `rules/notyet`, `first` over movements empty at the cut. For G3.
    #[test]
    fn m1_code_hole_notyet() {
        code_as_refcore("HOLE-NOTYET");
    }

    /// `HOLE-PRICE`, both forms: `prices/no-price` (`value`), `prices/price-year` (a `prices Y`
    /// figure with no index). For G8.
    #[test]
    fn m1_code_hole_price() {
        code_as_refcore("HOLE-PRICE");
    }

    /// `HOLE-STATED`: `rules/stated`, a row stated `?"…"`. Green before M1.
    #[test]
    fn m1_code_hole_stated() {
        code_as_refcore("HOLE-STATED");
    }

    /// #90 (the coordinator's pick P20): `adjust/bound-field` and `gifts/lifespan-zz`, a bare
    /// bound name's field its kind lacks, on M0's families. For G1.
    #[test]
    fn m1_code_load_field() {
        code_as_refcore("LOAD-FIELD");
    }

    /// `RUN-CONFLICT`: `rules/conflict`, two rules win and neither beats the other. Green before M1.
    #[test]
    fn m1_code_run_conflict() {
        code_as_refcore("RUN-CONFLICT");
    }

    /// `RUN-EMPTY`: `rules/empty`, `min` over a `where` that leaves no key. Green before M1.
    #[test]
    fn m1_code_run_empty() {
        code_as_refcore("RUN-EMPTY");
    }

    /// `RUN-FUTURE`'s `a decision on <d> binds over <K> through <d'>, after its day`:
    /// `decide/binds-future`. For G10.
    #[test]
    fn m1_code_run_future() {
        code_as_refcore("RUN-FUTURE");
    }

    /// `RUN-KEY`'s `<d> is outside the days of <def>`: `rules/outside-days`. Green before M1.
    #[test]
    fn m1_code_run_key() {
        code_as_refcore("RUN-KEY");
    }

    /// `RUN-NONE`'s `<x>[k] is none here: …`: `rules/none`. Green before M1.
    #[test]
    fn m1_code_run_none() {
        code_as_refcore("RUN-NONE");
    }

    /// `RUN-ORDER`'s `decisions <a> and <b> read each other on <d>`: `decide/order`. For G10.
    #[test]
    fn m1_code_run_order() {
        code_as_refcore("RUN-ORDER");
    }

    /// `RUN-PLACE`'s `<movement> takes <debt> above 0`: `rules/debt-above`. Green before M1.
    #[test]
    fn m1_code_run_place() {
        code_as_refcore("RUN-PLACE");
    }

    /// `RUN-PRORATA`'s `<total> isn't a whole number of <u>`: `rules/prorata`. For G4.
    #[test]
    fn m1_code_run_prorata() {
        code_as_refcore("RUN-PRORATA");
    }

    /// `RUN-SOLVE`: `rules/solve`, `f - g` keeps one sign over the bracket. For G5.
    #[test]
    fn m1_code_run_solve() {
        code_as_refcore("RUN-SOLVE");
    }

    /// `RUN-WEIGHTS`: `rules/weights`, `prorata` weights that sum to 0. For G4.
    #[test]
    fn m1_code_run_weights() {
        code_as_refcore("RUN-WEIGHTS");
    }

    /// `RUN-ZERO`: `rules/zero`, `add_business` with n = 0. Green before M1.
    #[test]
    fn m1_code_run_zero() {
        code_as_refcore("RUN-ZERO");
    }
}
