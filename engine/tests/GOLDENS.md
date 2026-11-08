# The goldens and the 36 pins, restated in tally 2 (DONE gate 2)

DONE gate 2: the prototype's goldens G1–G10 and its 36 pins (`engine/spec/verification/goldens.md`), carried into tally 2, hold on tally 2's canonical form, or each difference is explained by a finding it fixed. BUILD-ORDER places G1, G2, G3, G6–G10 and the pins at M1 (G4, G5 at M2). Written by T for issue #108 (M1), chunk C2, 2026-11-08; `MILESTONE-M1.md` row 2. The coordinator's picks: P6 (the digests as a projection of asks), P7 (the readings the fixtures leave open: G6's raise on top of indexation; G1 and G3 over the household as a whole; `pit_joint` under L-26).

## How a golden is restated

- **The prototype's fixtures** are `engine/spec/verification/blocks/cases/01..15.json` and `pins-1-7.json`. A restated text writes their figures as stated rows, every figure a row whose note names its block (Tally explicit).
- **The vector** is refcore's run of the text (`m1/vectors.tsv`, groups `goldens-…`, held by the runner's `m1_goldens`). No expected value is worked out by hand: the golden is the prototype's figure, the vector refcore's bytes, and this page records whether they agree.
- **A difference** is allowed only where `goldens.md` names the finding that moves it; any other is a STOP-REPORT in `GAPS.md`, and a wrong number from refcore on a valid household text is the M1 brief's §3 patch case.
- **The law-level figures reuse household X's law** (`integration/amendments/real11/c1`): X is the prototype's household rewritten, and its employment law is the prototype's `employment_pl@2022` (X's wife at 7 000 gross prints the pin `wife_tax/net[2026-10]`, 5 101.67, byte for byte). `m1/goldens/law/golden.tally` adds the prototype's `my_job` (12 000 gross; 12 480 from 2027 by `index.wage`).

## What the repository can't restate: the prototype's bank history

The prototype's fixtures name a bank history they don't contain. `01-month.json`'s daily spending is `hist_daily(category, 365) * index.prices`, learned from the bank rows, and G2's konto (11 685.67 on 2026-10-16) is "bank rows before today" (P-01). No block holds those rows, and no tracked file of the repository does (X has one invented month, `records/mbank/2026-09.tally`, not the prototype's history). Every figure that reads konto's history or the spending learned from it can't be restated without inventing it, and an invented history makes the prototype's figure meaningless as a key. Those are marked **blocked** below; what unblocks them is a question to the coordinator on issue #108 (T's chunk C2 report).

## The goldens

| Golden | The prototype's figure | Restated by | refcore (tally 2.1.6) | Verdict |
|---|---|---|---|---|
| G1 | net worth −40.96 (claim) / −2 040.96 (outside) | — | — | **blocked**: the household's net worth reads konto's history (14-marek's ledger alone gives neither figure) |
| G2 | konto 11 685.67 on 2026-10-16 | — | — | **blocked**: bank rows before today |
| G3 | the 15th: 7 887.29 invested, konto 11 594.97, 1 000.00 on 12-06 | — | — | **blocked**: the rule reads konto |
| G6 | my net in 2027-03, 8 837.78 → 9 262.77 with a 5 % raise from March | `goldens/pins_employment`, `goldens/g6_raise` (`m1/goldens/law/raise.tally`: 12 480 × 1.05 = 13 104 from 2027-03, on top of indexation, P7) | 8 837.78 PLN; 9 262.77 PLN in `raise` | **holds** |
| G6 | net worth 4 630 247 → 4 829 659; FIRE Feb 2050 → Feb 2049 | — | — | **blocked**: net worth reads konto's history |
| G7 | a rent of 2 500 from 2028 must not book 2 652.25 | `goldens/g7_price_year` (`m1/goldens/rent/model.tally`: the row's price year stated, 2028, and its twin in 2026 money) | 2 500.00 PLN, and 2 652.25 PLN for the twin | **holds**: the price year is stated, as U-13 asks |
| G8 | a job without its law warns; with law and return 381 885.38 | C3 | | not yet (C3, with 10- and 11-) |
| G9 | a question with a payment is refused (C-19) | C3 | | not yet (C3) |
| G10 | rent in 2030 3 601.63 → 3 889.62 at 5 % inflation | `goldens/g10_rent` (`m1/goldens/rent/`: 3 200 in 2026 prices, `index.prices` 1 until 2027, then × 1.03 a year; the scenario `infl5`, × 1.05) | 3 601.63 PLN; 3 889.62 PLN in `infl5` | **holds** |
| G10 | net worth 2056 ≈ 10 094k → 5 980k nominal, 4 159k → 1 384k in today's money | — | — | **blocked**: net worth reads konto's history |

## The 36 pins (`pins-1-7.json`)

| Pins | The prototype's values | Restated by | refcore (tally 2.1.6) | Verdict |
|---|---|---|---|---|
| `my_tax/net[2026-10]`, `[2027-03]` | 8 509.87; 8 837.78 | `goldens/pins_employment`, `g_my_net` | 8 509.87 PLN; 8 837.78 PLN | **holds** |
| `wife_tax/net[2026-10]` | 5 101.67 | `goldens/pins_employment`, `g_wife_net` | 5 101.67 PLN | **holds** |
| `pit_joint/due[2026]`, `[2027]` | −259; −1 245 | `goldens/pins_employment`, `g_pit37` | separate returns: me −5.00, 3.00; wife −2.00, −2.00 PLN | **moved by L-26** (P7): X's law files separate PIT-37 returns, the joint return isn't in its law (art. 6 ust. 8 PIT bars it beside ryczałt, as `goldens.md` expects) |
| `business_tax/health[2026-10]`, `pit.advance[2026-10]`, `health.settle[2026]`, `business_vat/vat.due[2026-10]` | 498.35; 690; 0; 1 288 | C3 (05-side-business's facts; X's own B2B differs: its `health_month[2026-10]` is 830.58) | | not yet (C3) |
| `y22_tax/pit.advance[2022-01, -06, -07, -12]`, `y22_return` tax, advances, due | 266; 266; 188; 188; 2 253; 2 724; −471 | C3 (06-pit-2022) | | not yet (C3) |
| `reality.digest`, `case2.digest` | 679da89c25717241, ed6bb9dd6f446c0a | — | — | **blocked**: P6's projection over the whole reality reads konto's history |
| konto and net worth at 2026-11-01, 2027-06-30, 2036-10-17, 2056-10-17 | 2 261.8 / 223 463.93; 5 743.5 / 298 270.74; 14 072.28 / 1 746 077.48; 22 613.79 / 10 093 811.25 | — | — | **blocked**: konto's history |
| `invest_15/invest` 2026-11, -12, 2027-01; `to_ike` 2026-11, -12; `invest_15_oki/to_ike` 2027-01, `to_rest` 2027-12 | 7 887.29; 7 424.19; 7 478.94; 0; 0; 7 478.94; 7 312.73 | — | — | **blocked**: the rule reads konto |
| `oki_tax/tax` 2027, 2030, 2040 | 204; 1 729; 10 409 | — | — | **blocked**: OKI's average value reads what the rule invested |

So far: 5 pins hold, 2 move by L-26; 11 wait for C3; 20 are blocked. Goldens: G6's and G10's first figures and G7 hold; G8, G9 wait for C3; G1, G2, G3 and the net-worth halves of G6 and G10 are blocked.
