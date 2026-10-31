# RULE-04 · map
Text: `probes/rule/RULE-04.tally`. Key: refusal (a solver for R-X -> R-Y -> R-X).

step 1 · cycle: rules reading each other = [R-X, R-Y, R-X] · lines 14–20 · L §5.10 (L:760 same-key dependencies form an acyclic graph, checked over every rule), L §8.3 (L:1600 LOAD-CYCLE `<x> depends on itself at the same key: <path>`), L §10 (L:1683, L:1684 a solver only where the law names one: `solve` in a rule with `v`, L:1069) · same: invest_x reads reserve_y and reserve_y reads invest_x at the same (empty) key; no rule names a solver; the loader refuses the reality naming invest_x -> reserve_y -> invest_x

ask returns: refusal · the model doesn't run (L §8.3: a load refusal stops it), so the ask prints no value; the refusal names the cycle. Neither tool checks cycles (loadcheck's own list says so), so both pass the text
checks: parsecheck ok · loadcheck ok
