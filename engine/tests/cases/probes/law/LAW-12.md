# LAW-12 · An exception through a chain

**Story.** Two heirs each inherit 50 000.00 DUK; neither reports it within 6 months. Kuba is the deceased's grandson, aged 15. Leon is his nephew, aged 15.

**Rules.**
1. An inheritance by a spouse, child or grandchild is exempt —
2. — unless it isn't reported within 6 months —
3. — but a minor needn't report it.
4. A taxed inheritance pays 12 % above 5 000.00.

**Question.** Each heir's tax.

**Worked by hand.** Kuba: rule 1 applies; rule 2 would remove it; rule 3 removes rule 2: exempt, **0.00**. Leon: rule 1 doesn't apply, so rule 3 doesn't let him through: 12 % × 45 000 = **5 400.00**.

```yaml
edge: "LAW-12"
inputs: {"amount": 50000.00, "free": 5000.00, "rate": 0.12, "heirs": {"Kuba": {"close": true, "unreported": true, "minor": true}, "Leon": {"close": false, "unreported": true, "minor": true}}}
expect: {"kind": "value", "value": {"Kuba": 0.00, "Leon": 5400.00}}
```
