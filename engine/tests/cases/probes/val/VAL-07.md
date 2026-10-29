# VAL-07 · The assumption register

**Story.** Halina's rent is 2 000.00 DUK a month in 2025. The forecast states two assumptions: A-RENT, the rent rises each January by the price index + 1 point; A-CPI, the price index rises 3 % a year from 2026.

**Rules.**
1. A result lists every assumption it reads, directly or through another.
2. Rent is rounded half up to 0.01 each year.

**Question.** The rent in 2027, and its register.

**Worked by hand.** 4 % a year: 2026 2 080.00; 2027 **2 163.20**. Register: **A-RENT, A-CPI** (A-RENT reads A-CPI).

```yaml
edge: "VAL-07"
inputs: {"rent": 2000.00, "from": 2025, "to": 2027, "reads": {"rent": ["A-RENT"], "A-RENT": ["A-CPI"], "A-CPI": []},
  "assumptions": {"A-RENT": {"plus": 0.01, "of": "A-CPI"}, "A-CPI": {"rate": 0.03}}}
expect: {"kind": "value", "value": {"rent_2027": 2163.20, "register": ["A-RENT", "A-CPI"]}}
```
