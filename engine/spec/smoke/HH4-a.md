# HH4-a: the household reality worked by hand

Evaluator D8-HH4-a, on 2026-10-19.

**What I read.**
- The language definition: `tmp/design/LANGUAGE.md` and `tmp/design/ENGINE.md`.
- The household reality. These thirteen files under `tmp/design/amendments/household/`, and nothing else:
  - `reality.tally`, `people.tally`, `places.tally`, `pays.tally` and `loans.tally`;
  - `asks/core.tally`;
  - `facts/2026.tally`;
  - `records/mbank/2026-09.tally`;
  - the five law files `law/pl/{benefits,business,common,employment,wrappers}.tally`.

Nothing else was opened.

**Arithmetic.** All of it is one-line `python3 -c` with `Decimal`. The calendar facts (weekdays, holidays) come from the reality's own `holidays_pl` rows and the ISO weekday of each date.

**Frame.**
- The ask cut is K = `on` = 2026-10-19. The run starts at S = 2026-01-01 and ends at E = 2059-12-31.
- One reading fork decides everything after September 2026. It is called **A/B** below (Ambiguity 1):
  - **A**, the literal read rule, makes konto a hole from 2026-10-02;
  - **B**, the evident intent, reads the September bank rows.
- Figures are under B unless marked; the A value is given beside each one.

---

## 1. Steps

### 1.1 Load: indexes, places, openings

The sections used are LANGUAGE §5.6 and §5.9, and ENGINE §3.1.

**Indexes.**
- `month` runs 2026-01..2059-12 and `tax_year` runs 2026..2059.
- `person = [me, wife]`, `job = [wife_acme]` and `wrapper = [ike, oki, mak]`, in that order. The order matters for the state pays in §5.13.2.
- `loan`, `marek_loan`, `marek_part` and `bank_row` are open.

**Places.**
- `konto` is an account in PLN that `holds vat_owed`.
- `gotowka` is cash, `marek` is a claim and `loan_debt[laptop]` is a debt.
- `wcash[w]` are accounts, and `wunits[w]` are holdings of `etf_unit` priced `etf_price`.
- The outside places include `urzad_skarbowy`.

**Openings.**

| Place | Opening |
|---|---|
| konto | 24000 |
| gotowka | 600 |
| wunits[ike] | 150 units |
| wunits[mak] | 80 units |
| everything else | 0 |

### 1.2 Read rule, law dates and cuts

The sections used are LANGUAGE §4.3, §4.5, §4.6 and §5.13.3.

**How a value is read.**
- The law date is the item's `at` date, else its first time key, else the reader's law date.
- A row is a candidate when its version is known ≤ K and its governs span contains the period of the time key, or the date d when the item has none.
- Precedence goes: more stated keys first, then the later `known`, then a row with a span over one without.
- A pay instance's `on` is read at K. Its `when` and amount are read at cut min(t, K). The look-ahead reads at cut min(walk day, K) (ENGINE §4.2).

**Versions that matter by cut:**
- `home_2026` and `forecast_2026` are known by 2025-12-31 (rows `assumed` where marked).
- `dg_2026` (the business) is known on 02-20. So the look-ahead of 2026-02-15 does not see the business. That look-ahead still sees `fee_net` = 0 and `cost` = 0 (the assumed forecast row) and no contributions.
- The OKI law is known on 07-15 (`oki_start` sets oki_from = 2027-01, and `oki_rules`, `oki_2027` and `oki_2028` follow).
- `etf_mark` is 121 from 2026-01 and 126 from 2026-07 (known 07-01).
- The Marek facts are known on their own days. The promise (800 PLN due 2026-12-31) is known on 09-15.
- `mbank_2026_09` is known on 10-02: 19 records, `covers konto`, and stated[konto] = 41386.27 on 09-30.
- `spend_2026_09` is known on 10-02: it sets `spend_window = 2026-09`, with `arrears none for spend`.

**Projected law** (LANGUAGE §5.10, the `missing()` rules in `common.tally`):
- `min_wage_y`, `proj_wage_y` and `health_base_wage` for 2027 and later are each the previous year × (1 + wage_growth), with wage_growth 0.04 `assumed`.
- Each such value enters the register as `missing record counted by <version>`.

| Year | min_wage_y | proj_wage_y | health base |
|---|---|---|---|
| 2027 | 4998.24 | 9796.80 | 9597.79 |
| 2028 | 5198.17 | 10188.67 | 9981.70 |

### 1.3 Flow pass: amounts and dates

The sections used are ENGINE §4.1 and LANGUAGE §5.10, §5.13 and §7.2. The business-day rule is §4.2, using `holidays_pl`.

**Wife's wage** (`employment.tally`).

| Item | Amount |
|---|---|
| Gross salary | 7000 |
| Employee ZUS: 683.20 + 105.00 + 171.50 | 959.70 |
| Health: (7000 − 959.70) × 0.09 | 543.63 |
| PIT base: round(7000 − 959.70 − 250) | 5790 |
| PIT advance | 395 |
| **Net** | **5101.67** |

- The October advance is pit_scale(57900) − pit_scale(52110) − 300 = 394.8, which rounds to 395. January's is 5790 × 0.12 − 300 = 394.8, also 395.
- Year to date, 63000 is far under zus_cap = 30 × proj_wage_y.
- The wage is paid on next_business(10th of m+1).

**Business** (`business.tally`), from 2026-03.

Revenue:
- Fees are 14000 + 3000 = 17000 a month.
- VAT out is 23%. client_a pays 17220 after 14 days and client_b pays 3690 after 30 days. Each pay_day is next_business(last day of m + terms).

Contributions:
- **ZUS stages:**
  - relief runs 03–08 2026, so zus_social is 0;
  - preferential runs 2026-09..2028-08, on a base of min_wage_y × 0.3;
  - standard follows, on a base of proj_wage_y × 0.6.

| Year | Preferential zus_social |
|---|---|
| 2026 | 456.18 |
| 2027 | 474.44 |
| 2028 | 493.41 |

- **Health** is health_month at the rev_ytd tier. The low tier (0.6 × base × 0.09) applies up to 60000 of revenue year to date, and the mid tier (1 × base × 0.09) above that. So the low tier covers 2026-03..05 and Jan–Mar of later years, and the mid tier the rest.

| Year | Low | Mid |
|---|---|---|
| 2026 | 498.35 | 830.58 |
| 2027 | 518.28 | 863.80 |
| 2028 | 539.01 | 898.35 |

Monthly taxes:
- **ryczalt_adv[m]** = round(round(rev − zus flows in m − 0.5 × health flows in m, 1) × 0.12, 1).

| Year | Advances by month of the year they are for |
|---|---|
| 2026 | 2040 (Mar), 2010 (Apr–Jun), 1990 (Jul–Sep), 1935 (Oct–Dec) |
| 2027 | 1935 (Jan), 1952 (Feb–Apr), 1931 (May–Dec) |
| 2028 | 1931 (Jan), 1948 (Feb–Apr), 1927 (May–Sep), 1736 (Oct–Dec) |

- **vat_due** = 3910 − 92 = 3818, paid on vat_day = next_business(25th of m+1).

Pay days:
- Health, ZUS and ryczałt are paid on next_business(20th of m+1).
- Costs of 492 are paid on next_business(5th).

**Other pays.**
- Rent is 3200 on the 1st. It is `prices 2026 by cpi`, so 2028 rent is 3296.
- Spend is daily.
  - The budget is 4600 a month (assumed), which gives 148.39, 164.29 and 153.33 a day for 31-, 28- and 30-day months.
  - Under B, from 2026-10-02 the learned figure is 3574.02 × cpi ratio. That gives 115.29 / 119.13 / 127.64 a day in 2026–27 (31 / 30 / 28 days), and 118.75 / 122.71 / 126.94 in 2028 (31 / 30 / 29 days).
