# Key · CUR-01+RAND-01

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "CUR-01+RAND-01"
key: {"kind": "value", "value": {"revenue": {"kind": "random", "r": 4250.10, "se_r": 0.47, "p": 2.00}, "cash": {"kind": "random", "r": 4398.53, "se_r": 1.12, "p": 5.00}, "same_seed_same_answer": true}}
```

## Derivation

1. the law's rate day: the last business day before 2026-03-02 (rule 1) = 2026-02-27
2. closed form: the mean revenue: 1000.00 × (4.2000 + 5 steps × 0.01, the mean step) (rules 1, 3) = 4250.00
3. closed form: the mean cash: 1000.00 × (4.2000 + 28 steps × 0.01 − 0.0800) (rules 2, 3) = 4400.00
4. seeded mean revenue: 20000 paths, seed 20260301 (rule 4); each path's amount to the ort (rule 3) = {"kind": "random", "r": 4250.10, "se_r": 0.47, "p": 2.00}
5. seeded mean cash: 20000 paths, seed 20260301 (rule 4); each path's amount to the ort (rule 3) = {"kind": "random", "r": 4398.53, "se_r": 1.12, "p": 5.00}
6. same seed, same answer: run twice = True
