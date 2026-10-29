# OWN-06 · Lots

**Story.** Halina bought 100 units of fund F for 5 000.00 DUK. On 10 May 2023 she gives them to Ania; that day's price is 62.00. Ania sells all 100 on 1 August 2024 at 70.00.

**Rules.**
1. A gifted lot's cost to the receiver is its value on the day of the gift; what the giver paid is not read.

**Question.** Ania's gain. (For contrast: the gain on the giver's cost.)

**Worked by hand.** Cost 100 × 62 = 6 200; proceeds 7 000; gain **800.00**. On the giver's cost it would be 2 000.00.

```yaml
edge: "OWN-06"
inputs: {"units": 100, "giver_cost": 5000.00, "gift_day_price": 62.00, "sale_price": 70.00}
expect: {"kind": "value", "value": {"cost": 6200.00, "gain": 800.00, "gain_on_giver_cost": 2000.00}}
```
