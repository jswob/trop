# Milestone M4: what must turn green

The checks M4 (law breadth) must turn green, from BUILD-ORDER § M4 ("Builds", "Green (77)", "Budgets touched") and DONE gates 1, 3, 8 and 12, with the items the run's brief puts in scope (issue #252). Written by T for issue #252 (2026-11-22) on refcore **tally 2.3**, from T's plan approved by the coordinator (picks P1–P5, `tmp/m4/auto-picks.md`; the paper check's open cells filed as #253). The builder reads this page; T changes it.

## The list

"Today" is the engine at master fb3d596f (tally 2.3 and the process step after it), 2026-11-22. Every `m4_…` target has `test = false` and runs only as `check`'s report until M4's marker commit (as at M1 to M3), so T's commits stay green with no `Logged-Red:` line.

| # | What | Ids, vectors | Run by | Gate | Today |
|---|---|---|---|---|---|
| 1 | BUILD-ORDER § M4's 77 checks | 73 in `m4/vectors.tsv`, group `checks-…` (§ Row 1); the other 4, `U:cur/CUR-01`, `U:cur/CUR-02`, `U:law/LAW-12`, `I:M:RC-07`, gate in `s_kernel` since S | `m4_checks` (4 tests) | 1, 3 | 69 green; 3 red (`checks-pins` 2, `checks-accrue` 1); `I:D:d11_not_follows_fund` held to M7 |
| 2 | the corners of M4's constructs, and the paper check | 242 in `m4/vectors.tsv`, group `constructs-…`: 103 corners in `m4/constructs/` (§ Row 2: `pins` 15, `accrue` 9, chunk C1; `round` 16, `logic` 14, `aggregates` 11, `argm` 9, `beats` 7, `fx` 9, `cal` 13, chunk C2) and 139 probes in `m4/paper/` (§ The paper check, chunk C2); a vector the engine doesn't answer today sits in the group of the construct that turns it green | `m4_constructs` (15 tests) | 1 | 221 green; 18 red (`constructs-pins` 3, `-accrue` 8, `-keydiff` 2, `-ratepin` 3, `-tuple` 2); `constructs-calrow`'s 3 held for #253 (equal to refcore today) |
| 3 | the in-scope backlog: #238's standing items, #245's "Built in part" (`RUN-NONE`'s wording, P3; `explain` of a decision's `flows` or `count`), #250's "The engine alone" (`check` of two months, `age_months` in the calendar's last month) | chunk C3 | `m4_items` | 1 | not written |
| 4 | P8-L2, P8-L10 at full size | chunk C3: the run lists for the rules step's `load:` recipe (pick P5), P8-L2's digest from the cut twin's blocks | `engine/tools/perf row` at the gate | 12 | P8-L2 0.47 s, 60 MB on the quick binary; P8-L10's c1_r2 7.39 s, 362 MB |
| 5 | gate 8, the floor 88.9 % | chunk C7, after the one mutation run | `m4_fixes` | 8 | — |

`check`'s report line counts the `m4_…` targets' tests: `milestone-report: M4: 12/19 green (gates once M4 is tagged)` after C2 (`m4_checks`' 4 and `m4_constructs`' 15; red: `m4_checks_pins`, `m4_checks_accrue`, `m4_constructs_pins`, `_accrue`, `_keydiff`, `_ratepin`, `_tuple`). After C1 it was 3/7.

The store holds 315 vectors (73 + 103 + 139); `m4/vectors.py --engine` lists the ones the release binary answers differently, `for` in `m4/vectors.tsv` names the construct each waits for.

## Row 1: the 77 checks (chunk C1)

**The bytes.** `m4/vectors.py` runs each check's command line on plain refcore (run.py's `run_one`, PyPy, 90 s, the watchdog) and stores the whole output in `m4/vectors.refcore`; a household check runs with its own asks, as M2 stored X's `@c3_fast`. Before it writes, each output is tied to the check's stored baseline by run.py's own compare (a household check by its ask's blocks, any other whole), so no expected byte is new. All 73 tie. `--check` runs in `run.py --self-test` (8u). The store took 67 s on plain refcore at 2 jobs; refcore_fast runs the 77 in 121 s summed.

