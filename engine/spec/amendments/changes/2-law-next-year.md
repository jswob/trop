# Change 2: a law change for the coming year (the 2027 figures)

**Statement:** none is needed. This is a law version (OPERATIONS §3), drafted by Claude from two publications.
1. The minimum wage regulation for 2027, published by 15 September 2026.
2. The projected average wage for 2027, announced in December 2026.

**The figures are illustrative.** The base was written knowing 2026 only, and these two are the 2027 rows it projected.

**Operation:** add a law version, twice, as one amendment. **Kind:** `add`.

**Known dates:**
- **Minimum wage:** known 2026-09-12. That is before the base's `on`, so the base simply didn't record it: a late entry, not a back-dated change. It governs 2027 only, and no booked instance reads it, so there is no back pay to state.
- **Projected wage:** known 2026-12-18. `on` advances to it first.

## Diff

```diff
--- reality.tally
+++ reality.tally
-  on 2026-10-19
+  on 2026-12-18

--- law/pl/common.tally
+++ law/pl/common.tally
 source dz_2025_1242 "…"
+source dz_2026_mw "Rozporządzenie RM w sprawie minimalnego wynagrodzenia za pracę w 2027 r. (kwota przykładowa)"
 source health_act "…"
 …
 source mp_2025_1206 "…"
 source mp_2026_117 "…"
+source mp_2026_pw "Obwieszczenie w sprawie prognozowanego przeciętnego wynagrodzenia na 2027 r. (kwota przykładowa)"
 …
 version health_base_2026
   …
+
+version min_wage_2027
+  cite dz_2026_mw
+  governs 2027
+  known 2026-09-12
+  min_wage = 5000 "§ 1"
+
+version proj_wage_2027
+  cite mp_2026_pw
+  governs 2027
+  known 2026-12-18
+  proj_wage = 9800 "pkt 1"
```

The canonical format sorts versions by `known` and then by id. `min_wage_2027` (2026-09-12) sorts after `health_base_2026` (2026-02-10), and `proj_wage_2027` (2026-12-18) comes last. Both are added at the end, so the diff is add-only.

Lines added: 13, plus the `on` line. Lines edited: the `on` line only.

## What it changes, in words

The defs `min_wage_y` and `proj_wage_y` stop projecting 2027. The `published` rule wins wherever the input has a row, and the `projected` rule excepts it only when `missing(…)`.

**Minimum wage**
- 2027 is 5 000.00 instead of the projected 4 998.24.
- Later years project from 5 000: 2028 is 5 200.00 instead of 5 198.17, and so on.

**Projected average wage**
- 2027 is 9 800.00 instead of the projected 9 796.80.
- 2028 becomes 10 192.00 instead of 10 188.67, and so on.

**First day anything differs:** 2027-02-15, which is not before the earliest `governs` (2027-01). That day the rule of the 15th measures room up to 2027-03-14. The window includes the ZUS payment of 2027-02-22, which is now 0.16 zł higher.

**Movements**

| Pay | What changes |
|---|---|
| `zus_social_pay`, preferential months (2027-01 to 2028-08) | The base is 30% of the minimum wage: 1 500.00 in 2027 instead of 1 499.47, and 1 560.00 in 2028 instead of 1 559.45. The 2027 contribution rises 0.16 zł a month (pension 292.80, disability 120.00, sickness 36.75, accident 25.05). |
| `zus_social_pay`, standard months (from 2028-09) | The base is 60% of the projected wage: 6 115.20 instead of 6 113.20 in 2028, and more in later years, compounding. |
| `ryczalt_pay` | The advance deducts the ZUS paid in the month, rounded to whole złoty. Most months are unchanged. A month can move by 1 zł where the rounding tips. |
| `invest[ike, …]` | The 2027 IKE limit is 3 × 9 800 = 29 400 instead of 29 390.40. In the month IKE fills, 9.60 zł more goes to IKE and 9.60 zł less to OKI. Every later month moves by cents through `room`. |
| The wife's pays | No change. Her ZUS cap is 30 × 9 800 = 294 000, far above her 84 000 a year. |

**Balances:** `konto` is lower by cents a month from 2027-02-22, and by a few złoty a month from 2028-09. The wrappers move by the same amounts the other way.

**Asks**
- `social_and_health`: 2027-01 and 2028-09 change by cents and złoty; 2026-10 is unchanged.
- `ike_left`: the 2027 row's limit changes, and the row still ends at 0.
- `taxes_paid`: within a few złoty.
- `pit28`: within a few złoty.
- `net_worth`: moves by small amounts from 2027 on.
- **Unchanged:**
  - `pit37`, `wife_net` and `replay_september`;
  - the health contribution. Its base is a third figure, the Q4 average wage, published in February 2027. It is not part of this change.

**Register:**
- Two 2027 projections leave the register of assumptions: the `projected` rule no longer wins for 2027.
- Later years' projections remain, now from the published 2027 rows.

**Warnings:** none.
