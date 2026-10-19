# Changes: tally 1.0 → tally 1.0.1

tally 1.0.1 is the first patch of the frozen spec (`DONE.md` rule 1). Each change in it has the three things that rule asks for: a logged contradiction, a row in the gap log below, and a patch that names both. It corrects the definition of `tally 1`, and files still state `tally 1` (`LANGUAGE.md` §9). Every gate is re-run on it.

This file is the gap log. The spec had none before. Line numbers in it (L:, E:, O:, `file:line`) are tally 1.0's, the tree at the tag `tally-1.0`.

**New in the package:**
- `CHANGES.md`, this file;
- `VERSION`, the tag this tree matches: `tally-1.0.1`;
- `amendments/expected/`, the expected outputs of the household's 7 changes (DONE gate 3, second half);
- `montecarlo/`, the first models that draw and their keys (DONE gate 7): `LH-17-mc.tally` with `LH-17-mc.md`, and `household-bands/` with `household-bands.md` and the oracle scripts in `household-bands-oracle/`.

**What it doesn't change:** no answer key value in `expected/`, `smoke/` or `verification/`. Every verifier checked its patches against the models and keys that use the construct, and the household and every change load with 0 refusals under `tools/loadcheck.py`. The change files' "in words" sections were corrected where both evaluators of a change disagreed with them (LX-1 to LX-7); where a change file and its expected file still differ, the expected file holds.

## How the contradictions were found

1. **Readers, R01–R13.** Thirteen readers each read one slice of the frozen text against the rest, and listed candidate contradictions with the lines on both sides:

   | Reader | Slice |
   |---|---|
   | R01 | `LANGUAGE.md` 1–251 (§1–§2) |
   | R02 | `LANGUAGE.md` 252–490 (§3 types and units, §4 time, knowledge and the read rule) |
   | R03 | `LANGUAGE.md` §5.1–§5.9 |
   | R04 | `LANGUAGE.md` §5.10–§5.13 |
   | R05 | `LANGUAGE.md` §5.14–§5.18 |
   | R06 | `LANGUAGE.md` §6 |
   | R07 | `LANGUAGE.md` §7–§10 |
   | R08 | `ENGINE.md` 1–234 |
   | R09 | `ENGINE.md` 235–442 |
   | R10 | `OPERATIONS.md` |
   | R11 | `PERFORMANCE.md` and `BOUNDS.md` |
   | R12 | `DECISIONS.md` |
   | R13 | `BUILD-ORDER.md`, `DONE.md` and `coverage.md` |

2. **Verifiers, V1–V5.** Each took two readers' candidates (V1: R01 and R05; V2: R02 and R08; V3: R03 and R06; V4: R04 and R09; V5: R07 and R10), settled each against the text as real, real in wording only, or not real, and wrote a patch for each real one. Where a choice between readings was not theirs to make, they wrote it up "for the user" with a recommendation.
3. **The lead, V6.** The lead triaged R11–R13 itself, merged the verifiers' patches, and decided the verifiers' "for the user" items on their recommendations, since those are engine semantics, not DONE gates.
4. **The seven changes, C1–C7.** Two evaluators worked each change by hand, blind to each other, from tally 1.0 and the household alone. The lead compared each pair cell by cell. Their findings are below; the agreed cells are `amendments/expected/`.
5. **The Monte Carlo keys, MC-1 and MC-2.** One evaluator wrote LH-17 with draws and its oracle (MC-1), another the household's path statistics (MC-2). The lead re-derived the figures and reviewed both. Their findings are below.

No engine ran. Nothing under `tmp/holdout/` was read.

## The merge

1. V1, V2, V3, V4, V5, each in its file's order, minus the drops below.
2. The lead's patches, in the order of the patch table below: L6-P1 to L6-P22, LC-1, the LM patches, the LU patches with L6-P23 and L6-P24 among them, the LR patches, LX-1 to LX-8, LA-1 to LA-3, LD-1 to LD-4.

A hunk's old text occurs exactly once in its file at the moment it is applied. The result: 373 hunks applied, 42 dropped, 0 held.

