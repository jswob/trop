# Change 2: the 2027 law figures — expected outputs

**Applied as:** an `advance` of `on`, 2026-10-19 → 2026-12-18, then one `add` of the two law versions: `min_wage_2027` (known 2026-09-12, a late entry) and `proj_wage_2027` (known 2026-12-18), both governing 2027. Final cut K = 2026-12-18. The diff adds 14 lines (12 non-blank).

**Derived by:** C2-a and C2-b, blind to each other; every value and every register entry agrees.

## Cells

| # | Ask | Key | Before | After | Status |
|---|---|---|---|---|---|
| 1 | wife_net | 2026-10 | 5101.67 | 5101.67 | exact |
| 2 | wife_net | 2027-01 | 5101.67 | 5101.67 | exact |
| 3 | social_and_health | 2026-10, zus_social | 456.18 | 456.18 | exact |
| 4 | social_and_health | 2026-10, health_month | 830.58 | 830.58 | exact |
| 5 | social_and_health | 2027-01, zus_social | 474.44 | **474.60** | exact |
| 6 | social_and_health | 2027-01, health_month | 518.28 | 518.28 | exact |
| 7 | social_and_health | 2028-09, zus_social | 2083.99 | **2084.67** | exact |
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
| 18 | konto_balance | 2027-12-31, bal | 6656.75 | 6656.75 | exact |
| 19 | konto_balance | 2027-12-31, bal − vat_owed | 6656.75 | 6656.75 | exact |
| 20 | net_worth | 2026-12-31 | 170202.16 | 170202.16 | exact |
| 21 | net_worth | 2031-12-31 | not computed (run-only) | lower than the base | run-only |
| 22 | net_worth | 2041-12-31 | not computed (run-only) | lower than the base | run-only |
| 23 | net_worth | 2059-12-31 | not computed (run-only) | lower than the base | run-only |
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
| 37 | health_settlement | 2027 | 1036.56 | 1036.56 | exact |
| 38 | health_settlement | 2028 | 1078.02 | 1078.02 | exact |
| 39 | taxes_paid | 2026 | 52272.00 | 52272.00 | exact |
| 40 | taxes_paid | 2027 | 69059.00 | 69059.00 | exact |
| 41 | taxes_paid | 2028 | 68997.00 | 68997.00 | exact |

**41 cells:** 38 exact, 0 holes, 3 run-only; 5 differ from the base (new cells included), 36 are the base's value.

## First day, and sanity

- **First day anything differs:** 2027-02-15, the movement `invest[ike, 2027-02]`, 9835.50 → 9835.34. Its window, 02-15..03-14, holds the ZUS payment of 2027-02-22, now 0.16 higher. Net worth and the outside place `zus` first differ on 2027-02-22.
- **Sanity:** the earliest `governs` is 2027-01-01, so it passes.

## Balances, to 2028-03

| Place | Difference |
|---|---|
| konto | +0.16 from each 15th to that month's ZUS day, then 0 (+0.17 in 2028); −0.16 from 2027-08-20 to 2027-09-14, the month whose window minimum was day t. Every asked month-end is unchanged |
| wcash[ike] | +9.60 at the end of 2027; +19.59 from 2028-03-15 |
| wcash[oki] | −11.36 at the end of 2027; −21.85 from 2028-03-15 |
| zus (outside) | +1.76 at 2027-12-31; +2.26 by 2028-03-31 |
| net worth | −1.76 at 2027-12-31; −2.26 at 2028-03-31: the extra ZUS paid |

In March 2027 IKE gets +9.76 and OKI −9.92. pit28 and taxes_paid are exactly unchanged.

## The advance alone

No value, hole, modelled past, ledger line or warning changes. 86 movements, over 60 days, move from plan to past. Every register's cut line moves to 2026-12-18, the `held` entries read for 2026-11 and 2026-12 drop, and the law date of the ask lines with no date of their own moves to K.

## Warnings and holes

None added and none cleared. No booked instance reads `min_wage_y[2027]` in its amount, so there is no back pay.

## The register

- `wife_net[2027-01]` and `pit37[wife, 2027]` lose `assumed: indeks płac +4% rocznie, założenie prototypu`, `missing record counted by wage_fc_2026` and `held: wage_growth = 0.04 (wage_fc_2026), read for 2027`, all of which came through `zus_cap[2027]` → `proj_wage_y[2027]`.
- `pit37[wife, 2028]` loses `held: wage_growth = 0.04 (wage_fc_2026), read for 2027` only.
- The other lines keep their entries through `health_base_y[2027]` and the 2028+ projections.

## Readings it rests on

- `held` and the printed cut are the ask's K: LR-1.
- The rule-4 law date of an ask line whose top value is not a def cell is K (C2-F7, gap row).
- `wage_growth`, an open-span row read after K, is `held` (LANGUAGE §8.2). `smoke/HH4-b.md` lists it as `assumed` only (C2-F9, gap row).
- The late entry `min_wage_2027` is seen by every cut from 2026-09-12, including `as known 2026-10-19` (C2-F5, gap row). No saved ask uses `as known`.
