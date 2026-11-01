//! P8-M1, `Num` add and multiply on the small tier: its instruction count on Linux CI
//! (D4-2, BUILDER §13.3).
//! Spec: P §8.4 (P8-M1), §6 (the tiers); L §7.1.
//! Twin: none, an instrument (R-91); its wall time is `engine/crates/tally/benches/p8_m1.rs`,
//! on the same inputs.
//! Invariants: the hard limit comes from `engine/tools/perf limits p8_m1`, never from this
//! file; the inputs are built before the count starts, and the teardown, outside the count,
//! compares every result with its known value, so a fast wrong answer fails the run.

use core::cmp::Ordering;
use std::hint::black_box;

use gungraun::prelude::*;
use tally::num::{Num, NumError};

/// The builder's inputs (BUILD-LOG, 2026-11-01): two short decimals and two whole numbers.
fn inputs() -> Result<[Num; 4], NumError> {
    Ok([
        Num::parse_decimal("1234.5678")?,
        Num::parse_decimal("0.0976")?,
        Num::int(6_536),
        Num::int(12),
    ])
}

/// Whether the results are the known ones.
fn known(results: Result<[Num; 3], NumError>) -> Result<bool, NumError> {
    let [sum, product, whole] = results?;
    Ok(
        sum.compare(&Num::parse_decimal("1234.6654")?)? == Ordering::Equal
            && product.compare(&Num::parse_decimal("120.49381728")?)? == Ordering::Equal
            && whole.compare(&Num::int(78_432))? == Ordering::Equal,
    )
}

fn check(results: Result<[Num; 3], NumError>) {
    assert_eq!(known(results), Ok(true), "P8-M1: a fast wrong answer fails");
}

#[library_benchmark(teardown = check)]
#[bench::small_tier(inputs())]
fn p8_m1(given: Result<[Num; 4], NumError>) -> Result<[Num; 3], NumError> {
    let [a, b, c, d] = black_box(given)?;
    Ok([a.checked_add(&b)?, a.checked_mul(&b)?, c.checked_mul(&d)?])
}

library_benchmark_group!(name = micro, benchmarks = p8_m1);

main!(library_benchmark_groups = micro);
