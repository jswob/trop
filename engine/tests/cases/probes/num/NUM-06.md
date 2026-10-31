# NUM-06 · Which year's money

**Story.** Halina's spending is 3 000.00 DUK a month in 2025 prices. The stated price-index path: +4 % in 2026, +3 % in 2027, +3 % in 2028, +2 % in 2029, +2 % in 2030.

**Rules.**
1. An amount in a year's prices is read in another year's money through the index path.
2. The nominal amount is rounded half up to 0.01 once, at the end (a view, not a booking).

**Question.** What is that spending in 2030 nominal DUK?

**Worked by hand.** 1.04 × 1.03 = 1.0712; × 1.03 = 1.103336; × 1.02 = 1.12540272; × 1.02 = 1.1479107744. × 3 000 = 3 443.732 → **3 443.73 DUK a month**.

```yaml
edge: "NUM-06"
inputs: {"amount": 3000.00, "factors": [1.04, 1.03, 1.03, 1.02, 1.02]}
expect: {"kind": "value", "value": 3443.73}
```
