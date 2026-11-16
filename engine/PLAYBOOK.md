# The playbook: how engine sessions are run

One place for how we brief, staff, launch, pace and hand over the sessions that build the Tally engine. The master session and every coordinator read it whole at the start of a session and after a compaction; the SessionStart hook names it for them.

- **It holds decisions, not procedure.** The coordinator's steps are the skill `engine-coordinator`, the builder's rules are `engine/CLAUDE.md`, and the walls are `engine/RULEBOOK.md`. Where one of them and this file disagree on how sessions are run, one is wrong: fix it in the same pull request.
- **Each decision is one entry:** the pick, the options, why (our own evidence, or Anthropic's guidance), who decided and when, where it is enforced, and when to look again.
- **To change an approach,** edit its entry and the code it names, in one pull request the user merges. This file isn't a rule file, so the master, a coordinator or a rules session may edit it. The tools it names are rule files (R-80), so their half is a rules step. `guard-test` fails when the job table below and `launch`'s differ.
- **Evidence** is cited by pull request, issue and date. The run logs are in the main folder's `tmp/<run>/`, which worktrees can't see; nothing here needs them.

## 1. Who runs what

A **run** is one coordinator, one pull request: a milestone (M2), a spec round (tally 2.1.8), a rules step, a study.

- **The user** starts runs, raises limits and merges pull requests.
- **The master** (app work, no role) follows the runs, writes each brief, keeps the order of runs and reports to the user.
- **A coordinator** (the `lead` role) runs one run from its brief: it launches the role sessions, rules what the brief leaves open, measures, and opens the pull request. It never builds.
- **Role sessions** (`build`, `tests`, `spec`, `rules`) each write only their own files; the guard holds that (R-79).

**The job table** (D2, D3). A job is what a session is for; it fixes the role, the model and the effort. `engine/tools/launch ROLE … --job JOB` reads the same table (`launch --jobs` prints it).

| Job            | Role  | Model              | Effort | What it does                                                               |
| -------------- | ----- | ------------------ | ------ | -------------------------------------------------------------------------- |
| `builder`      | build | `claude-opus-5-5`  | xhigh  | its plan; then one group of a milestone per fresh session                  |
| `tests`        | tests | `claude-opus-5-5`  | xhigh  | T: the test plan, the tests in chunks, a vector per fix and per survivor   |
| `rules-step`   | rules | `claude-opus-5-5`  | high   | a tool, a check, the pin, a fence's wiring, between other sessions' turns  |
| `study`        | rules | `claude-fable-5-2` | high   | a question about how we work, answered with measurements and one proposal  |
| `spec-writer`  | spec  | `claude-opus-5-5`  | high   | the spec text and refcore's change for rules already ruled; a patch case   |
| `planner`      | spec  | `claude-fable-5-2` | high   | a spec round's factoring: the open questions as a few general rules        |
| `critic`       | spec  | `claude-fable-5-2` | high   | fresh and read-only: tries to break the planner's rules, once              |
| `paper-check`  | spec  | `claude-fable-5-2` | high   | before a big milestone: its features against the existing ones, on paper   |
| `coordinator`  | lead  | `claude-opus-5-5`  | high   | a milestone's or a rules run's coordinator                                 |
| `round-lead`   | lead  | `claude-fable-5-2` | high   | a spec round's coordinator: it rules the factoring                         |
| `group-review` | lead  | `claude-opus-5-5`  | high   | `engine-reviewer` on one builder group's diff, and on its fix              |
| `gate`         | lead  | `claude-opus-5-5`  | xhigh  | the gate's procedure: evidence, fuzz, the mutation run, timing, the report |
| `gate-review`  | lead  | `claude-fable-5-2` | high   | the gate's one review of the whole diff                                    |

## 2. The decisions

### D1. Where this lives

- **Pick:** this file, tracked, plain Markdown. Three pointers reach it: the SessionStart hook's line for the lead, the rules role and app work, at the start and after every compaction; the first paragraph of the skill `engine-coordinator`; the first line of `engine/templates/brief.md`.
- **Options:** a skill (but `.claude/skills/engine-*` are rule files, so every edit would be a rules step with a checksum, and a skill is procedure, which the coordinator's already is); both (two texts to keep equal).
- **Decided:** the user, 2026-11-15: "in a single place that we can edit depending on what approach do we pick, and so it's not lost between different context compacts".
- **Enforced:** `engine/tools/session-start`; `guard-test`'s hook rows.

### D2. Which model runs which job

- **Pick:** Claude Fable 5.2 for the judgment a whole run builds on: a spec round's lead, planner and critic, the paper check, the gate's whole-diff review, a study. Claude Opus 5.5 for volume and procedure: builders, T, milestone coordinators, group reviews, the spec writer, rules steps, the gate's procedure.
- **Options:** all Opus (how we ran until 2026-11-15); Fable everywhere (twice the price per token on the building and testing that are most of a run's tokens); Fable for group reviews too (open, see Revisit).
- **Why:**
  - Anthropic: "If you're unsure which model to use, start with Claude Opus 5.5 for most workloads", and "Use Claude Fable 5.2 for demanding reasoning and long-horizon agentic work, or when your evals on Claude Opus 5.5 at higher effort still fall short." Fable costs $8 / $40 per million tokens in and out, Opus $4 / $20; cache reads are $0.20 on both.
  - Where Opus at xhigh fell short for us is judgment. 2.1.7's plan drew 23 findings from its critic, 8 of them blocking, and its patch left 14 forms where refcore didn't follow the new rules' own text (PR #157). Size plans ran 2 to 4 times low (D11). In M2 the share of spec questions an existing rule settles fell from 87 % to 71 % (PR #168).
  - Volume is where the tokens go: builders were 15.2 of M0's 26.4 % (PR #70), and their code doesn't compound the way a spec rule does.
  - So the round's planner no longer writes its own patch: the planner (Fable) factors, the lead rules, and a fresh spec writer (Opus) writes text and refcore from the ruled plan.
- **Decided:** the user, 2026-11-15, on the master's proposal: "I like what you said let's incorporate this". The job names and `study` are this file's.
- **Enforced:** `launch`'s `JOBS`; it refuses a model id outside its `MODELS`. A coordinator the user starts by hand takes the model from its brief's start command.
- **Revisit:**
  - At M3's start, from M2's gate, the first on Fable: if the gate review finds a blocking defect in code that a group review and its fix review passed, group reviews move to Fable for M3 (about 0.4 % a review becomes about 0.8 %).
  - After 2.1.8: its critic's findings, T's STOP-REPORTs against the patch, and the words added per question, beside 2.1.7's (23 and 8; 14; about 4).

### D3. Effort per job

- **Pick:** builders, T and the gate at xhigh; every other job at high; every Fable job at high. `launch` always passes the effort.
- **Options:** builders at high; Fable at xhigh.
- **Why:**
  - Anthropic's effort page: "Start with `high`. Most of the gain in capability is already present at `high`", and "Step up to `xhigh` for your hardest long-running coding and agentic work, where you've measured a gain".
  - Builders and T have run at xhigh since S, and no run compared it with high: the choice is unmeasured. It stays because reviews still find 2 to 5 blocking defects a group at xhigh (M2, groups 1 to 6).
  - Fable at high: its prompting page says xhigh "raised scores slightly … and used many more tokens", and that at xhigh and max it reviews its own work with subagents, which D8 forbids.
  - Always passed, because defaults differ: the API's is high for Fable 5.2 and medium for Opus 5.5, and Claude Code's own default for each couldn't be confirmed.
- **Decided:** the retro after M0 (the user's "Accept", 2026-11-05) for the Opus jobs; the master and this file, 2026-11-15, for Fable.
- **Enforced:** `launch`'s `JOBS`; an override prints a note to log as a pick.
- **Revisit:** at M3, one group that isn't kernel code runs its builder at high. Compare blocking findings and cost per 1 000 lines with its neighbours (§4). High stays if the findings don't rise.

### D4. The shape of a brief

- **Pick:** one brief a run, from `engine/templates/brief.md`, holding only what this run adds to the skill; written to the model that reads it (§3).
- **Why:** M0's coordinator worked from a 2 500-word brief and wrote 57 prompt files, 27 000 words, by hand, relearning lessons on the way (the retro after M0, blocker 4).
- **Decided:** the retro's decision 4 (2026-11-05); the Fable column of §3 is this file's, 2026-11-15, and unmeasured.
- **Enforced:** the template; the skill's list of what every start message names.
- **Revisit:** after 2.1.8, the first round briefed for Fable.

### D5. Who decides

- **Pick:** a session takes the option it would mark recommended, logs it and goes on. Four things wait for the user: a limit raise, a merge, a force-push, a new language construct.
- **Decided:** the user, 2026-11-04: "I would prefer if the session would just pick the recommended. It should just note what it picked so I can inspect it later, it shouldn't stop to ask me questions. I only sign on limit changes + I merge ready PR's".
- **Enforced:** the skill's §5: `auto-picks.md`, a comment on the issue, the pull request's "Decisions taken for you", which CI's `pr body` job checks for. Installs are never a pick: take a path that needs none, or log it as blocked.

### D6. How a session is started

- **Pick:** the user starts a coordinator in a terminal with the brief's command, so they can watch it. A coordinator starts role sessions headless with `engine/tools/launch`. The master launches a coordinator headless only while the user is away, and relays its questions. Every prompt sent is saved in the run's `notes/`. A resume says what changed since the session's last turn, then the next step.
- **Options:** a role per folder (rejected 2026-10-31: it brings back several checkouts); everything headless (the user had a headless T killed on 2026-10-31 to watch it instead).
- **Why:** the role is set by whoever launches, never by the session, or a session could make itself `rules`. Anthropic, on resuming Fable: "say exactly what you want rather than sending a simple 'continue.'"
- **Enforced:** `launch` (only the lead and app work run it; it fixes the session id first and writes the ids file); the guard refuses a nested `claude` run anywhere else (R-81).

### D7. What runs beside what

- **Pick:** one writer in a worktree at a time. Group N's review runs beside group N+1's builder, in its own worktree. T gets its own worktree and branch when it must write beside a builder, and the coordinator merges it in. A plan-only session never shares a worktree with a writer. Two heavy jobs at most (D18).
- **Why:** M1's builders sat idle through each review until the overlap (the user's "Yes", 2026-11-08). In M2 a plan-only builder's Stop hook went red on T's uncommitted edits in the same worktree (pick P10), and T's long ledger work blocked the next group until T moved to its own worktree (P39).
- **Enforced:** the skill's §3; `launch` warns when two heavy jobs already run.

### D8. Sub-agents

- **Pick:** none. Every start message says so. Two exceptions: a review's single `engine-reviewer`, told to spawn none; a study's page fetches.
- **Why:** on 2026-10-24 three agents spawned 11 of their own under open-ended scopes and used about 15 % of the week; the user: "I can't afford this". Anthropic, for Fable: "subagents can account for a large part of what a session costs".
- **Enforced:** by the prompt only.
- **Revisit:** the installed Claude Code knows `CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH`, and its docs say `1` turns nesting off. Untested here; if a run shows it holds, `launch` sets it and the prompt line goes.

### D9. Budget

- **Pick:** a limit is a safety stop, never a target to save against.
  - A brief gives the estimate with its basis, the limit, and a wind-down line one point under it. A small session gives a plan, a stop and a hard stop (1.5, 2.5 and 3 % for a study).
  - When the plans' re-estimate passes the limit, the coordinator asks then, in one question with the numbers.
  - At the wind-down: finish the green step, push, write `RESUME.md`, ask for the new limit.
  - Approved quality work is never cut to fit.
- **Decided:** the user, 2026-11-04: "I don't care about budget to be honest I just want to have a limt to not wake up to 0% left. I'm fine with raising it if necessary to deliver good results, just always run the ask through me".
- **Why:** every limit so far was raised: M0's four times, M1's to 40, 48 and 55 %, 2.1.7's from 20 to 25 %. M2's went from 30 to 50 % at plan time, before any code: the early ask working.
- **The account's limit is not the run's.** On 2026-10-26 the account hit its weekly limit while the session measured its own share at 22 %. When the week's meter is high before its reset (Mondays 16:00, Warsaw), the user sets an earlier wind-down (2026-11-08). After a limit error, resume the stopped sessions by id; never start new ones for the same work.
- **Enforced:** by the coordinator measuring after every chunk (the skill's §5), which is a prompt, so a runaway inside one chunk isn't caught. `launch --max-usd N` passes Claude Code's `--max-budget-usd`, a hard stop for one headless session by its own estimate; 1 % of the week is about $34 at Opus's list price. It is optional and untried: use it first on a review session, at twice its stated stop, since it ends a turn where it stands.

### D10. Measuring cost

- **Pick:** `engine/tools/usage <the run's ids file>` after every chunk, logged in `progress.md`. Its figure is a share of the week (8.6 million weighted tokens to 1 %, calibrated on Opus). A Fable call counts inside it at Fable's list price beside Opus's: twice on input, output and cache writes, the same on cache reads. One ids file a run holds every session, whatever its model; a block by model shows each model's part, and Fable's tokens as Opus's and in dollars.
- **Decided:** the user, 2026-11-15, asked how their plan limits Fable: it draws on the same weekly limit as Opus, about twice per token, and the "3 resets" they have are resets of that limit. So a run's budget and its stops count Fable.
- **Options:** Fable apart from the week's share, in a second ids file (the master's first practice, when the metering wasn't known); Fable in dollars only (if it billed to usage credits, as Claude Code's docs say it can).
- **Enforced:** `usage`'s `PRICED` table; `guard-test`'s planted transcripts.
- **Revisit:** the factor is the list-price ratio, not a reading of the plan's meter. At 2.1.8, read the week's meter (`/usage`) before and after the planner's session and compare with this tool's figure for it.

### D11. Estimating size and cost

- **Pick:** a group's size comes from the measured lines of the nearest group already built (BUILD-LOG's size lines), its cost from the last run's measured rates (§4). Never from refcore's length, never from a fresh guess. The overrun is reported after each group.
- **Why:** M0's plan priced a builder at 0.22 % per 1 000 lines, and it measured 0.6. M1's plan expected 10 600 lines, and M1 built 24 000 (PR #110). 2.1.7's planner said 880 engine lines, and the builder changed 2 800 (PR #157). Under this rule, M2's first four groups landed at 0.86, 0.63, 0.9 and 1.2 times their plan (PR #168).
- **Decided:** the retro's blocker 8 (2026-11-05); the nearest-group rule is the M2 brief's, 2026-11-14.
- **Enforced:** the template's §5; the skill's §3 and §5.

### D12. The review loop

- **Pick:** after each builder group, a fresh session reviews that group's diff with `engine-reviewer`, in its own worktree, writing nothing. Blocking findings go back to the group's builder. One review of the fix follows, which blocks only on a defect the fix introduced. A last fix is checked by the coordinator's own probes against refcore. There is no third review. The gate reviews the whole diff once.
- **Why:** M0 reviewed only at the gate: 5.1 % and 8 hours, in rounds of 6, 1, 3 and 0 blocking findings. Since then, fix reviews keep finding defects the fixes brought in: M1's groups 2, 3, 5 and 7 (2, 2, 1, 2), 2.1.7 (2), M2's groups 2, 3, 4 and 6 (2, 3, 1, 3). So the fix review stays, and a builder sweeps the engine against refcore over the probe files before each fix commit: every line that turns from same to different is explained.
- **Decided:** the retro's decision 1 (2026-11-05); the bound, M1's pick P37.
- **Enforced:** the skill's §3 and §8; `.claude/agents/engine-reviewer.md`.

### D13. Mutation runs

- **Pick:** on GitHub only. The coordinator dispatches gate 8 on each group's green commit, in the background, with `next=true` until the milestone gates; nobody waits for it, and its survivors go to that group's builder and T at their next turn. On the Mac, only a proof of one mutant, a few minutes. Never a re-run for test times.
- **Decided:** the user, 2026-11-12: "Why on earth are we running this locally instead of on github?"; 2026-11-14: "I don't want to wait an hour for full mutation run again".
- **Why:** M1's gate ran three mutation runs of over 3 hours, and T ran a 20-file batch for 2.5 hours on the shared Mac, which left a mutant in the tree. A per-test 20 s check went red in 4 of 5 runs on runner noise while no mutant timed out, each fix a new hour-long run, until PR #166 made it a report. The lesson holds for any check: one that guards a stand-in for what another check measures directly, and flaps, goes.
- **Enforced:** R-88; `.github/workflows/engine.yml`; the skill's §3 and §8; `engine/CLAUDE.md` §3.

### D14. The check loops

- **Pick:** `check quick` after an edit, `check commit` before a commit, the full check on GitHub for every push. The two local checks stop at the first red step and say which steps didn't run. `quick` runs the lints, the unit tests and the next milestone's report; `commit` adds T's gating vectors and the fences' equivalence suites, which `quick` leaves out. Targets: `quick` 45 s, `commit` 3 min, a pull request's CI 12 min with no gate-8 run beside it. A check's time is a requirement with a target, reported at every gate beside the cost. A change to a tool or to CI is proven by its selftest case and the pull request's own CI.
- **Decided:** the user, 2026-11-11: "it's impossible to develop anything with checks so slow." The stop, the suites out of `quick` and T's vectors in `commit`: the user, 2026-11-16, "I agree to chagnes 1,2 and 3", on the process study after M2.
- **Why:** at M1 the local full check took 15 to 47 minutes and builders waited 54 % of their active time. PR #139 brought the loops to 32 s, 59 s and 8.5 minutes. In M2 the loops grew back unseen: `quick` ran at a median 109 s over 189 runs and builders spent 7.4 of 32.5 hours in checks; 65 red runs failed on a lint and still ran every test; three fence suites were 97 % of `quick`'s test time, and no local check ran T's vectors (pick P53).
- **Enforced:** `engine/tools/check`'s modes, `check fences` for a builder on a fence, and `engine/tools/milestones.toml`'s `quick_leaves_out`; R-77 and R-82, with R-82's selftest cases; the Stop hook.

### D15. Spec questions and spec rounds

- **Pick, mid-build:** where the spec is clear the engine follows it, T holds the vector, and refcore waits for the round. The coordinator rules a spec question at once and sorts it: settled by an existing rule (cited), or new (an issue for the next round, labelled with its class). A patch release mid-build only for a wrong number or an unsound rule.
- **Pick, the round:** factoring before any text. Questions are grouped into classes, each given one rule in a named solved form. A rule passes only if it decides 3 or more items across 2 or more features with no exception, replaces case text and adds no construct. A fresh read-only critic tries to break the plan, and the planner answers once. The round closes whole: spec, refcore, tests and engine agree when it merges. The spec, counted in words, doesn't grow.
- **Decided:** the user, 2026-11-10: "we should think about general rules our engine could follow so we hit a lot of birds with one stone"; 2026-11-13: "I just don't want spec rounds to produce more edge cases. The point is that they make spec simpler and cleaner."
- **Why:** M0's three mid-build patches cost the builder about 3.5 hours of waiting, six steps each. 2.1.6 ruled case by case: 1 918 words for 24 issues. 2.1.7 factored: 361 words for 84 questions.
- **Tripwires for 2.1.8** (the master, 2026-11-15): M2's new questions fold into about 5 rules and the spec gets no longer, or the language is simplified before M4; a paper check before M5.
- **Enforced:** the skill's §4; the round's brief; the word counts in the pull request's "How it was checked".

### D16. Git

- **Pick:** one session, one role, one branch, one pull request into `master`. Sessions push and open pull requests; the user merges; Claude tags after the merge. Merge commits, never a squash or a rebase. A dependent pull request is stacked: built on its parent, base `master`, its body starting "Stacked on #X: merge that first". A wrong commit already pushed gets a new branch, not a force-push. Nothing under `tmp/` is tracked. No AI attribution anywhere. One `Closes #n` per line.
- **Decided:** the user, 2026-10-31 ("I could do merging but you should create tags") and 2026-11-01: "Instead of merging PR's on your own just make sure they are green and stack the ones that depnd on others."
- **Why, the newest lessons:** in M2, T pushed a commit whose scope `commit-check` refuses, then one with a 73-character subject, because its check ran through a pipe that hid the exit code (picks P56, P64). Read a check's own exit code.
- **Enforced:** the guard (R-132, and `commit-check` before every push, R-100); `merge-pr`; `worktree-done`; CI's job that fails on a tracked `tmp/` file.

### D17. Hand-offs and compaction

- **Pick:** state lives in files, never only in a context.
  - A run's folder `tmp/<run>/` in the main checkout: `progress.md` (one line a step, timed from `date`), `auto-picks.md`, `ids.txt`, `notes/` (every prompt), `RESUME.md` before any pause.
  - The builder's `engine/BUILD-LOG.md`. What must outlast the run goes in the pull request and its issues.
  - A worktree can't see the main folder's `tmp/`: a brief names such files by absolute path, and the coordinator copies in what a role session needs.
  - After a compaction, read this file, the brief, then `RESUME.md` or `progress.md`. Never read the big inputs whole again; write each deliverable to disk as soon as its inputs are read.
  - A brief names the form of the session's last message, in lines.
- **Why:** on 2026-10-22 a session re-read 400 KB of reports after every compaction and looped. M0's restart wiped a worktree kept in a scratchpad. Practices kept only in the master's memory were lost between compactions, which is why this file exists.
- **Enforced:** the SessionStart hook, at startup and after a compaction; the skill's §7; R-79 lets every role write `tmp/`.

### D18. The Mac

- **Pick:** two heavy jobs at most, niced, across every session. Each process within 8 GB. 30 GB of disk free before a launch. The user's own processes are never stopped, and stopping them is never suggested: shrink the engine's side. A session stops only a process whose pid it started, never by a name pattern.
- **Decided:** the user, 2026-10-31: "could we limit it so we don't cross 60%?"; 2026-11-06: "Why would I stop it, it's my work".
- **Why:** on 2026-10-27 one run reached 83 GB and filled the disk with swap. A full disk lost M0 a builder turn. In M2 a spec session's `pkill -f` killed the rules step running beside it (2026-11-15).
- **Enforced:** `launch` (the 30 GB floor, the job counts, `nice`, the warning at two heavy jobs); the watchdog `engine/tests/tools/memcap.py`; the skill's §9. The kill rule is a prompt line.

### D19. Talking to the user

- **Pick:** about 5 plain lines a message, about 8 for a run's final report; details go to files. Numbers before adjectives. A pull request's title is the outcome, its description the six sections of `engine/CONVENTIONS.md`. Every brief says how long the answer it wants back is.
- **Decided:** the user, 2026-10-23: "speak simpler to me. to be one-tenth of this text to answer my question."
- **Why, for Fable:** "Effort has little effect on the length of assistant turn responses. Use prompting to change the length and tone of replies", and "Write for an intelligent but low-context reader."
- **Enforced:** CI's `pr body` job; the skill's §9.

### D20. Facts about models

- **Pick:** before a brief or a message names a model id, a price or a default, check the installed Claude Code and Anthropic's pages, not memory. A new model is a rules step: its id in `launch`, its prices in `usage`, its jobs in the table above.
- **Why:** on 2026-11-15 the master sent `claude-fable-5-1` and a price of $10 / $50 from a stale cache; the user: "You told it to use Fable 5.1 when 5.2 is out for some time already".
- **Enforced:** `launch` refuses an id outside `MODELS` (checked against Claude Code 2.1.325, 2026-11-15).

## 3. Writing a brief

**Every brief has these parts,** in this order, and nothing a skill already says:

1. Who the session is: its job, role, model and effort; the command that starts it; the line the user pastes.
2. The goal, and what done means, as things that can be checked.
3. Why, in the user's words, with the date.
4. What is in scope, as a closed list; what isn't, and where each goes.
5. What differs in this run. Only that: the procedure is the skill's.
6. What waits for the user, as a closed list (D5). Everything else is the session's.
7. The budget: the estimate with its basis, the limit, the wind-down (D9).
8. The walls: the seal, the Mac, git. Each is held by a tool, so each is short.
9. What comes back: the files, and the last message's form in lines.

Measured sizes, in words: M1's brief 2 247, 2.1.7's 2 978, M2's 2 291; a rules session's 1 400 to 1 800; the brief this file came from, written for Fable, 1 314.

**The same brief, for Opus and for Fable:**

|                 | For an Opus job                                                     | For a Fable job                                                                                    |
| --------------- | ------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| It gives        | the scope and the order of steps; the method by pointing at a skill | the goal, the bar, the inputs and the form of the output; the method is the session's              |
| Steps           | named and ordered wherever order matters                            | only the constraints that are real: the critic never sees the planner's notes; one answer, bounded |
| Prior art       | may be given as forms to try first                                  | given as inputs, never as a script                                                                 |
| Open choices    | "take the recommended option and log it"                            | named one by one ("you choose where it lives"), beside the closed list of what waits               |
| Rules           | a closed list, verbatim in every start message                      | the same walls, each with its reason; "never" only where a tool also refuses                       |
| Old cautions    | kept while the failure they answer still happens                    | removed once it doesn't                                                                            |
| The budget line | the estimate, the limit, the wind-down                              | the stop alone; no running count shown to the session                                              |
| Length          | 2 000 to 3 000 words a run; a start message 150 to 400              | 600 to 1 400 words                                                                                 |

**Why the Fable column reads so,** from Anthropic's page on prompting Fable 5.2:

- "The model treats an instruction that contains 'never' as absolute." A rule that has a condition is written with it: "ask before deleting", not "never delete".
- Cautions pile up: "Remove notes you no longer need", and "Name the steps that need no approval". A listed approval made the model hold back about half of routine requests in Anthropic's tests.
- Removing a large token budget shown to the model lowered cost at equal results.
- A line that makes the session the owner of a design ("its design is up to you") raised consolidation from under 10 % of runs to about 75 %, and output tokens by 70 to 80 %, with more unrequested changes. Use it for a planner's factoring. Leave it out of every job that should change little.
- An example: 2.1.7's factoring section was seven numbered steps in about 900 words. For Fable it is the inventory, the five-part bar, the core's principles, what each rule's entry must hold, and the counts on top. Restating, sorting and grouping are method, and go.

**What isn't known yet:** no run of ours has compared a short brief with a long one on Fable. Anthropic's page says that "some instructions and interventions designed for earlier models raised cost, caused side effects, or no longer had an effect"; it doesn't say "too prescriptive" in those words. 2.1.8 is the first test (D4).

**A start message** for a role session is the skill's §2 list, plus one line since M2: stop only processes whose pid you started (D18).

## 4. The numbers a plan uses

| Run             | Cost, % of the week | Changed lines in `engine/crates` | % per 1 000 lines | Note                                                 |
| --------------- | ------------------- | -------------------------------- | ----------------- | ---------------------------------------------------- |
| M0 (PR #70)     | 26.4                | 32 800 added                     | about 1.05        | the builder alone 0.6; the gate about 20 % on top    |
| M1 (PR #110)    | 49.9                | 24 000                           | 2.1               | three limit raises; gate 8's method changed mid-gate |
| 2.1.7 (PR #157) | 20.2                | 2 800                            | not comparable    | 84 questions, 10 rules; the planner 6.5 of it        |
| M2 (PR #168)    | 37.6                | 17 300                           | 2.2               | 1.66 after group 5; the Fable gate review 1.2 of it  |

- **One session:** a group's builder 1 to 3 %; a group review about 0.4 %; a fix review 0.15 to 0.4 %; a turn of T 0.5 to 1.4 %; a study or a rules session 0.25 to 1.1 %.
- **A group's builder and review together,** M2: 0.85 to 1.85 % per 1 000 lines; the highest was a small group of corner cases.
- **Wall time:** a group with its fix loop 2.5 to 5 hours; a gate-8 dispatch 1 to 2 hours with its queue.
- A Fable session costs 1.3 to 1.6 times the same tokens on Opus by `usage`'s figure, not twice: twice on what it writes and reads fresh, the same on cache reads (D10). M2's gate review was 0.95 % as Opus's tokens and 1.22 % as Fable's; this file's own session 0.22 % and 0.34 %.

## 5. Sources

- **Anthropic, read 2026-11-15** on `platform.claude.com/docs/en/`: `models/fable-5-2/overview`, `models/fable-5-2/whats-new-fable-5-2`, `models/fable-5-2/migration-guide` (its second half only as a summary), `about-claude/models/overview`, `build-with-claude/effort`, `build-with-claude/prompt-engineering/prompting-claude-fable-5-2`.
- **Claude Code, read 2026-11-15** on `code.claude.com/docs/en/`: `model-config`, `cli-reference`, `headless`, `sub-agents`, `skills`, `costs`, `settings`. The installed version is 2.1.325; its binary names `claude-fable-5-2` and `claude-opus-5-5`.
- **Not confirmed:** Claude Code's default effort for each model; how a Pro or Max plan meters Fable, and what a "reset" is; whether `--max-budget-usd` and `CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH` behave as their docs say on our runs; any guidance from Anthropic on emphasis words, examples, or step scripts against goals for Fable.
- **Ours:** PR #70 (M0) and the retro after it, PR #92 (the process round), #110 (M1), #139 (the check loops), #157 (2.1.7), #166 (the slow-test check), #168 (M2).
