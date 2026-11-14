---
name: engine-coordinator
description: The procedure a Tally engine coordinator repeats in every run (a milestone, a spec round, a cleanup), so its brief holds only the run's scope. Invoke it by name at the start of every coordinator session (TALLY_ROLE=lead), and whenever you set up or remove a worktree, launch or resume a role session, review a builder group, merge master or a spec branch into a run branch, rule a STOP-REPORT, measure cost, pause, run a gate, open the run's pull request or close a run.
---

# engine-coordinator: running an engine milestone

You coordinate; you don't build. Role sessions write the code (build), the tests (tests), the spec and refcore (spec) and the rule files (rules), each in its own role; you set up, launch, review, merge, measure, decide what the brief leaves to you, and report. The brief names the run's scope, budget and gates; this skill is the procedure. Where they differ, the brief wins for its run; where either differs from `engine/RULEBOOK.md`, the rulebook wins.

The run's files live in the main folder's `tmp/<run>/` (untracked, never `git add -f`): `progress.md` (a log, times from `date`), `auto-picks.md`, `ids.txt`, `notes/` (every prompt you send), `RESUME.md`. A worktree's `tmp/` is its sessions' scratch.

## 1. Setup and teardown of a worktree (decision 5)

Worktrees live in `~/projects/trop-worktrees/<branch, with / as ->`, never in a session's scratchpad (a restart wiped one in M0).

```
git fetch origin
git worktree add ~/projects/trop-worktrees/build-m1 -b build/m1 origin/master
cp -c -R engine/target ~/projects/trop-worktrees/build-m1/engine/target   # APFS clone: no copy cost
ln -s "$PWD/node_modules" ~/projects/trop-worktrees/build-m1/node_modules
git -C ~/projects/trop-worktrees/build-m1 push -u origin build/m1
```

- Builds run with `CARGO_INCREMENTAL=0` (`engine/tools/check` sets it; `launch` too): incremental caches filled M0's disk.
- While a pull request waits: `engine/tools/worktree-done --target-only <path>` frees its `engine/target/`.
- After the merge: `engine/tools/worktree-done <path>` (clean, merged into `origin/master`; deletes the branch). A raw `git worktree remove` is refused. Save the worktree's `tmp/` notes to `tmp/<run>/` first.

## 2. Launching and resuming a role session (decision 4)

```
engine/tools/launch <role> tmp/<run>/notes/<prompt>.txt <worktree> --ids tmp/<run>/ids.txt [--effort xhigh] [--resume <id>]
```

