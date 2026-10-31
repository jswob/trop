//! One corpus record run on the engine, in process (`INTERFACE.md` §4).

use crate::corpus::{Record, Run};
use std::ffi::OsString;

/// Runs `record` through `tally_cli::run` and reads its output as `run.py` does: standard
/// output then standard error, as UTF-8, with `\r\n` and `\r` read as `\n`. Output that
/// isn't UTF-8 is read as a line saying so, which no law accepts.
pub fn run_record(record: &Record) -> Run {
    let args: Vec<OsString> = record.args.iter().map(OsString::from).collect();
    let output = tally_cli::run(&args, &record.files);
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
