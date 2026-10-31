# The reader: one consistency read of session 8b's drafts

Read `tmp/session8b/agents/common.md` first (the session brief's §1–§2, the agent rules, the facts). You write **one file, `tmp/session8b/reader.md`: findings only**, no fixes. You change no draft.

## The drafts

In `tmp/session8b/drafts/`: `PERFORMANCE.md`, `BOUNDS.md` (W1); `BUILD-ORDER.md`, `BUILD-ORDER-placement.tsv` (W2); `RULEBOOK.md`, `BUILDER.md` (W3; W3's change list is `tmp/session8b/agents/W3/changes.md`). Their plan is `tmp/session8b/outlines.md`; the questions the writers logged are in `tmp/session8b/open.md` (a point already logged there is not a finding, unless the draft contradicts it).

## The three comparisons

1. **Against G1.** Each draft against `tmp/session8/decisions.md` (items 1–9, every D-id) and `tmp/research_notes/engine-prep/NOTE.md` §2 (K8-1..K8-17): every pick the outline assigns to a draft is in it; nothing contradicts a pick (e.g. a hash map in the memo, an i128 tier, recursion "never", wall time on hosted CI, a budget not marked "proposed until S").
2. **Against each other.**
   - Every P8 id BUILD-ORDER cites exists in PERFORMANCE §8, and every PERFORMANCE §8 id is in outline §C (none renumbered, none dropped).
   - Every check of `engine/tests/checks.tsv` is placed once: run `python3 -I -B tmp/session8b/agents/W2/placement_check.py` and quote its last lines; spot-check 10 placements against their check's `question` and `note` columns.
   - RULEBOOK and BUILDER agree: every R-n BUILDER cites exists in the draft RULEBOOK with the same meaning; every new or amended row of outline §E is in RULEBOOK with its enforcement and the user's answer ("I approve", 2026-10-30, its G1 item); the summary's counts match the rows (count them with a script).
   - PERFORMANCE and BUILDER agree on numbers, threads, collections, counters and what blocks a merge.
3. **Against the spec: no new meaning.** For every sentence in PERFORMANCE, BOUNDS and BUILD-ORDER that states what a construct does (not how the engine computes or schedules it), check it against the L or E section it cites (or should cite). A finding is a sentence that says more than L or E says, or contradicts it. Read L and E by the cited sections only (`engine/spec/LANGUAGE.md`, 356 KB, and `ENGINE.md`, 61 KB: never whole). Check especially PERFORMANCE §2's list of what makes a day non-empty (each item's citation), §3's trimming, §5's merge rule, §9's caps and codes.

## Format of `reader.md`

A header (date, what was read). Then one table per comparison: `#` (F-1, F-2, …), draft and section (with line numbers), the finding in one or two sentences, the evidence (the quoted line and its source line), severity (**must**: contradicts G1 or the spec, or a broken cross-reference; **should**: missing, unclear or inconsistent; **note**: style). Then a short tally per draft. No recommendations beyond one line per finding if the fix is obvious.

## Scope

Aim for about 60–90 tool calls. Read the drafts whole (they are about 20–60 KB each); read everything else by sections. Your final message (about 10 lines): the file's path and size, the count of findings by severity and draft, and the three that matter most.
