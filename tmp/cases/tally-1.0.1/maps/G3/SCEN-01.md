# SCEN-01 · step map (text: probes/scen/SCEN-01/, files SCEN-01.tally and amend.tally)

step 1 · instalment 2025-02-15: − 1000.00 = 9000.00 · amend.tally lines 16–22, 30–34, 44–47 · L §5.10 (recurrence guarded by `first`), L §5.13 (flow pay), L §6.4 (scenario items) · same
step 2 · instalment 2025-03-15 = 8000.00 · same lines · same
step 3 · instalment 2025-04-15 = 7000.00 · same lines · same
step 4 · overpayment 2025-04-15: − 2000.00 = 5000.00 · amend.tally lines 21, 24–28, 48–50 · L §5.13, L §5.10 · same: `inst_amt` = min(1000.00, what is owed), so the term shortens
step 5 · instalment 2025-05-15 = 4000.00 · amend.tally lines 16–22, 30–34 · same
step 6 · instalment 2025-06-15 = 3000.00 · same lines · same
relation · cash difference +3000.00, debt 3000.00, net worth difference 0.00 · SCEN-01.tally lines 14, 16–20, 25 · L §6.4 (Asking: `x as if {scenario s} - x`), L §5.13.3 (`flows` of a place), L §5.9 (a debt is a balance ≤ 0, negated) · same, through a workaround: the loan's place is declared in the base with opening 0, because a base ask can't name an item declared in a scenario file (`LOAD-SCENARIO`, L §6.4) and an ask in the scenario has no form to read the base for the difference
ask returns: relation · 3000.00, 3000.00, 0.00
checks: parsecheck ok · loadcheck ok
