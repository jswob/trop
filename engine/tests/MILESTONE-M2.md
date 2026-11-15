# Milestone M2: what must turn green

The checks M2 (replay against reality) must turn green, from BUILD-ORDER § M2 ("Green (26)", "Properties", "Budgets touched") and DONE gates 1, 2, 3, 4, 6 and 8, with the items the run's brief puts in scope (issue #167). Written by T for issue #167 (2026-11-14) on refcore **tally 2.1.7**, from T's plan approved by the coordinator (picks P1–P5). The builder's groups are its plan's (`tmp/m2/build-plan.md`, approved with picks P6–P10): `forms` (G1), `explain` (G2), `windows` (G3), `records` (G4), `ledger` (G5), `sweep` (G6), `forks` (G7); the `for` column names them. The builder reads this page; T changes it.

## The list

"Today" is the engine at `build/m2` e80ff0b (master after tally 2.1.7 and PR #166), 2026-11-14. Every `m2_…` target has `test = false` and runs only as `check`'s report until M2's marker commit (as at M1), so T's commits stay green with no `Logged-Red:` line; each vector's builder group is the `for` column of `m2/vectors.tsv` and of the table below.

| # | What | Ids, vectors | Run by | Gate | For | Today |
|---|---|---|---|---|---|---|
| 1 | the 26 checks of § M2, whole output = refcore's | 3 already held by `s_kernel` (gating: `I:M:LH-05`, `U:rec/REC-03`, `U:surf/SURF-06`); the other 23 in `m2/vectors.refcore` (chunk C1) | `cargo nextest run -p tally-runner --test m2_checks` (4 tests: `checks-green`, `checks-forms`, `checks-records`, one that every group has its test); `m2/vectors.py --check` in `run.py --self-test` (8m) | 1, 3 | forms, records | 23 of 26 green (3 in `s_kernel`, 20 in `checks-green`); 3 red, each `tally run: not implemented` (exit 69): `U:rule/RULE-08`, `I:M:UL-15` (forms), `U:rule/RULE-07` (records) |
| 2 | goldens G4 and G5, and the halves of G1, G2, G3, G6 and G10 that read the prototype's bank history, with its 20 pins, on that ledger tracked under `engine/tests/` (#114) | chunk C4: `m2/bank/` (the ledger, `make-ledger.ts`, `to_tally.py`'s 34 record parts, 1 940 rows, links by P1; R7's bank half, `bank.tally`, the bank run's header `reality.tally`; G1's twins `marek.tally`, `marek_out.tally`; R7's forward half `r7/`: the 15th rule, IKE first, OKI, the business, the returns and their joint twin, a header per horizon); 12 vectors `goldens/bank_*` (G1, G2, G4, G5), `goldens/r7_*` (G3, 13 of the 20 pins, the joint twins) | `m2_goldens` (3 tests: `goldens-records`, `goldens-ledger`, one that every group has its test); `m2/bank/to_tally.py --check` in `run.py --self-test` (8p) | 2 | records, ledger | G1–G5 hold on refcore (`GOLDENS.md`), and 10 of the 20 pins; 3 pins moved by L-26, their joint twins holding; the engine prints refcore's bytes on `goldens-records` (G1, G2, G3, G5, the 13 pins) and stops `not implemented` on `goldens-ledger` (G4, ledger); the horizons 2036, 2050 and 2056 (G6's and G10's net worth, 7 pins) pass refcore's 8 GB cap and wait for the user's ruling (P2) |
| 3 | the hindcast fixtures H1–H5, H8, H9, each landing in the ledger line its answer key names | chunk C5 | `m2_hindcasts` | 4 | plan | not written |
| 4 | the codes of M2's constructs no output prints, one fixture each: `HOLE-SAME`'s disagreement form and four `RUN-RECORD` forms | chunk C6: `codes/m2.py`, the family `records` (`codes/m2/records/base.tally`), `codes/m2/edits.tsv` 4 fixtures (`same-differs` HOLE-SAME with its `WARN-LEGS`; `same-missing`, `unjoined`, `adjustment-unmade` RUN-RECORD), `codes/m2/held.tsv` 1 (`reading-after`, #169 P11: refcore prints no refusal); `codes/m2.refcore` 9 vectors (1 base, 4 answers, 4 fixtures) | `m2_codes` (3 tests: `HOLE-SAME`, `RUN-RECORD`, one that every code has its test); `codes/m2.py --check` in `run.py --self-test` (8k2); `tools/xref_check.py --gate M2`: 0 of 91 codes built by M2 without an output | 5 | records | the 4 fixtures on refcore 2.1.7 with P35's patch; the engine's run is G4's |
| 5 | the properties of § M2: masking at K, the three-way split, the fixed past, forks on and off | chunks C7, C8 | `m2_properties`, `m2_forks` | 6 | plan | not written |
| 6 | the brief's in-scope items: #147, #101, #158, #132, #42, #163, #164, item 9's pair; P6's known windows and `moves` at run; P47's kill texts | chunk C2 (§ Row 6): 9 texts under `m2/items/` in `m2/vectors.refcore`; the 7 kill texts in `m1/fixes/gate8/p47_*` (gating `m1_fixes`); the 15 rows `held.tsv` held for #147 (9), #101 (4), #158 (1), #132 (1) un-held once G1–G3 turned them (C9) | `m2_items` (6 tests: `items-green`, `items-forms`, `items-explain`, `items-windows`, `items-sweep`, one that every group has its test); `m1_fixes` `fixes-gate8` | 1, 8 | forms, explain, windows | 3 green on arrival (`items-green`), 6 red (not implemented): 1 `forms`, 3 `explain`, 2 `windows`; the 7 kill texts green; the 15 held rows red on the engine |
| 7 | item 10: the 63 over-specified `fixes/g12/*` vectors restated to their asks | chunk C3: `m1/restate_g12.py` (`--check` in `run.py --self-test`, 8n), 582 asks; `MILESTONE-M1.md` § Item 10 | `m1_fixes`, `m1_lows` (gating) | 1 | — | done: `m1/vectors.refcore` 318 591 lines, 11 281 620 bytes before; 229 737 lines, 7 575 415 bytes after (the 63's output 95 744 → 6 890 lines); each prints its asks as its whole run did |
| 8 | the builder groups' review fixes, refcore 2.1.7's misses (#169, #173) and gate 8's survivors | chunk C9 (§ Row 8): the probes' texts under `m2/fixes/`; 38 vectors with refcore's bytes held in `held.tsv` (`m2_fixes`; #169 36, #173 2), `g14/past_64_counts` held in `s_kernel` and `records/answers/q_konto`, `records/adjustment-unmade` in `m2_codes` (#169); `m2/spec.tsv` 41 rows, the spec's answers; 122 kill vectors `fixes/gate8/m2_*` (gating); `fixes/g4/adj_amt` for G5 (P51) | `m2_fixes` (11 tests: 6 groups, 4 spec groups, one that every group has its test); `m1_fixes` `fixes-gate8`; `m2/spec.py --check` | 1, 8 | each group | the spec rows green (the engine's answers, read against their sections); the 122 kill vectors green |

`check`'s report line counts the `m2_…` targets' tests: `milestone-report: M2: 2/3 green (gates once M2 is tagged)` after chunk C1, `4/9` after chunk C2, `14/17` after C9's holds and kills.

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

#164's other forms (C2's rest, `m2/items/sweep/`), each where its form gates: a month-stepped range from the 29th, 30th and 31st as a binding and an input's key (`b1/m2_164_month_ends`, `m1_fixes` `fixes-b`), R1 `argmin` whose `ties by` term is a hole on a candidate not tied and on a tied one (`r1/m2_164_argmin_ties_hole`, `fixes-r1`), S5 through four decisions and a cycle left after composing (`s5/m2_164_order_four`, `s5/m2_164_cycle_left`, `fixes-s`), all green and gating; `next(K, d)` over an uncertain kind, without and with `with uncertain` (`items/sweep/next_uncertain`, `items-sweep`, red for `sweep`: the engine stops `with uncertain` not implemented). `low … random at mean` is M7's. `pay law w` and `as known w` (a window as an ask's law date or cut) are not written: for C6.

## Row 8: review fixes and refcore's misses (chunk C9)

From the builders' review probes (`tmp/m2/b/`, the BUILD-LOG addenda of G1–G3), each ask run on refcore 2.1.7 and on the engine at 913f1088 and sorted by machine:

- **Same on both** (74 vectors, 33 from the probes the reviews name and 41 from the builders' other probe folders, `m1/vectors.tsv` `fixes/gate8/m2_*`, gating now in `m1_fixes` `fixes-gate8`): the asks gate 8's survivors in G1's and G2's code name (`read/bands.rs` `band_rows`, `chain`: `m2_g1_bands`; `walk/value.rs` `clause_keys`, `price_field`: `m2_g1_px`, `m2_g1_law`; `book/members.rs` `movements_of`, `walk/plan.rs` `traced_instances`: `m2_g2_sum`, `m2_g2_sum2`) and every other probe ask the two engines answer alike (G1's `solve` bracket NB-1 `m2_g1_sv`, bands, quantiles and records; G2's walk-bound explains `m2_g2_wb*`; G3's `windows each` forms).
- **Refcore's misses, held** (`held.tsv`, target `m2_fixes`, refcore's bytes kept): #169 (picks P5, P11, P15–P17, P21, P24, P27, P30; G1's R8 questions 1–2, `fixes/g1/legf*`) and #173 (P22); and `g14/past_64_counts` (`s_kernel`, P15). `m2/spec.tsv` gives the spec's answer beside each the engine already follows (the engine's run, read against the row's section).
- **Refcore crashes** (`m2/spec.tsv` only): a quantile whose level is `none` (P17 B-1), `residuals` over an axis (P14), a read's own `windows each` under another ask (`k2`, P27 B5), `windows each` past 20 000 day combinations (P28, #174 interim).
- **Held, the engine wrong too** (no spec row; `for` names the group): `fixes/g1/y4` (P16, LOAD-LITERAL, `sweep`), `fixes/g1/edge_hole` and `g14/past_64_counts` (P5, P15, `sweep`), `fixes/g2/after_m` (P11, P12, `records`).
- **Not stored:** `i2`'s `qp` (NB-4, the `(cal)` source tag) and `anyhole`'s `l1` (NB-4's held text): on the follow-up list, unruled. P20's probe (`g2/any/a` `e2`) answers alike on both engines now, so it gates with the others.

**Gate 8 run 43657284122** (89.4 %, the first to count M2's tests; `tmp/m2/gate8-next-survivors.txt`): texts in `m2/fixes/gate8/` (`at_holes`, `window_moves`, `at_window`, `at_ties`, `explain_dated`) and G3's last fix's probes `m2/fixes/g3/fix2_*`, each gating in `fixes-gate8` where both engines answer alike: a def whose window `at` reads unlike holes (`hole/questions.rs`), movements dated by windows at each edge of a span through `flows`, a binding, the residual ledger and the place's flows (`window_in`, `window_member`, `model_day`, `place_log`), moves whose window `at` reads rows that agree or not (`plan.rs`, `tuples.rs`, `order.rs` `ties_differ`). Held for #169 with spec rows: G3's last fix's `r2` `q`, `c2` `qc`, `d` `qd`, `ql`; and `at_window_mk2` `f_mk2` (P40, L §4.4): a move binding whose members differ across its window `at` is the window's hole, where refcore flows 1.00. Red for `explain`: `items/explain/walk_bound_dated`, where the engine leaves out a dated `bal` from a walk-bound derivation that refcore prints.

**G4 `records`** (its BUILD-LOG entry and two fix addenda; the review's and the builder's probes `tmp/m2/g4fix2/`, copied to `m2/fixes/g4/`; refcore with P42, P46 and P50's patches, the engine at 33abaf56): 37 kill vectors `fixes/gate8/m2_g4_*` (the `same` paths: groups, the latest member, `HOLE-SAME` through a binding and `observed`, the settlement once, `holed_at`, `left_to_partner`, `share_fulfils`, `p_rev_onefulfils` (P50); the unjoined refusal `s_unjoined`; `observed(m[k])`; bindings over pads `p_pad`; `walk/modelled.rs` `m_mod`, `m_one`; a field read of `none` `a_unc`; `nb3`'s load refusal), each the asks both engines answer alike. Held for #169 (P43, refcore drops the reading after m): `fixes/g4/{a_cov, a_mp, a_none, b1_since, bf_b1_more, b2_last, b4_cycle, b4own, b4own_cyc, c_since}` in `fixes-records`, with `spec/g4/*` rows (`spec-records`); `records/answers/q_konto` and `records/adjustment-unmade` in `m2_codes`, which now leaves a held fixture out. Refcore crashes: `spec/g4/nb3_{after, afterT, at, atT}` (L §5.14.2, a link's `when` that cycles is `RUN-CYCLE`). For G5: `fixes/g4/adj_amt` (`fixes-ledger`), `HOLE-SAME` naming its rows in the `same` line's order (P51, #177).

## What rulings hold back

None of row 1's vectors waits on a ruling: refcore and the stored baselines agree on all 26. The 15 rows `held.tsv` holds for M2's items (#147, #101, #158, #132) are row 6's; each is released by T once its builder group turns it green.

Row 6's three forms where refcore 2.1.7 differs from the spec (item 9's pair past the edge, `last` of an `after m` reading, `low … windows each` without `from`) are held for #169 (P11), with row 8's.

## Keeping master green

As at S, M0 and M1: every `m2_…` target has `test = false` and runs only as `check`'s report until M2 is in `engine/tools/milestones.toml`'s `gating`, which the rules step adds in `build/m2`'s last commit before its gate.
