# W3: `RULEBOOK.md` and `BUILDER.md`, updated for tally 2.1 and G1

Read `tmp/session8b/agents/common.md` first (the session brief's §1–§2, the agent rules, the facts). Then `tmp/session8b/outlines.md` §E: it is your plan.

## What you write

- `tmp/session8b/drafts/RULEBOOK.md`: a **full copy** of `tmp/research_notes/engine-rust-strictness/RULEBOOK.md`, changed as outline §E says. Copy the original first (`cp`), then edit the copy row by row. Never edit the original.
- `tmp/session8b/drafts/BUILDER.md`: a **full copy** of `tmp/research_notes/engine-rust-strictness/BUILDER.md`, changed as outline §E says, the same way.
- `tmp/session8b/agents/W3/changes.md`: a list of every row and section you changed or added, one line each, with its source (D-id, K8-id, R-report section). The lead and the reader check against it.
- Questions: append to `tmp/session8b/open.md` under a heading `## W3`, ids O-W3-1, O-W3-2, …
- Scratch and `progress.md`: `tmp/session8b/agents/W3/`.

## How to change them

- **Keep the rulebook's row format** (its "How to read a row" section): the same columns, the same enforcement words, the same way of recording the user's answer. A new or amended row records the user's G1 answer as decisions.md gives it: "I approve", 2026-10-30, with the G1 item number. An amended row keeps its old text visible where the format allows (e.g. "amended 2026-10-30 (G1 item 8): …"), so the change can be read without a diff.
- **New rows** R-114..R-118 from decisions.md item 8 and R6 §1.4; **amended rows** as outline §E lists, each from its source (decisions.md items 4–8; NOTE §2's K8 rows; R2, R3, R5, R6 by the sections they cite).
- **The readability group:** a new numbered group after the last, rows from R-119 on, one per D5-1..D5-7 (decisions.md item 6), split where one decision holds rules with different enforcement (e.g. D5-2's `dyn` error and its generics report). Cite R5's ids (RD-n, RS-n) for each. Enforcement as decisions.md's "Enforced by" column says.
- **R-58 and R-76's count:** don't recount; write "recount at 8c with `tools/xref_check.py`" (87 was 1.0.1's; R6 found about 95–98).
- **Update the summary** (counts per group, the total, how many are tool-enforced).
- **BUILDER:** the sections outline §E lists. Where G1 reverses a phase 1 sentence (§2 "only if benchmarks ask", §2's i128 money tier, §2's cap "after every operation", §3's "no interner", §4's "never Rust recursion"), replace it and say in one line what changed and why (the K8 id). §8's workaround table gains a row for each new ban, in its format: the lint or rule that blocks, the workaround it tempts (e.g. `#[allow]`, a wrapper trait to dodge `dyn`'s ban), what to write instead, and when to stop and report. The new performance section: fences (D5-6), counters (D4-8), instruments and what blocks (D4-1..D4-3), profiles (D4-4), profiling recipes (D4-5), in plain words with each tool's version as R4 names it.
- **BUILDER §11** (`engine/CLAUDE.md`'s outline): update it to D6-2 and D6-3: printed by the build role's SessionStart hook at start and after compaction, at most 150 lines, the skill `engine-rust` invoked by name at each build session's start, both checksummed in `RULES.sha256` and changed only by the rules role.
- **BUILDER §12** (the review checklist): add the readability section (D5-5: headers, the trace, the subset, nesting, fences), in the checklist's format.
- **Nothing beyond G1.** A rule G1 didn't approve is not added; if you find one is needed, log an O-W3 question.

## Inputs (read in this order)

1. The two originals: `tmp/research_notes/engine-rust-strictness/RULEBOOK.md` (55 KB: its Summary and "How to read a row" whole, then each group as you change it) and `BUILDER.md` (22 KB, **whole**).
2. Session 8: `tmp/session8/decisions.md` (whole), `tmp/research_notes/engine-prep/NOTE.md` (whole, 19 KB), `tmp/session8/plan.md` B1.4.
3. The reports, by Summary and the sections cited: R2 (numbers, layout), R3 (D3-1's five amendments), R4 (D4-1..D4-8, the profiling recipes Q3, the release settings Q4), R5 (D5-1..D5-7, its RD and RS ids, its rule texts), R6 §1.4 (the rulebook against 2.1: R-114..R-118, the amended rows) and its D6-2/D6-3. All in `tmp/research_notes/engine-prep/`.
4. Phase 1's `tmp/research_notes/engine-rust-strictness/NOTE.md` §2 (K-1..K-26), only where a row you amend cites a K id.
5. The spec, only for a row's citation: `engine/spec/ENGINE.md` (around line 118 for R-40; §9 for R-115), `engine/spec/LANGUAGE.md` (§5.12–§5.13 for R-114's split, §7.1 for R-117; by grep).

## Done when

Both full copies are complete and `changes.md` lists every change; every new or amended row has its enforcement and the user's answer with its date; the summary's counts are right; your final message (common.md §C) gives the row counts and your open items.
