# OWN-07 · Matching sales to lots

**Story.** Tomek holds fund G in two accounts: X: L1, 10 units bought 10 January 2022 for 1 000.00; Y: L2, 10 units bought 1 March 2022 for 1 200.00; X: L3, 10 units bought 1 June 2022 for 900.00. In 2023 he sells 15 units from X at 130.00.

**Rules.**
1. A sale is matched first in, first out, within the account it is sold from.

**Question.** The cost and gain; for contrast, FIFO across both accounts.

**Worked by hand.** Per account: L1 10 units (1 000) + 5 of L3 (450) = 1 450; proceeds 1 950; gain **500.00**. Across accounts: L1 + 5 of L2 (600) = 1 600; gain 350.00.

```yaml
edge: "OWN-07"
inputs: {"lots": [{"id": "L1", "account": "X", "date": "2022-01-10", "units": 10, "cost": 1000.00},
  {"id": "L2", "account": "Y", "date": "2022-03-01", "units": 10, "cost": 1200.00},
  {"id": "L3", "account": "X", "date": "2022-06-01", "units": 10, "cost": 900.00}], "sell": {"account": "X", "units": 15, "price": 130.00}}
expect: {"kind": "value", "value": {"cost": 1450.00, "gain": 500.00, "gain_across_accounts": 350.00}}
```
