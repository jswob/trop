# VAL-04 · A published figure that disagrees with its law ★

**Story.** Ania sells gifted fund units for 8 900.00 DUK. The giver paid 4 700.00; the units were worth 6 000.00 on the day of the gift. After the units moved brokers, the new broker's yearly statement shows a taxable gain of 4 200.00.

**Rules.**
1. A gifted lot's cost is its value on the day of the gift.
2. A record's figure is kept as stated; the law's figure is computed; the gap is named with its cause; neither is corrected silently.

**Question.** The statement's gain, the law's gain, the gap and its cause.

**Worked by hand.** Statement **4 200.00** (8 900 − 4 700: the giver's cost). Law 8 900 − 6 000 = **2 900.00**. Gap **1 300.00**: the cost used, 4 700.00 vs 6 000.00 by law.

```yaml
edge: "VAL-04"
inputs: {"proceeds": 8900.00, "giver_cost": 4700.00, "gift_day_value": 6000.00, "statement_gain": 4200.00}
expect: {"kind": "value", "value": {"statement": 4200.00, "law": 2900.00, "gap": 1300.00, "cause": {"cost_in_statement": 4700.00, "cost_by_law": 6000.00}}}
```
