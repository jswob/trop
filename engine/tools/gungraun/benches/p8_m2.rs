//! P8-M2, the gcd of a big and a small number, reached through division: its instruction
//! count on Linux CI (D4-2, BUILDER §13.3).
//! Spec: P §8.4 (P8-M2), §6 (the tiers); L §7.1.
//! Twin: none, an instrument (R-91); its wall time is `engine/crates/tally/benches/p8_m2.rs`,
//! on the same inputs.
//! Invariants: the hard limit comes from `engine/tools/perf limits p8_m2`, never from this
//! file; the inputs are built before the count starts, and the teardown, outside the count,
//! checks the quotient against n, so a fast wrong answer fails the run.

use core::cmp::Ordering;
use std::hint::black_box;

use gungraun::prelude::*;
use tally::num::{Num, NumError};

/// 2^63 − 2^7 = 2^7 × (2^56 − 1): it shares 2^8 − 1 = 255 with 2^5048 − 1.
const DIVISOR: i64 = 9_223_372_036_854_775_680;
/// The divisor over 255: the reduced quotient's denominator.
const COFACTOR: i64 = 36_170_086_419_038_336;

/// 2^5048 − 1, 5 048 bits, built by doubling.
fn mersenne_5048() -> Result<Num, NumError> {
    let two = Num::int(2);
    let mut power = Num::int(1);
    for _ in 0..5_048 {
        power = power.checked_mul(&two)?;
    }
    power.checked_sub(&Num::int(1))
}

/// Whether q × the cofactor is whole and q × the divisor is n.
fn known(q: Result<Num, NumError>) -> Result<bool, NumError> {
    let q = q?;
    Ok(q.checked_mul(&Num::int(COFACTOR))?.is_integer()
        && q.checked_mul(&Num::int(DIVISOR))?
            .compare(&mersenne_5048()?)?
            == Ordering::Equal)
}

fn check(q: Result<Num, NumError>) {
    assert_eq!(known(q), Ok(true), "P8-M2: a fast wrong answer fails");
}

#[library_benchmark(teardown = check)]
#[bench::big_by_small(mersenne_5048())]
fn p8_m2(n: Result<Num, NumError>) -> Result<Num, NumError> {
    black_box(n)?.checked_div(black_box(&Num::int(DIVISOR)))
}

library_benchmark_group!(name = micro, benchmarks = p8_m2);

main!(library_benchmark_groups = micro);
