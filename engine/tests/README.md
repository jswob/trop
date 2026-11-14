# engine/tests: the test suite

One runner for the interpreter, unit and integration layers (session 4.5a, 2026-10-29). It runs each check on the frozen `engine/tools/refcore`, unchanged, and compares its output with the stored CPython baseline. Standard library only. The rules are in `TESTING.md`; how an English text becomes tests is its §9.

## Where things are (4.5f)

The suite lives here, travelling with the engine, and runs from any checkout: every path in `checks.tsv` and the run definitions is relative to the repository root, which the tools find from their own place.

| Path | What |
|---|---|
| `run.py`, `checks.tsv`, `examples.tsv` | the runner, the checks, the examples and their three levels |
| `GAPS.md` | T's STOP-REPORTs: forms T wrote and ran where refcore and the engine differ outside a ruled family, held until ruled |
| `GOLDENS.md` | the prototype's goldens and 36 pins restated in tally 2 (DONE gate 2, issue #108): each one's text, refcore's figure against the prototype's, and what blocks the rest |
| `tools/` | `build_checks.py` (writes `checks.tsv`), `tracked_check.py`, `knobdiff.py`, `memcap.py` (the watchdog), `answers.py` (runs → answers for `compare.py`), `xref_check.py` (the codes L §8 and E §11 define against what the outputs print, K8-13: writes `map/codes.tsv`; `--check` in `run.py --self-test`; `--gate M0`) |
| `map/` | the tags, coverage, the halving test's knobs; `examples.py` builds and lints `examples.tsv`; `placement.tsv` (each check's milestone); `codes.tsv` (the codes' cross-reference, by `tools/xref_check.py`) |
| `cases/` | the case set (cases 1.1.1): stories, questions, keys, probe cards, `tools/compare.py` and the key scripts |
| `unit/probes/` | the probe texts |
| `integration/` | `households/` and `law/` (the one visible copy of the judged texts; session 5's writers copy from here), `cuts/` (the cut-down copies the household checks run on), `amendments/` (household X), `models/`, `models-1.0/`, `light/`, `blind/`, `new/` (4.5c's fill), `split/`, `recipes/`, `keys-extra/` |
| `load/` | `load.tsv` and `benchmarks.md`, the full-size runs for the engine, never run on refcore |
| `baselines/` | the stored outputs: `snap16-*/`, `e1-02-B16.out`, `4.5c/`, and `session5-visible/` (session 5's runs, a second witness) |
| `defs/` | the run lists `build_checks.py` reads |
| `hh4/` | household X's hand-checked vectors (`spec/smoke/HH4-*`) restated for milestone S: `vectors.tsv`, written and checked by the key script `hh4.py` from refcore's runs in `refcore/` (issue #16; `MILESTONE-S.md` row 4) |
| `kernel/` | milestone S's kernel vectors (issue #41; `MILESTONE-S.md` row 5): `vectors.tsv` lists them (a check of `checks.tsv`, or a fixture of its own), `vectors.py` writes refcore's whole output for each to `vectors.refcore` (`--check` in `run.py --self-test`), and the runner's `s_kernel` target holds the engine to it byte for byte; a fixture's `<name>.gen` file is a generated text's source (`@keys <prefix> <first> <last>@` for a key list, P36): `vectors.py` runs refcore on its expansion under `out/gen/`, the table stores the source, and `runner/src/kernel.rs` expands it before the engine runs (its test pins each expansion's digest) |
| `m0/` | milestone M0's vectors (issue #56; `MILESTONE-M0.md`): `vectors.py` runs refcore's `reprint` over every text set `checks.tsv` names and the format fixtures of `format/` (one per rule of L §2.4; `format/held.tsv` the ones held out, with why, their texts in `format/held/`), holds refcore to the round trip, and writes `format.refcore` (each canonical text by its digest) and `runner/src/m0_texts.rs`; it runs the load probe (`run <files> --ask zz_m0_probe`, `INTERFACE.md` §3) over the same sets, a refused scenario's own asks and row 1's 10 checks, ties each to the stored outputs, and writes `load.refcore`; `loadcheck.py` holds the spec's scan (`engine/spec/tools/loadcheck.py`, `--selftest` first) to refcore's load probe over the spec's 79 models, its household and X's 10 changes, and writes `loadcheck.refcore`, `loadcheck.tsv` and `runner/src/m0_spec_texts.rs`; each `--check` in `run.py --self-test`; the runner's `m0_format`, `m0_load` and `m0_loadcheck` targets hold the engine to them. `MILESTONE-M0.md` lists M0's eight rows with their tests and today's state, the message forms held and why, and the rulings that hold tests back (#72, #73, #75) |
| `properties/` | the laws (`PROPERTIES.md`): `gen.py` writes the kernel's seeded corpus `kernel.corpus` and `laws.py` proves its laws on refcore and writes `kernel.refcore` (milestone S, `s_properties`); `gen_m0.py` writes M0's `m0.corpus` (the format's and the loader's laws) and `laws_m0.py` proves them on refcore and writes `m0.refcore` (`--check` in `run.py --self-test`; `m0_properties`); `gen_m1.py` writes M1's `m1.corpus` (the walk's, the scenario's, the asks' and #98's laws) and `laws_m1.py` proves them on refcore and writes `m1.refcore` (`--check` in `run.py --self-test`, 8l; `m1_properties`) |
| `codes/` | the fixtures of the codes no check prints (DONE gate 5): `fixtures.tsv`, written by `codes.py` from refcore's runs, and their texts; `LOAD-PATH` first (issue #16), then tally 2.1.2's order of load's checks (`order/`, #23) and `LOAD-DEPTH`'s `ties by` term (`LOAD-DEPTH/`, #29), which only M0 reaches; then M0's fixture for each `LOAD-*` code no other output prints (issue #56): `m0.py` builds each from a family's base text (`m0/<family>/`) and one edit (`m0/edits.tsv`), runs it with the base beside it, and writes `m0.refcore` (`--check` in `run.py --self-test`); then a fixture for each `LOAD-*` message form no other output prints (G7, tier 2: `edits.tsv`'s `form` column, which `m0.py` checks), and in `m0/held.tsv` the ones held out, with why; the runner's `m0_codes` target holds the engine to them; then M1's (issue #108, chunk C4): `m1.py`, `m0.py` with the store m1, builds a fixture for each code and form of M1's constructs no other output prints from `m1/<family>/` and `m1/edits.tsv`, with each family's base and its answers (one run per ask, clean) as the stated twins, and #90's two on M0's families; it writes `m1.refcore` and `m1_invariant.refcore` (`ENGINE-INVARIANT`: refcore under `m1_fault.py`'s planted fault, `INTERFACE.md` §4), `--check` in `run.py --self-test`; `m1/held.tsv` holds `REQ-CLOCK` (GAPS.md); the runner's `m1_codes` target holds the engine to them |
| `held.tsv` | the gating vectors held "for #n": refcore's settled bytes the engine doesn't give yet, the work planned under issue n (tally 2.1.6: 53 for #97, the coordinator's pick P34, un-held by the pre-M1 run, whose builder groups G1a to G3 turn them green; one for #98, P39, where refcore is to be fixed at the next spec round and its bytes are stored as they are; the pre-M1 run's, likewise, where the engine follows the spec and refcore is fixed at M2's round: #100, #103, #104, #105, #106; and #107, where refcore and the engine agree and both follow the spec at that round); `runner/src/held.rs` reads it, and the gating targets (`s_kernel`, `m0_codes`, `m0_format`, `m0_load`, `m0_properties`) leave each out of their comparison; the stores and baselines stay as they are, so deleting a row un-holds it unchanged. `codes/m0/held.tsv` and `m0/format/held.tsv` are another kind: fixtures whose expected bytes aren't settled, kept out of their stores |
| `m1/` | milestone M1's vectors (issue #108; `MILESTONE-M1.md`): `vectors.tsv` lists them (a check of `checks.tsv`, or a fixture of its own) with the runner test that holds each and the builder group that turns it green (`for`); `vectors.py` runs plain refcore on each with `kernel/vectors.py`'s functions, ties each check to its stored baseline, and writes `vectors.refcore` (`--check` in `run.py --self-test`; `--times`, `--engine`, `--show`); `later/` holds #36's vector, moved from `later/`; `format/` the texts the format law holds (#109); `goldens/` the goldens' texts (`GOLDENS.md`); `carried/` the fixtures of the items the pre-M1 run left for M1, a folder per construct, and `carried/held.tsv` the forms held out with why (GAPS.md); `fixes/` the fixtures of the builder groups' review fixes, a folder per group (`g1/`, ...); the runner's `m1_checks`, `m1_format`, `m1_goldens`, `m1_carried` and `m1_fixes` targets hold the engine to them
| `m2/` | milestone M2's vectors (issue #167; `MILESTONE-M2.md`): `vectors.tsv` lists them (a check of `checks.tsv`, or a fixture of its own) with the runner test that holds each and the builder group that turns it green (`for`: `plan` until the builder's plan names one); `vectors.py` runs plain refcore on each with `kernel/vectors.py`'s functions, ties each check to its stored baseline, and writes `vectors.refcore` (`--check` in `run.py --self-test`, 8m; `--times`, `--engine`, `--show`); the runner's `m2_checks` target holds the engine to them |
| `later/` | vectors for a milestone not built yet (tally 2.1.6: #49's sample order for M7; #36's `inputs-of` over a balance moved to `m1/later/` at M1): `later.tsv` lists them with milestone, issue and rule; `later.py` stores refcore's exit and output beside each text (`<id>.refcore`, `--check`); no gating target reads them, and the milestone's T session moves them into its runner target |
| `reports/` | the measured tables of 4.5a, 4.5c and 4.5e, `adhoc.log` |

## How to run

```sh
python3 -I -B engine/tests/run.py --self-test                 # must pass before a real run
python3 -I -B engine/tests/run.py --jobs 3                    # every check, PyPy, fast, 90 s each
python3 -I -B engine/tests/run.py 'U:*' 'I:M:CB-0*'           # by id glob
python3 -I -B engine/tests/run.py --layer unit,interpreter --runtime both
python3 -I -B engine/tests/run.py --list 'I:B:*'              # what a selection holds
python3 -I -B engine/tests/run.py --tables                    # reports/before-after.md, over-budget.md
python3 -I -B engine/tests/map/examples.py                    # the examples' lint (also in --self-test)
```

- `--runtime pypy` (default), `cpython` or `both`. `both` runs each check on the two and also requires their outputs to be equal: the rule for any change to how the interpreter runs.
- `--interp fast` (the default since 4.5f), `refcore` or `both` (4.5e): `fast` runs `engine/tools/refcore_fast`, the memory layer in front of the same refcore; `both` requires equal outputs. Under `fast`, a check with no stored baseline runs on refcore by itself (a new output comes from refcore only, `TESTING.md` §5). `--verify N` (fast only) recomputes one in N remembered answers and stops on a difference (exit 70). Results go to `results/<runtime>+fast[+v<N>].tsv`. Use `--interp refcore` for new baselines and the exam.
- `--interp=engine` (T1, issue #5; `INTERFACE.md` §7): the engine's release binary `engine/target/release/tally` with the same command lines, cached by the binary's hash. An engine output never becomes a baseline: a check with no stored baseline is judged against refcore's cached run of it, never `ran`. A wrong exit code is an `error`. `R:all`, the key scripts and the load rows are listed as not run. Write it with `=`: at the repository root the guard reads a bare `engine` word as the folder. Every option takes `--opt=value`.
- **The watchdog:** `engine/tests/tools/memcap.py` stops any `refcore`, `refcore_fast` or engine (`target/release/tally run`) run past 8 GB (recorded as `killed`, logged in `out/memcap.log`). A run, `--self-test` and `--adhoc` refuse to start without it: `nohup python3 -I -B engine/tests/tools/memcap.py 8 5 > /dev/null 2>&1 &` (check with `pgrep -fl tests/tools/memcap`). It replaces session 5's `memcap.py` and 4.5e's `memcap_fast.py`.
- `--jobs N`, at most 4 (32 GB).
- `--budget S` (default 90, the user's rule since 4.5f; 60 before). A run past it is killed and recorded as `over`, and not rerun.
- `--force` reruns cached checks.
- Each call prints a table per layer (same, differ, over, killed, known fault, fixed, cached, not run, wall time). Results: `results/<runtime>.tsv`, the latest result per id; a differing output is kept in `out/<runtime>/`.
- **In git:** `python3 -I -B engine/tests/tools/tracked_check.py` lists any file `checks.tsv` names that git doesn't track (exit 1). `.prettierignore` keeps the formatter off `engine/tests/`: its files are exact records.

## The checks and their ids

`checks.tsv`, one row per check, built by `tools/build_checks.py` from the run definitions (read-only on them). Columns: `id`, `layer`, `cwd`, `files` (run order), `asks` (empty = all), `options`, `baseline`, `block`, `baseline_s`, `question`, `note`. **Ids are for life**: 4.5b's map and 4.5c join on them.

| Layer | id | From | Baseline |
|---|---|---|---|
| interpreter | `R:all` | `refcore test` | last line `1365 tests, 1365 passed, 0 failed` (tally 2.1.7 after P40-P41; 1356 after P36, 1345 before; 1251 on 2.1.6, 950 on 2.0) |
| unit | `U:<path under unit/probes, no .tally>` | `defs/probes.list` (session 4's names, `cases/probes/…`) | `snap16-probes/` |
| unit | `U:set/<first path>` | `probe-sets.list`, plus PTY-05 as `rerun.sh` runs it | `snap16-probe-sets/` |
| integration | `I:M:<model>` | `models.list` | `snap16-models/` |
| integration | `I:light/<name>`, `I:hardset/<path under hardset>`, `I:hardset/b2fix/division` | `light.list`, `b1.list`, `b2.list`, `rerun.sh` | `snap16-light/`, `-b1/`, `-b2/`, `-b2fix/` |
| integration | `I:<H>:<ask>`, H in A, B, D, E1, X; `@<run>` added when an ask is in several runs of a household | `house13.py` (A, B, D, X), `e1_rerun.sh` (E1) and `run5.py`'s E1-02-B run; **one check per ask** | that ask's blocks in `snap16-house/`, `snap16-e1/`, `e1-02-B16.out` |

- Household checks run on the inputs snap16 ran on (`integration/cuts/`, `integration/households/`), with the visible text changes of 4.5f (TC-1..TC-5, F-11) brought into every copy holding their lines; `tmp/session5/visible/` is retired.
- A row whose note starts `not-run:` is a run on `engine/tests/load/benchmarks.md`. It is listed, so the ids exist, and never run.
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

Every interpreter run gets 90 s (the user's rule, 4.5f). A check past it is `over`: a fault to fix (by the halving test, 4.5b, and the split, 4.5c), not a check to move. 60 s is the soft line: the table's `slow` column counts checks past it, and a check past it in 3 uncached runs in a row (`slow_runs` in `results/<variant>.tsv`; a cached result keeps the count, a run under 60 s resets it) is listed after the table as a candidate for a load row or a cut-down copy. `reports/over-budget.md` lists them with their files, asks and stored CPython seconds.

## Added in session 4.5c

- **`--adhoc [--cwd DIR] -- run <files> [--ask …]`:** one run of the frozen refcore on PyPy, 90 s, logged in `reports/adhoc.log`, not cached and not recorded in `results/`. For writers and the judge.
- **The split** (`TESTING.md` §4): capability copies under `integration/split/`, each with a `.knobs` file that `tools/knobdiff.py` checks; `integration/split/split.tsv` tells `build_checks.py` which checks to clone (`<id>@k<n>`). The modes are `twin`, `twin-cut` (adds `cmp=no-left-out`), `J`, `per-ask` (`<id>~<ask>@k<n>`) and `twin=<path>`.
- **The verdict `ran`:** a check with no baseline (no byte-equal judged twin). Its output is kept in `out/pypy/` for J to mark against the key.
- **Script checks** (`options` = `script`): a key script of `integration/keys-extra/`, run with the runtime's Python; `same` when it exits 0 and its last line ends `: 0 failed`. `files` lists the script and then the inputs it reads (hashed for the cache).
- **The `key` column** of `checks.tsv`: what stands behind the baseline (`TESTING.md` §6).
- **Retired checks:** `build_checks.py`'s `RETIRE` list prefixes a `not-run:` note. The row stays.
- **`load/load.tsv`:** the load layer, one row per full-size twin.
