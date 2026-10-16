---
name: llm-evals
description: The llm-evals harness and the app's eval tasks. Load it before any work on either, and for any job that tests, compares or improves an LLM prompt, a multi-step flow or a model setting against a reviewed answer key - "test this strategy against the others", "find the best prompt", "try a cheaper model", "does more reasoning help", "run the evals", "why did the run fail or hang", "add an eval for X", "review the answer key", "label new test cases".
---

# LLM evals harness

The user asks a question about a prompt, a flow or a model. You run the variants against an answer key they reviewed and report which one wins and by how much. They inspect every answer in promptfoo's viewer and decide. promptfoo is the runner and the only results screen; the harness picks the columns, guards the spending and decides whether a run can be trusted.

The harness is a package; the tasks are the app's, in its evals folder, listed in `evals.config.ts` there. `PROJECT.md` beside this file says where that folder is in this repo.

## Words

- **Task**: a problem with an answer key and a scorer. `evals.config.ts` lists them; each has a folder with a `README.md`.
- **Strategy**: a flow plus a prompt version. **Model setting**: a model and how hard it thinks. Both are entries in the task's `task.ts`.
- **Variant**: one strategy on one model setting, written `strategy/model`. One column in the viewer. The current best is the **champion**.
- **Run**: variants answering cases, several samples each, one row per case.
- **Practice cases** (`testset`) are for iterating. **Held-out cases** (`heldout`) are for decisions.

## Commands

Run from the repo root. `OPENROUTER_API_KEY` in the root `.env` covers every model.

| Command                                                     | What it does                                                  | Cost                 |
| ----------------------------------------------------------- | ------------------------------------------------------------- | -------------------- |
| `bun evals:doctor <task> [--model a,b]`                     | Key, credit, one real call per model setting                  | a fraction of a cent |
| `bun evals:try <task> [selection] <task's flags>`           | Variants on cases typed by hand; no key, you read the answers | under a cent         |
| `bun evals:run <task> [selection]`                          | Without `--max-cost`: says what the run can cost, and stops   | free                 |
| `bun evals:run <task> [selection] --max-cost 0.5`           | Variants on the answer key, then the verdict                  | up to the limit      |
| `bun evals:report [run]`                                    | The verdict on a finished run, again                          | free                 |
| `bun evals:view`                                            | promptfoo's viewer; every run appears in it                   | free                 |
| `bun evals:key <queue\|dispute\|draft\|page\|apply> <task>` | The answer key's review queue                                 | free                 |
| `bun evals:selftest`                                        | The whole harness against a fake model, no network            | free                 |
| `bun evals:test`, `bun evals:typecheck`                     | The tasks' tests (model calls are faked), types               | free                 |

- **selection**: `--variants a/x,b/y` or `all`; or `--strategy a,b --model x,y` (every combination; a side left out comes from the champion). Nothing chosen: the champion alone.
- **run options**: `--split testset|heldout|all` (default `testset`), `--samples n`, `--first n`, `--group g`, `--note "what this run is for"`.
- Answers are cached. A run with an unchanged prompt and model costs nothing, so `--max-cost 0` is a free re-run. It stops (exit 2) at the first answer it would have to pay for.
- **With no money to spend:** skip `doctor` and `try` (they make real calls) and use `report` and `--max-cost 0` runs only.
- Run ids are the folder names in `.runs/`, beside `evals.config.ts`; `report` without one reads the latest.

## Rules

1. **No paid run without the user's limit.** Ask for a dollar limit, quoting what `run` says it can cost, and pass it as `--max-cost`. `doctor`, `try`, `report`, `selftest`, the unit tests and a `--max-cost 0` run are yours to run.
2. **The answer key is the user's.** It changes only through `bun evals:key apply`, from picks they made. When the key looks wrong, queue a dispute; never edit the key.
3. **Practice cases to iterate, held-out cases to decide.** A held-out run needs `--reason` and is logged. Once per decision; never tune on its result.
4. **A technical failure is never an answer.** Exit code 2, 3 or 4 means there are no numbers to report. Fix the cause and run the same command again.
5. **Every model call goes through `cachedCall`, and the client makes one attempt.** promptfoo's scheduler does the waiting and retrying. Never add a retry loop, never cache or score a failed call.
6. **Read the differing cases before naming a winner.** When the report says "too small to tell apart", that is the result.
7. **No answer-key case inside a prompt.** Pick examples that are not in the key; the task's prompt test checks.
8. **Never import promptfoo** (it runs from a pinned `npx`), never `promptfoo share`, never put the user's email in a prompt.

