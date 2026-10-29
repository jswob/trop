# ASK-01 · Question primitives

**Story.** Ola's account holds 5 000.00 DUK on 1 March 2025. March: 5th −1 500.00; 10th +4 000.00; 12th −3 000.00; 20th −2 600.00; 25th −500.00; 28th +6 000.00.

**Rules.**
1. The balance after each day's rows is the day's balance.

**Question.** The first date the balance is below 2 000.00; the month's minimum and its date.

**Worked by hand.** 3 500; 7 500; 4 500; **1 900 on 20 March** (first below); 1 400 on 25 March; 7 400. Minimum **1 400.00 on 25 March**.

```yaml
edge: "ASK-01"
inputs: {"start": 5000.00, "floor": 2000.00, "items": [{"date": "2025-03-05", "amount": -1500.00}, {"date": "2025-03-10", "amount": 4000.00}, {"date": "2025-03-12", "amount": -3000.00},
  {"date": "2025-03-20", "amount": -2600.00}, {"date": "2025-03-25", "amount": -500.00}, {"date": "2025-03-28", "amount": 6000.00}]}
expect: {"kind": "value", "value": {"first_below": "2025-03-20", "minimum": 1400.00, "minimum_on": "2025-03-25"}}
```
