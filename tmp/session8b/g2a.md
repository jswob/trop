# Gate G2a: session 8b's documents, skill and open questions

**Made:** 2026-10-30, 23:15, by the 8b lead. The drafts are in `tmp/session8b/drafts/`. One consistency read was done (`reader.md`: 22 findings, 4 must, no G1 pick contradicted), and the lead fixed 21 of them (`reader-fixes.md`). The user is asked one item at a time; their words are recorded under each item.

Every open question below sits under the document it belongs to, with the lead's pick. Four are notes only (§6).

---

## 1. PERFORMANCE and BOUNDS (writer W1)

**What changed** (`drafts/PERFORMANCE.md`, 74 KB; 1.0.1's was 26 KB):
1. §1–§5 now hold G1's design: a tree-walk evaluator with counted recursion, the engine's own memo keyed by (item, key values, trimmed context) with contexts as layers, an event-driven walk with monthly checkpoints and later forks, read sets now and incrementality as a seam, paths in parallel with refcore's ordered merge.
2. §2.2 derives what makes a day non-empty (SO8-1): 14 rows, N-1 to N-14, each cited to L or E by line; the reader checked them all. A step-every-day walk is the test oracle: same bytes, more days.
3. §6 numbers (two tiers, Lehmer now, money `I64 | Big`), §7 costs with 2.1's parts, labels, observations, joint members and projections, §9 caps (path statistics and joint nesting uncapped), §10 21 unknowns, each with the milestone that measures it.
4. §8 has 36 budget rows with fixed ids: P8-1..9 interactive (kept from 1.0.1), P8-10..15 the six heavy checks at ≤ 1 s and ≤ 512 MB, P8-L1..L18 the 17 load rows at ≤ 4 GB (non-Monte-Carlo ≤ 10 s), P8-M1..M4 micro rows set at S. **Every number is proposed until S**, with refcore's figure beside it.
5. BOUNDS (`drafts/BOUNDS.md`, 22 KB): all 53 rows restated in 2.1's constructs and sections; **no status changed** (checked by script).

**Open points, with the lead's pick:**

| # | The point | Pick |
|---|---|---|
| O-1 | The derived list of non-empty days (§2.2) | approve it; the oracle walk catches a missing row |
| O-2, O-W1-1 | Trimming a projection out of a cell's memo context (§3.3) is a cache choice only if the "hole that nothing later answers" a projection gives for a not-yet-known fact is the read rule's own hole | approve the trimming under that reading, checked by refcore's bytes and an on/off test; if a case ever differs, trimming is off there and the reading goes to the next spec patch's gap log |
| O-W1-2 | A load row that mixes plain asks and Monte Carlo asks is budgeted as ≤ 10 s for its plain part plus N-2003's rate (1 s per 1 000 paths × 30 years on 6 workers) for its paths | approve, as a proposal until S |
| O-W1-4 | P8-13's and P8-14's def-miss budgets use refcore's partial counts (its runs stopped at 88 s), which is stricter than refcore's true figures | keep them, marked partial; S re-judges with the engine's own counts (a full refcore run would break the 90 s rule) |
| O-W1-5 | A BOUNDS row 54 for 2.1's labels ("ownership by origin", designed, untested), as 1.0.1 added rows 50–53 for the abilities its design added | add it |
| O-W1-6 | Interactive rows' memory ≤ 1 GB (R4's table; D4-7 named only 8 GB, 4 GB and 512 MB) | approve, as a proposal until S |

## 2. BUILD-ORDER (writer W2)

