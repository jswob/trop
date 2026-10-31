# OWN-02 · map

Text: `probes/own/OWN-02.tally`. Key kind: value.

step 1 · net worth: cash + claim (rule 1) = 5800.00 · lines 13–15, 48, 53–54 · L §5.9 (claim), §3.6, §5.16 · same: the lent money is a claim place, in the sum of account and claim balances on 15 May (L:714, L:353): 5 000.00 + 800.00
step 2 · lowest ahead: planned items only, on 2025-06-01 (rule 2) = 2000.00 · lines 29–31, 33–46, 55–59 · L §5.13 (promised), §5.13.1 (plan_low, LOAD-STATE), §5.10 (min, argmin) · same, through a workaround: the spec's look-ahead that leaves out promised pays is plan_low (L:903, L:920), which an ask may not read (L:911, LOAD-STATE). `bal` counts a promised pay (L:903), so ahead[d] takes the promised pay's movements back out of bal(konto, d) by hand; min over the window's days is 2 000.00 and argmin's first tie is 2025-06-01 (L:792)
ask returns: value · 5800.00 DUK; 2000.00 DUK, 2025-06-01
checks: parsecheck ok · loadcheck ok
