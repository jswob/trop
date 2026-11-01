//! The `tally` binary, the one file that does I/O (R-39, R-108): arguments, the reality's
//! files, the output bytes and the exit code; never standard input, as refcore.
//! Spec: engine/tests/INTERFACE.md §2 (reading the reality), §4 (the file table); E §3.1
//! step 1 (files by full path).
//! Twin: engine/tools/refcore/cli.py (`read_files`) and engine/tools/refcore/loader.py
//! (`collect`, its `os.walk`).
//! Invariants: `main` holds no exception, and a `process::exit` in it still fails (R-10); a
//! path that can't be read is left out of the table, so the library refuses it `LOAD-PATH`.
//!
//! It reads every path the arguments name into the file table, calls [`tally_cli::run`],
//! writes the output bytes itself and returns the exit code from `main` (R-10, R-11).
//!
//! Each I/O call sits in its own small function with its own exception (the CLI's budget of
//! three: the arguments, the reality's files, the output), so `main` holds none, and a
//! `process::exit` written in it still fails (R-10). The std items it may name are listed in
//! tools/cli-io.toml (R-39).

#![forbid(unsafe_code)]

use std::ffi::OsString;
use std::io::Write;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = arguments();
    let files = read_reality(&tally_cli::paths(&args));
    let output = tally_cli::run(&args, &tally_cli::pack(&files));
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

/// Every file the paths name: a file as given; under a directory, every file ending `.tally`
/// at any depth, named `<dir>/<relative path>`, its subdirectories walked without following
/// links, as `os.walk` does. A path that names nothing, or can't be read, is left out.
#[expect(
    clippy::disallowed_methods,
    reason = "R-39: the reality's files are read here, once, before the pure run takes them"
)]
fn read_reality(paths: &[String]) -> Vec<tally_cli::File> {
    let mut files = Vec::new();
    let mut dirs = Vec::new();
    for path in paths {
        let Ok(meta) = std::fs::metadata(path) else {
            continue;
        };
        if meta.is_dir() {
            dirs.push(path.clone());
        } else if let Ok(bytes) = std::fs::read(path) {
            files.push(tally_cli::File {
                path: path.clone(),
                bytes,
            });
        }
    }
    while let Some(dir) = dirs.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let (Ok(kind), Ok(name)) = (entry.file_type(), entry.file_name().into_string()) else {
                continue;
            };
            let path = tally_cli::join(&dir, &name);
            if kind.is_dir() {
                dirs.push(path);
            } else if tally_cli::is_tally(&name)
                && let Ok(bytes) = std::fs::read(&path)
            {
                files.push(tally_cli::File { path, bytes });
            }
        }
    }
    files
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
