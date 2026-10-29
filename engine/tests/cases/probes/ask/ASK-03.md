# ASK-03 · Holes, refusals and warnings

**Story.** (a) Marek's 2024 return form F-12 leaves the field "days abroad" blank; a relief applies only above 183 days abroad. (b) Tomek holds 100 units of fund G and his plan sells 120.

**Rules.**
1. A blank field the law needs is a hole, never 0.
2. Selling more than is held is refused.

**Question.** (a) Does the relief apply? (b) The sale's gain.

**Worked by hand.** (a) **Refused**: days abroad in 2024, blank on F-12. (b) **Refused**: 100 units held, 120 sold.

```yaml
edge: "ASK-03"
inputs: {"a": {"form": "F-12", "year": 2024, "fields": {"days_abroad": null}, "needs": "days_abroad", "limit": 183},
  "b": {"lots": [{"id": "G1", "account": "X", "date": "2023-01-10", "units": 100, "cost": 5000.00}], "sell": 120}}
expect: {"a": {"kind": "refusal", "missing": ["days abroad in 2024 (blank on F-12)"]}, "b": {"kind": "refusal", "missing": ["units held: 100 < 120 sold"]}}
```
