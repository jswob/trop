# Map · TIME-06 · `probes/time/TIME-06.tally` · expressed

step 1 · advance, month 1: tax on the base to date − withheld = 1560.00 · lines 8, 10, 12, 15–23, 30–32 · L §5.8 (`marginal(scale, x)`: Σ over rows with lo < x of rate × (min(x, hi) − lo)), L §5.10 aggregates (`sum` with `where k <= m`, `k < m`), L §5.10 recurrences "at a computed key" (advance reads itself at earlier bound keys; no cycle), L §7.2 · same: 13 000 → 1 560.00, nothing withheld
step 2 · advance, month 2 = 1560.00 · lines 15–23 · same sections · same: 3 120.00 − 1 560.00
step 3 · advance, month 3 = 1560.00 · lines 15–23 · same sections · same
step 4 · advance, month 4 = 1560.00 · lines 15–23 · same sections · same
step 5 · advance, month 5 = 1560.00 · lines 15–23 · same sections · same
step 6 · advance, month 6 = 1560.00 · lines 15–23 · same sections · same
step 7 · advance, month 7 = 1560.00 · lines 15–23 · same sections · same
step 8 · advance, month 8 = 1560.00 · lines 15–23 · same sections · same
step 9 · advance, month 9 = 1560.00 · lines 15–23, 25 · same sections · same: 117 000 → 14 040.00 − 12 480.00
step 10 · advance, month 10 = 3560.00 · lines 15–23, 25, 31–32 · L §5.8 `marginal` crosses the 120 000 edge · same: 130 000 → 14 400 + 3 200 = 17 600.00 − 14 040.00
note · money DUK read with minor unit 0.01 (S:101)
ask returns: value (1560.00, 3560.00) · one line
checks: parsecheck ok · loadcheck ok
