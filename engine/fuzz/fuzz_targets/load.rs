//! Fuzz target `load`: any bytes, split at the byte 0xFF into one to three files (`a.tally`,
//! `b.tally`, `c.tally`; a third piece keeps any later 0xFF, which no UTF-8 text holds), into
//! load alone (`tally::load_report`). What load prints is one `refused: <code> …` line whose
//! code is one of L §8.3's `LOAD-*` codes (a `member <id>: ` prefix allowed), or `scenario
//! <id>: refused: <LOAD-* code> …` lines and `warning: <code> …` lines (L §8, §8.5): never
//! `ENGINE-INVARIANT`, any other code or line, a panic or a hang (DONE gate 9; the builder's
//! plan §5, after M0's G3).
//!
//! `NotImplemented`, a form the engine builds later, is allowed for now and counted: every
//! power-of-two input prints `load: <n> inputs, <m> not implemented` to stderr, so a run's log
//! shows the share. To make it a finding, set `NOT_IMPLEMENTED_ALLOWED` below to `false`: one
//! line.

#![no_main]

use libfuzzer_sys::fuzz_target;
use std::sync::atomic::{AtomicU64, Ordering};
use tally::base::Code;
use tally::{Given, NotImplemented};

/// Whether `NotImplemented` passes (counted). M0 leaves 76 load stops "not built" on purpose,
/// each listed with the milestone or issue that builds it in BUILD-LOG's "M0 built" entry
/// (#86-#90, M1, M2, M5c, ...); as findings they would be all this target finds (the M0
/// coordinator's pick P14, issue #56). It flips to `false` after the last load stop is built,
/// at the latest when M5c lands.
const NOT_IMPLEMENTED_ALLOWED: bool = true;

static INPUTS: AtomicU64 = AtomicU64::new(0);
static NOT_BUILT: AtomicU64 = AtomicU64::new(0);

/// The input's files: its pieces at 0xFF, at most three.
fn files(data: &[u8]) -> Vec<Given> {
    ["a.tally", "b.tally", "c.tally"]
        .into_iter()
        .zip(data.splitn(3, |&b| b == 0xFF))
        .map(|(path, bytes)| Given {
            path: path.to_owned(),
            bytes: Some(bytes.to_vec()),
        })
        .collect()
}

/// Whether `word` is a code of the class `class` picks out.
fn is_code(word: &str, class: fn(&Code) -> bool) -> bool {
    Code::ALL.iter().any(|c| class(c) && c.as_str() == word)
}

fn is_load(code: &Code) -> bool {
    matches!(code, Code::Load(_))
}

fn is_warn(code: &Code) -> bool {
    matches!(code, Code::Warn(_))
}

/// The first word of a printed refusal is a `LOAD-*` code, after an optional `member <id>: `.
fn assert_load_refusal(text: &str, line: &str) {
    let text = match text.strip_prefix("member ").and_then(|t| t.split_once(": ")) {
        Some((_, rest)) => rest,
        None => text,
    };
    let code = text.split(' ').next().unwrap_or_default();
    assert!(is_code(code, is_load), "a refusal whose code is {code:?}, not one of L §8.3's: {line}");
}

/// Every line load prints is one of its three kinds, with a code of its class.
fn assert_report(lines: &[String]) {
    for line in lines {
        if let Some(text) = line.strip_prefix("refused: ") {
            assert!(lines.len() == 1, "a load refusal printed with other lines: {lines:?}");
            assert_load_refusal(text, line);
        } else if let Some((_, text)) = line.strip_prefix("scenario ").and_then(|t| t.split_once(": refused: ")) {
            assert_load_refusal(text, line);
        } else if let Some(text) = line.strip_prefix("warning: ") {
            let code = text.split(' ').next().unwrap_or_default();
            assert!(is_code(code, is_warn), "a warning whose code is {code:?}, not one of L §8.5's: {line}");
        } else {
            panic!("a line load doesn't print: {line:?}");
        }
    }
}

fn count(not_built: bool) {
    let n = INPUTS.fetch_add(1, Ordering::Relaxed) + 1;
    let m = NOT_BUILT.fetch_add(u64::from(not_built), Ordering::Relaxed) + u64::from(not_built);
    if n.is_power_of_two() {
        eprintln!("load: {n} inputs, {m} not implemented");
    }
}

fuzz_target!(|data: &[u8]| {
    match tally::load_report(files(data)) {
        Ok(answer) => {
            assert_report(answer.lines());
            count(false);
        }
        Err(NotImplemented) => {
            assert!(NOT_IMPLEMENTED_ALLOWED, "load answered NotImplemented");
            count(true);
        }
    }
});
