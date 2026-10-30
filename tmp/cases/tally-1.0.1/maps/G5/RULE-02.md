# RULE-02 · map
Text: `probes/rule/RULE-02.tally`. Key: value {lowest 300.00, on 2025-06-05, if_promise_counted 1100.00}.

step 1 · lowest balance to 30 June over planned items only: 300.00 on 2025-06-05 · lines 31–36, 56, 60–71 · L §5.13 (L:903 `promised`: bal counts it, plan_low and free leave it out), L §5.13.1 (L:911 plan_low is walk-only; L:920 it gives a value, no date), L §5.10 (min, argmin, ties to the first key) · other: the spec's planned-only look-ahead is `plan_low`, which an ask can't read and which gives no date; `bal(konto, d)` in an ask counts the promise. The text gets the key's figure only by subtracting the promised pay's own flows by hand: min over days of bal − flows(friend_back, 15 May..d) = 300.00, argmin 2025-06-05 (5–9 June tie at 300; the first key wins, L:792)
(if_promise_counted, no key step) · line 58 · L:903, L:918 · same: bal counts the promised pay: 1500, 2300, 1100, 7100; min 1100.00, argmin 2025-06-05

ask returns: value · two asks, each a money value and a date; the planned-only one rests on the hand subtraction, not on the spec's planned/promised distinction
checks: parsecheck ok · loadcheck ok
