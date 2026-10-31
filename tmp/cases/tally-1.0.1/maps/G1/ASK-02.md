# Map · ASK-02 · `probes/ask/ASK-02.tally` · awkward

step 1 · Piotr: flat 400000.00 × 1/2 = 200000.00 · lines 13, 18–19, 25–30, 41, 46 · L §5.9 (`worth(x, d)` of an asset reads its valuation rows), L §4.6 (the 10 January row is open-ended, so it is the last valuation on 30 June), L §3.6 (`owns`) · same
step 2 · Ola: flat 400000.00 × 1/2 = 200000.00 · lines 25–30, 45 · same sections · same, printed by `q_parts` (line 34)
step 3 · Piotr: account 20000.00 × 1 = 20000.00 · lines 14, 27, 42, 47 · L §5.9 (opening), L §5.13.1 (`bal(x, d)`) · same
net worth · Piotr 220000.00 · lines 21–23, 32 · L §5.10 aggregates (`sum` with `where owns > 0`: the car's term is left out, never 0 × hole) · same
partial · missing car value → `car_value` has no row (lines 12, 17) → `worth(car, …)` is a hole with its question (L §4.6 gap rule) → Ola's `net_worth` is a hole: a `sum` with a hole term is a hole (L §5.10 table, L §8.1), so her 200000.00 is not on her net-worth line. It prints only on the per-place ask `q_parts`. A result is a value or a hole (L §8), never "a value plus a hole", so the key's {Ola: 200000.00, missing: car} takes two asks
note · money DUK read with minor unit 0.01 (S:101)
ask returns: partial, split over two asks · q_net_worth: Piotr 220000.00, Ola hole (car value); q_parts: Piotr flat 200000.00, Piotr account 20000.00, Ola flat 200000.00, Ola car hole
checks: parsecheck ok · loadcheck ok
