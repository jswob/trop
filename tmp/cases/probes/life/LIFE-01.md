# LIFE-01 · A thing written once, with a lifetime

**Story.** Tomek's job contract, from January 2024: 6 000.00 DUK a month; a raise to 6 600.00 from 1 April 2025; from each January from 2026, the pay rises by the previous year's price-index change, rounded half up to whole DUK. The 2025 change is 3.4 %.

**Rules.**
1. The contract is written once; its terms change on their dates.

**Question.** His pay for 2025 in all; his monthly pay from January 2026.

**Worked by hand.** 2025: 3 × 6 000 + 9 × 6 600 = **77 400.00**. January 2026: 6 600 × 1.034 = 6 824.40 → **6 824.00**.

```yaml
edge: "LIFE-01"
inputs: {"pay": [["2024-01", 6000.00], ["2025-04", 6600.00]], "index_from": 2026, "index": {"2025": 0.034}}
expect: {"kind": "value", "value": {"total_2025": 77400.00, "jan_2026": 6824.00}}
```
