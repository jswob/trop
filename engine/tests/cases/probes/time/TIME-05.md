# TIME-05 · Clocks, windows and deadlines ★

**Story.** Three clocks. (a) Units bought 31 August 2022, sold 28 February 2023. (b) A flat acquired 15 March 2019, sold 30 December 2024. (c) Zbigniew dies 30 April 2024; Halina reports the inheritance on 31 October 2024.

**Rules.**
1. (a) A sale within 6 months of the purchase day is short-term. The period ends on the same-numbered day; a month lacking it, on its last day; the end day is inside.
2. (b) A flat's sale is exempt if it falls after 5 years counted from the end of the year of acquisition.
3. (c) The inheritance must be reported within 6 months counted from the end of the month of death; a period counted from a month's last day ends on the last day of its final month.

**Question.** (a) Short-term or not, and the window's end. (b) Exempt or not, and the first exempt day. (c) The deadline; in time or not; what day-for-day counting would have given.

**Worked by hand.** (a) 31 August + 6 months: no 31 February, so 28 February 2023; the sale is on the end day: **short-term**. (b) 31 December 2019 + 5 years = 31 December 2024; 30 December is not after it: **not exempt**, first exempt day **1 January 2025**. (c) From 30 April (the month's last day) + 6 months: the last day of October, **31 October 2024**: **in time**. Day-for-day would end 30 October and call it late.

```yaml
edge: "TIME-05"
inputs: {"a": {"bought": "2022-08-31", "sold": "2023-02-28", "months": 6},
  "b": {"acquired": "2019-03-15", "sold": "2024-12-30", "years": 5},
  "c": {"death": "2024-04-30", "reported": "2024-10-31", "months": 6}}
expect: {"kind": "value", "value": {"a": {"window_end": "2023-02-28", "short_term": true},
  "b": {"clock_end": "2024-12-31", "exempt": false, "first_exempt_day": "2025-01-01"},
  "c": {"deadline": "2024-10-31", "in_time": true, "day_for_day_deadline": "2024-10-30"}}}
```
