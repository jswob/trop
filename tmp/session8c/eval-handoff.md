# Step 5 hand-off: the skill's eval, for the master's brief of session 8d

On 2026-10-31 the user chose "Eval in a new session (Recommended)". Step 3 had cost about 1.6 % of the week (estimate 0.4–0.7 %), which put the projection for session 8c at 3.6–4.6 %, over its 3.8 % stop. 8c does step 4 and G2 for steps 1–4. Step 5 moves to a fresh session, 8d, with its own budget. This note gives what 8c knows; the brief is the master's to write.

## What step 5 is (session8c-brief.md §9, unchanged)

- Build 8b's `tmp/session8b/eval-design.md`:
  - a fixture crate with E1–E9, each case one red state that `check fast` reports;
  - grading scripts;
  - 26 headless runs (`TALLY_ROLE=build claude -p …` in fresh throwaway worktrees, two arms differing only in `.claude/skills/engine-rust/`), staged as G2a item 4 decided: E8 and E9 get 3 runs per arm, the other seven get 1 per arm.
- The user's bar (G1): no banned form in any run; E8 and E9 end in a STOP-REPORT in at least 5 of 6 runs.
- Get the user's go on the estimate before the first run. A failing case changes the skill or `engine/CLAUDE.md` through 8b's approval path, never a grader.
- Estimate in the brief: 0.7–1.4 % for the first 26 runs.

## The state 8d starts from (branch engine-scaffold, after PR #1 if the user merged it)

- **`engine/CLAUDE.md` exists** (8b's draft, plus five lines from 8c listed in `g2.md`). The build role's SessionStart hook prints it at startup and after each compaction: `engine/tools/session-start`, set in `.claude/settings.json` with the matcher `startup|compact`. A `claude -p` run in the build role therefore gets it, so the eval measures the skill on top of it, as the build will.
- **The rulebook runs through R-131** (`tools/rules-selftest` covers R-1..R-131). `check fast` now also runs the headers, the trace and the ast-grep scan (R-120, R-128/R-75, R-83).
- **The stub has the `counters` feature** (`--stats`); exceptions: 4 in all, the CLI's 2 of 3.
- **The fixture crate** is a rules-role path ("the eval's fixture crate", brief rule 2). Its cases must be red states that `check fast` reports on that tree.

## What 8c learned that 8d needs

- **The guard in the rules role refuses** these; write scripts with the Write tool and run them as files instead:
  - an interpreter (python3, awk, perl) whose command line names `engine/tests/…` or `engine/spec/…`;
  - a heredoc fed to python3 that names engine paths;
  - `git merge engine`;
  - plain `cargo fmt` (use `cargo fmt -p tally -p tally-cli`).
- **Never write into the tree while a `check` runs.** The checks read it.
- **The budget reading** is `python3 -I -B tmp/session8c/tools/usage.py <session ids>`. It reads every project directory, so worktree sessions are counted. The headless runs' ids come from their `claude -p` JSON output.
- **check full** takes about 200 s warm (27 steps). At most 6 heavy jobs at once.
- **Each step-3/4 commit carries `Rule-Change:` trailers**, and `tools/commit-check <base>` (R-112) checks them on a pull request. A fixture crate under the rules role's paths is a rule file, so its commit needs a trailer and a proof.
