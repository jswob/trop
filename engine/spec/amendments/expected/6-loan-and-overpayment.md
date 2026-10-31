# Change 6: a new loan, and an overpayment — expected outputs

**Applied as:** four amendments.
1. An `advance` of `on`, 2026-10-19 → 2027-04-02.
2. The `add` of `car_loan` (known 2027-04-02): the key `car` in the open index `loan`, its seller, terms and opening.
3. An `advance` of `on`, 2027-04-02 → 2028-01-08.
4. The `add` of `car_overpay_2028_01` (known 2028-01-08), which adds no key.

Final cut K = 2028-01-08. **Step 1** is the state after amendment 2, **step 2** the state after amendment 4 (the composed result).

**Derived by:** C6-a and C6-b, blind to each other; every value agrees, for step 1, step 2 and the composed result.

## Cells (the composed result against the base)

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
| 16 | konto_balance | 2027-06-30, bal | 6760.19 | **7755.40** | exact |
| 17 | konto_balance | 2027-06-30, bal − vat_owed | 6760.19 | **7755.40** | exact |
| 18 | konto_balance | 2027-12-31, bal | 6656.75 | 6656.75 | exact |
| 19 | konto_balance | 2027-12-31, bal − vat_owed | 6656.75 | 6656.75 | exact |
| 20 | net_worth | 2026-12-31 | 170202.16 | 170202.16 | exact |
| 21 | net_worth | 2031-12-31 | not computed (run-only) | lower than the base; after step 2, higher than after step 1 | run-only |
| 22 | net_worth | 2041-12-31 | not computed (run-only) | lower than the base; after step 2, higher than after step 1 | run-only |
| 23 | net_worth | 2059-12-31 | not computed (run-only) | lower than the base; after step 2, higher than after step 1 | run-only |
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
| 41 | taxes_paid | 2028 | 68997.00 | **68973.00** | exact |

**41 cells:** 38 exact, 0 holes, 3 run-only; 6 differ from the base (new cells included), 35 are the base's value.

- **Step 1** makes all three exact changes. **Step 2 against step 1** changes no exact cell; net worth after step 2 is above step 1 (C6-a estimates about +900 at 2031, not keyed).
- konto_balance 2027-06-30 rises by 995.21, one instalment: instalment 3 falls on Saturday 2027-07-10, before the wage of the 12th, and moves June's window minimum.
- taxes_paid 2028 falls because `oki_tax[2027]` goes 368 → 344.
- Unchanged, though the change file says otherwise: konto_balance 2027-12-31, `room[2027-04]` and ike_left 2027.

## The loan

- Payout 40000.00 on 2027-04-10. Annuity 995.21; instalment 1 is 299.67 interest + 695.54 principal.
- The overpayment of 5000.00 at instalment 9, on 2028-01-10, shortens the term: the loan ends at k = 42 on 2030-10-10, so instalments 43..48 vanish, 6 early. The last instalment's amount (about 416.65) is run-only.
- Room after the payout falls by 0, A or 2A by window (A = 995.21): May −A, June −2A, July 0, August −A, September −2A, October 0, November and December −A.

## First day, and sanity

- **First day anything differs:** 2027-04-10 (`loan_out[car]`) for step 1 and for the composed result; 2028-01-10 for step 2 against step 1. Net worth first differs from step 1 on 2028-02-10 (+37.46).
- **Sanity:** neither version has `governs`, so neither has a bound; it passes.

## The advances alone

No value changes. The registers' cut line and `held` entries change as each advance passes.

## Warnings and holes

- **None added and none cleared.** konto's end-of-day balance is 787.71 at the end of 2028-01-10 and its lowest is 431.46 on 2028-01-13. Within 2028-01-10 it passes through −4195.21 after the overpayment, before the wage, but `WARN-NEGATIVE` tests end-of-day balances only (LR-4).
- konto is below `keep_floor` on 2028-01-10..13, which no warning covers: December's room was taken before the overpayment was known.
- No hole. `car_contract` is cited, so there is no `WARN-UNREAD`.

## The register

The adds bring no entry. Each advance changes every cell's cut line and drops the `held` entries for the months it passes.

## Readings it rests on

- `WARN-NEGATIVE` tests end-of-day balances: LR-4.
- `held` and the printed cut are the ask's K: LR-1.
