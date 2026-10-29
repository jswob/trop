# LAW-05 · "As if"

**Story.** Tomek's 2022 income is 50 000.00 DUK.

**Rules.**
1. Rule set O: 17 % minus a yearly relief of 3 600.00.
2. Rule set N: 12 %, no relief.
3. The tax is the lower of the two, each run on the same income.

**Question.** The tax under each set, and the tax kept.

**Worked by hand.** O: 8 500 − 3 600 = 4 900.00. N: 6 000.00. Kept: **4 900.00** (O).

```yaml
edge: "LAW-05"
inputs: {"income": 50000.00, "sets": {"O": {"rate": 0.17, "relief": 3600.00}, "N": {"rate": 0.12, "relief": 0.00}}}
expect: {"kind": "value", "value": {"O": 4900.00, "N": 6000.00, "kept": 4900.00, "set": "O"}}
```
