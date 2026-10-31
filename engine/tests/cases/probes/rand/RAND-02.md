# RAND-02 · Closed forms

**Story.** The same life table as RAND-01: 0.2 at 85, 0.3 at 86, 0.5 at 87, 1 at 88.

**Rules.**
1. The expected whole years still lived is the sum, over coming years, of the chance of being alive at the year's end.

**Question.** Halina's expected remaining lifetime, exactly.

**Worked by hand.** 0.8 + 0.56 + 0.28 + 0 = **1.64** = 41/25.

```yaml
edge: "RAND-02"
inputs: {"qs": [0.2, 0.3, 0.5, 1]}
expect: {"kind": "relation", "value": {"expected": 1.64, "exact": "41/25"}}
```
