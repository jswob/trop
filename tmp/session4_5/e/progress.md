# Session 4.5e progress

Session id `82f57430-148b-4759-984f-46c6c2bc5e45`. Brief `tmp/active/session4_5e-brief.md`. Budget: expect 1.1–2.2 %, stop and ask at 3.5 %.

## Step 1, start checks (2026-10-29 16:00)

- `git diff --quiet tally-2.0 -- engine/spec engine/tools/refcore`: clean. Both `SHA256SUMS` (engine/tools, engine/spec) verify.
- `tracked_check.py`: 1390 files, 0 untracked, exit 0.
- Warm `run.py --jobs 3`: all cached (1 + 115 + 367 cached, 85 not run), 1 s.
- Watchdog memcap.py (pid 58644) running.
- Budget: 0.01 %.

## Step 2, profiles (16:05–16:12)

`plan.md`, from `tools/prof.py` (count, count2, sample; cProfile is unusable here, see plan.md). B's and E1's row scan: 1.5 M `candidates` calls over 649 distinct (rows, keys). Reads repeat only 3× on B and 50× on LH-10; LH-10 spends 84 % outside `read_input`.

## Step 3–4, layer and runner (16:12–16:20)

- `engine/tools/refcore_fast/`: `__main__.py` 27 lines, `layer.py` 235. It wraps:
  - `Engine.candidates` (the row index);
  - `Engine.read_input` (the memo, with its notes recorded in a child `Coll` and replayed, and `_K`);
  - `InputInfo.lawless` and `IndexInfo.holding`.
- `refcore_fast test` gives 950 passed, and 950 passed under `--verify 1`.
- **The plant:** the unit layer under `--plant --verify 1` stops (exit 70) on 7 checks: LAW-03, LAW-04, PTY-01, SURF-04, TIME-03, set LAW-13, set SURF-04. LAW-03 reads `rate` on two days, 17/100 against 3/25.
- **`run.py --interp refcore|fast|both`, `--verify N`:**
  - variants `<runtime>[+fast[+v<N>]]`;
  - the fast keys add the variant, the hash of refcore_fast and its argv;
  - the plain keys are unchanged (a warm run is all cached).
- The self-test has 13 items, 3 of them new, and passes.

## Step 5 (part), stop rule (16:21): STOPPED, median 1.65×

The 12 slowest integration checks on `fast`, `--jobs 3` (load average 5.3), against the 4.5c cold run's refcore times:

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

All 12 are `same`. The median is 1.65×, under the 2× bar, so this is a stop under rule 5. The refcore times were taken at load 7.6, so the like-for-like gain is, if anything, a little lower.

**The fast run's profile** (`tools/sampfast1.txt`, `sampfast3.txt`):
- On B, `read_input` is down to 13.5 % inclusive (it was 46 %).
- The rest is spread across refcore's own def cells: `read_def` 90 %, `walk.prepare` 73 %, `members` 36 %. The self time is spread too: a list comprehension 8.7 %, `key_values` 6 %, `datetime` 4 %.
- On LH-10, 92 % is in `read_def`. Most of that is refcore's own def memo (`_read_def` 14 % self, `key_values` 13 %): the cost of building the key on each of 1.45 M calls, not repeated work.
