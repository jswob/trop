# SCEN-02 · A counterfactual past ★

**Story.** Ania and Tomek sold their car on 15 September 2024: they got 23 050.00 DUK after the credit's payoff, and an insurance refund of 350.82 on 20 September; they gave notice on their parking, paid for the last time on 1 October. What if they had kept it?

**Rules.**
1. A branch from 15 September 2024 drops every later fact that needs the sale and keeps the rest.
2. Kept, the credit runs on (400.00 on the 20th of each month) and the parking runs on (150.00 on the 1st).

**Question.** Which facts are dropped and added; the branch's cash difference on 31 December 2024.

**Worked by hand.** Dropped: the sale, the refund. Kept: the October parking payment. Added: credit instalments 20 September, October, November, December (1 600.00) and parking 1 November and 1 December (300.00). Difference −23 050 − 350.82 − 1 600 − 300 = **−25 300.82**.

```yaml
edge: "SCEN-02"
inputs: {"branch": "2024-09-15", "asked": "2024-12-31",
  "actual": [{"id": "sale", "date": "2024-09-15", "amount": 23050.00, "needs_sale": true}, {"id": "insurance refund", "date": "2024-09-20", "amount": 350.82, "needs_sale": true},
    {"id": "parking 2024-10", "date": "2024-10-01", "amount": -150.00, "needs_sale": false}],
  "if_kept": [{"id": "credit", "amount": -400.00, "day": 20, "from": "2024-09"}, {"id": "parking", "amount": -150.00, "day": 1, "from": "2024-11"}]}
expect: {"kind": "value", "value": {"dropped": ["sale", "insurance refund"], "kept": ["parking 2024-10"], "added": ["credit 2024-09", "credit 2024-10", "credit 2024-11", "credit 2024-12", "parking 2024-11", "parking 2024-12"], "cash_difference": -25300.82}}
```
