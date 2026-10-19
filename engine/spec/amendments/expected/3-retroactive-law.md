# Change 3: a retroactive law change (lower health share, back to January) — expected outputs

**Applied as:** an `advance` of `on`, 2026-10-19 → 2027-06-15, then the `add` of `health_ryczalt_2027` (governs 2027.., known 2027-06-15), with its line `arrears none for health_pay`. Final cut K = 2027-06-15.

**Derived by:** C3-a and C3-b, blind to each other; every value agrees in the main key. In the variant they differed on one cell, settled by LR-7.

## Cells

| # | Ask | Key | Before | After | Status |
|---|---|---|---|---|---|
| 1 | wife_net | 2026-10 | 5101.67 | 5101.67 | exact |
| 2 | wife_net | 2027-01 | 5101.67 | 5101.67 | exact |
| 3 | social_and_health | 2026-10, zus_social | 456.18 | 456.18 | exact |
| 4 | social_and_health | 2026-10, health_month | 830.58 | 830.58 | exact |
| 5 | social_and_health | 2027-01, zus_social | 474.44 | 474.44 | exact |
| 6 | social_and_health | 2027-01, health_month | 518.28 | 518.28 | exact |
| 7 | social_and_health | 2028-09, zus_social | 2083.99 | 2083.99 | exact |
| 8 | social_and_health | 2028-09, health_month | 898.35 | **808.52** | exact |
| 9 | replay_september | konto 2026-09 | the HH4-b §5 ledger: balance line +32097.23 (stated 41386.27, booked 9289.04), then spend +4599.90, client_pay −3690.00, unexplained −3574.02, invest +535.30, loan_instalment +300.00, seven groups at 0.00 | unchanged | exact |
| 10 | explain_room | room[2026-11], (2026-11-15, phase 6, invest[ike, 2026-11]) | 4386.89 | 4386.89 | exact |
| 11 | explain_room | room[2026-11], (2026-11-15, phase 6, invest[mak, 2026-11]) | 4386.89 | 4386.89 | exact |
| 12 | konto_balance | 2026-10-31, bal | 6064.17 | 6064.17 | exact |
| 13 | konto_balance | 2026-10-31, bal − vat_owed | 6064.17 | 6064.17 | exact |
| 14 | konto_balance | 2026-12-31, bal | 6944.90 | 6944.90 | exact |
| 15 | konto_balance | 2026-12-31, bal − vat_owed | 6944.90 | 6944.90 | exact |
| 16 | konto_balance | 2027-06-30, bal | 6760.19 | 6760.19 | exact |
| 17 | konto_balance | 2027-06-30, bal − vat_owed | 6760.19 | 6760.19 | exact |
| 18 | konto_balance | 2027-12-31, bal | 6656.75 | 6656.75 | exact |
| 19 | konto_balance | 2027-12-31, bal − vat_owed | 6656.75 | 6656.75 | exact |
| 20 | net_worth | 2026-12-31 | 170202.16 | 170202.16 | exact |
| 21 | net_worth | 2031-12-31 | not computed (run-only) | higher than the base | run-only |
| 22 | net_worth | 2041-12-31 | not computed (run-only) | higher than the base | run-only |
| 23 | net_worth | 2059-12-31 | not computed (run-only) | higher than the base | run-only |
| 24 | ike_left | 2026 | −6000.00 | −6000.00 | exact |
| 25 | ike_left | 2027 | 0.00 | 0.00 | exact |
| 26 | ike_left | 2028 | 0.00 | 0.00 | exact |
| 27 | pit28 | 2026 | 2.00 | 2.00 | exact |
| 28 | pit28 | 2027 | 2.00 | **3.00** | exact |
| 29 | pit28 | 2028 | 1.00 | **4.00** | exact |
| 30 | pit37 | me, 2026 | 0.00 | 0.00 | exact |
| 31 | pit37 | me, 2027 | 0.00 | 0.00 | exact |
| 32 | pit37 | me, 2028 | 0.00 | 0.00 | exact |
| 33 | pit37 | wife, 2026 | −2.00 | −2.00 | exact |
| 34 | pit37 | wife, 2027 | −2.00 | −2.00 | exact |
| 35 | pit37 | wife, 2028 | −2.00 | −2.00 | exact |
| 36 | health_settlement | 2026 | 996.69 | 996.69 | exact |
| 37 | health_settlement | 2027 | 1036.56 | **691.04** | exact |
| 38 | health_settlement | 2028 | 1078.02 | **808.53** | exact |
| 39 | taxes_paid | 2026 | 52272.00 | 52272.00 | exact |
| 40 | taxes_paid | 2027 | 69059.00 | **69089.00** | exact |
| 41 | taxes_paid | 2028 | 68997.00 | **69045.00** | exact |

