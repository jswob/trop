# Map · TIME-07 · `probes/time/TIME-07.tally` · awkward

step 1 · stated order: pay, then the rule: 1500.00 + 6000.00 − floor 2000.00 = 5500.00 · lines 10–11, 13–16, 18–28, 32, 38–43 · L §5.13.1 (`sweep` reads `bal(konto)`, so it is a state pay; `wage` is a flow pay), L §5.13.2 (each day books flow pays' movements first, then runs state pays), L §5.9 (opening at S) · same: `bal(konto)` at the sweep's point is 1500.00 + 6000.00, the sweep moves 5500.00, `flows(sweep, 2025-05-15)` reads it. The order is the language's fixed one, not a line of the text: card rule 2 is true of every Tally model, and its reverse couldn't be stated
step 2 · reverse order: 1500.00 under the floor = 0 · line 30 · L §5.13.1 ("inferred, never declared": a flow pay can't be put after a state pay), L §5.16 (`bal(x, d)` in an ask) · same value, through a workaround: the rule's formula is rerun by hand on the balance at the end of the day before, since no text orders the pay after the rule
note · money DUK read with minor unit 0.01 (S:101)
ask returns: value (5500.00, 0.00) · two asks
checks: parsecheck ok · loadcheck ok
