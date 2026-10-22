//! A benchmark stub (gate 12). Benchmarks are instruments: the lead writes the real
//! ones on the builder's request (R-91).

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn parse_stub(c: &mut Criterion) {
    c.bench_function("parse_stub", |b| b.iter(|| tally::parse(black_box(b""))));
}

criterion_group!(benches, parse_stub);
criterion_main!(benches);
