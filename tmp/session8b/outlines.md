# Session 8b, step 1: the outlines

**Made:** 2026-10-30 by the 8b lead, from `tmp/session8/decisions.md` (G1, all nine items approved as recommended), `tmp/research_notes/engine-prep/NOTE.md` (K8-1..K8-17, SO8-1..SO8-10, §4 unknowns), `tmp/session8/plan.md` B1.1–B1.4 and `tmp/session8/shape.md`. Each writer turns its outline into a draft under `tmp/session8b/drafts/`. An id in brackets is what a section carries; a section with no id restates 1.0.1's text in 2.1's constructs.

**Conventions for all three spec documents** (they keep 1.0.1's):
- They plan the engine; they carry no language meaning (G1 item 9). A sentence that would need to say what a construct means, beyond what LANGUAGE or ENGINE says, is not written: it goes to `tmp/session8b/open.md` (DONE rule 3).
- They cite LANGUAGE and ENGINE by section (L §n, E §n), the decisions by id (D1-3, K8-1, SO8-1), and the reports by name (R1 §2.1). They name no crate and no Rust API: those live in BUILDER and RULEBOOK, cited by row (R-42, BUILDER §7).
- Every estimate shows its arithmetic, in a `python3 -c` block as 1.0.1 did, and is marked an estimate. Nothing on paper is "proven".
- Every budget number is **proposed until S**: measured at S, then fixed with the user, a too-tight number changed only through a logged row.

---

## A. `PERFORMANCE.md` for tally 2.1 (writer W1)

| § | Section | Carries |
|---|---|---|
| — | Header: what the document covers; tally 2.1; estimates, not measurements; "proposed until S" | G1 item 9; D4-6 |
| 1 | **Units and the strategy.** The units are shape.md's (ev, def cell with hit and miss, read and distinct read, walk day and non-empty day, legs booked, parts groups, Num results by bit size), printed by the counters behind `--stats`; then the strategy as a numbered list | D4-8, K8-15; D1-1 (tree-walk over a resolved tree; closures only where a measured ask misses), K8-1 (counted recursion, explicit stack past the count, joint nesting a worklist), D1-2 (own memo), D1-3, D1-4, D1-5, D1-6, D1-7 (one line each, detail in §2–§5). 1.0.1's items 1–7 kept where still true (load once per reality, rules as data, static dependencies, demand-driven, coarse cells, paths untracked); item 7 (memo across amendments) becomes §4's seam |
| 2 | **The walk.** 2.1 the flow pass (if 2.1 still has one; else say what replaced it); 2.2 the event-driven walk: only non-empty days are stepped; **SO8-1's list of what makes a day non-empty, derived item by item from L and E with a citation each** (readings, observations, legs, decision days and their strata, lifespan ends, parts checks, `known` dates that move what a projection may read, anything else the derivation finds); the step-every-day walk named as its test oracle (equal output, more days); 2.3 monthly checkpoints (interval an unknown, §10); 2.4 timelines forked at their first different day, built second, once the walk matches refcore; 2.5 the written projection (`low`, `bal(x, d) from t`, `next`, D-6) and `value`, replacing 1.0.1's `plan_low`, `next_on` and `worth` | D1-4; SO8-1; D2-6 (checkpoints as copied-on-write place states, small layers for pins and scenarios); shape.md "what carries over" 3; E §4.2–§4.3; D-5's stratified order within a day |
| 3 | **Contexts as layers.** The memo key (item, key values, interned trimmed context); the layers: scenarios, pin sets, axis combinations, members, paths, projections; a cell lives in the lowest layer its cone reaches; **trimming, including whether a cell whose cone never reaches a projection's walk drops that projection from its context (SO8-2)**, derived from L and E; a recurrence keyed by day or period as a dense column per (def, context); one memo per context, freed when its scope ends, no eviction; memory per cell and X's estimate with arithmetic; 1.0.1's cut classes, scenarios, pins and rolling origins restated as layers | D1-3, D1-6, D2-5, D2-6, D2-7, K8-6 (no hash map now); SO8-2; NOTE §1 item 11 (60–300 B per cell, R1; 70–90 B sparse, 24–32 B dense, R2); shape.md "what carries over" 1, 2, 6 |
| 4 | **A change costs what it touches.** Read sets built now; 1.0.1's §4.1–§4.4 (invalidation, early cut-off, the walk restart from a checkpoint, the cost of one turn, the fitting loop) kept as the seam's design, built only when a measured turn misses 1–2 s, or a cold 33-year run misses 1 s at 10× | D1-5; NOTE §4 (read-set overhead, the cold run at 10×) |
| 5 | **Paths and parallel timelines.** What runs in parallel first (Monte Carlo paths; axis combinations, `windows each`, rolling origins, scenarios as independent timelines) and later (joint members, parallel asks); workers fill numbered slots, one thread merges in task-index order; the merge rule (per line, terms in order; the first refusing path wins, else the union of holes; quantiles sorted stably by value then path; SO8-3, SO8-6); the shared layer run alone first, then frozen, with a counter of locally recomputed cells; the worker count an argument, the CLI's default 6, never read from the machine; every thread with an explicit stack size; testing gate 6 (the worker sweep, reversed completion, planted violations, a deep-stack case); a quantile's standard error by intervals refined to the printed 1e-6 rounding, refcore's exact method the oracle at small N; a path statistic's accumulation uncapped (2.1-15); paths keep no read sets | D3-1..D3-7 (as design, the Rust form cited to R-42 and BUILDER §7); K8-16, K8-17 (the engine follows L §7.1 R-13, gate 10 differs only past 4 096 digits); SO8-3..SO8-6 |
| 6 | **Numbers.** `Num` small (64-bit numerator and denominator) or big, no middle tier (E1: none between 64 and 1 024 bits), a 128-bit intermediate only inside one cross-multiplication; the whole-number shortcut; Lehmer's gcd written now; money a whole number of minor units, 64-bit or big; the cap (4 096 digits, 13 606 bits, 213 limbs) and why Lehmer is the most Tally needs; shape.md's bit-size figures | D2-1, D2-2, K8-2, K8-3, K8-5; NOTE §1 item 4; shape.md "what carries over" 4 |
| 7 | **Cost per construct**, 1.0.1's table restated for 2.1, with new rows: parts and labels (the arrival rule, takes and relabels, the split; paid only where a kind opts in, shape.md "what carries over" 5; R-115's check at O(labels) per booking), observations, joint members as contexts (a lazy walk to the last day read; pruning to the places read second), the written projections, the `parties` kind, keyed decisions | D1-7; R-114, R-115 (rows in W3's RULEBOOK); STALE.md's 2.1 note (L:18); shape.md's A and parts probe figures |
| 8 | **Budgets**: the rows of §C below, with their fixed ids, columns: id, what, size, refcore's figure (the floor column: an upper reference, not a target), budget (wall, footprint, count), where and when measured, what blocks | D4-1, D4-2, D4-3, D4-6, D4-7; SO8-10 |
| 9 | **What can grow, and what caps it**, 1.0.1's table checked against 2.1's codes and numbers, plus: a path statistic's accumulator, uncapped (2.1-15, R-117); joint-model nesting, uncapped, walked as a worklist (SO8-8, R-16); the memory ceiling (8 GB on any run); the out-of-reach steps restated in 2.1 (no `bound` keyword in tally 2: a hole with its reason, D-15) | 2.1-15; SO8-8; D4-7; K8-17 |
| 10 | **What stays open**: each of NOTE §4's unknowns that this document's numbers rest on, one row each, with the section it affects and the milestone that measures it | NOTE §4; D1-5's trigger; D4-4 (overflow checks measured once at S; PGO only at M9, no budget may need it) |

The instruments, the release profile and the profiling recipes (D4-1, D4-4, D4-5) are BUILDER's new performance section (W3); PERFORMANCE §8 names only where each row runs and what blocks.

## B. `BOUNDS.md` for tally 2.1 (writer W1)

- **Scope (plan B1.2): where the budgets and caps need it,** but enough that `STALE.md` can drop the document: every row's *Tally* column and notes name 2.1's constructs and sections (STALE.md flags the whole table, since it sits before the first heading).
- **No status changes** (the user's decision, 2026-10-18: paper adds nothing to "proven"). If refcore's runs, or 2.1, seem to bear on a status, the writer logs it on `open.md` and leaves the status.
- The intro's statuses restated: tally 2 has no `bound` keyword; an out-of-reach step is an ordinary hole with its reason (D-15). The out-of-reach list (rows 37, 40–43, 49) as PERFORMANCE §9 states it.
- The caps rows (exact numbers, paths, axis combinations, nesting) agree with PERFORMANCE §9 and L's codes.
- The 2.1 note: parts and labels have a cost row in PERFORMANCE §7, not a bound; if a 2.1 ability (labels, joint members, observations, `parties`) needs a BOUNDS row, the writer proposes it on `open.md` rather than adding it.

## C. PERFORMANCE §8's rows, with fixed ids (W1 writes them, W2 cites them)

Every number is **proposed until S**. Columns W1 fills: size, refcore's figure, wall, footprint, count, where, what blocks.

**Group I: the interactive targets kept** (1.0.1 §8, N-2001..N-2007; sizes in 2.1's models need T's generator, SO8-10; measured on the reference Mac at the milestone, D4-3)

| Id | Row | Budget kept |
|---|---|---|
| P8-1 | A conversation turn (N-2001) | 1–2 s end to end |
| P8-2 | A deterministic run, cold, 33 years daily at 10× (N-2002) | ≤ 1 s |
| P8-3 | A deterministic run, warm | ≤ 50 ms |
| P8-4 | Monte Carlo, 1 000 paths × 30 years daily, law on every path, 6 workers (N-2003) | ≤ 1 s |
| P8-5 | One look-ahead decision; in 2.1 one read of a written projection (`low`, D-6): W1 restates it, or logs it on `open.md` if 2.1 has no counterpart | ≤ 0.3 µs |
| P8-6 | One path replayed for an explanation (N-2004) | ≤ 20 ms |
| P8-7 | The gate on one law part (N-2007) | ≤ 1 s |
| P8-8 | One fitting step (N-2005) | ≤ 20 ms for two years of history |
| P8-9 | Determinism: the same bytes across runs, 1 and 6 workers, machines (gate 6) | equal hashes |

**Group C: the six measured checks at 1×** (≤ 1 s each, cold, 1 worker; ≤ 512 MB; def-cell misses ≤ refcore's; counts and instructions every PR on Linux, wall and footprint on the Mac at the milestone; D4-6, D4-7)

| Id | Check | refcore (shape.md): plain s, peak GB, def misses |
|---|---|---|
| P8-10 | `I:A:a05_ania@c05-ania` | 37.7 s, 0.81 GB, 150 k |
| P8-11 | `I:E1:tx2_mtg_day` | 60.5 s, 2.17 GB, 619 k |
| P8-12 | `I:E2:E2-01~e2_01a` | 49.1 s, 0.91 GB, 161 k |
| P8-13 | `I:E2:E2-01~e2_01b` | 65.8 s, 1.02 GB, ≥ 547 k cells (partial) |
| P8-14 | `I:X:replay_september@b_r1` | 52.6 s on fast, 4.95 GB †, ≥ 2.27 M (partial) |
| P8-15 | `U:set/pairs/PTY-05+OWN-12/ania` | 0.87 s, 0.08 GB, 74 |

**Group L: the load rows at full size**, one per `engine/tests/load/load.tsv` row, the id's number being the row's `L:n` (≤ 4 GB each, 8 GB a hard ceiling; non-Monte-Carlo ≤ 10 s; Monte Carlo at N-2003's rate, 1 s per 1 000 paths × 30 years on 6 workers, scaled with its arithmetic; on the Mac, nightly and at M9, and at each milestone's exit for the rows it touches):

P8-L1, P8-L2, P8-L3, P8-L4, P8-L5, P8-L6, P8-L8, P8-L9, P8-L10, P8-L11, P8-L12, P8-L13, P8-L14, P8-L15, P8-L16, P8-L17, P8-L18.

Note: `load.tsv` has **17 rows, numbered L:1–L:18 with no L:7** (checked 2026-10-30); G1's "18 load rows" counts the numbering. W1 marks each row Monte Carlo or not from `load.tsv` and `benchmarks.md` (L:1, L:5, L:11, L:12 and part of L:14 draw paths), and splits a mixed row's budget by its runs.

**Group M: micro rows** (criterion on the Mac, instruction hard limits by gungraun on Linux; each set at S from the first measurement × 1.2, then only ratcheted down by the lead; R4 Q6 group IV)

| Id | Row |
|---|---|
| P8-M1 | `Num` add and multiply on the small tier |
| P8-M2 | gcd of a big and a small number |
| P8-M3 | one big operation at 5 049 bits (E1's largest result) |
| P8-M4 | one read of a written projection (P8-5's micro form) |

W1 may add a micro row only where R1 or R4 names one; otherwise it goes on `open.md`.

---

## D. `BUILD-ORDER.md` for tally 2.1 (writer W2)

| Part | Carries |
|---|---|
| Header | the tests as they now exist: `engine/tests/checks.tsv` (unit probes `U:`, integration `I:`, refcore's own `R:all`), the load rows `L:n`, and what T still writes (plan §2: goldens, hindcast, the codes' missing fixtures, properties, the generator); the refusal fixtures' count left as "counted by `tools/xref_check.py`" (K8-13) |
| Summary | S, M0–M9 in 2.1's constructs: each with what it builds, the checks turning green (ids or globs), DONE gates touched, the P8 rows at 1× |
| S | the tracer slice on the household (kept); the kernel: the tree-walk evaluator with counted recursion, the memo with its key, the event-driven walk with the step-every-day oracle, monthly checkpoints, `Num` with Lehmer, money, the interner, `Table`, the counters, `ARCHITECTURE.md` and the `//!` headers, the trace script, the fences; P8 micro rows set (P8-M*), overflow checks measured once | D1-1..D1-4, K8-1, D2-1..D2-5, D4-4, D4-6 (micro), D4-8, D5-1, D5-4, D5-6 |
| M0–M9 | 1.0.1's milestones restated with 2.1's constructs. **Parts and labels** (owned and batched kinds, the arrival rule, takes, relabels, the split R-114, the parts check R-115) and **joint members as contexts** (D1-7: a lazy walk to the last day read; seam J's first step; pruning to the places read second) each get a named milestone; observations, `parties`, keyed decisions placed where they first appear. Layers (D1-3: scenarios, pins, axes, members, paths, projections) placed where their construct is built. Parallel paths and timelines (D3-1..D3-7) at the Monte Carlo milestone or where the first independent timeline appears, W2's call with its reason; forks (D1-4's second step) after the walk matches refcore; PGO only at M9 (D4-4) |
| Each milestone | its checks from `checks.tsv` by id or glob; **every one of the file's 634 checks placed exactly once** (shown by a placement table and a script, below); the P8 rows it touches at 1× (P8-10..P8-15 where their check's constructs are built; P8-L rows at their milestone's exit at full size where the construct is complete, or at M9); the DONE gates it advances; the milestone report (D5-5) |
| M3 | the operations rest on `OPERATIONS.md`, which stays stale: M3 says so, and names what waits for its rewrite (P8-1 and P8-7 lean on it too) |
| What every milestone keeps | 1.0.1's list plus: counts and instruction limits on every PR, wall and footprint on the Mac at the tag (D4-2), the milestone report (D5-5), `ARCHITECTURE.md` updated (D5-1) |

**The placement check.** W2 writes `tmp/session8b/drafts/BUILD-ORDER-placement.tsv` (check id, milestone, why) and a short script `tmp/session8b/agents/W2/placement_check.py` that reads `checks.tsv` and the placement and prints any check placed zero or two times; run with `python3 -I -B`. This is a document check, not engine code.

---

## E. RULEBOOK and BUILDER (writer W3), full copies changed

**RULEBOOK** (`drafts/RULEBOOK.md`):
- **New rows** R-114..R-118 (decisions.md item 8: R-114 the largest-remainder split written once, ties only by the written `ties by`, else `RUN-TIE`; R-115 ⚖ parts sum to the balance, ≥ 0 and whole steps after every arrival, take and relabel, a failure refusing as an invariant, checked at run time, O(labels) per booking, SO8-7; R-116 holes and hole parts built in one module, one per-operator rule for holes, seam U; R-117 a path statistic's accumulator its own type, outside the cap check, 2.1-15; R-118 every derived order on the engine's types reported for review).
- **Amended rows:** R-16 (joint nesting a worklist; counted recursion, K8-1), R-37 ("except under indexmap", K8-7), R-40 (full relative paths by code point, E l.118, K8-12), R-42 (R3's five points, K8-8, D3-1), R-47 (order keys for labels in ties, `same` classes, member prefixes), R-58 and R-76 (the count left as "recount at 8c with `tools/xref_check.py`", K8-13), R-73 (library macros an error, K8-11), R-74 (nesting an error at 5, length and cognitive complexity reports, K8-10), R-75 (a missing citation an error in the kernel), R-97 (wall time on the reference Mac at the milestone; counts and instructions on Linux CI per PR, K8-9).
- **A new group, readability,** rows from R-119 on, one per D5-1..D5-7 (R5's RD-n and RS-n ids cited), each with its enforcement (error, report, review) and tool.
- **Every new or amended row** carries its enforcement and the user's G1 answer: "I approve", 2026-10-30, with the item (items 1, 4, 5, 6, 8 as decisions.md records). The summary's counts updated.

**BUILDER** (`drafts/BUILDER.md`): §2 (Lehmer now; money `I64 | Big`; the cap per 2.1-15, R-117), §3 (the interner, `Table<I, T>`, collections D2-5, checkpoints D2-6), §4 (counted recursion, K8-1, replacing "never Rust recursion"), §7 (threads per D3-1, R-42 amended; collections per D2-5; no hash map now, K8-6), §8 (the workaround table: rows for the new bans, no `dyn`, `impl Trait` arguments, library macros, nesting at 5, a lock or channel outside the thread module, so the skill and the eval can quote them), §9 (the loop gains the milestone budget step: counts every PR, the Mac at the tag), §11 (`engine/CLAUDE.md`'s outline, printed by the build role's SessionStart hook; the skill invoked by name; both checksummed, D6-2, D6-3), §12 (the review checklist gains the readability section, D5-5), and **a new section on performance**: the fences (D5-6), the counters and `--stats` (D4-8), the instruments and what blocks (D4-1, D4-2, D4-3), the release and profiling profiles (D4-4), and the profiling recipes (samply plus a summariser on the Mac, `perf` and `callgrind_annotate` on Linux, the counters first, D4-5).
