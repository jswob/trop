# Known faults of the frozen interpreter

Faults the runner found in `engine/tools/refcore` (tally 2.0), logged for the next interpreter round. The interpreter is frozen: none is fixed here. Each fault has two checks in `checks.tsv`:
- the **guarded** check runs the ask the way its baseline was made, so the stored answer still guards it;
- the **`~alone`** check runs the failing form. It reports `known` while the fault stands and `fixed` once it matches the baseline, so a fix shows up.

The rule a fault breaks is in `tmp/engine-requirements.md` (§4 row 40, Q-19).

## KF-01: an answer that depends on which other asks run with it

- **Found:** 2026-10-29, session 4.5a, splitting household D's run `D-d05` into one check per ask.
- **Checks:** `I:D:d05_quarters` (guarded: its run's four asks, `same`) and `I:D:d05_quarters~alone` (`known`).
- **What happens:** asked alone, `d05_quarters` prints, as the last cell of each of its 9 rows,
  `refused: ENGINE-INVARIANT a booked plan amount changed: kuba_send[2026-07] booked at 2026-09-30 differs between the cuts 2026-09-30 and 2026-09-30 (E §9 invariant 5)`.
  Asked with `d05_duplicate`, `d05_missing_fee` and `d05_owed` (its snap16 run), the same cells hold dates, e.g. `2025-10-10`, and the whole run matches snap16.
- **Not the runtime:** CPython 3.14 and PyPy 3.11 print the same bytes in both forms (`tmp/session4_5/a/d05/`: `{pypy,cpython}-{alone,whole}.out`).
- **Reproduce** (cwd `tmp/design2/scratch/mc-d/cut11`, about 18 s on CPython):
  `python3 -I -B engine/tools/refcore run <the cut11/*.tally files, sorted> --ask d05_quarters`
  against the same with `--ask d05_duplicate --ask d05_missing_fee --ask d05_owed --ask d05_quarters`.
  (`python3 -I -B tmp/tests/run.py 'I:D:d05_quarters*' --force` runs both.)
- **Not explained.** The two cuts the message names are the same day, which suggests a value memoised or booked by one ask's walk is read by another, or a check that compares a fresh computation with a cached one. That is for the interpreter round to settle.
- **Scope:** 1 of the 234 household asks run alone. The other 233 print alone what they printed in their run. That covers "alone against its run", not every grouping of asks.
