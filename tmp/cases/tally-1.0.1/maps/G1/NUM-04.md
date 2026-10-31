# Map · NUM-04 · `probes/num/NUM-04.tally` · expressed

step 1 · reading T: 1200.00 × 16/24 = 800.00 · lines 8–9, 12–13, 16–19, 21, 24, 31–32 · L §5.5 (axis; a rule tests it with `when`), L §5.10 (one winner per option), L §3.2 · same: months = count(inst) = 24, months − paid = 16
step 2 · reading B: 1200.00 × Σ(1..16) / Σ(1..24) = 544.00 · lines 10, 21, 23 · L §5.10 aggregates (`sum` over `where k > paid`), L §4.2 (an integer key is an int) · same: balance parts in month k are 24 − k + 1, months 9–24 give 16..1
by-case · the axis `refund_reading` keeps the readings apart; the ask fans out one line per option, each naming its option (L §5.5 Fan-out, §8.2 `reading:`); no `round`, as the card states none (L §7.2)
ask returns: by-case (by_time 800, by_balance 544) · one line per axis option
checks: parsecheck ok · loadcheck ok
