# Engine prep: the lead's note (session 8)

**Made:** 2026-10-30 by the session 8 lead, from R1–R6 in this directory and `tmp/session8/shape.md`. **Read with:** phase 1's `../engine-rust-strictness/NOTE.md`, `RULEBOOK.md` and `BUILDER.md`, which this note extends and in places amends. **Next:** `tmp/session8/decisions.md` (gate G1) and `tmp/session8/plan.md` (Part B).

Nothing was installed, compiled or benchmarked. Every engine figure below is an estimate (marked **inference** in its report) until the engine runs. Decision ids (D1-3, D4-6, …) point into the reports.

| Report | Topic | Size | Agent's calls | Audit |
|---|---|---|---|---|
| R1 | the evaluation design | 33 KB | 71 | clean |
| R2 | data and numbers | 34 KB | 74 | clean |
| R3 | parallelism and determinism | 34 KB | 80 | clean |
| R4 | measuring as a guarded habit | 38 KB | 116 | clean |
| R5 | readable Rust | 35 KB | 91 | clean |
| R6 | how Claude is instructed; the rulebook against 2.1 | 35 KB | 95 | one parser flag, reviewed: curl fetches of public data into its scratch folder, nothing installed or run |

No agent opened, listed or searched `tmp/holdout/` (`tmp/session8/agents/R<n>/audit.txt`).

## 1. Merged findings

1. **The speed is in the structure, not in how fast one node runs** (R1 §2.1, R5 §3, shape.md). refcore steps every calendar day, though only 2–7 % have work, and redoes the same long walk 64× (A), 8–12× (E2) and 20× (X). The design removes that work:
   - a walk driven by events, so empty days cost nothing;
   - monthly checkpoints, which also let timelines share their common start (a fork at the first day they differ);
   - a memo context trimmed to what each item reads;
   - recurrences keyed by day held as dense columns.
   The evaluator's own form matters about 1.5× at most: closures against a tree walk (Mitchell 2020: 2.1 s / 1.4 s / 1.5 s), and AST and bytecode on par (Larose et al., OOPSLA 2023). So: **a tree-walk over a resolved tree**, readable, with closures kept as a later local optimisation if day-dense asks like E1 miss their budget (D1-1).
2. **The memo is the engine's own; no framework crate fits the approved rules** (R1 D1-2, R2 Q3–Q5). salsa 0.28.5 depends on `hashbrown` and `rustc-hash`, both banned by name (R-37), and brings about 15 more crates. The interning crates (lasso, string-interner) need hashbrown too. `im` is flagged unmaintained, `imbl` pulls rand crates into what ships, and smallvec has advisories. Each replacement is a few dozen to a few hundred lines of plain code. salsa's *ideas* carry over: read sets, early cut-off, durability.
3. **The memo's shape** (R1 D1-3, D1-5, D1-6; R2 D2-5, D2-7):
   - the key is (item, key values, an interned and trimmed context);
   - scenarios, pin sets, axis combinations, members, paths and projections are layers, and a cell lives in the lowest layer its cone reaches;
   - a recurrence keyed by day or period is a dense column per (def, context);
   - one memo per context, freed when the context ends, with no eviction, so peak memory is deterministic;
   - read sets are built now; incrementality across amendments (PERFORMANCE §4) stays a seam until a measured turn misses 1–2 s.
