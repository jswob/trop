# LLM Evals Harness

Measures the app's LLM prompts, multi-step flows and model settings against answer keys the user reviewed. [promptfoo](https://www.promptfoo.dev) is the runner and the only results screen; the harness picks the columns, guards the spending and decides whether a run can be trusted. Nothing here ships: once a variant wins, it moves into the app's API.

**Load the `llm-evals` skill before doing anything here.** It is the manual: the commands, the rules, how to run an experiment, what to do when a run fails. This file only says where things are.

The tasks are not here: they are the app's, in its evals folder (`apps/evals` in this repo), listed in its `evals.config.ts`.

## Layout

- `src/core/`: the harness. It knows no task and no adapter.
  - `task.ts` the task contract · `grid.ts` variants and concurrency · `config.ts` the generated promptfoo config
  - `provider.ts` N samples per row · `scorer.ts` per-case scores · `promptfoo.ts` contract types and `cachedCall`
  - `outcome.ts` what a failure becomes for promptfoo · `budget.ts` the spending limit · `hooks.ts` + `doctor.ts` the health check
  - `record.ts` + `integrity.ts` the gate after a run · `stats.ts` + `report.ts` the verdict · `key.ts` + `review/` the answer-key queue and review page
  - `cli.ts` the commands
- `src/adapters/openrouter/`: the one model client. Plain fetch, one attempt per call.
- `src/api.ts`, `src/testing.ts`: the package's two exports (`.` and `./testing`), the only things a task or an app's `evals.config.ts` imports. `src/boundary.test.ts` keeps core apart from the adapter.
- `src/cli.ts` (the `llm-evals` bin), `src/config.ts`, `src/promptfoo-entries.ts`: the composition root. The CLI runs in the app's evals folder, reads `evals.config.ts` there (`defineConfig({ tasks, title })`), and before every command writes the three files promptfoo loads into `.promptfoo/entry/`, each handing the app's tasks to core. `title` goes out as OpenRouter's `X-Title`.
- `src/selftest/`: the whole chain against a fake model (`bun evals:selftest`). Its `task.ts` is the smallest complete task.
- `skill/`: the `llm-evals` skill's source, declared in `package.json`'s `claude` field. `.claude/skills/llm-evals/` holds a copy of it plus the repo's own `PROJECT.md`; change the skill here, then run `bun toolkit sync`.
- In the app's evals folder (gitignored): `.runs/<run>/` with `config.json`, `meta.json`, `record.json`, `report.json`, `spend.log`; `.promptfoo/` with promptfoo's database, the answer cache and the entry files.

promptfoo is pinned in `src/cli.ts` and run through `npx`; it is not a dependency (its install is about 2.2 GB and the Dockerfiles copy `node_modules` into the API image). Types the harness needs from it are declared in `core/promptfoo.ts`.
