# Key · OWN-09

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "OWN-09"
key: {"kind": "value", "value": {"interest_per_bond": 15.57, "fee_per_bond": 2.00, "tax": 129, "payout": 5549.50}}
```

## Derivation

1. year 1 interest a bond: 100.00 × 0.0680 (rule 2) = 6.80
2. year 2 rate: index 2024-01 + margin (rule 1) = 0.0495
3. year 2 interest a bond: 106.80 × 0.0495 (rule 2) = 5.29
4. year 3 rate: index 2025-01 + margin (rule 1) = 0.0615
5. year 3 part interest a bond: 112.09 × 0.0615 × 184/365 (rule 2) = 3.48
6. fee a bond: capped at the interest (rule 3) = 2.00
7. tax: 0.19 × 50 × (15.57 − 2.00) (rule 4) = 129
8. payout: 50 × 115.57 − fees − tax = 5549.50
