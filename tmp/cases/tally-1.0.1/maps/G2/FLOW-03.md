# FLOW-03 · map

Text: `probes/flow/FLOW-03.tally`. Key kind: relation.

step 1 · Halina: paid − share × costs + share × rent − received (rule 2) = 1600.00 · lines 18–24, 26, 33–39 · L §3.2, §5.10 (sum) · same: 1 200 − 0.5 × 1 200 + 0.5 × 2 000 − 0
step 2 · Ania: same formula = 200.00 · lines 18–26 · L §3.2 · same: 0 − 300 + 500 − 0
step 3 · Kuba: same formula = −1800.00 · lines 18–26 · L §3.2 · same: 0 − 300 + 500 − 2 000
relation · the balances add up to 0 · line 28 · L §5.16 (check) · same: Σ share = 1, so Σ(share × costs) = costs paid and Σ(share × rent) = rent received; q_sum prints ok
ask returns: relation · 1600, 200, −1800 DUK (num); ok
checks: parsecheck ok · loadcheck ok
