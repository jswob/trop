# TIME-07 · Order within one day

**Story.** On 15 May 2025 Ania's account holds 1 500.00 DUK before anything happens. Her pay, 6 000.00, lands that day, and the 15th rule runs that day.

**Rules.**
1. The 15th rule: keep a floor of 2 000.00; move the rest to savings.
2. The stated order: on one day, incoming pay is booked before house rules run.

**Question.** How much does the rule move? What would the reverse order give?

**Worked by hand.** Pay first: 7 500 − 2 000 = **5 500.00** moved. Rule first: 1 500 is under the floor, nothing moves, then pay lands: 0.00. The order is stated, never read from file order.

```yaml
edge: "TIME-07"
inputs: {"before": 1500.00, "pay": 6000.00, "floor": 2000.00}
expect: {"kind": "value", "value": {"stated_order": 5500.00, "reverse_order": 0.00}}
```
