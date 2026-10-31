# VAL-02 · Which date a value is read at

**Story.** A bond series starts in March 2025. The price index: December 2024 104.1, January 2025 104.9, February 2025 105.3, March 2025 105.0.

**Rules.**
1. The series reads the index two months before the month it starts.

**Question.** Which month is read, and its value.

**Worked by hand.** March − 2 months = **January 2025**: **104.9**.

```yaml
edge: "VAL-02"
inputs: {"starts": "2025-03", "lag_months": 2, "index": {"2024-12": 104.1, "2025-01": 104.9, "2025-02": 105.3, "2025-03": 105.0}}
expect: {"kind": "value", "value": {"read_month": "2025-01", "value": 104.9}}
```
