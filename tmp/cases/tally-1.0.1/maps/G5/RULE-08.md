# RULE-08 · map
Text: `probes/rule/RULE-08.tally`. Key: value {2025-04-15: 3216.67, 2025-06-15: 3266.67}.

step 1 · floor 2025-04-15: mean of 2025-01..03 = 3216.67 · lines 10–18, 23–25, 28 · L §5.10 (aggregates over keys anywhere; `mean`), L §4.2 (L:405 the difference of two keys is an int), L §7.2 (`round` half-up) · same: k < 2025-04 and 2025-04 − k ≤ 3 binds January to March; 9650 / 3 = 3216.666… → 3216.67
step 2 · floor 2025-06-15: mean of 2025-03..05 = 3266.67 · lines 16, 25–27 · as step 1 · same: 9800 / 3 → 3266.67; June's row is never read (the `where` is false first, L:797)

ask returns: value · two money values 3216.67 DUK and 3266.67 DUK
checks: parsecheck ok · loadcheck ok
