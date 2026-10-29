# ASK-04 · No advice

**Story.** Halina asks: "which loan is better?" Loan A: 20 000.00 DUK for a year, a fee of 500.00 and 6 % interest. Loan B: 20 000.00 for a year, no fee, 8 % interest.

**Rules.**
1. A question asking for a choice gets no choice; it gets both consequences and the rate at which they are equal.

**Question.** As asked.

**Worked by hand.** No choice made. A costs 500 + 1 200 = **1 700.00**; B costs **1 600.00**; they are equal when B's rate is 1 700 / 20 000 = **8.5 %**.

```yaml
edge: "ASK-04"
inputs: {"principal": 20000.00, "A": {"fee": 500.00, "rate": 0.06}, "B": {"fee": 0.00, "rate": 0.08}}
expect: {"kind": "refusal", "missing": [], "values": {"cost_A": 1700.00, "cost_B": 1600.00, "breakeven_rate_B": 0.085}}
```
