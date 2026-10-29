# FLOW-03 · Shared costs and netting ★

**Story.** A flat is owned by Halina 1/2, Ania 1/4 and Kuba 1/4. In 2025 Halina pays all its costs, 1 200.00 DUK; the tenant pays the year's rent, 2 000.00, into Kuba's account.

**Rules.**
1. Costs and rent are shared by ownership.
2. Each owner's balance = costs paid − share of costs + share of rent − rent received; + is owed to them.

**Question.** Each owner's balance.

**Worked by hand.** Halina 1 200 − 600 + 1 000 − 0 = **+1 600.00**; Ania 0 − 300 + 500 − 0 = **+200.00**; Kuba 0 − 300 + 500 − 2 000 = **−1 800.00** (Kuba owes). They add up to 0.

```yaml
edge: "FLOW-03"
inputs: {"shares": {"Halina": "1/2", "Ania": "1/4", "Kuba": "1/4"}, "costs": [["Halina", 1200.00]], "rents": [["Kuba", 2000.00]]}
expect: {"kind": "relation", "value": {"balances": {"Halina": 1600.00, "Ania": 200.00, "Kuba": -1800.00}, "sum": 0.00}}
```
