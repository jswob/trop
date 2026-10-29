# TIME-02 · Calendars and dates stated as rules

**Story.** Piotr's contribution for each month is due "by the 20th of the next month; if the 20th is not a business day, the next business day".

**Rules.**
1. Business days are Monday to Friday, except holidays.
2. A holiday on 22 June is in force from 2026 (it didn't exist before).

**Question.** (a) The due date when the 20th is 20 June 2026. (b) The same rule on 20 June 2020. (c) The third-to-last business day of June 2026.

**Worked by hand.** (a) 20 June 2026 is a Saturday, 21 a Sunday, 22 the new holiday: **23 June 2026**. (b) 20 June 2020 is a Saturday; 22 June 2020, a Monday, was no holiday then: **22 June 2020**. (c) June 2026 ends Tuesday 30: business days back 30, 29, 26: **26 June 2026**.

```yaml
edge: "TIME-02"
inputs: {"holidays": [{"md": "06-22", "from": 2026}], "a": "2026-06-20", "b": "2020-06-20", "c": {"year": 2026, "month": 6, "n": 3}}
expect: {"kind": "value", "value": {"a": "2026-06-23", "b": "2020-06-22", "c": "2026-06-26"}}
```
