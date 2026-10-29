# LAW-02 · Cliffs and tapers

**Story.** A relief and a contribution tier, each tested at its edge.

**Rules.**
1. The relief is 1 200.00 DUK for income not more than 100 000.00; above it, it is cut by 1.00 for each 1.00 of income over, never below 0.
2. The contribution is 300.00 a month for yearly revenue not more than 60 000.00, and 500.00 for more.

**Question.** The relief at 100 000.00 and 100 000.01; the contribution at 60 000.00 and 60 000.01.

**Worked by hand.** Relief **1 200.00**, then **1 199.99** (a taper). Contribution **300.00**, then **500.00** (a cliff: one minor unit costs 200.00 a month).

```yaml
edge: "LAW-02"
inputs: {"taper": {"full": 1200.00, "limit": 100000.00, "at": [100000.00, 100000.01]},
  "cliff": {"rows": [{"lo": null, "lo_incl": false, "hi": 60000.00, "hi_incl": true, "value": 300.00}, {"lo": 60000.00, "lo_incl": false, "hi": null, "hi_incl": false, "value": 500.00}], "at": [60000.00, 60000.01]}}
expect: {"kind": "value", "value": {"relief": [1200.00, 1199.99], "contribution": [300.00, 500.00]}}
```
