# Milestone M4: what must turn green

The checks M4 (law breadth) must turn green, from BUILD-ORDER § M4 ("Builds", "Green (77)", "Budgets touched") and DONE gates 1, 3, 8 and 12, with the items the run's brief puts in scope (issue #252). Written by T for issue #252 (2026-11-22) on refcore **tally 2.3**, from T's plan approved by the coordinator (picks P1–P5, `tmp/m4/auto-picks.md`; the paper check's open cells filed as #253). The builder reads this page; T changes it.

## The list

"Today" is the engine at master fb3d596f (tally 2.3 and the process step after it), 2026-11-22. Every `m4_…` target has `test = false` and runs only as `check`'s report until M4's marker commit (as at M1 to M3), so T's commits stay green with no `Logged-Red:` line.

| # | What | Ids, vectors | Run by | Gate | Today |
|---|---|---|---|---|---|
| 1 | BUILD-ORDER § M4's 77 checks | 73 in `m4/vectors.tsv`, group `checks-…` (§ Row 1); the other 4, `U:cur/CUR-01`, `U:cur/CUR-02`, `U:law/LAW-12`, `I:M:RC-07`, gate in `s_kernel` since S | `m4_checks` (4 tests) | 1, 3 | 69 green; 3 red (`checks-pins` 2, `checks-accrue` 1); `I:D:d11_not_follows_fund` held to M7 |
| 2 | the corners of M4's constructs | `m4/constructs/`, group `constructs-…` (§ Row 2): `pins` 15, `accrue` 9 (chunk C1); `round`, `round-axis`, `logic`, `aggregates`, `argm`, `beats-given`, `fx`, `cal` and the paper check's cells follow in chunk C2 | `m4_constructs` (3 tests) | 1 | `pins` 12 of 15 green, `accrue` 1 of 9 |
| 3 | the in-scope backlog: #238's standing items, #245's "Built in part" (`RUN-NONE`'s wording, P3; `explain` of a decision's `flows` or `count`), #250's "The engine alone" (`check` of two months, `age_months` in the calendar's last month) | chunk C3 | `m4_items` | 1 | not written |
| 4 | P8-L2, P8-L10 at full size | chunk C3: the run lists for the rules step's `load:` recipe (pick P5), P8-L2's digest from the cut twin's blocks | `engine/tools/perf row` at the gate | 12 | P8-L2 0.47 s, 60 MB on the quick binary; P8-L10's c1_r2 7.39 s, 362 MB |
| 5 | gate 8, the floor 88.9 % | chunk C7, after the one mutation run | `m4_fixes` | 8 | — |

`check`'s report line counts the `m4_…` targets' tests: `milestone-report: M4: 3/7 green (gates once M4 is tagged)` after C1 (`m4_checks`' 4 and `m4_constructs`' 3; green: the two every-group tests and `m4_checks_green`).

## Row 1: the 77 checks (chunk C1)

**The bytes.** `m4/vectors.py` runs each check's command line on plain refcore (run.py's `run_one`, PyPy, 90 s, the watchdog) and stores the whole output in `m4/vectors.refcore`; a household check runs with its own asks, as M2 stored X's `@c3_fast`. Before it writes, each output is tied to the check's stored baseline by run.py's own compare (a household check by its ask's blocks, any other whole), so no expected byte is new. All 73 tie. `--check` runs in `run.py --self-test` (8u). The store took 67 s on plain refcore at 2 jobs; refcore_fast runs the 77 in 121 s summed.

