# Key · LIFE-02

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "LIFE-02"
key: {"kind": "value", "value": {"insurance": {"ends": "2024-09-15", "refund": 350.82}, "credit": {"ends": "2024-09-15", "repaid": 8000.00}, "parking": {"ends": "2024-10-31", "still_paid": 150.00}, "car_tax": {"ends": "2024-09-15", "refund": 0.00}}}
```

## Derivation

1. insurance refund: 1200.00 × 107/366 (rule 1) = 350.82
2. parking ends: end of the month after notice (rule 3) = 2024-10-31
3. parking still paid: months after the sale's month until the end = 150.00