**What changed** (`drafts/BUILD-ORDER.md`, 48 KB; 1.0.1's was 21 KB):
1. S and M0–M9 kept, in 2.1's constructs; the tracer slice keeps the amendments household (X). M5 splits into M5a (fill, many parties), **M5b parts and labels**, **M5c joint members as contexts** (D1-7's two steps).
2. **All 634 checks of `checks.tsv` placed exactly once** (a script checks it): S 4, M0 10, M1 90, M2 26, M3 0, M4 77, M5a 166, M5b 79, M5c 40, M6 36, M7 21, M8 0, M9 85 (the load rows' not-run checks).
3. Every milestone names its checks, its DONE gates and the P8 rows it touches. The kernel design (evaluator, memo, walk, numbers, counters, map, trace) lands at S; forks at the end of M2; the thread module and parallel paths at M7; PGO only at M9.
4. M3 (the operations) says what ENGINE alone gives and what waits for `OPERATIONS.md`, which stays stale.
5. What T still writes (goldens, hindcast, the codes' fixtures, properties, the generator, the count-budget check) is placed where 1.0.1 placed it.

**Open points, with the lead's pick:**

| # | The point | Pick |
|---|---|---|
| O-W2-1 | No session plans `OPERATIONS.md`'s rewrite; M3, gate 5's OP codes, P8-1 (a turn) and P8-7 (the law gate) wait on it | the master schedules it as a paper session before M3; it doesn't block S–M2 |
| O-W2-4 | A household's checks wait for its heaviest construct (A for parts, B for the fill, C and D for law, E1 and E2 for joint members), a convention rather than a per-ask analysis | accept the convention; a check that passes earlier is a signal only, never a tag's condition |
| F-6 | The placement table and its check script can't land in `engine/spec/` (only six files may change) | they stay in `tmp/session8b/` (committed), cited by path; T carries them into `engine/tests/` when it wires the runner |

## 3. The rulebook and BUILDER (writer W3)

**What changed:**
1. RULEBOOK: **131 rules** (113 before) in 8 groups; 115 keep (about 99 by a tool), 9 report, 7 dropped. New rows R-114..R-118 for 2.1 (the split written once, parts sum to the balance at run time, holes in one module, the path statistic's accumulator, derived orders reported).
2. A new group 8, readability, R-119..R-131 (the map, the `//!` header, names, no `dyn`, no `impl Trait` arguments, generics and long chains reported, the trace, the milestone page, fences, newtypes).
3. Twelve rows amended, each keeping its old text and recording "I approve" (2026-10-30) with its G1 item: R-16, R-37, R-40, R-42, R-47, R-58, R-73, R-74, R-75, R-76, R-97, and **R-80** (the skill and `engine/CLAUDE.md` checksummed, D6-3: the reader found no row said it, so the lead wrote G1 item 7's decision into it).
4. BUILDER: §2–§4 and §7 follow G1 (each reversed phase 1 sentence marked *Changed* with its K8 id); §8 gains a table for the ten new bans; §9 the per-PR count budgets and the milestone budget step; §11 the SessionStart hook; §12 the readability checklist.
5. A new BUILDER §13, performance: fences, counters, instruments and what blocks, profiles, profiling recipes.

**Open points, with the lead's pick:**

| # | The point | Pick |
|---|---|---|
| O-W3-1 | Three readability reports R5 picked have no planned tool: explicit lifetimes, long iterator chains, `#[inline(always)]`/`#[cold]` outside a fence | 8c adds them as ast-grep reports (never failing the build), each with its planted case |
| O-W3-2 | No rulebook row keeps the `counters` feature and gate 6's reversed-order test switch out of the shipped binary (G1 said "like R-51's fault injection") | 8c amends R-51 through R-112 to cover both, with R-51's checks (the release binary built alone, a CI probe) |

## 4. The skill, the reviewer and `engine/CLAUDE.md`'s text

**What changed** (all drafts, landed only after this gate):
1. `.claude/skills/engine-rust/SKILL.md` (about 3 900–4 300 tokens, under the 5 000 that survive compaction): the four promises, 13 hard rules each with its check, what to do when a check blocks, a 30-row "write this instead" table, the loop, the STOP-REPORT, performance in one paragraph.
2. Seven references read on demand: numbers, data layout, the evaluator and the memo, parallelism, performance, readability, the seams.
3. Paper checks pass: every cited R-n exists; all 23 rows of BUILDER §8 appear; skill-creator's `quick_validate.py` passes (run with a small stand-in for its YAML library, since none is installed). Ideas borrowed only: Bad/Good pairs, "measure first", a bug-shape list for the reviewer.
4. `.claude/agents/engine-reviewer.md`: read-only (Read, Grep, Glob, read-only Bash), loads the skill, BUILDER §12's checklist with its readability section, a list of bug shapes; its final message is the review.
5. `drafts/engine-CLAUDE.md` (100 lines) for 8c to place behind the SessionStart hook, and `eval-design.md` (E1–E9, script graders, the two arms, staging, your bar) for 8c to build.

**Open point and two notes:**

| # | The point | Pick |
|---|---|---|
| O-4 | Staging the eval (one run per arm first) may leave the two stop cases E8 and E9 at one run per arm, too few to judge your bar "5 of 6" | E8 and E9 always get 3 runs per arm; the other seven stay staged (26 runs at least, about 0.7–1.4 %) |
| note | The skill has no `paths:` trigger in its frontmatter: skill-creator's validator allows only its listed keys, and G1's D6-2 invokes it by name anyway | — |
| note | The reviewer's `skills:` field format is checked against Claude Code's docs by 8c when it lands the hook | — |

## 5. Landing (after your go)

The approved drafts go to `engine/spec/` (PERFORMANCE, BOUNDS, BUILD-ORDER) with `STALE.md`, one DONE approvals row (with gate 12's evidence wording and its benchmarks path, SO8-9) and `SHA256SUMS`; `engine/README.md`'s stale line; RULEBOOK and BUILDER over their `.before-8b` copies; the skill and the reviewer into `.claude/`. `VERSION` stays `tally-2.1`. Three local commits on `engine`, never pushed. The session's open ids in the landed text (O-n) are reworded to your answers.

## 6. Notes, no question

- **O-3:** G1 said "18 load rows"; `load.tsv` has 17 (L:1–L:18, no L:7). §8 budgets the 17.
- **O-W1-3:** the path-walk years of three Monte Carlo load rows aren't written anywhere; T fixes them from each recipe, and §8.3 gives the formula.
- **O-W2-2:** C-12's test-map tag (a rule over paths) may be wrong; a question for T's map, not the spec. Placed at M7, the safe side.
- **O-W2-3:** whether T's generator can draw only from the constructs built so far is T's design.
- **Sizes:** PERFORMANCE and BUILD-ORDER are about twice their outline's estimate: the derived list, four budget groups and 21 unknowns in one; the per-milestone check lists and a glob appendix in the other. Trimming would cost a writer round and drop required rows.

---

## The user's answers

(recorded as given, with the time)

### Item 1: PERFORMANCE and BOUNDS (asked 23:18)

Asked: approve item 1 with the picks (the derived non-empty-day list; the memo trimming checked byte for byte against refcore; the mixed Monte Carlo split, the partial-count budgets and the 1 GB interactive limit as proposals; BOUNDS row 54 for 2.1's labels).

**The user's answer:** "yes" (2026-10-30, 23:18): item 1 approved with every pick: O-1, O-2/O-W1-1, O-W1-2, O-W1-4, O-W1-5 (add row 54), O-W1-6; the P8 numbers proposed until S.

### Item 2: BUILD-ORDER (asked 23:18)

Asked: approve item 2 with the picks (O-W2-1: the master schedules OPERATIONS.md's rewrite as a paper session before M3, not blocking S–M2; O-W2-4: the household floors, an earlier pass a signal only; F-6: the placement table and script stay in tmp/session8b/, cited by path, T carries them into engine/tests/).

**The user's answer:** "yes" (2026-10-30, 23:18): item 2 approved with every pick.

### Item 3: the rulebook and BUILDER (asked 23:19)

Asked: approve item 3 with the picks (the 131 rules, the twelve amended rows including R-80 for D6-3, BUILDER with §13; O-W3-1: 8c adds the three ast-grep reports with planted cases; O-W3-2: 8c amends R-51 through R-112 to cover `counters` and gate 6's reversed-order switch).

**The user's answer:** "yes" (2026-10-30, 23:19): item 3 approved with every pick.

### Item 4: the skill, the reviewer and engine/CLAUDE.md's text (asked 23:19)

Asked: approve item 4 with the pick (O-4: E8 and E9 always 3 runs per arm, the other seven staged; 26 runs at least, about 0.7–1.4 %).

**The user's answer:** "yes" (2026-10-30, 23:19): item 4 approved with the pick.

### Item 5: the open list and the go to land (asked 23:20)

Asked: every open question was decided within items 1–4; the four notes (§6) need no decision; may the lead land (the three documents into engine/spec/ with STALE.md, one DONE approvals row and SHA256SUMS, VERSION unchanged; RULEBOOK and BUILDER beside their .before-8b copies; the skill and the reviewer into .claude/; three local commits on engine, never pushed)?

**The user's answer:** "yes" (2026-10-30, 23:20): the go to land.
