//! The `tally` binary. This is the one file that does I/O (R-39, R-108): it reads the
//! arguments and standard input, calls [`tally_cli::run`], writes the output bytes
//! itself and returns the exit code from `main` (R-10, R-11).

#![forbid(unsafe_code)]

use std::ffi::OsString;
use std::io::{IsTerminal, Read, Write};
use std::process::ExitCode;

#[expect(
    clippy::disallowed_methods,
    reason = "R-39: the CLI's I/O (args_os, stdin, stdout, stderr) lives in this one sanctioned file"
)]
fn main() -> ExitCode {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let mut input = Vec::new();
    let mut stdin = std::io::stdin().lock();
    if !stdin.is_terminal() && stdin.read_to_end(&mut input).is_err() {
        return ExitCode::from(tally_cli::EXIT_IO);
    }
    let output = tally_cli::run(&args, &input);
    let mut stdout = std::io::stdout().lock();
    if stdout.write_all(&output.stdout).and_then(|()| stdout.flush()).is_err() {
        return ExitCode::from(tally_cli::EXIT_IO);
    }
    let mut stderr = std::io::stderr().lock();
    if stderr.write_all(&output.stderr).and_then(|()| stderr.flush()).is_err() {
        return ExitCode::from(tally_cli::EXIT_IO);
    }
    ExitCode::from(output.code)
}
