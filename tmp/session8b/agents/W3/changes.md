# W3: every change in the two drafts, one line each, with its source

Drafts: `tmp/session8b/drafts/RULEBOOK.md` (full copy of `tmp/research_notes/engine-rust-strictness/RULEBOOK.md`), `tmp/session8b/drafts/BUILDER.md` (full copy of `BUILDER.md` there). Every new or amended rulebook row records "I approve" (2026-10-30) with its G1 item.

## RULEBOOK

| Where | Change | Source |
|---|---|---|
| header | "Updated" paragraph: 2026-10-30, session 8b, the rows added and amended, the `.before-8b` copy | decisions.md items 1, 4, 5, 6, 8 |
| Summary item 1 | 113 → 131 rules in 8 groups; 115 keep (about 99 tool, 16 review), 9 report, 7 drop; a per-group table (counted by `agents/W3/count_rows.py`) | the rows below |
| Summary item 8 | ⚖ list adds R-115 | R6 D6-6, SO8-7 |
| Summary items 11, 12 | new: tally 2.1's rows; the readability group | decisions.md items 6, 8 |
| How to read a row | two bullets: how an amended row and a new row are marked | W3 brief ("old text visible") |
| R-16 (group 1) | amended: counted recursion with a work stack and replay past the count; every thread with an explicit stack; joint nesting as a worklist; enforcement adds the deep-stack case at 1 and 6 workers | K8-1, D1-1, R1 Q6; R6 §1.4; SO8-8; D3-6; G1 items 1, 8 |
| group 2 intro | one sentence: the group now also covers parts, holes and path statistics | R-114..R-117 |
| R-114 (group 2) | new: the largest-remainder split written once; ties only by `ties by`, else `RUN-TIE` | R6 §1.4; L §3.6 l.476, l.484, §5.13 l.1309, l.1312; G1 item 8 |
| R-115 ⚖ (group 2) | new: parts sum to the balance, ≥ 0, whole steps after every arrival, take and relabel; `ENGINE-INVARIANT`; checked at run time; other side SO8-7 | R6 §1.4, D6-6; L §3.6 l.472; E l.482; G1 item 8 |
| R-116 (group 2) | new: holes and hole parts built in one module, one per-operator rule (seam U) | R6 §1.4; BUILD-NOTES U; G1 item 8 |
| R-117 (group 2) | new: a path statistic's accumulator its own type, outside the cap | R6 §1.4; L l.2103 (2.1-15); K8-5, K8-17; G1 item 8 |
| R-37 (group 3) | amended: "except hashbrown under indexmap" | K8-7; R2 §Contradictions; G1 item 8 |
| R-40 (group 3) | amended: full relative paths sorted by code point, not per directory | K8-12; R6 §1.4; E l.118; G1 item 8 |
| R-42 (group 3) | amended with R3's five points (a)–(e); enforcement adds the sync-type bans and gate 6's tests | K8-8; R3 Q2 verdict, Q7; D3-1, D3-6; G1 items 4, 8 |
| R-47 (group 3) | amended: explicit keys for labels in ties, `same` classes, member prefixes; interned symbols by text | R6 §1.4; D2-3, R2 Q3; G1 item 8 |
| R-118 (group 3) | new: every derived order reported (report) | R6 §1.4; G1 item 8 |
| R-58 (group 4) | amended: "87" is 1.0.1's; recount at 8c with `tools/xref_check.py` | K8-13; G1 item 8 |
| R-73 (group 5) | amended: raised to an ast-grep error outside the lead's list; Rec. report → keep | K8-11; R5 RS-6; G1 items 6 (group B), 8 |
| R-74 (group 5) | amended: nesting an error at 5; length 100 and cognitive complexity 25 stay reports; advisory thresholds aligned; re-judged at S; `too_many_arguments` 7 and `type_complexity` 250 recorded as already errors; Rec. report → keep | K8-10; R5 D5-3, Q3; phase 1 K-21; G1 items 6 (group B), 8 |
| R-75 (group 5) | amended: a missing citation an error in the kernel, a report elsewhere, by the `trace` script; Rec. report → keep | K8-11; R5 D5-4; G1 items 6 (group A), 8 |
| R-76 (group 5) | amended: the count (91 in 2.0, four more in 2.1, about 95–98 by R6's regex); recount at 8c with `tools/xref_check.py` | K8-13; R6 §1.4; G1 item 8 |
| R-97 (group 6) | amended: wall time and footprint on the reference Mac (nightly report, the tag blocks); counts and gungraun hard limits block every PR on Linux; criterion for micro rows only | K8-9; R4 D4-1, D4-2, D4-3; G1 items 5, 8 |
| group 8 (new) | intro, glosses (trait, `dyn`, generic, `impl Trait`, lifetime, newtype, `//!`), and the note that RS-6, D5-3's nesting and D5-4's kernel citations are R-73, R-74, R-75 | decisions.md item 6; R5 |
| R-119 | new: `ARCHITECTURE.md` (review) | D5-1, RD-1; G1 item 6 group A |
| R-120 | new: the `//!` header (script, error) | D5-1, RD-2; G1 item 6 group A |
| R-121 | new: layout and names (review) | D5-1, RD-3, RD-4; G1 item 6 group A |
| R-122 | new: no `dyn` (ast-grep, error) | D5-2, RS-1; G1 item 6 group B |
| R-123 | new: no `impl Trait` arguments (clippy, error) | D5-2, RS-3; G1 item 6 group B |
| R-124 | new: no own generics, no one-implementation traits, lifetimes only on short-lived views (ast-grep report, review) | D5-2, RS-2, RS-4, RS-5; G1 item 6 group B; O-W3-1 |
| R-125 | new: iterator chains about 4 steps, single-expression closures (review, ast-grep report) | D5-2, RS-7; G1 item 6 group B; O-W3-1 |
| R-126 | new: one call or expression per dispatch arm (review) | D5-3; G1 item 6 group B |
| R-127 | new: files over 1 000 code lines reported (report) | D5-3; G1 item 6 group B |
| R-128 | new: the `trace` script (error) with the reverse map | D5-4; G1 item 6 group A |
| R-129 | new: the milestone page and the checklist's readability section (gate session, review) | D5-5; G1 item 6 group A |
| R-130 | new: fences (review, the trace); `#[inline(always)]`/`#[cold]` outside fences listed for review | D5-6; R5 D5-6 (b); G1 item 6 group C; O-W3-1 |
| R-131 | new: readable strictness (review) | D5-7; G1 item 6 group C |

## BUILDER

| Where | Change | Source |
|---|---|---|
| header | "Updated" paragraph; note that the [Tally syntax] line citations are 1.0.1's and were not rechecked | decisions.md; L l.325 |
| §2 title | sources add R2 | — |
| §2 `Num` | two tiers, no i128 tier; the `d == 1` fast path; the 16-byte size test | D2-1; NOTE §1 item 4; R2 D2-1 |
| §2 `Money` | `I64 \| Big`; *Changed* line | D2-2, K8-3 |
| §2 gcd | Lehmer now, tested against num-integer and Python; *Changed* line | D2-1, K8-2 |
| §2 the cap | the exception for a path statistic's accumulation; *Changed* line | K8-5, K8-17, R-117; L l.2103 |
| §2 new bullet | a quantile's standard error at size | D3-7 |
| §3 title | "the parser, the tree and the tables" | — |
| §3 `Table<I, T>` | new bullet | D2-4 |
| §3 interner | replaces "no interner"; *Changed* line | D2-3, K8-4 |
| §3 derived orders | R-118 named | R-118 |
| §3 collections, walk storage, memo | new bullets (pointer to §7; checkpoints `Vec<Rc<PlaceState>>`; the memo's key, scope, no eviction, dense columns, read sets) | D2-5, D2-6, D1-2, D1-3, D1-4, D1-5, D1-6, D2-7 |
| §4 title | sources add R1 | — |
| §4 the shape | new bullet: tree-walk, closures only in a fence | D1-1 |
| §4 recursion | counted recursion replaces "an explicit work stack, not Rust recursion"; joint nesting a worklist; *Changed* line; E §3.2 l.155 replaces 1.0.1's "ENGINE l.141" | K8-1, D1-1, R1 Q6; SO8-8 |
| §4 joint members | new bullet | D1-7 |
| §6 codes | *Changed* line: recount at 8c with `tools/xref_check.py` | K8-13 (R-76) |
| §7 title | sources add R2, R3 | — |
| §7 collections | the D2-5 policy; no hash map now; *Changed* line | D2-5, K8-6 |
| §7 threads | rewritten per R-42 amended: `Builder::spawn_scoped` with a stack size, the 1-worker run, what runs in parallel, task-index merge, the merge rule, the shared layer, the worker count, no sync types, gate 6's tests; *Changed* lines | D3-1 to D3-6, K8-8, K8-16, SO8-3, SO8-5 |
| §7 CLI `main` | full relative paths by code point; *Changed* line | K8-12 (R-40) |
| §8 | a second table, "the bans added on 2026-10-30", four columns (what blocks, the workaround, write instead, stop and report): `dyn`, `impl Trait` arguments, macros, nesting at 5, locks/channels/atomics, `spawn_scoped`, hash maps, extension traits, fences, derived orders | R-122, R-123, R-73, R-74, R-42, R-36/R-37/K8-6, R-131, R-130, R-47/R-118; D6-4 |
| §9 title, table | an "every PR, Linux CI" row (count budgets, gungraun hard limits); nightly adds the Mac's wall and footprint; milestone end adds the milestone report and `tools/perf milestone` on the Mac, a miss blocking the tag | D4-1, D4-2, D4-3; R4 Q5, §5 item 5; R-129 |
| §9 session start | the SessionStart hook prints `engine/CLAUDE.md`; the skill invoked by name | D6-2 |
| §9 STOP-REPORT | adds the trigger "a budget looks wrong" | R4 Q5 (D4-2) |
| §11 | how it reaches the builder: the SessionStart hook (startup, compact), the skill by name, both checksummed and changed only by the rules role; outline items 3, 5, 6, 8, 10 updated | D6-2, D6-3, K8-14; the rows above |
| §12 | the reviewer is `engine-reviewer` loading the skill; item 9, readability (headers, the trace, the subset, nesting and shape, fences, names, strictness) | D6-5, D5-5; R-119..R-131 |
| §13 (new) | performance: 13.1 fences; 13.2 counters and `--stats`; 13.3 instruments and what blocks, the reference machine, memory budgets, where things live, a miss's line, the env-variable gap; 13.4 profiles; 13.5 profiling recipes; tool versions as R4 names them | D5-6; D4-8, K8-15; D4-1, D4-2, D4-3, D4-7; D4-4; D4-5; R4 Q1–Q5, §5 |

## Lead's fixes after the reader (2026-10-30; `tmp/session8b/reader-fixes.md`)

| Where | Change | Source |
|---|---|---|
| R-80 (group 6) | amended: the skill and `engine/CLAUDE.md` in the rule globs and `RULES.sha256`; answer "I approve" (2026-10-30, G1 item 7); header says twelve amended rows | D6-3, K8-14; reader F-2 |
| R-122; BUILDER §8 `dyn` row | a real need is a STOP-REPORT and a rules change (R-112), not an R-108 file | reader F-4 |
| BUILDER §7 | gate 6's switch: reverse completion order only | D3-6; reader F-3 |
| BUILDER §13.2 | `--stats` lists ev and walks shared | D4-8; reader F-10 |
| BUILDER §9, §13.3 | the budget table is the protected one 8c builds (B2.3), read by T's runner | plan B2.3; reader F-11 |
