# RULE-05 · "Once, when…" on a state

**Story.** Ania and Tomek's car savings hold 30 000.00 DUK on 31 January 2025 and grow by 2 500.00 at each month's end.

**Rules.**
1. House rule A-3: once the savings reach 40 000.00, buy the car on that day; its credit's instalments fall on the 10th, from the next month.
2. The 15th rule sees every instalment already paid that month.

**Question.** The trigger date; the first instalment; the first 15th that sees it.

**Worked by hand.** February 32 500, March 35 000, April 37 500, May 40 000: trigger **31 May 2025**. First instalment **10 June 2025**; seen by the 15th rule on **15 June 2025**.

```yaml
edge: "RULE-05"
inputs: {"start": ["2025-01-31", 30000.00], "monthly": 2500.00, "threshold": 40000.00, "instalment_day": 10, "rule_day": 15}
expect: {"kind": "value", "value": {"trigger": "2025-05-31", "first_instalment": "2025-06-10", "first_seen": "2025-06-15"}}
```
