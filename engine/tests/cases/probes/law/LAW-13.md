# LAW-13 · Future law

**Story.** Ania's forecast realises a gain of 10 000.00 DUK every year 2026–2035. The gains tax is 19 %. A what-if: "the gains tax is 25 % from 1 January 2030".

**Rules.**
1. Future law is a stated what-if from a date; the run applies it from that date on and changes nothing before it.

**Question.** The tax over 2026–2035 in the base and in the what-if; 2030's tax in the what-if; are 2026–2029 unchanged?

**Worked by hand.** Base 10 × 1 900 = **19 000.00**. What-if 4 × 1 900 + 6 × 2 500 = **22 600.00**; 2030: 2 500.00; 2026–2029 unchanged: yes.

```yaml
edge: "LAW-13"
inputs: {"years": [2026, 2035], "gain": 10000.00, "base_rate": 0.19, "whatif": {"from": 2030, "rate": 0.25}}
expect: {"kind": "relation", "value": {"base_total": 19000.00, "whatif_total": 22600.00, "whatif_2030": 2500.00, "before_unchanged": true}}
```
