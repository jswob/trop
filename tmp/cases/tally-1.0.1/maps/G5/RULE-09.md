# RULE-09 · map
Texts: `probes/rule/RULE-09/RULE-09.tally` (part a), `probes/rule/RULE-09/inpath.tally` (part b, a scenario). Key: a value {share 0.4, decision stop buying, decided_on 2025-01-01}; b refusal (a statistic across paths, read inside one path).

step 1 · share of ruined paths: 2 of 5 on 2025-01-01 = 0.4 · RULE-09.tally lines 8–16, 20–24 · L §5.16 (L:1058 `share` over path prints n of N and the fraction), L §4.2 (L:404 a key of an integer index is an int, so path p reads runs_dry[p]), L §4.5 (the cut is `on`) · same, through a workaround: the forecast's outcome per path is a row keyed by the path's number, since the card states which paths run dry and no draw can be written to give that; 2 of 5 = 0.4
step 2 · decision: more than 0.2 → stop buying · RULE-09.tally line 16 (second term), 21 · L §5.16 (an ask's expressions, a comparison as a bool) · same: 0.4 > 0.2 is true, the "stop buying" branch; the decision lives in an ask, since a def can't hold `share` over path
(b) refused · inpath.tally line 6, 8 · L §8.3 (L:1617 LOAD-ASKONLY `share … over path outside an ask`), L §8 (L:1545 a load refusal in a scenario file refuses only that scenario) · same: the scenario is refused at load; the reality and q_a run

ask returns: a value (0.4 and true, with its standard error and verdict, L:1058); b refusal (LOAD-ASKONLY) · the ask's `seed 1` is the text's own: the card gives no seed, and no draw reads it
checks: parsecheck ok · loadcheck ok (the directory as one reality; loadcheck doesn't check LOAD-ASKONLY)
