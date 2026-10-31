CO close-out is finished. All six checks pass. No existing file under tmp/cases/ was changed, nothing was committed, the holdout folders were never touched, and no sub-agents were used.

**Files written**
- Gap log (7 parts, 677 lines): /Users/jswob/projects/trop/tmp/cases/gaps-1.0.1.md
- Merge script that writes the gap log: /Users/jswob/projects/trop/tmp/cases/tally-1.0.1/tools/merge_gaps.py. With `--check` it runs check 1 instead.
- Checks 2–5 script: /Users/jswob/projects/trop/tmp/cases/tally-1.0.1/tools/final_checks.py
- Folder README: /Users/jswob/projects/trop/tmp/cases/tally-1.0.1/README.md
- Loadcheck copies (household plus law): tmp/cases/tally-1.0.1/scratch/CO/{A,B,D,E1}/

**Counts (part 2)**
| Kind | breaks | can't express | awkward | expressed | suspect | total |
|---|---|---|---|---|---|---|
| P | 3 | 0 | 18 | 69 | 2 | 92 |
| Q | 8 | 3 | 20 | 4 | 4 | 39 |
| O | 1 | 7 | 26 | 0 | 2 | 36 |
| S | 17 | 0 | 0 | 0 | 0 | 17 |
| S:13–14, withdrawn as findings | 0 | 0 | 2 | 0 | 0 | 2 |
| all | 29 | 10 | 66 | 73 | 8 | 186 |

Edges by worst status (part 3): breaks 44, can't express 15, awkward 28, expressed 5. "(+ suspect)" is shown beside the worst status, never as it.

**Ten worst rows**
1. Q:B-09 breaks: FV 016 keeps 4270.00 where the key has a hole, because the spec is silent on what a hole movement does to claims (S:9).
2. Q:B-08 breaks: `residuals` shows corrections at anchors, not the key's four rows (S:10).
3. Q:A-01 breaks (A-02 likewise): `where earner[e,y] = s` with `-` gives RUN-NA, so every 2021–22 figure is refused.
4. Q:B-01 breaks: `first` over an empty set at the cut gives RUN-EMPTY, so tax_b is refused.
5. Q:B-04 breaks (B-05 likewise): a `known` line in a `from D` scenario is LOAD-SCENARIO, refused at load.
6. Q:D-12 breaks: fund units typed `int` against a DUK/fund_unit price is LOAD-UNIT, on NUM-01 itself.
7. S:4 breaks: DUK and FLR have no minor unit in the spec.
8. S:2 breaks: the EBNF `call` gives the built-ins with their own form two parses (41 places in 14 spec files).
9. Q:A-09 can't express: OPERATIONS §2.2 names no dialect, so the record CSVs can't be imported.
10. Q:D-07 can't express: the same gap; the exports can't be read.

**Settlements by cause (part 5, 42 rows)**
- writer misread 17, supervisor 10, spec unclear 9, case suspect 6.
- Judge downgrades upheld: 13.
- Downgraded by the supervisor with no judge downgrade: P:LAW-12, Q:B-05, S:13, S:14.

**Suspects (part 6)**
- P:VAL-06: no 2024 earnings stated, yet the key adds none.
- P:SCEN-02: the 20 Sep instalment falls after the branch, but the key starts in October.
- Q:A-10: the dossier settles the rule order that the key calls missing.
- Q:A-11: Ania's 0.01 cash is left out; the rate is arguably settled.
- Q:A-12: contradicts A-10's key; story item 2 reads two ways.
- Q:B-12: AVG-WAGE-Q4 is published after the cut.
- O:B-2: the `without` rows aren't in any document.
- O:B-3: the same gazette row as B-12.
- Also logged, with no row: the D-D04 "became final" oddity.

**Cost (part 7)**
- About 4.61 % of the week: sub-agents 3.92 %, supervisor 0.69 %. 1087 calls, 39.85 M weighted.
- Wall time 8h55 (05:00–13:55).
- That is well under the 7–11 % estimate and under 10–12 hours. My own cost is not in agents.tsv.

**Checks**
1. P 92; Q 39 (A 12, B 12, D 12, E1 3); O 36; S 19, of which 2 are withdrawn. All ids are unique across the settled files, and the probe ids match the edges in INDEX.md exactly.
2. parsecheck: "305 files, 305 ok, 0 failed", exit 0.
3. loadcheck: 92 probe cards and 4 households; 1 of 96 realities is refused. That one is SURF-04's LOAD-DUP: `redefine.tally:6 def contrib` declared twice, first at `SURF-04.tally:28`. P:SURF-04 names it as the refusal its card asks for (expressed; maps/G5/SURF-04.md lines 9 and 13). There are no other failures.
4. check.py: "# 17 of 17 checks pass: PASS", exit 0.
5. `find tmp/cases engine -name __pycache__` prints nothing.
6. Every agent's audit line from agents.tsv is in part 7.

**Two settled rows were split wrongly in their source files.** settle.py broke them at a `\|` inside a cell:
- O:D-1 is missing its Text cell.
- S:19 is missing its Gap and Text cells.

The merge script takes those cells, and the reason's spacing, from the writer's rows (S:19 is the writer's S:802). It lists this in part 1. The source files are unchanged.

**Two more source quirks:**
- P0.settled.md has no Settlement column, so part 5 takes S:1–3's settlement from the file's header note.
- O:B-4 and O:B-5 say "settled:" with no cause written; part 5 shows them as "supervisor (status set; no cause written)".

All Text paths exist.
