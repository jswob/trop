# Session 8c, step 2: branch `engine` merged into the scaffold

The user ran `git -C /Users/jswob/projects/trop-engine-scaffold merge engine` in their own shell on 2026-10-31 at 07:59. In the `rules` role the guard refuses it, because `engine` is both the branch and a folder holding `engine/spec/DONE.md`. Merge base `6f5e676`; `engine` at `4e57229`; scaffold at `eac9062` (phase 1's `db0262e` plus A-1).

## Text conflicts

| File | Resolution |
|---|---|
| `.prettierignore` | Both sides' lines kept. From `engine`: the comment now covers the reference interpreter, plus `engine/tools/refcore/` and `engine/tests/`. From the scaffold: `engine/target/`, `engine/fuzz/target/`, `engine/mutants.out/`, `engine/mutants.out.old/`. |

## Meaning-level conflicts

| # | What | Why it conflicts | Resolution |
|---|---|---|---|
| M-1 | `engine/spec/VERSION` is now `tally-2.1`, but the spec differs from the tag `tally-2.1` in six files | Session 8b's G2a amendment (`c62550a`: PERFORMANCE, BOUNDS, BUILD-ORDER, STALE, DONE, SHA256SUMS) landed after the tag and, as G2a decided, kept `VERSION`. check-frozen compared the spec with the tag, so it would be red forever. Moving the tag is the user's call and would rewrite a published version. | New rule file `engine/tools/frozen.toml`: `tag = "tally-2.1"`, `pin = c62550a…` (the full sha), `approved = "G2a, 2026-10-30 …"`. check-frozen now checks that the tag equals `spec/VERSION`, that the pin descends from the tag, and that the spec and the oracle match the pin byte for byte (no diff, no untracked file). A hash pins the bytes as firmly as a tag. A later amendment or version moves the pin through R-112. |
| M-2 | `engine/tools/refcore/` (1,147 files), `refcore_fast/` (3) and `engine/tools/SHA256SUMS` land under the guard's rule glob `engine/tools/*` | They would become rule files: writable only in the `rules` role, and 1,151 entries missing from `RULES.sha256`. But refcore is the spec's executable twin, owned by the spec side (7c's writer fixes), and it has its own checksums (`sums.py`). | A new guard area, **the oracle** (`ORACLE_GLOBS`): written only in the `spec` role, refused to `build`, `rules`, `tests` and app work, and left out of the rule files and `RULES.sha256`. The `ANCHORS` gain `refcore/x`, `refcore_fast/x` and `SHA256SUMS`, so `rm -rf engine/tools` is refused to every role. check-frozen verifies `tools/SHA256SUMS` over `refcore/` (changed, missing, unlisted), and the pin covers refcore, refcore_fast and SHA256SUMS. |
| M-3 | `refcore_fast/` has no checksum line | `sums.py` lists refcore only. | The pin covers it (`git diff` against `c62550a`), and the guard protects it. Adding it to `SHA256SUMS` is the spec role's choice; logged as a limit. |
| M-4 | `engine/tests/` now holds T's Python suite beside the scaffold's `engine/tests/runner/` (tally-runner) | No file clash. The suite's `.gitignore` names `cache/`, `out/`, `results/`, `__pycache__/`. Once T writes `TESTS.sha256`, check-frozen would report every run output as unlisted. | check-frozen skips `cache/`, `out/` and `results/` when it verifies `TESTS.sha256`. The guard needed no change: `engine/tests/**` was already the tests role's only. T places the runner, `INTERFACE.md` and `TESTS.sha256`. |
| M-5 | The rules-selftest's scratch repo has one commit, tagged with `VERSION` | The pin names a commit only the real history has. | `rules_selftest.py` `pin_the_tag()`: the scratch's `frozen.toml` drops `pin` (so the pin is the tag), and its `RULES.sha256` line for `frozen.toml` follows. The other lines stay as they are, so the clean run still proves every other checksum. |
| M-6 | CI | `engine/**` already triggers the workflow, and `fetch-depth: 0` plus `fetch-tags: true` bring the tag and the pinned commit. | No change at the merge. The skill, the reviewer and `engine/CLAUDE.md` join the triggers with F-23 in step 3. |
| M-7 | Root `package.json` and the `spec-loadcheck` step | `engine` didn't touch `package.json`, but tally 2's `spec/tools/loadcheck.py` has no default run: with no path it prints its usage and exits 2. The first `check full` on the merge was red at that one step (23 of 24 green). | `check.py`'s `spec-loadcheck` passes the 79 models and `amendments/household` (BUILD-ORDER l.122's list; the changes are Markdown, which tally 2's scan doesn't read): 0 refusals in 80 realities. `engine:spec-check` becomes `tools/check step spec-loadcheck-selftest && tools/check step spec-loadcheck && tools/check-frozen`, with its frozen copy in `frozen-lints.toml`. |
| M-8 | `parsecheck` | The brief lists it under `engine/tools/`, but branch `engine` has no parsecheck. | Nothing to place. |
| M-9 | `tools/special-casing` (R-94, a report) expects T's vectors as JSON (`engine/tests/**/*.json`, ids from `tests/cases/*.json`) | The suite T inherits is Markdown cases, `.tally` texts, `.out` baselines and `checks.tsv`: special-casing finds no JSON, prints its "don't exist yet" notice, and searches for nothing. It stays green, but R-94 is silently off. | Not changed at the merge. Proposed for step 3 beside the audit's fixes: take the ids from `checks.tsv` and the values from the visible baselines (never the `-hidden` files), and fail the report's notice once `engine/tests/` holds a suite. Or leave it to T's interface. |

## Checks on the merge

- `check full` #1 on the merge state (before M-7's fix): red, 1 of 24 (`spec-loadcheck`: usage, exit 2), 187 s.
- `rules-selftest/run R-80`: green, 16 cases (4 new), clean run and ban coverage 455/455.
- `guard-test`: 770 checks, 0 failed (725 + 45: six oracle and pin paths in five roles, three Bash forms).
- `check full` #2: **green, 24 steps**, 183 s.
- Merge commit `05103df` (parents `eac9062`, `4e57229`). After the commit hook's Prettier pass: `check-frozen` green ("spec, oracle, pin and rules unchanged"); `git diff engine HEAD` over `tmp/`, `.claude/skills`, `.claude/agents`, `engine/spec`, `engine/tests`, refcore, refcore_fast and `tools/SHA256SUMS` shows only the scaffold's own `engine/tests/runner/` (2 files), so nothing from `engine` was altered.
