# RAND-01 · map
Text: `probes/rand/RAND-01.tally`. Key: random {r 1.637500, se_r 0.004463, p 0.02, same_seed_same_answer true}.

step 1 · seeded mean: 60000 paths, seed 20251024 = 1.637500 · lines 10–20, 24–30 · L §5.14 (draw addresses: SHA-256 of seed, draw id and key; binomial(1, q) by an exact comparison of the uniform with q), L §5.16 (L:1057 `mean … over path` ask-only, printed with its standard error), L §7.4 (L:1509 the only randomness is `draw`; seed and paths in the ask) · same: distribution binomial(1, q_die[a]) per path and year of age, parameters 0.2, 0.3, 0.5, 1 from rows; death_age is the first year with die = 1 (q_die[88] = 1 makes it total), lifetime = its position − 1 (0..3); mean over 60000 paths, seed 20251024. The figure is the engine's own draw, not the key's generator: judged against r within the key's tolerance (p 0.02); the exact mean is 1.64 (RAND-02)
step 2 · standard error: sd / √n = 0.004463 · line 20 · L §5.16 (L:1058–1060: a path statistic prints its standard error) · same: printed beside the mean, computed from the run's own paths
step 3 · same seed, same answer: run twice = true · lines 20, 22 · L §7.5 (determinism: same text, same seed, same result), L §5.14 (a draw's value is a function of its address only) · same: q_mean and q_mean_again carry the same seed and paths, so their values are equal

ask returns: random · a mean with its standard error, twice equal; the engine's generator is not the key's, so the value is judged by tolerance
checks: parsecheck ok · loadcheck ok
