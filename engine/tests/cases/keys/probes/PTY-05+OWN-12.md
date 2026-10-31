# Key · PTY-05+OWN-12

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "PTY-05+OWN-12"
key: {"kind": "value", "value": {"members": {"a": {"ania": 2000.00, "tomek": 1500.00}, "d": {"ania": 4000.00, "halina": 10000.00}, "b": {"ania": 700.00, "piotr": 3000.00}}, "inner": 6000.00, "outer_member": 700.00, "total": 6700.00}}
```

## Derivation

1. member a: parts of sav at the end of 2026-06-30: the member alone (its own rules 1-2) = {"ania": 2000.00, "tomek": 1500.00}
2. member d: parts of deposit at the end of 2026-06-30: the member alone (its own rules 1-2) = {"ania": 4000.00, "halina": 10000.00}
3. member b: parts of pot at the end of 2026-06-30: the member alone (its own rules 1-2) = {"ania": 700.00, "piotr": 3000.00}
4. the inner joint model fam: ania's parts across its members: a: ania + d: ania (stated same in the inner header, L §6.1) = 6000.00
5. the outer joint model: fam.ania's parts at member b: fam.ania = b.ania (stated same in the outer header) = 700.00
6. the party's parts across the whole family: inner + outer member (rule 3) = 6700.00
