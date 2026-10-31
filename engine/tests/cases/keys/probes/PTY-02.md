# Key · PTY-02

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "PTY-02"
key: {"kind": "value", "value": {"joint": 14400.00, "alone": {"Tomek": 16800.00, "Ania": 3600.00}, "parts": {"Tomek": 11858.82, "Ania": 2541.18}}}
```

## Derivation

1. joint tax: 2 × scale(60000.00) (rule 2) = 14400.00
2. Tomek alone: scale(90000.00) = 16800.00
3. Ania alone: scale(30000.00) = 3600.00
4. parts: joint × alone / Σ alone, largest remainders (rule 3) = {"Tomek": 11858.82, "Ania": 2541.18}
