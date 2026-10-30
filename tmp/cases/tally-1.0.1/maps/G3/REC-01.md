# REC-01 · step map (text: probes/rec/REC-01.tally)

step 1 · flagged: in the records, not in the model = [PKB-7731] · lines 14, 25–30 · O:124, O:144, O:376 (`OP-ACCOUNT`: an account with no place, at import), L §5.18 (importer's inputs) · same, through a workaround: the spec flags an account with no place as an import error (`OP-ACCOUNT`), an operation, not something an ask computes; the text writes the statement's accounts as `account_place` rows and PKB-7731's as a `?` row, so the ask prints the hole for pkb-7731 only. L §5.18 says "the model never reads them; the importer does", and is silent on an ask reading them
step 2 · residual: statement − model = −50.00 · lines 16, 20–23, 32–37 · L §5.18 (a `stated` row is an anchor; the difference stated − booked is the statement's balance line), E §6.3 (the residual ledger) · same: booked is the opening 3200.00 with no movement, stated 3150.00, balance line −50.00
relation · booked: no · line 18 · L §5.18 (an anchor isn't a movement: nothing is booked and `flows` doesn't count it) · same: `flows(pkb_1201, 2025-03)` = 0 and `bal(pkb_1201, 2025-03-31)` = 3150.00, the statement stands
ask returns: relation · the flag (a hole on one account), the ledger's −50.00 line, and nothing booked
checks: parsecheck ok · loadcheck ok
