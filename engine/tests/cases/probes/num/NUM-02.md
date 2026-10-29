# NUM-02 · Day counts and pro-rating by days

**Story.** Tomek's car policy runs 1 September 2023 to 31 August 2024, premium 732.00 DUK paid up front. He sells the car; cover ends at the end of 31 March 2024.

**Rules.**
1. The refund is the premium × unused days / days in the policy year (actual/actual: the policy year has 366 days, 29 February 2024 inside it).
2. Unused days run from the day after cover ends to the policy's last day, inclusive.
3. The refund is rounded half up to 0.01.

**Question.** What is the refund under the contract's convention, and what would actual/365 give?

**Worked by hand.** Unused: 1 April – 31 August 2024 = 30 + 31 + 30 + 31 + 31 = 153 days. Actual/actual: 732.00 × 153 / 366 = **306.00**. Actual/365: 732.00 × 153 / 365 = 306.838 → **306.84**.

```yaml
edge: "NUM-02"
inputs: {"premium": 732.00, "start": "2023-09-01", "end": "2024-08-31", "cover_ends": "2024-03-31"}
expect: {"kind": "value", "value": {"actual_actual": 306.00, "actual_365": 306.84, "unused_days": 153}}
```
