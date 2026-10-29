# LAW-04 · Retroactive law

**Story.** Ania earns 8 000.00 DUK a month in 2022. Her advances were 17 % to June and 12 % from July. An act published in July cuts the rate to 12 % for the whole of 2022; she files her return on 30 April 2023.

**Rules.**
1. The return applies the year's law to the year's income; what was withheld is credited.
2. A change covering a closed period is booked as a refund or arrears on the day it is settled, never by rewriting past months.

**Question.** What the return settles, with its direction and date.

**Worked by hand.** Withheld 6 × 1 360 + 6 × 960 = 13 920. Due 12 % × 96 000 = 11 520. **Refund 2 400.00 to Ania**, booked 30 April 2023; the months stay as they were.

```yaml
edge: "LAW-04"
inputs: {"monthly": 8000.00, "withheld_rates": [0.17, 0.17, 0.17, 0.17, 0.17, 0.17, 0.12, 0.12, 0.12, 0.12, 0.12, 0.12], "year_rate": 0.12, "filed": "2023-04-30"}
expect: {"kind": "value", "value": {"withheld": 13920.00, "due": 11520.00, "settlement": 2400.00, "direction": "refund to Ania", "booked": "2023-04-30"}}
```
