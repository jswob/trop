# Key · NUM-05

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "NUM-05"
key: {"kind": "value", "value": {"parts": {"Ania": 33.34, "Piotr": 33.33, "Marta": 33.33}, "sum": 100.00, "printed_parts": [33, 33, 33], "printed_sum": 99, "printed_total": 100}}
```

## Derivation

1. parts: floor to 0.01, leftover units in listing order (rule 1) = [33.34, 33.33, 33.33]
2. printed parts: half up to 1 (rule 2) = [33, 33, 33]
