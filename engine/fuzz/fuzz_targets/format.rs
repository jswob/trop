//! Fuzz target `format`: any bytes, as one file, into the canonical printer
//! (`tally::format_file`, L §2.4). The oracle is the round trip: a text it prints parses and
//! prints as itself, byte for byte (idempotence). A rejection is a refusal with one of L
//! §8.3's `LOAD-*` codes, never "not built"; a crash, a hang or an abort is a finding
//! (DONE gate 9; the builder's plan §5, after M0's G2).

#![no_main]

use libfuzzer_sys::fuzz_target;
use tally::base::Code;
use tally::syntax::Stop;

/// A rejection is a refusal whose code is one of L §8.3's `LOAD-*` codes.
fn assert_load_code(stop: &Stop) {
    match stop {
        Stop::Refused(refusal) => assert!(
            matches!(refusal.code(), Code::Load(_)),
            "a rejection whose code is {}, not one of L §8.3's",
            refusal.code().as_str()
        ),
        Stop::NotBuilt(what) => panic!("a rejection as not built: {what}"),
        Stop::Deep => panic!("the parser stopped as Deep, which only a read may"),
    }
}

fuzz_target!(|data: &[u8]| {
    match tally::format_file(data) {
        Ok(canonical) => match tally::format_file(canonical.as_bytes()) {
            Ok(again) => assert!(
                again == canonical,
                "the canonical text reprints otherwise:\n--- printed\n{canonical}\n--- reprinted\n{again}"
            ),
            Err(stop) => panic!("the canonical text is rejected: {stop:?}\n--- printed\n{canonical}"),
        },
        Err(stop) => assert_load_code(&stop),
    }
});
