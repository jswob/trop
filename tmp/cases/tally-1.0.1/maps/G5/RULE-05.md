# RULE-05 · map
Text: `probes/rule/RULE-05.tally`. Key: value {trigger 2025-05-31, first_instalment 2025-06-10, first_seen 2025-06-15}.

step 1 · savings 2025-02-28: + 2500.00 = 32500.00 · lines 29–33, 48–49 · L §5.9 (L:721 opening at S), L §4.2 (L:391 `day_of(m, 31)` clamps to the month's last day), L:918 `bal(x, d)` end of day · same
step 2 · savings 2025-03-31 = 35000.00 · lines 29–33 · as step 1 · same
step 3 · savings 2025-04-30 = 37500.00 · lines 29–33 · as step 1 · same
step 4 · savings 2025-05-31 = 40000.00 · lines 29–33, 42 (first term) · as step 1, L §5.10 (`first`, L:780) · same: the first day with bal(savings, d) ≥ 40000 is 2025-05-31
(first_instalment) · lines 35–40, 42 (second term), 47 · L §5.13.1 (a `when` with a state read makes a state pay; L:918 bal(x, d) for d before the pay's day), L:901 (`when` false: no instance), L:923 (`flows` of a state pay) · same, through a workaround: the June instance's `when` reads a day before 1 June at ≥ 40000 (the once-latch is `any` over earlier days), earlier instances don't exist; its date is read through its movement, so the card's unstated instalment takes an assumed amount (next_on refuses a state pay, L:921)
(first_seen) · lines 25–27, 42 (third term) · L:927 (a span ending on the rule's day counts what is booked up to it), L:938–939 (a day's earlier movements before its state pays) · same: the first month whose flows of instal up to the 15th are above 0 is June: 2025-06-15

ask returns: value · three dates; the second and third rest on instal_amt, an assumed row (L §4.5, L §8.2 register `assumed:`)
checks: parsecheck ok · loadcheck ok
