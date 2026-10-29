# Case set index

Each visible edge (`boundaries.md`), its probe card, and the questions of A, B, D and E1 that use it, taken from the keys' `edges:` lines. Session 3 fills the status column with the gap log's worst status for the edge (breaks, can't express, awkward, expressed; brief §11); session 4 works from here, one edge at a time.

Written by `tools/index.py`; don't edit by hand.

## NUM: money, units, rounding

| Edge | Name | Probe | Questions | Status |
|---|---|---|---|---|
| NUM-01 | Amounts, quantities and rates carry their units | `probes/num/NUM-01.md` | B-02, B-03, D-01, D-06, D-12 | |
| NUM-02 | Day counts and pro-rating by days | `probes/num/NUM-02.md` | A-04, A-07, B-04, B-05 | |
| NUM-03 ★ | Rounding where and as the law rounds | `probes/num/NUM-03.md` | A-01, A-03, D-06 | |
| NUM-04 | An unclear convention, keyed under each reading | `probes/num/NUM-04.md` | A-07 | |
| NUM-05 | Parts and wholes | `probes/num/NUM-05.md` | A-11, D-01, D-04, D-05, D-08, D-09 | |
| NUM-06 | Which year's money | `probes/num/NUM-06.md` | A-12, D-10, D-11 | |
| NUM-07 | Gross, net, sign and zero | `probes/num/NUM-07.md` | A-05, B-07 | |

## TIME: dates, calendars, periods, windows

| Edge | Name | Probe | Questions | Status |
|---|---|---|---|---|
| TIME-01 | A date known only to the month | `probes/time/TIME-01.md` | A-05, A-13 | |
| TIME-02 | Calendars and dates stated as rules | `probes/time/TIME-02.md` | A-03, A-09, B-01, B-02, B-09, B-13 | |
| TIME-03 | The period an amount is for, and the day it is paid | `probes/time/TIME-03.md` | A-01, A-03, B-01 | |
| TIME-04 | Periods each law declares, split by facts | `probes/time/TIME-04.md` | A-04, A-13, B-01, D-03 | |
| TIME-05 ★ | Clocks, windows and deadlines | `probes/time/TIME-05.md` | A-04, A-06, A-13, D-02, D-13, E1-03 | |
| TIME-06 | Running totals and repricing within a period | `probes/time/TIME-06.md` | A-03, B-01 | |
| TIME-07 | Order within one day | `probes/time/TIME-07.md` | A-09, B-08, B-10, B-13 | |

## PTY: people, relations, groups

| Edge | Name | Probe | Questions | Status |
|---|---|---|---|---|
| PTY-01 | People and their dated relations | `probes/pty/PTY-01.md` | B-04, B-11, D-01 | |
| PTY-02 | Groups the law computes on | `probes/pty/PTY-02.md` | A-01, A-13, B-07, D-01 | |
| PTY-03 | Succession down the family tree | `probes/pty/PTY-03.md` | D-01, D-02, E1-03 | |
| PTY-04 ★ | A law reads another party | `probes/pty/PTY-04.md` | A-01, A-13, D-03 | |
| PTY-05 | Joins across households | `probes/pty/PTY-05.md` | E1-01, E1-02, E1-03 | |

## OWN: places, ownership, holdings

| Edge | Name | Probe | Questions | Status |
|---|---|---|---|---|
| OWN-01 | Places, their kinds and lifetimes | `probes/own/OWN-01.md` | A-11, B-04, D-08, D-09 | |
| OWN-02 | Claims and money that isn't ours | `probes/own/OWN-02.md` | B-09, B-11, D-05, D-08, E1-02 | |
| OWN-03 | Owed but not yet paid | `probes/own/OWN-03.md` | A-11, B-11, D-06, D-08, E1-02 | |
| OWN-04 | Four ownership regimes | `probes/own/OWN-04.md` | A-05, A-11, B-04, B-11, D-01, E1-02 | |
| OWN-05 ★ | Events that move ownership | `probes/own/OWN-05.md` | A-06, D-01, E1-03 | |
| OWN-06 | Lots | `probes/own/OWN-06.md` | A-05, A-13, D-01, D-06, D-13 | |
| OWN-07 | Matching sales to lots | `probes/own/OWN-07.md` | A-05, A-13, D-07, D-13 | |
| OWN-08 | Kept-apart parts and their limits | `probes/own/OWN-08.md` | A-05, A-09 | |
| OWN-09 | Indexed bonds in monthly series | `probes/own/OWN-09.md` | D-07 | |

## FLOW: transfers and conservation

| Edge | Name | Probe | Questions | Status |
|---|---|---|---|---|
| FLOW-01 | Transfers and conservation | `probes/flow/FLOW-01.md` | A-09, E1-01 | |
| FLOW-02 | One event, several legs | `probes/flow/FLOW-02.md` | A-07, A-08, B-06, E1-01 | |
| FLOW-03 ★ | Shared costs and netting | `probes/flow/FLOW-03.md` | B-11, D-01, D-05 | |

## CUR: two currencies

| Edge | Name | Probe | Questions | Status |
|---|---|---|---|---|
| CUR-01 ★ | Which rate for each leg | `probes/cur/CUR-01.md`; pair `probes/pairs/CUR-01+RAND-01.md` | B-03, B-13 | |
| CUR-02 | Converted amounts are whole minor units | `probes/cur/CUR-02.md` | B-03, B-13 | |
| CUR-03 | Revaluation is not a transfer | `probes/cur/CUR-03.md` | B-03, B-13 | |

## VAL: figures, sources, versions, prices

| Edge | Name | Probe | Questions | Status |
|---|---|---|---|---|
| VAL-01 | Figures from the gazette | `probes/val/VAL-01.md` | D-06 | |
| VAL-02 | Which date a value is read at | `probes/val/VAL-02.md` | A-09, B-01, D-06, D-11 | |
| VAL-03 | Two dates on every figure | `probes/val/VAL-03.md` | A-02 | |
| VAL-04 ★ | A published figure that disagrees with its law | `probes/val/VAL-04.md` | A-05, D-03, D-08 | |
| VAL-05 | Prices come from records | `probes/val/VAL-05.md` | A-05, B-05 | |
| VAL-06 | Indexed values and histories | `probes/val/VAL-06.md` | A-12, D-03, D-11 | |
| VAL-07 | The assumption register | `probes/val/VAL-07.md` | D-11, D-13 | |

## LAW: law arithmetic and how laws fit

| Edge | Name | Probe | Questions | Status |
|---|---|---|---|---|
| LAW-01 | Edges: thresholds, brackets and tables | `probes/law/LAW-01.md` | A-01, B-01 | |
| LAW-02 | Cliffs and tapers | `probes/law/LAW-02.md` | A-01, B-01, B-06 | |
| LAW-03 ★ | Law versions by date | `probes/law/LAW-03.md` | A-01, A-02, D-03 | |
| LAW-04 | Retroactive law | `probes/law/LAW-04.md` | A-01, D-03, D-04 | |
| LAW-05 | "As if" | `probes/law/LAW-05.md` | A-03, A-13 | |
| LAW-06 | Elections | `probes/law/LAW-06.md` | A-01, B-01 | |
| LAW-07 | Eligibility and classification from facts | `probes/law/LAW-07.md` | A-01, A-13, B-02, B-06, D-03 | |
| LAW-08 | Pipelines across laws and forms | `probes/law/LAW-08.md` | A-01, A-03, B-01, B-07 | |
| LAW-09 | Carry-forwards with caps | `probes/law/LAW-09.md` | A-05, D-13 | |
| LAW-10 | A result pending until a window closes | `probes/law/LAW-10.md` | B-04, D-09 | |
| LAW-11 | A law reads history | `probes/law/LAW-11.md` | A-06, B-01, E1-03 | |
| LAW-12 | An exception through a chain | `probes/law/LAW-12.md` | A-05, A-06, D-02, E1-03 | |
| LAW-13 | Future law | `probes/law/LAW-13.md` | A-12, B-12 | |
| LAW-20 | A circular rule the law solves by its own named method | `probes/law/LAW-20.md` | — | |

## LIFE: contracts, loans, things with a lifetime

| Edge | Name | Probe | Questions | Status |
|---|---|---|---|---|
| LIFE-01 | A thing written once, with a lifetime | `probes/life/LIFE-01.md` | A-12, D-10 | |
| LIFE-02 ★ | When a thing ends | `probes/life/LIFE-02.md` | A-07, A-08, B-04, B-06 | |
| LIFE-03 | Loans | `probes/life/LIFE-03.md` | A-07, B-05, D-01 | |
| LIFE-04 | Benefits recalculated | `probes/life/LIFE-04.md` | A-01, A-04, D-03, D-04 | |
| LIFE-05 | A sole trader's month and year | `probes/life/LIFE-05.md` | B-01 | |

## RULE: the household's own rules

| Edge | Name | Probe | Questions | Status |
|---|---|---|---|---|
| RULE-01 | A rule on the real balance on a date | `probes/rule/RULE-01.md` | A-09, B-10, B-13 | |
| RULE-02 | What a rule may see | `probes/rule/RULE-02.md` | A-09, B-10, D-07 | |
| RULE-03 ★ | Rules competing for the same money | `probes/rule/RULE-03.md` | A-10, B-10 | |
| RULE-04 | A loop | `probes/rule/RULE-04.md` | B-10 | |
| RULE-05 | "Once, when…" on a state | `probes/rule/RULE-05.md` | A-12, D-07 | |
| RULE-06 | A rule changed "as if since January" | `probes/rule/RULE-06.md` | B-10 | |
| RULE-07 | What a rule meant vs what the records show | `probes/rule/RULE-07.md` | A-09, B-11 | |
| RULE-08 | Parameters estimated from history | `probes/rule/RULE-08.md` | A-09 | |
| RULE-09 | A rule on a forecast | `probes/rule/RULE-09.md` | B-12 | |

## REC: records, replay, knowledge dates

| Edge | Name | Probe | Questions | Status |
|---|---|---|---|---|
| REC-01 | Records are the past; the model is the rest | `probes/rec/REC-01.md` | B-08 | |
| REC-02 ★ | A statement with errors | `probes/rec/REC-02.md` | B-08, D-05 | |
| REC-03 | Awkward rows | `probes/rec/REC-03.md` | A-09, B-08 | |
| REC-04 | Matching and certainty | `probes/rec/REC-04.md` | B-09, E1-01 | |
| REC-05 | Records as the law's inputs | `probes/rec/REC-05.md` | A-01, B-07 | |
| REC-06 | Clocks and knowledge dates | `probes/rec/REC-06.md` | A-02, B-07, E1-01 | |
| REC-07 | Replaying over recorded history | `probes/rec/REC-07.md` | B-10, D-12 | |

## SCEN: scenarios and amendments

| Edge | Name | Probe | Questions | Status |
|---|---|---|---|---|
| SCEN-01 | Scenarios and amendments | `probes/scen/SCEN-01.md` | A-13, B-05, B-13, D-13 | |
| SCEN-02 ★ | A counterfactual past | `probes/scen/SCEN-02.md` | A-08, A-13, B-05 | |
| SCEN-03 | Linked overrides | `probes/scen/SCEN-03.md` | B-13, D-11 | |
| SCEN-04 | Named extreme paths | `probes/scen/SCEN-04.md` | A-12 | |

## RAND: uncertainty over paths

| Edge | Name | Probe | Questions | Status |
|---|---|---|---|---|
| RAND-01 | Random families | `probes/rand/RAND-01.md`; pair `probes/pairs/CUR-01+RAND-01.md` | B-12, D-10, D-13 | |
| RAND-02 | Closed forms | `probes/rand/RAND-02.md` | B-12, D-10, D-13 | |
| RAND-03 ★ | Law on every path | `probes/rand/RAND-03.md` | B-12, D-10, D-13 | |
| RAND-04 | What a run may say | `probes/rand/RAND-04.md` | B-12, D-10, D-13 | |

## ASK: questions, holes, refusals

| Edge | Name | Probe | Questions | Status |
|---|---|---|---|---|
| ASK-01 | Question primitives | `probes/ask/ASK-01.md` | B-01 | |
| ASK-02 ★ | Net worth per person, with holes | `probes/ask/ASK-02.md` | A-11, B-11, D-08, E1-02 | |
| ASK-03 | Holes, refusals and warnings | `probes/ask/ASK-03.md` | A-04, A-10, A-11, B-07, B-08, B-09, B-10 | |
| ASK-04 | No advice | `probes/ask/ASK-04.md` | D-09 | |
| ASK-05 | Explain | `probes/ask/ASK-05.md` | A-01, A-13 | |

## SURF: how a model is written

| Edge | Name | Probe | Questions | Status |
|---|---|---|---|---|
| SURF-01 | No bare figures | `probes/surf/SURF-01.md` (a passage, no key) | — | |
| SURF-02 | Surface words | `probes/surf/SURF-02.md` (a passage, no key) | — | |
| SURF-03 | Draws written | `probes/surf/SURF-03.md` (a passage, no key) | B-12, D-10 | |
| SURF-04 | Stated once, checked before it runs | `probes/surf/SURF-04.md` (a passage, no key) | — | |
