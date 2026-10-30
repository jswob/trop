# VAL-01 · Figures from the gazette

**Story.** The reference wage RW is published in the gazette: 3 490.00 DUK in force 1 January – 31 December 2023 (published 15 November 2022); 4 242.00 in force 1 January – 30 June 2024 (published 15 November 2023); 4 300.00 in force 1 July – 31 December 2024 (published 20 May 2024). Nothing later is published.

**Rules.**
1. A figure is read from the gazette entry in force on the date.
2. No entry in force: refused, never 0. A date past the last entry's end reads that entry, held, and says so.

**Question.** RW on (a) 1 June 2022 and (b) 1 March 2026.

**Worked by hand.** (a) No entry yet: **refused**, RW on 2022-06-01 missing. (b) Past the last entry: **4 300.00**, held from the entry in force from 1 July 2024; partial, RW for 2026-03-01 listed as held.

```yaml
edge: "VAL-01"
inputs: {"entries": [{"id": "RW", "value": 3490.00, "published": "2022-11-15", "from": "2023-01-01", "to": "2023-12-31"},
  {"id": "RW", "value": 4242.00, "published": "2023-11-15", "from": "2024-01-01", "to": "2024-06-30"},
  {"id": "RW", "value": 4300.00, "published": "2024-05-20", "from": "2024-07-01", "to": "2024-12-31"}], "a": "2022-06-01", "b": "2026-03-01"}
expect: {"a": {"kind": "refusal", "missing": ["RW on 2022-06-01"]}, "b": {"kind": "partial", "value": 4300.00, "missing": ["RW on 2026-03-01 (held at the entry from 2024-07-01)"]}}
```
