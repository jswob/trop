# <Milestone or round>: <outcome, in spec words>

<!-- A milestone brief, from engine/templates/brief.md. It holds only this run's scope;
     the procedure is the skill engine-coordinator, the conventions engine/CONVENTIONS.md.
     How a brief is written, and to which model, is engine/PLAYBOOK.md §3: a brief for a
     Fable job (a spec round's lead, a planner, a critic, a study) gives the goal, the bar,
     the inputs and the output's form, and leaves the method to the session. -->

**You are the <run> coordinator,** a lead-role session: the job `coordinator`, Opus 5.5 at effort high (a spec round's is `round-lead`, Fable 5.2 at high: `--model claude-fable-5-2`). The master session wrote this brief on <date>. Invoke the skill `engine-coordinator` first; it holds the procedure. Your run folder is `tmp/<run>/`.

**How the user starts you,** in the main folder, on an up-to-date master:

```
cd /Users/jswob/projects/trop && TALLY_ROLE=lead claude --model claude-opus-5-5 --effort high
```

Then the user pastes: "You are the <run> coordinator. Your brief is tmp/active/<run>-brief.md. Read it and follow it."

## 1. The goal

**Build <milestone>** (`engine/spec/BUILD-ORDER.md` § <milestone>), all the way: T's tests, the plans at gate G-<run>, the build green, the gate, the user's merge, the tag `engine-<m>`. **Or stop at <limit> % of the week** (§5).

**Green** (§ <milestone>'s list, the DONE gates it names):

- …

**In scope:** the run's issue (filed at setup on `engine/templates/issue.md` if none), and: #…
**Not yours:** #… (where each goes, and why).

## 2. Who does what

| Who               | Job (`launch --job`) | Role, model, effort      | Does                                                                |
| ----------------- | -------------------- | ------------------------ | ------------------------------------------------------------------- |
| T                 | `tests`              | tests, Opus, xhigh       | the tests first; for each group's review, the fixes as tests        |
| the builder       | `builder`            | build, Opus, xhigh       | its plan; one fresh session per group                               |
| the group review  | `group-review`       | lead, Opus, high         | `engine-reviewer` on each group's diff                              |
| rules steps       | `rules-step`         | rules, Opus, high        | fuzz targets, benches, the kernel list, `gating`                    |
| the gate          | `gate`               | lead, Opus, xhigh, fresh | the evidence, fuzz, mutants, timing, the report                     |
| the gate's review | `gate-review`        | lead, Fable, high, fresh | the one review of the whole diff                                    |
| you               | `coordinator`        | lead, Opus, high         | the worktree, launches, merges in, picks, the pull request, the tag |

A spec round has a `planner` and a fresh `critic` (spec, Fable, high) and a `spec-writer` (spec, Opus, high) for the ruled patch. The table every job comes from is `engine/PLAYBOOK.md` §1.

## 3. What differs in this run

- <kernel or not; P8 rows (a timing run) or not; codes refcore never prints, which T lists before the builder starts>
- <the worktree: `~/projects/trop-worktrees/<branch>`; jobs and threads if another run shares the Mac>

## 4. The gates that wait for the user

- **G-<run>:** both plans, with the brief's questions, each with its recommendation.
- **The merge** of the run's pull request (`merge-pr` refuses `build/*`).

Everything else is yours: take the (Recommended) option and log it (the skill, §5).

## 5. Budget

**Limit <L> % of the week; wind down at <L − 1> %.** Measure with `engine/tools/usage tmp/<run>/ids.txt` after every chunk.

**The estimate,** from the last run's measured rates (`engine/PLAYBOOK.md` §4; the rows below are M0's, and M1 ran 2.1 % and M2 2.2 % per 1 000 changed lines all-in):

| Part                                     | Rate                                                | This run        |
| ---------------------------------------- | --------------------------------------------------- | --------------- |
| builder                                  | about 0.6 % per 1 000 lines                         | <lines> → <x> % |
| all-in (T, builder, reviews, steps, you) | about 1 % per 1 000 lines; ×1.3–1.5 for kernel code | <x> %           |
| the gate                                 | about 20 % on top of the all-in                     | <x> %           |
| **total**                                |                                                     | **<x> %**       |

A plan's size guess states its basis: each group from the measured lines of the nearest group already built (the playbook's D11). S's, M0's and M1's guesses from refcore's length ran 2–4× low.

## 6. Hard rules for this run

- the seal, CPU, git and chat rules of the skill (§9), in every start message;
- <anything the run adds>.
