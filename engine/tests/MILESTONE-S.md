# Milestone S: what must turn green

The checks the tracer slice must turn green, by id, from BUILD-ORDER § S ("Green") and its placement (`map/placement.tsv`, carried in from session 8b, checked by `map/placement.py` in `run.py --self-test`). Written by T1 (issue #5, 2026-10-31). The builder reads this page; T changes it.

## The list

| # | What | Ids | Run by | Gate | Today |
|---|---|---|---|---|---|
| 1 | X's tracer, against refcore's stored output | `I:X:konto_balance@b_r1`, `I:X:replay_september@b_r1`, `I:X:replay_september@b_r1s` | `run.py --interp=engine 'I:X:*@b_r1' 'I:X:*@b_r1s'` | 3, 10 | `error` on the stub (exit 69) |
| 2 | refcore's own tests stay green | `R:all` | `run.py` on refcore (the engine runs nothing in it) | — | green: 1124 tests (tally 2.1.1) |
| 3 | the kernel's properties | `s_determinism_files_and_sections_in_any_order`, `s_determinism_across_runs`, `s_money_conserved`, `s_no_foresight_past_the_cut`, `s_no_foresight_in_a_decision`, `s_no_foresight_through_the_written_projection`, `s_a_hole_never_becomes_zero` (`PROPERTIES.md`) | `cargo nextest run -p tally-runner --test s_properties` | 6 | 7 of 7 fail on the stub |
| 4 | X's base asks against the hand-checked vectors (`spec/smoke/HH4-*`), restated in tally 2.1.1's output form | `s_hh4_konto_balance_at_b_r1`, `s_hh4_replay_september_at_b_r1`, `s_hh4_replay_september_at_b_r1s`: the 75 vectors of `hh4/vectors.tsv` on row 1's three checks | `cargo nextest run -p tally-runner --test s_household` (in process, `INTERFACE.md` §4) | 3 | 3 of 3 fail on the stub; on refcore 2.1.1's runs every vector holds (`hh4/hh4.py`, `tally-runner`'s own tests) |
| 5 | the mutation score, at least 90 % on the kernel as S builds it | cargo-mutants over `tools/kernel.toml`'s globs, T's tests only (R-84) | nightly `mutants`, `tools/mutants-survivors` | 8 | the kernel's globs are set at S by the rules role (SCAFFOLD §4); the score counts T's gated tests, so row 3 must be gated first |
| 6 | the budget rows S measures | P8-14 (X's replay at 1×) and P8-M1..P8-M4 | `tools/perf row`, `tools/perf counts` (CI: P8-10..P8-15) | 12 | proposed until S; `perf-digests.tsv` (T1) now gives each check row's expected output, so a fast wrong answer fails |

**Row 4, the household vectors** (issue #16, 2026-11-01). A vector is a line refcore prints for X's base in one of row 1's checks, with the figures HH4's two hand runs write on it, each with the HH4 line it stands on, and the rule it checks: konto_balance at 2026-10-31 (the one date `b_r1` asks) with its register's anchor and cut; replay_september's ledger line by line, its groups in order, and the two instances HH4 says print nothing; the two `WARN-NEGATIVE` lines. `hh4/hh4.py` picks each line from refcore's stored runs (`hh4/refcore/`), reads its figures off it and checks each against HH4's text, so nothing is worked by hand (`--self-test`: each kind of disagreement caught when planted). On refcore 2.1.1 no figure disagrees with HH4. The rest of konto_balance's register (`assumed:`, `unfulfilled:`, `left out:`, `held:`) has no HH4 figure, and row 1 holds it byte for byte. HH4's other figures aren't S's: `wife_net`, `social_and_health` and `pit37` are M1's `@c*_fast` checks, the long-horizon asks (`explain_room`, `ike_left`, `net_worth`, `taxes_paid` and konto_balance's later dates, on `cut11/c1_r2`) are load rows' runs, placed at M9, and no check asks `pit28` or `health_settlement`.

**Not T's to test, and why:**
- **The event walk equals the step-every-day walk on every S check** (BUILD-ORDER § S): the step-every-day walk is the builder's own oracle inside the engine, and `INTERFACE.md` has no switch to choose a walk from outside. If the user wants it as T's evidence, it needs a switch in the interface first. That is a later T item, not added here.

**The order question** (`INTERFACE.md` §5) is settled by tally 2.1.1 (issue #13): warnings print sorted by code, then text. X's runs print two `WARN-NEGATIVE` lines, already in that order; the 29 whole-output baselines whose order changed were re-run on refcore 2.1.1 and rewritten (issue #16: only their `WARN-*` lines' order differs).

## Keeping master green

T's tests are written before the engine they test, so they fail until their milestone is built. If they joined `check full` (CI runs the runner with `--all-features`), every pull request would be red until S. **As approved at gate G-T, and wired by issue #14:**

1. **Every T test belongs to one milestone.** A check by `map/placement.tsv`; a Rust test by its target, named for its milestone: S's `s_properties` and `s_household`, M0's `m0_codes` (the fixtures of the codes no check prints, `codes/`), then `m1_…`. Each such target has `test = false`, so a plain `cargo nextest run` never runs it.
2. **A protected marker says which milestones gate:** `engine/tools/milestones.toml`, `gating = []` today. It is a rule file (RULES.sha256), written only by the rules role. The rules role adds a milestone in the commit after the gate session tags it.
3. **Gating at the tag, not at the start:** within a milestone, the builder's pull requests land while its checks are still partly red. Once the milestone is tagged, its tests can never go red again.
4. **What runs where** (the rules role's side):
   - `check full` and CI run every gating milestone's targets (`--test s_properties` …) as failing steps, beside the default targets.
   - A report step runs the next milestone's targets and prints a line such as `S: 3/7 green`, never failing.
   - `check fast` prints the same report, so the builder sees the red work list.
   - The mutants baseline (gate 8) includes the gating targets.
5. **The Python side** (T's, not written yet): `run.py --interp=engine --gating` runs the checks placed in gating milestones, and CI runs it once the build role's binary is built there.

The marker exists (`engine/tools/milestones.toml`, issue #14), with `gating = []` until S is tagged: S's targets run only as `check`'s report (`milestone-report: S: 0/10 green` on the stub), and nothing T adds turns `check full` red. **Approved by the user at G-T, 2026-10-31.**
