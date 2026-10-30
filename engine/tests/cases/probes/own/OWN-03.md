# OWN-03 · Owed but not yet paid

**Story.** On 31 December 2024 Ola holds 10 000.00 DUK. Her tax for 2024, 1 840.00, is due by 30 April 2025. The run ends 31 March 2025; her cash doesn't change before then.

**Rules.**
1. Tax for a year is owed from the year's end, paid when due.
2. An amount due after the run's end still shows as owed at the end.

**Question.** Net worth on 31 December 2024; what is still owed at the run's end, and when it is due.

**Worked by hand.** 10 000 − 1 840 = **8 160.00**. At 31 March 2025: **1 840.00** owed, due **30 April 2025**.

```yaml
edge: "OWN-03"
inputs: {"cash": 10000.00, "tax": 1840.00, "due": "2025-04-30", "run_end": "2025-03-31"}
expect: {"kind": "value", "value": {"net_worth_dec31": 8160.00, "owed_at_run_end": 1840.00, "due": "2025-04-30"}}
```
