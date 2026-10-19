# Tally: coverage, constructs × cases

The minimality check (brief §4.4): every construct is used by at least one case, or it goes, or it is justified in writing. Written in D7, 2026-10-19, after the D7 fixes.

**How it was counted.**
- Scratchpad scripts, counting only: `d7/coverage_scan2.py` (table 1: constructs by section) and `d7/cov_tokens.py` (table 2: each built-in name, mode, day count, distribution, type word and place kind). `d7/gen_coverage.py` formats this file.
- Columns: the 79 `models/*.tally`; `household`, the 13 files of `amendments/household/` as one; `chg1`–`chg7`, the added lines of each change in `amendments/changes/`.
- Strings are blanked and comments dropped, so a word in a note doesn't count. A regular expression can miss a use: every zero row was searched again with grep over models/ and amendments/ before its decision.
- A construct used only by the household or a change counts as used: the brief makes the household and the seven changes part of the deliverable (§6, D6).

## Summary

| | Rows | Used by a case | Zero |
|---|---|---|---|
| Table 1, constructs by section | 173 | 154 | 19 |
| Table 2, built-in words | 113 | 88 | 25 |

The zero rows come down to 19 decisions (table 3):
- **Removed in D7 (4):** the `law D for [x, y]` pin, a binding over a range, the `half-down` mode, and `day` and `quarter` as type words. Each was used by no case, and no spec ID needs it.
- **Kept, with a reason (15):** each is named by a spec ID, the brief or a hindcast fixture. No case among the 79 exercises it, so it is untested here, and REPORT §7 lists it.
- **The largest of these is Monte Carlo (K13).** No model draws: the stochastic cases are worked as closed forms (`cdf`, `pdf`, `pmf`, `crps`) or declared bounds. Monte Carlo is in the design because the brief and the spec require it, and only hindcast fixtures H0, H6 and H7 test it (BUILD-ORDER M7).

## 1. Constructs by section

