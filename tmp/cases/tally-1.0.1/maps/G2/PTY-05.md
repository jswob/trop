# PTY-05 · map

Text: `probes/pty/PTY-05.tally`. Key kind: relation.

step 1 · union transfers: matched on date, from, to, amount = 1 · lines 24–28, 34, 36–47 · L §5.18 (L:1125, L:1127, L:1131–1137), §6.1 · other: Tally matches nothing. The text states the one transfer as one pay (D's derived statement) and A's outside fact as a record whose `explains` names it, a link written by hand (L:1127: only confirmed links are written). Booking then takes A's record as the movement and makes D's pay the modelled past, never booked (L:1131–1136): the outside fact stands and the derived statement is compared, the reverse of rule 1. No form counts movements: q_transfer prints the amount booked once, 5 000.00 DUK, not the count 1
step 2 · each household's own key: from its own statements = Ania 5000.00, Halina −5000.00 · lines 30–32 · L §6.1, §5.16 · silent: q_people prints +5 000.00 and −5 000.00 and q_family 0.00, read from the union. A reality is one set of files (L §6.1) and an ask reads one reality (L §5.16); no ask runs A's or D's reality alone beside the union, so "joining changes no answer" (rule 2) and keys_hold have no form
ask returns: value · flows only (5000.00, −5000.00, 0.00, 5000.00); the relation's union_transfers = 1 and keys_hold = true are not results of the text
checks: parsecheck ok · loadcheck ok
