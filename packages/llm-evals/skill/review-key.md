# Review the answer key

Everything that needs the user's judgement waits in one queue per task (`key/queue.json`) and leaves it one way: a review page where they pick.

## What goes in the queue

| Source    | When                                                         | How                                                                      |
| --------- | ------------------------------------------------------------ | ------------------------------------------------------------------------ |
| `dispute` | An answer in the key looks wrong, to a model or to the user  | `bun evals:key dispute <task> <case id> --note "…" [--propose <option>]` |
| `draft`   | New cases: a new task, a fresh held-out set, a gap you found | write a JSON list, then `bun evals:key draft <task> <file>`              |
| `rule`    | A labelling rule changed: every case it touches goes back    | same as draft, with `"source": "rule"`                                   |

A queue item: `{ "id": "<case id>", "source": "dispute", "note": "<the doubt, in a sentence the user can judge>", "proposed": "<option>", "agreed": false, "draft": { … } }`.

- `note` is what the user reads. Say what the doubt is and what each choice would mean, not what you would pick.
- `agreed: true` only when nothing is in doubt. Those appear as one "confirm all" block. Anything else is shown first, with nothing pre-selected, so the user is not nudged.
- `draft` is for a case not in the key yet: what the task needs to add it (its split, its group, and so on).
- `bun evals:key queue <task>` lists what waits, and how many held-out runs were made on this key version.

Before queueing new cases, get candidate answers cheaply (`bun evals:try` on two different variants): where they agree and the rules are clear, mark `agreed`; where they differ, that is the doubt to describe. Keep new cases out of every prompt.

## The review

1. `bun evals:key page <task>` writes the page to `.runs/<task>-review.html` in the evals folder.
2. Publish it as an Artifact **with the `db` capability** (load the `artifact-capabilities` skill first). The page saves the user's picks in its database as document `review/picks`; without the capability it falls back to a "copy picks" box.
3. Give the user the link. When they say they are done, read the picks with the `ArtifactData` tool (`get`, collection `review`, document `picks`) and save the `picks` object to a file.
4. **Check the picks before applying.** Read them against `key/GUIDE.md` and against each other. A pick that contradicts a rule, or two picks that treat the same kind of case differently, go back to the user as a question; that is how label mistakes are caught. If a pick shows a rule is wrong, the rule changes first (with the user), then every case it touches is queued again as `rule`.
5. `bun evals:key apply <task> <picks.json>`. It refuses the whole batch if anything cannot be applied, and says what. A changed answer bumps the key version and is written in `key/CHANGELOG.md`; picks that only confirm the key keep the version. Format the key afterwards with prettier.
6. Run `bun evals:test`.

## After the key changed

- Scores from different key versions are never compared. Run the champion again on the new version before the next experiment; cached answers are re-scored for free.
- **Refresh the held-out set** when it is worn. After a handful of decisions on one version, fold the old held-out cases into the practice set and have the user review fresh ones.
