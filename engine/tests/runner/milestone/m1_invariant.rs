//! Milestone M1's `ENGINE-INVARIANT` fixture on the engine (DONE gate 5; `MILESTONE-M1.md`
//! row 3; rules turn 2's R1; the coordinator's pick P8; issue #108): each vector of
//! `codes/m1_invariant.refcore`, a family's answers run on refcore with E §9 invariant 1's
//! planted fault (`codes/m1_fault.py`), run through the CLI's twin of that plant,
//! `tally_cli::run_faulted` with `Fault::Conservation` (INTERFACE §4), its output and exit
//! code refcore's, byte for byte. The plant is behind the runner's `fault-injection` feature,
//! which turns on `tally-cli`'s (`required-features`), so a plain build never has it.
//! `test = false` until M1's marker (pick P4).

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use tally_cli::tally::Fault;
    use tally_runner::codes::{M1_INVARIANT, m0_fixtures};
    use tally_runner::engine::read_output;
    use tally_runner::kernel::difference;

    /// `decide/conservation`: the decision family's answers under the plant print refcore's
    /// `refused: ENGINE-INVARIANT conservation: the legs booked on <day> in <unit> sum to <x>,
    /// not 0 (E §9 invariant 1)` at each value the broken walk reads.
    #[test]
    fn m1_invariant_conservation_under_the_planted_fault() {
        let all = m0_fixtures(M1_INVARIANT).expect("codes/m1_invariant.refcore reads");
        assert!(!all.is_empty(), "the store holds the fixture");
        let failed: Vec<String> = all
            .iter()
            .filter_map(|f| {
                let args: Vec<OsString> = f.args.iter().map(OsString::from).collect();
                let got = read_output(tally_cli::run_faulted(&args, &f.files, Fault::Conservation));
                difference(&f.run, &got).map(|d| format!("{}: {d}", f.id))
            })
            .collect();
        assert!(
            failed.is_empty(),
            "{} of {} fixtures differ from refcore under the plant:\n{}",
            failed.len(),
            all.len(),
            failed.join("\n")
        );
    }
}
