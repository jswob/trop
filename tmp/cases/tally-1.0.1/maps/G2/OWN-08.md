# OWN-08 · map

Text: `probes/own/OWN-08.tally`. Key kind: value.

step 1 · 15 December 2024: min(want, limit − used) (rule 1) = 400.00 · lines 30–32, 46, 55, 59, 61, 69 · L §5.10 (min), §4.6 "Reading at a date" · same: limit and used are read at year_of(2024-12-15) = 2024: min(1 000, 10 000 − 9 600)
step 2 · 15 January 2025: min(want, limit − used) (rule 1) = 1000.00 · lines 30–32, 56, 60, 62, 70 · L §5.10 · same: the 2025 limit, min(1 000, 10 500 − 0)
step 3 · from earnings: contributions first (rule 2) = 500.00 · lines 28, 38–44, 48, 57–58, 63 · L §5.11 · same: arcs sorted by the pocket key in index order (L:851): contributed takes 10 000, earned 500 (L:853)
step 4 · tax: 0.19 × 500.00 = 95.00 · lines 34–36, 50, 68 · L §7.2 · same: round(95, 0.01, half-up) = 95.00
ask returns: value · 400.00, 1000.00; 10000.00, 500.00; 95.00 DUK
checks: parsecheck ok · loadcheck ok
