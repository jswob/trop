# Key · TIME-05

Written by `tools/keys_probes.py`; don't edit.

```yaml
edge: "TIME-05"
key: {"kind": "value", "value": {"a": {"window_end": "2023-02-28", "short_term": true}, "b": {"clock_end": "2024-12-31", "exempt": false, "first_exempt_day": "2025-01-01"}, "c": {"deadline": "2024-10-31", "in_time": true, "day_for_day_deadline": "2024-10-30"}}}
```

## Derivation

1. (a) window end: same-numbered day, clipped (rule 1) = 2023-02-28
2. (b) clock end: end of the year of acquisition + 5 years (rule 2) = 2024-12-31
3. (c) deadline: from 2024-04-30, a month's last day: the last day of the final month (rule 3) = 2024-10-31
4. (c) day-for-day: for contrast = 2024-10-30