| Group (`for`) | Checks | Today | What turns it green |
|---|---|---|---|
| `checks-green` (`-`) | 69: `U:cur/CUR-03`, `U:law/LAW-02`, `-07`, `-10`, `-11`; 32 cases; X's `wife_net`, `social_and_health`, `pit37` on change 5 (`@c5_fast`) and its variant (`@v5_fast`); `I:C:C-01~c01`, `C-02~c02`, `C-06~c06`, `C-08~c08`; 22 of D's 23 | green, whole output byte for byte | kept green |
| `checks-green` (`M7`) | `I:D:d11_not_follows_fund` | held to M7 by name (`held.tsv`, pick P5): `inputs-of r_path[2031] … paths 1 seed 20261001 path 1` reads the `zr` draws (M7) through `sqrt(…) to 1e-12` (M6); D-11's edges name no RAND, so BUILD-ORDER's placement rule 4 left it at M4 | M7 |
| `checks-pins` (`pins`) | `I:M:UL-14` (`q20_ln20`, `q21_ln21`, `q24_smaller`, `q25_line6b`), `I:C:C-04~c04` (`c04_tenth`) | stopped "not implemented" | an `as if` pin in a def of two keys whose value reads the def's keys (Row 2's `b_def`, `b_lit`, `b_mix`): UL-14's `ws2_stop[t, y] = stop[t, y] as if {benefits[t, y] = benefits[t, y] + lump_for[t, y]}`, C-04's `tl1[y, f]`, `tl2`, `tu1`, `tu2` and `tenth` (`as if {gift_used[y] = tl0[y, f]}`, and the inverse `min(t for t in tl2[y, f] .. tu2[y, f] step 1 DUK where … as if {gift_used[y] = t})`) (L §5.11 `as if`, "An inverse") |
| `checks-accrue` (`accrue`) | `I:E2:E2-01~e2c` (`e2c_02_jan_from_marta`, `…_reported`, `…_unreported`, `e2c_03_loan`) | stopped "not implemented" | `accrue(p, r, a, b, act365, simple)` (L §3.3): C's loan law, `day_interest[d] = accrue(owed_start[d], loan_rate[d], d, d + 1, act365, simple)` |

