# Evals

This app's LLM eval tasks, measured by the harness, `@jswob/llm-evals`. **Load the `llm-evals`
skill before doing anything here.**

- `evals.config.ts` lists the tasks and the name OpenRouter shows for their calls.
- `tasks/<task>/`: one folder per task, with its `README.md`, `task.ts`, `key/` and
  `EXPERIMENTS.md`. A task imports the harness only as `@jswob/llm-evals` (and
  `@jswob/llm-evals/testing` in tests).
- `.runs/` and `.promptfoo/` (gitignored): run records, promptfoo's database and answer cache.
- The root `bun evals:*` scripts run the CLI here with the root `.env`; `bun evals:test` and
  `bun evals:typecheck` check the tasks.
- Nothing here ships. A winning prompt moves into `apps/api`.
