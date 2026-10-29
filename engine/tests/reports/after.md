# After session 4.5: the quick set, cold

Session 4.5c, 2026-10-29 15:17. `run.py --force --jobs 4`, every check of the interpreter, unit and integration layers on PyPy, against 4.5a's first full pass (`before-after.4.5a.md`; `before-after.md` is the new per-group table).

## Per layer

| Layer | 4.5a: checks, result, wall | 4.5c: checks, result, wall |
|---|---|---|
| interpreter | 1: same, 15 s | 1: same, 18 s |
| unit | 112: all same, 25 s | **115**: all same, 67 s (the pair card's two runs are 30 s each) |
| integration | 360 run: 320 same, 1 differ (became KF-01), 4 over, 35 not run; 649 s | 452 rows: **363 same, 1 known fault (KF-01), 0 differ**, 3 over (below), 85 not run; 861 s |
| **total** | 474 rows, **689 s**, 4 over | **568 rows, 946 s**; no check over budget at 3 jobs |

- **The 3 overs of the cold run** were `I:M:LH-10` (60.02 s), `I:X:konto_balance@b_r1` (60.17 s) and `I:X:replay_september@b_r1` (60.09 s). In 4.5a they took 53, 50 and 51 s. The load average was 7.6–7.8 on 6 performance cores: the 4 jobs, a system process stuck at 100 % for 6 days (`BTLEServer`), and another session. Rerun with `--jobs 3`, they took 55.2, 51.3 and 49.1 s, all same. **The margin is thin:** under that load 12 checks ran at 55–59 s and 26 at 50–59 s, mostly B's asks on cs11 and E1-02-B's. Session 5 should run with `--jobs 3`, or on a quiet machine.
- **The 85 not run:**
  - the 35 benchmark rows 4.5a listed;
  - 50 retired by the user's calls: K3's 27 (no speed gain), K10's 6 and K11's 5 (over alone), B-12's 7 walk asks, B-05's 3, the whole LH-16-light run;
  - A-13 (b)'s `windows each` ask.
  Each is a row of `load.tsv` or names one.
- **What made it fit:** 4.5a's 4 overs are gone. B-05's 3 are now load row L:16, measured once at 85–91 s, byte-equal to snap16. LH-16-light now runs as 22 one-ask checks, the slowest 39 s, all byte-equal to its twin.
- **Against before 4.5:** session 5's B runs took 10–22 min each on CPython, and one stopped after 15 min with nothing. Now every check is under 60 s. The whole quick set takes 16 min cold, and after that only changed checks run (the cache: a second pass of unchanged inputs takes seconds).

## What was added

| | Checks | Result |
|---|---|---|
| question column filled | 26 household asks | 0 left without a question |
| the split (capability copies) | K13 22, K4 6, K5 4 (+ 7 retired), K3 27 retired, K10/K11 11 retired | K13 22 byte-equal; K4 6 and K5 2 passed by J; O:B-3's 2 have no key (read against the papers) |
| the fill (cases 1.1, blind texts) | A-13 5, B-13 5, D-13 2, pair card 2, LAW-20 1 | 14 passed by J (A-13 (c) after one text fix: both marks recorded), 1 over (load L:17) |
| independent keys (item 6) | `I:M:LH-11~key`, `I:M:CMP-01~key` | both pass |

The 24 J-passed outputs are the baselines of their checks (`baselines/4.5c/`), so each now catches a change.

## Coverage (`map/coverage.py`)

| | Before (4.5b) | After |
|---|---|---|
| edges | 92 | 93 (LAW-20) |
| axis pairs, households + old models (of 120) | 91 | **115**; 116 with the pair card |
| edge pairs | 696 | 892 |
| ★ edges met at least twice, households alone | 13 / 14 | 14 / 14 |

**Unmet:** the 4 SURF pairs, which are accepted.

## The load list

`load.tsv`: **16 rows** (L:1–6, L:8–17), each with its capability checks and the figures left unjudged until the engine.

## Over budget

None left in the quick set at `--jobs 3` (`over-budget.md` is empty). The brief's target is met, with the thin margin noted above.
