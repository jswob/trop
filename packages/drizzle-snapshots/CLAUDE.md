# Drizzle Snapshots

The `drizzle-snapshots` CLI: docker volume snapshots of the dev database, a push that can be
undone, and the test and e2e database setup, for a Drizzle app whose Postgres runs in compose. A
dev tool; nothing here ships in the API's image, and nothing here knows the app.

## How an app plugs in

- **`drizzle-snapshots.config.ts`** beside the app's `drizzle.config.ts` (`defineConfig`): its
  `dbTarget` (from `createDbTargets` in `@jswob/drizzle-utils`), its `migrate` and `seed` scripts,
  its `rawConstraints`, and optionally its `schema` folders and `migrations` folder.
- **The app's scripts** call `bun --env-file=<root .env> drizzle-snapshots <command>`, so the CLI
  runs from the app's folder with the environment loaded. Every path in the config is relative to
  that folder.
- **Conventions it relies on:** `COMPOSE_PROJECT_NAME` set; compose containers named
  `<project>-postgres` (or `<project>-$POSTGRES_CONTAINER_NAME`), `<project>-postgres-test` and
  `<project>-postgres-e2e`; data volumes `<project>_postgres_data` and `<project>_postgres_e2e_data`;
  `drizzle.config.test.ts` and `drizzle.config.e2e.ts` beside `drizzle.config.ts`. Its hints name
  the root scripts (`bun db:undo`, `bun db:base`…).

## Commands

`snapshot [name]` (`base` is the protected baseline), `restore [name]`, `undo`, `list`,
`clean [--force]`, `push`, `constraints`, `test-setup`, `e2e-setup`, `e2e-restore`,
`e2e-teardown`, `schema-check`. `drizzle-snapshots` alone prints them.

## Rules

- **Volumes are the project's own.** Every volume name starts with `COMPOSE_PROJECT_NAME`; names
  from before that (`postgres_snapshot_*`, `e2e_snapshot_base`) are listed and never pruned or
  removed (`volumes.ts`).
- **The test and e2e databases are reached by name**, never through `DATABASE_URL`: a seed
  empties every table of whatever it is pointed at.
- **drizzle-kit runs under Bun** (`DRIZZLE_KIT` in `drizzle-kit.ts`, and the app's own drizzle-kit
  scripts): Node refuses to strip types under `node_modules`, so a drizzle config that imports an
  installed TypeScript package fails to load under it.
- **A command throws; the entry point prints and exits 1** (`runScript`), so `withStopped` always
  starts its container again.
- Tests: `bun drizzle-snapshots:test`. The commands themselves are checked against a sandbox
  compose project (another `COMPOSE_PROJECT_NAME`, its own ports), never the real volumes.
