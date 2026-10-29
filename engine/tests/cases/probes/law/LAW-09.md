# LAW-09 · Carry-forwards with caps

**Story.** Ania's 2020 fund loss is 10 000.00 DUK. Her gains: 3 000.00 in 2021, 8 000.00 in 2022, 9 000.00 in 2023.

**Rules.**
1. A loss is used against the next 5 years' gains, in any one year at most half its original size.

**Question.** The loss used and the taxable gain each year.

**Worked by hand.** 2021: use 3 000 (left 7 000), taxable 0. 2022: at most 5 000: use **5 000** (left 2 000), taxable 3 000. 2023: use 2 000, taxable 7 000.

```yaml
edge: "LAW-09"
inputs: {"loss": 10000.00, "gains": [3000.00, 8000.00, 9000.00]}
expect: {"kind": "value", "value": {"used": [3000.00, 5000.00, 2000.00], "taxable": [0.00, 3000.00, 7000.00], "left": 0.00}}
```
