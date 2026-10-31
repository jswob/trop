# RULE-03 · Rules competing for the same money ★

**Story.** A month's surplus is 5 000.00 DUK. Rule R-A: put 3 000.00 into the tax reserve. Rule R-B: overpay the mortgage with up to 4 000.00.

**Rules.**
1. With a stated priority, each rule takes in turn from what is left.
2. With none, two rules wanting the same money are refused, naming both.

**Question.** (a) With the priority R-A before R-B. (b) With no priority stated.

**Worked by hand.** (a) R-A **3 000.00**, R-B **2 000.00**, nothing left. (b) **Refused**: the priority between R-A and R-B is missing.

```yaml
edge: "RULE-03"
inputs: {"surplus": 5000.00, "claims": {"R-A": 3000.00, "R-B": 4000.00}, "a": ["R-A", "R-B"], "b": null}
expect: {"a": {"kind": "value", "value": {"R-A": 3000.00, "R-B": 2000.00, "left": 0.00}}, "b": {"kind": "refusal", "missing": ["priority between R-A, R-B"]}}
```
