# SCEN-03 · step map (text: probes/scen/SCEN-03/, files SCEN-03.tally and infl8.tally)

step 1 · rent 2029, override: the previous year's index, override years then base = 2595.00 · SCEN-03.tally lines 29–31, 41, 43–47, 49–51; infl8.tally lines 6–8 · L §6.4 (Reading: the higher layer first, so 0.08 for 2025–2027 and the base 0.03 for 2028), L §5.10, L §7.2 · same
step 2 · rent 2029, base: the base path = 2251.01 · SCEN-03.tally lines 29–31, 41, 47 · L §5.10, L §7.2 · same
value · what the override moves = [bonds, pension, rent] · SCEN-03.tally lines 23–27, 33–39 · L §5.16 (`inputs-of`), L §6.4 · same, through a workaround: every reader of `cpi_change` moves with the scenario by the read rule, but no ask form lists what reads an input; the text asks `inputs-of` of each reader in the scenario, and those listing `override_8`'s rows (rent, pension, bonds; not the mortgage, which reads REF) are the moved ones. The card gives no pension, bond or mortgage formula, so those defs are stand-ins over inputs whose missing figures are holes with questions
ask returns: value · 2595.00 and 2251.01, and four row sets read by hand
checks: parsecheck ok · loadcheck ok
