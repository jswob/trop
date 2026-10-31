# Key · LAW-11

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "LAW-11"
key: {"kind": "value", "value": {"taxable": 5000.00, "tax": 350.00}}
```

## Derivation

1. sum within 5 years: gifts after 2019-03-15 + this one (rule 1) = 15000.00
2. taxable: above 10000.00, less earlier taxed 0 (rule 2) = 5000.00
