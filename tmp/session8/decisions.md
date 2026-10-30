# Gate G1: the engine prep's decisions (session 8)

**Made:** 2026-10-30 by the session 8 lead, from `tmp/research_notes/engine-prep/NOTE.md` and R1–R6. Each item below is asked in chat one at a time: the problem, the options, my pick and its cost, and what it does to readability and to speed. The user's words are recorded under each item as given. Ids (D1-3, K8-1, …) point into NOTE.md and the reports.

How to read a pick's cost: **build** is effort in Part B or the build, **read** is readability for the owner, **speed** is the engine's speed, **risk** is what could go wrong.

---

## 1. The evaluation design

**The problem.** Tally's engine is a spreadsheet with a calendar: each def at its keys is a cell, computed when asked and remembered in a cache. refcore spends its time stepping empty days (93–98 % of the days it walks) and redoing the same long walk 8–64 times. The question is how the engine avoids that work, and how it runs one rule.

**Picks** (R1):

| id | Decision | Pick | Cost |
|---|---|---|---|
| D1-1 | How a rule runs | A tree-walk over a resolved tree, one `match` per node kind. Closures later, only where a measured ask misses its budget | build least; read best; speed ~1.5× below closures on node-bound asks |
| K8-1 | Recursion | Recursion to a counted depth, then an explicit work stack; every thread spawned with an explicit stack size; joint-model nesting as a worklist. Keeps R-16. Changes BUILDER §4's wording ("never Rust recursion") | read: code that reads like the spec; risk: low, depth is counted, never measured |
| D1-2 | The cache | The engine's own memo, not the salsa crate (it breaks R-37 and brings ~15 crates) | build: a few hundred lines; read: one table |
| D1-3 | The cache's key | (item, key values, a trimmed context); scenarios, pins, axes, members, paths and projections as layers; a recurrence by day as a dense column | build: moderate; speed: the largest saving in misses and memory; risk: a trimming bug, caught by the byte-for-byte check against refcore |
| D1-4 | The walk | Event-driven (empty days cost nothing) with monthly checkpoints first; timelines forked at their first different day second, once the walk matches refcore | speed: the largest saving shape.md shows; build: the fork needs each timeline's first different day |
| D1-5 | Incrementality across amendments | Read sets now; the rest of PERFORMANCE §4 (verification, early cut-off) stays a seam until a measured turn misses 1–2 s | build: avoids a second code path now; risk: if a cold run misses 1 s at 10×, it is built then |
| D1-6 | Eviction | None; memory is released when its scope ends (a path, a search step, a projection) | deterministic peak memory |
| D1-7 | Joint members (seam J) | A member is its own context with its own memo; its walk runs lazily up to the last day read; pruning to the places read comes second | build: reuses D1-3 and D1-4 |

**Readability:** one evaluator shape, close to refcore's and the spec's. **Speed:** the order of magnitude comes from D1-3 and D1-4.

**The user's answer:** "I approve" (2026-10-30, 22:06): every pick of item 1, K8-1 included.

---

## 2. Numbers

**The problem.** Every number is an exact fraction. Like V8 keeping small integers fast and switching to BigInt only on overflow, the engine needs a small fast form and a big exact form. Over 99.9 % of refcore's results fit 64 bits, but a real mortgage check reaches 5 049 bits, and reducing big fractions needs a fast gcd (greatest common divisor).

**Picks** (R2):