4. **Numbers** (R2 D2-1, D2-2):
   - `Num` is `Small(i64/i64)` or `Big`, with **no i128 tier**. In E1, all 1 693 results above 63 bits are also above 1 024 bits (shape.md: none between), so an i128 tier would hold nothing. i128 stays only for the intermediate of a cross-multiplication.
   - A `d == 1` shortcut, since 77–99 % of results are integers.
   - **Write the Lehmer gcd now.** GMP, CPython, malachite and dashu use Lehmer at Tally's sizes, and num-bigint alone uses the slower binary gcd. GMP's switch to its faster large-number algorithm comes above 330 limbs on arm64, and the cap is 213 limbs (64-bit words), so Lehmer is the most Tally ever needs.
   - num-bigint 0.4.8 stays (R-30's pin; 0.5.1 exists).
   - Money is `I64 | Big` minor units: every money value in six checks was a whole number of cents, and phase 1's i128 money tier is dropped.
5. **Data layout** (R2 D2-3, D2-4, D2-6):
   - one hand-written interner, `Symbol(u32)`, whose output order is by its text, as rustc does, so ids never decide output (R-47);
   - an own `Table<I, T>` with `get` only and no `Index` impl, so indexing is impossible, not just linted;
   - B-trees by default, sorted slices for frozen tables, dense vectors for position-keyed data;
   - checkpoints as `Vec<Rc<PlaceState>>`, copied on write;
   - no collection or arena crate.
6. **Parallelism** (R3 D3-1 to D3-7):
   - workers only fill numbered slots, and one thread merges them in task order;
   - std's scoped threads with an explicit stack size, no rayon (its `reduce` and its multi-error choice are documented as not deterministic);
   - paths and independent timelines (axis combinations, `windows each`, rolling origins, scenarios) in the first build; members and parallel asks later;
   - the shared layer frozen after the first task;
   - the worker count is an argument, the CLI defaults to 6, never read from the machine.
   Two new facts: std's default thread stack reads `RUST_MIN_STACK` (an environment read R-39 forbids) and is 2 MiB against the main thread's 8 MiB. And refcore's exact Maritz–Jarrett standard error is out of reach at D-10's 10 000 paths, so the engine refines intervals to the 1e-6 rounding, which L §7.1 allows (D3-7).
7. **Measuring** (R4 D4-1 to D4-8):
   - **Only counts block a merge:** the engine's own counts (shape.md's units, behind a `counters` feature that is off in release) and Valgrind instruction counts (gungraun, the renamed iai-callgrind, 0.20.0) with hard limits on Linux CI.
   - Wall time and peak memory are judged at each milestone tag on **the reference machine**, the user's MacBook Pro M2 Pro (32 GB), on AC power, as the median of 10 runs.
   - Wall time on hosted CI gives about 45 % false alarms at a 2 % gate (CodSpeed 2025). Valgrind has no Apple Silicon port. On the Mac, `/usr/bin/time -l` prints instructions retired and peak footprint without sudo.
   - The scaffold's release profile stays (fat LTO, one codegen unit, abort on panic, overflow checks). The overflow checks' cost is measured once at S; PGO comes only at M9, and no budget may depend on it.
8. **Readability** (R5 D5-1 to D5-7). The owner audits through three things, not through Rust:
   - a ~200-line `ARCHITECTURE.md`;
   - a script-checked `//!` header in every file (purpose, `Spec:`, `Twin:` the refcore file, `Invariants:`);
   - a script-checked trace from code to spec sections, with the reverse map.
   A small subset follows: no `dyn`, no own generics or single-implementation traits, no `impl Trait` arguments, no library macros, short iterator chains. Nesting becomes an error at 5; length stays a report, since clippy scores a whole `match` as +1. Performance code lives only in fenced modules, each with a benchmark, a plain twin and an equivalence test. Strictness stays readable through newtypes with checked methods. Readyset measured removing all bounds checks as "negligible", so the subset costs no speed.
9. **How Claude is instructed** (R6 D6-1 to D6-5):
   - **Nothing exists to adopt.** Anthropic publishes no Rust skill; its marketplace has only the `rust-analyzer-lsp` plugin. The registry the `skills` CLI uses lists about 100 Rust skills. The best five each recommend something the rulebook bans (`anyhow`, `unwrap_or_default`, fast hashers, `insta`), and one injects "MANDATORY" text on every prompt and runs an unpinned `npx …@latest` server.
   - So the 2026-10-22 decision stands: **create `engine-rust`**, borrowing ideas only.
   - Claude Code's docs say CLAUDE.md and skills are context, not enforcement. A nested `engine/CLAUDE.md` isn't loaded when a session starts at the repo root, or after compaction. So the never-miss rules stay in lints, the guard and the Stop hook; a build-role SessionStart hook prints `engine/CLAUDE.md`; the skill is invoked by name.
   - The scaffold's guard doesn't checksum the skill or `engine/CLAUDE.md`: a gap.
   - The skill's eval is 9 planted cases graded by scripts, run in Part B.
