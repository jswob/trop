# LAW-10 · A result pending until a window closes

**Story.** Halina sells a flat on 1 June 2024 for 500 000.00 DUK, a gain of 100 000.00. In 2025 she spends 300 000.00 of it on her own home. It is now 2026.

**Rules.**
1. The gain is taxed at 19 %, except the part spent on one's own home by the end of the third year after the year of sale: exempt gain = gain × spent / proceeds (at most all of it).

**Question.** The tax.

**Worked by hand.** The window closes **31 December 2027**. If nothing more is spent: exempt 100 000 × 300/500 = 60 000; tax 19 % × 40 000 = **7 600.00**. If a further 200 000 is spent by then: **0.00**. By-case until the window closes.

```yaml
edge: "LAW-10"
inputs: {"sold": "2024-06-01", "proceeds": 500000.00, "gain": 100000.00, "spent": 300000.00, "rate": 0.19, "branches": {"no more spent": 0.00, "200 000 more spent": 200000.00}}
expect: {"kind": "by-case", "cases": {"no more spent": 7600.00, "200 000 more spent": 0.00}, "settles": "2027-12-31"}
```
