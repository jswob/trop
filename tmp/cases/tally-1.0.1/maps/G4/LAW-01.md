# Map · LAW-01 · probes/law/LAW-01.tally

step 1 · benefit at 49999.99 = 200.00 · lines 12–15, 21, 28 · L §5.8 · same (row `(-inf, 50000]` holds it)
step 2 · benefit at 50000.00 = 200.00 · lines 12–15, 21, 29 · L §5.8 · same (closed `]` takes the edge)
step 3 · benefit at 50000.01 = 100.00 · lines 12–15, 22, 30 · L §5.8 · same (open `(50000`)
step 4 · benefit at 79999.99 = 100.00 · lines 12–15, 22, 31 · L §5.8 · same
step 5 · benefit at 80000.00 = 0.00 · lines 12–15, 23, 32 · L §5.8 · same (open `80000)` then closed `[80000`)
step 6 · benefit at 80000.01 = 0.00 · lines 12–15, 23, 33 · L §5.8 · same
ask returns: value · six band reads by the row whose span holds x; the rows' brackets carry "not more than", "more than", "less than", "not less than"
checks: parsecheck ok · loadcheck ok
