# SURF-02 · map
Text: `probes/surf/SURF-02.tally`. No key (a passage).

req 1 · the condition compares two amounts ("is more than": a question, not a statement) · line 21 · L §5.10 (a rule's `when` is a bool expression; its `=` after the guard states the value), L §2.3 (`when … = …` split) · same: `when value_at_division > value_at_death` is the question, `= heir_share[h] * value_at_division - advance[h]` the value
req 2 · the buyouts summed over the heirs as one aggregate · line 25 · L §5.10 (aggregate `sum(h in heir where …: …)`) · same: keeper_pays, one sum, the heirs without a buyout kept out by `where buyout[h] != -`
req 3 · "does not apply" kept apart from a negative amount · lines 23, 36–39 · L §3.1 (L:281 `-`, not applicable, tested only with = - or != -; not 0, not a hole), L §5.10 (`excepts`) · same: jan's heir_share is `-`, so no_share gives `-`; olek's 0.5 × 360000 − 200000 = −20000.00, a negative amount (he pays in)
req 4 · the language's one order of words · lines 20–23 · L §2.3 (rule: name, `excepts`, `when`, `=`) · same: every rule reads `rule <name> [excepts …] [when <cond>] = <value>`
(the card's silence when the value did not rise) · line 22 · L §8.1 (a stated hole with its question) · same: not_risen = ?, so a non-rise is a hole, not a guess

ask returns: value · q_buyouts {ewa 180000.00, jan -, olek −20000.00}; q_keeper 160000.00 (the illustrative rows are `assumed`)
checks: parsecheck ok · loadcheck ok
