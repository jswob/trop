# TIME-03 · The period an amount is for, and the day it is paid

**Story.** Ania's pay for June 2022, 8 000.00 DUK gross, is paid on 10 July 2022.

**Rules.**
1. The tax on pay is 17 % for pay paid up to 30 June 2022 and 12 % for pay paid from 1 July 2022. The act keys on the day of payment.
2. The tax is rounded half up to 0.01.

**Question.** The tax on June's pay. What would a law keyed on the period give?

**Worked by hand.** Paid 10 July: 12 % × 8 000 = **960.00**. Keyed on the period (June): 17 % × 8 000 = 1 360.00. The pay's period never sets the law date by itself.

```yaml
edge: "TIME-03"
inputs: {"gross": 8000.00, "period": "2022-06", "paid": "2022-07-10",
  "versions": [{"from": "2022-01-01", "to": "2022-06-30", "rate": 0.17}, {"from": "2022-07-01", "to": "2022-12-31", "rate": 0.12}]}
expect: {"kind": "value", "value": {"by_payment": 960.00, "by_period": 1360.00}}
```
