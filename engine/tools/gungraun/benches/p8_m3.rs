//! P8-M3, one big multiply and its reduction at about 5 049 bits: its instruction count on
//! Linux CI (D4-2, BUILDER §13.3).
//! Spec: P §8.4 (P8-M3), §6 (the tiers); L §7.1.
//! Twin: none, an instrument (R-91); its wall time is `engine/crates/tally/benches/p8_m3.rs`,
//! on the same inputs.
//! Invariants: the hard limit comes from `engine/tools/perf limits p8_m3`, never from this
//! file; x is built before the count starts, and the teardown, outside the count, checks
//! that the square times 1/x twice is 1, so a fast wrong answer fails the run.

use core::cmp::Ordering;
use std::hint::black_box;

use gungraun::prelude::*;
use tally::num::{Num, NumError};

/// x = (2^5048 − 1)/(2^5048 + 1), its numerator built by doubling.
fn x() -> Result<Num, NumError> {
    let two = Num::int(2);
    let mut power = Num::int(1);
    for _ in 0..5_048 {
        power = power.checked_mul(&two)?;
    }
    let n = power.checked_sub(&Num::int(1))?;
    n.checked_div(&n.checked_add(&Num::int(2))?)
}

/// Whether r × (1/x) × (1/x) is 1.
fn known(r: Result<Num, NumError>) -> Result<bool, NumError> {
    let y = Num::int(1).checked_div(&x()?)?;
    Ok(r?.checked_mul(&y)?.checked_mul(&y)?.compare(&Num::int(1))? == Ordering::Equal)
}

fn check(r: Result<Num, NumError>) {
    assert_eq!(known(r), Ok(true), "P8-M3: a fast wrong answer fails");
}

#[library_benchmark(teardown = check)]
#[bench::balanced(x())]
fn p8_m3(x: Result<Num, NumError>) -> Result<Num, NumError> {
    let x = black_box(x)?;
    x.checked_mul(&x)
}

library_benchmark_group!(name = micro, benchmarks = p8_m3);

main!(library_benchmark_groups = micro);
