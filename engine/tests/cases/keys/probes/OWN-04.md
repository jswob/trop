# Key · OWN-04

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "OWN-04"
key: {"kind": "value", "value": {"community": {"Ania": 150000.00, "Tomek": 150000.00}, "separate": {"Ania": 300000.00}, "co-owned": {"Piotr": 180000.00, "Ola": 120000.00}, "estate": {"Halina": 75000.00, "Ania": 56250.00, "Piotr": 56250.00, "Marta": 56250.00, "Kuba": 56250.00}}}
```

## Derivation

1. community: Ania: 300000.00 × 1/2 = 150000.00
2. community: Tomek: 300000.00 × 1/2 = 150000.00
3. separate: Ania: 300000.00 × 1 = 300000.00
4. co-owned: Piotr: 300000.00 × 3/5 = 180000.00
5. co-owned: Ola: 300000.00 × 2/5 = 120000.00
6. estate: Halina: 300000.00 × 1/4 = 75000.00
7. estate: Ania: 300000.00 × 3/16 = 56250.00
8. estate: Piotr: 300000.00 × 3/16 = 56250.00
9. estate: Marta: 300000.00 × 3/16 = 56250.00
10. estate: Kuba: 300000.00 × 3/16 = 56250.00
