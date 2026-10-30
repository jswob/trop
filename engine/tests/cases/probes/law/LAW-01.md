# LAW-01 · Edges: thresholds, brackets and tables

**Story.** A benefit is set by yearly income in a table.

**Rules.**
1. Income not more than 50 000.00 DUK: 200.00. More than 50 000.00 and less than 80 000.00: 100.00. Not less than 80 000.00: 0.00.

**Question.** The benefit at 49 999.99, 50 000.00, 50 000.01, 79 999.99, 80 000.00 and 80 000.01.

**Worked by hand.** 200.00, **200.00** (the edge belongs to the lower band), 100.00, 100.00, **0.00** (the edge belongs to the upper band), 0.00.

```yaml
edge: "LAW-01"
inputs: {"rows": [{"lo": null, "lo_incl": false, "hi": 50000.00, "hi_incl": true, "value": 200.00},
  {"lo": 50000.00, "lo_incl": false, "hi": 80000.00, "hi_incl": false, "value": 100.00},
  {"lo": 80000.00, "lo_incl": true, "hi": null, "hi_incl": false, "value": 0.00}],
  "at": [49999.99, 50000.00, 50000.01, 79999.99, 80000.00, 80000.01]}
expect: {"kind": "value", "value": [200.00, 200.00, 100.00, 100.00, 0.00, 0.00]}
```
