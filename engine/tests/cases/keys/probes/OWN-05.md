# Key · OWN-05

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "OWN-05"
key: {"kind": "value", "value": {"shares": {"Halina": "5/8", "Ania": "3/32", "Piotr": "3/32", "Marta": "3/32", "Kuba": "3/32"}, "payouts": {"Ania": 45000.00, "Piotr": 45000.00, "Marta": 45000.00, "Kuba": 45000.00}, "keeper_pays": 180000.00, "payout_at_death_value": 37500.00}}
```

## Derivation

1. shares after the death: own half + half × estate share (rules 1–2) = {"Halina": "5/8", "Ania": "3/32", "Piotr": "3/32", "Marta": "3/32", "Kuba": "3/32"}
2. payout Ania: 3/32 × 480000.00 (rule 3) = 45000.00
3. payout Piotr: 3/32 × 480000.00 (rule 3) = 45000.00
4. payout Marta: 3/32 × 480000.00 (rule 3) = 45000.00
5. payout Kuba: 3/32 × 480000.00 (rule 3) = 45000.00
