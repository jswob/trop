//! P8-M1, `Num` add and multiply on the small tier: its wall time on the reference Mac.
//! Spec: P §8.4 (P8-M1), §6 (the tiers); L §7.1.
//! Twin: none, an instrument (R-91); its instruction count is
//! `engine/tools/gungraun/benches/p8_m1.rs`, on the same inputs.
//! Invariants: the inputs and the known answers are the builder's request at S (BUILD-LOG,
//! 2026-11-01); after timing, every result is compared with its known value, so a fast
//! wrong answer fails the run.

use core::cmp::Ordering;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use tally::num::{Num, NumError};

/// One timed step: a short-decimal add, a short-decimal multiply and a whole multiply.
fn small_ops(a: &Num, b: &Num, c: &Num, d: &Num) -> Result<[Num; 3], NumError> {
    Ok([a.checked_add(b)?, a.checked_mul(b)?, c.checked_mul(d)?])
}

/// Times the step, then says whether its results are the known ones.
fn timed(bench: &mut Criterion) -> Result<bool, NumError> {
    let a = Num::parse_decimal("1234.5678")?;
    let b = Num::parse_decimal("0.0976")?;
    let c = Num::int(6_536);
    let d = Num::int(12);
    bench.bench_function("P8-M1", |t| {
        t.iter(|| small_ops(black_box(&a), black_box(&b), black_box(&c), black_box(&d)));
    });
    let [sum, product, whole] = small_ops(&a, &b, &c, &d)?;
    Ok(
        sum.compare(&Num::parse_decimal("1234.6654")?)? == Ordering::Equal
            && product.compare(&Num::parse_decimal("120.49381728")?)? == Ordering::Equal
            && whole.compare(&Num::int(78_432))? == Ordering::Equal,
    )
}

fn p8_m1(bench: &mut Criterion) {
    assert_eq!(timed(bench), Ok(true), "P8-M1: a fast wrong answer fails");
}

criterion_group!(benches, p8_m1);
criterion_main!(benches);
