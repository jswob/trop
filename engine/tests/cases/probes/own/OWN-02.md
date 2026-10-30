# OWN-02 · Claims and money that isn't ours

**Story.** Piotr lent a friend 2 000.00 DUK; 1 200.00 came back. The friend promises the other 800.00 "by the end of June". On 15 May 2025 Piotr's account holds 5 000.00; his rent, 3 000.00, is planned for 1 June.

**Rules.**
1. Money lent is a claim, in net worth at its amount until paid or written off.
2. A look-ahead counts planned items only; a promise is not planned.

**Question.** Net worth on 15 May 2025; the lowest balance ahead to 30 June, and its date.

**Worked by hand.** Net worth 5 000 + 800 = **5 800.00**. Ahead: 5 000 − 3 000 = **2 000.00** on **1 June**; the 800 isn't counted.

```yaml
edge: "OWN-02"
inputs: {"cash": 5000.00, "claim": 800.00, "items": [{"date": "2025-06-01", "amount": -3000.00, "certainty": "planned"},
  {"date": "2025-06-30", "amount": 800.00, "certainty": "promised"}]}
expect: {"kind": "value", "value": {"net_worth": 5800.00, "lowest_ahead": 2000.00, "lowest_on": "2025-06-01"}}
```
