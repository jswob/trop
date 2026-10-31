# LIFE-02 · When a thing ends ★

**Story.** Ania and Tomek sell their car on 15 September 2024. Tied to it: an insurance, 1 200.00 DUK for 2024 paid in January; a credit with 8 000.00 still owed; a parking contract, 150.00 a month; the yearly car tax, 360.00 for 2024, paid in January.

**Rules.**
1. The insurance ends with the sale; the unused days from the next day to 31 December are refunded, over the policy year's 366 days, rounded half up to 0.01.
2. The credit is repaid in full from the price on the day of sale.
3. The parking contract ends at the end of the month after the notice; notice is given on the day of sale.
4. The car tax ends with the sale and is not refunded.

**Question.** For each thing tied to the car: when it ends and its money.

**Worked by hand.** Insurance: ends 15 September; unused 16 September – 31 December = 15 + 31 + 30 + 31 = 107 days: 1 200 × 107/366 = 350.8197 → **350.82** refund. Credit: ends 15 September, **8 000.00** repaid. Parking: ends **31 October**, October's **150.00** still paid. Car tax: ends 15 September, refund **0.00**.

```yaml
edge: "LIFE-02"
inputs: {"sold": "2024-09-15", "insurance": {"premium": 1200.00, "start": "2024-01-01", "end": "2024-12-31"}, "credit": 8000.00,
  "parking": {"monthly": 150.00}, "car_tax": {"yearly": 360.00, "refund": false}}
expect: {"kind": "value", "value": {"insurance": {"ends": "2024-09-15", "refund": 350.82}, "credit": {"ends": "2024-09-15", "repaid": 8000.00},
  "parking": {"ends": "2024-10-31", "still_paid": 150.00}, "car_tax": {"ends": "2024-09-15", "refund": 0.00}}}
```
