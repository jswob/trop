# Project binding

**The only file in this skill that is this repo's own.** Every other file here is a copy of the
engine's skill, the `skill/` folder of `@jswob/design-workbench`, and never changes alone: where the
package is this repo's own source, change it there and run `bun workbench skill` (or
`bun toolkit sync`); where it is installed, `bun toolkit patch @jswob/design-workbench` first (the
`upstream` skill). `bun workbench check` and `bun toolkit check` fail while the copy and the source
differ.

If this file describes a project that does not match the repo you are in, stop and say so rather
than guessing — a design loop pointed at the wrong directory writes into the wrong universe.

---

## This repo

The workbench reads its binding from `apps/workbench/workbench.config.ts`: the port, the Vite
config, the theme, the settled spaces, the bans file and the gates `check` runs. What that file
says is not repeated here.

|                            |                                                                                                                                                                                                                                                                                                                                                                                                               |
| -------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Design root**            | `apps/workbench/`, plus the settled code in the UI package that the web draws: `packages/ui/src/kit/` (each component with its `<Name>.specimen.tsx`), `packages/ui/src/views/` and the stylesheets `packages/ui/src/{theme,scroll,glaze,document}.css`. Rule 0's blast radius — never write outside it                                                                                                       |
| **Not the design root**    | `@jswob/design-workbench` — the engine: the bench's chrome, the verbs and this skill's source. A design round never writes there; a change to it is work on the workbench. All of `apps/web/` — `src/frame/`, `src/routes/`, hooks — is the app's                                                                                                                                                             |
| **Baseline**               | A place's earlier drawing, argued against on a board                                                                                                                                                                                                                                                                                                                                                          |
| **Start the workbench**    | `bun designs` from the monorepo root → `http://localhost:5175`                                                                                                                                                                                                                                                                                                                                                |
| **The verbs**              | `bun workbench <verb>` from the monorepo root; `WORKBENCH.md` § The verbs lists them, and `bun workbench` alone prints their flags                                                                                                                                                                                                                                                                            |
| **Workbench instance doc** | `apps/workbench/WORKBENCH.md` — routes, file map, local gotchas                                                                                                                                                                                                                                                                                                                                               |
| **The system**             | None settled yet: the UI package is a seed. When a system settles, write it down as `apps/workbench/SYSTEM.md` and name it here                                                                                                                                                                                                                                                                               |
| **The bans**               | `apps/workbench/BANS.md` — what not to. Read every session, appended every round. By subject: `bun workbench ban find --for density`; appended with `bun workbench ban add`                                                                                                                                                                                                                                   |
| **The round's gate**       | `bun workbench check` — design-scoped eslint, the workbench's and the UI package's types and tests, census, the skill's copy and blast radius. Run it constantly                                                                                                                                                                                                                                              |
| **The session's gate**     | `bun lint` · `bun web:test` · `bun ui:test` · `bun designs:test`, whole repo. Once, before calling a round settled: a design change can break a shared type                                                                                                                                                                                                                                                   |
| **Probes**                 | `bun workbench probe` — `--space proto` by default; `--frames id:variant,…`, `--widths …`, `--all`, `--quiet`, `--full`, `--accept`. Baseline: `apps/workbench/probe/baseline.json`                                                                                                                                                                                                                           |
| **The thesis's numbers**   | `bun workbench conform <id>` against `apps/workbench/theses/<id>.thesis.ts`; `--all` prints what held too. `new <id>` creates the file; its numbers are written when the thesis is approved, before the build. Parts are found by `data-part="<name>"`, or a selector the file names. Seconds                                                                                                                 |
| **The sweep**              | `bun workbench sweep <id>[:<state>]` (or `--space views`). `--every` presses long runs whole, `--accept` writes a read finding to `apps/workbench/probe/sweep-baseline.json`. About 2.5s a state and width                                                                                                                                                                                                    |
| **Cross-drawing census**   | `bun workbench census` — every `SlotState` and the files that paint it; exits non-zero when one is painted from two places                                                                                                                                                                                                                                                                                    |
| **Boards**                 | Written to the job scratchpad (`$CLAUDE_JOB_DIR/tmp/<board>/`), never the design root. **Drawn with the board kit**: `bun workbench board <folder>`, at the workbench's widths (390 × 844, 1440 × 900). A new drawing is `<id>.tsx` in the folder, importing `../kit`, `../fixtures/…` and `@jswob/design-workbench`, and exporting `<camelId>: Subject`. `apps/workbench/WORKBENCH.md` § Boards has the rest |
| **A component's page**     | A declared specimen, `<Name>.specimen.tsx` beside the component (`packages/ui/src/kit/Button.specimen.tsx`), its default export typed against the component and drawn by the bench, which finds it by its name. `bun workbench kit <id>` prints one                                                                                                                                                           |

## Where this repo starts

The UI package is a **seed**, copied once from the app this template was cut from, and this app's
own from then on. Nothing settled it for this app, so the first rounds may change anything in it.
Two things hold all the same:

- **The kit is the base to build on**, not something to avoid. `packages/ui/src/kit/` (through
  `apps/workbench/kit/`'s barrel) and the `--color-ui-*` tokens are the system until a round
  replaces them, and candidates are expected to use them.
- **There is one token namespace, `ui-`.** The workbench's own chrome is the engine's,
  `src/bench/` in `@jswob/design-workbench`, in `wb` variables no frame loads, and it is not the app's. Never
  draw a candidate in it.

Number rounds from 1 in the bans' rounds table, and keep the numbering continuous, including
letter-suffixed correction rounds (`3b`, `3c`).

## Standing project rules that bite here

From the repo's `CLAUDE.md`, restated because a design session trips over them:

- Test files are collocated, `[name].test.ts`. Test paths are relative to the package's own root —
  `fixtures/foo.test.ts`, not `apps/workbench/fixtures/foo.test.ts`.
- ESLint disables are inline with a stated reason. `bun lint` runs with `--max-warnings 0`.
- Prettier here is `useTabs`, `printWidth: 100`, `singleQuote`, `arrowParens: avoid`. Write it
  roughly right and let `bun lint:fix` settle it.
- `noUncheckedIndexedAccess` is on — an index into an array is possibly `undefined`.
- Never write E2E (Playwright) tests unless explicitly asked.
