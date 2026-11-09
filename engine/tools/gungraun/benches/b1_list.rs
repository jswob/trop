//! B1, a list index's lookup at 100 000 keys (#97's G7, P §3.4, D2-5): its instruction count
//! on Linux CI (D4-2, BUILDER §13.3).
//! Spec: P §3.4 (an index's keys found by halving), D2-5; L §5.11 (`sum … for k in`).
//! Twin: none, an instrument (R-91); its wall time is `engine/crates/tally/benches/b1_list.rs`,
//! on the same reality.
//! Invariants: the hard limit comes from `engine/tools/perf limits b1_list`, never from this
//! file; the file and the request are built before the count starts, and the teardown,
//! outside the count, checks the printed lines against T's stored answer, so a fast wrong
//! answer fails the run.
//!
//! No counter sees a lookup's steps (G4's review, G4-b; pick P11), so this count is the row's
//! guard on a pull request: a lookup that scans instead of halving takes about 5 × 10⁹ key
//! comparisons here, where halving takes about 1.7 × 10⁶.

use std::hint::black_box;

use gungraun::prelude::*;
use tally::{Answer, Asks, Days, Given, NotImplemented, Pin, Request, run};

/// The index's keys, k1 to k100000.
const KEYS: u32 = 100_000;

/// T's reality (`engine/tests/m1/carried/big/list_100000.tally.gen`), its keys written out.
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

/// The run's one file and the request for its ask.
fn setup() -> (Vec<Given>, Request) {
    let files = vec![Given {
        path: "list_100000.tally".to_owned(),
        bytes: Some(reality().into_bytes()),
    }];
    let asked = Request {
        asks: Asks::Named(vec!["b1_sum".to_owned()]),
        pin: Pin::Canonical,
        days: Days::Events,
    };
    (files, asked)
}

/// T's stored answer (`m1/vectors.refcore`, `carried/big/b1_sum`).
const KNOWN: [&str; 2] = ["b1_sum: 100 000", "  cut 2026-12-31"];

fn check(answer: Result<Answer, NotImplemented>) {
    assert!(
        answer.as_ref().is_ok_and(|a| a.lines() == KNOWN.as_slice()),
        "B1: a fast wrong answer fails"
    );
}

#[library_benchmark(teardown = check)]
#[bench::sum(setup())]
fn b1_list(input: (Vec<Given>, Request)) -> Result<Answer, NotImplemented> {
    let (files, asked) = black_box(input);
    run(files, &asked)
}

library_benchmark_group!(name = micro, benchmarks = b1_list);

main!(library_benchmark_groups = micro);
