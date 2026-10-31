# Milestone S: what must turn green

The checks the tracer slice must turn green, by id, from BUILD-ORDER § S ("Green") and its placement (`map/placement.tsv`, carried in from session 8b, checked by `map/placement.py` in `run.py --self-test`). Written by T1 (issue #5, 2026-10-31). The builder reads this page; T changes it.

## The list

| # | What | Ids | Run by | Gate | Today |
|---|---|---|---|---|---|
| 1 | X's tracer, against refcore's stored output | `I:X:konto_balance@b_r1`, `I:X:replay_september@b_r1`, `I:X:replay_september@b_r1s` | `run.py --interp=engine 'I:X:*@b_r1' 'I:X:*@b_r1s'` | 3, 10 | `error` on the stub (exit 69) |
| 2 | refcore's own tests stay green | `R:all` | `run.py` on refcore (the engine runs nothing in it) | — | green |
| 3 | the kernel's properties | `s_determinism_files_and_sections_in_any_order`, `s_determinism_across_runs`, `s_money_conserved`, `s_no_foresight_past_the_cut`, `s_no_foresight_in_a_decision`, `s_no_foresight_through_the_written_projection`, `s_a_hole_never_becomes_zero` (`PROPERTIES.md`) | `cargo nextest run -p tally-runner --test s_properties` | 6 | 7 of 7 fail on the stub |
| 4 | X's base asks against the hand-checked vectors (`spec/smoke/HH4-*`), restated in tally 2's output form | not written yet | T's household vectors | 3 | **open: a follow-up issue** (T's, milestone S); issue #5 listed it, the T1 brief did not |
| 5 | the mutation score, at least 90 % on the kernel as S builds it | cargo-mutants over `tools/kernel.toml`'s globs, T's tests only (R-84) | nightly `mutants`, `tools/mutants-survivors` | 8 | the kernel's globs are set at S by the rules role (SCAFFOLD §4); the score counts T's gated tests, so row 3 must be gated first |
| 6 | the budget rows S measures | P8-14 (X's replay at 1×) and P8-M1..P8-M4 | `tools/perf row`, `tools/perf counts` (CI: P8-10..P8-15) | 12 | proposed until S; `perf-digests.tsv` (T1) now gives each check row's expected output, so a fast wrong answer fails |

**Not T's to test, and why:**
- **The event walk equals the step-every-day walk on every S check** (BUILD-ORDER § S): the step-every-day walk is the builder's own oracle inside the engine, and `INTERFACE.md` has no switch to choose a walk from outside. If the user wants it as T's evidence, it needs a switch in the interface first. That is a later T item, not added here.
- **The order question** (`INTERFACE.md` §5) touches none of rows 1–3: X's tracer runs are compared in the household form, which drops warning lines, and the property realities raise no `WARN-UNREAD`.

## Keeping master green

T's tests are written before the engine they test, so they fail until their milestone is built. If they joined `check full` (CI runs the runner with `--all-features`), every pull request would be red until S. **The proposal, for gate G-T:**

1. **Every T test belongs to one milestone.** A check by `map/placement.tsv`; a Rust test by its target, one per milestone: `s_properties` now, then `m0_…`, `m1_…`. Each such target has `test = false`, so a plain `cargo nextest run` never runs it.
2. **A protected marker says which milestones gate:** `engine/tools/milestones.toml`, `gating = []` today. It is a rule file (RULES.sha256), written only by the rules role. The rules role adds a milestone in the commit after the gate session tags it.
3. **Gating at the tag, not at the start:** within a milestone, the builder's pull requests land while its checks are still partly red. Once the milestone is tagged, its tests can never go red again.
4. **What runs where** (the rules role's side):
   - `check full` and CI run every gating milestone's targets (`--test s_properties` …) as failing steps, beside the default targets.
   - A report step runs the next milestone's targets and prints a line such as `S: 3/7 green`, never failing.
   - `check fast` prints the same report, so the builder sees the red work list.
   - The mutants baseline (gate 8) includes the gating targets.
5. **The Python side** (T's, once the marker exists): `run.py --interp=engine --gating` runs the checks placed in gating milestones, and CI runs it once the build role's binary is built there.

Until the marker exists, `s_properties` runs only by name, and nothing T adds turns `check full` red. The CI and `check.py` side is the rules role's: follow-up issue (filed at G-T).
