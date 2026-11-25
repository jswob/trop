---
name: engine-coordinator
description: The procedure a Tally engine coordinator repeats in every run (a milestone, a spec round, a cleanup), so its brief holds only the run's scope. Invoke it by name at the start of every coordinator session (TALLY_ROLE=lead), and whenever you set up or remove a worktree, launch or resume a role session, review a builder group, merge master or a spec branch into a run branch, rule a STOP-REPORT, measure cost, pause, run a gate, open the run's pull request or close a run.
---

# engine-coordinator: running an engine milestone

You coordinate; you don't build. Role sessions write the code (build), the tests (tests), the spec and refcore (spec) and the rule files (rules), each in its own role; you set up, launch, review, merge, measure, decide what the brief leaves to you, and report. The brief names the run's scope, budget and gates; this skill is the procedure. `engine/PLAYBOOK.md` holds the decisions behind both (which model and effort a job gets, how a brief is written, budgets, what runs beside what), each with its reason and where it is enforced: read it whole now, and again after a compaction. Where they differ, the brief wins for its run; where either differs from `engine/RULEBOOK.md`, the rulebook wins.

The run's files live in the main folder's `tmp/<run>/` (untracked, never `git add -f`): `progress.md` (a log, times from `date`), `auto-picks.md`, `ids.txt`, `notes/` (every prompt you send), `RESUME.md`. A worktree's `tmp/` is its sessions' scratch.

## 1. Setup and teardown of a worktree (decision 5)

Worktrees live in `~/projects/trop-worktrees/<branch, with / as ->`, never in a session's scratchpad (a restart wiped one in M0).

```
git fetch origin
git worktree add ~/projects/trop-worktrees/build-m1 -b build/m1 origin/master
cp -c -R engine/target ~/projects/trop-worktrees/build-m1/engine/target   # APFS clone: no copy cost
git -C ~/projects/trop-worktrees/build-m1 push -u origin build/m1
```

- `engine/tools/check` builds `quick`, `commit` and `fences` incrementally and `full` without (R-82); `launch` leaves `CARGO_INCREMENTAL` to it. A worktree's caches are about 2.7 GB; `launch` keeps 30 GB free.
- While a pull request waits: `engine/tools/worktree-done --target-only <path>` frees its `engine/target/`.
- After the merge: `engine/tools/worktree-done <path>` (clean, merged into `origin/master`; deletes the branch). A raw `git worktree remove` is refused. Save the worktree's `tmp/` notes to `tmp/<run>/` first.

## 2. Launching and resuming a role session (decision 4)

```
engine/tools/launch <role> tmp/<run>/notes/<prompt>.txt <worktree> --ids tmp/<run>/ids.txt [--job <job>] [--max-usd N] [--resume <id>]
```

The session's **job** fixes its model and effort (the playbook's table, D2 and D3; `launch --jobs` prints it). With no `--job` a role runs its default job, on Opus: the `builder`, `tests`, a `rules-step`, the `spec-writer`, a `coordinator`. Name the job when it is another: `group-review`, `fix-review`, `gate`, `gate-review`, `planner`, `critic`, `paper-check`, `study`, `round-lead`. A `--model` or `--effort` against the job is a pick, logged; a model id `launch` doesn't know is refused. `--max-usd N` is a hard stop for that session (the playbook's D9).

