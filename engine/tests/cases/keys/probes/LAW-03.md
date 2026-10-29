# Key · LAW-03

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "LAW-03"
key: {"kind": "relation", "value": {"Ania 2022-06": 1060.00, "Ania 2022-07": 960.00, "Ola 2022-07": 40.00, "june_unchanged": true}}
```

## Derivation

1. Ania 2022-06: old version (rule 1) = 1060.00
2. Ania 2022-07: lower of old 1060.00 and new 960.00 (rule 3) = 960.00
3. Ola 2022-07: lower of old 40.00 and new 240.00 (rule 3) = 40.00
4. June without the change: the law as before 1 July = {"Ania 2022-06": 1060.00}
