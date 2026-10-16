# Project binding

**The only file in this skill that is this repo's own.** Every other file here is a copy of the
skill `@jswob/toolkit` ships (`claude/skills/elysia-drizzle-api-best-practices/` in it), and never
changes alone: where the package is this repo's own source, change it there and run
`bun toolkit sync`; where it is installed, `bun toolkit patch @jswob/toolkit` first (the `upstream`
skill). `bun toolkit check` (part of `bun lint`) fails while a copy and its source differ.

If this file describes a project that does not match the repo you are in, stop and say so rather
than guessing.

---

## Where the API is

The API is `apps/api` (`@app/api`): Elysia on Bun, Drizzle over postgres.js,
PostgreSQL, dev port 3001. Its `CLAUDE.md` has the module patterns; the repo's root `CLAUDE.md` has
the hard rules. Paths below are relative to `apps/api` unless they start at the repo root.

|                            |                                                                                                                                                                                                                                                                           |
| -------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Shell and composition**  | `src/index.ts` is the shell (logger, CORS, security guard, sign-in, wake, `/health`, the `App` type). Modules are mounted with `.use()` in `routes(db)` in `src/routes.ts`                                                                                                |
| **Modules**                | `src/modules/<name>/`: `index.ts` exports `create<Name>Route(db)` with its prefix; `service.ts`, `validation.ts` and helpers beside it; tests collocated                                                                                                                  |
| **Schema**                 | One file per table in `src/db/schema/`, shared columns in `common.ts` (`uuidPrimaryKey`: `uuid` with `defaultRandom()`; `timestamps`), barrel `src/db/schema/index.ts`. drizzle-kit reads the glob `./src/db/schema/*.ts`; `db.query` and the type export read the barrel |
| **Connection**             | `db` in `src/db/connection.ts`; `DatabaseConnection` (`src/db/types.ts`) is what routes and services take, so tests can pass `tx`. Every query goes out prepared (`withPreparedStatements`); `DATABASE_PREPARE=false` turns it off                                        |
| **ServiceResult**          | `src/utils/service-result.ts`: `ok`, `fail(code, message, details?)`, `mapError`. `VALIDATION` also covers state-guard refusals                                                                                                                                           |
| **The API's type export**  | `@app/api/types` → `src/db/schema/index.ts`                                                                                                                                                                                                                               |
| **The client**             | `apps/web`, through Eden Treaty. Its typecheck is `cd apps/web && npx tsc -b --noEmit --force`: `bun web:typecheck` can pass on a stale build cache                                                                                                                       |
| **Runtime helpers**        | `@jswob/drizzle-utils`: `caseBy`, `dbNow`, `isUniqueViolation`, `withPreparedStatements`, `createQueryCounter`                                                                                                                                                            |
| **Shared TypeBox schemas** | Live beside their owner and are imported from there (the notes module's `title` lives in `src/modules/notes/index.ts`)                                                                                                                                                    |

## Tests

- **The test context module** is `src/test/test-context.ts`, imported relatively
  (`'../../test/test-context'` from a module folder). It exports `test`, `expect`, `describe` and
  the before/after hooks, with the `tx`, `factory` and `db` fixtures.
- **The fixtures package** is `@jswob/drizzle-fixtures`.
  One file per table, `src/test/fixtures/<name>.fixture.ts`, composed in
  `src/test/fixtures/index.ts`.
- **`countQueries`** is in `src/test/db.ts`, built on `createQueryCounter`.
- **The test database**: `vitest.setup.ts` loads the root `.env` and sets `DB_TARGET=test`, so the
  app's own connection reaches it too (`TEST_DB_PORT` in `.env.example`). The root
  `bun run test` sets it up before the run.

## Commands

All from the monorepo root.

| The skill says   | Here                                                                                                                                                     |
| ---------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| API typecheck    | `bun api:typecheck`                                                                                                                                      |
| Client typecheck | `cd apps/web && npx tsc -b --noEmit --force` (`bun web:typecheck` can pass on a stale build cache)                                                       |
| Full typecheck   | `bun typecheck`                                                                                                                                          |
| Format           | `bun lint:fix` for sources, `bunx prettier --write <files>` for docs. Never repo-wide `bun format`: its prettier pass corrupts `.claude/skills` markdown |
| Format check     | `bun format:check`                                                                                                                                       |
| Tests, run once  | `bun run test` (every Vitest project in `apps/*` and `packages/*`); the API alone: `bun api:test`                                                        |
| Tests, watch     | `bun test:watch`                                                                                                                                         |
| Lint             | `bun lint`: ESLint at `--max-warnings 0`, `bun typecheck`, `bun toolkit:check`                                                                           |
| Lint fix         | `bun lint:fix`                                                                                                                                           |

## Database

| The skill says     | Here                                                                                                                                                                                                    |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Safe push          | `bun db:push:safe`. Never plain `bun db:push`                                                                                                                                                           |
| Migrations         | **Never** `bun db:generate` or `bun db:generate:custom`: the user creates migration files                                                                                                               |
| Baseline snapshot  | `bun db:base`, before any major experiment                                                                                                                                                              |
| Undo / reset       | `bun db:undo` / `bun db:reset`                                                                                                                                                                          |
| Snapshot / restore | `bun db:snapshot [name]` / `bun db:restore [name]`; `bun db:snapshot:list`                                                                                                                              |
| Clean              | `bun db:clean --force`                                                                                                                                                                                  |
| Snapshot tool      | `@jswob/drizzle-snapshots`, bound in `drizzle-snapshots.config.ts`. Its `schema` option is left at the default `['src/db/schema/']`; add it, with the new folder, only when the schema imports from one |

**Seeding**: `bun db:seed` empties the seeded tables and loads the newest snapshot of real data from
`src/db/seed-data/` (not committed). With no snapshot it seeds the fallback in
`src/db/seed-tables.ts` (`seedFallback`: two sample notes). `src/db/seed-core.ts` is the generic
loader, `src/db/seed-tables.ts` this app's part: a new table that holds real data goes into
`SEED_TABLES` there.

## This repo's own example

**Notes** (`src/modules/notes/`) is the example resource and the shape to copy for a new one: the
table `src/db/schema/notes.ts`, its fixture `src/test/fixtures/note.fixture.ts`, the module's
`index.ts` (routes, TypeBox bodies, `mapError` on every failure) and `service.ts` (each function
returns a `ServiceResult`), and its tests `notes.test.ts`. `docs/removing-notes.md` lists every
file it takes, so an app removes it in one commit. QUERIES.md's larger stand-ins (`getPostsWithDetails`,
`savePosts`, one transaction per save) have no counterpart here yet: write the app's first one
after them.

## Standing rules that bite here

- Test files are collocated, `[name].test.ts`; never `__tests__`.
- Every `apps/api` script that runs `bun` or `drizzle-kit` carries `--env-file=../../.env`.
- ESLint disables are inline, one line, with `-- Reason: …`.
- Idle means silent: no timer may query the database on a tick, and `/health` never touches it.
  Background work is woken by what causes it.
