# Map · NUM-02 · `probes/num/NUM-02.tally` · expressed

step 1 · unused days 2024-03-31 → 2024-08-31 = 153 · lines 8–9, 25–27, 34–35 · L §4.2 (`days(a, b)` is b − a in days: from the day after cover ends to the last day, inclusive) · same
step 2 · days in the policy year, both ends counted = 366 · lines 9–10, 13–15, 35–36 · L §4.2 (`days` + 1) · same
step 3 · refund actual/actual 732.00 × 153 / 366, half up to 0.01 = 306.00 · lines 11, 21–23, 37 · L §3.2 (money × int, / int), L §7.2 · same
step 4 · refund actual/365 732.00 × 153 / 365, half up to 0.01 = 306.84 · lines 17–19 · L §7.1 (365 is an allowed literal), L §7.2 · same: the convention is the card's own, written as the divisor, not `dcf` (whose `act365` formula L §3.3 names but doesn't define)
note · money DUK read with minor unit 0.01 (S:101)
ask returns: value (306.00, 306.84, 153) · one line, three expressions
checks: parsecheck ok · loadcheck ok
