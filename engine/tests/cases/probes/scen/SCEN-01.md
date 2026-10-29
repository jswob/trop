# SCEN-01 · Scenarios and amendments

**Story.** Piotr's base plan has no loan. An amendment: a 10 000.00 DUK loan at 0 %, received 15 January 2025, repaid 1 000.00 on the 15th of each month from February, with an overpayment of 2 000.00 after April's instalment that shortens the term.

**Rules.**
1. An amendment's effect is shown as the exact difference from the base.

**Question.** On 30 June 2025: the difference in cash, the debt, the difference in net worth.

**Worked by hand.** Cash +10 000 − 5 × 1 000 (February–June) − 2 000 = **+3 000.00**. Debt 10 000 − 7 000 = **3 000.00**. Net worth difference **0.00** (at 0 % the loan moves nothing).

```yaml
edge: "SCEN-01"
inputs: {"loan": {"amount": 10000.00, "received": "2025-01-15", "instalment": 1000.00, "day": 15, "from": "2025-02"}, "overpay": {"month": "2025-04", "amount": 2000.00}, "asked": "2025-06-30"}
expect: {"kind": "relation", "value": {"cash_difference": 3000.00, "debt": 3000.00, "net_worth_difference": 0.00}}
```
