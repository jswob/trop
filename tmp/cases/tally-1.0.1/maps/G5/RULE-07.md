# RULE-07 · map
Text: `probes/rule/RULE-07.tally`. Key: value {by_records 3500.00, by_rule 4000.00, gap {2025-03: 500.00}}.

step 1 · by records: 1000.00 + 5 × 500.00 = 3500.00 · lines 12, 20, 24–30, 39, 42–52 · L §5.18 (L:1130–1133: on a covered day the records are booked and the pays are modelled past; L:1135 an explained record books as the instance's movement), L §5.9 (opening) · same: savings covered January to June, five records of 500.00, March has none; bal(savings, 2025-06-30) = 3500.00
step 2 · by rule: 1000.00 + 6 × 500.00 = 4000.00 · lines 24–28, 32, 45–46 · L §4.5 (the cut), L §4.6 (a version known after K isn't a candidate), L:1139 (from K on, and on uncovered days, the pays are booked) · same, through a workaround: "by the rule" is read as the plan as known on 30 June, before the export's known date, which the card doesn't give and the text assumes (2025-07-01); no ask form prints the pays' own balance over days a statement covers
step 3 · gap: months the rule names and the records lack = {2025-03: 500.00} · line 34 · ENGINE §6.3 (E:267–289: an expected-but-missing line, signed − model) · same: one line in group 2025-03 / pay standing, −500.00 (500.00 expected, missing)

ask returns: value · two money values and the residual ledger; by_rule rests on the assumed known date of `bank_2025_h1` (register `assumed:`)
checks: parsecheck ok · loadcheck ok
