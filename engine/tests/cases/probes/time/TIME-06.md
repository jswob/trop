# TIME-06 · Running totals and repricing within a period

**Story.** Tomek's taxable base is 13 000.00 DUK every month of 2024.

**Rules.**
1. The yearly scale: 12 % up to and including 120 000.00, 32 % on the excess.
2. Each month's advance is the scale's tax on the base to date, rounded half up to 0.01, minus the advances already withheld this year.

**Question.** September's and October's advances.

**Worked by hand.** To September: 117 000 → tax 14 040; withheld Jan–Aug 8 × 1 560 = 12 480; September **1 560.00**. To October: 130 000 → 14 400 + 32 % × 10 000 = 17 600; minus 14 040 withheld: October **3 560.00**.

```yaml
edge: "TIME-06"
inputs: {"monthly_base": 13000.00, "months": 10, "bands": [[120000.00, 0.12], [null, 0.32]]}
expect: {"kind": "value", "value": {"september": 1560.00, "october": 3560.00}}
```
