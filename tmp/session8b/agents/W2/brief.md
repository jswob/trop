# W2: `BUILD-ORDER.md` for tally 2.1

Read `tmp/session8b/agents/common.md` first (the session brief's §1–§2, the agent rules, the facts). Then `tmp/session8b/outlines.md` whole: its §D is your plan, and §C holds PERFORMANCE §8's fixed row ids, which you cite (W1 writes PERFORMANCE while you write; cite only the ids in §C).

## What you write

- `tmp/session8b/drafts/BUILD-ORDER.md`: the full document for tally 2.1. Expect about 22–30 KB.
- `tmp/session8b/drafts/BUILD-ORDER-placement.tsv`: one line per check of `engine/tests/checks.tsv` (634), columns `id`, `milestone`, `why` (the construct or capability that puts it there, a few words). The document names checks by id or glob; the TSV is the exact placement it summarises.
- `tmp/session8b/agents/W2/placement_check.py`: reads `engine/tests/checks.tsv` and the placement TSV; prints every check placed zero times or more than once, every placement id not in `checks.tsv`, and the count per milestone; exits non-zero on any fault. Run it with `python3 -I -B`; it must exit 0 before you finish. If the document names globs, the script also expands them against `checks.tsv` and checks they agree with the TSV.
- Questions: append to `tmp/session8b/open.md` under a heading `## W2`, ids O-W2-1, O-W2-2, …
- Scratch and `progress.md`: `tmp/session8b/agents/W2/`.

## What the document must do (beyond the outline)

- **Keep 1.0.1's frame:** a tracer slice S, then M0 (load and format) to M9 (performance), each done when its named checks are green and every earlier milestone's stay green. The tracer slice keeps the household (in 2.1 the amendments household, the `I:X:` checks, with its vectors), unless the checks show another household fits a thin path better; then say why.
- **2.1's constructs** in each milestone's "builds" (L's sections cited): kinds and their parts clauses, parties as a prelude kind, places owned by shares or by parts, the arrival rule, takes and relabels, observations, keyed decisions, the written projections (`low`, `bal(x, d) from t`, `next`, D-6), joint models and members as contexts, holes and hole parts, scenarios, axes, paths. CHANGES.md's 2.0 and 2.1 rows say what changed since 1.0.1.
- **Parts and labels** and **joint members as contexts** each get a named milestone (or a named step inside one), with D1-7's first step (a member's own context and memo, its walk lazy to the last day read) and its second (pruning to the places read).
- **Where the G1 design lands** (outline §D's S row and M rows): the evaluator, memo, walk, checkpoints and numbers at S; layers where their construct is built; the thread module and parallel paths and timelines where you judge best, with the reason (D3-2: paths and independent timelines first, members and parallel asks later); forks after the walk matches refcore; PGO only at M9.
- **Each milestone** names: what it builds; its checks (ids or globs; every check once, by the TSV); the DONE gates it advances (DONE.md); the P8 rows it touches at 1× (P8-10..P8-15 where their check's constructs are complete; P8-M rows at S; P8-L rows at the exit of the milestone that completes their constructs, at full size, or at M9 if they need the 10× and parallel work); the case models (`I:M:`) turning green, by the case lists. 1.0.1's arithmetic (3 + 7 + 34 + 23 + 12 + 0 = 79) is redone for the 2.1 placement.
- **M3** (the operations): `OPERATIONS.md` stays stale. Say what M3 can build from E alone, and what waits for `OPERATIONS.md`'s rewrite. No operation's meaning is written here.
- **What T still writes** (goldens G1–G10 and the 36 pins, hindcast H0–H9, the codes' missing fixtures, properties, the generator of 10 000 realities; `tmp/session8/plan.md` §2): place each where 1.0.1 placed its tally 1 form, marked "T's, not yet in `checks.tsv`".
- **No new meaning.** Placing a check needs only what it exercises (its `question`, `note` and `files` columns, `map/tags.tsv`'s edges, the case's construct list in `coverage.md`). If placing one would need a reading of the spec that L or E doesn't give, put it where its plainest construct puts it and log an O-W2 question.

## Inputs (read in this order)

1. The current document, **whole**: `engine/spec/BUILD-ORDER.md` (21 KB); `engine/spec/STALE.md` lines 1–6 and 56–75 (its stale rows); `engine/spec/DONE.md` (whole, 11 KB).
2. Session 8: `tmp/session8/decisions.md`, `tmp/research_notes/engine-prep/NOTE.md` §1–§2, `tmp/session8/plan.md` §2 (where T fits).
3. The tests, **by columns, never whole**: `engine/tests/checks.tsv` (634 lines; read the `id`, `layer`, `files`, `asks`, `question`, `note` columns with `awk -F'\t'`, in groups), `engine/tests/map/tags.tsv` (`id`, `edges`), `engine/tests/README.md` and `TESTING.md` (whole, 10 KB each: the layers), `engine/tests/load/load.tsv` (17 rows), `engine/tests/cases/boundaries.md` (57 KB: by its headings and the rows you need). Never open `engine/tests/cases/*-hidden.md`.
4. The spec's maps: `engine/spec/coverage.md` (84 KB: by its headings, the construct-by-case map), `engine/spec/CHANGES.md` (lines 1–140: what 2.0 and 2.1 changed), `engine/BUILD-NOTES.md` (whole, 5 KB).
5. The spec, by headings and the sections a milestone names: `engine/spec/LANGUAGE.md` (grep `^## \|^### ` for the map; 356 KB, never whole), `engine/spec/ENGINE.md` (61 KB; §1–§11 by headings).

## Done when

The draft is complete, every check is placed once (the script exits 0, its output pasted at the end of your `progress.md`), every P8 id you cite is in outline §C, and your final message (common.md §C) gives the count per milestone and your open items.