**41 cells:** 38 exact, 0 holes, 3 run-only; 10 differ from the base (new cells included), 31 are the base's value.

## First day, and sanity

- **First day anything differs:** 2027-06-15, `invest[oki, 2027-06]` +86.38. Net worth first differs on 2027-06-21.
- **Sanity:** the earliest `governs` is 2027-01-01, so it passes.

## Back pay

Only `health_pay[2027-04]` (paid 2027-05-20) changes among the booked instances: δ −86.38, covered by `arrears none for health_pay`, so no movement is booked. `health_pay[2027-01..03]` are skipped by `ENGINE.md` §5 step 2, and every other booked flow pay has δ 0.

## The advance alone

No value changes. Only the registers do: the cut line moves to 2027-06-15, and the `held` entries for the months it passes drop.

## Warnings and holes

- **Added:** `WARN-BACKPAY`: `health_pay[2027-04]`, δ −86.38, covered by `arrears none for health_pay` (LR-8: the warning lists covered instances too).
- **Cleared:** none. The base's `WARN-NEGATIVE` on konto (2026-03-01, 2026-03-17) stands.
- **Holes:** none.

## The variant without the `arrears` line

The same version with `arrears none for health_pay` removed.

| # | Ask | Key | Before | After | Status |
|---|---|---|---|---|---|
| 1 | wife_net | 2026-10 | 5101.67 | 5101.67 | exact |
| 2 | wife_net | 2027-01 | 5101.67 | 5101.67 | exact |
| 3 | social_and_health | 2026-10, zus_social | 456.18 | 456.18 | exact |
| 4 | social_and_health | 2026-10, health_month | 830.58 | 830.58 | exact |
| 5 | social_and_health | 2027-01, zus_social | 474.44 | 474.44 | exact |
| 6 | social_and_health | 2027-01, health_month | 518.28 | 518.28 | exact |
| 7 | social_and_health | 2028-09, zus_social | 2083.99 | 2083.99 | exact |
| 8 | social_and_health | 2028-09, health_month | 898.35 | **808.52** | exact |
| 9 | replay_september | konto 2026-09 | the HH4-b §5 ledger: balance line +32097.23 (stated 41386.27, booked 9289.04), then spend +4599.90, client_pay −3690.00, unexplained −3574.02, invest +535.30, loan_instalment +300.00, seven groups at 0.00 | unchanged | exact |
| 10 | explain_room | room[2026-11], (2026-11-15, phase 6, invest[ike, 2026-11]) | 4386.89 | 4386.89 | exact |
| 11 | explain_room | room[2026-11], (2026-11-15, phase 6, invest[mak, 2026-11]) | 4386.89 | 4386.89 | exact |
| 12 | konto_balance | 2026-10-31, bal | 6064.17 | 6064.17 | exact |
| 13 | konto_balance | 2026-10-31, bal − vat_owed | 6064.17 | 6064.17 | exact |
| 14 | konto_balance | 2026-12-31, bal | 6944.90 | 6944.90 | exact |
| 15 | konto_balance | 2026-12-31, bal − vat_owed | 6944.90 | 6944.90 | exact |
| 16 | konto_balance | 2027-06-30, bal | 6760.19 | **hole: `health_pay[2027-04] on 2027-05-20 changed after it was paid (version health_ryczalt_2027); is back pay due, and when?`** | hole |
| 17 | konto_balance | 2027-06-30, bal − vat_owed | 6760.19 | **hole: `health_pay[2027-04] on 2027-05-20 changed after it was paid (version health_ryczalt_2027); is back pay due, and when?`** | hole |
| 18 | konto_balance | 2027-12-31, bal | 6656.75 | **hole: `health_pay[2027-04] on 2027-05-20 changed after it was paid (version health_ryczalt_2027); is back pay due, and when?`** | hole |
| 19 | konto_balance | 2027-12-31, bal − vat_owed | 6656.75 | **hole: `health_pay[2027-04] on 2027-05-20 changed after it was paid (version health_ryczalt_2027); is back pay due, and when?`** | hole |
| 20 | net_worth | 2026-12-31 | 170202.16 | 170202.16 | exact |
| 21 | net_worth | 2031-12-31 | not computed (run-only) | **hole: `health_pay[2027-04] on 2027-05-20 changed after it was paid (version health_ryczalt_2027); is back pay due, and when?`** | hole |
| 22 | net_worth | 2041-12-31 | not computed (run-only) | **hole: `health_pay[2027-04] on 2027-05-20 changed after it was paid (version health_ryczalt_2027); is back pay due, and when?`** | hole |
| 23 | net_worth | 2059-12-31 | not computed (run-only) | **hole: `health_pay[2027-04] on 2027-05-20 changed after it was paid (version health_ryczalt_2027); is back pay due, and when?`** | hole |
| 24 | ike_left | 2026 | −6000.00 | −6000.00 | exact |
| 25 | ike_left | 2027 | 0.00 | **hole: `health_pay[2027-04] on 2027-05-20 changed after it was paid (version health_ryczalt_2027); is back pay due, and when?`** | hole |
| 26 | ike_left | 2028 | 0.00 | **hole: `health_pay[2027-04] on 2027-05-20 changed after it was paid (version health_ryczalt_2027); is back pay due, and when?`** | hole |
| 27 | pit28 | 2026 | 2.00 | 2.00 | exact |
| 28 | pit28 | 2027 | 2.00 | **hole: `health_pay[2027-04] on 2027-05-20 changed after it was paid (version health_ryczalt_2027); is back pay due, and when?`** | hole |
| 29 | pit28 | 2028 | 1.00 | **hole: `health_pay[2027-04] on 2027-05-20 changed after it was paid (version health_ryczalt_2027); is back pay due, and when?`** | hole |
| 30 | pit37 | me, 2026 | 0.00 | 0.00 | exact |
| 31 | pit37 | me, 2027 | 0.00 | 0.00 | exact |
| 32 | pit37 | me, 2028 | 0.00 | 0.00 | exact |
| 33 | pit37 | wife, 2026 | −2.00 | −2.00 | exact |
| 34 | pit37 | wife, 2027 | −2.00 | −2.00 | exact |
| 35 | pit37 | wife, 2028 | −2.00 | −2.00 | exact |
| 36 | health_settlement | 2026 | 996.69 | 996.69 | exact |
| 37 | health_settlement | 2027 | 1036.56 | **hole: `health_pay[2027-04] on 2027-05-20 changed after it was paid (version health_ryczalt_2027); is back pay due, and when?`** | hole |
| 38 | health_settlement | 2028 | 1078.02 | **808.53** | exact |
| 39 | taxes_paid | 2026 | 52272.00 | 52272.00 | exact |
| 40 | taxes_paid | 2027 | 69059.00 | **hole: `health_pay[2027-04] on 2027-05-20 changed after it was paid (version health_ryczalt_2027); is back pay due, and when?`** | hole |
| 41 | taxes_paid | 2028 | 68997.00 | **hole: `health_pay[2027-04] on 2027-05-20 changed after it was paid (version health_ryczalt_2027); is back pay due, and when?`** | hole |

