//! P8-M3, one big multiply and its reduction at about 5 049 bits: its wall time on the Mac.
//! Spec: P §8.4 (P8-M3), §6 (the tiers); L §7.1.
//! Twin: none, an instrument (R-91); its instruction count is
//! `engine/tools/gungraun/benches/p8_m3.rs`, on the same inputs.
//! Invariants: the inputs and the known answer are the builder's request at S (BUILD-LOG,
//! 2026-11-01); after timing, the square times 1/x twice must be 1, so a fast wrong answer
//! fails the run.
//!
//! x = (2^5048 − 1)/(2^5048 + 1), and x × x takes two balanced gcds and two multiplies of
//! 5 048- and 5 049-bit integers.

use core::cmp::Ordering;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use tally::num::{Num, NumError};

/// 2^5048 − 1, 5 048 bits, built by doubling (untimed).
fn mersenne_5048() -> Result<Num, NumError> {
    let two = Num::int(2);
    let mut power = Num::int(1);
    for _ in 0..5_048 {
        power = power.checked_mul(&two)?;
    }
    power.checked_sub(&Num::int(1))
}

/// Times the square, then says whether it is the known one.
fn timed(bench: &mut Criterion) -> Result<bool, NumError> {
    let n = mersenne_5048()?;
    let x = n.checked_div(&n.checked_add(&Num::int(2))?)?;
    let y = Num::int(1).checked_div(&x)?;
    bench.bench_function("P8-M3", |t| {
        t.iter(|| black_box(&x).checked_mul(black_box(&x)));
    });
    let r = x.checked_mul(&x)?;
    Ok(r.checked_mul(&y)?.checked_mul(&y)?.compare(&Num::int(1))? == Ordering::Equal)
}

fn p8_m3(bench: &mut Criterion) {
    assert_eq!(timed(bench), Ok(true), "P8-M3: a fast wrong answer fails");
}

criterion_group!(benches, p8_m3);
criterion_main!(benches);
