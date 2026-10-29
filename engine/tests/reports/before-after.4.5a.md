# Before and after: the checks on CPython (stored) and on PyPy (session 4.5a)

Made by `python3 -I -B tmp/tests/run.py --tables` from `results/pypy.tsv` (and `results/cpython.tsv` where the `both` mode ran) and `checks.tsv`. **Before:** the CPython seconds stored with the baselines (snap16's `summary.txt`, whole seconds; for household asks only the whole run's seconds are stored, from session 5's runs this morning, `run_cpython_s` in the note). **After:** PyPy seconds per check, budget 60 s, 4 jobs. **Equality:** `same` = byte for byte (whole output), or the ask's blocks in run5.py's form (households).

## Per layer

| group | checks | same | differ | over | known fault | not run | CPython s (stored, finished checks) | PyPy s (same checks) | PyPy s (all run) |
|---|---|---|---|---|---|---|---|---|---|
| interpreter | 1 | 1 | 0 | 0 | 0 | 0 | – | – | 15 |
| unit: probe sets | 9 | 9 | 0 | 0 | 0 | 0 | 9 | 10 | 11 |
| unit: probes | 103 | 103 | 0 | 0 | 0 | 0 | 101 | 89 | 89 |
| integration: household A | 42 | 40 | 0 | 0 | 0 | 2 | – | – | 298 |
| integration: household B | 69 | 44 | 0 | 3 | 0 | 22 | – | – | 1118 |
| integration: household D | 70 | 69 | 0 | 0 | 1 | 0 | – | – | 235 |
| integration: household E1 | 29 | 29 | 0 | 0 | 0 | 0 | – | – | 368 |
| integration: household X | 60 | 49 | 0 | 0 | 0 | 11 | – | – | 190 |
| integration: light and hardset | 14 | 13 | 0 | 1 | 0 | 0 | 420 | 118 | 179 |
| integration: old models | 77 | 77 | 0 | 0 | 0 | 0 | 131 | 126 | 126 |

## Household runs with stored CPython seconds

The run's CPython seconds (session 5's run this morning, on its copies of the inputs) against its asks run one by one on PyPy: the slowest ask is what the 60 s budget sees.

| run | asks | CPython s (whole run) | PyPy s (sum of asks) | PyPy s (slowest ask) | verdicts |
|---|---|---|---|---|---|
| B-cs_b09_b11 | 6 | 1301 | 235 | 47 | 6 same |
| E1-02-B | 6 | 597 | 189 | 49 | 6 same |
| A-q12 | 8 | 133 | 92 | 12 | 8 same |
| A-c05-ania | 1 | 44 | 13 | 13 | 1 same |
| A-c05-why | 1 | 39 | 13 | 13 | 1 same |
| A-c05-up | 1 | 22 | 10 | 10 | 1 same |
| A-c05-down | 1 | 21 | 10 | 10 | 1 same |
| A-q10-11-05a | 6 | 20 | 40 | 8 | 6 same |
| A-d10 | 2 | 12 | 13 | 7 | 2 same |
| A-q06-09 | 6 | 10 | 34 | 7 | 6 same |
| A-q01-04 | 12 | 8 | 61 | 6 | 12 same |
| A-d05 | 2 | 7 | 12 | 6 | 2 same |

E1-02-B's stored run asked 3 of the 6 asks its baseline printed and this table checks (`tx2_owes`, `tx2_worth`, `tx2_mtg_day`), so its CPython seconds cover only those.

## interpreter

| id | CPython s | PyPy s | verdict | note |
|---|---|---|---|---|
| `R:all` |  | 14.8 | same (cached) | cpython 12.96 s, same |

## unit: probe sets

