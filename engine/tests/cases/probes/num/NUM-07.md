# NUM-07 · Gross, net, sign and zero

**Story.** In 2024 Tomek sold two funds: one at a gain of 1 500.00 DUK, one at a loss of 2 300.00 DUK.

**Rules.**
1. Gains and losses of a year are netted.
2. The tax is 19 % of a positive net; a negative net gives tax 0 and its size is carried to later years as a loss.

**Question.** The year's tax and the loss carried.

**Worked by hand.** Net = 1 500 − 2 300 = −800. Tax **0.00** (never −152); loss carried **800.00**.

```yaml
edge: "NUM-07"
inputs: {"results": [1500.00, -2300.00], "rate": 0.19}
expect: {"kind": "value", "value": {"net": -800.00, "tax": 0.00, "loss_carried": 800.00}}
```
