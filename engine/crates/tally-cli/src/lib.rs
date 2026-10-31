//! The `tally` command as a function, `run(args, stdin) -> Output`, which the binary
//! and the test runner both call (engine-tests §4).
//! Spec: none, a stub.
//! Twin: none, a stub.
//! Invariants: the exit codes stay clear of 101 (a panic) and 134 (an abort).
//!
//! A stub. Every command answers "not implemented". The exit codes are placeholders
//! until the test interface fixes them, and they stay clear of 101 (a panic) and
//! 134 (an abort).

#![forbid(unsafe_code)]

use std::ffi::OsString;

/// What one run of the command prints, and the code it exits with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    /// The bytes written to standard output.
    pub stdout: Vec<u8>,
    /// The bytes written to standard error.
    pub stderr: Vec<u8>,
    /// The process's exit code.
    pub code: u8,
}

/// Exit code for a command line the stub doesn't understand.
pub const EXIT_USAGE: u8 = 64;
/// Exit code for a command the stub doesn't implement yet.
pub const EXIT_NOT_IMPLEMENTED: u8 = 69;
/// Exit code for a failed read of standard input or a failed write of the output.
pub const EXIT_IO: u8 = 74;

/// The commands of the test interface.
const COMMANDS: [&str; 5] = ["load", "ask", "run", "project", "format"];

/// Runs one command. `args` are the arguments after the program name; `stdin` is
/// standard input's bytes (empty when it is a terminal). With the `counters` feature, a
/// `--stats` argument is taken out and the counters' line follows standard error (D4-8).
pub fn run(args: &[OsString], stdin: &[u8]) -> Output {
    #[cfg(feature = "counters")]
    if args.iter().any(|arg| arg.to_str() == Some("--stats")) {
        let rest: Vec<OsString> = args
            .iter()
            .filter(|arg| arg.to_str() != Some("--stats"))
            .cloned()
            .collect();
        let mut output = command(&rest, stdin);
        output
            .stderr
            .extend_from_slice(tally::Counters::default().line().as_bytes());
        return output;
    }
    command(args, stdin)
}

/// One command of the test interface, answered by the stub.
fn command(args: &[OsString], _stdin: &[u8]) -> Output {
    match args.first().and_then(|first| first.to_str()) {
        Some(command) if COMMANDS.contains(&command) => Output {
            stdout: Vec::new(),
            stderr: format!("tally {command}: not implemented\n").into_bytes(),
            code: EXIT_NOT_IMPLEMENTED,
        },
        Some(_) | None => Output {
            stdout: Vec::new(),
            stderr: b"usage: tally <load|ask|run|project|format> ...\n".to_vec(),
            code: EXIT_USAGE,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{EXIT_NOT_IMPLEMENTED, EXIT_USAGE, run};
    use std::ffi::OsString;

    #[test]
    fn every_command_answers_not_implemented() {
        for command in ["load", "ask", "run", "project", "format"] {
            let output = run(&[OsString::from(command)], b"");
            assert_eq!(output.code, EXIT_NOT_IMPLEMENTED, "{command}");
        }
    }

    #[cfg(feature = "counters")]
    #[test]
    fn stats_adds_the_counters_line() {
        let output = run(&[OsString::from("run"), OsString::from("--stats")], b"");
        assert_eq!(output.code, EXIT_NOT_IMPLEMENTED, "run --stats");
        assert!(
            output
                .stderr
                .ends_with(b"recomputed_locally=0 num_results=0\n"),
            "{:?}",
            output.stderr
        );
    }

    #[test]
    fn anything_else_is_a_usage_error() {
        assert_eq!(run(&[], b"").code, EXIT_USAGE, "no arguments");
        assert_eq!(
            run(&[OsString::from("lod")], b"").code,
            EXIT_USAGE,
            "unknown command"
        );
    }
}
