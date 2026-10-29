# REC-04 · Matching and certainty

**Story.** Piotr invoices Kowal Sp. 2 000.00 DUK (INV-12), due 31 March 2025. Kowal Sp. merged into Kowal-Nowak SA on 1 February 2025. On 3 April a row shows 1 200.00 from Kowal-Nowak SA. The client writes that the rest will "probably" come next week.

**Rules.**
1. Rows match invoices by the counterparty's one registered name, within 10 days after the due date, up to the amount still open.
2. A part payment leaves the remainder open, asked for.
3. The lexicon: "probably" = promised, probability 0.75.

**Question.** The match; what is paid and what is open; the remainder's certainty.

**Worked by hand.** Kowal Sp. → Kowal-Nowak SA; 3 April is 3 days after due: matched to **INV-12**; paid **1 200.00**; open **800.00**, **promised, 0.75**.

```yaml
edge: "REC-04"
inputs: {"invoice": {"id": "INV-12", "client": "Kowal Sp.", "amount": 2000.00, "due": "2025-03-31"}, "registry": {"Kowal Sp.": "Kowal-Nowak SA"},
  "row": {"date": "2025-04-03", "from": "Kowal-Nowak SA", "amount": 1200.00}, "window_days": 10, "said": "probably", "lexicon": {"probably": ["promised", 0.75]}}
expect: {"kind": "value", "value": {"matched": "INV-12", "paid": 1200.00, "open": 800.00, "certainty": "promised", "p": 0.75}}
```
