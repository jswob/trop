# Key · OWN-12

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "OWN-12"
key: {"p10": {"kind": "value", "value": {"own_worth": {"ania": -18227.500, "ania_tomek": 5045.00, "tomek": -20750.000}, "net_worth": {"ania": -15705.00, "ania_tomek": 5045.00, "tomek": -18227.50}}}, "p17": {"kind": "value", "value": {"members": {"a": {"ania": 2000.00, "ania_tomek": 1500.00, "tomek": 0}, "d": {"ania": 4000.00, "halina": 10000.00}}, "same": {"ania": {"a": 2000.00, "d": 4000.00, "total": 6000.00}}}}, "p23": {"kind": "value", "value": {"own_worth": {"ania": -17927.500, "ania_tomek": 4445.00, "tomek": -20450.000}, "net_worth": {"ania": -15705.00, "ania_tomek": 4445.00, "tomek": -18227.50}}, "warning": "WARN-LABELS: labels dropped at current from 2026-03-02: r6"}}
```

## Derivation

1. (P-10) the places counted by owns, at the end of 2026-06-30: opening plus the rows = {"current": 6500.00, "mortgage": -50000.00}
2. (P-10) own worth of ania: its parts in sav + its units × 52.25 + its owns shares of ['current', 'mortgage'] (rule 1) = -18227.500
3. (P-10) own worth of ania_tomek: its parts in sav + its units × 52.25 + its owns shares of ['current', 'mortgage'] (rule 1) = 5045.00
4. (P-10) own worth of tomek: its parts in sav + its units × 52.25 + its owns shares of ['current', 'mortgage'] (rule 1) = -20750.000
5. (P-10) net worth of ania: own worth + Σ stake × the entity's own worth, half-even to 0.01 (rule 2) = -15705.00
6. (P-10) net worth of ania_tomek: own worth + Σ stake × the entity's own worth, half-even to 0.01 (rule 2) = 5045.00
7. (P-10) net worth of tomek: own worth + Σ stake × the entity's own worth, half-even to 0.01 (rule 2) = -18227.50
8. (P-17) member a: parts of sav at the end of 2026-06-30: the member alone (its own rules) = {"ania": 2000.00, "ania_tomek": 1500.00, "tomek": 0}
9. (P-17) member d: parts of deposit at the end of 2026-06-30: the member alone (its own rules) = {"ania": 4000.00, "halina": 10000.00}
10. (P-17) ania across the members: of p reads the parts labelled with either side of the same line, in every member (L §6.1) = {"a": 2000.00, "d": 4000.00, "total": 6000.00}
11. (P-23) the places counted by owns, at the end of 2026-06-30: opening plus the rows = {"current": 7100.00, "mortgage": -50000.00}
12. (P-23) own worth of ania: its parts in sav + its units × 52.25 + its owns shares of ['current', 'mortgage'] (rule 1) = -17927.500
13. (P-23) own worth of ania_tomek: its parts in sav + its units × 52.25 + its owns shares of ['current', 'mortgage'] (rule 1) = 4445.00
14. (P-23) own worth of tomek: its parts in sav + its units × 52.25 + its owns shares of ['current', 'mortgage'] (rule 1) = -20450.000
15. (P-23) net worth of ania: own worth + Σ stake × the entity's own worth, half-even to 0.01 (rule 2) = -15705.00
16. (P-23) net worth of ania_tomek: own worth + Σ stake × the entity's own worth, half-even to 0.01 (rule 2) = 4445.00
17. (P-23) net worth of tomek: own worth + Σ stake × the entity's own worth, half-even to 0.01 (rule 2) = -18227.50
18. (P-23) the warning: r6 took labelled parts to current, a place of the model that keeps none and isn't outside (rule 3) = WARN-LABELS: labels dropped at current from 2026-03-02: r6
