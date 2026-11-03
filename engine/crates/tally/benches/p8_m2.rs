//! P8-M2, the gcd of a big and a small number: its wall time on the reference Mac.
//! Spec: P §8.4 (P8-M2), §6 (the tiers); L §7.1.
//! Twin: none, an instrument (R-91); its instruction count is
//! `engine/tools/gungraun/benches/p8_m2.rs`, on the same inputs.
//! Invariants: the inputs and the known answers are the builder's request at S (BUILD-LOG,
//! 2026-11-01); after timing, the quotient is checked against n, so a fast wrong answer
//! fails the run.
//!
//! The gcd isn't public, so the bench reaches it through division, which reduces by it.

use core::cmp::Ordering;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use tally::num::{Num, NumError};

/// 2^63 − 2^7 = 2^7 × (2^56 − 1): it shares 2^8 − 1 = 255 with 2^5048 − 1.
const DIVISOR: i64 = 9_223_372_036_854_775_680;
/// The divisor over 255: the reduced quotient's denominator.
const COFACTOR: i64 = 36_170_086_419_038_336;

/// 2^5048 − 1, 5 048 bits, built by doubling (untimed).
fn mersenne_5048() -> Result<Num, NumError> {
    let two = Num::int(2);
    let mut power = Num::int(1);
    for _ in 0..5_048 {
        power = power.checked_mul(&two)?;
    }
    power.checked_sub(&Num::int(1))
}

/// Times the division, then says whether its quotient is the known one.
fn timed(bench: &mut Criterion) -> Result<bool, NumError> {
    let n = mersenne_5048()?;
    let m = Num::int(DIVISOR);
    bench.bench_function("P8-M2", |t| {
        t.iter(|| black_box(&n).checked_div(black_box(&m)));
    });
    let q = n.checked_div(&m)?;
    Ok(q.checked_mul(&Num::int(COFACTOR))?.is_integer()
        && q.checked_mul(&m)?.compare(&n)? == Ordering::Equal)
}

fn p8_m2(bench: &mut Criterion) {
    assert_eq!(timed(bench), Ok(true), "P8-M2: a fast wrong answer fails");
}

criterion_group!(benches, p8_m2);
criterion_main!(benches);
