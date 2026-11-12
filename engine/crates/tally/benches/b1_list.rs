//! B1, a list index's lookup at 100 000 keys (#97's G7, P §3.4, D2-5): its wall time on the
//! reference Mac.
//! Spec: P §3.4 (an index's keys found by halving), D2-5; L §5.11 (`sum … for k in`).
//! Twin: none, an instrument (R-91); its instruction count is
//! `engine/tools/gungraun/benches/b1_list.rs`, on the same reality.
//! Invariants: the reality is T's vector `carried/big/b1_sum` (`engine/tests/m1/carried/big/
//! list_100000.tally.gen`, its keys expanded as the runner expands `@keys k 1 100000@`); the
//! library is reached only through `tally::run`, the reality given as bytes; after timing, the
//! ask's printed lines are checked against T's stored answer, so a fast wrong answer fails.
//!
//! The row exists because no counter sees a lookup's steps (G4's review, G4-b; pick P11): a
//! sum over 100 000 keys, each weighted by one row, read through the index's lookup once per
//! key. A lookup that scans instead of halving takes about 5 × 10⁹ key comparisons here, where
//! halving takes about 1.7 × 10⁶.

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use tally::{Answer, Asks, Days, Given, NotImplemented, Pin, Request, run};

/// The index's keys, k1 to k100000.
const KEYS: u32 = 100_000;

/// T's reality, its keys written out.
fn reality() -> String {
    let keys: Vec<String> = (1..=KEYS).map(|i| format!("k{i}")).collect();
    format!(
        "model big_list\n  tally 2\n  on 2026-12-31\n  start 2026-01-01\n  end 2026-12-31\n  \
         note \"T's carried item 6 (issue #108, invented): #97's B1, a sum over a list index of \
         100 000 keys, k1 to k100000, each weighted by one row; the list lookup's count budget is \
         a rules-step row\"\n\n\
         source card \"T's card (invented)\"\n\n\
         index big = [{}]\n\n\
         input w[k in big]: num\n\n\
         ask b1_sum: sum(w[k] for k in big)\n\n\
         version card_2026\n  cite card\n  governs always\n  known 2026-01-01\n  \
         w = 1 \"the card: every key weighs 1\"\n",
        keys.join(", ")
    )
}

/// The run's one file (built untimed).
fn given(text: &str) -> Vec<Given> {
    vec![Given {
        path: "list_100000.tally".to_owned(),
        bytes: Some(text.as_bytes().to_vec()),
    }]
}

/// T's stored answer (`m1/vectors.refcore`, `carried/big/b1_sum`).
const KNOWN: [&str; 2] = ["b1_sum: 100 000", "  cut 2026-12-31"];

/// Whether the answer printed is T's.
fn right(answer: &Result<Answer, NotImplemented>) -> bool {
    answer.as_ref().is_ok_and(|a| a.lines() == KNOWN.as_slice())
}

fn b1(bench: &mut Criterion) {
    let text = reality();
    let asked = Request {
        asks: Asks::Named(vec!["b1_sum".to_owned()]),
        pin: Pin::Canonical,
        days: Days::Events,
    };
    bench.bench_function("B1", |t| {
        t.iter_batched(
            || given(&text),
            |files| run(black_box(files), black_box(&asked)),
            BatchSize::SmallInput,
        );
    });
    assert!(
        right(&run(given(&text), &asked)),
        "B1: a fast wrong answer fails"
    );
}

criterion_group! {
    name = benches;
    // a run takes a sizeable fraction of a second, so ten samples, criterion's least
    config = Criterion::default().sample_size(10);
    targets = b1
}
criterion_main!(benches);
