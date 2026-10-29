# Key · RAND-03

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "RAND-03"
key: {"kind": "random", "r": 0.074075, "se_r": 0.001309, "p": 0.01}
```

## Derivation

1. exact share: P(K ≥ 9), K ~ Bin(12, 0.5) = 299/4096
2. seeded share: 40000 paths, seed 7 (rules 1–2) = 0.074075
