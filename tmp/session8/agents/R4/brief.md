# R4: measuring as a guarded habit

Read `tmp/session8/agents/common.md` first; it holds the session brief's §1–§2, the agent rules, and the output format. You write `tmp/research_notes/engine-prep/R4.md`.

## Your question

How performance is measured from the first milestone, by checks an LLM builder can run and read but can't edit or game. The machine is an Apple Silicon Mac (M-series, 6 performance cores, macOS 26); CI is GitHub Actions on Linux.

1. **Benchmark harnesses.** criterion (phase 1 pinned 0.8.2), divan, the instruction-count tools (iai-callgrind and whatever succeeded or renamed it by 2026; cachegrind), hyperfine for the CLI end to end, and services (CodSpeed, Bencher). Which run on this Mac and which only on Linux (Valgrind on Apple Silicon), and what each prints that an LLM can read.
2. **Noise, and what can block a merge without flaking.** Wall-clock noise on shared CI runners and on a laptop; instruction counts as a stable proxy and where they mislead (memory-bound code, allocation); relative before/after runs on one machine; thresholds and statistics. What rustc-perf, ruff (CodSpeed), rust-analyzer and other strict projects gate on, and what they only report.
3. **Profiling an LLM can run and read.** samply (macOS and Linux; its saved profile), cargo-flamegraph (dtrace on macOS), Instruments through `xctrace` export, `perf` on Linux, dhat-rs for heap and allocation counts, counting allocators. For each: one command, what it outputs as text, and whether it needs sudo or a GUI. Recipes the builder can follow without help.
4. **Release settings.** LTO (thin or fat), `codegen-units = 1`, `opt-level`, `panic = "abort"` (phase 1's K-1), PGO (cargo-pgo) and BOLT, and the cost of `overflow-checks = true` in release (phase 1 left it open: NOTE §3.2). Published measurements with their method. Read the scaffold's profiles (RULEBOOK R-8, R-9) in `/Users/jswob/projects/trop-engine-scaffold/engine/Cargo.toml`.
5. **Budgets the builder can't edit.** BUILD-ORDER says each milestone's exit includes the budget rows it touches, at 1× size. How those rows become checks: where the budget table and the benchmark files live (phase 1's K-17 protects benchmarks; the guard refuses `benches/` to the build role), who writes them, what runs them (`engine/tools/check`, CI), and how a failing budget is reported (a STOP-REPORT, BUILDER §9). Count-based budgets (memo cells, walk days, allocations) as non-flaky proxies beside time.
6. **PERFORMANCE §8 for tally 2.1.** How shape.md's figures and the load rows (`engine/tests/load/load.tsv`, `benchmarks.md`) become §8's first rows: what, at what size, what budget, measured where. Propose a definition of "the reference machine" (DONE gate 12; NOTE SO-27).
7. **Memory as a measured budget:** peak footprint on macOS (`/usr/bin/time -l`), max RSS on Linux, allocation counts; how the 8 GB cap of the user's machine becomes a budget row.

## Read (by range; don't read whole)

- `tmp/session8/shape.md` (whole, one page).
- `engine/spec/PERFORMANCE.md` §8 (lines 312–332), §10 (362–369); `BUILD-ORDER.md` lines 18–39 (summary) and 258–end (M9, what every milestone keeps); `DONE.md` gate 12 and "Milestone gates on the way".
- `engine/tests/load/load.tsv`, `benchmarks.md`; `engine/tests/TESTING.md` §1 (the layers and the budget) and the docstring of `engine/tests/run.py` (lines 1–45).
- `tmp/research_notes/engine-rust-strictness/NOTE.md` §1 item 15, §2 K-17, §3; `RULEBOOK.md` group 6 (lines 149–181); `BUILDER.md` §9.
- The scaffold worktree: `engine/SCAFFOLD.md` §2 (commands) and §5 (the DONE gates and what measures them); `engine/Cargo.toml`; `engine/crates/tally/benches/`; `.github/workflows/engine.yml`.

## Decisions to frame (merge or split as the evidence shows)

The harness; what blocks a merge and where it runs (this Mac, Linux CI); the reference machine; the release profile, `overflow-checks` and PGO; the profiling recipes; how PERFORMANCE §8 is rewritten for 2.1 and its first rows.
