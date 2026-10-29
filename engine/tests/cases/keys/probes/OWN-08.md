# Key · OWN-08

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "OWN-08"
key: {"kind": "value", "value": {"dec_2024": 400.00, "jan_2025": 1000.00, "withdrawal": {"from_contributions": 10000.00, "from_earnings": 500.00, "tax": 95.00}}}
```

## Derivation

1. 15 December 2024: min(want, limit − used) (rule 1) = 400.00
2. 15 January 2025: min(want, limit − used) (rule 1) = 1000.00
3. from earnings: contributions first (rule 2) = 500.00
4. tax: 0.19 × 500.00 = 95.00
