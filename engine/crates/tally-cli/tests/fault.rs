//! The builder's test of the planted fault: `run_faulted` refuses each value that rests on the broken walk `ENGINE-INVARIANT`, and `run` answers the same reality clean.
//! Spec: E §9 item 1 (conservation: a day's whole legs sum to 0 in each unit), §11 (an engine
//! bug refuses the result with its evidence, `ENGINE-INVARIANT`); INTERFACE §4 (`run_faulted`,
//! the plant `Fault::Conservation`, pick P8).
//! Twin: `engine/tests/codes/m1_fault.py` (refcore's close of the day, patched in memory); each
//! expected line is refcore's run of `fault/fault.tally` with and without that plant.
//! Invariants: reads its own reality only, never a test, a vector or an answer key (R-93); only
//! with the `fault-injection` feature.

#![forbid(unsafe_code)]

#[cfg(test)]
#[cfg(feature = "fault-injection")]
mod tests {
    use std::ffi::OsString;
    use tally_cli::{EXIT_ANSWERED, File, Output, pack, run, run_faulted};

    /// A rent from a bank account to the world in February, and a pay back in April, after
    /// the cut: the first day a walk books a leg whole is the rent's.
    const REALITY: &str = include_str!("fault/fault.tally");

    /// The run's arguments and its file table.
    fn command() -> (Vec<OsString>, Vec<u8>) {
        let args = ["run", "fault.tally", "--ask", "q"]
            .iter()
            .map(OsString::from)
            .collect();
        let files = pack(&[File {
            path: "fault.tally".to_owned(),
            bytes: REALITY.as_bytes().to_vec(),
        }]);
        (args, files)
    }

    /// The printed text of an output, with its exit code.
    fn printed(output: &Output) -> (String, u8) {
        (
            String::from_utf8(output.stdout.clone()).expect("UTF-8"),
            output.code,
        )
    }

    #[test]
    fn the_plant_refuses_each_value_on_the_broken_walk() {
        let (args, files) = command();
        let refused = "refused: ENGINE-INVARIANT conservation: the legs booked on 2026-02-10 \
                       in PLN sum to 0.01 PLN, not 0 (E §9 invariant 1)";
        let want = format!("q: ({refused}, {refused})\n  cut 2026-03-01\n");
        let output = run_faulted(&args, &files, tally::Fault::Conservation);
        assert_eq!(
            printed(&output),
            (want, EXIT_ANSWERED),
            "every value on the walk refused, exit 0"
        );
        assert!(output.stderr.is_empty(), "nothing on standard error");
    }

    #[test]
    fn without_the_plant_the_run_answers() {
        let (args, files) = command();
        let want = "q: (500.00 PLN, 560.00 PLN)\n  unfulfilled: rent\n  cut 2026-03-01\n";
        assert_eq!(
            printed(&run(&args, &files)),
            (want.to_owned(), EXIT_ANSWERED),
            "the same reality answered clean"
        );
    }

    #[test]
    fn a_command_that_walks_nothing_is_answered_as_run_answers_it() {
        let (_, files) = command();
        let args: Vec<OsString> = ["reprint", "fault.tally"]
            .iter()
            .map(OsString::from)
            .collect();
        let planted = run_faulted(&args, &files, tally::Fault::Conservation);
        assert_eq!(planted, run(&args, &files), "a reprint walks no day");
    }
}
