# Project

This repo's facts for the React skill. Where they and the skill disagree, these win.

## Where the UI lives

- **The UI is a package**: `@app/ui` (`packages/ui`). It holds the kit
  (`src/kit/`), the views (`src/views/`) and the theme.
  It imports neither the web nor the API. Read `packages/ui/CLAUDE.md`.
- **The web has no `src/components/`.** `apps/web` holds the routes (`src/routes/`, route-only code
  in `routes/-components/`, a page's data and presses as a hook there), the data hooks
  (`src/hooks/api/`) and the frame's wiring (`src/frame/`). Read `apps/web/CLAUDE.md`.
- **A drawing changes through the `design` skill**, never by hand in a route. Read
  `apps/workbench/BANS.md` before writing or restyling any UI.
- A page tells the one frame what it puts beside its body with `useDeclare`
  (`src/frame/declare.ts`); it never draws chrome. The example resource, Notes, shows every layer.

## In place of the skill's defaults

- **No CVA, no `cn()`.** Classes are Tailwind v4 strings built from constants and template literals
  (`FOCUS`, `CONTROL_H` in the kit), on the `ui-` tokens of `packages/ui/src/theme.css`. No other
  palette exists.
- **No React Compiler.** Memoize by hand where a render is hot (`useMemo`/`useCallback`), as the
  frame's ground does.
- **Zod 3** (`^3.24`).
- **Icons** are `lucide-react`. The `@/` alias is `apps/web/src`.
- **Queries**: `queryOptions()` + `useQuery()` hooks in `src/hooks/api/`, keys from the factories in
  `src/hooks/api/keys.ts`, the barrel kept sorted. The Eden client is `src/lib/api.ts`; a failed read
  throws `ApiError` (`src/lib/api-utils.ts`), and `extractErrorMessage()` gives its text.
- **Tests** sit beside their code. jest-dom's matchers are not typed under `tsc` here: assert on
  `classList`, `textContent` and the queries instead. Mock `@/hooks/api` and wrap hooks in a fresh
  `QueryClientProvider`. No Playwright test unless asked.

## Commands

From the repo root:

| Command                                      | What it does                                      |
| -------------------------------------------- | ------------------------------------------------- |
| `bun web:test`                               | The web's tests                                   |
| `bun ui:test`                                | The UI package's tests                            |
| `cd apps/web && npx tsc -b --noEmit --force` | The web's types (`web:typecheck` can skip)        |
| `bun ui:typecheck`                           | The UI package's types                            |
| `bunx prettier --write <files>`              | Format the files you touched — never `bun format` |
