# Map · TIME-01 · `probes/time/TIME-01.tally` · expressed

step 1 · gain 1600.00 − 1000.00 = 600.00 · lines 13–14, 21, 36–37 · L §3.2 · same
step 2 · purchase days giving an exempt sale = 1–13 · lines 10, 12, 15, 17–19, 33, 35, 38 · L §4.4 (a month as a `date` is imprecise; `add_years` gives the interval of its values at the period's ends, [2024-03-01, 2024-03-31]; a comparison is decided only when the whole interval lies on one side), L §4.2 (`add_years`: same day number, clamped) · same: days 1–13 put the clock's end before 2024-03-14, so `sold > …` would be true; the text keeps that open rather than listing the days
step 3 · purchase days giving a taxed sale = 14–31 · lines 17–19, 23–26 · L §4.4, L §5.10 step 5 (rule `exempt_sale`'s `when` is a hole and would except the winner, so the result is a hole carrying its question), L §8.1 `HOLE-IMPRECISE` · same: never the 1st or the 15th by default
partial · missing purchase day → `bought[l1] = 2019-03` (imprecise) → `tax[l1]` is a hole: "date within 2019-03 not precise enough for sold[l1] > add_years(bought[l1], hold_years)"; `gain[l1]` stays 600.00
note · money DUK read with minor unit 0.01 (S:101)
ask returns: partial · (600.00, hole naming the purchase date within 2019-03)
checks: parsecheck ok · loadcheck ok
