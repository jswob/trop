//! The instruction count of the stub's one entry point, proving the gungraun form on Linux
//! CI (D4-2, BUILDER §13.3). Not a P8 row: the micro rows' benches (P8-M1 to P8-M4) come
//! with S, written by the lead on the builder's request (R-91).
//! Spec: none, a stub.
//! Twin: none, a stub.
//! Invariants: the hard limit comes from `engine/tools/perf limits`, never from this file.

use std::hint::black_box;

use gungraun::prelude::*;

#[library_benchmark]
fn parse_empty() -> Result<(), tally::NotImplemented> {
    black_box(tally::parse(black_box(b"")))
}

library_benchmark_group!(name = stub, benchmarks = parse_empty);

main!(library_benchmark_groups = stub);
