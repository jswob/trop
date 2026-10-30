# OWN-03 · map

Text: `probes/own/OWN-03.tally`. Key kind: value.

step 1 · net worth 31 December: cash − tax owed (rule 1) = 8160.00 · lines 12–26, 36, 43–46 · L §5.9 (debt), §5.13, §3.6 · same: the year's tax accrues on its last day as a pay out of the debt place (balance ≤ 0, L:713), so bal(tax_debt, 2024-12-31) = −1 840.00 at the end of the day (L:918); the sum of account and debt balances is 8 160.00
step 2 · owed at run end: due 2025-04-30 after run end 2025-03-31 (rule 2) = 1840.00 · lines 29–33, 38, 45 · L §5.9, §5.13 · same: nothing pays the debt before the end, so −bal(tax_debt, 2025-03-31) = 1 840.00 and tax_due[2024] prints 2025-04-30. The payment instance dated after E is outside the run; no asked figure reads it (O:OWN-03-1)
ask returns: value · 8160.00 DUK; 1840.00 DUK, 2025-04-30
checks: parsecheck ok · loadcheck ok
