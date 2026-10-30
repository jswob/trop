# R1: the evaluation design

Read `tmp/session8/agents/common.md` first; it holds the session brief's §1–§2, the agent rules, and the output format. You write `tmp/research_notes/engine-prep/R1.md`.

## Your question

How fast rule and query engines are built in Rust, from their source code and their authors' writing, and which shape fits Tally's engine. Speed is the user's core concern; the answer must also stay buildable under phase 1's strict rules and readable by its owner.

1. **Prior art, from source and authors' writing.** For each, how it evaluates, how "a change costs what it touches" is done, and what that costs in memory:
   - **salsa and rust-analyzer:** demand-driven memoised queries, revision numbers and red-green verification, early cut-off (backdating), durability, interning, LRU eviction; rust-analyzer's own memory figures and its architecture notes.
   - **Datalog and dataflow in Rust:** differential dataflow, DDlog (and why PERFORMANCE §1 point 2 calls compiling rules into code "DDlog's mistake"), ascent.
   - **Spreadsheet engines:** IronCalc (Rust); a spreadsheet's dependency graph and dirty marking (Excel's documented recalculation model is fine prior art).
   - **Policy evaluators:** cedar (a strict Rust evaluator; phase 1 found it the closest precedent for the rulebook).
   - **The evaluator's form:** tree-walking over a resolved tree, against closures compiled once per rule, against bytecode, in Rust. Measured evidence where it exists (published interpreter comparisons in Rust and elsewhere, with their method).
2. **Which shape fits Tally.** Map each prior-art mechanism onto Tally's evaluation, and say which to take:
   - the read rule (LANGUAGE §4.6) and rows read by (target, keys, cut);
   - the memo keyed by key values (BUILD-NOTES seam F), walk-bound defs (ENGINE §3.2), cut classes (PERFORMANCE §3);
   - the day walk with its strata and decisions (LANGUAGE §5.14.1–§5.14.2; ENGINE §4.1–§4.3: the plan pass, the walk, projections);
   - scenarios as overlays (LANGUAGE §6.4), pins (`as if`, `with`), axes;
   - joint members as their own contexts (seam J), paths (ENGINE §7), "is it random" per context (seam R);
   - a change costs what it touches (PERFORMANCE §4: static cone, verification, early cut-off, durability tiers, the walk restart from checkpoints).
3. **What shape.md says about where the time goes, and what the design does about it.** In refcore, half of all def reads are first reads; a few defs keyed by day carry most misses (A 85 %, X 95 %); reads repeat 2–47× per key; refcore steps every calendar day though only 2–7 % have work; and the same long walk is redone many times (64× in A under `windows each`, 8–12× in E2 for members and asks, 20 written projections in X). Say how each candidate design avoids that work (skipping empty days, an event queue, sharing a walk's prefix across windows, cuts, members and projections; how a recurrence by day is held), and what it costs.
4. **Memory.** The cost per memo cell in each design, and how it fits the 8 GB cap at the sizes of `engine/tests/load/benchmarks.md` (refcore's X check held 2.27 M cells). Whether eviction is needed, and whether it can be deterministic.
5. **Build now or leave the seam.** Incremental re-evaluation across amendments (PERFORMANCE §4) against a plain demand-driven memo per run. Which DONE budgets need incrementality (PERFORMANCE §8: a conversation turn 1–2 s; one fitting step ≤ 20 ms), and what building it from the start costs.
6. **Fit with the rulebook.** BUILDER.md §4 asks for an explicit work stack instead of recursion (ENGINE l.141; SO-10). salsa and most query engines recurse. Phase 1's dependency rules (`deny-tally.toml`, crate allow-lists) make every crate a decision. Say what each option means under those rules: a crate (salsa) or the engine's own memo; recursion or a work stack.

## Read (by range; don't read whole)

- `tmp/session8/shape.md` (whole, one page).
- `engine/spec/PERFORMANCE.md` §1–§4 (lines 25–210), §7 (275–310), §10 (362–369). Its prior art: `tmp/design/prior-art/datalog-incremental.md`, by its own summary and §3–§5 (don't redo it; build on it and check what changed since).
- `engine/spec/ENGINE.md` §2–§5 (lines 77–274), §7 (355–387), §9 (419–435).
- `engine/spec/LANGUAGE.md` §1 (lines 21–94), §4.6 (708–745), §5.14.1–§5.14.2 (1372–1426), §6.1 (1775–1808), §6.4 (1848–1883).
- `engine/BUILD-NOTES.md` (whole, 5 KB).
- `tmp/research_notes/engine-rust-strictness/BUILDER.md` §3–§5; `NOTE.md` §1 and §3.
- `engine/tools/refcore/HANDOVER.md` § Layout (lines 942–977), to see how the oracle is organised.

## Decisions to frame (merge or split as the evidence shows)

The evaluator's form; the memo (the engine's own or a crate, its key, its layers for scenarios, pins and members); the walk (event-driven, checkpoints, prefix sharing); incrementality now or as a seam; what of seam J is built at first.
