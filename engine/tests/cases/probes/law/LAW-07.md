# LAW-07 · Eligibility and classification from facts

**Story.** Piotr is VAT-exempt by revenue. By 30 September 2024 his year's revenue is 192 500.00 DUK. In October he issues #41 (3 Oct) 4 000.00, #42 (12 Oct) 5 000.00, #43 (20 Oct) 2 000.00, all net.

**Rules.**
1. The exemption is lost when the year's revenue exceeds 200 000.00; VAT applies from the sale by which it is exceeded, on that sale's whole value.
2. VAT is 23 %, rounded half up to 0.01.

**Question.** The first invoice with VAT, and the VAT on October's invoices.

**Worked by hand.** After #41: 196 500. After #42: 201 500, more than 200 000: **#42**. VAT 23 % × (5 000 + 2 000) = **1 610.00**.

```yaml
edge: "LAW-07"
inputs: {"before": 192500.00, "threshold": 200000.00, "rate": 0.23, "invoices": [["#41", "2024-10-03", 4000.00], ["#42", "2024-10-12", 5000.00], ["#43", "2024-10-20", 2000.00]]}
expect: {"kind": "value", "value": {"first_with_vat": "#42", "vat": 1610.00}}
```
