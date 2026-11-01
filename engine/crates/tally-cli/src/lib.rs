//! The `tally` command as a function, `run(args, files) -> Output`, which the binary and T's
//! runner both call.
//! Spec: E §1 (the run function); L §8.3 (`LOAD-PATH`, and `LOAD-PARSE` for bytes);
//! engine/tests/INTERFACE.md §1–§4 (commands, reading, output, the file table).
//! Twin: engine/tools/refcore/cli.py (`main`, `command_lines`, `run_paths`) and
//! engine/tools/refcore/loader.py (`collect`).
//! Invariants: pure, no I/O (R-39); everything is printed to standard output; the exit is 0
//! answered, 1 refused, 2 usage, 74 a failed write, never 101 (a panic) or 134 (an abort).
//!
//! The file table is netstrings (`<decimal length>:<bytes>,`), a path then its bytes, pair
//! after pair (INTERFACE §4). A path in the arguments is a key of the table, or a directory
//! prefix of keys standing for every key under it that ends `.tally`, or a path that names
//! nothing. While the engine is built, a reality that reads goes no further: `run` answers
//! "not implemented" with the code 69, which `run.py --interp engine` shows red.

#![forbid(unsafe_code)]

use core::ops::Bound;
use std::collections::BTreeMap;
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

/// Exit code of an answered run: holes and per-value refusals are printed answers.
pub const EXIT_ANSWERED: u8 = 0;
/// Exit code of a refused run: its first line starts `refused: `.
pub const EXIT_REFUSED: u8 = 1;
/// Exit code of a command line the engine doesn't take.
pub const EXIT_USAGE: u8 = 2;
/// Exit code of a run past the part of the engine built so far (milestone S, groups G3 on).
pub const EXIT_NOT_IMPLEMENTED: u8 = 69;
/// Exit code of a failed write of the output.
pub const EXIT_IO: u8 = 74;

/// The usage text, printed to standard output with [`EXIT_USAGE`].
const USAGE: &str = "usage: tally run <path>... [--pin=bytes] [--ask <id>...]\n       \
                     tally reprint <path>...\n";

/// One file of the reality as the binary reads it: its path as the run names it, and its bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    /// The path: as given in the arguments, or `<dir>/<relative path>` under a directory given.
    pub path: String,
    /// The file's bytes.
    pub bytes: Vec<u8>,
}

/// A command line the engine takes.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Command {
    /// `run <path>… [--pin=bytes] [--ask <id>…]`.
    Run {
        /// The paths, in the order given.
        paths: Vec<String>,
        /// The asks and the pin's form.
        request: tally::Request,
    },
    /// `reprint <path>…`, built in a later milestone.
    Reprint {
        /// The paths, in the order given.
        paths: Vec<String>,
    },
}

/// The file table could not be read as netstring pairs with UTF-8 paths, each path once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BadTable;

/// Runs one command. `args` are the words after the program name; `files` is the file table
/// (INTERFACE §4). With the `counters` feature, a `--stats` argument is taken out and the
/// counters' line follows standard error (D4-8).
#[must_use]
pub fn run(args: &[OsString], files: &[u8]) -> Output {
    #[cfg(feature = "counters")]
    if args.iter().any(|arg| arg.to_str() == Some("--stats")) {
        let rest: Vec<OsString> = args
            .iter()
            .filter(|arg| arg.to_str() != Some("--stats"))
            .cloned()
            .collect();
        let mut output = command(&rest, files);
        output
            .stderr
            .extend_from_slice(tally::Counters::default().line().as_bytes());
        return output;
    }
    command(args, files)
}

/// The paths the command line names, in the order given: what the binary reads into the file
/// table. None for a command line the engine doesn't take.
#[must_use]
pub fn paths(args: &[OsString]) -> Vec<String> {
    match parse(args) {
        Some(Command::Run { paths, .. } | Command::Reprint { paths }) => paths,
        None => Vec::new(),
    }
}

/// `dir` joined with a name in it, as Python's `os.path.join` does for refcore's walk: one `/`
/// between them.
#[must_use]
pub fn join(dir: &str, name: &str) -> String {
    if dir.ends_with('/') {
        format!("{dir}{name}")
    } else {
        format!("{dir}/{name}")
    }
}

/// Whether a file under a directory is a reality's file: its name ends `.tally`, compared
/// exactly as refcore's `f.endswith('.tally')` does, so `a.TALLY` is not one (INTERFACE §2).
#[must_use]
pub fn is_tally(name: &str) -> bool {
    name.rsplit_once('.')
        .is_some_and(|(_, extension)| extension == "tally")
}

