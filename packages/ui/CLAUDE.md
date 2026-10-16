# UI

`@app/ui`: the app's kit, views and theme. The web (`apps/web`) draws it and the design workbench (`apps/workbench`) stages it on fixtures; it imports neither, nor the API.

**A seed, and the app's own.** The template ships this package as a starting point: a frame, a kit and a theme with no domain of their own, and one example page (`views/notes.tsx`). An app made from the template owns its copy from then on — it gives the package its own look, its own components and its own views, and nothing flows back.

Change a drawing through the `design` skill: this package is the workbench's settled code, so a design is promoted into it rather than edited here by hand.

## Layout

- `src/kit/` — the primitives (`SplitShell`, `RailShell`, `PagedList`, `MonthPicker`…). The kit knows nothing of the views. Its index, `kit/index.ts`, re-exports `kit/core.ts`: the controls, the page and its sections, the paged list, the month and its periods, the rail and the split (`RailShell`, `SplitShell`), the waits, and `kit/viewing` (the shell's word that its column is read, not worked).
- `src/views/` — the frame and the pages it ships with (`frame.tsx`, `sign-in.tsx`, `read-failed.tsx`, `account.tsx`, `notes.tsx`). The `Frame` draws only the destinations it is handed and is told the ground's name (`home`) and where it rests (`resting`); `views/destinations` holds the app's.
- `src/lib/` — what both lean on (`plural`). It knows neither the kit nor the views.
- `src/theme.css` (the `ui-` tokens), `scroll.css`, `glaze.css`, `document.css` — the stylesheets, imported by path (`@app/ui/theme.css`). The web and the workbench each add `src/` to their own Tailwind scan.
- `src/kit/<Name>.specimen.tsx` — a component's page on the workbench, beside the component: its states, props, parts and what it holds and refuses, as the file's default export. The workbench finds it by its name; the web never loads one (its Tailwind scan leaves them out, ESLint and `src/boundaries.test.ts` forbid the import). A specimen's scenes are its own and never the app's; the workbench's fixtures may borrow them.

## Rules

- **Package exports only.** `@app/ui/kit` is the kit's index, and any file is reachable as `@app/ui/<space>/<file>`. Inside the package, a sibling is imported relatively or by the same specifier.
- **The walls are a test.** `src/boundaries.test.ts`: a file imports only this package and what its manifest names (runtime dependencies for shipped code; dev dependencies too for a test or a specimen), the kit imports no view, `lib/` imports neither, and nothing shipped imports a specimen.
- **Tests sit beside their code**, as `<name>.test.ts(x)`. A test that needs the workbench's fixtures stays in the workbench and reaches this package through its exports.

## Commands

From the monorepo root:

| Command            | What it does         |
| ------------------ | -------------------- |
| `bun ui:test`      | This package's tests |
| `bun ui:typecheck` | This package's types |
