# Stale in tally 2.0

These documents are carried into the package as they are. They plan or check the engine's build for tally 1.0.1, and nothing in session 4's design round checked them against tally 2 (brief §8). Each is rewritten for tally 2 before a gate that cites it is judged (`DONE-proposals.md`). Below, per document, the sections that name a construct tally 2 removed, merged or changed, and what reaches it. The list comes from a word scan (`scratch/lead/stale_map.py`), so it says where to reread, not what the new text is. A section not listed may still be reached through a word the scan doesn't know.


## `OPERATIONS.md`

| Section | Words (count) | Reached by |
|---|---|---|
| (before the first section) | `records` (1), `scenario` (1) | D-10 (overlay of ordinary versions); D-8 (`observed` inputs) |
| 1. The store, and the life of an amendment | `record` (2), `records` (2), `scenario` (2) | D-10 (overlay of ordinary versions); D-8 (`observed` inputs) |
| 2.1 Add a fact | `arrears` (2), `correction` (1), `governs` (2), `known` (3), `pay` (3), `record` (1) | D-2, D-8 (`move`, `observed`); D-3 (per row); D-3 (required); D-6; D-6 (adjustments as ordinary movements); D-8 (`observed` inputs) |
| 2.2 Import records | `correction` (3), `governs` (3), `holds` (1), `known` (1), `pay` (2), `pays` (1), `record` (17) | D-2, D-6 (obligations are places); D-2, D-8 (`move`, `observed`); D-3 (per row); D-3 (required); D-6; D-8 (`observed` inputs) |
| 3. Add a law version | `arrears` (3), `excepts` (3), `governs` (3), `holds` (1), `known` (3), `pay` (6), `scenario` (1) | D-10 (overlay of ordinary versions); D-12 (`beats`); D-2, D-6 (obligations are places); D-2, D-8 (`move`, `observed`); D-3 (per row); D-3 (required); D-6 (adjustments as ordinary movements) |
| 4. Amend: add, advance, restate, revise | `correction` (1), `draws` (1), `known` (2), `pay` (1), `pays` (1) | D-11; D-2, D-8 (`move`, `observed`); D-3 (per row); D-6 |
| 5. Add a scenario patch | `arrears` (1), `correction` (1), `governs` (1), `known` (3), `pays` (1), `scenario` (26) | D-10 (overlay of ordinary versions); D-2, D-8 (`move`, `observed`); D-3 (per row); D-3 (required); D-6; D-6 (adjustments as ordinary movements) |
| 6. End | `holds` (1), `known` (1), `pay` (4) | D-2, D-6 (obligations are places); D-2, D-8 (`move`, `observed`); D-3 (per row) |
| 7. Plan | `bound` (1), `bounds` (1), `governs` (2), `path` (2), `pay` (1), `scenario` (4) | D-10 (overlay of ordinary versions); D-11 (`path p`, no path key); D-15; D-15 (an ordinary hole with its reason); D-2, D-8 (`move`, `observed`); D-3 (required) |
| 8. Ask a question | `pay` (1), `record` (1), `scenario` (1) | D-10 (overlay of ordinary versions); D-2, D-8 (`move`, `observed`); D-8 (`observed` inputs) |
| 9. The rest of the surface | `ledger` (1), `path` (1), `pay` (1), `record` (3), `records` (1), `scenario` (1) | D-10 (overlay of ordinary versions); D-11 (`path p`, no path key); D-2, D-8 (`move`, `observed`); D-8 (`observed` inputs); E §6.3 (the residual ledger) |
| 10. Errors and warnings of the operations | `correction` (1), `pay` (2), `pays` (1), `scenario` (2) | D-10 (overlay of ordinary versions); D-2, D-8 (`move`, `observed`); D-6 |


## `PERFORMANCE.md`