| id | CPython s | PyPy s | verdict | note |
|---|---|---|---|---|
| `U:set/law/LAW-13/LAW-13` | 0 | 0.5 | same (cached) | cpython 0.23 s, same |
| `U:set/rand/RAND-04/RAND-04` | 9 | 6.6 | same (cached) | cpython 10.12 s, same |
| `U:set/rule/RULE-09/RULE-09` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:set/scen/SCEN-01/SCEN-01` | 0 | 0.7 | same (cached) | cpython 0.26 s, same |
| `U:set/scen/SCEN-02/SCEN-02` | 0 | 0.6 | same (cached) | cpython 0.23 s, same |
| `U:set/scen/SCEN-03/SCEN-03` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:set/scen/SCEN-04/SCEN-04` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:set/surf/SURF-04/SURF-04` | 0 | 0.6 | same (cached) | cpython 0.23 s, same |
| `U:set/pty/PTY-05/ania` |  | 0.5 | same (cached) | cpython 0.23 s, same |

## unit: probes

| id | CPython s | PyPy s | verdict | note |
|---|---|---|---|---|
| `U:ask/ASK-01` | 0 | 0.6 | same (cached) | cpython 0.24 s, same |
| `U:ask/ASK-02` | 0 | 0.5 | same (cached) | cpython 0.21 s, same |
| `U:ask/ASK-03` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:ask/ASK-04` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:ask/ASK-05` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:cur/CUR-01` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:cur/CUR-02` | 0 | 0.6 | same (cached) | cpython 0.22 s, same |
| `U:cur/CUR-03` | 0 | 0.6 | same (cached) | cpython 0.22 s, same |
| `U:flow/FLOW-01` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:flow/FLOW-02` | 0 | 0.5 | same (cached) | cpython 0.23 s, same |
| `U:flow/FLOW-03` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:law/LAW-01` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:law/LAW-02` | 0 | 0.5 | same (cached) | cpython 0.28 s, same |
| `U:law/LAW-03` | 0 | 0.5 | same (cached) | cpython 0.23 s, same |
| `U:law/LAW-04` | 0 | 0.6 | same (cached) | cpython 0.25 s, same |
| `U:law/LAW-05` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:law/LAW-06` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:law/LAW-07` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:law/LAW-08` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:law/LAW-09` | 0 | 0.5 | same (cached) | cpython 0.23 s, same |
| `U:law/LAW-10` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:law/LAW-11` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:law/LAW-12` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:law/LAW-13/LAW-13` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:law/LAW-13/rate25` | 0 | 0.5 | same (cached) | cpython 0.21 s, same |
| `U:life/LIFE-01` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:life/LIFE-02` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:life/LIFE-03` | 0 | 0.5 | same (cached) | cpython 0.23 s, same |
| `U:life/LIFE-04` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:life/LIFE-05` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:num/NUM-01` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:num/NUM-02` | 0 | 0.5 | same (cached) | cpython 0.24 s, same |
| `U:num/NUM-03` | 0 | 0.5 | same (cached) | cpython 0.24 s, same |
| `U:num/NUM-04` | 0 | 0.5 | same (cached) | cpython 0.23 s, same |
| `U:num/NUM-05` | 0 | 0.5 | same (cached) | cpython 0.23 s, same |
| `U:num/NUM-06` | 0 | 0.5 | same (cached) | cpython 0.23 s, same |
| `U:num/NUM-07` | 0 | 0.5 | same (cached) | cpython 0.23 s, same |
| `U:own/OWN-01` | 0 | 0.6 | same (cached) | cpython 0.23 s, same |
| `U:own/OWN-02` | 0 | 0.6 | same (cached) | cpython 0.25 s, same |
| `U:own/OWN-03` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:own/OWN-04` | 0 | 0.6 | same (cached) | cpython 0.22 s, same |
| `U:own/OWN-05` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:own/OWN-06` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:own/OWN-07` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:own/OWN-08` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:own/OWN-09` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:pty/PTY-01` | 0 | 0.6 | same (cached) | cpython 0.3 s, same |
| `U:pty/PTY-02` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:pty/PTY-03` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:pty/PTY-04` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:pty/PTY-05/ania` | 0 | 0.5 | same (cached) | cpython 0.23 s, same |
| `U:pty/PTY-05/halina` | 0 | 0.5 | same (cached) | cpython 0.23 s, same |
| `U:pty/PTY-05/joint` | 0 | 0.5 | same (cached) | cpython 0.25 s, same |
| `U:rand/RAND-01` | 35 | 13.7 | same (cached) | cpython 36.03 s, same |
| `U:rand/RAND-02` | 0 | 0.5 | same (cached) | cpython 0.23 s, same |
| `U:rand/RAND-03` | 39 | 14.6 | same (cached) | cpython 39.07 s, same |
| `U:rand/RAND-04/plan_b` | 0 | 0.5 | same (cached) | cpython 0.23 s, same |
| `U:rand/RAND-04/RAND-04` | 0 | 0.5 | same (cached) | cpython 0.24 s, same |
| `U:rec/REC-01` | 0 | 0.5 | same (cached) | cpython 0.24 s, same |
| `U:rec/REC-02` | 0 | 0.5 | same (cached) | cpython 0.3 s, same |
| `U:rec/REC-03` | 0 | 0.5 | same (cached) | cpython 0.32 s, same |
| `U:rec/REC-04` | 0 | 0.5 | same (cached) | cpython 0.24 s, same |
| `U:rec/REC-05` | 7 | 2.6 | same (cached) | cpython 7.2 s, same |
| `U:rec/REC-06` | 0 | 0.5 | same (cached) | cpython 0.25 s, same |
| `U:rec/REC-07` | 0 | 0.5 | same (cached) | cpython 0.29 s, same |
| `U:rule/RULE-01` | 0 | 0.5 | same (cached) | cpython 0.27 s, same |
| `U:rule/RULE-02` | 0 | 0.5 | same (cached) | cpython 0.27 s, same |
| `U:rule/RULE-03` | 0 | 0.5 | same (cached) | cpython 0.25 s, same |
| `U:rule/RULE-04` | 0 | 0.5 | same (cached) | cpython 0.23 s, same |
| `U:rule/RULE-05` | 0 | 0.7 | same (cached) | cpython 0.26 s, same |
| `U:rule/RULE-06` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:rule/RULE-07` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:rule/RULE-08` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:rule/RULE-09/inpath` | 0 | 0.5 | same (cached) | cpython 0.21 s, same |
| `U:rule/RULE-09/RULE-09` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:scen/SCEN-01/amend` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:scen/SCEN-01/SCEN-01` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:scen/SCEN-02/kept` | 0 | 0.5 | same (cached) | cpython 0.21 s, same |
| `U:scen/SCEN-02/SCEN-02` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:scen/SCEN-03/infl8` | 0 | 0.5 | same (cached) | cpython 0.21 s, same |
| `U:scen/SCEN-03/SCEN-03` | 0 | 0.5 | same (cached) | cpython 0.23 s, same |
| `U:scen/SCEN-04/crash` | 0 | 0.5 | same (cached) | cpython 0.26 s, same |
| `U:scen/SCEN-04/SCEN-04` | 0 | 0.5 | same (cached) | cpython 0.25 s, same |
| `U:surf/SURF-01` | 0 | 0.5 | same (cached) | cpython 0.26 s, same |
| `U:surf/SURF-02` | 0 | 0.5 | same (cached) | cpython 0.24 s, same |
| `U:surf/SURF-03` | 20 | 8.2 | same (cached) | cpython 20.77 s, same |
| `U:surf/SURF-04/redefine` | 0 | 0.5 | same (cached) | cpython 0.25 s, same |
| `U:surf/SURF-04/SURF-04` | 0 | 0.6 | same (cached) | cpython 0.27 s, same |
| `U:time/TIME-01` | 0 | 0.5 | same (cached) | cpython 0.23 s, same |
| `U:time/TIME-02` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:time/TIME-03` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:time/TIME-04` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:time/TIME-05` | 0 | 0.5 | same (cached) | cpython 0.23 s, same |
| `U:time/TIME-06` | 0 | 0.5 | same (cached) | cpython 0.23 s, same |
| `U:time/TIME-07.reverse` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:time/TIME-07` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:val/VAL-01` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:val/VAL-02` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:val/VAL-03` | 0 | 0.5 | same (cached) | cpython 0.21 s, same |
| `U:val/VAL-04` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:val/VAL-05` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:val/VAL-06` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |
| `U:val/VAL-07` | 0 | 0.5 | same (cached) | cpython 0.22 s, same |

