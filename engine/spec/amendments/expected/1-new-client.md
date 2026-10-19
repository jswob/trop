# Change 1: a third B2B client — expected outputs

**Applied as:** an `advance` of `on`, 2026-10-19 → 2027-01-20, then the `add` of the version holding `client_c`'s key and rows (known 2027-01-20). Final cut K = 2027-01-20.

**Derived by:** C1-a and C1-b, blind to each other; every value agrees. One cell's reading differed (replay_september) and the lead settled it (below).

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
| 8 | social_and_health | 2028-09, health_month | 898.35 | 898.35 | exact |
| 9 | replay_september | konto 2026-09 | the HH4-b §5 ledger: balance line +32097.23 (stated 41386.27, booked 9289.04), then spend +4599.90, client_pay −3690.00, unexplained −3574.02, invest +535.30, loan_instalment +300.00, seven groups at 0.00 | unchanged | exact |
| 10 | explain_room | room[2026-11], (2026-11-15, phase 6, invest[ike, 2026-11]) | 4386.89 | 4386.89 | exact |
| 11 | explain_room | room[2026-11], (2026-11-15, phase 6, invest[mak, 2026-11]) | 4386.89 | 4386.89 | exact |
| 12 | konto_balance | 2026-10-31, bal | 6064.17 | 6064.17 | exact |
| 13 | konto_balance | 2026-10-31, bal − vat_owed | 6064.17 | 6064.17 | exact |
| 14 | konto_balance | 2026-12-31, bal | 6944.90 | 6944.90 | exact |
| 15 | konto_balance | 2026-12-31, bal − vat_owed | 6944.90 | 6944.90 | exact |
| 16 | konto_balance | 2027-06-30, bal | 6760.19 | 6760.19 | exact |
| 17 | konto_balance | 2027-06-30, bal − vat_owed | 6760.19 | 6760.19 | exact |
| 18 | konto_balance | 2027-12-31, bal | 6656.75 | **9623.81** | exact |
| 19 | konto_balance | 2027-12-31, bal − vat_owed | 6656.75 | **9623.81** | exact |
| 20 | net_worth | 2026-12-31 | 170202.16 | 170202.16 | exact |
| 21 | net_worth | 2031-12-31 | not computed (run-only) | higher than the base | run-only |
| 22 | net_worth | 2041-12-31 | not computed (run-only) | higher than the base | run-only |
| 23 | net_worth | 2059-12-31 | not computed (run-only) | higher than the base | run-only |
| 24 | ike_left | 2026 | −6000.00 | −6000.00 | exact |
| 25 | ike_left | 2027 | 0.00 | 0.00 | exact |
| 26 | ike_left | 2028 | 0.00 | 0.00 | exact |
| 27 | pit28 | 2026 | 2.00 | 2.00 | exact |
| 28 | pit28 | 2027 | 2.00 | 2.00 | exact |
| 29 | pit28 | 2028 | 1.00 | 1.00 | exact |
| 30 | pit37 | me, 2026 | 0.00 | 0.00 | exact |
| 31 | pit37 | me, 2027 | 0.00 | 0.00 | exact |
| 32 | pit37 | me, 2028 | 0.00 | 0.00 | exact |
| 33 | pit37 | wife, 2026 | −2.00 | −2.00 | exact |
| 34 | pit37 | wife, 2027 | −2.00 | −2.00 | exact |
| 35 | pit37 | wife, 2028 | −2.00 | −2.00 | exact |
| 36 | health_settlement | 2026 | 996.69 | 996.69 | exact |
| 37 | health_settlement | 2027 | 1036.56 | **691.04** | exact |
| 38 | health_settlement | 2028 | 1078.02 | **718.68** | exact |
| 39 | taxes_paid | 2026 | 52272.00 | 52272.00 | exact |
| 40 | taxes_paid | 2027 | 69059.00 | **90038.00** | exact |
| 41 | taxes_paid | 2028 | 68997.00 | **94350.00** | exact |

**41 cells:** 38 exact, 0 holes, 3 run-only; 9 differ from the base (new cells included), 32 are the base's value.

`oki_tax[2027]` goes 368 → 542, inside taxes_paid 2028.

## First day, and sanity

- **First day anything differs:** 2027-03-15. `invest[oki, 2027-03]` rises by 5280.00, from 5540.98 to 10820.98, and konto falls by the same that day. `room[2027-02]` is unchanged at 9835.50: the change file's 2027-02-15 is the first day a register differs, not a value.
- **Sanity:** the earliest `governs` is 2026-01 (`fee_net[client_c] = 0 governs 2026-01..2027-01`), so it passes.

## The advance alone

No value, hole, modelled past or ledger line changes, since no base version is known between the two dates. The registers' cut line moves to 2027-01-20, and the `held` entries read for 2026-11..2027-01 drop.

## Warnings and holes

None added and none cleared. konto's lowest end-of-day balance in each worked window stays at 1800 or above.

## The register

From 2027-02-15, the lines that read client C's fee gain `held: fee_net[client_c] = 6000 (client_c_2027), read for <period>` entries, the period being the months each line reads after K (for example `2027-02..2027-12` on a 2027 line, `2027-02..2059-12` on a walk line). The 2026-01..2027-01 row has a closed span and is never `held`.

## Readings it rests on

- **replay_september is unchanged and exact.** At September 2026's booking cuts, before the version's `known`, the key `client_c` doesn't exist, so no client_c instance is a candidate there: LC-1 (LANGUAGE §5.3). C1-a had held this cell on the question; C1-b read it from the text as above.
- `held` and the printed cut are the ask's K: LR-1.
