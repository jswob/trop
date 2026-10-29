# REC-05 · Records as the law's inputs

**Story.** Ola's May 2025 payslip is missing. Her bank shows the net pay: 5 148.00 DUK.

**Rules.**
1. Pension contribution 10 % of gross, rounded half up to 0.01; tax advance 12 % of gross minus the contribution, rounded half up to whole DUK; net = gross − contribution − advance.
2. Where only net is known, gross is inferred by the law and marked inferred.

**Question.** May's gross.

**Worked by hand.** Net ≈ 0.792 × gross, so gross ≈ 6 500. Check 6 500.00: contribution 650.00; 12 % × 5 850 = 702; net 5 148.00 ✓. A gross of 6 499.99 or 6 500.01 gives 5 147.99 or 5 148.01, and the tax steps (701, 703) can't reach 5 148.00: the only gross is **6 500.00**, inferred: partial, the payslip named missing.

```yaml
edge: "REC-05"
inputs: {"net": 5148.00, "month": "2025-05", "pension": 0.10, "tax": 0.12}
expect: {"gross": {"kind": "partial", "value": 6500.00, "missing": ["payslip for 2025-05 (gross inferred from net)"]}, "inferred": true}
```
