# REC-07 · Replaying over recorded history

**Story.** A fund's recorded price at the start of each year: 100, 120, 90, 110, 130, 100, 150 (years 0–6). A plan buys 1 000.00 DUK of it at the start of each of 3 years and values the holding at the start of the 4th.

**Rules.**
1. The plan is replayed from every start year with a full 3 years in the record; units are kept exact; values are rounded half up to 0.01.

**Question.** The end value from each start; the worst and the best.

**Worked by hand.** Start 0: (10 + 8⅓ + 11⅑) × 110 = 3 238.89. Start 1: (8⅓ + 11⅑ + 9 1/11) × 130 = 3 709.60. Start 2: (11⅑ + 9 1/11 + 7 9/13) × 100 = 2 789.43. Start 3: (9 1/11 + 7 9/13 + 10) × 150 = 4 017.48. Worst **start 2, 2 789.43**; best **start 3, 4 017.48**.

```yaml
edge: "REC-07"
inputs: {"prices": [100, 120, 90, 110, 130, 100, 150], "buy": 1000.00, "years": 3}
expect: {"kind": "value", "value": {"ends": [3238.89, 3709.60, 2789.43, 4017.48], "worst": [2, 2789.43], "best": [3, 4017.48]}}
```
