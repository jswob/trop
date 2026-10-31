# Map · ASK-01 · `probes/ask/ASK-01.tally` · expressed

step 1 · balances after each day's rows = 3500, 7500, 4500, 1900, 1400, 7400 · lines 10–11, 13, 15, 18–20, 29, 31–42 · L §5.18 (records of a covered place are booked on their day; unexplained ones against the outside place), L §5.9 (opening at S), L §5.13.1 (`bal(x, d)`: the balance at the end of day d), L §5.16 (an ask may read it on any day of the run) · same
first below · `first(d in mday where …)` → 2025-03-20 · lines 16, 22, 28 · L §5.10 aggregates (`first` with no term returns the key, in binding order) · same
minimum and its date · `min(…)` → 1400.00, `argmin(…)` → 2025-03-25 · line 22 · L §5.10 (`argmin`: ties go to the first key in binding order; 1400.00 holds 25–27 March) · same
note · cut K 2025-04-01 is after every day read, so day K (C1-F6) isn't reached; money DUK read with minor unit 0.01 (S:101)
ask returns: value (2025-03-20, 1400.00, 2025-03-25) · one line
checks: parsecheck ok · loadcheck ok
