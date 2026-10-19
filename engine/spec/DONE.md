# Tally engine: done when

The acceptance contract for the Rust engine (step B), written before the build. The engine is done when every gate below is green on one tagged commit, as run by a gate session, never by the builder. It is the user's contract: a gate changes only with the user's approval, logged here with its date.

**Status:** approved by the user, 2026-10-19.

## The rules the gates rest on

1. **The spec is frozen as `tally 1.0`** in `engine/spec/` of this repository, the commit tagged `tally-1.0` (`LANGUAGE.md`, `ENGINE.md`, `OPERATIONS.md`, `PERFORMANCE.md`, `BOUNDS.md`, `DECISIONS.md`, `BUILD-ORDER.md`, `coverage.md`, and the models, expected files and amendments they cite). After the freeze, a change to it needs three things: a failing test or a logged contradiction, a row in the gap log, and a patch that names both. Each patch raises the version (`tally 1.0.1`, …), and every gate is re-run on it.
2. **The tests are not the builder's.** Step T writes them from the frozen spec and the answer keys. A guard refuses every edit to tests, vectors and expected values from the build session. When a test looks wrong, the builder stops and reports it; one hand run decides whether the model, the spec, the test or the code is wrong, and the decision is logged.
3. **No silent fix.** Where the spec is unclear, the spec is fixed first (rule 1), never only the code.
4. **Every gate is evidence, not a claim.** Each gate's evidence is a CI run on the tagged commit, whose log is kept.

## Gates

| # | Gate | Green when | Evidence |
|---|---|---|---|
| 1 | **The 79 cases** | Every ask of every model in `models/` gives the figure its `expected/` file names, under every option of its axes. The six declared bounds of the three bound cases (MP-15's fewest transfers, LH-17 step 6, and CH-03's `jss_obs`, `jss_unseeded`, `logs_sample` (step 14) and `np_stream`) print their bound, as LANGUAGE §5.17 says | T's case vectors |
| 2 | **Goldens and pins** | G1–G10 and the 36 pins (`tmp/verification/goldens.md`) hold on the canonical projection (CHG-14) | T's golden vectors |
| 3 | **The household and its 7 changes** | The household's asks, ledger, registers and holes match the hand-checked vectors (REPORT §13, `smoke/HH4-*`). Each of the 7 changes (`amendments/changes/`) gives the outputs predicted for it, by hand, before the build | T's household vectors; the predicted outputs of the 7 changes |
| 4 | **Hindcast H0–H9** | Each fixture's planted cause lands in the ledger line its answer key names (`tmp/verification/hindcast.md`) | T's hindcast fixtures |
| 5 | **The 87 codes** | Each of the 79 refusal and hole codes (LANGUAGE §8, ENGINE §11, OPERATIONS §10) has a fixture that gives exactly that code and message, and nothing else. Each of the 8 warnings has one too. `ENGINE-INVARIANT` goes through a fault-injection hook that is off in release builds | T's refusal fixtures |
| 6 | **Properties** | Under property tests: determinism (equal result hashes with 1 and 6 workers, and across runs); money conserved (ENGINE §9.1); net worth moves only at the boundary, a price or an anchor (§9.2); a scenario changes nothing before its `from` (§9.3); asks move no money (§9.4); no foresight (a cut never reads a version known after it, and changing a draw's value, a state pay's later result or any row after the look-ahead's window never changes a state pay's decision before it, C-13); a hole never becomes 0; `format(parse(x)) = x`, and `format` is idempotent | T's property specs |
| 7 | **Monte Carlo** | LH-17 run with draws (`montecarlo/LH-17-mc.tally`) matches its exact closed form (Yor 1992 at λ = μ, `montecarlo/LH-17-mc.md`) within the oracle tolerance (4·SE plus the step bias plus the oracle's own error, ENGINE §8). The household's path statistics (`montecarlo/household-bands.md`) give their predicted values within the same tolerance. A seeded run replays byte for byte; one path replays alone to the same figures. The `share` verdict gives the predicted `sayable` / `not sayable` with its reasons | T's statistical tests; `montecarlo/` |
| 8 | **Mutation score** | At least 90 % of mutants killed in the kernel: the numbers and rounding, the read rule, the walk and `plan_low`, booking and the ledger. Every surviving mutant is listed, each with the reason it is equivalent | cargo-mutants report |
| 9 | **Fuzzing** | The parser, the formatter and the loader each run 8 hours without a crash, a hang or an input that gets no code. Every crash found earlier is a regression test | cargo-fuzz logs |
| 10 | **Differential testing** | The engine and the Python reference core (written blind from the spec) agree byte for byte on 10 000 random realities from the generator. Every disagreement found earlier is a regression case, with its decision logged | the differential run's log |
| 11 | **Held-out** | The 20 working cases are green, modelled blind and run by the gate session. The final 5 are then run once, and their result is reported as it is, with any gap in generic terms. The final 5 don't block "done": the user reads the result and decides | the gate session's report |
| 12 | **Performance** | Every budget of `PERFORMANCE.md` §8 is met at the size its row states (the cold run at 10× the prototype's instances), on the reference machine. Every optimisation leaves every output of gates 1 to 7 byte-identical | criterion reports; the gate 1–7 hashes before and after |

## Milestone gates on the way

Each milestone of `BUILD-ORDER.md` (S, then M0 to M9) is tagged when its named tests are green and every earlier milestone's stay green, with the properties, the mutation score and the budget rows it touches at 1× size. The tracer slice S is gated by the household's vectors, the kernel's properties and the mutation score.

## What "done" doesn't promise

- **Laws unlike any tested case.** The final 5 measure this; nothing proves it.
- **Modelling the household's real data well.** That is the product check, outside the engine and its tests.

## Approvals

| Date | Change | By |
|---|---|---|
| 2026-10-19 | Approved as drafted: 12 gates; fuzzing 8 hours per target; 10 000 random realities; mutation score ≥ 90 % on the kernel; the final 5 reported, not blocking | the user |
| 2026-10-19 | tally 1.0.1 (`CHANGES.md`, D-1 to D-5): gate 1 names the six declared bounds; gate 6's no foresight includes the look-ahead's (C-13); gate 7 tests LH-17 against its exact closed form, names `montecarlo/` and says path statistics; gate 12 takes each budget at the size its row states | the user |
