| Row | Status | Reason | Spec | Gap | Text |
|---|---|---|---|---|---|
| Q:B-01 | expressed | | L §5.10 | — | households/B/business.tally; asks/B-01.tally; maps/B/B-01.md |
| Q:B-02 | expressed | | L §4.2 | — | households/B/business.tally; asks/B-02.tally; maps/B/B-02.md |
| Q:B-03 | expressed | | L §4.3 | — | households/B/business.tally; asks/B-03.tally; maps/B/B-03.md |
| Q:B-04 | expressed | | L §6.4 | — | households/B/flat.tally; scenarios/kitchen.tally; scenarios/nothing_more.tally; asks/B-04.tally; maps/B/B-04.md |
| Q:B-05 | awkward | Step 4: the difference is base and branch in one ask, `x as if {scenario no_holidays} - x`, the L:1258 pattern that brief note 10 calls a workaround. Step 2: the annuity's periodic rate is assumed to be r/12, because B-D07 .4 is silent | L:1258 | — | households/B/flat.tally; scenarios/no_holidays.tally; asks/B-05.tally; maps/B/B-05.md |
| Q:B-07 | awkward | Step 1: the October gross is found by `argmin` over a cent index with a pin per key, not by solve (the net is a step function). The key's "only one within ±20.00" is not checked. The 2024 return is a second ask that pins the found gross | L §5.10 | — | households/B/ola.tally; asks/B-07.tally; maps/B/B-07.md |
| Q:B-06 | expressed | | L §5.10 | — | households/B/business.tally; asks/B-06.tally; maps/B/B-06.md |
| Q:B-08 | breaks | Steps 2-5: the `residuals` ledger shows corrections at anchors, not the key's four rows. The duplicate shows as +111.27 at the 10-15 anchor. The reversal is explained, so it is not listed. The gap's net is a hole. The card fee shows only at the 12-29 anchor | L §5.18 | — | households/B/records/cis-2025.tally; asks/B-08.tally; maps/B/B-08.md |
| Q:B-09 | breaks | Step 2: FV 016 should be a hole. The gap's hole row sits on Cis, and the spec does not say whether it touches the claim `inv_owed[fv_016_2025]`. So the claim keeps 4270.00 with status "missed": a value where the key has a hole | L §5.18 | — | households/B/rules.tally; asks/B-09.tally; maps/B/B-09.md |
| Q:B-10 | awkward | Partial: 11-26 is a hole, so the known days need a second ask. v2 since January is built from recursive `as if` defs. v3 is a LOAD-CYCLE in a scenario file, argued from the spec (loadcheck has no cycle check). The `without` block needs rows the dossier lacks | L §5.13 | — | households/B/rules.tally; scenarios/hr2_v3.tally; asks/B-10.tally; maps/B/B-10.md |
| Q:B-11 | awkward | Step 3: Ola's Wrzos balance is a hole (no statement after March), so the known part needs a second ask. The `without` block needs rows the dossier lacks | L §8 | — | households/B/rules.tally; households/B/flat.tally; asks/B-11.tally; maps/B/B-11.md |
| Q:B-12 | case suspect | F6 "before those, the gazette's" needs AVG-WAGE-Q4 9180.65, published 2026-01-20. That is after the cut of 2026-01-02 and after the model's `on` (LOAD-KNOWN). Also: the B-HR3 decision needs two asks. The text has no tax/VAT/health pays, no F5 January items and no B-HR5 | L §4.5 | — | households/B/forecast.tally; scenarios/no_overpay.tally; asks/B-12.tally; maps/B/B-12.md |
| O:B-1 | observation | The record CSVs (Cis `#`-header, Wrzos `;` with decimal comma, Lazur table, Faktura+) have no format a text loads. The rows are written out, each citing its file and line | O §2.2 | — | households/B/records/; households/B/registers.tally |
| O:B-2 | case suspect | The `without` blocks of B-08 to B-11 depend on the 11-14..30 rows and on no reversal. No document holds those rows, so they can't be written from the dossier | L §5.18 | — | maps/B/B-08.md |
| O:B-3 | case suspect | The B-12 key reads AVG-WAGE-Q4 2026, published 2026-01-20, at a known-as-of of 2026-01-02. The reality's `on` 2026-01-07 refuses that row, so I dropped awq4_2026 from SOC-4 | L §4.5 | — | law/SOC-4.tally |
| O:B-4 | observation | GEN-1 lists holidays only to 2026, so `next_business` in the 2027-28 forecast sees weekends only | L §4.2 | — | law/GEN-1.tally |
| O:B-5 | observation | B-D07 .4 says "annuity" but gives no periodic rate. I assumed nominal 12/year (`periodic(r, month)`) | L:323 | — | households/B/flat.tally |
| S:701 | finding | The spec is silent on what a hole movement's unknown `explains` does to the claim places and pays it might explain. It only makes its own place's balance a hole | L §5.18 | — | households/B/records/cis-2025.tally |
| S:702 | finding | `residuals` reports corrections at anchors, not the rows behind them. A duplicate, an explained reversal and a missing pay can't be listed as rows | L §5.18 | — | asks/B-08.tally |
| S:703 | finding | `flows` takes only a place or a state pay, so a flow pay's realised amount (`client_pay[i]`) can't be read. The workaround is one claim place per invoice | L:923 | — | households/B/business.tally |
| S:704 | finding | `opening[x[*]]` for a keyed place over an open index does not parse (`*` only at the top row-key level). I used `opening[*]` for all places | L §2.3 | — | households/B/places.tally |
| S:705 | finding | parsecheck rejects the built-in outside places `outside_FLR` / `outside_DUK` (upper-case currency in an id). I declared my own outside places | L §5.9 | — | households/B/places.tally |
| S:706 | finding | Notes are allowed only on rule-form defs. A short-form `def … = expr` can't carry its reason note | L §2.3 | — | households/B/business.tally |
| S:707 | finding | The spec does not say whether a pin's value expression counts in the LOAD-CYCLE check (v3's reserve pins the overpayment) | L §5.10 | — | scenarios/hr2_v3.tally |
| S:708 | finding | The spec does not say whether an ask may be declared inside a scenario file (`b10_v3`) | L §6.4 | — | scenarios/hr2_v3.tally |
| S:709 | finding | Pinning a record field (`sale[k].net = …`) is allowed by the `pref` grammar, but no sentence in L §5.10 says a pin may target a record input's field | L §5.10 | — | asks/B-06.tally |

