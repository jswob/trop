# TIME-04 · Periods each law declares, split by facts

**Story.** Zosia is born on 17 October 2023. Her parents claim the child benefit for the benefit period June 2023 – May 2024.

**Rules.**
1. The benefit is 100.00 DUK a month for each month of the period in which the child lives.
2. The month of birth is pro-rated by its own days, from the birth day inclusive, rounded half up to 0.01.

**Question.** October's amount and the period's total.

**Worked by hand.** October has 31 days; 17–31 is 15 days: 100 × 15/31 = 48.387 → **48.39**. November–May: 7 × 100 = 700. Total **748.39**.

```yaml
edge: "TIME-04"
inputs: {"born": "2023-10-17", "period": ["2023-06", "2024-05"], "monthly": 100.00}
expect: {"kind": "value", "value": {"first_month": 48.39, "total": 748.39}}
```
