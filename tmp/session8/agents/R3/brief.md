# R3: parallelism without losing determinism

Read `tmp/session8/agents/common.md` first; it holds the session brief's §1–§2, the agent rules, and the output format. You write `tmp/research_notes/engine-prep/R3.md`.

## Your question

How the engine uses up to 6 cores and still prints the same bytes, with equal result hashes on 1 and 6 workers (DONE gate 6). Speed is the user's core concern; determinism is the spec's.

1. **Where Tally's work splits.** Monte Carlo paths (ENGINE §7), axis combinations (up to 64 per ask, `LOAD-FORKS`), scenarios, joint members as their own contexts (BUILD-NOTES seam J), independent asks, `windows each`, rolling origins (ENGINE §8). For each: is it independent by the spec, what it shares (the shared layer of E §7 step 1, law cells, the reality), and what merging it needs. Use shape.md: refcore redoes a long walk 64× in A, 8–12× in E2, 20× in X; LH-17-mc-light runs 256 paths at about 0.2 s a path in Python.
2. **The tools.** `std::thread::scope` (phase 1's pick, RULEBOOK R-42: threads only in one sanctioned module, worker count passed in, merged in path order, no rayon, no CPU-count read) against rayon (work stealing; which of its operations keep order, such as `collect` on an indexed iterator, and which don't, such as `reduce` and `fold`), crossbeam, and any 2026 alternative. What each costs to read and to audit. Whether R-42 should change, with the evidence.
3. **Deterministic merges.** Fixed chunking by path index, fixed-shape tree reductions, merging in path order. Where exact arithmetic makes order irrelevant and where it doesn't:
   - rational sums are associative and commutative, but `RUN-DIGITS` is checked on partial sums (NOTE SO-22, LANGUAGE l.1531), so a different grouping can refuse where another prints;
   - two workers that each refuse or each meet a hole: which refusal and which holes print, in what order (SO-4);
   - path statistics: the spec has no floats anywhere (NOTE §1 item 2); a standard error needs a square root, a transcendental (SO-23); quantiles.
4. **Per-path draws.** Tally's draws are addressed by SHA-256 (LANGUAGE §5.14 draws, §7.4). Compare with counter-based generators (Random123's Philox, JAX's splittable keys, NumPy's `SeedSequence.spawn`, R's L'Ecuyer streams): what the address scheme already gives, and what to check.
5. **Memory per worker.** The shared layer read by every path against each path's own layer; how Rust shares read-only data across threads (`Arc`, `Sync`, scoped borrows) and what it costs; the 8 GB cap with 6 workers at the benchmark sizes (D-10 at 10 000 paths, B-12 at 3 000, household-bands at 1 000; `engine/tests/load/benchmarks.md`).
6. **What the strictest real projects do.** For example: ruff and uv (rayon over files, then a sorted merge), rust-analyzer and salsa's parallel queries, typst, cargo's own job ordering, TigerBeetle and FoundationDB (deterministic by single-threaded design and simulation). Only what bears on the decisions.
7. **Testing gate 6.** How to prove equal hashes on 1 and 6 workers and across runs: a property test, a forced schedule (loom or shuttle for the merge module), a chunk-size sweep. What can be checked by a tool and what only by review.

## Read (by range; don't read whole)

- `tmp/session8/shape.md` (whole, one page).
- `engine/spec/ENGINE.md` §7 (lines 355–387), §8 (388–418), §9 (419–435); `LANGUAGE.md` §5.14 draws as needed (1318–1455), §7.4–§7.5 (2166–2197); `PERFORMANCE.md` §3 (110–139), §5 (211–238), §8 (312–332).
- `engine/spec/DONE.md` gates 6 and 7; `engine/BUILD-NOTES.md` J and R.
- `tmp/research_notes/engine-rust-strictness/RULEBOOK.md` rows R-41 and R-42 (group 3, lines 85–107); `NOTE.md` §2 K-14, K-15, K-25 and §4 SO-4, SO-22–SO-24; `BUILDER.md` §7.
- `engine/tests/load/benchmarks.md`.

## Decisions to frame (merge or split as the evidence shows)

The threading tool (keep or change R-42); what runs in parallel in the first build and what later; the merge rule; the order of refusals and holes across workers; memory per worker; how gate 6 is tested.