| Construct | LANGUAGE § | Cases | Used by |
|---|---|---|---|
| model header (on) | 5.1 | 80 | CB-01, CB-02, CB-03, CB-04, CB-05, CB-06, CB-07, CB-08, CB-09, CB-10, CB-11, CB-12, CB-13, CB-14, CH-01, CH-02, CH-03, CH-04, CH-05, CH-06, CMP-01, CMP-02, LH-01, LH-02, LH-03, LH-04, LH-05, LH-06, LH-07, LH-08, LH-09, LH-10, LH-11, LH-12, LH-13, LH-14, LH-15, LH-16, LH-17, LH-18, MP-01, MP-02, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-09, MP-10, MP-11, MP-12, MP-13, MP-14, MP-15, MP-16, RC-01, RC-02, RC-03, RC-04, RC-05, RC-06, RC-07, RC-08, UL-01, UL-02, UL-03, UL-04, UL-05, UL-06, UL-07, UL-08, UL-09, UL-10, UL-11, UL-12, UL-13, UL-14, UL-15, household |
| header start | 5.1 | 3 | CMP-01, RC-08, household |
| header end | 5.1 | 2 | CMP-01, household |
| law part header | 6.1 | 2 | household, chg5 |
| part header | 6.1 | 5 | household, chg1, chg4, chg5, chg6 |
| scenario header (from) | 6.4 | 1 | chg7 |
| scenario over | 6.4 | 0 | — (K11) |
| source | 5.2 | 86 | CB-01, CB-02, CB-03, CB-04, CB-05, CB-06, CB-07, CB-08, CB-09, CB-10, CB-11, CB-12, CB-13, CB-14, CH-01, CH-02, CH-03, CH-04, CH-05, CH-06, CMP-01, CMP-02, LH-01, LH-02, LH-03, LH-04, LH-05, LH-06, LH-07, LH-08, LH-09, LH-10, LH-11, LH-12, LH-13, LH-14, LH-15, LH-16, LH-17, LH-18, MP-01, MP-02, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-09, MP-10, MP-11, MP-12, MP-13, MP-14, MP-15, MP-16, RC-01, RC-02, RC-03, RC-04, RC-05, RC-06, RC-07, RC-08, UL-01, UL-02, UL-03, UL-04, UL-05, UL-06, UL-07, UL-08, UL-09, UL-10, UL-11, UL-12, UL-13, UL-14, UL-15, household, chg1, chg2, chg3, chg4, chg5, chg6 |
| source at | 5.2 | 79 | CB-01, CB-02, CB-03, CB-04, CB-05, CB-06, CB-07, CB-08, CB-09, CB-10, CB-11, CB-12, CB-13, CB-14, CH-01, CH-02, CH-03, CH-04, CH-05, CH-06, CMP-01, CMP-02, LH-01, LH-02, LH-03, LH-04, LH-05, LH-06, LH-07, LH-08, LH-09, LH-10, LH-11, LH-12, LH-13, LH-14, LH-15, LH-16, LH-17, LH-18, MP-01, MP-02, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-09, MP-10, MP-11, MP-12, MP-13, MP-14, MP-15, MP-16, RC-01, RC-02, RC-03, RC-04, RC-05, RC-06, RC-07, RC-08, UL-01, UL-02, UL-03, UL-04, UL-05, UL-06, UL-07, UL-08, UL-09, UL-10, UL-11, UL-12, UL-13, UL-14, UL-15 |
| version | 5.3 | 87 | CB-01, CB-02, CB-03, CB-04, CB-05, CB-06, CB-07, CB-08, CB-09, CB-10, CB-11, CB-12, CB-13, CB-14, CH-01, CH-02, CH-03, CH-04, CH-05, CH-06, CMP-01, CMP-02, LH-01, LH-02, LH-03, LH-04, LH-05, LH-06, LH-07, LH-08, LH-09, LH-10, LH-11, LH-12, LH-13, LH-14, LH-15, LH-16, LH-17, LH-18, MP-01, MP-02, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-09, MP-10, MP-11, MP-12, MP-13, MP-14, MP-15, MP-16, RC-01, RC-02, RC-03, RC-04, RC-05, RC-06, RC-07, RC-08, UL-01, UL-02, UL-03, UL-04, UL-05, UL-06, UL-07, UL-08, UL-09, UL-10, UL-11, UL-12, UL-13, UL-14, UL-15, household, chg1, chg2, chg3, chg4, chg5, chg6, chg7 |
| version governs | 5.3 | 51 | CB-02, CB-03, CB-06, CB-07, CB-08, CB-09, CB-10, CB-13, CB-14, CH-01, CH-05, CMP-01, LH-04, LH-07, LH-12, LH-14, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-09, MP-10, MP-11, MP-13, MP-16, RC-01, RC-02, RC-04, RC-05, RC-06, RC-07, RC-08, UL-01, UL-02, UL-03, UL-04, UL-05, UL-06, UL-08, UL-09, UL-10, UL-11, UL-12, UL-13, UL-14, UL-15, household, chg2, chg3 |
| enters | 5.3 | 3 | RC-08, UL-04, UL-15 |
| moves | 5.3 | 2 | RC-01, RC-05 |
| arrears on | 5.13.3 | 2 | LH-14, chg4 |
| arrears none | 5.13.3 | 2 | household, chg3 |
| arrears of | 5.13.3 | 1 | LH-14 |
| arrears for | 5.13.3 | 3 | household, chg3, chg4 |
| correction | 4.5 | 1 | CH-04 |
| version assumed | 5.3 | 1 | MP-16 |
| version covers | 5.18 | 1 | household |
| version with no items | 5.3 | 47 | CB-02, CB-03, CB-04, CB-05, CB-07, CB-08, CB-09, CB-10, CB-11, CB-12, CB-13, CB-14, CH-06, CMP-01, CMP-02, LH-01, LH-02, LH-03, LH-07, LH-08, LH-09, LH-10, LH-11, LH-12, LH-17, MP-01, MP-02, MP-04, MP-05, MP-06, MP-07, MP-08, MP-09, MP-11, MP-12, MP-13, MP-14, MP-15, MP-16, RC-01, RC-03, RC-05, RC-07, RC-08, UL-10, UL-15, household |
| index addition | 5.3 | 4 | chg1, chg4, chg5, chg6 |
| row governs | 4.6 | 17 | CB-03, CH-05, CMP-01, LH-05, LH-07, MP-09, MP-11, RC-03, RC-08, UL-05, UL-06, UL-08, UL-13, household, chg1, chg4, chg5 |
| row keys * or range | 4.6 | 13 | CB-14, LH-04, LH-06, LH-10, LH-11, LH-13, LH-14, MP-15, RC-01, RC-02, RC-06, RC-08, UL-05 |
| given (row on a def) | 5.10 | 14 | CB-10, CB-11, CB-12, CB-14, LH-13, LH-17, MP-03, MP-04, MP-06, MP-07, MP-08, RC-01, RC-02, UL-13 |
| unit | 5.4 | 7 | CB-06, CH-02, CH-04, CH-06, LH-01, LH-02, household |
| axis | 5.5 | 62 | CB-02, CB-03, CB-05, CB-06, CB-07, CB-10, CB-11, CB-12, CB-14, CH-01, CH-02, CH-03, CH-04, CH-05, CH-06, CMP-01, CMP-02, LH-02, LH-03, LH-04, LH-07, LH-08, LH-09, LH-10, LH-11, LH-12, LH-13, LH-14, LH-15, LH-16, LH-17, LH-18, MP-01, MP-02, MP-03, MP-04, MP-05, MP-06, MP-07, MP-09, MP-11, MP-12, MP-13, MP-14, MP-16, RC-01, RC-02, RC-04, RC-05, RC-08, UL-01, UL-02, UL-03, UL-05, UL-07, UL-08, UL-09, UL-10, UL-11, UL-12, UL-14, UL-15 |
| axis option pin | 5.5 | 1 | LH-15 |
| index list | 5.6 | 77 | CB-01, CB-02, CB-03, CB-04, CB-05, CB-06, CB-07, CB-08, CB-09, CB-10, CB-11, CB-12, CB-13, CB-14, CH-01, CH-02, CH-03, CH-04, CH-05, CH-06, CMP-01, CMP-02, LH-01, LH-02, LH-03, LH-04, LH-05, LH-06, LH-07, LH-08, LH-09, LH-10, LH-11, LH-12, LH-13, LH-14, LH-15, LH-16, LH-17, LH-18, MP-01, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-09, MP-10, MP-11, MP-12, MP-13, MP-14, MP-15, MP-16, RC-01, RC-02, RC-03, RC-04, RC-06, RC-07, RC-08, UL-01, UL-02, UL-03, UL-04, UL-05, UL-06, UL-07, UL-08, UL-09, UL-10, UL-12, UL-13, UL-14, UL-15, household |
| index range (int) | 5.6 | 22 | CH-01, CH-02, CH-03, CH-05, CH-06, LH-02, LH-07, LH-08, LH-10, LH-11, LH-12, LH-13, LH-14, LH-15, LH-16, LH-18, MP-15, MP-16, RC-01, UL-07, UL-09, household |
| index range (year/month/day) | 4.1 | 34 | CB-02, CB-03, CB-05, CB-07, CB-08, CB-09, CB-10, CB-13, CB-14, LH-04, LH-05, LH-07, LH-08, LH-09, LH-14, LH-15, LH-16, MP-04, MP-07, MP-08, RC-04, RC-08, UL-01, UL-02, UL-04, UL-05, UL-06, UL-08, UL-10, UL-12, UL-13, UL-14, UL-15, household |
| index by n | 4.1 | 2 | LH-15, UL-06 |
| index from (anchor) | 4.1 | 7 | CB-03, CB-10, MP-07, RC-04, UL-02, UL-04, UL-06 |
| index open | 5.6 | 1 | household |
| index v | 5.6 | 6 | CB-04, CB-05, CB-09, CB-14, LH-09, LH-18 |
| index assumed | 5.6 | 1 | CB-14 |
| input | 5.7 | 81 | CB-01, CB-02, CB-03, CB-04, CB-05, CB-06, CB-07, CB-08, CB-09, CB-10, CB-11, CB-12, CB-13, CB-14, CH-01, CH-02, CH-03, CH-04, CH-05, CH-06, CMP-01, CMP-02, LH-01, LH-02, LH-03, LH-04, LH-05, LH-06, LH-07, LH-08, LH-09, LH-10, LH-11, LH-12, LH-13, LH-14, LH-15, LH-16, LH-17, LH-18, MP-01, MP-02, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-09, MP-10, MP-11, MP-12, MP-13, MP-14, MP-15, MP-16, RC-01, RC-02, RC-03, RC-04, RC-05, RC-06, RC-07, RC-08, UL-01, UL-02, UL-03, UL-04, UL-05, UL-06, UL-07, UL-08, UL-09, UL-10, UL-11, UL-12, UL-13, UL-14, UL-15, household, chg5 |
| input question | 5.7 | 50 | CB-02, CB-03, CB-05, CB-07, CB-09, CB-11, CH-02, CH-04, CH-05, CMP-01, CMP-02, LH-07, LH-08, LH-11, LH-13, LH-14, LH-15, LH-16, LH-17, MP-09, MP-10, MP-11, MP-12, MP-13, MP-14, MP-16, RC-01, RC-02, RC-03, RC-04, RC-05, RC-06, RC-07, RC-08, UL-01, UL-02, UL-03, UL-04, UL-05, UL-06, UL-08, UL-09, UL-10, UL-11, UL-12, UL-13, UL-14, UL-15, household, chg5 |
| prices | 3.4 | 2 | CH-05, household |
| prices by | 3.4 | 1 | household |
| assumed row | 4.5 | 53 | CB-02, CB-03, CB-05, CB-06, CB-07, CB-09, CB-11, CB-12, CB-13, CB-14, CH-03, CH-06, CMP-01, CMP-02, LH-01, LH-02, LH-03, LH-04, LH-07, LH-08, LH-09, LH-10, LH-12, LH-13, LH-14, LH-16, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-09, MP-13, MP-14, MP-16, RC-01, RC-02, RC-03, RC-05, RC-06, RC-07, RC-08, UL-04, UL-09, UL-10, UL-11, UL-12, UL-13, UL-14, UL-15, household, chg5 |
| hole row or rule (? "") | 8.1 | 51 | CB-01, CB-02, CB-03, CB-06, CB-07, CB-08, CB-10, CB-11, CB-12, CB-13, CB-14, CH-02, CH-04, CH-05, CMP-01, CMP-02, LH-01, LH-02, LH-04, LH-05, LH-10, LH-14, LH-15, MP-01, MP-02, MP-03, MP-04, MP-05, MP-08, MP-09, MP-10, MP-13, MP-14, MP-16, RC-02, RC-03, RC-06, RC-07, RC-08, UL-03, UL-05, UL-07, UL-08, UL-09, UL-10, UL-11, UL-12, UL-13, UL-15, household, chg5 |
| - value | 3.1 | 19 | CB-07, CMP-02, LH-01, LH-02, LH-04, LH-06, LH-10, MP-03, MP-05, MP-14, RC-04, UL-05, UL-06, UL-07, UL-09, UL-14, UL-15, household, chg4 |
| - tested (= - / != -) | 3.1 | 16 | CB-07, CMP-02, LH-01, LH-04, LH-06, LH-10, MP-03, MP-05, MP-14, RC-04, UL-05, UL-06, UL-07, UL-14, UL-15, household |
| record value | 3.1 | 7 | LH-10, LH-13, LH-14, MP-13, RC-02, household, chg6 |
| list value | 4.2 | 1 | household |
| type money | 3.1 | 71 | CB-01, CB-02, CB-03, CB-04, CB-05, CB-07, CB-08, CB-09, CB-10, CB-11, CB-12, CB-13, CB-14, CH-02, CH-05, CMP-01, CMP-02, LH-01, LH-02, LH-03, LH-04, LH-05, LH-06, LH-07, LH-08, LH-09, LH-10, LH-11, LH-12, LH-13, LH-14, MP-01, MP-02, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-09, MP-10, MP-11, MP-12, MP-13, MP-14, MP-16, RC-01, RC-02, RC-03, RC-04, RC-05, RC-06, RC-07, RC-08, UL-01, UL-02, UL-03, UL-04, UL-05, UL-06, UL-07, UL-08, UL-09, UL-10, UL-11, UL-12, UL-13, UL-14, UL-15, household, chg5 |
| type money XXX | 3.1 | 3 | CB-01, CB-09, LH-11 |
| type num with unit | 3.2 | 63 | CB-01, CB-02, CB-03, CB-04, CB-05, CB-06, CB-07, CB-08, CB-09, CB-10, CB-11, CB-13, CB-14, CH-02, CH-04, CH-06, CMP-01, LH-01, LH-02, LH-04, LH-05, LH-07, LH-08, LH-10, LH-11, LH-12, LH-13, LH-14, LH-15, LH-16, LH-17, MP-01, MP-02, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-11, MP-12, MP-13, RC-03, RC-04, RC-05, RC-06, RC-08, UL-01, UL-02, UL-03, UL-04, UL-05, UL-07, UL-08, UL-09, UL-10, UL-11, UL-12, UL-13, UL-14, UL-15, household, chg5 |
| type int | 3.1 | 49 | CB-04, CB-06, CB-07, CB-11, CB-14, CH-01, CH-02, CH-03, CH-05, CH-06, CMP-01, CMP-02, LH-01, LH-02, LH-04, LH-06, LH-07, LH-08, LH-09, LH-10, LH-11, LH-13, LH-14, LH-15, LH-16, LH-17, LH-18, MP-03, MP-04, MP-05, MP-06, MP-07, MP-09, MP-10, MP-11, MP-13, MP-14, MP-15, RC-02, RC-03, RC-04, RC-05, UL-04, UL-05, UL-07, UL-09, UL-13, UL-15, household |
| type bool | 3.1 | 53 | CB-02, CB-03, CB-05, CB-06, CB-07, CB-09, CB-10, CB-11, CB-12, CB-14, CMP-01, CMP-02, LH-01, LH-02, LH-03, LH-04, LH-06, LH-07, LH-08, LH-13, LH-14, LH-15, LH-16, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-09, MP-11, MP-13, MP-14, MP-15, MP-16, RC-01, RC-02, RC-03, RC-05, RC-06, RC-07, RC-08, UL-04, UL-05, UL-07, UL-08, UL-09, UL-10, UL-12, UL-13, UL-14, UL-15, household |
| type date | 3.1 | 22 | CB-07, CB-08, CB-14, CMP-01, LH-01, LH-02, LH-07, LH-09, LH-10, LH-11, LH-12, LH-14, MP-05, MP-06, MP-07, MP-08, MP-13, MP-16, UL-08, UL-09, UL-10, household |
| type dates | 3.1 | 1 | household |
| type month/year | 3.1 | 7 | CMP-01, LH-07, LH-09, LH-14, MP-11, UL-05, household |
| type key | 3.1 | 59 | CB-02, CB-03, CB-04, CB-05, CB-06, CB-07, CB-08, CB-09, CB-10, CB-11, CB-13, CB-14, CH-01, CH-02, CH-05, CMP-01, CMP-02, LH-01, LH-02, LH-03, LH-04, LH-07, LH-08, LH-09, LH-10, LH-12, LH-13, LH-14, LH-15, LH-16, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-09, MP-10, MP-13, MP-14, MP-15, MP-16, RC-02, RC-03, RC-06, RC-07, RC-08, UL-01, UL-02, UL-03, UL-04, UL-05, UL-07, UL-08, UL-09, UL-10, UL-13, UL-14, household |
| type record | 3.1 | 4 | LH-10, LH-13, LH-14, household |
| rate nominal | 3.3 | 2 | LH-10, LH-11 |
| rate effective | 3.3 | 2 | CH-05, LH-18 |
| rate simple | 3.3 | 0 | — (K2) |
| rate discount | 3.3 | 1 | LH-11 |
| rate /month or /day | 3.3 | 0 | — (K2) |
| band | 5.8 | 23 | CB-01, CB-02, CB-03, CB-04, CB-07, CH-05, CMP-01, MP-05, MP-06, MP-08, MP-09, MP-10, MP-11, MP-16, RC-07, RC-08, UL-01, UL-03, UL-07, UL-09, UL-10, UL-15, household |
| keyed band | 5.8 | 2 | MP-06, UL-03 |
| band row | 5.8 | 23 | CB-01, CB-02, CB-03, CB-04, CB-07, CH-05, CMP-01, MP-05, MP-06, MP-08, MP-09, MP-10, MP-11, MP-16, RC-07, RC-08, UL-01, UL-03, UL-07, UL-09, UL-10, UL-15, household |
| -inf / inf | 5.8 | 20 | CB-02, CB-03, CB-04, CB-07, CMP-01, MP-05, MP-06, MP-08, MP-09, MP-10, MP-11, RC-07, RC-08, UL-01, UL-03, UL-07, UL-09, UL-10, UL-15, household |
| band_lo/band_hi | 5.8 | 9 | CB-02, CB-04, CB-07, MP-06, MP-08, RC-08, UL-03, UL-10, UL-15 |
| marginal | 5.8 | 5 | CB-03, CB-04, CMP-01, MP-11, UL-09 |
| place account/cash | 5.9 | 3 | CMP-01, household, chg5 |
| place holds | 5.9 | 1 | household |
| place debt/claim | 5.9 | 1 | household |
| place outside | 5.9 | 2 | household, chg5 |
| place holding priced | 5.9 | 1 | household |
| place asset priced | 5.9 | 0 | — (K5) |
| kind()/currency() | 5.9 | 1 | household |
| opening (built-in input) | 5.9 | 2 | household, chg6 |
| def short form | 5.10 | 67 | CB-01, CB-02, CB-03, CB-04, CB-05, CB-06, CB-07, CB-08, CB-09, CB-10, CB-11, CB-12, CB-13, CB-14, CH-01, CH-02, CH-03, CH-04, CH-05, CH-06, CMP-01, CMP-02, LH-01, LH-02, LH-03, LH-04, LH-06, LH-07, LH-08, LH-09, LH-10, LH-11, LH-12, LH-13, LH-14, LH-15, LH-16, LH-17, MP-01, MP-02, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-09, MP-12, MP-14, MP-15, MP-16, RC-03, UL-01, UL-02, UL-03, UL-04, UL-05, UL-06, UL-07, UL-08, UL-09, UL-10, UL-12, UL-13, UL-14, UL-15, household |
| def with rules | 5.10 | 80 | CB-01, CB-02, CB-03, CB-04, CB-05, CB-06, CB-07, CB-08, CB-09, CB-10, CB-11, CB-12, CB-13, CB-14, CH-01, CH-02, CH-03, CH-05, CH-06, CMP-01, CMP-02, LH-01, LH-02, LH-03, LH-04, LH-05, LH-06, LH-07, LH-08, LH-09, LH-10, LH-11, LH-12, LH-13, LH-14, LH-15, LH-16, LH-17, LH-18, MP-01, MP-02, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-09, MP-10, MP-11, MP-12, MP-13, MP-14, MP-15, MP-16, RC-01, RC-02, RC-03, RC-04, RC-05, RC-06, RC-07, RC-08, UL-01, UL-02, UL-03, UL-04, UL-05, UL-06, UL-07, UL-08, UL-09, UL-10, UL-11, UL-12, UL-13, UL-14, UL-15, household, chg5 |
| def at | 4.3 | 13 | CB-08, CB-09, CB-10, CMP-01, LH-12, MP-05, MP-08, MP-11, MP-13, UL-04, UL-08, UL-09, household |
| rule when | 5.10 | 78 | CB-01, CB-02, CB-03, CB-05, CB-06, CB-07, CB-08, CB-09, CB-10, CB-11, CB-12, CB-13, CB-14, CH-01, CH-02, CH-03, CH-05, CH-06, CMP-01, CMP-02, LH-01, LH-02, LH-03, LH-04, LH-05, LH-06, LH-07, LH-08, LH-09, LH-10, LH-11, LH-12, LH-13, LH-14, LH-15, LH-16, LH-17, LH-18, MP-01, MP-02, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-09, MP-10, MP-11, MP-12, MP-13, MP-14, MP-15, MP-16, RC-01, RC-02, RC-03, RC-04, RC-05, RC-06, RC-07, RC-08, UL-01, UL-02, UL-03, UL-04, UL-05, UL-06, UL-07, UL-08, UL-09, UL-10, UL-11, UL-12, UL-13, UL-14, UL-15, household |
| rule excepts | 5.10 | 67 | CB-02, CB-03, CB-06, CB-07, CB-09, CB-11, CB-13, CB-14, CH-02, CH-03, CMP-01, CMP-02, LH-01, LH-02, LH-03, LH-04, LH-06, LH-07, LH-08, LH-09, LH-10, LH-11, LH-12, LH-13, LH-14, LH-15, LH-16, LH-17, LH-18, MP-01, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-09, MP-10, MP-11, MP-12, MP-13, MP-14, MP-16, RC-01, RC-02, RC-03, RC-04, RC-05, RC-06, RC-07, RC-08, UL-01, UL-02, UL-03, UL-04, UL-05, UL-06, UL-07, UL-08, UL-09, UL-10, UL-11, UL-12, UL-13, UL-14, UL-15, household |
| rule v | 5.10 | 56 | CB-02, CB-03, CB-04, CB-05, CB-06, CB-07, CB-08, CB-09, CB-10, CB-11, CB-12, CB-13, CB-14, CH-03, CH-06, CMP-01, CMP-02, LH-03, LH-07, LH-08, LH-09, LH-10, LH-11, LH-12, LH-17, MP-01, MP-02, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-09, MP-10, MP-11, MP-12, MP-13, MP-14, MP-15, MP-16, RC-01, RC-02, RC-03, RC-04, RC-05, RC-06, RC-07, RC-08, UL-02, UL-05, UL-09, UL-10, UL-11, UL-15, household |
| if then else | 5.10 | 15 | CB-06, CB-07, CH-05, CMP-01, LH-10, LH-13, MP-03, MP-04, MP-06, MP-11, MP-15, RC-02, UL-08, UL-13, household |
| and/or/not | 5.10 | 57 | CB-01, CB-03, CB-05, CB-06, CB-07, CB-09, CB-10, CB-11, CB-12, CB-14, CH-01, CH-02, CH-03, CH-06, CMP-01, CMP-02, LH-01, LH-02, LH-04, LH-06, LH-07, LH-09, LH-10, LH-13, LH-14, LH-15, LH-16, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-09, MP-12, MP-13, MP-14, MP-15, MP-16, RC-02, RC-04, RC-05, RC-06, RC-08, UL-02, UL-03, UL-04, UL-05, UL-06, UL-08, UL-09, UL-10, UL-12, UL-13, UL-14, UL-15, household |
| in (membership) | 5.10 | 25 | CB-06, CMP-01, LH-02, LH-04, LH-06, LH-10, LH-12, LH-13, LH-17, MP-03, MP-05, MP-10, RC-01, RC-02, RC-08, UL-01, UL-03, UL-05, UL-06, UL-09, UL-10, UL-13, UL-14, UL-15, household |
| record field .f | 3.1 | 13 | CB-05, CB-09, LH-01, LH-02, LH-04, LH-06, LH-09, LH-10, LH-13, LH-14, MP-15, UL-13, household |
| to UNIT | 7.3 | 11 | CH-01, CH-03, CH-06, LH-10, LH-11, LH-15, LH-16, LH-17, LH-18, MP-07, UL-08 |
| as if | 5.10 | 10 | CB-03, CB-09, CH-04, CMP-01, LH-04, LH-12, MP-11, RC-08, UL-14, UL-15 |
| pin value (ref = expr) | 5.10 | 19 | CB-03, CB-07, CB-09, CMP-01, LH-03, LH-04, LH-06, LH-12, MP-03, MP-04, MP-05, MP-07, MP-08, MP-10, MP-11, RC-08, UL-01, UL-03, UL-14 |
| pin law | 4.3 | 10 | CH-05, LH-12, MP-16, RC-03, RC-05, RC-06, RC-07, UL-11, UL-13, UL-15 |
| pin law for | 4.3 | 0 | — (R1) |
| pin wording | 4.3 | 1 | UL-15 |
| pin known | 5.10 | 1 | CH-04 |
| pin scenario | 6.4 | 0 | — (K12) |
| pin u() | 5.14 | 0 | — (K13) |
| recurrence k-1 | 5.10 | 14 | CH-02, CH-06, LH-02, LH-04, LH-05, LH-07, LH-10, LH-13, LH-15, LH-16, LH-18, MP-15, RC-01, household |
| recurrence k+1 | 5.10 | 1 | LH-18 |
| bind where | 5.10 | 48 | CB-03, CB-04, CB-05, CB-06, CB-07, CB-09, CB-10, CB-12, CB-13, CB-14, CH-01, CH-02, CH-06, CMP-01, CMP-02, LH-01, LH-04, LH-06, LH-09, LH-10, LH-13, LH-14, LH-15, LH-16, MP-03, MP-05, MP-06, MP-07, MP-08, MP-09, MP-13, MP-14, MP-15, MP-16, RC-01, RC-02, RC-03, RC-06, RC-08, UL-02, UL-04, UL-05, UL-08, UL-09, UL-13, UL-14, UL-15, household |
| bind range | 5.10 | 0 | — (R2) |
| bind list | 5.10 | 30 | CB-01, CB-02, CB-06, CH-01, CMP-01, LH-02, LH-04, LH-06, LH-10, LH-11, LH-12, LH-13, LH-17, MP-03, MP-05, MP-10, RC-01, RC-02, RC-03, RC-08, UL-01, UL-03, UL-05, UL-06, UL-09, UL-10, UL-13, UL-14, UL-15, household |
| sum | 5.10 | 50 | CB-04, CB-05, CB-06, CB-08, CB-09, CB-10, CB-12, CB-13, CB-14, CH-02, CH-03, CH-05, CH-06, CMP-01, CMP-02, LH-01, LH-02, LH-04, LH-06, LH-09, LH-10, LH-11, LH-13, LH-15, LH-16, MP-03, MP-05, MP-06, MP-07, MP-08, MP-09, MP-12, MP-13, MP-14, MP-15, MP-16, RC-01, RC-02, RC-03, RC-06, RC-07, RC-08, UL-05, UL-08, UL-09, UL-12, UL-13, UL-14, UL-15, household |
| count | 5.10 | 18 | CB-06, CH-02, CH-03, CH-05, CH-06, CMP-02, LH-13, LH-15, LH-16, MP-14, MP-15, RC-02, RC-03, UL-04, UL-05, UL-13, UL-15, household |
| min/max | 5.10 | 59 | CB-01, CB-02, CB-03, CB-04, CB-05, CB-06, CB-07, CB-10, CB-11, CB-12, CB-13, CB-14, CH-03, CH-05, CH-06, CMP-01, CMP-02, LH-01, LH-03, LH-04, LH-05, LH-06, LH-09, LH-13, LH-14, LH-16, LH-17, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-09, MP-11, MP-12, MP-13, MP-14, MP-15, RC-01, RC-02, RC-03, RC-04, RC-05, RC-06, RC-07, RC-08, UL-03, UL-04, UL-05, UL-07, UL-08, UL-09, UL-10, UL-11, UL-12, UL-14, UL-15, household |
| any/all | 5.10 | 11 | CH-06, LH-04, LH-16, MP-06, MP-08, MP-13, MP-15, RC-02, UL-13, UL-15, household |
| first/last | 5.10 | 9 | CB-03, CB-07, CH-01, LH-14, LH-16, MP-05, MP-07, RC-02, UL-02 |
| argmin/argmax | 5.10 | 5 | CB-04, CH-05, CH-06, LH-11, MP-08 |
| share | 5.10 | 2 | LH-15, LH-16 |
| mean | 5.10 | 2 | LH-15, LH-16 |
| quantile | 5.10 | 3 | CH-01, CH-03, LH-16 |
| missing() | 5.10 | 1 | household |
| abs | 7.1 | 5 | CH-03, CH-06, LH-11, MP-04, MP-15 |
| fill | 5.11 | 9 | CB-05, CB-09, LH-01, LH-02, LH-04, LH-06, LH-09, MP-15, UL-13 |
| keyed fill | 5.11 | 2 | LH-04, UL-13 |
| fill supply/demand for | 5.11 | 9 | CB-05, CB-09, LH-01, LH-02, LH-04, LH-06, LH-09, MP-15, UL-13 |
| prorata | 5.12 | 4 | CMP-01, MP-01, MP-09, MP-12 |
| prorata rest largest | 5.12 | 3 | CMP-01, MP-09, MP-12 |
| prorata rest last/none | 5.12 | 3 | CMP-01, MP-01, MP-09 |
| round | 7.2 | 53 | CB-02, CB-06, CB-10, CB-11, CB-12, CH-01, CH-02, CH-03, CH-04, CH-05, CH-06, CMP-01, CMP-02, LH-02, LH-07, LH-08, LH-10, LH-11, LH-12, LH-13, LH-14, LH-15, LH-16, LH-17, LH-18, MP-03, MP-04, MP-07, MP-09, MP-11, MP-12, MP-13, MP-14, MP-16, RC-01, RC-04, RC-05, RC-07, RC-08, UL-01, UL-02, UL-03, UL-04, UL-05, UL-07, UL-08, UL-09, UL-10, UL-11, UL-12, UL-15, household, chg5 |
| round half-even | 7.2 | 14 | CB-10, CH-02, CH-03, LH-02, LH-10, LH-12, LH-14, LH-15, LH-16, LH-18, UL-01, UL-02, UL-03, UL-07 |
| round down/up/floor/ceil | 7.2 | 17 | CB-02, CB-06, CB-11, CH-06, LH-02, LH-10, LH-11, LH-13, LH-14, MP-16, RC-04, RC-05, RC-08, UL-04, UL-05, UL-07, household |
| pay | 5.13 | 3 | CMP-01, household, chg5 |
| pay for | 5.13 | 2 | household, chg5 |
| pay when | 5.13 | 2 | household, chg5 |
| pay phase | 5.13.2 | 2 | household, chg5 |
| pay promised | 5.13 | 1 | household |
| bal(x) state read | 5.13.1 | 2 | household, chg5 |
| free() | 5.13.1 | 0 | — (K6) |
| plan_low() | 5.13.1 | 1 | household |
| next_on() | 5.13.1 | 0 | — (K7) |
| bal(x, d) | 5.13.1 | 1 | household |
| worth | 5.13.1 | 1 | household |
| flows | 5.13.3 | 2 | CMP-01, household |
| flows for | 5.13.3 | 0 | — (K8) |
| flows conversion | 5.13.3 | 1 | CMP-01 |
| draw | 5.14 | 0 | — (K13) |
| draw plan | 5.14 | 0 | — (K13) |
| distribution (normal, … discrete) | 5.14 | 3 | CH-01, CH-03, LH-17 |
| cdf/pdf/pmf | 5.14 | 3 | CH-01, CH-03, LH-17 |
| crps | 5.15 | 1 | CH-03 |
| crps weight | 5.15 | 1 | CH-03 |
| orient | 5.15 | 1 | CH-03 |
| ask | 5.16 | 80 | CB-01, CB-02, CB-03, CB-04, CB-05, CB-06, CB-07, CB-08, CB-09, CB-10, CB-11, CB-12, CB-13, CB-14, CH-01, CH-02, CH-03, CH-04, CH-05, CH-06, CMP-01, CMP-02, LH-01, LH-02, LH-03, LH-04, LH-05, LH-06, LH-07, LH-08, LH-09, LH-10, LH-11, LH-12, LH-13, LH-14, LH-15, LH-16, LH-17, LH-18, MP-01, MP-02, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-09, MP-10, MP-11, MP-12, MP-13, MP-14, MP-15, MP-16, RC-01, RC-02, RC-03, RC-04, RC-05, RC-06, RC-07, RC-08, UL-01, UL-02, UL-03, UL-04, UL-05, UL-06, UL-07, UL-08, UL-09, UL-10, UL-11, UL-12, UL-13, UL-14, UL-15, household |
| ask for | 5.16 | 60 | CB-01, CB-02, CB-03, CB-04, CB-05, CB-06, CB-07, CB-08, CB-09, CB-10, CB-12, CB-13, CB-14, CH-01, CH-02, CH-03, CH-04, CH-05, CH-06, CMP-01, CMP-02, LH-01, LH-02, LH-03, LH-04, LH-06, LH-10, LH-12, LH-13, LH-14, LH-15, LH-16, LH-17, MP-01, MP-03, MP-05, MP-09, MP-10, MP-11, MP-12, MP-14, MP-15, MP-16, RC-01, RC-02, RC-03, RC-04, RC-08, UL-01, UL-02, UL-03, UL-05, UL-06, UL-09, UL-10, UL-12, UL-13, UL-14, UL-15, household |
| ask with | 5.16 | 24 | CB-07, CH-05, LH-03, LH-04, LH-06, LH-12, LH-15, MP-03, MP-04, MP-05, MP-07, MP-08, MP-10, MP-16, RC-03, RC-05, RC-06, RC-07, UL-01, UL-03, UL-11, UL-13, UL-14, UL-15 |
| ask check | 5.16 | 23 | CB-01, CB-03, CB-05, CB-08, CB-09, CH-02, CH-04, CH-05, CH-06, CMP-01, CMP-02, MP-02, MP-09, MP-12, MP-13, MP-14, MP-15, MP-16, UL-03, UL-04, UL-08, UL-11, UL-13 |
| ask solve | 5.16 | 5 | LH-10, LH-11, LH-17, MP-07, UL-08 |
| ask explain | 5.16 | 2 | CB-07, household |
| ask inputs-of | 5.16 | 1 | CMP-01 |
| ask residuals | 5.16 | 1 | household |
| residuals by | 5.16 | 0 | — (K9) |
| as known | 5.16 | 4 | CH-04, LH-05, LH-14, RC-08 |
| paths seed | 5.16 | 0 | — (K13) |
| real() | 5.16 | 0 | — (K15) |
| bound | 5.17 | 3 | CH-03, LH-17, MP-15 |
| records (movement input) | 5.18 | 1 | household |
| covered() | 5.18 | 1 | household |
| match_days/match_amount | 5.18 | 0 | — (K10) |
| account_place/party_place | 5.18 | 0 | — (K10) |
| stated/short | 5.18 | 1 | household |
| periodic | 3.3 | 3 | LH-10, LH-11, household |
| factor | 3.3 | 3 | CH-05, LH-18, household |
| dcf | 3.3 | 0 | — (K1) |
| exp/ln/sqrt/pow | 7.3 | 9 | CH-03, CH-04, CH-06, LH-10, LH-11, LH-15, LH-16, LH-17, LH-18 |
| age/age_months | 4.2 | 4 | LH-07, LH-11, MP-05, household |
| add_days | 4.2 | 3 | CB-14, LH-07, household |
| add_months/add_years | 4.2 | 7 | CB-07, LH-10, LH-11, MP-05, MP-06, MP-13, household |
| business days | 4.2 | 1 | household |
| days() | 4.2 | 5 | CB-14, LH-01, LH-02, LH-11, household |
| overlap_days | 4.2 | 2 | CB-07, CB-14 |
| weekday | 4.2 | 1 | UL-04 |
| start_of/end_of | 4.2 | 13 | CB-07, CB-08, CB-09, CB-10, CMP-01, LH-07, MP-07, MP-11, UL-02, UL-04, UL-08, UL-10, household |
| year_of/month_of | 4.2 | 7 | CB-07, CB-14, LH-09, LH-16, MP-08, UL-08, household |
| day_of | 4.2 | 2 | household, chg5 |
| pos() | 4.2 | 7 | CH-03, LH-04, LH-05, LH-06, LH-10, LH-16, household |

