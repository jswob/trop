# Milestone M1: what must turn green

The checks M1 (the household forecast) must turn green, from BUILD-ORDER § M1 ("Green (90)", "Properties", "Budgets touched") and DONE gates 1, 2, 3, 5 and 6, with the items the pre-M1 run left for M1 (`tmp/active/m1-brief.md` §1). Written by T for issue #108 (2026-11-08) on refcore **tally 2.1.6**, from T's plan approved by the coordinator (pick P14; its questions' picks P3–P13). The builder's groups G1–G14 are `tmp/m1/build-plan.md` §3's (picks P15–P20). The builder reads this page; T changes it.

## The list

"Today" is the engine at `build/m1` d508eae (master ccea2ed and the rules step), 2026-11-08. Every `m1_…` target has `test = false` and runs only as `check`'s report until M1's marker commit (pick P4), so T's commits stay green with no `Logged-Red:` line; each vector's builder group is the `for` column of `m1/vectors.tsv` and of the table below.

| # | What | Ids, vectors | Run by | Gate | For | Today |
|---|---|---|---|---|---|---|
| 1 | the 90 checks of § M1, whole output = refcore's | 26 already held by `s_kernel` (gating; pick P5); the other 64 and #36's `inputs_of_balance` in `m1/vectors.refcore` (65 vectors) | `cargo nextest run -p tally-runner --test m1_checks` (12 tests: one per builder group, one for the checks green before M1, one that every group has its test); `m1/vectors.py --check` in `run.py --self-test` (8j) | 1, 3 | G1–G8, G10, G11 | 43 of 90 green (26 in `s_kernel`, 17 in `checks-green`); 47 red |
| 2 | goldens G1, G2, G3, G6–G10 and the 36 pins on tally 2's canonical form | `GOLDENS.md` records each against the prototype's figure; chunk C2: 4 vectors in `m1/vectors.refcore` (groups `goldens-…`); chunk C3: 5 more, the 2022 pins (`goldens-y22`, `m1/goldens/y22/`), the side business's (`goldens-business`), G8's warning (`goldens-g8`) and G9 (`goldens-g9`, 2); 14 pins hold, 2 move by L-26; 20 pins, R7 whole, G1, G2, G3, G8's figure and the net-worth halves of G6 and G10 blocked (the prototype's bank history isn't in the repository; P21, #114) | `cargo nextest run -p tally-runner --test m1_goldens` (9 tests: one per golden's group, one that every group has its test) | 2 | G1 | 9 of 9 green after G1 (the four of chunk C3 green before their commit) |
| 3 | the hole and run codes of M1's constructs, one fixture each, and the forms of M1's constructs no output prints (pick P9) | chunk C4: `codes/m1.refcore` (`codes/m1.py`, `codes/m0.py` with the store m1): 3 families (`rules`, `prices`, `decide`), each one's base and 11 answers (one per ask, clean), 18 fixtures of 16 codes: the 12 codes' 11 that refcore prints (`HOLE-STATED`, `HOLE-NOTYET`, `HOLE-PRICE` ×2, `RUN-CONFLICT`, `RUN-EMPTY`, `RUN-ZERO`, `RUN-WEIGHTS`, `RUN-PRORATA`, `RUN-ORDER`, `RUN-SOLVE`), 5 forms (`HOLE-ANCHOR` no reading, `RUN-NONE`, `RUN-KEY` outside a def's days, `RUN-PLACE` a debt above 0, `RUN-FUTURE` binds through), #90's two (`LOAD-FIELD`); `ENGINE-INVARIANT` in `codes/m1_invariant.refcore`, refcore under the planted fault (`codes/m1_fault.py`, `INTERFACE.md` §4); `REQ-CLOCK` held (`codes/m1/held.tsv`; GAPS.md, refcore answers it); `tools/xref_check.py --gate M1` red on `REQ-CLOCK` alone, 26 forms of codes built by M1 unprinted (most M2's or M5's: parts, records, fills) | `cargo nextest run -p tally-runner --test m1_codes` (17 tests: one per code, one that every code has its test); `codes/m1.py --check` in `run.py --self-test` (8k); `m1_invariant` (the rules step's, P8) | 5 | G1, G3–G5, G8, G10 | 9 of 17 green: 8 codes green before M1 (`HOLE-ANCHOR`, `HOLE-STATED`, `RUN-CONFLICT`, `RUN-EMPTY`, `RUN-KEY`, `RUN-NONE`, `RUN-PLACE`, `RUN-ZERO`); red: `LOAD-FIELD` (G1), `HOLE-NOTYET` (G3), `RUN-PRORATA`, `RUN-WEIGHTS` (G4), `RUN-SOLVE` (G5), `HOLE-PRICE` (G8: `value`), `RUN-ORDER`, `RUN-FUTURE` (G10) |
| 4 | the properties of § M1 and #98's "a key moved by whole steps lands on a key" (pick P10) | `m1_properties` (chunk C6) | | 6 | G1–G14 | not written |
| 5 | the budgets P8-2 (1×), P8-5 with P8-M4 and F5, P8-9 | P8-2's 1× twin is `I:X:pit37@c7_fast`, the slowest M1 check of X on refcore (1.40 s; pick P11); its digest goes into `perf-digests.tsv` when the rules step points P8-2's recipe at it | `engine/tools/perf row P8-2`; the gate's timing run | 12 | G9 (P8-5) | P8-2's recipe still `waits:` |
| 6 | the items pre-M1 left for M1 (brief §1 1–8) | chunk C5: 45 vectors in `m1/vectors.refcore` (groups `carried-<item>`, fixtures under `m1/carried/`): item 1, scenarios as layers (6, `layers/`); item 2, reads at run (6, `reads/`: a kind-keyed input, a range's key no row states, a row for every key, money and fraction ranges); item 3, ranges an ask steps through (4, `ranges/`); item 5, P6/P12's parenthesised literal (4, `years/`: refcore refuses six of the nine places, held out in `m1/carried/held.tsv`, GAPS.md); item 6, N1 (2), N4 (3), N7 (6: `weekday`, `last_business`, an `added` window both ways, `explain` of a balance a movement moves), B1 (1, a 100 000-key list, `big/`); item 4, the 26 sites (12, `sites/`: a base and a probe per cause class, all 11 refused at load by refcore, bin i); item 7, a plan whose day is a hole (1, `walk/`). Not yet: item 2's G4a deviation (a `fulfils` and a `same` side off a range's values), item 7's other three notes, item 8's listing | `cargo nextest run -p tally-runner --test m1_carried` (8 tests: one per item, one that every group has its test) | 1, 5 | G1–G4, G6, G10, G12, G13 | 2 of 8 green (item 7 and the group check); 19 of the 45 vectors green; red: G1 (5: the base and `(2026)` compared, `2026 + 1 year`, `[2026, 2027]`, a missing field named by its kind), G2 (4), G3 (3), G6 (4), G12 (2), G13 (8) |
| 7 | #109: a canonical text with rows keyed by two literal kinds reprints as itself | `m1/format/key_kinds.tally` (minimised from the format fuzz target's find) | `cargo nextest run -p tally-runner --test m1_format` (1 test) | 6 | G1 | red: the engine's reprint of its own output moves the integer-keyed row |
| 8 | the builder groups' review fixes, one vector per fixed finding and per mutant `mutants-local` left for T (the engine-coordinator skill: "T then turns each fix into tests") | chunk C7: 66 vectors in `m1/vectors.refcore` (groups `fixes-g1`, `fixes-g2`, `fixes-g3`, fixtures under `m1/fixes/g<n>/`), `for` the group whose fix each holds. G1 (17): the narrowed conjunction in an `if` (5), the rule-`when` form and a def keyed by a movement's binding (5), `unreplaced`'s survivors (5: a later replace, a wider replace beats only its own keys, a scenario's replace known before its base's rows, a base's replace below a scenario's), and the two load refusals that leave a survivor no input at run (`LOAD-OVERLAP` two rows of one layer known one day, `rows.rs` `>=`; `LOAD-LAWDATE`, `rows.rs:363`). G2 (23): a window's identity (B1's four, `paid == paid`, the fix review's F1 and F2 forms, `w + 1 day == w + 1 day`), 9999-12-31 (3), `pow` (7: `pow(0, -1)`, `pow(0, 0)`, `pow(0, 2)`, `pow(2, -1)`, `pow(1.5, -2)`, `pow(10, 3000)`, `pow(10, 5000)`). G3 (26): B1–B5, `since`, a `last` at an instant (`rows.rs:356`), the fix review's F1 and F2. 21 held "for #n" in `held.tsv` (refcore's bytes, the spec differs): #115 13, #116 3, #117 2, #118 3 | `cargo nextest run -p tally-runner --test m1_fixes` (4 tests: one per group, one that every group has its test) | 1 | G1–G3 | 4 of 4 green; the 45 vectors not held are green |

`check`'s report line counts the `m1_…` targets' tests: `milestone-report: M1: 2/13 green (gates once M1 is tagged)` after chunk C1, `M1: 5/18 green` after chunk C2, `M1: 14/35 green` after chunk C4, `M1: 16/41 green` after chunk C5's first part, `M1: 16/43 green` after its second, `M1: 25/43 green` after G1–G3, `M1: 29/47 green` after chunk C7's first part, `M1: 33/51 green` after chunk C3.

## Row 1: the 90

`m1/vectors.py` runs plain refcore 2.1.6 (PyPy, run.py's `run_one`) on each vector of `m1/vectors.tsv` and stores the whole output and exit code in `m1/vectors.refcore`, in `kernel/vectors.refcore`'s form, using `kernel/vectors.py`'s own functions. Before it writes, each vector that is a check of `checks.tsv` with a stored baseline is tied to it by `run.py`'s compare (a household check by its ask's blocks), so no expected byte is new: refcore 2.1.6 prints all 89 stored baselines. `U:time/TIME-08-before` has no baseline (`no key by design (TC-5)`), so refcore's run is its expected output. A household check is held as refcore's whole output of its one-ask run (`run <files> --ask <ask>`), as `s_kernel` holds `I:X:konto_balance@b_r1`.

What the reds are (the builder's plan §1.1 has each one's modules):
- **G1:** X's `pit37` binds `y in [2026, 2027, 2028]` over the year index `tax_year`; the engine prints `RUN-KEY 2 026 is outside tax_year`. `LIFE-01`'s `year_of(m) == 2025`: `LOAD-UNIT 2024 and 2 025 compared`. `min_m2`'s scenario replaces a lifespan on its base version's own `known` day: the engine refuses `LOAD-REPLACE … no earlier row to replace`, where L §6.4 replaces it "whatever the two rows' `known` dates" (refcore follows it; `min_m2b`, a day earlier, is green).
- **G2–G11:** "not implemented" today (38 checks), each at the construct its group builds.

The 90, with who holds each and the builder group that turns it green (`green`: green today):

| # | Id | Edge or construct | Held by | For | Spec |
|---|---|---|---|---|---|
| 1 | `U:ask/ASK-01` | edge ASK-01 | `m1_checks` `checks-g3` | G3 | L §5.14.1 `low`/`next` in an ask, §5.18 |
| 2 | `U:ask/ASK-03` | edge ASK-03 | `s_kernel` (gating) | green | L §8.1, §8.4 `RUN-PLACE`, §5.18 partial answers |
| 3 | `U:ask/ASK-04` | edge ASK-04 | `m1_checks` `checks-g5` | G5 | L §5.18 `solve` in an ask, differences |
| 4 | `U:ask/ASK-05` | edge ASK-05 | `m1_checks` `checks-g11` | G11 | L §5.18 `explain`, E §10 |
| 5 | `U:flow/FLOW-01` | edge FLOW-01 | `m1_checks` `checks-g8` | G8 | L §5.19 readings, E §6.1 anchors |
| 6 | `U:flow/FLOW-02` | edge FLOW-02 | `m1_checks` `checks-g8` | G8 | L §5.14 legs, §5.13 `prorata` |
| 7 | `U:law/LAW-01` | edge LAW-01 | `s_kernel` (gating) | green | L §5.8 `band` |
| 8 | `U:law/LAW-03` | edge LAW-03 | `m1_checks` `checks-g5` | G5 | L §5.11 rules, `as if`, §4.5 |
| 9 | `U:law/LAW-05` | edge LAW-05 | `m1_checks` `checks-g3` | G3 | L §5.11 `as if`, the lower kept |
| 10 | `U:law/LAW-06` | edge LAW-06 | `m1_checks` `checks-g3` | G3 | L §5.5 `axis`, §8.1 a hole |
| 11 | `U:law/LAW-08` | edge LAW-08 | `s_kernel` (gating) | green | L §5.11 rules in order, §5.9 running totals |
| 12 | `U:law/LAW-13/LAW-13` | edge LAW-13 | `m1_checks` `checks-green` | green | L §6.4 a scenario from a date |
| 13 | `U:life/LIFE-01` | edge LIFE-01 | `m1_checks` `checks-g1` | G1 | L §4.5 versions, §5.11; P6 a year literal |
| 14 | `U:life/LIFE-02` | edge LIFE-02 | `s_kernel` (gating) | green | L §3.5, §5.7 lifespans |
| 15 | `U:life/LIFE-03` | edge LIFE-03 | `m1_checks` `checks-g2` | G2 | L §5.9 recurrences, §7.2 `round` |
| 16 | `U:life/LIFE-04` | edge LIFE-04 | `s_kernel` (gating) | green | L §4.5, §4.6 the read rule |
| 17 | `U:life/LIFE-05` | edge LIFE-05 | `s_kernel` (gating) | green | L §5.11, §4.2, §5.9 |
| 18 | `U:num/NUM-01` | edge NUM-01 | `m1_checks` `checks-g2` | G2 | L §3.2, §3.3 units, rates |
| 19 | `U:num/NUM-02` | edge NUM-02 | `m1_checks` `checks-g2` | G2 | L §4.2 day counts, §5.5 readings |
| 20 | `U:num/NUM-03` | edge NUM-03 | `s_kernel` (gating) | green | L §7.2 `round` |
| 21 | `U:num/NUM-04` | edge NUM-04 | `m1_checks` `checks-g7` | G7 | L §5.5 `axis`, each reading |
| 22 | `U:num/NUM-05` | edge NUM-05 | `m1_checks` `checks-g4` | G4 | L §5.13 `prorata` rests |
| 23 | `U:num/NUM-06` | edge NUM-06 | `s_kernel` (gating) | green | L §3.4 price years |
| 24 | `U:num/NUM-07` | edge NUM-07 | `s_kernel` (gating) | green | L §5.9, §7.1 sign and zero |
| 25 | `U:own/OWN-01` | edge OWN-01 | `s_kernel` (gating) | green | L §5.7, §5.10 places, `value` |
| 26 | `U:own/OWN-02` | edge OWN-02 | `m1_checks` `checks-g8` | G8 | L §3.6 claims, §5.14 `uncertain`, §5.14.1 |
| 27 | `U:own/OWN-03` | edge OWN-03 | `s_kernel` (gating) | green | L §3.6 debts, §5.10 `value` |
| 28 | `U:pty/PTY-01` | edge PTY-01 | `m1_checks` `checks-g3` | G3 | L §3.5 parties, dated relations |
| 29 | `U:rec/REC-06` | edge REC-06 | `s_kernel` (gating) | green | L §4.5, §4.6 clocks, `known` |
| 30 | `U:rule/RULE-01` | edge RULE-01 | `s_kernel` (gating) | green | L §5.14.1 decisions, declared reads |
| 31 | `U:rule/RULE-02` | edge RULE-02 | `m1_checks` `checks-g8` | G8 | L §5.14.1 `low`, `with uncertain` |
| 32 | `U:rule/RULE-05` | edge RULE-05 | `m1_checks` `checks-g10` | G10 | L §5.14.1, §5.14.2 order within a day |
| 33 | `U:scen/SCEN-01/SCEN-01` | edge SCEN-01 | `s_kernel` (gating) | green | L §6.1–§6.4 scenarios, amendments |
| 34 | `U:scen/SCEN-03/SCEN-03` | edge SCEN-03 | `m1_checks` `checks-g11` | G11 | L §6.4 linked overrides |
| 35 | `U:scen/SCEN-04/SCEN-04` | edge SCEN-04 | `m1_checks` `checks-green` | green | L §6.4 named extreme paths |
| 36 | `U:surf/SURF-01` | edge SURF-01 | `s_kernel` (gating) | green | L §2.3, §5.11 named facts |
| 37 | `U:surf/SURF-02` | edge SURF-02 | `s_kernel` (gating) | green | L §5.11 a rule's own value, `none` |
| 38 | `U:surf/SURF-04/SURF-04` | edge SURF-04 | `m1_checks` `checks-g5` | G5 | L §2.5, §6.4 one def read thrice (set: `redefine` refused) |
| 39 | `U:surf/SURF-05` | edge SURF-05 | `s_kernel` (gating) | green | L §2.3, §2.4 notes after a one-line def |
| 40 | `U:surf/SURF-07` | edge SURF-07 | `s_kernel` (gating) | green | L §3.1 a day key as a date (2.1-5) |
| 41 | `U:time/TIME-01` | edge TIME-01 | `m1_checks` `checks-g2` | G2 | L §3.1 windows, §8.1 `HOLE-IMPRECISE` |
| 42 | `U:time/TIME-02` | edge TIME-02 | `m1_checks` `checks-g2` | G2 | L §4.2 calendars, business days |
| 43 | `U:time/TIME-03` | edge TIME-03 | `s_kernel` (gating) | green | L §4.3 the law date |
| 44 | `U:time/TIME-04` | edge TIME-04 | `m1_checks` `checks-g2` | G2 | L §4.1 periods split by facts |
| 45 | `U:time/TIME-05` | edge TIME-05 | `m1_checks` `checks-g2` | G2 | L §4.4 windows, deadlines |
| 46 | `U:time/TIME-06` | edge TIME-06 | `m1_checks` `checks-g3` | G3 | L §5.9 running totals, §5.8 `band` |
| 47 | `U:time/TIME-07.reverse` | edge TIME-07 | `s_kernel` (gating) | green | L §5.14.2 order within a day |
| 48 | `U:time/TIME-07` | edge TIME-07 | `s_kernel` (gating) | green | L §5.14.2 order within a day |
| 49 | `U:val/VAL-01` | edge VAL-01 | `m1_checks` `checks-g3` | G3 | L §4.5 rows from a source |
| 50 | `U:val/VAL-02` | edge VAL-02 | `s_kernel` (gating) | green | L §4.3, §4.6 the date a value is read at |
| 51 | `U:val/VAL-03` | edge VAL-03 | `m1_checks` `checks-g11` | G11 | L §4.5 `governs` and `known` |
| 52 | `U:val/VAL-04` | edge VAL-04 | `m1_checks` `checks-g5` | G5 | L §5.11 a published figure against its law |
| 53 | `U:val/VAL-06` | edge VAL-06 | `s_kernel` (gating) | green | L §3.4, §5.9 indexed values |
| 54 | `U:val/VAL-07` | edge VAL-07 | `s_kernel` (gating) | green | L §8.2 the register, `assumed` |
| 55 | `U:set/law/LAW-13/LAW-13` | edge LAW-13 | `m1_checks` `checks-g6` | G6 | L §6.4 a scenario from a date |
| 56 | `U:set/scen/SCEN-01/SCEN-01` | edge SCEN-01 | `m1_checks` `checks-g6` | G6 | L §6.1–§6.4 scenarios, amendments |
| 57 | `U:set/scen/SCEN-03/SCEN-03` | edge SCEN-03 | `m1_checks` `checks-g11` | G11 | L §6.4 linked overrides |
| 58 | `U:set/scen/SCEN-04/SCEN-04` | edge SCEN-04 | `m1_checks` `checks-g6` | G6 | L §6.4 named extreme paths |
| 59 | `U:set/surf/SURF-04/SURF-04` | edge SURF-04 | `m1_checks` `checks-g6` | G6 | L §2.5, §6.4 one def read thrice (set: `redefine` refused) |
| 60 | `I:M:UL-01` | case UL-01 | `m1_checks` `checks-g7` | G7 | L §5.8 `band`, §5.11 rules, §7.2 (PL law case UL-01) |
| 61 | `I:M:UL-02` | case UL-02 | `m1_checks` `checks-g7` | G7 | L §5.8 `band`, §5.11 rules, §7.2 (PL law case UL-02) |
| 62 | `I:M:UL-05` | case UL-05 | `m1_checks` `checks-g7` | G7 | L §5.8 `band`, §5.11 rules, §7.2 (PL law case UL-05) |
| 63 | `I:X:wife_net@c1_fast` | X change 1 | `m1_checks` `checks-green` | green | L §6.1–§6.3 amendments; E §4 the walk; L §5.14.1 |
| 64 | `I:X:social_and_health@c1_fast` | X change 1 | `m1_checks` `checks-green` | green | L §6.1–§6.3 amendments; E §4 the walk; L §5.14.1 |
| 65 | `I:X:pit37@c1_fast` | X change 1 | `m1_checks` `checks-g1` | G1 | L §5.11 law, §5.6 a bind over a year list (P6 year forms) |
| 66 | `I:X:wife_net@c2_fast` | X change 2 | `m1_checks` `checks-green` | green | L §6.1–§6.3 amendments; E §4 the walk; L §5.14.1 |
| 67 | `I:X:social_and_health@c2_fast` | X change 2 | `m1_checks` `checks-green` | green | L §6.1–§6.3 amendments; E §4 the walk; L §5.14.1 |
| 68 | `I:X:pit37@c2_fast` | X change 2 | `m1_checks` `checks-g1` | G1 | L §5.11 law, §5.6 a bind over a year list (P6 year forms) |
| 69 | `I:X:wife_net@c4_fast` | X change 4 | `m1_checks` `checks-green` | green | L §6.1–§6.3 amendments; E §4 the walk; L §5.14.1 |
| 70 | `I:X:social_and_health@c4_fast` | X change 4 | `m1_checks` `checks-green` | green | L §6.1–§6.3 amendments; E §4 the walk; L §5.14.1 |
| 71 | `I:X:pit37@c4_fast` | X change 4 | `m1_checks` `checks-g1` | G1 | L §5.11 law, §5.6 a bind over a year list (P6 year forms) |
| 72 | `I:X:wife_net@c6_fast` | X change 6 | `m1_checks` `checks-green` | green | L §6.1–§6.3 amendments; E §4 the walk; L §5.14.1 |
| 73 | `I:X:social_and_health@c6_fast` | X change 6 | `m1_checks` `checks-green` | green | L §6.1–§6.3 amendments; E §4 the walk; L §5.14.1 |
| 74 | `I:X:pit37@c6_fast` | X change 6 | `m1_checks` `checks-g1` | G1 | L §5.11 law, §5.6 a bind over a year list (P6 year forms) |
| 75 | `I:X:wife_net@c7_fast` | X change 7, a scenario | `m1_checks` `checks-green` | green | L §6.4 a scenario; E §4 the walk; PL law |
| 76 | `I:X:social_and_health@c7_fast` | X change 7, a scenario | `m1_checks` `checks-green` | green | L §6.4 a scenario; E §4 the walk; PL law |
| 77 | `I:X:pit37@c7_fast` | X change 7, a scenario | `m1_checks` `checks-g1` | G1 | L §5.11 law, §5.6 a bind over a year list (P6 year forms) |
| 78 | `I:X:wife_net@v4_fast` | X change 4, variant | `m1_checks` `checks-green` | green | L §6.1–§6.3 amendments; E §4 the walk; L §5.14.1 |
| 79 | `I:X:social_and_health@v4_fast` | X change 4, variant | `m1_checks` `checks-green` | green | L §6.1–§6.3 amendments; E §4 the walk; L §5.14.1 |
| 80 | `I:X:pit37@v4_fast` | X change 4, variant | `m1_checks` `checks-g1` | G1 | L §5.11 law, §5.6 a bind over a year list (P6 year forms) |
| 81 | `I:X:t@min_m13` | X minimal text m13 | `s_kernel` (gating) | green | L §5.7, §5.11 (X minimal text) |
| 82 | `I:X:t@min_m13b` | X minimal text m13b | `s_kernel` (gating) | green | L §5.7, §5.11 (X minimal text) |
| 83 | `I:X:t@min_F4g` | X minimal text F4g | `m1_checks` `checks-g10` | G10 | L §5.7, §5.11 (X minimal text) |
| 84 | `I:X:t@min_F4h` | X minimal text F4h | `m1_checks` `checks-g10` | G10 | L §5.7, §5.11 (X minimal text) |
| 85 | `I:X:t@min_m10l` | X minimal text m10l | `m1_checks` `checks-g10` | G10 | L §5.7, §5.11 (X minimal text) |
| 86 | `I:X:n@min_m2` | X minimal text m2 | `m1_checks` `checks-g1` | G1 | L §6.4 a scenario `replace` of a lifespan |
| 87 | `I:X:n2@min_m2` | X minimal text m2 | `m1_checks` `checks-g1` | G1 | L §6.4 a scenario `replace` of a lifespan |
| 88 | `I:X:n@min_m2b` | X minimal text m2b | `m1_checks` `checks-green` | green | L §6.4 a scenario `replace` of a lifespan |
| 89 | `I:X:n2@min_m2b` | X minimal text m2b | `m1_checks` `checks-green` | green | L §6.4 a scenario `replace` of a lifespan |
| 90 | `U:time/TIME-08-before` | edge TIME-08 | `m1_checks` `checks-green` | green | L §5.14.2 `before` |

## Row 7: #109

The format fuzz target's find (`tmp/m1-rules/crash-1fc61e46…` in the worktree, 75 KB) minimised to `m1/format/key_kinds.tally`: a part whose input keyed by days has rows keyed by two days, a day range and the integer `20240` (the fuzz turned a range key into it). On 2026-11-08 the engine and refcore printed it alike, the integer's row third (sha256 `4546c13d…`); reprinting that output, both moved it first (again the same bytes), so the canonical form wasn't a fixed point. Without the range row both are fixed points. Keys of different literal kinds have no index order beside each other, and the comparison wasn't transitive.

The test holds the law (L §2 "one canonical form"; `PROPERTIES.md` § M0 `m0_format_idempotent`): the text reprinted, then that reprint reprinted, prints the same bytes, or the first reprint is refused. No refcore byte is its expected value; refcore's half is #111 (M2's round). The coordinator's pick P3 orders keys of different literal kinds by kind first, in the grammar's order of literals, then by value, ties by canonical line (GD-28); no vector of a single-kind text may move (`m0_format` holds them). Once the test is green, the rules step adds the artifact to `engine/fuzz/regressions/format` (pick P1). When refcore follows P3 (M2), T adds a vector of the order's bytes.

## What rulings hold back

None of row 1's vectors waits on a ruling. Held so far: none (`held.tsv` names no M1 vector).

## Keeping master green

As at S and M0 (`MILESTONE-S.md` § Keeping master green): every `m1_…` target has `test = false` and runs only as `check`'s report until M1 is in `engine/tools/milestones.toml`'s `gating`, which the rules step adds in `build/m1`'s last commit (the marker, pick P20).