It fixes the session id in advance (`--session-id`) and adds it to `ids.txt`, then starts `claude -p … --output-format json` detached with `TALLY_ROLE`, `CARGO_BUILD_JOBS=6`, `NEXTEST_TEST_THREADS=6` and `nice -n 10`, without `CARGO_INCREMENTAL`. It refuses under 30 GB free and warns when two heavy jobs already run. `--dry-run` shows the command. The answer lands in `tmp/<run>/out/<id>.<n>.json` (`.result` is the session's last message; when it is a one-line afterthought, the report is the transcript's last long message, #221); watch for it with Monitor, never sit idle on it.

- **One sub-session at a time,** unless the brief allows two; never more than two heavy jobs on the Mac (yours included).
- **Every start message** opens with a heading line, `# <run>, <what>`, whose words `engine/tools/timeline` reads for a builder's turn: `plan only`, `builder group <g> (fresh session)`, `the review's fixes`, `the last fix`, `the gate`. It names: the run's issue; its part of the brief by absolute path (a worktree's `tmp/` is empty); the role, the job and the effort; what to read first; the step and where it ends ("until a green commit": a builder's last commit of its group on `check full`, its commits inside the group on `check fast`); "end your turn with N lines: …" (what you need back); and these rules, verbatim:
  - the seal (§9);
  - no agents, subagents or forks (the gate's single `engine-reviewer` agent excepted, told to spawn none);
  - never install anything: take a path that needs none, or say it's blocked;
  - CPU and disk (§9): one heavy command at a time, at the suite's own job count; stop at once on a full disk; stop only a process whose pid you started, never by a name pattern (in M2 one session's `pkill -f` killed the rules step beside it); stop every background process you started before the turn ends (a watchdog held a headless session open 35 minutes in tally 2.2);
  - no AI attribution in commits, pull requests or issues, whatever the harness's reminder says (no `Co-Authored-By:`, no `Claude-Session:`, no session link; R-100); no `--no-verify`, no `git add -f` under `tmp/`;
  - commits in `engine/CONVENTIONS.md`'s form (`type(scope): …`, ids in the body);
  - its budget, and where it stops with a hand-off.
- **Resume** with `--resume <id>` and the same flags: say what changed since its last turn, then the next step.

## 3. The build: one fresh builder per group, its review beside the next group's builder (decisions 1, 4)

1. **The plans first:** T's plan ("plan only, write no tests"), then the builder's ("plan only, write no code", T's plan as input): a traceability table, the module list (a new kernel file is a rules step, R-85), the order of groups, the fuzz targets and benches it asks for, a size guess with its basis (§5's rates), the spec questions it sees. The user approves both at one gate, if the brief has one.
2. **T writes the tests,** in chunks, each ending on a green commit, in its own worktree and branch from its first chunk (§1), branched from the run branch, never in the build worktree. You merge each green chunk into the run branch between builder turns (§6), and the builder meets it at its next turn (2026-11-16, the process study after M2: T's turn inside the build worktree held it 27 minutes in M2). The next milestone's `milestone-report:` line is the builder's work list.
3. **Each builder group is a fresh session** (`build`, xhigh): the start message names the group, the plan's rows for it, the rulings that bind it (§4), the last BUILD-LOG entries. Inside the group it commits on `check quick`; it ends on a green commit with its BUILD-LOG entry, the group's last, on `check commit`, and pushes (2026-11-13, the user's "yes, write brief"). CI runs `check full` on every push (R-77), its verdict about 10 minutes later; a red one goes back to the group's builder, fixed before the next group's first commit.
4. **Then the group's review, beside the next group's builder** (2026-11-08, the user's "Yes"; 2026-11-16, "the build worktree never waits"): after group N's green last commit, launch group N+1's builder and group N's review together, in the same minute, always. If group N+1 lacks T's vectors or a patch, its builder starts on the part of its plan that needs neither, and the rest reaches it at its next turn; never hold it for them (in M2 the build worktree stood idle 2.8 hours for reviews). That's two heavy jobs at most, so T's runs wait for a gap. Locally, only a single-mutant proof of a few minutes; any batch of mutants goes to a gate-8 dispatch on GitHub (the user, 2026-11-12: "Why on earth are we running this locally instead of on github?"), and there is one whole run a run, at the gate (§8), none per group (the user, 2026-11-19: "Do so"); a builder's or T's single-mutant proof (`mutants-local -F '<one mutant>'`) waits for a gap like T's runs. A fresh review session (`--job group-review`, on Opus: the playbook's D2), in its own worktree at group N's commit, writing nothing on the branch but its probes under that worktree's `tmp/`, runs `engine-reviewer` (one agent, told to spawn none) on that group's diff alone (`<last reviewed commit>..<group N's commit>`, `engine/crates/`), against BUILDER §12 and the plan's rows, by the gate review's method: its start message lists the group's new forms and the forms already built that they meet, to be crossed on both engines on texts of about ten lines, and read against the spec where the two agree (the agent's "Cross the forms"). A blocking finding (a wrong result, a refusal of a valid text, a crash or `ENGINE-INVARIANT` from a user's text, a broken rule) goes back to group N's builder (`--resume`) once group N+1's builder has ended its turn on a green commit, the fix on top; then its fix review (`--job fix-review`, on Opus), on the fix's range alone. Only one session writes in the build worktree at a time: T writes the vectors for the fixes in its own worktree, and you merge them between builder turns. When a finding touches code group N+1 built on, the fix covers that too, and the fix review checks both. When a fix review finds a blocking defect in a form the last fix mended, stop patching: rule the one statement that decides every case of it (§4) and have the next fix build that (the playbook's D12: M3's #235 drew findings from four reviews in a row). The rest goes on the run's follow-up issue as a list.
5. **After each chunk:** push, measure (§5), and report a size overrun against the plan at that chunk.
6. Rules steps (fuzz targets, benches, the kernel list, the gating marker) run between groups, in the `rules` role.

## 4. STOP-REPORTs and the mid-build spec policy (decision 2)

Reproduce each STOP-REPORT, file it on the issue template (`engine/templates/issue.md`, one impact label and one kind label), and classify it:

- **The code is wrong:** back to the builder.
- **A spec question:** you rule it by the recommended option and record the ruling on its issue (the options, the pick, why, the date). That ruling is the spec decided first (DONE rule 3); its text lands at the next spec round.
- **The spec is clear and refcore is wrong:** the engine follows the spec at once; T holds the vector "for #n" (a held row, never a changed expected value); refcore is fixed at the next round.
- **A patch release mid-build** (`spec/2.1.x`, T's logged-red re-run under R-77) only for a wrong number or an unsound rule. Everything else waits for the round between milestones.
- **A patch never holds a builder** (2026-11-16, the process study after M2: G4's builder waited 86 minutes for M2's first patch). The builder starts, or goes on, on the spec's text as ruled; the patch, its pin and T's re-run merge into the run branch at the next turn boundary (§6), and the reds they bring are that builder's next turn.
- **A test or a rule is wrong:** T's or a rules step, in its role, through R-112 for a rule.

## 5. Cost, the wind-down and the picks (decisions 4, 8)

- **Measure** after every chunk: `engine/tools/usage tmp/<run>/ids.txt` (it dedupes the ids and counts each call once). Log the total in `progress.md`. A session on Fable counts in that total at Fable's list price, twice Opus's but for cache reads, and a `by model:` block shows each model's part. The total is the run's share; the last two lines estimate the account's own meters, the week's and Fable's, in points (the playbook's D10), never a session's share; a run with a Fable session logs both beside the total. One ids file holds every session of the run.
- **Rates:** the playbook's §4 holds each run's measured cost per 1 000 changed lines (M0 about 1 %, M1 2.1 %, M2 2.2 %) and what one session of each kind costs. A plan's estimate uses the last run's, and sizes each group from the nearest group already built (D11).
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

**The gate** is a fresh session (`--job gate`: `lead`, xhigh), once every row of the milestone is green and the builder says it is built:

1. one review of the whole diff (the groups were reviewed one by one), by a fresh `--job gate-review` session on Fable (the playbook's D2), in its own worktree, writing nothing but its probes. Its prompt is short: the diff range, the plan, the spec sections, what counts as blocking, and the form of its answer; it runs `engine-reviewer` once, told to spawn none. Blocking findings go back to the builder, then a review of the fix alone (`--job fix-review`). Record which blocking findings sit in code a group review and its fix review had passed, and what fixing them at the gate cost: D2's Revisit takes both numbers to the user;
2. each green row checked against its evidence;
3. three 30-minute fuzz runs, one target at a time, niced, with `ASAN_OPTIONS=quarantine_size_mb=64`, a fresh corpus from the seeds and the regressions; a crash becomes a regression;
4. one whole mutation run for a run that changes kernel files, dispatched the moment the last group's fixes are merged, beside step 1, so nobody waits for it at the end: `gh workflow run engine.yml --ref <branch> -f mutants-only=true -f next=true` (gate 8 alone, about two hours; `next` counts the milestone's own tests green unmutated, before its marker; R-88, 2026-11-19, the user's "Do so"). Before it, `engine/tools/mutants-gate slow` on the last whole run's `mutants-plan` artifact (`gh run download <id> -n mutants-plan`): a test it warns would pass its cap gets a cap of its own first, in a rules step (M4 lost two dispatches at their baseline). Its score goes in the gate's report beside the last tagged milestone's, and may be no more than 2 points under it (M3: 90.9 %); the final engine's gate needs 90 % (R-87). A second whole run only when the score misses that floor, or when the gate's fixes change kernel files. Then the builder sorts the survivors (an input for each row a test could kill), T takes one turn with the sort in hand, the builder lists the rest (R-87), and the survivors job is green. The baseline's notices of T tests over 20 s are information: never re-run gate 8, and never have T split a test, for test times alone (R-88, 2026-11-14);
5. the timing run on the reference Mac when the milestone has P8 rows (`engine/tools/perf row` with its rows and P8-2), the Mac otherwise idle; then `engine/tools/perf growth`, one text at two horizons, whose table and ratios go into `engine/perf/M<n>.md`;
6. the milestone report `engine/perf/M<n>.md` (R-129) and `engine/ARCHITECTURE.md` (R-119);
7. the run's timeline: `engine/tools/timeline tmp/<run>/ids.txt`, its four tables pasted into the gate's report beside the cost (the playbook's D14); a loop whose median passed its target is a follow-up issue;
8. the milestone into `gating` (`engine/tools/milestones.toml`) as the branch's last commit, in a rules step, when the brief says so.

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
