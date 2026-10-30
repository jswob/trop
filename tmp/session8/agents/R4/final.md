<!-- saved by tools/save_final.py from agent agent-a3ad5bbc805b88196.jsonl (R4) -->

R4 is done. The file is `/Users/jswob/projects/trop/tmp/research_notes/engine-prep/R4.md`, 38 KB, which is over the ~30 KB target; the sources list takes about 4 KB of that. Scratch notes are in `tmp/research_notes/engine-prep/scratch/R4/progress.md`. Nothing was installed, built or benchmarked. The one thing I ran was a form check: `/usr/bin/time -l /bin/ls` three times, to see what the tool prints.

**The three findings that matter most:**
1. **Only counts should block a merge.** The engine's own work counts (`shape.md`'s units) are exact, and Valgrind instruction counts via gungraun repeat to many digits. Gungraun (the renamed iai-callgrind, 0.20.0 of 2026-09-26) has absolute hard limits that exit with code 3. Wall time is too noisy for a PR gate: hosted runners vary by 2.66 %, which means 45 % false alarms at a 2 % gate (CodSpeed, 2025). Wall time and memory should therefore judge the milestone tag only, on the reference Mac. rustc, rust-analyzer and ruff all only report performance.
2. **Valgrind has no Apple Silicon port** (3.28.0, 2026-10-23), so instruction-count gates must live on Linux CI. On the Mac, `/usr/bin/time -l` on macOS 26 prints "instructions retired", "cycles elapsed" and "peak memory footprint" with no sudo. That gives a cheap instruction-count proxy and the watchdog's own memory measure. `cargo flamegraph` 0.6.14 now needs `xctrace`, which needs full Xcode, and only the Command Line Tools are installed. samply needs neither, but it hasn't released since 2025-02 and its output needs a small text summariser.
3. **The scaffold's release profile is sound.** It matches uv's: fat LTO, `codegen-units = 1`, `panic = "abort"`. Turning on `overflow-checks` cost rustc's test suite +2.7 %, and Dan Luu estimates "a few percent". For Tally it should cost less, because R-19 already forces hand-written `checked_*` arithmetic; the open risk is num-bigint's inner loops. PGO gains about 10 % (oxc). BOLT works on ELF binaries only, so not on the Mac.

**Decisions, with my pick:**
- **D4-1 Instruments:** three of them. Engine counts through T's runner, gungraun on Linux, and wall time and footprint on the Mac through a lead-written `engine/tools/perf`, with criterion kept for micro rows.
- **D4-2 What blocks:** count budgets and gungraun hard limits on every PR. Wall time and footprint block only the milestone tag, run by the gate session.
- **D4-3 Reference machine:** the user's MacBook Pro M2 Pro (Mac14,9: 6 performance + 4 efficiency cores, 32 GB, macOS 26). It runs on AC power, idle and never during fuzzing, takes the median of 10 runs after 2 warm-ups, and records a machine fingerprint. This settles SO-27.
- **D4-4 Profile:** keep the scaffold's. Measure the overflow-check cost once at S; if num-bigint pays more than 3 %, the user decides on an override for that package only. Add a `profiling` profile for the builder. PGO only at M9, and no budget may depend on it.
- **D4-5 Profiling:** samply plus a summariser on the Mac; `perf report --stdio` and `callgrind_annotate` on Linux.
- **D4-6 PERFORMANCE §8 for 2.1** in four groups:
  - the old interactive targets, unchanged;
  - the six `shape.md` checks at ≤ 1 s and ≤ 512 MB each, with def-cell misses no higher than refcore's;
  - the load rows L:1–L:18 at ≤ 4 GB each: non-Monte-Carlo rows ≤ 10 s, Monte Carlo rows at N-2003's rate;
  - micro rows ratcheted from S.
  - Every number is "proposed" until G1.
- **D4-7 Memory:** an 8 GB ceiling on any run, ≤ 4 GB per load row (6 jobs fit the 32 GB machine), ≤ 512 MB per 1× check. Peak footprint on the Mac, max RSS on Linux.
- **D4-8 Engine counts:** a cargo feature `counters`, off in release like R-51's fault injection, printing `shape.md`'s units behind `--stats`.

**What the lead must know:**
- **Contradiction with DONE gate 12:** R-97 and the scaffold's `bench` job time on `ubuntu-latest`, but gate 12 judges "on the reference machine". The repo is private, so hosted runners have only 2 vCPU and 8 GB on Linux, and 7 GB on macOS.
- **DONE gate 12, two more points:** it names "criterion reports" as evidence, though most rows are end-to-end CLI runs. It also cites `tmp/design2/benchmarks.md`, which has moved to `engine/tests/load/benchmarks.md`. Both are spec observations.
- **A gap in the scaffold:** `check.py`'s cleared variables don't include `GUNGRAUN_*` or `CRITERION_*`. Gungraun's environment variables override a bench file's limits, so a builder could loosen them.
- **Versions changed since phase 1:** iai-callgrind is now gungraun; hyperfine 2.0.0 came out three weeks ago with a new JSON format and peak memory measurement.
- **Not settled:** the Instruments (`xctrace`) flags are unverified, because Xcode is absent.
