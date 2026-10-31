# RAND-02 · map
Text: `probes/rand/RAND-02.tally`. Key: relation {expected 1.64, exact 41/25}.

step 1 · expected remaining lifetime: Σ chance alive at each year's end = 41/25 · lines 8–19, 21–27 · L §5.10 (recurrence over an integer index's keys, `excepts`/guard order; L:760 no cycle: alive_end[a] reads a − 1, a different key), L §7.1 (L:1446 exact rationals, no floating point), L §4.2 (`pos`, key arithmetic `a - 1`) · same: alive_end = 0.8, 0.56, 0.28, 0 (rule `first` for 85, `base` after); sum = 1.64 = 41/25 exactly. No draw: the expectation is computed, not sampled

ask returns: value · 1.64 (num, exact 41/25); the relation (expected = Σ survival) holds because the spec's arithmetic is exact
checks: parsecheck ok · loadcheck ok
