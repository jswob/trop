# RULE-06 · map
Text: `probes/rule/RULE-06.tally`. Key: value {gap 1000.00 for each of 2025-01..03, gap_total 3000.00, rebooked false, floor_from_april 3000.00}.

step 1 · gap 2025-01: 3000.00 − 2000.00 left = 1000.00 · lines 20, 28, 36–53 · L §4.6 (precedence: the later known wins, L:484), L §5.18 (L:1141 a `stated` row anchors the end-of-day balance), L:918 · same: keep_floor[2025-01] at K = 2025-04-15 is b1_v2's 3000 (known 2025-04-10 beats 2025-01-01, both govern January); bal(konto, 2025-01-15) is the anchor 2000; 3000 − 2000 = 1000.00
step 2 · gap 2025-02 = 1000.00 · lines 28, 46 · as step 1 · same
step 3 · gap 2025-03 = 1000.00 · lines 28, 47 · as step 1 · same
(gap_total) · line 30 · L §5.10 sum · same: 3000.00
(rebooked false) · lines 22–26, 34 · L §5.13.3 (L:956 state pays are never re-measured; `arrears` covers flow pays only) · same: the sweep is a state pay booked at cut min(t, K), so January to March keep v1's floor; q_not_rebooked checks that the savings' movements from February to 14 April are the same as known on 9 April (ok)
(floor_from_april) · line 32 · L §4.6 · same: 3000.00

ask returns: value · the gaps, their total, the floor and an `ok` check. The January sweep itself is a hole movement (konto has no opening, L:721), dated on its anchor's day, so it reaches no asked figure (L:1143)
checks: parsecheck ok · loadcheck ok
