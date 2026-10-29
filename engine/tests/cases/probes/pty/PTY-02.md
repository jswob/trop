# PTY-02 · Groups the law computes on

**Story.** In 2024 Tomek earns a taxable 90 000.00 DUK and Ania 30 000.00. They file jointly.

**Rules.**
1. The scale: 12 % up to and including 60 000.00, 32 % on the excess.
2. The joint tax is twice the scale's tax on half the spouses' combined income.
3. The joint tax is traced to each spouse in proportion to the tax each would pay alone; floored to 0.01, the leftover minor units go to the largest dropped remainders.

**Question.** The joint tax and each spouse's part.

**Worked by hand.** Half of 120 000 is 60 000: tax 7 200; joint **14 400.00**. Alone: Tomek 7 200 + 32 % × 30 000 = 16 800; Ania 3 600; together 20 400. Tomek 14 400 × 16 800/20 400 = 11 858.8235 → **11 858.82**; Ania 2 541.1765 → **2 541.18**; they add to 14 400.00.

```yaml
edge: "PTY-02"
inputs: {"incomes": {"Tomek": 90000.00, "Ania": 30000.00}, "bands": [[60000.00, 0.12], [null, 0.32]]}
expect: {"kind": "value", "value": {"joint": 14400.00, "alone": {"Tomek": 16800.00, "Ania": 3600.00}, "parts": {"Tomek": 11858.82, "Ania": 2541.18}}}
```
