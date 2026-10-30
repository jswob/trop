# NUM-01 · Amounts, quantities and rates carry their units

**Story.** Ola keeps 1 250.00 DUK in her current account and 300.00 FLR in a foreign-currency account. She also takes a credit of 12 000.00 DUK over 12 months at a nominal 12 % a year.

**Rules.**
1. An amount carries its currency; amounts in two currencies are added only at a stated rate.
2. The credit's rate is nominal yearly, compounded monthly: the monthly rate is a twelfth of it. The instalment is level (annuity), rounded half up to 0.01 DUK.

**Question.** (a) What does Ola hold in total, in DUK? (b) What is the credit's monthly instalment?

**Worked by hand.**
(a) No rate FLR→DUK is stated: refused, the missing rate named.
(b) r = 12 % / 12 = 1 %. 1.01^12 = 1.1268250; 1 − 1/1.1268250 = 0.1125508; 120.00 / 0.1125508 = 1 066.185 → **1 066.19 DUK a month**.

```yaml
edge: "NUM-01"
inputs: {"a": {"items": [{"amount": 1250.00, "cur": "DUK"}, {"amount": 300.00, "cur": "FLR"}]},
  "b": {"principal": 12000.00, "yearly_nominal": 0.12, "months": 12}}
expect: {"a": {"kind": "refusal", "missing": ["rate FLR->DUK"]}, "b": {"kind": "value", "value": 1066.19}}
```
