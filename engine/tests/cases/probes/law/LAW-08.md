# LAW-08 · Pipelines across laws and forms

**Story.** Tomek's gross pay is 16 500.00 DUK every month of 2024.

**Rules.**
1. The pension contribution is 9.76 % of gross, rounded half up to 0.01, on gross only until the year's gross to date reaches 180 000.00.
2. The tax advance is 12 % of gross minus the contribution, rounded half up to whole DUK.
3. Net pay is gross minus the contribution minus the advance.

**Question.** November's and December's net pay.

**Worked by hand.** By October 165 000; November's base is the 15 000 left under the cap: contribution 1 464.00; tax 12 % × 15 036 = 1 804.32 → 1 804; net **13 232.00**. December: no contribution; tax 1 980; net **14 520.00**.

```yaml
edge: "LAW-08"
inputs: {"gross": 16500.00, "cap": 180000.00, "pension": 0.0976, "tax": 0.12}
expect: {"kind": "value", "value": {"november": 13232.00, "december": 14520.00}}
```
