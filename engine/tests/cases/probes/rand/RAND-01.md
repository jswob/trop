# RAND-01 · Random families

**Story.** Halina is 85. Her stated life table gives the chance of dying within each coming year: 0.2 at 85, 0.3 at 86, 0.5 at 87, 1 at 88.

**Rules.**
1. A lifetime is drawn year by year from the table; it counts the whole years lived before the year of death.
2. The run uses seed 20251024 and 60 000 paths; the same seed gives the same answer.

**Question.** The mean drawn lifetime (target error 0.02).

**Worked by hand.** The exact mean is 0.8 + 0.8 × 0.7 + 0.8 × 0.7 × 0.5 = 0.8 + 0.56 + 0.28 = **1.64** years; the seeded mean must lie within 4 of its standard errors of it. Its spread: E[X²] = 0.24 × 1 + 0.28 × 4 + 0.28 × 9 = 3.88, variance 3.88 − 1.64² = 1.1904, so 60 000 paths give a standard error near 0.0045, under 0.02/4.

```yaml
edge: "RAND-01"
inputs: {"qs": [0.2, 0.3, 0.5, 1], "seed": 20251024, "paths": 60000, "p": 0.02}
expect: {"kind": "random", "r": 1.64, "p": 0.02}
```
