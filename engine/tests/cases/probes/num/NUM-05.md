# NUM-05 · Parts and wholes

**Story.** Three siblings, Ania, Piotr and Marta, share a refund of 100.00 DUK equally. A summary prints each part and the total in whole DUK.

**Rules.**
1. Each part is a third, floored to 0.01; the minor units left over go one each to the heirs in the order listed.
2. Printed figures are rounded half up to whole DUK for display only.

**Question.** Each part; the sum of the parts; the printed parts, their sum and the printed total.

**Worked by hand.** 10 000 minor units / 3 = 3 333 rem 1: Ania 33.34, Piotr 33.33, Marta 33.33; sum **100.00**. Printed: 33, 33, 33, summing to 99, while the printed total is 100: display rounding, fed to nothing.

```yaml
edge: "NUM-05"
inputs: {"total": 100.00, "heirs": ["Ania", "Piotr", "Marta"]}
expect: {"kind": "value", "value": {"parts": {"Ania": 33.34, "Piotr": 33.33, "Marta": 33.33}, "sum": 100.00,
  "printed_parts": [33, 33, 33], "printed_sum": 99, "printed_total": 100}}
```
