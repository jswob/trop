# Testing Tally: the rules

Session 4.5c, 2026-10-29. One page of rules for the test set and for everything added to it later. The runner is `run.py` (`README.md`), the checks are `checks.tsv`, the map is `map/`, the load list is `load.tsv`. The prior art behind each rule is in `tmp/active/session4_5-research.md` §3.

## 1. The layers

| Layer | What | Budget | Runs |
|---|---|---|---|
| **interpreter** | refcore's own 950 tests (`R:all`) | 60 s | every pass |
| **unit** | the probes: one card per catalogue edge (and pair cards for two edges that interact), seconds each | 60 s each; the layer about 2 min | every pass |
| **integration** | the household questions (one check per ask), the 79 old models, the light twins, the blind texts, the amendments household, the key scripts of `keys-extra/` | 60 s each; the layer under 10 min on 4 jobs | every pass |
| **acceptance** | the holdout: blind, exam only, never split into visible probes | the exam's | session 5 only |
| **load** | the full-size twins in `load.tsv` | none on refcore | the engine only, never on refcore |

A check past its 60 s budget is a fault to fix (by the halving test and the split below), not a check to move. *(Bazel's test sizes.)*

## 2. The one catalogue

The edge list `tmp/cases/boundaries.md` is the only list of capabilities. Every check names the edges it uses in `map/tags.tsv`: from a card's `edge:`, a key's `edges:` (a question's aim), a model's story header, or a `reach` tag that cites the derivation step passing the edge. Coverage is measured by machine (`map/coverage.py`): every edge has a unit check, every edge but the SURF writing rules meets integration, every ★ edge meets two integration checks, and the goal is that every pair of axes that can interact meets in some integration check. *(test262's feature tags; NIST pairwise coverage: most failures need one or two interacting parameters.)*

## 3. The breakdown rule for anything new

Every new feature, law, household or bug is broken down three ways:
1. **Its capabilities** go into the catalogue: a new edge gets a probe card, keyed by `keys_probes.py`. Two edges that interact and meet nowhere else get a pair card.
2. **Its combinations** go into integration: a question in a household (a scenario or a pure ask, so no existing key moves), aimed at the axis pairs `coverage.py --unmet` lists.
3. **Its scale** goes into a load twin: the full size is a row of `load.tsv`, and the capability copy (§4) is the check.

## 4. The halving test and the split

A number that can shrink without dropping a rule is **scale** (paths, the run end after the question's last date, search steps, asks per run, cohorts, payout periods); anything else is **capability** (the asked dates, the span start that holds the history, a branch per option). A heavy check gets a capability copy that differs from its original **only in its knob lines** (`knobdiff.py` checks this). The copy lives under `texts/split/` and is a row of `texts/split/split.tsv`. It passes in one of two ways:
- **byte-equal to a judged twin:** the original's stored output, or a session-judged run. A run-end cut may differ only in its `  left out:` lines (`cmp=no-left-out`), the run end's own list of what falls past it;
- **marked by the judge** against the key (verdict `ran`), where the outputs can't agree by construction.

If the copy is still over 60 s at every knob, the user decides: a load row with its measured number, or more work. *(TPC-H's one generator with a scale knob and answers per scale; small scope: a program that is wrong is almost always wrong on a small case.)*

## 5. Both ways for any interpreter change

refcore stays frozen and is the judge. A faster way to run it (PyPy today; any later layer behind a switch) is accepted only if the 950 tests and every stored output match the frozen interpreter byte for byte: `run.py --runtime both`. *(Cedar and ShardStore: an obviously right reference model, fast enough on small inputs to judge many of them.)*

## 6. The cache, the known faults and the labels

- **The cache:** a run is keyed by the hash of the interpreter, its runtime and version, every input file's path and bytes, the asks and the options. An unchanged check is never run again; `--force` reruns. *(Bazel's content-addressed test cache.)*
- **Known faults** (`known-faults.md`): each gets a guarded check that still runs its stored form, and an `~alone` check that reads `known` until the fault is fixed and `fixed` after. A `fixed` is a signal to retire the fault, not a failure.
- **The key column** of `checks.tsv` says what stands behind a baseline when it isn't a key script or a published expected file: `hand arithmetic, weaker key` (the amendments household), `interpreter-keyed` (X's 12 cells), `no key by design` (the SURF cards), `no key: a blind text`, `no byte-equal twin: J marks`. A check with a weaker key catches changes. It isn't proof of correctness.
- **Ids are for life:** checks are added through `build_checks.py` and never renumbered or deleted. A retired check keeps its row with a `not-run: retired …` note saying why and where its load twin is.

## 7. The small-world rule for random figures

A random figure from a run cut in paths passes if it lies within 4 combined standard errors of the key's reference: `|value − r| ≤ 4 × √(se² + se_r²)` (`compare.py --small`). The key keeps its full-n reference and its own se. The precision bar `p` waits for the engine at full size. A quantile prints no se, so at a cut path count it is unjudged until the engine and is listed in `load.tsv`. *(Statistical oracles; session 4 judged D-10 this way at 2 000 paths.)*

## 8. What a pass costs

The quick set is the interpreter, unit and integration layers run cold on 4 jobs: see `after.md` for the measured time. Only changed checks run after that (the cache). A step that costs more than it finds is cut: no rerun of a settled check, and no run of a load row on refcore.
