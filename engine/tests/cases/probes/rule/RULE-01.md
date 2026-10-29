# RULE-01 · A rule on the real balance on a date

**Story.** On 15 June 2025, after pay, Ania's account holds 9 400.00 DUK. This year she has already put 8 500.00 into her wrapped account, whose yearly limit is 10 000.00.

**Rules.**
1. House rule A-1: on the 15th, keep 3 000.00; put the rest into the wrapped account first, up to its limit, and the remainder into the broker account.

**Question.** What goes where, and what stays.

**Worked by hand.** Surplus 6 400. Wrapped room 1 500: **1 500.00**; broker **4 900.00**; stays **3 000.00**.

```yaml
edge: "RULE-01"
inputs: {"balance": 9400.00, "floor": 3000.00, "wrapped": {"limit": 10000.00, "used": 8500.00}}
expect: {"kind": "value", "value": {"wrapped": 1500.00, "broker": 4900.00, "stays": 3000.00}}
```
