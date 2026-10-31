# Map · ASK-03 · `probes/ask/ASK-03.tally` · expressed

step 1 · (a): days_abroad blank on F-12 = refused · lines 13, 17–18, 26–28, 46, 54–55 · L §5.7 (a row's value may be a hole `? "…"`), L §8.1 `HOLE-STATED` (it propagates through the comparison) · same: the blank field is a stated hole, never 0; `relief_applies[2024]` prints its question
step 2 · (b): selling more than is held = refused · lines 10, 12, 15, 20–24, 30–32, 34–44, 48, 56–61 · L §5.9 (a holding below zero is refused, `RUN-PLACE`, unless short), L §8.4 (a refusal propagates like a hole and outranks it, L §5.10) · same: 100 units bought, 120 sold → the sale instance is refused, so `flows(sell, …)` and `sale_gain` are refused, the unknown price's hole outranked
refusal (a) · missing days abroad → row `? "…"` on line 55 → hole with that question (the key's refusal, its missing fact named)
refusal (b) · units held 100 < 120 sold → `RUN-PLACE` "sell takes fund_g below 0 units"
note · the sale needs a day the card doesn't give (O:ASK-03-1); money DUK read with minor unit 0.01 (S:101)
ask returns: (a) hole naming the blank field · (b) refused: RUN-PLACE
checks: parsecheck ok · loadcheck ok
