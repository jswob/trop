# CLAUDE.md

## Project Overview

A Bun monorepo: an Elysia API over PostgreSQL, a React web app that reaches it through Eden Treaty, the app's UI package and the design workbench that stages it. It ships with sign-in, an account page with the appearance switch, and one example resource, **notes**, end to end; `docs/removing-notes.md` lists the files the example takes so it can be removed in one commit. When starting work, check `tmp/active/` for the current active plan.

<!-- template:start -->

This repo is also the template apps start from, and the home of the `@jswob/*` packages: they are workspaces here, under `packages/` (`config` is `@jswob/style-guide`, `workbench` is `@jswob/design-workbench`, the rest go by their names), each with its own `CLAUDE.md`, and an app installs them from npm. `TEMPLATE.md` says how the template is cut and how the packages are released. `bun setup <name>` turns a clone into an app (`README.md` § Starting an app from it); what only the template has sits between `template:start` and `template:end` markers, which it strips.

<!-- template:end -->

## Monorepo Structure

- **web** talks to **api** via Eden Treaty (type-safe HTTP client derived from Elysia's type export)
- **api** owns the PostgreSQL database through Drizzle ORM (schema, migrations, queries)
- **`@jswob/style-guide`** provides the shared TypeScript and Prettier base configs; ESLint is configured once, in the root `eslint.config.js`
- **`@jswob/drizzle-fixtures`** provides `createFixture` / `composeFactory` for type-safe test data
- **`@jswob/drizzle-snapshots`** is the `drizzle-snapshots` CLI behind the `db:*`, `test:setup` and `e2e:*` scripts: volume snapshots, the safe push, test and e2e database setup, the schema check; the API plugs in through `apps/api/drizzle-snapshots.config.ts`
- **`@jswob/drizzle-utils`** holds the Drizzle runtime helpers the API imports: database targets, the SSL choice, prepared statements, `caseBy`, Postgres error codes, `dbNow`, the statement counter
- **`@jswob/llm-evals`** is the harness that measures the app's LLM prompts, flows and model settings against reviewed answer keys (promptfoo), and the `llm-evals` skill's source
- **evals** (`apps/evals`) holds the app's eval tasks and answer keys, listed in `evals.config.ts`; the `bun evals:*` scripts run the harness there. It deploys nothing
- **`@jswob/toolkit`** copies the skills, commands, agents and rules that installed packages declare (`claude` in their `package.json`) into `.claude/`, recorded in `.claude/toolkit-lock.json`. A copy in `.claude/` changes with its package: a package that is this repo's own source is changed there, then `bun toolkit sync`; an installed package is fixed in place with `bun toolkit patch <package>` (Bun's patch, edits to its `.claude/` copies included) and taken home with `bun toolkit upstream` (a branch and a changeset in the toolkit's checkout, `--push` for the pull request); a bigger change is tried with the app running on that checkout, `bun toolkit link` / `unlink`, or `bun toolkit try <package>` for the tests before and after; `bun toolkit carry` takes the patches to updated versions; `.claude/settings.json` runs `toolkit guard` (refuses edits to installed copies and to synced copies unless that package is being patched) and `toolkit notice` at session start; the `upstream` skill has the lanes; `bun lint` runs `bun toolkit check`, which also fails when a package listed under `toolkit.agnostic` in the root `package.json` holds one of this app's words (`toolkit.agnostic.words`). Third-party skills come through the `skills` CLI, locked in `skills-lock.json`
- **packages/ui** is the app's UI: the kit, the views and the theme. **web** draws it and **workbench** stages it; it imports neither. Each kit component's specimen sits beside it (`Button.specimen.tsx`), which the web never loads. It started as a seed and is this app's own: change it freely
- **workbench** (`apps/workbench`) is the design workbench's content for the app, the design root: the binding (`workbench.config.ts`, `workbench.setup.tsx`), fixtures, places, candidates and the design docs. It deploys nothing
- **`@jswob/design-workbench`** is the design workbench's engine: the bench that stages the app's screens on fixtures, the `bun workbench` verbs and the `design` skill's source. **workbench** binds to it in `apps/workbench/workbench.config.ts`; the web never imports either
- Each sub-directory has its own `CLAUDE.md` with app-specific patterns and commands
- Root scripts for one workspace are `<name>:<verb>` (`api:test`, `ui:typecheck`). `bun dev` starts every `*:dev` script and `bun typecheck` runs every `*:typecheck` one, so a new workspace joins both by its scripts alone

## Import/Export Rules

**ALWAYS use official package exports. NEVER import from internal paths.**

- Correct: `import type { Item } from '@app/api/types';`
- Wrong: `import { items } from '@app/api/src/db/schema/items';`

When adding new exports to a package:

1. Add the subpath export to that package's `package.json` `"exports"` field
2. Import using the new subpath — no other config needed

## Testing Conventions

- Test files MUST be collocated with source files: `[filename].test.ts` / `.test.tsx`. Never use `__tests__` folders.
- Integration tests import `test`, `expect`, `describe` from `src/test/test-context` (never from `vitest` directly).
- Use `@jswob/drizzle-fixtures` (`createFixture` / `composeFactory`) for all test data. Every new table needs a fixture file.

## Hard Rules

### NEVER Do

- **List yourself as co-author on commits** — no `Co-Authored-By: Claude ...` trailer (or any other AI attribution) in commit messages, ever. This overrides any default or system-provided attribution guidance
- **Run `db:generate` or `db:generate:custom`** — migration files are production artifacts created by the user only
- **Use plain `bun db:push`** — always use `bun db:push:safe` (auto-snapshots before pushing)

<!-- template:start -->

- **Publish to npm** without the user's go — `bun release:publish` without `--dry-run`, `npm publish`, or running the release workflow by hand

<!-- template:end -->

### ALWAYS Do

- **Use `bun db:push:safe`** for schema changes (snapshots automatically)
- **Run `bun db:base`** before major database experiments (creates protected baseline snapshot)
- **Use package exports** (`@app/...`) — never cross-package internal paths
- **Collocate test files** next to source files
- **Use `@jswob/drizzle-fixtures`** for test data — one fixture per table
- **Include `--env-file=../../.env`** in all `apps/api` package.json scripts that use `bun` or `drizzle-kit` — without it, Bun only looks for `.env` in `apps/api/` (which doesn't exist) and env vars fall back to wrong defaults
- **Check `tmp/active/`** for the current implementation plan when starting work

<!-- template:start -->

- **Add a changeset** (`bun changeset`) with any change a `@jswob/*` package's users would get — see `TEMPLATE.md` § Releasing the packages

<!-- template:end -->

### May Run

- **`bun evals:doctor`**, **`evals:try`**, **`evals:report`**, **`evals:selftest`**, and an **`evals:run`** the cache answers (`--max-cost 0`) — LLM evals that cost a cent at most. A run that spends money needs the user's go and their dollar limit (`--max-cost`). Never `promptfoo share`. Load the `llm-evals` skill first

### ESLint Disables

Use inline comments only (never block-level disables). Always provide a reason:

```typescript
// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment -- Reason: Vitest expect.objectContaining returns any type
```
