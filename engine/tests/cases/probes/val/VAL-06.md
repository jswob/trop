# VAL-06 · Indexed values and histories

**Story.** Halina's pension account records her earnings: 40 000.00 DUK in 2021, 44 000.00 in 2022, 48 000.00 in 2023, and none in 2024. The yearly indices are 1.050 for 2022, 1.080 for 2023, 1.040 for 2024.

**Rules.**
1. At each year's end the account is multiplied by that year's index, then that year's earnings are added; amounts are rounded half up to 0.01.

**Question.** The account at the end of 2024.

**Worked by hand.** End 2021: 40 000. 2022: × 1.05 = 42 000 + 44 000 = 86 000. 2023: × 1.08 = 92 880 + 48 000 = 140 880. 2024: × 1.04 = **146 515.20**.

```yaml
edge: "VAL-06"
inputs: {"earnings": {"2021": 40000.00, "2022": 44000.00, "2023": 48000.00, "2024": 0.00}, "index": {"2022": 1.050, "2023": 1.080, "2024": 1.040}, "to": 2024}
expect: {"kind": "value", "value": 146515.20}
```
