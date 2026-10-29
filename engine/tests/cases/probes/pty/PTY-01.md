# PTY-01 · People and their dated relations

**Story.** Piotr and Ola live together from 2019 and marry on 8 July 2023.

**Rules.**
1. Spouses may file a joint return for a tax year (the calendar year) in which they were married throughout.

**Question.** The first tax year whose joint rule reads them as spouses. (For contrast: under a rule reading "married on the year's last day".)

**Worked by hand.** Married from 8 July 2023, so not throughout 2023; throughout 2024: **2024**. Under the last-day rule: 2023.

```yaml
edge: "PTY-01"
inputs: {"married": "2023-07-08"}
expect: {"kind": "value", "value": {"throughout": 2024, "on_last_day": 2023}}
```