10. **The rulebook against tally 2.1** (R6 D6-6, plus R2–R5):
    - five new rows: R-114, the largest-remainder split written once; R-115, parts sum to the balance (⚖); R-116, holes built in one module (seam U); R-117, a path statistic's accumulator outside the cap; R-118, a report of derived orders;
    - four amended for 2.1: R-16 (joint nesting is a worklist), R-40 (full-path code-point order), R-47 (labels, `same` classes, member prefixes), R-76 (the code count);
    - amendments from the other reports, listed in §2.
11. **Memory fits 8 GB, by estimate only:** 60–300 B per memo cell (R1), 70–90 B sparse and 24–32 B dense (R2), against refcore's ~2.2 KB. X's 2.27 M cells come to about 0.1–0.7 GB. B-07's peak is unknown: it never finished in refcore. Proposed budgets: ≤ 512 MB per 1× check, ≤ 4 GB per load row, 8 GB a hard ceiling (R4 D4-7).

**Where the reports met independently.** Structure over node speed (R1, R2, R5, shape.md). No crate for the core (salsa, interners, arenas, persistent maps, rayon each fail a rule or determinism). Memory released by scope (R1, R2, R3). Counts as the stable measure (R4, which reuses shape.md's units, the same ones R1 names as the cost drivers).

## 2. Contradictions and how I settled them

"G1" marks a settlement the user decides at gate G1; the decision item is named.

| # | Question | Positions | Settlement | Why |
|---|---|---|---|---|
| K8-1 | Recursion in the evaluator | BUILDER §4: an explicit work stack, never Rust recursion. R1 D1-1: recursion across cells up to a counted depth, an explicit stack and replay past it. R6 O-2: joint models nest with no limit or code, so members need a worklist. R3: a worker's stack is 2 MiB unless set | **G1, item 1.** My pick: R1's counted recursion, with every thread spawned with an explicit stack sized for the count (R3) and member nesting as a worklist (R6). Keeps R-16 (depth counted, never measured) | Recursion reads like the spec; the count makes it safe; the work stack stays the fallback past the count |
| K8-2 | When to write Lehmer's gcd | BUILDER §2: only if benchmarks ask. R2: now | **R2: now** (G1 item 2) | E1, a real household check, reaches 5 049 bits; GMP's thresholds make Lehmer the ceiling below the cap |
| K8-3 | Money's tiers | BUILDER §2, A3 F-17: `I64 \| I128 \| Big`. R2: `I64 \| Big` | **R2** (G1 item 2) | No household value nears 9.2·10¹⁶; one fewer tier to test |
| K8-4 | An interner | BUILDER §3: none. R2: one, ordered by text | **R2** (G1 item 3) | Memo keys hold key values (seam F); text order honours R-47 |
| K8-5 | The cap "after every operation" | BUILDER §2. Tally 2.1's change 2.1-15 (L l.2103): a path statistic's accumulation is uncapped | **The spec wins**; new row R-117 (G1 item 8) | BUILDER predates 2.1 |
| K8-6 | A hash map for the memo | R-36/R-37 ban all hash maps. R2 D2-5 (b) and R6 P-1: one sanctioned, never-iterated hash memo. R1 D1-3: dense columns | **None now** (G1 item 3); dense columns and B-trees; revisit only if the first engine profile shows sparse memo lookups dominating | The heavy misses are day-keyed (dense); an exception needs evidence first |
| K8-7 | R-37's text | bans hashbrown outright; the scaffold's `deny-tally.toml` lets indexmap wrap it; BUILDER §7 allows IndexMap | **Reword R-37:** "except under indexmap" (G1 item 8) | The scaffold and BUILDER already agree |
| K8-8 | Thread spawning | R-42: `thread::scope`. R3: its plain spawn reads `RUST_MIN_STACK` and gets a smaller stack | **Amend R-42** with R3's five points (G1 item 4) | A hole, not a reversal |
| K8-9 | Where wall time is measured | R-97 and the scaffold's `bench` job: `ubuntu-latest`. DONE gate 12: "on the reference machine". R4: hosted runners are 2 vCPU and 8 GB for a private repo, and noisy | **R4:** wall time and memory on the reference Mac at the milestone; counts on Linux per PR (G1 item 5) | Gate 12's own words |
| K8-10 | Function size and nesting | K-21: both advisory. R5: nesting an error at 5, length a report. The scaffold's `advisory` reports at 60/20/4; A5 said 100/6 | **G1 item 6.** My pick R5's, thresholds aligned, re-judged at S | clippy scores a `match` as +1, so dispatch isn't penalised; the library's exception budget is 0 |
| K8-11 | R-73 (macros) and R-75 (citations) | report-only. R5: errors (RS-6; the trace in the kernel) | **G1 item 6**, through R-112's procedure | — |
| K8-12 | File order | R-40: a listing sorted by name bytes. ENGINE l.118: full relative paths by code point | **Amend R-40** to full paths (G1 item 8) | `a/b.tally` and `a.x.tally` swap; the hash changes |
| K8-13 | The code count | R-76 and R-58 say 87. 2.0 has 91 and 2.1 adds four; R6's regex finds 98 names | **Recount with `tools/xref_check.py`** in Part B; fix the number only | The rule's test compares against the spec's tables anyway |
| K8-14 | The guard and the instructions | memory note: the guard already refuses `.claude/**` and `engine/CLAUDE.md` to the build role. R6 C-3: true for direct edits, but neither is checksummed | **Add both to the rule globs and `RULES.sha256`** (D6-3; G1 item 7) | The 2026-10-22 requirement is "the builder can't rewrite its own rules" |
| K8-15 | Counters and R-93 | R-93 forbids test-only behaviour. R4 D4-8: a `counters` feature off in release | **Accept**, like R-51's fault injection; outputs stay byte-identical (G1 item 5) | Counts are deterministic and never printed in release |
| K8-16 | SO-22 | phase 1: path-order merge required because the cap checks partial sums | **Superseded** by 2.1-15; the order is still required, now for which refusal prints (R3 D3-3) | — |
| K8-17 | refcore against the spec | refcore caps a mixed-unit or rate `mean`'s partial sums in path order (`eval2.py` l.952–958, `evaluate.py` l.4340); L §7.1 R-13 leaves accumulations uncapped | **The engine follows the spec.** Logged for the spec record; gate 10 differs only past 4 096 digits | refcore is frozen; DONE rule 3 |

## 3. Spec observations (new; not fixed, the spec is frozen)

They go to the next spec patch's gap log (`CHANGES.md`), or to DONE where marked.

| # | Where | Observation | From |
|---|---|---|---|
| SO8-1 | ENGINE §4.2; PERFORMANCE §1 | No section lists what makes a day non-empty (readings, observations, legs, decision days, lifespan ends, parts checks, `known` dates that move the look-ahead). An event-driven walk must derive the list; a step-every-day walk kept as a test oracle checks it | R1 |
| SO8-2 | ENGINE §3.2; PERFORMANCE §3 | Whether a cell whose cone never reaches a projection's walk drops the projection from its context is unsaid; X's 95 % of misses suggest it matters | R1 |
| SO8-3 | LANGUAGE §8.4, §7.5 | Which refusal prints when several paths or terms refuse (refcore: term order, then the lowest path) | R3 SO-paths-1 |
| SO8-4 | LANGUAGE §5.16 | The byte encoding of a draw's address (refcore: UTF-8) | R3 SO-paths-2 |
| SO8-5 | ENGINE §7 step 5 | "Contiguous blocks, one per worker" is a means; the guarantee is the ordered merge | R3 SO-paths-3 |
| SO8-6 | LANGUAGE §5.16 | Which of several equal quantile values with different `≈` enclosures prints (refcore: a stable sort) | R3 SO-paths-4 |
| SO8-7 | ENGINE §9; LANGUAGE §3.6 | "Parts sum to the balance" is not among §9's invariants, though `ENGINE-INVARIANT` is defined as "§9 fails" | R6 O-1 |
| SO8-8 | ENGINE l.118; LANGUAGE §6.1 | Joint models nest with no limit and no refusal code | R6 O-2 |
| SO8-9 | DONE gate 12 | Its evidence says "criterion reports", but most rows are end-to-end runs; it cites `tmp/design2/benchmarks.md`, now `engine/tests/load/benchmarks.md` (a DONE change for the user) | R4 |
| SO8-10 | PERFORMANCE §8 | The interactive rows are sized by "the prototype's instances", a tally 1 notion; their 2.1 sizes need T's generator | R4 |

## 4. What stays unknown until the engine runs

Each is a measurement, never settled on paper. They become Part B's or S's first measurements, and the budget rows that depend on them stay "proposed".

- **The evaluator:** the nanoseconds per node with exact arithmetic on the i64 tier (shape.md assumed 10–50 ns); whether day-dense asks like E1 need closures; the read-set overhead on a cold run.
- **The memo and the walk:** bytes per memo cell; how many cells remain once empty days are skipped and prefixes shared; the checkpoint interval (monthly is a guess); how late A's and E2's timelines diverge, which sets the fork's saving; whether a cold 33-year run meets ≤ 1 s at 10× (which decides incrementality, D1-5).
- **Memory:** B-07's and B-12's peaks with per-context memos (L:4, L:5, L:14); the size of one path's layer at B-12, D-10 and household-bands (decides D3-5).
- **Numbers:** the share of balanced big gcds in E1 (what Lehmer buys); how often i64 intermediates overflow inside one operation; whether `Num` is 16 bytes.
- **Parallelism:** the 1→6 speed-up; duplicated shared cells under the warm-up freeze; the Maritz–Jarrett kernel's cost; the deepest stack a case needs.
- **The build:** overflow checks' cost, inside num-bigint in particular; fat LTO's build time; gungraun's count stability across CI machines; Valgrind's slowdown on a 1 s run; samply on macOS 26; laptop thermals over a 10-run median; whether nesting 5 fits the lexer, the parser and the walk without exceptions.
- **The instructions:** whether the skill changes the builder's behaviour beyond the lints and `engine/CLAUDE.md` (D6-4's runs); whether rust-analyzer's diagnostics arrive fresh; whether a SessionStart hook's output survives compaction as reliably as the root CLAUDE.md; whether a session started in `engine/` loads the root settings.

