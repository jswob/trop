# Key · NUM-07

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "NUM-07"
key: {"kind": "value", "value": {"net": -800.00, "tax": 0.00, "loss_carried": 800.00}}
```

## Derivation

1. net: 1500.00 + -2300.00 (rule 1) = -800.00
2. tax: 0.19 × max(0, net) (rule 2) = 0.00
3. loss carried: −net when negative (rule 2) = 800.00
