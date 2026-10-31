# CUR-03 · Revaluation is not a transfer

**Story.** Piotr holds 2 000.00 FLR from 31 May to 30 June 2024; nothing comes in or goes out.

**Rules.**
1. A foreign balance is valued in DUK at the day's stated rate: 4.30 on 31 May, 4.25 on 30 June.
2. A revaluation moves net worth, never the foreign balance, and is no transfer.

**Question.** The DUK value on each day, the revaluation, the FLR balance, the transfers.

**Worked by hand.** 8 600.00 on 31 May; 8 500.00 on 30 June; revaluation **−100.00** (a loss); FLR balance 2 000.00 both days; no transfers.

```yaml
edge: "CUR-03"
inputs: {"balance": 2000.00, "rates": {"2024-05-31": 4.30, "2024-06-30": 4.25}}
expect: {"kind": "relation", "value": {"values": {"2024-05-31": 8600.00, "2024-06-30": 8500.00}, "revaluation": -100.00, "flr_balance": [2000.00, 2000.00], "transfers": 0}}
```
