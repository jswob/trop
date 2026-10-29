# VAL-05 · Prices come from records

**Story.** Ania holds 120 units of fund F. The fund's price file has 23 December 2024 50.80 and 27 December 2024 51.20, then nothing until January. The forecast's price file for F gives March 2027 63.40.

**Rules.**
1. A holding is units × the recorded price of the day; a day with no price reads the last earlier one, and says which.
2. A forecast resolves prices by month: a sale in a month uses that month's price.

**Question.** (a) The holding's value on 31 December 2024. (b) A forecast sale of 50 units on 15 March 2027.

**Worked by hand.** (a) Last price 27 December, 51.20: 120 × 51.20 = **6 144.00**. (b) 50 × 63.40 = **3 170.00**.

```yaml
edge: "VAL-05"
inputs: {"units": 120, "prices": {"2024-12-23": 50.80, "2024-12-27": 51.20}, "asked": "2024-12-31",
  "forecast": {"2027-03": 63.40}, "sale": {"date": "2027-03-15", "units": 50}}
expect: {"kind": "value", "value": {"a": {"price_date": "2024-12-27", "value": 6144.00}, "b": 3170.00}}
```
