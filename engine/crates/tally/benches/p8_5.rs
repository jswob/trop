//! P8-5, one look-ahead decision: fence F5's range minimum alone, its wall time on the
//! reference Mac.
//! Spec: P §8.1 (P8-5), §2.3 (the tree, about 14 steps); E §4.3 (a served
//! `low(x, a .. b) from t`); L §5.14.1.
//! Twin: none, an instrument (R-91); P8-5's instruction limit is P8-M4's (P §8.1), so it has
//! no gungraun twin.
//! Invariants: the library is reached only through `tally::walk::RangeMinimum`, the fence's
//! own tree over days and per-day sums as a gathered tree holds them (G9's builder, 238fbfb);
//! the tree is built untimed and must hold whole minor units (pick P56); before timing, every
//! window's lowest change is checked against a plain walk of the same steps, so a fast wrong
//! answer fails the run.
//!
//! P8-5's size: a 33-year place, n = 12 054 days from 2025-01-01, one step each, in a unit
//! whose minor unit is 0.01: 1.25 a day in, and every seventh day 9.00 out instead, so a
//! window's running sum dips below zero and the read adds the dip. Each timed read is
//! `low(x, d .. E) from d` for the next day d in turn, as G9's profile read them (`low_all`):
//! three halvings over the days and two queries of the tree.

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use tally::num::Num;
use tally::time::Day;
use tally::walk::RangeMinimum;

/// The days of the place, S through E: 33 years.
const DAYS: usize = 12_054;
/// A day's step in, in minor units (1.25).
const IN: i64 = 125;
/// Every seventh day's step out instead, in minor units (9.00).
const OUT: i64 = -900;
/// The minor units in one unit.
const MINOR: i64 = 100;

/// A read: the day its legs are counted from, and its window, first through last.
type Window = (Day, Day, Day);

/// Day `i`'s step, in minor units.
fn step(i: usize) -> i64 {
    if i.checked_rem_euclid(7) == Some(6) {
        OUT
    } else {
        IN
    }
}

/// The days, in order, and each day's step in minor units.
fn ledger() -> Option<(Vec<Day>, Vec<i64>)> {
    let mut days = Vec::with_capacity(DAYS);
    let mut day = Day::from_ymd(2025, 1, 1)?;
    for _ in 0..DAYS {
        days.push(day);
        day = day.next()?;
    }
    Some((days, (0..DAYS).map(step).collect()))
}

/// Each read `low(x, d .. E) from d`'s lowest change, by a plain walk of the steps: the day's
/// own step, then the least running sum of the days after it through E, if below zero.
fn known(steps: &[i64]) -> Option<Vec<i64>> {
    let mut through = Vec::with_capacity(steps.len());
    let mut sum = 0_i64;
    for s in steps {
        sum = sum.checked_add(*s)?;
        through.push(sum);
    }
    let mut least_from = Vec::with_capacity(steps.len());
    let mut low = i64::MAX;
    for t in through.iter().rev() {
        low = low.min(*t);
        least_from.push(low);
    }
    least_from.reverse();
    let after = least_from
        .iter()
        .skip(1)
        .map(Some)
        .chain(std::iter::once(None));
    steps
        .iter()
        .zip(&through)
        .zip(after)
        .map(
            |((s, t), after)| match after.map(|least| least.checked_sub(*t)) {
                Some(Some(dip)) if dip < 0 => s.checked_add(dip),
                Some(Some(_)) | None => Some(*s),
                Some(None) => None,
            },
        )
        .collect()
}

/// Whether the tree gives each window the plain walk's lowest change.
fn right(tree: &RangeMinimum, windows: &[Window], known: &[i64]) -> bool {
    windows.len() == known.len()
        && windows
            .iter()
            .zip(known)
            .all(|(&(counts, first, last), want)| {
                Num::ratio(*want, MINOR).is_ok_and(|want| {
                    tree.lowest_change(counts, (first, last))
                        .is_ok_and(|got| got == want)
                })
            })
}

/// The tree in whole minor units, every read's window, and whether the tree answers each one
/// as the plain walk does.
fn setup() -> Option<(RangeMinimum, Vec<Window>, bool)> {
    let (days, minor_steps) = ledger()?;
    let last = *days.last()?;
    let windows: Vec<Window> = days.iter().map(|d| (*d, *d, last)).collect();
    let mut steps = Vec::with_capacity(DAYS);
    for s in &minor_steps {
        let Ok(n) = Num::ratio(*s, MINOR) else {
            return None;
        };
        steps.push(n);
    }
    let Ok(cent) = Num::ratio(1, MINOR) else {
        return None;
    };
    let Ok(tree) = RangeMinimum::of(days, steps, Some(&cent)) else {
        return None;
    };
    let answered = tree.in_minor_units() && right(&tree, &windows, &known(&minor_steps)?);
    Some((tree, windows, answered))
}

fn p8_5(bench: &mut Criterion) {
    let built = setup();
    assert!(
        built.as_ref().is_some_and(|(_, _, answered)| *answered),
        "P8-5: the tree in whole minor units, answering every read as the plain walk does"
    );
    let Some((tree, windows, _)) = built else {
        return;
    };
    let mut reads = windows.iter().cycle();
    bench.bench_function("P8-5", |t| {
        t.iter(|| {
            reads.next().map(|&(counts, first, last)| {
                tree.lowest_change(black_box(counts), black_box((first, last)))
            })
        });
    });
}

criterion_group! {
    name = benches;
    // a read takes well under a microsecond: criterion's default, 100 samples, each of many
    // reads, taken in turn from the 12 054 windows
    config = Criterion::default();
    targets = p8_5
}
criterion_main!(benches);