- The laptop loan (`loans.tally`) pays out +3000 on 2026-05-05. Instalments of 300 at 0% fall on the 5th from 2026-06 to 2027-03, with no business-day shift.
- Marek: konto lends −2000 on 03-12 and gets back +700 on 06-30 and +500 on 09-15. The final +800 on 2026-12-31 is `promised`.
- pit28_pay and pit37_pay fall on return_day, which is next_business(30 April of y+1). Refunds fall on return_day + `refund_days` (45 days), so the wife's 2026 refund is on 2027-06-14.
- health_settle is paid on next_business(start of y + 16 months + 19 days), which is 2027-05-20 for 2026.
- oki_tax_pay is on 2028-05-31 for 2027.

### 1.4 The day walk, January–August 2026

The sections used are ENGINE §4.2 and LANGUAGE §5.13.1–§5.13.2.

**State pays.** invest[w, m] runs on the 15th at phase 6. buy_units (phase 4) and buy_cash (phase 5) run on the 16th, with units = round(bal(wcash)/price, 1, down).

**Room.** room = max(0, plan_low(konto, 15th, 14th of next month) − keep_floor), where keep_floor is 1000. plan_low is defined as:

plan_low = bal(konto) − P(t) + min over u in [a, b] of (F(u) − F(t) − h(u))

- F counts the flow pays only, as known at cut min(t, K). State and promised pays are left out.
- P(t) is the promised movements booked so far that no record confirms.
- h is vat_owed. The 3220 of client_a's VAT is held from its pay_day until vat_day. client_b always pays after vat_day, so its VAT is never held.

**Reading room.** room is walk-bound (LANGUAGE §5.13.1 and ENGINE §3.2), so it is evaluated again at each reader's point:
- ike takes min(room, max(0, ike_limit − flows(invest[ike], year start..today))), using same-day totals;
- oki takes room if oki_from applies, else 0;
- mak takes 0 if oki_from applies, else room.

The `if` evaluates only the branch it takes.

B is konto at phase 6 on the 15th.

| 15th of | B | Window min (day) | Room | Invest |
|---|---|---|---|---|
| Jan | 18574.15 | −7052.85 (02-09) | 10521.30 | ike |
| Feb | 5115.93 | −6671.28 | 0 | — (the cut of 02-15 does not see dg_2026) |
| Mar | 163.98 | −7446.21 | 0 | — |
| Apr | 14119.46 | −10142.20 | 2977.26 | ike |
| May | 22579.72 | −10382.56 | 11197.16 | ike |
| Jun | 22401.69 | −10263.81 | 11137.88 | ike 3564.28 (fills 28260), then mak 7573.60 |
| Jul | 23131.33 | −10670.33 | 11461.00 | mak |
| Aug | 22431.33 | −10694.79 | 10736.54 | mak |

- konto at the end of 2026-08-31 is 6371.97, checked by totals.
- WARN-NEGATIVE fires from 2026-03-01 (back above zero on 03-10). It fires again from 2026-03-17 (back above zero on 04-14). See LANGUAGE §5.9 and ENGINE §4.2 step 5.
- ike_limit for 2026 is 3 × 9420 = 28260.

**Units.**

| Wrapper | Buys (units) | Cash left |
|---|---|---|
| ike | 86 (Jan), 25 (Apr), 93 (May), 29 (Jun) | 67.00 after June |
| mak | 62 (Jun), 91 (Jul), 85 (Aug) | 93.14 after August |

- The buys are at 121 until June and at 126 from July.

### 1.5 Back pay

The sections used are LANGUAGE §5.13.3 and ENGINE §5.

- Versions known after S that change booked flow-pay instances are `dg_2026` (02-20), the OKI law (07-15), `etf_mark` 126 (07-01), the Marek rows and `spend_2026_09` (10-02).
- `dg_2026` makes the business pays exist only from 2026-03, so none of them was booked before it.
- `spend_2026_09` would re-measure the spend of Jan–Aug, but it carries `arrears none for spend`, so there is no movement.
- No HOLE-CHANGED arises.
- State pays are never re-measured.

### 1.6 September: records, modelled past and anchor

The sections used are LANGUAGE §5.18, and ENGINE §4.2 steps 1, 3 and 4, §6.1 and §6.2.

**Booked side.**
- September is covered for konto, and 09-30 is before K. So the 19 records are booked, and the model's pays that touch konto are modelled past and not booked.
- The records net to +2917.07: 22821.67 in and 19904.60 out.
- The booked balance on 09-30 is 6371.97 + 2917.07 = **9289.04**. The anchor 41386.27 replaces it at the close of 09-30.

**Modelled invest on 09-15.**
- B at ike's point is 24408.12 and room is 12535.30. ike = min(room, 28260 − 28260) = 0.
- The −6000 record explains invest[ike, 09], so it is booked at that instance's point after it is computed (ENGINE §4.2 step 3). That leaves room 6535.30 at mak's point, so the modelled invest[mak, 09] is 6535.30.

**Places that are not covered.**
- wcash[ike] does get the record's +6000 as the instance's movement.
- wcash[mak] gets nothing, because its instance is modelled past.
- The 2026-09 loan instalment has no record, so loan_debt is never reduced for it.
- The 09-16 buys run against wcash, which is not covered: ike buys 48 units at 126 (cash left 19.00) and mak buys 0.

### 1.7 Residual ledger, konto, 2026-09

The sections used are ENGINE §6.3 and LANGUAGE §5.16.

**Matched lines**, each a difference of 0.00 with timing 0:
- rent_pay[2026-09], costs_pay[2026-09] and wage[wife_acme, 2026-08];
- client_pay[client_a, 2026-08] and marek_back[back2];
- health_pay[2026-08], ryczalt_pay[2026-08] and vat_pay[2026-08].

**Matched with a difference.** invest[ike, 2026-09]: model 0, record −6000.00, difference −6000.00.

**Expected but missing** (−model):

| Instance | Day | Difference |
|---|---|---|
| client_pay[client_b, 2026-08] | 09-30 | −3690.00 |
| loan_instalment[laptop, 4] | 09-05 | +300.00 |
| invest[mak, 2026-09] | 09-15 | +6535.30 |
| spend[2026-09-01..30] | daily | 30 lines of +153.33, totalling +4599.90 |

**Nothing printed** for the zero-amount loan_interest and invest[oki, 2026-09].

**Unexplained.** Ten records, totalling −3574.02: groceries 1979.03, transport 430.00, health_care 145.00 and other 1019.99.

**Balance line.** 41386.27 − 9289.04 = **+32097.23**.

**Groups by absolute difference.** Ties go by month, then group name.

| Group | Difference |
|---|---|
| balance | +32097.23 |
| spend | +4599.90 |
| client_pay | −3690.00 |
| unexplained | −3574.02 |
| invest | +535.30 |
| loan_instalment | +300.00 |
| costs_pay | 0.00 |
| health_pay | 0.00 |
| marek_back | 0.00 |
| rent_pay | 0.00 |
| ryczalt_pay | 0.00 |
| vat_pay | 0.00 |
| wage | 0.00 |

### 1.8 October–December 2026

The sections used are LANGUAGE §5.18 (`covered`, the learned rule) and §4.6, and ENGINE §4.2.

