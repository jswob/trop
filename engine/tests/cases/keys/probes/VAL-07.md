# Key · VAL-07

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "VAL-07"
key: {"kind": "value", "value": {"rent_2027": 2163.20, "register": ["A-RENT", "A-CPI"]}}
```

## Derivation

1. register: assumptions read, transitively (rule 1) = ["A-RENT", "A-CPI"]
2. rent 2026: × (1 + 0.04) (rule 2) = 2080.00
3. rent 2027: × (1 + 0.04) (rule 2) = 2163.20
