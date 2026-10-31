# R4 progress (scratch)
21:42 start. Read common, brief, shape, PERF §8/§10, BUILD-ORDER, DONE, load.tsv, benchmarks.md, TESTING §1, run.py doc, NOTE §3, RULEBOOK g6, BUILDER §9, scaffold Cargo.toml/benches/engine.yml/SCAFFOLD §2 §5.
Facts: machine M2 Pro, 6P+4E, 32 GB, macOS 26.6.2 (sw_vers), CLT only (no Xcode -> no xctrace), dtrace present; no hyperfine/samply/valgrind.
Scaffold: release panic=abort, overflow-checks=true, lto=fat, cgu=1, debug=line-tables-only; bench profile overflow-checks=true (inherits release); bench job ubuntu-latest nightly, uploads target/criterion 90 d; self-hosted macOS runner exists (fuzz).
Versions 2026-10-30: criterion 0.8.2 (2026-02-04); divan 0.1.21 (2025-04-10); iai-callgrind 0.16.1 (2025-07-30) -> gungraun 0.20.0 (2026-09-26); cargo-pgo 0.3.0 (2026-01-25); dhat 0.3.3 (2024-02-04); samply 0.13.1 (2025-02-01).
gungraun: hard limits absolute (exit 3), soft %, --callgrind-limits/--dhat-limits env GUNGRAUN_*; valgrind>=3.20; runner version must match; no Windows; macOS not supported (valgrind no arm64 macOS).
criterion: noise 1%, alpha .05, 100 samples, 3s warm,5s meas; FAQ: don't run in CI; no fail on regression.
ruff: codspeed simulation+memory+walltime, not in required-checks; profiling profile lto=false. uv: release cgu=1 fat lto abort; profiling lto=false.
overflow: danluu few %; rustc PR87784 stage1 tests 23m19->23m57 (~2.7%), worst 23.8% instr ctfe-stress.
PGO: oxc ~10% linter, 0-20% parser; perf book PGO 10%+, LTO 10-20%; Kobzol: BOLT 2-5% cycles rustc LLVM.
dhat-rs: testing mode HeapStats asserts; max heap; experimental.
22:05 research done (~60 calls). Extra facts: Mac14,9 MBP14 M2 Pro on battery at check; repo PRIVATE -> ubuntu-latest 2 vCPU 8 GB; macos-latest 3 M1 7 GB. /usr/bin/time -l prints instructions retired + cycles + peak footprint (ri_instructions in sys/resource.h). valgrind 3.28.0 2026-10-23 no arm64 darwin. hyperfine 2.0.0 2026-10-07 (peak mem, JSON v2). flamegraph 0.6.14 macOS needs xctrace (Xcode absent). samply 0.13.1 last release 2025-02. BOLT ELF only. SQLite cachegrind 7 digits. CodSpeed CV 2.66% GH. check.py CLEARED lacks GUNGRAUN_/CRITERION_. cargo: overrides can't set panic/lto; bench ignores panic.
Next: write R4.md in 2 pieces.
22:00 R4.md written (≈38 KB), D4-1..D4-8; final message next