It fixes the session id in advance (`--session-id`) and adds it to `ids.txt`, then starts `claude -p … --output-format json` detached with `TALLY_ROLE`, `CARGO_BUILD_JOBS=6`, `NEXTEST_TEST_THREADS=6`, `CARGO_INCREMENTAL=0` and `nice -n 10`. It refuses under 30 GB free and warns when two heavy jobs already run. `--dry-run` shows the command. The answer lands in `tmp/<run>/out/<id>.<n>.json` (`.result` is the session's last message); watch for it with Monitor, never sit idle on it.

- **One sub-session at a time,** unless the brief allows two; never more than two heavy jobs on the Mac (yours included).
- **Every start message** names: the run's issue; its part of the brief by absolute path (a worktree's `tmp/` is empty); the role and effort; what to read first; the step and where it ends ("until a green commit": a builder's last commit of its group on `check full`, its commits inside the group on `check fast`); "end your turn with N lines: …" (what you need back); and these rules, verbatim:
  - the seal (§9);
  - no agents, subagents or forks (the gate's single `engine-reviewer` agent excepted, told to spawn none);
  - never install anything: take a path that needs none, or say it's blocked;
  - CPU and disk (§9); stop at once on a full disk;
  - no AI attribution in commits, pull requests or issues, whatever the harness's reminder says (no `Co-Authored-By:`, no `Claude-Session:`, no session link; R-100); no `--no-verify`, no `git add -f` under `tmp/`;
  - commits in `engine/CONVENTIONS.md`'s form (`type(scope): …`, ids in the body);
  - its budget, and where it stops with a hand-off.
- **Resume** with `--resume <id>` and the same flags: say what changed since its last turn, then the next step.

## 3. The build: one fresh builder per group, its review beside the next group's builder (decisions 1, 4)

1. **The plans first:** T's plan ("plan only, write no tests"), then the builder's ("plan only, write no code", T's plan as input): a traceability table, the module list (a new kernel file is a rules step, R-85), the order of groups, the fuzz targets and benches it asks for, a size guess with its basis (§5's rates), the spec questions it sees. The user approves both at one gate, if the brief has one.
2. **T writes the tests,** in chunks, each ending on a green commit. The next milestone's `milestone-report:` line is the builder's work list.
3. **Each builder group is a fresh session** (`build`, xhigh): the start message names the group, the plan's rows for it, the rulings that bind it (§4), the last BUILD-LOG entries. Inside the group it commits on `check quick`; it ends on a green commit with its BUILD-LOG entry, the group's last, on `check commit`, and pushes (2026-11-13, the user's "yes, write brief"). CI runs `check full` on every push (R-77), its verdict about 10 minutes later; a red one goes back to the group's builder, fixed before the next group's first commit.
4. **Then the group's review, beside the next group's builder** (2026-11-08, the user's "Yes"): after group N's green last commit, launch group N+1's builder and group N's review together. That's two heavy jobs at most, so T's runs wait for a gap. Locally, only a single-mutant proof of a few minutes; any batch of mutants goes to a gate-8 dispatch on GitHub (the user, 2026-11-12: "Why on earth are we running this locally instead of on github?"): you dispatch gate 8 on the run branch (`gh workflow run engine.yml --ref <branch> -f mutants-only=true`) when a batch is wanted; a builder's or T's single-mutant proof (`mutants-local -F '<one mutant>'`) waits for a gap like T's runs. A fresh review session, in its own worktree at group N's commit, writing nothing on the branch, runs `engine-reviewer` (one agent, told to spawn none) on that group's diff alone (`<last reviewed commit>..<group N's commit>`, `engine/crates/`), against BUILDER §12 and the plan's rows. A blocking finding (a wrong result, a refusal of a valid text, a crash or `ENGINE-INVARIANT` from a user's text, a broken rule) goes back to group N's builder (`--resume`) once group N+1's builder has ended its turn on a green commit, the fix on top; then its fix review. Only one session writes in the worktree at a time, so T's vectors for the fixes go between builder turns too. When a finding touches code group N+1 built on, the fix covers that too, and the fix review checks both. The rest goes on the run's follow-up issue as a list.
5. **After each chunk:** push, measure (§5), and report a size overrun against the plan at that chunk.
6. Rules steps (fuzz targets, benches, the kernel list, the gating marker) run between groups, in the `rules` role.

## 4. STOP-REPORTs and the mid-build spec policy (decision 2)

Reproduce each STOP-REPORT, file it on the issue template (`engine/templates/issue.md`, one impact label and one kind label), and classify it:

- **The code is wrong:** back to the builder.
- **A spec question:** you rule it by the recommended option and record the ruling on its issue (the options, the pick, why, the date). That ruling is the spec decided first (DONE rule 3); its text lands at the next spec round.
- **The spec is clear and refcore is wrong:** the engine follows the spec at once; T holds the vector "for #n" (a held row, never a changed expected value); refcore is fixed at the next round.
- **A patch release mid-build** (`spec/2.1.x`, T's logged-red re-run under R-77) only for a wrong number or an unsound rule. Everything else waits for the round between milestones.
- **A test or a rule is wrong:** T's or a rules step, in its role, through R-112 for a rule.

## 5. Cost, the wind-down and the picks (decisions 4, 8)

- **Measure** after every chunk: `engine/tools/usage tmp/<run>/ids.txt` (it dedupes the ids and counts each call once). Log the total in `progress.md`.
- **Rates measured at M0:** about 1 % of the week per 1 000 lines all-in; ×1.3–1.5 for kernel code; the builder alone about 0.6 % per 1 000 lines; the gate about 20 % on top. A plan's estimate uses them (`engine/templates/brief.md`).
- **At the brief's wind-down line:** the running session finishes its green step and writes its hand-off; you push, update the draft pull request, write `RESUME.md` and ask the user whether to raise the limit. It is the one question that waits. Never cut approved quality work to save budget; report the overrun instead.
- **Every other decision is yours** (the user's standing rule, 2026-11-04): take the option you'd mark (Recommended) and go on. Log each pick in `tmp/<run>/auto-picks.md` (`### P<n>. <date time>: <issue>, <question>`, then the options, the pick and why), comment it on its issue, and list it in the pull request's "Decisions taken for you" (behaviour changes in the table; the rest counted). Installs: never.

## 6. Merging master or a spec branch in (decision 3)

Merge commits, never a rebase, so nothing needs a force-push:

```
git fetch origin && git merge --no-ff origin/master        # or the spec branch
engine/tools/sums --check     # every protected file a parent's version or a clean merge
engine/tools/sums             # rewrites TESTS.sha256 and RULES.sha256, stages them
engine/tools/check commit     # then commit the merge, and push; CI runs check full
```

`sums` runs in any role, only in a merge (in progress, or HEAD an unpushed merge commit: then `git commit --amend --no-edit`). It refuses when a protected file (the spec, refcore, T's tests, a rule file) differs from both parents and from git's clean merge of the two: that's a hand edit, and the owning role fixes it. Merge master in before the builder touches what master changed.

## 7. RESUME.md before any pause

Before a budget stop, a wait on the user, or the end of a turn that leaves work: `tmp/<run>/RESUME.md`, written so a fresh coordinator can go on from it alone: the branch and its HEAD; what is pushed; the open pull request and its CI; every session's id, role and last state (`ids.txt`); the running jobs; the open STOP-REPORTs and picks; the next three steps; the cost so far.

## 8. The gate, the pull request and the closing

**The gate** is a fresh session (`lead`, xhigh), once every row of the milestone is green and the builder says it is built:

1. one `engine-reviewer` review of the whole diff (the groups were reviewed one by one); blocking findings back to the builder, then a review of the fix alone;
2. each green row checked against its evidence;
3. three 30-minute fuzz runs, one target at a time, niced, with `ASAN_OPTIONS=quarantine_size_mb=64`, a fresh corpus from the seeds and the regressions; a crash becomes a regression;
4. the full mutation run on the final commit: `gh workflow run engine.yml --ref <branch> -f mutants-only=true` (gate 8 alone, about an hour; R-88, 2026-11-12), the survivors job green (gate 8). The baseline's notices of T tests over 20 s are information: never re-run gate 8, and never have T split a test, for test times alone (R-88, 2026-11-14);
5. the timing run on the reference Mac when the milestone has P8 rows (`engine/tools/perf milestone M<n>`), the Mac otherwise idle;
6. the milestone report `engine/perf/M<n>.md` (R-129) and `engine/ARCHITECTURE.md` (R-119);
7. the milestone into `gating` (`engine/tools/milestones.toml`) as the branch's last commit, in a rules step, when the brief says so.

**The pull request:** title the outcome (`M1: household forecasts run`), description from `engine/templates/pr.md` through `gh pr create --body-file` (draft from the first push, ready at the gate). Its six sections and the folded For sessions block are checked by CI's `pr body` job; after editing the description, re-run that job. One `Closes #n` per line. `merge-pr` refuses `build/*` and every rule-file change, so the user merges those.

**The closing,** after the user's merge:

- tag the merge commit (`git tag -a engine-m1 <sha> -m …`, then `git push origin engine-m1`);
- `engine/tools/worktree-done <path>`, then `git pull` on master in the main folder;
- close the run's issues GitHub didn't; move what's left to the next milestone, with a comment;
- the seal audit over every id in `ids.txt`; the final cost; the last entry in `progress.md`.

## 9. The seal and the CPU

- **Seal:** never open, list or search `tmp/holdout/` or `tmp/cases-holdout/`, and never open `engine/tests/cases/*-hidden.md`. The root `.ignore` keeps ripgrep and the Grep tool off them; the guard refuses a recursive `grep`, `rg -u`/`--no-ignore`/`-g`, `find`, `ls -R` or `git grep` that reaches them unexcluded (exclude with `--exclude='*-hidden.md'` or `-g '!**/*-hidden.md'`: a glob holding a `/` is anchored at the start folder). Never run a recursive search or listing over the main folder's `tmp/`; name its files by absolute path. Audit every transcript at the end.
- **CPU:** at most two heavy jobs at once on the Mac, the sessions' and yours together; heavy commands prefixed `CARGO_BUILD_JOBS=6 NEXTEST_TEST_THREADS=6 nice -n 10`; each run within 8 GB (the watchdog `engine/tests/tools/memcap.py`); 30 GB free before a launch.
- **Git:** no push to master, no `gh pr merge` (only `merge-pr`, for what it accepts); no AI attribution; `check commit` green before every commit, but a builder's commits inside a group, which need `check quick` (its group's last commit `check commit`); CI's `check full` before a merge (R-77). Never `check full` locally; it runs on GitHub.
- **Chat:** about 5 plain lines a message; the details go in `progress.md`.