**Spend.**
- spend on 10-01 is 148.39: its cut is 10-01, and it reads the budget.
- From 10-02, spend_2026_09 is visible and covered(konto, 2026-09) is true, so the learned rule wins.
  - Under B the daily spend is 115.29 (31-day months) or 119.13 (30-day months).
  - Under A it is a hole (Ambiguity 1).

**15 October.**
- B = 41386.27 − 3200 − 148.39 − 14 × 115.29 − 492 − 300 + 5101.67 + 17220 = 57953.49.
- The window minimum is −10313.57, on 11-09, so room = 46639.92.
- ike gets 0, because flows(invest[ike], 2026) = 28260 + 6000 = 34260. mak gets 46639.92 and buys 370 units.

**konto at 2026-10-31** = 1000 + (−5249.40) + 10313.57 = **6064.17**. vat_owed is 0 that day: client_a's September VAT is paid on 10-26, and its October invoice is paid on 11-16.

**15 November** (explain_room).
- B = 1000 + 5101.67 − 6 × 119.13 = 5386.89.
- client_a's October invoice is paid on 11-16 (11-14 is a Saturday), so nothing is held on 11-15. The window's lowest end-of-day value is 0, on 11-15 itself: later days stay above it (11-24: 9706.07; 12-09: 7053.68).
- **room[2026-11] = 4386.89.** mak takes 4386.89 and buys 35 units.

**15 December.**
- B = 1000 + 7053.68 + 5101.67 + 17220 − 6 × 115.29 = 29683.61.
- The window minimum is −10339.30, on 2027-01-10. The promised +800 of 12-31 is left out.
- room = 18344.31, all to mak (146 units, cash left 38.26).

**konto at 2026-12-31** = 1000 − 5194.40 + 10339.30 + 800 (the promised pay is booked) = **6944.90**. vat_owed is 0.

### 1.9 The walk through 2027 (B)

The sections used are LANGUAGE §5.13 (`promised`) and §5.13.1, ENGINE §4.2 (P(t)), and the OKI law.

**The promised 800.**
- From 2026-12-31, marek_last's +800 is booked with no record confirming it. So every plan_low takes it out (P = 800).
- The 2027-01 room is therefore B 23545.22 − 800 − 10335.16 − 1000 = 11410.06.
- After that, konto sits at 1800 on each window's lowest day, and the rooms equal the plain "floor 1000" figures.

**Who takes the room.** In 2027 oki_from applies: ike first up to ike_limit 29390.40, then oki, and mak 0.

**Other dated pays in the 2027 windows:**
- pit28_pay[2026] of 2 on 04-30;
- health_settle_pay[2026] of 996.69 on 05-20;
- pit37_refund[wife, 2026] of +2 on 06-14.

**The October window.** client_b's September pay falls on 11-02, after the 11-01 rent, because 10-30 is a Saturday and 11-01 a holiday. So its minimum is on 11-01.

**The August window.** client_a's July pay falls on 08-16 (08-14 is a Saturday and 08-15 a holiday), so the minimum is 0 on 08-15 itself.

| 15th of | B (floor 1000 basis) | Min (day) | Room | To |
|---|---|---|---|---|
| Jan | 23545.22 (P 800) | −10335.16 (02-09) | 11410.06 | ike |
| Feb | 22555.83 | −11720.33 (03-01) | 9835.50 | ike |
| Mar | 24605.61 | −9919.79 (04-11) | 13685.82 | ike 8144.84 (fills 29390.40), oki 5540.98 |
| Apr | 22845.15 | −9591.28 (05-09) | 12253.87 | oki |
| May | 22629.93 | −11023.74 (06-09) | 10606.19 | oki |
| Jun | 22608.89 | −10144.38 (07-11) | 11464.51 | oki |
| Jul | 22860.51 | −9971.49 (08-09) | 11889.02 | oki |
| Aug | 5409.93 | 0 (08-15) | 4409.93 | oki |
| Sep | 29820.84 | −10029.09 (10-10) | 18791.75 | oki |
| Oct | 22745.22 | −12251.01 (11-01) | 9494.21 | oki |
| Nov | 24851.85 | −9913.80 (12-09) | 13938.05 | oki |
| Dec | 22629.93 | −10098.63 (2028-01-09) | 11531.30 | oki |

- From 2027-02, the actual B is the column + 800, and P = 800 is taken back out.

**Year-end and mid-year balances.**
- konto at 2027-06-30 = 1800 − 5184.19 + 10144.38 = **6760.19**.
- konto at 2027-12-31 = 1800 − 5241.88 + 10098.63 = **6656.75**.
- vat_owed is 0 on both days: client_b's May and November invoices are paid after their vat_day, and client_a's June and December invoices are paid later.
- ike_left[2027] = 29390.40 − 29390.40 = **0.00**.

### 1.10 January–March 2028 (B)

The sections and rules are the same as in 1.9.

- ike_limit for 2028 is 3 × 10188.67 = 30566.01.

| 15th of | B | Min (day) | Room | ike |
|---|---|---|---|---|
| Jan | 22609.17 | −10227.70 (02-09) | 11381.47 | 11381.47 |
| Feb | 22560.03 | −9725.33 (03-09) | 11834.70 | 11834.70 (total 23216.17) |
| Mar | 22609.17 | −9900.81 (04-09) | 11708.36 | 7349.84, which fills the limit |

- ike_left[2028] = **0.00**.

### 1.11 Annual figures

These use the business and employment laws, and `flows` as defined in LANGUAGE §5.13.3.

**pit28_due[y]** = pit28_tax − the ryczałt advances for y's months. pit28_tax = round(round(revenue − flows(zus_social_pay, y) − 0.5 × flows(health_pay, y), 1) × 0.12, 1).

| Year | Revenue | ZUS flows | Health flows | Before rounding | Tax | Advances | Due |
|---|---|---|---|---|---|---|---|
| 2026 | 170000 | 1368.54 | 6478.53 | 165392.195 | 19847 | 19845 | **2.00** |
| 2027 | 204000 | 5675.02 | 9295.82 | 193677.07 | 23241 | 23239 | **2.00** |
| 2028 | 204000 | 10673.69 | 9667.63 | 188492.495 | 22619 | 22618 | **1.00** |

**health_settle[y]** = health_year − health_paid.

| Year | health_year | health_paid | Settlement |
|---|---|---|---|
| 2026 | 10 × 830.58 = 8305.80 | 7309.11 | **996.69** |
| 2027 | 12 × 863.80 = 10365.60 | 9329.04 | **1036.56** |
| 2028 | 12 × 898.35 = 10780.20 | 9702.18 | **1078.02** |

**pit37_due[p, y].**
- For the wife each year: base = round(84000 − 11516.40 − 3000) = 69484; tax = round(69484 × 0.12 − 3600) = 4738; advances are 12 × 395 = 4740. So her due is **−2.00**.
- `me` has no income, so mine is **0.00**.

**taxes_paid[y]** = flows(urzad_skarbowy, y), covering ryczałt, VAT, PIT-28, PIT-37 and the OKI tax by the date each is paid.

| Year | Ryczałt | VAT | PIT-28 | PIT-37 refund | OKI tax | Total |
|---|---|---|---|---|---|---|
| 2026 | 17910 | 34362 | — | — | — | **52272.00** |
| 2027 | 23243 | 45816 | +2 | −2 | — | **69059.00** |
| 2028 | 22813 | 45816 | +2 | −2 | +368 | **68997.00** |

### 1.12 The 2027 OKI tax (B)

These are the OKI rules in `wrappers.tally`; days(a, b) is from LANGUAGE §4.2.

