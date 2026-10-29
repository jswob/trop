# LAW-20 · A circular rule the law solves by its own named method

**Story.** Piotr's business income for the year is 60 000.00 DUK, and he paid 9 000.00 DUK of health premium. The premium is deducted from his income, less a premium credit; the credit is sized from his income after the deduction.

**Rules.**
1. The deduction is the premium paid less the premium credit, never below 0.
2. The premium credit is 20 % of the income less the deduction less 30 000.00 DUK, never below 0 and never more than the premium, rounded half up to 0.01 DUK.
3. Rules 1 and 2 are solved in four steps: (i) an interim deduction, the premium; (ii) an interim credit, by rule 2 from the interim deduction; (iii) the final deduction, by rule 1 from the interim credit; (iv) the final credit, by rule 2 from the final deduction. The final deduction and the final credit are the result.
4. Instead of rule 3, the taxpayer may repeat steps (iii) and (iv), starting from the interim deduction, until a deduction differs from the one before it by less than 1.00 DUK; that deduction, and the credit by rule 2 from it, are then the result.

**Question.** (a) The deduction and the credit by rule 3. (b) The same by rule 4: how many rounds it takes, and where it ends.

**Worked by hand.** (a) (i) 9 000.00; (ii) 20 % × (60 000.00 − 9 000.00 − 30 000.00) = 4 200.00; (iii) 9 000.00 − 4 200.00 = **4 800.00**; (iv) 20 % × (60 000.00 − 4 800.00 − 30 000.00) = **5 040.00**. The rules are linear and no bound is reached, so in closed form the method's credit is r(I − P − T)(1 + r) and its deduction P − r(I − P − T): the same 5 040.00 and 4 800.00. (b) The deductions run 4 800.00, 3 960.00, 3 792.00, 3 758.40, 3 751.68, 3 750.34, 3 750.07: **7 rounds**, the last change 0.27, so the deduction is **3 750.07** and the credit **5 249.99**. Both rules hold at once where the credit is r(I − P − T)/(1 − r), 5 250.00, and the deduction 3 750.00; the iteration ends within 1.00 of that point. Rule 3's four steps don't reach it: the law's own method is the answer, not the point where both rules hold.

```yaml
edge: "LAW-20"
inputs: {"income": 60000.00, "premium": 9000.00, "threshold": 30000.00, "rate": 0.20}
expect: {"a": {"kind": "relation", "value": {"deduction": 4800.00, "credit": 5040.00, "closed_form_agrees": true}}, "b": {"kind": "relation", "value": {"deduction": 3750.07, "credit": 5249.99, "rounds": 7, "within_1_of_fixed_point": true}}}
```