| id | Decision | Pick | Cost |
|---|---|---|---|
| D2-1 | The number type | `Num` = Small (i64/i64) or Big, with no i128 tier (no result fell between 64 and 1 024 bits); a whole-number shortcut; num-bigint 0.4.8 stays (R-30's pin). Our own Lehmer gcd written **now**, not "if benchmarks ask" (BUILDER §2) | build: ~150–250 lines of gcd, tested against num-bigint's and Python's; speed: the big-number path stays faster than refcore's Python; risk: a gcd bug, caught by the canonical-form check |
| D2-2 | Money | A whole number of minor units, `I64 \| Big`; phase 1's i128 money tier dropped (BUILDER §2, A3 F-17) | less code; money + money is one checked add with no gcd |

**Readability:** two tiers instead of three. **Speed:** the common case is one machine add; the rare big case gets the algorithm GMP and CPython use.

**The user's answer:** "I approve" (2026-10-30, 22:06): D2-1 and D2-2.

---

## 3. Data layout

**The problem.** How names, tables and the walk's state are stored. In TypeScript terms: whether names become small numeric ids (like an enum), whether lookups return `undefined` rather than throwing, and whether maps keep a sorted order so output never depends on hashing.

**Picks** (R2):

| id | Decision | Pick | Cost |
|---|---|---|---|
| D2-3 | Interning | One hand-written interner (`Symbol(u32)`), output ordered by text as rustc does. Replaces BUILDER §3's "no interner" | ~50 lines; fixed-size memo keys; risk: an id order leaking, guarded by R-47 and a reordering test |
| D2-4 | Ids and tables | Our own `Table<I, T>` with `get` only, no indexing at all; no arena crate | ~40 lines; indexing becomes impossible, not just linted |
| D2-5, K8-6 | Collections | B-trees by default, sorted slices for frozen tables, dense vectors for day-keyed data; **no hash map**, revisited only if the first engine profile shows sparse lookups dominating | no new rule now |
| D2-6 | Checkpoints and overlays | `Vec<Rc<PlaceState>>`, copied on write; pins and scenarios as small layers; no persistent-map crate (`im` unmaintained, `imbl` pulls rand crates) | std only |
| D2-7 | The memo's scope | One memo per evaluation context, freed when the context ends; eviction decided only after an engine run of B-07 | memory estimate: X's 2.27 M cells in ~0.1–0.7 GB, against refcore's 4.95 GB (inference) |

**Readability:** plain std types, no extra crates to learn. **Speed:** dense columns where the misses are; memory well inside 8 GB by estimate.

**The user's answer:** "I approve" (2026-10-30, 22:07): D2-3 to D2-7, K8-6.

---

## 4. Parallelism

**The problem.** Using 6 cores must not change one printed byte (DONE gate 6: 1 and 6 workers give equal hashes). In TypeScript terms: `Promise.all` writing each result into `results[i]`, then one loop reducing them in index order, never "whoever finishes first".

**Picks** (R3):

| id | Decision | Pick | Cost |
|---|---|---|---|
| D3-1 | The tool | Keep R-42 (std scoped threads, no rayon), amended: spawn only through `Builder::spawn_scoped` with a stack size passed in (std otherwise reads `RUST_MIN_STACK` and gives 2 MiB); the 1-worker run goes through the same module; "path order" becomes "task-index order"; channels, locks and atomics banned outside the module; `Scope::spawn` banned | ~40 lines, no dependency |
| D3-2 | What runs in parallel first | Monte Carlo paths plus independent timelines (axis combinations, `windows each`, rolling origins, scenarios); joint members and parallel asks later | reaches A's 64× and X's 20× repetition at up to 6× |
| D3-3 | The merge rule | refcore's: per line, terms in order; the first refusing path wins, else the union of holes; quantiles sorted stably by (value, path). Logged as a spec observation (SO8-3) | one scan; gate 10 holds |
| D3-4 | The shared layer | Run the first task alone, freeze the shared cells, then the rest; a counter of cells recomputed locally | no locks; duplication measured |
| D3-5 | Memory per worker | The worker count is an argument; the CLI defaults to 6; a flag lowers it; never read from the machine | none |
| D3-6 | Testing gate 6 | Worker sweep {1, 2, 3, 5, 6, 7}, a test-only reversed completion order, planted violations, a deep-stack case | ~6× the case runs in CI |
| D3-7 | Standard error of a quantile at size | Refined intervals to the printed 1e-6 rounding (the spec allows any method); refcore's exact method as the oracle at small N | a careful numeric kernel on gate 8's list; refcore's method is out of reach at 10 000 paths |

**Readability:** one small thread module; everything else single-threaded. **Speed:** up to 6× on paths and repeated timelines.

**The user's answer:** "I approve" (2026-10-30, 22:08): D3-1 to D3-7.

---

## 5. The performance gates

**The problem.** Performance must be checked from the first milestone by checks the builder can't edit or game. Wall-clock timing is like a flaky Jest timing test: on shared CI it gives about 45 % false alarms at a 2 % threshold. Counts (cells computed, days walked, instructions executed) give the same number every run.

**Picks** (R4):

| id | Decision | Pick | Cost |
|---|---|---|---|
| D4-1 | The instruments | Three: the engine's own counts (shape.md's units); instruction counts under Valgrind (gungraun) on Linux CI; wall time and peak memory on the Mac through a lead-written `engine/tools/perf`. criterion only for micro rows | a perf runner, a profile summariser, gungraun benches |
| D4-2 | What blocks a merge | Counts and instruction hard limits block every PR on Linux; wall time and memory block only the milestone tag, run by the gate session on the reference machine | hard limits re-baselined at each toolchain bump (rules role) |
| D4-3 | The reference machine (SO-27) | The user's MacBook Pro M2 Pro (32 GB, macOS 26): on AC power, idle, never during fuzzing; 2 warm-ups then the median of 10; a fingerprint in every report | the Mac is busy during gate runs |
| D4-4 | The release profile | Keep the scaffold's (fat LTO, one codegen unit, abort, overflow checks). Measure overflow checks once at S; above 3 % inside num-bigint, the user decides an override for that crate only. A `profiling` profile for the builder. PGO only at M9, and no budget may need it | small |
| D4-5 | Profiling | samply plus a summariser on the Mac (no sudo, no Xcode); `perf` and `callgrind_annotate` on Linux; the engine's counters first | samply's last release is 2025-02 (an unknown on macOS 26) |
| D4-6 | PERFORMANCE §8 for 2.1 | Four groups: the old interactive targets kept; the six shape.md checks at ≤ 1 s and ≤ 512 MB each, with no more def-cell misses than refcore; the 18 load rows at ≤ 4 GB, non-Monte-Carlo ≤ 10 s, Monte Carlo at N-2003's rate; micro rows ratcheted from S. Every number "proposed" until measured at S | a too-tight number later needs a logged spec change |
| D4-7 | Memory budgets | 8 GB a hard ceiling on any run; ≤ 4 GB per load row (6 jobs fit 32 GB); ≤ 512 MB per 1× check | none |
| D4-8 | Counters | A cargo feature `counters`, off in release (like R-51's fault injection), printing shape.md's units behind `--stats` | the counted build differs from the timed one; outputs stay byte-identical |

Also: DONE gate 12's evidence column says "criterion reports" and cites the old benchmarks path (SO8-9). That is a DONE change for the user, folded into item 9's approvals row.

**Readability:** one-line results the owner can read. **Speed:** none (counters off in release).

**The user's answer:** "I approve" (2026-10-30, 22:08): D4-1 to D4-8, the reference machine as proposed.

---

## 6. The readability rules

**The problem.** The owner can't read Rust, so they audit the engine through a map, file headers and a trace to the spec, not through the code. The rules here work like an ESLint config: some are errors, some only reports, and each has a tool or a reviewer behind it.

**Picks** (R5), approvable group by group like the rulebook:

| id | Group | Pick | Enforced by | Cost |
|---|---|---|---|---|
| D5-1 | Map and names | `engine/ARCHITECTURE.md` (~200 lines, written at S, updated at milestones); every `.rs` file starts with a `//!` header: purpose, `Spec:`, `Twin:` (the refcore file), `Invariants:`; one directory per pipeline stage; the spec's words as names | a script (error) and review | ~40-line script |
| D5-2 | The Rust subset | no `dyn` (error); no own generics, no single-implementation traits (reports); no `impl Trait` arguments (clippy error); no library macros (R-73 rises to an error); iterator chains of about 4 steps at most | ast-grep and clippy, each with a selftest case | some duplicated code, reported |
| D5-3, K8-10 | Size limits | nesting an **error at 5** (revises K-21); length (100) and cognitive complexity (25) stay reports; one call or expression per dispatch arm; files over 1 000 lines reported; thresholds re-judged at S | clippy.toml through R-112 | the library's exception budget is 0, so the threshold must fit without exceptions |
| D5-4 | The spec trace | a `trace` script: an unknown section or line is an error; a missing citation is an error in the kernel (R-75 rises) and a report elsewhere; the reverse map section → code → refcore → tests | the script | ~150 lines |
| D5-5 | The owner's kit | a generated one-page milestone report (modules added, spec coverage delta, exceptions, size and benchmark deltas) plus a readability section in BUILDER §12's review checklist | the gate session | reuses existing tools |
| D5-6 | Fencing performance code | optimised code only in named fence modules, each with a `PERF:` header, a benchmark, a plain twin and an equivalence test; expected fences: the walk, the memo, the read index, the number tiers, threads | review, the trace | a twin per fence where cheap |
| D5-7 | Readable strictness | newtypes with checked methods (`day.next()?`), named helpers in one kernel module, iteration instead of indices; no arithmetic macros or extension traits | review | none at speed (newtypes compile away) |

**Readability:** the point of the item. **Speed:** none lost (Readyset: bounds checks "negligible"); the fences hold the fast code.

**The user's answer:** asked in three groups. Group A (D5-1, D5-4, D5-5): "I approve". Group B (D5-2, D5-3 with K8-10): "I approve". Group C (D5-6, D5-7): "I approve" (2026-10-30, 22:1x–22:10).

---

## 7. The skill (the user's question)

**The question, plainly:** should the build use an existing Rust skill, or should we create one before implementation?

**What you decided on 2026-10-22 (the default):** an `engine-rust` skill comes after phase 1, built from the approved `RULEBOOK.md`, `BUILDER.md` and the exceptions policy. It lives at `.claude/skills/engine-rust/` (local, not toolkit-synced) with a short `engine/CLAUDE.md` pointing to it. The build guard protects it, it gets a small eval (a planted lint failure: does the agent fix the code rather than add an `allow`?), and it is ready before T's Rust runner and S.

**What the research adds** (R6):
- **Nothing exists to adopt.** Anthropic publishes no Rust skill; its marketplace has only a `rust-analyzer-lsp` plugin. The `skills` registry lists ~100 Rust skills. The best five each recommend something our rules ban (`anyhow`, `unwrap_or_default`, fast hashers), and one injects "MANDATORY" text on every prompt and runs unpinned downloaded code.
- **A skill and CLAUDE.md are advice, not enforcement** (Claude Code's docs). A nested `engine/CLAUDE.md` isn't loaded when a session starts at the repo root, or after compaction, and only a skill's first 5 000 tokens survive compaction.
- **The guard has a gap:** the skill and `engine/CLAUDE.md` aren't checksummed.

**Picks:**

| id | Decision | Pick | Cost |
|---|---|---|---|
| D6-1 | Which skill | **Create `engine-rust`** (your default), borrowing ideas only: a rule-per-entry Bad/Good format, "measure first, name the hot paths", bug-shape checklists for the reviewer, plain-words rules. It also carries the new performance and readability content (items 1–6) as on-demand references | about one session from existing texts |
| D6-2 | Where the never-miss rules live | In the lints, the guard and the Stop hook, as now. A build-role SessionStart hook prints `engine/CLAUDE.md` at start and after compaction; the skill is invoked by name at the start of each build session | a 5-line hook through the rules role |
| D6-3 | Protecting them | Add the skill and `engine/CLAUDE.md` to the guard's rule globs and `RULES.sha256`, changed only by the rules role (R-112) | two globs, a selftest case |
| D6-4 | The eval | 9 planted cases from BUILDER §8's workaround table (two must end in a STOP-REPORT), graded by scripts, with and without the skill, 3 runs each, in Part B. Your bar, proposed: no banned form in any run, and STOP-REPORTs in at least 5 of 6 runs of the two stop cases | 54 agent runs in Part B |
| D6-5 | Helpers | a read-only `engine-reviewer` subagent that loads the skill (BUILDER §12's checklist); the `rust-analyzer-lsp` plugin installed in Part B as a speed aid | small |

**Readability:** the builder gets one source of rules in the rulebook's words. **Speed:** none at run time.

**The user's answer:** "I approve" (2026-10-30, 22:10): D6-1 to D6-5, the eval's bar as proposed.

---

## 8. The rulebook's 2.1 rows

**The problem.** Phase 1's 113 rules were written for tally 1.0.1. Tally 2.1 adds parts and labels, joint members as contexts, the `parties` kind and observations, and the research found holes in a few rules. Like adding ESLint rules for a framework's new features.

**New rows** (R6 §1.4):

| id | The rule | Pick |
|---|---|---|
| R-114 | the largest-remainder split written once, used by every split; ties only by the written `ties by`, else `RUN-TIE` | keep |
| R-115 ⚖ | after every arrival, take and relabel, parts sum to the balance, are ≥ 0 and whole steps; a failure refuses as an invariant. Other side: ENGINE §9 doesn't list it (SO8-7) | keep, checked at run time (O(labels) per booking: cheap) |
| R-116 | holes and hole parts built only in one module, with one per-operator rule for holes (seam U) | keep |
| R-117 | a path statistic's accumulator is its own type, outside the cap check (2.1-15; replaces BUILDER §2's "after every operation") | keep |
| R-118 | every derived order on the engine's types reported for review (R-47) | keep |

**Amended rows:**

| Row | Change | From |
|---|---|---|
| R-16 | joint-model nesting walked as a worklist (no limit or code exists) | R6 |
| R-40 | sort the full relative paths by code point (ENGINE l.118), not each directory's names | R6 |
| R-47 | explicit order keys also for labels in ties, `same` classes and member prefixes | R6 |
| R-76, R-58 | the code count recounted with `tools/xref_check.py` (87 is 1.0.1's) | R6 |
| R-37 | "except under indexmap" | R2 |
| R-42 | R3's five amendments (item 4) | R3 |
| R-73, R-75, R-74 | macros and kernel citations become errors; nesting an error at 5 (item 6) | R5 |
| R-97 | wall-time benchmarks on the reference Mac, not `ubuntu-latest` (item 5) | R4 |

**Readability and speed:** rules the builder meets anyway; R-115 costs O(labels) per booking.

**The user's answer:** "I approve" (2026-10-30, 22:10): R-114 to R-118 (R-115 checked at run time), the amended rows.

---

## 9. Landing the stale documents

**The problem.** `PERFORMANCE.md`, `BOUNDS.md` and `BUILD-ORDER.md` are tally 1.0.1's, frozen as stale, and Part B rewrites them for 2.1. They sit inside the frozen spec: they are listed in `engine/spec/SHA256SUMS` and named stale by DONE rule 1. Changing them is like editing docs inside a tagged release.

**Pick (proposed):**
- rewrite them in place;
- update `SHA256SUMS` and `STALE.md`;
- add a DONE.md approvals row, which also takes gate 12's evidence wording and its benchmarks path (SO8-9);
- `VERSION` stays `tally-2.1`, since no language meaning changes.

**Option:** a version bump (`tally 2.1.1`) instead, which re-runs every gate (DONE rule 1). Not recommended: these documents carry no language meaning.

**The user's answer:** "I approve" (2026-10-30, 22:11): rewritten in place, `SHA256SUMS`, `STALE.md`, one DONE approvals row (with gate 12's wording), `VERSION` stays `tally-2.1`.