| Group (`for`) | Checks | Today | What turns it green |
|---|---|---|---|
| `checks-green` (`-`) | 69: `U:cur/CUR-03`, `U:law/LAW-02`, `-07`, `-10`, `-11`; 32 cases; X's `wife_net`, `social_and_health`, `pit37` on change 5 (`@c5_fast`) and its variant (`@v5_fast`); `I:C:C-01~c01`, `C-02~c02`, `C-06~c06`, `C-08~c08`; 22 of D's 23 | green, whole output byte for byte | kept green |
| `checks-green` (`M7`) | `I:D:d11_not_follows_fund` | held to M7 by name (`held.tsv`, pick P5): `inputs-of r_path[2031] … paths 1 seed 20261001 path 1` reads the `zr` draws (M7) through `sqrt(…) to 1e-12` (M6); D-11's edges name no RAND, so BUILD-ORDER's placement rule 4 left it at M4 | M7 |
| `checks-pins` (`pins`) | `I:M:UL-14` (`q20_ln20`, `q21_ln21`, `q24_smaller`, `q25_line6b`), `I:C:C-04~c04` (`c04_tenth`) | stopped "not implemented" | an `as if` pin in a def of two keys whose value reads the def's keys (Row 2's `b_def`, `b_lit`, `b_mix`): UL-14's `ws2_stop[t, y] = stop[t, y] as if {benefits[t, y] = benefits[t, y] + lump_for[t, y]}`, C-04's `tl1[y, f]`, `tl2`, `tu1`, `tu2` and `tenth` (`as if {gift_used[y] = tl0[y, f]}`, and the inverse `min(t for t in tl2[y, f] .. tu2[y, f] step 1 DUK where … as if {gift_used[y] = t})`) (L §5.11 `as if`, "An inverse") |
| `checks-accrue` (`accrue`) | `I:E2:E2-01~e2c` (`e2c_02_jan_from_marta`, `…_reported`, `…_unreported`, `e2c_03_loan`) | stopped "not implemented" | `accrue(p, r, a, b, act365, simple)` (L §3.3): C's loan law, `day_interest[d] = accrue(owed_start[d], loan_rate[d], d, d + 1, act365, simple)` |

**Household D's law parts:** `tally check` on cut11's 89 files (D's 63 law parts in full, the household, the asks) prints `ok`; on the engine 65 of D's 69 asks run (the four that stop: `d05_duplicate`, M5a; `d10_bands`, `d10_paths`, M7; `d11_not_follows_fund`, above).

## Row 2: the constructs' corners (chunk C1 of C2)

Every text is invented; every figure and date goes through a row (`LOAD-LITERAL`). One vector is one ask.

**`pins`** (`m4/constructs/pins/pins.tally`, L §5.11 `as if`, §8.2 `pinned:`): `a_def` (a pin keyed by the def's own key, its value reading the pinned input unpinned: 220, 420, 620), `a_for` (keyed by an ask's `for`), `a_agg` (by an aggregate's bound variable, `sum((dbl[k] as if {…}) for k in ks)`), `a_other` (a pin at another key than the one read: only k1 moves), `a_least` (the least t such that c, the pin's value the bound t: 30, 60, and `RUN-EMPTY` where no t satisfies), `a_two` (one def under two pin sets in one line), `a_hole` (a pin key that is a hole: the line is the hole), `a_with` (a `with` pin over a def's own pin), `a_explain`; in a def of two keys: `b_def` (a pin at both keys, its value reading them), `b_for` (an ask's two `for` binds), `b_one` (a pin at one of the two keys, its value fixed), `b_lit` (`b_def`'s def at literal keys), `b_mix` (a pin at one key whose value reads the two-key def's keys), `b_flat` (a pin at both keys, its value fixed). **Today:** 12 green; `b_def`, `b_lit` and `b_mix` stop "not implemented", the form UL-14 and C-04 stop on.

**`accrue`** (`m4/constructs/accrue/accrue.tally`, L §3.3): `c_day` (act365 simple, one day: 36 500 × 5 % / 365 = 5 PLN), `c_month` (31 days: 155), `c_year_eff` (`effective` over a whole year: 1 825), `c_nominal` (`nominal 365` over one day: 5), `c_back` (b before a: −5), `c_hole` (a rate that is a hole), `c_dcf` (`dcf` both ways: ±31/365), `c_change` (a rate read at its day across a change on 2026-07-01: 5, 5, 7.3, 7.3), `c_sum` (their sum: 24.6). **Today:** `c_dcf` green; the 8 others stop "not implemented".

## Meets an open issue, gating on refcore's bytes (pick P4)

None in chunk C1. Chunk C2 lists here each vector of the paper check that meets an open #250 row both interpreters share (D-1's last element, row 6).
