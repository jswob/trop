# Key · LAW-01

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "LAW-01"
key: {"kind": "value", "value": [200.00, 200.00, 100.00, 100.00, 0.00, 0.00]}
```

## Derivation

1. benefit at 49999.99: rule 1 = 200.00
2. benefit at 50000.00: rule 1 = 200.00
3. benefit at 50000.01: rule 1 = 100.00
4. benefit at 79999.99: rule 1 = 100.00
5. benefit at 80000.00: rule 1 = 0.00
6. benefit at 80000.01: rule 1 = 0.00
