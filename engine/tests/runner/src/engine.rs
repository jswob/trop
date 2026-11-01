//! A run on the engine, in process (`INTERFACE.md` §4): a corpus record, or any command
//! line with its file table.

use crate::corpus::{Record, Run};
use std::ffi::OsString;

/// Runs `record` on the engine, as [`run_args`] does.
pub fn run_record(record: &Record) -> Run {
    run_args(&record.args, &record.files)
}

/// Runs the command line `args` (the words after the program name) on the file table
/// `files` through `tally_cli::run`, and reads its output as `run.py` does: standard output
/// then standard error, as UTF-8, with `\r\n` and `\r` read as `\n`. Output that isn't UTF-8
/// is read as a line saying so, which no test accepts.
pub fn run_args(args: &[String], files: &[u8]) -> Run {
    let args: Vec<OsString> = args.iter().map(OsString::from).collect();
    let output = tally_cli::run(&args, files);
    let mut bytes = output.stdout;
    bytes.extend_from_slice(&output.stderr);
    let out = match String::from_utf8(bytes) {
        Ok(text) => text.replace("\r\n", "\n").replace('\r', "\n"),
        Err(e) => format!("the output is not UTF-8: {e}\n"),
    };
    Run {
        out,
        code: output.code,
    }
}

#[cfg(test)]
mod tests {
    use crate::corpus::{netstrings, pack};

    #[test]
    fn the_file_table_is_netstring_pairs() {
        let table = pack(&[("a.tally", "model a\n"), ("dir/b.tally", "")]);
        assert_eq!(
            table,
            b"7:a.tally,8:model a\n,11:dir/b.tally,0:,".to_vec(),
            "INTERFACE.md section 4"
        );
        let parts = netstrings(&table).expect("it reads back");
        assert_eq!(parts.len(), 4, "two files, a path and a content each");
    }
}
