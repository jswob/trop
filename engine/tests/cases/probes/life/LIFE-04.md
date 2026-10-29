# LIFE-04 · Benefits recalculated

**Story.** Halina receives a benefit of 1 000.00 DUK a month through 2025. An act raises running benefits to 1 100.00 from 1 September 2025.

**Rules.**
1. A running benefit is recalculated from the date the law raises it.

**Question.** Her benefit for 2025 in all.

**Worked by hand.** 8 × 1 000 + 4 × 1 100 = **12 400.00**.

```yaml
edge: "LIFE-04"
inputs: {"year": 2025, "amounts": [["2025-01", 1000.00], ["2025-09", 1100.00]]}
expect: {"kind": "value", "value": 12400.00}
```
