# Key · SCEN-02

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "SCEN-02"
key: {"kind": "value", "value": {"dropped": ["sale", "insurance refund"], "kept": ["parking 2024-10"], "added": ["credit 2024-09", "credit 2024-10", "credit 2024-11", "credit 2024-12", "parking 2024-11", "parking 2024-12"], "cash_difference": -25300.82}}
```

## Derivation

1. dropped: need the sale (rule 1) = ["sale", "insurance refund"]
2. added: run on if kept (rule 2) = ["credit 2024-09", "credit 2024-10", "credit 2024-11", "credit 2024-12", "parking 2024-11", "parking 2024-12"]
3. cash difference: added − dropped = -25300.82