## integration: household A

| id | CPython s | PyPy s | verdict | note |
|---|---|---|---|---|
| `I:A:a01_2021` | (run 8) | 4.9 | same |  |
| `I:A:a01_2022` | (run 8) | 4.9 | same |  |
| `I:A:a01_2022_causes` | (run 8) | 4.9 | same |  |
| `I:A:a01_2023` | (run 8) | 4.9 | same |  |
| `I:A:a01_without_2022` | (run 8) | 5.6 | same |  |
| `I:A:a01_without_2023` | (run 8) | 5.7 | same |  |
| `I:A:a02` | (run 8) | 5.6 | same |  |
| `I:A:a03` | (run 8) | 5.2 | same |  |
| `I:A:a03_advances` | (run 8) | 4.9 | same |  |
| `I:A:a03_without` | (run 8) | 5.0 | same |  |
| `I:A:a04_months` | (run 8) | 4.8 | same |  |
| `I:A:a04_total` | (run 8) | 4.8 | same |  |
| `I:A:a06` | (run 10) | 4.8 | same |  |
| `I:A:a07` | (run 10) | 4.8 | same |  |
| `I:A:a08` | (run 10) | 5.6 | same |  |
| `I:A:a08_steps` | (run 10) | 5.7 | same |  |
| `I:A:a09` | (run 10) | 6.6 | same |  |
| `I:A:a09_steps` | (run 10) | 6.6 | same |  |
| `I:A:a10` | (run 20) | 6.4 | same |  |
| `I:A:a11` | (run 20) | 7.5 | same |  |
| `I:A:a11_known_part` | (run 20) | 7.6 | same |  |
| `I:A:a05_tomek` | (run 20) | 6.3 | same |  |
| `I:A:a05_tomek_2024` | (run 20) | 6.3 | same |  |
| `I:A:a05_wrapped` | (run 20) | 5.5 | same |  |
| `I:A:a12_base` | (run 133) | 11.7 | same |  |
| `I:A:a12_t25_base` | (run 133) | 11.7 | same |  |
| `I:A:a12_crash` | (run 133) | 11.9 | same |  |
| `I:A:a12_t25_crash` | (run 133) | 11.5 | same |  |
| `I:A:a12_flat` | (run 133) | 11.2 | same |  |
| `I:A:a12_t25_flat` | (run 133) | 11.3 | same |  |
| `I:A:a12_switch` | (run 133) | 11.3 | same |  |
| `I:A:a12_t25_switch` | (run 133) | 11.6 | same |  |
| `I:A:mca05_bal` | (run 7) | 5.9 | same |  |
| `I:A:mca05_plain` | (run 7) | 6.3 | same |  |
| `I:A:mca_a10_bal` | (run 12) | 6.6 | same |  |
| `I:A:mca_a10_explain` | (run 12) | 6.5 | same |  |
| `I:A:a05_ania@c05-ania` | (run 44) | 12.8 | same |  |
| `I:A:a05_ania_why@c05-why` | (run 39) | 12.8 | same |  |
| `I:A:mca05_down` | (run 21) | 10.0 | same |  |
| `I:A:mca05_up` | (run 22) | 10.0 | same |  |
| `I:A:a05_ania@q05-ania` |  | – | not run | benchmarks.md row 9 |
| `I:A:a05_ania_why@q05-why` |  | – | not run | benchmarks.md row 9 |

