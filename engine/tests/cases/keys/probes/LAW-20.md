# Key · LAW-20

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "LAW-20"
key: {"a": {"kind": "relation", "value": {"deduction": 4800.00, "credit": 5040.00, "closed_form_agrees": true}}, "b": {"kind": "relation", "value": {"deduction": 3750.07, "credit": 5249.99, "rounds": 7, "within_1_of_fixed_point": true}}}
```

## Derivation

1. (a) (i) interim deduction: the premium (rule 3) = 9000.00
2. (a) (ii) interim credit: 0.20 × (60000.00 − 9000.00 − 30000.00) (rule 2) = 4200.00
3. (a) (iii) final deduction: 9000.00 − 4200.00 (rule 1) = 4800.00
4. (a) (iv) final credit: 0.20 × (60000.00 − 4800.00 − 30000.00) (rule 2) = 5040.00
5. (a) closed form of the method (rule linear, no bound reached): credit = r(I − P − T)(1 + r), deduction = P − r(I − P − T), r 0.20, I 60000.00, P 9000.00, T 30000.00 = {"deduction": 4800.00, "credit": 5040.00}
6. (a) the method's result equals its closed form = True
7. (b) round 1: the credit from the deduction, then the deduction from the credit (rule 4) = {"deduction": 4800.00, "change": -4200.00}
8. (b) round 2: the credit from the deduction, then the deduction from the credit (rule 4) = {"deduction": 3960.00, "change": -840.00}
9. (b) round 3: the credit from the deduction, then the deduction from the credit (rule 4) = {"deduction": 3792.00, "change": -168.00}
10. (b) round 4: the credit from the deduction, then the deduction from the credit (rule 4) = {"deduction": 3758.40, "change": -33.60}
11. (b) round 5: the credit from the deduction, then the deduction from the credit (rule 4) = {"deduction": 3751.68, "change": -6.72}
12. (b) round 6: the credit from the deduction, then the deduction from the credit (rule 4) = {"deduction": 3750.34, "change": -1.34}
13. (b) round 7: the credit from the deduction, then the deduction from the credit (rule 4) = {"deduction": 3750.07, "change": -0.27}
14. (b) closed form: the fixed point both rules hold at: credit = r(I − P − T)/(1 − r), deduction = P − credit = {"deduction": 3750.00, "credit": 5250.00}
15. (b) the iteration within 1.00 of the fixed point = True
