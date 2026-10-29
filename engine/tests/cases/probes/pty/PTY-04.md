# PTY-04 · A law reads another party ★

**Story.** Ania earns 60 000.00 DUK and Tomek 52 000.00 in 2024; they have one child. Then Tomek's year is restated with a raise: 52 000.01.

**Rules.**
1. A child relief of 1 000.00 DUK a year is given if the parents' joint income is not more than 112 000.00.
2. The parents split it by their statement: half each.

**Question.** Each parent's relief, before and after the raise.

**Worked by hand.** Before: 60 000 + 52 000 = 112 000, not more than the limit: 500.00 each. After: 112 000.01 is more: **0.00** each. Tomek's raise moves Ania's relief.

```yaml
edge: "PTY-04"
inputs: {"relief": 1000.00, "limit": 112000.00, "split": {"Ania": 0.5, "Tomek": 0.5},
  "before": {"Ania": 60000.00, "Tomek": 52000.00}, "after": {"Ania": 60000.00, "Tomek": 52000.01}}
expect: {"kind": "value", "value": {"before": {"eligible": true, "Ania": 500.00, "Tomek": 500.00}, "after": {"eligible": false, "Ania": 0.00, "Tomek": 0.00}}}
```
