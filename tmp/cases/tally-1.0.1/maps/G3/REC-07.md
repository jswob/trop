# REC-07 · step map (text: probes/rec/REC-07.tally)

step 1 · start 0: units 265/9 × 110 = 3238.89 · lines 19–22, 26, 33–34, 39–42 · L §5.10 (`sum` over keys, exact), L §7.1 (exact rationals), L §7.2 (`round` 0.01 half-up at the value) · same
step 2 · start 1: units 2825/99 × 130 = 3709.60 · lines 19–22, 40–43 · same
step 3 · start 2: units 35900/1287 × 100 = 2789.43 · lines 19–22, 41–44 · same
step 4 · start 3: units 3830/143 × 150 = 4017.48 · lines 19–22, 42–45 · same; the ask's `where` keeps starts with a full plan in the record (s + 3 ≤ 6)
value · worst start 2, 2789.43; best start 3, 4017.48 · lines 24, 28 · L §5.10 (`argmin`, `argmax`, `min`, `max`; ties to the first key), L §10 (an ask prints and decides nothing) · same
ask returns: value · the table of ends, the worst and the best
checks: parsecheck ok · loadcheck ok
