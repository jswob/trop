//! Fuzz target `load`: any bytes, split at the byte 0xFF into one to three files (`a.tally`,
//! `b.tally`, `c.tally`; a third piece keeps any later 0xFF, which no UTF-8 text holds), into
//! load alone (`tally::load_report`). What load prints is one `refused: <code> …` line whose
//! code is one of L §8.3's `LOAD-*` codes (a `member <id>: ` prefix allowed), or `scenario
//! <id>: refused: <LOAD-* code> …` lines and `warning: <code> …` lines (L §8, §8.5): never
//! `ENGINE-INVARIANT`, any other code or line, a panic or a hang (DONE gate 9; the builder's
//! plan §5, after M0's G3).
//!
//! `NotImplemented`, a form the engine builds later, is allowed for now and counted, each by
//! the stop it names, so a run's log says which stops a generated text reaches (#192): the
//! first input to reach a stop prints `load: input <n> reached a new stop: <stop>`, and every
//! power-of-two input prints `load: <n> inputs, <m> not implemented`, then `: <k> <stop>` for
//! each stop met, `; ` between them, by the stop's text, to stderr. A stop is the text of
//! load's `NotBuilt` (the construct the engine doesn't build yet), or `a def past the counted
//! depth` for `Deep`; `NotImplemented` itself carries no text, so the target loads the same
//! files again through `tally::load` (what `load_report` does before it prints) to read it, and
//! counts an input it can't name as `unnamed`. A report, never a failure. To make
//! `NotImplemented` a finding, set `NOT_IMPLEMENTED_ALLOWED` below to `false`: one line.

#![no_main]

use libfuzzer_sys::fuzz_target;
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use tally::base::Code;
use tally::load::{load_run, split_files};
use tally::syntax::Stop;
use tally::{Given, NotImplemented, Pin, Source};

/// Whether `NotImplemented` passes (counted). M0 leaves 76 load stops "not built" on purpose,
/// each listed with the milestone or issue that builds it in BUILD-LOG's "M0 built" entry
/// (#86-#90, M1, M2, M5c, ...); as findings they would be all this target finds (the M0
/// coordinator's pick P14, issue #56). It flips to `false` after the last load stop is built,
/// at the latest when M5c lands.
const NOT_IMPLEMENTED_ALLOWED: bool = true;

static INPUTS: AtomicU64 = AtomicU64::new(0);
static NOT_BUILT: AtomicU64 = AtomicU64::new(0);
/// Each stop met, by its text, and how many inputs reached it.
static STOPS: Mutex<BTreeMap<&'static str, u64>> = Mutex::new(BTreeMap::new());

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

/// The stop a `NotImplemented` input reached: its files read and split as `load_report` reads
/// them (every piece is UTF-8 here, or load would have refused it), then loaded, and the
/// `Stop` that load returns named. `unnamed` if the second load doesn't stop the same way.
fn stop_of(data: &[u8]) -> &'static str {
    let mut sources = Vec::new();
    for file in files(data) {
        let Some(text) = file.bytes.and_then(|b| String::from_utf8(b).ok()) else {
            return "unnamed";
        };
        sources.push(Source { path: file.path, text });
    }
    let Ok(split) = split_files(sources) else {
        return "unnamed";
    };
    match load_run(&split, &BTreeMap::new(), Pin::Canonical) {
        Err(Stop::NotBuilt(what)) => what,
        Err(Stop::Deep) => "a def past the counted depth",
        Ok(_) | Err(Stop::Refused(_)) => "unnamed",
    }
}

/// Counts one input, and the stop it reached if any; prints a stop the first time it is met
/// and the counts at every power-of-two input.
fn count(stop: Option<&'static str>) {
    let n = INPUTS.fetch_add(1, Ordering::Relaxed) + 1;
    let not_built = u64::from(stop.is_some());
    let m = NOT_BUILT.fetch_add(not_built, Ordering::Relaxed) + not_built;
    let mut stops = STOPS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(what) = stop {
        let seen = stops.entry(what).or_insert(0);
        *seen += 1;
        if *seen == 1 {
            eprintln!("load: input {n} reached a new stop: {what}");
        }
    }
    if n.is_power_of_two() {
        let by_stop: Vec<String> = stops.iter().map(|(what, k)| format!("{k} {what}")).collect();
        let named = if by_stop.is_empty() { String::new() } else { format!(": {}", by_stop.join("; ")) };
        eprintln!("load: {n} inputs, {m} not implemented{named}");
    }
}

fuzz_target!(|data: &[u8]| {
    match tally::load_report(files(data)) {
        Ok(answer) => {
            assert_report(answer.lines());
            count(None);
        }
        Err(NotImplemented) => {
            assert!(NOT_IMPLEMENTED_ALLOWED, "load answered NotImplemented");
            count(Some(stop_of(data)));
        }
    }
});