**Household D's law parts:** `tally check` on cut11's 89 files (D's 63 law parts in full, the household, the asks) prints `ok`; on the engine 65 of D's 69 asks run (the four that stop: `d05_duplicate`, M5a; `d10_bands`, `d10_paths`, M7; `d11_not_follows_fund`, above).

## Row 2: the constructs' corners (chunks C1 and C2)

Every text is invented; every figure and date goes through a row (`LOAD-LITERAL`). One vector is one ask; a file refused whole is one vector, `constructs/<group>/<file's stem>`. Each was written against the spec's sentence and its refcore output read before it was stored.

| Group (`for`) | Vectors | Today | What turns it green |
|---|---|---|---|
| `constructs-pins` (`pins`) | 15 | 12 green; `b_def`, `b_lit`, `b_mix` red | an `as if` pin at a two-key def's keys, as Row 1's `checks-pins` |
| `constructs-accrue` (`accrue`) | 9 | `c_dcf` green; 8 red | `accrue`, as Row 1's `checks-accrue` |
| `constructs-round`, `-logic`, `-aggregates`, `-argm`, `-beats`, `-fx`, `-cal` (`-`) | 16, 14, 11, 8, 7, 7, 13 | green | kept green |
| `constructs-keydiff` (`keydiff`) | `paper/axis__kleene_agg_argm/chk`, `paper/explain_check__rows/b` | red: a `check` of two keys drops its `difference` line | `check` of a def of two keys or periods printing its `difference` (L §5.18 `check`) |
| `constructs-ratepin` (`ratepin`) | `constructs/fx/r_pin_inv`, `r_pin_fx`, `paper/fx__hole_law_ver/c` | red: `RUN-DOMAIN`, the bare literal pinned to a rate takes no unit | "a bare literal in a row or a pin takes its target's declared type and unit" (L §3.2 Inline figures): `with fx = 4.5` on a `num PLN/EUR` input is 4.5 PLN/EUR |
| `constructs-tuple` (`tuple`) | `constructs/argm/t_tuple`, `paper/print__rows/c` | red: stops on `argmax`'s tuple | `argmax`/`argmin` over two binds returning the tuple, printed as refcore prints it, `(k1, 1)` (#253 item 8, pick P1) |
| `constructs-calrow` (`calrow`) | `paper/cal__ver/a`, `paper/cal__ver/f`, `paper/scen_branch/n` | held for #253 (`held.tsv`); equal to refcore today | #253 item 3 (pick P1): the engine follows L §4.2's "`len`'s row governing d"; refcore's bytes stay stored and the spec's answers are filled from the engine's run after the builder's group, M2's way |

**`pins`** (`m4/constructs/pins/pins.tally`, L §5.11 `as if`, §8.2 `pinned:`): `a_def` (a pin keyed by the def's own key, its value reading the pinned input unpinned: 220, 420, 620), `a_for` (keyed by an ask's `for`), `a_agg` (by an aggregate's bound variable, `sum((dbl[k] as if {…}) for k in ks)`), `a_other` (a pin at another key than the one read: only k1 moves), `a_least` (the least t such that c, the pin's value the bound t: 30, 60, and `RUN-EMPTY` where no t satisfies), `a_two` (one def under two pin sets in one line), `a_hole` (a pin key that is a hole: the line is the hole), `a_with` (a `with` pin over a def's own pin), `a_explain`; in a def of two keys: `b_def` (a pin at both keys, its value reading them), `b_for` (an ask's two `for` binds), `b_one` (a pin at one of the two keys, its value fixed), `b_lit` (`b_def`'s def at literal keys), `b_mix` (a pin at one key whose value reads the two-key def's keys), `b_flat` (a pin at both keys, its value fixed). **Today:** 12 green; `b_def`, `b_lit` and `b_mix` stop "not implemented", the form UL-14 and C-04 stop on.

**`accrue`** (`m4/constructs/accrue/accrue.tally`, L §3.3): `c_day` (act365 simple, one day: 36 500 × 5 % / 365 = 5 PLN), `c_month` (31 days: 155), `c_year_eff` (`effective` over a whole year: 1 825), `c_nominal` (`nominal 365` over one day: 5), `c_back` (b before a: −5), `c_hole` (a rate that is a hole), `c_dcf` (`dcf` both ways: ±31/365), `c_change` (a rate read at its day across a change on 2026-07-01: 5, 5, 7.3, 7.3), `c_sum` (their sum: 24.6). **Today:** `c_dcf` green; the 8 others stop "not implemented".

**C2's texts.** `round` (`round.tally`: the seven modes at a tie and off it, both signs; steps of 0.05 PLN and 1 %, `minor(EUR)`, a whole count; the mode as an axis's key, pinned with `with`, on a negative amount; `round_step_unit.tally` refused whole, a num rounded by a num step stored as money, L §7.2). `logic` (`logic.tally`: Kleene's `and`, `or`, `not` over true, false and a hole; `if` on a hole and on a branch not taken; `stop` and `stop` under `not`, `RUN-DIV`; `none` under `and` and `or`, `RUN-NONE` in refcore's words, pick P3; L §5.11 Logic). `aggregates` (`agg.tally`: `any`, `all`, `count`, `sum` over a hole and a term known to be 0, `min`, `max`, `first`, `last`, each over no member, `mean`, `share`; L §5.11 Aggregates). `argm` (`argm.tally`: by position and by a rank, a tuple, over booleans, dates, keys and things, a tie with no `ties by`, `RUN-TIE`; `first_things.tally` refused whole). `beats` (`beats.tally`: a list, a rule off it, `RUN-NO-RULE`, two equal rules, `RUN-CONFLICT`; a wide `given` against a narrower one at a later law date, which wins; `explain`). `fx` (`fx.tally`: a rate PLN/EUR and its inverse applied each way and divided back, a rate plus a margin and times a number, pinned by a bare literal both ways, rates multiplied; `fx_times_wrong.tally` and `fx_add_two.tally` refused whole). `cal` (`cal.tally`: calendars per country, weekends Saturday–Sunday, Friday–Saturday and Sunday alone; `is_business`, `add_business` forward, back and through a key's country, `last_business`, a holiday `from` 2027, an `added` day, the coverage hole `HOLE-CALENDAR` for 2028; `schedule` under `following`, `preceding`, `modified_following` and `unadjusted`, under the other weekend, and on the 31st in February, `none`; L §4.2).

## The paper check (chunk C2)

`m4/paper/` holds the paper check's 43 probes (`tmp/m4/paper-check.md`, 2026-11-22, refcore tally 2.3 at fb3d596f), unchanged: 139 vectors, one per ask (129), one per file refused whole (9: `beats__ver`, `beats__ver3`, `cal__window_added`, `fx__load`, `given__ver_replace`, `load__beats_unknown`, `load__cal_def`, `load__given_key`, `load__round_mode`), and `given__scen`'s `d` run after `round__axis.tally`. A scenario runs after its base: `scen_branch.tally` after `scen_base.tally` (D-9, D-20). The paper check's 52 decided cells are its entries D-1 to D-26 (§2 there); each entry's probe lines are vectors here, every one refcore's bytes.

Its 9 open cells, filed as #253's 8 items, are stored the same way, by the picks:

| #253 item | Vectors | Pick | Gate |
|---|---|---|---|
| 1 `beats` chains | `paper/beats__hole/*` | P1: the engine follows refcore | gates on refcore's bytes |
| 2 a given can't be replaced | `paper/given__ver_replace` (refused whole) | P1 | gates |
| 3 a business-day count across a calendar's rows | `paper/cal__ver/a`, `/f`, `paper/scen_branch/n` | P1: the engine follows the text | held for #253 (`constructs-calrow`) |
| 4 a `beats` cycle across versions | `paper/beats__ver`, `paper/beats__ver3` (refused whole) | P1 | gates |
| 5 `held:` not merged | `paper/cal__register/*`; `unfulfilled:` the same, `paper/beats__move_keys/*` | P2: no merge in M4 | gates on refcore's bytes |
| 6 `any`/`all` with a refused member | `paper/kleene_agg__refusal/*`, `paper/agg__refusal_first/*` | P1 | gates |
| 7 a hole added day (design question) | `paper/cal__hole_movement/*` | decided as written | gates |
| 8 no printer for a calendar or a tuple | `paper/print__rows/*` | P1 | gates (`c` in `constructs-tuple`) |

## Meets an open issue, gating on refcore's bytes (pick P4)

`paper/kleene_agg_argm__hole/a` (D-1): its last element, `w <= w2` over overlapping windows, names only `w`, not `w2`: #250 row 6, which both interpreters share. It gates on refcore's bytes (green today).
