# RULE-04 · A loop

**Story.** Rule R-X: invest half of what is left after the tax reserve. Rule R-Y: the tax reserve is 20 % of the expected gains on what R-X invests this month.

**Rules.**
1. Rules that depend on each other in a loop are refused unless a law names a solver; none does here.

**Question.** How much does R-X invest?

**Worked by hand.** R-X reads R-Y, and R-Y reads R-X: a loop, **refused**, naming R-X → R-Y → R-X.

```yaml
edge: "RULE-04"
inputs: {"reads": {"R-X": ["R-Y"], "R-Y": ["R-X"]}}
expect: {"kind": "refusal", "missing": ["a solver for R-X -> R-Y -> R-X"]}
```
