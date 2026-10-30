# Holdout index

The holdout's own edges (`boundaries-hidden.md`), then every visible edge the holdout's questions use, with those questions (C, E2), from the keys' `edges:` lines. The status column is for the holdout's one-shot scoring (brief §8, §11).

Written by `tmp/cases/tools/index.py --hidden`; don't edit by hand.

## Hidden edges

| Edge | Name | Questions | Status |
|---|---|---|---|
| HID-01 | The treaty names the relief method | C-04 | |
| HID-02 | The foreign-tax credit and its limit | C-03 | |
| HID-03 | A foreign wage-tax algorithm | C-02 | |
| HID-04 | Two countries coordinate | C-05 | |
| HID-05 | House-rule shapes the visible set never shows | C-03, C-04, C-07, C-08, C-12 | |

## Visible edges the holdout uses

| Edge | Name | Probe | Questions | Status |
|---|---|---|---|---|
| ASK-02 ★ | Net worth per person, with holes | `probes/ask/ASK-02.md` | C-10 | |
| ASK-03 | Holes, refusals and warnings | `probes/ask/ASK-03.md` | C-10, C-12 | |
| CUR-01 ★ | Which rate for each leg | `probes/cur/CUR-01.md`; pair `probes/pairs/CUR-01+RAND-01.md` | C-03, C-05, C-06, C-08 | |
| CUR-02 | Converted amounts are whole minor units | `probes/cur/CUR-02.md` | C-03, C-05, C-06 | |
| CUR-03 | Revaluation is not a transfer | `probes/cur/CUR-03.md` | C-06, C-10 | |
| FLOW-01 | Transfers and conservation | `probes/flow/FLOW-01.md` | E2-01 | |
| FLOW-02 | One event, several legs | `probes/flow/FLOW-02.md` | E2-01 | |
| FLOW-03 ★ | Shared costs and netting | `probes/flow/FLOW-03.md` | C-07 | |
| LAW-02 | Cliffs and tapers | `probes/law/LAW-02.md` | C-04 | |
| LAW-07 | Eligibility and classification from facts | `probes/law/LAW-07.md` | C-02, C-09 | |
| LAW-08 | Pipelines across laws and forms | `probes/law/LAW-08.md` | C-01, C-03, C-04 | |
| LAW-10 | A result pending until a window closes | `probes/law/LAW-10.md` | C-03 | |
| LAW-11 | A law reads history | `probes/law/LAW-11.md` | C-09, C-11, E2-02 | |
| LAW-12 | An exception through a chain | `probes/law/LAW-12.md` | E2-02 | |
| LIFE-02 ★ | When a thing ends | `probes/life/LIFE-02.md` | C-01 | |
| LIFE-03 | Loans | `probes/life/LIFE-03.md` | C-11, C-12, E2-03 | |
| LIFE-04 | Benefits recalculated | `probes/life/LIFE-04.md` | C-05 | |
| LIFE-05 | A sole trader's month and year | `probes/life/LIFE-05.md` | C-01 | |
| NUM-01 | Amounts, quantities and rates carry their units | `probes/num/NUM-01.md` | C-10 | |
| NUM-02 | Day counts and pro-rating by days | `probes/num/NUM-02.md` | C-01 | |
| NUM-03 ★ | Rounding where and as the law rounds | `probes/num/NUM-03.md` | C-02 | |
| NUM-04 | An unclear convention, keyed under each reading | `probes/num/NUM-04.md` | C-08, C-11 | |
| OWN-01 | Places, their kinds and lifetimes | `probes/own/OWN-01.md` | C-10 | |
| OWN-02 | Claims and money that isn't ours | `probes/own/OWN-02.md` | C-09, C-10 | |
| OWN-03 | Owed but not yet paid | `probes/own/OWN-03.md` | C-01, C-05, C-07, C-10 | |
| OWN-04 | Four ownership regimes | `probes/own/OWN-04.md` | C-10, E2-03 | |
| OWN-05 ★ | Events that move ownership | `probes/own/OWN-05.md` | C-10, C-11, E2-02, E2-03 | |
| PTY-03 | Succession down the family tree | `probes/pty/PTY-03.md` | E2-02 | |
| PTY-04 ★ | A law reads another party | `probes/pty/PTY-04.md` | C-03, C-05, C-09, C-11, E2-02 | |
| PTY-05 | Joins across households | `probes/pty/PTY-05.md`; pair `probes/pairs/PTY-05+OWN-12.md` | E2-01, E2-02, E2-03 | |
| REC-01 | Records are the past; the model is the rest | `probes/rec/REC-01.md` | C-12 | |
| REC-04 | Matching and certainty | `probes/rec/REC-04.md` | C-08, E2-01 | |
| REC-05 | Records as the law's inputs | `probes/rec/REC-05.md` | C-02 | |
| REC-06 | Clocks and knowledge dates | `probes/rec/REC-06.md` | E2-01 | |
| RULE-01 | A rule on the real balance on a date | `probes/rule/RULE-01.md` | C-06, C-09 | |
| RULE-02 | What a rule may see | `probes/rule/RULE-02.md` | C-08 | |
| RULE-04 | A loop | `probes/rule/RULE-04.md` | C-12 | |
| RULE-05 | "Once, when…" on a state | `probes/rule/RULE-05.md` | C-06 | |
| RULE-06 | A rule changed "as if since January" | `probes/rule/RULE-06.md` | C-07 | |
| RULE-07 | What a rule meant vs what the records show | `probes/rule/RULE-07.md` | C-06 | |
| RULE-09 | A rule on a forecast | `probes/rule/RULE-09.md` | C-12 | |
| TIME-03 | The period an amount is for, and the day it is paid | `probes/time/TIME-03.md` | C-01, C-02, C-04, C-07 | |
| TIME-04 | Periods each law declares, split by facts | `probes/time/TIME-04.md` | C-02 | |
| TIME-05 ★ | Clocks, windows and deadlines | `probes/time/TIME-05.md` | C-05, C-08, C-09, C-11, E2-02 | |
