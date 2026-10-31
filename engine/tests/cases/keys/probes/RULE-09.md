# Key · RULE-09

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "RULE-09"
key: {"a": {"kind": "value", "value": {"share": 0.4, "decision": "stop buying", "decided_on": "2025-01-01"}}, "b": {"kind": "refusal", "missing": ["a statistic across paths, read inside one path"], "reason": "rule 2"}}
```

## Derivation

1. share of ruined paths: 2 of 5 on 2025-01-01 = 0.4
2. decision: more than 0.2: True (rule 1) = stop buying
