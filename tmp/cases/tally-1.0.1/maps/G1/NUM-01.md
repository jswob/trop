# Map · NUM-01 · `probes/num/NUM-01.tally` · expressed

step 1 · (a) total: rule 1, no FLR->DUK rate stated, refused · lines 11–13, 23–25, 27 · L §3.2 (exchange rates: converting is explicit, `num DUK/FLR` applied from FLR to DUK), L §4.6 rule 4 (gap rule: no row, a hole with the input's `question`), L §8.1 · same: the sum reads `flr_rate`, which has no row, so `total_duk` is a hole that prints "At what rate do FLR convert to DUK?"; a direct `cur_acct + flr_acct` would be `LOAD-UNIT` and refuse the whole model, (b) included
step 2 · (b) monthly rate 0.12 / 12 = 0.01 · lines 10, 19–21, 37 · L §3.3 (`periodic` of `nominal 12 /year` over `month`: (1 + r/12)^1 − 1 = r/12) · same
step 3 · (b) instalment 12000.00 × r / (1 − (1 + r)^−12), half up to 0.01 = 1066.19 · lines 8–9, 15–17, 35–36 · L §7.1 (`pow` with an integer exponent is exact), L §7.2 (`round` half-up, 0.01) · same
note · `money DUK` and `money FLR` are read with minor unit 0.01 (S:101)
ask returns: (a) hole naming the missing rate (the key's refusal, its missing fact named) · (b) value 1066.19 DUK
checks: parsecheck ok · loadcheck ok