## integration: household B

| id | CPython s | PyPy s | verdict | note |
|---|---|---|---|---|
| `I:B:o3_jan` | (run 906) | – | not run | benchmarks.md rows 5, 14 |
| `I:B:o3_feb` | (run 906) | – | not run | benchmarks.md rows 5, 14 |
| `I:B:b12_zero_tax` | (run 906) | – | not run | benchmarks.md rows 5, 14 |
| `I:B:b12_mortgage` | (run 906) | – | not run | benchmarks.md rows 5, 14 |
| `I:B:b12_mortgage_q07` | (run 906) | – | not run | benchmarks.md rows 5, 14 |
| `I:B:b12_liquid` |  | – | not run | benchmarks.md row 5 |
| `I:B:b12_share` | (run 852) | – | not run | benchmarks.md rows 5, 14 |
| `I:B:b12_share_without` | (run 852) | – | not run | benchmarks.md rows 5, 14 |
| `I:B:b12_shortfall` | (run 852) | – | not run | benchmarks.md rows 5, 14 |
| `I:B:b12_shortfall_without` | (run 852) | – | not run | benchmarks.md rows 5, 14 |
| `I:B:b12_tax` | (run 852) | – | not run | benchmarks.md rows 5, 14 |
| `I:B:b07_october@cd7full_b07` |  | – | not run | benchmarks.md row 4 |
| `I:B:b07_unique@cd7full_b07` |  | – | not run | benchmarks.md row 4 |
| `I:B:b07_return@cd7full_b07` |  | – | not run | benchmarks.md row 4 |
| `I:B:b07_jointly@cd7full_b07` |  | – | not run | benchmarks.md row 4 |
| `I:B:b07_wrzos_seen` |  | – | not run | benchmarks.md row 4 |
| `I:B:b07_wrzos_was@cd7full_b07` |  | – | not run | benchmarks.md row 4 |
| `I:B:b07_october@cd7full_b07a` |  | – | not run | benchmarks.md row 4 |
| `I:B:b07_unique@cd7full_b07a` |  | – | not run | benchmarks.md row 4 |
| `I:B:b07_return@cd7full_b07a` |  | – | not run | benchmarks.md row 4 |
| `I:B:b07_jointly@cd7full_b07a` |  | – | not run | benchmarks.md row 4 |
| `I:B:b07_wrzos_was@cd7full_b07was` |  | – | not run | benchmarks.md row 4 |
| `I:B:b03_lazur` |  | 45.0 | same |  |
| `I:B:b03_november` |  | 46.2 | same |  |
| `I:B:b08_balances` |  | 43.3 | same |  |
| `I:B:b08_residuals` |  | 43.9 | same |  |
| `I:B:b08_unmatched` |  | 44.5 | same |  |
| `I:B:b08_reversed` |  | 44.3 | same |  |
| `I:B:b08_gap` |  | 43.5 | same |  |
| `I:B:b08_no_row` |  | 45.6 | same |  |
| `I:B:b08_cards` |  | 45.7 | same |  |
| `I:B:b05_as_was` |  | > 60.0 | over | past 60 s |
| `I:B:b05_balance` |  | > 60.0 | over | past 60 s |
| `I:B:b05_instalment` |  | > 60.0 | over | past 60 s |
| `I:B:b05_last` |  | 3.1 | same |  |
| `I:B:b05_as_was_k07` |  | 5.2 | same |  |
| `I:B:b05_balance_k07` |  | 5.5 | same |  |
| `I:B:b05_instalment_k07` |  | 5.5 | same |  |
| `I:B:b09_invoices` | (run 1301) | 46.3 | same |  |
| `I:B:b09_florens` | (run 1301) | 46.4 | same |  |
| `I:B:b09_bartek` | (run 1301) | 46.0 | same |  |
| `I:B:b09_total` | (run 1301) | 46.1 | same |  |
| `I:B:b11_costs` | (run 1301) | 3.4 | same |  |
| `I:B:b11_worth` | (run 1301) | 46.6 | same |  |
| `I:B:b10_2025` |  | 42.1 | same |  |
| `I:B:b10_v2` |  | 42.8 | same |  |
| `I:B:b10_short` |  | 45.2 | same |  |
| `I:B:b11x_wrzos` |  | 45.2 | same |  |
| `I:B:b11x_ola_rest` |  | 5.1 | same |  |
| `I:B:b01_months` |  | 3.0 | same |  |
| `I:B:b01_settlement` |  | 3.0 | same |  |
| `I:B:b01_form_b` |  | 2.9 | same |  |
| `I:B:b01_form_c` |  | 2.9 | same |  |
| `I:B:b02_limit` |  | 2.8 | same |  |
| `I:B:b02_returns` |  | 2.8 | same |  |
| `I:B:b04_deed` |  | 2.7 | same |  |
| `I:B:b04_studio` |  | 2.6 | same |  |
| `I:B:b04_end_nothing` |  | 2.6 | same |  |
| `I:B:b04_end_kitchen` |  | 2.7 | same |  |
| `I:B:b04_not_seller` |  | 2.6 | same |  |
| `I:B:b04_rent` |  | 2.6 | same |  |
| `I:B:b04_deposit` |  | 2.6 | same |  |
| `I:B:b06_legs` |  | 2.8 | same |  |
| `I:B:b06_left` |  | 2.8 | same |  |
| `I:B:b06_purchase_vat` |  | 2.6 | same |  |
| `I:B:b06_health` |  | 2.9 | same |  |
| `I:B:b03_invoices` |  | 2.7 | same |  |
| `I:B:b03_sold` |  | 48.4 | same |  |
| `I:B:b10_v3` |  | 3.0 | same |  |

