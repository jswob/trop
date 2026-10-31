# SURF-03 · map
Text: `probes/surf/SURF-03.tally`. No key (a passage).

req 1 · the draw written apart from a fixed value · lines 16, 18, 28–34 · L §5.14 (`draw` vs `input`) · same: the life table is an `input` with rows; the yearly death is `draw die[p in path, k in person, a in age_year] ~ binomial(1, q_die[k, a])`
req 2 · the path it lives on · lines 18–24 · L §5.14 (a draw's first index is `path`), L §5.10 · same: death_age and years_lived carry `p in path`, one value per path
req 3 · where the seed and the path count live · line 26 · L §5.16 (`paths n seed s` on the ask), L §7.4 · same: in the ask, not in the model or a row
req 4 · what is reported: the mean with its standard error · line 26 · L §5.16 (L:1057–1060) · same: `mean … over path` prints the mean and its standard error
req 5 · a statistic across paths written outside any one path · line 26 · L §5.16 (path statistics are ask-only), L §8.3 (L:1617 LOAD-ASKONLY) · same: the mean lives only in the ask; in a def it is refused (shown in RULE-09 b)
req 6 · a household model's shape · lines 8–14 · L §5.2, §5.3 · same: sources, an index of persons, a version citing the table

ask returns: random · lifetime_mean with its standard error (60000 paths, seed 20251024), the quantity RAND-01 asks
checks: parsecheck ok · loadcheck ok