- **Hardest reading:** L §5.18, records. That covers coverage versus modelled pays, anchors, and how hole rows spread. Also the known date of coverage (B-07's as-seen-on-02-27 needs gap days covered by a later-known version) and `flows` being limited to places and state pays.
- **Law files added** (new, all parse): GEN-2, GEN-4, GEN-5, PIT-10, PIT-11, PIT-12, PIT-15, SOC-3, SOC-4, TT-1, TT-2, VAT-1 to VAT-6, MTG-1, MTG-2, CIV-4. Reused as they stand: GEN-1, GEN-6, PIT-2 to PIT-6, SOC-1, SOC-2, CC-2, MOT-1, MOT-2.
- **Checks:** parsecheck exit 0 on households/B (31 files) plus law. loadcheck reports 0 refusals on the merged copy in the scratchpad (`merged_B`, 75 files), after fixing these:
  - LOAD-DUP: the movement input `cis` against `place cis` (now `cisr`), and `version car_tax` against `def car_tax`.
  - LOAD-SHADOW: SOC-4's band argument `rev`.
  - LOAD-KEY: `version sickness` against the key `sickness`.
- **Possibly the tools' fault:** parsecheck refusing `outside_FLR` (S:705), and loadcheck checking no cycles, so the v3 refusal is argued from the spec, not shown.
- **Rule check:** no file under `tmp/cases/` that existed before this session was changed. The record files are written out whole: all 2025 Cis rows (420), Lazur and Wrzos. Nothing under the holdout folders was opened, and no `__pycache__` was left behind.
