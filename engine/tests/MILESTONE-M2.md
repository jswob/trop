# Milestone M2: what must turn green

The checks M2 (replay against reality) must turn green, from BUILD-ORDER § M2 ("Green (26)", "Properties", "Budgets touched") and DONE gates 1, 2, 3, 4, 6 and 8, with the items the run's brief puts in scope (issue #167). Written by T for issue #167 (2026-11-14) on refcore **tally 2.1.7**, from T's plan approved by the coordinator (picks P1–P5). The builder's groups are its plan's (`tmp/m2/build-plan.md`); where a group isn't known yet, the `for` column says `plan`. The builder reads this page; T changes it.

## The list

"Today" is the engine at `build/m2` e80ff0b (master after tally 2.1.7 and PR #166), 2026-11-14. Every `m2_…` target has `test = false` and runs only as `check`'s report until M2's marker commit (as at M1), so T's commits stay green with no `Logged-Red:` line; each vector's builder group is the `for` column of `m2/vectors.tsv` and of the table below.

| # | What | Ids, vectors | Run by | Gate | For | Today |
|---|---|---|---|---|---|---|
| 1 | the 26 checks of § M2, whole output = refcore's | 3 already held by `s_kernel` (gating: `I:M:LH-05`, `U:rec/REC-03`, `U:surf/SURF-06`); the other 23 in `m2/vectors.refcore` (chunk C1) | `cargo nextest run -p tally-runner --test m2_checks` (3 tests: `checks-green`, `checks-plan`, one that every group has its test); `m2/vectors.py --check` in `run.py --self-test` (8m) | 1, 3 | plan | 23 of 26 green (3 in `s_kernel`, 20 in `checks-green`); 3 red, each `tally run: not implemented` (exit 69): `U:rule/RULE-07`, `U:rule/RULE-08`, `I:M:UL-15` |
| 2 | goldens G4 and G5, and the halves of G1, G2, G3, G6 and G10 that read the prototype's bank history, with its 20 pins, on that ledger tracked under `engine/tests/` (#114) | chunk C4 | `m2_goldens` | 2 | plan | not written |
| 3 | the hindcast fixtures H1–H5, H8, H9, each landing in the ledger line its answer key names | chunk C5 | `m2_hindcasts` | 4 | plan | not written |
| 4 | the codes of M2's constructs no output prints, one fixture each: `HOLE-SAME`'s disagreement form and four `RUN-RECORD` forms | chunk C6 | `m2_codes` | 5 | plan | not written |
| 5 | the properties of § M2: masking at K, the three-way split, the fixed past, forks on and off | chunks C7, C8 | `m2_properties`, `m2_forks` | 6 | plan | not written |
| 6 | the brief's in-scope items: #147, #101, #158, #132, #42, #163, #164, item 9's pair | chunk C2; 15 rows held in `held.tsv` for #147 (9), #101 (4), #158 (1), #132 (1), all `m1_fixes` | `m2_items` | 1 | plan | not written; the 15 held rows red on the engine (not implemented) |
| 7 | item 10: the 63 over-specified `fixes/g12/*` vectors restated to their asks | chunk C3 | `m1_fixes`, `m1_lows` (gating) | 1 | — | chunk C3 |
| 8 | the builder groups' review fixes and gate 8's survivors | chunk C9 on | `m2_fixes` | 1, 8 | each group | — |

`check`'s report line counts the `m2_…` targets' tests: `milestone-report: M2: 2/3 green (gates once M2 is tagged)` after chunk C1.

## Row 1: the 26

`m2/vectors.py` runs plain refcore 2.1.7 (PyPy, run.py's `run_one`) on each vector of `m2/vectors.tsv` and stores the whole output and exit code in `m2/vectors.refcore`, in `kernel/vectors.refcore`'s form, using `kernel/vectors.py`'s own functions. Before it writes, each vector is tied to its stored baseline by `run.py`'s compare (a household check by its ask's blocks), so no expected byte is new: refcore 2.1.7 prints all 26 stored baselines (`run.py --interp refcore`, 2026-11-14). A household check is held as refcore's whole output of its one-ask run (`run <files> --ask <ask>`), as M1 holds X's.

The 26, with who holds each and the builder group that turns it green (`green`: green today):

| # | Id | Construct | Held by | For | Spec |
|---|---|---|---|---|---|
| 1 | `I:X:wife_net@c3_fast` | X change 3, the retroactive law: back pay as adjustments | `m2_checks` `checks-green` | green | L §5.14.3; E §5; L §6.1–§6.3 |
| 2 | `I:X:social_and_health@c3_fast` | the same | `m2_checks` `checks-green` | green | L §5.14.3; E §5 |
| 3 | `I:X:pit37@c3_fast` | the same, the return over the changed months | `m2_checks` `checks-green` | green | L §5.14.3; E §5; L §4.3 |
| 4 | `I:X:wife_net@v3_fast` | X change 3's variant | `m2_checks` `checks-green` | green | L §5.14.3; E §5 |
| 5 | `I:X:social_and_health@v3_fast` | the same | `m2_checks` `checks-green` | green | L §5.14.3; E §5 |
| 6 | `I:X:pit37@v3_fast` | the same | `m2_checks` `checks-green` | green | L §5.14.3; E §5; L §4.3 |
| 7 | `I:M:LH-05` | IKE/IKZE limits 2026 read `as known` | `s_kernel` (gating) | green | L §4.5, §4.6; E §2 |
| 8 | `I:M:LH-14` | WEP by eligibility year, `as known` reads | `m2_checks` `checks-green` | green | L §4.5; E §2; L §5.14.3 |
| 9 | `I:M:RC-08` | PL ulga 2022, enacted and removed: law dates and `as known` cuts | `m2_checks` `checks-green` | green | L §4.3, §4.5; E §2 |
| 10 | `I:M:UL-06` | NI 2022-23 by tax week and month, rates by payment date (#42's case) | `m2_checks` `checks-green` | green | L §4.3, §4.5 |
| 11 | `I:M:UL-09` | Box 3 2022 and 2025, the rechtsherstel comparison | `m2_checks` `checks-green` | green | L §4.3–§4.5, §5.11 |
| 12 | `I:M:UL-15` | PIT 2022, two employments: the return as amended mid-year | `m2_checks` `checks-plan` | plan | L §4.3, §5.11, §5.14.3 |
| 13 | `I:M:CH-04` | GDP vintages: one value as known at each release | `m2_checks` `checks-green` | green | E §2; L §4.5, §4.6 |
| 14 | `U:law/LAW-04` | edge LAW-04, retroactive law: advances keep their booked amounts | `m2_checks` `checks-green` | green | L §5.14.3; E §5 |
| 15 | `U:rec/REC-01` | edge REC-01, records are the past | `m2_checks` `checks-green` | green | L §5.19; E §6.1, §6.3 |
| 16 | `U:rec/REC-02` | edge REC-02, a statement with errors | `m2_checks` `checks-green` | green | L §5.19; E §6.1 |
| 17 | `U:rec/REC-03` | edge REC-03, awkward rows | `s_kernel` (gating) | green | L §5.19 |
| 18 | `U:rec/REC-05` | edge REC-05, records as the law's inputs | `m2_checks` `checks-green` | green | L §5.19; E §6.2 |
| 19 | `U:rec/REC-07` | edge REC-07, replaying over recorded history | `m2_checks` `checks-green` | green | E §6.1–§6.3 |
| 20 | `U:rule/RULE-06` | edge RULE-06, a rule changed "as if since January" | `m2_checks` `checks-green` | green | L §5.14.3; E §5, §6.1 |
| 21 | `U:rule/RULE-07` | edge RULE-07, the rule against the records: `bal … modelled`, `residuals` | `m2_checks` `checks-plan` | plan | L §5.19; E §6.2, §6.3 |
| 22 | `U:rule/RULE-08` | edge RULE-08, a parameter estimated from history | `m2_checks` `checks-plan` | plan | L §5.11; E §2 |
| 23 | `U:scen/SCEN-02/SCEN-02` | edge SCEN-02, a counterfactual past | `m2_checks` `checks-green` | green | L §6.4; E §2 |
| 24 | `U:set/scen/SCEN-02/SCEN-02` | the same, with `kept.tally` | `m2_checks` `checks-green` | green | L §6.4 |
| 25 | `U:surf/SURF-06` | edge SURF-06, a record row with a negative field | `s_kernel` (gating) | green | L §2.3, §5.19 |
| 26 | `U:val/VAL-05` | edge VAL-05, prices from records | `m2_checks` `checks-green` | green | L §5.8, §5.19 |

## What rulings hold back

None of row 1's vectors waits on a ruling: refcore and the stored baselines agree on all 26. The 15 rows `held.tsv` holds for M2's items (#147, #101, #158, #132) are row 6's; each is released by T once its builder group turns it green.

## Keeping master green

As at S, M0 and M1: every `m2_…` target has `test = false` and runs only as `check`'s report until M2 is in `engine/tools/milestones.toml`'s `gating`, which the rules step adds in `build/m2`'s last commit before its gate.
