# ASK-05 · Explain

**Story.** Ania's gross pay for May 2025 is 8 000.00 DUK under her contract.

**Rules.**
1. Pension contribution: 9.76 % of gross, rounded half up to 0.01.
2. Health contribution: 9 % of gross minus the pension contribution, rounded half up to 0.01.
3. Tax advance: 12 % of gross minus the pension contribution, rounded half up to whole DUK.

**Question.** Her net pay, with its causes in order.

**Worked by hand.** Gross 8 000.00 (contract); pension −780.80 (rule 1); health 9 % × 7 219.20 = 649.728 → −649.73 (rule 2); advance 12 % × 7 219.20 = 866.304 → −866.00 (rule 3). Net **5 703.47**.

```yaml
edge: "ASK-05"
inputs: {"gross": 8000.00, "pension": 0.0976, "health": 0.09, "tax": 0.12}
expect: {"kind": "value", "value": {"net": 5703.47, "causes": [["gross", "contract", 8000.00], ["pension", "rule 1", -780.80], ["health", "rule 2", -649.73], ["tax advance", "rule 3", -866.00]]}}
```