| Section | Words (count) | Reached by |
|---|---|---|
| 1. Units, and the evaluation strategy | `bounds` (1), `known` (1), `path` (1), `pay` (3), `records` (2), `state pay` (1) | D-11 (`path p`, no path key); D-15; D-2, D-8 (`move`, `observed`); D-3 (per row); D-5 (declared reads); D-8 (`observed` inputs) |
| 2.1 The flow pass | `flow pay` (1), `pay` (2) | D-2, D-8 (`move`, `observed`); D-5, D-6 |
| 2.2 The day walk | `pay` (1), `pays` (1), `records` (1), `state pay` (1), `state pays` (1) | D-2, D-8 (`move`, `observed`); D-5; D-5 (declared reads); D-8 (`observed` inputs) |
| 2.3 `plan_low`, the look-ahead | `draw` (1), `holds` (1), `known` (3), `look-ahead` (1), `path` (1), `plan_low` (1), `promised` (1), `record` (1) | D-11 (`path p`, no path key); D-11 (a distribution is a fact's value); D-2, D-6 (certainty on event kinds); D-2, D-6 (obligations are places); D-3 (per row); D-6; D-6 (the written projection, `low`); D-8 (`observed` inputs) |
| 2.4 `next_on` and `worth` | `known` (1), `next_on` (1), `worth` (1) | D-1 (`value`); D-3 (per row); D-6 (`next`) |
| 3. Cut classes, scenarios, pins and rolling origins | `known` (5), `path` (1), `pay` (1), `pays` (5), `records` (1), `scenario` (2), `state pay` (1) | D-10 (overlay of ordinary versions); D-11 (`path p`, no path key); D-2, D-8 (`move`, `observed`); D-3 (per row); D-5 (declared reads); D-8 (`observed` inputs) |
| 4.1 Invalidation | `draws` (1), `holds` (1), `known` (1), `path` (1), `records` (1), `scenario` (1) | D-10 (overlay of ordinary versions); D-11; D-11 (`path p`, no path key); D-2, D-6 (obligations are places); D-3 (per row); D-8 (`observed` inputs) |
| 4.2 The walk restart | `pay` (1), `records` (1), `state pay` (1) | D-2, D-8 (`move`, `observed`); D-5 (declared reads); D-8 (`observed` inputs) |
| 4.3 What one turn costs | `look-ahead` (1), `scenario` (3) | D-10 (overlay of ordinary versions); D-6 |
| 4.4 The fitting loop | `ledger` (1), `records` (2) | D-8 (`observed` inputs); E §6.3 (the residual ledger) |
| 5. Monte Carlo | `draw` (5), `draws` (1), `path` (13) | D-11; D-11 (`path p`, no path key); D-11 (a distribution is a fact's value) |
| 6. Numbers | `bound` (2) | D-15 (an ordinary hole with its reason) |
| 7. Cost per construct | `bound` (1), `draw` (2), `flow pay` (1), `known` (1), `law date` (1), `path` (2), `pay` (2), `plan_low` (1), `records` (1), `scenario` (2), `state pay` (1) | D-10 (overlay of ordinary versions); D-11 (`path p`, no path key); D-11 (a distribution is a fact's value); D-15 (an ordinary hole with its reason); D-2, D-8 (`move`, `observed`); D-3 (per row); D-4 (written, never inherited); D-5 (declared reads); D-5, D-6; D-6 (the written projection, `low`); D-8 (`observed` inputs) |
| 8. Budgets | `draw` (2), `look-ahead` (1), `path` (4), `scenario` (1) | D-10 (overlay of ordinary versions); D-11 (`path p`, no path key); D-11 (a distribution is a fact's value); D-6 |
| 9. What can grow, and what caps it | `bound` (1), `bounds` (3), `path` (1), `records` (1), `scenario` (1) | D-10 (overlay of ordinary versions); D-11 (`path p`, no path key); D-15; D-15 (an ordinary hole with its reason); D-8 (`observed` inputs) |
| 10. What this document doesn't settle | `draw` (1), `path` (1), `worth` (1) | D-1 (`value`); D-11 (`path p`, no path key); D-11 (a distribution is a fact's value) |


## `BOUNDS.md`

| Section | Words (count) | Reached by |
|---|---|---|
| (before the first section) | `arrears` (1), `bound` (7), `bounds` (3), `canonical` (1), `correction` (1), `draw` (5), `draws` (4), `excepts` (1), `flow pay` (1), `governs` (3), `holds` (1), `known` (5), `law date` (1), `ledger` (1), `look-ahead` (1), `path` (4), `pay` (5), `pays` (6), `plan_low` (1), `record` (6), `records` (2), `scenario` (4), `state pays` (1), `worth` (3) | D-1 (`value`); D-10 (overlay of ordinary versions); D-11; D-11 (`path p`, no path key); D-11 (a distribution is a fact's value); D-12 (`beats`); D-15; D-15 (an ordinary hole with its reason); D-2, D-6 (obligations are places); D-2, D-8 (`move`, `observed`); D-3 (per row); D-3 (required); D-4 (written, never inherited); D-5; D-5, D-6; D-6; D-6 (adjustments as ordinary movements); D-6 (the written projection, `low`); D-8 (`observed` inputs); E §6.3 (the residual ledger); L §2.4 (tally 2's canonical form) |
| Changed by the design | `bounds` (1), `draws` (1) | D-11; D-15 |


## `BUILD-ORDER.md`

| Section | Words (count) | Reached by |
|---|---|---|
| (before the first section) | `canonical` (2), `ledger` (1), `path` (1) | D-11 (`path p`, no path key); E §6.3 (the residual ledger); L §2.4 (tally 2's canonical form) |
| Summary | `bound` (2), `bounds` (2), `canonical` (1), `draws` (2), `excepts` (1), `known` (2), `ledger` (1), `path` (2), `pay` (1), `pays` (1), `plan_low` (1), `records` (1), `scenario` (1) | D-10 (overlay of ordinary versions); D-11; D-11 (`path p`, no path key); D-12 (`beats`); D-15; D-15 (an ordinary hole with its reason); D-2, D-8 (`move`, `observed`); D-3 (per row); D-6 (the written projection, `low`); D-8 (`observed` inputs); E §6.3 (the residual ledger); L §2.4 (tally 2's canonical form) |
| S. The tracer slice (before M0) | `holds` (1), `known` (1), `ledger` (1), `path` (1), `pay` (1), `pays` (1), `plan_low` (1), `promised` (1), `records` (1) | D-11 (`path p`, no path key); D-2, D-6 (certainty on event kinds); D-2, D-6 (obligations are places); D-2, D-8 (`move`, `observed`); D-3 (per row); D-6 (the written projection, `low`); D-8 (`observed` inputs); E §6.3 (the residual ledger) |
| M0. Load and format | `canonical` (1) | L §2.4 (tally 2's canonical form) |
| M1. The household forecast | `canonical` (1), `draw` (1), `excepts` (1), `free` (1), `holds` (1), `known` (1), `law date` (1), `look-ahead` (2), `next_on` (1), `pay` (2), `pays` (2), `plan_low` (1), `records` (2), `state pay` (2), `state pays` (1), `worth` (1) | D-1 (`value`); D-11 (a distribution is a fact's value); D-12 (`beats`); D-2, D-6 (obligations are places); D-2, D-8 (`move`, `observed`); D-3 (per row); D-4 (written, never inherited); D-5; D-5 (declared reads); D-6; D-6 (`next`); D-6 (the written projection, `low`); D-8 (`observed` inputs); L §2.4 (tally 2's canonical form) |
| M2. Replay against reality | `arrears` (1), `correction` (2), `holds` (1), `known` (2), `ledger` (3), `pay` (1), `pays` (1), `record` (2), `records` (3), `state pay` (1) | D-2, D-6 (obligations are places); D-2, D-8 (`move`, `observed`); D-3 (per row); D-5 (declared reads); D-6; D-6 (adjustments as ordinary movements); D-8 (`observed` inputs); E §6.3 (the residual ledger) |
| M3. The operations | `record` (2), `records` (1), `scenario` (1) | D-10 (overlay of ordinary versions); D-8 (`observed` inputs) |
| M4. Law breadth | `draw` (1), `excepts` (1), `pays` (1) | D-11 (a distribution is a fact's value); D-12 (`beats`); D-2, D-8 (`move`, `observed`) |
| M5. Allocation, holdings and many parties | `bound` (1) | D-15 (an ordinary hole with its reason) |
| M6. Exact functions and iteration | `bound` (2), `draw` (1) | D-11 (a distribution is a fact's value); D-15 (an ordinary hole with its reason) |
| M7. Monte Carlo | `canonical` (1), `draw` (3), `draws` (5), `holds` (1), `look-ahead` (2), `path` (5), `scenario` (1) | D-10 (overlay of ordinary versions); D-11; D-11 (`path p`, no path key); D-11 (a distribution is a fact's value); D-2, D-6 (obligations are places); D-6; L §2.4 (tally 2's canonical form) |
| M8. The evaluation layer | `draw` (1), `holds` (1), `known` (1), `records` (1) | D-11 (a distribution is a fact's value); D-2, D-6 (obligations are places); D-3 (per row); D-8 (`observed` inputs) |
| M9. Performance (step F) | `canonical` (1), `look-ahead` (1), `path` (2) | D-11 (`path p`, no path key); D-6; L §2.4 (tally 2's canonical form) |
| What every milestone keeps | `bounds` (2) | D-15 |


## `smoke/HH4-a.md`

| Section | Words (count) | Reached by |
|---|---|---|
| (before the first section) | `pays` (1), `records` (1) | D-2, D-8 (`move`, `observed`); D-8 (`observed` inputs) |
| 1.1 Load: indexes, places, openings | `holds` (1), `pays` (1), `state pays` (1) | D-2, D-6 (obligations are places); D-2, D-8 (`move`, `observed`); D-5 |
| 1.2 Read rule, law dates and cuts | `arrears` (1), `governs` (1), `known` (10), `law date` (2), `look-ahead` (3), `pay` (1), `record` (1), `records` (1) | D-2, D-8 (`move`, `observed`); D-3 (per row); D-3 (required); D-4 (written, never inherited); D-6; D-6 (adjustments as ordinary movements); D-8 (`observed` inputs) |
| 1.3 Flow pass: amounts and dates | `pay` (1), `pays` (4), `promised` (1) | D-2, D-6 (certainty on event kinds); D-2, D-8 (`move`, `observed`) |
| 1.4 The day walk, January–August 2026 | `known` (1), `pays` (4), `phase` (4), `plan_low` (3), `promised` (2), `record` (1), `state pays` (1) | D-2, D-6 (certainty on event kinds); D-2, D-8 (`move`, `observed`); D-3 (per row); D-5; D-5 (stratified order); D-6 (the written projection, `low`); D-8 (`observed` inputs) |
| 1.5 Back pay | `arrears` (1), `known` (1), `pays` (2), `state pays` (1) | D-2, D-8 (`move`, `observed`); D-3 (per row); D-5; D-6 (adjustments as ordinary movements) |
| 1.6 September: records, modelled past and anchor | `pays` (1), `record` (3), `records` (2) | D-2, D-8 (`move`, `observed`); D-8 (`observed` inputs) |
| 1.7 Residual ledger, konto, 2026-09 | `record` (1), `records` (1) | D-8 (`observed` inputs) |
| 1.8 October–December 2026 | `pay` (1), `promised` (2) | D-2, D-6 (certainty on event kinds); D-2, D-8 (`move`, `observed`) |
| 1.9 The walk through 2027 (B) | `pay` (2), `pays` (1), `plan_low` (1), `promised` (2), `record` (1) | D-2, D-6 (certainty on event kinds); D-2, D-8 (`move`, `observed`); D-6 (the written projection, `low`); D-8 (`observed` inputs) |
| 1.12 The 2027 OKI tax (B) | `worth` (1) | D-1 (`value`) |
| 1.13 net_worth at 2026-12-31 (B) | `worth` (1) | D-1 (`value`) |
| 1.14 explain room[2026-11] (B) | `holds` (1), `law date` (1), `pays` (1), `phase` (2), `plan_low` (1), `state pays` (1) | D-2, D-6 (obligations are places); D-2, D-8 (`move`, `observed`); D-4 (written, never inherited); D-5; D-5 (stratified order); D-6 (the written projection, `low`) |
| 3. Arithmetic (one-line commands with their output) | `plan_low` (1), `promised` (4), `worth` (3) | D-1 (`value`); D-2, D-6 (certainty on event kinds); D-6 (the written projection, `low`) |
| 4. Outputs | `law date` (1), `ledger` (1), `phase` (1), `plan_low` (1), `record` (2), `records` (2) | D-4 (written, never inherited); D-5 (stratified order); D-6 (the written projection, `low`); D-8 (`observed` inputs); E §6.3 (the residual ledger) |
| 5. Ambiguities | `governs` (2), `law date` (1), `ledger` (2), `look-ahead` (1), `pay` (2), `pays` (1), `record` (8), `records` (2), `state pay` (1), `worth` (1) | D-1 (`value`); D-2, D-8 (`move`, `observed`); D-3 (required); D-4 (written, never inherited); D-5 (declared reads); D-6; D-8 (`observed` inputs); E §6.3 (the residual ledger) |


## `smoke/HH4-b.md`

| Section | Words (count) | Reached by |
|---|---|---|
| (before the first section) | `known` (1), `pays` (1), `records` (1) | D-2, D-8 (`move`, `observed`); D-3 (per row); D-8 (`observed` inputs) |
| Pay dates | `pay` (1), `phase` (1), `phases` (1) | D-2, D-8 (`move`, `observed`); D-5; D-5 (stratified order) |
| Standing figures | `excepts` (1), `holds` (1), `known` (2), `law date` (1), `pay` (1), `pays` (1), `promised` (1) | D-12 (`beats`); D-2, D-6 (certainty on event kinds); D-2, D-6 (obligations are places); D-2, D-8 (`move`, `observed`); D-3 (per row); D-4 (written, never inherited) |
| 2.1 wife_net (LANGUAGE §5.10, §7.2; employment.tally) | `governs` (1), `holds` (2), `law date` (2), `record` (2) | D-2, D-6 (obligations are places); D-3 (required); D-4 (written, never inherited); D-8 (`observed` inputs) |
| 2.2 social_and_health (business.tally, common.tally) | `law date` (1), `record` (1) | D-4 (written, never inherited); D-8 (`observed` inputs) |
| 2.3 pit37 (employment.tally) | `pays` (1) | D-2, D-8 (`move`, `observed`) |
| 2.4 pit28 and the ryczałt advances (business.tally) | `pays` (1) | D-2, D-8 (`move`, `observed`) |
| 2.5 health_settlement (business.tally) | `pays` (1) | D-2, D-8 (`move`, `observed`) |
| Rules applied | `holds` (1), `known` (2), `look-ahead` (1), `pay` (2), `pays` (4), `phase` (4), `plan_low` (3), `promised` (2), `record` (1), `records` (1), `state pays` (2) | D-2, D-6 (certainty on event kinds); D-2, D-6 (obligations are places); D-2, D-8 (`move`, `observed`); D-3 (per row); D-5; D-5 (stratified order); D-6; D-6 (the written projection, `low`); D-8 (`observed` inputs) |
| Results | `pays` (2), `plan_low` (5) | D-2, D-8 (`move`, `observed`); D-6 (the written projection, `low`) |
| 4. September 2026, covered by `mbank_2026_09` (LANGUAGE §5.18; ENGINE §4.2, §6.1, §6.2) | `known` (1), `ledger` (1), `look-ahead` (1), `pay` (2), `pays` (2), `plan_low` (3), `record` (2), `records` (7), `state pays` (1) | D-2, D-8 (`move`, `observed`); D-3 (per row); D-5; D-6; D-6 (the written projection, `low`); D-8 (`observed` inputs); E §6.3 (the residual ledger) |
| 5. replay_september: `residuals konto 2026-09` (LANGUAGE §5.16; ENGINE §6.3) | `record` (4), `records` (5) | D-8 (`observed` inputs) |
| Booking and the walk | `arrears` (1), `known` (1), `plan_low` (5), `promised` (4) | D-2, D-6 (certainty on event kinds); D-3 (per row); D-6 (adjustments as ordinary movements); D-6 (the written projection, `low`) |
| explain_room: `explain room[2026-11]` (LANGUAGE §5.16; ENGINE §10) | `draws` (1), `free` (1), `holds` (2), `known` (1), `law date` (1), `look-ahead` (2), `pay` (2), `phase` (3), `plan_low` (1), `record` (1), `state pay` (1) | D-11; D-2, D-6 (obligations are places); D-2, D-8 (`move`, `observed`); D-3 (per row); D-4 (written, never inherited); D-5 (declared reads); D-5 (stratified order); D-6; D-6 (the written projection, `low`); D-8 (`observed` inputs) |
| 7. Units, net_worth[2026-12-31], ike_left[2026] (§5.9; wrappers.tally; pays.tally) | `phase` (2), `record` (2), `worth` (2) | D-1 (`value`); D-5 (stratified order); D-8 (`observed` inputs) |
| 8. 2027, and Jan–Mar 2028 | `plan_low` (32), `record` (1) | D-6 (the written projection, `low`); D-8 (`observed` inputs) |
| Results that follow | `pay` (1), `state pay` (1), `worth` (1) | D-1 (`value`); D-2, D-8 (`move`, `observed`); D-5 (declared reads) |
| 10. Outputs | `law date` (1), `ledger` (1), `phase` (2), `plan_low` (1), `records` (1) | D-4 (written, never inherited); D-5 (stratified order); D-6 (the written projection, `low`); D-8 (`observed` inputs); E §6.3 (the residual ledger) |
| A. A promised movement after its date, in a forecast (§5.13 `promised`; ENGINE §4.2 P(t)) | `plan_low` (1), `promised` (2), `record` (2) | D-2, D-6 (certainty on event kinds); D-6 (the written projection, `low`); D-8 (`observed` inputs) |
| B. Modelled past when only one end is covered (§5.18 Coverage; ENGINE §6.2) | `pay` (1), `pays` (1), `records` (1) | D-2, D-8 (`move`, `observed`); D-8 (`observed` inputs) |
| C. What `plan_low`'s register holds (§8.2 "transitive"; §5.13.1; ENGINE §4.2) | `known` (1), `look-ahead` (1), `plan_low` (1), `record` (1) | D-3 (per row); D-6; D-6 (the written projection, `low`); D-8 (`observed` inputs) |
| D. The hold def scans every month to 2059 (business.tally `vat_owed`; §8.2) | `law date` (1), `plan_low` (1) | D-4 (written, never inherited); D-6 (the written projection, `low`) |
| E. The totals of a `balance` group (ENGINE §6.3 "Totals") | `records` (2) | D-8 (`observed` inputs) |
| F. `worth(x, d)` within a day (§5.13.1; wrappers.tally `oki_avg`) | `worth` (1) | D-1 (`value`) |
| Points I checked that the text settles | `arrears` (1), `look-ahead` (1), `pays` (2), `plan_low` (1), `record` (2), `records` (1) | D-2, D-8 (`move`, `observed`); D-6; D-6 (adjustments as ordinary movements); D-6 (the written projection, `low`); D-8 (`observed` inputs) |


## `verification/goldens.md`

| Section | Words (count) | Reached by |
|---|---|---|
| Goldens | `ledger` (2), `pay` (1), `record` (1), `records` (1), `scenario` (1), `worth` (3) | D-1 (`value`); D-10 (overlay of ordinary versions); D-2, D-8 (`move`, `observed`); D-8 (`observed` inputs); E §6.3 (the residual ledger) |
| Pins (36) | `canonical` (1), `known` (3), `worth` (2) | D-1 (`value`); D-3 (per row); L §2.4 (tally 2's canonical form) |
| Findings for `changes.md` | `canonical` (1), `worth` (1) | D-1 (`value`); L §2.4 (tally 2's canonical form) |


## `verification/hindcast.md`

| Section | Words (count) | Reached by |
|---|---|---|
| (before the first section) | `bounds` (1), `known` (2), `ledger` (1) | D-15; D-3 (per row); E §6.3 (the residual ledger) |
| 1. Roles | `ledger` (4) | E §6.3 (the residual ledger) |
| 2. The base fixture | `ledger` (1), `pay` (1), `record` (1) | D-2, D-8 (`move`, `observed`); D-8 (`observed` inputs); E §6.3 (the residual ledger) |
| 3. Planted causes | `pay` (1) | D-2, D-8 (`move`, `observed`) |
| 4. Rules the fixtures also check | `holds` (3) | D-2, D-6 (obligations are places) |
| 6. Open points | `bounds` (1) | D-15 |