| Dropped | Taken instead | Why |
|---|---|---|
| V2-P13:0 | V3-P2 | the same text; V3's wording is fuller |
| V4-P8:0, 1 | V3-P11 | the same finding |
| V4-P6:0 | V2-P16 | the same finding |
| V5-P33:0 | V3-P23 | the same text (V5-P33:1 stays) |
| V4-P14:0, 1 | V5-P5 | the same finding; V5 covers the scenario file's load error in full |
| V3-P13:0 | V5-P6 | the cases of `LOAD-KIND` |
| V4-P13:0 | V2-P19 | the same finding |
| V5-P13:0 | V2-P23 | `WARN-UNREAD` |
| V2-P21:0, V5-P16:0 | V4-P4 (both hunks, change 3's included) | the `HOLE-CHANGED` date; V4's covers the change file too |
| V3-P25:0 | V5-P24 | the four change kinds |
| V3-P16:0, V3-P17:0, 1, V5-P1:0, 1, 2, V1-P19 (all) | V2-P4 | six against seven rounding modes: DECISIONS U-03 and decision 11 say seven, and a decision wins (R02-6) |
| V1-P4 (all) | V5-P14 | `solve` in a rule with `v` |
| V1-P2 | V3-P4 | the key literal that is a place id |
| V3-P12:0, V5-P9:0, V1-P15:1 | V1-P15:0, V5-P9:1 | the header refusals of a missing `on` |
| V1-P16 (all) | V3-P18 | §6.5 names its versions; V3 renames the versions, V1 the sources. V3's apply in order and cover the `LOAD-KEY` cases |
| V1-P8 | V5-P3 | the functions that need `to`: V5 names the rows of §5.14, the table's own wording |
| V2-P10:0, V5-P7:0 | L6-P23 | both rewrote the `HOLE-MISSING` row; combined (V2-P10:1, in ENGINE, stays) |
| V2-P5:1, V3-P15:0 | L6-P24 | both rewrote the `LOAD-UNIT` row; combined (V2-P5:0 stays) |
| L6-P2:0 | V2-P15 | the PERFORMANCE booking-cut bullet: the same reading, citing `ENGINE.md` §5 |
| L6-P6:0 | V2-P22 | the walk day gains "anchor": the same text |
| L6-P15:0, 1 | V4-P17 | the carry figure: V4's "1.504 years (printed 1.50)" agrees with the lead's |

**V1-P25** (gate 1's declared bounds) waited for the user, since `DONE.md` is the user's contract. The user approved it with the other `DONE.md` items on 2026-10-19 (below), and LD-1 to LD-4 carry those.

## The patches

A hunk is one replacement of old text by new. "For" names the findings each patch answers; the gap log says what each is. LM-5 (`*` in a pin, MC1-5 and MC2-3) has no row of its own: V1-P6's pin grammar already takes `*`.

| Patch | For | What it settles | Files | Hunks in 1.0.1 |
|---|---|---|---|---|
| V1-P1 | R01-1, R05-1 | `place` is a keyword that also names two built-ins | `LANGUAGE.md` | 2 |
| V1-P2 | R01-2 | place ids as keys of `place` | `LANGUAGE.md` | 0 of 1; 1 dropped for V3-P4 |
| V1-P3 | R01-3 | the `explains` field in the grammar | `LANGUAGE.md` | 1 |
| V1-P4 | R01-4, R05-2 | `solve` in a def rule | `LANGUAGE.md` | 0 of 3; 3 dropped for V5-P14 |
| V1-P5 | R01-5 | groups of places, and currency codes in expressions | `LANGUAGE.md` | 2 |
| V1-P6 | R01-6 | `*` in a pin | `LANGUAGE.md` | 2 |
| V1-P7 | R01-7, R05-3, R05-4(a) | distributions in the grammar, and `to` on a draw | `LANGUAGE.md` | 2 |
| V1-P8 | R05-4(b) | `uniform` needs no `to` in `cdf`, `pdf` and `quantile` | `LANGUAGE.md` | 0 of 1; 1 dropped for V5-P3 |
| V1-P9 | R01-10 | parse rule 1 covers `solve` | `LANGUAGE.md` | 1 |
| V1-P10 | R01-11 | row stable ids with their own spans | `LANGUAGE.md` | 1 |
| V1-P11 | R01-12 | canonical order of rows that differ only by span | `LANGUAGE.md` | 1 |
| V1-P12 | R01-17 | a version's spans have literal ends | `LANGUAGE.md` | 4 |
| V1-P13 | R01-18 | what "nest" counts | `LANGUAGE.md` | 1 |
| V1-P14 | R01-19 | the short form may carry a hole | `LANGUAGE.md` | 1 |
| V1-P15 | R01-20 | the header refusals | `LANGUAGE.md` | 1 of 2; 1 dropped for V5-P9 |
| V1-P16 | R01-9 | distinct source ids in §6.5's examples | `LANGUAGE.md` | 0 of 4; 4 dropped for V3-P18 |
| V1-P17 | R05-5 | the log-score form | `LANGUAGE.md` | 1 |
| V1-P18 | R05-9, importer half | the importer gives each closing balance its day | `OPERATIONS.md` | 1 |
| V1-P19 | R01-8 | six rounding modes | `DECISIONS.md` | 0 of 3; 3 dropped for V2-P4 |
| V1-P20 | R01-14, R01-21 | `rule` and `prices` among the clause words | `LANGUAGE.md` | 2 |
| V1-P21 | R01-15 | the free-text list | `LANGUAGE.md` | 1 |
| V1-P22 | R01-16 | where date literals appear | `LANGUAGE.md` | 1 |
| V1-P23 | R01-23 | the Contents title of §8 | `LANGUAGE.md` | 1 |
| V1-P24 | R05-11 | which days `bal(x, d)` reads | `LANGUAGE.md` | 1 |
| V1-P25 | R05-12 | the declared bounds of gate 1 | `DONE.md` | 1 |
| V2-P1 | R02-1, R08-5 | — | `LANGUAGE.md` | 1 |
| V2-P2 | R02-2 | — | `LANGUAGE.md` | 1 |
| V2-P3 | R02-3 | — | `LANGUAGE.md` | 1 |
| V2-P4 | R02-6 | — | `LANGUAGE.md`, `coverage.md` | 5 |
| V2-P5 | R02-7 | — | `LANGUAGE.md` | 1 of 2; 1 dropped for L6-P24 |
| V2-P6 | R02-8, R02-9 | — | `LANGUAGE.md` | 1 |
| V2-P7 | R02-11 | — | `LANGUAGE.md` | 1 |
| V2-P8 | R02-12 | — | `LANGUAGE.md` | 1 |
| V2-P9 | R02-13, R02-12, R02-1 | — | `LANGUAGE.md` | 1 |
| V2-P10 | R08-6, R02-14 | — | `LANGUAGE.md`, `ENGINE.md` | 1 of 2; 1 dropped for L6-P23 |
| V2-P11 | R02-14 | — | `LANGUAGE.md` | 1 |
| V2-P12 | R02-15 | — | `LANGUAGE.md` | 1 |
| V2-P13 | R02-16 | — | `LANGUAGE.md` | 0 of 1; 1 dropped for V3-P2 |
| V2-P14 | R02-17 | — | `LANGUAGE.md` | 1 |
| V2-P15 | R08-1 | — | `PERFORMANCE.md` | 1 |
| V2-P16 | R08-2 | — | `LANGUAGE.md` | 1 |
| V2-P17 | R08-3, R08-10 | — | `ENGINE.md` | 1 |
| V2-P18 | R08-4 | — | `ENGINE.md` | 1 |
| V2-P19 | R08-7 | — | `ENGINE.md` | 1 |
| V2-P20 | R08-8 | — | `ENGINE.md` | 1 |
| V2-P21 | R08-11 | — | `ENGINE.md` | 0 of 1; 1 dropped for V4-P4 |
| V2-P22 | R08-12 | — | `PERFORMANCE.md` | 1 |
| V2-P23 | R08-13 | — | `ENGINE.md` | 1 |
| V3-P1 | R03-1 | — | `LANGUAGE.md` | 1 |
| V3-P2 | R03-2, R06-7 | — | `LANGUAGE.md` | 1 |
| V3-P3 | R03-2, R06-7 | — | `ENGINE.md` | 1 |
| V3-P4 | R03-3 | — | `LANGUAGE.md` | 1 |
| V3-P5 | R03-4 | — | `LANGUAGE.md` | 1 |
| V3-P6 | R03-5 | — | `LANGUAGE.md` | 1 |
| V3-P7 | R03-6 | — | `LANGUAGE.md` | 1 |
| V3-P8 | R03-6, R03-9 (e) | — | `LANGUAGE.md` | 1 |
| V3-P9 | R03-7 | — | `LANGUAGE.md` | 1 |
| V3-P10 | R03-7 | — | `ENGINE.md` | 1 |
| V3-P11 | R03-8 | — | `LANGUAGE.md` | 2 |
| V3-P12 | R03-9 (a) | — | `LANGUAGE.md` | 0 of 1; 1 dropped for V1-P15 and V5-P9 |
| V3-P13 | R03-9 (b) | — | `LANGUAGE.md` | 0 of 1; 1 dropped for V5-P6 |
| V3-P14 | R03-9 (c) | — | `LANGUAGE.md` | 1 |
| V3-P15 | R03-9 (d) | — | `LANGUAGE.md` | 0 of 1; 1 dropped for L6-P24 |
| V3-P16 | R03-10 | — | `BUILD-ORDER.md` | 0 of 1; 1 dropped for V2-P4 |
| V3-P17 | R03-10 | `DECISIONS.md` is the record | `DECISIONS.md` | 0 of 2; 2 dropped for V2-P4 |
| V3-P18 | R06-1 | — | `LANGUAGE.md` | 5 |
| V3-P19 | R06-2 | — | `LANGUAGE.md` | 4 |
| V3-P20 | R06-3 | — | `LANGUAGE.md` | 4 |
| V3-P21 | R06-3 | `DECISIONS.md` is the record | `DECISIONS.md` | 2 |
| V3-P22 | R06-4 | — | `LANGUAGE.md` | 1 |
| V3-P23 | R06-5 | — | `LANGUAGE.md` | 1 |
| V3-P24 | R06-6 | — | `LANGUAGE.md` | 1 |
| V3-P25 | R06-8 | — | `BUILD-ORDER.md` | 0 of 1; 1 dropped for V5-P24 |
| V3-P26 | R06-9 | — | `LANGUAGE.md` | 1 |
| V3-P27 | R06-9 | — | `OPERATIONS.md` | 1 |
| V3-P28 | R06-10 | — | `LANGUAGE.md` | 1 |
| V3-P29 | R06-10 | — | `OPERATIONS.md` | 1 |
| V3-P30 | R06-11 | — | `LANGUAGE.md` | 1 |
| V3-P31 | R06-12 | — | `LANGUAGE.md` | 1 |
| V3-P32 | R06-13 | — | `amendments/README.md` | 1 |
| V3-P33 | R06-14 | — | `amendments/README.md` | 8 |
| V3-P34 | R06-15 | — | `LANGUAGE.md` | 1 |
| V3-P35 | R06-15 | — | `OPERATIONS.md` | 1 |
| V3-P36 | R06-16 | — | `LANGUAGE.md` | 1 |
| V3-P37 | R06-16 | — | `LANGUAGE.md` | 1 |
| V4-P1 | R04-1 | — | `LANGUAGE.md` | 1 |
| V4-P2 | R04-2 | — | `LANGUAGE.md`, `PERFORMANCE.md` | 2 |
| V4-P3 | R04-3 | — | `LANGUAGE.md`, `PERFORMANCE.md` | 2 |
| V4-P4 | R04-4 | — | `ENGINE.md`, `amendments/changes/3-retroactive-law.md` | 2 |
| V4-P5 | R04-5 | — | `ENGINE.md`, `LANGUAGE.md` | 2 |
| V4-P6 | R04-7 | — | `LANGUAGE.md` | 0 of 1; 1 dropped for V2-P16 |
| V4-P7 | R04-8 | — | `LANGUAGE.md` | 2 |
| V4-P8 | R04-9 | — | `LANGUAGE.md` | 0 of 2; 2 dropped for V3-P11 |
| V4-P9 | R04-11 | — | `LANGUAGE.md` | 1 |
| V4-P10 | R09-1 | — | `ENGINE.md` | 2 |
| V4-P11 | R09-3 | — | `ENGINE.md` | 3 |
| V4-P12 | R09-5 | — | `ENGINE.md` | 1 |
| V4-P13 | R09-6 | — | `ENGINE.md` | 0 of 1; 1 dropped for V2-P19 |
| V4-P14 | R09-7 | — | `LANGUAGE.md` | 0 of 2; 2 dropped for V5-P5 |
| V4-P15 | R09-8 | — | `ENGINE.md` | 1 |
| V4-P16 | R09-9 | — | `LANGUAGE.md`, `ENGINE.md` | 3 |
| V4-P17 | R09-10 | — | `ENGINE.md`, `DECISIONS.md` | 2 |
| V4-P18 | R09-12 | — | `OPERATIONS.md` | 1 |
| V4-P19 | S-1, S-2 | — | `ENGINE.md`, `LANGUAGE.md`, `BUILD-ORDER.md` | 7 |
| V5-P1 | R07-1 | six rounding modes | `BUILD-ORDER.md`, `DECISIONS.md` | 0 of 3; 3 dropped for V2-P4 |
| V5-P2 | R07-2 | a step that isn't a power of ten | `LANGUAGE.md` | 1 |
| V5-P3 | R07-3 | uniform is exact | `LANGUAGE.md` | 1 |
| V5-P4 | R07-4 | the kinds of result | `LANGUAGE.md` | 1 |
| V5-P5 | R07-5 | a load error in a scenario file | `LANGUAGE.md`, `ENGINE.md` | 5 |
| V5-P6 | R07-6 | the cases of `LOAD-KIND` | `LANGUAGE.md` | 1 |
| V5-P7 | R07-7 | the code of the `next_on` hole and of `flows` before S | `LANGUAGE.md` | 0 of 1; 1 dropped for L6-P23 |
| V5-P8 | R07-8 | an `if` branch is a whole value | `LANGUAGE.md` | 1 |
| V5-P9 | R07-9 | a missing `on` (or `from`) | `LANGUAGE.md` | 1 of 2; 1 dropped for V1-P15 |
| V5-P10 | R07-11 | the message of `RUN-UNDECIDED` | `LANGUAGE.md` | 1 |
| V5-P11 | R07-12 | non-integer powers, and the domain of `pow` | `LANGUAGE.md` | 4 |
| V5-P12 | R07-13 | where a hole stops | `LANGUAGE.md` | 1 |
| V5-P13 | R07-15 | what `WARN-UNREAD` covers | `ENGINE.md` | 0 of 1; 1 dropped for V2-P23 |
| V5-P14 | R07-16 | `solve` in a rule with `v` | `LANGUAGE.md` | 6 |
| V5-P15 | R07-17 | patches of the frozen text | `LANGUAGE.md` | 1 |
| V5-P16 | R07-18 | the date of a `HOLE-CHANGED` movement | `ENGINE.md` | 0 of 1; 1 dropped for V4-P4 |
| V5-P17 | R07-19 | the `1e-N` step | `LANGUAGE.md` | 1 |
| V5-P18 | R07-20 | two messages of `LOAD-KEY` | `LANGUAGE.md` | 1 |
| V5-P19 | R07-21 | the result records its version | `ENGINE.md` | 1 |
| V5-P20 | R10-1 | Claude accepts rows; the household confirms | `OPERATIONS.md` | 1 |
| V5-P21 | R10-4 | a draft law part in a scenario run | `OPERATIONS.md` | 1 |
| V5-P22 | R10-5 | what a law part holds | `LANGUAGE.md` | 2 |
| V5-P23 | R10-6 | what `catalog` returns | `OPERATIONS.md` | 1 |
| V5-P24 | R10-7 | four change kinds | `BUILD-ORDER.md` | 1 |
| V5-P25 | R10-8 a, b | the matching window and tolerance | `OPERATIONS.md` | 1 |
| V5-P26 | R10-10 | exact, near, ambiguous | `OPERATIONS.md` | 1 |
| V5-P27 | R10-11 | the sanity line's bound | `OPERATIONS.md` | 1 |
| V5-P28 | R10-12 | a saved analysis keeps its R | `OPERATIONS.md` | 1 |
| V5-P29 | R10-13 | the `known` of a law version | `OPERATIONS.md` | 1 |
| V5-P30 | R10-14 | a `revise` without a reason | `OPERATIONS.md` | 2 |
| V5-P31 | R10-15 | a check that fails in a scenario | `OPERATIONS.md` | 2 |
| V5-P32 | R10-16 | the pipeline stops at the first failing step | `OPERATIONS.md` | 1 |
| V5-P33 | R10-17 | the conflicts of additions, and `OP-BASE` | `LANGUAGE.md` | 1 of 2; 1 dropped for V3-P23 |
| V5-P34 | R10-18 | path statistics, not bands | `OPERATIONS.md`, `PERFORMANCE.md` | 3 |
| V5-P35 | R10-19 | the `save` input | `OPERATIONS.md` | 1 |
| L6-P1 | R11 | PERFORMANCE "exact actuarial values to 4–5 decimals" as a declared bound | `PERFORMANCE.md` | 1 |
| L6-P2 | R11 | the flow pay cost's cut | `PERFORMANCE.md` | 1 of 2; 1 dropped for V2-P15 |
| L6-P4 | R11 | PERFORMANCE §4.3 "re-walks from next year" | `PERFORMANCE.md` | 1 |
| L6-P6 | R11-7 | walk day misses anchors | `PERFORMANCE.md` | 0 of 1; 1 dropped for V2-P22 |
| L6-P7 | R11 | "396 checkpoints of 13 places" | `PERFORMANCE.md` | 1 |
| L6-P8 | R11 | BO:262 grouping aggregates is in P §7, not §10 | `BUILD-ORDER.md` | 1 |
| L6-P9 | R11 | P:267 transcendentals list short of L §7.3 | `PERFORMANCE.md` | 1 |
| L6-P10 | R12 | decision 11 "LOAD-NAME" for a clash | `DECISIONS.md` | 1 |
| L6-P11 | R12 | conversion cited at L §5.13.1 | `DECISIONS.md`, `BOUNDS.md`, `BUILD-ORDER.md` | 5 |
| L6-P12 | R12 | T-15 "`plan` between two record times" | `DECISIONS.md` | 1 |
| L6-P13 | R12 | L-39 cites O §3 step 2 | `DECISIONS.md` | 1 |
| L6-P14 | R12 | R-13 `flows(buy, ..d - 1)` reaches before S, a hole | `DECISIONS.md`, `LANGUAGE.md` | 2 |
| L6-P15 | R12 | "= 1.50 years" is 1.50425 | `ENGINE.md`, `DECISIONS.md` | 0 of 2; 2 dropped for V4-P17 |
| L6-P16 | R12 | "taken differently" missing from the statuses | `DECISIONS.md` | 1 |
| L6-P17 | R13 (BO-1) | BUILD-ORDER placement: cases and changes in milestones that lack their constructs | `BUILD-ORDER.md` | 17 |
| L6-P18 | R13 | M7 "0" while `montecarlo/` adds two keys | `BUILD-ORDER.md` | 2 |
| L6-P19 | R13 | M8's refusals with no code | `BUILD-ORDER.md`, `verification/hindcast.md` | 2 |
| L6-P20 | R13 | BO:8 "one planted cause" (H0 has none, H9 two) | `BUILD-ORDER.md` | 1 |
| L6-P21 | R13 | coverage K12/K13 counts are 1.0's | `coverage.md` | 1 |
| L6-P22 | C3-F7 | the gate premise | `amendments/README.md`, `BUILD-ORDER.md` | 2 |
| LC-1 | C1-F2 | a key added after a booking cut | `LANGUAGE.md` | 1 |
| LM-1 | MC1-1, MC2-15 | the oracle test's statistic | `ENGINE.md` | 1 |
| LM-2 | MC1-4 | the horizon of a term that reads no day | `ENGINE.md` | 1 |
| LM-3 | MC1-6 | the test mode's reason | `ENGINE.md` | 1 |
| LM-4 | MC1-8 | draws in the ≈ bound | `LANGUAGE.md` | 1 |
| LM-6 | MC2-1 | a path-keyed pay | `LANGUAGE.md` | 1 |
| LM-7 | MC2-2 | draws in a scenario file | `LANGUAGE.md` | 1 |
| LM-8 | MC2-8 | a draw's law date | `LANGUAGE.md` | 1 |
| LU-1 | V1 R05-6 | the SE of `mean` and `quantile`: `mean`: s/√N, sample variance over N − 1, to 1e-6; `quantile`: none | `LANGUAGE.md`, `ENGINE.md` | 3 |
| LU-2 | V1 R05-7 | the canonical order of a draw with no time key: first; periods by their first day | `LANGUAGE.md` | 1 |
| LU-3 | V1 R05-8 | carry with a draw that has no `plan`: the share prints, and the verdict fails with a reason | `ENGINE.md` | 1 |
| LU-4 | V1 R05-9 | a `stated` row longer than a day: anchors at its last day | `LANGUAGE.md`, `ENGINE.md` | 2 |
| LU-5 | V2 R02-4 | a price year through a monthly index: refused, `LOAD-KEY` | `LANGUAGE.md` | 2 |
| LU-6 | V4 R04-6 | is `next_on` a state read?: no, for a flow pay | `LANGUAGE.md` | 1 |
| LU-7 | V4 R09-2 | a record dated after its state pay instance: booked on its own day | `ENGINE.md` | 2 |
| LU-8 | V4 R09-4 | lines without a record under `by f`: by pay id, printed `pay <id>` | `ENGINE.md` | 1 |
| LU-9 | V5 U1, reading A | imports and advances apply without `confirm` | `OPERATIONS.md`, `ENGINE.md` | 4 |
| LU-10 | V5 U2, reading A | a built-in `pay` index | `LANGUAGE.md` | 4 |
| LU-11 | V5 U3, reading A | an own-place candidate needs no mapping | `OPERATIONS.md` | 1 |
| L6-P23 | V2-P10 hunk 0 and V5-P7 hunk 0 combined | the `HOLE-MISSING` row | `LANGUAGE.md` | 1 |
| L6-P24 | V2-P5 hunk 1 and V3-P15 hunk 0 combined | the `LOAD-UNIT` row | `LANGUAGE.md` | 1 |
| LR-1 | C1-F1, C2-F4, C3-F3, C4-F8, C6-F1 | `held` and the printed cut are the ask's K | `LANGUAGE.md` | 2 |
| LR-2 | C2-F8, C3-F4, C5-F3 | what a cut class keeps the same | `ENGINE.md` | 1 |
| LR-3 | C1-F3, C2-F3, C6-F2 | what sanity counts | `OPERATIONS.md` | 1 |
| LR-4 | C6-F8 | `WARN-NEGATIVE` tests end-of-day balances | `ENGINE.md`, `LANGUAGE.md` | 2 |
| LR-5 | C5-F2 | a version that makes a hole `when` false answers it | `LANGUAGE.md`, `ENGINE.md` | 2 |
| LR-6 | C2-F1, C2a-8, C5-F3 | the advance is its own amendment | `amendments/README.md` | 3 |
| LR-7 | C3-F1, C3b-2 | which `flows` a `HOLE-CHANGED` movement makes holes | `LANGUAGE.md`, `ENGINE.md` | 2 |
| LR-8 | C3-F2, C3b-3 | what `WARN-BACKPAY` lists | `OPERATIONS.md` | 1 |
| LR-9 | C7-F1, C7a-1 | a scenario row is never `held` | `LANGUAGE.md` | 1 |
| LR-10 | C4-F4, C4a-4, C4b-7 | a fact version that changes booked pays | `OPERATIONS.md` | 3 |
| LX-1 | C1-F5, C1-F7 | change 1's file: its words against its expected outputs | `amendments/changes/1-new-client.md` | 15 |
| LX-2 | C2-F1, C2-F10, and the table "Where both say the change file is wrong" | change 2's file: its words against its expected outputs | `amendments/changes/2-law-next-year.md` | 9 |
| LX-3 | C3's table "Where both say the change file is wrong", C3-a and C3-b §7 | change 3's file: its words against its expected outputs | `amendments/changes/3-retroactive-law.md` | 11 |
| LX-4 | C4-F1, C4-F6, and the table "Where both say the change file is wrong" | change 4's file: its words against its expected outputs | `amendments/changes/4-child-born.md` | 14 |
| LX-5 | C5-F1, C5-F5, C5-F6, C5-F7, C5-F10 | change 5's file: its words against its expected outputs | `amendments/changes/5-job-abroad.md` | 12 |
| LX-6 | C6-F3, C6-F5, C6-F6, C6-F7, C6-F11 | change 6's file: its words against its expected outputs | `amendments/changes/6-loan-and-overpayment.md` | 12 |
| LX-7 | the table "Where both say the change file is wrong", C7-a and C7-b §7 | change 7's file: its words against its expected outputs | `amendments/changes/7-scenario-partner-stops.md` | 10 |
| LX-8 | C1-F4, C5-F8, C6-F10, and the line counts | the amendments README | `amendments/README.md` | 7 |
| LA-1 | V1 aside 1 | a keyed place among a row's keys | `LANGUAGE.md` | 1 |
| LA-2 | V1 aside 2 | the printer's parentheses under parse rule 1 | `LANGUAGE.md` | 1 |
| LA-3 | V3-S2 | a missing header line | `LANGUAGE.md` | 1 |
| LD-1 | D-2, R13-11 | gate 6's no foresight includes C-13 | `DONE.md` | 1 |
| LD-2 | D-3, D-5, R13-10, MC1-3 | gate 7's closed form, its files, and path statistics | `DONE.md` | 1 |
| LD-3 | D-4, R13-16 | gate 12 takes each budget at its row's size | `DONE.md`, `BUILD-ORDER.md` | 2 |
| LD-4 | D-1 to D-5 | the approvals log | `DONE.md` | 1 |

## The gap log

### R01–R10, as the verifiers settled them

Each table is the verifier's own, with a last column for what carries the finding in 1.0.1. Where a verifier's patch was dropped for another's, the carrier is named. "real" changes what loads, a code, a figure or what prints; "real, wording only" changes no behaviour.

#### V1: R01 and R05

| Id(s) | Verdict | Effect | Settling text or defect (1.0 lines) | Patch | In 1.0.1 |
|---|---|---|---|---|---|
| R01-1, R05-1 | real | what loads | `place` is a keyword, refused as an id, key, record field or bound variable (L:84, :87). Yet it is the built-in index (L:113, :603), written `x in place` (L:351, :919, household `asks/core.tally:13`). It is also the built-in record field (L:111, :1113). Only `tools/loadcheck.py` exempts it, and silently. The `path` half is settled: `bind = ID "in" (ID \| "[" key {"," key} "]" \| "path")` (L:166) takes `path` as a literal. | V1-P1 | V1-P1 |
| R01-2 | real | what loads | L:247 "A key literal that is also a top-level id is refused (`LOAD-KEY`)". The keys of `place` are place ids (L:603), so `opening[konto]`, `stated[konto]` and a record's `konto` field are refused as written. The household has 11 such rows plus its record fields. | V1-P2 | V3-P4 (for V1-P2) |
| R01-3 | real | what loads | `field = ["-"] lit \| hole \| "-"` (L:171) cannot hold `wage[ben_bolt, 2026-03]` (L:1106; L:1115 "written `pay[keys]`"). The household's `records/mbank/2026-09.tally` has 8 such fields. | V1-P3 | V1-P3 |
| R01-4, R05-2 | real | what loads | Several decisions require `solve` in a def rule: U-16, R-14 and R-22 (DECISIONS:109, :132, :140, "`solve` in a rule with `v`"), and L:1059 and :1595 say the same. Yet only `askbody` derives it (L:163). Also, its bracket literals would hit the literal rule (L:1435–1438), which exempts the bracket only in asks (L:1444). No model uses a def `solve`. | V1-P4 | V5-P14 (for V1-P4) |
| R01-5 | real | what loads | `plan_low(x in place where …, a, b)` (L:919) is neither a `call` (its arguments are `expr`) nor an `agg` (not in `AGG`, L:197–198). `PLN` in `currency(x) = PLN` is not an `atom` (L:194–195; `CUR` is not a `lit`). The household's `net_worth` ask (`core.tally:13`) uses `currency(x) = PLN`. | V1-P5 | V1-P5 |
| R01-6 | real | what loads | `with theta[*] = theta_mean` (L:1495, the CHG-13 test mode) does not parse: `*` is only an `rk` (L:167), not an `expr` in `ref` (L:206). | V1-P6 | V1-P6 |
| R01-7, R05-3, R05-4(a) | real | what loads | `DIST` is never defined (L:204). `discrete(w[j] for j in IDX)` (L:992) is not `DIST "(" expr {"," expr} ")"`. `draw` requires `to UNIT` (L:155), but L:969 says a discrete draw is exact, and `discrete`'s values are keys (L:992). | V1-P7 | V1-P7 |
| R05-4(b) | real | code | The draw value is settled: L:969 "correctly rounded to the stated unit for a continuous D", so u = 0.3777 with `to 0.01` gives 0.38. The function rule conflicts, though. §7.3 (L:1473) says the `cdf`/`pdf`/`quantile` of every continuous distribution need `to u`, while the table (L:988) marks `uniform` "exact". So `cdf(uniform(0, 3), 1)` is either `LOAD-TO` or exactly 1/3. No model uses `uniform` (K14). | V1-P8 | V5-P3 (for V1-P8) |
| R01-8 | real, wording only | wording | §2.3 `MODE` (L:210) and §7.2 list six modes. `half-down` was removed in D7 (`coverage.md`:19, R3), a removal DECISIONS:447 incorporates ("`coverage.md` §3 has the constructs no case uses, each removed or kept with its reason"). Three places still say "seven": DECISIONS:96, :383 and BUILD-ORDER:84. The grammar's closed `MODE` list keeps a builder right. | V1-P19 | V2-P4 (for V1-P19) |
| R01-9 | real | what loads (examples) | §6.5's examples declare a source and a version with one id: `pit_2028` (L:1313/1315), `pit_2027_retro` (L:1326/1328) and `car_loan` (L:1391/1394). One namespace refuses that (L:242, `LOAD-DUP`). The change files themselves use distinct ids (change 6: `source car_contract`, `version car_loan`). | V1-P16 | V3-P18 (for V1-P16) |
| R01-10 | real | code | Judge's fix 12 (DECISIONS:354): "The single `=` after `when` and in `solve` \| C's parse rule (L §2.3)". Parse rule 1 (L:215) names only `when`, `at` and `check`. On `solve r in [0, 1] to 1e-6: f = if k = 1 then a else b`, the rule gives `LOAD-PARSE` (an unparenthesised `=` on the right), while a generic parser loads it. | V1-P9 | V1-P9 |
| R01-11 | real | code or message | `version:target[keys]` (L:243) is not unique when one version holds rows of one target and keys with different own spans. Examples: household `facts/2026.tally`, `forecast_2026`'s two `inflation` rows and `dg_2026`'s three `zus_stage` rows. A band table's rows are the same case. Operations name by stable id (O:69), so `OP-TARGET` "matches <a>, <b>" cannot list distinct candidates. | V1-P10 | V1-P10 |
| R01-12 | real | canonical text, hence the hash | L:234 sorts version items by target, then keys. Rows that tie there (the same `inflation` pair) print in the written order, so equal models print differently. That breaks L:227, "so equal models print identically". Band rows are already ascending (L:651), so only own-span rows need a tie-break. | V1-P11 | V1-P11 |
| R01-13 | not real | — | Every version in a scenario file lacks `known`: L:134 "(known: required outside a scenario file)" and L:455 "A version in a scenario file has no `known` line." So no file mixes versions with and without `known`. A scenario file's versions all tie on `known` and sort by id, the second key of L:232. | — | — |
| R01-14 | real, wording only | wording | `rule` (L:151) is in no word class of §2.2. Either reading leaves `input rule: num` loading, since contextual words are "free everywhere else" (L:103). | V1-P20 | V1-P20 |
| R01-15 | real, wording only | wording | L:78's "only free text" omits source titles and addresses (L:130), unit labels (L:139), axis-option labels (L:140) and an input's `question` (L:148). The grammar's `STR`s decide what parses. | V1-P21 | V1-P21 |
| R01-16 | real, wording only | wording | L:1440's list of date-literal places omits `from` (L:126), `enters` (L:131), `moves … to` (L:133), index ranges (L:142) and asks. L:1444 allows those in asks, and the grammar takes `DAY`/`lit` there, so nothing is refused. | V1-P22 | V1-P22 |
| R01-17 | real | what loads | Decision 39 (DECISIONS:426): "A governs end is a literal (§7.1), so (a) can't be written". L:369 says the same. The grammar takes `span = expr ".." …` (L:131, :168, :173). So `governs start_of(2027)..` loads by the grammar but should be refused. Every `governs` and `moves … to` in models and household is literal (grep). | V1-P12 | V1-P12 |
| R01-18 | real | what loads | "Nest" is undefined (L:223). Read as parse-tree depth, 10 answer-key lines are `LOAD-DEPTH` (household `pays.tally:70` (11) and `asks/core.tally:13` (9); MP-08:125 and :92, LH-18:76, LH-11:147, UL-08:97, LH-14:184, LH-13:102, LH-10:177). BUILD-ORDER M0 needs every model and the household to load. Counting open brackets plus open `if`s, the maximum over all 92 files is 7 (LH-04:103), and 5 counting brackets alone. | V1-P13 | V1-P13 |
| R01-19 | real | what loads and prints | L:236 prints a one-rule `base` def short as `def x: T = expr`. A `base = ? "q"` with no note would then print as `= ? "q"`, which L:150 does not parse (`"=" expr`). So `format(parse(x)) = x` fails. No model has a note-less base hole. | V1-P14 | V1-P14 |
| R01-20 | real | code | §5.1 (L:506) lists two `LOAD-HEADER` causes, while the catalogue (L:1565) lists three. A missing `on` (required, L:503) has no message, even though a missing `tally` does. A fixture can't know which code, or which message, to expect. | V1-P15 | V1-P15, part by V5-P9 |
| R01-21 | real, wording only | wording | `prices` is listed "in a type" (L:104), but the grammar puts it after the type, as an input clause (L:148; `coverage.md` table 2). Either class leaves it free elsewhere. | V1-P20 | V1-P20 |
| R01-22 | not real | — | L:227 "The loader accepts any spacing that the grammar allows. It then reprints the model as follows". The excerpts and models are inputs, not printer output. | — | — |
| R01-23 | real, wording only | wording | The Contents (L:15) gives a different title from the heading at L:1515. | V1-P23 | V1-P23 |
| R05-5 | real | code | L:1007 `ln(pdf(…)) to u`: the `to u` binds to `ln(…)` (L:193 `post`), so the inner `pdf` has no `to`. That is `LOAD-TO` by §7.3 (L:1473) and its "Composites are exact" rule, under which each transcendental is rounded on its own. CH-03:75 writes `pdf(normal(mu, sig), gx[i]) to 1e-30`. | V1-P17 | V1-P17 |
| R05-6 | real | figure | Only `share` has a standard-error formula (L:1048). `mean` (L:1049) states none, so N and N − 1 as divisor give 0.559017 or 0.645497 on 1, 2, 3, 4. `quantile` (L:1050) has no SE at all, although L:1047 and ENGINE:318 say one is printed. | for the user | LU-1 |
| R05-7 | real | figure | L:978 orders draws by path, time key, id, other keys. A draw with no time key (L:962 `life[p in path]`) has no stated place in that order, and fixed uniforms are dealt in it. | for the user | LU-2 |
| R05-8 | real | figure (verdict) or code | The carry weighs movements "each at its plan value (every draw at its plan value …)" (ENGINE:360). But `LOAD-PLAN` covers only look-aheads (L:974), and the rule has "Refusals: none of its own" (ENGINE:379). So a flow pay reading a draw with no `plan` leaves the carry undefined. | for the user | LU-3 |
| R05-9 | real | figure | L:1131 says an anchor is "on the day D they govern". ENGINE:177 anchors on every day t that "a `stated[x]` row governing t" covers. A row with no own span governs its version's month (L §4.6), and the importer is not told to give a closing balance its own day (O:119). Only the §5.18 example and the household row (`governs 2026-09-30`) do. | V1-P18 and for the user | V1-P18; LU-4 |
| R05-10 | not real | — | ENGINE:362 "The horizon h = (d − K) … bounds c (c ≤ h), and is printed beside it". ENGINE:365 prints it in the reasons, "`not sayable: carry 14.98 years > 2 (horizon 29.97 years)`". So the rule reads h for its reasons, and the three tests (ENGINE:358–365) decide the verdict. Nothing differs. | — | — |
| R05-11 | real, wording only | wording | L:908 "for d before today", while L:1510 defines today as "the ask's cut, or the model's `on`". L:1034 is explicit, though: "An ask may make every state read of §5.13.1 at any day of the run: `bal(x, d)`". And the state-pay limit is `LOAD-FUTURE`/`RUN-FUTURE` (L:923). | V1-P24 | V1-P24 |
| R05-12 | real, wording only | wording | DONE.md:18 counts "three declared bounds (MP-15 …, LH-17 step 6, CH-03 step 14)". The models declare six: CH-03 has four (`jss_obs`, `jss_unseeded`, `logs_sample`, `np_stream`, CH-03:110–116). The gate's first sentence ("Every ask … gives the figure its `expected/` file names") already covers them all, and `expected/CH-03.md` names each bound. | V1-P25 | V1-P25 |

#### V2: R02 and R08

| Id(s) | Verdict | Effect | Settling text or defect (1.0 lines) | Patch | In 1.0.1 |
|---|---|---|---|---|---|
| R02-1, R08-5 | real | figure | L428 "Otherwise `flows` over that span is a hole" also catches a period that lies wholly outside the span. ENGINE.md:160 ("counts … when the period lies inside the span, is a `HOLE-IMPRECISE` when the period crosses an end") and §8.1 L1537 ("a movement's period crosses an end of a `flows` span") make only a crossing period a hole | V2-P1, V2-P9 | V2-P1; V2-P9 |
| R02-2 | real | figure (backtest look-ahead) | L411 "or at the walk's day in a look-ahead". ENGINE.md:190 says "all at cut min(t, K)", and L910 says "all at cut min(day, K)". DECISIONS R-04 (L122): "No foresight by construction". At t > K a cut of t would read versions known after K | V2-P2 | V2-P2 |
| R02-3 | real, wording only | wording | §4.3 is the law date. The rule that reads a key by the period holding it is §4.6 L479 ("reads the key whose period holds it") | V2-P3 | V2-P3 |
| R02-4 | for the user | figure, or what loads | Nothing says what `cpi[2026]` is when `cpi`'s index is monthly. No month's period holds a year (L479), and L325 says only "the key that holds the price year" | for the user | LU-5 |
| R02-5 | not real | — | L1435: "In defs, rules, pays, fills and draws, the only number literals allowed are …". An ask, a check or a bound may hold `8000`, and literal adoption (L293) is a typing rule for any expression | — | — |
| R02-6 | real | what loads | DECISIONS U-03 (L96) says "`round(x, u, mode)`: seven modes", and decision 11 (L383) says "**Rounding to any positive step, seven modes, the mode an axis**". LANGUAGE L106, L210 (`MODE`) and §7.2 (L1456-1462) list six, because coverage R3 (L327) removed `half-down` in D7. BUILD-ORDER.md:84 still says "seven modes". A decision wins over text | V2-P4, plus a confirmation for the user | V2-P4; confirmed by the lead: a decision wins (R02-6) |
| R02-7 | real | code (where `LOAD-UNIT` fires) | DECISIONS U-05 (L98): "the same rate applied to PLN money is `LOAD-UNIT`". §3.2 L284 ("units multiply") types `money PLN × num PLN/EUR` as `num PLN^2/EUR`, which is refused only when it is used as money or added to PLN. In an ask it loads | V2-P5 | V2-P5, part by L6-P24 |
| R02-8, R02-9 | real | figure. R02-9 also what loads | L313 defines i only for `discount`, so for `nominal m` at other periods, `/month` and `/day` the "effective conversion (1 + i)^t − 1" has no i. "Otherwise it is the effective conversion" fixes the method: 1.01^12 − 1 = 0.126825, not 0.12. `simple /year` over `day` needs a day's length in years, which is a day count, and §10 L1664 forbids a default one ("A default is a silent assumption") | V2-P6 | V2-P6 |
| R02-10 | not real | — | L386 states its own rule: "The anniversary of 29 February in a common year is 1 March". L387 defines `age_months` by `add_months`, which clamps (L384). Each is determinate, and no text says `age` = `age_months` ÷ 12. Born 2024-02-29, on 2025-02-28 they give 0 and 12, as defined | — | — |
| R02-11 | real | figure (decided or `HOLE-IMPRECISE`) | L425 "the interval of their values at the two ends" is right only for a built-in that follows date order. `weekday` (L388) doesn't: 2019-01-01 and 2019-12-31 are both 2, so the rule decides `weekday(2019) = 2`, yet a day in 2019 can be any weekday | V2-P7 | V2-P7 |
| R02-12 | real | figure (hole or interval) | L426 lists "an aggregate's term" among the uses that need one value. L771 says "`min` and `max` give the interval of the ends' min (max) … `argmin` and `argmax` give the key whose value is decided below (above) every other key's". expected/MP-08.md:18 follows L771 (argmax over year-dated deaths) | V2-P8, V2-P9 | V2-P8; V2-P9 |
| R02-13 | real, wording only | wording | L410 ("otherwise it is `HOLE-IMPRECISE`") and L935 ("otherwise the instance's amount is `HOLE-IMPRECISE`") name causes that are missing from §8.1 L1537 | V2-P9 | V2-P9 |
| R02-14 | real | code (`HOLE-MISSING` or `RUN-NO-RULE`) | L477 "**No candidate** gives a hole" covers givens and dated rules (L463). §5.10 L729: a given that doesn't match → the def's rules. L730: a rule label with no candidate version drops out. L737: "None: refused, `RUN-NO-RULE`". §8.1 L1535 repeats "input, band or given" | V2-P11, V2-P10 | V2-P11; V2-P10, part by L6-P23 |
| R02-15 | real | what loads | L449 says stipulations are "known from the model's `start`, else its `on`". L452 says "A version known after `on` is refused (`LOAD-KNOWN`)". L506 doesn't forbid `start` > `on`. The clause's own intent is "given as though known all along" | V2-P12 | V2-P12 |
| R02-16 | real | what loads | §5.3 L556 "the same keys". §4.6 L476 "Two candidates still tied, with overlapping spans, are refused at load", §6.3 L1218 "two rows tied after precedence" and §8.3 L1576 "rows <a> and <b> of <x> tie at <keys> on <span>" are wider: `x[ana, *]` and `x[*, 2026]` tie at [ana, 2026]. No model has such a pair (a scan of the models found only disjoint ranges) | V2-P13 | V3-P2 (for V2-P13) |
| R02-17 | real, wording only | wording | L468 "its version's `known` ≤ K" doesn't apply to a scenario version, which has none. L455, §6.4 L1239-1241 and DECISIONS decision 3 settle it: "visible by date … not masked by K" | V2-P14 | V2-P14 |
| R08-1 | real, wording only | wording (cost document) | ENGINE.md:159 and §5 step 0 (L204), LANGUAGE L935, and DECISIONS decision 24 (L401): "An instance's `on` is read at the ask's cut; its `when` and amount at min(t, K)" | V2-P15 | V2-P15 |
| R08-2 | real | figure (only for a record dated after the walk day and visible at cut min(t, K)) | L910 "counting only flow pays and records". ENGINE.md:190: "Records and anchors after t are never used". DECISIONS R-04 (L122): "`plan_low` counts only flow pays as known that day". Records up to t are already in `bal(x)` | V2-P16 | V2-P16 |
| R08-3 | real | code (`RUN-PLACE` or not) | ENGINE step 5 (L178-179) puts the check after the day is booked, yet says "The movement isn't booked". Step 3 (L175) books each state pay "at once, so the next state pay sees it". L703 says "a pay that would take it above 0", and §8.3 L1623 "`<pay> takes <debt> above 0`". Only a check made as each movement is booked fits all of these | V2-P17 | V2-P17 |
| R08-4 | real | figure (or a hole) | ENGINE.md:160 takes the law date from the `on` period with no condition. LANGUAGE L411 uses `on` only "with neither" a time key nor `for`. DECISIONS 36 (L423) is about how an imprecise date gives law dates, not about whether it is used | V2-P18 | V2-P18 |
| R08-6 | real | code or message | ENGINE.md:78 "`flows` over days before S is a hole: `before the run's start`" gives no code, and none of the four codes in §8.1 lists it | V2-P10 | V2-P10, part by L6-P23 |
| R08-7 | real | what loads, and code (`REQ-CLOCK`) | ENGINE.md:35 "clocks: S and E (default: the header's start and end)" lets a request set E. §2 L80 says "E is the header's `end`", L78 lets a request set only S ("a request may set a later one"), and REQ-CLOCK (L433) checks only S | V2-P19 | V2-P19 |
| R08-8 | real | what loads | ENGINE.md:219 "V's own line, else a line `of V` … (at most one, `LOAD-ARREARS`)" reads as if V's own line silently wins. §5.3 L554 refuses "two `arrears` lines that cover one pay for one version's change", and L942 lets W answer only "where V's own lines don't cover them". DECISIONS 33 (L415) | V2-P20 | V2-P20 |
| R08-9 | not real | — | L897: a pay is a state pay if "its amount, date or condition makes a state read". A pay has other parts that can read a def: `for expr` (L157) and a binding's `where expr` (L166). A pay that reaches a state read only through one of them is a flow pay that is reached, and `LOAD-STATE` (L922, L1582) refuses it. The reader's `pay x amount 0.1 * bal(konto)` is a state pay and loads under both texts | — | — |
| R08-10 | real | code (refused or allowed) | ENGINE.md:179 "a holding below 0 units is refused" leaves out L701 "unless the holding is short (N-511): a row of the built-in input `short[x] = true` allows it" and §8.3 L1623 "or marks the holding short" | V2-P17 | V2-P17 |
| R08-11 | real, wording only | wording (the figures agree) | ENGINE.md:222 "dated V.known" against L943 "dated on or after V.known. Every `flows` over a span that reaches V.known or later is a hole". Both make holes of exactly the spans and balances from V.known on (ENGINE.md:231 "from 2028-03-01 on") | V2-P21 | V4-P4 (for V2-P21) |
| R08-12 | real, wording only | wording (cost document) | PERFORMANCE.md:30 leaves out anchors. ENGINE.md:177 says "only days with movements or anchors cost anything" | V2-P22 | V2-P22 |
| R08-13 | real | code (extra `WARN-UNREAD`) | ENGINE.md:117 says "items nothing reads", and the item graph (L111) holds bands, fills, draws, pays and asks. §8.5 L1637 limits the warning to "`<source, input, def or place> is read by nothing`". Read literally, every ask would warn | V2-P23 | V2-P23 |

#### V3: R03 and R06

| Id(s) | Verdict | Effect | Settling text or defect (1.0 lines) | Patch | In 1.0.1 |
|---|---|---|---|---|---|
| R03-1 | real | what loads | §5.5:582 doesn't say when `LOAD-FORKS` is counted. :577 says the axes are "found at run time". ENGINE.md:120 counts them "statically". A load refusal can't depend on run-time reads (ENGINE.md:105: "When it finishes, no failure of structure remains"). Static is right. No model is newly refused: the most any ask reaches statically is LH-16's 32 (`instances`, `abstract_range`), and LH-15's whole product is 64. | V3-P1 | V3-P1 |
| R03-2, R06-7 | real | what loads | §5.3:556 ("the same keys, the same known date and overlapping spans") disagrees with §4.6:473–476. (a) Under §4.6, a row with a span beats one without at step 2.4, so they don't tie. (b) Under §4.6, rows with as many stated keys but different keys do tie (`[a, *]` and `[*, b]` at (a, b)). The other places that use the code rely on §4.6: §6.3:1218 ("two rows tied after precedence") and OPERATIONS.md:98. A script over the 79 models and the household found no pair that the §4.6 reading newly refuses. The near-misses, LH-06 rows 147/148 and LH-13 rows 255/294, 271/419, don't meet in index order. | V3-P2, V3-P3 | V3-P2; V3-P3 |
| R03-3 | real | what loads | §2.5:247 refuses a key literal that is also a top-level id, with no exception. §5.6:603 makes place ids the keys of `place`. So the household's `opening[konto] = 24000` (places.tally:43) is refused if read literally. `tools/loadcheck.py`'s doc already makes the exception ("the keys of `place` are the place ids, so they are not"). | V3-P4 | V3-P4 |
| R03-4 | real | figure (the law date of an `as known` ask) | §4.3:405 rule 4: "the ask's cut K: the model's `on`, or the D of its `as known D`". Line :408 adds: "Rule 4 makes a view as known in 2020 read its undated law at 2020". The design record agrees (gaps/resolution.md G50: "Rule 4 becomes the ask's cut K"). §5.1:503's "an ask that gives none" can be read as "gives no date" alone, which gives the model's `on`. The §4.3 reading is right. CH-04, LH-05, LH-14, RC-08 and CB-03 have `as known` asks, and none of their expected values reads undated law at the top. | V3-P5 | V3-P5 |
| R03-5 | real | what loads (or a figure) | The grammar at :138 gives a band row `rowmeta`, which allows `governs`. §5.3:540 and §5.8:660–663 give a table only its version's span ("never mixes brackets of two years"). §5.8 is right. No band row in the models, the household or the changes has its own `governs`. | V3-P6 | V3-P6 |
| R03-6 | real | what loads | §5.3:554 and §8.3:1586 refuse "two `arrears` lines that cover one pay for one version's change". §5.13.3:942 and ENGINE.md:219 use a later `of V` line only "where V's own lines don't cover them", so an `of V` line for a pay V covers would load and never act. I chose refusal. It matches §5.3, §8.3 and OPERATIONS.md:159 (`of V` "answers back pay an earlier version V left open"), and it fits the rule that a line which never acts is silent. The patch also says how far a line without `for` reaches, which decides when two lines cover one pay. LH-14 is the only `of` user, and its `repeal_v` has no line of its own, so it is unaffected. | V3-P7, V3-P8 | V3-P7; V3-P8 |
| R03-7 | real | code or message (how many warnings; which kinds) | §5.9:700 says "the first such day". §8.5:1635 and ENGINE.md:180 say "each stretch". smoke/HH4-a.md:188 and HH4-b.md:314 and :824 expect one warning per stretch. ENGINE names only "An account", but §5.9 covers `cash`, and so does the inventory's N-1607 ("an account or cash below zero"). | V3-P9, V3-P10 | V3-P9; V3-P10 |
| R03-8 | real, wording only | wording | PERFORMANCE.md:31 defines **P** as the number of places. The per-place lookups cost O(log Mₓ) (PERFORMANCE.md:69–70, :287, :295). | V3-P11 | V3-P11 |
| R03-9 | real (one code); the rest wording | code or message | (a) §5.1:506 leaves out the missing `start`/`end` case that §5.1:504 and §8.3:1565 have. (b) §8.3:1572 leaves out "or an open index", which §5.3:552 has. (c) For an undeclared unit, §5.4:562 says `LOAD-UNIT`, but a unit is a top-level id (§2.5:242), and `LOAD-NAME` is `unknown id <x>` / "declares it" (§8.3:1569). §8.3 has no `LOAD-UNIT` message that fits. I chose `LOAD-NAME`. No fixture names either code for this. (d) §5.5:575's mode-axis `LOAD-UNIT` has no §8.3 message. (e) §5.3:554's "`for` naming something that isn't a pay" has no §8.3 message. | V3-P12, V3-P13, V3-P14, V3-P15, V3-P8 | V1-P15 and V5-P9 (for V3-P12); V5-P6 (for V3-P13); V3-P14; L6-P24 (for V3-P15); V3-P8 |
| R03-10 | real, wording only | wording | `MODE` at :210 and the list at :106 give six modes. coverage.md:19 and :327 (R3) record that D7 removed `half-down`. BUILD-ORDER.md:84, DECISIONS.md:96 (U-03) and DECISIONS.md:383 (decision 11) still say "seven". | V3-P16, V3-P17 | V2-P4 (for V3-P16); V2-P4 (for V3-P17) |
| R06-1 | real | what loads | §2.5:242 (`LOAD-DUP`): source and version `pit_2028`, `pit_2027_retro` and `car_loan`. §2.5:247 (`LOAD-KEY`): version `ben_cogs` with key `ben_cogs`, and version `ana_berlin` with key `ana_berlin`. I confirmed all five with `loadcheck.py` on scratch realities. Renaming the versions changes one line each and no count. | V3-P18 | V3-P18 |
| R06-2 | real | figure (which results are holes) | As printed, change 5 has three problems. `wage_eur` has no `when`, so every job has an instance from 2027-01 whose amount is the stub's hole. `salary_eur` has no row for the other jobs. `salary[ana_berlin]` has no row, so `wage[ana_berlin, m]` is a gap-rule hole (§4.6:477). The same change in `amendments/changes/5-job-abroad.md` adds all three guards and says why they are needed. The new keyed place also has no `opening`, so by §5.9:711 its `bal` is a hole from S. | V3-P19 | V3-P19 |
| R06-3 | real | figure | Change 1: `salary[ben_cogs]` has no row for 2027-01..2028-02. So `wage[ben_cogs, m]` is a hole there, and the hole reaches `konto[ben]`: a hole movement for the instance of 2028-02-10, and new instances before 2028-02-10 whose amount is a hole (ENGINE.md §5 step 3, "When V makes an instance exist"). The household writes a zero row before a start (`amendments/changes/1-new-client.md`: `fee_net[client_c] = 0 governs 2026-01..2027-01 "przed umową"`). Change 4: `pit[ola, 2029]` reads `residence[ola, 2029-01]`, which has no row, so the household's 2029 `pit` totals are holes. | V3-P20, V3-P21 | V3-P20; V3-P21 |
| R06-4 | real | figure (which year is a hole) | The rule at :1276 reads `residence[p, month_of(start_of(y))]`, the January of y. `residence[ana]` is `de` only from 2030-07 (:1372). So the hole starts with 2031, not 2030. | V3-P22 | V3-P22 |
| R06-5 | real, wording only | wording | "There are only three kinds" (:1216) is false. `LOAD-ARREARS` and `LOAD-MOVES` both refuse versions known the same day (§8.3:1586–1587), and two adds can also meet in `LOAD-CYCLE` or `LOAD-KEY`. The checks themselves are defined in §5.3 and §8.3, and the engine loads the union of the files, so no implementation goes wrong on this. | V3-P23 | V3-P23 |
| R06-6 | real, wording only | wording | :1188 "never changed" disagrees with :1193 and OPERATIONS.md:204. `restate` may fix a locator or a note, or move an item between files, and `OP-EDIT` is defined by kind (OPERATIONS.md:202–205), so the operation is unambiguous. | V3-P24 | V3-P24 |
| R06-8 | real, wording only | wording | BUILD-ORDER.md:26 says "three change kinds". The same file's :147, LANGUAGE.md:1190 and OPERATIONS.md:198 say four. | V3-P25 | V5-P24 (for V3-P25) |
| R06-9 | real | what loads | :1237 lets a rule elsewhere point at a scenario version with `v`. DECISIONS.md:201 L-24 ("an unpublished one only a scenario") and the inventory's N-931 ("an unenacted version exists only as a scenario") need that from a reality def. A def lives wholly in one file (:1172), so a scenario can't add a rule to it. :1237's last sentence and OPERATIONS.md:240 forbid the reality from reading scenario items. The intent is to allow it, with the rule a candidate only inside the scenario. | V3-P26, V3-P27 | V3-P26; V3-P27 |
| R06-10 | real | figure | :1248 and OPERATIONS.md:251 "copy" the versions. A copied version with no `governs` governs every date (§5.3:536). With a later `known` it wins everywhere, so promoting `ben_stops` would set Ben's salary to 0 for every month. The scenario meant `D..` (:1240), and "The scenario changes nothing before D" (:1238). Promotion should write that span. No model, change or key uses promotion. | V3-P28, V3-P29 | V3-P28; V3-P29 |
| R06-11 | real, wording only | wording | :1295 and the table's "0 edited" leave out the `advance` that :1192 and OPERATIONS.md:97 put before each add known after `on`. `amendments/README.md:57` counts it as the one edited line. Both hold once the advance is named as a separate change. | V3-P30 | V3-P30 |
| R06-12 | real, wording only | wording | "Either order" (:1215) is true of adds. An add known after `on` waits for its advance (:1192; OPERATIONS.md:212, "An `advance` is the store's to make … an add whose `known` is after `on`"). | V3-P31 | V3-P31 |
| R06-13 | real, wording only | wording | `amendments/README.md:55` quotes "a restate of the header", which OPERATIONS §2.1 no longer says (OPERATIONS.md:97 says "an `advance` to that day comes first"). README:56 already tells the D6 history. | V3-P32 | V3-P32 |
| R06-14 | real, wording only | wording | `wc -l` over `amendments/household/` gives 1008 lines. Eight rows of the README's table are behind the files: reality 34, pays 133, common 103, employment 105, business 212, wrappers 86, benefits 41, facts/2026 87. | V3-P33 | V3-P33 |
| R06-15 | real, wording only | wording | An account is a `place` section (§5.9). For an existing person it is not "an index addition" (:1199). `add_keys` takes "a person, a job, a loan" (OPERATIONS.md:89), and add-a-fact makes one version, never a section (OPERATIONS.md:92), yet OPERATIONS.md:77 lists "a new account". Change 5 adds its account as a section. | V3-P34, V3-P35 | V3-P34; V3-P35 |
| R06-16 | real | code | :1166 "Exactly one file has it" has no code. §8.3:1565 `LOAD-HEADER` ("completes the header") is the natural home for it. `LOAD-DUP` doesn't fit, because the `model` id is not in the namespace list (:242). | V3-P36, V3-P37 | V3-P36; V3-P37 |

#### V4: R04 and R09

| Id(s) | Verdict | Effect | Settling text or defect (1.0 lines) | Patch | In 1.0.1 |
|---|---|---|---|---|---|
| R04-1 | real, wording only | none (an example) | The example's `y2016` has its own label and no `when`, so whenever rm2016 governs, `base` and `y2016` both win: `RUN-RULES` (LANGUAGE:736). The rule a formula change needs is "a rule with the same label" (LANGUAGE:743). | V4-P1 | V4-P1 |
| R04-2 | real, wording only | none (a cost figure; no code exists for a sixth rule) | Counted over `models/`: 514 defs with 1 rule, 285 with 2, 37 with 3, 13 with 4, 4 with 5, 3 with 6 (UL-06 `emp_rate`, `dir_rate`; LH-06 `aa17`). | V4-P2 | V4-P2 |
| R04-3 | real, wording only | none (a cost figure) | The §5.11 example (sx = dx, t from o−1 to o+10, 12 keys) gives 608 arcs over 30 years. CB-05's `spouses` (t − o = −1 or 1..10, four spouse pairs) gives at most 30 × 11 × 4 = 1 320, and 1 096 inside 30 years. The 1 300 is CB-05's, not the example's. | V4-P3 | V4-P3 |
| R04-4 | real | what is a hole (a figure: hole or number) | ENGINE:222 "dated V.known" stands alone against LANGUAGE:943 ("dated on or after V.known. Every `flows` over a span that reaches V.known or later is a hole"), ENGINE:231's own example ("every `flows` … from 2028-03-01 on would be a hole") and the question itself ("is back pay due, and when?"). The reader's input doesn't show the difference: a span containing V.known is a hole either way. The difference is a span lying wholly after V.known (a hole only under LANGUAGE), and the balances after a later anchor (ENGINE:253: a movement that may fall after D keeps them holes). | V4-P4 | V4-P4 |
| R04-5 | real | what loads | The reader's reading B isn't supported: `flows` of a state pay is a state read (LANGUAGE:913), so `settle` is a state pay by LANGUAGE:897 and runs in the walk. The real defect is in ENGINE:118: "propagate the mark to every def and pay that reaches it, and refuse a flow pay that is reached" reads as making every reaching pay a state pay, which leaves the refusal vacuous. LANGUAGE:897 makes a pay a state pay only by its amount, date or `when`, so a pay whose places' keys, `for` or bindings reach a state read stays a flow pay that makes one, refused by LANGUAGE:922. Input: `to wcash[argmin(w in wrapper: bal(wcash[w]))]` loads under the literal ENGINE and is `LOAD-STATE` under LANGUAGE. | V4-P5 | V4-P5 |
| R04-6 | for the user | a figure (room, and the replay key on an altered input) | LANGUAGE lists `next_on` among the state reads (911) and exempts only `flows(p, span)` of a flow pay (915); ENGINE:166 says flow pays may read other flow pays through `flows` and `next_on`. No model uses `next_on` (coverage K7). | for the user | LU-6 |
| R04-7 | real | a figure | ENGINE:190: "Records and anchors after t are never used". HH4-b §4 relies on it ("never later records", room 12535.30). LANGUAGE:910's "counting only flow pays and records" would count Sep 16–30 records in the Sep 15 look-ahead. | V4-P6 | V2-P16 (for V4-P6) |
| R04-8 | real, wording only | only the order of an intermediate `RUN-PLACE` | ENGINE:171–172 books every record of the day (step 1), then the flow movements in pay id order (step 2). Unexplained records have no pay id, so LANGUAGE:928's single pay-id order can't be the rule. LANGUAGE:907 leaves records out of `bal(x)`. | V4-P7 | V4-P7 |
| R04-9 | real, wording only | none | In PERFORMANCE, P is the number of places; the search is over one place's change points, O(log Mₓ) (PERFORMANCE:69–70, 287). | V4-P8 | V3-P11 (for V4-P8) |
| R04-10 | not real | — | LANGUAGE:1537 (`HOLE-IMPRECISE`: "a state pay's date is imprecise (§4.4)") and ENGINE:174 ("its date must be precise (`HOLE-IMPRECISE` otherwise)"). An imprecise `on` loads (LANGUAGE:889) and is a hole on a state pay; the two texts agree. | — | — |
| R04-11 | real, wording only | none | ENGINE:127–131 (§3.2): context = (law date class, cut class, axis options, pin set, scenario, path [, walk point]). LANGUAGE:815's list leaves out scenario, path and the walk point, which LANGUAGE §5.13.1's walk-bound defs need. | V4-P9 | V4-P9 |
| R09-1 | real | what prints | ENGINE:280 lists a balance line only for a `stated` row that differs or is a hole; ENGINE:252 and LANGUAGE:1132 give one line per `stated` row, LANGUAGE:1554 lists a 0 difference in the register, and Decision 40 (DECISIONS:427) says "one line per statement". | V4-P10 | V4-P10 |
| R09-2 | for the user | a figure (the modelled past, the ledger's groups) | ENGINE:171 and 246 put a record that explains a state pay instance "at the instance's point", without saying which day when the record's date differs. OPERATIONS:128 matches records within `match_days[p]` days, so this happens. | for the user | LU-7 |
| R09-3 | real | a figure (group totals) and what prints | A promised instance with records summing to less is both "explained by records" (ENGINE:276) and "partly fulfilled" (279). Both lines would count the instance twice in the group's model total (ENGINE:285). P-06 (DECISIONS:151) and N-1206 need the partly fulfilled line and its question. | V4-P11 | V4-P11 |
| R09-4 | for the user | a figure (group totals and order) | ENGINE:282 says where unexplained records and balance lines go under `by f`, not where a line with no record, or with records that disagree on f, goes. | for the user | LU-8 |
| R09-5 | real | a code | ENGINE:432 makes an ask given with the request that fails to load `REQ-ASK`; OPERATIONS:313 and 329 give its own load codes. BUILD-ORDER:9: a fixture gives "exactly that code". | V4-P12 | V4-P12 |
| R09-6 | real | a code (refused or runs) | ENGINE §2 (78, 80): "E is the header's `end`"; only S may be set by a request, and `REQ-CLOCK` (433) checks S only. ENGINE:35 lets the request set E. | V4-P13 | V2-P19 (for V4-P13) |
| R09-7 | real | what loads | ENGINE:429: "A load error in a scenario file refuses only that scenario". LANGUAGE:1523 and 1560 state the general rule without it. The specific exception is taken. | V4-P14 | V5-P5 (for V4-P14) |
| R09-8 | real, wording only | none once read with LANGUAGE | LANGUAGE:1155 and 1627 also refuse a record explaining an instance that doesn't exist; ENGINE:249 leaves that case out. | V4-P15 | V4-P15 |
| R09-9 | real, wording only | none | From tmp/design/inventory.md: N-1201 is "Before the run's start, real records are the past…", so ENGINE:243 and LANGUAGE:1110 are right. Determinism is R-16 / N-1401 (DECISIONS:134, inventory:315). "A released version never changes" is C-15 (DECISIONS:278). LANGUAGE:1499, 1648 and ENGINE:393 cite N-1201 wrongly. | V4-P16 | V4-P16 |
| R09-10 | real, wording only | none (the verdict holds) | h = 1077/365 = 2.950685, g = 1.04³ − 1 = 0.124864, c = h·(1/2 + g/3)/(1 + g/2) = 1.504241 > 1.50. It prints as 1.50 (ENGINE:365: c to 0.01, half to even), and it is ≤ 2. | V4-P17 | V4-P17 |
| R09-11 | not real | — | ENGINE:362 defines c from "the latest law date at which a `held` or `assumed` entry of its register is read". Every def evaluation carries a law date (ENGINE:92), and the read sets of §3.2 hold it. The engine has that date even though the printed `assumed:` entry doesn't show it. For the reader's `growth[2040, p]`, c ≈ (2040-01-01 − 2026-10-19)/365 ≈ 13.2, not sayable. A read at a past law date gives c < 0, which passes; that is harmless. | — | — |
| R09-12 | real, wording only | none | ENGINE §10 (399–409) has no "movement" row. A pay instance's row covers its corrections and holes, and the balance row lists every movement since the anchor. A record's matching is `drill(line)` (OPERATIONS:350). | V4-P18 | V4-P18 |
| S-1, S-2 | S-1 real, wording only; S-2 real | a figure on a realistic input; no answer key changes | Yes, a booked state pay's amount can change when a later cut sees records or an anchor dated before it. Its state reads come from the walk at cut K: ENGINE:171 (records dated t, t before K), 173–176 (step 3; 176: modelled past "computed against the booked balances, which come from the records"), §6.2:260, §9 inv 5's second sentence, and LANGUAGE:1134 ("A state pay after D reads the anchored balance"). HH4-b §4 (records known 10-02 read by the Sep 15 state pays; `replay_september`'s invest +535.30, mak 6535.30) and HH4-a §5 item 3 rely on it. What stays "as known then" is the law and facts (κ₀, ENGINE §5 step 1) and the look-ahead (cut min(t, K), ENGINE:190). The title of inv 5, LANGUAGE:935 and 946 ("with what was known then"), ENGINE:212 and 254 ("as booked") read as if the amount were fixed. | V4-P19 | V4-P19 |

#### V5: R07 and R10

| Id(s) | Verdict | Effect | Settling text or defect (1.0 lines) | Patch | In 1.0.1 |
|---|---|---|---|---|---|
| R07-1 | real, wording only | wording (a count) | The grammar (L:210) and the §7.2 table have six modes. `coverage.md`:19 and :327 (R3) record that `half-down` was removed in D7. BUILD-ORDER:84, DECISIONS:96 (U-03) and decision 11 (DECISIONS:383) still say "seven". | V5-P1 | V2-P4 (for V5-P1) |
| R07-2 | real, wording only | wording | $0.10 is 10⁻¹, so it doesn't illustrate "steps that aren't powers of ten". | V5-P2 | V5-P2 |
| R07-3 | real | what loads (`LOAD-TO`) | §7.3 (L:1473) says every continuous distribution needs `to`. The §5.14 table (L:988) says `uniform` is exact, and its cdf, pdf and quantile are rational. The table is the more specific text. | V5-P3 | V5-P3 |
| R07-4 | real, wording only | wording | L §8's "four things" leaves out the bound (§5.17, decision 29, E:139 "a value, a hole, a bound or a refusal") and counts "a value with warnings" as a fourth kind. | V5-P4 | V5-P4 |
| R07-5 | real | what loads | L:1523 and L:1560 say a load refusal stops the model. E:429 says "A load error in a scenario file refuses only that scenario". E's reading is the one that keeps a stale scenario (one the reality moved under, O:254) from stopping the reality. L:657 and the `LOAD-BAND` message say the same as L:1523. | V5-P5 | V5-P5 |
| R07-6 | real, wording only | wording (catalogue text) | L:552 (an open index) and L:702 (a pay *to or from* an asset) are cases of `LOAD-KIND` that the catalogue's examples at L:1572 leave out. The message template covers them already. | V5-P6 | V5-P6 |
| R07-7 | real | code (a hole with no code) | The `next_on` hole (L:911, `no further <p> after <d>`) and E:78's `before the run's start` have no hole code. Gate 5 needs a code for each. The patch puts them under `HOLE-MISSING`, which keeps the count at 87. | V5-P7 | L6-P23 (for V5-P7) |
| R07-8 | real | what loads (unit adoption) | L:1442's list of whole values leaves out "an `if` branch", which L:293 includes. Models rely on L:293 (`if … then X else 0` in UL-13, MP-03, UL-08, RC-02, MP-15, CH-05, and the household's pays.tally and employment.tally). The `base - 8000` example in L:293 is not a defect: a band row may hold any literal (L:1440). | V5-P8 | V5-P8 |
| R07-9 | real | code | `on` is "required" (L:503), but no code or message covers its absence. The patch uses `LOAD-HEADER`, following the precedent of a missing `tally`. | V5-P9 | V5-P9, part by V1-P15 |
| R07-10 | not real | — | Settled by E:433, "`REQ-CLOCK` \| S outside the header's [start, end], or `as known` after `on` \| the request is refused". This applies to every ask a request answers, saved or given (E:33). `LOAD-KNOWN` (L:452) is about versions, and the ask adds no version. A side note, not this candidate: an `as if {known D}` pin whose D is computed and falls after `on` has no code. | — | — |
| R07-11 | real | message | L:1480 quotes as the message a text that isn't the catalogue's message (L:1617). | V5-P10 | V5-P10 |
| R07-12 | real | code (`LOAD-TO`, `RUN-DOMAIN`) | `factor(r, n, P)` with a non-integer n is a non-integer power, but it isn't in §7.3's list. `pow(x, q)` with x < 0 and a non-integer q has no refusal. `RUN-DOMAIN`'s message names only ln and sqrt. No model is touched: every exponent is `int`-typed, and every non-integer `pow` has a positive base. | V5-P11 | V5-P11 |
| R07-13 | real, wording only | wording | L:1529 says "only", but two more stops exist in the text: `missing(r)` (L:791, never a hole) and an anchor, after which earlier hole movements no longer reach the balance (L:1133). `covered()` reads no hole, so it isn't a stop. | V5-P12 | V5-P12 |
| R07-14 | not real | — | Settled by E:180–181, which give two independent tests ("An account that goes below zero prints `WARN-NEGATIVE` on the first day of each stretch. A free balance below zero on a day whose hold is above 0 prints `WARN-HELD`."), and by L:1636, where "A day with no hold prints `WARN-NEGATIVE` only" limits only days with no hold. For balance −5 and hold 3 on three days, both texts give `WARN-NEGATIVE` on day 1 and `WARN-HELD` on days 1 to 3. | — | — |
| R07-15 | real | code or message (which items warn) | E:117's "items nothing reads" would include bands, fills, draws and pays. L:1637's message lists "source, input, def or place". The patch keeps the catalogue's list, the one with a message a fixture can match. | V5-P13 | V2-P23 (for V5-P13) |
| R07-16 | real | what loads | The grammar has `solve` only in `askbody` (L:163). Yet L:1059, `LOAD-ASKONLY`'s note (L:1595) and decisions R-14, R-22 and U-16 allow it in a rule with `v`. Such a rule is `LOAD-PARSE` today, and its bracket (say [0, 1000000]) breaks the literal rule (L:1435–1438). No model uses it. The patch is larger than minimal, because grammar, parse rule, printer and literal rule must agree. | V5-P14 | V5-P14 |
| R07-17 | real, wording only | wording | L:1648 says "A released version never changes", while DONE rule 1 (DONE:9) patches the frozen text as `tally 1.0.1`, …. The patch says what those patches are and that headers still state `tally 1`. | V5-P15 | V5-P15 |
| R07-18 | real | figure (`flows` after V.known) | E:222 dates the hole movement exactly at V.known. L:943 says "dated on or after V.known", and "Every `flows` over a span that reaches V.known or later is a hole". E's own example (E:231) agrees with L, as do change 3 (L:1339) and the question "and when?". With a hole date, a `flows` span that ends after V.known is a hole even when it starts after V.known. | V5-P16 | V4-P4 (for V5-P16) |
| R07-19 | real, wording only | wording | L:74 says "No exponent", but the grammar's `UNIT` (L:209) allows `1e-N`, and §7.2 uses `1e-9`. | V5-P17 | V5-P17 |
| R07-20 | real | message | Two uses of `LOAD-KEY` (L:466, a range out of order; L:248, one bound variable used as a key of two indexes) have no message in the catalogue (L:1571). | V5-P18 | V5-P18 |
| R07-21 | real, wording only | wording | L:1651 says "Every result records the `tally` version and the engine build", but E §1's result has only their hash. | V5-P19 | V5-P19 |
| R10-1 | real, wording only | wording | O:271 says "Claude or the household confirms each proposal". Only the household confirms (O:30, O:343 "the household only"). The proposals at O:270 are rows of the draft. | V5-P20 | V5-P20 |
| R10-2, R10-3 | for the user | what loads (what applies without `confirm`) | O:30 names one exception, exact matches. O:212 makes the advance "the store's to make", and O:123–124 and O:138 treat a statement's records as already in the reality. The two questions are coupled: an import known after `on` is `LOAD-KNOWN` until an advance applies. | U1 | LU-9 |
| R10-4 | real | what loads (in a scenario run) | O:29 says "no reality run reads it", against O:184, O:241, O:379 and C-09 (DECISIONS:272, "warns in a scenario"). The decision settles the effect. O:29 also doesn't say how a scenario run reaches a draft law part. | V5-P21 | V5-P21 |
| R10-5 | real, wording only | wording | The gate reads `check` asks and pays "in the part" (O:174, O:177). M-07 (DECISIONS:50) lists "`check` asks" in a law part, and BUILD-ORDER M3 gates "the household's Polish PIT part with its published checks". The household's law parts hold pays. L:1167 and L:1177 give the short list. | V5-P22 | V5-P22 |
| R10-6 | real, wording only | what `catalog` returns | A-01 (DECISIONS:290) makes the catalogue LANGUAGE §2, §5 and §8, with a reference card of about 13k tokens cut from them (BUILD-ORDER:152). O:340 says it is the whole of LANGUAGE.md and ENGINE.md. The decision wins. | V5-P23 | V5-P23 |
| R10-7 | real, wording only | wording | BUILD-ORDER:26 says "the three change kinds"; decision 6, O:198 and BUILD-ORDER:147 say four. | V5-P24 | V5-P24 |
| R10-8 | real (part for the user, U2) | which records link or are proposed; what loads | Three gaps. (a) No type or unit is given for `match_amount`; "within `match_amount[p]` of the record's" reads as an absolute amount. (b) A pay with no rows "gets proposals only", but no window is given for it. (c) Found while verifying: "per pay" inputs need an index whose keys are the pays, and none exists. L:113 lists the built-in indexes `place bank_account bank_party`, and L:247 refuses a key that is also an id. The prototype loader (`tools/loadcheck.py`:75–76) keys them by `place`. (a) and (b) are settled by their own wording and patched; (c) is U2. | V5-P25; U2 | V5-P25; LU-10 |
| R10-9 | for the user | figure (a link, and its question) | Criterion 2 (O:127) is skipped when the pay's other end isn't an outside place, while O:136 makes every record with an unmapped counterparty unexplained. For a transfer between the household's own accounts, the two give different bookings. | U3 | LU-11 |
| R10-10 | real | what is proposed | "Candidates" is the in-window set (O:125–129). So "near: one best candidate" overlaps "ambiguous: several candidates". The patch reads "one candidate" as a set of one, for both exact and near. That is the defined term, and decision 21's "matches the importer is sure of". A same-day, same-amount instance among two candidates is therefore ambiguous, never written. | V5-P26 | V5-P26 |
| R10-11 | real | message (the sanity line) | O:296 bounds the first day of difference by "the amendment's earliest `governs`". A `revise` or an `advance` has no `governs` (O:203, O:205), and neither has an `add` of sections alone. Changes 1, 2, 4, 5 and 6 rely on the bound for versions; change 6 relies on "no `governs`" meaning every date. | V5-P27 | V5-P27 |
| R10-12 | real | figure (a re-opened analysis) | O:327 says a re-run "gives the same result", but the saved record keeps no record time R. A re-run on the current reality, after a new fact or an advance, differs. O:26 (rebuild at any R) and the hash at E:41 make R the reading that holds. | V5-P28 | V5-P28 |
| R10-13 | real, wording only | wording (the date an author enters) | Settled by L §4.5 (L:435–436): law is known "from the day its text was made public in the form that binds", which is the official journal where it binds only once published, "Otherwise … its assent". Decision 32 says the same. O:157's "published or enacted" offers both dates as alternatives. | V5-P29 | V5-P29 |
| R10-14 | real | code or message | `reason` is required for a `revise` (O:47, O:196, O:205), but its absence has no code or message. The patch adds a message to `OP-EDIT`, the code of the `revise` validation, so the count stays 87. | V5-P30 | V5-P30 |
| R10-15 | real | code or message | S-08's failure is "returned as a warning on the scenario" (O:243), but no warning code exists for it. A new code would make 9 warnings and 88 codes, against DONE gate 5, the user's contract. The failure is the check's own printed result, `differs: …` (L:1028). | V5-P31 | V5-P31 |
| R10-16 | real | code (which one fires) | `OP-CURRENCY` (O:142, O:374) and `LOAD-UNIT` (L:1154) both cover a record in the wrong currency. O:50 puts an operation's own checks at step 2, before load at step 4, but nothing says the pipeline stops at the first step that fails. Gate 5 needs a fixture that gives "exactly that code … and nothing else". | V5-P32 | V5-P32 |
| R10-17 | real, wording only | wording | L:1216 says "only three kinds", but those are conflicts between additions. `OP-BASE` (O:54) is the store's conflict between edits against a changed base. L:1220 itself says additions to different items merge cleanly. | V5-P33 | V5-P33, part by V3-P23 |
| R10-18 | real, wording only | wording | "Bands" in O:288 and O:300 (and in PERFORMANCE:233, in the paragraph where line 230 uses "bands" for law tables) means path statistics, not law tables (L §5.8). The same use in DONE:24 is flagged, not patched (the user's contract). | V5-P34 | V5-P34 |
| R10-19 | real, wording only | wording | O:315 uses `save`, but §8's list of inputs (O:309–310) doesn't name it. | V5-P35 | V5-P35 |

### R11–R13, as the lead triaged them

| Candidate | Verdict | Patch | In 1.0.1 |
|---|---|---|---|
| R11 PERFORMANCE "exact actuarial values to 4–5 decimals" as a declared bound | real, wording: LH-18's are exact (BOUNDS 49); LH-17 step 6 needs a table the case lacks | L6-P1 | L6-P1 |
| R11 the flow pay cost's cut | real, wording: `on` is read at K (E §5 step 0) | L6-P2 | L6-P2, part by V2-P15 |
| R11 O(log P) for `bal`/`flows` | real, wording | = V3-P11 | V3-P11 |
| R11 PERFORMANCE §4.3 "re-walks from next year" | real, wording: a look-ahead window reaches back | L6-P4 | L6-P4 |
| R11 L:910 "and records" | real | = V2-P16 | V2-P16 |
| R11-7 walk day misses anchors | real, wording | L6-P6 | V2-P22 (for L6-P6) |
| R11 "396 checkpoints of 13 places" | real, wording | L6-P7 | L6-P7 |
| R11 BO:262 grouping aggregates is in P §7, not §10 | real, wording | L6-P8 | L6-P8 |
| R11 P:267 transcendentals list short of L §7.3 | real, wording | L6-P9 | L6-P9 |
| R11-5 | not real | — | — |
| R12 decision 11 "LOAD-NAME" for a clash | real, wording: a clash is `LOAD-DUP` | L6-P10 | L6-P10 |
| R12 conversion cited at L §5.13.1 | real, wording: it is §5.13.3 | L6-P11 | L6-P11 |
| R12 T-15 "`plan` between two record times" | real: `plan` diffs a draft against the current reality only (O §7) | L6-P12, gap row | L6-P12; open gap, below |
| R12 L-39 cites O §3 step 2 | real, wording: expected answers are step 3's checks | L6-P13 | L6-P13 |
| R12 R-13 `flows(buy, ..d - 1)` reaches before S, a hole | real | L6-P14 | L6-P14 |
| R12 "= 1.50 years" is 1.50425 | real, wording | L6-P15 | V4-P17 (for L6-P15) |
| R12 "taken differently" missing from the statuses | real, wording | L6-P16 | L6-P16 |
| R12-1, R13-6 seven modes | duplicate | V2-P4 (half-down restored) | V2-P4 |
| R12-9, R13-7 three change kinds | duplicate | V5-P24 | V5-P24 |
| R13 BUILD-ORDER placement: cases and changes in milestones that lack their constructs | real | L6-P17 (BO-1) | L6-P17 |
| R13 M7 "0" while `montecarlo/` adds two keys | real | L6-P18 | L6-P18 |
| R13 M8's refusals with no code | real | L6-P19, gap row | L6-P19; open gap, below |
| R13 BO:8 "one planted cause" (H0 has none, H9 two) | real, wording | L6-P20 | L6-P20 |
| R13 coverage K12/K13 counts are 1.0's | real, wording | L6-P21 | L6-P21 |
| R13-9 gate 1's "CH-03 step 14" (no such step; four bound rows) | real: **DONE.md, the user's** | D-1 (pending) | V1-P25 (D-1, the user's) |
| R13-10 gate 7's evidence path | real: **DONE.md, the user's** | D-3 (pending) | LD-2 (D-3, the user's) |
| R13-11 gate 6's no-foresight scope omits C-13 | real: **DONE.md, the user's** | D-2 (pending) | LD-1 (D-2, the user's) |
| R13-16 gate 12's 10× applies only to the cold run in P §8 | real: **DONE.md, the user's** | D-4 (pending) | LD-3 (D-4, the user's) |
| R13-5 | not real: goldens.md:16 "A plain forecast sets S = K" | — | — |
| R13-13 | not real | — | — |

### The verifiers' items for the user, decided by the lead

Each was decided on the verifier's recommendation; none changes an answer key.

| Item | Decision | Patch | Hunks |
|---|---|---|---|
| V1 R05-6: the SE of `mean` and `quantile` | `mean`: s/√N, sample variance over N − 1, to 1e-6; `quantile`: none | LU-1 | 3 |
| V1 R05-7: the canonical order of a draw with no time key | first; periods by their first day | LU-2 | 1 |
| V1 R05-8: carry with a draw that has no `plan` | the share prints, and the verdict fails with a reason | LU-3 | 1 |
| V1 R05-9: a `stated` row longer than a day | anchors at its last day | LU-4 | 2 |
| V2 R02-4: a price year through a monthly index | refused, `LOAD-KEY` | LU-5 | 2 |
| V4 R04-6: is `next_on` a state read? | no, for a flow pay | LU-6 | 1 |
| V4 R09-2: a record dated after its state pay instance | booked on its own day | LU-7 | 2 |
| V4 R09-4: lines without a record under `by f` | by pay id, printed `pay <id>` | LU-8 | 1 |
| V5 U1: what applies without `confirm` | statement imports and the store's advances | LU-9 | 4 |
| V5 U2: the key of `match_days` / `match_amount` | a built-in `pay` index | LU-10 | 4 |
| V5 U3: an unmapped counterparty between own places | links (criterion 2) | LU-11 | 1 |

### The seven changes (C1–C7)

The findings of the change compares, merged across each pair. Most concern registers or presentation; every pair used the same reading, so no expected value depends on the choice. The LR patches write those shared readings into the text.

| Finding | What it is (1.0 lines) | In 1.0.1 |
|---|---|---|
| C1-F1 | `held` judged against the ask's K or the look-ahead's cut (registers only; 18 cells) | LR-1 |
| C1-F2 | A key added after a booking cut. **Patch:** LANGUAGE §5.3 (L:548) gets "At a booking cut before V's `known` the key doesn't exist: an instance at it counts as A₀ = 0 (`ENGINE.md` §5 step 1), and the modelled past at that cut doesn't list it (§6.2)" | LC-1 |
| C1-F3 | Does a register-only difference count for plan sanity? Here the check can't fail (O:296) | LR-3 |
| C1-F4 | README:55–56 stale; L12 and the J-ids are defined nowhere in the spec (only in D8.md) | LR-6, LX-8 |
| C1-F5 | Change file:34's closed-world check omits forecast_2026's keyless row | LX-1 |
| C1-F6 | Day K is neither past nor planned (E:239 vs E:225) | open gap, below |
| C1-F7 | The change file's words, where both evaluators disagree with them | LX-1 |
| C2-F1 | an `add` that edits `on` is `OP-EDIT`; the change is an advance, then an add | LR-6, LX-2 |
| C2-F2 | the gate's coverage step fails on an untested law part; the README waives it | L6-P22 |
| C2-F3 | what "anything differs" counts in sanity; an advance has no `governs` | LR-3 |
| C2-F4 | the cut of a walk point's register after K passes it: K or min(t, K) | LR-1 |
| C2-F5 | a late entry (known before the old `on`, added later) is absorbed unmeasured, and breaks "as known D sees the reality as it was"; only R gives the old view | open gap, below |
| C2-F6 | whether `flows`' register lists what it read to drop out-of-span instances | open gap, below |
| C2-F7 | the rule-4 law date of an ask line whose top value is not a def cell: K or its `for` binding | open gap, below |
| C2-F8 | E 97 says two cuts admitting the same versions give the same results, yet K moves past/plan, `held` and the printed cut | LR-2 |
| C2-F9 | `wage_growth` (open span) read after K is `held` by L 1555, but HH4-b lists it as `assumed` only; "read for <period>" is undefined for an input with no time key | open gap, below |
| C2-F10 | 14 lines, not 13 | LX-2, LX-8 |
| C3-F1 | The date of a `HOLE-CHANGED` movement, and which `flows` it makes holes (L:943, L:954, E:222, E:231, L:1339) | V4-P4, LR-7 |
| C3-F2 | Is WARN-BACKPAY printed when every changed booked instance is covered (O:170 vs O:378)? | LR-8 |
| C3-F3 | Inside a look-ahead before K, is `held` measured against K or the look-ahead's cut (L:1555 vs L:910)? C3-a took K as settled by §8.2 | LR-1 |
| C3-F4 | ENGINE:97: a cut class gives the same results, yet `held` and `law date` depend on K's date | LR-2 |
| C3-F5 | The law date printed for top-level reads (`bal`, `flows`, `worth`): K or the bound key (L:405, L:1556)? | open gap, below |
| C3-F6 | The plan's register part (O:294) has no place for dropped or replaced entries; warnings do (O:295) | open gap, below |
| C3-F7 | A law part is read only once `tested` (O:171) and the add runs the gate. Yet `amendments/README.md:7` says the law files are ungated by design, and `pl_business` has no `check` asks. Enforced, every law-version change is refused `OP-GATE` | L6-P22 |
| C4-F1 | The diff states `residence[ola]` from 2028-05 only, and the base's keyless row doesn't cover a key added later, so `pit37_due[ola, 2026..2028]` are holes that reach konto, room and the wrappers | LX-4 |
| C4-F2 | A re-measured past instance with a hole δ, whose key was added later: a `HOLE-CHANGED` hole on the known day, or nothing booked? The question's text for an instance that never existed | LC-1, V4-P4 |
| C4-F3 | Does plan_low count an arrears correction in its window (L:910 vs E:190)? | open gap, below |
| C4-F4 | An add-a-fact has no `arrears` input, and `WARN-BACKPAY` is defined for law versions only (O:378), yet these facts change booked pays | LR-10, LR-8 |
| C4-F5 | The sanity check's "first day" (O:296) is undefined for spanless rows, period-keyed asks and the advance's relabels | LR-3 |
| C4-F6 | The claim adds 8 lines, not 7; `source zus_pue` is out of canonical position; "Kind: add" edits `on`, so it is four amendments | LX-4, LX-8; `zus_pue`'s canonical position: open (presentation) |
| C4-F7 | The law date of dateless asks is K (L:405), so every advance rewrites it | open gap, below |
| C4-F8 | Is `held` measured against the ask's cut or the cell's cut (L:1555)? | LR-1 |
| C5-F1 | No opening row for konto_eur, so it is a hole from S. Fix: `opening[konto_eur] = 0` in berlin_job, as loans.tally:80 does. **Applied in 1.0.1** | LX-5 |
| C5-F2 | Does a δ = 0 version answer a hole `when` at booking cuts before its known date (LANGUAGE:945 vs ENGINE:211/219/223)? If not, O also gets holes from 2026-01-25. To merge with the V-set's reading | LR-5 |
| C5-F3 | `held` and `cut` depend on K, against ENGINE:97's cut classes; README:56 is incomplete | LR-2, LR-1, LR-6 |
| C5-F4 | Whether a hole-`when` instance's amount is evaluated, which decides fx_eur's `assumed`/`held` entries; and the `held` period of keyless fx_eur | open gap, below |
| C5-F5 | The earliest governs is 2026-01, not 2029-07 (change lines 80–81); a 2029-07 bound would have caught C5-F1 | LX-5 |
| C5-F6 | After a mid-year re-anchor, konto's hole returns on the next 15th (the ike cap), not the 28th | LX-5 |
| C5-F7 | The register words don't match §8.2: a stub or a source is not an entry, and five to seven `held` entries go unmentioned | LX-5 |
| C5-F8 | A stale README:55 citation; J91/J93/J104 and L1/L8/L11 are cited but not in the spec | LR-6, LX-8 |
| C5-F9 | How W's modelled-past fx_in prints in the ledger; residuals over an uncovered month (these affect W only) | open gap, below |
| C5-F10 | The change file's words: first day 2029-07-25 under O; the 15th's hole from 2029-08-15; earliest governs 2026-01 | LX-5 |
| C6-F1 | `held` measured against the ask's K or the look-ahead's cut (registers only; C6-b counts 17 cells) | LR-1 |
| C6-F2 | Plan sanity's "earliest governs" and "first day" are undefined here; the bound is vacuous without `governs` | LR-3 |
| C6-F3 | "Kind: add" but the diff edits `on` (OP-EDIT); README:55–56 is stale; README:56 vs O:203 on whether an advance changes results | LR-6, LX-6 |
| C6-F4 | The overpay default that the look-ahead reads is not `assumed` (registers only) | open gap, below |
| C6-F5 | "add_keys, twice", but the second version adds no key; `add_keys` against the open index `loan` | LX-6 |
| C6-F6 | `reduce` has no k guard; the words' reduce-hole "from instalment 10" is from k = 1 | LX-6 |
| C6-F7 | WARN-ENDED belongs to `end`, not to an add | LX-6 |
| C6-F8 | WARN-NEGATIVE end of day vs per movement: konto goes to −4195.21 intraday on 2028-01-10. Merge with V3-P9/P10 | LR-4 |
| C6-F9 | The payout date is a model convention; how a new place's balance difference is reported | open gap, below |
| C6-F10 | J91, J104 and J93 are not in the spec (= C5-F8) | LX-8 |
| C6-F11 | The change file's words: its "Unchanged" list, and taxes_paid 2028, which changes | LX-6 |
| C7-F1 | Is a scenario row listed as `held` (L:1555 vs L:1239)? | LR-9 |
| C7-F2 | The unstated-key test (L:467) cites the row's `known` date, which a scenario row lacks | open gap, below |
| C7-F3 | The plan's register part (O:293–294) has no place for `scenario:` or cleared entries | open gap, below |
| C7-F4 | `WARN-DRAFT` can't be decided, because gate status isn't in the files | L6-P22 |
| C7-F5 | An amount that goes to or from 0: is it a changed movement, or one removed or added (O:291 vs L:888)? | open gap, below |
| C7-F6 | Is the Balances part's "largest difference" signed or absolute (O:292)? | open gap, below |

### The Monte Carlo keys (MC-1, MC-2)

| Finding | 1.0 lines | What it is | In 1.0.1 |
|---|---|---|---|
| MC1-1 | ENGINE.md:353; LANGUAGE.md:1047, 1595 | ENGINE §8 has "the statistic, its standard error, the oracle and the tolerance" each "a def or a row". But a path aggregate is ask-only (`LOAD-ASKONLY`), so the statistic can only be an ask term, and its printed SE is read by no expression. §8 also doesn't say whether the tolerance's SE is the statistic's printed one or the oracle's | LM-1 |
| MC1-2 | LANGUAGE.md:1049–1050; ENGINE.md:318 | `mean` and `quantile` over paths "print a standard error" with no formula. ENGINE §7 says SEs are "computed exactly", but a square root needs a `to` (only `share`'s is "correctly rounded to 1e-6") | LU-1; the key's `q_life` line prints no SE |
| MC1-3 | DONE.md:24; models/LH-17.tally:5; PERFORMANCE.md:356 | "LH-17 … matches its closed form": LH-17's closed form, Eq. 8, is exact only at λ = 0 (eventual ruin, out of reach for paths) and a moment match otherwise. At a reachable cell it is 0.1057 against the exact 0.1496 (script A), so a correct engine fails against it with near certainty (12.3 SE) | LD-2 (D-3, the user's) |
| MC1-4 | ENGINE.md:362 | Carry and horizon of a term that reads no balance and no flow. c = "the latest law date at which a `held` or `assumed` entry … is read, minus K" gives 0 when every read is at K, but the horizon h = d − K has no d: the term reads no day | LM-2 |
| MC1-5 | LANGUAGE.md:1495 vs 175, 206 (and 167) | The test mode's syntax `with theta[*] = theta_mean` doesn't parse: a pin is `ref "=" expr`, a ref's keys are expressions, and `*` exists only in a version row's keys | V1-P6 (LM-5) |
| MC1-6 | ENGINE.md:363–365 | The third test's reason, a pin on a draw, has no text. §8 gives the precision and carry texts only | LM-3 |
| MC1-7 | LANGUAGE.md:1047, 768, 601 | An ask can't read its own N. `count` and `sum` over `path` are neither allowed nor refused: the ask-only list names `share`, `mean` and `quantile`, and §5.10 says only those exist over `path`, in asks. So the SE defs read a row `npaths` that matches `paths 10000` only by hand, and a mismatch gives a wrong tolerance in silence | open gap, below |
| MC1-8 | LANGUAGE.md:1485, 969 | §7.3's ≈ bound widens "each rounded value". A draw is a correctly rounded quantile (§5.14 step 4), and an exact cell-edge value (exp(0), Φ⁻¹(1/2) = 0, sqrt(10000)) is "detected exactly". Whether either is widened is unstated | LM-4 |
| MC1-9 | LANGUAGE.md:504 | `start` and `end` are required when a model has draws, though no draw here is on a time index and nothing reads the run span. The model sets start = end = on | no change: required as written |
| MC2-1 | L:886, L:601, L:966, L:1493; E:311–316, E:323–326 | How a pay or def reads a path-keyed draw, and which instances a `p in path` pay has on path q. L:886 says "one per path"; E:325's `alive[p, m]` and ENGINE §8's `share(p in path: bal(konto, …))` read per-path values with no path key. | LM-6 |
| MC2-2 | L:1237, L:1591; grammar L:127–128 | A scenario file's items list versions, index additions, places, inputs, defs, pays and asks, but not `draw` or `index`. | LM-7 |
| MC2-3 | L:175–177, L:206 vs L:1495 | A pin is `ref "=" expr` and a ref's keys are exprs, so `theta[*]` (L:1495) has no grammar; nor is it said whether two keys need `[*, *]`. | V1-P6 (LM-5) |
| MC2-4 | E:363, E:365 | No reason wording for the test-mode failure; and does a pin to the plan or median (`repair_hit = 0`; the mean is 0.1) count as "pinned to its mean"? | LM-3 |
| MC2-5 | L:1047–1050; L:1007 | `mean` and `quantile` over paths print an SE, but no formula is given; the asymptotic quantile SE needs a density, and "a sample has no density". | LU-1 |
| MC2-6 | DONE.md:24, E:353 | "Step bias" is undefined. | LM-1 (the step bias is a bound the model states) |
| MC2-7 | E:360–362 | Carry: (1) the places a term reads mix unit-counted holdings with PLN; (2) do state pays (`invest`) and promised movements count as "plan movements"; (3) the places of a `flows(pay)` term; (4) a def over paths reading balances (`year_low`). | open gap, below |
| MC2-8 | L:399–410 | A draw's law date is unstated: which date reads `repair_chance` and `spend_sd`. | LM-8 |
| MC2-9 | L:1240–1242 + L:405 | Scenario rows govern D..; an ask with no date reads at K < D, so a scenario row read directly in an ask is a hole. | open gap, below (the key works around it) |
| MC2-10 | L:155, L:969, L:988 | `to UNIT` is mandatory on every draw, even an exact one (`binomial … to 1`); `uniform` is "exact" in the table but continuous by rule 4. | V1-P7 (a discrete draw takes no `to`; the key drops its `to 1`) and V5-P3 (`uniform` is exact) |
| MC2-11 | L:1237, L:1246, L:1408 | Does an ask declared in a scenario file run in that scenario by default? | open gap, below |
| MC2-12 | `amendments/household/pays.tally:48–53`; L:1172; grammar L:136–137, L:169 | An ETF price draw can't be added in new files: `etf_price` and `etf_growth` live in pays.tally (an item lives wholly in one file), and rows and givens are literals, so `etf_return` can't be set to a draw. | open gap, below |
| MC2-13 | E:317, E:406, E:413; DONE.md:24 | "A path is replayed alone", but there is no syntax for it, and the kept terms aren't exposed. | open gap, below |
| MC2-14 | E:62; HH4-b.md:741–755 (ambiguity A) | A promised movement after K is never confirmed, so P = 800 for the run. | open gap, below (HH4-b ambiguity A) |
| MC2-15 | E:353 vs L:1047, L:1595; L:1048 | E:353 says the statistic and its SE are "each a def or a row", but a path aggregate outside an ask is `LOAD-ASKONLY`, and the printed SE has no expression form. Nor is it said what `share(…)` is as an operand (n/N?), or whether a verdict prints inside a `check`. | LM-1 |

### Asides and side findings

The verifiers' notes outside their candidates.

| Note | What it is (1.0 lines) | In 1.0.1 |
|---|---|---|
| V1 aside 1 | `rowkeys` (L:167) doesn't admit a keyed place as a key: `rk = key \| "*" \| key ".." key` leaves out L:603's own `opening[wcash[ike]]`, which the household writes (`places.tally`:45–47) | LA-1 |
| V1 aside 2 | The printer clause (L:238) parenthesises a `check`'s left side with an `=` at bracket depth 0, while parse rule 1 needs the right side parenthesised (`check x = (y = z)`) | LA-2 |
| V3-S1 | Change 5's `konto_eur` has no `opening` row, so `bal(konto_eur)` is a hole from S, and the 28th's conversions make konto a hole; an opening known 2029-05-01 might not serve earlier booking cuts | LX-5 adds the row. It serves the earlier cuts: a state pay reads its law and facts at its booking cut, but its balances are the walk's at cut K (`ENGINE.md` §5 step 2) |
| V3-S2 | A missing `tally` line fails the grammar's `header` first (`LOAD-PARSE`), while §5.1:506 and §8.3:1565 give `LOAD-HEADER` | LA-3 |
| V3-S3 | §5.7:627's "`LOAD-UNIT` for a value of the wrong type" had no §8.3 message | V3-P15, carried by L6-P24 |
| V3-S4 | A new pay section added by an amendment has instances dated before its rows' `known`; LANGUAGE has no sentence on how a state pay among them is booked beyond §5.13.3:948 | open gap, below |

## Open gaps

Each is a place where the text leaves a choice open. None decides an expected value: where an answer key touches one, the reading it uses is named. A builder who meets one stops and reports it (`DONE.md` rule 3); the fix is a later patch.

**Registers and presentation**
- **C1-F6.** Day K is neither past nor planned (E:239 against E:225). Both evaluators evaluate day K at cut K, so only the ledger's past/plan label of day K's movements is open.
- **C2-F6.** Whether a `flows` register lists the instances it read only to drop them as out of span. The change 2 key doesn't list them.
- **C2-F7.** The law date (§8.2 rule 4) of an ask line whose top value is not a def cell: K or its `for` binding. The keys use K.
- **C2-F9.** An open-span input with no time key (`wage_growth`) read after K is `held` by L:1555, while HH4-b lists it `assumed` only; "read for <period>" is undefined for an input with no time key. The change 2 key lists it `held`; HH4-b's register is the smoke file's, not an answer key.
- **C3-F5, C4-F7.** The law date printed for a top-level read (`bal`, `flows`, `worth`) and for a dateless ask is K (L:405), so every advance rewrites it. The keys print K.
- **C3-F6, C7-F3.** The plan's register part (O:293–294) has no place for dropped or replaced entries, for `scenario:` or for cleared entries. The keys name them in words.
- **C4-F6 (part).** `source zus_pue` in change 4 sits out of canonical position. The printer moves it; nothing reads the order.
- **C5-F4.** Whether a hole-`when` instance's amount is evaluated, which decides `fx_eur`'s `assumed` and `held` entries in change 5, and the `held` period of the keyless `fx_eur`. The change 5 key lists C5-b's entries and says so.
- **C5-F9.** How the modelled past prints `fx_in` in the ledger, and residuals over an uncovered month. They affect the change 5 variant's replay only.
- **C6-F4.** The overpayment default that the look-ahead reads is not `assumed`. Registers only.
- **C6-F9.** The payout date of the loan is a model convention, and the plan doesn't say how a new place's balance difference is reported.
- **C7-F5.** An amount that goes to or from 0: a changed movement, or one removed or added (O:291 against L:888).
- **C7-F6.** Whether the Balances part's "largest difference" is signed or absolute (O:292).
- **HH4-b ambiguity C** (the design log's J93). Whether a look-ahead's far-future `when` entries go into the register of every `room` and of what reads it. The keys don't list them.

**Semantics**
- **C2-F5.** A late entry, known before the old `on` but added later, is absorbed unmeasured, which breaks "as known D sees the reality as it was"; only a record time R gives the old view. No figure in the keys depends on it.
- **C4-F3.** Whether `plan_low` counts an arrears correction inside its window (L:910 against E:190). It moves the look-ahead's F only, and no asked cell of change 4.
- **C7-F2.** The unstated-key test (L:467) cites a row's `known` date, which a scenario row lacks. The change 7 key reads the scenario as acting, as L:1245 and the §6.4 example do.
- **V3-S4.** How a state pay instance of a pay section added later, dated before the section's `known`, is booked. Change 5's conversions have a false `when` under every reading, so its key doesn't depend on it.
- **MC1-7.** An ask can't read its own N: `count` and `sum` over `path` are neither allowed nor refused, so the SE defs read a row `npaths` typed to match `paths`. The key's row and its `paths` agree.
- **MC2-7.** The carry (E:360–362): places that mix unit-counted holdings with PLN; whether state pays and promised movements count as plan movements; the places of a `flows(pay)` term; a def over paths that reads balances. In the household-bands key c ≤ h ≤ 1.20 < 2 under every reading, so no verdict depends on it; the printed c does.
- **MC2-9.** Scenario rows govern from the scenario's `from`; an ask with no date reads at K before it, so a scenario row read directly in an ask is a hole. The key writes its thresholds as `2 * rent` and keeps its oracles in a reality part.
- **MC2-11.** Whether an ask declared in a scenario file runs in that scenario by default. The key writes `with scenario` on each, right under either reading.
- **MC2-12.** An ETF price draw can't be added in new files: `etf_price` and `etf_growth` live in `pays.tally`, and rows are literals. The key draws no price; a price draw needs an amendment to `pays.tally`.
- **MC2-13.** "A path is replayed alone" (E:317, gate 7) has no syntax, and the kept terms aren't exposed. Until a patch gives one, gate 7's replay test goes through the engine's interface.
- **MC2-14.** A promised movement after K is never confirmed, so P = 800 for the whole run (HH4-b ambiguity A). The key gives the figures under HH4-b's reading A(a), and A(b)'s beside them.
- **L6-P12.** `plan` diffs a draft against the current reality only (O §7), so a diff of two record times in one result has no form; two runs at the two record times compare them.
- **L6-P19.** The hindcast refusals (`verification/hindcast.md` §4) of a MAPE over a series that crosses zero and of a read of the answer key before the model hash is frozen belong to the judge script, which holds the key. Neither has an engine code; the code census stays 87.

**Tools**
- **`tools/loadcheck.py`** doesn't know the built-in index `pay` (LU-10). It scans what loads, not what `match_days` and `match_amount` are keyed by, and no model, household file or change writes them.

## Decided in `DONE.md`

`DONE.md` is the user's contract (its first paragraph), so the lead patched none of it on its own. The user decided these on 2026-10-19, each as the lead recommended, and the decision is logged in `DONE.md`'s approvals (LD-4):

| Item | Gate | Finding | What was wrong | Decided | Patch |
|---|---|---|---|---|---|
| D-1 | 1 | R05-12, R13-9 | "The three declared bounds (MP-15's fewest transfers, LH-17 step 6, CH-03 step 14)": the models declare six, four of them in CH-03 | name all six | V1-P25 |
| D-2 | 6 | R13-11 | "No foresight" covered cut masking only, not BUILD-ORDER's look-ahead property (C-13) | the gate includes C-13 | LD-1 |
| D-3 | 7 | R13-10, MC1-3 | LH-17's Eq. 8 is exact only at λ = 0, so a correct engine fails against it; the evidence "the Monte Carlo models" named no files | the exact closed form (Yor 1992 at λ = μ), and `montecarlo/` | LD-2 |
| D-4 | 12 | R13-16 | "At 10× size": only the cold run is stated at 10×; the other rows state a size of their own | each budget at the size its row states | LD-3 |
| D-5 | 7 | V5-P34 | "The household's bands": `band` is a rate table in the language | path statistics | LD-2 |
