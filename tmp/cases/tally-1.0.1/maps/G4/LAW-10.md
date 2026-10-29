# Map · LAW-10 · probes/law/LAW-10.tally

step 1 · window closes 2027-12-31 · lines 31–35, 48, 56 · L §4.2 · same (end of year 2027 is exclusive 2028-01-01, less a day)
step 2 · branch no more spent: 0.19 × (gain − gain × 0.6) = 7600.00 · lines 19, 23–29, 39 · L §5.16, §5.10 · same (branch kept apart as its own ask pinning spent_more = 0)
step 3 · branch 200 000 more spent: 0.19 × (gain − gain × 1) = 0.00 · lines 19, 23–29, 41 · L §5.16, §5.10 · same (own ask pinning spent_more = 200000; min caps exempt at the gain)
ask returns: by-case · settles gives 2027-12-31; tax_pending is a hole on spent_more (no row, the window hasn't closed on 2026-10-25); the two branches are separate asks with pins
checks: parsecheck ok · loadcheck ok
