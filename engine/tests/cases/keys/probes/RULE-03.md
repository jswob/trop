# Key · RULE-03

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "RULE-03"
key: {"a": {"kind": "value", "value": {"R-A": 3000.00, "R-B": 2000.00, "left": 0.00}}, "b": {"kind": "refusal", "missing": ["priority between R-A, R-B"], "reason": "two rules want the same money and no priority is stated"}}
```

## Derivation

1. (a): in turn ['R-A', 'R-B'] (rule 1) = {"R-A": 3000.00, "R-B": 2000.00}
2. (b): no priority (rule 2) = refused
