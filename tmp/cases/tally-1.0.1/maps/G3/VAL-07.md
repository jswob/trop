# VAL-07 · step map (text: probes/val/VAL-07.tally)

step 1 · register: the assumptions read, transitively = [A-RENT, A-CPI] · lines 16–18, 22–33 · L §5.3 (`assumed` on a version: every row of it and every rule pointing at it with `v` put the reason in the register), L §8.2 (the register is transitive) · same: rule `next` points at `a_rent` and reads `rent_margin` (A-RENT) and `cpi_change` (A-CPI); the keyed rows have no governs span, so no `held:` entry is added (L §8.2)
step 2 · rent 2026: × (1 + 0.04) = 2080.00 · lines 17–18, 26, 33, 38 · L §5.10, L §7.2 · same
step 3 · rent 2027: × (1 + 0.04) = 2163.20 · lines 18, 27, 33 · L §5.10, L §7.2 · same
ask returns: value · 2163.20 DUK with `assumed: A-RENT…` and `assumed: A-CPI…` under it
checks: parsecheck ok · loadcheck ok