/// The file table of `files` (INTERFACE §4): each path then its bytes, as netstrings, the
/// files in the order of their paths.
#[must_use]
pub fn pack(files: &[File]) -> Vec<u8> {
    let mut order: Vec<&File> = files.iter().collect();
    order.sort_by_key(|file| file.path.as_str());
    let mut table = Vec::new();
    for file in order {
        netstring(&mut table, file.path.as_bytes());
        netstring(&mut table, &file.bytes);
    }
    table
}

/// Appends `bytes` to `table` as one netstring.
fn netstring(table: &mut Vec<u8>, bytes: &[u8]) {
    table.extend_from_slice(format!("{}:", bytes.len()).as_bytes());
    table.extend_from_slice(bytes);
    table.push(b',');
}

/// One command, read from the arguments and answered.
fn command(args: &[OsString], files: &[u8]) -> Output {
    match parse(args) {
        Some(Command::Run { paths, request }) => run_paths(&paths, &request, files),
        Some(Command::Reprint { .. }) => not_implemented("reprint"),
        None => usage(),
    }
}

/// The command line as refcore's `command_lines` reads it: after `run`, every word before
/// `--ask` that isn't `--pin=…` is a path, every word after it an ask id, and `--pin=…` may
/// stand anywhere. None for any other command, or a word that isn't UTF-8.
fn parse(args: &[OsString]) -> Option<Command> {
    let words: Vec<&str> = args.iter().map(|arg| arg.to_str()).collect::<Option<_>>()?;
    let (first, rest) = words.split_first()?;
    match *first {
        "run" => Some(run_line(rest)),
        "reprint" => Some(Command::Reprint {
            paths: rest.iter().map(|word| (*word).to_owned()).collect(),
        }),
        _ => None,
    }
}

/// The words after `run`.
fn run_line(words: &[&str]) -> Command {
    let mut paths = Vec::new();
    let mut asks = Vec::new();
    let mut pin = tally::Pin::Canonical;
    let mut after_ask = false;
    for word in words {
        if *word == "--ask" {
            after_ask = true;
        } else if let Some(form) = word.strip_prefix("--pin=") {
            // refcore acts only on `bytes` (INTERFACE §8, gap I-4)
            pin = if form == "bytes" {
                tally::Pin::Bytes
            } else {
                tally::Pin::Canonical
            };
        } else if after_ask {
            asks.push((*word).to_owned());
        } else {
            paths.push((*word).to_owned());
        }
    }
    let asks = if asks.is_empty() {
        tally::Asks::Every
    } else {
        tally::Asks::Named(asks)
    };
    Command::Run {
        paths,
        request: tally::Request {
            asks,
            pin,
            days: tally::Days::Events,
        },
    }
}

/// `run`: the paths resolved against the table, then the engine.
fn run_paths(paths: &[String], request: &tally::Request, files: &[u8]) -> Output {
    let Ok(table) = table(files) else {
        return Output {
            stdout: b"tally: the file table is not netstring pairs of a path and its bytes\n"
                .to_vec(),
            stderr: Vec::new(),
            code: EXIT_USAGE,
        };
    };
    let given = resolve(paths, &table);
    match tally::run(given, request) {
        Ok(answer) => answered(&answer),
        Err(tally::NotImplemented) => not_implemented("run"),
    }
}

/// The file table read into a map from path to bytes.
fn table(files: &[u8]) -> Result<BTreeMap<&str, &[u8]>, BadTable> {
    let mut table = BTreeMap::new();
    let mut rest = files;
    while !rest.is_empty() {
        let (path, after_path) = take_netstring(rest)?;
        let (bytes, after_bytes) = take_netstring(after_path)?;
        let path = core::str::from_utf8(path).map_err(|_| BadTable)?;
        if table.insert(path, bytes).is_some() {
            return Err(BadTable);
        }
        rest = after_bytes;
    }
    Ok(table)
}

/// The first netstring of `bytes` and what follows it.
fn take_netstring(bytes: &[u8]) -> Result<(&[u8], &[u8]), BadTable> {
    let colon = bytes.iter().position(|b| *b == b':').ok_or(BadTable)?;
    let (digits, after) = bytes.split_at_checked(colon).ok_or(BadTable)?;
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
        return Err(BadTable);
    }
    let len: usize = core::str::from_utf8(digits)
        .map_err(|_| BadTable)?
        .parse()
        .map_err(|_| BadTable)?;
    let body_on = after.strip_prefix(b":").ok_or(BadTable)?;
    let (body, rest) = body_on.split_at_checked(len).ok_or(BadTable)?;
    let rest = rest.strip_prefix(b",").ok_or(BadTable)?;
    Ok((body, rest))
}

