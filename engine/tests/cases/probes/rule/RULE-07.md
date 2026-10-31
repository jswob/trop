# RULE-07 · What a rule meant vs what the records show

**Story.** A standing order should move 500.00 DUK into savings on the 1st of each month, January to June 2025. Savings held 1 000.00 on 31 December 2024. The bank rows show the transfers for January, February, April, May and June.

**Rules.**
1. The records win; the gap between the rule and the records is named.

**Question.** The savings on 30 June 2025 by the records and by the rule; the gap.

**Worked by hand.** Records: 1 000 + 5 × 500 = **3 500.00**. Rule: 4 000.00. Gap **500.00, March 2025**.

```yaml
edge: "RULE-07"
inputs: {"start": 1000.00, "amount": 500.00, "months": ["2025-01", "2025-02", "2025-03", "2025-04", "2025-05", "2025-06"], "rows": ["2025-01", "2025-02", "2025-04", "2025-05", "2025-06"]}
expect: {"kind": "value", "value": {"by_records": 3500.00, "by_rule": 4000.00, "gap": {"2025-03": 500.00}}}
```
