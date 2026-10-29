# Map · NUM-05 · `probes/num/NUM-05.tally` · expressed

step 1 · parts: floor to 0.01, leftover units in listing order = [33.34, 33.33, 33.33] · lines 8–9, 12–16, 37 · L §5.12 (`rest largest`: each share rounded down to 0.01, the leftover units one each to the largest remainders, ties in index order), L §5.6 (a list index is ordered as written) · same: the weights are all 1, so every remainder is equal and the one leftover unit goes by index order, Ania first, for any total; "rounded down" equals "floored" on positive shares (L §7.2 `down`/`floor`)
step 2 · printed parts: half up to 1 = [33, 33, 33] · lines 20–22 · L §7.2 · same
relation · parts add back to the whole: `rest largest` makes the shares sum to the total (L §5.12, its own sentence), so `part_sum` (line 18) is 100.00; printed sum 99 (line 24) and printed total 100 (lines 26–28) are read only by asks, so display rounding feeds nothing
note · money DUK read with minor unit 0.01 (S:101)
ask returns: value (parts per heir, sum 100.00, printed 33/33/33, 99, 100) · two asks
checks: parsecheck ok · loadcheck ok
