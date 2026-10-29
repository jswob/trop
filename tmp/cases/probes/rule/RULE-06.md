# RULE-06 · A rule changed "as if since January"

**Story.** Rule B-1, version 1, from 1 January 2025: on the 15th, keep 2 000.00 DUK and sweep the rest. On 10 April 2025 Piotr and Ola write version 2: "keep 3 000.00, from January". The January, February and March sweeps each left 2 000.00.

**Rules.**
1. A new version applies from the date it states, but what was done stays done.
2. The gap between what was done and what the new version says is named, never rebooked.

**Question.** For January to March, the gap each month and in all; from April, the floor.

**Worked by hand.** Each month 3 000 − 2 000 = **1 000.00** short; in all **3 000.00**, named, not rebooked. From April the floor is 3 000.00.

```yaml
edge: "RULE-06"
inputs: {"done": {"2025-01": 2000.00, "2025-02": 2000.00, "2025-03": 2000.00}, "v2": {"floor": 3000.00, "stated": "2025-04-10", "from": "2025-01-01"}}
expect: {"kind": "value", "value": {"gap": {"2025-01": 1000.00, "2025-02": 1000.00, "2025-03": 1000.00}, "gap_total": 3000.00, "rebooked": false, "floor_from_april": 3000.00}}
```
