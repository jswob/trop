# RULE-08 · Parameters estimated from history

**Story.** Ania's spending: January 3 100.00, February 2 900.00, March 3 650.00, April 3 200.00, May 2 950.00 DUK.

**Rules.**
1. The 15th rule's floor is the mean of the three full months before the decision's month, rounded half up to 0.01.

**Question.** The floor on 15 April and on 15 June 2025.

**Worked by hand.** 15 April: (3 100 + 2 900 + 3 650) / 3 = 3 216.667 → **3 216.67**. 15 June: (3 650 + 3 200 + 2 950) / 3 = 3 266.667 → **3 266.67**.

```yaml
edge: "RULE-08"
inputs: {"spending": {"2025-01": 3100.00, "2025-02": 2900.00, "2025-03": 3650.00, "2025-04": 3200.00, "2025-05": 2950.00}, "decisions": ["2025-04-15", "2025-06-15"], "months": 3}
expect: {"kind": "value", "value": {"2025-04-15": 3216.67, "2025-06-15": 3266.67}}
```
