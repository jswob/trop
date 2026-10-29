# Key · VAL-01

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "VAL-01"
key: {"a": {"kind": "refusal", "missing": ["RW on 2022-06-01"], "reason": "no gazette entry in force on the date"}, "b": {"kind": "partial", "value": 4300.00, "missing": ["RW on 2026-03-01 (held at the entry from 2024-07-01)"]}}
```

## Derivation

1. (a) RW on 2022-06-01: no entry in force (rule 2) = refused
2. (b) RW on 2026-03-01: entry from 2024-07-01, past its end 2024-12-31: held (rules 1–2) = 4300.00