## 2. Built-in words

`call f` is f directly before `(` (§2.2). Rows dropped as not constructs: `index kind quarter`: not in the grammar: a quarter index is `month … by 3` (§4.1); `word quarter`: the same word as the `period arg quarter` row; `type prices`: `prices` is an input clause, not a type: the `prices` row of table 1 (CH-05, household).

| Word | Cases | Used by |
|---|---|---|
| call sum | 50 | CB-04, CB-05, CB-06, CB-08, CB-09, CB-10, CB-12, CB-13, CB-14, CH-02, CH-03, CH-05, CH-06, CMP-01, CMP-02, LH-01, LH-02, LH-04, LH-06, LH-09, LH-10, LH-11, LH-13, LH-15, LH-16, MP-03, MP-05, MP-06, MP-07, MP-08, MP-09, MP-12, MP-13, MP-14, MP-15, MP-16, RC-01, RC-02, RC-03, RC-06, RC-07, RC-08, UL-05, UL-08, UL-09, UL-12, UL-13, UL-14, UL-15, household |
| call count | 18 | CB-06, CH-02, CH-03, CH-05, CH-06, CMP-02, LH-13, LH-15, LH-16, MP-14, MP-15, RC-02, RC-03, UL-04, UL-05, UL-13, UL-15, household |
| call min | 43 | CB-01, CB-02, CB-03, CB-04, CB-05, CB-06, CB-07, CB-10, CB-13, CB-14, CH-05, CH-06, CMP-01, LH-03, LH-04, LH-06, LH-09, LH-13, LH-14, LH-16, LH-17, MP-04, MP-06, MP-07, MP-08, MP-11, MP-12, MP-13, MP-15, RC-02, RC-04, RC-05, RC-06, RC-07, UL-04, UL-07, UL-08, UL-09, UL-11, UL-12, UL-14, UL-15, household |
| call max | 50 | CB-03, CB-05, CB-07, CB-10, CB-11, CB-12, CB-14, CH-03, CH-05, CMP-01, CMP-02, LH-01, LH-04, LH-05, LH-06, LH-13, LH-14, LH-16, LH-17, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-09, MP-11, MP-12, MP-13, MP-14, MP-15, RC-01, RC-02, RC-03, RC-04, RC-05, RC-06, RC-07, RC-08, UL-03, UL-05, UL-07, UL-08, UL-09, UL-10, UL-11, UL-12, UL-14, UL-15, household |
| call any | 8 | LH-04, MP-06, MP-08, MP-13, RC-02, UL-13, UL-15, household |
| call all | 3 | CH-06, LH-16, MP-15 |
| call first | 8 | CB-03, CB-07, CH-01, LH-14, MP-05, MP-07, RC-02, UL-02 |
| call last | 2 | LH-14, LH-16 |
| call argmin | 2 | CH-06, LH-11 |
| call argmax | 3 | CB-04, CH-05, MP-08 |
| call share | 2 | LH-15, LH-16 |
| call mean | 2 | LH-15, LH-16 |
| call quantile | 3 | CH-01, CH-03, LH-16 |
| call prorata | 4 | CMP-01, MP-01, MP-09, MP-12 |
| call flows | 2 | CMP-01, household |
| call bal | 2 | household, chg5 |
| call free | 0 | — (K6) |
| call plan_low | 1 | household |
| call next_on | 0 | — (K7) |
| call worth | 1 | household |
| call covered | 1 | household |
| call round | 53 | CB-02, CB-06, CB-10, CB-11, CB-12, CH-01, CH-02, CH-03, CH-04, CH-05, CH-06, CMP-01, CMP-02, LH-02, LH-07, LH-08, LH-10, LH-11, LH-12, LH-13, LH-14, LH-15, LH-16, LH-17, LH-18, MP-03, MP-04, MP-07, MP-09, MP-11, MP-12, MP-13, MP-14, MP-16, RC-01, RC-04, RC-05, RC-07, RC-08, UL-01, UL-02, UL-03, UL-04, UL-05, UL-07, UL-08, UL-09, UL-10, UL-11, UL-12, UL-15, household, chg5 |
| call pow | 8 | CH-04, CH-06, LH-10, LH-11, LH-15, LH-16, LH-17, LH-18 |
| call exp | 3 | CH-06, LH-17, LH-18 |
| call ln | 4 | CH-03, LH-11, LH-17, LH-18 |
| call sqrt | 2 | CH-03, LH-16 |
| call abs | 5 | CH-03, CH-06, LH-11, MP-04, MP-15 |
| call periodic | 3 | LH-10, LH-11, household |
| call factor | 3 | CH-05, LH-18, household |
| call dcf | 0 | — (K1) |
| call days | 5 | CB-14, LH-01, LH-02, LH-11, household |
| call add_days | 3 | CB-14, LH-07, household |
| call add_months | 4 | CB-07, LH-10, LH-11, household |
| call add_years | 3 | MP-05, MP-06, MP-13 |
| call start_of | 11 | CB-07, CB-08, CB-09, CB-10, CMP-01, LH-07, MP-11, UL-02, UL-04, UL-08, household |
| call end_of | 6 | LH-07, MP-07, UL-02, UL-04, UL-10, household |
| call year_of | 7 | CB-07, CB-14, LH-09, LH-16, MP-08, UL-08, household |
| call month_of | 1 | household |
| call day_of | 2 | household, chg5 |
| call overlap_days | 2 | CB-07, CB-14 |
| call pos | 7 | CH-03, LH-04, LH-05, LH-06, LH-10, LH-16, household |
| call age | 3 | LH-07, MP-05, household |
| call age_months | 2 | LH-11, MP-05 |
| call weekday | 1 | UL-04 |
| call next_business | 1 | household |
| call prev_business | 0 | — (K4) |
| call add_business | 0 | — (K4) |
| call band_lo | 9 | CB-02, CB-04, CB-07, MP-06, MP-08, RC-08, UL-03, UL-10, UL-15 |
| call band_hi | 1 | UL-15 |
| call marginal | 5 | CB-03, CB-04, CMP-01, MP-11, UL-09 |
| call kind | 1 | household |
| call currency | 1 | household |
| call missing | 1 | household |
| call real | 0 | — (K15) |
| call normal | 3 | CH-01, CH-03, LH-17 |
| call lognormal | 0 | — (K14) |
| call exponential | 0 | — (K14) |
| call gamma | 1 | LH-17 |
| call poisson | 0 | — (K14) |
| call binomial | 1 | CH-01 |
| call negbin | 1 | CH-03 |
| call uniform | 0 | — (K14) |
| call discrete | 0 | — (K14) |
| call cdf | 2 | CH-01, LH-17 |
| call pdf | 1 | CH-03 |
| call pmf | 2 | CH-01, CH-03 |
| call crps | 1 | CH-03 |
| call u | 0 | — (K13) |
| mode down | 14 | CB-02, CH-06, LH-13, LH-14, LH-16, MP-16, RC-04, RC-05, RC-08, UL-04, UL-05, UL-07, household, chg5 |
| mode up | 2 | LH-13, LH-14 |
| mode floor | 7 | CB-06, CB-11, LH-10, LH-11, RC-04, RC-05, household |
| mode ceil | 4 | LH-02, LH-10, LH-11, RC-05 |
| mode half-up | 22 | CB-02, CB-11, CMP-01, CMP-02, LH-02, LH-13, LH-17, MP-07, MP-09, MP-11, MP-12, MP-13, MP-14, MP-16, RC-01, RC-04, RC-05, RC-07, RC-08, UL-10, UL-15, household |
| mode half-even | 3 | LH-02, LH-15, LH-16 |
| mode half-down | 0 | — (R3) |
| daycount act365 | 0 | — (K1) |
| daycount act360 | 0 | — (K1) |
| daycount actact | 0 | — (K1) |
| daycount us30_360 | 0 | — (K1) |
| daycount e30_360 | 0 | — (K1) |
| type money | 71 | CB-01, CB-02, CB-03, CB-04, CB-05, CB-07, CB-08, CB-09, CB-10, CB-11, CB-12, CB-13, CB-14, CH-02, CH-05, CMP-01, CMP-02, LH-01, LH-02, LH-03, LH-04, LH-05, LH-06, LH-07, LH-08, LH-09, LH-10, LH-11, LH-12, LH-13, LH-14, MP-01, MP-02, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-09, MP-10, MP-11, MP-12, MP-13, MP-14, MP-16, RC-01, RC-02, RC-03, RC-04, RC-05, RC-06, RC-07, RC-08, UL-01, UL-02, UL-03, UL-04, UL-05, UL-06, UL-07, UL-08, UL-09, UL-10, UL-11, UL-12, UL-13, UL-14, UL-15, household, chg5 |
| type num | 75 | CB-01, CB-02, CB-03, CB-04, CB-05, CB-06, CB-07, CB-08, CB-09, CB-10, CB-11, CB-12, CB-13, CB-14, CH-01, CH-02, CH-03, CH-04, CH-05, CH-06, CMP-01, CMP-02, LH-01, LH-02, LH-04, LH-05, LH-07, LH-08, LH-10, LH-11, LH-12, LH-13, LH-14, LH-15, LH-16, LH-17, LH-18, MP-01, MP-02, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-11, MP-12, MP-13, MP-14, MP-15, MP-16, RC-01, RC-03, RC-04, RC-05, RC-06, RC-07, RC-08, UL-01, UL-02, UL-03, UL-04, UL-05, UL-06, UL-07, UL-08, UL-09, UL-10, UL-11, UL-12, UL-13, UL-14, UL-15, household, chg5 |
| type int | 49 | CB-04, CB-06, CB-07, CB-11, CB-14, CH-01, CH-02, CH-03, CH-05, CH-06, CMP-01, CMP-02, LH-01, LH-02, LH-04, LH-06, LH-07, LH-08, LH-09, LH-10, LH-11, LH-13, LH-14, LH-15, LH-16, LH-17, LH-18, MP-03, MP-04, MP-05, MP-06, MP-07, MP-09, MP-10, MP-11, MP-13, MP-14, MP-15, RC-02, RC-03, RC-04, RC-05, UL-04, UL-05, UL-07, UL-09, UL-13, UL-15, household |
| type bool | 53 | CB-02, CB-03, CB-05, CB-06, CB-07, CB-09, CB-10, CB-11, CB-12, CB-14, CMP-01, CMP-02, LH-01, LH-02, LH-03, LH-04, LH-06, LH-07, LH-08, LH-13, LH-14, LH-15, LH-16, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-09, MP-11, MP-13, MP-14, MP-15, MP-16, RC-01, RC-02, RC-03, RC-05, RC-06, RC-07, RC-08, UL-04, UL-05, UL-07, UL-08, UL-09, UL-10, UL-12, UL-13, UL-14, UL-15, household |
| type date | 22 | CB-07, CB-08, CB-14, CMP-01, LH-01, LH-02, LH-07, LH-09, LH-10, LH-11, LH-12, LH-14, MP-05, MP-06, MP-07, MP-08, MP-13, MP-16, UL-08, UL-09, UL-10, household |
| type dates | 1 | household |
| type month | 4 | CMP-01, LH-14, UL-05, household |
| type year | 6 | CMP-01, LH-07, LH-09, LH-10, MP-11, household |
| type day | 0 | — (R4) |
| type quarter | 0 | — (R4) |
| type key | 59 | CB-02, CB-03, CB-04, CB-05, CB-06, CB-07, CB-08, CB-09, CB-10, CB-11, CB-13, CB-14, CH-01, CH-02, CH-05, CMP-01, CMP-02, LH-01, LH-02, LH-03, LH-04, LH-07, LH-08, LH-09, LH-10, LH-12, LH-13, LH-14, LH-15, LH-16, MP-03, MP-04, MP-05, MP-06, MP-07, MP-08, MP-09, MP-10, MP-13, MP-14, MP-15, MP-16, RC-02, RC-03, RC-06, RC-07, RC-08, UL-01, UL-02, UL-03, UL-04, UL-05, UL-07, UL-08, UL-09, UL-10, UL-13, UL-14, household |
| type rate | 5 | CH-05, LH-10, LH-11, LH-18, household |
| type movement | 1 | household |
| rate nominal | 2 | LH-10, LH-11 |
| rate effective | 2 | CH-05, LH-18 |
| rate simple | 0 | — (K2) |
| rate discount | 1 | LH-11 |
| index kind year | 28 | CB-02, CB-03, CB-05, CB-07, CB-08, CB-10, CB-13, CB-14, LH-04, LH-05, LH-07, LH-08, LH-09, LH-16, MP-04, MP-07, MP-08, RC-04, RC-08, UL-01, UL-02, UL-04, UL-06, UL-08, UL-10, UL-12, UL-14, household |
| index kind month | 11 | CB-09, LH-14, LH-15, LH-16, RC-08, UL-02, UL-05, UL-06, UL-13, UL-15, household |
| index kind day | 3 | UL-04, UL-06, household |
| period arg year | 3 | CH-05, LH-11, LH-18 |
| period arg month | 3 | LH-10, LH-11, household |
| period arg day | 0 | — (K3) |
| period arg quarter | 0 | — (K3) |
| place kind account | 3 | CMP-01, household, chg5 |
| place kind cash | 1 | household |
| place kind holding | 1 | household |
| place kind asset | 0 | — (K5) |
| place kind debt | 1 | household |
| place kind claim | 1 | household |
| place kind outside | 2 | household, chg5 |
| word inf | 20 | CB-02, CB-03, CB-04, CB-07, CMP-01, MP-05, MP-06, MP-08, MP-09, MP-10, MP-11, RC-07, RC-08, UL-01, UL-03, UL-07, UL-09, UL-10, UL-15, household |
| word path | 0 | — (K13) |

