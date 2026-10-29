# RAND-03 · Law on every path ★

**Story.** Each month of 2026 a client of Piotr's pays 10 000.00 DUK with chance 1/2, independently.

**Rules.**
1. A year's revenue of more than 80 000.00 moves Piotr into the higher tax regime; the law is applied on every path.
2. The run uses seed 7 and 40 000 paths.

**Question.** The share of paths in the higher regime (target error 0.01).

**Worked by hand.** More than 80 000 means 9 or more payments: (C(12,9) + C(12,10) + C(12,11) + C(12,12)) / 2^12 = (220 + 66 + 12 + 1)/4 096 = 299/4 096 = **0.072998046875**; the seeded share must lie within 4 of its standard errors (about 0.0013) of it.

```yaml
edge: "RAND-03"
inputs: {"months": 12, "chance": 0.5, "amount": 10000.00, "threshold": 80000.00, "seed": 7, "paths": 40000, "p": 0.01}
expect: {"kind": "random", "r": 0.072998046875, "p": 0.01}
```
