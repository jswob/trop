# OWN-08 · Kept-apart parts and their limits

**Story.** Ania's wrapped account takes contributions up to a yearly limit: 10 000.00 DUK in 2024, 10 500.00 in 2025. By 14 December 2024 she has put in 9 600.00. On 15 December 2024 and on 15 January 2025 her rule wants to put in 1 000.00. Later the account holds 10 000.00 of contributions and 1 500.00 of earnings, and she withdraws 10 500.00.

**Rules.**
1. A contribution is taken only up to the year's limit left.
2. A withdrawal comes from contributions first, earnings last; earnings withdrawn are taxed at 19 %, rounded half up to 0.01.

**Question.** What goes in on each 15th; the withdrawal's parts and its tax.

**Worked by hand.** December: room 400: **400.00**. January: a new year's limit: **1 000.00**. Withdrawal: 10 000 contributions, 500 earnings, tax **95.00**.

```yaml
edge: "OWN-08"
inputs: {"limits": {"2024": 10000.00, "2025": 10500.00}, "used": {"2024": 9600.00, "2025": 0.00}, "want": 1000.00,
  "withdrawal": {"amount": 10500.00, "contributions": 10000.00, "earnings": 1500.00, "rate": 0.19}}
expect: {"kind": "value", "value": {"dec_2024": 400.00, "jan_2025": 1000.00, "withdrawal": {"from_contributions": 10000.00, "from_earnings": 500.00, "tax": 95.00}}}
```
