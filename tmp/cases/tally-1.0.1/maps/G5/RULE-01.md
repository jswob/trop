# RULE-01 · map
Text: `probes/rule/RULE-01.tally`. Key: value {wrapped 1500.00, broker 4900.00, stays 3000.00}.

step 1 · wrapped: floor kept, wrapped first up to its room = 1500.00 · lines 24–28, 37–42, 49–52 · L §5.13.1 (L:917 `bal(x)` at the walk point), L §5.13.2 (L:939 phase 5 before 6), L §5.9 (L:721 opening at S), L §3.2 (min, max) · same: to_wrapped runs first at phase 5, reads bal(konto) = 9400, surplus = max(0, 9400 − 3000) = 6400, room = max(0, 10000 − 8500) = 1500, amount min(6400, 1500) = 1500.00
step 2 · broker: the remainder after the wrapped share = 4900.00 · lines 24–26, 30–35 · L §5.13.1 (L:913: a def that reaches a state read is evaluated afresh at each reader's point), L:917, L:939 · same: to_broker at phase 6 sees bal(konto) = 9400 − 1500 = 7900, surplus = 4900.00
(stays, no key step) · line 44 `bal(konto, 2025-06-15)` · L:918 (end of day d), L §5.16 (L:1044) · same: 9400 − 1500 − 4900 = 3000.00

ask returns: value · three money values 1500.00 DUK, 4900.00 DUK, 3000.00 DUK; `flows(place, day)` is movements in minus out (L:964), and the places with no opening are read through `flows` only, which needs none (L:721)
checks: parsecheck ok · loadcheck ok
