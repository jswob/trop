# SURF-01 · map
Text: `probes/surf/SURF-01.tally`. No key (a passage).

req 1 · the rule A-1 written: on the 15th, keep the floor, move the rest to savings · lines 12–14, 20–24 · L §5.13 (pay with `on`), L §5.13.1 (L:917 `bal(x)` at the walk point) · same: sweep_a1 moves max(0, bal(konto) − keep_floor) on day_of(m, 15)
req 2 · the floor 1 000.00 a fact of the house rules, with A-1's start · lines 8, 16, 26–30 · L §5.3 (version: cite, governs, known), L §7.1 (figures are rows) · same: keep_floor is a row of a1_v1, cite house_rules, governs from 2025-01-01
req 3 · the 15th may stay bare, as a day of the month · line 24 · L §7.1 (L:1454: 0–31 bare) · same
req 4 · a named figure nothing reads is flagged · lines 18, 31 · L §8.5 (L:1659 WARN-UNREAD) · same: savings_goal is read by no def, rule, pay or ask; the loader warns
req 5 · no amount appears inside a rule's arithmetic as a literal · line 23 · L §7.1 (L:1454: 1000 is on the bare list, beside 0–31, 100, 360, 365, 366) · other: the passage holds the requirement by choice, but the spec doesn't enforce it for this card's own figure: `amount max(0, bal(konto) - 1000)` loads without LOAD-LITERAL, since the whitelist admits 1000 as a bare integer whatever it counts, so the reader can't tell from the language that a 1 000 DUK floor must be a fact (S:501)

ask returns: none (a passage; no ask asked) · the passage loads with one warning (WARN-UNREAD savings_goal)
checks: parsecheck ok · loadcheck ok (WARN-UNREAD savings_goal)
