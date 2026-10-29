# Key · PTY-05

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "PTY-05"
key: {"kind": "relation", "value": {"union_transfers": 1, "Ania": 5000.00, "Halina": -5000.00, "family_total": 0.00, "keys_hold": true}}
```

## Derivation

1. union transfers: matched on date, from, to, amount = 1
2. each household's own key: from its own statements = {"Ania": 5000.00, "Halina": -5000.00}
