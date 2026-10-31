# Change 7: a scenario, "what if one partner stops working" — expected outputs

**Applied as:** a new scenario file, `scenarios/wife_stops.tally`, from 2027-01. The reality is untouched and `on` doesn't move, so K = 2026-10-19. The cells are those of `plan scenario wife_stops` (`amendments/changes/7-scenario-partner-stops.md`, "How to run it").

**Derived by:** C7-a and C7-b; every value agrees. C7-a read the change file's words before deriving, against its brief; C7-b saw them only after working the walk, and re-derived every figure from the base. Both disagree with the words in the same four places.

## Cells

| # | Ask | Key | Before | After | Status |
|---|---|---|---|---|---|
| 1 | wife_net | 2026-10 | 5101.67 | 5101.67 | exact |
| 2 | wife_net | 2027-01 | 5101.67 | **0.00** | exact |
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
| 16 | konto_balance | 2027-06-30, bal | 6760.19 | **6990.77** | exact |
| 17 | konto_balance | 2027-06-30, bal − vat_owed | 6760.19 | **6990.77** | exact |
| 18 | konto_balance | 2027-12-31, bal | 6656.75 | **7131.75** | exact |
| 19 | konto_balance | 2027-12-31, bal − vat_owed | 6656.75 | **7131.75** | exact |
| 20 | net_worth | 2026-12-31 | 170202.16 | 170202.16 | exact |
| 21 | net_worth | 2031-12-31 | not computed (run-only) | lower than the base by more than 301006.53 | run-only |
| 22 | net_worth | 2041-12-31 | not computed (run-only) | lower than the base by more than 913226.93 | run-only |
| 23 | net_worth | 2059-12-31 | not computed (run-only) | lower than the base by more than 2015223.65 | run-only |
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
| 34 | pit37 | wife, 2027 | −2.00 | **0.00** | exact |
| 35 | pit37 | wife, 2028 | −2.00 | **0.00** | exact |
| 36 | health_settlement | 2026 | 996.69 | 996.69 | exact |
| 37 | health_settlement | 2027 | 1036.56 | 1036.56 | exact |
| 38 | health_settlement | 2028 | 1078.02 | 1078.02 | exact |
| 39 | taxes_paid | 2026 | 52272.00 | 52272.00 | exact |
| 40 | taxes_paid | 2027 | 69059.00 | 69059.00 | exact |
| 41 | taxes_paid | 2028 | 68997.00 | **68783.00** | exact |

**41 cells:** 38 exact, 0 holes, 3 run-only; 11 differ from the base (new cells included), 30 are the base's value.

- taxes_paid 2028 falls because `oki_tax[2027]` goes 368 → 152, and the wife's −2 refund is gone.
- ike_left 2027 and 2028 stay 0.00, though IKE now fills on 2027-04-15 and 2028-05-15. OKI gets 0 in March and August 2027.
- `room[2027-01]` is only 638.20 lower; `room[2027-08]` is 0.
- **net_worth, run-only:** the bound is C7-b's, after < before − L. C7-a gives the sign only, and net worth's first differing day, 2027-02-10, the first missing wage.

## First day, and sanity

- **First day anything differs:** 2027-01-15, `invest[ike, 2027-01]` 11410.06 → 10771.86. That day the rule of the 15th measures room up to 2027-02-14, and the window includes the wage of 2027-02-10, now 0.
- **Sanity:** the first day is not before the scenario's `from`, so it passes (`ENGINE.md` §9).

## The advance

None: a scenario has no `known`.

## Warnings and holes

None added and none cleared. konto's lowest from 2027-01-15 to 2028-06-14 is 1684.71, on 2027-08-15; later, run-only. The household has no `check` ask, so the scenario's own checks have nothing to run.

## The register

Every after-result gains `scenario: wife_stops`. The scenario's row `salary[wife_acme] = 0` (version `wife_stops_salary`, span 2027-01-01..) is not listed as `held`: a scenario row isn't knowledge, and the `scenario:` entry stands for it (LR-9). Under 1.0's letter it would be `held`, as C7-b listed it, on wife_net 2027-01, pit37 wife 2027 and 2028, konto_balance 2027 (4), ike_left 2027 and 2028 and taxes_paid 2028; C7-a held those lines on the question. No new `assumed`.

## Readings it rests on

- `held` and the printed cut are the ask's K: LR-1.
