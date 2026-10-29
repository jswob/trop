# OWN-09 · Indexed bonds in monthly series

**Story.** On 1 March 2023 Halina buys 50 bonds of the March 2023 series, 100.00 DUK each. She redeems them all on 1 September 2025, in the series' third interest year.

**Rules.**
1. Year 1 pays 6.80 %. Each later year pays the price index's yearly change for the month two months before the year starts, plus 1.25 %.
2. Interest is capitalised yearly, computed per bond and rounded half up to 0.01; a part year is pro-rated by its days over the interest year's days.
3. Early redemption costs 2.00 a bond, never more than the bond's interest.
4. The tax is 19 % of the holding's interest less fees, rounded half up to whole DUK.
5. The index: January 2024 3.70 %, January 2025 4.90 %.

**Question.** Interest per bond, the fee, the tax, the payout.

**Worked by hand.** Y1: 6.80 → 106.80. Y2 (rate 3.70 + 1.25 = 4.95 %): 106.80 × 4.95 % = 5.2866 → 5.29 → 112.09. Y3 (6.15 %) from 1 March to 1 September 2025, 184 of 365 days: 112.09 × 6.15 % × 184/365 = 3.4751 → 3.48. Interest **15.57** a bond; fee **2.00**. Tax 19 % × 50 × 13.57 = 128.915 → **129**. Payout 50 × 115.57 − 100 − 129 = **5 549.50**.

```yaml
edge: "OWN-09"
inputs: {"bonds": 50, "face": 100.00, "bought": "2023-03-01", "redeemed": "2025-09-01", "first_rate": 0.0680, "margin": 0.0125, "lag_months": 2,
  "index": {"2024-01": 0.0370, "2025-01": 0.0490}, "fee": 2.00, "tax_rate": 0.19}
expect: {"kind": "value", "value": {"interest_per_bond": 15.57, "fee_per_bond": 2.00, "tax": 129, "payout": 5549.50}}
```
