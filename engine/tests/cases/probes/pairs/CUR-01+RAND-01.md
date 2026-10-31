# CUR-01 × RAND-01 · A drawn rate on the law's leg

**Story.** Piotr will invoice a foreign client 1 000.00 FLR on Monday 2 March 2026. The client will pay on Wednesday 1 April 2026, and Piotr's bank will convert the money at its own rate. Neither rate is known yet: the forecast draws them.

**Rules.**
1. Revenue in a foreign currency is converted at the central bank's mid rate of the last business day before the invoice date.
2. The bank converts at its own rate: that day's mid rate less 0.0800 DUK/FLR.
3. Converted amounts are rounded half up to 0.01 DUK, on each path.
4. The mid rate is 4.2000 on Friday 20 February 2026. On each business day after it (Monday to Friday; no holidays in these weeks) it is the day before's plus 0.0400 or less 0.0200, each with chance 1/2. The run draws with Python's `random.Random(20260301)`: 20 000 paths in turn, and in each one `random()` for each business day from 23 February to 1 April in order, below 0.5 meaning up. The same seed gives the same answer.

**Question.** The mean revenue in DUK (target error 2.00), the mean cash received in DUK (target error 5.00), and does the same seed give the same answer?

**Worked by hand.** The law's rate is Friday 27 February's, 5 steps after 20 February; the money's is 1 April's, 28 steps after it. A step is 0.01 on average, so the mean revenue is exactly 1 000.00 × (4.2000 + 5 × 0.01) = **4 250.00**, and the mean cash 1 000.00 × (4.2000 + 28 × 0.01 − 0.0800) = **4 400.00**. Each seeded mean must lie within 4 of its standard errors of these. Reading the invoice day's own rate, or the bank's, for the revenue moves its mean by a step or more. Same seed, same answer: **yes**.

```yaml
edge: ["CUR-01", "RAND-01"]
inputs: {"amount": 1000.00, "issued": "2026-03-02", "paid": "2026-04-01", "start": "2026-02-20", "start_rate": 4.2000, "up": 0.0400, "down": -0.0200, "chance": 0.5, "spread": 0.0800, "seed": 20260301, "paths": 20000, "p": {"revenue": 2.00, "cash": 5.00}}
expect: {"kind": "value", "value": {"revenue": {"kind": "random", "r": 4250.00, "p": 2.00}, "cash": {"kind": "random", "r": 4400.00, "p": 5.00}, "same_seed_same_answer": true}}
```