/// Each path of the command line against the table (INTERFACE §4): a key is that file; a
/// directory prefix of keys is every key under it that ends `.tally`, in path order; anything
/// else names nothing.
fn resolve(paths: &[String], table: &BTreeMap<&str, &[u8]>) -> Vec<tally::Given> {
    let mut given = Vec::new();
    for path in paths {
        if let Some(bytes) = table.get(path.as_str()) {
            given.push(file(path, bytes));
            continue;
        }
        let prefix = join(path, "");
        let before = given.len();
        let from = (Bound::Included(prefix.as_str()), Bound::Unbounded);
        for (key, bytes) in table.range::<str, _>(from) {
            if !key.starts_with(prefix.as_str()) {
                break;
            }
            if is_tally(key) {
                given.push(file(key, bytes));
            }
        }
        if given.len() == before {
            given.push(tally::Given {
                path: path.clone(),
                bytes: None,
            });
        }
    }
    given
}

/// A file of the table, as the engine takes it.
fn file(path: &str, bytes: &[u8]) -> tally::Given {
    tally::Given {
        path: path.to_owned(),
        bytes: Some(bytes.to_vec()),
    }
}

/// The output of an answered or refused run: its lines on standard output.
fn answered(answer: &tally::Answer) -> Output {
    let mut stdout = Vec::new();
    for line in answer.lines() {
        stdout.extend_from_slice(line.as_bytes());
        stdout.push(b'\n');
    }
    Output {
        stdout,
        stderr: Vec::new(),
        code: if answer.is_refused() {
            EXIT_REFUSED
        } else {
            EXIT_ANSWERED
        },
    }
}

/// The output of a command past the part of the engine built so far.
fn not_implemented(command: &str) -> Output {
    Output {
        stdout: Vec::new(),
        stderr: format!("tally {command}: not implemented\n").into_bytes(),
        code: EXIT_NOT_IMPLEMENTED,
    }
}

