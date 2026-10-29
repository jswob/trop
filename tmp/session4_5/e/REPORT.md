# Session 4.5e: report (stopped at the 2× rule)

2026-10-29. The memory layer `engine/tools/refcore_fast/` is built and proven equal, but its gain is **1.65× median** on the 12 slowest integration checks, under the brief's 2× bar (rule 5). So the session stopped after round 1, as the brief says. The user decides whether to go on.

## What was built

- **`engine/tools/refcore_fast/`** (262 lines: `layer.py` 235, `__main__.py` 27). refcore and the spec are untouched (`git diff tally-2.0` clean). The layer wraps four methods from outside:
  1. **`Engine.candidates`:** an index of the rows matching the keys. refcore's own `candidates` then runs on those rows.
  2. **`Engine.read_input`:**
     - a plain read only, with no pins, no tracing, no expression rows and no prices;
     - remembered with its notes: register entries, rows used, axes and the rest, recorded in a child `Coll` and merged again on reuse, plus `self._K`.
  3. **`InputInfo.lawless`** and **4. `IndexInfo.holding`:** pure scans.
- **`--verify N`:** recomputes one in N remembered answers per memo with the layer bypassed, and compares the answer and its notes. A difference exits 70.
- **`--plant`:** a planted key omission.
- **`run.py --interp refcore|fast|both [--verify N]`:** the default and its cache keys are unchanged.
- **The self-test:** 3 new items, all 13 pass, including the planted omission being caught on LAW-03.

## Equality so far

- refcore's 950 tests through the layer: 950 passed, also under `--verify 1`.
- The unit layer under `--verify 1`: 108 same; the 7 that differ are only with the plant.
- The 12 slowest integration checks: all `same`.

Not yet run: the whole quick set under `--interp both`, and the heavy checks under `--verify 50`. These are steps 5–6 if the user goes on.

## Why the gain stops at 1.65×

The profile (`plan.md`, `progress.md`):
- **B and E1:** the row scan (31 %) and the repeated reads are gone. `read_input` falls from 46 % to 13.5 % of the time.
  - The rest is the evaluator's own work, spread thin: plan instances (`walk.prepare` 73 %), def cells, `key_values` and `datetime`.
  - The reads repeat only 3× there, because a walk reads many different days.
- **LH-10 and LH-17-light (1.0–1.1×):** 92 % is refcore's own def memo. Its cost is building a key on each of 1.45 M calls over 690 k distinct cells, not repeated work a cache could save.

## The user's options

1. **Keep round 1 and finish the proof** (steps 5–7, about 0.3–0.5 %).
   - The B and E1 asks go from about 57 s to about 34–40 s, so the thin 60 s margin becomes comfortable.
   - LH-10 and LH-17-light stay at about 55 s.
   - The integration layer would drop from about 861 s to roughly 600–650 s cold (an estimate, not measured).
   - Some of the load rows (B-05's 85–91 s) might fit.
2. **A round 2** on the def cells (`read_def`, the instance bodies of `m_eval_inst`).
   - Their notes include refcore's `Engine.memo` journal, which walks roll back (`m_guarded_walk`). That is the hazard rule 2 warns about, and it would push the layer past the 300-line cap.
   - Not recommended.
3. **Drop the layer:** revert the commit. The suite is unchanged without it.

## Cost

About 0.15 % of the week at the stop (estimate 1.1–2.2 %).