## Answering a question

1. **Pin down the question**: which variants, which task. Read the task's `README.md` and `EXPERIMENTS.md` first: the question may already be answered by a logged run, and `bun evals:report <run>` shows it again for free. A new prompt version or model setting is one new entry in the task's `task.ts`, beside the champion's; the champion stays untouched so both run as columns.
2. `bun evals:doctor <task> --model <settings in play>`.
3. `bun evals:try <task> <selection> <a few typed cases>`: are the answers the right shape, do the variants differ at all?
4. `bun evals:run <task> <selection>` to learn the cost, get the user's limit, then run it with `--max-cost` and `--note`. Include the champion so both answer the same cases.
5. **Read the report.** Ignore promptfoo's own "passed / failed" line above it: it counts rows, not answers. Per variant: share correct, cases ever wrong with their 95% interval, cases where all samples agree, what the answers cost when they were taken (cached or not), time. Against the champion: the difference with its interval, and every case where the two differ. Read each of those: what did each say, what does the key want, is the key right? A doubtful key entry goes to the queue (`bun evals:key dispute <task> <id> --note "…"`) and counts for neither side.
6. **To decide**, run the two on the held-out cases once: `--split heldout --reason "<the decision>"`.
7. **Report** (below), and add the round to the task's `EXPERIMENTS.md`; a re-run of a round already logged needs no new entry. Moving `champion` is the user's call.

**Improving a prompt** is this loop repeated on the practice cases, one change per round: read the champion's wrong answers, name one cause, fix the rule's wording rather than adding a special case for the missed case, run champion against challenger, check what the change broke. Stop when no miss is left that a prompt could fix, when two rounds brought no gain, or at the limit. Then delete the losing versions; `EXPERIMENTS.md` remembers them.

## When a run is not trusted

| Exit | Meaning                                              | What to do                                                       |
| ---- | ---------------------------------------------------- | ---------------------------------------------------------------- |
| 0    | Trusted. Wrong answers never change the exit code    | Report it                                                        |
| 1    | Usage: a missing limit, an unknown variant           | Read the message                                                 |
| 2    | Stopped by the spending limit                        | Tell the user what was spent; with their new limit, run it again |
| 3    | Too many rows failed for technical reasons           | `doctor`; wait if it is a rate limit; run it again               |
| 4    | Aborted: key, credit or model refused, or run killed | Read the health check's lines; fix; run it again                 |

Running the same command again is always safe: answers already taken come from the cache and are not paid twice.

- **"It looks stuck."** promptfoo is waiting out a rate limit: it pauses the whole queue as long as the provider asked. If the limit is real and low, lower `requestsPerMinute` / `tokensPerMinute` on the model setting; they set how many calls run at once.
- **A model rejects the request.** Usually the strict answer format is not supported. Try one case with `bun evals:try` and read the error in the viewer (filter Errors).
- **What a run really spent** is the "spent" line it prints, from `.runs/<run>/spend.log` (no file when nothing was spent). Its numbers are in `report.json` beside it.

## Reporting a result

Lead with the verdict in one sentence. Then the table from the report, the cases that differ with one line each on what went wrong, any key entry you queued as a dispute, what was spent, and the run id.

- "Cases ever wrong" matters more than the share correct: one wrong sample in three means the case is unreliable.
- "Fell back" counts only the safe answer given where the key wanted more. A safe answer the key wanted is correct.
- Step scores under the table are the task's own; its `README.md` says what each means.
- A variant that is never wrong but falls back a lot is only safe, not good. Report both.

## Rare jobs

- A new problem to evaluate: [add-task.md](add-task.md).
- New cases, a disputed answer, a rule change in the key: [review-key.md](review-key.md).

## Changing the harness itself

The layout is in the harness package's `CLAUDE.md`. After any change to the client, the provider, the failure mapping, the CLI or the pinned promptfoo version, run `bun evals:selftest`: it replays a rate limit, an outage, a broken reply, a dead key, the spending limit and a killed run against a fake model. Tasks import the harness only by its package name: its main export, and `/testing` in tests. The harness's own tests and types are the `llm-evals:test` and `llm-evals:typecheck` scripts of the repo that holds its source; in an app, `bun toolkit try @jswob/llm-evals` runs the app's tests against a change to it.