**41 cells:** 27 exact, 14 holes, 0 run-only; 16 differ from the base (new cells included), 25 are the base's value.

- `health_pay[2027-04]` gets a hole movement, `HOLE-CHANGED`, dated on or after 2027-06-15 (V4-P4). konto and zus are holes from 2027-06-15 to E; `wcash[ike]` and `wcash[oki]` from 2027-06-15; `wunits[ike]`, `wunits[oki]` and `rynek` from 2027-06-16; `urzad_skarbowy` from 2027-07-20.
- `health_settlement` 2028 stays known at 808.53: its `flows(health_pay[2028-m], …)` are keyed to 2028's instances and can't count `[2027-04]`'s movement (LR-7). `pit28` 2028 reads `flows(health_pay, 2028)`, the pay as a whole over a span after 2027-06-15, and is a hole.
- **Warnings:** `WARN-BACKPAY` lists `health_pay[2027-04]`, δ −86.38, as covered by no line: it becomes `HOLE-CHANGED`.
- **Register:** every hole carries the one question above. There is no new `assumed`.
- **Sanity:** the first day is 2027-06-15, so it passes.

## Readings it rests on

- `HOLE-CHANGED` is dated on or after V.known (V4-P4), and only a `flows` that can count the movement is a hole (LR-7).
- `WARN-BACKPAY` lists covered instances too (LR-8).
- `held` and the printed cut are the ask's K: LR-1.
- The law-part gate premise (`README.md` here).
