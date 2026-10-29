# Testing Tally: the rules

Session 4.5c, 2026-10-29. One page of rules for the test set and for everything added to it later. The runner is `run.py` (`README.md`), the checks are `checks.tsv`, the map is `map/`, the load list is `load/load.tsv`, all under `engine/tests/` since 4.5f. The prior art behind each rule is in `tmp/active/session4_5-research.md` §3.

## 1. The layers

| Layer | What | Budget | Runs |
|---|---|---|---|
| **interpreter** | refcore's own 950 tests (`R:all`) | 90 s | every pass |
| **unit** | the probes: one card per catalogue edge (and pair cards for two edges that interact), seconds each | 90 s each; the layer about 2 min | every pass |
| **integration** | the household questions (one check per ask), the 79 old models, the light twins, the blind texts, the amendments household, the key scripts of `keys-extra/` | 90 s each; the layer under 10 min on 4 jobs | every pass |
| **acceptance** | the holdout: blind, exam only, never split into visible probes | the exam's | session 5 only |
| **load** | the full-size twins in `load.tsv` | none on refcore | the engine only, never on refcore |

Every run gets 90 s, the hard budget; a check past it is `over`, a fault to fix (by the halving test and the split below), not a check to move. 60 s is the soft line: a check past it in 3 uncached runs in a row is listed after the run's table as a **candidate** for a load row or a cut-down copy (`results/<variant>.tsv`, column `slow_runs`), for the user to decide. (4.5f, the user's rule: 60 s was the hard budget until then, and checks at 50–57 s on a quiet machine went over it under load.) *(Bazel's test sizes.)*

## 2. The one catalogue

The edge list `engine/tests/cases/boundaries.md` is the only list of capabilities. Every check names the edges it uses in `map/tags.tsv`: from a card's `edge:`, a key's `edges:` (a question's aim), a model's story header, or a `reach` tag that cites the derivation step passing the edge. Coverage is measured by machine (`map/coverage.py`): every edge has a unit check, every edge but the SURF writing rules meets integration, every ★ edge meets two integration checks, and the goal is that every pair of axes that can interact meets in some integration check. *(test262's feature tags; NIST pairwise coverage: most failures need one or two interacting parameters.)*

## 3. The breakdown rule for anything new

Every new feature, law, household or bug is broken down three ways:
1. **Its capabilities** go into the catalogue: a new edge gets a probe card, keyed by `keys_probes.py`. Two edges that interact and meet nowhere else get a pair card.
2. **Its combinations** go into integration: a question in a household (a scenario or a pure ask, so no existing key moves), aimed at the axis pairs `coverage.py --unmet` lists.
3. **Its scale** goes into a load twin: the full size is a row of `load/load.tsv`, and the capability copy (§4) is the check.

§9 gives the recipe from an English text, and `examples.tsv` records it per example.

## 4. The halving test and the split

A number that can shrink without dropping a rule is **scale** (paths, the run end after the question's last date, search steps, asks per run, cohorts, payout periods); anything else is **capability** (the asked dates, the span start that holds the history, a branch per option). A heavy check gets a capability copy that differs from its original **only in its knob lines** (`knobdiff.py` checks this). The copy lives under `integration/split/` and is a row of `integration/split/split.tsv`. It passes in one of two ways:
- **byte-equal to a judged twin:** the original's stored output, or a session-judged run. A run-end cut may differ only in its `  left out:` lines (`cmp=no-left-out`), the run end's own list of what falls past it;
- **marked by the judge** against the key (verdict `ran`), where the outputs can't agree by construction.

If the copy is still a candidate (over 60 s) at every knob, the user decides: a load row with its measured number, or more work. *(TPC-H's one generator with a scale knob and answers per scale; small scope: a program that is wrong is almost always wrong on a small case.)*

## 5. Both ways for any interpreter change

refcore stays frozen and is the judge. A faster way to run it (PyPy today; any later layer behind a switch) is accepted only if the 950 tests and every stored output match the frozen interpreter byte for byte: `run.py --runtime both` for a runtime, `run.py --interp both` for the memory layer `engine/tools/refcore_fast` (4.5e), and `--verify 50` on the heavy checks after any change to the layer. The fast side serves routine passes over checks with a stored baseline; a new baseline, a `ran` check J marks, and the exam run on refcore. *(Cedar and ShardStore: an obviously right reference model, fast enough on small inputs to judge many of them.)*

## 6. The cache, the known faults and the labels

- **The cache:** a run is keyed by the hash of the interpreter, its runtime and version, every input file's path and bytes, the asks and the options. An unchanged check is never run again; `--force` reruns. *(Bazel's content-addressed test cache.)*
- **Known faults** (`known-faults.md`): each gets a guarded check that still runs its stored form, and an `~alone` check that reads `known` until the fault is fixed and `fixed` after. A `fixed` is a signal to retire the fault, not a failure.
- **The key column** of `checks.tsv` says what stands behind a baseline when it isn't a key script or a published expected file: `hand arithmetic, weaker key` (the amendments household), `interpreter-keyed` (X's 12 cells), `no key by design` (the SURF cards), `no key: a blind text`, `no byte-equal twin: J marks`. A check with a weaker key catches changes. It isn't proof of correctness.
- **Ids are for life:** checks are added through `build_checks.py` and never renumbered or deleted. A retired check keeps its row with a `not-run: retired …` note saying why and where its load twin is.

## 7. The small-world rule for random figures

A random figure from a run cut in paths passes if it lies within 4 combined standard errors of the key's reference: `|value − r| ≤ 4 × √(se² + se_r²)` (`compare.py --small`). The key keeps its full-n reference and its own se. The precision bar `p` waits for the engine at full size. A quantile prints no se, so at a cut path count it is unjudged until the engine and is listed in `load.tsv`. *(Statistical oracles; session 4 judged D-10 this way at 2 000 paths.)*

## 8. What a pass costs

The quick set is the interpreter, unit and integration layers run cold: see `reports/after.md` and `reports/after-fast.md` for the measured times. Only changed checks run after that (the cache). A step that costs more than it finds is cut: no rerun of a settled check, and no run of a load row on refcore.

## 9. From English to tests: the recipe

Session 4.5f. Any English description (a household's story, a law, a bug report) becomes tests the same way. A story gives **rules**, the rules give **examples**, and what the text leaves open becomes **questions** to its writer, never a guess. *(Example Mapping, Wynne 2015: a story's rules, examples and questions on cards; Specification by Example, Adzic 2011: key examples, refined, kept as the living documentation.)*

| From the English | Goes to |
|---|---|
| each rule the text states | a catalogue edge with its probe card: **unit** (§2, §3.1) |
| each question a reader would ask of the story | an integration check, an ask in its household (§3.2), aimed at `map/coverage.py --unmet` |
| each size (people, years, paths, rows) | the halving test (§4): the capability copy is a check, the full size a **load** row |
| what the text leaves open | a stated hole in the answer, or a case item for the case set's writer, never a guess |

**The record:** `examples.tsv` has one row per example (each household, law text, old model, light twin and blind text) with its story and its three entries: the unit edges, the integration checks, the load rows, or `none: <reason>` for any of them. `map/examples.py --build` back-fills it from `map/tags.tsv`, `checks.tsv` and `load/load.tsv`; nobody edits it by hand. `map/examples.py` is its lint, and runs in `run.py --self-test`. It fails on a missing entry, an id, edge or load row that doesn't exist, an edge with no unit check, a retired check whose example has no load row, an integration check in no example, and a unit check whose edges no example meets (the probe-only SURF edges of §2 apart).

**Worked once, on household E1** (the smallest that meets all three levels: `cases/E1/story.md`, 750 words, 3 questions):

| The story says | Rule or question | Level | Where it is |
|---|---|---|---|
| "Households A, B and D … are one family. E1 reads their three dossiers together" | rule: one model joins three households' texts | unit | PTY-05 · joins across households (`U:pty/PTY-05/*`) |
| "Zbigniew … died 2024-05-17, leaving no will"; "the certificate … final: Halina, Piotr, Anna, Marta and, in Adam's place, Kuba" | rule: succession down the tree, a grandchild in a dead parent's place | unit | PTY-03 · succession (`U:pty/PTY-03`) |
| "Halina pays the whole sum into the notary's deposit … the notary pays out to Piotr, Ania, Marta and Kuba" | rule: money moves and is conserved, one event with several legs | unit | FLOW-01, FLOW-02 |
| "Ania's and Piotr's payouts reach their savings accounts that day, as their records show" | rule: a stated event matched against a record | unit | REC-04 · matching and certainty |
| "2025-07-01 · known 2025-06-24" | rule: a fact has its own known date | unit | REC-06 · clocks and knowledge dates |
| E1-01: "What did Halina pay into the notary's deposit … What was left in the deposit?" | question | integration | `I:E1:e1_01_deposit`, `…_payouts`, `…_in_is_out` and 5 more |
| E1-02: "what was each of us worth … Who among us owed whom" | question | integration | `I:E1:e1_02_people`, `…_owed`, and the B-side asks `I:E1:tx2_*` (12 checks) |
| E1-03: "On what day did the inheritance tax … arise, and by what day did an heir have to report" | question | integration | `I:E1:e1_03_dates`, `…_reported`, `…_not_reported` and 6 more (LAW-11, LAW-12, TIME-05) |
| "a walk of B takes about 3 minutes" (the joint model over B's full span) | size: B's span and walks | load | `L:8`; the capability copy is the `cdB` cut with B stubbed, which the 29 `I:E1:*` checks run on |
| "Nothing in the family's papers says whether any other heir reported" | open | answered under each reading | E1-03's key gives each other heir's tax by case, reported in time or not (`I:E1:e1_03_reported`, `…_not_reported`) |
| "nobody in A, B or D keeps a record of Marta's or Kuba's account" | open | a stated hole | E1-01's key refuses Marta's and Jakub's arrival day, naming the missing record |

E1's row of `examples.tsv`: 14 edges (TIME-05, PTY-03, PTY-05, OWN-02..05, FLOW-01, FLOW-02, LAW-11, LAW-12, REC-04, REC-06, ASK-02), the checks `I:E1:*`, the load row `L:8`.
