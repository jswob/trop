# REC-06 · Clocks and knowledge dates

**Story.** Ania earns 96 000.00 DUK in 2022. The gazette lists the 2022 tax rate as 17 % (published 20 November 2021); an act published 15 July 2022 sets 12 % for the whole of 2022.

**Rules.**
1. A question is asked as of a date and answered as known on a date: only what was published by then counts.

**Question.** Ania's 2022 tax, as of 31 December 2022, as known on 31 May 2022 and on 1 August 2022.

**Worked by hand.** Known 31 May: 17 % × 96 000 = **16 320.00**. Known 1 August: 12 % = **11 520.00**.

```yaml
edge: "REC-06"
as_of: "2022-12-31"
inputs: {"income": 96000.00, "on": "2022-12-31", "known": ["2022-05-31", "2022-08-01"],
  "entries": [{"id": "RATE", "value": 0.17, "published": "2021-11-20", "from": "2022-01-01", "to": "2022-12-31"}, {"id": "RATE", "value": 0.12, "published": "2022-07-15", "from": "2022-01-01", "to": "2022-12-31"}]}
expect: {"kind": "value", "value": {"2022-05-31": 16320.00, "2022-08-01": 11520.00}}
```