## 3. Zero-use constructs: the decisions

| # | Construct | LANGUAGE | Decision | Reason |
|---|---|---|---|---|
| R1 | `law D for [x, y]` (a law-date pin on named items); with it, `wording D` without `for` | §2.3, §4.3, §5.10 | removed | No case. UL-15, the case that pins named items, needs the wording in force on D, which `wording D for [x, y]` gives (DECISIONS 34), and the 10 models that move the law date move every item (`law D`). `wording D` without `for` had no case either, so `for` is now required with `wording`. |
| R2 | a binding over a range, `k in a..b [by n]` | §2.3 `bind`, §5.6 | removed | No case. A binding is over an index, a key list or `path`; a computed bound is a `where` over a declared index (`k in inst where k <= lh_terms[l].months`, as LH-10 does), so every binding stays bounded at load. |
| R3 | the `half-down` rounding mode | §2.3 `MODE`, §7.2 | removed | No case, and no spec ID names it. The cases use `down`, `up`, `floor`, `ceil`, `half-up` and `half-even`. A law that rounds half toward zero would come in a later language version (§9). |
| R4 | `day` and `quarter` as type words | §2.2 | removed from the type list | They are not types in the `type` production; they stay period words (an index kind, `periodic`, `factor`). |
| K1 | `dcf(a, b, c)` and its five day counts | §3.3 | kept | U-15 (a rate carries its convention) and CHG-06 (taken: "dcf with five day counts"). The cases state their interest per period, so none counts days between dates. |
| K2 | rates `simple /year`, `/month`, `/day` | §3.3 | kept | U-15, CHG-06 (compounding and basis are part of a rate). Statutory late-payment interest, which B2B invoices carry, is simple interest. LH-10, LH-11, CH-05, LH-18 use `nominal`, `effective`, `discount`. |
| K3 | `periodic` and `factor` over `day` and `quarter` | §3.3 | kept | L-08 (quarterly VAT filing). `month` and `year` are used. |
| K4 | `prev_business`, `add_business` | §4.2 | kept | T-05 ("the third-to-last business day"), U-04 and L-10 (FX at the NBP rate of the previous business day). `next_business` (the household) runs on the same calendar. |
| K5 | `place x: asset CUR priced v` | §5.9 | kept | M-02 and M-19 (things no bank shows exist as places with a valuation). The household values its holdings (`holding … priced`) but owns no house or car. |
| K6 | `free(x)` | §5.13.1 | kept | Spec §4#24 (VAT collected isn't ours, N-504) and J66. The household's rule reads `plan_low`, which subtracts the hold itself. |
| K7 | `next_on(p[keys], d)` | §5.13.1 | kept | R-05 and Q-17 ("no further pay"), and the spec's own rule "invest what's above 1 000 until the next pay". The household writes its look-ahead window as dates, the 15th to the day before the next 15th. |
| K8 | `flows(x, span, for k)` | §5.13.3 | kept | N-305: the period a pay is for, apart from the day it is paid (a December wage paid in January). |
| K9 | `residuals x span by f` | §5.16, E §6.3 | kept | A-23 (reconcile by category or counterparty) and P-02. The household's replay asks the default grouping. |
| K10 | the importer's inputs `account_place`, `party_place`, `match_days`, `match_amount` | §5.18 | kept | P-24 and DECISIONS 15. The importer reads them and the model never does, so no model can use them; OPERATIONS §2.2 does. |
| K11 | `scenario s over s0` (stacking) | §6.4 | kept | Spec §8's edge cases ("Model shape") include scenarios stacked several deep. Change 7 is one scenario. |
| K12 | `with scenario s`, `as if {scenario s}` | §6.4, §5.16 | kept, used | Change 7 asks `net_worth with scenario wife_stops` and an `as if` difference in its outputs table, which the scan doesn't read (it reads added lines only). |
| K13 | Monte Carlo: `draw`, its `plan` value, `u(r) = x` pins, `paths n seed s`, bindings over `path` | §5.14, §5.16, E §7 | kept, untested | The brief §1 lists Monte Carlo in the engine; R-05, R-19, P-16, Q-06, Q-08; BOUNDS 13, 20, 46, 51; hindcast fixtures H0, H6, H7. No model draws: LH-17, LH-18 and CH-06 compute their probabilities exactly, CH-01 and CH-03 score stated distributions, MP-14 and CMP-02 stipulate the lot's winner, and CH-03 declares numpy's stream a bound. The planned lists of DECISIONS §7 named 7 cases; none drew. |
| K14 | distributions `lognormal`, `exponential`, `poisson`, `uniform`, `discrete` | §5.14 | kept, untested | P-16 (a compound-Poisson drain of spending, rare costs as a marked point process); N-1402, a draw by lot (MP-14's lot, stipulated). `normal`, `gamma`, `binomial`, `negbin` are used (CH-01, CH-03, LH-17). |
| K15 | `real(x, d, base, idx)` | §5.16 | kept, untested | U-10 and the spec's "the deflator is a lens" (brief §4). No case asks in today's money; the household carries an inflation index but its asks are nominal. |

## 4. Where the plan and the models differ

DECISIONS §7 and LANGUAGE's **Cases** lines were written in D5 from the case files, before the models. The models chose otherwise in places:

| Construct | Planned (D5) | In the models |
|---|---|---|
| `draw` | LH-17, LH-18, CH-06, MP-14, CMP-02, CH-03, CH-01 | none (K13) |
| path statistics | LH-17, LH-18, CH-06 | none (K13) |
| `pay` | CMP-01, CMP-02, MP-14, MP-15, LH-14, LH-15, LH-16, UL-09, RC-08, the household | CMP-01, the household, change 5. The others hold their money as defs, which their dates allow |
| scenario | LH-10, CB-01, LH-04, UL-10, MP-14, the household | change 7 only: the cases' options are readings, so they are axes |
| `as if` | 18 models | 10 models; with the `with` pins of asks, 30 models pin something |
| `missing()` | CH-01 | the household |
| `check` | CH-06, the household's replay | 23 models |

So DECISIONS §7 is regenerated from table 1, and so are LANGUAGE's **Cases** lines. BOUNDS row 51 named LANGUAGE §5.14's list for draws by address; no model draws, so its evidence is the design only. BUILD-ORDER M7 turns no case green: its five cases go to M4 (MP-14, CMP-02) and M6 (LH-17, LH-18, CH-06), and M7 keeps H0, H6 and H7.

## 5. Constructs per case

The number of table-1 constructs each column uses: the fewest is chg7 (2), the most household (95).

CB-01 21, CB-02 32, CB-03 37, CB-04 26, CB-05 30, CB-06 32, CB-07 44, CB-08 24, CB-09 35, CB-10 31, CB-11 27, CB-12 24, CB-13 24, CB-14 37, CH-01 25, CH-02 29, CH-03 34, CH-04 21, CH-05 34, CH-06 33, CMP-01 54, CMP-02 31, LH-01 32, LH-02 37, LH-03 22, LH-04 41, LH-05 18, LH-06 30, LH-07 33, LH-08 24, LH-09 30, LH-10 46, LH-11 37, LH-12 31, LH-13 37, LH-14 40, LH-15 35, LH-16 38, LH-17 29, LH-18 22, MP-01 20, MP-02 16, MP-03 35, MP-04 31, MP-05 41, MP-06 35, MP-07 36, MP-08 37, MP-09 36, MP-10 25, MP-11 33, MP-12 25, MP-13 33, MP-14 31, MP-15 32, MP-16 35, RC-01 30, RC-02 34, RC-03 30, RC-04 26, RC-05 26, RC-06 26, RC-07 27, RC-08 42, UL-01 28, UL-02 27, UL-03 33, UL-04 32, UL-05 37, UL-06 24, UL-07 28, UL-08 34, UL-09 38, UL-10 35, UL-11 22, UL-12 25, UL-13 39, UL-14 32, UL-15 44, household 95, chg1 5, chg2 3, chg3 5, chg4 8, chg5 22, chg6 6, chg7 2.
