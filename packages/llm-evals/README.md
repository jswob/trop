# @jswob/llm-evals

Measures an app's LLM prompts, multi-step flows and model settings against answer keys a person
reviewed. [promptfoo](https://www.promptfoo.dev) runs the cases and is the results screen; the
harness picks the columns, guards the spending and decides whether a run can be trusted.

```bash
bun add -d @jswob/llm-evals
```

Requires Bun. It ships TypeScript source. promptfoo is run through a pinned `npx`, not installed.
Models are reached through OpenRouter: `OPENROUTER_API_KEY` in the environment.

## Plug in

An evals folder (say `apps/evals`) with `evals.config.ts`:

```ts
import { defineConfig } from '@jswob/llm-evals';

export default defineConfig({
	title: 'my-app evals', // sent to OpenRouter as X-Title
	tasks: {}, // each task: tasks/<name>/task.ts, listed here
});
```

and a script that runs the bin there with the environment loaded:

```json
"evals": "bun --env-file=../../.env llm-evals"
```

A task imports only `@jswob/llm-evals` and, in its tests, `@jswob/llm-evals/testing`.

## Commands

| Command                                           | What it does                                               |
| ------------------------------------------------- | ---------------------------------------------------------- |
| `doctor <task>`                                   | Key, credit, one real call per model setting               |
| `try <task> …`                                    | Variants on cases typed by hand                            |
| `run <task> …`                                    | Without `--max-cost`, says what the run can cost and stops |
| `run <task> … --max-cost 0.5`                     | Variants on the answer key, then the verdict               |
| `report [run]`                                    | The verdict on a finished run, again                       |
| `view`                                            | promptfoo's viewer                                         |
| `key <queue\|dispute\|draft\|page\|apply> <task>` | The answer key's review queue                              |
| `selftest`                                        | The whole harness against a fake model, with no network    |

Answers are cached, so an unchanged run costs nothing (`--max-cost 0`). A run stops at its
spending limit, and a run with too many technical failures reports no numbers.

## Claude Code

The package ships the `llm-evals` skill, the manual for all of the above. `@jswob/toolkit` copies
it into `.claude/skills/` (`bunx toolkit sync`).

## License

MIT
