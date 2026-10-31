# Key · REC-02

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "REC-02"
key: {"kind": "relation", "value": {"closing": 1150.00, "issues": [{"row": 3, "kind": "no-effect", "amount": 500.00}], "naive_sum": 1650.00, "without_faulty": 1150.00}}
```

## Derivation

1. replay: balance-after walk (rule 1) = [{"row": 3, "kind": "no-effect", "amount": 500.00}]
2. naive sum: opening + every row = 1650.00
3. without faulty rows: rows [3] dropped = 1150.00
