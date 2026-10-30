# RAND-04 · What a run may say

**Story.** Two plans over 10 years: A puts 10 000.00 DUK into a fund; B puts the same 10 000.00 into the fund and keeps 1 000.00 in cash at 0 %. The fund's yearly log-return is normal, mean 0.05, deviation 0.15.

**Rules.**
1. Both plans run on the same seeded paths (seed 11, 1 000 paths): common random numbers.
2. The run reports the path-wise difference by a stated statistic (the median) and the share of paths where B ends above A; no winner.

**Question.** The median path-wise difference B − A and the share of paths where B is above.

**Worked by hand.** On every path both hold the same fund value; B adds 1 000.00 of cash: the difference is **1 000.00** on every path, so the median is 1 000.00 and the share is **1** (1 000 of 1 000). On independent seeds the difference would scatter.

```yaml
edge: "RAND-04"
inputs: {"fund": 10000.00, "cash": 1000.00, "years": 10, "mu": 0.05, "sigma": 0.15, "seed": 11, "paths": 1000}
expect: {"kind": "relation", "value": {"median_difference": 1000.00, "share_b_above": 1, "paths": 1000, "common_random_numbers": true}}
```
