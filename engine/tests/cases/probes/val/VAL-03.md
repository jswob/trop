# VAL-03 · Two dates on every figure

**Story.** The 2022 income limit LIM is published as a draft on 20 May 2022, 85 000.00 DUK, and as final on 15 July 2022, 88 000.00; both in force for all of 2022.

**Rules.**
1. A figure is read as known on a date: only entries published by then count; at equal in-force dates the later publication wins.

**Question.** LIM for 31 December 2022, as known on 31 May 2022 and on 1 August 2022.

**Worked by hand.** Known 31 May: only the draft: **85 000.00** (draft). Known 1 August: the final: **88 000.00**.

```yaml
edge: "VAL-03"
inputs: {"entries": [{"id": "LIM", "value": 85000.00, "published": "2022-05-20", "from": "2022-01-01", "to": "2022-12-31", "source": "draft"},
  {"id": "LIM", "value": 88000.00, "published": "2022-07-15", "from": "2022-01-01", "to": "2022-12-31", "source": "final"}], "on": "2022-12-31", "known": ["2022-05-31", "2022-08-01"]}
expect: {"kind": "value", "value": {"2022-05-31": {"value": 85000.00, "source": "draft"}, "2022-08-01": {"value": 88000.00, "source": "final"}}}
```