## integration: household D

| id | CPython s | PyPy s | verdict | note |
|---|---|---|---|---|
| `I:D:d01_deposit_row` |  | 2.7 | same |  |
| `I:D:d01_division` |  | 2.8 | same |  |
| `I:D:d01_flat` |  | 2.4 | same |  |
| `I:D:d01_paid` |  | 2.9 | same |  |
| `I:D:d01_payouts` |  | 2.9 | same |  |
| `I:D:d01_shares` |  | 2.3 | same |  |
| `I:D:d01_shares_whole` |  | 2.3 | same |  |
| `I:D:d02_dates` |  | 2.3 | same |  |
| `I:D:d02_group_life` |  | 2.3 | same |  |
| `I:D:d02_halina` |  | 2.3 | same |  |
| `I:D:d02_jakub_not_reported` |  | 4.0 | same |  |
| `I:D:d02_jakub_reported` |  | 2.3 | same |  |
| `I:D:d02_in_time` |  | 2.3 | same |  |
| `I:D:d03_arrears` |  | 2.5 | same |  |
| `I:D:d03_cap` |  | 2.3 | same |  |
| `I:D:d03_months` |  | 2.5 | same |  |
| `I:D:d03_owed` |  | 2.5 | same |  |
| `I:D:d03_own` |  | 2.3 | same |  |
| `I:D:d03_without` |  | 2.4 | same |  |
| `I:D:d03_without_owed` |  | 2.3 | same |  |
| `I:D:d03_zb` |  | 2.3 | same |  |
| `I:D:d03_gap` |  | 2.4 | same |  |
| `I:D:d04_check_advances` |  | 2.5 | same |  |
| `I:D:d04_check_paid` |  | 2.5 | same |  |
| `I:D:d04_check_payment` |  | 4.7 | same |  |
| `I:D:d04_check_rent_advance` |  | 4.2 | same |  |
| `I:D:d04_pensions` |  | 2.5 | same |  |
| `I:D:d04_rent` |  | 4.3 | same |  |
| `I:D:d04_total` |  | 4.5 | same |  |
| `I:D:d04_without` |  | 4.5 | same |  |
| `I:D:d05_duplicate` |  | 5.6 | same |  |
| `I:D:d05_missing_fee` |  | 5.5 | same |  |
| `I:D:d05_owed` |  | 3.5 | same |  |
| `I:D:d05_quarters` |  | 9.9 | same |  |
| `I:D:d05_quarters~alone` |  | 8.0 | known (cached) | blocks differ: d05_quarters (q = 2025-07) [hh_round = *]; d05_quarters (q = 2025-10) [hh_round = *]; d05_quarters (q = 2026-01) [hh_round = *]; d05_quarters (q = 2026-04) [hh_round = *]; d05_quarters (q = 2026-07) [hh_round = *] |
| `I:D:d06_all_redeemed` |  | 2.7 | same |  |
| `I:D:d06_end_2025` |  | 2.7 | same |  |
| `I:D:d06_end_2026` |  | 2.7 | same |  |
| `I:D:d06_end_2026_known` |  | 2.6 | same |  |
| `I:D:d07_check_credits` |  | 3.5 | same |  |
| `I:D:d07_check_hr2` |  | 3.7 | same |  |
| `I:D:d07_check_net` |  | 2.9 | same |  |
| `I:D:d07_check_tax` |  | 2.9 | same |  |
| `I:D:d07_left` |  | 2.4 | same |  |
| `I:D:d07_low_balances` |  | 6.9 | same |  |
| `I:D:d07_lowest` |  | 6.8 | same |  |
| `I:D:d07_redemptions` |  | 2.9 | same |  |
| `I:D:d07_series` |  | 2.2 | same |  |
| `I:D:d07_one_at_a_time` |  | 2.2 | same |  |
| `I:D:d08_net_worth` |  | 4.2 | same |  |
| `I:D:d08_parts` |  | 4.1 | same |  |
| `I:D:d09_advice` |  | 2.2 | same |  |
| `I:D:d09_all_spent` |  | 2.2 | same |  |
| `I:D:d09_keep` |  | 2.2 | same |  |
| `I:D:d09_nothing_spent` |  | 2.2 | same |  |
| `I:D:d09_sell` |  | 2.2 | same |  |
| `I:D:d09_parts` |  | 2.2 | same |  |
| `I:D:d10_bands` |  | 8.7 | same |  |
| `I:D:d10_closed` |  | 3.0 | same |  |
| `I:D:d10_stream` |  | 2.2 | same |  |
| `I:D:d10_paths` |  | 9.6 | same |  |
| `I:D:d11_follows` |  | 2.2 | same |  |
| `I:D:d11_follows_bonds` |  | 2.2 | same |  |
| `I:D:d11_isb` |  | 2.3 | same |  |
| `I:D:d11_pairs` |  | 3.5 | same |  |
| `I:D:d11_register` |  | 2.2 | same |  |
| `I:D:d11_not_follows_life` |  | 2.2 | same |  |
| `I:D:d11_not_follows_fund` |  | 2.2 | same |  |
| `I:D:d12_extremes` |  | 4.0 | same |  |
| `I:D:d12_starts` |  | 4.0 | same |  |

