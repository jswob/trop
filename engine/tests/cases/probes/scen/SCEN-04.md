# SCEN-04 · Named extreme paths

**Story.** Ania's plan: 100 000.00 DUK invested, plus 12 000.00 at the start of each year, for 10 years. The base return is 5 % a year. The named path "crash": −50 % in year 2, 5 % in every other year. The floor she watches: 100 000.00 at the end of a year.

**Rules.**
1. Each year: add the contribution, then apply the year's return; round half up to 0.01.

**Question.** On the crash path: the end of year 10, and the first year ending below the floor.

**Worked by hand.** Y1 (112 000) × 1.05 = 117 600.00. Y2 129 600 × 0.5 = 64 800.00, below the floor: **year 2**. Y3 80 640.00; Y4 97 272.00; Y5 114 735.60; Y6 133 072.38; Y7 152 325.999 → 152 326.00; Y8 172 542.30; Y9 193 769.415 → 193 769.42; Y10 205 769.42 × 1.05 = 216 057.891 → **216 057.89**.

```yaml
edge: "SCEN-04"
inputs: {"start": 100000.00, "yearly": 12000.00, "years": 10, "returns": {"default": 0.05, "2": -0.50}, "floor": 100000.00}
expect: {"kind": "value", "value": {"end": 216057.89, "first_below_floor": 2}}
```
