# REC-04 · step map (text: probes/rec/REC-04.tally)

step 1 · same counterparty: registry names = true · lines 47–48, 72–74, 76–81, 86–88, 100 · L §4.3 (`at`: the def's law date is the row's day), L §4.6 (the registry row governs from 2025-02-01) · same: at 2025-04-03 both Kowal Sp. and Kowal-Nowak SA register as kowal_nowak, and the row is 3 days after the due date, within 10
step 2 · open: rule 2 = 800.00 · lines 45, 50–52, 58–63, 65 · L §5.11 (fill: taken = min(remaining supply, remaining demand), "up to the amount still open") · same: taken[r1, inv_12] = 1200.00, so `match_of[r1]` = inv_12, paid 1200.00, open 800.00
value · the remainder's certainty: promised, 0.75 · lines 54–56, 93, 98–99 · L §5.7 (keyed inputs; the lexicon as rows) · same; the remainder's date is asked as a hole (line 67)
ask returns: value · inv_12, 1200.00, 800.00, promised, 0.75
checks: parsecheck ok · loadcheck ok
