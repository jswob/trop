# Map · TIME-02 · `probes/time/TIME-02.tally` · expressed

step 1 · (a) due: 2026-06-20 or the next business day = 2026-06-23 · lines 8, 10, 12–14, 20, 26 · L §4.2 (`day_of`, `next_business`: Saturdays, Sundays and the days in h aren't business days), L §4.3 (`at`: the law date is the due day), L §4.6 (the row governing 2026 is read) · same: Sat 20, Sun 21, holiday 22 → 23
step 2 · (b) due: 2020-06-20 or the next business day = 2020-06-22 · lines 12–14, 20, 25 · L §4.2, L §4.3, L §4.6 (the row `governs ..2025` gives `[]`) · same: Sat 20, Sun 21 → Mon 22, no holiday then
step 3 · (c) third-to-last business day of 2026-06 = 2026-06-26 · lines 16–18, 20, 26 · L §4.2 (`end_of(m)` is 2026-07-01; `add_business(d, -3, h)`: three business days before it: 30, 29, 26 June) · same
ask returns: value (2026-06-23, 2020-06-22, 2026-06-26) · one line
checks: parsecheck ok · loadcheck ok
