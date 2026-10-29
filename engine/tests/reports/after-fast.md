# After session 4.5e: the memory layer

Session 4.5e, 2026-10-29. `engine/tools/refcore_fast/` is the frozen refcore with a memory layer in front, wrapping its methods from outside (refcore and the spec are byte-identical to `tally-2.0`). Measured on PyPy at `--jobs 3`, load average 3–6.

## Equality

- `run.py --interp both --jobs 3`, the whole quick set: **0 pair differences**.
  - interpreter: 1, the 950 tests passed;
  - unit: 115 same;
  - integration: 366 same, and 1 known fault reading `known` on both sides, as before;
  - 85 not run, as before.
- The 950 tests also pass under `--verify 1`.
- The heavy checks (43 over 20 s on refcore) under `--verify 50`: **43 same, no mismatch**. `I:X:konto_balance@b_r1` went over at 60.2 s at 3 jobs (verify's recomputations cost time), and passed alone in 50.8 s.
- The planted key omission (`--plant`) is caught by `--verify 1` on 7 unit checks; LAW-03 is in `run.py --self-test`.

## Per layer, cold

Check-seconds are summed from `results/pypy.tsv` (the 4.5c cold run) and `results/pypy+fast.tsv`.

| Layer | refcore check-s | fast check-s | gain | fast wall, 3 jobs |
|---|---|---|---|---|
| interpreter | 17 | 16 | 1.1× | 16 s |
| unit | 196 | 162 | 1.2× | 66 s |
| integration | 3 331 | 2 582 | 1.29× | about 860 s (711 s measured with 12 checks cached, plus their 455 s ÷ 3) |

- The 4.5c refcore cold run was 946 s at 4 jobs, integration 861 s.
- At 3 jobs, refcore's integration would be about 1 110 s (3 331 ÷ 3).

**The target is not met:**
- **Integration under 10 minutes cold at 3 jobs:** about 14 minutes on fast, about 4 minutes short.
- **No check over 30 s:** 36 checks are still over 30 s, and 5 over 50 s: LH-10 54 s, X's `replay_september@b_r1` 53 s and `konto_balance@b_r1` 51 s, LH-17-light 51 s, `replay_september@b_r1s` 50 s.

## The 12 slowest (refcore) on fast

| check | refcore s | fast s | gain |
|---|---|---|---|
| I:B:b03_sold | 58.67 | 34.96 | 1.68× |
| I:B:b08_gap | 57.81 | 33.18 | 1.74× |
| I:B:b08_no_row | 57.38 | 32.39 | 1.77× |
| I:B:b08_unmatched | 57.01 | 34.32 | 1.66× |
| I:E1:tx2_worth | 57.01 | 39.84 | 1.43× |
| I:E1:tx2_cis_explain | 56.50 | 39.35 | 1.44× |
| I:B:b08_reversed | 56.44 | 33.66 | 1.68× |
| I:B:b03_lazur | 56.18 | 34.02 | 1.65× |
| I:B:b08_residuals | 55.87 | 33.91 | 1.65× |
| I:B:b03_november | 55.80 | 34.05 | 1.64× |
| I:light/LH-17-mc-light | 55.77 | 51.22 | 1.09× |
| I:M:LH-10 | 55.21 | 54.06 | 1.02× |

The median is **1.65×**, under the brief's 2× bar (rule 5). The user chose to keep the layer and finish the proof.

**What it buys:** the B and E1 asks move from the edge of the 60 s budget to 32–40 s.

**What doesn't gain:** LH-10, LH-17-light and household X. Their time is refcore's own def memo, the key built on each of about 1.45 M calls, not repeated work (`tmp/session4_5/e/plan.md`).

## What was added

- **`engine/tools/refcore_fast/`:** 262 lines, `layer.py` 235 and `__main__.py` 27. It wraps:
  - `Engine.candidates`: a row index (the rows whose pattern matches the keys), then refcore's own `candidates` on those rows;
  - `Engine.read_input`: plain reads only (no pins, no tracing, no expression rows, no prices). Each is remembered with its notes (register entries, rows used and the rest of the collector, recorded in a child `Coll` and merged on reuse) and with `_K`;
  - `InputInfo.lawless` and `IndexInfo.holding`: pure scans of static lists.
- **`run.py`:**
  - `--interp refcore|fast|both`, `--verify N`, variants `<runtime>+fast[+v<N>]`. The fast keys add the variant, the hash of refcore_fast and its argv; the plain keys are unchanged.
  - 3 new self-test items, 13 in all.
  - The process-group kill accepts EPERM (macOS, the group already gone), which crashed a run.
- **`TESTING.md` §5** and **`README.md`:** the `--interp both` rule, and which side serves what.

## The load rows on fast (step 6, 2026-10-29 17:03–17:17)

The 50 checks retired for time ran once on fast, at 60 s and `--jobs 3`, with the watchdog on. The `not-run:` prefix was lifted for this one run only; `checks.tsv` is unchanged.

| Rows | Checks | On fast |
|---|---|---|
| B-05's 3 asks (L:16) | 3 | over 60 s |
| B-12's walk asks `@k5` (L:5) | 7 | over 60 s |
| K10 (L:10), K11 (L:11) | 6, 5 | over 60 s |
| A-13 (b) `I:A:A-13~A5` (L:17) | 1 | over 60 s |
| LH-16-light whole (L:12) | 1 | over 60 s |
| K3 `I:B:*@k3` | 27 | 24 same (4–45 s), and B-05's 3 over |

- **Nothing that was retired for being over 60 s fits.** Every load row stays load, and `load.tsv` is unchanged.
- **K3's 24** fit on refcore too: K3 was retired because its cut buys no time (44 s against 45 s), not for being over. They stay retired unless the user wants them back. They would add about 16 minutes of check time with no new capability.
- **B-12's walks stopped for memory on snap14 (L:14) were not run.** The memo grows memory: `I:B:b03_sold` peaks at 2.1 GB on refcore and 3.2 GB on fast (`/usr/bin/time -l`).
- **The watchdog gap:** session 5's `memcap.py` matches `refcore run`, so it doesn't see `refcore_fast … run`. Every fast run before 17:02 ran without the 8 GB cap. No fast run took more than 60 s, so the exposure was a few GB at most, but it went unchecked. `tmp/session4_5/e/tools/memcap_fast.py` is a copy that matches `refcore_fast` (only the match and the log differ). It has run since 17:02 and logged no stop. **Fast runs need it on** (`pgrep -fl memcap_fast`).