## integration: household E1

| id | CPython s | PyPy s | verdict | note |
|---|---|---|---|---|
| `I:E1:e1_01_anna_shown` |  | 9.0 | same |  |
| `I:E1:e1_01_anna_stated` |  | 9.0 | same |  |
| `I:E1:e1_01_deposit` |  | 8.8 | same |  |
| `I:E1:e1_01_in_is_out` |  | 8.9 | same |  |
| `I:E1:e1_01_payouts` |  | 9.1 | same |  |
| `I:E1:e1_01_payouts_to` |  | 5.1 | same |  |
| `I:E1:e1_01_piotr_shown` |  | 9.0 | same |  |
| `I:E1:e1_01_piotr_stated` |  | 9.1 | same |  |
| `I:E1:e1_03_anna_from_halina` |  | 4.9 | same |  |
| `I:E1:e1_03_anna_from_halina_sum` |  | 4.9 | same |  |
| `I:E1:e1_03_dates` |  | 4.8 | same |  |
| `I:E1:e1_03_gifts_in_estate` |  | 4.9 | same |  |
| `I:E1:e1_03_givers` |  | 4.9 | same |  |
| `I:E1:e1_03_halina` |  | 4.9 | same |  |
| `I:E1:e1_03_heirs` |  | 8.6 | same |  |
| `I:E1:e1_03_reported` |  | 4.9 | same |  |
| `I:E1:e1_03_not_reported` |  | 8.6 | same |  |
| `I:E1:e1_02_day` |  | 4.9 | same |  |
| `I:E1:e1_02_owed` |  | 5.9 | same |  |
| `I:E1:e1_02_people` |  | 10.9 | same |  |
| `I:E1:e1_02_family` |  | 11.1 | same |  |
| `I:E1:e1_02_people_known` |  | 13.5 | same |  |
| `I:E1:e1_02_family_known` |  | 14.0 | same |  |
| `I:E1:tx2_owes` | (run 597) | 2.2 | same |  |
| `I:E1:tx2_owes_explain` | (run 597) | 2.2 | same |  |
| `I:E1:tx2_worth` | (run 597) | 49.3 | same |  |
| `I:E1:tx2_cis_explain` | (run 597) | 47.9 | same |  |
| `I:E1:tx2_mtg_day` | (run 597) | 44.0 | same |  |
| `I:E1:tx2_mtg_live` | (run 597) | 43.0 | same |  |

## integration: household X

