# tmp/tests: the runner

One runner for the interpreter, unit and integration layers (session 4.5a, 2026-10-29). It runs each check on the frozen `engine/tools/refcore`, unchanged, and compares its output with the stored CPython baseline. Standard library only.

## How to run

```sh
python3 -I -B tmp/tests/run.py --self-test                 # must pass before a real run
python3 -I -B tmp/tests/run.py                             # every check, PyPy, 4 jobs, 60 s each
python3 -I -B tmp/tests/run.py 'U:*' 'I:M:CB-0*'           # by id glob
python3 -I -B tmp/tests/run.py --layer unit,interpreter --runtime both
python3 -I -B tmp/tests/run.py --list 'I:B:*'              # what a selection holds
python3 -I -B tmp/tests/run.py --tables                    # before-after.md, over-budget.md
```

- `--runtime pypy` (default), `cpython` or `both`. `both` runs each check on the two and also requires their outputs to be equal: the rule for any change to how the interpreter runs.
- `--jobs N`, at most 4 (32 GB; the watchdog `tmp/session5/tools/memcap.py` stops any `refcore run` past 8 GB, recorded as `killed`).
- `--budget S` (default 60, the user's rule). A run past it is killed and recorded as `over`, and not rerun.
- `--force` reruns cached checks.
- Each call prints a table per layer (same, differ, over, killed, known fault, fixed, cached, not run, wall time). Results: `results/<runtime>.tsv`, the latest result per id; a differing output is kept in `out/<runtime>/`.
- **In git** (4.5d): `tmp/` is ignored, so a new suite file needs `git add -f` until session 4.5f moves the suite out of `tmp/`; `python3 -I -B tmp/tests/tracked_check.py` lists any file `checks.tsv` names that git doesn't track (exit 1).

## The checks and their ids

`checks.tsv`, one row per check, built by `build_checks.py` from the run definitions (read-only on them). Columns: `id`, `layer`, `cwd`, `files` (run order), `asks` (empty = all), `options`, `baseline`, `block`, `baseline_s`, `question`, `note`. **Ids are for life**: 4.5b's map and 4.5c join on them.

| Layer | id | From | Baseline |
|---|---|---|---|
| interpreter | `R:all` | `refcore test` | last line `950 tests, 950 passed, 0 failed` |
| unit | `U:<path under cases/probes, no .tally>` | `scratch/lead/check/probes.list` | `snap16-probes/` |
| unit | `U:set/<first path>` | `probe-sets.list`, plus PTY-05 as `rerun.sh` runs it | `snap16-probe-sets/` |
| integration | `I:M:<model>` | `models.list` | `snap16-models/` |
| integration | `I:light/<name>`, `I:hardset/<path under hardset>`, `I:hardset/b2fix/division` | `light.list`, `b1.list`, `b2.list`, `rerun.sh` | `snap16-light/`, `-b1/`, `-b2/`, `-b2fix/` |
| integration | `I:<H>:<ask>`, H in A, B, D, E1, X; `@<run>` added when an ask is in several runs of a household | `house13.py` (A, B, D, X), `e1_rerun.sh` (E1) and `run5.py`'s E1-02-B run; **one check per ask** | that ask's blocks in `snap16-house/`, `snap16-e1/`, `e1-02-B16.out` |

- Household checks run on the inputs snap16 ran on (`tmp/design2/scratch/`, `tmp/design2/cases/`). Session 5's copies under `tmp/session5/visible/` carry later edits (cases 1.0.2) and are not the baselines' inputs.
- A row whose note starts `not-run:` is a run on `tmp/design2/benchmarks.md`. It is listed, so the ids exist, and never run.
- `question`: the question whose ask file declares the ask (`asks/B-01.tally`, `asks-D-01.tally`), empty where no file names one. `note`: `run=` the run the ask was cut from, `run_asks=` its number of asks, `run_cpython_s=` that whole run's CPython seconds where stored.

## Compare

- Whole output (stdout then stderr), byte for byte, for unit, model, light and hardset checks.
- A household check in `run5.py`'s form: the ask's blocks (each unindented line opens a block; `#`, blank, `warning` and `WARN-UNREAD` lines dropped, lines right-stripped), plus the baseline's blocks that no ask of its run owns (a load refusal, say).
- A difference is a stop: the runner notes whether any input postdates the baseline; it never explains one away.

## Known faults and the ask rule

- **The rule** (`tmp/engine-requirements.md` §4 row 40, Q-19): a question's answer must not depend on which other questions run with it. Splitting each household run into one check per ask tests it: every ask runs alone against its run's stored answer.
- **A known fault** (`known-faults.md`) gets two checks. The guarded one runs the ask the way its baseline was made, so the stored answer still guards it (`same`). The one ending `~alone` runs the failing form (note `xfail:`): it reports `known` while the fault stands and `fixed` once it matches the baseline. A `fixed` is a signal to retire the fault, not a failure.
- KF-01: `I:D:d05_quarters` (in its run, `same`) and `I:D:d05_quarters~alone` (`known`).

## The cache

`cache/<k[:2]>/<k>.json` holds the output, seconds, rc and budget. The key `k` is sha256 over: the runtime and its version; refcore's lines of `engine/tools/SHA256SUMS` (the runner first checks every refcore file against them and refuses to start on a mismatch or an unlisted file); each input file's path and bytes in run order; the asks; the options, the full argv and the cwd. Unchanged inputs are never run again. An `over` is cached with its budget and rerun only under a larger `--budget` or `--force`.

## The budget

Every interpreter run gets 60 s. A check past it is `over`: a fault to fix (by the halving test, 4.5b, and the split, 4.5c), not a check to move. `over-budget.md` lists them with their files, asks and stored CPython seconds.

## Added in session 4.5c

- **`--adhoc [--cwd DIR] -- run <files> [--ask …]`:** one run of the frozen refcore on PyPy, 60 s, logged in `adhoc.log`, not cached and not recorded in `results/`. For writers and the judge.
- **The split** (`TESTING.md` §4): capability copies under `texts/split/`, each with a `.knobs` file that `knobdiff.py` checks; `texts/split/split.tsv` tells `build_checks.py` which checks to clone (`<id>@k<n>`). The modes are `twin`, `twin-cut` (adds `cmp=no-left-out`), `J`, `per-ask` (`<id>~<ask>@k<n>`) and `twin=<path>`.
- **The verdict `ran`:** a check with no baseline (no byte-equal judged twin). Its output is kept in `out/pypy/` for J to mark against the key.
- **Script checks** (`options` = `script`): a key script of `keys-extra/`, run with the runtime's Python; `same` when it exits 0 and its last line ends `: 0 failed`. `files` lists the script and then the inputs it reads (hashed for the cache).
- **The `key` column** of `checks.tsv`: what stands behind the baseline (`TESTING.md` §6).
- **Retired checks:** `build_checks.py`'s `RETIRE` list prefixes a `not-run:` note. The row stays.
- **`load.tsv`:** the load layer, one row per full-size twin.
