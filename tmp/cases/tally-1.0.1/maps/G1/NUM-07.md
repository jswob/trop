# Map · NUM-07 · `probes/num/NUM-07.tally` · expressed

step 1 · net 1500.00 + −2300.00 = −800.00 · lines 8, 11, 18–20, 33–34 · L §5.10 aggregates (`sum`), L §3.2 · same
step 2 · tax 0.19 × max(0, net) = 0.00 · lines 10, 22–25, 32 · L §5.10 (rule `loss` excepts `base` when net < 0; only the winner's value is evaluated) · same: net −800 gives 0, never −152; no `round`, as the card states none
step 3 · loss carried −net when negative = 800.00 · lines 13–16 · L §5.10, L §2.3 parse rule 4 (`-net` is unary minus) · same
note · money DUK read with minor unit 0.01 (S:101)
ask returns: value (−800.00, 0, 800) · one line
checks: parsecheck ok · loadcheck ok
