# CUR-02 · Converted amounts are whole minor units

**Story.** Ten payments of 33.33 FLR each are converted to DUK at 4.2857.

**Rules.**
1. Each converted transfer is a whole number of minor units, rounded half up.
2. A total converted at once is a view, never a booking.

**Question.** The booked total, the view's total, and the difference.

**Worked by hand.** 33.33 × 4.2857 = 142.842381 → 142.84; × 10 = **1 428.40** booked. 333.30 × 4.2857 = 1 428.42381 → **1 428.42** as a view. Difference **0.02**.

```yaml
edge: "CUR-02"
inputs: {"amount": 33.33, "count": 10, "rate": 4.2857}
expect: {"kind": "value", "value": {"booked": 1428.40, "view": 1428.42, "difference": 0.02}}
```
