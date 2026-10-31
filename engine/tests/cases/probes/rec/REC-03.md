# REC-03 · Awkward rows

**Story.** Ania's card cycle runs 21 February – 20 March 2025, payable 10 April. Card rows: 25 February 300.00; 3 March 1 200.00; 12 March a refund of 200.00 (part of the 25 February purchase); 21 March 50.00.

**Rules.**
1. A cycle's bill is the sum of rows dated in the cycle; a refund is a negative row in the cycle it lands in.

**Question.** The bill due 10 April.

**Worked by hand.** 300 + 1 200 − 200 = **1 300.00**; the 50.00 falls in the next cycle.

```yaml
edge: "REC-03"
inputs: {"cycle": ["2025-02-21", "2025-03-20"], "due": "2025-04-10", "rows": [{"date": "2025-02-25", "amount": 300.00}, {"date": "2025-03-03", "amount": 1200.00}, {"date": "2025-03-12", "amount": -200.00}, {"date": "2025-03-21", "amount": 50.00}]}
expect: {"kind": "value", "value": {"bill": 1300.00, "due": "2025-04-10"}}
```
