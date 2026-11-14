# Milestone M2: what must turn green

The checks M2 (replay against reality) must turn green, from BUILD-ORDER § M2 ("Green (26)", "Properties", "Budgets touched") and DONE gates 1, 2, 3, 4, 6 and 8, with the items the run's brief puts in scope (issue #167). Written by T for issue #167 (2026-11-14) on refcore **tally 2.1.7**, from T's plan approved by the coordinator (picks P1–P5). The builder's groups are its plan's (`tmp/m2/build-plan.md`, approved with picks P6–P10): `forms` (G1), `explain` (G2), `windows` (G3), `records` (G4), `ledger` (G5), `sweep` (G6), `forks` (G7); the `for` column names them. The builder reads this page; T changes it.

## The list

"Today" is the engine at `build/m2` e80ff0b (master after tally 2.1.7 and PR #166), 2026-11-14. Every `m2_…` target has `test = false` and runs only as `check`'s report until M2's marker commit (as at M1), so T's commits stay green with no `Logged-Red:` line; each vector's builder group is the `for` column of `m2/vectors.tsv` and of the table below.

| # | What | Ids, vectors | Run by | Gate | For | Today |
|---|---|---|---|---|---|---|
| 1 | the 26 checks of § M2, whole output = refcore's | 3 already held by `s_kernel` (gating: `I:M:LH-05`, `U:rec/REC-03`, `U:surf/SURF-06`); the other 23 in `m2/vectors.refcore` (chunk C1) | `cargo nextest run -p tally-runner --test m2_checks` (4 tests: `checks-green`, `checks-forms`, `checks-records`, one that every group has its test); `m2/vectors.py --check` in `run.py --self-test` (8m) | 1, 3 | forms, records | 23 of 26 green (3 in `s_kernel`, 20 in `checks-green`); 3 red, each `tally run: not implemented` (exit 69): `U:rule/RULE-08`, `I:M:UL-15` (forms), `U:rule/RULE-07` (records) |
| 2 | goldens G4 and G5, and the halves of G1, G2, G3, G6 and G10 that read the prototype's bank history, with its 20 pins, on that ledger tracked under `engine/tests/` (#114) | chunk C4 | `m2_goldens` | 2 | plan | not written |
| 3 | the hindcast fixtures H1–H5, H8, H9, each landing in the ledger line its answer key names | chunk C5 | `m2_hindcasts` | 4 | plan | not written |
| 4 | the codes of M2's constructs no output prints, one fixture each: `HOLE-SAME`'s disagreement form and four `RUN-RECORD` forms | chunk C6 | `m2_codes` | 5 | plan | not written |
| 5 | the properties of § M2: masking at K, the three-way split, the fixed past, forks on and off | chunks C7, C8 | `m2_properties`, `m2_forks` | 6 | plan | not written |
| 6 | the brief's in-scope items: #147, #101, #158, #132, #42, #163, #164, item 9's pair; P6's known windows and `moves` at run; P47's kill texts | chunk C2 (§ Row 6): 9 texts under `m2/items/` in `m2/vectors.refcore`; the 7 kill texts in `m1/fixes/gate8/p47_*` (gating `m1_fixes`); 15 rows held in `held.tsv` for #147 (9), #101 (4), #158 (1), #132 (1), all `m1_fixes` | `m2_items` (5 tests: `items-green`, `items-forms`, `items-explain`, `items-windows`, one that every group has its test); `m1_fixes` `fixes-gate8` | 1, 8 | forms, explain, windows | 3 green on arrival (`items-green`), 6 red (not implemented): 1 `forms`, 3 `explain`, 2 `windows`; the 7 kill texts green; the 15 held rows red on the engine |
| 7 | item 10: the 63 over-specified `fixes/g12/*` vectors restated to their asks | chunk C3: `m1/restate_g12.py` (`--check` in `run.py --self-test`, 8n), 582 asks; `MILESTONE-M1.md` § Item 10 | `m1_fixes`, `m1_lows` (gating) | 1 | — | done: `m1/vectors.refcore` 318 591 lines, 11 281 620 bytes before; 229 737 lines, 7 575 415 bytes after (the 63's output 95 744 → 6 890 lines); each prints its asks as its whole run did |
| 8 | the builder groups' review fixes and gate 8's survivors | chunk C9 on | `m2_fixes` | 1, 8 | each group | — |

`check`'s report line counts the `m2_…` targets' tests: `milestone-report: M2: 2/3 green (gates once M2 is tagged)` after chunk C1, `4/9` after chunk C2.

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
| 12 | `I:M:UL-15` | PIT 2022, two employments: the return as amended mid-year | `m2_checks` `checks-forms` | forms | L §4.3, §5.11, §5.14.3 |
| 13 | `I:M:CH-04` | GDP vintages: one value as known at each release | `m2_checks` `checks-green` | green | E §2; L §4.5, §4.6 |
| 14 | `U:law/LAW-04` | edge LAW-04, retroactive law: advances keep their booked amounts | `m2_checks` `checks-green` | green | L §5.14.3; E §5 |
| 15 | `U:rec/REC-01` | edge REC-01, records are the past | `m2_checks` `checks-green` | green | L §5.19; E §6.1, §6.3 |
| 16 | `U:rec/REC-02` | edge REC-02, a statement with errors | `m2_checks` `checks-green` | green | L §5.19; E §6.1 |
| 17 | `U:rec/REC-03` | edge REC-03, awkward rows | `s_kernel` (gating) | green | L §5.19 |
| 18 | `U:rec/REC-05` | edge REC-05, records as the law's inputs | `m2_checks` `checks-green` | green | L §5.19; E §6.2 |
| 19 | `U:rec/REC-07` | edge REC-07, replaying over recorded history | `m2_checks` `checks-green` | green | E §6.1–§6.3 |
| 20 | `U:rule/RULE-06` | edge RULE-06, a rule changed "as if since January" | `m2_checks` `checks-green` | green | L §5.14.3; E §5, §6.1 |
| 21 | `U:rule/RULE-07` | edge RULE-07, the rule against the records: `bal … modelled`, `residuals` | `m2_checks` `checks-records` | records | L §5.19; E §6.2, §6.3 |
| 22 | `U:rule/RULE-08` | edge RULE-08, a parameter estimated from history | `m2_checks` `checks-forms` | forms | L §5.11; E §2 |
| 23 | `U:scen/SCEN-02/SCEN-02` | edge SCEN-02, a counterfactual past | `m2_checks` `checks-green` | green | L §6.4; E §2 |
| 24 | `U:set/scen/SCEN-02/SCEN-02` | the same, with `kept.tally` | `m2_checks` `checks-green` | green | L §6.4 |
| 25 | `U:surf/SURF-06` | edge SURF-06, a record row with a negative field | `s_kernel` (gating) | green | L §2.3, §5.19 |
| 26 | `U:val/VAL-05` | edge VAL-05, prices from records | `m2_checks` `checks-green` | green | L §5.8, §5.19 |

## Row 6: the in-scope items (chunk C2)

Each text is T's, under `m2/items/`, its note naming the item and the forms; each vector holds refcore 2.1.7's output of the asks named (all of them where `asks` is empty), read against the spec before it was stored.

| Vector | Item, forms | Group | For | Today |
|---|---|---|---|---|
| `items/law09/each_year` | #42: LAW-09's `each_year`, `for y in [2021, 2022, 2023]` over a year index, alone (L §5.18) | `items-green` | — | green |
| `items/r8/reach2` | #147, R8's reach (L §5.5): `solve` whose F reads an axis, `inputs-of` across it, `bal` and its register per combination, each with a twin pinned to one option; `residuals` pinned | `items-green` | — | green |
| `items/shift/hole` | item 9 (P5): `add_business` over a calendar governed in 2026 only, landing inside it (`h_in`) and past it (`h_in2`, the hole decides); past the day range over a calendar governed always (`RUN-LIMIT`) | `items-green` | — | green |
| `items/s2/forms` | #101: an unkeyed record input's field as the price; a field of `last(…)`, keyed and unkeyed, past its reading and before the first; `value … by` a keyed field on a reading's day and after it (L §2.3, §4.6, §5.10) | `items-forms` | forms | red |
| `items/explain/m` | #132: `readers-of` a kind and its subkind; `explain bal` whose last reading pads, anchors, or has a hole movement since it (E §6.3); #163/#164: `last` of a reading `after m` before its day, of an `at D end` reading past it and on it; `explain` of a def bound by the walk | `items-explain` | explain | red |
| `items/explain/any` | #163: a place of unit `any` with an opening row per unit, `bal(x, u, d)` and its `explain` | `items-explain` | explain | red |
| `items/explain/from` | #164 R7, #163: `before from:` under `flows` and `bal`, for a leg dated before `from` with its law date after it, and a leg whose day is a window, under `windows each` (L §6.4, §4.4) | `items-explain` | explain | red |
| `items/windows/m` | #158, #164 R6: `windows each` on an ask over `add_business`, over a balance whose move's `at` is a window, under an axis, under a pin, and on `low … from` (L §4.4) | `items-windows` | windows | red |
| `items/windows/known` | P6: a `known` window at run on a version, a row and an observation's version, plain and `windows each`; a `moves` line at run, the moved version read in its old and new span as known before and after it (L §4.5, §5.3) | `items-windows` | windows | red |
| `fixes/gate8/p47_k1` … `p47_k9` (7) | gate 8 run 43582832630's survivors (P47): the kill texts k1, k2, k4, k5, k6, k7, k9, each its whole run (in `m1/vectors.tsv`, group `fixes-gate8`, gating) | `m1_fixes` | — | green |

Not written in C2, for the next chunk: #164's month-stepped range from the 29th–31st, S5 through four or more decisions and a cycle left after composing, R1 `argmin` with a hole `ties by` term, `low … random at mean`, `next(K, d) with uncertain`; and the forms refcore crashes on (`m2/spec.tsv`, as M1's): `residuals` over a place whose plan reads an axis (refcore 2.1.7: `REFCORE-NOT-YET … an interpreter guard`), `pay law w`, `as known w`.

## What rulings hold back

None of row 1's vectors waits on a ruling: refcore and the stored baselines agree on all 26. The 15 rows `held.tsv` holds for M2's items (#147, #101, #158, #132) are row 6's; each is released by T once its builder group turns it green.

Three forms of row 6 are stored in no file, since refcore 2.1.7's answer differs from the spec's and nothing may hold refcore's bytes as expected (refcore cases for 2.1.8, each awaiting the coordinator's issue):

- **Item 9's pair** (P5; R2, L §8.1): `add_business(2026-06-01, -739768, cal_h)` and `add_business(2026-06-01, 2912292, cal_h)`, `cal_h` governed in 2026 only. n business days move at least n days, so both pass the day range whatever the hole on the way; the spec's answer is `refused: RUN-LIMIT a day comes before 0001-01-01` (`… passes 9999-12-31`), as over a calendar governed always. refcore prints the hole, `?"calendar cal_h doesn't cover 2025" (cal_h)` (`… 2027`). The text is `m2/items/shift/hole.tally` (`h_back`, `h_fwd`).
- **`last` of a reading `after m`** (#163, #164; L §4.6): `last(stated[konto], d)` on or after the day of a reading `stated[konto] = 10.00 PLN after save[2026-03]`: L §4.6 takes the last reading on or before d, so 10.00 PLN (its `held` mark is P31's question); refcore prints `?"stated[konto] has no reading on or before …"`. The text is `m2/items/explain/m.tally` with an ask `last(stated[konto], 2026-05-01)`.
- **`low … windows each` without `from`** (#164 R6; L §4.4): on an ask line the clause splits every window the line meets; refcore prints the holes of the plain form, unsplit. The text is `m2/items/windows/m.tally` with an ask `low(konto, 2026-03-01 .. 2026-04-30) windows each`.

## Keeping master green

As at S, M0 and M1: every `m2_…` target has `test = false` and runs only as `check`'s report until M2 is in `engine/tools/milestones.toml`'s `gating`, which the rules step adds in `build/m2`'s last commit before its gate.