| id | CPython s | PyPy s | verdict | note |
|---|---|---|---|---|
| `I:X:base_values` |  | – | not run | benchmarks.md row 14 |
| `I:X:check_tm_konto` |  | – | not run | benchmarks.md row 14 |
| `I:X:check_tm_net_worth` |  | – | not run | benchmarks.md row 14 |
| `I:X:check_tm_room` |  | – | not run | benchmarks.md row 14 |
| `I:X:check_tm_zero_dip` |  | – | not run | benchmarks.md row 14 |
| `I:X:konto_balance@b_r1` |  | 50.1 | same |  |
| `I:X:replay_september@b_r1` |  | 51.5 | same |  |
| `I:X:wife_net@c1_fast` |  | 0.9 | same |  |
| `I:X:social_and_health@c1_fast` |  | 0.9 | same |  |
| `I:X:pit37@c1_fast` |  | 1.2 | same |  |
| `I:X:wife_net@c2_fast` |  | 0.9 | same |  |
| `I:X:social_and_health@c2_fast` |  | 0.9 | same |  |
| `I:X:pit37@c2_fast` |  | 1.1 | same |  |
| `I:X:wife_net@c3_fast` |  | 0.9 | same |  |
| `I:X:social_and_health@c3_fast` |  | 0.9 | same |  |
| `I:X:pit37@c3_fast` |  | 1.1 | same |  |
| `I:X:wife_net@c4_fast` |  | 0.9 | same |  |
| `I:X:social_and_health@c4_fast` |  | 0.9 | same |  |
| `I:X:pit37@c4_fast` |  | 1.2 | same |  |
| `I:X:wife_net@c5_fast` |  | 0.9 | same |  |
| `I:X:social_and_health@c5_fast` |  | 1.0 | same |  |
| `I:X:pit37@c5_fast` |  | 1.4 | same |  |
| `I:X:wife_net@c6_fast` |  | 1.0 | same |  |
| `I:X:social_and_health@c6_fast` |  | 1.0 | same |  |
| `I:X:pit37@c6_fast` |  | 1.2 | same |  |
| `I:X:wife_net@c7_fast` |  | 1.1 | same |  |
| `I:X:social_and_health@c7_fast` |  | 1.1 | same |  |
| `I:X:pit37@c7_fast` |  | 1.4 | same |  |
| `I:X:wife_net@v3_fast` |  | 0.9 | same |  |
| `I:X:social_and_health@v3_fast` |  | 0.9 | same |  |
| `I:X:pit37@v3_fast` |  | 1.2 | same |  |
| `I:X:wife_net@v4_fast` |  | 0.9 | same |  |
| `I:X:social_and_health@v4_fast` |  | 0.9 | same |  |
| `I:X:pit37@v4_fast` |  | 1.2 | same |  |
| `I:X:wife_net@v5_fast` |  | 0.9 | same |  |
| `I:X:social_and_health@v5_fast` |  | 0.9 | same |  |
| `I:X:pit37@v5_fast` |  | 1.3 | same |  |
| `I:X:t@min_m13` |  | 0.5 | same |  |
| `I:X:t@min_m13b` |  | 0.5 | same |  |
| `I:X:t@min_F3w` |  | 0.7 | same |  |
| `I:X:t@min_F3w3` |  | 0.7 | same |  |
| `I:X:t@min_F4g` |  | 0.6 | same |  |
| `I:X:t@min_F4h` |  | 0.6 | same |  |
| `I:X:t@min_F7a` |  | 0.5 | same |  |
| `I:X:t@min_F3k` |  | 0.6 | same |  |
| `I:X:t@min_m10l` |  | 0.5 | same |  |
| `I:X:t@min_m12d` |  | 0.5 | same |  |
| `I:X:t@min_m12e` |  | 0.5 | same |  |
| `I:X:n@min_m2` |  | 0.5 | same |  |
| `I:X:n2@min_m2` |  | 0.5 | same |  |
| `I:X:n@min_m2b` |  | 0.5 | same |  |
| `I:X:n2@min_m2b` |  | 0.5 | same |  |
| `I:X:explain_room` |  | – | not run | benchmarks.md row 14 |
| `I:X:ike_left` |  | – | not run | benchmarks.md row 14 |
| `I:X:konto_balance@c1_r2` |  | – | not run | benchmarks.md row 14 |
| `I:X:net_worth` |  | – | not run | benchmarks.md row 14 |
| `I:X:replay_september@c1_r2` |  | – | not run | benchmarks.md row 14 |
| `I:X:taxes_paid` |  | – | not run | benchmarks.md row 14 |
| `I:X:q1` |  | 0.8 | same |  |
| `I:X:replay_september@b_r1s` |  | 47.9 | same |  |

## integration: light and hardset

| id | CPython s | PyPy s | verdict | note |
|---|---|---|---|---|
| `I:light/LH-15-light` | 121 | 21.7 | same |  |
| `I:light/LH-16-light` | 347 | > 60.0 | over | past 60 s |
| `I:light/LH-17-mc-light` | 144 | 46.4 | same |  |
| `I:hardset/modeller-b1/I-6/model` | 110 | 23.9 | same |  |
| `I:hardset/modeller-b1/I-7/model` | 4 | 4.0 | same |  |
| `I:hardset/modeller-b1/I-8/model` | 0 | 2.1 | same |  |
| `I:hardset/modeller-b1/I-9/model` | 0 | 1.2 | same |  |
| `I:hardset/modeller-b1/I-10/model` | 0 | 0.9 | same |  |
| `I:hardset/modeller-b2/I-11/budget` | 5 | 3.8 | same |  |
| `I:hardset/modeller-b2/I-12/division` | 0 | 0.9 | same |  |
| `I:hardset/modeller-b2/I-13/claim` | 28 | 6.4 | same |  |
| `I:hardset/modeller-b2/I-14/homes` | 1 | 1.5 | same |  |
| `I:hardset/modeller-b2/I-15/gold` | 7 | 5.6 | same |  |
| `I:hardset/b2fix/division` |  | 0.8 | same |  |

## integration: old models

