# REC-01 · Records are the past; the model is the rest

**Story.** Piotr's model names two accounts at his bank, PKB-1201 and PKB-1202. The bank's March 2025 statement lists three: PKB-1201, PKB-1202 and PKB-7731. For PKB-1201 the model's own past gives 3 200.00 DUK on 31 March; the statement says 3 150.00.

**Rules.**
1. Every account in the records is covered by the model or flagged.
2. The model's past is compared with the statement, never booked: the residual is reported, the statement stands.

**Question.** Which accounts are flagged; the residual for PKB-1201; is anything booked?

**Worked by hand.** Flagged **PKB-7731**. Residual 3 150 − 3 200 = **−50.00** (the statement is 50.00 lower). Booked: **no**.

```yaml
edge: "REC-01"
inputs: {"model_accounts": ["PKB-1201", "PKB-1202"], "statement_accounts": ["PKB-1201", "PKB-1202", "PKB-7731"], "model_balance": 3200.00, "statement_balance": 3150.00}
expect: {"kind": "relation", "value": {"flagged": ["PKB-7731"], "residual": -50.00, "booked": false}}
```
