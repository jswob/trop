# Map · LAW-13 · probes/law/LAW-13/ (LAW-13.tally, rate25.tally)

step 1 · base total = Σ yearly tax = 19000.00 · LAW-13.tally lines 14–20, 32, 37 · L §5.10 · same
step 2 · what-if total, 0.25 from 2030 = 22600.00 · LAW-13.tally lines 22, 24; rate25.tally lines 1–8 · L §6.4 · same (scenario from 2030-01-01 layers rate25_law over the base; whatif_2030 = 2500.00)
ask returns: relation · before_unchanged (LAW-13.tally line 26) checks tax[y] = tax[y] as if {scenario rate25} for 2026–2029; the scenario's layer starts 2030-01-01 (L §6.4), so the years before it read the base row and the check holds by the spec, not by a figure
checks: parsecheck ok · loadcheck ok (directory)
