# W1: `PERFORMANCE.md` and `BOUNDS.md` for tally 2.1

Read `tmp/session8b/agents/common.md` first (the session brief's §1–§2, the agent rules, the facts). Then `tmp/session8b/outlines.md` whole: its §A, §B and §C are your plan, and §C's row ids are fixed (W2 cites them in BUILD-ORDER while you write; never renumber, never drop one: a row you can't budget stays, marked open).

## What you write

- `tmp/session8b/drafts/PERFORMANCE.md`: the full document for tally 2.1, sections §1–§10 as outline §A lists, §8's rows as outline §C fixes. Expect about 30–40 KB.
- `tmp/session8b/drafts/BOUNDS.md`: the full document for tally 2.1, as outline §B scopes it. Keep its rows' numbers (1–53) and statuses; restate their *Tally* column and notes in 2.1's constructs and sections.
- Questions: append to `tmp/session8b/open.md` under a heading `## W1`, ids O-W1-1, O-W1-2, …
- Scratch and `progress.md`: `tmp/session8b/agents/W1/`.

## What each section must do (beyond the outline)

- **§2's list of what makes a day non-empty (SO8-1, open item O-1).** Derive it from L and E, item by item, each with its citation (section and line). For each, say which construct makes the day carry work, and what the walk does that day. Then name the step-every-day walk as the oracle: the same output bytes, more days stepped; a test runs both. If an item can't be derived from the text, it is an O-W1 question, not a sentence.
- **§3's trimming (SO8-2, open item O-2).** Derive what L and E allow: a trimming that can change no printed byte is a cache choice; say why, from the read rule (L §4.6), cells and the memo (E §3.2) and projections (E §4.3). If a trimming could change an output, it is an O-W1 question.
- **§8.** One table per group (I, C, L, M), columns: id, what, size, refcore's figure (the old "Floor": an upper reference, not a target), wall budget, footprint, count budget, where and when measured, what blocks a merge or a tag. Every number "proposed until S". Groups C and L show their arithmetic (e.g. the Monte Carlo rate for L:1's 10 000 paths, L:5's 3 000, L:11's 1 000; the cut-down twin's counts for a load row). P8-5 and P8-M4: restate the look-ahead in 2.1's terms (the written projection, D-6), or, if 2.1 has no counterpart, keep the row marked open with an O-W1 question. P8-1 and P8-7 lean on `OPERATIONS.md`, which stays stale: say so in the row.
- **§9.** Check each cap against L's and E's codes (L §8.3–§8.4, E §11) and numbers as 2.1 states them; add the two uncapped rows and the 8 GB ceiling.
- **§10.** One row per NOTE §4 unknown that a number of yours rests on: what it is, the section it moves, and the milestone that measures it (S unless BUILD-ORDER would say later).
- **Every estimate** shows its arithmetic in a `python3 -c` block, as 1.0.1's did (run it, paste its output).
- **No technology names** in either document (1.0.1's convention): no crate, no Rust API. Cite BUILDER and RULEBOOK rows instead (e.g. "the thread module, R-42"; "the big tier's library, R-30"). D4-4 and D4-5 (release profile, profiling) are BUILDER's; §8 says only where a row runs.

## Inputs (read in this order)

1. The current documents, **whole**: `engine/spec/PERFORMANCE.md` (26 KB), `engine/spec/BOUNDS.md` (18 KB); `engine/spec/STALE.md` lines 1–6 and 26–55 (their stale rows); `engine/spec/DONE.md` (gates 6, 7, 8, 12).
2. Session 8: `tmp/session8/decisions.md` (items 1–5, 8, 9), `tmp/research_notes/engine-prep/NOTE.md` (whole, 19 KB), `tmp/session8/shape.md` (whole).
3. The reports, by their Summary and the sections a section of yours carries: R1 (the walk, the memo, memory: §2, §3, its decisions), R2 (numbers, layout: its Q on numbers, memory per cell, D2-1..D2-7), R3 (paths and the merge: D3-1..D3-7, its spec observations), R4 (Q6 and Q7: the §8 rows and memory). All in `tmp/research_notes/engine-prep/`.
4. The size targets: `engine/tests/load/load.tsv` (17 rows) and `engine/tests/load/benchmarks.md`.
5. The spec, **by ranges** (line numbers as of today):
   - `engine/spec/ENGINE.md` §1–§9 (lines 30–435) and §11 (from line 470: the codes you cite);
   - `engine/spec/LANGUAGE.md` §1 (21–94), §3.1 (361–392), §3.5–§3.6 (452–540: parties, places and parts), §4.6 (708–745: the read rule), §5.7 (896–960: kinds and their parts clauses), §5.10 (1055–1103), §5.14 (1318–1455: move, the walk's strata and decisions), §5.16 (1473–1574: distributions and paths), §5.19 (1665–1770: observations), §6.1 (1775–1808: joint models), §6.4 (1848–1883: scenarios), §7 (2090–2197: numbers, the cap, determinism), §8.3–§8.4 (2263–2369: the codes), by grep for the rest;
   - `engine/spec/CHANGES.md` lines 1–40 (2.0 and 2.1's change rows; 2.1-15 is the cap);
   - `engine/BUILD-NOTES.md` (whole, 5 KB).

## Done when

Both drafts are complete; every outline row of §A–§C is carried or marked open with an O-W1 id; every estimate shows its arithmetic; your final message (common.md §C) lists what you wrote and your open items.
