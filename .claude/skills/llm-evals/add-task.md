# Add a task

A task is a problem with an answer key, a scorer and at least one strategy. The harness's `src/selftest/task.ts` is the smallest complete one (100 lines); its `src/core/task.ts` is the contract.

1. **Settle with the user what "right" means** before writing a prompt: the input, the decision the app will store, and the outcomes. Every answer is one of: correct, fell back (the safe answer where more was wanted), wrong (would show something incorrect), invalid. A task with no safe answer never uses "fell back".
2. **Write the labelling rules** as `key/GUIDE.md`: a handful of rules a person can apply, with a tie-breaker. If the rules need a long list of exceptions, the task is not settled yet.
3. **Draft the cases.** Aim for 150–250, grouped by the trap each one tests (`group`), from real data where there is any, and balanced between cases where the safe answer is right and cases where it is not. Mark about two thirds `heldout`.
4. **Get the key reviewed.** New cases go through the queue and the review page, never straight into the key: [review-key.md](review-key.md).
5. **Create `tasks/<name>/`** in the evals folder, importing the harness only by its package name (and its `/testing` export in tests):
   - `task.ts`: `defineTask({...})` with `cases`, `tryCases`, `input`, `strategies`, `models`, `champion`, `samples`, `canary` (an easy case that is not in the key), `show`, `judge`, `keyVersion`, `dir`, `review`.
   - the first strategy: its prompt, its strict answer schema, and the function that turns the model's answer into a decision. Every model call goes through `cachedCall`.
   - `key/`: the answer key with a `version`, and its loader.
   - `README.md`: what the task decides, its strategies and champion, anything special about its models.
   - tests beside each file: the judge's outcome table, the prompt-leak check (no key case in any prompt), the strategy with faked model calls (`fakeModels`).
6. **Register it** in `evals.config.ts`.
7. **Check without spending:** `bun evals:test`, `bun evals:typecheck`, then `bun evals:doctor <name>` and a `bun evals:try`.
8. The first run on the key needs the user's limit.

## Model settings

`{ model, effort?, expectByok?, estCostPerCall, requestsPerMinute, tokensPerMinute, tokensPerCall, secondsPerCall }`. Pin the model id (never a `latest` alias). The limits are the provider account's; they decide how many calls run at once. After the first real run, replace the estimates with what was measured.
