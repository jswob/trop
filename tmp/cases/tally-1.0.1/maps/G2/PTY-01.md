# PTY-01 · map

Text: `probes/pty/PTY-01.tally`. Key kind: value.

step 1 · first year married throughout (married 2023-07-08) = 2024 · lines 27–31, 33, 38–39 · L §3.5, §4.3 rule 2, §4.6, §5.10 (all, first) · same: married[piotr_ola] has rows governing ..2023-07-07 (false) and 2023-07-08.. (true) (L §3.5 "a law reads a relation as it stood on its law date"); married_day[d] is read at d's start (L §4.3 rule 2), so the input is read at d (L:422, §4.6 candidates); joint_throughout[y] is `all` over the year's days (L:790); `first` returns the first year whose condition is true (L:779–780): 2019–2023 false (2023 has false days before 8 July), 2024 true
step 2 · first year married on its last day = 2023 (contrast) · lines 23–25, 31, 33 · L §4.2 (end_of, add_days), §4.6 "Reading at a date" · same: add_days(end_of(y), -1) is 31 December (L §4.2); married_day read at that day is true first for 2023
ask returns: value · 2024, 2023 (the two `first` results, printed as year keys)
checks: parsecheck ok · loadcheck ok
