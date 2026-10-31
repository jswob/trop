# LAW-11 · A law reads history

**Story.** Halina gives Piotr 6 000.00 DUK on 1 April 2020 and 9 000.00 on 15 March 2024.

**Rules.**
1. Gifts from one giver within the 5 years before a gift are added to it; 10 000.00 of the sum is free of tax.
2. The tax is 7 % of the sum above the free amount, less what earlier gifts already used of it.

**Question.** The second gift's taxable part and tax.

**Worked by hand.** The 2020 gift is within 5 years of 15 March 2024: 6 000 + 9 000 = 15 000; above 10 000: 5 000; the first gift used none: taxable **5 000.00**, tax **350.00**.

```yaml
edge: "LAW-11"
inputs: {"gifts": [["2020-04-01", 6000.00], ["2024-03-15", 9000.00]], "free": 10000.00, "years": 5, "rate": 0.07}
expect: {"kind": "value", "value": {"taxable": 5000.00, "tax": 350.00}}
```
