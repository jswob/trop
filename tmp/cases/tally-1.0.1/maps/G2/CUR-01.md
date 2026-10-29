# CUR-01 · map

Text: `probes/cur/CUR-01.tally`. Key kind: value.

step 1 · law's rate date: last business day before 2024-03-15 (rule 1) = 2024-03-14 · lines 28–30, 36, 42, 44 · L §4.2 (prev_business, add_days, weekday) · same: add_days(2024-03-15, −1) is Thursday 14 March, a business day, so prev_business returns it (L:399). The card names no holiday; the calendar is the row `holidays = []`, marked assumed (L:469), which the register lists
step 2 · revenue: 1000.00 × 4.3125 = 4312.50 · lines 32–34, 43, 51 · L §3.2 (exchange rates), §7.2 · same: money FLR × num DUK/FLR is num DUK (L:304); mid[rate_day] reads the key 2024-03-14 (L:489); half up to 0.01
step 3 · cash: 1000.00 × 4.2510 = 4251.00 · lines 22–26, 41, 45 · L §3.2, §7.2 · same; fx_loss = 61.50, positive: revenue above cash, a loss
ask returns: value · 2024-03-14, 4312.50, 4251.00, 61.50 DUK
checks: parsecheck ok · loadcheck ok
