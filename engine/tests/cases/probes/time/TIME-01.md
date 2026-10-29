# TIME-01 · A date known only to the month

**Story.** Tomek's father gave him notes saying lot L1, 10 units, was "bought in March 2019" for 1 000.00 DUK. Tomek sold L1 on 14 March 2024 for 1 600.00 DUK.

**Rules.**
1. A gain is exempt if the sale falls after the day 5 years from the purchase day (same day number; a month lacking it, its last day).
2. Otherwise it is taxed at 19 %.

**Question.** The gain, and whether it is taxed.

**Worked by hand.** Gain 1 600 − 1 000 = **600.00**. Bought on the 1st–13th: the clock ends 1–13 March 2024, the sale is after it, exempt. Bought on the 14th or later: the clock ends on or after the sale, taxed 114.00. The purchase day is unknown: **partial**, the day named missing. Never the 1st or the 15th by default.

```yaml
edge: "TIME-01"
inputs: {"lot": "L1", "bought_month": "2019-03", "cost": 1000.00, "proceeds": 1600.00, "sold": "2024-03-14", "years": 5}
expect: {"kind": "partial", "value": {"gain": 600.00}, "missing": ["purchase day of L1 (known only as 2019-03)"]}
```