/// The usage text, for a command line the engine doesn't take.
fn usage() -> Output {
    Output {
        stdout: USAGE.as_bytes().to_vec(),
        stderr: Vec::new(),
        code: EXIT_USAGE,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EXIT_NOT_IMPLEMENTED, EXIT_REFUSED, EXIT_USAGE, File, join, pack, paths, run, table,
    };
    use std::ffi::OsString;

    /// The smallest model the parser takes.
    const MODEL: &[u8] =
        b"model m\n  tally 2\n  on 2026-01-01\n  start 2026-01-01\n  end 2026-12-31\n";

    fn args(words: &[&str]) -> Vec<OsString> {
        words.iter().map(OsString::from).collect()
    }

    fn files(pairs: &[(&str, &[u8])]) -> Vec<u8> {
        let files: Vec<File> = pairs
            .iter()
            .map(|(path, bytes)| File {
                path: (*path).to_owned(),
                bytes: bytes.to_vec(),
            })
            .collect();
        pack(&files)
    }

    fn out(words: &[&str], pairs: &[(&str, &[u8])]) -> (String, u8) {
        let output = run(&args(words), &files(pairs));
        let mut bytes = output.stdout;
        bytes.extend_from_slice(&output.stderr);
        (String::from_utf8(bytes).expect("UTF-8"), output.code)
    }

    #[test]
    fn the_table_is_netstring_pairs_in_path_order() {
        assert_eq!(
            files(&[("dir/b.tally", b""), ("a.tally", b"model a\n")]),
            b"7:a.tally,8:model a\n,11:dir/b.tally,0:,".to_vec(),
            "INTERFACE section 4"
        );
        let read = table(b"7:a.tally,8:model a\n,").expect("a table");
        assert_eq!(read.get("a.tally"), Some(&&b"model a\n"[..]), "read back");
        for bad in [
            &b"7:a.tally"[..],
            b"7:a.tally,1:x",
            b"x:a,1:b,",
            b"3:a,1:b,",
            b"1:a,1:b,1:a,1:c,",
            b"1:\xff,1:b,",
        ] {
            assert!(table(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn a_path_that_names_nothing_is_refused() {
        assert_eq!(
            out(&["run", "LOAD-PATH/nope.tally"], &[]),
            (
                "refused: LOAD-PATH LOAD-PATH/nope.tally doesn't exist\n".to_owned(),
                EXIT_REFUSED
            ),
            "the fixture LOAD-PATH/alone"
        );
        assert_eq!(
            out(
                &[
                    "run",
                    "LOAD-PATH/m.tally",
                    "LOAD-PATH/a.tally",
                    "LOAD-PATH/z.tally"
                ],
                &[
                    ("LOAD-PATH/m.tally", b"model m"),
                    ("LOAD-PATH/a.tally", b"broken"),
                ]
            ),
            (
                "refused: LOAD-PATH LOAD-PATH/z.tally doesn't exist\n".to_owned(),
                EXIT_REFUSED
            ),
            "read before parsed: the fixture LOAD-PATH/before-parse"
        );
    }

    #[test]
    fn bytes_that_are_not_utf8_are_refused() {
        assert_eq!(
            out(&["run", "a.tally"], &[("a.tally", b"\xc3")]),
            (
                "refused: LOAD-PARSE cannot parse: file is not UTF-8 (a.tally)\n".to_owned(),
                EXIT_REFUSED
            ),
            "named as given"
        );
    }

    #[test]
    fn a_directory_stands_for_its_tally_files() {
        let table = [
            ("r/a.tally", &b"\xff"[..]),
            ("r/notes.txt", b"\xff"),
            ("r/sub/b.tally", MODEL),
            ("rx.tally", b"\xff"),
        ];
        assert_eq!(
            out(&["run", "r"], &table).0,
            "refused: LOAD-PARSE cannot parse: file is not UTF-8 (r/a.tally)\n",
            "r/a.tally is read; r/notes.txt and rx.tally are not under r"
        );
        assert_eq!(
            out(&["run", "r/"], &table).0,
            "refused: LOAD-PARSE cannot parse: file is not UTF-8 (r/a.tally)\n",
            "a trailing slash"
        );
        assert_eq!(
            out(&["run", "r/sub"], &table),
            (String::new(), super::EXIT_ANSWERED),
            "read, parsed and loaded: no ask, nothing printed"
        );
        assert_eq!(
            out(&["run", "m.tally"], &[("m.tally", b"model m")]),
            (
                "refused: LOAD-HEADER missing tally version (m.tally:1)\n".to_owned(),
                EXIT_REFUSED
            ),
            "a parse refusal"
        );
        let later = [MODEL, b"\nask x: explain y\n"].concat();
        assert_eq!(
            out(&["run", "m.tally"], &[("m.tally", &later)]),
            (
                "tally run: not implemented\n".to_owned(),
                EXIT_NOT_IMPLEMENTED
            ),
            "explain is built later"
        );
    }

    #[test]
    fn the_arguments_read_as_refcore_reads_them() {
        let words = args(&["run", "a.tally", "--pin=bytes", "b", "--ask", "q", "r"]);
        assert_eq!(paths(&words), ["a.tally", "b"], "paths before --ask");
        assert_eq!(
            paths(&args(&["reprint", "x"])),
            ["x"],
            "reprint takes paths"
        );
        assert!(paths(&args(&["lod", "x"])).is_empty(), "not a command");
        assert_eq!(join("r", "a.tally"), "r/a.tally", "one slash");
        assert_eq!(join("r/", "a.tally"), "r/a.tally", "still one");
    }

    #[test]
    fn anything_else_is_a_usage_error() {
        let cases: [&[&str]; 4] = [&[], &["lod"], &["test"], &["load", "a"]];
        for words in cases {
            let (text, code) = out(words, &[]);
            assert_eq!(code, EXIT_USAGE, "{words:?}");
            assert!(text.starts_with("usage: tally run"), "{text}");
        }
        assert_eq!(
            run(&args(&["run"]), b"1:a").code,
            EXIT_USAGE,
            "a table that isn't netstrings"
        );
    }

    #[cfg(feature = "counters")]
    #[test]
    fn stats_adds_the_counters_line() {
        let output = run(
            &args(&["run", "--stats", "a.tally"]),
            &files(&[("a.tally", MODEL)]),
        );
        assert_eq!(output.code, super::EXIT_ANSWERED, "run --stats");
        assert!(
            output
                .stderr
                .ends_with(b"recomputed_locally=0 num_results=0\n"),
            "{:?}",
            output.stderr
        );
    }
}
