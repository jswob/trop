# LAW-03 · Law versions by date ★

**Story.** Two employees' monthly pay advances in 2022: Ania's base is 8 000.00 DUK, Ola's 2 000.00.

**Rules.**
1. Old version, to 30 June 2022: 17 % of the base minus a monthly relief of 300.00.
2. New version, from 1 July 2022: 12 % of the base, no relief.
3. Transitional rule, July–December 2022: the payroll computes both and keeps the lower.
4. Advances are rounded half up to 0.01.

**Question.** Ania's June and July advances; Ola's July advance. Does the change move June?

**Worked by hand.** Ania June: 1 360 − 300 = **1 060.00**. July: old 1 060.00, new 960.00: **960.00**. Ola July: old 340 − 300 = 40.00, new 240.00: **40.00** (the transitional rule keeps the old). June without the change: also 1 060.00: unchanged.

```yaml
edge: "LAW-03"
inputs: {"old": {"rate": 0.17, "relief": 300.00, "to": "2022-06-30"}, "new": {"rate": 0.12, "relief": 0.00, "from": "2022-07-01"},
  "transitional": ["2022-07-01", "2022-12-31"], "cases": [["Ania", "2022-06", 8000.00], ["Ania", "2022-07", 8000.00], ["Ola", "2022-07", 2000.00]]}
expect: {"kind": "relation", "value": {"Ania 2022-06": 1060.00, "Ania 2022-07": 960.00, "Ola 2022-07": 40.00, "june_unchanged": true}}
```