- oki_holds is foreign, so oki_exempt is 0. oki_rate[2027] is 0.0085.
- etf_price is constant through 2027 at 126 × 1.06 = 133.56. A buy turns cash into units at that price, so worth + cash on each day equals the total invested in OKI by that day.
- oki_avg[2027] = Σ (invest × days left in the year, counting its own day) / days(2027-01-01, 2028-01-01). The divisor is 365, because end_of is the first day after the period.

| Month | Invest | Days |
|---|---|---|
| Mar | 5540.98 | 292 |
| Apr | 12253.87 | 261 |
| May | 10606.19 | 231 |
| Jun | 11464.51 | 200 |
| Jul | 11889.02 | 170 |
| Aug | 4409.93 | 139 |
| Sep | 18791.75 | 108 |
| Oct | 9494.21 | 78 |
| Nov | 13938.05 | 47 |
| Dec | 11531.30 | 17 |

- The sum is 15814449.62, so oki_avg = 43327.259…
- oki_tax = round(368.28, 1) = **368**, paid on 2028-05-31.

### 1.13 net_worth at 2026-12-31 (B)

This uses LANGUAGE §5.9 (kinds, worth) and §4.6 (reading at a date).

| Place | Value |
|---|---|
| konto | 6944.90 |
| gotowka | 600 |
| marek | 0 |
| loan_debt[laptop] | −1200 |
| wcash[ike] | 19.00 |
| wcash[mak] | 38.26 |
| wcash[oki] | 0 |
| wunits[ike] | 431 × 126 = 54306 |
| wunits[mak] | 869 × 126 = 109494 |
| **Total** | **170202.16** |

- loan_debt is −1200 because 3000 − 6 × 300 = 1200 is still owed: September's instalment was modelled past and never booked.
- wunits[ike] is 150 + 86 + 25 + 93 + 29 + 48 = 431 units. wunits[mak] is 80 + 62 + 91 + 85 + 370 + 35 + 146 = 869 units.

### 1.14 explain room[2026-11] (B)

These are LANGUAGE §5.16 (`explain`) and §8.2.

**Walk points.** Two state pays read room[2026-11]:
- (2026-11-15, phase 6, invest[ike, 2026-11]) gives 4386.89. ike takes min(4386.89, max(0, 28260 − 34260)) = 0.
- (2026-11-15, phase 6, invest[mak, 2026-11]) gives 4386.89, all of which mak takes.

invest[oki, 2026-11] does not read it, because its `if` takes the else branch.

**Rules.** The room rule won. `keep_floor = 1000` comes from `home_2026`.

The inputs to plan_low:

| Input | Value |
|---|---|
| bal | 5386.89 |
| P | 0 |
| Lowest point | 0 on 11-15 |
| Window | 11-15..12-14 |

**Register:**
- law date 2026-11-01, cut 2026-10-19;
- `hold: konto holds vat_owed`;
- `held:` rows. These have open spans and are read for periods after K: salary, the employee and contribution rates, fee_net, cost_net and cost_vat, vat_rate, rent, health_rate and health_base_wage 2026;
- `assumed: holidays [] from 2031`. vat_owed's `where` evaluates pay_day and vat_day for every month to 2059;
- whatever further entries come from the `when`s of instances outside the window (Ambiguity 3).

---

## 2. Where I stopped, and why

- **net_worth at 2031-12-31, 2041-12-31 and 2059-12-31: not computed.** Each needs every monthly window of the walk up to that date: 60, 180 and 396 windows. Each window has about a dozen dated flows, its own calendar and holidays (with `[]` assumed from 2031), and changing ZUS, health and ryczałt figures. Each also needs the integer unit buys of three wrappers at a yearly price, and a yearly OKI tax with its own daily average. The machinery is the one shown in 1.4–1.12, and nothing in it is undefined. It is only too long to carry by hand in one-line arithmetic.
  - My hand run reaches 2028-03, plus the 2027 year-end.
  - Under A all four dates are holes.
- **Everything else asked is computed.** That includes konto_balance at 2027-06-30 and 2027-12-31, and taxes_paid 2028 (through oki_tax[2027]).

---

## 3. Arithmetic (one-line commands with their output)

**Provenance.** The first part of this run was compacted out of my context. The one-liners behind the January–August 2026 window minima, the January–March 2027 minima, the January 2028 minimum and the 2026 unit buys are not reproduced word for word here. Their results are carried, and re-checked below through the room arithmetic and the chain from the anchored September balance. Every other figure was computed by the commands shown.

**Wife net (1.3).**
```
$ python3 -c "from decimal import Decimal as D, ROUND_HALF_UP as H; q=lambda x,s='0.01': x.quantize(D(s),H); s=D(7000); z=q(s*D('0.0976'))+q(s*D('0.015'))+q(s*D('0.0245')); h=q((s-z)*D('0.09')); b=q(s-z-250,'1'); oct_adv=q((D(57900)*D('0.12')-3600)-(D(52110)*D('0.12')-3600)-300,'1'); jan_adv=q(b*D('0.12')-300,'1'); print('emp_zus',z,'health',h,'pit_base',b,'adv oct',oct_adv,'adv jan',jan_adv,'net oct',s-z-h-oct_adv,'net jan',s-z-h-jan_adv)"
emp_zus 959.70 health 543.63 pit_base 5790 adv oct 395 adv jan 395 net oct 5101.67 net jan 5101.67
```

**Contributions (1.3).**
```
$ python3 -c "from decimal import Decimal as D, ROUND_HALF_UP as H; q=lambda x: x.quantize(D('0.01'),H); b6=q(D(4806)*D('0.3')); b7m=q(D(4806)*D('1.04')); b7=q(b7m*D('0.3')); p8=q(D(9420)*D('1.04')*D('1.04')); b8=q(p8*D('0.6')); hb7=q(D('9228.64')*D('1.04')); hb8=q(D('9228.64')*D('1.04')*D('1.04')); print('2026-10 base',b6,'parts',q(b6*D('0.1952')),q(b6*D('0.08')),q(b6*D('0.0245')),q(b6*D('0.0167')),'zus',q(b6*D('0.1952'))+q(b6*D('0.08'))+q(b6*D('0.0245'))+q(b6*D('0.0167')),'health mid',q(D('9228.64')*D('0.09'))); print('2027-01 min_wage_y',b7m,'base',b7,'parts',q(b7*D('0.1952')),q(b7*D('0.08')),q(b7*D('0.0245')),q(b7*D('0.0167')),'zus',q(b7*D('0.1952'))+q(b7*D('0.08'))+q(b7*D('0.0245'))+q(b7*D('0.0167')),'health base',hb7,'low',q(hb7*D('0.6')*D('0.09'))); print('2028-09 proj_wage_y',p8,'base',b8,'parts',q(b8*D('0.1952')),q(b8*D('0.08')),q(b8*D('0.0245')),q(b8*D('0.0167')),q(b8*D('0.0245')),'zus',q(b8*D('0.1952'))+q(b8*D('0.08'))+q(b8*D('0.0245'))+q(b8*D('0.0167'))+q(b8*D('0.0245')),'health base',hb8,'mid',q(hb8*D('0.09')))"
2026-10 base 1441.80 parts 281.44 115.34 35.32 24.08 zus 456.18 health mid 830.58
2027-01 min_wage_y 4998.24 base 1499.47 parts 292.70 119.96 36.74 25.04 zus 474.44 health base 9597.79 low 518.28
2028-09 proj_wage_y 10188.67 base 6113.20 parts 1193.30 489.06 149.77 102.09 149.77 zus 2083.99 health base 9981.70 mid 898.35
```

