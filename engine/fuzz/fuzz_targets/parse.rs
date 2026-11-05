//! Fuzz target `parse`: any bytes, as one file, into the loader's parser (`tally::parse`).
//! A crash, a hang or an abort is a finding (DONE gate 9), and so is a rejection without a
//! code of L §8.3's catalogue: every rejected input gets a `LOAD-*` code, never "not built"
//! (gate 9; the builder's plan §5, after M0's G1).

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
    if let Err(stop) = tally::parse(data) {
        assert_load_code(&stop);
    }
});
