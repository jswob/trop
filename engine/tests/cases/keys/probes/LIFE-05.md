# Key · LIFE-05

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "LIFE-05"
key: {"kind": "value", "value": {"vat": 2185.00, "vat_due": "2025-04-25", "social": 400.00, "health": 350.00, "cash_march": 7995.00, "late": ["#8"]}}
```

## Derivation

1. VAT: 0.23 × 9500.00 (rule 1) = 2185.00
2. social: business month 20 (rule 2) = 400.00
3. health: revenue to date 26500.00 (rule 3) = 350.00
4. cash in March: gross of invoices paid in the month = 7995.00
