# Key · NUM-02

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "NUM-02"
key: {"kind": "value", "value": {"actual_actual": 306.00, "actual_365": 306.84, "unused_days": 153}}
```

## Derivation

1. unused days: 2024-03-31 → 2024-08-31 (rule 2) = 153
2. days in the policy year: 2023-09-01 → 2024-08-31 inclusive = 366
3. refund, actual/actual: 732.00 × 153 / 366 (rules 1, 3) = 306.00
4. refund, actual/365: 732.00 × 153 / 365 = 306.84
