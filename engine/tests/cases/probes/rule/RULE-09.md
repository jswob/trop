# RULE-09 · A rule on a forecast

**Story.** On 1 January 2025 Halina's forecast runs 5 seeded paths; on paths 2 and 4 the money runs out before she is 85.

**Rules.**
1. House rule D-3: "on 1 January each year, if more than 20 % of the paths run dry before 85, stop buying bonds this year". It reads that date's paths.
2. A statistic across paths can't be read inside one path.

**Question.** (a) The decision on 1 January 2025. (b) The same rule written inside a path's own run.

**Worked by hand.** (a) 2 of 5 = **0.4**, more than 0.2: **stop buying**. (b) **Refused**: a share of paths read inside one path.

```yaml
edge: "RULE-09"
inputs: {"decided_on": "2025-01-01", "ruined": [false, true, false, true, false], "limit": 0.2}
expect: {"a": {"kind": "value", "value": {"share": 0.4, "decision": "stop buying", "decided_on": "2025-01-01"}}, "b": {"kind": "refusal", "missing": ["a statistic across paths, read inside one path"]}}
```
