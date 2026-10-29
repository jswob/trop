# OWN-01 · Places, their kinds and lifetimes

**Story.** Ania has had a current account for years; on 15 February 2025 it holds 1 000.00 DUK. She opens a savings account on 1 March 2025; it charges a fee of 5.00 at each month's end while open.

**Rules.**
1. A place exists from its opening to its closing; before it opens it has no line, not a zero line.
2. The savings fee is charged at the end of each month the account is open.

**Question.** (a) The savings fees in January–March 2025. (b) Ania's net worth on 15 February 2025, and its lines.

**Worked by hand.** (a) Only March's end falls while open: **5.00**. (b) Only the current account exists: **1 000.00**, one line (current).

```yaml
edge: "OWN-01"
inputs: {"places": [{"id": "current", "opened": "2015-01-01", "balance_on": {"2025-02-15": 1000.00}},
  {"id": "savings", "opened": "2025-03-01", "fee": 5.00}], "quarter": ["2025-01", "2025-03"], "asked": "2025-02-15"}
expect: {"kind": "value", "value": {"fees_q1": 5.00, "net_worth": 1000.00, "lines": ["current"]}}
```