| id | CPython s | PyPy s | verdict | note |
|---|---|---|---|---|
| `I:M:CB-01` | 0 | 0.5 | same |  |
| `I:M:CB-02` | 0 | 0.6 | same |  |
| `I:M:CB-03` | 0 | 0.7 | same |  |
| `I:M:CB-04` | 0 | 0.6 | same |  |
| `I:M:CB-05` | 0 | 1.1 | same |  |
| `I:M:CB-06` | 0 | 0.6 | same |  |
| `I:M:CB-07` | 0 | 0.6 | same |  |
| `I:M:CB-08` | 0 | 0.5 | same |  |
| `I:M:CB-09` | 0 | 0.6 | same |  |
| `I:M:CB-10` | 0 | 0.6 | same |  |
| `I:M:CB-11` | 0 | 0.6 | same |  |
| `I:M:CB-12` | 0 | 0.6 | same |  |
| `I:M:CB-13` | 0 | 0.7 | same |  |
| `I:M:CB-14` | 0 | 0.9 | same |  |
| `I:M:CH-01` | 0 | 0.9 | same |  |
| `I:M:CH-02` | 0 | 1.0 | same |  |
| `I:M:CH-03` | 21 | 12.8 | same |  |
| `I:M:CH-04` | 0 | 0.5 | same |  |
| `I:M:CH-05` | 0 | 0.8 | same |  |
| `I:M:CH-06` | 0 | 0.8 | same |  |
| `I:M:CMP-01` | 2 | 2.5 | same |  |
| `I:M:CMP-02` | 0 | 0.6 | same |  |
| `I:M:LH-01` | 0 | 1.1 | same |  |
| `I:M:LH-02` | 0 | 0.6 | same |  |
| `I:M:LH-03` | 0 | 0.5 | same |  |
| `I:M:LH-04` | 1 | 1.9 | same |  |
| `I:M:LH-05` | 0 | 0.5 | same |  |
| `I:M:LH-06` | 0 | 0.9 | same |  |
| `I:M:LH-07` | 0 | 0.5 | same |  |
| `I:M:LH-08` | 0 | 0.5 | same |  |
| `I:M:LH-09` | 0 | 0.6 | same |  |
| `I:M:LH-10` | 96 | 53.2 | same |  |
| `I:M:LH-11` | 0 | 0.9 | same |  |
| `I:M:LH-12` | 0 | 0.6 | same |  |
| `I:M:LH-13` | 8 | 2.8 | same |  |
| `I:M:LH-14` | 0 | 1.1 | same |  |
| `I:M:LH-17` | 1 | 2.4 | same |  |
| `I:M:LH-18` | 2 | 2.3 | same |  |
| `I:M:MP-01` | 0 | 0.5 | same |  |
| `I:M:MP-02` | 0 | 0.5 | same |  |
| `I:M:MP-03` | 0 | 0.6 | same |  |
| `I:M:MP-04` | 0 | 0.6 | same |  |
| `I:M:MP-05` | 0 | 0.6 | same |  |
| `I:M:MP-06` | 0 | 0.8 | same |  |
| `I:M:MP-07` | 0 | 1.0 | same |  |
| `I:M:MP-08` | 0 | 0.9 | same |  |
| `I:M:MP-09` | 0 | 0.6 | same |  |
| `I:M:MP-10` | 0 | 0.6 | same |  |
| `I:M:MP-11` | 0 | 0.6 | same |  |
| `I:M:MP-12` | 0 | 0.6 | same |  |
| `I:M:MP-13` | 0 | 0.7 | same |  |
| `I:M:MP-14` | 0 | 0.6 | same |  |
| `I:M:MP-15` | 0 | 0.9 | same |  |
| `I:M:MP-16` | 0 | 0.5 | same |  |
| `I:M:RC-01` | 0 | 0.7 | same |  |
| `I:M:RC-02` | 0 | 0.7 | same |  |
| `I:M:RC-03` | 0 | 0.5 | same |  |
| `I:M:RC-04` | 0 | 1.2 | same |  |
| `I:M:RC-05` | 0 | 0.6 | same |  |
| `I:M:RC-06` | 0 | 0.7 | same |  |
| `I:M:RC-07` | 0 | 0.6 | same |  |
| `I:M:RC-08` | 0 | 1.0 | same |  |
| `I:M:UL-01` | 0 | 0.6 | same |  |
| `I:M:UL-02` | 0 | 0.5 | same |  |
| `I:M:UL-03` | 0 | 0.7 | same |  |
| `I:M:UL-04` | 0 | 0.6 | same |  |
| `I:M:UL-05` | 0 | 0.7 | same |  |
| `I:M:UL-06` | 0 | 0.6 | same |  |
| `I:M:UL-07` | 0 | 0.6 | same |  |
| `I:M:UL-08` | 0 | 0.8 | same |  |
| `I:M:UL-09` | 0 | 1.5 | same |  |
| `I:M:UL-10` | 0 | 0.5 | same |  |
| `I:M:UL-11` | 0 | 0.5 | same |  |
| `I:M:UL-12` | 0 | 0.5 | same |  |
| `I:M:UL-13` | 0 | 0.6 | same |  |
| `I:M:UL-14` | 0 | 0.8 | same |  |
| `I:M:UL-15` | 0 | 1.1 | same |  |
