# Milestone S, the tracer slice: the gate's report

Issue #26, branch `build/s` at **51ed737**, 2026-11-02. Written by the S gate session (R-129, BUILDER §13.3). The review's rows are in the worktree's `tmp/s/gate-review.md`, and the raw timing output is in `tmp/s/timing/`. **Not committed:** the coordinator decides where it lands.

## MILESTONE-S's six rows

| # | Row | State | Evidence |
|---|---|---|---|
| 1 | X's tracer, C1–C3 against refcore's stored output | **green, 3/3** | `run.py --interp=engine 'I:X:*@b_r1' 'I:X:*@b_r1s'` at 51ed737: 3 checks, 3 same, 0 cached, exit 0 |
| 2 | `R:all`, refcore's own tests | **green, unchanged** | `git diff origin/master..HEAD -- engine/tools/refcore engine/tools/refcore_fast` is empty, so master's green run stands |
| 3 | the kernel's seven properties (`s_properties`) | **green, 7/7, gating** | S has been in `gating` since 060451c (the user's row-5 choice). The builder's `check full` at 51ed737 is green (31 steps, 1 173 s) with `s_properties` as a failing step |
| 4 | the 75 household vectors (`s_household`) | **green, 3/3, gating** | the same `check full` |
| 5 | mutation ≥ 90 % on the kernel | **open, blocks the merge** | CI run 41467061452 at 060451c: about **44 %** on shards 0, 2 and 3 (477 caught, 608 missed, 245 unviable); shard 1 was cut off by a runner shutdown. The user, 2026-11-02: "Hold merge; T fills gap". A tests-role session writes kernel tests from the survivor list, then the mutants run again. Run 41492324014 at f76ca07 is the coordinator's to read |
| 6 | the budget rows S measures | **measured below**; P8-14 meets its proposed budgets; **P8-M4 misses** its proposed budget | `tools/perf row` on the reference Mac, un-niced (the user's ruling for this run only) |

## The timing run (row 6)

**Machine:** `fingerprint: Mac14,9; macOS 26.6.2 25G83; rustc 1.99.1 (58cf8f9c3 2026-10-13); power: AC Power; lowpowermode 0`. The run took 19:21–19:25. The user was away and no build, test or session ran. The load average was 4.3–8.5, from system daemons (`BTLEServer` near 98 % of one core, `storagekitd`). The runs are single-threaded on a 10-core Mac. Release binary sha256 `3e876de0…`. Each P8-14 series is 2 warm-ups then the median of 10. The micro rows are criterion medians.

**The budgets, fixed by the user on 2026-11-02 (D4-6): "Fix as proposed (Recommended)"**: P8-14 ≤ 1 s, ≤ 512 MB and def misses ≤ 30 400; P8-M1 ≤ 55.5 ns, P8-M2 ≤ 2.20 µs, P8-M3 ≤ 7.81 µs. They go into PERFORMANCE §8 (a pull request in the spec role) and `tools/perf-budgets.toml` (a rules step), outside `build/s`.

| Row | What | Measured | Proposed budget (D4-6; the user fixes them) |
|---|---|---|---|
| **P8-14** | `I:X:replay_september@b_r1` at 1× (refcore 52.6 s, 4.95 GB, ≥ 2.27 M misses) | wall **760.9 ms** (three series: 760.9, 763.4, 760.1); peak footprint **154.4 MB**; 11.86 G instructions; def misses **25 312**; 926 days stepped (906 non-empty); 21 walks | **≤ 1 s and ≤ 512 MB**, as proposed: met with a margin of 1.31× and 3.3×. Count budget: **def misses ≤ 30 400** (25 312 × 1.2) in place of 2 270 000, since counts are exact |
| **P8-M1** | `Num` add and multiply, small tier | **46.26 ns** [46.20, 46.29] | **≤ 55.5 ns** (× 1.2) |
| **P8-M2** | gcd, 5 049-bit by 64-bit | **1.83 µs** (two runs: 1.830, 1.826) | **≤ 2.20 µs** (× 1.2) |
| **P8-M3** | one 5 049-bit multiply and its reduction | **6.51 µs** first (later runs 6.70 and 6.63: the series drift by 3 %) | **≤ 7.81 µs** (× 1.2 of the first) |
| **P8-M4** | one read of a written projection, E §4.3's common case (P8-5's micro form), on a 33-year place with 12 054 movements | the read is **72.9 ms**: the whole run with `bal(x, E) from S` is 387.6 ms, and the same run with `bal(x, E)` is 314.7 ms | **≤ 0.3 µs** (P8-5, fixed by the spec, not × 1.2): **missed by about 2.4 × 10⁵**. The read is the plain walk of the window, since fence F5 (P §2.3's prefix sums F) was closed under R-130. Under R-130 this row now asks for F5. **The user, 2026-11-02: "Defer to M1 (Recommended)":** 72.9 ms is S's first figure, and P8-M4's budget moves to M1, which builds `low`, P8-5 and F5 together. Its instruction limit (× 1.2) is gungraun's, on Linux CI |

**An instrument gap (the lead's, R-91):** `tools/perf row P8-M4` compares the median of the whole `P8-M4` run (387.6 ms) with the read's budget. The bench's header says the read is the difference `P8-M4` − `P8-M4-plain`. So `perf`'s BUDGET line overstates the miss by the run's fixed cost, but it is a miss either way.

**Overflow checks' cost** (D4-4, U-16; BUILDER §13.4), measured once:
- **The whole binary, on P8-14:** checks on gave 760.9, 763.4 and 760.1 ms; checks off (`--config profile.release.overflow-checks=false`) gave 761.7 and 761.3 ms. Instructions: on 11.855–11.865 G, off 11.860 G. **No measurable cost (+0.0 %, inside ±0.3 % noise).** The engine's own arithmetic is checked explicitly (R-8), so the compiler's checks add almost nothing.
- **Inside the big tier's library** (`--config profile.bench.package.num-bigint.overflow-checks=false`), measured by criterion's run-to-run comparison:
  - P8-M2: off against on +0.05 % (p = 0.71); on against off +0.20 % (p = 0.10);
  - P8-M3: off against on +1.09 % twice; on against off −2.57 %.
  
  **No measurable cost:** the differences go both ways and are the size of the noise. That is **below D4-4's 3 %**, so no override is needed and the release profile stays as frozen.

## The unknowns PERFORMANCE §10 gives S

| Unknown | Figure | By |
|---|---|---|
| U-1, the cost per node | **0.86 µs a node, all in** (760.9 ms over 883 039 nodes, walks, ledger and allocation included). `shape.md` assumed 10–50 ns for the node alone | gate (counts: the counted build at 51ed737) |
| U-3, read-set overhead (cold) | **1.06× wall, 1.03× instructions, 1.22× footprint** (126 → 154 MB); 1.0.1 cited 1.27× | builder, G12 |
| U-4, bytes per cell | about **280 B inline** per kept cell plus its heap; about **1.1 KB more** with its read set (the read cells' keys on the heap). Dense columns aren't built (closed, R-130) | builder, G11c, G12 |
| U-5, cells left | **25 312 misses** with law-date classes (about 2.51 M at exact law dates; refcore ≥ 2.27 M, partial); 926 days stepped in 21 walks | builder, G11b; gate's count unchanged |
| U-6, checkpoint interval | monthly costs **+0.5–0.8 MB** of footprint on P8-14, with instructions inside the noise; weekly and quarterly are alike. The benefit waits for forks and the walk restart (SQ-5). **Monthly kept** | builder, G11c |
| U-11 | `Num` is **16 bytes**; Lehmer's share is E1's | builder, G1, G11c |
| U-12 | proxy: no `num::` function in P8-14's top 120 (samply); a counter inside the kernel's pure functions isn't built | builder, G11a |
| U-15, stack | dev overflows at 448 KiB and passes at 512 KiB; `release-test` overflows at 128 KiB and passes at 256 KiB. Counted depth 32, 0 cells unwound on P8-14 | builder, G11c |
| U-16, overflow checks | no measurable cost (above) | gate |
| U-17, instruction stability | on the Mac, P8-14's instructions stay within **0.08 %** (11.855–11.865 G over five series). CI's stability and gungraun's slowdown are CI's bench job, the coordinator's to read | gate, part |
| U-18, thermals | five 10-run medians of P8-14 within **0.4 %** (760.1–763.4 ms) over two minutes; P8-M3 drifted 2.9 % between series | gate |

## What S added (R-129)

- **Modules:** the whole library, which didn't exist before S. There are 78 source files, about 40 000 lines with inline tests and docs: the kernel (`num`, `read`, `walk`, `book`, `hole`) is 26 files and about 13 900 lines. The builder's tests are 3 500 lines in `crates/*/tests/`, plus four benches. Each file's purpose line is in `engine/ARCHITECTURE.md`'s codemap.
- **Spec coverage:** none before S; now **70 of 257 sections have code** (`tools/trace --map`): 1 813 citations, 1 473 functions, 94 files (26 in the kernel), and every citation is in the spec. The 187 sections without code are later milestones' (L §6 and E §7–§8, scenarios, paths, the evaluation layer; E §10 `explain`, M1) or the G-S2 deferrals (P §3.2 layers).
- **Exceptions:** **+2**: kernel `num/big.rs` `struct Big` (R-29, the one big-integer wrapper) and CLI `main.rs` `read_reality` (R-39). Count 6, budgets kernel 1/20, cli 3/3, library 0/0, lexer-parser 0/0. Plus 5 report-only hits (`base/table.rs` type parameter, `syntax/parse.rs` lifetimes).
- **Size, nesting, clones** (`tools/advisory`):
  - nesting is 5 at most (a lint error);
  - five functions are over 100 lines (`walk/day.rs:405` 119, `book/reading.rs:239` 117, `load/places.rs:38` 117, `walk/order.rs:308` 107, `eval/read.rs:735` 103);
  - the most clones are in `book/reading.rs` (8.2 per 100 lines), `load/places.rs` 7.4, `book/adjust.rs` 7.3 and `book/members.rs` 6.2;
  - five files are over 1 000 code lines (`syntax/item.rs`, `syntax/expr.rs`, `load/graph.rs`, `walk/plan.rs`, `walk/reads.rs`), ten with tests counted (review rows O5 and T5).
  
  Two kernel functions the advisory reports as uncited (`read/gap.rs:168`, `read/rows.rs:478`) do cite their section across a line break. `trace`, the binding check, passes them.
- **Benchmarks:** first figures, above. Against refcore on P8-14: **69× faster** (52.6 s → 0.76 s), **32× less memory** (4.95 GB → 154 MB), **90× fewer cells** (≥ 2.27 M → 25 312).
- **Fences** (R-130):
  - F1, the event walk (`walk/agenda.rs`) and its checkpoints (`walk/state.rs`), for P8-14;
  - F2, the memo's law-date classes (`eval/memo.rs`), for P8-14;
  - F4, the number tiers (`num/small.rs` for P8-M1, `num/gcd.rs` for P8-M2).
  
  Each has a plain twin and an equivalence test. Closed: F2's dense columns and F3, since no row asks. **F5 is now asked for by P8-M4.**

## The review

A fresh `engine-reviewer` made three passes, and the gate checked each blocking row against the code and refcore.
- **First pass** (`origin/master..79b7f52`): 4 blocking and 7 other.
- **Second pass** (G12): 3 new blocking.
- **Third pass** (G13, G13b): 1 new blocking, T1, closed by G13c (51ed737).

**Every blocking row is closed.** The open rows:
- **Issues:**
  - #33, the ledger's tie order (refcore's bytes kept; the spec wins after S);
  - #34, a refused re-measurement is skipped silently (spec);
  - #35, B1's T gap;
  - #36, read sets and walk and plan reads (M1);
  - #37, T's gaps for N1–N3, N6, N7 and T1;
  - #38, place flows in a projection's walk (spec against refcore);
  - #39, the binding order of movements (spec).
- **Not issues:** O5 and T5, ten files over 1 000 lines (a note); G-S2's deferrals to M1 (the interner, `Table<I, T>`, the layer slots).

**`ARCHITECTURE.md`** matches the code at f76ca07. **T2**, line 130's list of what a read set holds leaving out "a def's givens at its keys" (G13's `Read::Givens`), is fixed by the lead in the commit that adds this report.
