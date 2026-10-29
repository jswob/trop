# Key · NUM-01

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "NUM-01"
key: {"a": {"kind": "refusal", "missing": ["rate FLR->DUK"], "reason": "amounts in two currencies with no stated rate"}, "b": {"kind": "value", "value": 1066.19, "unit": "DUK a month"}}
```

## Derivation

1. (a) total: rule 1: rate FLR->DUK not stated = refused
2. (b) monthly rate: 0.12 / 12 (rule 2) = 0.01
3. (b) instalment: 12000.00 × r / (1 − (1 + r)^−12), half up to 0.01 = 1066.19
