# VAL-05 · step map (text: probes/val/VAL-05.tally)

step 1 · (a) value: 120 × the price of 2024-12-27 = 6144.00 · lines 28–34, 38, 42–46, 56, 59 · L §5.10 (`missing()` in a cited rule; `last` evaluates from the last binding and stops at the first true `where`) · same: days 31, 30, 29, 28 have no row, so rule `none` gives `-`; 2024-12-27 has 51.20; `price_date` = 2024-12-27 and `value_a` = 120 × 51.20 = 6144 DUK (exact, no rounding stated, L §7.2); the register lists `missing record counted by card_rules` for the days without a price
step 2 · (b) sale: 50 × the price of 2027-03 = 3170.00 · lines 36, 40, 48–51, 57–58 · L §4.6 (reading at a date: a day in a month index's key position reads the month that holds it) · same: `f_fc[2027-03-15]` reads key 2027-03 = 63.40
ask returns: value · (a) the price's date and 6144 DUK, (b) 3170 DUK
checks: parsecheck ok · loadcheck ok
