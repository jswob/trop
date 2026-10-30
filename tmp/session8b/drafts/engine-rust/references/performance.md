# Performance: measure, fence, guard, profile

Performance is core, and it comes from the structure (PERFORMANCE §1–§5: the event walk, checkpoints, forks, a trimmed memo key, dense columns), not from clever code. refcore steps every calendar day though only 2–7 % have work, and redoes the same long walk up to 64 times. This file says how speed is kept readable, measured, guarded and profiled (BUILDER §13; D4-1 to D4-8, D5-6).

## Contents
1. The method: measure first, name the hot paths
2. Fences
3. Counters and `--stats`
4. The budgets and what blocks
5. Profiles
6. Profiling recipes

## 1. The method: measure first, name the hot paths

- **Count before you time.** Most slowness here is more work (cells recomputed, days stepped, walks repeated), not slower work. The counters show it exactly and the same way every run.
- **The hot paths are already named** (refcore's counts, `tmp/session8/shape.md`): the day walk, repeated walks (`windows each`, members, projections), the memo's misses (a few day-keyed defs carry 85–95 %), the read index (reads repeat 2–47× per key), the number tiers. Look there first.
- **Optimise only against a budget row** (PERFORMANCE §8, its P8 id), never "while I'm here".
- **A fast wrong answer fails:** every benchmark checks its output hash against T's digest after timing, and every optimisation leaves gates 1–7's outputs byte-identical (gate 12).

## 2. Fences (D5-6, R-130)

- Plain code first. A fence module is opened only when a budget row asks for it, by its P8 id.
- A fence has: a `PERF:` line in its `//!` header (what it speeds up, the P8 row that asked, the name of its plain twin); a benchmark (written by the lead on request, R-91); a **plain twin** (the simple version, kept beside it); an **equivalence test** (twin and fence agree on generated input, proptest with a fixed seed, and the whole run still prints refcore's bytes).
- A twin is kept where it is cheap; refcore is the twin of last resort.
- Expected fences: the walk (skipping empty days, checkpoints), the memo, the read index, the number tiers, the threads. Closures for day-dense asks would be one more, only if a measured ask misses.
- `#[inline(always)]` and `#[cold]` (hints that steer the compiler's inlining) belong inside fences; outside one they are listed for review.

**Bad / good**
```rust
// Bad: an ad-hoc cache inside the evaluator, no twin, no budget row
static mut LAST: Option<(ItemId, Num)> = None;
// Good: a fence module whose header says why it exists
//! The read index: rows by (input, keys, cut), so a repeated read is one lookup.
//! Spec: L §4.6 (the read rule).
//! Twin: engine/tools/refcore/evaluate.py (the read rule, from l.816).
//! Invariants: the same rows, in the same order, as the plain read.
//! PERF: P8-10, P8-12 (reads repeat 25–43× per key); plain twin: read_plain.rs.
```

## 3. Counters and `--stats` (D4-8, K8-15)

- A cargo feature `counters` (a compile-time switch, like a build flag that strips code from a bundle), **off in release**. When on, `--stats` prints `shape.md`'s units on stderr: expression nodes evaluated, def cells (hits and misses), reads and distinct reads, walk days and non-empty days, legs booked, parts groups, `Num` results by bit size, walks shared, and the cells a worker recomputed locally.
- **Outputs stay byte-identical** with the feature on and off. Counts are exact at 1 worker; 6-worker rows check only the result hash.
- `--stats` is a build-side instrument, never part of the shipped CLI (the spec defines no stats output).

## 4. The budgets and what blocks (D4-1, D4-2, D4-3, D4-7)

| Instrument | Measures | Where | Blocks |
|---|---|---|---|
| the engine's counts, checked by T's runner | `shape.md`'s units against each vector's count budget | Linux CI, every PR, 1 worker | yes |
| gungraun 0.20.0 under Valgrind 3.28.0 | instructions, cache, allocations, with hard limits from the protected budget table, passed by `engine/tools/perf` | Linux CI, every PR (Valgrind has no Apple Silicon port) | yes; drift is a report |
| `engine/tools/perf` (the release binary under `/usr/bin/time -l`) | wall time, peak memory footprint, instructions retired | the reference Mac: nightly report, the milestone tag | at the tag |
| criterion 0.8.2 | wall time of the micro rows (P8-M1..M4) | the reference Mac | no (reports) |

- **Wall time on hosted CI is too noisy to gate** (about 45 % false alarms at a 2 % threshold, like a flaky Jest timing test). Only counts block a PR; wall time blocks only the milestone tag, on one fixed machine.
- **The reference machine:** the user's MacBook Pro M2 Pro (32 GB, macOS 26), on AC power, idle, never during a fuzz run; 2 warm-ups, then the median of 10; a fingerprint in every report.
- **Memory:** 8 GB peak footprint is a hard ceiling on any run; a load row ≤ 4 GB; a 1× check ≤ 512 MB.
- **Every number is proposed until S**, then fixed with the user. You never edit a budget, a bench's limits or the budget table; the guard refuses. A miss is one line (`BUDGET …` or `COUNT …`), red in `check`; if a correct design can't meet it, the exit is a STOP-REPORT ("a budget looks wrong").

## 5. Profiles (D4-4)

- **release** stays the scaffold's: `panic = "abort"`, `overflow-checks = true`, `lto = "fat"`, `codegen-units = 1`, `debug = "line-tables-only"`. **bench** inherits it. Gates always measure `release`.
- Overflow checks are measured once at S, by the lead, on the six checks; above 3 % inside num-bigint, the user decides an override for that crate only.
- **profiling**: release plus full debug info, for your profiling only.
- **PGO** only at M9, release-only; every budget is met without it. No BOLT, no `target-cpu=native`. Allocators are an M9 measurement.
- The profiles live in protected files (R-80).

## 6. Profiling recipes (D4-5)

1. **Is it slower?** `engine/tools/perf row <id>`: one line per budget, with the counts.
2. **Why more work?** The `counters` build with `--stats`; diff the counts against the row's count budget and refcore's counted run.
3. **Where does the time go?** `engine/tools/perf profile <id>`:
   - on the Mac, samply 0.13.1: `samply record --save-only -o prof.json.gz -- <bin> <args>`, then the lead's summariser, which prints the top 30 functions by self and inclusive time as text;
   - on Linux, `perf record -e cpu-clock -g -- <bin>` then `perf report --stdio --no-children`, or gungraun's callgrind output through `callgrind_annotate`.

None needs sudo, a GUI or Xcode. Not used: cargo-flamegraph (needs Xcode's `xctrace` on macOS), Instruments. samply on macOS 26 is an unknown until S.
