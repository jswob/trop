# RULE-02 · What a rule may see

**Story.** On 15 May 2025 Ania's account holds 4 000.00 DUK. Planned: rent 2 500.00 on 1 June, a loan instalment 1 200.00 on 5 June, pay 6 000.00 on 10 June. A friend promises to return 800.00 "around 3 June".

**Rules.**
1. A look-ahead sees planned items only; a promise is not planned.

**Question.** The lowest balance to 30 June and its date; what it would be if the promise were counted.

**Worked by hand.** 1 June 1 500; 5 June **300.00**; 10 June 6 300: lowest **300.00 on 5 June**. Counting the promise: 1 500, 2 300, 1 100: 1 100.00.

```yaml
edge: "RULE-02"
inputs: {"balance": 4000.00, "items": [{"date": "2025-06-01", "amount": -2500.00, "certainty": "planned"}, {"date": "2025-06-03", "amount": 800.00, "certainty": "promised"},
  {"date": "2025-06-05", "amount": -1200.00, "certainty": "planned"}, {"date": "2025-06-10", "amount": 6000.00, "certainty": "planned"}]}
expect: {"kind": "value", "value": {"lowest": 300.00, "on": "2025-06-05", "if_promise_counted": 1100.00}}
```
