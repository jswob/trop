# Design Workbench Engine

A second dev server that stages an app's screens and components on fixtures, with no backend, and the verbs that measure them. It knows nothing of the app it draws: no token, no component, no path. The app hands itself over in two files, and this package is meant to be lifted into another repo as it is.

**Load the `design` skill before a design round.** A round writes the app's design root, never this package. A change here is work on the workbench: it needs its own tests, and a full `bun workbench shots` before and after when it touches anything a frame loads.

## The binding

- `workbench.config.ts` (`defineWorkbench`, `src/config.ts`): the terminal's half. Plain data, read by the verbs and by the server before anything is bundled: the port, the app's Vite config, the stubs, the theme, the settled spaces, the bans file (with the words `ban find` reads it by and the sections every round gets), the census, the gates `check` runs, where the skill's copy lives. Every path is relative to the config file, and the folder it stands in is the design root.
- `workbench.setup.tsx` (`defineSetup`, `src/lib/setup.ts`): the browser's half. The app's own modules: the kit's catalogue, the places, the candidates, the specimens, what wraps a frame.

The repo's root `CLAUDE.md` says where its binding lives.

A file named `<Name>.specimen.tsx` is a component's page on the bench, standing beside the component wherever the app keeps it; the setup hands the bench the specimens it gathered. The engine never counts one among the design root's sources (`isSpecimen`, `src/sources.ts`) nor among the callers of what it draws (`usesOf`, `src/cli/callers.ts` and `src/lib/kit-callers.ts`).

## Layout

- `src/index.ts`, `src/config.ts`, `src/cli/thesis.ts`: the three package exports (`.`, `./config`, `./thesis`). A design root imports nothing else.
- `src/vite.ts`: how the bench is served. The app's Vite config is read, not merged: its plugins and aliases are taken, its server, `define` and environment are not. The binding reaches the browser as `virtual:workbench/{setup,styles.css,host,sources,spend}`.
- `src/index.html`, `src/main.tsx`, `src/router.ts`, `src/routes/`: the bench's own document, root and routes. `src/routeTree.gen.ts` is generated; do not edit it.
- `src/bench/`: the chrome around a frame. Plain CSS in `wb` names (`bench.css`), its own face (`fonts/`). Nothing in it loads inside a frame.
- `src/components/`: `FrameDocument` is what a frame is; the frame route and a board both draw it. What the engine draws inside a frame is styled inline, because the app's Tailwind does not scan this package.
- `src/lib/`: the registry that crosses the four spaces, the `Subject` shape, frame addresses, viewports, the frame clock (`motion.ts`), a board's frames.
- `src/cli/`: the verbs. `main.ts` is the `workbench` binary; each verb's logic that needs no browser or filesystem sits in a file with a test beside it (`*-files.ts`, `select.ts`, `steps.ts`).
- `src/probe/`: the measured floor and the cross-drawing census.
- `src/test/doubles/`: what `virtual:workbench/*` resolves to under Vitest, so the bench's tests run without an app.
- `skill/`: the design skill's source, every file but `PROJECT.md`, declared in `package.json`'s `claude` field. `@jswob/toolkit` copies it into `.claude/skills/design/` and records it in `.claude/toolkit-lock.json` (`bun toolkit sync`, or `bun workbench skill` for this package alone), and `check` fails while the copy and the source differ. Edit the skill here, never in the copy.

## Commands

From the monorepo root:

| Command                   | What it does                                                        |
| ------------------------- | ------------------------------------------------------------------- |
| `bun workbench <verb>`    | Any verb, bound to the app's `workbench.config.ts`                  |
| `bun workbench`           | Prints every verb with its flags                                    |
| `bun designs`             | `workbench serve`: the bench, on the port the config names          |
| `bun workbench:test`      | This package's tests                                                |
| `bun workbench:typecheck` | This package's types                                                |
| `bun workbench check`     | The design root's gates, the census, the skill's copy, blast radius |

## Rules

- **Nothing of the app in here.** No app name, token, component or path, in code, tests or messages. Tests use invented names (`--color-ink-2`, `kit/Meter.tsx`, `apps/ledger`). The package scope in an import is the one exception.
- **The app never imports this package** (ESLint forbids it in the app's source), and this package imports the app only through the binding.
- **The router plugin stays on in the server.** It code-splits only the routes its own generator has listed, so with generation off every frame loads the whole bench. Installed from npm, the engine is never written: the generated tree goes to the app's `node_modules/.vite-workbench/` (`routeTreeFile`), because Bun may hardlink the package from its shared cache.
- **The port is strict.** A workbench that drifts to the next free port serves a stale build to every verb that addresses it by number. Under Bun a second server can also bind a port that is taken, and the two split the requests, so `serve` asks the port (`portAnswers`) before Vite starts.
- **The engine's version is recorded with the skill's copy.** A change to `skill/` carries a changeset like any other, and the release stamps the new version on the copy (`toolkit sync` in `bun release:version`).
