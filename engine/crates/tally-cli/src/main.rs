//! The `tally` binary, the one file that does I/O (R-39, R-108): it reads the arguments
//! and standard input and writes the output bytes and the exit code.
//! Spec: none, a stub.
//! Twin: none, a stub.
//! Invariants: `main` holds no exception, and a `process::exit` in it still fails (R-10).
//!
//! It calls [`tally_cli::run`], writes the output bytes itself and returns the exit code
//! from `main` (R-10, R-11).
//!
//! Each I/O call sits in its own small function with its own exception (the CLI's
//! budget of three: arguments, reading the reality, writing output), so `main` holds
//! none, and a `process::exit` written in it still fails (R-10).

#![forbid(unsafe_code)]

use std::ffi::OsString;
use std::io::{IsTerminal, Read, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = arguments();
    let Some(input) = read_input() else {
        return ExitCode::from(tally_cli::EXIT_IO);
    };
    let output = tally_cli::run(&args, &input);
    if write_output(&output).is_err() {
        return ExitCode::from(tally_cli::EXIT_IO);
    }
    ExitCode::from(output.code)
}

/// The arguments after the program name, read once, in the form that can't crash.
#[expect(
    clippy::disallowed_methods,
    reason = "R-12: the arguments are read once, here, through args_os, which can't crash"
)]
fn arguments() -> Vec<OsString> {
    std::env::args_os().skip(1).collect()
}

/// Standard input's bytes, empty when it is a terminal, or `None` when the read fails.
#[expect(
    clippy::disallowed_methods,
    reason = "R-39: the CLI reads the reality from standard input in this one sanctioned file"
)]
fn read_input() -> Option<Vec<u8>> {
    let mut input = Vec::new();
    let mut stdin = std::io::stdin().lock();
    if !stdin.is_terminal() && stdin.read_to_end(&mut input).is_err() {
        return None;
    }
    Some(input)
}

/// Writes the output's bytes to standard output and standard error.
#[expect(
    clippy::disallowed_methods,
    reason = "R-39: the CLI writes its output bytes in this one sanctioned file"
)]
fn write_output(output: &tally_cli::Output) -> std::io::Result<()> {
    let mut stdout = std::io::stdout().lock();
    stdout.write_all(&output.stdout)?;
    stdout.flush()?;
    let mut stderr = std::io::stderr().lock();
    stderr.write_all(&output.stderr)?;
    stderr.flush()
}
