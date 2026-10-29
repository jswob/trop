# RAND-03 · map
Text: `probes/rand/RAND-03.tally`. Key: random {r 0.074075, se_r 0.001309, p 0.01}.

step 1 · exact share: P(K ≥ 9), K ~ Bin(12, 0.5) = 299/4096 · lines 10–16, 24, 28, 33–35 · L §5.14 (distribution table: `binomial(n, q)` with `cdf`, exact for discrete rows; L:1493 only the continuous rows need `to`), L §7.1 (exact rationals) · same: count(m in month) = 12, threshold / fee = 8; revenue > 80000 means at least 9 payments; 1 − cdf(Bin(12, 1/2), 8) = 299/4096 ≈ 0.0730
step 2 · seeded share: 40000 paths, seed 7 = 0.074075 · lines 18–26 · L §5.14 (binomial(1, chance) per path and month; draw address from seed, draw id, key), L §5.16 (L:1058 `share … over path` prints n of N, the fraction and its standard error), L §7.4 (seed and paths in the ask) · same: distribution binomial(1, 0.5) independent per month (distinct addresses), seed 7, 40000 paths; the regime rule is applied on each path (higher[p]). The engine's generator is not the key's: the value is judged against r within p 0.01

ask returns: random (q_share, with its standard error) and value (q_exact 299/4096) · the exact value sits inside the key's tolerance, so both asks agree with the key
checks: parsecheck ok · loadcheck ok
