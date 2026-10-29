# REC-02 · step map (text: probes/rec/REC-02.tally)

step 1 · replay: the balance-after walk = [row 3, no-effect, 500.00] · lines 26–31, 37, 46, 51–60 · L §5.10 (a cited rule; the shifted read at r − 1 behind an excepting rule) · same: `no_effect[r]` is true only at 3 (1300.00 after 1300.00), and its amount is 500.00
step 2 · naive sum: opening + every row = 1650.00 · line 24 · L §5.10 (`sum`) · same
step 3 · without faulty rows: row 3 dropped = 1150.00 · line 33 · L §5.10 · same
relation · closing 1150.00 = the last balance-after; dropping row 3 moves the sum by exactly 500.00 · lines 19–22, 39, 41 · L §5.10 (`last` with no term returns the key), L §5.16 (`check`) · same: both checks print ok
ask returns: relation · the closing balance, the faulty row, the two sums and two `ok` checks
checks: parsecheck ok · loadcheck ok
