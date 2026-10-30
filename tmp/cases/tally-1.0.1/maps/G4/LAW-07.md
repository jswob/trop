# Map · LAW-07 · probes/law/LAW-07.tally

step 1 · revenue after #41 = 196500.00 · lines 20–22, 35, 49, 52, 55 · L §5.10 · same
step 2 · revenue after #42 = 201500.00 · lines 20–22, 35, 50, 53 · L §5.10 · same
step 3 · revenue after #43 = 203500.00 · lines 20–22, 35, 51, 54 · L §5.10 · same
step 4 · VAT on invoices from #42 = 1610.00 · lines 24–33, 37, 43–44 · L §5.10, §7.2 · same (first(cum > threshold) = i42; lost excepts exempt for i >= i42, keys compared in index order)
ask returns: value · first_taxed gives i42 and 1610.00
checks: parsecheck ok · loadcheck ok
