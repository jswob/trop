<!-- saved by tools/save_final.py from agent agent-a836237822c7dac0b.jsonl (R1) -->

R1 is finished. The report is at `/Users/jswob/projects/trop/tmp/research_notes/engine-prep/R1.md`, 33 010 bytes, a little over the ~30 KB target. My scratch progress log is at `/Users/jswob/projects/trop/tmp/research_notes/engine-prep/scratch/R1/progress.md`. I wrote nothing else, ran nothing, and never touched `tmp/holdout/`.

**The three findings that matter most:**
1. **The time goes into repeated and empty work, not into how fast each node runs.** refcore steps every calendar day (only 5–7 % have an act) and redoes the same walk 64× in A, 8–12× in E2 and 20× in X. Three day-keyed defs hold 95 % of X's misses. The big saving comes from an event-driven walk, checkpoints that let timelines share their common start, and a memo context trimmed to what each item reads. Choosing between tree-walk, closures and bytecode is worth about 1.5× at most (Mitchell 2020: 2.1 s / 1.4 s / 1.5 s; Larose et al., OOPSLA 2023, found AST and bytecode on par).
2. **salsa breaks the approved rulebook.** salsa 0.28.5 (2026-09-24) has `hashbrown` and `rustc-hash` as normal dependencies, and R-37 bans both by name. It would also need about 15 crates allow-listed under R-52, it brings locking outside R-42's one thread module, and it runs queries on the native call stack. Its ideas carry over: verification by read set, backdating, durability (rust-analyzer measured about 300 ms of verification saved per edit).
3. **Memory is not the constraint if layers are released by scope.** I estimate 60–300 B per engine cell against refcore's ~2.2 KB (inference, no measurement), so X's 2.27 M cells come to 0.14–0.68 GB. Releasing a path, a `solve` step's pins or a projection when its scope ends is deterministic. LRU is not, and salsa's own maintainers record its churn.

**Decisions:**
- D1-1: tree-walk over a resolved tree, with recursion across cells up to a counted depth and an explicit stack plus replay past it. Closures come later, only if E1-shaped asks (work on every day) miss budget.
- D1-2: the engine's own memo, not salsa.
- D1-3: memo key = (item, key values, interned trimmed context); scenarios, pins, axes, members, paths and projections as layers; day-keyed recurrences as dense columns; a separate read memo.
- D1-4: event-driven walk plus monthly checkpoints first; forking timelines at their first divergence day comes second, once the walk is byte-identical to refcore.
- D1-5: build the per-run memo with read sets now; incrementality across amendments stays a seam, built only if a measured turn misses 1–2 s.
- D1-6: no eviction; release by scope.
- D1-7: seam J first as its own member context with a lazy walk up to the last day read; place-cone pruning second.

**What you need to know:**
- **Contradiction with phase 1:** D1-1 departs from the wording of BUILDER §4 ("explicit work stack, not Rust recursion"). It keeps R-16 (depth is counted, never measured), and the pure work stack stays the fallback. The user decides.
- **Two spec observations, not fixed:** no section lists what makes a day non-empty, so an event-driven walk has to derive that list. And ENGINE §3.2 doesn't say whether the projection component is trimmed from cells that never reach the projection's walk, which matters for X.
- **Sources I couldn't read directly:** the rust-analyzer release-note figure (648 MB) came only through search results. The OOPSLA paper and the Time Warp checkpoint papers came through abstracts and secondary sources. DDlog's repository banner showed an archive date of 2026-07-13; phase 0 had already found it archived.
- **Versions checked 2026-10-30:** salsa 0.28.5, differential-dataflow 0.25.1, ascent 0.8.1, ironcalc_base 0.9.1, cedar-policy-core 4.14.0.
