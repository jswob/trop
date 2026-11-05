//! Fuzz target `format`: any bytes, as one file, into the canonical printer
//! (`tally::format_file`, L §2.4). The oracle is the round trip: a text it prints parses and
//! prints as itself, byte for byte (idempotence). A rejection is a refusal with one of L
//! §8.3's `LOAD-*` codes; a crash, a hang or an abort is a finding (DONE gate 9; the
//! builder's plan §5, after M0's G2).
//!
//! A "not built" stop, a form whose order the engine prints later, is allowed for now and
//! counted, as the `load` target counts `NotImplemented`: every power-of-two input prints
//! `format: <n> inputs, <m> not built` to stderr, so a run's log shows the share. To make it a
//! finding, set `NOT_BUILT_ALLOWED` below to `false`: one line. The canonical text's own
//! reprint is held to the law whatever it meets: a stop there fails.

#![no_main]

use libfuzzer_sys::fuzz_target;
use std::sync::atomic::{AtomicU64, Ordering};
use tally::base::Code;
use tally::syntax::Stop;

/// Whether a "not built" stop passes (counted). M0 leaves the canonical order of keys of a
/// member's index or kind "not built" on purpose, for M5c, as refcore does (`NotYet`), and
/// BUILD-LOG's "M0 built" entry lists it there; a plain file reaches it through a qualified
/// domain such as `c.nt` (the gate's advisory fuzz, F1). As a finding it would stop every run
/// at its first such input (the M0 coordinator's pick P22, issue #56). It flips to `false`
/// when M5c builds a member's key order.
const NOT_BUILT_ALLOWED: bool = true;

static INPUTS: AtomicU64 = AtomicU64::new(0);
static NOT_BUILT: AtomicU64 = AtomicU64::new(0);

/// A rejection is a refusal whose code is one of L §8.3's `LOAD-*` codes, or, while
/// `NOT_BUILT_ALLOWED`, a "not built" stop; whether it was that stop.
fn assert_rejection(stop: &Stop) -> bool {
    match stop {
        Stop::Refused(refusal) => {
            assert!(
                matches!(refusal.code(), Code::Load(_)),
                "a rejection whose code is {}, not one of L §8.3's",
                refusal.code().as_str()
            );
            false
        }
        Stop::NotBuilt(what) => {
            assert!(NOT_BUILT_ALLOWED, "a rejection as not built: {what}");
            true
        }
        Stop::Deep => panic!("the parser stopped as Deep, which only a read may"),
    }
}

fn count(not_built: bool) {
    let n = INPUTS.fetch_add(1, Ordering::Relaxed) + 1;
    let m = NOT_BUILT.fetch_add(u64::from(not_built), Ordering::Relaxed) + u64::from(not_built);
    if n.is_power_of_two() {
        eprintln!("format: {n} inputs, {m} not built");
    }
}

fuzz_target!(|data: &[u8]| {
    let not_built = match tally::format_file(data) {
        Ok(canonical) => match tally::format_file(canonical.as_bytes()) {
            Ok(again) => {
                assert!(
                    again == canonical,
                    "the canonical text reprints otherwise:\n--- printed\n{canonical}\n--- reprinted\n{again}"
                );
                false
            }
            Err(stop) => panic!("the canonical text is rejected: {stop:?}\n--- printed\n{canonical}"),
        },
        Err(stop) => assert_rejection(&stop),
    };
    count(not_built);
});
