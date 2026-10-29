# Gap log, tally 1.0.1 against cases 1.0

Session 3 of the case-set plan. Written by `tally-1.0.1/tools/merge_gaps.py` from the settled rows in `tally-1.0.1/rows/`; do not edit by hand, rerun the script. Text paths are relative to `tmp/cases/tally-1.0.1/` (or to the repository root for `engine/…`).

## 1. What it is and how it was made

One row per probe card (`P:`, 92), per household question (`Q:`, 39: A 12, B 12, D 12, E1 3), per modelling observation (`O:`, a fact or rule that is hard to write although no question asks about it) and per spec finding (`S:`, a place where the spec itself is unclear or contradicts itself). Statuses, worst first: **breaks**, **can't express**, **awkward**, **expressed**; **case suspect** is outside the scale and goes to `ERRATA.md` (part 6).

- **Agents.** A pilot probe writer (P0, spec findings from parsecheck); five probe writers (W-G1 … W-G5), one judge each (J-G1 … J-G5); four household writers (W-A, W-B, W-D, W-E1, the last for the joined family), one judge each (J-A, J-B, J-D, J-E1); the supervisor, who settled every disagreement; and this close-out (CO), which merged the rows. Part 7 lists each with its cost and audit.
- **Step map.** Each writer wrote a Tally text and a step map (`maps/<chunk>/`): every step of the card's or key's derivation, marked "same", a workaround, or missing, with the lines of the text that do it. A row's status follows from its map.
- **Judges.** Each judge tried to break every row its writer marked "expressed", line by line against the spec, and proposed a downgrade where it succeeded. The supervisor settled each verdict (part 5).
- **What "expressed" means here.** Judged against the spec (`engine/spec/`, tally 1.0.1), line by line, and not run: nothing runs Tally yet. A row is expressed when every step of its map means, by the spec, what the key's step computes.
- **Tools.** `parsecheck` checks syntax; `loadcheck` is a line scan of ids and keywords. Neither checks units (LOAD-UNIT), names (LOAD-NAME), scenario rules (LOAD-SCENARIO) or run-time refusals (RUN-NA, RUN-EMPTY), which is how several rows that pass both tools were still downgraded (A-01, A-02, B-01, B-04, B-05, D-12, E1-03).
- **Escalations.** W-A ran at xhigh by plan; J-A overturned 3 of 6 expressed rows, so W-B ran at xhigh; J-B overturned 4 of 5, so W-E1 ran at max; W-D (already running) stayed at xhigh; J-D overturned 3 of 3. The judges of the probe chunks overturned 0, 1, 0, 1, 1 rows (G1 … G5; G4's is P:LAW-12, which the supervisor downgraded after reading the card, part 5).
- **Cost cap.** The cap (14 % of the week) did not stop the session: everything ran (part 7).
- **Scope audit.** No agent touched the holdout folders; the other flags were false positives or harmless listings, as `agents.tsv` records (part 7).

**The supervisor's decisions:**

- D1. Final messages are saved from each agent's transcript by `tools/save_final.py` (never retyped).
- D2. Spec findings (`S:`) have no judge; the supervisor checks and settles them. S ids were renumbered at settlement into one sequence (S:1 …).
- D3. The made-up currencies DUK and FLR have no minor unit in the spec (S:4): rows are judged reading a minor unit of 0.01, and not downgraded for that alone.
- D4. Prompts: a short prompt restating the hard rules plus a filled brief file; all prompts and templates are kept in `tally-1.0.1/tools/prompts/`.
- D5. A row a writer's message left out is recovered from its step map (`rows/G5.recovered.md`: P:RULE-08), then judged like any other.
- D6. Household B's writer converted every record row with a script (one movement row per CSV row, citing file and line), standing in for the import pipeline of OPERATIONS §2.2; the brief allowed "at most a few rows". The work was kept and the deviation logged; households D and E1 followed the brief. So B's record questions are judged on full record rows, while A's (A-09) and D's (D-07) are can't express for want of them: the gap is the same (O:A-1, O:B-1, O:D-1: the import names no file format), only the writers' handling differs.
- D7. A load refusal in one item of a writer's own file stops the whole reality (L §8): household D's `fund.tally` LOAD-UNIT (row Q:D-12) is charged to that row only; D's other rows and E1's are judged as if that one fault were fixed.
- D8. A candidate spec finding (HOLE-IMPRECISE for a pay `on m` when a version becomes known inside m) was dropped: L:420-421 settle it.
- S:13 and S:14 keep their ids but are not spec findings (writer misread, status awkward): they are marked "withdrawn as a finding" in part 4 and left out of the spec-findings count.
- Dossier oddity, no row: D-D04 (received 2024-10-21) says the certificate "became final on 3 November 2024"; D's story says "becomes final".

**Source rows mended by the merge** (the settled files are frozen and stay as they are):

- O:D-1 (D.settled.md): 6 cells; text taken from the writer's row O:D-1; reason spacing from the writer's row (same text)
- S:19 (D.settled.md): 5 cells; gap, text taken from the writer's row S:802; reason spacing from the writer's row (same text)

## 2. Counts

| Kind | breaks | can't express | awkward | expressed | case suspect | Total |
|---|---:|---:|---:|---:|---:|---:|
| probe (P) | 3 | 0 | 18 | 69 | 2 | 92 |
| question (Q) | 8 | 3 | 20 | 4 | 4 | 39 |
| observation (O) | 1 | 7 | 26 | 0 | 2 | 36 |
| spec finding (S) | 17 | 0 | 0 | 0 | 0 | 17 |
| S withdrawn (not a finding) | 0 | 0 | 2 | 0 | 0 | 2 |
| **all** | **29** | **10** | **66** | **73** | **8** | **186** |

Per chunk (all kinds; P0 holds spec findings only):

| Chunk | P | Q | O | S | breaks | can't express | awkward | expressed | case suspect |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| P0 | 0 | 0 | 0 | 3 | 3 | 0 | 0 | 0 | 0 |
| G1 | 19 | 0 | 4 | 2 | 3 | 1 | 6 | 15 | 0 |
| G2 | 20 | 0 | 4 | 0 | 2 | 0 | 6 | 16 | 0 |
| G3 | 18 | 0 | 2 | 0 | 0 | 1 | 6 | 11 | 2 |
| G4 | 18 | 0 | 1 | 0 | 0 | 0 | 3 | 16 | 0 |
| G5 | 17 | 0 | 1 | 1 | 2 | 0 | 6 | 11 | 0 |
| A | 0 | 12 | 8 | 2 | 4 | 5 | 7 | 3 | 3 |
| B | 0 | 12 | 5 | 9 | 12 | 1 | 9 | 1 | 3 |
| D | 0 | 12 | 4 | 2 | 3 | 1 | 14 | 0 | 0 |
| E1 | 0 | 3 | 7 | 0 | 0 | 1 | 9 | 0 | 0 |

## 3. By edge

In `INDEX.md` order. Probe: the `P:<edge>` row. Questions: the worst of the `Q:` rows of the questions INDEX.md lists for the edge. Worst: of both. Case suspect is outside the scale and shows beside the worst status, never as it.


### NUM

| Edge | Name | Probe | Questions | Questions' worst | Worst |
|---|---|---|---|---|---|
| NUM-01 | Amounts, quantities and rates carry their units | expressed | B-02, B-03 (awkward), D-01 (awkward), D-06 (awkward), D-12 (breaks) | breaks | breaks |
| NUM-02 | Day counts and pro-rating by days | expressed | A-04, A-07, B-04 (breaks), B-05 (breaks) | breaks | breaks |
| NUM-03 | Rounding where and as the law rounds | expressed | A-01 (breaks), A-03 (awkward), D-06 (awkward) | breaks | breaks |
| NUM-04 | An unclear convention, keyed under each reading | expressed | A-07 | expressed | expressed |
| NUM-05 | Parts and wholes | expressed | A-11 (case suspect), D-01 (awkward), D-04 (awkward), D-05 (awkward), D-08 (awkward), D-09 (awkward) | awkward (+ suspect) | awkward (+ suspect) |
| NUM-06 | Which year's money | expressed | A-12 (case suspect), D-10 (awkward), D-11 (awkward) | awkward (+ suspect) | awkward (+ suspect) |
| NUM-07 | Gross, net, sign and zero | expressed | A-05 (can't express), B-07 (awkward) | can't express | can't express |

### TIME

| Edge | Name | Probe | Questions | Questions' worst | Worst |
|---|---|---|---|---|---|
| TIME-01 | A date known only to the month | expressed | A-05 (can't express) | can't express | can't express |
| TIME-02 | Calendars and dates stated as rules | expressed | A-03 (awkward), A-09 (can't express), B-01 (breaks), B-02, B-09 (breaks) | breaks | breaks |
| TIME-03 | The period an amount is for, and the day it is paid | expressed | A-01 (breaks), A-03 (awkward), B-01 (breaks) | breaks | breaks |
| TIME-04 | Periods each law declares, split by facts | expressed | A-04, B-01 (breaks), D-03 (awkward) | breaks | breaks |
| TIME-05 | Clocks, windows and deadlines | expressed | A-04, A-06, D-02 (awkward), E1-03 (awkward) | awkward | awkward |
| TIME-06 | Running totals and repricing within a period | expressed | A-03 (awkward), B-01 (breaks) | breaks | breaks |
| TIME-07 | Order within one day | awkward | A-09 (can't express), B-08 (breaks), B-10 (awkward) | breaks | breaks |

### PTY

| Edge | Name | Probe | Questions | Questions' worst | Worst |
|---|---|---|---|---|---|
| PTY-01 | People and their dated relations | expressed | B-04 (breaks), B-11 (awkward), D-01 (awkward) | breaks | breaks |
| PTY-02 | Groups the law computes on | expressed | A-01 (breaks), B-07 (awkward), D-01 (awkward) | breaks | breaks |
| PTY-03 | Succession down the family tree | expressed | D-01 (awkward), D-02 (awkward), E1-03 (awkward) | awkward | awkward |
| PTY-04 | A law reads another party | expressed | A-01 (breaks), D-03 (awkward) | breaks | breaks |
| PTY-05 | Joins across households | breaks | E1-01 (awkward), E1-02 (awkward), E1-03 (awkward) | awkward | breaks |

### OWN

| Edge | Name | Probe | Questions | Questions' worst | Worst |
|---|---|---|---|---|---|
| OWN-01 | Places, their kinds and lifetimes | awkward | A-11 (case suspect), B-04 (breaks), D-08 (awkward), D-09 (awkward) | breaks (+ suspect) | breaks (+ suspect) |
| OWN-02 | Claims and money that isn't ours | awkward | B-09 (breaks), B-11 (awkward), D-05 (awkward), D-08 (awkward), E1-02 (awkward) | breaks | breaks |
| OWN-03 | Owed but not yet paid | expressed | A-11 (case suspect), B-11 (awkward), D-06 (awkward), D-08 (awkward), E1-02 (awkward) | awkward (+ suspect) | awkward (+ suspect) |
| OWN-04 | Four ownership regimes | awkward | A-05 (can't express), A-11 (case suspect), B-04 (breaks), B-11 (awkward), D-01 (awkward), E1-02 (awkward) | breaks (+ suspect) | breaks (+ suspect) |
| OWN-05 | Events that move ownership | expressed | A-06, D-01 (awkward), E1-03 (awkward) | awkward | awkward |
| OWN-06 | Lots | expressed | A-05 (can't express), D-01 (awkward), D-06 (awkward) | can't express | can't express |
| OWN-07 | Matching sales to lots | expressed | A-05 (can't express), D-07 (can't express) | can't express | can't express |
| OWN-08 | Kept-apart parts and their limits | expressed | A-05 (can't express), A-09 (can't express) | can't express | can't express |
| OWN-09 | Indexed bonds in monthly series | expressed | D-07 (can't express) | can't express | can't express |

### FLOW

| Edge | Name | Probe | Questions | Questions' worst | Worst |
|---|---|---|---|---|---|
| FLOW-01 | Transfers and conservation | expressed | A-09 (can't express), E1-01 (awkward) | can't express | can't express |
| FLOW-02 | One event, several legs | expressed | A-07, A-08 (awkward), B-06 (awkward), E1-01 (awkward) | awkward | awkward |
| FLOW-03 | Shared costs and netting | expressed | B-11 (awkward), D-01 (awkward), D-05 (awkward) | awkward | awkward |

### CUR

| Edge | Name | Probe | Questions | Questions' worst | Worst |
|---|---|---|---|---|---|
| CUR-01 | Which rate for each leg | expressed | B-03 (awkward) | awkward | awkward |
| CUR-02 | Converted amounts are whole minor units | expressed | B-03 (awkward) | awkward | awkward |
| CUR-03 | Revaluation is not a transfer | expressed | B-03 (awkward) | awkward | awkward |

### VAL

| Edge | Name | Probe | Questions | Questions' worst | Worst |
|---|---|---|---|---|---|
| VAL-01 | Figures from the gazette | awkward | D-06 (awkward) | awkward | awkward |
| VAL-02 | Which date a value is read at | expressed | A-09 (can't express), B-01 (breaks), D-06 (awkward), D-11 (awkward) | breaks | breaks |
| VAL-03 | Two dates on every figure | expressed | A-02 (breaks) | breaks | breaks |
| VAL-04 | A published figure that disagrees with its law | expressed | A-05 (can't express), D-03 (awkward), D-08 (awkward) | can't express | can't express |
| VAL-05 | Prices come from records | expressed | A-05 (can't express), B-05 (breaks) | breaks | breaks |
| VAL-06 | Indexed values and histories | case suspect | A-12 (case suspect), D-03 (awkward), D-11 (awkward) | awkward (+ suspect) | awkward (+ suspect) |
| VAL-07 | The assumption register | expressed | D-11 (awkward) | awkward | awkward |

### LAW

| Edge | Name | Probe | Questions | Questions' worst | Worst |
|---|---|---|---|---|---|
| LAW-01 | Edges: thresholds, brackets and tables | expressed | A-01 (breaks), B-01 (breaks) | breaks | breaks |
| LAW-02 | Cliffs and tapers | expressed | A-01 (breaks), B-01 (breaks), B-06 (awkward) | breaks | breaks |
| LAW-03 | Law versions by date | expressed | A-01 (breaks), A-02 (breaks), D-03 (awkward) | breaks | breaks |
| LAW-04 | Retroactive law | awkward | A-01 (breaks), D-03 (awkward), D-04 (awkward) | breaks | breaks |
| LAW-05 | "As if" | expressed | A-03 (awkward) | awkward | awkward |
| LAW-06 | Elections | expressed | A-01 (breaks), B-01 (breaks) | breaks | breaks |
| LAW-07 | Eligibility and classification from facts | expressed | A-01 (breaks), B-02, B-06 (awkward), D-03 (awkward) | breaks | breaks |
| LAW-08 | Pipelines across laws and forms | expressed | A-01 (breaks), A-03 (awkward), B-01 (breaks), B-07 (awkward) | breaks | breaks |
| LAW-09 | Carry-forwards with caps | expressed | A-05 (can't express) | can't express | can't express |
| LAW-10 | A result pending until a window closes | expressed | B-04 (breaks), D-09 (awkward) | breaks | breaks |
| LAW-11 | A law reads history | expressed | A-06, B-01 (breaks), E1-03 (awkward) | breaks | breaks |
| LAW-12 | An exception through a chain | awkward | A-05 (can't express), A-06, D-02 (awkward), E1-03 (awkward) | can't express | can't express |
| LAW-13 | Future law | expressed | A-12 (case suspect), B-12 (case suspect) | — (+ suspect) | expressed (+ suspect) |

### LIFE

| Edge | Name | Probe | Questions | Questions' worst | Worst |
|---|---|---|---|---|---|
| LIFE-01 | A thing written once, with a lifetime | expressed | A-12 (case suspect), D-10 (awkward) | awkward (+ suspect) | awkward (+ suspect) |
| LIFE-02 | When a thing ends | expressed | A-07, A-08 (awkward), B-04 (breaks), B-06 (awkward) | breaks | breaks |
| LIFE-03 | Loans | expressed | A-07, B-05 (breaks), D-01 (awkward) | breaks | breaks |
| LIFE-04 | Benefits recalculated | expressed | A-01 (breaks), A-04, D-03 (awkward), D-04 (awkward) | breaks | breaks |
| LIFE-05 | A sole trader's month and year | expressed | B-01 (breaks) | breaks | breaks |

### RULE

| Edge | Name | Probe | Questions | Questions' worst | Worst |
|---|---|---|---|---|---|
| RULE-01 | A rule on the real balance on a date | expressed | A-09 (can't express), B-10 (awkward) | can't express | can't express |
| RULE-02 | What a rule may see | awkward | A-09 (can't express), B-10 (awkward), D-07 (can't express) | can't express | can't express |
| RULE-03 | Rules competing for the same money | expressed | A-10 (case suspect), B-10 (awkward) | awkward (+ suspect) | awkward (+ suspect) |
| RULE-04 | A loop | expressed | B-10 (awkward) | awkward | awkward |
| RULE-05 | "Once, when…" on a state | awkward | A-12 (case suspect), D-07 (can't express) | can't express (+ suspect) | can't express (+ suspect) |
| RULE-06 | A rule changed "as if since January" | expressed | B-10 (awkward) | awkward | awkward |
| RULE-07 | What a rule meant vs what the records show | awkward | A-09 (can't express), B-11 (awkward) | can't express | can't express |
| RULE-08 | Parameters estimated from history | awkward | A-09 (can't express) | can't express | can't express |
| RULE-09 | A rule on a forecast | awkward | B-12 (case suspect) | — (+ suspect) | awkward (+ suspect) |

### REC

| Edge | Name | Probe | Questions | Questions' worst | Worst |
|---|---|---|---|---|---|
| REC-01 | Records are the past; the model is the rest | awkward | B-08 (breaks) | breaks | breaks |
| REC-02 | A statement with errors | expressed | B-08 (breaks), D-05 (awkward) | breaks | breaks |
| REC-03 | Awkward rows | expressed | A-09 (can't express), B-08 (breaks) | breaks | breaks |
| REC-04 | Matching and certainty | expressed | B-09 (breaks), E1-01 (awkward) | breaks | breaks |
| REC-05 | Records as the law's inputs | awkward | A-01 (breaks), B-07 (awkward) | breaks | breaks |
| REC-06 | Clocks and knowledge dates | expressed | A-02 (breaks), B-07 (awkward), E1-01 (awkward) | breaks | breaks |
| REC-07 | Replaying over recorded history | expressed | B-10 (awkward), D-12 (breaks) | breaks | breaks |

### SCEN

| Edge | Name | Probe | Questions | Questions' worst | Worst |
|---|---|---|---|---|---|
| SCEN-01 | Scenarios and amendments | awkward | B-05 (breaks) | breaks | breaks |
| SCEN-02 | A counterfactual past | case suspect | A-08 (awkward), B-05 (breaks) | breaks | breaks (+ suspect) |
| SCEN-03 | Linked overrides | awkward | D-11 (awkward) | awkward | awkward |
| SCEN-04 | Named extreme paths | expressed | A-12 (case suspect) | — (+ suspect) | expressed (+ suspect) |

### RAND

| Edge | Name | Probe | Questions | Questions' worst | Worst |
|---|---|---|---|---|---|
| RAND-01 | Random families | expressed | B-12 (case suspect), D-10 (awkward) | awkward (+ suspect) | awkward (+ suspect) |
| RAND-02 | Closed forms | expressed | B-12 (case suspect), D-10 (awkward) | awkward (+ suspect) | awkward (+ suspect) |
| RAND-03 | Law on every path | expressed | B-12 (case suspect), D-10 (awkward) | awkward (+ suspect) | awkward (+ suspect) |
| RAND-04 | What a run may say | expressed | B-12 (case suspect), D-10 (awkward) | awkward (+ suspect) | awkward (+ suspect) |

### ASK

| Edge | Name | Probe | Questions | Questions' worst | Worst |
|---|---|---|---|---|---|
| ASK-01 | Question primitives | expressed | B-01 (breaks) | breaks | breaks |
| ASK-02 | Net worth per person, with holes | awkward | A-11 (case suspect), B-11 (awkward), D-08 (awkward), E1-02 (awkward) | awkward (+ suspect) | awkward (+ suspect) |
| ASK-03 | Holes, refusals and warnings | expressed | A-04, A-10 (case suspect), A-11 (case suspect), B-07 (awkward), B-08 (breaks), B-09 (breaks), B-10 (awkward) | breaks (+ suspect) | breaks (+ suspect) |
| ASK-04 | No advice | awkward | D-09 (awkward) | awkward | awkward |
| ASK-05 | Explain | breaks | A-01 (breaks) | breaks | breaks |

### SURF

| Edge | Name | Probe | Questions | Questions' worst | Worst |
|---|---|---|---|---|---|
| SURF-01 | No bare figures | breaks | — | — | breaks |
| SURF-02 | Surface words | expressed | — | — | expressed |
| SURF-03 | Draws written | expressed | B-12 (case suspect), D-10 (awkward) | awkward (+ suspect) | awkward (+ suspect) |
| SURF-04 | Stated once, checked before it runs | expressed | — | — | expressed |

Questions column: every question INDEX.md lists, with its status where it is not expressed. Edges by worst status: breaks 44, can't express 15, awkward 28, expressed 5.

## 4. The rows

Statuses as settled; part 5 has each change. A Text path marked **missing** does not exist.

### 4.1 Probes, by axis

#### NUM

| Row | Status | Reason | Spec | Gap | Text |
|---|---|---|---|---|---|
| `P:NUM-01` | expressed |   | `L §3.2`, `L §3.3`, `L §4.6`, `L §7.2`, `L §8.1` | — | `probes/num/NUM-01.tally` |
| `P:NUM-02` | expressed |   | `L §4.2`, `L §7.1`, `L §7.2` | — | `probes/num/NUM-02.tally` |
| `P:NUM-03` | expressed |   | `L §7.2`, `L §3.2` | — | `probes/num/NUM-03.tally` |
| `P:NUM-04` | expressed |   | `L §5.5`, `L §5.10` | — | `probes/num/NUM-04.tally` |
| `P:NUM-05` | expressed |   | `L §5.12`, `L §5.6`, `L §7.2` | — | `probes/num/NUM-05.tally` |
| `P:NUM-06` | expressed |   | `L §3.4`, `L §4.3`, `L §4.6`, `L §5.10`, `L §7.2` | — | `probes/num/NUM-06.tally` |
| `P:NUM-07` | expressed |   | `L §5.10`, `L §2.3` | — | `probes/num/NUM-07.tally` |


#### TIME

| Row | Status | Reason | Spec | Gap | Text |
|---|---|---|---|---|---|
| `P:TIME-01` | expressed |   | `L §4.4`, `L §4.2`, `L §5.10`, `L §8.1` | — | `probes/time/TIME-01.tally` |
| `P:TIME-02` | expressed |   | `L §4.2`, `L §4.3`, `L §4.6` | — | `probes/time/TIME-02.tally` |
| `P:TIME-03` | expressed |   | `L §4.3`, `L §4.6` | — | `probes/time/TIME-03.tally` |
| `P:TIME-04` | expressed |   | `L §4.2`, `L §5.10`, `L §7.2` | — | `probes/time/TIME-04.tally` |
| `P:TIME-05` | expressed |   | `L §4.2` | — | `probes/time/TIME-05.tally` |
| `P:TIME-06` | expressed |   | `L §5.8`, `L §5.10`, `L §7.2` | — | `probes/time/TIME-06.tally` |
| `P:TIME-07` | awkward | step 2 (reverse order): a pay can't be declared a state pay, so a flow pay can't be put after a state pay (L §5.13.1 "inferred, never declared"). The reverse is a workaround: the rule's formula run by hand in an ask on the previous day's `bal(konto, d)`. Step 1's stated order is the fixed L §5.13.2 order, not a line of the text | `L §5.13.1`, `L §5.13.2`, `L §5.16` | — | `probes/time/TIME-07.tally` |


#### PTY

| Row | Status | Reason | Spec | Gap | Text |
|---|---|---|---|---|---|
| `P:PTY-01` | expressed |   | `L §3.5`, `L §4.3`, `L §4.6`, `L §5.10` | — | `probes/pty/PTY-01.tally` |
| `P:PTY-02` | expressed |   | `L §5.8`, `L §5.12`, `L §3.2` | — | `probes/pty/PTY-02.tally` |
| `P:PTY-03` | expressed |   | `L §3.5`, `L §5.10` | — | `probes/pty/PTY-03.tally` |
| `P:PTY-04` | expressed |   | `L §5.10`, `L §5.16` | — | `probes/pty/PTY-04.tally` |
| `P:PTY-05` | breaks | step 1: Tally has no matching. The link is a hand-written `explains`, and booking takes A's record as the movement and makes D's pay the modelled past (the reverse of rule 1). No form counts transfers, so flows gives an amount, not the count 1 (L:1127, L:1131–1136). step 2 is silent: "each household's key holds" compares realities, but an ask reads one reality (L §6.1, L §5.16) | `L §5.18`, `L §6.1`, `L §5.16` | — | `probes/pty/PTY-05.tally` |


#### OWN

| Row | Status | Reason | Spec | Gap | Text |
|---|---|---|---|---|---|
| `P:OWN-01` | awkward | step 2: a place has no lifetime (L §5.9), so savings exists from S with no opening and its balance is a hole, which makes net worth a hole. The workaround is an `opened[x]` input used in the fee pay's `when` and in the asks' `where` | `L §5.9`, `L §5.13`, `L §5.18` | — | `probes/own/OWN-01.tally` |
| `P:OWN-02` | awkward | step 2: the look-ahead that leaves out promised pays is `plan_low`, which an ask may not read (L §5.13.1:911, LOAD-STATE). The workaround rebuilds it as min over `bal` minus the promised pay's flows by hand (L:903) | `L §5.13`, `L §5.13.1`, `L §5.10` | — | `probes/own/OWN-02.tally` |
| `P:OWN-03` | expressed |   | `L §5.9`, `L §5.13`, `L §3.6` | — | `probes/own/OWN-03.tally` |
| `P:OWN-04` | awkward | workaround not named: every asked value reads worth(x, valued_on) (line 37), and valued_on = 2025-01-01 (line 74) is a date the card never gives, written as a plain row, not assumed (L §4.5: assumed marks any figure no source states; L §5.9: an undated illustration writes its money as defs). Values match the key only because flat_value has no span | `L §3.6`, `L §5.9`, `L §5.10` | — | `probes/own/OWN-04.tally` |
| `P:OWN-05` | expressed |   | `L §3.6`, `L §5.9`, `L §4.6` | — | `probes/own/OWN-05.tally` |
| `P:OWN-06` | expressed |   | `L §4.3`, `L §4.6`, `L §3.2` | — | `probes/own/OWN-06.tally` |
| `P:OWN-07` | expressed |   | `L §5.11`, `L §10` | — | `probes/own/OWN-07.tally` |
| `P:OWN-08` | expressed |   | `L §5.10`, `L §5.11`, `L §7.2` | — | `probes/own/OWN-08.tally` |
| `P:OWN-09` | expressed |   | `L §3.3`, `L §4.2`, `L §5.10`, `L §7.2` | — | `probes/own/OWN-09.tally` |


#### FLOW

| Row | Status | Reason | Spec | Gap | Text |
|---|---|---|---|---|---|
| `P:FLOW-01` | expressed |   | `L §5.18`, `L §5.13.3`, `L §5.16` | — | `probes/flow/FLOW-01.tally` |
| `P:FLOW-02` | expressed |   | `L §5.13`, `L §5.13.3`, `L §5.16` | — | `probes/flow/FLOW-02.tally` |
| `P:FLOW-03` | expressed |   | `L §3.2`, `L §5.10`, `L §5.16` | — | `probes/flow/FLOW-03.tally` |


#### CUR

| Row | Status | Reason | Spec | Gap | Text |
|---|---|---|---|---|---|
| `P:CUR-01` | expressed |   | `L §4.2`, `L §4.5`, `L §3.2`, `L §7.2` | — | `probes/cur/CUR-01.tally` |
| `P:CUR-02` | expressed |   | `L §3.2`, `L §7.2`, `L §5.10` | — | `probes/cur/CUR-02.tally` |
| `P:CUR-03` | expressed |   | `L §5.9`, `L §5.13.3`, `L §3.2` | — | `probes/cur/CUR-03.tally` |


#### VAL

| Row | Status | Reason | Spec | Gap | Text |
|---|---|---|---|---|---|
| `P:VAL-01` | awkward | step 2: "held at the last entry" is written as a day-keyed def with a cited rule `held` (`missing(rw)` and `d > rw_end`) reading `rw as if {law rw_end}`. A rule can't read its own law date or a row's span end, so `rw_end` is restated as a row. L §8.2 `held:` covers only open-span rows read after K. The key's partial comes back as a value with register entries (there is no partial kind, L §8). Step 1 is the gap-rule hole (L §4.6), same | `L §4.6`, `L §5.10`, `L §4.3`, `L §8.2` | — | `probes/val/VAL-01.tally` |
| `P:VAL-02` | expressed |   | `L §4.2`, `L §4.6` | — | `probes/val/VAL-02.tally` |
| `P:VAL-03` | expressed |   | `L §4.5`, `L §4.6`, `L §5.16` | — | `probes/val/VAL-03.tally` |
| `P:VAL-04` | expressed |   | `L §5.10`, `L §5.16`, `L §6.2` | — | `probes/val/VAL-04.tally` |
| `P:VAL-05` | expressed |   | `L §5.10`, `L §4.6`, `L §7.2` | — | `probes/val/VAL-05.tally` |
| `P:VAL-06` | case suspect | step 6: rule 1 adds "that year's earnings" every year, but the key adds none in 2024 and the card states no 2024 earnings. By the gap rule (L §4.6) `earn[2024]` would be a hole and so would the whole result (L §8.1). The text writes `earn[2024] = 0 assumed` to follow the key | `L §5.10`, `L §4.6`, `L §7.2` | — | `probes/val/VAL-06.tally` |
| `P:VAL-07` | expressed |   | `L §5.3`, `L §8.2`, `L §5.10` | — | `probes/val/VAL-07.tally` |


#### LAW

| Row | Status | Reason | Spec | Gap | Text |
|---|---|---|---|---|---|
| `P:LAW-01` | expressed |   | L §5.8 | — | `probes/law/LAW-01.tally` |
| `P:LAW-02` | expressed |   | L §5.8 | — | `probes/law/LAW-02.tally` |
| `P:LAW-03` | expressed |   | L §4.3 | — | `probes/law/LAW-03.tally` |
| `P:LAW-04` | awkward | step 1 (withheld): the card gives no day for the monthly advances, but a pay needs a date. adv_day = 28 is an `assumed` row (line 73). No figure asked depends on the day, as long as June's advance falls before the act is known (31 July) | L §5.9, L §4.5, L §8.2 | — | `probes/law/LAW-04.tally` |
| `P:LAW-05` | expressed |   | L §10 | — | `probes/law/LAW-05.tally` |
| `P:LAW-06` | expressed |   | L §5.10 | — | `probes/law/LAW-06.tally` |
| `P:LAW-07` | expressed |   | L §5.10 | — | `probes/law/LAW-07.tally` |
| `P:LAW-08` | expressed |   | L §7.2 | — | `probes/law/LAW-08.tally` |
| `P:LAW-09` | expressed |   | L §5.10 | — | `probes/law/LAW-09.tally` |
| `P:LAW-10` | expressed |   | L §5.16 | — | `probes/law/LAW-10.tally` |
| `P:LAW-11` | expressed |   | L §5.10 | — | `probes/law/LAW-11.tally` |
| `P:LAW-12` | awkward | workaround not named in the row: the card and edge (boundaries LAW-12: "where the middle rule does not apply, the outer one must not let the case through") want an exception to an exception; a plain excepts chain base<-close<-late<-minors lets Leon through, since an applicable rule beats every rule down its chain whether the middle applies or not (L 5.10 step 4). The text moves rule 3 onto a second def must_report (lines 28-32); the writer named this only in O:LAW-12-1 | L §5.10 | — | `probes/law/LAW-12.tally` |
| `P:LAW-13` | expressed |   | L §6.4 | — | `probes/law/LAW-13` |


#### LIFE

| Row | Status | Reason | Spec | Gap | Text |
|---|---|---|---|---|---|
| `P:LIFE-01` | expressed |   | L §4.6 | — | `probes/life/LIFE-01.tally` |
| `P:LIFE-02` | expressed |   | L §4.2 | — | `probes/life/LIFE-02.tally` |
| `P:LIFE-03` | expressed |   | L §5.10 | — | `probes/life/LIFE-03.tally` |
| `P:LIFE-04` | expressed |   | L §4.6 | — | `probes/life/LIFE-04.tally` |
| `P:LIFE-05` | expressed |   | L §5.8 | — | `probes/life/LIFE-05.tally` |


#### RULE

| Row | Status | Reason | Spec | Gap | Text |
|---|---|---|---|---|---|
| `P:RULE-01` | expressed |   | L §5.13.1 | — | `probes/rule/RULE-01.tally` |
| `P:RULE-02` | awkward | step 1: the planned-only lowest balance and its date. `plan_low` is walk-only and gives no date (L:911, L:920), and `bal(x, d)` in an ask counts the promised pay (L:903). Workaround: the ask takes min/argmin of bal − flows(friend_back, …), subtracting the promised pay by hand | L:903 | — | `probes/rule/RULE-02.tally` |
| `P:RULE-03` | expressed |   | L §5.11 | — | `probes/rule/RULE-03.tally` |
| `P:RULE-04` | expressed |   | L:1600 | — | `probes/rule/RULE-04.tally` |
| `P:RULE-05` | awkward | first instalment and first 15th: a state pay's dates can be read only through `flows`, since `next_on` refuses a state pay (L:921). The once-latch is `any(d < start_of(m): bal(savings, d) >= threshold)`, and the card's unstated instalment amount is an `assumed` row | L:921 | — | `probes/rule/RULE-05.tally` |
| `P:RULE-06` | expressed |   | L:956 | — | `probes/rule/RULE-06.tally` |
| `P:RULE-07` | awkward | step 2, "by the rule": no ask prints the pays' own balance on days the records cover (L:1130–1133). Workaround: `as known 2025-06-30`, a day before the export's assumed known date of 2025-07-01 | L §5.18 | — | `probes/rule/RULE-07.tally` |
| `P:RULE-08` | awkward | step 1-2 provenance: version card_2025 is `known 2025-01-01` (line 22), a date the card never gives, written plainly; with no `start`, L 4.5 dates the stipulations from `on` (2025-06-15). As written, April and May spending are known in January, against the edge (boundaries RULE-08: an estimator over history up to the decision date). Floors still come out right at cut K | `L §5.10`, `L §4.2`, `L §7.2` | — | `probes/rule/RULE-08.tally` |
| `P:RULE-09` | awkward | step 1: the per-path outcomes are rows keyed by path number (runs_dry[p]), since no draw can be written to give the card's chosen paths. `seed 1` is the text's own because the card gives none. Part b's ask sits in a scenario file | L:1617 | MC2-11 | `probes/rule/RULE-09/RULE-09.tally` |


#### REC

| Row | Status | Reason | Spec | Gap | Text |
|---|---|---|---|---|---|
| `P:REC-01` | awkward | step 1: flagging an account the model doesn't hold is the importer's `OP-ACCOUNT` (O:144, O:376), not something an ask computes, and L §5.18 doesn't say whether an ask may read importer inputs. Workaround: `account_place` rows with a `?` row for PKB-7731, so the ask prints its hole. Step 2 (residual −50.00 = stated − booked, L §5.18, E §6.3) and "nothing booked" are same | `L §5.18`, `L §8.1` | — | `probes/rec/REC-01.tally` |
| `P:REC-02` | expressed |   | `L §5.10`, `L §5.16` | — | `probes/rec/REC-02.tally` |
| `P:REC-03` | expressed |   | `L §5.10` | — | `probes/rec/REC-03.tally` |
| `P:REC-04` | expressed |   | `L §5.11`, `L §4.3`, `L §4.6`, `L §5.7` | — | `probes/rec/REC-04.tally` |
| `P:REC-05` | awkward | step 1: the "inferred" gross is a cited rule using `missing(gross_in)` and `solve` over [6000,7000] by 0.01 (L §5.16, §5.10), giving 6500.00 and a uniqueness count of 1. The key's partial (value plus missing payslip) is split into a value with the `missing record counted by` register and a second ask for the hole (lesson 5, L §8) | `L §5.16`, `L §5.10`, `L §8` | — | `probes/rec/REC-05.tally` |
| `P:REC-06` | expressed |   | `L §4.5`, `L §4.6`, `L §4.3` | — | `probes/rec/REC-06.tally` |
| `P:REC-07` | expressed |   | `L §5.10`, `L §7.2`, `L §10` | — | `probes/rec/REC-07.tally` |


#### SCEN

| Row | Status | Reason | Spec | Gap | Text |
|---|---|---|---|---|---|
| `P:SCEN-01` | awkward | relation step: the scenario's loan place is declared in the base with opening 0. A base ask can't name an item declared in a scenario file (`LOAD-SCENARIO`, L §6.4), and an ask inside the scenario has no form to read the base for the difference. Steps 1–6 are same | `L §6.4`, `L §5.13`, `L §5.9` | `MC2-11` | `probes/scen/SCEN-01/SCEN-01.tally` |
| `P:SCEN-02` | case suspect | step 2: the 20 Sep 2024 credit instalment falls after the 15 Sep branch, but the key and its inputs start the credit in 2024-10 with no reason given. The text follows the inputs (`credit_from = 2024-10`) | `L §6.4`, `L §4.3`, `L §5.13` | — | `probes/scen/SCEN-02/SCEN-02.tally` |
| `P:SCEN-03` | awkward | value step "what the override moves": no ask form lists the readers of an input, so the text asks `inputs-of` of each reader inside the scenario (L §5.16) and picks those that show the override's rows. The card gives no pension, bond or mortgage formula, so these are stand-in defs over hole inputs. The rent steps (2595.00 / 2251.01, L §6.4 layer order) are same | `L §6.4`, `L §5.16`, `L §5.10` | — | `probes/scen/SCEN-03/SCEN-03.tally` |
| `P:SCEN-04` | expressed |   | `L §6.4`, `L §5.10`, `L §7.2` | `MC2-9` | `probes/scen/SCEN-04/SCEN-04.tally` |


#### RAND

| Row | Status | Reason | Spec | Gap | Text |
|---|---|---|---|---|---|
| `P:RAND-01` | expressed |   | L §5.14 | — | `probes/rand/RAND-01.tally` |
| `P:RAND-02` | expressed |   | L §5.10 | — | `probes/rand/RAND-02.tally` |
| `P:RAND-03` | expressed |   | L §5.14 | — | `probes/rand/RAND-03.tally` |
| `P:RAND-04` | expressed |   | L:982 | — | `probes/rand/RAND-04/RAND-04.tally` |


#### ASK

| Row | Status | Reason | Spec | Gap | Text |
|---|---|---|---|---|---|
| `P:ASK-01` | expressed |   | `L §5.18`, `L §5.9`, `L §5.13.1`, `L §5.10` | — | `probes/ask/ASK-01.tally` |
| `P:ASK-02` | awkward | partial: Ola's net worth is a hole, because a `sum` with a hole term is a hole (L §5.10 aggregates table, L §8.1). A result is a value or a hole (L §8), never "200 000 plus a hole". Her known 200 000 shows only through a second, per-place ask (`q_parts`) | `L §5.9`, `L §5.10`, `L §4.6`, `L §8` | — | `probes/ask/ASK-02.tally` |
| `P:ASK-03` | expressed |   | `L §5.7`, `L §8.1`, `L §5.9`, `L §8.4` | — | `probes/ask/ASK-03.tally` |
| `P:ASK-04` | awkward | refusal (no winner): L §8 has no refusal for a question that asks for a choice. The text declares the choice a `bound` (L §5.17), which is meant for a step no row can give, and prints it beside both costs. Steps 1–3 are same (`solve`, L §5.16) | `L §5.17`, `L §10`, `L §5.16`, `L §3.3` | — | `probes/ask/ASK-04.tally` |
| `P:ASK-05` | breaks | causes in order: `explain` prints "in the canonical order" (L §5.16), and L §2.4 defines that order only for model text (kind, then id), which would put health before pension. The spec doesn't say in what order causes print (S:5). Steps 1–3 and the net 5 703.47 are same | `L §5.16`, `L §2.4`, `L §5.10` | — | `probes/ask/ASK-05.tally` |


#### SURF

| Row | Status | Reason | Spec | Gap | Text |
|---|---|---|---|---|---|
| `P:SURF-01` | breaks | req 5, "no amount inside a rule's arithmetic as a literal": the passage keeps the floor as a row, but the spec doesn't enforce this. `amount max(0, bal(konto) - 1000)` loads without LOAD-LITERAL, because the literal whitelist admits 1000 whatever it counts | L:1454 | — | `probes/surf/SURF-01.tally` |
| `P:SURF-02` | expressed |   | L:281 | — | `probes/surf/SURF-02.tally` |
| `P:SURF-03` | expressed |   | L §5.16 | — | `probes/surf/SURF-03.tally` |
| `P:SURF-04` | expressed |   | L:1590 | — | `probes/surf/SURF-04/SURF-04.tally` |

### 4.2 Questions, by household

#### A

| Row | Status | Reason | Spec | Gap | Text |
|---|---|---|---|---|---|
| `Q:A-01` | breaks | steps 3-14, 24-25: base_p, adv_p, hded_p sum `where earner[e, y] = s` (tax.tally:72-76); for 2021-2022 earner[sio, y] is `-` (people.tally:19), and comparing `-` with a key is RUN-NA (L:281: `-` only tested by `= -` / `!= -`), so every 2021 and 2022 figure is refused where the key has values. The language can say it (`earner[e, y] != - and earner[e, y] = s`, as MP-03:102); with that guard the judge's hand figures match the key. Also PIT-6 .1's joint-filing conditions (married all year, community, residents) are never held or tested, so joint_tax is unconditional (LAW-07) | `L §5.5`, `L §5.10`, `L §4.3`, `L §6.4` | — | `households/A/asks/A-01.tally` |
| `Q:A-02` | breaks | steps 1-4: the same `where earner[e, y] = s` at y = 2022; as known 2022-05-31 earner[sio, 2022] is `-`, so joint_tax[2022], adv_p and together_joint[2022] are refused (RUN-NA, L:281) where the key has values; same fix as A-01 | `L §4.5`, `L §4.6`, `L §4.3` | — | `households/A/asks/A-02.tally` |
| `Q:A-03` | awkward | income and net steps: er_paid and ee_paid round the basic and top-up/extra contributions once, on their sum (payroll.tally:169-181), where WCP-1 .4 (acts.md:299, GEN-5 .3) rounds each contribution; the values agree only on this dossier's pays (6500, 116500). Named for the employer (payroll.tally:5), not the employee; a workaround around NUM-03 (L §7.2) | `L §4.2`, `L §4.3`, `L §5.10`, `L §7.2` | — | `households/A/asks/A-03.tally` |
| `Q:A-04` | expressed |   | `L §5.5`, `L §4.2`, `L §4.3` | — | `households/A/asks/A-04.tally` |
| `Q:A-05` | can't express | Steps 1–3 and 5–7: Tomek's lots are the BUY rows of a fund register. A record is `movement CUR` (L:188) with an amount in the place's currency (L:1125), and fund units have no currency; the CSV dialect can't be imported either (O:A-1, O:A-2). So his gains, losses and taxes are holes where the key has values. Steps 10–11: Ania's March 2019 lot has an imprecise date, so the plain ask is HOLE-IMPRECISE, as the key expects; two pinned asks with one day each stand for the key's two date ranges (awkward). Step 12: "why Ania's payment differs" is a list of reasons, which only `explain` and broker_tax beside cg_tax can show (awkward) | `L:188`, `L:1125`, `L §5.18`, `L §4.4`, `L §5.16`, `L §5.11` | — | `households/A/asks/A-05.tally` |
| `Q:A-06` | expressed |   | `L §5.8`, `L §4.3`, `L §3.2`, `L §7.2` | — | `households/A/asks/A-06.tally` |
| `Q:A-07` | expressed |   | `L §5.5`, `L §5.10`, `L §4.2`, `L §7.2` | — | `households/A/asks/A-07.tally` |
| `Q:A-08` | awkward | Step 1: the money that moved because of the sale is picked by naming record rows in a def (sold_in); no construct selects movements by cause. The car kept isn't a scenario, because no pays are booked before walk_from (the records can't be imported), so the kept costs are defs (kept_cost) | `L §5.18`, `L §6.4` | — | `households/A/asks/A-08.tally` |
| `Q:A-09` | can't express | Steps 1–12: the balance on the 15th, the card average, the year's wrapped contributions and what was moved all come from wrz-joint-*.csv and brz-wrapped-ania.csv. OPERATIONS §2.2 names no dialect for their header lines, `;` separators, decimal comma or day-first dates, so no version covers those accounts and every aggregate guarded by covered() is a hole where the key has values. Rule days, known payments and both own transfers match the key | `L §5.18`, `L:1150`, `L §8.1` | — | `households/A/asks/A-09.tally` |
| `Q:A-10` | case suspect | key step 3 calls the order of A-HR1 v2 and A-HR4 unstated, but A-HR1 v2 .1 counts the balance "with everything booked that day except the rule's own transfers", so Zosia's 500.00 is counted first and investing takes the rest above the floor: the dossier settles what the key calls missing. The text still gives investing as a hole, for other reasons (card average from unimported records, no January 2026 card data) | `L §5.13.1`, `L §5.13.2`, `L §8.1`, `L §4.4` | — | `households/A/asks/A-10.tally` |
| `Q:A-11` | case suspect | key step 17 counts only the units bought with the gifted units' proceeds and leaves out Ania's 0.01 cash left in Rzeka 4471-07 (rzk-4471-07-anna.csv line 7, "Cash after 0,01"), hers alone (CIV-1 .3): Ania and the household are 0.01 higher. Also step 7: A-D31 lets the bank change the rate only "by notice", A-D32 sets 4.20 % from 2024 with no end, and the story says the bank didn't write, which arguably settles the rate the key calls missing | `L §5.18`, `L §3.6`, `L §8.1`, `L §7.2` | — | `households/A/asks/A-11.tally` |
| `Q:A-12` | case suspect | the key gives values that need January 2026's A-HR1 v2 investment (story forecast item 1 starts the fixed sum in February), which A-10's key calls a refusal: keys A-10 and A-12 contradict each other. Story item 2 ("after 2026 we take 27 300.00 and 600.00 more each year") reads two ways for 2027 | `L §6.4`, `L §5.13.1`, `L §5.5`, `L §4.3` | — | `households/A/asks/A-12.tally` |


#### B

| Row | Status | Reason | Spec | Gap | Text |
|---|---|---|---|---|---|
| `Q:B-01` | breaks | key step income-tax base 2024 (form B): ask b01_forms at K 2025-04-30 reaches depreciation[2024] (business.tally:162), which reads car_sold_on (:164) = first(i in inv_duk where sale[i].sup = asset_sale: …); the only asset_sale invoice, fv_019_2025, is known 2025-09-12, so at K the set is empty and `first` is refused (RUN-EMPTY, L:776): tax_b is refused where the key has a value. Also step 14: deductible (:151-153) rounds the 50 % fuel VAT half-up to the ort, while the key carries the exact half and GEN-5 .5 rounds only named amounts (L §7.2) | L §5.10 | — | `households/B/business.tally`; `households/B/asks/B-01.tally`; `maps/B/B-01.md` |
| `Q:B-02` | expressed |   | L §4.2 | — | `households/B/business.tally`; `households/B/asks/B-02.tally`; `maps/B/B-02.md` |
| `Q:B-03` | awkward | key steps dukats got / credited day: duk_got and credited (business.tally:200-204) hang on sold_on (:196, `first` over laz rows), so b03_invoices prints RUN-EMPTY cells for the unsold FV 015, 018, 026, 028 where the key shows no sale; B-HR4 .5 names no rounding mode and :202 uses half-up silently instead of a mode axis (L:1485; same value under every mode); fate tests bal at end_of(year) = 2026-01-01 (same value) | L §4.3 | — | `households/B/business.tally`; `households/B/asks/B-03.tally`; `maps/B/B-03.md` |
| `Q:B-04` | breaks | key steps studio tax if nothing more is spent / if the kitchen is done: the versions in scenarios/nothing_more.tally:8 and kitchen.tally:8 carry `known 2022-05-01`; a version of a `from D` scenario has no `known` line (LOAD-SCENARIO, L:1250-1251), so both scenarios and the asks b04_end_nothing, b04_end_kitchen are refused at load. Also studio_tax (flat.tally:136-138) never tests PIT-12 .2's 5-year condition (no acquisition-date fact) and tt1_tax (:128) never tests TT-1's conditions stated in B-D04 | L §6.4 | — | `households/B/flat.tally`; `households/B/scenarios/kitchen.tally`; `households/B/scenarios/nothing_more.tally`; `households/B/asks/B-04.tally`; `maps/B/B-04.md` |
| `Q:B-05` | breaks | step 4: scenarios/no_holidays.tally:8 carries `known 2023-01-24` in a `from 2022-08-01` scenario: LOAD-SCENARIO (L:1250-1251) refuses the scenario, so the base-and-branch ask is refused at load (on top of the writer's named workarounds: the L:1258 difference pattern, the assumed r/12 periodic rate) | L:1258 | — | `households/B/flat.tally`; `households/B/scenarios/no_holidays.tally`; `households/B/asks/B-05.tally`; `maps/B/B-05.md` |
| `Q:B-06` | awkward | values match the key, but: car_tax (business.tally:217) applies the flat rate to the net and skips PIT-15 .4 (contributions off the 12 % part first); VAT-5 .2's 50 % is applied by purchase kind, mixed use and no mileage log (B-D11) not held or tested; car_loan_interest (:219) hard-codes one day and half-up where B-D12 .4 names no mode, standing in for B-D16's stated 2.92; car_loan_fee_due (:221-223) never tests CC-2's cap or fixed-rate condition; version car_tax_notice (:54-58) invents `known 2025-01-01` with an assumed line, though B-D14 is issued and received 2025-02-17 (L §4.5) | L §5.10 | — | `households/B/business.tally`; `households/B/asks/B-06.tally`; `maps/B/B-06.md` |
| `Q:B-07` | awkward | Step 1: the October gross is found by `argmin` over a cent index with a pin per key, not by solve (the net is a step function). The key's "only one within ±20.00" is not checked. The 2024 return is a second ask that pins the found gross | L §5.10 | — | `households/B/ola.tally`; `households/B/asks/B-07.tally`; `maps/B/B-07.md` |
| `Q:B-08` | breaks | Steps 2-5: the `residuals` ledger shows corrections at anchors, not the key's four rows. The duplicate shows as +111.27 at the 10-15 anchor. The reversal is explained, so it is not listed. The gap's net is a hole. The card fee shows only at the 12-29 anchor | L §5.18 | — | `households/B/records/cis-2025.tally`; `households/B/asks/B-08.tally`; `maps/B/B-08.md` |
| `Q:B-09` | breaks | Step 2: FV 016 should be a hole. The gap's hole row sits on Cis, and the spec does not say whether it touches the claim `inv_owed[fv_016_2025]`. So the claim keeps 4270.00 with status "missed": a value where the key has a hole | L §5.18 | — | `households/B/rules.tally`; `households/B/asks/B-09.tally`; `maps/B/B-09.md` |
| `Q:B-10` | awkward | Partial: 11-26 is a hole, so the known days need a second ask. v2 since January is built from recursive `as if` defs. v3 is a LOAD-CYCLE in a scenario file, argued from the spec (loadcheck has no cycle check). The `without` block needs rows the dossier lacks | L §5.13 | — | `households/B/rules.tally`; `households/B/scenarios/hr2_v3.tally`; `households/B/asks/B-10.tally`; `maps/B/B-10.md` |
| `Q:B-11` | awkward | Step 3: Ola's Wrzos balance is a hole (no statement after March), so the known part needs a second ask. The `without` block needs rows the dossier lacks | L §8 | — | `households/B/rules.tally`; `households/B/flat.tally`; `households/B/asks/B-11.tally`; `maps/B/B-11.md` |
| `Q:B-12` | case suspect | the key's tax deducts contributions whose AVG-WAGE-Q4 for February 2026 on (9180.65, gazette.csv:16) is published 2026-01-20, after the cut 2026-01-02 (story: the forecast as we see it on 2026-01-02; F6 "before those, the gazette's"). Text faults too: scenarios/no_overpay.tally:8 carries `known` in a `from` scenario (LOAD-SCENARIO, L:1251), and the text has no tax/VAT/health pays, no F5 January items, no B-HR5 | L §4.5 | — | `households/B/forecast.tally`; `households/B/scenarios/no_overpay.tally`; `households/B/asks/B-12.tally`; `maps/B/B-12.md` |


#### D

| Row | Status | Reason | Spec | Gap | Text |
|---|---|---|---|---|---|
| `Q:D-01` | awkward | Every figure maps as "same". Relation 3 ("each child's payout is the one A and B were told") compares this reality with households A and B. An ask reads only one reality (L §6.1), so the relation holds only because the case set was built that way; no line here shows it. | `L §5.10`, `L §7.2`, `L §5.18`, `L §6.1` | — | `households/D/asks-D-01.tally` |
| `Q:D-02` | awkward | step 3: estate.tally:80 (rule `reported`) exempts any heir who reported by report_by and :78 taxes every heir as group_1, without testing IGT-4 .1's condition (spouse or descendant): it holds only because every heir happens to be close family, said only in the note at :77. Off the key's value path: acq_value (:74) values the bonds at death_day, not the day the tax arose (IGT-2 .2); igt_free read at the cut, not that day (IGT-3 .2); no IGT-3 .1 adding up; base rounded to 0.01 under hh_round, not whole dukats (IGT-2 .3) | `L §5.10`, `L §4.2`, `L §4.3`, `L §4.6`, `L §8.1` | — | `households/D/asks-D-02.tally` |
| `Q:D-03` | awkward | Step 16 (without the twist): `surv_from` and `v2_effective` are pinned to 2064-12-01, a date that stands for "never". Pinning them to `-` instead would make `start_of(m) >= surv_from` a RUN-NA (L:281, pins L:811). Relation 1 (the records' credits are the law's nets) isn't checked: the SIO credits are export rows that aren't loaded (O:D-1). | `L §5.10`, `L §5.16`, `L §4.3`, `L:281` | — | `households/D/asks-D-03.tally` |
| `Q:D-04` | awkward | Step 4 (without the twist): `surv_from`, `v2_effective` and `lease_from` are pinned to far dates that stand for "never" (`as if` recomputes pays, L:809–821). The relation (the records' advances and payment are the return's) is checked against D-D17's statement, not the export (O:D-1). | `L §5.10`, `L §5.13`, `L §7.2`, `L §5.16` | — | `households/D/asks-D-04.tally` |
| `Q:D-05` | awkward | Steps 1–2: the two export problems are shown, not found. Finding them means chaining every row's balance, and the exports can't be read (O:D-1, L §5.18). Step 3: the quarter index is a month range `by 3`, so each quarter's span is written out (O:D-4, L §5.6). The replay relations need every row. | `L §5.18`, `L §5.13`, `L §5.6`, `L §4.2` | — | `households/D/asks-D-05.tally` |
| `Q:D-06` | awkward | Step 5: the 2026-12-31 total is a hole, as the key says. A partial has no form (a sum with a hole term is a hole, L §5.10), so a second ask filters the series by an interest-year month written by hand. | `L §5.10`, `L §4.6`, `L §8.1`, `L §7.2` | — | `households/D/asks-D-06.tally` |
| `Q:D-07` | can't express | Step 3 (without the twist: the lowest balance with no escrow and no widow's costs) needs every movement in the exports, and tally 1.0.1 can't read them (O:D-1, L §5.18). Steps 1–2 are workarounds: the low days and the lowest day are read from anchors on the key's own dates, not found by D-HR2's test. Paid-on, bonds, tax and net are "same". | `L §5.18`, `L:918`, `L §4.2`, `L §5.10` | — | `households/D/asks-D-07.tally` |
| `Q:D-08` | awkward | The house is a hole as the key says. The net worth is partial, which has no form, so a second ask leaves the house out (L §5.10). | `L §5.10`, `L §4.6`, `L §8.1`, `L §5.18` | — | `households/D/asks-D-08.tally` |
| `Q:D-09` | awkward | step 4: sale_income (sale.tally:29-35) covers only the bought 9/16 and sale_tax tests only bought_taxed; inherited_taxed (:23) is read by nothing, so step 2's answer is built into the tax by hand. LAW-10's settling date (2029-12-31) is never computed: pit12_spend_years is unread (WARN-UNREAD), the window lives only in sale_spent's question text. Step 2 takes Zbigniew's own acquisition (2011-12-02) where the key takes his mother's (PIT-12 .2 inheritance fiction); both give false. Figures match | `L §5.10`, `L §5.16`, `L §7.2`, `L §8.1` | — | `households/D/asks-D-09.tally` |
| `Q:D-10` | awkward | Paths: L §5.14 has no Student-t, so the t with 4 df is built from five normal draws. The key's Python random stream can't be reproduced, so it is a bound (`py_stream`, L §5.17) and the path figures can only agree within their standard errors. The uniform draw needs `to` (S:18). | `L §5.14`, `L §7.4`, `L §5.16`, `L §5.17`, `L §8.2` | — | `households/D/asks-D-10.tally` |
| `Q:D-11` | awkward | The key's "follows / doesn't follow the index" lists are a reverse dependency. `inputs-of` only goes forward (L §5.16), so the lists are built from one ask per item. The bonds follow the index only through the scenario's own `cpi_yy` rows, a second statement of the 8 % kept equal to `lv_infl` by hand. | `L §6.4`, `L §5.16`, `L:1258`, `L §8.2` | `C7-F2` | `households/D/asks-D-11.tally` |
| `Q:D-12` | breaks | steps 1-3: fund.tally:18-34 types units `int` (f_units, f_sell) against a price `num DUK/fund_unit` (:6, :9): round(fund_capital / f_price, 1, down) is `num fund_unit`, not int, and f_units * f_price is `num DUK/fund_unit`, which no round turns into money: LOAD-UNIT (L §3.2, L §8.3), on NUM-01, the edge the question tests. Retyping as `num fund_unit` (as the spec's LH-01) fixes it; the judge reproduced all 69 key rows from the CSVs otherwise. A load refusal stops the whole reality (L §8): per D7 it is charged to this row only | `L §5.10`, `L §7.2` | — | `households/D/asks-D-12.tally` |


#### E1

| Row | Status | Reason | Spec | Gap | Text |
|---|---|---|---|---|---|
| `Q:E1-01` | awkward | Every key step maps. But the payout accounts are keys in lower case with hyphens (`O:E1-6`), and Ania's arrival day needs one Wrzos export row written by hand, citing its file and line (`O:E1-5`). Marta's account and Marta's and Jakub's arrival days are holes with questions. Five checks compare D's payouts with what B and A state and what their records show. | L §5.18 | — | `households/E1/asks/E1-01.tally` |
| `Q:E1-02` | awkward | The key is partial. The full asks are holes, so the known part needs a second pair of asks that pins five missing items to 0: `laz_rate` as A-11 does, `house_value` as D-08 leaves it out, and three E1 inputs. B books Ola's Wrzos balance from its plan after the March statement, so E1 takes that balance out and asks for it. Jakub is counted from D's papers. | L §5.16 | — | `households/E1/asks/E1-02.tally` |
| `Q:E1-03` | awkward | step 3 and step 5's base: asks/E1-03.tally:7 reads gift_tax, acq, acq_from, acq_to, cash2021/units2023 (declared only in households/A/gift.tally), mk_glb/px_glb (A/prices.tally) and the stated[konto] anchor (D/records.tally), none of them in households/E1/: the text reaches them only through the writer's own assembly rule (family.tally note: the households' files "as they are", a copy under a/, b/, d/ "stands in for" the household's file), which the spec has no construct for (L §6.1: one reality, no include, no override). Read as households/E1 + law the ids are unknown (LOAD-NAME, unseen by loadcheck) and the reality is refused at load. In the hand-assembled union every step means the key's (igt_arises 2024-11-03; report_by 2025-05-05; 308; reported/small chain, L §5.10 steps 4-5), and E1's copy now tests IGT-4 .1. Same assembly under E1-01 and E1-02 | L §5.10 | — | `households/E1/asks/E1-03.tally` |

### 4.3 Observations

| Row | Status | Reason | Spec | Gap | Text |
|---|---|---|---|---|---|
| `O:TIME-02-1` | can't express | A holiday stated as a rule ("22 June, from 2026") can only be written as a `dates` list of literals per span (L §2.3 `value`, L §4.2 Lists). There is no form for a computed list of days, so the text lists 2026 only and later years are holes | `L §4.2`, `L §2.3` | — | `probes/time/TIME-02.tally` |
| `O:TIME-03-1` | awkward | In Tally a time key does set the law date by default (L §4.3 rule 2). A law keyed on payment needs an explicit `at`, so a forgotten `at` gives the by-period tax without a warning. That is the opposite of the boundary's "never by itself" | `L §4.3` | — | `probes/time/TIME-03.tally` |
| `O:ASK-03-1` | awkward | The planned sale has no date, and a pay needs one (L §5.9 "Places need dates"), so the text uses an `assumed` day. The card also gives no sale price (a hole) and no cost method (average, noted in the text) | `L §5.9`, `L §4.5` | — | `probes/ask/ASK-03.tally` |
| `O:ASK-04-1` | awkward | The card gives 6 % and 8 % with no basis; the text types them `simple /year` (L §3.3). A basis lives in a type, which can't be marked `assumed` (only rows, versions and indexes can, L §8.2), so this reading never reaches the register | `L §3.3`, `L §8.2` | — | `probes/ask/ASK-04.tally` |
| `O:PTY-04-1` | awkward | the restated income can't be a later version: both figures are stipulations known from `on` (L:459), so two rows of `income[tomek, 2024]` tie (L §4.6 step 3, LOAD-OVERLAP). The workaround writes "after" as a `with` pin | `L §4.5`, `L §4.6`, `L §5.16` | — | `probes/pty/PTY-04.tally` |
| `O:OWN-03-1` | breaks | the `tax_payment` instance on 2025-04-30 falls after E (2025-03-31). The spec says the plan runs to E (E:81) but not what becomes of a pay instance dated after E (silent). No asked value depends on it | `L §5.13`, `E:81` | — | `probes/own/OWN-03.tally` |
| `O:FLOW-01-1` | awkward | the balancing flow is never booked: an anchor's difference is a balance line, not a movement (L:1142, E:253). So `flows(wallet, 2025-01)` reads +300.00, not −150.00. The text computes the flow as a def | `L §5.18`, `E:253` | — | `probes/flow/FLOW-01.tally` |
| `O:FLOW-02-1` | awkward | nothing groups the three legs of the settlement or makes them settle together. The text writes them as three separate pays sharing one date | `L §5.13` | — | `probes/flow/FLOW-02.tally` |
| `O:REC-02-1` | can't express | The card's CSV dialect (separator, decimal comma, dd.mm.yyyy dates, header lines) belongs to the importer. Tally has no text-input forms (O §2.2, L §10 "Text, input and output"), so the text holds plain rows | `L §10` | — | `probes/rec/REC-02.tally` |
| `O:SCEN-02-1` | awkward | A month-keyed pay's `when` is read on the month's first day (L §4.3), so a fact that changes mid-month (sale on 15 Sep) is read as of 1 Sep for a 20 Sep instance. Getting it right needs a def using `at day_of(m, 20)`. A scenario row also can't govern a month whose first day comes before the scenario's `from` | `L §4.3`, `L §6.4` | `C7-F2` | `probes/scen/SCEN-02/SCEN-02.tally` |
| `O:LAW-12-1` | awkward | The minors' exception belongs on the duty to report, not on exempt. Written on exempt, a rule excepting `late` would be transitive over `close` and make Leon exempt. Workaround: a separate def `must_report`, with `late` guarded by it | L §5.10 | — | `probes/law/LAW-12.tally` |
| `O:RULE-02-1` | awkward | the spec has a planned-only look-ahead (`plan_low`, without promised pays), but no ask or dated form of it, so a "lowest planned balance and when" question can't use it | L:920 | — | `probes/rule/RULE-02.tally` |
| `O:A-1` | can't express | The record CSVs (header lines above the columns, `;`, decimal comma, dd.mm.yyyy and dd/mm/yyyy dates) have no importer dialect (OPERATIONS §2.2). Only the rows a question reads are written, each by key, and pasting the rest by hand is forbidden | `L §5.18`, `L:1128` | — | `households/A/records.tally` |
| `O:A-2` | can't express | Fund registers (Brzask, Rzeka, DM Kopia, Wektor) move units. A record is `movement CUR` (L:188), so the Brzask and Rzeka positions stand as `assumed` stated anchors | `L:188`, `L:1125`, `L §5.18` | — | `households/A/records.tally` |
| `O:A-3` | can't express | GEN-1: the fixed holidays are settled years ahead, but a year's list is one `dates` value, so it is complete only once the Spring Day notice exists. A year without a notice can't say "every day settled except one Monday in April", because `dates` holds days, not imprecise dates. The forecast assumes 2027–2035 | `L §3.1`, `L §4.2` | — | `law/GEN-1.tally` |
| `O:A-4` | awkward | One lot input covers several funds, so every fund shares one unit, fund_u (an input has one type). The unit algebra then can't stop GLB units being multiplied by a BND price | `L §3.2`, `L §5.7` | — | `households/A/reality.tally` |
| `O:A-5` | awkward | `next_business(d, h)` reads h at the def's law date, not in d's year. With one holiday row per year, a mismatch silently uses the wrong year's list. The text adds `at` to rule_day, so_day, pay_on and lot_cost | `L:399`, `L §4.3` | — | `households/A/rules.tally` |
| `O:A-6` | awkward | `owns` splits a place by share, not by origin: Ania's savings hold her estate payout (hers, CIV-1 .3) and its interest (community, CIV-1 .2). The def nw_ania_sav_interest does the split | `L §3.6`, `L §5.9` | — | `households/A/worth.tally` |
| `O:A-7` | awkward | A movement input has no string field type for the counterparty (L:186–188), so the party's name goes in the locator | `L:186`, `L §5.18` | — | `households/A/records.tally` |
| `O:A-8` | awkward | The 2021–2025 past isn't walked: walk_from 2026-01-01 is assumed because the records can't be imported, so the closing anchors of 2025 and the dated documents stand in for the walk | `L §5.18`, `L §5.13` | — | `households/A/plan.tally` |
| `O:B-1` | can't express | the record CSVs (Cis `#` header lines and dd-mm-yyyy, Wrzos `;` and decimal comma, Lazur, Faktura+) have no format a text loads: O §2.2 names no file format (same gap as O:A-1). The writer stood in for the pipeline with a script writing every row, citing file and line (supervisor decision D6: accepted, against the brief's "at most a few rows") | O §2.2 | — | `households/B/records`; `households/B/registers.tally` |
| `O:B-2` | case suspect | The `without` blocks of B-08 to B-11 depend on the 11-14..30 rows and on no reversal. No document holds those rows, so they can't be written from the dossier | L §5.18 | — | `maps/B/B-08.md` |
| `O:B-3` | case suspect | The B-12 key reads AVG-WAGE-Q4 2026, published 2026-01-20, at a known-as-of of 2026-01-02. The reality's `on` 2026-01-07 refuses that row, so I dropped awq4_2026 from SOC-4 | L §4.5 | — | `law/SOC-4.tally` |
| `O:B-4` | awkward | GEN-1 lists holidays only to 2026, so `next_business` in the 2027-28 forecast sees weekends only | L §4.2 | — | `law/GEN-1.tally` |
| `O:B-5` | awkward | B-D07 .4 says "annuity" but gives no periodic rate. I assumed nominal 12/year (`periodic(r, month)`) | L:323 | — | `households/B/flat.tally` |
| `O:D-1` | awkward | The Bank Olcha exports are `\|`-separated, with yyyymmdd dates, a decimal comma and header lines. tally 1.0.1 can't read them as they stand, because the import (O §2.2) names no format; this is the same gap as O:A-1. As the supervisor said, there is no import stand-in: `records.tally` holds only the rows a question reads (anchors, the deposit row, the two February rows, rent days, Kuba's transfers), each citing its file and line. | `L §5.18` | — | `households/D/records.tally` |
| `O:D-2` | awkward | TSB-2 wants the CPI "as published on the interest year's first day". It is written `cpi_yy[…] as if {known iy_start[s, k]}` (bonds.tally lines 23–26). A `known D` pin sets the inner cut to D even when D is later than the ask's cut (L:815). So an ask as known K, before an interest year that starts by `on`, would read a CPI published after K. The right pin is min(iy_start, K), and no expression can read K. No asked figure is affected. | `L:815`, `L §4.5` | — | `households/D/bonds.tally` |
| `O:D-3` | awkward | Several sources say "to the ort" without a rounding mode: the D-D25 rent indexation, distribution gross amounts, units × price, the estate sum, the life table. These go on the axis `hh_round`, so every ask that reaches them prints two lines, which differ only on a tie. | `L §7.2`, `L §5.5` | — | `households/D/reality.tally` |
| `O:D-4` | awkward | There is no quarter period. L §5.6 gives `by` only for integer ranges, though the EBNF (L:144) also allows a month range with `by`. So `quarter = month 2025-07..2026-09 by 3` keys each quarter by its first month, and every quarterly `flows` writes its span out. | `L §5.6`, `L:144`, `L §4.1` | — | `households/D/ogrodowa.tally` |
| `O:E1-1` | awkward | One reality takes one model header and one namespace, so the three households join only as copies. Three headers became part headers; 34 clashing ids were renamed `b_`/`d_` (652 tokens in 37 files). There is one on/start/end for all, so an ask with no `as known` now reads at 2026-09-30. Tally has no way to include one reality in another. | L §6.1 | — | `households/E1/family.tally` |
| `O:E1-2` | awkward | A sums its spouses' worth over every place, filtered by `owns`. In the family the place index also holds 65 places of B and D that are not outside, and a sum whose `where` meets a hole is a hole. So each of those places gets an `owns` row of 0, generated; each invoice claim's row is known on the day B adds that key. | L:795 | — | `households/E1/owns.tally` |
| `O:E1-3` | awkward | B's `opening[*] = 0` covers every key the place index has on its known date. In the family that would open A's joint account and D's konto, both left without an opening on purpose. The row is restated for B's 19 places in a copy of B/places.tally. | L:1222 | — | `households/E1/b/places.tally` |
| `O:E1-4` | awkward | A's `nw_place` and `fc_place` are typed money DUK over every place, and B's florens places now reach them. The copies add a rule that makes another currency a hole. That loads only if units are checked key by key, which L §3.2 doesn't say. | L §3.2 | — | `households/E1/a/worth.tally` |
| `O:E1-5` | can't express | As `O:A-1`: Tally reads no export file. The one Wrzos row E1-01 needs is written by hand with its file and line, in a version that covers nothing. | L §5.18 | — | `households/E1/estate.tally` |
| `O:E1-6` | awkward | Tally has no text type, and a key can't hold spaces or capitals. The answer prints ln45-2090-0017-5521-3380-0002 where the key has "LN45 2090 0017 5521 3380 0002". | L §2.1 | — | `households/E1/asks/E1-01.tally` |
| `O:E1-7` | awkward | One person has several keys: Ania is A's `ania` and D's `anna`, and Jakub is D's `jakub` and the Kuba in D's names. Nothing in Tally says that keys of two indexes are one person, so `e1_person` and its rules map them by hand. | L §5.6 | — | `households/E1/worth.tally` |

### 4.4 Spec findings

| Row | Status | Reason | Spec | Gap | Text |
|---|---|---|---|---|---|
| `S:1` | breaks | `vitem` row: `value = lit ["assumed" STR]` precedes `rowmeta = ["governs" lspan] [STR]`, so the EBNF needs `x = v assumed "r" governs span`; the spec's own household writes `x = v governs span assumed "r"` (facts/2026.tally:10–16, checked by the supervisor). Grammar and spec text disagree on clause order | `L §2.3` | — | `engine/spec/amendments/household/facts/2026.tally` |
| `S:2` | breaks | `call = (ID … \| BUILTIN) "(" …` with BUILTIN covering every §2.2 built-in also derives the built-ins that have their own production (`agg`, `flowsx`, `crps`, `distfn`, `prorata`): `count(x in xage)` parses as `agg` or `call` (models/CH-02.tally:82), `flows(invest[ike], y)` as `flowsx` or `call` (asks/core.tally:9); 41 places in 14 spec files. The "Calls" prose names only `min`/`max` as taking plain arguments, while §5.13.1 also writes `bal(x)` plainly | `L §2.3`, `L §5.13.1` | — | `engine/spec/models/CH-02.tally` |
| `S:3` | breaks | latent EBNF ambiguities no spec text hits: `agg`'s `{"," bind}` against `["," expr]` (`count(k in a, j in b)`: second part a bind or an `in` comparison, two meanings); `bal(x in place)` / `plan_low(…)` as `group` or plain `call`; `min(k in a)` as aggregate or call | `L §2.3` | — | — |
| `S:4` | breaks | The CUR grammar takes any 3 upper-case letters (L:218), but `money CUR` takes its minor unit from ISO 4217, or `XXX` (L:268, §3.1). A made-up currency (DUK, FLR) has no minor unit, and a model can't declare one, so `LOAD-PRECISION` and turning a `num CUR` into money (§3.2, §7.2) are undefined for it | `L §3.1`, `L §3.2`, `L §7.2` | — | `engine/spec/LANGUAGE.md` |
| `S:5` | breaks | `explain` prints "in the canonical order" (L:1040), but canonical order is defined only for model text (L §2.4). The order of a derivation's causes is undefined, and ENGINE §10 (E:404) gives none either | `L §5.16`, `L §2.4` | — | `engine/spec/LANGUAGE.md` |
| `S:6` | breaks | L §7.1 lets 1000 (and 100, 360, 365, 366) stay bare in defs, rules and pays as plain integers. A 1 000 DUK amount therefore passes the literal rule, so the language can't promise that an amount in a rule's arithmetic is a named fact | L:1454 | — | `engine/spec/LANGUAGE.md` |
| `S:7` | breaks | L:1120 names "a broker's fill" as a record, but a record is `movement CUR` (L:188) whose amount is money in the place's currency (L:1125); a fill moves fund units, which have no currency | `L:1120`, `L:188`, `L:1125` | — | `households/A/records.tally` |
| `S:8` | breaks | L:1150 gives `covered(x, span)`, and LOAD-COVERED requires it on span aggregates, but `call` (L:215) takes only exprs and a span `a..b` (L:177) isn't one. Only a single period value can be passed (the spec's example passes a month); a day range needs `all(d in day where …: covered(x, d))` | `L:1150`, `L:215`, `L:177` | — | `households/A/rules.tally` |
| `S:9` | breaks | The spec is silent on what a hole movement's unknown `explains` does to the claim places and pays it might explain. It only makes its own place's balance a hole | L §5.18 | — | `households/B/records/cis-2025.tally` |
| `S:10` | breaks | `residuals` reports corrections at anchors, not the rows behind them. A duplicate, an explained reversal and a missing pay can't be listed as rows | L §5.18 | — | `households/B/asks/B-08.tally` |
| `S:11` | breaks | `flows` takes only a place or a state pay, so a flow pay's realised amount (`client_pay[i]`) can't be read. The workaround is one claim place per invoice | L:923 | — | `households/B/business.tally` |
| `S:12` | breaks | `opening[x[*]]` for a keyed place over an open index does not parse (`*` only at the top row-key level). I used `opening[*]` for all places | L §2.3 | — | `households/B/places.tally` |
| `S:13` | awkward — withdrawn as a finding | not a spec finding: L:115 and L:717 name the built-in outside places in lower case (`outside_pln`, so `outside_flr`); parsecheck rightly refuses `outside_FLR`. The writer declared its own outside places instead of using the built-in ones | L §5.9 | — | `households/B/places.tally` |
| `S:14` | awkward — withdrawn as a finding | not a contradiction: the grammar (L:152) gives notes only to the rule form of a def, so a short-form def's reason needs the rule form `def x: T` + `note` + `rule base = …` | L §2.3 | — | `households/B/business.tally` |
| `S:15` | breaks | The spec does not say whether a pin's value expression counts in the LOAD-CYCLE check (v3's reserve pins the overpayment) | L §5.10 | — | `households/B/scenarios/hr2_v3.tally` |
| `S:16` | breaks | The spec does not say whether an ask may be declared inside a scenario file (`b10_v3`) | L §6.4 | — | `households/B/scenarios/hr2_v3.tally` |
| `S:17` | breaks | Pinning a record field (`sale[k].net = …`) is allowed by the `pref` grammar, but no sentence in L §5.10 says a pin may target a record input's field | L §5.10 | — | `households/B/asks/B-06.tally` |
| `S:18` | breaks | The EBNF `draw` (L:158) and `cdist` (L:212) put `uniform` among the distributions that need `to UNIT`, but §5.14's table (L:998) says uniform is exact. V5-P3 patched the prose, not the EBNF. parsecheck follows the EBNF, so longview.tally line 28 writes `to 1e-12`. | `L:158`, `L:212`, `L §5.14` | — | `households/D/longview.tally` |
| `S:19` | breaks | The EBNF `pin` (L:180) is `pref "=" expr \| "u" "(" ref ")" "=" lit \| "law" expr`. But §5.10 (L:809–815) also gives `known D` and `wording D for […]`, and §6.4 (L:1258) gives `scenario s`. parsecheck's grammar accepts all three (tally.lark lines 203–205). They are used at bonds.tally line 26 and asks-D-11.tally lines 5, 7 and 21. | `L:180`, `L §5.10`, `L §6.4` | — | `households/D/asks-D-11.tally` |

Text paths that do not exist: none.

## 5. Settlements

Every row whose final status differs from its writer's, or on which the supervisor ruled. The writer's status is the one in its raw message (S rows under their old ids); "—", "observation", "finding" and "ambiguous" are writers' words off the scale, set by the supervisor.

| Row | Writer's status | Final status | Cause | Settlement |
|---|---|---|---|---|
| `S:1` | breaks | breaks | supervisor | settled: supervisor (P0.settled.md has no Settlement column; its note: the supervisor checked each against the spec, "ambiguous" mapped to breaks) |
| `S:2` | ambiguous | breaks | spec unclear | settled: spec unclear (P0.settled.md has no Settlement column; its note: the supervisor checked each against the spec, "ambiguous" mapped to breaks) |
| `S:3` | ambiguous (latent) | breaks | spec unclear | settled: spec unclear (P0.settled.md has no Settlement column; its note: the supervisor checked each against the spec, "ambiguous" mapped to breaks) |
| `O:TIME-02-1` | — | can't express | supervisor | settled: supervisor (status set: W-G1 gave none) |
| `O:TIME-03-1` | — | awkward | supervisor | settled: supervisor (status set: W-G1 gave none) |
| `O:ASK-03-1` | — | awkward | supervisor | settled: supervisor (status set: W-G1 gave none) |
| `O:ASK-04-1` | — | awkward | supervisor | settled: supervisor (status set: W-G1 gave none) |
| `S:4` | — (as S:101) | breaks | supervisor | settled: supervisor (status set; checked L:218, L:268) |
| `S:5` | — (as S:102) | breaks | supervisor | settled: supervisor (status set; checked L:1040, L §2.4) |
| `P:OWN-04` | expressed | awkward | writer misread | judge: downgrade; settled: writer misread (judge upheld; supervisor read lines 37, 74) |
| `P:VAL-06` | case suspect | case suspect | case suspect | settled: case suspect (supervisor read card and key: no 2024 earnings stated, worked line adds none; by L §4.6 earn[2024] is a hole unless the card meant 0; text writes 0 assumed) |
| `P:SCEN-02` | case suspect | case suspect | case suspect | settled: case suspect (supervisor read card and key: kept credit's 20 Sep 2024 instalment falls after the 15 Sep branch, key starts it 2024-10 with no reason; would make the difference -25300.82) |
| `P:LAW-12` | expressed | awkward | writer misread | settled: writer misread (judge confirmed; supervisor downgrade after reading the card, boundaries LAW-12 and L 5.10 step 4) |
| `P:RULE-08` | expressed | awkward | writer misread | judge: downgrade; settled: writer misread (judge upheld; supervisor read line 22 and boundaries RULE-08) |
| `Q:A-01` | expressed | breaks | writer misread | judge: downgrade; settled: writer misread (judge downgrade; supervisor read L:279-282, people.tally:19, tax.tally:70-77) |
| `Q:A-02` | expressed | breaks | writer misread | judge: downgrade; settled: writer misread (judge downgrade; supervisor checked with A-01) |
| `Q:A-03` | expressed | awkward | writer misread | judge: downgrade; settled: writer misread (judge downgrade; supervisor read payroll.tally:165-182 and acts.md:299) |
| `Q:A-10` | case suspect | case suspect | case suspect | settled: case suspect (supervisor read house-rules A-HR1 v2, A-HR4 and the key) |
| `Q:A-11` | case suspect | case suspect | case suspect | settled: case suspect (supervisor read the key, the Rzeka CSV, A-D31, A-D32, story lines 34-77) |
| `Q:A-12` | case suspect | case suspect | case suspect | settled: case suspect (supervisor read story forecast items 1-7 and both keys) |
| `Q:B-01` | expressed | breaks | writer misread | judge: downgrade; settled: writer misread (judge downgrade; supervisor read business.tally:160-165 and L:776) |
| `Q:B-03` | expressed | awkward | writer misread | judge: downgrade; settled: writer misread (judge downgrade) |
| `Q:B-04` | expressed | breaks | writer misread | judge: downgrade; settled: writer misread (judge downgrade; supervisor read L:1236-1251 and the scenario headers) |
| `Q:B-05` | awkward | breaks | writer misread | settled: writer misread (supervisor, from J-B's note; read no_holidays.tally:1-8) |
| `Q:B-06` | expressed | awkward | writer misread | judge: downgrade; settled: writer misread (judge downgrade; supervisor read B-D14 header and business.tally:54-58) |
| `Q:B-12` | case suspect | case suspect | case suspect | settled: case suspect (supervisor read gazette.csv:14-16, story lines 78-90, the key's derivation) |
| `O:B-1` | observation | can't express | supervisor | settled: supervisor (D6) |
| `O:B-4` | observation | awkward | supervisor (status set; no cause written) | settled: |
| `O:B-5` | observation | awkward | supervisor (status set; no cause written) | settled: |
| `S:9` | finding (as S:701) | breaks | spec unclear | settled: spec unclear (unjudged; writer's finding) |
| `S:10` | finding (as S:702) | breaks | spec unclear | settled: spec unclear (unjudged; writer's finding) |
| `S:11` | finding (as S:703) | breaks | spec unclear | settled: spec unclear (unjudged; writer's finding) |
| `S:12` | finding (as S:704) | breaks | spec unclear | settled: spec unclear (unjudged; writer's finding) |
| `S:13` | finding (as S:705) | awkward | writer misread | settled: writer misread (supervisor) |
| `S:14` | finding (as S:706) | awkward | writer misread | settled: writer misread (supervisor) |
| `S:15` | finding (as S:707) | breaks | spec unclear | settled: spec unclear (unjudged; writer's finding) |
| `S:16` | finding (as S:708) | breaks | spec unclear | settled: spec unclear (unjudged; writer's finding) |
| `S:17` | finding (as S:709) | breaks | spec unclear | settled: spec unclear (unjudged; writer's finding) |
| `Q:D-02` | expressed | awkward | writer misread | judge: downgrade; settled: writer misread (judge downgrade) |
| `Q:D-09` | expressed | awkward | writer misread | judge: downgrade; settled: writer misread (judge downgrade) |
| `Q:D-12` | expressed | breaks | writer misread | judge: downgrade; settled: writer misread (judge downgrade; supervisor read fund.tally:5-34) |
| `Q:E1-03` | expressed | awkward | writer misread | judge: downgrade; settled: writer misread (judge downgrade; supervisor read family.tally:1-9 and the a/, b/, d/ listings) |

By cause: writer misread 17, spec unclear 9, case suspect 6, supervisor 10; 42 rows in all.

Judge downgrades upheld by the supervisor: 13 (P:OWN-04, P:RULE-08, Q:A-01, Q:A-02, Q:A-03, Q:B-01, Q:B-03, Q:B-04, Q:B-06, Q:D-02, Q:D-09, Q:D-12, Q:E1-03). Rows the supervisor downgraded with no judge downgrade: P:LAW-12, Q:B-05, S:13, S:14.

## 6. Suspects

Each row settled as case suspect, with its reason. The exam side settles them (ERRATA, version bump) before session 4 starts.

| Row | Reason | Settlement |
|---|---|---|
| `P:VAL-06` | step 6: rule 1 adds "that year's earnings" every year, but the key adds none in 2024 and the card states no 2024 earnings. By the gap rule (L §4.6) `earn[2024]` would be a hole and so would the whole result (L §8.1). The text writes `earn[2024] = 0 assumed` to follow the key | settled: case suspect (supervisor read card and key: no 2024 earnings stated, worked line adds none; by L §4.6 earn[2024] is a hole unless the card meant 0; text writes 0 assumed) |
| `P:SCEN-02` | step 2: the 20 Sep 2024 credit instalment falls after the 15 Sep branch, but the key and its inputs start the credit in 2024-10 with no reason given. The text follows the inputs (`credit_from = 2024-10`) | settled: case suspect (supervisor read card and key: kept credit's 20 Sep 2024 instalment falls after the 15 Sep branch, key starts it 2024-10 with no reason; would make the difference -25300.82) |
| `Q:A-10` | key step 3 calls the order of A-HR1 v2 and A-HR4 unstated, but A-HR1 v2 .1 counts the balance "with everything booked that day except the rule's own transfers", so Zosia's 500.00 is counted first and investing takes the rest above the floor: the dossier settles what the key calls missing. The text still gives investing as a hole, for other reasons (card average from unimported records, no January 2026 card data) | settled: case suspect (supervisor read house-rules A-HR1 v2, A-HR4 and the key) |
| `Q:A-11` | key step 17 counts only the units bought with the gifted units' proceeds and leaves out Ania's 0.01 cash left in Rzeka 4471-07 (rzk-4471-07-anna.csv line 7, "Cash after 0,01"), hers alone (CIV-1 .3): Ania and the household are 0.01 higher. Also step 7: A-D31 lets the bank change the rate only "by notice", A-D32 sets 4.20 % from 2024 with no end, and the story says the bank didn't write, which arguably settles the rate the key calls missing | settled: case suspect (supervisor read the key, the Rzeka CSV, A-D31, A-D32, story lines 34-77) |
| `Q:A-12` | the key gives values that need January 2026's A-HR1 v2 investment (story forecast item 1 starts the fixed sum in February), which A-10's key calls a refusal: keys A-10 and A-12 contradict each other. Story item 2 ("after 2026 we take 27 300.00 and 600.00 more each year") reads two ways for 2027 | settled: case suspect (supervisor read story forecast items 1-7 and both keys) |
| `Q:B-12` | the key's tax deducts contributions whose AVG-WAGE-Q4 for February 2026 on (9180.65, gazette.csv:16) is published 2026-01-20, after the cut 2026-01-02 (story: the forecast as we see it on 2026-01-02; F6 "before those, the gazette's"). Text faults too: scenarios/no_overpay.tally:8 carries `known` in a `from` scenario (LOAD-SCENARIO, L:1251), and the text has no tax/VAT/health pays, no F5 January items, no B-HR5 | settled: case suspect (supervisor read gazette.csv:14-16, story lines 78-90, the key's derivation) |
| `O:B-2` | The `without` blocks of B-08 to B-11 depend on the 11-14..30 rows and on no reversal. No document holds those rows, so they can't be written from the dossier | not judged (writer's status) |
| `O:B-3` | The B-12 key reads AVG-WAGE-Q4 2026, published 2026-01-20, at a known-as-of of 2026-01-02. The reality's `on` 2026-01-07 refuses that row, so I dropped awq4_2026 from SOC-4 | not judged (writer's status) |

Also, with no row: the dossier oddity in part 1 (D-D04 "became final on 3 November 2024" against D's story "becomes final").

## 7. Cost and time

From `rows/agents.tsv` (usage lines from `tmp/cases/tools/usage.py`). The close-out's own cost is not in it.

| Slot | Agent | Role | Chunk | Effort | Start | End | Minutes | Calls | M weighted | % of week | Audit |
|---|---|---|---|---|---|---|---:|---:|---:|---:|---|
| 1 | P0 | probe writer (pilot) | P0 | high | 05:13 | 06:02 | 49 | 47 | 2.01 | 0.23 | no holdout; false positives only |
| 2 | W-G1 | probe writer | G1 | high | 06:06 | 06:22 | 16 | 24 | 1.30 | 0.15 | no holdout; heredoc false positives, scratchpad writes |
| 3 | J-G1 | judge | G1 | high | 06:24 | 06:29 | 5 | 10 | 0.41 | 0.05 | no holdout; loop-variable false positives |
| 3 | W-G2 | probe writer | G2 | high | 06:24 | 06:41 | 17 | 24 | 1.40 | 0.16 | clean |
| 4 | J-G2 | judge | G2 | high | 06:44 | 06:47 | 3 | 11 | 0.46 | 0.05 | no holdout; loop-variable false positives |
| 4 | W-G3 | probe writer | G3 | high | 06:44 | 07:02 | 18 | 19 | 1.29 | 0.15 | clean |
| 5 | J-G3 | judge | G3 | high | 07:05 | 07:10 | 5 | 9 | 0.38 | 0.04 | no holdout; loop-variable false positives |
| 5 | W-G4 | probe writer | G4 | high | 07:05 | 07:23 | 18 | 26 | 1.41 | 0.16 | no holdout; one harmless listing of maps/ |
| 6 | J-G4 | judge | G4 | high | 07:27 | 07:33 | 6 | 10 | 0.46 | 0.05 | clean |
| 6 | W-G5 | probe writer | G5 | high | 07:27 | 07:46 | 19 | 33 | 1.26 | 0.15 | no holdout; one attempted whole-tree grep of tmp/cases aborted by zsh (nothing ran); listing of maps/ |
| 7 | J-G5 | judge | G5 | high | 07:52 | 07:59 | 7 | 9 | 0.45 | 0.05 | clean |
| 8 | W-A | household writer | A | xhigh | 08:03 | 10:05 | 122 | 275 | 9.30 | 1.08 | no holdout; relative-path false positives, scratchpad merged copy |
| 9 | J-A | judge | A | high | 10:15 | 10:25 | 10 | 24 | 0.94 | 0.11 | clean |
| 10 | W-B | household writer | B | xhigh (escalated) | 10:35 | 11:25 | 50 | 87 | 3.34 | 0.39 | no holdout; listings of tally-1.0.1/households/maps (names only); scratchpad scripts; whole-file record conversion by script (D6, deviation) |
| 11 | J-B | judge | B | high | 11:35 | 11:50 | 15 | 21 | 1.09 | 0.13 | no holdout; one harmless listing |
| 11 | W-D | household writer | D | xhigh | 11:35 | 12:30 | 55 | 113 | 3.63 | 0.42 | no holdout; harmless listings |
| 12 | J-D | judge | D | high | 12:35 | 12:45 | 10 | 19 | 0.69 | 0.08 | no holdout; a script written to the supervisor's scratchpad (outside its list, outside the repo) |
| 12 | W-E1 | household writer (family) | E1 | max (escalated) | 12:35 | 13:35 | 60 | 66 | 3.36 | 0.39 | no holdout; listings of tally-1.0.1/households/maps (names); scripts in the supervisor's scratchpad |
| 13 | J-E1 | judge | E1 | high | 13:40 | 13:50 | 10 | 24 | 0.72 | 0.08 | no holdout; loop-path false positives |
| all | supervisor | supervisor (this session, to the close-out's launch) | — | — | 05:00 | 13:55 | 535 | 236 | 5.95 | 0.69 | audits every agent; no holdout access |
| | **sub-agents** | | | | | | 495 | 851 | 33.90 | 3.92 | |
| | **all** | | | | | | | 1087 | 39.85 | **4.61** | |

**Total:** about 4.61 % of the week (sub-agents 3.92 %, supervisor 0.69 %), 1087 calls, 39.85 M weighted; wall time 8h55 (supervisor 05:00–13:55), sub-agent time summed 8h15 (agents ran two at a time). **Against the estimate** (7–11 % of the week, about 10–12 hours): well under on cost, under on time. The cap (14 %) was never near.

**Audits,** one line per agent:

- P0 (probe writer (pilot), P0): no holdout; false positives only
- W-G1 (probe writer, G1): no holdout; heredoc false positives, scratchpad writes
- J-G1 (judge, G1): no holdout; loop-variable false positives
- W-G2 (probe writer, G2): clean
- J-G2 (judge, G2): no holdout; loop-variable false positives
- W-G3 (probe writer, G3): clean
- J-G3 (judge, G3): no holdout; loop-variable false positives
- W-G4 (probe writer, G4): no holdout; one harmless listing of maps/
- J-G4 (judge, G4): clean
- W-G5 (probe writer, G5): no holdout; one attempted whole-tree grep of tmp/cases aborted by zsh (nothing ran); listing of maps/
- J-G5 (judge, G5): clean
- W-A (household writer, A): no holdout; relative-path false positives, scratchpad merged copy
- J-A (judge, A): clean
- W-B (household writer, B): no holdout; listings of tally-1.0.1/households/maps (names only); scratchpad scripts; whole-file record conversion by script (D6, deviation)
- J-B (judge, B): no holdout; one harmless listing
- W-D (household writer, D): no holdout; harmless listings
- J-D (judge, D): no holdout; a script written to the supervisor's scratchpad (outside its list, outside the repo)
- W-E1 (household writer (family), E1): no holdout; listings of tally-1.0.1/households/maps (names); scripts in the supervisor's scratchpad
- J-E1 (judge, E1): no holdout; loop-path false positives
- supervisor (supervisor (this session, to the close-out's launch), —): audits every agent; no holdout access
