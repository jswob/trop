# OWN-09 · map

Text: `probes/own/OWN-09.tally`. Key kind: value.

step 1 · year 1 interest a bond: 100.00 × 0.0680 (rule 2) = 6.80 · lines 33–35, 47–49, 56–58, 74, 83 · L §3.3 (periodic), §7.2 · same: rate_of[1] = first_rate; periodic of a simple annual rate over a year is r (L:323); round(6.8, 0.01, half-up)
step 2 · year 2 rate: index 2024-01 + margin (rule 1) = 0.0495 · lines 47–50, 63, 73, 80, 84–85 · L §3.3, §4.2 · same: month_of(2024-03-01) − 2 = 2024-01 (L:406); rate + rate of one basis, "a reference rate plus a margin" (L:318); the index's change is typed as the reference rate
step 3 · year 2 interest a bond: 106.80 × 0.0495 (rule 2) = 5.29 · lines 33–35, 56–59 · L §5.10 (recurrence), §7.2 · same: value[2] = value[1] + earned[1] = 106.80, guarded by rule first (L:758); 5.2866 half up to 5.29
step 4 · year 3 rate: index 2025-01 + margin (rule 1) = 0.0615 · lines 47–50, 81 · L §3.3 · same
step 5 · year 3 part interest a bond: 112.09 × 0.0615 × 184/365 (rule 2) = 3.48 · lines 33–37, 61–63, 75 · L §4.2 (days, add_years), §5.10 (excepts), §7.2 · same: redeemed is before year_end[3] = 2026-03-01, so rule part wins; days(2025-03-01, 2025-09-01) = 184, days(2025-03-01, 2026-03-01) = 365; 3.4750… half up to 3.48
step 6 · fee a bond: capped at the interest (rule 3) = 2.00 · lines 39–43, 82 · L §5.10 (min) · same: min(2.00, 15.57)
step 7 · tax: 0.19 × 50 × (15.57 − 2.00) (rule 4) = 129 · lines 52–54, 72, 86 · L §7.2 · same: 128.915 rounded half up to step 1, money (L:1471)
step 8 · payout: 50 × 115.57 − fees − tax = 5549.50 · lines 45 · L §3.2 · same: 5 778.50 − 100.00 − 129.00
ask returns: value · 15.57, 2.00, 129.00, 5549.50 DUK (the key's 129 is the same value)
checks: parsecheck ok · loadcheck ok
