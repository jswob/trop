# Key · RULE-08

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "RULE-08"
key: {"kind": "value", "value": {"2025-04-15": 3216.67, "2025-06-15": 3266.67}}
```

## Derivation

1. floor 2025-04-15: mean of ['2025-01', '2025-02', '2025-03'] (rule 1) = 3216.67
2. floor 2025-06-15: mean of ['2025-03', '2025-04', '2025-05'] (rule 1) = 3266.67
