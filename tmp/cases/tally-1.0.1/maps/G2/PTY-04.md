# PTY-04 · map

Text: `probes/pty/PTY-04.tally`. Key kind: value.

step 1 · before: joint income 60000.00 + 52000.00 = 112000.00 · lines 31, 39, 44–45 · L §3.5, §5.10 (sum) · same: the sum over the child's parents (a relation input, L §3.5) reads both parents' rows
step 2 · before: Ania: not more than 112000.00: True = 500.00 · lines 27–29, 33–35, 39 · L §5.10, §3.2 · same: eligible true; 1 000.00 × 0.5 = 500 DUK (num: no rounding is stated, L:1485)
step 3 · before: Tomek: True = 500.00 · lines 27–35, 39 · L §5.10 · same
step 4 · after: joint income 60000.00 + 52000.01 = 112000.01 · lines 31, 37, 46 · L §5.10 (as if), §5.16 (with) · same: the `with` pin replaces income[tomek, 2024] by income_restated[tomek, 2024], evaluated unpinned (L:810, L:817), and the sum is recomputed (L:821)
step 5 · after: Ania: False = 0.00 · lines 27–35, 37 · L §5.10 · same: Tomek's figure moves Ania's relief: eligible false, the `if` gives 0
step 6 · after: Tomek: False = 0.00 · lines 27–35, 37 · L §5.10 · same
ask returns: value · q_before 112000.00, true, 500/500; q_after 112000.01, false, 0/0 (q_after's register lists the pin, L:1572)
checks: parsecheck ok · loadcheck ok
note: the restatement is a with pin, not a later version (O:PTY-04-1): both figures are stipulations known from `on` (L:459), and two rows of income[tomek, 2024] known the same day tie (L §4.6 step 3, LOAD-OVERLAP)
