# NUM-04 · An unclear convention, keyed under each reading

**Story.** Ania took a consumer credit of 24 equal principal instalments; its non-interest cost was 1 200.00 DUK. She repays it in full after the 8th instalment.

**Rules.**
1. On early repayment the lender refunds "the cost for the remaining period". The act doesn't say how.
2. Reading T: pro rata by time, months left / all months.
3. Reading B: by the falling balance, the balances still owed in the months left / the balances of all months (equal principal: the balances run 24, 23, …, 1 parts).

**Question.** What is the refund?

**Worked by hand.** Reading T: 1 200 × 16/24 = **800.00**. Reading B: months 9–24 owe 16 + 15 + … + 1 = 136 parts of 24 + … + 1 = 300: 1 200 × 136/300 = **544.00**.

```yaml
edge: "NUM-04"
inputs: {"cost": 1200.00, "months": 24, "paid": 8}
expect: {"kind": "by-case", "cases": {"T: pro rata by time": 800.00, "B: by the falling balance": 544.00}}
```