**Annual figures (1.11).**
```
$ python3 -c "from decimal import Decimal as D, ROUND_HALF_UP as H; q=lambda x: x.quantize(D('1'),H); z6=3*D('456.18'); h6=3*D('498.35')+6*D('830.58'); z7=D('456.18')+11*D('474.44'); h7=D('830.58')+3*D('518.28')+8*D('863.80'); z8=D('474.44')+8*D('493.41')+3*D('2083.99'); h8=D('863.80')+3*D('539.01')+8*D('898.35'); t6=q(q(170000-z6-h6*D('0.5'))*D('0.12')); t7=q(q(204000-z7-h7*D('0.5'))*D('0.12')); t8=q(q(204000-z8-h8*D('0.5'))*D('0.12')); a6=2040+3*2010+3*1990+3*1935; a7=1935+3*1952+8*1931; a8=1931+3*1948+5*1927+3*1736; print('2026 zus',z6,'health',h6,'pre',170000-z6-h6*D('0.5'),'tax',t6,'adv',a6,'due',t6-a6); print('2027 zus',z7,'health',h7,'pre',204000-z7-h7*D('0.5'),'tax',t7,'adv',a7,'due',t7-a7); print('2028 zus',z8,'health',h8,'pre',204000-z8-h8*D('0.5'),'tax',t8,'adv',a8,'due',t8-a8)"
2026 zus 1368.54 health 6478.53 pre 165392.195 tax 19847 adv 19845 due 2
2027 zus 5675.02 health 9295.82 pre 193677.070 tax 23241 adv 23239 due 2
2028 zus 10673.69 health 9667.63 pre 188492.495 tax 22619 adv 22618 due 1
$ python3 -c "... (settlements, pit37, taxes) ..."
settle 2026 8305.80 7309.11 996.69
settle 2027 10365.60 9329.04 1036.56
settle 2028 10780.20 9702.18 1078.02
pit37 wife base 69484 tax 4738 adv 4740 due -2
taxes 2026 17910 34362 52272
taxes 2027 23243 45816 69059
taxes 2028 22813 45816 68997
```

The elided command, in full:
```
python3 -c "from decimal import Decimal as D, ROUND_HALF_UP as H; q=lambda x: x.quantize(D('1'),H); print('settle 2026',10*D('830.58'),3*D('498.35')+7*D('830.58'),10*D('830.58')-3*D('498.35')-7*D('830.58')); print('settle 2027',12*D('863.80'),3*D('518.28')+9*D('863.80'),12*D('863.80')-3*D('518.28')-9*D('863.80')); print('settle 2028',12*D('898.35'),3*D('539.01')+9*D('898.35'),12*D('898.35')-3*D('539.01')-9*D('898.35')); b=q(84000-12*D('959.70')-3000); t=q(b*D('0.12')-3600); print('pit37 wife base',b,'tax',t,'adv',12*395,'due',t-12*395); print('taxes 2026',2040+3*2010+3*1990+2*1935,9*3818,2040+3*2010+3*1990+2*1935+9*3818); print('taxes 2027',1935+1935+3*1952+7*1931,12*3818,1935+1935+3*1952+7*1931+12*3818+2-2); print('taxes 2028',1931+1931+3*1948+5*1927+2*1736,12*3818,1931+1931+3*1948+5*1927+2*1736+12*3818+2-2+368)"
```

**September and the October–December 2026 chain (1.6–1.8).**
```
$ python3 -c "from decimal import Decimal as D; booked=D('6371.97')+D('2917.07'); print('booked 09-30',booked,'balance line',D('41386.27')-booked,'spend',30*D('153.33'),'invest group',-6000+D('6535.30'),'unexplained',-(D('1979.03')+D('430.00')+D('145.00')+D('1019.99')),'room at mak',D('12535.30')-6000)"
booked 09-30 9289.04 balance line 32097.23 spend 4599.90 invest group 535.30 unexplained -3574.02 room at mak 6535.30
$ python3 -c "from decimal import Decimal as D; b=D('115.29'); a=D('119.13'); W=D('5101.67'); B=D('41386.27')-3200-D('148.39')-14*b-492-300+W+17220; f=D('830.58')+D('456.18')+1990; r31=-16*b-f-3818+3690; r09=r31-3200-492-300-9*a; print('OCT B',B,'rel 10-31',r31,'rel 11-09',r09,'room',B+r09-1000,'bal 10-31',1000+r31-r09,'B 11-15',1000+W-6*a)"
OCT B 57953.49 rel 10-31 -5249.40 rel 11-09 -10313.57 room 46639.92 bal 10-31 6064.17 B 11-15 5386.89
$ python3 -c "from decimal import Decimal as D; b=D('115.29'); a=D('119.13'); W=D('5101.67'); f=D('830.58')+D('456.18')+1935; r24=-9*a+17220-f-3220; r25=-10*a+17220-f-3818; r30=-15*a+17220-f-3818+3690; r01=r30-b-3200; r09=r01-8*b-300-492; print('NOV B',1000+W-6*a,'rel 11-15',0,'11-24',r24,'11-25',r25,'11-30',r30,'12-01',r01,'12-09',r09,'room',1000+W-6*a-1000,'B 12-15',1000+r09+W+17220-6*b)"
NOV B 5386.89 rel 11-15 0 11-24 9706.07 11-25 8988.94 11-30 12083.29 12-01 8768.00 12-09 7053.68 room 4386.89 B 12-15 29683.61
$ python3 -c "from decimal import Decimal as D; b=D('115.29'); W=D('5101.67'); B=D('29683.61'); f=D('830.58')+D('456.18')+1935; r27=-12*b-f-3220; r28=-13*b-f-3818; r31=-16*b-f-3818+3690; r10=r31-3200-492-300-10*b; print('DEC B',B,'12-27',r27,'12-28',r28,'12-31',r31,'01-10',r10,'room',B+r10-1000,'bal 12-31 with promised 800',1000+r31-r10+800,'B 2027-01-15',1000+800+W+17220-5*b)"
DEC B 29683.61 12-27 -7825.24 12-28 -8538.53 12-31 -5194.40 01-10 -10339.30 room 18344.31 bal 12-31 with promised 800 6944.90 B 2027-01-15 23545.22
```

**2026 rooms, IKE and net worth (1.4, 1.9, 1.10, 1.13).**
```
$ python3 -c "from decimal import Decimal as D; print('rooms 2026: jan',D('18574.15')-D('7052.85')-1000,'feb',max(0,D('5115.93')-D('6671.28')-1000),'mar',max(0,D('163.98')-D('7446.21')-1000),'apr',D('14119.46')-D('10142.20')-1000,'may',D('22579.72')-D('10382.56')-1000,'jun',D('22401.69')-D('10263.81')-1000,'jul',D('23131.33')-D('10670.33')-1000,'aug',D('22431.33')-D('10694.79')-1000); ike=D('10521.30')+D('2977.26')+D('11197.16'); print('ike jun',28260-ike,'mak jun',D('11137.88')-(28260-ike),'flows ike 2026',28260+6000,'ike_left 2026',28260-(28260+6000))"
rooms 2026: jan 10521.30 feb 0 mar 0 apr 2977.26 may 11197.16 jun 11137.88 jul 11461.00 aug 10736.54
ike jun 3564.28 mak jun 7573.60 flows ike 2026 34260 ike_left 2026 -6000
$ python3 -c "from decimal import Decimal as D; print('net worth 2026-12-31',D('6944.90')+600+0-1200+D('19.00')+D('38.26')+431*126+869*126,'units ike',150+86+25+93+29+48,'units mak',80+62+91+85+370+35+146); print('ike 2027',D('29390.40')-(D('11410.06')+D('9835.50')+D('8144.84')),'rooms 2027 jan feb mar',D('23545.22')-800-D('10335.16')-1000,D('22555.83')-D('11720.33')-1000,D('24605.61')-D('9919.79')-1000); print('ike 2028',D('30566.01')-(D('11381.47')+D('11834.70')+D('7349.84')),'limit 2028',3*D('10188.67'),'jan 2028 room',D('22609.17')-D('10227.70')-1000)"
net worth 2026-12-31 170202.16 units ike 431 units mak 869
ike 2027 0.00 rooms 2027 jan feb mar 11410.06 9835.50 13685.82
ike 2028 0.00 limit 2028 30566.01 jan 2028 room 11381.47
```

