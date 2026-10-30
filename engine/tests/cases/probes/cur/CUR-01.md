# CUR-01 · Which rate for each leg ★

**Story.** On Friday 15 March 2024 Piotr invoices a foreign client 1 000.00 FLR. The client pays on 15 April; his bank converts the money at its own rate, 4.2510 DUK/FLR.

**Rules.**
1. Revenue in a foreign currency is converted at the central bank's mid rate of the last business day before the invoice date.
2. The central bank's mid rates: 13 March 4.3050, 14 March 4.3125, 15 March 4.3200.
3. Converted amounts are rounded half up to 0.01 DUK.

**Question.** The revenue in DUK, the cash received in DUK, and the difference.

**Worked by hand.** The last business day before Friday 15 March is Thursday 14 March: 4.3125 × 1 000 = **4 312.50** revenue. Cash 4.2510 × 1 000 = **4 251.00**. Difference **61.50**, a currency loss.

```yaml
edge: "CUR-01"
inputs: {"amount": 1000.00, "issued": "2024-03-15", "mid": {"2024-03-13": 4.3050, "2024-03-14": 4.3125, "2024-03-15": 4.3200}, "bank_rate": 4.2510}
expect: {"kind": "value", "value": {"law_rate_date": "2024-03-14", "revenue": 4312.50, "cash": 4251.00, "fx_loss": 61.50}}
```
