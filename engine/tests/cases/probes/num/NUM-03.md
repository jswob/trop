# NUM-03 · Rounding where and as the law rounds ★

**Story.** Tomek's monthly income is 4 560.50 DUK; his deductible costs are 250.00 DUK; the monthly relief is 43.37 DUK.

**Rules.**
1. The base is income minus costs, rounded half up to whole DUK.
2. The advance is 17 % of the base minus the relief, rounded half up to whole DUK.

**Question.** What is the advance? What would rounding once, at the end, give instead?

**Worked by hand.** Base 4 310.50 → 4 311. 17 % × 4 311 = 732.87; − 43.37 = 689.50 → **690**. Rounding once: 17 % × 4 310.50 = 732.785; − 43.37 = 689.415 → 689. The law's order gives 1 DUK more.

```yaml
edge: "NUM-03"
inputs: {"income": 4560.50, "costs": 250.00, "rate": 0.17, "relief": 43.37}
expect: {"kind": "value", "value": {"advance": 690, "rounded_once": 689, "difference": 1}}
```
