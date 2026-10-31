//! Fuzz target `parse`: any bytes into the loader. A crash, a hang or an abort is a
//! finding (DONE gate 9).

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _refused = tally::parse(data).is_err();
});
