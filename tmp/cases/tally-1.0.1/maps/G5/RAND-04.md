# RAND-04 · map
Texts: `probes/rand/RAND-04/RAND-04.tally` (plan A, the reality), `probes/rand/RAND-04/plan_b.tally` (plan B, a scenario). Key: relation {median_difference 1000.00, share_b_above 1, paths 1000, common_random_numbers true}.

step 1 · median B − A, common paths = 1000.00 · RAND-04.tally lines 20, 24, 28, 32, 36–42; plan_b.tally lines 1–8 · L §5.14 (L:982 draw addresses don't name the scenario: a scenario reruns the same paths, common random numbers), L §6.4 (L:1258 `x as if {scenario s}`), L §7.3 (normal draws and `exp` take `to u`) · same: on each path both plans read the same r[p, y], so growth[p] is the same rounded value and end_worth B − A = cash_kept 1000 − 0 = 1000.00 exactly on every path; the median of 1000 equal values is 1000.00. Distribution normal(0.05, 0.15) per year, seed 11, 1000 paths
step 2 · share B above = 1 · line 32 (second term) · L §5.16 (L:1058 `share` over path) · same: 1000 of 1000 paths, B − A = 1000 > 0, share 1 (standard error 0)
step 3 · median B − A, independent draws (contrast) = 956.08 · lines 22, 26, 30, 34 · L §5.14 (a different draw id gives different addresses), L §7.4 · same in kind, other in figure: plan B reads r_ind, a second draw of the same distribution, so the difference no longer cancels; the median differs from 1000.00 but its figure is the engine generator's own, not the key's 956.08 (the key's contrast rests on its own seeds)

ask returns: value · median 1000.00 and share 1 exactly (a relation the spec's common random numbers guarantee), the contrast a random median ≠ 1000. The scenario's `from` is the model's `on` (2026-01-01), so MC2-9 (scenario rows govern from `from`) changes nothing here
checks: parsecheck ok · loadcheck ok (the directory as one reality)
