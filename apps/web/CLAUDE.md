# Web Application

React SPA frontend with Vite, TanStack Router, TanStack Query, Eden Treaty. Dev port 5173.

The app's UI is the `@app/ui` package (`packages/ui`), with `apps/workbench/BANS.md` beside it —
**read it before writing or restyling any UI**, and change a drawing through the `design` skill,
never by hand in a route.

## Project Structure

- `@app/ui/kit` — the primitives (SplitShell, PagedList, fields, buttons…), and `@app/ui/views/*`
  — the pages' drawings and the frame (`frame.tsx`). Both live in `packages/ui`, which knows nothing
  of the app; its own `CLAUDE.md` says what it holds.
- `src/frame/declare-channel.tsx` — the channel a page declares through, for any shape of
  declaration (`declareChannel<D>()`), and the fields every frame reads (`FrameDeclared`).
  `src/frame/declare.ts` binds it to the app's (`PageDeclared`); `src/frame/pages.ts` says what
  each page tells the frame. `src/frame/adapters/` holds sign-in's server and the failed-read watch.
- `routes/-components/frame.tsx` is the one `AppFrame`; the ground it stands on — the home page, the
  rail's facts and the slug's reading — is `ground.tsx` beside it.
- `src/routes/` — file-based routes. `__root.tsx` draws the one `AppFrame` around every page;
  `<dest>.tsx` is a destination, `<dest>.<kind>.$id.tsx` a record nested in it, `$.tsx` the redirect
  for unknown addresses. A page's data and presses are a hook in `routes/-components/`
  (`use-notes-page.ts`).
- `src/hooks/api/` — TanStack Query hooks over Eden Treaty.
- `src/routeTree.gen.ts` — auto-generated route tree (do not edit).
- The design workbench is not here: it is `apps/workbench`, which stages `packages/ui` on fixtures
  with no API, database or `.env`, on the engine, `@jswob/design-workbench`. Run with `bun designs`,
  drive with `bun workbench <verb>`; see `apps/workbench/WORKBENCH.md`. The app never imports the
  workbench, its engine or a specimen (`src/frame/boundaries.test.ts` and ESLint).

## The example resource

**Notes** shows every layer a resource takes, end to end: the API's table, fixture and module, the
hooks in `src/hooks/api/useNotes.ts`, the route `src/routes/notes.tsx` with its hook, the page
`packages/ui/src/views/notes.tsx` and its workbench place. `docs/removing-notes.md` lists every file,
so an app removes it in one commit. Copy its shape for the app's first resource.

## Key Patterns

- **A page declares, the frame draws.** A page calls `useDeclare` for what it puts beside its body
  (reading, nest, hold, errand, facts); it never draws chrome. The frame is mounted once and stays
  mounted.
- **The rail draws the app's own list**: `APP_DESTINATIONS` in `src/frame/pages.ts`, the
  destinations with a route. A destination the workbench stages before the app has a page for it
  joins the list when its route lands; `pages.test.ts` fails while the two disagree.
- **Addresses**: a destination is `/<dest>` (`destPath`), a nested record `/<dest>/<kind>/<id>`
  (`nestPath`). A link on a page is `nestHref`/`recordHref`, which the frame reads back.
- **Query hooks**: in `hooks/api/`, export `queryOptions()` + `useQuery()` hooks; keys are
  hierarchical factories in `hooks/api/keys.ts`. A failed read throws `ApiError` with the HTTP
  status, which the frame reads to say a read failed.
- **API client**: Eden Treaty in `lib/api.ts`, typed against `App`. Error text via
  `extractErrorMessage()`.
- **Styling**: Tailwind CSS v4 with the UI's `ui-` tokens (`packages/ui/src/theme.css`, imported by
  `src/index.css` with the UI package's other sheets). No other palette exists in the app.
- **Tailwind content scope**: `src/index.css` pins detection with `@import 'tailwindcss' source('.')`,
  then adds `packages/ui/src` back with `@source`, without its specimens and tests (`@source not`).
  Without the pin Tailwind scans all of `apps/web`; without the exclusions a specimen's utilities
  ship in the production bundle.
- **Icons**: `lucide-react`.
- **Path alias**: `@/` maps to `src/`.
- **Query config**: stale 5min, GC 10min, retry 3x (no retry on 4xx), window focus refetch disabled.
- **Feature components**: route-specific code lives in `routes/-components/` (TanStack Router ignores
  `-` prefixed dirs).
- **Testing**: collocate tests as `[name].test.ts(x)`. For hooks, mock `@/hooks/api` and wrap in
  `QueryClientProvider` with a fresh `QueryClient`.

## Environment Variables

| Variable       | Description          | Default                 |
| -------------- | -------------------- | ----------------------- |
| `VITE_API_URL` | API server URL       | `http://localhost:3001` |
| `WEB_PORT`     | Vite dev server port | `5173`                  |

## Development Rules

### Testing

- **ALWAYS** write comprehensive tests for developed functionality considering our best practices
- **ALWAYS** follow TDD (red-green-refactor): write a failing test first, implement the minimum code to pass, then refactor
- **ALWAYS** write tests incrementally per plan phase — not all at the beginning or all at the end
- **NEVER** write E2E (Playwright) tests unless explicitly requested by the developer

### Documentation

- **ALWAYS** prefer JSDoc over inline comments — avoid inline comments whenever possible
- JSDoc should describe **what** a function does, not implementation details
- JSDoc should be concise; **NEVER** include `@example` blocks

### ESLint

- When disabling an ESLint rule inline, **ALWAYS** provide a reason in the comment

### Session End Checklist (web package only)

At the end of each Claude session, run these commands **in order** from monorepo root:

1. Format the files you touched (`bunx prettier --write <files>`) — never a repo-wide `bun format`, which corrupts `.claude/skills` markdown
2. `bun lint:fix` — lint and auto-fix (from web dir or scoped to web)
3. `bun test:web` — run all Vitest tests (excludes E2E)

## Related Documentation

See [README.md](README.md) for user interface features and roadmap.
