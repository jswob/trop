# Edge list: the holdout's own edges

Edges that live only in household C (and E2). Same entry format as `tmp/cases/boundaries.md`; checked with it by `python3 -I tmp/cases/tools/check_edges.py --hidden tmp/cases-holdout/boundaries-hidden.md`. Never copied into the visible tree.

C is made up like everything else: a foreign country with its own tax year, its own rounding, its own wage-tax algorithm and a treaty with the home country. Step 8 writes C; it starts by asking the user for 5–10 one-line house-rule ideas (HID-05).

## HID: the foreign side

### HID-01 · The treaty names the relief method
- hard: the treaty, not the home law, says whether foreign pay is relieved by a credit or exempt with progression; the same home law runs under each method; the method is a stated fact of the treaty
- where: C (Jan's foreign job from July 2024, the home return for 2024)
- key: value; by-case where the treaty's method reads two ways
- probe: one year's foreign pay under the credit method and under exemption with progression: the home tax under each
- ids: N-931 MX-01

### HID-02 · The foreign-tax credit and its limit
- hard: the credit is capped at the home tax that falls on the foreign income (a proportional formula); the order among credits; what happens to unused credit, stated by the treaty or the home law
- where: C (the 2024 and 2025 home returns)
- key: value
- probe: foreign tax above the cap: the credit, and the unused part
- ids: MX-02

### HID-03 · A foreign wage-tax algorithm
- hard: the foreign employer withholds by a published algorithm with tax classes chosen per year; the foreign tax year and rounding differ from the home ones; the employer's yearly certificate is the input the home return reads
- where: C (Jan's pay from July 2024)
- key: value
- probe: one month's foreign withholding under two tax classes
- ids: N-2108

### HID-04 · Two countries coordinate
- hard: social insurance moves to the country of work from the job's first day; which country pays the child benefit first, and a top-up from the other that reads the other parent's work; a status that changes mid-year
- where: C (from July 2024: insurance abroad; the child benefit paid by one country, topped up by the other)
- key: value, partial
- probe: a child benefit with one parent working abroad: who pays, and the top-up
- ids: MX-03 MX-07
- counts: L3 (the third rule that reads another party)

### HID-05 · House-rule shapes the visible set never shows
- hard: rules shaped like nothing in A, B or D, from the user's ideas (step 8), plus C's own: convert the foreign currency once it is above a sum
- where: C
- key: value, refusal
- probe: written by step 8 from the user's ideas
- ids: none
- counts: R1

---

## Where C also hits visible edges

Step 8 aims C's questions at these as well (the second household of threads T1, T5, T7, T8):

- **T1, earnings:** LAW-08 (several laws on one income, in order), LAW-05 ("as if": exemption with progression), REC-05 (gross inferred from net), LAW-06 (a tax class per year, LC-24)
- **T5, a sole trader closed mid-year:** LIFE-02 (closing ends contributions and VAT), LAW-08 (tax on profit; a business and a job side by side, LC-13), TIME-04 (the switch splits the year)
- **T7, a child:** PTY-04 (the top-up reads the other parent's work), LIFE-04 (benefits recalculated; a correction clawed back in instalments, LC-23)
- **T8, two currencies:** CUR-01, CUR-02, CUR-03, NUM-01 (a salary in the foreign currency, a foreign-currency account), OWN-01 (accounts opened at the switch, LC-12; one net worth over two currencies, LC-17), LAW-07 (residence and cross-border status, N-914)

## Counts and twist, holdout part

| Tag | Count | Edges | Where |
|---|---|---|---|
| L3 | the third rule that reads another party | HID-04 | C |
| R1 | C's house rules: dates, state, forecast, and shapes the visible set never shows | HID-05, RULE-01, RULE-09 | C |

| Household | Twist | Edges it moves |
|---|---|---|
| C | a currency switch: Jan closes his home business and starts a foreign job paid in the foreign currency, July 2024 | HID-01, HID-03, HID-04, CUR-01, LIFE-02, TIME-04 |
