# OWN-05 · Events that move ownership ★

**Story.** Halina and Zbigniew own a flat as community property, worth 400 000.00 DUK when he dies on 10 March 2024. At the division on 1 June 2025, valued at 480 000.00, Halina keeps the flat and buys out the four other heirs.

**Rules.**
1. At a death, community property splits in halves; the deceased's half is the estate.
2. The estate passes Halina 1/4, Ania, Piotr, Marta and Kuba 3/16 each.
3. A buyout pays each heir's share at the division's valuation.

**Question.** Halina's share of the flat after the death; each other heir's share; each payout; what Halina pays in all; what each payout would be at the death's value.

**Worked by hand.** Halina 1/2 + 1/2 × 1/4 = **5/8**; each other heir 1/2 × 3/16 = **3/32**. Payout 3/32 × 480 000 = **45 000.00** each; Halina pays **180 000.00**. At 400 000 it would be 37 500.00.

```yaml
edge: "OWN-05"
inputs: {"value_at_death": 400000.00, "value_at_division": 480000.00, "keeper": "Halina",
  "estate": {"Halina": "1/4", "Ania": "3/16", "Piotr": "3/16", "Marta": "3/16", "Kuba": "3/16"}}
expect: {"kind": "value", "value": {"shares": {"Halina": "5/8", "Ania": "3/32", "Piotr": "3/32", "Marta": "3/32", "Kuba": "3/32"},
  "payouts": {"Ania": 45000.00, "Piotr": 45000.00, "Marta": 45000.00, "Kuba": 45000.00}, "keeper_pays": 180000.00, "payout_at_death_value": 37500.00}}
```
