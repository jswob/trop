# LAW-06 · Elections

**Story.** Tomek earns 90 000.00 DUK and Ania 30 000.00 in 2023 and 2024. For 2023 their dossier states no filing choice; for 2024 they state "joint".

**Rules.**
1. Spouses elect, per year, to file jointly or separately; no election, no tax.
2. Separate: each pays the scale on their own income. Joint: twice the scale on half the combined income.
3. The scale: 12 % up to and including 60 000.00, 32 % on the excess.

**Question.** (a) Their 2023 tax. (b) Their 2024 tax, with both sides shown.

**Worked by hand.** (a) **Refused**: the 2023 filing election is missing. (b) Joint 2 × 7 200 = 14 400.00; separate 16 800 + 3 600 = 20 400.00; elected joint: **14 400.00**.

```yaml
edge: "LAW-06"
inputs: {"incomes": {"Tomek": 90000.00, "Ania": 30000.00}, "bands": [[60000.00, 0.12], [null, 0.32]], "elections": {"2023": null, "2024": "joint"}}
expect: {"a": {"kind": "refusal", "missing": ["filing election for 2023"]}, "b": {"kind": "value", "value": {"joint": 14400.00, "separate": 20400.00, "elected": "joint", "tax": 14400.00}}}
```
