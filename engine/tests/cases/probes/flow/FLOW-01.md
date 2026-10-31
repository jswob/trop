# FLOW-01 · Transfers and conservation

**Story.** Halina counts her wallet: 400.00 DUK on 1 January 2025 and 250.00 on 31 January. On 10 January she took 300.00 from the cash machine into it. Nothing else is recorded.

**Rules.**
1. A counted balance is an assertion; between two assertions the unexplained change is one balancing flow.
2. Start + recorded flows + balancing flow = end.

**Question.** The balancing flow, with its direction.

**Worked by hand.** 250 − (400 + 300) = −450: **450.00 out of the wallet** (spent, not itemised). 400 + 300 − 450 = 250 holds.

```yaml
edge: "FLOW-01"
inputs: {"asserted": [["2025-01-01", 400.00], ["2025-01-31", 250.00]], "flows": [["2025-01-10", 300.00]]}
expect: {"kind": "relation", "value": {"balancing_flow": -450.00, "direction": "out of the wallet", "holds": true}}
```
