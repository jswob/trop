# Map · ASK-04 · `probes/ask/ASK-04.tally` · awkward

step 1 · cost A: fee + interest = 1700.0000 · lines 8, 10–12, 14–16, 28, 30, 32 · L §3.3 (`periodic` of a `simple /year` rate over `year` is r × 1), L §3.2 · same: 500.00 + 20000.00 × 0.06
step 2 · cost B: fee + interest = 1600.0000 · lines 14–16, 29, 31 · same sections · same
step 3 · breakeven rate for B = 0.085 · line 22 · L §5.16 `solve` (B's rate pinned on the grid 0..0.2 by 0.0001; F(0) < 0 < F(0.2); the root 0.085 is a grid point) · same
refusal · no winner: line 18 declares the choice a `bound`, read by `q_asked` (line 20), which prints "bound: no_choice: …" beside both costs · L §5.17, L §10 ("Choose the best") · workaround: L §8 has no refusal for a question asking for a choice (L §10 even lets an ask take the max over `as if` options, "since it prints and decides nothing"), and a bound is meant for a step no row can give (L §5.17 "A hole or a bound?")
note · the card gives no basis for 6 % and 8 % (O:ASK-04-1); money DUK read with minor unit 0.01 (S:101)
ask returns: refusal with values, through the bound · q_asked: (bound, 1700, 1600); q_breakeven: 0.085 simple /year
checks: parsecheck ok · loadcheck ok
