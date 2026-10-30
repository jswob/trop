# Map · LAW-03 · probes/law/LAW-03.tally

step 1 · Ania 2022-06 by the old version = 1060.00 · lines 20–27, 29, 49–55 · L §4.3, §4.6 · same (June's law date reads card_old; card_transitional doesn't govern June, so rule base)
step 2 · Ania 2022-07 = min(old 1060.00, new 960.00) · lines 20–27, 29, 41–47, 57–60 · L §5.10, §4.3 · same (transitional excepts base in July; `as if {law 2022-06-30}` reads card_old's rate and relief)
step 3 · Ola 2022-07 = min(old 40.00, new 240.00) · lines 20–27, 29, 38–39 · L §5.10, §4.3 · same
step 4 · June without the change = {Ania 2022-06: 1060.00} · lines 33, 41–47 · L §4.3, §4.6, §5.16 · same (relation: `check` against `as if {wording 2022-06-30 for [adv, rate, relief]}`; card_new governs from 2022-07-01, so June's read rule never sees it and the check holds for every reading of the spec)
ask returns: relation · advances gives 1060.00, 960.00, 40.00; june_unchanged is a check that is true by the read rule, not by a figure
checks: parsecheck ok · loadcheck ok
