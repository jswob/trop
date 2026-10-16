# llm-evals in this repo

This repo's half of the skill. The other files are copies of the harness package's `skill/`, and
never change alone: where the package is this repo's own source they are changed there, then copied
with `bun toolkit sync`; where it is installed, `bun toolkit patch @jswob/llm-evals` first (the
`upstream` skill). This one is written here and never overwritten.

- **The evals folder** is `apps/evals` (`@app/evals`): `evals.config.ts` lists the
  tasks, one folder each under `apps/evals/tasks/`. Its `.runs/` and `.promptfoo/` are gitignored.
- **The harness** is `@jswob/llm-evals`; its `CLAUDE.md` has the layout.
- **The commands** are the root `bun evals:*` scripts, which run the CLI in `apps/evals` with the
  root `.env`.
