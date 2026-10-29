# ASK-02 · Net worth per person, with holes ★

**Story.** On 30 June 2025 Piotr and Ola own a flat half each, valued 400 000.00 DUK on 10 January 2025. Piotr's account holds 20 000.00. Ola owns a car nobody has valued.

**Rules.**
1. A person's net worth sums their shares of what they own at its last valuation; something with no valuation is a hole, never 0.

**Question.** Each person's net worth on 30 June 2025.

**Worked by hand.** Piotr 200 000 + 20 000 = **220 000.00**. Ola **200 000.00 plus a hole**: the car's value on 30 June 2025. Partial.

```yaml
edge: "ASK-02"
inputs: {"asked": "2025-06-30", "holdings": [{"what": "flat", "value": 400000.00, "owners": {"Piotr": "1/2", "Ola": "1/2"}}, {"what": "account", "value": 20000.00, "owners": {"Piotr": "1"}},
  {"what": "car", "value": null, "owners": {"Ola": "1"}}]}
expect: {"Piotr": 220000.00, "Ola": {"kind": "partial", "value": 200000.00, "missing": ["value of Ola's car on 2025-06-30"]}}
```