**2027 windows, April–December (1.9).**
- a is the spend of a 30-day month and b of a 31-day month (j is 2028's).
- W is the wage.
- t is the 20th-of-month contributions plus ryczałt.
- rNN is F(u) − F(t) − h(u).
```
$ python3 -c "from decimal import Decimal as D; a=D('119.13'); b=D('115.29'); B=1000+D('5101.67')+17220-4*a; t=D('518.28')+D('474.44')+1952; r20=-5*a-t-3220; r25=-10*a-t-3220; r26=-11*a-t-3818; r29=-14*a-t-3818; r30=r29-a+3690-2; r01=r30-b-3200; r05=r01-4*b-492; r09=r05-4*b; m=min(r20,r25,r26,r29,r30,r01,r05,r09); print('APR B',B,'r04-20',r20,'r04-25',r25,'r04-26',r26,'r04-29',r29,'r04-30',r30,'r05-01',r01,'r05-05',r05,'r05-09',r09,'min',m,'room',B+m-1000); B=1000+D('5101.67')+17220-6*b; t=D('863.80')+D('474.44')+1952+D('996.69'); s20=-5*b-t-3220; s24=-9*b-t-3220; s25=-10*b-t-3818; s30=-15*b-t-3818; s31=s30-b+3690; s01=s31-a-3200; s07=s01-6*a-492; s09=s07-2*a; m=min(s20,s24,s25,s30,s31,s01,s07,s09); print('MAY B',B,'r05-20',s20,'r05-24',s24,'r05-25',s25,'r05-30',s30,'r05-31',s31,'r06-01',s01,'r06-07',s07,'r06-09',s09,'min',m,'room',B+m-1000)"
APR B 22845.15 r04-20 -6760.37 r04-25 -7356.02 r04-26 -8073.15 r04-29 -8430.54 r04-30 -4861.67 r05-01 -8176.96 r05-05 -9130.12 r05-09 -9591.28 min -9591.28 room 12253.87
MAY B 22629.93 r05-20 -8083.38 r05-24 -8544.54 r05-25 -9257.83 r05-30 -9834.28 r05-31 -6259.57 r06-01 -9578.70 r06-07 -10785.48 r06-09 -11023.74 min -11023.74 room 10606.19
$ python3 -c "from decimal import Decimal as D; a=D('119.13'); b=D('115.29'); W=D('5101.67'); t=D('863.80')+D('474.44')+1931; B=1000+W+17220+2-6*a; r21=-6*a-t-3220; r24=-9*a-t-3220; r25=-10*a-t-3818; r29=-14*a-t-3818; r30=r29-a+3690; r01=r30-b-3200; r05=r01-4*b-492; r11=r05-6*b; m=min(r21,r24,r25,r29,r30,r01,r05,r11); print('JUN B',B,'r06-21',r21,'r06-24',r24,'r06-25',r25,'r06-29',r29,'r06-30',r30,'r07-01',r01,'r07-05',r05,'r07-11',r11,'min',m,'room',B+m-1000,'bal 06-30',1000+r30-m); B=1000+W+17220-4*b; s20=-5*b-t-3220; s25=-10*b-t-3220; s26=-11*b-t-3818; s29=-14*b-t-3818; s30=s29-b+3690; s01=s30-2*b-3200; s05=s01-4*b-492; s09=s05-4*b; s14=s09+W-5*b; m=min(s20,s25,s26,s29,s30,s01,s05,s09,s14); print('JUL B',B,'r07-20',s20,'r07-25',s25,'r07-26',s26,'r07-29',s29,'r07-30',s30,'r08-01',s01,'r08-05',s05,'r08-09',s09,'r08-14',s14,'min',m,'room',B+m-1000,'B 08-15',1000+W-6*b)"
JUN B 22608.89 r06-21 -7204.02 r06-24 -7561.41 r06-25 -8278.54 r06-29 -8755.06 r06-30 -5184.19 r07-01 -8499.48 r07-05 -9452.64 r07-11 -10144.38 min -10144.38 room 11464.51 bal 06-30 5960.19
JUL B 22860.51 r07-20 -7065.69 r07-25 -7642.14 r07-26 -8355.43 r07-29 -8701.30 r07-30 -5126.59 r08-01 -8557.17 r08-05 -9510.33 r08-09 -9971.49 r08-14 -5446.27 min -9971.49 room 11889.02 B 08-15 5409.93
$ python3 -c "from decimal import Decimal as D; a=D('119.13'); b=D('115.29'); W=D('5101.67'); t=D('863.80')+D('474.44')+1931; B=1000+W-6*b; r15=D(0); r16=-b+17220-3220; r20=-5*b+17220-t-3220; r24=-9*b+17220-t-3220; r25=-10*b+17220-t-3818; r29=-14*b+17220-t-3818; r01=r29-2*b+3690-a-3200; r06=r01-5*a-492; r09=r06-3*a; m=min(r15,r16,r20,r24,r25,r29,r01,r06,r09); print('AUG B',B,'r08-15',r15,'r08-16',r16,'r08-20',r20,'r08-24',r24,'r08-25',r25,'r08-29',r29,'r09-01',r01,'r09-06',r06,'r09-09',r09,'min',m,'room',B+m-1000,'B 09-15',1000+r09+W+17220-6*a); B=1000+r09+W+17220-6*a; s26=-11*a-t-3220; s27=-12*a-t-3818; s29=-14*a-t-3818; s30=s29-a+3690; s01=s30-b-3200; s05=s01-4*b-492; s10=s05-5*b; m=min(s26,s27,s29,s30,s01,s05,s10); print('SEP B',B,'r09-26',s26,'r09-27',s27,'r09-29',s29,'r09-30',s30,'r10-01',s01,'r10-05',s05,'r10-10',s10,'min',m,'room',B+m-1000,'B 10-15',1000+W+17220-5*b)"
AUG B 5409.93 r08-15 0 r08-16 13884.71 r08-20 10154.31 r08-24 9693.15 r08-25 8979.86 r08-29 8518.70 r09-01 8658.99 r09-06 7571.34 r09-09 7213.95 min 0 room 4409.93 B 09-15 29820.84
SEP B 29820.84 r09-26 -7799.67 r09-27 -8516.80 r09-29 -8755.06 r09-30 -5184.19 r10-01 -8499.48 r10-05 -9452.64 r10-10 -10029.09 min -10029.09 room 18791.75 B 10-15 22745.22
$ python3 -c "from decimal import Decimal as D; a=D('119.13'); b=D('115.29'); W=D('5101.67'); t=D('863.80')+D('474.44')+1931; B=D('22745.22'); r20=-5*b-t-3220; r24=-9*b-t-3220; r25=-10*b-t-3818; r31=-16*b-t-3818; r01=r31-a-3200; r02=r01-a+3690; r05=r02-3*a-492; r09=r05-4*a; m=min(r20,r24,r25,r31,r01,r02,r05,r09); print('OCT B',B,'r10-20',r20,'r10-24',r24,'r10-25',r25,'r10-31',r31,'r11-01',r01,'r11-02',r02,'r11-05',r05,'r11-09',r09,'min',m,'room',B+m-1000,'B 11-15',1000+r09+W+17220-6*a-m); B=1000+r09+W+17220-6*a-m; s22=-7*a-t-3220; s24=-9*a-t-3220; s25=-10*a-t-3818; s29=-14*a-t-3818; s30=s29-a+3690; s01=s30-b-3200; s06=s01-5*b-492; s09=s06-3*b; m=min(s22,s24,s25,s29,s30,s01,s06,s09); print('NOV B',B,'r11-22',s22,'r11-24',s24,'r11-25',s25,'r11-29',s29,'r11-30',s30,'r12-01',s01,'r12-06',s06,'r12-09',s09,'min',m,'room',B+m-1000,'B 12-15',1000+W+17220-6*b)"
OCT B 22745.22 r10-20 -7065.69 r10-24 -7526.85 r10-25 -8240.14 r10-31 -8931.88 r11-01 -12251.01 r11-02 -8680.14 r11-05 -9529.53 r11-09 -10006.05 min -12251.01 room 9494.21 B 11-15 24851.85
NOV B 24851.85 r11-22 -7323.15 r11-24 -7561.41 r11-25 -8278.54 r11-29 -8755.06 r11-30 -5184.19 r12-01 -8499.48 r12-06 -9567.93 r12-09 -9913.80 min -9913.80 room 13938.05 B 12-15 22629.93
$ python3 -c "from decimal import Decimal as D; b=D('115.29'); j=D('118.75'); W=D('5101.67'); t=D('863.80')+D('474.44')+1931; B=D('22629.93'); r20=-5*b-t-3220; r26=-11*b-t-3220; r27=-12*b-t-3818; r29=-14*b-t-3818; r30=r29-b+3690; r31=r30-b; r01=r31-j-3296; r05=r01-4*j-492; r09=r05-4*j; m=min(r20,r26,r27,r29,r30,r31,r01,r05,r09); print('DEC B',B,'r12-20',r20,'r12-26',r26,'r12-27',r27,'r12-29',r29,'r12-30',r30,'r12-31',r31,'r01-01',r01,'r01-05',r05,'r01-09',r09,'min',m,'room',B+m-1000,'bal 2027-12-31',1000+r31-m,'B 2028-01-15',1000+W+17220-6*j)"
DEC B 22629.93 r12-20 -7065.69 r12-26 -7757.43 r12-27 -8470.72 r12-29 -8701.30 r12-30 -5126.59 r12-31 -5241.88 r01-01 -8656.63 r01-05 -9623.63 r01-09 -10098.63 min -10098.63 room 11531.30 bal 2027-12-31 5856.75 B 2028-01-15 22609.17
```
- These run on the floor-1000 basis. With the promised 800 the balances are +800, so 06-30 is 6760.19 and 12-31 is 6656.75. The rooms are unchanged.

**February and March 2028 (1.10).**
```
$ python3 -c "from decimal import Decimal as D; j=D('118.75'); f=D('126.94'); B=1000+D('5101.67')+17220-6*f; a=-6*f-D('539.01')-D('493.41')-1931; b=a-3*f; c=b-f-3818; z=c-4*f; g=z-3296+3690-j; h=g-4*j-492-j; i=h-3*j; print('B 2028-02-15',B,'rel 02-21 less hold',a-3220,'02-24 less hold',b-3220,'02-25',c,'02-29',z,'03-01',g,'03-06',h,'03-09 (min?)',i,'03-13',i+D('5101.67')-4*j,'PL',B+min(z,i),'room',B+min(z,i)-1000,'cum Jan+Feb',D('11381.47')+B+min(z,i)-1000)"
B 2028-02-15 22560.03 rel 02-21 less hold -6945.06 02-24 less hold -7325.88 02-25 -8050.82 02-29 -8558.58 03-01 -8283.33 03-06 -9369.08 03-09 (min?) -9725.33 03-13 -5098.66 PL 12834.70 room 11834.70 cum Jan+Feb 23216.17
$ python3 -c "from decimal import Decimal as D; j=D('118.75'); a4=D('122.71'); B=1000+D('5101.67')+17220-6*j; t=-D('539.01')-D('493.41')-1948; r26=-11*j+t-3220; r29=-14*j+t-3818; r0401=-16*j+t-3818+3690-3296-a4; r0409=r0401-8*a4-492; print('B 2028-03-15',B,'03-26',r26,'03-29',r29,'04-01',r0401,'04-09',r0409,'04-14',r0409-5*a4+D('5101.67')+17220-3220,'room',B+min(r26,r29,r0401,r0409)-1000,'ike left before',D('30566.01')-D('23216.17'))"
B 2028-03-15 22609.17 03-26 -7506.67 03-29 -8460.92 04-01 -8427.13 04-09 -9900.81 04-14 8587.31 room 11708.36 ike left before 7349.84
$ python3 -c "from decimal import Decimal as D, ROUND_HALF_UP as H; print((D('3574.02')*D('1.03')/30).quantize(D('0.01'),H), (D('3574.02')*D('1.03')/31).quantize(D('0.01'),H), (D('3574.02')*D('1.03')/29).quantize(D('0.01'),H))"
122.71 118.75 126.94
```

**The 2027 OKI tax, with P = 800 (1.9, 1.12).**
```
$ python3 -c "from decimal import Decimal as D, ROUND_HALF_UP as H; P=800; jan=D('23545.22')-P-D('10335.16')-1000; mar_ike=D('29390.40')-jan-D('9835.50'); mar_oki=D('13685.82')-mar_ike; s=mar_oki*292+D('12253.87')*261+D('10606.19')*231+D('11464.51')*200+D('11889.02')*170+D('4409.93')*139+D('18791.75')*108+D('9494.21')*78+D('13938.05')*47+D('11531.30')*17; avg=s/365; print('jan room',jan,'mar ike',mar_ike,'mar oki',mar_oki,'sum',s,'avg',avg,'tax pre',avg*D('0.0085'),'oki_tax',(avg*D('0.0085')).quantize(D('1'),H),'konto 2027-06-30',1800-D('5184.19')+D('10144.38'),'konto 2027-12-31',1800-D('5241.88')+D('10098.63'),'taxes 2028',22813+45816+2-2+368)"
jan room 11410.06 mar ike 8144.84 mar oki 5540.98 sum 15814449.62 avg 43327.25923287671232876712329 tax pre 368.2817034794520547945205480 oki_tax 368 konto 2027-06-30 6760.19 konto 2027-12-31 6656.75 taxes 2028 68997
```

**Corrections made during the run.** Each is now reflected in every figure above.
- **Spend on 07-15.** A first one-liner used 5 spend days instead of 6, giving B on 07-15 = 23131.33.
- **January–August spend.** Five 31-day months, not six: the total is 36800.37 and konto ends August at 6371.97.
- **February 2027 window minimum.** It is −11720.33 on 03-01, not the earlier −9744.65 on 03-09.
- **December 2026 year-end.** A hand subtraction slip was corrected to 6944.90.
- **The promised 800.** It was at first not taken out of plan_low in 2027. Taking it out (P = 800) gives the January 2027 room 11410.06, not 12210.06.
- **The wife's 2026 PIT-37 refund.** It falls on return_day + 45 days (2027-06-14), not on 04-30.

---

## 4. Outputs

Figures are under reading B. Where reading A differs, the A value is in brackets.

**wife_net**

| Month | emp_net[wife_acme, m] |
|---|---|
| 2026-10 | 5101.67 |
| 2027-01 | 5101.67 |

The 2027-01 register adds: `assumed: wage_growth 0.04`, `missing record counted by` the proj_wage projection rule (via zus_cap), and `held:` salary and rates.

**social_and_health**

| Month | zus_social | health_month |
|---|---|---|
| 2026-10 | 456.18 | 830.58 |
| 2027-01 | 474.44 | 518.28 |
| 2028-09 | 2083.99 | 898.35 |

From 2027 the projected law values are registered as `missing record counted by …` and `assumed: wage_growth`.

**replay_september** (residuals konto 2026-09). The groups are:

| Group | Difference |
|---|---|
| balance | +32097.23 (stated 41386.27, booked 9289.04) |
| spend | +4599.90 (30 × +153.33 expected but missing) |
| client_pay | −3690.00 (client_b, 08, expected but missing, 09-30; client_a, 08 matched at 0.00) |
| unexplained | −3574.02 (ten records) |
| invest | +535.30 (ike, 09 matched: model 0.00, records −6000.00, difference −6000.00; mak, 09 expected but missing +6535.30) |
| loan_instalment | +300.00 (laptop, 4, 09-05, expected but missing) |
| costs_pay | 0.00 |
| health_pay | 0.00 |
| marek_back | 0.00 |
| rent_pay | 0.00 |
| ryczalt_pay | 0.00 |
| vat_pay | 0.00 |
| wage | 0.00 |

loan_interest and invest[oki] print nothing. The ledger is the same under A.

**explain_room**
- room[2026-11] = **4386.89**, read at (2026-11-15, phase 6) by invest[ike] (which takes 0) and invest[mak] (which takes 4386.89).
- The rule is room, with `keep_floor` 1000 from home_2026. plan_low has bal 5386.89, P 0, and its lowest point is 0 on 11-15.
- The register has law date 2026-11-01, cut 2026-10-19, the hold, held rows, and `assumed: holidays [] from 2031`.
- [A: a hole. Its question is the `bank` input's question, or else `bank at <r>: no value at <date>`.]

**konto_balance**

| Date | bal(konto, d) | bal − vat_owed |
|---|---|---|
| 2026-10-31 | 6064.17 | 6064.17 |
| 2026-12-31 | 6944.90 | 6944.90 |
| 2027-06-30 | 6760.19 | 6760.19 |
| 2027-12-31 | 6656.75 | 6656.75 |

[A: all eight are holes, from 2026-10-02.]

**net_worth**
- 2026-12-31: **170202.16** [A: a hole].
- 2031-12-31, 2041-12-31 and 2059-12-31: not computed (§2) [A: holes].

**ike_left**

| Year | Value | [A] |
|---|---|---|
| 2026 | −6000.00 | hole |
| 2027 | 0.00 | hole |
| 2028 | 0.00 | hole |

**pit28**

| Year | pit28_due |
|---|---|
| 2026 | 2.00 |
| 2027 | 2.00 |
| 2028 | 1.00 |

These are the same under A.

**pit37**

| Person | 2026 | 2027 | 2028 |
|---|---|---|---|
| me | 0.00 | 0.00 | 0.00 |
| wife | −2.00 | −2.00 | −2.00 |

**health_settlement**

| Year | health_settle |
|---|---|
| 2026 | 996.69 |
| 2027 | 1036.56 |
| 2028 | 1078.02 |

**taxes_paid**

| Year | flows(urzad_skarbowy, y) | [A] |
|---|---|---|
| 2026 | 52272.00 | same |
| 2027 | 69059.00 | same |
| 2028 | 68997.00 | hole: oki_tax[2027] reads the OKI balances |

---

## 5. Ambiguities

**1. Bank rows read outside their governs span (A/B).** This is the one that matters.
- **Where.** The learned spend rule reads `bank[r]` rows. The records sit in version `mbank_2026_09`, which governs 2026-09. `bank` is keyed only by the open index `bank_row`, which is not a time index. So by LANGUAGE §4.6 a candidate needs its span to contain the reader's law date d. For spend_day on 2026-10-02 and after, d is October or later.
- **Choice A, the literal §4.6.** There is no candidate, so this is the gap rule:
  - spend[d] is a hole from 10-02, and so is konto (no later anchor);
  - so are room from 2026-10 and invest (min(hole, 0) is a hole, §8.1), and the wrappers and oki_tax;
  - so are konto_balance (all four dates), explain_room, ike_left for 2026–2028, net_worth and taxes_paid 2028;
  - the other asks are unchanged.
- **Choice B, the evident intent.** Record rows are history, readable at any later date (§5.18 "Reading history"; its own example sums `bank` rows of year y − 1). So learned = 3574.02, which gives every B figure above.
- §5.18's own `spend_month` example has the same gap. I computed both choices.

**2. Whether a look-ahead evaluates the `when` of instances outside its window.** This affects the register only.
- **The text.** ENGINE §4.2 and LANGUAGE §5.13.1 put `when` before the date: a false `when` drops the instance, and only then does a date outside (t, b] drop it.
- **Choice (i), literal.** Every future instance's `when` is evaluated up to E. Examples are ryczalt_pay's `ryczalt_adv > 0`, the pit pays' `due > 0` and pit37 `< 0`, and health_settle. So the register of explain_room, and of every room, grows by what those `when`s reach: `assumed: wage_growth`, the `missing record counted by` projection entries, inflation 0.03 from 2028 (assumed), and holidays [] from 2031.
- **Choice (ii).** Only instances whose date can fall in the window are tested. The register is then as given in §4.
- **Values.** The values are the same either way: no such `when` is a hole whose date is unknown.

**3. A record that explains a modelled-past state pay on the same day.**
- **The two texts.** ENGINE §4.2 step 1 books such a record at its instance's point, in step 3. LANGUAGE §5.13.2 step 1 books all records of the day first.
- **Choice (i), the engine.** ike's room on 2026-09-15 is 12535.30, then 6535.30 at mak's point.
- **Choice (ii), the language.** Both points read 6535.30.
- **Outcome.** Either way invest[ike, 09] is modelled as 0, because its limit is already met (same-day flows include the record under (ii)), and invest[mak, 09] is 6535.30. The ledger is identical.

**4. Does an explained record count in `flows(invest[ike], y)`?**
- **Counts.** §5.18 books an explained record "as that pay instance's movement", so the −6000 of 09-15 is invest[ike, 2026-09]'s movement. flows is then 34260 and ike_left[2026] = −6000.00. I used this.
- **Does not count.** flows is 28260 and ike_left[2026] = 0.00.
- No later figure changes: October's ike is 0 either way.

**5. No balance line on loan_debt.**
- Not an ambiguity in the text, but worth flagging. The September instalment has no record, so it is never booked on either end. konto absorbs it through the anchor's balance line. loan_debt stays 300 too negative from then on (−1200 at 2026-12-31 instead of −900), with no ledger line on that place, because it has no `stated` rows.
- I followed the text: −1200.

**6. The register of explain_room.**
- "held" covers rows with open spans read for a period after K (§8.2). Whether rows inside `vat_owed`'s `where` count as "read" for months beyond the window (they are evaluated, and §8.2 says the register is transitive over every operand) adds `assumed: holidays [] from 2031` and the far-future held rows.
- I listed them. That reading follows §8.2 literally.
