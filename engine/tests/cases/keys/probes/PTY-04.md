# Key · PTY-04

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "PTY-04"
key: {"kind": "value", "value": {"before": {"eligible": true, "Ania": 500.00, "Tomek": 500.00}, "after": {"eligible": false, "Ania": 0.00, "Tomek": 0.00}}}
```

## Derivation

1. before: joint income: 60000.00 + 52000.00 = 112000.00
2. before: Ania: not more than 112000.00: True (rules 1–2) = 500.00
3. before: Tomek: not more than 112000.00: True (rules 1–2) = 500.00
4. after: joint income: 60000.00 + 52000.01 = 112000.01
5. after: Ania: not more than 112000.00: False (rules 1–2) = 0.00
6. after: Tomek: not more than 112000.00: False (rules 1–2) = 0.00