## 5. Versions checked on 2026-10-30

Rust stable 1.99.1 (1.100 due 2026-11-12); ast-grep 0.51.0; num-bigint 0.4.8 (0.5.1 newest); salsa 0.28.5; differential-dataflow 0.25.1; ascent 0.8.1; ironcalc_base 0.9.1; cedar-policy-core 4.14.0; malachite 0.14.0; rayon 1.12.0; sha2 0.11.1; proptest 1.12.0; loom 0.7.2 (no `thread::scope`); shuttle 0.9.9; crossbeam-utils 0.8.23; gungraun 0.20.0 (iai-callgrind renamed); Valgrind 3.28.0 (no Apple Silicon); hyperfine 2.0.0; samply 0.13.1 (2025-02); cargo-flamegraph 0.6.14 (needs Xcode's `xctrace` on macOS); the `skills` CLI 1.7.4; the `rust-analyzer-lsp` plugin 1.0.0.

## 6. Where the decisions go at G1

| G1 item | Decisions |
|---|---|
| 1. The evaluation design | D1-1 to D1-7, K8-1 |
| 2. Numbers | D2-1, D2-2 |
| 3. Data layout | D2-3 to D2-7, K8-6 |
| 4. Parallelism | D3-1 to D3-7 |
| 5. The performance gates | D4-1 to D4-8 |
| 6. The readability rules | D5-1 to D5-7 |
| 7. The skill | D6-1 to D6-5 |
| 8. The rulebook's 2.1 rows | D6-6, plus the amendments K8-7, K8-8, K8-10, K8-11, K8-12, K8-13 |
| 9. Landing the stale documents | (the brief's §6 item 9) |
