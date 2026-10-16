# Design Workbench

`@app/workbench-app`: the app bound to the design workbench. It stages the UI package (`packages/ui`) on fixtures, with no API, database, `.env` or auth, on the engine, `@jswob/design-workbench`. It deploys nothing, and the web never imports it.

**Load the `design` skill before a design round.** This folder is the design root: with the UI package's settled code, the one place a round may write. Read `BANS.md` beside this file first; `WORKBENCH.md` has the routes, the verbs, the file map and the local gotchas.

## The binding

- `workbench.config.ts`: the terminal's half. The port, the Vite config, the theme, the settled spaces (`packages/ui/src`), the bans, the census, the gates `check` runs and where the skill's copy lives.
- `workbench.setup.tsx`: the browser's half. The kit's catalogue, the places, the candidates, the specimens and the frames they stand on.

## Layout

- `kit/` — `index.ts`: the UI package's kit as one barrel, which a candidate or a board imports as `../kit`.
- `views/` — the places: the ground (`home`), each destination's states (`notes`, `account`), staged on fixtures, and what frames and walks them.
- `kit-docs/` — the kit's catalogue, the stress page, and `specimens/`, which gathers every `*.specimen.tsx` and makes their states frames.
- `fixtures/` — literal data the places draw on. Never imported by the UI package or the web.
- `proto/`, `theses/`, `probe/` — a round's candidates, their numbers, and the findings read and held. `theses/` and `probe/` are created when first needed.

## Commands

From the monorepo root:

| Command                 | What it does                                                                                                 |
| ----------------------- | ------------------------------------------------------------------------------------------------------------ |
| `bun designs`           | The bench on `http://localhost:5175`                                                                         |
| `bun workbench <verb>`  | Any verb; `bun workbench` alone prints them with their flags                                                 |
| `bun workbench check`   | The round's gate: eslint, types and tests here and in the UI package, census, the skill's copy, blast radius |
| `bun designs:test`      | This folder's tests                                                                                          |
| `bun designs:typecheck` | This folder's types                                                                                          |
