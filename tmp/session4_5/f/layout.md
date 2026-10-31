# The clean layout: the proposal (session 4.5f, Part 2 step 1)

Generated table: `moves.tsv` (2 112 rows, old → new, by `tools/layout.py`). The script checks that no two files land on one path, that nothing already sits at a new path, and that every `use "…"` line resolves to its moved target. It passes: 0 problems.

## The tree under `engine/tests/`

```
engine/tests/
  README.md  TESTING.md  findings.md  known-faults.md      the rules and the records
  run.py                                                  the runner (finds the root from its own place)
  checks.tsv  examples.tsv (new)                          the checks; the examples and their three levels
  tools/      build_checks.py tracked_check.py knobdiff.py memcap.py (one watchdog) answers.py (session 5's bridge)
  map/        tags.tsv coverage.py knobs.md gaps.md …     the catalogue's map
  cases/      the case set 1.1 (stories, questions, keys, probe cards, compare.py, key scripts)
  unit/
    probes/   the probe texts, one per catalogue edge     (was tmp/design2/cases/probes, 103)
  integration/
    households/  A B D E1, the judged texts               (was tmp/design2/cases/households)
    law/         the law texts                            (was tmp/design2/cases/law)
    cuts/        A B D E1 E1-02-B B-law: the cut-down copies the household checks run on (was scratch/mc-a, mc-b, mc-d, w-e1, tx2, w-b)
    amendments/  household X: its changes, cut-downs and minimal texts (was scratch/mc-x)
    models/      the 77 old models; models-1.0/ CMP-01's tally-1 text (read by its key script)
    light/       the light twins LH-15/16/17
    blind/       the blind texts I-6..I-15 (modeller-b1, -b2, b2fix)
    new/         A-13, B-13, D-13, LAW-20 (4.5c's fill)
    split/       the capability copies (k3-b, k4-b07, k5-b12) and split.tsv
    recipes/  keys-extra/
  load/       load.tsv  benchmarks.md                     the load rows and their recipes
  baselines/  snap16-probes/ -probe-sets/ -models/ -light/ -b1/ -b2/ -b2fix/ -house/ -e1/, e1-02-B16.out, 4.5c/, session5-visible/ (the 14 witness runs)
  defs/       models.list probes.list probe-sets.list light.list b1.list b2.list (the run definitions build_checks.py reads)
  reports/    after.md after-fast.md before-after*.md over-budget*.md adhoc.log
  .gitignore  cache/ out/ results/ __pycache__/
```

## Counts

| | files |
|---|---|
| moved by `git mv` | 2 112 |
| stay where they are | `engine/` 1 215 (spec, refcore, refcore_fast), `tmp/cases/tally-1.0.1/` 509 (session 3's case set, read by no check), `tmp/session4_5/` 17, `tmp/active/` 1 |
| added (untracked today) | the judged texts not yet tracked: 160 household files (D 89, E1 9, A 27 and B 35 law copies and asks), 18 law texts; the 10 blind texts' stories (`hardset/invented-b1`, `-b2`), so each example has its story |

## What stays in `tmp/`

- `tmp/design2/scratch/`: everything no check reads (untracked today, and it stays untracked).
- `tmp/session5/visible/`: retired after Part 3 with a README line pointing here.
- The holdout: untouched.
- Cited by path, not moved: `tmp/active/session4_5-research.md`.
