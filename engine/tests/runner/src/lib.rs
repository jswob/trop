//! The test runner. A stub that belongs to the test owner (T): T's content replaces
//! it, and on merge T's version wins. It runs every vector and fixture through
//! `tally_cli::run` once T has written them.

#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    #[test]
    fn the_stub_answers_not_implemented() {
        let output = tally_cli::run(&[OsString::from("load")], b"");
        assert_eq!(
            output.code,
            tally_cli::EXIT_NOT_IMPLEMENTED,
            "stub exit code"
        );
    }
}
