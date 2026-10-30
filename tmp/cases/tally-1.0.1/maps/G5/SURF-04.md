# SURF-04 · map
Texts: `probes/surf/SURF-04/SURF-04.tally`, `probes/surf/SURF-04/redefine.tally` (a scenario). No key (a passage).

req 1 · the formula stated once · lines 10, 20–22, 28–33, 47–52 · L §5.10 (a def with one rule, `v` naming its version), L §7.2 (`round` half-up to 0.01) · same: contrib = round(rate × max(0, min(gross, cap − gross before)), 0.01, half-up); 9.76 % and the cap are rows of contrib_2025
req 2 · used in the payroll · lines 35–39, 41 · L §5.13 (a pay's amount reads a def) · same: contrib_pay moves contrib[m] on the 10th
req 3 · used in the yearly return's check · line 43 · L §5.16 (`check`), L:964 (`flows` of a pay over a span) · same: what was withheld (the pay's flows over 2025) equals the sum of the formula
req 4 · used in a what-if with a raise · line 45 · L §5.16 (`with` pins an input for one ask) · same: the same def with salary pinned to raised_salary
req 5 · ids in ASCII · whole text · L §2.1 (L:70 ids are ASCII) · same
req 6 · the same id defined a second time is refused before anything runs · redefine.tally line 6, SURF-04.tally line 28 · L §2.5 (one namespace), L §8.3 (L:1590 LOAD-DUP; L:1545 a load refusal in a scenario file refuses only that scenario) · same: loadcheck reports LOAD-DUP contrib (redefine.tally:6 vs SURF-04.tally:28); the reality alone loads clean
(the figures the card leaves out) · lines 51, 57, 62 · L §4.5, L §8.2 · same: cap, gross and raise are `assumed` illustration rows

ask returns: value (payroll, raise), check (ok) · the scenario redefine is refused at load (LOAD-DUP), the reality runs
checks: parsecheck ok · loadcheck LOAD-DUP on the directory (intended: redefine.tally:6); SURF-04.tally alone ok
