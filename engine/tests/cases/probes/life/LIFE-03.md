# LIFE-03 · Loans

**Story.** Piotr borrows 6 000.00 DUK, repaid in 12 monthly annuity instalments at 1 % a month. From month 7 the rate is reset to 2 % a month. After paying month 9 he overpays 1 000.00.

**Rules.**
1. Interest is the balance × the monthly rate, rounded half up to 0.01; the instalment is level over the months left, rounded half up to 0.01.
2. A reset re-levels the instalment over the months left; an overpayment lowers it, keeping the end date.

**Question.** The instalment at the start, after the reset, and after the overpayment.

**Worked by hand.** Start: 6 000 × 0.01 / (1 − 1.01^−12) = 533.0927 → **533.09**. Months 1–6 (interest, principal, balance): 60.00, 473.09, 5 526.91; 55.27, 477.82, 5 049.09; 50.49, 482.60, 4 566.49; 45.66, 487.43, 4 079.06; 40.79, 492.30, 3 586.76; 35.87, 497.22, 3 089.54. Reset: 3 089.54 × 0.02 / (1 − 1.02^−6) = 551.5626 → **551.56**. Months 7–9: 61.79, 489.77, 2 599.77; 52.00, 499.56, 2 100.21; 42.00, 509.56, 1 590.65. Overpaid: 590.65; 590.65 × 0.02 / (1 − 1.02^−3) = 204.81 → **204.81**.

```yaml
edge: "LIFE-03"
inputs: {"principal": 6000.00, "rate": 0.01, "months": 12, "events": [{"month": 7, "kind": "reset", "rate": 0.02}, {"month": 9, "kind": "overpay", "amount": 1000.00}]}
expect: {"kind": "value", "value": {"start": 533.09, "after_reset": 551.56, "after_overpay": 204.81, "balance_after_month_6": 3089.54}}
```
