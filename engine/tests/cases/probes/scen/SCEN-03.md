# SCEN-03 · Linked overrides

**Story.** Halina's rent is 2 000.00 DUK a month in 2025, raised each January by the previous year's price-index change. The base path: 3 % a year. Her rent, her pension and her bonds read the index. An override: "8 % for 2025–2027".

**Rules.**
1. An override replaces the index for its years; after them the base path resumes. Everything that reads the index moves with it.
2. Rent is rounded half up to 0.01 each year.

**Question.** The rent in 2029 under the override and in the base; what the override moves.

**Worked by hand.** Override: 2026 2 160.00; 2027 2 332.80; 2028 2 519.424 → 2 519.42; 2029 (3 %) 2 595.0026 → **2 595.00**. Base: 2 060.00; 2 121.80; 2 185.454 → 2 185.45; 2 251.0135 → **2 251.01**. Moves: **bonds, pension, rent**.

```yaml
edge: "SCEN-03"
inputs: {"rent": 2000.00, "from": 2025, "to": 2029, "base": 0.03, "override": {"years": [2025, 2027], "rate": 0.08}, "readers": {"rent": "CPI", "pension": "CPI", "bonds": "CPI", "mortgage": "REF"}}
expect: {"kind": "value", "value": {"rent_2029": 2595.00, "base_rent_2029": 2251.01, "moved": ["bonds", "pension", "rent"]}}
```
